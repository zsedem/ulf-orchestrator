//! CLI commands for the `ulf workspace` namespace.
//!
//! Manage multi-workspace environments via the central daemon.

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde_json::json;

use crate::daemon_client::{is_daemon_running, rpc_call, rpc_mutate};

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
    #[arg(short, long)]
    pub prompt: Option<String>,

    /// Run setup prompt autonomously (no TUI)
    #[arg(long)]
    pub autonomous: bool,
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

pub async fn execute(args: WorkspaceArgs) -> Result<()> {
    if !is_daemon_running().await {
        anyhow::bail!(
            "ulf daemon is not running.\n\
             Start it with: ulf daemon start\n\
             Or set ULF_API_URL to a running daemon."
        );
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

    let result: WorkspaceResult = rpc_mutate(
        "workspace.create",
        json!({
            "id": args.id,
            "name": name,
            "path": args.path,
            "setupPrompt": args.prompt,
        }),
    )
    .await?;

    let ws = result.workspace;
    println!("Created workspace '{}' at {}", ws.id, ws.path);
    println!("Status: {}", ws.status);

    // TODO: Phase 3 — run setup prompt if provided
    // TODO: Phase 2 — clone from `--from` if provided

    Ok(())
}

async fn list_workspaces() -> Result<()> {
    let result: WorkspaceListResult = rpc_call("workspace.list", json!({})).await?;

    if result.workspaces.is_empty() {
        println!("No workspaces found.");
        println!("Create one with: ulf workspace create <id>");
        return Ok(());
    }

    println!("{:<20} {:<12} {}", "ID", "STATUS", "PATH");
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
    }

    Ok(())
}

async fn attach_workspace(args: WorkspaceAttachArgs) -> Result<()> {
    // Phase 4: spawn Middle-Manager session
    println!("Attaching to workspace '{}'...", args.id);
    println!("(Middle-Manager not yet implemented — Phase 4)");
    Ok(())
}
