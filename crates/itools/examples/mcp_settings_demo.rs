use eframe::egui;
use itools::ui::mcp_settings::McpSettingsUi;

/// Demo application for MCP Settings UI
struct McpSettingsDemo {
    mcp_settings: McpSettingsUi,
}

impl McpSettingsDemo {
    fn new() -> Self {
        // Create a temporary config path for demo
        let config_path = std::env::temp_dir().join("mcp_demo_config.json");
        let mcp_settings = McpSettingsUi::new(config_path);

        Self {
            mcp_settings,
        }
    }
}

impl eframe::App for McpSettingsDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MCP Settings Demo");
            ui.separator();
            
            // Render the MCP settings UI
            self.mcp_settings.render(ctx, ui);
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Initialize logging

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("MCP Settings Demo"),
        ..Default::default()
    };

    eframe::run_native(
        "MCP Settings Demo",
        options,
        Box::new(|_cc| Ok(Box::new(McpSettingsDemo::new()))),
    )
}
