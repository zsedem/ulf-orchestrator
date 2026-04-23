//! Completion gates: end-of-session backpressure scripts.
//!
//! When the agent emits the completion promise (e.g. `LOOP_COMPLETE`), Ralph
//! runs the configured gates *before* accepting termination. If any gate exits
//! non-zero, its output is injected into the event bus as a `task.resume` event
//! so the agent can fix the issue and retry.
//!
//! # Example
//!
//! ```yaml
//! event_loop:
//!   completion_gates:
//!     - name: tests-pass
//!       command: ["cargo", "test"]
//!       timeout_seconds: 120
//! ```

use crate::config::CompletionGateConfig;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Result of running a set of completion gates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionGateResult {
    /// All gates exited with code 0.
    AllPassed,
    /// The first failing gate and its captured output.
    Failed {
        /// Gate name from config.
        name: String,
        /// Process exit code (None if killed by signal/timeout).
        exit_code: Option<i32>,
        /// Captured stdout (possibly truncated).
        stdout: String,
        /// Captured stderr (possibly truncated).
        stderr: String,
        /// Whether the gate hit its timeout.
        timed_out: bool,
    },
}

/// Structured failure info for building backpressure payloads.
pub struct CompletionGateFailed {
    pub name: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

impl CompletionGateResult {
    /// Returns true if all gates passed.
    pub fn all_passed(&self) -> bool {
        matches!(self, Self::AllPassed)
    }


}

/// Runner for completion gates.
#[derive(Debug, Clone, Default)]
pub struct CompletionGateRunner;

impl CompletionGateRunner {
    /// Creates a new runner.
    pub fn new() -> Self {
        Self
    }

    /// Runs the configured gates sequentially.
    ///
    /// Returns `AllPassed` only if every gate exits with code 0.
    /// On the first failure, returns `Failed` immediately.
    pub fn run_gates(
        &self,
        gates: &[CompletionGateConfig],
        workspace_root: &Path,
    ) -> CompletionGateResult {
        for gate in gates {
            debug!(gate = %gate.name, command = ?gate.command, "Running completion gate");
            let start = Instant::now();

            let result = run_single_gate(gate, workspace_root);

            match result {
                Ok((exit_code, stdout, stderr)) => {
                    if exit_code == 0 {
                        debug!(
                            gate = %gate.name,
                            duration_ms = start.elapsed().as_millis(),
                            "Completion gate passed"
                        );
                        continue;
                    }

                    warn!(
                        gate = %gate.name,
                        exit_code,
                        duration_ms = start.elapsed().as_millis(),
                        "Completion gate failed"
                    );
                    return CompletionGateResult::Failed {
                        name: gate.name.clone(),
                        exit_code: Some(exit_code),
                        stdout,
                        stderr,
                        timed_out: false,
                    };
                }
                Err(GateRunError::TimedOut) => {
                    warn!(
                        gate = %gate.name,
                        timeout = gate.timeout_seconds,
                        "Completion gate timed out"
                    );
                    return CompletionGateResult::Failed {
                        name: gate.name.clone(),
                        exit_code: None,
                        stdout: String::new(),
                        stderr: format!("Gate timed out after {} seconds", gate.timeout_seconds),
                        timed_out: true,
                    };
                }
                Err(GateRunError::Io { source }) => {
                    warn!(
                        gate = %gate.name,
                        error = %source,
                        "Completion gate spawn failed"
                    );
                    return CompletionGateResult::Failed {
                        name: gate.name.clone(),
                        exit_code: None,
                        stdout: String::new(),
                        stderr: format!("Gate spawn error: {source}"),
                        timed_out: false,
                    };
                }
            }
        }

        CompletionGateResult::AllPassed
    }
}

#[derive(Debug)]
enum GateRunError {
    TimedOut,
    Io { source: std::io::Error },
}

fn run_single_gate(
    gate: &CompletionGateConfig,
    workspace_root: &Path,
) -> Result<(i32, String, String), GateRunError> {
    let executable = gate
        .command
        .first()
        .map(String::as_str)
        .unwrap_or("");

    let resolved_cwd = gate
        .cwd
        .as_ref()
        .map(|p| {
            if p.is_absolute() {
                p.clone()
            } else {
                workspace_root.join(p)
            }
        })
        .unwrap_or_else(|| workspace_root.to_path_buf());

    let mut command = Command::new(executable);
    command.args(gate.command.iter().skip(1));
    command.current_dir(&resolved_cwd);
    command.envs(&gate.env);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|e| GateRunError::Io { source: e })?;

    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    let max_output = gate.max_output_bytes as usize;

    let stdout_handle = spawn_stream_collector(stdout, max_output);
    let stderr_handle = spawn_stream_collector(stderr, max_output);

    let timeout = Duration::from_secs(gate.timeout_seconds);
    let start = Instant::now();

    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = stdout_handle.join();
                    let _ = stderr_handle.join();
                    return Err(GateRunError::TimedOut);
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = stdout_handle.join();
                let _ = stderr_handle.join();
                return Err(GateRunError::Io { source: e });
            }
        }
    };

    let stdout_text = stdout_handle.join().unwrap_or_default();
    let stderr_text = stderr_handle.join().unwrap_or_default();

    let exit_code = status.code().unwrap_or(-1);
    Ok((exit_code, stdout_text, stderr_text))
}

fn spawn_stream_collector(
    mut stream: impl Read + Send + 'static,
    max_bytes: usize,
) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    let remaining = max_bytes.saturating_sub(buf.len());
                    if remaining > 0 {
                        buf.extend_from_slice(&chunk[..n.min(remaining)]);
                    }
                }
                Err(_) => break,
            }
        }
        String::from_utf8_lossy(&buf).into_owned()
    })
}

/// Builds the backpressure payload injected as `task.resume` when a gate fails.
pub fn build_gate_backpressure_payload(
    name: &str,
    exit_code: Option<i32>,
    stdout: &str,
    stderr: &str,
    timed_out: bool,
) -> String {
    let status = if timed_out {
        "timed out".to_string()
    } else {
        format!("failed (exit code {})", exit_code.unwrap_or(-1))
    };

    let mut payload = format!(
        "Completion gate '{}' {}.\n\n",
        name, status
    );

    if !stdout.trim().is_empty() {
        payload.push_str("## stdout\n");
        payload.push_str(stdout);
        payload.push('\n');
    }

    if !stderr.trim().is_empty() {
        payload.push_str("## stderr\n");
        payload.push_str(stderr);
        payload.push('\n');
    }

    payload.push_str("\nFix the issue and emit LOOP_COMPLETE again.");
    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gate_config(name: &str, command: &[&str]) -> CompletionGateConfig {
        CompletionGateConfig {
            name: name.to_string(),
            command: command.iter().map(|s| s.to_string()).collect(),
            cwd: None,
            env: HashMap::new(),
            timeout_seconds: 30,
            max_output_bytes: 1024,
        }
    }

    #[test]
    fn test_gate_passes() {
        let runner = CompletionGateRunner::new();
        let gates = vec![gate_config("true-gate", &["true"])];
        let result = runner.run_gates(&gates, Path::new("."));
        assert!(result.all_passed());
    }

    #[test]
    fn test_gate_fails() {
        let runner = CompletionGateRunner::new();
        let gates = vec![gate_config("false-gate", &["false"])];
        let result = runner.run_gates(&gates, Path::new("."));
        assert!(!result.all_passed());
        match result {
            CompletionGateResult::Failed { name, exit_code, .. } => {
                assert_eq!(name, "false-gate");
                assert_eq!(exit_code, Some(1));
            }
            _ => panic!("Expected Failed"),
        }
    }

    #[test]
    fn test_gate_short_circuits() {
        let runner = CompletionGateRunner::new();
        let gates = vec![
            gate_config("false-gate", &["false"]),
            gate_config("true-gate", &["true"]),
        ];
        let result = runner.run_gates(&gates, Path::new("."));
        match result {
            CompletionGateResult::Failed { name, .. } => {
                assert_eq!(name, "false-gate");
            }
            _ => panic!("Expected first gate to fail"),
        }
    }

    #[test]
    fn test_gate_captures_output() {
        let runner = CompletionGateRunner::new();
        let gates = vec![gate_config("echo-gate", &["sh", "-c", "echo hello; exit 1"])];
        let result = runner.run_gates(&gates, Path::new("."));
        match result {
            CompletionGateResult::Failed { stdout, .. } => {
                assert!(stdout.contains("hello"));
            }
            _ => panic!("Expected Failed"),
        }
    }

    #[test]
    fn test_gate_timeout() {
        let runner = CompletionGateRunner::new();
        let mut gate = gate_config("sleep-gate", &["sleep", "10"]);
        gate.timeout_seconds = 1;
        let gates = vec![gate];
        let start = Instant::now();
        let result = runner.run_gates(&gates, Path::new("."));
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(3),
            "Gate should have timed out quickly, took {:?}",
            elapsed
        );
        match result {
            CompletionGateResult::Failed { timed_out, .. } => {
                assert!(timed_out);
            }
            _ => panic!("Expected timeout"),
        }
    }

    #[test]
    fn test_backpressure_payload_format() {
        let payload = build_gate_backpressure_payload(
            "tests-pass",
            Some(1),
            "running 1 test\ntest foo ... FAILED\n",
            "error: assertion failed\n",
            false,
        );
        assert!(payload.contains("Completion gate 'tests-pass' failed"));
        assert!(payload.contains("exit code 1"));
        assert!(payload.contains("## stdout"));
        assert!(payload.contains("## stderr"));
        assert!(payload.contains("Fix the issue and emit LOOP_COMPLETE again."));
    }

    #[test]
    fn test_backpressure_payload_timeout() {
        let payload = build_gate_backpressure_payload("slow-gate", None, "", "", true);
        assert!(payload.contains("timed out"));
    }
}
