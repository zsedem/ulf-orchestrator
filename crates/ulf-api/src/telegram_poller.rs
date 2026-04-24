//! Daemon-side Telegram poller for multi-workspace human-in-the-loop.
//!
//! This is the sole entity that polls Telegram getUpdates. It routes
//! human replies to the correct workspace loop via HumanDomain, and
//! writes guidance events to workspace events files.

use std::sync::Arc;
use std::time::Duration;

use tracing::{info, warn};

use crate::human_domain::HumanDomain;
use crate::workspace_domain::WorkspaceDomain;

/// Spawns the daemon Telegram poller if RObot is configured.
///
/// Returns immediately if `bot_token` is None (RObot not configured).
/// Otherwise spawns a background tokio task that continuously polls
/// Telegram and routes messages.
pub fn spawn_if_configured(
    bot_token: Option<String>,
    api_url: Option<String>,
    human_domain: Arc<HumanDomain>,
    workspace_domain: Arc<std::sync::Mutex<WorkspaceDomain>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
) {
    let Some(token) = bot_token else {
        info!("RObot not configured — skipping Telegram poller");
        return;
    };

    tokio::spawn(async move {
        run_poller(token, api_url, human_domain, workspace_domain, shutdown).await;
    });

    info!("Daemon Telegram poller spawned");
}

async fn run_poller(
    bot_token: String,
    api_url: Option<String>,
    human_domain: Arc<HumanDomain>,
    workspace_domain: Arc<std::sync::Mutex<WorkspaceDomain>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
) {
    use teloxide::payloads::{GetUpdatesSetters, SetMessageReactionSetters};
    use teloxide::requests::Requester;

    let bot = ulf_telegram::apply_api_url(teloxide::Bot::new(&bot_token), api_url.as_deref());
    let mut offset: i32 = 0;

    // Register bot commands
    let commands = vec![
        teloxide::types::BotCommand::new("status", "Current loop status"),
        teloxide::types::BotCommand::new("tasks", "Open tasks"),
        teloxide::types::BotCommand::new("pending", "Pending human questions"),
        teloxide::types::BotCommand::new("help", "List available commands"),
    ];
    let _ = bot.set_my_commands(commands).await;

    info!("Daemon Telegram poller started");

    while !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
        let request = bot.get_updates().offset(offset).timeout(10);
        match request.await {
            Ok(updates) => {
                for update in updates {
                    #[allow(clippy::cast_possible_wrap)]
                    {
                        offset = update.id.0 as i32 + 1;
                    }

                    let msg = match update.kind {
                        teloxide::types::UpdateKind::Message(msg) => msg,
                        _ => continue,
                    };

                    let text = match msg.text() {
                        Some(t) => t,
                        None => continue,
                    };

                    let chat_id = msg.chat.id.0;
                    let reply_to: Option<i32> = msg.reply_to_message().map(|r| r.id.0);

                    // Handle slash commands
                    if text.starts_with('/') {
                        let response = handle_command(text, &human_domain, &workspace_domain);
                        let _ = bot
                            .send_message(teloxide::types::ChatId(chat_id), response)
                            .await;
                        continue;
                    }

                    // Route reply to pending question
                    if let Some(reply_id) = reply_to {
                        if let Some((workspace_id, loop_id)) = human_domain.find_by_reply(reply_id) {
                            let _ = human_domain.resolve_response(&workspace_id, &loop_id, text);

                            // Acknowledge
                            let _ = bot
                                .set_message_reaction(
                                    teloxide::types::ChatId(chat_id),
                                    msg.id,
                                )
                                .reaction(vec![teloxide::types::ReactionType::Emoji {
                                    emoji: "👍".to_string(),
                                }])
                                .await;
                            continue;
                        }
                    }

                    // Check @workspace-id prefix for guidance
                    if let Some(rest) = text.strip_prefix('@') {
                        if let Some(ws_id) = rest.split_whitespace().next() {
                            let guidance = rest[ws_id.len()..].trim();
                            if write_guidance_event(ws_id, guidance).is_ok() {
                                let _ = bot
                                    .send_message(
                                        teloxide::types::ChatId(chat_id),
                                        format!("📝 Guidance received for {ws_id}"),
                                    )
                                    .await;
                            }
                            continue;
                        }
                    }

                    // Default: if exactly one pending question globally, treat as response
                    let pending = human_domain.list_pending();
                    if pending.len() == 1 {
                        let q = &pending[0];
                        let _ = human_domain.resolve_response(&q.workspace_id, &q.loop_id, text);
                        let _ = bot
                            .set_message_reaction(
                                teloxide::types::ChatId(chat_id),
                                msg.id,
                            )
                            .reaction(vec![teloxide::types::ReactionType::Emoji {
                                emoji: "👍".to_string(),
                            }])
                            .await;
                        continue;
                    }

                    // Ambiguous — send help
                    let _ = bot
                        .send_message(
                            teloxide::types::ChatId(chat_id),
                            "I see multiple pending questions. Reply to the specific question message, or use @workspace-id prefix.\nUse /pending to see all pending questions.",
                        )
                        .await;
                }
            }
            Err(e) => {
                if !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                    warn!(error = %e, "Telegram polling error — retrying in 5s");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    info!("Daemon Telegram poller stopped");
}

fn handle_command(
    text: &str,
    human_domain: &HumanDomain,
    _workspace_domain: &Arc<std::sync::Mutex<WorkspaceDomain>>,
) -> String {
    let parts: Vec<&str> = text.split_whitespace().collect();
    let cmd = parts.first().copied().unwrap_or("");

    match cmd {
        "/pending" => {
            let pending = human_domain.list_pending();
            if pending.is_empty() {
                "No pending questions. 🎉".to_string()
            } else {
                let mut lines = vec![format!("Pending questions ({}):", pending.len())];
                for q in &pending {
                    lines.push(format!(
                        "• [{} → {}]: {}",
                        q.workspace_id, q.loop_id, q.question
                    ));
                }
                lines.join("\n")
            }
        }
        "/status" => {
            let count = human_domain.pending_count();
            format!("Daemon online. {} pending question(s).", count)
        }
        "/help" => {
            "Commands:\n/status — daemon status\n/pending — list pending questions\n/help — this message".to_string()
        }
        _ => "Unknown command. Use /help.".to_string(),
    }
}

/// Write a human.guidance event to a workspace's events file.
fn write_guidance_event(workspace_id: &str, guidance: &str) -> std::io::Result<()> {
    use std::io::Write;

    let workspace_path = std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".ulf")
        .join("workspaces")
        .join(workspace_id);

    let events_path = workspace_path.join(".ulf").join("events.jsonl");

    if let Some(parent) = events_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let event = serde_json::json!({
        "topic": "human.guidance",
        "payload": guidance,
        "ts": chrono::Utc::now().to_rfc3339(),
    });

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&events_path)?;

    writeln!(file, "{}", serde_json::to_string(&event)?)?;
    Ok(())
}
