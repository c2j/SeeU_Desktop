use eframe::egui;
use crate::app::{SeeUApp, Module};

/// Home page state
pub struct HomeState {
    // 可以添加一些状态，比如最近使用的功能等
}

impl Default for HomeState {
    fn default() -> Self {
        Self {}
    }
}

/// Render the home page
pub fn render_home(ui: &mut egui::Ui, app: &mut SeeUApp) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(20.0);

        // Check if we have search results to display
        if app.global_search_results.has_results {
            render_search_results_section(ui, app);
        } else {
            // 应用标题和欢迎信息
            ui.vertical_centered(|ui| {
                ui.heading("🌟 欢迎使用 SeeU Desktop");
                ui.add_space(8.0);
                ui.label("您的智能桌面助手，集成多种强大功能");
                ui.add_space(20.0);
            });

            // 主要功能区域
            render_main_features_section(ui, app);
        }

        ui.add_space(30.0);

        // 底部信息区域
        ui.separator();
        ui.add_space(15.0);

        ui.horizontal(|ui| {
            ui.label("💡 提示：");
            ui.label("使用顶部搜索栏可以快速搜索文件和内容");
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("🎯 快捷键：");
            ui.label("Ctrl+/ 打开命令面板，Ctrl+K 快速搜索");
        });

        ui.add_space(20.0);
    });
}

/// Render search results section
fn render_search_results_section(ui: &mut egui::Ui, app: &mut SeeUApp) {
    // Search results header
    ui.vertical_centered(|ui| {
        ui.heading(format!("🔍 搜索结果: \"{}\"", app.global_search_results.query));
        ui.add_space(8.0);

        let total_results = app.global_search_results.inote_results.len() +
                           app.global_search_results.itools_results.len() +
                           app.global_search_results.isearch_results.len();
        ui.label(format!("找到 {} 个结果", total_results));
        ui.add_space(20.0);
    });

    // Clear search results button
    ui.horizontal(|ui| {
        if ui.button("✖ 清除搜索结果").clicked() {
            app.global_search_results = crate::app::GlobalSearchResults::default();
            app.search_query.clear();
        }
        ui.add_space(10.0);
        ui.label("点击结果项可跳转到对应工作区");
    });

    ui.add_space(15.0);

    // Render results by category
    ui.columns(1, |columns| {
        columns[0].group(|ui| {
            ui.vertical(|ui| {
                // iNote results
                if !app.global_search_results.inote_results.is_empty() {
                    render_inote_results(ui, app);
                    ui.add_space(15.0);
                }

                // iTools results
                if !app.global_search_results.itools_results.is_empty() {
                    render_itools_results(ui, app);
                    ui.add_space(15.0);
                }

                // iSearch results
                if !app.global_search_results.isearch_results.is_empty() {
                    render_isearch_results(ui, app);
                }
            });
        });
    });
}

/// Render main features section (when no search results)
fn render_main_features_section(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.columns(2, |columns| {
        // 左列 - 核心功能
        columns[0].group(|ui| {
            ui.vertical(|ui| {
                ui.heading("🚀 核心功能");
                ui.add_space(10.0);

                // 笔记功能
                render_feature_card(ui, "📝", "智能笔记",
                    "创建、编辑和管理您的笔记，支持 Markdown 格式和富文本编辑",
                    Module::Note, app);

                ui.add_space(8.0);

                // 搜索功能
                render_feature_card(ui, "🔍", "全局搜索",
                    "快速搜索文件、笔记和内容，支持高级搜索语法",
                    Module::Search, app);

                ui.add_space(8.0);

                // iTools 功能
                render_feature_card(ui, "🔧", "AI 工具集",
                    "管理 AI 插件，扩展应用功能，支持 MCP 协议",
                    Module::ITools, app);
            });
        });

        // 右列 - 系统信息和快速操作
        columns[1].group(|ui| {
            ui.vertical(|ui| {
                ui.heading("📊 系统概览");
                ui.add_space(10.0);

                // 系统信息卡片
                render_system_info_card(ui, app);

                ui.add_space(15.0);

                ui.heading("⚡ 快速操作");
                ui.add_space(10.0);

                // 快速操作按钮
                render_quick_actions(ui, app);
            });
        });
    });
}

/// 渲染功能卡片
fn render_feature_card(ui: &mut egui::Ui, icon: &str, title: &str, description: &str, module: Module, app: &mut SeeUApp) {
    let card_frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12))
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color));

    card_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(icon).size(24.0));
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).heading().strong());
                ui.label(description);
                ui.add_space(4.0);
                if ui.small_button("打开").clicked() {
                    app.active_module = module;
                }
            });
        });
    });
}

/// 渲染系统信息卡片
fn render_system_info_card(ui: &mut egui::Ui, app: &SeeUApp) {
    let card_frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12))
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color));

    card_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // CPU 使用率
            ui.horizontal(|ui| {
                ui.label("🖥️ CPU:");
                ui.label(format!("{:.1}%", app.system_service.get_cpu_usage()));
            });

            ui.add_space(4.0);

            // 内存使用率
            ui.horizontal(|ui| {
                ui.label("💾 内存:");
                ui.label(format!("{:.1}%", app.system_service.get_memory_usage()));
            });

            ui.add_space(4.0);

            // 笔记统计
            ui.horizontal(|ui| {
                ui.label("📝 笔记:");
                ui.label(format!("{} 个", app.inote_state.notes.len()));
            });

            ui.add_space(4.0);

            // 搜索索引状态
            ui.horizontal(|ui| {
                ui.label("🔍 索引:");
                let indexed_count = app.isearch_state.indexed_directories.len();
                ui.label(format!("{} 个目录", indexed_count));
            });
        });
    });
}

/// 渲染快速操作区域
fn render_quick_actions(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.vertical(|ui| {
        // 创建新笔记
        if ui.button("📝 创建新笔记").clicked() {
            app.active_module = Module::Note;
            // 如果有笔记本，创建新笔记
            if !app.inote_state.notebooks.is_empty() {
                // 如果没有选中笔记本，选择第一个
                if app.inote_state.current_notebook.is_none() {
                    app.inote_state.select_notebook(0);
                }
                // 创建新笔记
                let note_id = app.inote_state.create_note(
                    "新笔记".to_string(),
                    "".to_string()
                );
                // 如果创建成功，选择这个笔记
                if let Some(note_id) = note_id {
                    app.inote_state.select_note(&note_id);
                }
            }
        }

        ui.add_space(4.0);

        // 打开搜索
        if ui.button("🔍 开始搜索").clicked() {
            app.active_module = Module::Search;
        }

        ui.add_space(4.0);

        // 管理插件
        if ui.button("🔧 管理插件").clicked() {
            app.active_module = Module::ITools;
        }

        ui.add_space(4.0);

        // 系统设置
        if ui.button("⚙️ 系统设置").clicked() {
            app.show_right_sidebar = !app.show_right_sidebar;
        }
    });
}

/// Render iNote search results
fn render_inote_results(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.heading("📝 笔记 (iNote)");
    ui.add_space(8.0);

    for result in &app.global_search_results.inote_results {
        let card_frame = egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(10))
            .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color));

        card_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("📝");
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    if ui.link(&result.title).clicked() {
                        // Switch to Note module and select the note
                        app.active_module = Module::Note;
                        app.inote_state.select_note(&result.id);
                    }
                    ui.label(format!("笔记本: {}", result.notebook_name));
                    if !result.content_preview.is_empty() {
                        ui.label(egui::RichText::new(&result.content_preview).weak());
                    }
                });
            });
        });
        ui.add_space(4.0);
    }
}

/// Render iTools search results
fn render_itools_results(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.heading("🔧 AI 工具 (iTools)");
    ui.add_space(8.0);

    for result in &app.global_search_results.itools_results {
        let card_frame = egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(10))
            .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color));

        card_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("🔧");
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    if ui.link(&result.name).clicked() {
                        // Switch to ITools module
                        app.active_module = Module::ITools;
                        // Set the selected plugin in iTools state
                        if let Ok(plugin_id) = uuid::Uuid::parse_str(&result.id) {
                            app.itools_state.ui_state.selected_plugin = Some(plugin_id);
                        }
                    }
                    ui.label(format!("分类: {}", result.category));
                    ui.label(egui::RichText::new(&result.description).weak());
                });
            });
        });
        ui.add_space(4.0);
    }
}

/// Render iSearch search results
fn render_isearch_results(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.heading("🔍 文件搜索 (iSearch)");
    ui.add_space(8.0);

    for result in &app.global_search_results.isearch_results {
        let card_frame = egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(10))
            .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color));

        card_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // File type icon
                let icon = match result.file_type.as_str() {
                    "pdf" => "📄",
                    "doc" | "docx" => "📝",
                    "xls" | "xlsx" => "📊",
                    "ppt" | "pptx" => "📽",
                    "txt" | "md" => "📃",
                    "rs" | "js" | "py" | "cpp" => "💻",
                    "jpg" | "png" | "gif" => "🖼",
                    _ => "📁",
                };
                ui.label(icon);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    if ui.link(&result.filename).clicked() {
                        // Switch to Search module
                        app.active_module = Module::Search;
                    }
                    ui.label(egui::RichText::new(&result.path).weak());
                    if !result.content_preview.is_empty() {
                        ui.label(egui::RichText::new(&result.content_preview).weak());
                    }
                });
            });
        });
        ui.add_space(4.0);
    }
}
