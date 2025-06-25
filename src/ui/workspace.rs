use eframe::egui;
use crate::app::Module;
use crate::modules::{
    home::render_home,
    file_manager::render_file_manager,
    data_analysis::render_data_analysis,
};
use inote;
use isearch;
use itools;
use iterminal;

/// Render the main workspace area
pub fn render_workspace(ui: &mut egui::Ui, active_module: &Module, app: &mut crate::app::SeeUApp, right_sidebar_width: Option<f32>) {
    // 创建一个垂直布局容器
    egui::containers::Frame::none()
        .fill(ui.style().visuals.window_fill)
        .show(ui, |ui| {

            // Add a heading for the current module (except for Home which has its own layout)
            if *active_module != Module::Home {
                ui.heading(match active_module {
                    Module::Home => "主页", // This won't be shown due to the if condition
                    Module::Terminal => "终端",
                    Module::Files => "文件管理",
                    Module::DataAnalysis => "数据分析",
                    Module::Note => "笔记",
                    Module::Search => "搜索",
                    Module::ITools => "iTools - AI 工具集成",
                    Module::Settings => "设置",
                });

                ui.separator();
            }

            // 创建一个内容容器
            egui::containers::Frame::none()
                .fill(ui.style().visuals.window_fill)
                .show(ui, |ui| {

                    // Render the active module
                    match active_module {
                        Module::Home => render_home(ui, app),
                        Module::Terminal => {
                            iterminal::render_iterminal(ui, &mut app.iterminal_state);
                        },
                        Module::Files => render_file_manager(ui),
                        Module::DataAnalysis => render_data_analysis(ui),
                        Module::Note => {
                            // 传递右侧边栏状态和宽度给笔记模块
                            inote::render_db_inote_with_sidebar_info(ui, &mut app.inote_state, app.show_right_sidebar, right_sidebar_width);

                            // 渲染思源笔记导入对话框
                            inote::db_ui_import::render_siyuan_import_dialog(ui, &mut app.inote_state);
                        },
                        Module::Search => {
                            isearch::render_isearch_with_sidebar_info(ui, &mut app.isearch_state, app.show_right_sidebar, right_sidebar_width);
                        },
                        Module::ITools => {
                            itools::render_itools(ui, &mut app.itools_state);
                        },
                        Module::Settings => {
                            crate::ui::settings::render_settings(ui, app);

                            // 在设置页面也需要渲染思源笔记导入对话框
                            inote::db_ui_import::render_siyuan_import_dialog(ui, &mut app.inote_state);
                        },
                    }
                });
        });
}