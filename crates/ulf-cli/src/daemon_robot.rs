//! Daemon-backed RobotService for workspace-attached loops.
//!
//! Implements `RobotService` by delegating to the daemon's `human.ask`
//! and `human.get_response` RPC methods. No local Telegram polling.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Deserialize;
use serde_json::json;
use tracing::{debug, info};

use ulf_proto::RobotService;

use crate::daemon_client;

/// A RobotService that delegates human-in-the-loop to the daemon.
pub struct DaemonRobotClient {
    workspace_id: String,
    loop_id: String,
    timeout_secs: u64,
    shutdown: Arc<AtomicBool>,
}

#[derive(Debug, Deserialize)]
struct ResponseResult {
    status: String,
    #[serde(default)]
    response: Option<String>,
}

impl DaemonRobotClient {
    /// Create a new daemon robot client.
    pub fn new(workspace_id: String, loop_id: String, timeout_secs: u64) -> Self {
        Self {
            workspace_id,
            loop_id,
            timeout_secs,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if the daemon has RObot enabled.
    pub async fn is_robot_available() -> bool {
        match daemon_client::rpc_call::<serde_json::Value>("system.capabilities", json!({})).await
        {
            Ok(capabilities) => capabilities
                .get("robot")
                .and_then(|r| r.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            Err(e) => {
                debug!(error = %e, "failed to query daemon capabilities for RObot");
                false
            }
        }
    }
}

impl RobotService for DaemonRobotClient {
    fn send_question(&self, payload: &str) -> anyhow::Result<i32> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|e| anyhow::anyhow!("no tokio runtime: {e}"))?;

        let _result: serde_json::Value = rt.block_on(async {
            daemon_client::rpc_mutate(
                "human.ask",
                json!({
                    "workspaceId": self.workspace_id,
                    "loopId": self.loop_id,
                    "question": payload,
                }),
            )
            .await
        })?;

        info!("human question sent via daemon");

        // Return a synthetic message ID (question hash)
        Ok(payload.len() as i32)
    }

    fn wait_for_response(&self, _events_path: &Path) -> anyhow::Result<Option<String>> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|e| anyhow::anyhow!("no tokio runtime: {e}"))?;

        info!(
            timeout_secs = self.timeout_secs,
            "waiting for human response via daemon (long-polling)"
        );

        // Single long-polling call. The server holds the connection open
        // until a response arrives or its own timeout elapses.
        // We add a small client-side buffer (timeout + 5s) so the server
        // always times out first and returns cleanly.
        let client_timeout = Duration::from_secs(self.timeout_secs + 5);

        let result: ResponseResult = rt.block_on(async {
            tokio::time::timeout(client_timeout, async {
                daemon_client::rpc_call(
                    "human.get_response",
                    json!({
                        "workspaceId": self.workspace_id,
                        "loopId": self.loop_id,
                        "timeoutSecs": self.timeout_secs,
                    }),
                )
                .await
            })
            .await
        }).map_err(|_| anyhow::anyhow!("client-side timeout waiting for daemon response"))??;

        match result.status.as_str() {
            "answered" => {
                info!("human response received via daemon");
                Ok(result.response)
            }
            _ => {
                info!("human response still pending or timed out");
                Ok(None)
            }
        }
    }

    fn send_checkin(
        &self,
        iteration: u32,
        _elapsed: Duration,
        _context: Option<&ulf_proto::CheckinContext>,
    ) -> anyhow::Result<i32> {
        // Check-ins are not implemented for daemon mode yet.
        debug!(iteration, "check-in skipped in daemon robot mode");
        Ok(0)
    }

    fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }

    fn shutdown_flag(&self) -> Arc<AtomicBool> {
        self.shutdown.clone()
    }

    fn stop(self: Box<Self>) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}
