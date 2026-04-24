use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Maximum age (in minutes) a workspace may remain in `Creating` before being
/// considered stale on daemon restart.
const STALE_CREATING_MINUTES: i64 = 5;

/// Valid workspace IDs: alphanumeric, hyphen, underscore, dot only.
fn is_valid_workspace_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Reject paths that contain `..` components, which could escape the intended
/// sandbox.
fn has_path_traversal(path: &str) -> bool {
    Path::new(path).components().any(|c| matches!(c, std::path::Component::ParentDir))
}

/// Validate that a local `from` path is safe to copy from.
fn validate_from_path(from: &str) -> Result<PathBuf, ApiError> {
    if has_path_traversal(from) {
        return Err(ApiError::invalid_params(
            "'from' path cannot contain '..' components",
        ));
    }
    let src = PathBuf::from(from);
    if !src.exists() {
        return Err(ApiError::invalid_params(format!(
            "source path '{}' does not exist",
            src.display()
        )));
    }
    // Ensure the source is under the current working directory or the user's home
    // directory to prevent copying arbitrary system paths.
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| cwd.clone());
    let canonical_src = src.canonicalize().unwrap_or_else(|_| src.clone());
    let canonical_cwd = cwd.canonicalize().unwrap_or_else(|_| cwd.clone());
    let canonical_home = home.canonicalize().unwrap_or_else(|_| home.clone());
    if !canonical_src.starts_with(&canonical_cwd) && !canonical_src.starts_with(&canonical_home) {
        return Err(ApiError::forbidden(format!(
            "source path '{}' is outside the current directory or home directory",
            src.display()
        )));
    }
    Ok(src)
}

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
    /// All workspace paths must reside under this root for sandbox safety.
    workspace_root: PathBuf,
}

impl WorkspaceDomain {
    pub fn new(
        daemon_state_dir: impl AsRef<Path>,
        workspace_root: impl AsRef<Path>,
    ) -> Self {
        let store_path = daemon_state_dir.as_ref().join("workspaces.json");
        let mut domain = Self {
            store_path,
            workspaces: HashMap::new(),
            workspace_root: workspace_root.as_ref().to_path_buf(),
        };
        if let Err(e) = domain.load() {
            tracing::warn!(error = %e, "failed to load workspace registry, starting fresh");
        }
        domain.recover_stale_workspaces();
        domain
    }

    /// Quick check that a workspace ID is available (does not do I/O).
    pub fn validate_id(&self, id: &str) -> Result<(), ApiError> {
        if !is_valid_workspace_id(id) {
            return Err(ApiError::invalid_params(
                "workspace id must be 1-64 characters of alphanumeric, hyphen, or underscore",
            ));
        }
        if self.workspaces.contains_key(id) {
            return Err(ApiError::invalid_params(format!(
                "workspace '{}' already exists",
                id
            )));
        }
        Ok(())
    }

    /// Perform slow I/O to prepare the workspace directory.
    ///
    /// This is a static method so it can be called without holding the registry
    /// mutex, avoiding blocking concurrent reads during `git clone`.
    pub fn create_workspace_dir(
        params: &WorkspaceCreateParams,
        workspace_root: &Path,
    ) -> Result<Workspace, ApiError> {
        if !is_valid_workspace_id(&params.id) {
            return Err(ApiError::invalid_params(
                "workspace id must be 1-64 characters of alphanumeric, hyphen, underscore, or dot",
            ));
        }

        let path = match params.path.as_deref() {
            Some(p) => {
                if has_path_traversal(p) {
                    return Err(ApiError::invalid_params(
                        "workspace path cannot contain '..' components",
                    ));
                }
                let pb = PathBuf::from(p);
                let canonical = pb.canonicalize().unwrap_or_else(|_| pb.clone());
                let canonical_root = workspace_root
                    .canonicalize()
                    .unwrap_or_else(|_| workspace_root.to_path_buf());
                if !canonical.starts_with(&canonical_root) {
                    return Err(ApiError::forbidden(format!(
                        "workspace path '{}' is outside the workspace root '{}'",
                        pb.display(),
                        canonical_root.display()
                    )));
                }
                pb
            }
            None => Self::default_workspace_path_static(&params.id),
        };

        // If `from` is provided, clone/copy into the target path.
        if let Some(ref from) = params.from {
            if from.starts_with("http://")
                || from.starts_with("https://")
                || from.starts_with("git@")
            {
                // Git clone
                let status = std::process::Command::new("git")
                    .args(["clone", from, &path.display().to_string()])
                    .status()
                    .map_err(|e| {
                        ApiError::internal(format!("failed to spawn git clone: {}", e))
                    })?;
                if !status.success() {
                    return Err(ApiError::internal(format!(
                        "git clone failed for source '{}'",
                        from
                    )));
                }
            } else {
                // Local path — recursive copy
                let src = validate_from_path(from)?;
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
        workspace.setup_prompt = params.setup_prompt.clone();

        Ok(workspace)
    }

    pub fn create(&mut self, params: WorkspaceCreateParams) -> Result<Workspace, ApiError> {
        self.validate_id(&params.id)?;
        let workspace = Self::create_workspace_dir(&params, &self.workspace_root)?;
        self.register(workspace.clone())?;
        Ok(workspace)
    }

    /// Insert a workspace into the registry and persist atomically.
    pub fn register(&mut self, workspace: Workspace) -> Result<(), ApiError> {
        self.workspaces.insert(workspace.id.clone(), workspace);
        self.save()?;
        Ok(())
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
        let writable = if directory_exists {
            tempfile::NamedTempFile::new_in(path)
                .map(|_| true)
                .unwrap_or(false)
        } else {
            false
        };
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

        if params.remove_files {
            let canonical_path = workspace
                .path
                .canonicalize()
                .unwrap_or_else(|_| workspace.path.clone());
            let canonical_root = self
                .workspace_root
                .canonicalize()
                .unwrap_or_else(|_| self.workspace_root.clone());
            if !canonical_path.starts_with(&canonical_root) {
                tracing::warn!(
                    workspace_id = %params.id,
                    path = %workspace.path.display(),
                    "refusing to remove workspace files outside sandbox"
                );
            } else if let Err(e) = fs::remove_dir_all(&workspace.path) {
                tracing::warn!(
                    workspace_id = %params.id,
                    path = %workspace.path.display(),
                    error = %e,
                    "failed to remove workspace files"
                );
            }
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

        // Atomic write: write to temp file, then rename.
        let temp_path = self.store_path.with_extension("tmp");
        {
            let mut temp_file = fs::File::create(&temp_path).map_err(|e| {
                ApiError::internal(format!(
                    "failed to create temp registry file '{}': {}",
                    temp_path.display(),
                    e
                ))
            })?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&temp_path).map_err(|e| {
                    ApiError::internal(format!("failed to stat temp registry file: {}", e))
                })?.permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&temp_path, perms).map_err(|e| {
                    ApiError::internal(format!("failed to set registry permissions: {}", e))
                })?;
            }
            temp_file.write_all(content.as_bytes()).map_err(|e| {
                ApiError::internal(format!(
                    "failed to write temp registry file '{}': {}",
                    temp_path.display(),
                    e
                ))
            })?;
            temp_file.sync_all().map_err(|e| {
                ApiError::internal(format!("failed to sync temp registry file: {}", e))
            })?;
        }

        fs::rename(&temp_path, &self.store_path).map_err(|e| {
            ApiError::internal(format!(
                "failed to rename temp registry file to '{}': {}",
                self.store_path.display(),
                e
            ))
        })?;

        Ok(())
    }

    fn default_workspace_path(&self, id: &str) -> PathBuf {
        Self::default_workspace_path_static(id)
    }

    fn default_workspace_path_static(id: &str) -> PathBuf {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".ulf")
            .join("workspaces")
            .join(id)
    }

    /// On startup, transition any `Creating` workspaces that are older than
    /// `STALE_CREATING_MINUTES` to `Error` so they don't remain zombies after a
    /// daemon crash.
    fn recover_stale_workspaces(&mut self) {
        let now = chrono::Utc::now();
        let threshold = chrono::Duration::minutes(STALE_CREATING_MINUTES);
        let mut changed = false;
        for workspace in self.workspaces.values_mut() {
            if workspace.status == ulf_core::WorkspaceStatus::Creating
                && now.signed_duration_since(workspace.created_at) > threshold
            {
                workspace.mark_error("setup did not complete before daemon restart");
                changed = true;
            }
        }
        if changed {
            if let Err(e) = self.save() {
                tracing::warn!(error = ?e, "failed to save registry after stale workspace recovery");
            }
        }
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
        let mut domain = WorkspaceDomain::new(tmp.path(), tmp.path());

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
        let mut domain = WorkspaceDomain::new(tmp.path(), tmp.path());

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
            let mut domain = WorkspaceDomain::new(tmp.path(), tmp.path());
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
            let domain = WorkspaceDomain::new(tmp.path(), tmp.path());
            let list = domain.list();
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].setup_prompt, Some("setup".to_string()));
        }
    }
}
