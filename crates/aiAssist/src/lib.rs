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
    AIAssistState::default()
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
