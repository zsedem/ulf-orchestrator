//! Daemon-backed RobotService for workspace-attached loops.
//!
//! Implements `RobotService` by delegating to the daemon's `human.ask`
//! and `human.get_response` RPC methods. No local Telegram polling.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, info, warn};

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
struct AskResult {
    question_id: String,
    status: String,
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

        let result: AskResult = rt.block_on(async {
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

        info!(
            question_id = %result.question_id,
            "human question sent via daemon"
        );

        // Return a synthetic message ID (question hash)
        Ok(result.question_id.len() as i32)
    }

    fn wait_for_response(&self, _events_path: &Path) -> anyhow::Result<Option<String>> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|e| anyhow::anyhow!("no tokio runtime: {e}"))?;

        let deadline = Instant::now() + Duration::from_secs(self.timeout_secs);
        let poll_interval = Duration::from_secs(1);

        info!(
            timeout_secs = self.timeout_secs,
            "waiting for human response via daemon"
        );

        loop {
            if Instant::now() >= deadline {
                warn!("timed out waiting for human response via daemon");
                return Ok(None);
            }

            if self.shutdown.load(Ordering::Relaxed) {
                info!("shutdown while waiting for human response");
                return Ok(None);
            }

            let result: ResponseResult = match rt.block_on(async {
                daemon_client::rpc_call(
                    "human.get_response",
                    json!({
                        "workspaceId": self.workspace_id,
                        "loopId": self.loop_id,
                        "timeoutSecs": self.timeout_secs,
                    }),
                )
                .await
            }) {
                Ok(r) => r,
                Err(e) => {
                    warn!(error = %e, "daemon human.get_response failed");
                    std::thread::sleep(poll_interval);
                    continue;
                }
            };

            match result.status.as_str() {
                "answered" => {
                    info!("human response received via daemon");
                    return Ok(result.response);
                }
                "pending" => {
                    std::thread::sleep(poll_interval);
                    continue;
                }
                other => {
                    warn!(status = other, "unexpected human.get_response status");
                    std::thread::sleep(poll_interval);
                    continue;
                }
            }
        }
    }

    fn send_checkin(
        &self,
        iteration: u32,
        elapsed: Duration,
        context: Option<&ulf_proto::CheckinContext>,
    ) -> anyhow::Result<i32> {
        // Check-ins are not implemented for daemon mode yet.
        // The daemon could support this via a separate endpoint.
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
