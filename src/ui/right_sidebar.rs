use eframe::egui;
use crate::app::SeeUApp;
use aiAssist;

/// Render the right sidebar
pub fn render_right_sidebar(ui: &mut egui::Ui, app: &mut SeeUApp) {
    // Header
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("🤖 AI助手").strong());
    });

    ui.separator();

    // AI Assistant content
    aiAssist::render_ai_assist(ui, &mut app.ai_assist_state);
}
