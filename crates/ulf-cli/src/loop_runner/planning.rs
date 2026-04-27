use super::*;

/// Checks for planning session user responses and publishes them as events.
///
/// When running in planning mode (ULF_PLANNING_SESSION_ID is set),
/// this function reads the conversation file for new user responses and
/// publishes them as `user.response` events to the event loop.
pub(crate) fn check_planning_session_responses(event_loop: &mut EventLoop) -> Result<()> {
    // Get the planning session ID from environment
    let session_id = match std::env::var("ULF_PLANNING_SESSION_ID") {
        Ok(id) => id,
        Err(_) => return Ok(()), // Not in planning mode
    };
    check_planning_session_responses_for_session(event_loop, &session_id)
}

pub(crate) fn check_planning_session_responses_for_session(
    event_loop: &mut EventLoop,
    session_id: &str,
) -> Result<()> {
    // Get loop context to find the conversation file path
    let ctx = match event_loop.loop_context() {
        Some(ctx) => ctx,
        None => return Ok(()), // No context, can't find conversation file
    };

    let conversation_path = ctx.planning_conversation_path(session_id);


    // Read conversation entries and look for new responses
    // We track which response IDs we've already processed to avoid duplicates
    // Track processed response IDs (static to persist across iterations)
    pub(crate) static PROCESSED_RESPONSES: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

    let conversation_content = match fs::read_to_string(&conversation_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()), // File doesn't exist yet
        Err(e) => {
            warn!(
                session_id = %session_id,
                error = %e,
                "Failed to read planning conversation file"
            );
            return Ok(());
        }
    };

    let mut processed = PROCESSED_RESPONSES.lock().unwrap();

    for line in conversation_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse the conversation entry
        let entry: ulf_core::planning_session::ConversationEntry = match serde_json::from_str(line)
        {
            Ok(entry) => entry,
            Err(e) => {
                warn!(
                    session_id = %session_id,
                    line = %line,
                    error = %e,
                    "Failed to parse conversation entry"
                );
                continue;
            }
        };

        // Only process user_response entries
        if entry.entry_type != ulf_core::planning_session::ConversationType::UserResponse {
            continue;
        }

        // Check if we've already processed this response
        let response_key = format!("{}:{}", entry.id, entry.ts);
        if processed.contains(&response_key) {
            continue;
        }

        // Publish as user.response event
        let event = Event::new(
            "user.response",
            format!("[id: {}] {}", entry.id, entry.text),
        );
        event_loop.bus().publish(event.clone());

        info!(
            session_id = %session_id,
            response_id = %entry.id,
            "Published user response from planning session"
        );

        // Mark as processed
        processed.push(response_key);
    }

    Ok(())
}
