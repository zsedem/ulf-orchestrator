//! Multi-workspace registry types for the global daemon.
//!
//! These types define what a workspace is in the installable, multi-repo
//! vibe-coding model.  Workspaces are created under `~/.ulf/workspaces/`
//! (or a user-configured root) and are tracked by the central daemon.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Lifecycle state of a workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceStatus {
    /// Workspace directory exists but the setup prompt is still running.
    Creating,
    /// Setup prompt finished successfully; workspace is usable.
    Ready,
    /// Setup prompt failed or another error occurred.
    Error,
    /// Workspace has been soft-deleted / archived.
    Archived,
}

/// A workspace managed by the central daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Stable workspace identifier (usually derived from the name).
    pub id: String,

    /// Human-readable name (e.g. "jira-007").
    pub name: String,

    /// Absolute path to the workspace root directory.
    pub path: PathBuf,

    /// Current lifecycle state.
    pub status: WorkspaceStatus,

    /// When the workspace was first created.
    pub created_at: DateTime<Utc>,

    /// When the workspace was last modified.
    pub updated_at: DateTime<Utc>,

    /// Optional prompt that is run once when the workspace is created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup_prompt: Option<String>,

    /// Error message when status is `Error`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl Workspace {
    /// Create a new workspace in the `Creating` state.
    pub fn new(id: impl Into<String>, name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            path: path.into(),
            status: WorkspaceStatus::Creating,
            created_at: now,
            updated_at: now,
            setup_prompt: None,
            error_message: None,
        }
    }

    /// Mark the workspace as ready.
    pub fn mark_ready(&mut self) {
        self.status = WorkspaceStatus::Ready;
        self.updated_at = Utc::now();
        self.error_message = None;
    }

    /// Mark the workspace as failed with an error message.
    pub fn mark_error(&mut self, message: impl Into<String>) {
        self.status = WorkspaceStatus::Error;
        self.updated_at = Utc::now();
        self.error_message = Some(message.into());
    }

    /// Mark the workspace as archived.
    pub fn mark_archived(&mut self) {
        self.status = WorkspaceStatus::Archived;
        self.updated_at = Utc::now();
    }
}

/// Serializable registry of all workspaces.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WorkspaceRegistryData {
    pub workspaces: Vec<Workspace>,
}

/// Errors that can occur during workspace operations.
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceRegistryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse workspace registry: {0}")]
    ParseError(String),

    #[error("Workspace not found: {0}")]
    NotFound(String),

    #[error("Workspace already exists: {0}")]
    AlreadyExists(String),
}
