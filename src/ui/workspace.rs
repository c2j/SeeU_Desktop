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
                    Module::FileEditor => "文件编辑器",
                    Module::DataAnalysis => "数据分析",
                    Module::Note => "笔记",
                    Module::Search => "搜索",
                    Module::ITools => "iTools - AI 工具集成",
                    Module::Browser => "浏览器",
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
                        Module::FileEditor => {
                            ifile_editor::render_file_editor(ui, &mut app.ifile_editor_state);
                        },
                        Module::DataAnalysis => render_data_analysis(ui),
                        Module::Note => {
                            // 传递右侧边栏状态、宽度和字体设置给笔记模块
                            let font_family = Some(app.app_settings.font_family.as_str());
                            inote::render_db_inote_with_sidebar_info(ui, &mut app.inote_state, app.show_right_sidebar, right_sidebar_width, font_family);

                            // 渲染思源笔记导入对话框
                            inote::db_ui_import::render_siyuan_import_dialog(ui, &mut app.inote_state);
                        },
                        Module::Search => {
                            // 创建文件编辑器回调
                            let open_in_editor_callback = {
                                let app_ptr = app as *mut crate::app::SeeUApp;
                                move |file_path: String| {
                                    unsafe {
                                        (*app_ptr).open_file_in_editor(file_path);
                                    }
                                }
                            };

                            isearch::ui::render_isearch_with_sidebar_info_and_editor(
                                ui,
                                &mut app.isearch_state,
                                app.show_right_sidebar,
                                right_sidebar_width,
                                Some(open_in_editor_callback)
                            );

                            // 渲染文档导入对话框
                            if app.isearch_state.show_document_import_dialog {
                                log::info!("Rendering document import dialog");
                                render_document_import_dialog(ui, app);
                            }
                        },
                        Module::ITools => {
                            itools::render_itools(ui, &mut app.itools_state);
                        },
                        Module::Browser => {
                            ibrowser::render_ibrowser(ui, &mut app.ibrowser_state);
                        },
                        Module::Settings => {
                            crate::ui::modular_settings::render_modular_settings(ui, app);

                            // 在设置页面也需要渲染思源笔记导入对话框
                            inote::db_ui_import::render_siyuan_import_dialog(ui, &mut app.inote_state);
                        },
                    }
                });
        });
}

/// Render document import dialog
fn render_document_import_dialog(ui: &mut egui::Ui, app: &mut crate::app::SeeUApp) {
    let ctx = ui.ctx();

    // Clone necessary data to avoid borrowing issues
    let file_path = app.isearch_state.import_file_path.clone();
    let file_name = app.isearch_state.import_file_name.clone();
    let notebooks = app.inote_state.notebooks.clone();

    // Initialize the notebook selector dialog if it's not already shown
    if !app.inote_state.notebook_selector.show_dialog {
        app.inote_state.notebook_selector.show_for_file(file_path.clone(), file_name);
    }

    // Show notebook selector dialog and check if an action was selected
    if let Some(import_action) = inote::notebook_selector::render_notebook_selector_dialog(
        ctx,
        &mut app.inote_state.notebook_selector,
        &notebooks,
    ) {
        let (selected_notebook_id, should_edit) = match import_action {
            inote::notebook_selector::ImportAction::Import(notebook_id) => (notebook_id, false),
            inote::notebook_selector::ImportAction::ImportAndEdit(notebook_id) => (notebook_id, true),
        };

        // Import the document
        match app.inote_state.import_document_as_note(&file_path, &selected_notebook_id) {
            Ok(note_id) => {
                log::info!("Successfully imported document as note: {}", note_id);

                // If "Import and Edit" was selected, switch to note editing
                if should_edit {
                    // Switch to note module
                    app.active_module = crate::app::Module::Note;

                    // Select the imported note for editing
                    app.inote_state.select_note(&note_id);
                    log::info!("Switched to editing imported note: {}", note_id);
                }

                // Reset the isearch dialog state
                app.isearch_state.reset_document_import_dialog();

                // Mark import as successful
                app.inote_state.notebook_selector.import_success = true;
                app.inote_state.notebook_selector.import_in_progress = false;
            },
            Err(error) => {
                log::error!("Failed to import document: {}", error);
                app.inote_state.notebook_selector.import_error = Some(error);
                app.inote_state.notebook_selector.import_in_progress = false;
            }
        }
    }

    // If the notebook selector dialog was closed, also close the isearch dialog
    if !app.inote_state.notebook_selector.show_dialog {
        app.isearch_state.reset_document_import_dialog();
    }
}