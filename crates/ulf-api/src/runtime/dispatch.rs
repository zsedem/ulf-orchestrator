use serde_json::{Value, json};
use tracing::warn;

use super::{IdOnlyParams, RpcRuntime, WorkspaceRuntime};
use crate::collection_domain::{
    CollectionCreateParams, CollectionImportParams, CollectionUpdateParams,
};
use crate::config_domain::ConfigUpdateParams;
use crate::errors::ApiError;
use crate::loop_domain::{
    LoopListParams, LoopRetryParams, LoopStopMergeParams, LoopTriggerMergeTaskParams,
};
use crate::planning_domain::{
    PlanningGetArtifactParams, PlanningRespondParams, PlanningStartParams,
};
use crate::protocol::{API_VERSION, RpcRequestEnvelope};
use crate::stream_domain::{StreamAckParams, StreamSubscribeParams, StreamUnsubscribeParams};
use crate::task_domain::{TaskCreateParams, TaskListParams, TaskUpdateInput};
use crate::workspace_domain::{WorkspaceCreateParams, WorkspaceDeleteParams};

impl RpcRuntime {
    pub(super) fn dispatch(
        &self,
        request: &RpcRequestEnvelope,
        principal: &str,
    ) -> Result<Value, ApiError> {
        let result = match request.method.as_str() {
            "system.health" => Ok(self.health_payload()),
            "system.version" => Ok(json!({
                "apiVersion": API_VERSION,
                "serverVersion": env!("CARGO_PKG_VERSION")
            })),
            "system.capabilities" => Ok(self.capabilities_payload()),
            method if method.starts_with("task.") => self.dispatch_task(request),
            method if method.starts_with("loop.") => self.dispatch_loop(request),
            method if method.starts_with("planning.") => self.dispatch_planning(request),
            method if method.starts_with("config.") => self.dispatch_config(request),
            method if method.starts_with("preset.") => self.dispatch_preset(request),
            method if method.starts_with("collection.") => self.dispatch_collection(request),
            method if method.starts_with("workspace.") => self.dispatch_workspace(request),
            method if method.starts_with("stream.") => self.dispatch_stream(request, principal),
            "_internal.publish" => self.dispatch_internal_publish(request),
            _ => {
                warn!(
                    method = %request.method,
                    "recognized method is not implemented in rpc runtime"
                );
                Err(ApiError::service_unavailable(format!(
                    "method '{}' is recognized but not implemented in rpc runtime",
                    request.method
                )))
            }
        };

        if let Ok(payload) = &result
            && !request.method.starts_with("stream.")
        {
            self.stream_domain()
                .publish_rpc_side_effect(&request.method, &request.params, payload);
        }

        result
    }

    fn with_workspace<F, T>(&self, request: &RpcRequestEnvelope, f: F) -> Result<T, ApiError>
    where
        F: FnOnce(&WorkspaceRuntime) -> Result<T, ApiError>,
    {
        let workspace_id = self.workspace_id_from_params(request);
        let runtime = self.workspace_runtime(workspace_id.as_deref())?;
        f(&runtime)
    }

    fn dispatch_task(&self, request: &RpcRequestEnvelope) -> Result<Value, ApiError> {
        self.with_workspace(request, |wr| {
            let mut tasks = wr.tasks.lock().map_err(|_| ApiError::internal("task domain lock poisoned"))?;
            match request.method.as_str() {
                "task.list" => {
                    let params: TaskListParams = self.parse_params(request)?;
                    Ok(json!({ "tasks": tasks.list(params) }))
                }
                "task.get" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let task = tasks.get(&params.id)?;
                    Ok(json!({ "task": task }))
                }
                "task.ready" => {
                    Ok(json!({ "tasks": tasks.ready() }))
                }
                "task.create" => {
                    let params: TaskCreateParams = self.parse_params(request)?;
                    let task = tasks.create(params)?;
                    Ok(json!({ "task": task }))
                }
                "task.update" => {
                    let input = parse_task_update_input(request)?;
                    let task = tasks.update(input)?;
                    Ok(json!({ "task": task }))
                }
                "task.close" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let task = tasks.close(&params.id)?;
                    Ok(json!({ "task": task }))
                }
                "task.archive" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let task = tasks.archive(&params.id)?;
                    Ok(json!({ "task": task }))
                }
                "task.unarchive" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let task = tasks.unarchive(&params.id)?;
                    Ok(json!({ "task": task }))
                }
                "task.delete" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    tasks.delete(&params.id)?;
                    Ok(json!({ "success": true }))
                }
                "task.clear" => {
                    tasks.clear()?;
                    Ok(json!({ "success": true }))
                }
                "task.run" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let result = tasks.run(&params.id)?;
                    Ok(json!(result))
                }
                "task.run_all" => {
                    let result = tasks.run_all();
                    Ok(json!(result))
                }
                "task.retry" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let result = tasks.retry(&params.id)?;
                    Ok(json!(result))
                }
                "task.cancel" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let task = tasks.cancel(&params.id)?;
                    Ok(json!({ "task": task }))
                }
                "task.status" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let status = tasks.status(&params.id);
                    Ok(json!(status))
                }
                _ => Err(ApiError::service_unavailable(format!(
                    "method '{}' is recognized but not implemented",
                    request.method
                ))),
            }
        })
    }

    fn dispatch_loop(&self, request: &RpcRequestEnvelope) -> Result<Value, ApiError> {
        self.with_workspace(request, |wr| {
            let mut loops = wr.loops.lock().map_err(|_| ApiError::internal("loop domain lock poisoned"))?;
            match request.method.as_str() {
                "loop.list" => {
                    let params: LoopListParams = self.parse_params(request)?;
                    let loops_list = loops.list(params)?;
                    Ok(json!({ "loops": loops_list }))
                }
                "loop.status" => {
                    let status = loops.status();
                    Ok(json!(status))
                }
                "loop.process" => {
                    loops.process()?;
                    Ok(json!({ "success": true }))
                }
                "loop.prune" => {
                    loops.prune()?;
                    Ok(json!({ "success": true }))
                }
                "loop.retry" => {
                    let params: LoopRetryParams = self.parse_params(request)?;
                    loops.retry(params)?;
                    Ok(json!({ "success": true }))
                }
                "loop.discard" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    loops.discard(&params.id)?;
                    Ok(json!({ "success": true }))
                }
                "loop.stop" => {
                    let params: LoopStopMergeParams = self.parse_params(request)?;
                    loops.stop(params)?;
                    Ok(json!({ "success": true }))
                }
                "loop.merge" => {
                    let params: LoopStopMergeParams = self.parse_params(request)?;
                    loops.merge(params)?;
                    Ok(json!({ "success": true }))
                }
                "loop.merge_button_state" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let state = loops.merge_button_state(&params.id)?;
                    Ok(json!(state))
                }
                "loop.trigger_merge_task" => {
                    let params: LoopTriggerMergeTaskParams = self.parse_params(request)?;
                    let mut tasks = wr.tasks.lock().map_err(|_| ApiError::internal("task domain lock poisoned"))?;
                    let result = loops.trigger_merge_task(params, &mut tasks)?;
                    Ok(json!(result))
                }
                _ => Err(ApiError::service_unavailable(format!(
                    "method '{}' is recognized but not implemented",
                    request.method
                ))),
            }
        })
    }

    fn dispatch_planning(&self, request: &RpcRequestEnvelope) -> Result<Value, ApiError> {
        self.with_workspace(request, |wr| {
            let mut planning = wr.planning.lock().map_err(|_| ApiError::internal("planning domain lock poisoned"))?;
            match request.method.as_str() {
                "planning.list" => {
                    let sessions = planning.list()?;
                    Ok(json!({ "sessions": sessions }))
                }
                "planning.get" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let session = planning.get(&params.id)?;
                    Ok(json!({ "session": session }))
                }
                "planning.start" => {
                    let params: PlanningStartParams = self.parse_params(request)?;
                    let session = planning.start(params)?;
                    Ok(json!({ "session": session }))
                }
                "planning.respond" => {
                    let params: PlanningRespondParams = self.parse_params(request)?;
                    planning.respond(params)?;
                    Ok(json!({ "success": true }))
                }
                "planning.resume" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    planning.resume(&params.id)?;
                    Ok(json!({ "success": true }))
                }
                "planning.delete" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    planning.delete(&params.id)?;
                    Ok(json!({ "success": true }))
                }
                "planning.get_artifact" => {
                    let params: PlanningGetArtifactParams = self.parse_params(request)?;
                    let artifact = planning.get_artifact(params)?;
                    Ok(json!(artifact))
                }
                _ => Err(ApiError::service_unavailable(format!(
                    "method '{}' is recognized but not implemented",
                    request.method
                ))),
            }
        })
    }

    fn dispatch_config(&self, request: &RpcRequestEnvelope) -> Result<Value, ApiError> {
        self.with_workspace(request, |wr| {
            match request.method.as_str() {
                "config.get" => {
                    let config = wr.config_domain.get()?;
                    Ok(json!(config))
                }
                "config.update" => {
                    let params: ConfigUpdateParams = self.parse_params(request)?;
                    let result = wr.config_domain.update(params)?;
                    Ok(json!(result))
                }
                _ => Err(ApiError::service_unavailable(format!(
                    "method '{}' is recognized but not implemented",
                    request.method
                ))),
            }
        })
    }

    fn dispatch_preset(&self, request: &RpcRequestEnvelope) -> Result<Value, ApiError> {
        self.with_workspace(request, |wr| {
            match request.method.as_str() {
                "preset.list" => {
                    let collections = wr.collections.lock().map_err(|_| ApiError::internal("collection domain lock poisoned"))?.list();
                    let presets = wr.preset_domain.list(&collections);
                    Ok(json!({ "presets": presets }))
                }
                _ => Err(ApiError::service_unavailable(format!(
                    "method '{}' is recognized but not implemented",
                    request.method
                ))),
            }
        })
    }

    fn dispatch_collection(&self, request: &RpcRequestEnvelope) -> Result<Value, ApiError> {
        self.with_workspace(request, |wr| {
            let mut collections = wr.collections.lock().map_err(|_| ApiError::internal("collection domain lock poisoned"))?;
            match request.method.as_str() {
                "collection.list" => {
                    Ok(json!({ "collections": collections.list() }))
                }
                "collection.get" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let collection = collections.get(&params.id)?;
                    Ok(json!({ "collection": collection }))
                }
                "collection.create" => {
                    let params: CollectionCreateParams = self.parse_params(request)?;
                    let collection = collections.create(params)?;
                    Ok(json!({ "collection": collection }))
                }
                "collection.update" => {
                    let params: CollectionUpdateParams = self.parse_params(request)?;
                    let collection = collections.update(params)?;
                    Ok(json!({ "collection": collection }))
                }
                "collection.delete" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    collections.delete(&params.id)?;
                    Ok(json!({ "success": true }))
                }
                "collection.import" => {
                    let params: CollectionImportParams = self.parse_params(request)?;
                    let collection = collections.import(params)?;
                    Ok(json!({ "collection": collection }))
                }
                "collection.export" => {
                    let params: IdOnlyParams = self.parse_params(request)?;
                    let yaml = collections.export(&params.id)?;
                    Ok(json!({ "yaml": yaml }))
                }
                _ => Err(ApiError::service_unavailable(format!(
                    "method '{}' is recognized but not implemented",
                    request.method
                ))),
            }
        })
    }

    fn dispatch_workspace(&self, request: &RpcRequestEnvelope) -> Result<Value, ApiError> {
        let mut workspaces = self.workspace_domain_mut()?;
        match request.method.as_str() {
            "workspace.create" => {
                let params: WorkspaceCreateParams = self.parse_params(request)?;
                let workspace = workspaces.create(params)?;
                Ok(json!({ "workspace": workspace }))
            }
            "workspace.list" => {
                let workspaces_list = workspaces.list();
                Ok(json!({ "workspaces": workspaces_list }))
            }
            "workspace.get" => {
                let params: IdOnlyParams = self.parse_params(request)?;
                let workspace = workspaces.get(&params.id)?;
                Ok(json!({ "workspace": workspace }))
            }
            "workspace.delete" => {
                let params: WorkspaceDeleteParams = self.parse_params(request)?;
                workspaces.delete(params)?;
                Ok(json!({ "success": true }))
            }
            "workspace.update_status" => {
                let object = request.params.as_object().ok_or_else(|| {
                    ApiError::invalid_params("workspace.update_status params must be an object")
                })?;
                let id = object
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| ApiError::invalid_params("workspace.update_status requires 'id'"))?;
                let status_str = object
                    .get("status")
                    .and_then(Value::as_str)
                    .ok_or_else(|| ApiError::invalid_params("workspace.update_status requires 'status'"))?;
                let status = match status_str {
                    "creating" => ulf_core::WorkspaceStatus::Creating,
                    "ready" => ulf_core::WorkspaceStatus::Ready,
                    "error" => ulf_core::WorkspaceStatus::Error,
                    "archived" => ulf_core::WorkspaceStatus::Archived,
                    _ => {
                        return Err(ApiError::invalid_params(format!(
                            "invalid workspace status '{}'",
                            status_str
                        )))
                    }
                };
                let error_message = object.get("errorMessage").and_then(Value::as_str).map(String::from);
                let workspace = workspaces.update_status(id, status, error_message)?;
                Ok(json!({ "workspace": workspace }))
            }
            _ => Err(ApiError::service_unavailable(format!(
                "method '{}' is recognized but not implemented",
                request.method
            ))),
        }
    }

    fn dispatch_stream(
        &self,
        request: &RpcRequestEnvelope,
        principal: &str,
    ) -> Result<Value, ApiError> {
        match request.method.as_str() {
            "stream.subscribe" => {
                let params: StreamSubscribeParams = self.parse_params(request)?;
                let result = self.stream_domain().subscribe(params, principal)?;
                Ok(json!(result))
            }
            "stream.unsubscribe" => {
                let params: StreamUnsubscribeParams = self.parse_params(request)?;
                self.stream_domain().unsubscribe(params)?;
                Ok(json!({ "success": true }))
            }
            "stream.ack" => {
                let params: StreamAckParams = self.parse_params(request)?;
                self.stream_domain().ack(params)?;
                Ok(json!({ "success": true }))
            }
            _ => Err(ApiError::service_unavailable(format!(
                "method '{}' is recognized but not implemented",
                request.method
            ))),
        }
    }
}

use serde::Deserialize as InternalDeserialize;

#[derive(Debug, Clone, InternalDeserialize)]
#[serde(rename_all = "camelCase")]
struct InternalPublishParams {
    topic: String,
    resource_type: String,
    resource_id: String,
    payload: Value,
}

impl RpcRuntime {
    /// Internal-only method for the orchestration loop to inject events
    /// into the stream domain. Not part of the public RPC contract.
    fn dispatch_internal_publish(&self, request: &RpcRequestEnvelope) -> Result<Value, ApiError> {
        let params: InternalPublishParams = self.parse_params(request)?;
        self.stream_domain().publish(
            &params.topic,
            &params.resource_type,
            &params.resource_id,
            params.payload,
        );
        Ok(json!({ "success": true }))
    }
}

fn parse_task_update_input(request: &RpcRequestEnvelope) -> Result<TaskUpdateInput, ApiError> {
    let object = request.params.as_object().ok_or_else(|| {
        ApiError::invalid_params("task.update params must be an object")
            .with_details(json!({ "method": request.method }))
    })?;

    let id = object
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| ApiError::invalid_params("task.update requires non-empty 'id'"))?
        .to_string();

    let title = object
        .get("title")
        .and_then(Value::as_str)
        .map(std::string::ToString::to_string);

    let status = object
        .get("status")
        .and_then(Value::as_str)
        .map(std::string::ToString::to_string);

    let priority = object
        .get("priority")
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok());

    let blocked_by = if object.contains_key("blockedBy") {
        let value = object
            .get("blockedBy")
            .expect("contains_key check guarantees blockedBy exists");
        if value.is_null() {
            Some(None)
        } else {
            let blocked_by = value.as_str().ok_or_else(|| {
                ApiError::invalid_params("task.update blockedBy must be a string or null")
            })?;
            Some(Some(blocked_by.to_string()))
        }
    } else {
        None
    };

    Ok(TaskUpdateInput {
        id,
        title,
        status,
        priority,
        blocked_by,
    })
}
