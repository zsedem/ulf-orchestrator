use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::TelegramResult;

/// Persistent state for the Telegram bot, stored at `.ulf/telegram-state.json`
/// (per-workspace) or `~/.ulf/daemon/telegram-state.json` (daemon-global).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramState {
    /// The chat ID for the human operator (auto-detected from first message).
    pub chat_id: Option<i64>,

    /// Timestamp of the last message seen.
    pub last_seen: Option<DateTime<Utc>>,

    /// Last Telegram update ID processed by the bot.
    #[serde(default)]
    pub last_update_id: Option<i32>,

    /// Pending questions keyed by loop ID, tracking which message awaits a reply.
    /// Legacy field — used for per-workspace state.
    #[serde(default)]
    pub pending_questions: HashMap<String, PendingQuestion>,

    /// Pending questions keyed by workspace ID, then loop ID.
    /// Used for daemon-global state in multi-workspace mode.
    #[serde(default)]
    pub workspace_pending_questions: HashMap<String, HashMap<String, PendingQuestion>>,
}

/// A question sent to the human that is awaiting a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingQuestion {
    /// When the question was sent.
    pub asked_at: DateTime<Utc>,

    /// The Telegram message ID, used to match reply-to routing.
    pub message_id: i32,

    /// The question text (persisted so it survives daemon restarts).
    #[serde(default)]
    pub question_text: Option<String>,
}

/// Manages persistence of Telegram bot state to disk.
pub struct StateManager {
    path: PathBuf,
}

impl StateManager {
    /// Create a new StateManager that reads/writes to the given path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Load state from disk. Returns `None` if the file doesn't exist.
    pub fn load(&self) -> TelegramResult<Option<TelegramState>> {
        if !self.path.exists() {
            return Ok(None);
        }
        let contents = std::fs::read_to_string(&self.path)?;
        let state: TelegramState = serde_json::from_str(&contents)?;
        Ok(Some(state))
    }

    /// Save state to disk using atomic write (temp file + rename).
    pub fn save(&self, state: &TelegramState) -> TelegramResult<()> {
        let json = serde_json::to_string_pretty(state)?;
        let tmp_path = self.path.with_extension("json.tmp");

        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&tmp_path, &json)?;
        std::fs::rename(&tmp_path, &self.path)?;
        Ok(())
    }

    /// Load existing state or create a fresh empty state.
    pub fn load_or_default(&self) -> TelegramResult<TelegramState> {
        Ok(self.load()?.unwrap_or_else(|| TelegramState {
            chat_id: None,
            last_seen: None,
            last_update_id: None,
            pending_questions: HashMap::new(),
            workspace_pending_questions: HashMap::new(),
        }))
    }

    /// Set the chat ID and persist immediately.
    pub fn set_chat_id(&self, chat_id: i64) -> TelegramResult<()> {
        let mut state = self.load_or_default()?;
        state.chat_id = Some(chat_id);
        self.save(&state)
    }

    /// Set the last update ID and persist immediately.
    pub fn set_last_update_id(&self, update_id: i32) -> TelegramResult<()> {
        let mut state = self.load_or_default()?;
        state.last_update_id = Some(update_id);
        self.save(&state)
    }

    /// Add a pending question for a given loop.
    pub fn add_pending_question(
        &self,
        state: &mut TelegramState,
        loop_id: &str,
        message_id: i32,
    ) -> TelegramResult<()> {
        state.pending_questions.insert(
            loop_id.to_string(),
            PendingQuestion {
                asked_at: Utc::now(),
                message_id,
                question_text: None,
            },
        );
        self.save(state)
    }

    /// Remove a pending question for a given loop.
    pub fn remove_pending_question(
        &self,
        state: &mut TelegramState,
        loop_id: &str,
    ) -> TelegramResult<()> {
        state.pending_questions.remove(loop_id);
        self.save(state)
    }

    /// Given a reply_to_message_id, find which loop it belongs to (legacy per-workspace).
    pub fn get_loop_for_reply(
        &self,
        state: &TelegramState,
        reply_message_id: i32,
    ) -> Option<String> {
        state
            .pending_questions
            .iter()
            .find(|(_, q)| q.message_id == reply_message_id)
            .map(|(loop_id, _)| loop_id.clone())
    }

    /// Given a reply_to_message_id, find which (workspace, loop) it belongs to (daemon-global).
    pub fn get_workspace_loop_for_reply(
        &self,
        state: &TelegramState,
        reply_message_id: i32,
    ) -> Option<(String, String)> {
        for (workspace_id, loop_questions) in &state.workspace_pending_questions {
            for (loop_id, question) in loop_questions {
                if question.message_id == reply_message_id {
                    return Some((workspace_id.clone(), loop_id.clone()));
                }
            }
        }
        None
    }

    /// Add a pending question for a workspace + loop in daemon-global state.
    pub fn add_workspace_pending_question(
        &self,
        state: &mut TelegramState,
        workspace_id: &str,
        loop_id: &str,
        message_id: i32,
        question_text: Option<&str>,
    ) -> TelegramResult<()> {
        state
            .workspace_pending_questions
            .entry(workspace_id.to_string())
            .or_default()
            .insert(
                loop_id.to_string(),
                PendingQuestion {
                    asked_at: Utc::now(),
                    message_id,
                    question_text: question_text.map(|s| s.to_string()),
                },
            );
        self.save(state)
    }

    /// Remove a pending question for a workspace + loop in daemon-global state.
    pub fn remove_workspace_pending_question(
        &self,
        state: &mut TelegramState,
        workspace_id: &str,
        loop_id: &str,
    ) -> TelegramResult<()> {
        if let Some(loop_map) = state.workspace_pending_questions.get_mut(workspace_id) {
            loop_map.remove(loop_id);
            if loop_map.is_empty() {
                state.workspace_pending_questions.remove(workspace_id);
            }
        }
        self.save(state)
    }

    /// Return the path to the state file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_manager() -> (StateManager, TempDir) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("telegram-state.json");
        (StateManager::new(path), dir)
    }

    #[test]
    fn load_missing_file_returns_none() {
        let (mgr, _dir) = test_manager();
        assert!(mgr.load().unwrap().is_none());
    }

    #[test]
    fn save_and_load_round_trip() {
        let (mgr, _dir) = test_manager();
        let state = TelegramState {
            chat_id: Some(123_456),
            last_seen: Some(Utc::now()),
            last_update_id: Some(101),
            pending_questions: HashMap::new(),
            workspace_pending_questions: HashMap::new(),
        };
        mgr.save(&state).unwrap();

        let loaded = mgr.load().unwrap().unwrap();
        assert_eq!(loaded.chat_id, Some(123_456));
        assert_eq!(loaded.last_update_id, Some(101));
    }

    #[test]
    fn corrupted_json_returns_error() {
        let (mgr, _dir) = test_manager();
        std::fs::write(mgr.path(), "not json").unwrap();
        assert!(mgr.load().is_err());
    }

    #[test]
    fn pending_question_tracking() {
        let (mgr, _dir) = test_manager();
        let mut state = mgr.load_or_default().unwrap();

        mgr.add_pending_question(&mut state, "main", 42).unwrap();
        assert!(state.pending_questions.contains_key("main"));
        assert_eq!(state.pending_questions["main"].message_id, 42);

        mgr.remove_pending_question(&mut state, "main").unwrap();
        assert!(!state.pending_questions.contains_key("main"));
    }

    #[test]
    fn reply_routing_lookup() {
        let (mgr, _dir) = test_manager();
        let mut state = mgr.load_or_default().unwrap();

        mgr.add_pending_question(&mut state, "main", 10).unwrap();
        mgr.add_pending_question(&mut state, "feature-auth", 20)
            .unwrap();

        assert_eq!(mgr.get_loop_for_reply(&state, 10), Some("main".to_string()));
        assert_eq!(
            mgr.get_loop_for_reply(&state, 20),
            Some("feature-auth".to_string())
        );
        assert_eq!(mgr.get_loop_for_reply(&state, 99), None);
    }
}
