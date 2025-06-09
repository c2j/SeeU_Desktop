use eframe::egui;

/// Data analysis module state
pub struct DataAnalysisState {
    data_source: String,
    chart_type: ChartType,
}

/// Chart type
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Scatter,
}

impl Default for DataAnalysisState {
    fn default() -> Self {
        Self {
            data_source: String::new(),
            chart_type: ChartType::Bar,
        }
    }
}

/// Render the data analysis module
pub fn render_data_analysis(ui: &mut egui::Ui) {
    // 获取可用高度
    let available_height = ui.available_height();

    // 创建一个垂直布局容器，确保内容撑满高度
    egui::containers::Frame::none()
        .fill(ui.style().visuals.window_fill)
        .show(ui, |ui| {
            // 设置最小高度，确保撑满可用空间
            ui.set_min_height(available_height);

            ui.vertical(|ui| {
                ui.label("数据分析模块尚未完全实现");

                // Data source selection
                ui.horizontal(|ui| {
                    ui.label("数据源:");
                    let mut data_source = String::new();
                    ui.add(
                        egui::TextEdit::singleline(&mut data_source)
                            .hint_text("选择数据文件...")
                            .desired_width(ui.available_width() - 100.0)
                    );

                    if ui.button("浏览").clicked() {
                        // TODO: Open file dialog
                        log::info!("Open file dialog for data source");
                    }
                });

                ui.separator();

                // Chart type selection
                ui.horizontal(|ui| {
                    ui.label("图表类型:");

                    ui.selectable_value(&mut ChartType::Bar, ChartType::Bar, "柱状图");
                    ui.selectable_value(&mut ChartType::Line, ChartType::Line, "折线图");
                    ui.selectable_value(&mut ChartType::Pie, ChartType::Pie, "饼图");
                    ui.selectable_value(&mut ChartType::Scatter, ChartType::Scatter, "散点图");
                });

                ui.separator();

                // 计算图表区域高度（减去标题、数据源选择、图表类型选择和分隔符的高度）
                let chart_area_height = available_height - 150.0;

                // 创建一个图表容器，确保图表区域撑满剩余高度
                egui::containers::Frame::none()
                    .fill(ui.style().visuals.window_fill)
                    .show(ui, |ui| {
                        // 设置最小高度，确保撑满剩余空间
                        ui.set_min_height(chart_area_height);

                        // Chart area
                        ui.add_space(20.0);
                        ui.centered_and_justified(|ui| {
                            ui.label("图表区域 - 选择数据源后显示图表");
                        });

                        // Draw a placeholder chart
                        let rect = ui.available_rect_before_wrap();
                        let painter = ui.painter();

                        // Draw chart background
                        painter.rect_filled(
                            rect,
                            0.0,
                            egui::Color32::from_rgb(240, 240, 240),
                        );

                        // Draw chart axes
                        let margin = 40.0;
                        let chart_rect = egui::Rect::from_min_max(
                            rect.min + egui::vec2(margin, margin),
                            rect.max - egui::vec2(margin, margin),
                        );

                        painter.line_segment(
                            [chart_rect.left_bottom(), chart_rect.right_bottom()],
                            (1.0, egui::Color32::BLACK),
                        );

                        painter.line_segment(
                            [chart_rect.left_bottom(), chart_rect.left_top()],
                            (1.0, egui::Color32::BLACK),
                        );

                        // Draw some sample bars
                        let bar_width = 30.0;
                        let bar_spacing = 20.0;
                        let bar_count = 5;
                        let total_width = bar_count as f32 * (bar_width + bar_spacing) - bar_spacing;
                        let start_x = chart_rect.center().x - total_width / 2.0;

                        let values = [0.3, 0.7, 0.5, 0.8, 0.4];

                        for (i, &value) in values.iter().enumerate() {
                            let bar_height = chart_rect.height() * value;
                            let bar_rect = egui::Rect::from_min_max(
                                egui::pos2(
                                    start_x + i as f32 * (bar_width + bar_spacing),
                                    chart_rect.bottom() - bar_height,
                                ),
                                egui::pos2(
                                    start_x + i as f32 * (bar_width + bar_spacing) + bar_width,
                                    chart_rect.bottom(),
                                ),
                            );

                            painter.rect_filled(
                                bar_rect,
                                3.0,
                                egui::Color32::from_rgb(66, 150, 250),
                            );
                        }
                    });
            });
        });
}