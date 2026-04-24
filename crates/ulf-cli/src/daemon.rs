//! CLI commands for the `ulf daemon` namespace.
//!
//! Controls the central ulf-api daemon process.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::daemon_client::{daemon_url, is_daemon_running};

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
    }
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
