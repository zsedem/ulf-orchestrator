//! CLI commands for the `ulf workspace` namespace.
//!
//! Manage multi-workspace environments via the central daemon.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::daemon_client::{is_daemon_running, rpc_call, rpc_mutate};
use crate::daemon::start_daemon;

/// Manage workspaces.
#[derive(Parser, Debug)]
pub struct WorkspaceArgs {
    #[command(subcommand)]
    pub command: WorkspaceCommands,
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceCommands {
    /// Create a new workspace
    Create(WorkspaceCreateArgs),

    /// List all workspaces
    List,

    /// Show a single workspace
    Get(WorkspaceGetArgs),

    /// Delete a workspace
    Delete(WorkspaceDeleteArgs),

    /// Show status summary across all workspaces
    Status,

    /// Attach to a workspace (spawns Middle-Manager) [PHASE 4]
    Attach(WorkspaceAttachArgs),
}

#[derive(Parser, Debug)]
pub struct WorkspaceCreateArgs {
    /// Workspace identifier (e.g. "jira-007")
    pub id: String,

    /// Human-readable name (defaults to id)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Source to initialize from: git URL or local path
    #[arg(short, long)]
    pub from: Option<String>,

    /// Custom workspace directory path
    #[arg(short, long)]
    pub path: Option<String>,

    /// Setup prompt to run after creation
    #[arg(short = 'P', long)]
    pub prompt: Option<String>,

    /// Run setup prompt autonomously (no TUI)
    #[arg(long)]
    pub autonomous: bool,

    /// Wait for setup to complete (blocks until Ready or Error)
    #[arg(long)]
    pub wait: bool,
}

#[derive(Parser, Debug)]
pub struct WorkspaceGetArgs {
    pub id: String,
}

#[derive(Parser, Debug)]
pub struct WorkspaceDeleteArgs {
    pub id: String,

    /// Also remove workspace files from disk
    #[arg(long)]
    pub remove_files: bool,
}

#[derive(Parser, Debug)]
pub struct WorkspaceAttachArgs {
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct WorkspaceResult {
    workspace: WorkspaceOutput,
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
    setup_prompt: Option<String>,
    #[serde(default)]
    error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceStatusResult {
    workspace: WorkspaceOutput,
    health: WorkspaceHealth,
}

#[derive(Debug, Deserialize)]
struct WorkspaceHealth {
    directory_exists: bool,
    readable: bool,
    writable: bool,
    has_git_repo: bool,
    has_ulf_config: bool,
}

pub async fn execute(args: WorkspaceArgs) -> Result<()> {
    let is_mutating = matches!(
        args.command,
        WorkspaceCommands::Create(_)
            | WorkspaceCommands::Delete(_)
            | WorkspaceCommands::Attach(_)
    );

    if !is_daemon_running().await {
        if is_mutating {
            eprintln!("ulf daemon is not running — starting it now...");
            start_daemon().await?;
        } else {
            anyhow::bail!(
                "ulf daemon is not running.\nStart it with: ulf daemon start"
            );
        }
    }

    match args.command {
        WorkspaceCommands::Create(create_args) => create_workspace(create_args).await,
        WorkspaceCommands::List => list_workspaces().await,
        WorkspaceCommands::Get(get_args) => get_workspace(get_args).await,
        WorkspaceCommands::Delete(delete_args) => delete_workspace(delete_args).await,
        WorkspaceCommands::Status => workspace_status().await,
        WorkspaceCommands::Attach(attach_args) => attach_workspace(attach_args).await,
    }
}

async fn create_workspace(args: WorkspaceCreateArgs) -> Result<()> {
    let name = args.name.unwrap_or_else(|| args.id.clone());

    // Resolve setup prompt: explicit flag > global config default > none
    let setup_prompt = args.prompt.or_else(|| {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let user_config_path = home.join(".ulf").join("config.yml");
        let content = std::fs::read_to_string(user_config_path).ok()?;
        let config: ulf_core::UlfConfig = serde_yaml::from_str(&content).ok()?;
        config.workspace.default_setup_prompt
    });

    let mut params = serde_json::Map::new();
    params.insert("id".to_string(), json!(args.id));
    params.insert("name".to_string(), json!(name));
    if let Some(path) = args.path {
        params.insert("path".to_string(), json!(path));
    }
    if let Some(from) = args.from {
        params.insert("from".to_string(), json!(from));
    }
    if let Some(prompt) = setup_prompt {
        params.insert("setupPrompt".to_string(), json!(prompt));
    }

    let result: WorkspaceResult = rpc_mutate("workspace.create", Value::Object(params))
        .await?;

    let ws = result.workspace;
    println!("Created workspace '{}' at {}", ws.id, ws.path);
    println!("Status: {}", ws.status);

    if args.wait && ws.status == "creating" {
        println!("Waiting for setup to complete...");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        let timeout = std::time::Duration::from_secs(300);
        let start = std::time::Instant::now();

        loop {
            interval.tick().await;
            if start.elapsed() > timeout {
                anyhow::bail!("timed out waiting for workspace setup");
            }

            let status: WorkspaceStatusResult = rpc_call(
                "workspace.status",
                json!({ "id": ws.id }),
            )
            .await?;

            match status.workspace.status.as_str() {
                "ready" => {
                    println!("Workspace '{}' is ready!", ws.id);
                    return Ok(());
                }
                "error" => {
                    anyhow::bail!(
                        "workspace '{}' setup failed: {}",
                        ws.id,
                        status.workspace.error_message.unwrap_or_else(|| "unknown error".to_string())
                    );
                }
                _ => {
                    print!(".");
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                }
            }
        }
    }

    Ok(())
}

async fn list_workspaces() -> Result<()> {
    let result: WorkspaceListResult = rpc_call("workspace.list", json!({})).await?;

    if result.workspaces.is_empty() {
        println!("No workspaces found.");
        println!("Create one with: ulf workspace create <id>");
        return Ok(());
    }

    println!("{:<20} {:<12} PATH", "ID", "STATUS");
    for ws in result.workspaces {
        println!("{:<20} {:<12} {}", ws.id, ws.status, ws.path);
    }

    Ok(())
}

async fn get_workspace(args: WorkspaceGetArgs) -> Result<()> {
    let result: WorkspaceResult = rpc_call("workspace.get", json!({ "id": args.id })).await?;
    let ws = result.workspace;

    println!("Workspace: {}", ws.name);
    println!("  ID:     {}", ws.id);
    println!("  Path:   {}", ws.path);
    println!("  Status: {}", ws.status);
    if let Some(prompt) = ws.setup_prompt {
        println!("  Setup:  {}", prompt);
    }
    if let Some(error) = ws.error_message {
        println!("  Error:  {}", error);
    }

    Ok(())
}

async fn delete_workspace(args: WorkspaceDeleteArgs) -> Result<()> {
    let _: serde_json::Value = rpc_mutate(
        "workspace.delete",
        json!({
            "id": args.id,
            "removeFiles": args.remove_files,
        }),
    )
    .await?;

    println!("Deleted workspace '{}'", args.id);
    Ok(())
}

async fn workspace_status() -> Result<()> {
    let result: WorkspaceListResult = rpc_call("workspace.list", json!({})).await?;

    let total = result.workspaces.len();
    let ready = result.workspaces.iter().filter(|w| w.status == "ready").count();
    let creating = result.workspaces.iter().filter(|w| w.status == "creating").count();
    let error = result.workspaces.iter().filter(|w| w.status == "error").count();

    println!("Workspace Summary");
    println!("  Total:   {}", total);
    println!("  Ready:   {}", ready);
    println!("  Creating: {}", creating);
    println!("  Error:   {}", error);

    for ws in result.workspaces {
        println!();
        println!("  {} — {}", ws.id, ws.status);
        println!("    path: {}", ws.path);
        if ws.error_message.is_some() {
            println!("    error: {}", ws.error_message.unwrap());
        }
    }

    Ok(())
}

async fn attach_workspace(args: WorkspaceAttachArgs) -> Result<()> {
    // B8: Validate ID before use to prevent prompt injection.
    if !is_valid_workspace_id(&args.id) {
        anyhow::bail!(
            "workspace id must be 1-64 characters of alphanumeric, hyphen, underscore, or dot"
        );
    }

    let result: WorkspaceResult = rpc_call("workspace.get", json!({ "id": args.id })).await?;
    let ws = result.workspace;

    if ws.status != "ready" {
        anyhow::bail!(
            "workspace '{}' is not ready (status: {}). Wait for setup to complete or check errors with `ulf workspace get {}`",
            ws.id,
            ws.status,
            ws.id
        );
    }

    println!("Attaching to workspace '{}' at {}", ws.id, ws.path);

    // Load user config for middle-manager presets and prompt extensions.
    let user_config = load_user_config().unwrap_or_default();
    let mm_config = &user_config.workspace.middle_manager;
    let preset = mm_config
        .backend_preset
        .as_deref()
        .and_then(|name| user_config.workspace.backend_presets.get(name));

    // Build composable prompt: base template + user extensions.
    // Sanitize path for safe inclusion in prompt text.
    let safe_path = ws.path
        .replace('\\', "/")
        .replace('"', "\\\"")
        .replace('\n', " ")
        .replace('\r', " ");
    let mut prompt = format!(
        "You are the Ulf Workspace Middle-Manager for workspace '{}' at {}.\n\n\
        Your role is to help the user plan and execute coding tasks. You run inside the workspace directory, so you can inspect files and run commands directly.\n\n\
        When the user describes a task:\n\
        1. Explore the workspace to understand the codebase.\n\
        2. Suggest a concrete plan with workflow commands.\n\
        3. When the user agrees, execute the plan by running ulf commands.\n\n\
        To start a background workflow that returns immediately, use:\n\
          nohup ulf run --autonomous -P \"<prompt_file>\" > .ulf/manager-loop.log 2>&1 &\n\
        Then report the loop ID from the output.\n\n\
        To check running loops:\n\
          ulf loops\n\n\
        Available presets:\n\
          --config presets/code-assist.yml    (implementation tasks)\n\
          --config presets/review.yml         (code review)\n\
          --config presets/debug.yml          (debugging)\n\
          --config presets/research.yml       (research)\n\n",
        ws.id,
        safe_path
    );

    if let Some(workflow_preset) = &mm_config.workflow_preset {
        prompt.push_str(&format!(
            "Workflow context preset loaded from: {}\n\n",
            workflow_preset
        ));
    }

    for extension in &mm_config.prompt_extensions {
        prompt.push_str(extension);
        prompt.push('\n');
    }

    prompt.push_str("Start by asking the user what they'd like to work on.");

    // Write prompt to a file so we can use -P instead of -p (avoids shell escaping issues).
    let prompt_file = PathBuf::from(&ws.path).join(".ulf").join("manager-prompt.txt");
    if let Some(parent) = prompt_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&prompt_file, &prompt)
        .with_context(|| format!("failed to write manager prompt file: {}", prompt_file.display()))?;

    // Build the ulf command using the resolved backend preset (if any).
    let mut cmd = std::process::Command::new(
        preset
            .and_then(|p| p.command.as_deref())
            .unwrap_or("ulf"),
    );
    cmd.arg("run").arg("-P").arg(&prompt_file).current_dir(&ws.path);
    cmd.env("ULF_WORKSPACE_ID", &ws.id);

    // Apply preset args before the -P argument.
    if let Some(p) = preset {
        if !p.args.is_empty() {
            // Insert preset args after "run" but before -P
            let preset_args: Vec<String> = p.args.clone();
            let mut new_args: Vec<std::ffi::OsString> = vec!["run".into()];
            for a in &preset_args {
                new_args.push(a.into());
            }
            new_args.push("-P".into());
            new_args.push(prompt_file.as_os_str().to_os_string());
            let mut cleared = false;
            for arg in new_args {
                if !cleared {
                    cmd = std::process::Command::new(
                        p.command.as_deref().unwrap_or("ulf"),
                    );
                    cleared = true;
                }
                cmd.arg(arg);
            }
            cmd.current_dir(&ws.path);
        }
    }

    let status = cmd
        .status()
        .with_context(|| format!("failed to spawn ulf in workspace directory '{}'", ws.path))?;

    if !status.success() {
        anyhow::bail!("ulf session exited with status: {:?}", status.code());
    }

    Ok(())
}

fn load_user_config() -> Option<ulf_core::UlfConfig> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let path = home.join(".ulf").join("config.yml");
    let content = std::fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&content).ok()
}

/// Valid workspace IDs: alphanumeric, hyphen, underscore, dot only.
fn is_valid_workspace_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64
        && id.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}
