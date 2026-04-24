mod dispatch;

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tracing::debug;

use crate::auth::{Authenticator, from_config};
use crate::collection_domain::CollectionDomain;
use crate::config::ApiConfig;
use crate::config_domain::ConfigDomain;
use crate::errors::{ApiError, RpcErrorCode};
use crate::idempotency::{
    IdempotencyCheck, IdempotencyStore, InMemoryIdempotencyStore, StoredResponse,
};
use crate::loop_domain::LoopDomain;
use crate::planning_domain::PlanningDomain;
use crate::preset_domain::PresetDomain;
use crate::protocol::{
    API_VERSION, KNOWN_METHODS, RpcRequestEnvelope, STREAM_TOPICS, error_envelope, is_known_method,
    is_mutating_method, parse_json_value, parse_request, request_context, success_envelope,
    validate_request_schema,
};
use crate::stream_domain::StreamDomain;
use crate::task_domain::TaskDomain;
use crate::workspace_domain::WorkspaceDomain;

/// Per-workspace container for all domains.
#[derive(Clone)]
pub struct WorkspaceRuntime {
    pub tasks: Arc<Mutex<TaskDomain>>,
    pub loops: Arc<Mutex<LoopDomain>>,
    pub planning: Arc<Mutex<PlanningDomain>>,
    pub collections: Arc<Mutex<CollectionDomain>>,
    pub config_domain: ConfigDomain,
    pub preset_domain: PresetDomain,
}

impl WorkspaceRuntime {
    pub fn new(config: &ApiConfig, workspace_root: &std::path::Path) -> Self {
        let tasks = Arc::new(Mutex::new(TaskDomain::new(workspace_root)));
        let loops = Arc::new(Mutex::new(LoopDomain::new(
            workspace_root,
            config.loop_process_interval_ms,
            config.ulf_command.clone(),
        )));
        let planning = Arc::new(Mutex::new(PlanningDomain::new(workspace_root)));
        let collections = Arc::new(Mutex::new(CollectionDomain::new(workspace_root)));
        let config_domain = ConfigDomain::new(workspace_root);
        let preset_domain = PresetDomain::new(workspace_root);

        Self {
            tasks,
            loops,
            planning,
            collections,
            config_domain,
            preset_domain,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IdOnlyParams {
    pub(crate) id: String,
}

#[derive(Clone)]
pub struct RpcRuntime {
    pub(crate) config: ApiConfig,
    auth: Arc<dyn Authenticator>,
    idempotency: Arc<dyn IdempotencyStore>,
    /// Global stream domain (events are tagged with workspaceId by callers).
    streams: StreamDomain,
    /// Workspace registry (global daemon state).
    workspaces: Arc<Mutex<WorkspaceDomain>>,
    /// Per-workspace runtimes, created on demand.
    workspace_runtimes: Arc<Mutex<std::collections::HashMap<String, WorkspaceRuntime>>>,
}

enum ExecutionOutcome {
    Fresh(Value),
    Replay(StoredResponse),
}

impl RpcRuntime {
    pub fn new(config: ApiConfig) -> anyhow::Result<Self> {
        config.validate()?;

        let auth = from_config(&config)?;
        let idempotency = Arc::new(InMemoryIdempotencyStore::new(Duration::from_secs(
            config.idempotency_ttl_secs,
        )));

        Ok(Self::with_components(config, auth, idempotency))
    }

    pub fn with_components(
        config: ApiConfig,
        auth: Arc<dyn Authenticator>,
        idempotency: Arc<dyn IdempotencyStore>,
    ) -> Self {
        let streams = StreamDomain::new();
        let workspaces = Arc::new(Mutex::new(WorkspaceDomain::new(&config.daemon_state_dir)));
        let workspace_runtimes = Arc::new(Mutex::new(std::collections::HashMap::new()));

        Self {
            config,
            auth,
            idempotency,
            streams,
            workspaces,
            workspace_runtimes,
        }
    }

    /// Get or create a `WorkspaceRuntime` for the given workspace ID.
    ///
    /// Falls back to the default workspace (from `config.workspace_root`) when `workspace_id` is None.
    pub fn workspace_runtime(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<WorkspaceRuntime, ApiError> {
        let id = match workspace_id {
            Some(id) => id.to_string(),
            None => {
                // Backwards compatibility: use the default workspace root.
                let default_path = &self.config.workspace_root;
                let default_id = default_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("default")
                    .to_string();

                // Ensure the default workspace is registered
                let mut registry = self.workspaces.lock().map_err(|_| {
                    ApiError::internal("workspace registry lock poisoned")
                })?;

                if registry.get(&default_id).is_err() {
                    let _ = registry.create(crate::workspace_domain::WorkspaceCreateParams {
                        id: default_id.clone(),
                        name: default_id.clone(),
                        path: Some(default_path.display().to_string()),
                        setup_prompt: None,
                    });
                }
                drop(registry);
                default_id
            }
        };

        let mut runtimes = self.workspace_runtimes.lock().map_err(|_| {
            ApiError::internal("workspace runtimes lock poisoned")
        })?;

        if let Some(runtime) = runtimes.get(&id) {
            return Ok(runtime.clone());
        }

        // Resolve workspace path from registry
        let registry = self.workspaces.lock().map_err(|_| {
            ApiError::internal("workspace registry lock poisoned")
        })?;
        let workspace = registry.get(&id)?;
        let path = workspace.path.clone();
        drop(registry);

        let runtime = WorkspaceRuntime::new(&self.config, &path);
        runtimes.insert(id, runtime.clone());
        Ok(runtime)
    }

    pub fn health_payload(&self) -> Value {
        json!({
            "status": "ok",
            "timestamp": crate::loop_support::now_ts()
        })
    }

    pub fn capabilities_payload(&self) -> Value {
        json!({
            "methods": KNOWN_METHODS,
            "streamTopics": STREAM_TOPICS,
            "auth": {
                "mode": self.auth.mode().as_contract_mode(),
                "supportedModes": ["trusted_local", "token"]
            },
            "idempotency": {
                "requiredForMutations": true,
                "retentionSeconds": self.config.idempotency_ttl_secs
            }
        })
    }

    pub fn invoke_method(
        &self,
        request_id: impl Into<String>,
        method: &str,
        params: Value,
        principal: &str,
        idempotency_key: Option<String>,
    ) -> Result<Value, ApiError> {
        let request_id = request_id.into();
        let mut raw = json!({
            "apiVersion": API_VERSION,
            "id": request_id,
            "method": method,
            "params": params,
        });

        if let Some(idempotency_key) = idempotency_key {
            raw["meta"] = json!({
                "idempotencyKey": idempotency_key,
            });
        }

        let request = self.parse_and_validate_request_value(raw)?;
        match self.execute_request(&request, principal)? {
            ExecutionOutcome::Fresh(result) => Ok(result),
            ExecutionOutcome::Replay(response) => self.replay_stored_response(&request, response),
        }
    }

    pub fn handle_http_request(&self, body: &[u8], headers: &HeaderMap) -> (StatusCode, Value) {
        let request = match self.parse_and_validate_request(body) {
            Ok(request) => request,
            Err(error) => {
                let status = error.status;
                let envelope = error_envelope(&error, &self.config.served_by);
                return (status, envelope);
            }
        };

        let principal =
            match self.auth.authorize(&request, headers).map_err(|error| {
                error.with_context(request.id.clone(), Some(request.method.clone()))
            }) {
                Ok(p) => p,
                Err(error) => {
                    let status = error.status;
                    let envelope = error_envelope(&error, &self.config.served_by);
                    return (status, envelope);
                }
            };

        let (status, envelope) = match self.execute_request(&request, &principal) {
            Ok(ExecutionOutcome::Fresh(result)) => (
                StatusCode::OK,
                success_envelope(&request, result, &self.config.served_by),
            ),
            Ok(ExecutionOutcome::Replay(response)) => (
                StatusCode::from_u16(response.status).unwrap_or(StatusCode::OK),
                response.envelope,
            ),
            Err(error) => {
                let status = error.status;
                let envelope = error_envelope(&error, &self.config.served_by);
                (status, envelope)
            }
        };

        (status, envelope)
    }

    pub fn authenticate_websocket(&self, headers: &HeaderMap) -> Result<String, ApiError> {
        let dummy_request = crate::protocol::RpcRequestEnvelope {
            api_version: "v1".to_string(),
            id: "ws-upgrade".to_string(),
            method: "stream.subscribe".to_string(),
            params: serde_json::Value::Object(serde_json::Map::new()),
            meta: None,
        };

        self.auth
            .authorize(&dummy_request, headers)
            .map_err(|error| error.with_context("ws-upgrade", Some("stream.subscribe".to_string())))
    }

    pub(crate) fn stream_domain(&self) -> StreamDomain {
        self.streams.clone()
    }

    pub(crate) fn workspace_domain_mut(&self) -> Result<MutexGuard<'_, WorkspaceDomain>, ApiError> {
        self.workspaces
            .lock()
            .map_err(|_| ApiError::internal("workspace domain lock poisoned"))
    }

    /// Extract `workspaceId` from request params, if present.
    pub(crate) fn workspace_id_from_params(&self, request: &RpcRequestEnvelope) -> Option<String> {
        request.params.get("workspaceId").and_then(|v| v.as_str()).map(String::from)
    }

    pub(crate) fn parse_params<T>(&self, request: &RpcRequestEnvelope) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(request.params.clone()).map_err(|error| {
            ApiError::invalid_params(format!(
                "invalid params for method '{}': {error}",
                request.method
            ))
        })
    }

    fn parse_and_validate_request(&self, body: &[u8]) -> Result<RpcRequestEnvelope, ApiError> {
        let raw = parse_json_value(body)?;
        self.parse_and_validate_request_value(raw)
    }

    fn parse_and_validate_request_value(&self, raw: Value) -> Result<RpcRequestEnvelope, ApiError> {
        let (request_id, method) = request_context(&raw);

        if !raw.is_object() {
            return Err(
                ApiError::invalid_request("request body must be a JSON object")
                    .with_context(request_id, method),
            );
        }

        let method = method.ok_or_else(|| {
            ApiError::invalid_request("missing required field 'method'")
                .with_context(request_id.clone(), None)
        })?;

        if !is_known_method(&method) {
            return Err(
                ApiError::method_not_found(method.clone()).with_context(request_id, Some(method))
            );
        }

        if let Err(errors) = validate_request_schema(&raw) {
            return Err(
                ApiError::invalid_params("request does not match rpc-v1 schema")
                    .with_context(request_id, Some(method))
                    .with_details(json!({ "errors": errors })),
            );
        }

        let request = parse_request(&raw)
            .map_err(|error| error.with_context(request_id.clone(), Some(method.clone())))?;

        if request.api_version != API_VERSION {
            return Err(ApiError::invalid_request(format!(
                "unsupported apiVersion '{}'; expected '{API_VERSION}'",
                request.api_version
            ))
            .with_context(request.id, Some(request.method)));
        }

        Ok(request)
    }

    fn execute_request(
        &self,
        request: &RpcRequestEnvelope,
        principal: &str,
    ) -> Result<ExecutionOutcome, ApiError> {
        let mut idempotency_context: Option<String> = None;
        if is_mutating_method(&request.method) {
            let key = match request
                .meta
                .as_ref()
                .and_then(|meta| meta.idempotency_key.as_deref())
            {
                Some(key) => key,
                None => {
                    return Err(ApiError::invalid_params(
                        "mutating methods require meta.idempotencyKey",
                    )
                    .with_context(request.id.clone(), Some(request.method.clone())));
                }
            };

            match self
                .idempotency
                .check(&request.method, key, &request.params)
            {
                IdempotencyCheck::Replay(response) => {
                    debug!(
                        method = %request.method,
                        request_id = %request.id,
                        "idempotency replay"
                    );
                    return Ok(ExecutionOutcome::Replay(response));
                }
                IdempotencyCheck::Conflict => {
                    return Err(ApiError::idempotency_conflict(
                        "idempotency key was already used with different parameters",
                    )
                    .with_context(request.id.clone(), Some(request.method.clone()))
                    .with_details(json!({
                        "method": request.method.clone(),
                        "idempotencyKey": key
                    })));
                }
                IdempotencyCheck::New => {
                    idempotency_context = Some(key.to_string());
                }
            }
        }

        let result = self
            .dispatch(request, principal)
            .map_err(|error| error.with_context(request.id.clone(), Some(request.method.clone())));

        if let Some(key) = idempotency_context {
            let (status, envelope) = match &result {
                Ok(value) => (
                    StatusCode::OK.as_u16(),
                    success_envelope(request, value.clone(), &self.config.served_by),
                ),
                Err(error) => (
                    error.status.as_u16(),
                    error_envelope(error, &self.config.served_by),
                ),
            };
            self.idempotency.store(
                &request.method,
                &key,
                &request.params,
                &StoredResponse { status, envelope },
            );
        }

        result.map(ExecutionOutcome::Fresh)
    }

    fn replay_stored_response(
        &self,
        request: &RpcRequestEnvelope,
        response: StoredResponse,
    ) -> Result<Value, ApiError> {
        if let Some(result) = response.envelope.get("result") {
            return Ok(result.clone());
        }

        let error_body = response
            .envelope
            .get("error")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                ApiError::internal("stored idempotency response was missing an error payload")
                    .with_context(request.id.clone(), Some(request.method.clone()))
            })?;

        let code = error_body
            .get("code")
            .and_then(Value::as_str)
            .and_then(RpcErrorCode::from_contract)
            .unwrap_or(RpcErrorCode::Internal);
        let message = error_body
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("stored idempotency replay failed");
        let retryable = error_body
            .get("retryable")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let mut error = ApiError::new(code, message)
            .with_context(request.id.clone(), Some(request.method.clone()));
        error.retryable = retryable;
        error.details = error_body.get("details").cloned();
        error.status = StatusCode::from_u16(response.status).unwrap_or(error.status);
        Err(error)
    }
}
