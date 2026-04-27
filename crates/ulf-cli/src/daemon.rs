//! CLI commands for the `ulf daemon` namespace.
//!
//! Controls the central ulf-api daemon process.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::daemon_client::{daemon_url, is_daemon_running, rpc_call};

/// Manage the ulf daemon.
#[derive(Parser, Debug)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub command: DaemonCommands,
}

#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
    /// Start the daemon in the background
    Start,

    /// Stop the running daemon
    Stop,

    /// Show daemon status
    Status,

    /// Launch an interactive manager agent that monitors all workspaces
    Manager,
}

#[derive(Debug, Deserialize)]
struct WorkspaceListResult {
    workspaces: Vec<WorkspaceOutput>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceOutput {
    id: String,
    name: String,
    path: String,
    status: String,
    #[serde(default)]
    error_message: Option<String>,
}

fn daemon_pid_file() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".ulf")
        .join("daemon")
        .join("daemon.pid")
}

pub async fn execute(args: DaemonArgs) -> Result<()> {
    match args.command {
        DaemonCommands::Start => start_daemon().await,
        DaemonCommands::Stop => stop_daemon().await,
        DaemonCommands::Status => daemon_status().await,
        DaemonCommands::Manager => daemon_manager().await,
    }
}

async fn daemon_manager() -> Result<()> {
    if !is_daemon_running().await {
        eprintln!("ulf daemon is not running — starting it now...");
        start_daemon().await?;
    }

    // Fetch workspace list
    let result: WorkspaceListResult = rpc_call("workspace.list", json!({})).await?;

    // Group workspaces by status
    let mut ready = Vec::new();
    let mut creating = Vec::new();
    let mut error = Vec::new();

    for ws in result.workspaces {
        match ws.status.as_str() {
            "ready" => ready.push(ws),
            "creating" => creating.push(ws),
            "error" => error.push(ws),
            _ => ready.push(ws), // Unknown status defaults to ready
        }
    }

    // Build system prompt
    let mut prompt = String::from(
        "You are the Ulf Daemon Manager. You monitor and coordinate all Ulf workspaces.\n\n"
    );

    prompt.push_str("## Ulf System Knowledge\n\n");
    prompt.push_str("Ulf is an AI orchestration system for vibe-coding across multiple workspaces.\n");
    prompt.push_str("Each workspace is an isolated project directory with its own git repo and Ulf configuration.\n");
    prompt.push_str("Available commands:\n");
    prompt.push_str("  ulf workspace list          - List all workspaces\n");
    prompt.push_str("  ulf workspace create <id>   - Create a new workspace\n");
    prompt.push_str("  ulf workspace get <id>      - Show workspace details\n");
    prompt.push_str("  ulf workspace attach <id>   - Attach to a workspace (interactive middle-manager)\n");
    prompt.push_str("  ulf workspace delete <id>   - Delete a workspace\n");
    prompt.push_str("  ulf workspace status        - Show workspace health summary\n");
    prompt.push_str("  ulf loops                   - List active orchestration loops\n");
    prompt.push_str("  ulf daemon start/stop/status- Control the daemon\n\n");

    prompt.push_str("## Current Workspace Status\n\n");

    prompt.push_str(&format!("### Ready ({}):\n", ready.len()));
    if ready.is_empty() {
        prompt.push_str("  (none)\n");
    } else {
        for ws in &ready {
            prompt.push_str(&format!("  - {}: {} ({}): {}\n", ws.id, ws.name, ws.status, ws.path));
        }
    }

    prompt.push_str(&format!("\n### Creating ({}):\n", creating.len()));
    if creating.is_empty() {
        prompt.push_str("  (none)\n");
    } else {
        for ws in &creating {
            prompt.push_str(&format!("  - {}: {} ({}): {}\n", ws.id, ws.name, ws.status, ws.path));
        }
    }

    prompt.push_str(&format!("\n### Error ({}):\n", error.len()));
    if error.is_empty() {
        prompt.push_str("  (none)\n");
    } else {
        for ws in &error {
            prompt.push_str(&format!("  - {}: {} ({}): {}\n", ws.id, ws.name, ws.status, ws.path));
            if let Some(ref err) = ws.error_message {
                prompt.push_str(&format!("    Error: {}\n", err));
            }
        }
    }

    prompt.push_str("\n## Your Role\n\n");
    prompt.push_str("As the Daemon Manager, you can:\n");
    prompt.push_str("1. Suggest plans for workspace setup or maintenance\n");
    prompt.push_str("2. Check on workspace health and diagnose issues\n");
    prompt.push_str("3. Trigger workflows by spawning background loops\n");
    prompt.push_str("4. Report status and summarize activity across workspaces\n\n");
    prompt.push_str("Start by summarizing the current state and asking what the user would like to do.");

    // Resolve backend from config or auto-detect
    let backend_name = resolve_backend()?;

    // Get interactive backend
    let cli_backend = ulf_adapters::CliBackend::for_interactive_prompt(&backend_name)
        .with_context(|| format!("failed to create interactive backend for '{}'", backend_name))?;

    // Spawn interactive session
    let (command, args, _stdin_input, _temp_file) = cli_backend.build_command(&prompt, true);

    let mut cmd = Command::new(&command);
    cmd.args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    cmd.envs(cli_backend.env_vars.iter().map(|(k, v)| (k, v)));

    let mut child = cmd.spawn()
        .with_context(|| format!("failed to spawn backend process: {}", command))?;

    child.wait()
        .with_context(|| "backend process exited unexpectedly")?;

    Ok(())
}

/// Resolve backend from config or auto-detect.
fn resolve_backend() -> Result<String> {
    // Try user config
    if let Some(config) = load_user_config()
        && config.cli.backend != "auto"
    {
        return Ok(config.cli.backend);
    }

    // Auto-detect
    ulf_adapters::detect_backend_default()
        .with_context(|| "no supported backend found. Install one of: claude, kiro, gemini, codex, amp, copilot, opencode, pi")
}

fn load_user_config() -> Option<ulf_core::UlfConfig> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let path = home.join(".ulf").join("config.yml");
    let content = std::fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&content).ok()
}

pub async fn start_daemon() -> Result<()> {
    if is_daemon_running().await {
        println!("Daemon is already running at {}", daemon_url());
        return Ok(());
    }

    let pid_file = daemon_pid_file();
    if let Some(parent) = pid_file.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create daemon state dir: {}", parent.display()))?;
    }

    let log_file = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".ulf")
        .join("daemon")
        .join("daemon.log");
    if let Some(parent) = log_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .with_context(|| format!("failed to open daemon log file: {}", log_file.display()))?;

    let mut cmd = Command::new("ulf-api");
    cmd.stdin(Stdio::null())
        .stdout(log.try_clone().with_context(|| "failed to clone log handle for stdout")?)
        .stderr(log);

    // If ulf-api is not in PATH, try to find it next to the current binary
    if Command::new("ulf-api").arg("--version").output().is_err()
        && let Ok(current_exe) = std::env::current_exe()
            && let Some(bin_dir) = current_exe.parent() {
                let local_api = bin_dir.join("ulf-api");
                if local_api.exists() {
                    let log2 = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&log_file)
                        .with_context(|| format!("failed to open daemon log file: {}", log_file.display()))?;
                    cmd = Command::new(local_api);
                    cmd.stdin(Stdio::null())
                        .stdout(log2.try_clone().with_context(|| "failed to clone log handle for stdout")?)
                        .stderr(log2);
                }
            }

    let child = cmd.spawn().with_context(|| {
        "failed to start ulf-api daemon.\n\
         Make sure `ulf-api` is installed and in PATH, or build it with:\n\
         cargo build -p ulf-api"
    })?;

    let pid = child.id();
    std::fs::write(&pid_file, pid.to_string())
        .with_context(|| format!("failed to write PID file: {}", pid_file.display()))?;

    // Wait a moment and verify it's actually up
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    if is_daemon_running().await {
        println!("Daemon started on {}", daemon_url());
        println!("PID: {}", pid);
    } else {
        anyhow::bail!("daemon process started but is not responding to health checks");
    }

    Ok(())
}

async fn stop_daemon() -> Result<()> {
    let pid_file = daemon_pid_file();

    if !pid_file.exists() {
        if is_daemon_running().await {
            println!("Daemon is running but no PID file found at {}", pid_file.display());
            println!("You may need to stop it manually.");
            return Ok(());
        }
        println!("Daemon is not running.");
        return Ok(());
    }

    let pid_str = std::fs::read_to_string(&pid_file)
        .with_context(|| format!("failed to read PID file: {}", pid_file.display()))?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .with_context(|| format!("invalid PID in file: {}", pid_str))?;

    #[cfg(unix)]
    {
        use nix::sys::signal::{Signal, kill};
        use nix::unistd::Pid as NixPid;

        kill(NixPid::from_raw(pid as i32), Signal::SIGTERM)
            .with_context(|| format!("failed to send SIGTERM to daemon PID {}", pid))?;
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, we can't easily signal a process. Just report.
        anyhow::bail!("ulf daemon stop is not supported on this platform");
    }

    // Wait for it to go down
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        if !is_daemon_running().await {
            let _ = std::fs::remove_file(&pid_file);
            println!("Daemon stopped.");
            return Ok(());
        }
    }

    println!("Daemon did not stop gracefully. You may need to kill PID {} manually.", pid);
    Ok(())
}

async fn daemon_status() -> Result<()> {
    let pid_file = daemon_pid_file();
    let url = daemon_url();

    if is_daemon_running().await {
        let pid_info = if pid_file.exists() {
            match std::fs::read_to_string(&pid_file) {
                Ok(pid_str) => format!(" (PID: {})", pid_str.trim()),
                Err(_) => String::new(),
            }
        } else {
            String::new()
        };
        println!("Daemon is running on {}{}", url, pid_info);
    } else {
        println!("Daemon is not running.");
        println!("Start it with: ulf daemon start");
    }

    Ok(())
}
