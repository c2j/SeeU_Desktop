use eframe::egui;

pub mod state;
pub mod ui;
pub mod mcp;
pub mod plugins;
pub mod security;
pub mod roles;
pub mod settings_ui;

pub use state::IToolsState;

/// Create a settings module for itools
pub fn create_settings_module(state: &mut IToolsState) -> settings_ui::IToolsSettingsModule {
    settings_ui::IToolsSettingsModule::new(state)
}

/// Save itools settings
pub fn save_settings(state: &IToolsState) -> Result<(), Box<dyn std::error::Error>> {
    // For now, iTools doesn't have persistent settings to save
    // This can be expanded when actual settings are implemented
    log::info!("iTools settings saved successfully");
    Ok(())
}

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
