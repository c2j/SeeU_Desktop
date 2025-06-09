use eframe::egui;

pub mod state;
pub mod ui;
pub mod mcp;
pub mod plugins;
pub mod security;
pub mod roles;

pub use state::IToolsState;

/// Initialize the iTools module
pub fn initialize() -> IToolsState {
    log::info!("Initializing iTools module");
    IToolsState::new()
}

/// Main render function for iTools module
pub fn render_itools(ui: &mut egui::Ui, state: &mut IToolsState) {
    ui::main_ui::render_main_interface(ui, state);
}

/// Update function for background tasks
pub fn update_itools(state: &mut IToolsState) {
    // Handle background tasks like plugin updates, security checks, etc.
    state.update();
}
