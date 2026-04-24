use std::path::Path;

use anyhow::Result;
use reqwest::Client;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tempfile::TempDir;

use ulf_api::{ApiConfig, RpcRuntime, serve_with_listener};

struct TestServer {
    base_url: String,
    shutdown: Option<oneshot::Sender<()>>,
    join: tokio::task::JoinHandle<anyhow::Result<()>>,
    workspace: TempDir,
}

impl TestServer {
    async fn start(mut config: ApiConfig) -> Self {
        let workspace = tempfile::tempdir().expect("workspace tempdir should be created");
        config.workspace_root = workspace.path().to_path_buf();
        config.daemon_state_dir = workspace.path().join(".ulf").join("daemon");

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let local_addr = listener
            .local_addr()
            .expect("listener local addr should exist");
        let runtime = RpcRuntime::new(config).expect("runtime should initialize");
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let join = tokio::spawn(async move {
            serve_with_listener(listener, runtime, async move {
                let _ = shutdown_rx.await;
            })
            .await
        });

        Self {
            base_url: format!("http://{local_addr}"),
            shutdown: Some(shutdown_tx),
            join,
            workspace,
        }
    }

    fn workspace_path(&self) -> &Path {
        self.workspace.path()
    }

    async fn stop(mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        let result = self.join.await.expect("server task should join");
        result.expect("server should shutdown cleanly");
    }
}

async fn post_rpc(client: &Client, server: &TestServer, body: &Value) -> Result<(u16, Value)> {
    let response = client
        .post(format!("{}/rpc/v1", server.base_url))
        .header("content-type", "application/json")
        .json(body)
        .send()
        .await?;

    let status = response.status().as_u16();
    let payload = response.json::<Value>().await?;
    Ok((status, payload))
}

fn rpc_request(id: &str, method: &str, params: Value, idempotency_key: Option<&str>) -> Value {
    let mut request = json!({
        "apiVersion": "v1",
        "id": id,
        "method": method,
        "params": params,
    });

    if let Some(idempotency_key) = idempotency_key {
        request["meta"] = json!({
            "idempotencyKey": idempotency_key,
        });
    }

    request
}

#[tokio::test]
async fn workspace_create_list_get_delete_lifecycle() -> Result<()> {
    let server = TestServer::start(ApiConfig::default()).await;
    let client = Client::new();

    let ws_a_path = server.workspace_path().join("ws-a");
    let ws_b_path = server.workspace_path().join("ws-b");

    // Create workspace A
    let create_a = rpc_request(
        "req-ws-create-a",
        "workspace.create",
        json!({
            "id": "ws-a",
            "name": "Workspace A",
            "path": ws_a_path,
        }),
        Some("idem-ws-create-a"),
    );
    let (status, payload) = post_rpc(&client, &server, &create_a).await?;
    assert_eq!(status, 200);
    assert_eq!(payload["result"]["workspace"]["id"], "ws-a");
    assert_eq!(payload["result"]["workspace"]["status"], "creating");

    // Create workspace B
    let create_b = rpc_request(
        "req-ws-create-b",
        "workspace.create",
        json!({
            "id": "ws-b",
            "name": "Workspace B",
            "path": ws_b_path,
        }),
        Some("idem-ws-create-b"),
    );
    let (status, _) = post_rpc(&client, &server, &create_b).await?;
    assert_eq!(status, 200);

    // List workspaces
    let list = rpc_request("req-ws-list", "workspace.list", json!({}), None);
    let (_, payload) = post_rpc(&client, &server, &list).await?;
    let workspaces = payload["result"]["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 2);

    // Get workspace A
    let get_a = rpc_request("req-ws-get-a", "workspace.get", json!({ "id": "ws-a" }), None);
    let (_, payload) = post_rpc(&client, &server, &get_a).await?;
    assert_eq!(payload["result"]["workspace"]["id"], "ws-a");

    // Delete workspace A
    let delete_a = rpc_request(
        "req-ws-delete-a",
        "workspace.delete",
        json!({ "id": "ws-a", "removeFiles": true }),
        Some("idem-ws-delete-a"),
    );
    let (status, _) = post_rpc(&client, &server, &delete_a).await?;
    assert_eq!(status, 200);

    // Verify deletion
    let list = rpc_request("req-ws-list-2", "workspace.list", json!({}), None);
    let (_, payload) = post_rpc(&client, &server, &list).await?;
    let workspaces = payload["result"]["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 1);
    assert_eq!(workspaces[0]["id"], "ws-b");

    server.stop().await;
    Ok(())
}

#[tokio::test]
async fn workspace_status_returns_health_info() -> Result<()> {
    let server = TestServer::start(ApiConfig::default()).await;
    let client = Client::new();

    let ws_path = server.workspace_path().join("ws-health");
    let create = rpc_request(
        "req-ws-create-health",
        "workspace.create",
        json!({ "id": "ws-health", "name": "Health Test", "path": ws_path }),
        Some("idem-ws-create-health"),
    );
    let (status, _) = post_rpc(&client, &server, &create).await?;
    assert_eq!(status, 200);

    // Get status with health
    let status_req = rpc_request(
        "req-ws-status-health",
        "workspace.status",
        json!({ "id": "ws-health" }),
        None,
    );
    let (_, payload) = post_rpc(&client, &server, &status_req).await?;
    let result = &payload["result"];
    assert_eq!(result["workspace"]["id"], "ws-health");
    assert_eq!(result["health"]["directoryExists"], true);
    assert_eq!(result["health"]["readable"], true);
    assert_eq!(result["health"]["writable"], true);
    assert_eq!(result["health"]["hasGitRepo"], false);
    assert_eq!(result["health"]["hasUlfConfig"], false);

    server.stop().await;
    Ok(())
}

#[tokio::test]
async fn task_domain_is_isolated_per_workspace() -> Result<()> {
    let server = TestServer::start(ApiConfig::default()).await;
    let client = Client::new();

    // Create two workspaces with explicit isolated paths
    let ws_alpha_path = server.workspace_path().join("ws-alpha");
    let ws_beta_path = server.workspace_path().join("ws-beta");
    for (ws_id, ws_path) in [("ws-alpha", &ws_alpha_path), ("ws-beta", &ws_beta_path)] {
        let create = rpc_request(
            &format!("req-ws-create-{ws_id}"),
            "workspace.create",
            json!({ "id": ws_id, "name": ws_id, "path": ws_path }),
            Some(&format!("idem-ws-create-{ws_id}")),
        );
        let (status, _) = post_rpc(&client, &server, &create).await?;
        assert_eq!(status, 200);
    }

    // Create a task in workspace alpha
    let create_task = rpc_request(
        "req-task-create-alpha",
        "task.create",
        json!({
            "id": "task-alpha-1",
            "title": "Alpha Task",
            "status": "open",
            "autoExecute": false,
            "workspaceId": "ws-alpha",
        }),
        Some("idem-task-alpha-1"),
    );
    let (status, payload) = post_rpc(&client, &server, &create_task).await?;
    assert_eq!(status, 200);
    assert_eq!(payload["result"]["task"]["id"], "task-alpha-1");

    // List tasks in workspace alpha — should find the task
    let list_alpha = rpc_request(
        "req-task-list-alpha",
        "task.list",
        json!({ "workspaceId": "ws-alpha" }),
        None,
    );
    let (_, payload) = post_rpc(&client, &server, &list_alpha).await?;
    let tasks = payload["result"]["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"], "task-alpha-1");

    // List tasks in workspace beta — should find nothing
    let list_beta = rpc_request(
        "req-task-list-beta",
        "task.list",
        json!({ "workspaceId": "ws-beta" }),
        None,
    );
    let (_, payload) = post_rpc(&client, &server, &list_beta).await?;
    let tasks = payload["result"]["tasks"].as_array().unwrap();
    assert!(tasks.is_empty(), "workspace beta should have no tasks");

    server.stop().await;
    Ok(())
}
