//! Thin HTTP client for talking to the ulf-api daemon.
//!
//! The daemon is expected to be running on `http://127.0.0.1:3000` by default.
//! If it is not reachable, CLI commands that require the daemon will fail
//! with a helpful message (auto-start is planned for Phase 7).

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{Value, json};

const DEFAULT_DAEMON_URL: &str = "http://127.0.0.1:3000";

/// Resolve the daemon URL from environment or default.
pub fn daemon_url() -> String {
    std::env::var("ULF_API_URL").unwrap_or_else(|_| DEFAULT_DAEMON_URL.to_string())
}

/// Check whether the daemon is reachable.
pub async fn is_daemon_running() -> bool {
    let url = format!("{}/rpc/v1", daemon_url());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let body = json!({
        "apiVersion": "v1",
        "id": "ping",
        "method": "system.health",
        "params": {}
    });

    match client.post(&url).json(&body).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// Send an RPC request to the daemon and deserialize the result.
pub async fn rpc_call<T>(method: &str, params: Value) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let url = format!("{}/rpc/v1", daemon_url());
    let client = reqwest::Client::new();

    let body = json!({
        "apiVersion": "v1",
        "id": uuid::Uuid::new_v4().to_string(),
        "method": method,
        "params": params,
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("failed to connect to daemon at {url}"))?;

    let status = response.status();
    let envelope: Value = response
        .json()
        .await
        .with_context(|| "daemon returned invalid JSON")?;

    if !status.is_success() {
        let message = envelope
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("unknown daemon error");
        anyhow::bail!("daemon error: {}", message);
    }

    if let Some(error) = envelope.get("error") {
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown daemon error");
        anyhow::bail!("daemon error: {}", message);
    }

    let result = envelope
        .get("result")
        .cloned()
        .unwrap_or_else(|| json!({}));

    serde_json::from_value(result).with_context(|| format!("failed to parse daemon response for {method}"))
}

/// Simple wrapper for mutating RPC calls that require an idempotency key.
pub async fn rpc_mutate<T>(method: &str, params: Value) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let url = format!("{}/rpc/v1", daemon_url());
    let client = reqwest::Client::new();

    let body = json!({
        "apiVersion": "v1",
        "id": uuid::Uuid::new_v4().to_string(),
        "method": method,
        "params": params,
        "meta": {
            "idempotencyKey": uuid::Uuid::new_v4().to_string(),
        }
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("failed to connect to daemon at {url}"))?;

    let status = response.status();
    let envelope: Value = response
        .json()
        .await
        .with_context(|| "daemon returned invalid JSON")?;

    if !status.is_success() {
        let message = envelope
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("unknown daemon error");
        anyhow::bail!("daemon error: {}", message);
    }

    if let Some(error) = envelope.get("error") {
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown daemon error");
        anyhow::bail!("daemon error: {}", message);
    }

    let result = envelope
        .get("result")
        .cloned()
        .unwrap_or_else(|| json!({}));

    serde_json::from_value(result).with_context(|| format!("failed to parse daemon response for {method}"))
}
