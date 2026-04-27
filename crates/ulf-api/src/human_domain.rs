//! Human-in-the-loop domain for daemon-centric RObot.
//!
//! Manages pending questions from multiple workspace loops and routes
//! human responses back to the correct loop. Used by the daemon's
//! Telegram poller and the `human.*` RPC methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;
use teloxide::prelude::Requester;
use tracing::{debug, info, warn};

use ulf_telegram::StateManager;

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
}

/// Human-in-the-loop domain. Thread-safe via interior mutability.
pub struct HumanDomain {
    state_manager: StateManager,
    /// In-memory pending questions keyed by (workspace_id, loop_id).
    pending: Arc<Mutex<HashMap<(String, String), PendingQuestion>>>,
    /// Notifiers for waiting `get_response` callers.
    notifiers: Arc<Mutex<HashMap<(String, String), std::sync::mpsc::Sender<String>>>>,
    /// Optional Telegram bot for sending outbound messages.
    bot: Option<teloxide::Bot>,
}

impl HumanDomain {
    /// Create a new HumanDomain with the given daemon state directory.
    pub fn new(
        daemon_state_dir: &std::path::Path,
        bot_token: Option<String>,
        api_url: Option<String>,
    ) -> Self {
        let state_path = daemon_state_dir.join("telegram-state.json");
        let state_manager = StateManager::new(&state_path);
        let pending = Arc::new(Mutex::new(HashMap::new()));
        let notifiers = Arc::new(Mutex::new(HashMap::new()));

        let bot = bot_token.map(|token| {
            ulf_telegram::apply_api_url(teloxide::Bot::new(&token), api_url.as_deref())
        });

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
                            question: q.question_text.clone().unwrap_or_default(),
                            asked_at: q.asked_at,
                            telegram_message_id: q.message_id,
                            response: None,
                        },
                    );
                }
            }
        }

        Self {
            state_manager,
            pending,
            notifiers,
            bot,
        }
    }

    /// Ask a question on behalf of a workspace loop.
    ///
    /// Sends the question via Telegram if a bot is configured and chat_id is known.
    /// Stores the pending question and returns a question ID.
    pub fn ask(
        &self,
        workspace_id: &str,
        loop_id: &str,
        question: &str,
    ) -> Result<String, crate::errors::ApiError> {
        let question_id = format!("{}-{}", workspace_id, loop_id);

        let mut message_id = 0;

        // Send the question via Telegram if we have a bot and chat_id
        if let Some(ref bot) = self.bot
            && let Ok(state) = self.state_manager.load_or_default() {
                if let Some(chat_id) = state.chat_id {
                    let result = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            bot.send_message(teloxide::types::ChatId(chat_id), question).await
                        })
                    });
                    match result {
                        Ok(sent) => {
                            message_id = sent.id.0;
                            info!(chat_id, message_id, "sent human question via Telegram");
                        }
                        Err(e) => {
                            warn!(error = %e, "failed to send Telegram message");
                        }
                    }
                } else {
                    warn!("no chat_id known — question queued but not sent to Telegram");
                }
            }

        let mut state = self
            .state_manager
            .load_or_default()
            .map_err(|e| crate::errors::ApiError::internal(format!("failed to load telegram state: {e}")))?;

        self.state_manager
            .add_workspace_pending_question(&mut state, workspace_id, loop_id, message_id, Some(question))
            .map_err(|e| crate::errors::ApiError::internal(format!("failed to save pending question: {e}")))?;

        let mut pending = self.pending.lock().unwrap();
        pending.insert(
            (workspace_id.to_string(), loop_id.to_string()),
            PendingQuestion {
                workspace_id: workspace_id.to_string(),
                loop_id: loop_id.to_string(),
                question: question.to_string(),
                asked_at: Utc::now(),
                telegram_message_id: message_id,
                response: None,
            },
        );

        info!(workspace_id, loop_id, %question_id, message_id, "human question queued");
        Ok(question_id)
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
            drop(pending); // release lock before notifying

            // Notify any waiting get_response caller
            let mut notifiers = self.notifiers.lock().unwrap();
            if let Some(tx) = notifiers.remove(&key) {
                let _ = tx.send(response.to_string());
            }
            drop(notifiers);

            info!(workspace_id, loop_id, "human response resolved");
            Ok(true)
        } else {
            debug!(workspace_id, loop_id, "no pending question found for response");
            Ok(false)
        }
    }

    /// Check if a question has been answered.
    ///
    /// Returns the response if available, or None if still pending or timed out.
    /// Blocks with a long-polling wait (up to timeout_secs) for a response.
    pub fn get_response(
        &self,
        workspace_id: &str,
        loop_id: &str,
        timeout_secs: u64,
    ) -> Option<String> {
        let key = (workspace_id.to_string(), loop_id.to_string());

        // Fast path: check if already resolved or timed out
        {
            let mut pending = self.pending.lock().unwrap();
            if let Some(q) = pending.get_mut(&key) {
                if let Some(ref response) = q.response {
                    let response = response.clone();
                    pending.remove(&key);

                    // Also clean up state file
                    let _ = self.state_manager.load_or_default().and_then(|mut state| {
                        self.state_manager
                            .remove_workspace_pending_question(&mut state, workspace_id, loop_id)
                    });

                    return Some(response);
                }

                // Check for timeout
                let elapsed = Utc::now().signed_duration_since(q.asked_at);
                if elapsed.num_seconds() > timeout_secs as i64 {
                    pending.remove(&key);

                    let _ = self.state_manager.load_or_default().and_then(|mut state| {
                        self.state_manager
                            .remove_workspace_pending_question(&mut state, workspace_id, loop_id)
                    });

                    info!(workspace_id, loop_id, timeout_secs, "human question timed out");
                    return None;
                }
            } else {
                // No such question — treat as timeout/already resolved
                return None;
            }
        }

        // Slow path: create a channel and wait for notification
        let (tx, rx) = std::sync::mpsc::channel();
        {
            let mut notifiers = self.notifiers.lock().unwrap();
            notifiers.insert(key.clone(), tx);
        }

        let result = rx.recv_timeout(Duration::from_secs(timeout_secs));

        // Clean up notifier regardless of result
        {
            let mut notifiers = self.notifiers.lock().unwrap();
            notifiers.remove(&key);
        }

        match result {
            Ok(response) => {
                let mut pending = self.pending.lock().unwrap();
                pending.remove(&key);

                let _ = self.state_manager.load_or_default().and_then(|mut state| {
                    self.state_manager
                        .remove_workspace_pending_question(&mut state, workspace_id, loop_id)
                });

                Some(response)
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Check one more time in case response arrived just after timeout
                let mut pending = self.pending.lock().unwrap();
                if let Some(q) = pending.get_mut(&key) {
                    if let Some(ref response) = q.response {
                        let response = response.clone();
                        pending.remove(&key);

                        let _ = self.state_manager.load_or_default().and_then(|mut state| {
                            self.state_manager
                                .remove_workspace_pending_question(&mut state, workspace_id, loop_id)
                        });

                        Some(response)
                    } else {
                        let elapsed = Utc::now().signed_duration_since(q.asked_at);
                        if elapsed.num_seconds() > timeout_secs as i64 {
                            pending.remove(&key);

                            let _ = self.state_manager.load_or_default().and_then(|mut state| {
                                self.state_manager
                                    .remove_workspace_pending_question(&mut state, workspace_id, loop_id)
                            });
                        }
                        None
                    }
                } else {
                    None
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // Notifier was dropped without sending — check pending
                let mut pending = self.pending.lock().unwrap();
                if let Some(q) = pending.get_mut(&key) {
                    if let Some(ref response) = q.response {
                        let response = response.clone();
                        pending.remove(&key);

                        let _ = self.state_manager.load_or_default().and_then(|mut state| {
                            self.state_manager
                                .remove_workspace_pending_question(&mut state, workspace_id, loop_id)
                        });

                        Some(response)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    /// List all pending questions.
    pub fn list_pending(&self) -> Vec<PendingQuestionSummary> {
        let pending = self.pending.lock().unwrap();
        pending
            .values()
            .filter(|q| q.response.is_none())
            .map(|q| PendingQuestionSummary {
                workspace_id: q.workspace_id.clone(),
                loop_id: q.loop_id.clone(),
                question: q.question.clone(),
                asked_at: q.asked_at,
            })
            .collect()
    }

    /// Return the count of pending questions.
    pub fn pending_count(&self) -> usize {
        let pending = self.pending.lock().unwrap();
        pending.values().filter(|q| q.response.is_none()).count()
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

        // Notify any waiting get_response caller so it can exit
        let mut notifiers = self.notifiers.lock().unwrap();
        if let Some(tx) = notifiers.remove(&(workspace_id.to_string(), loop_id.to_string())) {
            let _ = tx.send(String::new());
        }
        drop(notifiers);

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

    /// Return a reference to the state manager for the poller to use.
    pub fn state_manager(&self) -> &StateManager {
        &self.state_manager
    }

    /// Persist chat_id from an incoming Telegram message.
    pub fn maybe_persist_chat_id(&self, chat_id: i64) {
        let _ = self.state_manager.load_or_default().and_then(|mut state| {
            if state.chat_id.is_none() {
                state.chat_id = Some(chat_id);
                self.state_manager.save(&state)?;
            }
            Ok(())
        });
    }
}

/// Summary of a pending question for listing.
#[derive(Debug, Clone, Serialize)]
pub struct PendingQuestionSummary {
    pub workspace_id: String,
    pub loop_id: String,
    pub question: String,
    pub asked_at: DateTime<Utc>,
}
