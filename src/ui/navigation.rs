use eframe::egui;
use crate::app::Module;

/// Render the navigation panel
pub fn render_navigation(ui: &mut egui::Ui, active_module: &mut Module) {
    ui.vertical_centered(|ui| {
        ui.add_space(8.0);

        // Home button
        if ui.add(egui::Button::new("🏠")
            .selected(*active_module == Module::Home))
            .on_hover_text("首页")
            .clicked() {
            *active_module = Module::Home;
        }

        ui.add_space(4.0);

        // Terminal button
        if ui.add(egui::Button::new("🖥")
            .selected(*active_module == Module::Terminal))
            .on_hover_text("安全终端")
            .clicked() {
            *active_module = Module::Terminal;
        }

        ui.add_space(4.0);

        // Files button
        // TODO 后续再实现，暂时封禁
        // if ui.add(egui::Button::new("📁")
        //     .selected(*active_module == Module::Files))
        //     .clicked() {
        //     *active_module = Module::Files;
        // }

        // ui.add_space(4.0);

        // File Editor button
        if ui.add(egui::Button::new("📄")
            .selected(*active_module == Module::FileEditor))
            .on_hover_text("文件编辑器")
            .clicked() {
            *active_module = Module::FileEditor;
        }

        ui.add_space(4.0);

        // Data Analysis button
        // TODO 后续再实现，暂时封禁
        // if ui.add(egui::Button::new("📊")
        //     .selected(*active_module == Module::DataAnalysis))
        //     .clicked() {
        //     *active_module = Module::DataAnalysis;
        // }

        // ui.add_space(4.0);

        // iTools button
        if ui.add(egui::Button::new("🛠")
            .selected(*active_module == Module::ITools))
            .on_hover_text("智能工具")
            .clicked() {
            *active_module = Module::ITools;
        }

        ui.add_space(4.0);

        // Note button
        if ui.add(egui::Button::new("📝")
            .selected(*active_module == Module::Note))
            .on_hover_text("笔记管理")
            .clicked() {
            *active_module = Module::Note;
        }

        ui.add_space(4.0);

        // Search button
        if ui.add(egui::Button::new("🔍")
            .selected(*active_module == Module::Search))
            .on_hover_text("文件搜索")
            .clicked() {
            *active_module = Module::Search;
        }

        ui.add_space(4.0);

        // Settings button
        if ui.add(egui::Button::new("🔩")
            .selected(*active_module == Module::Settings))
            .on_hover_text("设置")
            .clicked() {
            *active_module = Module::Settings;
        }
    });
}