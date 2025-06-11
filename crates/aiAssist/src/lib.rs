pub mod state;
pub mod ui;
pub mod models;
pub mod api;

use eframe::egui;
use state::AIAssistState;

// Re-export types that are needed by the app
pub use state::SlashCommand;

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
        "streaming": state.ai_settings.streaming
    });

    let json = serde_json::to_string_pretty(&settings)?;
    fs::write(config_path, json)?;

    log::info!("AI assistant settings saved successfully");
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

            log::info!("AI assistant settings loaded successfully");
        }
    }

    Ok(())
}
