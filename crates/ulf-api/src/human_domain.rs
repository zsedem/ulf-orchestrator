//! Human-in-the-loop domain for daemon-centric RObot.
//!
//! Manages pending questions from multiple workspace loops and routes
//! human responses back to the correct loop. Used by the daemon's
//! Telegram poller and the `human.*` RPC methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use ulf_telegram::{StateManager, TelegramState};

/// A pending question awaiting human response.
#[derive(Debug, Clone)]
pub struct PendingQuestion {
    pub workspace_id: String,
    pub loop_id: String,
    pub question: String,
    pub asked_at: DateTime<Utc>,
    pub telegram_message_id: i32,
    /// The response, if received.
    pub response: Option<String>,
    /// Whether this question has timed out.
    pub timed_out: bool,
}

/// Human-in-the-loop domain. Thread-safe via interior mutability.
pub struct HumanDomain {
    state_manager: StateManager,
    /// In-memory pending questions keyed by (workspace_id, loop_id).
    /// The state file is the source of truth for persistence; this map
    /// holds the runtime state including responses.
    pending: Arc<Mutex<HashMap<(String, String), PendingQuestion>>>,
}

impl HumanDomain {
    /// Create a new HumanDomain with the given daemon state directory.
    pub fn new(daemon_state_dir: &std::path::Path) -> Self {
        let state_path = daemon_state_dir.join("telegram-state.json");
        let state_manager = StateManager::new(&state_path);
        let pending = Arc::new(Mutex::new(HashMap::new()));

        // Hydrate in-memory state from disk
        if let Ok(Some(state)) = state_manager.load() {
            let mut map = pending.lock().unwrap();
            for (workspace_id, loop_map) in &state.workspace_pending_questions {
                for (loop_id, q) in loop_map {
                    map.insert(
                        (workspace_id.clone(), loop_id.clone()),
                        PendingQuestion {
                            workspace_id: workspace_id.clone(),
                            loop_id: loop_id.clone(),
                            question: String::new(), // Not stored in file; will be empty on restart
                            asked_at: q.asked_at,
                            telegram_message_id: q.message_id,
                            response: None,
                            timed_out: false,
                        },
                    );
                }
            }
        }

        Self {
            state_manager,
            pending,
        }
    }

    /// Ask a question on behalf of a workspace loop.
    ///
    /// Stores the pending question and returns a question ID.
    /// Does not send the Telegram message — the caller (poller) does that
    /// and then updates the `telegram_message_id`.
    pub fn ask(
        &self,
        workspace_id: &str,
        loop_id: &str,
        question: &str,
    ) -> Result<String, crate::errors::ApiError> {
        let question_id = format!("{}-{}", workspace_id, loop_id);
        let mut state = self
            .state_manager
            .load_or_default()
            .map_err(|e| crate::errors::ApiError::internal(format!("failed to load telegram state: {e}")))?;

        self.state_manager
            .add_workspace_pending_question(&mut state, workspace_id, loop_id, 0)
            .map_err(|e| crate::errors::ApiError::internal(format!("failed to save pending question: {e}")))?;

        let mut pending = self.pending.lock().unwrap();
        pending.insert(
            (workspace_id.to_string(), loop_id.to_string()),
            PendingQuestion {
                workspace_id: workspace_id.to_string(),
                loop_id: loop_id.to_string(),
                question: question.to_string(),
                asked_at: Utc::now(),
                telegram_message_id: 0,
                response: None,
                timed_out: false,
            },
        );

        info!(workspace_id, loop_id, %question_id, "human question queued");
        Ok(question_id)
    }

    /// Record the Telegram message ID for a pending question.
    pub fn set_telegram_message_id(
        &self,
        workspace_id: &str,
        loop_id: &str,
        message_id: i32,
    ) {
        let mut pending = self.pending.lock().unwrap();
        if let Some(q) = pending.get_mut(&(workspace_id.to_string(), loop_id.to_string())) {
            q.telegram_message_id = message_id;
        }
    }

    /// Resolve a human response for a pending question.
    ///
    /// Returns true if a matching pending question was found and resolved.
    pub fn resolve_response(
        &self,
        workspace_id: &str,
        loop_id: &str,
        response: &str,
    ) -> Result<bool, crate::errors::ApiError> {
        let mut pending = self.pending.lock().unwrap();
        let key = (workspace_id.to_string(), loop_id.to_string());

        if let Some(q) = pending.get_mut(&key) {
            q.response = Some(response.to_string());
            drop(pending); // release lock before I/O

            let mut state = self
                .state_manager
                .load_or_default()
                .map_err(|e| crate::errors::ApiError::internal(format!("failed to load telegram state: {e}")))?;

            self.state_manager
                .remove_workspace_pending_question(&mut state, workspace_id, loop_id)
                .map_err(|e| crate::errors::ApiError::internal(format!("failed to remove pending question: {e}")))?;

            info!(workspace_id, loop_id, "human response resolved");
            Ok(true)
        } else {
            debug!(workspace_id, loop_id, "no pending question found for response");
            Ok(false)
        }
    }

    /// Check if a question has been answered.
    ///
    /// Returns the response if available, or None if still pending.
    /// Also marks timed-out questions.
    pub fn get_response(
        &self,
        workspace_id: &str,
        loop_id: &str,
        timeout_secs: u64,
    ) -> Option<String> {
        let mut pending = self.pending.lock().unwrap();
        let key = (workspace_id.to_string(), loop_id.to_string());

        if let Some(q) = pending.get_mut(&key) {
            // Check for explicit response
            if let Some(ref response) = q.response {
                let response = response.clone();
                pending.remove(&key);
                return Some(response);
            }

            // Check for timeout
            let elapsed = Utc::now().signed_duration_since(q.asked_at);
            if elapsed.num_seconds() > timeout_secs as i64 {
                q.timed_out = true;
                pending.remove(&key);

                // Also clean up state file
                let _ = self.state_manager.load_or_default().and_then(|mut state| {
                    self.state_manager
                        .remove_workspace_pending_question(&mut state, workspace_id, loop_id)
                });

                info!(workspace_id, loop_id, timeout_secs, "human question timed out");
                return None; // None means timeout to caller
            }

            // Still pending
            None
        } else {
            // No such question — treat as timeout/already resolved
            None
        }
    }

    /// List all pending questions.
    pub fn list_pending(&self) -> Vec<PendingQuestionSummary> {
        let pending = self.pending.lock().unwrap();
        pending
            .values()
            .filter(|q| q.response.is_none() && !q.timed_out)
            .map(|q| PendingQuestionSummary {
                workspace_id: q.workspace_id.clone(),
                loop_id: q.loop_id.clone(),
                question: q.question.clone(),
                asked_at: q.asked_at,
            })
            .collect()
    }

    /// Cancel a pending question.
    pub fn cancel(
        &self,
        workspace_id: &str,
        loop_id: &str,
    ) -> Result<bool, crate::errors::ApiError> {
        let mut pending = self.pending.lock().unwrap();
        let removed = pending
            .remove(&(workspace_id.to_string(), loop_id.to_string()))
            .is_some();
        drop(pending);

        if removed {
            let mut state = self
                .state_manager
                .load_or_default()
                .map_err(|e| crate::errors::ApiError::internal(format!("failed to load telegram state: {e}")))?;

            let _ = self
                .state_manager
                .remove_workspace_pending_question(&mut state, workspace_id, loop_id);
        }

        Ok(removed)
    }

    /// Find the (workspace_id, loop_id) for a given Telegram reply message ID.
    pub fn find_by_reply(&self, reply_message_id: i32) -> Option<(String, String)> {
        let pending = self.pending.lock().unwrap();
        pending
            .values()
            .find(|q| q.telegram_message_id == reply_message_id)
            .map(|q| (q.workspace_id.clone(), q.loop_id.clone()))
    }

    /// Count total pending questions.
    pub fn pending_count(&self) -> usize {
        let pending = self.pending.lock().unwrap();
        pending.values().filter(|q| q.response.is_none() && !q.timed_out).count()
    }
}

/// Serializable summary of a pending question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingQuestionSummary {
    pub workspace_id: String,
    pub loop_id: String,
    pub question: String,
    pub asked_at: DateTime<Utc>,
}
