use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ulf_core::{Workspace, WorkspaceRegistryData, WorkspaceRegistryError, WorkspaceStatus};

use crate::errors::ApiError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatusResult {
    pub workspace: Workspace,
    pub health: WorkspaceHealth,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceHealth {
    pub directory_exists: bool,
    pub readable: bool,
    pub writable: bool,
    pub has_git_repo: bool,
    pub has_ulf_config: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceCreateParams {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup_prompt: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDeleteParams {
    pub id: String,
    #[serde(default)]
    pub remove_files: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceListResult {
    pub workspaces: Vec<Workspace>,
}

/// Manages the global workspace registry for the daemon.
///
/// Persisted at `{daemon_state_dir}/workspaces.json`.
pub struct WorkspaceDomain {
    store_path: PathBuf,
    workspaces: HashMap<String, Workspace>,
}

impl WorkspaceDomain {
    pub fn new(daemon_state_dir: impl AsRef<Path>) -> Self {
        let store_path = daemon_state_dir.as_ref().join("workspaces.json");
        let mut domain = Self {
            store_path,
            workspaces: HashMap::new(),
        };
        if let Err(e) = domain.load() {
            tracing::warn!(error = %e, "failed to load workspace registry, starting fresh");
        }
        domain
    }

    pub fn create(&mut self, params: WorkspaceCreateParams) -> Result<Workspace, ApiError> {
        if self.workspaces.contains_key(&params.id) {
            return Err(ApiError::invalid_params(format!(
                "workspace '{}' already exists",
                params.id
            )));
        }

        let path = match params.path {
            Some(p) => PathBuf::from(p),
            None => self.default_workspace_path(&params.id),
        };

        // If `from` is provided, clone/copy into the target path.
        if let Some(ref from) = params.from {
            if from.starts_with("http://") || from.starts_with("https://") || from.starts_with("git@") {
                // Git clone
                let status = std::process::Command::new("git")
                    .args(["clone", from, &path.display().to_string()])
                    .status()
                    .map_err(|e| ApiError::internal(format!("failed to spawn git clone: {}", e)))?;
                if !status.success() {
                    return Err(ApiError::internal(format!(
                        "git clone failed for source '{}'",
                        from
                    )));
                }
            } else {
                // Local path — recursive copy
                let src = PathBuf::from(from);
                if !src.exists() {
                    return Err(ApiError::invalid_params(format!(
                        "source path '{}' does not exist",
                        src.display()
                    )));
                }
                copy_dir_all(&src, &path).map_err(|e| {
                    ApiError::internal(format!(
                        "failed to copy from '{}' to '{}': {}",
                        src.display(),
                        path.display(),
                        e
                    ))
                })?;
            }
        } else {
            // Create the workspace directory eagerly so downstream setup has a
            // well-defined root to operate in.
            fs::create_dir_all(&path).map_err(|e| {
                ApiError::internal(format!(
                    "failed to create workspace directory '{}': {}",
                    path.display(),
                    e
                ))
            })?;
        }

        let mut workspace = Workspace::new(&params.id, &params.name, &path);
        workspace.setup_prompt = params.setup_prompt;

        self.workspaces.insert(params.id, workspace.clone());
        self.save()?;

        Ok(workspace)
    }

    pub fn list(&self) -> Vec<Workspace> {
        let mut workspaces: Vec<Workspace> = self.workspaces.values().cloned().collect();
        workspaces.sort_by_key(|a| a.created_at);
        workspaces
    }

    pub fn get(&self, id: &str) -> Result<Workspace, ApiError> {
        self.workspaces
            .get(id)
            .cloned()
            .ok_or_else(|| ApiError::not_found(format!("workspace '{}' not found", id)))
    }

    pub fn status(&self, id: &str) -> Result<WorkspaceStatusResult, ApiError> {
        let workspace = self.get(id)?;
        let path = &workspace.path;

        let directory_exists = path.exists();
        let readable = directory_exists && fs::read_dir(path).is_ok();
        let writable = directory_exists
            && fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path.join(".ulf-health-check.tmp"))
                .and_then(|_| fs::remove_file(path.join(".ulf-health-check.tmp")))
                .is_ok();
        let has_git_repo = path.join(".git").exists();
        let has_ulf_config = path.join("ulf.yml").exists();

        Ok(WorkspaceStatusResult {
            workspace,
            health: WorkspaceHealth {
                directory_exists,
                readable,
                writable,
                has_git_repo,
                has_ulf_config,
            },
        })
    }

    pub fn update_status(
        &mut self,
        id: &str,
        status: WorkspaceStatus,
        error_message: Option<String>,
    ) -> Result<Workspace, ApiError> {
        {
            let workspace = self
                .workspaces
                .get_mut(id)
                .ok_or_else(|| ApiError::not_found(format!("workspace '{}' not found", id)))?;

            workspace.status = status;
            workspace.updated_at = chrono::Utc::now();
            if let Some(msg) = error_message {
                workspace.error_message = Some(msg);
            } else if status == WorkspaceStatus::Ready {
                workspace.error_message = None;
            }
        }

        self.save()?;
        self.get(id)
    }

    pub fn delete(&mut self, params: WorkspaceDeleteParams) -> Result<(), ApiError> {
        let workspace = self
            .workspaces
            .remove(&params.id)
            .ok_or_else(|| ApiError::not_found(format!("workspace '{}' not found", params.id)))?;

        if params.remove_files
            && let Err(e) = fs::remove_dir_all(&workspace.path) {
                tracing::warn!(
                    workspace_id = %params.id,
                    path = %workspace.path.display(),
                    error = %e,
                    "failed to remove workspace files"
                );
            }

        self.save()?;
        Ok(())
    }

    pub fn ensure_dir(&self, id: &str) -> Result<PathBuf, ApiError> {
        let workspace = self.get(id)?;
        fs::create_dir_all(&workspace.path).map_err(|e| {
            ApiError::internal(format!(
                "failed to create workspace directory '{}': {}",
                workspace.path.display(),
                e
            ))
        })?;
        Ok(workspace.path)
    }

    // ------------------------------------------------------------------
    // Persistence
    // ------------------------------------------------------------------

    fn load(&mut self) -> Result<(), WorkspaceRegistryError> {
        if !self.store_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.store_path)?;
        if content.trim().is_empty() {
            return Ok(());
        }

        let data: WorkspaceRegistryData =
            serde_json::from_str(&content).map_err(|e| WorkspaceRegistryError::ParseError(e.to_string()))?;

        self.workspaces = data
            .workspaces
            .into_iter()
            .map(|w| (w.id.clone(), w))
            .collect();

        Ok(())
    }

    fn save(&self) -> Result<(), ApiError> {
        if let Some(parent) = self.store_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ApiError::internal(format!(
                    "failed to create workspace registry directory '{}': {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        let data = WorkspaceRegistryData {
            workspaces: self.workspaces.values().cloned().collect(),
        };

        let content = serde_json::to_string_pretty(&data).map_err(|e| {
            ApiError::internal(format!("failed to serialize workspace registry: {}", e))
        })?;

        fs::write(&self.store_path, content).map_err(|e| {
            ApiError::internal(format!(
                "failed to write workspace registry '{}': {}",
                self.store_path.display(),
                e
            ))
        })?;

        Ok(())
    }

    fn default_workspace_path(&self, id: &str) -> PathBuf {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".ulf")
            .join("workspaces")
            .join(id)
    }
}

/// Recursively copy a directory tree.
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_list() {
        let tmp = TempDir::new().unwrap();
        let mut domain = WorkspaceDomain::new(tmp.path());

        let w = domain
            .create(WorkspaceCreateParams {
                id: "ws-1".to_string(),
                name: "My Workspace".to_string(),
                path: None,
                from: None,
                setup_prompt: None,
            })
            .unwrap();

        assert_eq!(w.id, "ws-1");
        assert_eq!(w.name, "My Workspace");
        assert_eq!(w.status, WorkspaceStatus::Creating);

        let list = domain.list();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_duplicate_id_fails() {
        let tmp = TempDir::new().unwrap();
        let mut domain = WorkspaceDomain::new(tmp.path());

        domain
            .create(WorkspaceCreateParams {
                id: "ws-1".to_string(),
                name: "First".to_string(),
                path: None,
                from: None,
                setup_prompt: None,
            })
            .unwrap();

        let result = domain.create(WorkspaceCreateParams {
            id: "ws-1".to_string(),
            name: "Second".to_string(),
            path: None,
            from: None,
            setup_prompt: None,
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_persistence() {
        let tmp = TempDir::new().unwrap();

        {
            let mut domain = WorkspaceDomain::new(tmp.path());
            domain
                .create(WorkspaceCreateParams {
                    id: "persist".to_string(),
                    name: "Persisted".to_string(),
                    path: None,
                    from: None,
                    setup_prompt: Some("setup".to_string()),
                })
                .unwrap();
        }

        {
            let domain = WorkspaceDomain::new(tmp.path());
            let list = domain.list();
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].setup_prompt, Some("setup".to_string()));
        }
    }
}
