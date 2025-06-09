use eframe::egui;

/// Common UI components for iTools

/// Render a status badge
pub fn status_badge(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    ui.colored_label(color, format!("● {}", text));
}

/// Render a permission level indicator
pub fn permission_level_indicator(ui: &mut egui::Ui, level: &crate::state::PermissionLevel) {
    let (text, color) = match level {
        crate::state::PermissionLevel::Low => ("低", egui::Color32::from_rgb(100, 255, 100)),
        crate::state::PermissionLevel::Medium => ("中", egui::Color32::from_rgb(255, 200, 100)),
        crate::state::PermissionLevel::High => ("高", egui::Color32::from_rgb(255, 150, 100)),
        crate::state::PermissionLevel::Critical => ("严重", egui::Color32::from_rgb(255, 100, 100)),
    };
    
    ui.colored_label(color, text);
}

/// Render a plugin status indicator
pub fn plugin_status_indicator(ui: &mut egui::Ui, status: &crate::plugins::PluginStatus) {
    let (text, color) = match status {
        crate::plugins::PluginStatus::NotInstalled => ("未安装", egui::Color32::GRAY),
        crate::plugins::PluginStatus::Installing => ("安装中", egui::Color32::from_rgb(255, 200, 100)),
        crate::plugins::PluginStatus::Installed => ("已安装", egui::Color32::from_rgb(150, 150, 255)),
        crate::plugins::PluginStatus::Enabled => ("已启用", egui::Color32::from_rgb(100, 255, 100)),
        crate::plugins::PluginStatus::Disabled => ("已禁用", egui::Color32::GRAY),
        crate::plugins::PluginStatus::Updating => ("更新中", egui::Color32::from_rgb(255, 200, 100)),
        crate::plugins::PluginStatus::Error(_) => ("错误", egui::Color32::from_rgb(255, 100, 100)),
        crate::plugins::PluginStatus::Uninstalling => ("卸载中", egui::Color32::from_rgb(255, 200, 100)),
    };
    
    status_badge(ui, text, color);
}

/// Render a confirmation dialog
pub fn confirmation_dialog(
    ui: &mut egui::Ui,
    title: &str,
    message: &str,
    on_confirm: impl FnOnce(),
    on_cancel: impl FnOnce(),
) {
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.label(message);
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                if ui.button("确认").clicked() {
                    on_confirm();
                }
                
                if ui.button("取消").clicked() {
                    on_cancel();
                }
            });
        });
}

/// Render a loading spinner with text
pub fn loading_spinner(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.add(egui::Spinner::new());
        ui.label(text);
    });
}

/// Render a progress bar with text
pub fn progress_bar(ui: &mut egui::Ui, progress: f32, text: &str) {
    ui.add(egui::ProgressBar::new(progress).text(text));
}

/// Render an info box
pub fn info_box(ui: &mut egui::Ui, title: &str, content: &str, icon: &str) {
    egui::Frame::NONE
        .fill(ui.style().visuals.faint_bg_color)
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .corner_radius(egui::Rounding::same(5))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(20.0));
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(title).strong());
                    ui.label(content);
                });
            });
        });
}

/// Render a warning box
pub fn warning_box(ui: &mut egui::Ui, title: &str, content: &str) {
    egui::Frame::NONE
        .fill(egui::Color32::from_rgba_unmultiplied(255, 200, 100, 30))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 200, 100)))
        .corner_radius(egui::Rounding::same(5))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠").size(20.0).color(egui::Color32::from_rgb(255, 200, 100)));
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 200, 100), egui::RichText::new(title).strong());
                    ui.label(content);
                });
            });
        });
}

/// Render an error box
pub fn error_box(ui: &mut egui::Ui, title: &str, content: &str) {
    egui::Frame::NONE
        .fill(egui::Color32::from_rgba_unmultiplied(255, 100, 100, 30))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 100, 100)))
        .corner_radius(egui::Rounding::same(5))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("❌").size(20.0));
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), egui::RichText::new(title).strong());
                    ui.label(content);
                });
            });
        });
}

/// Render a success box
pub fn success_box(ui: &mut egui::Ui, title: &str, content: &str) {
    egui::Frame::NONE
        .fill(egui::Color32::from_rgba_unmultiplied(100, 255, 100, 30))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 255, 100)))
        .corner_radius(egui::Rounding::same(5))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("✅").size(20.0));
                ui.vertical(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), egui::RichText::new(title).strong());
                    ui.label(content);
                });
            });
        });
}

/// Render a collapsible section
pub fn collapsible_section<R>(
    ui: &mut egui::Ui,
    title: &str,
    default_open: bool,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::CollapsingResponse<R> {
    egui::CollapsingHeader::new(title)
        .default_open(default_open)
        .show(ui, add_contents)
}

/// Render a card container
pub fn card<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Frame::NONE
        .fill(ui.style().visuals.faint_bg_color)
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .corner_radius(egui::Rounding::same(5))
        .inner_margin(egui::Margin::same(15))
        .show(ui, add_contents)
}

/// Render a metric display
pub fn metric_display(ui: &mut egui::Ui, label: &str, value: &str, icon: Option<&str>) {
    ui.horizontal(|ui| {
        if let Some(icon) = icon {
            ui.label(egui::RichText::new(icon).size(16.0));
        }
        ui.label(format!("{}: ", label));
        ui.label(egui::RichText::new(value).strong());
    });
}

/// Render a tag
pub fn tag(ui: &mut egui::Ui, text: &str, color: Option<egui::Color32>) {
    let bg_color = color.unwrap_or(ui.style().visuals.selection.bg_fill);
    
    egui::Frame::NONE
        .fill(bg_color)
        .corner_radius(egui::Rounding::same(3))
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(12.0));
        });
}
