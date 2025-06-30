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
    // 确保数据已加载（用于正确显示统计信息）
    app.inote_state.ensure_data_loaded();

    // Check if we have search results to display
    if app.global_search_results.has_results {
        egui::ScrollArea::vertical().show(ui, |ui| {
            render_search_results_section(ui, app);
        });
    } else {
        // 使用豆腐块布局
        ui.vertical(|ui| {
            ui.add_space(12.0);

            // 标题栏
            render_header_section(ui);

            ui.add_space(16.0);

            // 豆腐块区域
            render_dashboard_blocks(ui, app);

            ui.add_space(12.0);
        });
    }
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

/// 渲染标题栏
fn render_header_section(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        // 主标题
        ui.label(egui::RichText::new("🌟 SeeU Desktop")
            .size(24.0)
            .strong()
            .color(ui.style().visuals.text_color()));

        ui.add_space(4.0);

        // 副标题
        ui.label(egui::RichText::new("您的智能桌面助手")
            .size(14.0)
            .color(ui.style().visuals.weak_text_color()));
    });
}

/// 渲染豆腐块仪表板
fn render_dashboard_blocks(ui: &mut egui::Ui, app: &mut SeeUApp) {
    // 使用网格布局，2列
    ui.columns(2, |columns| {
        // 左列
        columns[0].vertical(|ui| {
            // 笔记豆腐块
            render_notes_block(ui, app);
            ui.add_space(12.0);

            // MCP工具豆腐块
            render_mcp_tools_block(ui, app);
        });

        // 右列
        columns[1].vertical(|ui| {
            // 搜索豆腐块
            render_search_block(ui, app);
            ui.add_space(12.0);

            // 快速操作豆腐块
            render_quick_actions_block(ui, app);
        });
    });
}

/// 渲染笔记豆腐块
fn render_notes_block(ui: &mut egui::Ui, app: &mut SeeUApp) {
    let block_frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12.0))
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .fill(ui.style().visuals.widgets.noninteractive.weak_bg_fill);

    block_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // 标题栏
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📝").size(18.0));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("笔记").size(16.0).strong());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("管理").clicked() {
                        app.active_module = Module::Note;
                    }
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // 统计信息
            let notebook_count = app.inote_state.notebooks.len();
            let total_notes = app.inote_state.notebooks.iter()
                .map(|nb| nb.note_ids.len())
                .sum::<usize>();

            ui.horizontal(|ui| {
                ui.label(format!("📚 {} 个笔记本", notebook_count));
                ui.add_space(12.0);
                ui.label(format!("📄 {} 篇笔记", total_notes));
            });

            ui.add_space(8.0);

            // 快速操作
            ui.horizontal(|ui| {
                if ui.button("📝 新建笔记").clicked() {
                    app.active_module = Module::Note;
                    // 如果有笔记本，创建新笔记
                    if !app.inote_state.notebooks.is_empty() {
                        if app.inote_state.current_notebook.is_none() {
                            app.inote_state.select_notebook(0);
                        }
                        let note_id = app.inote_state.create_note(
                            "新笔记".to_string(),
                            "".to_string()
                        );
                        if let Some(note_id) = note_id {
                            app.inote_state.select_note(&note_id);
                        }
                    }
                }

                ui.add_space(4.0);

                if ui.button("📚 新建笔记本").clicked() {
                    app.active_module = Module::Note;
                    app.inote_state.create_notebook("新笔记本".to_string(), "".to_string());
                }
            });

            // 最近打开的笔记本（TOP3）
            if !app.inote_state.notebooks.is_empty() {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("最近笔记本").size(12.0).weak());
                ui.add_space(4.0);

                // 显示最多3个最近的笔记本
                let notebooks_info: Vec<_> = app.inote_state.notebooks.iter()
                    .take(3)
                    .map(|nb| (nb.id.clone(), nb.name.clone(), nb.note_ids.len()))
                    .collect();

                for (notebook_id, notebook_name, note_count) in notebooks_info {
                    ui.horizontal(|ui| {
                        if ui.small_button(&notebook_name).clicked() {
                            app.active_module = Module::Note;
                            // 选择笔记本
                            if let Some(index) = app.inote_state.notebooks.iter().position(|nb| nb.id == notebook_id) {
                                app.inote_state.select_notebook(index);
                            }
                        }
                        ui.label(egui::RichText::new(format!("({} 篇笔记)", note_count)).size(10.0).weak());
                    });
                }
            }

            // 最近打开的笔记（TOP5）
            let recent_notes = app.inote_state.get_recent_notes(5);

            if !recent_notes.is_empty() {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("最近笔记").size(12.0).weak());
                ui.add_space(4.0);

                for note in recent_notes {
                    ui.horizontal(|ui| {
                        // 笔记标题按钮
                        if ui.small_button(&note.note_title).clicked() {
                            app.active_module = Module::Note;
                            // 直接选择并编辑笔记
                            app.inote_state.select_note(&note.note_id);
                        }

                        // 显示访问时间
                        let time_str = note.accessed_at.format("%m-%d %H:%M").to_string();
                        ui.label(egui::RichText::new(format!("({})", time_str)).size(10.0).weak());
                    });
                }
            }
        });
    });
}

/// 渲染搜索豆腐块
fn render_search_block(ui: &mut egui::Ui, app: &mut SeeUApp) {
    let block_frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12.0))
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .fill(ui.style().visuals.widgets.noninteractive.weak_bg_fill);

    block_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // 标题栏
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔍").size(18.0));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("搜索").size(16.0).strong());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("高级").clicked() {
                        app.active_module = Module::Search;
                    }
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // 快速搜索框
            ui.horizontal(|ui| {
                let search_response = ui.text_edit_singleline(&mut app.search_query);
                if search_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !app.search_query.trim().is_empty() {
                        app.active_module = Module::Search;
                        // 这里可以触发搜索
                    }
                }

                if ui.button("🔍").clicked() && !app.search_query.trim().is_empty() {
                    app.active_module = Module::Search;
                    // 这里可以触发搜索
                }
            });

            ui.add_space(8.0);

            // 最近搜索（模拟数据）
            ui.label(egui::RichText::new("最近搜索").size(12.0).weak());
            ui.add_space(4.0);

            let recent_searches = vec!["Rust", "egui", "笔记", "MCP"];
            for search_term in recent_searches.iter().take(4) {
                ui.horizontal(|ui| {
                    if ui.small_button(*search_term).clicked() {
                        app.search_query = search_term.to_string();
                        app.active_module = Module::Search;
                    }
                });
            }
        });
    });
}

/// 渲染紧凑版功能卡片
fn render_feature_card_compact(ui: &mut egui::Ui, icon: &str, title: &str, description: &str, module: Module, app: &mut SeeUApp) {
    let is_current = app.active_module == module;

    let card_frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(8.0))
        .stroke(egui::Stroke::new(
            if is_current { 1.5 } else { 0.5 },
            if is_current {
                ui.style().visuals.selection.stroke.color
            } else {
                ui.style().visuals.widgets.noninteractive.bg_stroke.color
            }
        ))
        .fill(if is_current {
            ui.style().visuals.selection.bg_fill.gamma_multiply(0.2)
        } else {
            ui.style().visuals.widgets.noninteractive.weak_bg_fill
        });

    let response = card_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            // 图标
            ui.label(egui::RichText::new(icon).size(20.0));
            ui.add_space(8.0);

            // 标题和描述
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title)
                    .size(14.0)
                    .strong()
                    .color(if is_current {
                        ui.style().visuals.selection.stroke.color
                    } else {
                        ui.style().visuals.text_color()
                    }));

                ui.label(egui::RichText::new(description)
                    .size(12.0)
                    .color(ui.style().visuals.weak_text_color()));
            });

            // 状态指示器
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if is_current {
                    ui.label(egui::RichText::new("●").size(12.0).color(ui.style().visuals.selection.stroke.color));
                }
            });
        });
    });

    // 整个卡片可点击
    if !is_current && response.response.clicked() {
        app.active_module = module;
    }
}

/// 渲染功能卡片（保留原版本用于搜索结果等）
fn render_feature_card(ui: &mut egui::Ui, icon: &str, title: &str, description: &str, module: Module, app: &mut SeeUApp) {
    let is_current = app.active_module == module;

    let card_frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(16.0))
        .stroke(egui::Stroke::new(
            if is_current { 2.0 } else { 1.0 },
            if is_current {
                ui.style().visuals.selection.stroke.color
            } else {
                ui.style().visuals.widgets.noninteractive.bg_stroke.color
            }
        ))
        .fill(if is_current {
            ui.style().visuals.selection.bg_fill.gamma_multiply(0.3)
        } else {
            ui.style().visuals.widgets.noninteractive.weak_bg_fill
        });

    let response = card_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // 图标和标题行
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(28.0));
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(title)
                        .size(18.0)
                        .strong()
                        .color(if is_current {
                            ui.style().visuals.selection.stroke.color
                        } else {
                            ui.style().visuals.text_color()
                        }));

                    ui.add_space(4.0);

                    ui.label(egui::RichText::new(description)
                        .size(14.0)
                        .color(ui.style().visuals.weak_text_color()));
                });
            });

            ui.add_space(12.0);

            // 操作按钮
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let button_text = if is_current { "当前模块" } else { "打开" };
                    let button = egui::Button::new(button_text)
                        .min_size(egui::vec2(80.0, 24.0));

                    if is_current {
                        ui.add_enabled(false, button);
                    } else if ui.add(button).clicked() {
                        app.active_module = module;
                    }
                });
            });
        });
    });

    // 整个卡片可点击
    if !is_current && response.response.clicked() {
        app.active_module = module;
    }
}



/// 渲染MCP工具豆腐块
fn render_mcp_tools_block(ui: &mut egui::Ui, app: &mut SeeUApp) {
    let block_frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12.0))
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .fill(ui.style().visuals.widgets.noninteractive.weak_bg_fill);

    block_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // 标题栏
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔧").size(18.0));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("MCP 工具").size(16.0).strong());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("管理").clicked() {
                        app.active_module = Module::ITools;
                    }
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // MCP服务器状态
            let (ready_servers, total_servers) = if let Some(manager) = app.itools_state.get_mcp_server_manager() {
                let servers = manager.list_servers();
                let ready_count = servers.iter()
                    .filter(|server| matches!(server.health_status, itools::mcp::rmcp_client::ServerHealthStatus::Green))
                    .count();
                (ready_count, servers.len())
            } else {
                (0, 0)
            };

            ui.horizontal(|ui| {
                ui.label(format!("🟢 {} 个就绪", ready_servers));
                ui.add_space(12.0);
                ui.label(format!("📊 共 {} 个服务", total_servers));
            });

            ui.add_space(8.0);

            // 常用工具（显示就绪的MCP服务器）
            ui.label(egui::RichText::new("常用工具").size(12.0).weak());
            ui.add_space(4.0);

            if let Some(manager) = app.itools_state.get_mcp_server_manager() {
                let servers = manager.list_servers();
                let ready_servers: Vec<_> = servers.iter()
                    .filter(|server| matches!(server.health_status, itools::mcp::rmcp_client::ServerHealthStatus::Green))
                    .take(4)
                    .collect();

                if ready_servers.is_empty() {
                    ui.label(egui::RichText::new("暂无可用工具").size(12.0).weak());
                    ui.add_space(4.0);
                    if ui.small_button("添加 MCP 服务器").clicked() {
                        app.active_module = Module::ITools;
                    }
                } else {
                    for server in ready_servers {
                        ui.horizontal(|ui| {
                            ui.label("🟢");
                            if ui.small_button(&server.name).clicked() {
                                app.active_module = Module::ITools;
                                // 可以在这里设置选中的服务器
                            }
                        });
                    }
                }
            } else {
                ui.label(egui::RichText::new("MCP 服务未初始化").size(12.0).weak());
                ui.add_space(4.0);
                if ui.small_button("初始化 MCP 服务").clicked() {
                    app.active_module = Module::ITools;
                }
            }
        });
    });
}

/// 渲染快速操作豆腐块
fn render_quick_actions_block(ui: &mut egui::Ui, app: &mut SeeUApp) {
    let block_frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12.0))
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .fill(ui.style().visuals.widgets.noninteractive.weak_bg_fill);

    block_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // 标题栏
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚡").size(18.0));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("快速操作").size(16.0).strong());
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // 快速操作按钮网格
            ui.columns(2, |columns| {
                // 左列
                columns[0].vertical(|ui| {
                    if ui.button("💻 终端").clicked() {
                        app.active_module = Module::Terminal;
                    }
                    ui.add_space(4.0);
                    if ui.button("📁 文件").clicked() {
                        app.active_module = Module::Files;
                    }
                });

                // 右列
                columns[1].vertical(|ui| {
                    if ui.button("📊 数据").clicked() {
                        app.active_module = Module::DataAnalysis;
                    }
                    ui.add_space(4.0);
                    if ui.button("⚙️ 设置").clicked() {
                        app.active_module = Module::Settings;
                    }
                });
            });

            ui.add_space(8.0);

            // 系统信息（简化版）
            ui.label(egui::RichText::new("系统状态").size(12.0).weak());
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("🟢");
                ui.label(egui::RichText::new("系统运行正常").size(12.0));
            });
        });
    });
}

/// 渲染紧凑版快速操作区域
fn render_quick_actions_compact(ui: &mut egui::Ui, app: &mut SeeUApp) {
    let card_frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(8.0))
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .fill(ui.style().visuals.widgets.noninteractive.weak_bg_fill);

    card_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // 创建新笔记
            let create_note_button = egui::Button::new("📝 创建新笔记")
                .min_size(egui::vec2(ui.available_width(), 24.0));

            if ui.add(create_note_button).clicked() {
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
            let search_button = egui::Button::new("🔍 开始搜索")
                .min_size(egui::vec2(ui.available_width(), 24.0));

            if ui.add(search_button).clicked() {
                app.active_module = Module::Search;
            }

            ui.add_space(4.0);

            // 智能终端
            let terminal_button = egui::Button::new("💻 智能终端")
                .min_size(egui::vec2(ui.available_width(), 24.0));

            if ui.add(terminal_button).clicked() {
                app.active_module = Module::Terminal;
            }

            ui.add_space(4.0);

            // 管理插件
            let tools_button = egui::Button::new("🔧 管理插件")
                .min_size(egui::vec2(ui.available_width(), 24.0));

            if ui.add(tools_button).clicked() {
                app.active_module = Module::ITools;
            }

            ui.add_space(4.0);

            // 打开设置
            let settings_button = egui::Button::new("⚙️ 应用设置")
                .min_size(egui::vec2(ui.available_width(), 24.0));

            if ui.add(settings_button).clicked() {
                app.active_module = Module::Settings;
            }
        });
    });
}

/// 渲染快速操作区域（保留原版本）
fn render_quick_actions(ui: &mut egui::Ui, app: &mut SeeUApp) {
    let card_frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(15.0))
        .stroke(egui::Stroke::new(1.5, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .fill(ui.style().visuals.widgets.noninteractive.weak_bg_fill);

    card_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // 标题
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚡").size(20.0));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("快速操作").heading().strong());
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(10.0);

            // 创建新笔记
            let create_note_button = egui::Button::new("📝 创建新笔记")
                .min_size(egui::vec2(ui.available_width(), 32.0));

            if ui.add(create_note_button).clicked() {
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

            ui.add_space(8.0);

            // 打开搜索
            let search_button = egui::Button::new("🔍 开始搜索")
                .min_size(egui::vec2(ui.available_width(), 32.0));

            if ui.add(search_button).clicked() {
                app.active_module = Module::Search;
            }

            ui.add_space(8.0);

            // 管理插件
            let tools_button = egui::Button::new("🔧 管理插件")
                .min_size(egui::vec2(ui.available_width(), 32.0));

            if ui.add(tools_button).clicked() {
                app.active_module = Module::ITools;
            }

            ui.add_space(8.0);

            // 打开设置
            let settings_button = egui::Button::new("⚙️ 应用设置")
                .min_size(egui::vec2(ui.available_width(), 32.0));

            if ui.add(settings_button).clicked() {
                app.active_module = Module::Settings;
            }
        });
    });
}

/// Render iNote search results
fn render_inote_results(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.heading("📝 笔记 (iNote)");
    ui.add_space(8.0);

    for result in &app.global_search_results.inote_results {
        let card_frame = egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(10.0))
            .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color));

        card_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("📝");
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    let truncated_title = inote::truncate_note_title(&result.title);
                    if ui.link(&truncated_title).clicked() {
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
            .inner_margin(egui::Margin::same(10.0))
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
            .inner_margin(egui::Margin::same(10.0))
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

/// 渲染紧凑版底部信息区域
fn render_footer_section_compact(ui: &mut egui::Ui) {
    ui.separator();
    ui.add_space(8.0);

    // 提示信息 - 紧凑版
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("💡").size(14.0));
        ui.add_space(4.0);
        ui.label(egui::RichText::new("提示:").size(12.0).strong());
        ui.add_space(4.0);
        ui.label(egui::RichText::new("使用顶部搜索栏快速搜索").size(12.0).weak());

        ui.add_space(12.0);

        ui.label(egui::RichText::new("🎯").size(14.0));
        ui.add_space(4.0);
        ui.label(egui::RichText::new("快捷键:").size(12.0).strong());
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Ctrl+/ 命令面板").size(12.0).weak());
    });
}

/// 渲染底部信息区域（保留原版本）
fn render_footer_section(ui: &mut egui::Ui) {
    ui.separator();
    ui.add_space(15.0);

    // 提示信息卡片
    let card_frame = egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12.0))
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color))
        .fill(ui.style().visuals.widgets.noninteractive.weak_bg_fill);

    card_frame.show(ui, |ui| {
        ui.vertical(|ui| {
            // 使用提示
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("💡").size(16.0));
                ui.add_space(6.0);
                ui.label(egui::RichText::new("使用提示:").strong());
                ui.add_space(8.0);
                ui.label("使用顶部搜索栏可以快速搜索文件和内容");
            });

            ui.add_space(8.0);

            // 快捷键提示
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🎯").size(16.0));
                ui.add_space(6.0);
                ui.label(egui::RichText::new("快捷键:").strong());
                ui.add_space(8.0);
                ui.label("Ctrl+/ 打开命令面板，Ctrl+K 快速搜索");
            });

            ui.add_space(8.0);

            // 版权信息
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("ℹ️").size(16.0));
                ui.add_space(6.0);
                ui.label(egui::RichText::new("关于:").strong());
                ui.add_space(8.0);
                ui.label(egui::RichText::new("SeeU Desktop - 智能桌面助手").weak());
            });
        });
    });
}
