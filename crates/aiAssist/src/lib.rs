pub mod state;
pub mod ui;
pub mod models;
pub mod api;
pub mod mcp_tools;
pub mod mcp_integration;
pub mod settings_ui;



use eframe::egui;
use state::AIAssistState;

// Re-export types that are needed by the app
pub use state::{
    SlashCommand, TerminalCommand, NoteCommand, EditorCommand,
    TerminalContext, NoteContext, FileContext, ChatMessage, MessageRole
};

/// Render the AI assistant
pub fn render_ai_assist(ui: &mut egui::Ui, state: &mut AIAssistState) {
    ui::render_ai_assist(ui, state);
}

/// Initialize the AI assistant state
pub fn initialize() -> AIAssistState {
    let mut state = AIAssistState::default();

    // Load settings
    if let Err(err) = load_settings(&mut state) {
        log::warn!("Failed to load AI assistant settings: {}", err);
    }

    state
}

/// Create a settings module for AI assistant
pub fn create_settings_module(state: &mut AIAssistState) -> settings_ui::AIAssistSettingsModule {
    settings_ui::AIAssistSettingsModule::new(state)
}

/// Set the slash command callback
pub fn set_slash_command_callback<F>(state: &mut AIAssistState, callback: F)
where
    F: FnMut(SlashCommand) + Send + 'static,
{
    state.set_slash_command_callback(callback);
}

/// Set the insert to note callback
pub fn set_insert_to_note_callback<F>(state: &mut AIAssistState, callback: F)
where
    F: FnMut(String) + Send + 'static,
{
    state.set_insert_to_note_callback(callback);
}

/// Set the MCP refresh callback
pub fn set_mcp_refresh_callback<F>(state: &mut AIAssistState, callback: F)
where
    F: FnMut() + Send + 'static,
{
    state.set_mcp_refresh_callback(callback);
}

/// Add a search result reference to the current chat
pub fn add_search_reference(state: &mut AIAssistState, query: &str, result_count: usize) {
    state.add_search_reference(query, result_count);
}

/// Set search results for @search references
pub fn set_search_results(state: &mut AIAssistState, results: String) {
    state.set_search_results(results);
}

/// Update whether the user can insert to note
pub fn update_can_insert_to_note(state: &mut AIAssistState, can_insert: bool) {
    state.update_can_insert_to_note(can_insert);
}

/// Set terminal context
pub fn set_terminal_context(state: &mut AIAssistState, context: TerminalContext) {
    state.set_terminal_context(context);
}

/// Update terminal output
pub fn update_terminal_output(state: &mut AIAssistState, output: String) {
    state.update_terminal_output(output);
}



/// Update note context
pub fn update_note_context(state: &mut AIAssistState, title: String, content: String) {
    state.update_note_context(title, content);
}

/// Clear note context
pub fn clear_note_context(state: &mut AIAssistState) {
    state.clear_note_context();
}

/// Update file context
pub fn update_file_context(state: &mut AIAssistState, file_name: String, content: String) {
    state.update_file_context(file_name, content);
}

/// Clear file context
pub fn clear_file_context(state: &mut AIAssistState) {
    state.clear_file_context();
}



/// Process @ references in content
pub fn process_at_references(state: &AIAssistState, content: &str) -> String {
    state.process_at_references(content)
}



/// Set note context
pub fn set_note_context(state: &mut AIAssistState, context: NoteContext) {
    state.set_note_context(context);
}

/// Update current note
pub fn update_current_note(state: &mut AIAssistState, note_id: String, title: String, content: String) {
    state.update_current_note(note_id, title, content);
}

/// Update note search results
pub fn update_note_search_results(state: &mut AIAssistState, query: String, results: Vec<String>) {
    state.update_note_search_results(query, results);
}

/// Set file context
pub fn set_file_context(state: &mut AIAssistState, context: Option<FileContext>) {
    state.current_file_context = context;
}

/// Save AI assistant settings
pub fn save_settings(state: &AIAssistState) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use serde_json;

    let base_path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_dir = base_path.join("seeu_desktop");
    let config_path = config_dir.join("ai_settings.json");

    fs::create_dir_all(&config_dir)?;

    let settings = serde_json::json!({
        "base_url": state.ai_settings.base_url,
        "api_key": state.ai_settings.api_key,
        "model": state.ai_settings.model,
        "temperature": state.ai_settings.temperature,
        "max_tokens": state.ai_settings.max_tokens,
        "streaming": state.ai_settings.streaming,
        "show_api_key_masked": state.show_api_key_masked
    });

    let json = serde_json::to_string_pretty(&settings)?;
    fs::write(config_path, json)?;

    log::info!("AI assistant settings saved successfully");
    Ok(())
}

/// Save AI assistant chat sessions
pub fn save_chat_sessions(state: &AIAssistState) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use serde_json;

    let base_path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let data_dir = base_path.join("seeu_desktop").join("ai_assistant");
    let sessions_path = data_dir.join("chat_sessions.json");

    fs::create_dir_all(&data_dir)?;

    let sessions_data = serde_json::json!({
        "sessions": state.chat_sessions,
        "active_session_idx": state.active_session_idx
    });

    let json = serde_json::to_string_pretty(&sessions_data)?;
    fs::write(sessions_path, json)?;

    log::info!("AI assistant chat sessions saved successfully");
    Ok(())
}

/// Load AI assistant settings
pub fn load_settings(state: &mut AIAssistState) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use serde_json;

    let base_path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_path = base_path.join("seeu_desktop").join("ai_settings.json");

    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(value) = settings.get("base_url").and_then(|v| v.as_str()) {
                state.ai_settings.base_url = value.to_string();
            }
            if let Some(value) = settings.get("api_key").and_then(|v| v.as_str()) {
                state.ai_settings.api_key = value.to_string();
            }
            if let Some(value) = settings.get("model").and_then(|v| v.as_str()) {
                state.ai_settings.model = value.to_string();
            }
            if let Some(value) = settings.get("temperature").and_then(|v| v.as_f64()) {
                state.ai_settings.temperature = value as f32;
            }
            if let Some(value) = settings.get("max_tokens").and_then(|v| v.as_u64()) {
                state.ai_settings.max_tokens = value as u32;
            }
            if let Some(value) = settings.get("streaming").and_then(|v| v.as_bool()) {
                state.ai_settings.streaming = value;
            }
            if let Some(value) = settings.get("show_api_key_masked").and_then(|v| v.as_bool()) {
                state.show_api_key_masked = value;
            }

            log::info!("AI assistant settings loaded successfully");
        }
    }

    Ok(())
}

/// Load AI assistant chat sessions (fast mode for startup)
pub fn load_chat_sessions(state: &mut AIAssistState) -> Result<(), Box<dyn std::error::Error>> {
    // For fast startup, just load basic settings and defer session loading
    log::info!("AI assistant initialized (sessions will be loaded on demand)");
    Ok(())
}

/// Load AI assistant chat sessions (full mode when actually needed)
pub fn load_chat_sessions_full(state: &mut AIAssistState) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use serde_json;
    use crate::state::ChatSession;

    let base_path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let data_dir = base_path.join("seeu_desktop").join("ai_assistant");
    let sessions_path = data_dir.join("chat_sessions.json");

    if let Ok(content) = fs::read_to_string(sessions_path) {
        if let Ok(sessions_data) = serde_json::from_str::<serde_json::Value>(&content) {
            // Load sessions
            if let Some(sessions_array) = sessions_data.get("sessions").and_then(|v| v.as_array()) {
                let mut loaded_sessions = Vec::new();

                for session_value in sessions_array {
                    if let Ok(session) = serde_json::from_value::<ChatSession>(session_value.clone()) {
                        loaded_sessions.push(session);
                    }
                }

                // Only replace sessions if we successfully loaded some
                if !loaded_sessions.is_empty() {
                    state.chat_sessions = loaded_sessions;

                    // Load active session index
                    if let Some(active_idx) = sessions_data.get("active_session_idx").and_then(|v| v.as_u64()) {
                        let idx = active_idx as usize;
                        if idx < state.chat_sessions.len() {
                            state.active_session_idx = idx;
                        }
                    }

                    // Update current chat messages to match active session
                    if let Some(active_session) = state.chat_sessions.get(state.active_session_idx) {
                        state.chat_messages = active_session.messages.clone();
                    }

                    log::info!("AI assistant chat sessions loaded successfully: {} sessions", state.chat_sessions.len());
                }
            }
        }
    } else {
        log::info!("No existing chat sessions found, using default session");
    }

    Ok(())
}
