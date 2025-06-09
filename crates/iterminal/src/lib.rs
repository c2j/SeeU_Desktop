pub mod state;
pub mod ui;
pub mod terminal;
pub mod command;
pub mod history;
pub mod config;
pub mod session;

use eframe::egui;

pub use state::ITerminalState;

/// Initialize the iTerminal module
pub fn initialize() -> ITerminalState {
    log::info!("Initializing iTerminal module");
    ITerminalState::new()
}

/// Main render function for iTerminal module
pub fn render_iterminal(ui: &mut egui::Ui, state: &mut ITerminalState) {
    ui::render_terminal_interface(ui, state);
}

/// Update function for background tasks
pub fn update_iterminal(state: &mut ITerminalState) {
    // Handle background tasks like command execution, output processing, etc.
    state.update();
}
