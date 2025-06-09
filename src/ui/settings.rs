use eframe::egui;
use crate::app::SeeUApp;
use crate::ui::theme::Theme;

/// Settings categories
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SettingsCategory {
    Appearance,
    General,
    Search,
    Terminal,
    ITools,
    AIAssistant,
    Advanced,
}

impl SettingsCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            SettingsCategory::Appearance => "🎨 外观设置",
            SettingsCategory::General => "🔧 常规设置",
            SettingsCategory::Search => "🔍 搜索设置",
            SettingsCategory::Terminal => "💻 终端设置",
            SettingsCategory::ITools => "🛠️ iTools设置",
            SettingsCategory::AIAssistant => "🤖 AI助手设置",
            SettingsCategory::Advanced => "⚙️ 高级设置",
        }
    }

    pub fn all() -> Vec<SettingsCategory> {
        vec![
            SettingsCategory::Appearance,
            SettingsCategory::General,
            SettingsCategory::Search,
            SettingsCategory::Terminal,
            SettingsCategory::ITools,
            SettingsCategory::AIAssistant,
            SettingsCategory::Advanced,
        ]
    }
}

/// Settings state
#[derive(Debug, Default)]
pub struct SettingsState {
    pub current_category: SettingsCategory,
}

impl Default for SettingsCategory {
    fn default() -> Self {
        SettingsCategory::Appearance
    }
}

/// Render the settings interface
pub fn render_settings(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.heading("⚙️ 设置");
    ui.separator();
    ui.add_space(10.0);

    // Get current category before borrowing
    let current_category = app.settings_state.current_category;

    // Settings layout with categories on the left and content on the right
    ui.horizontal(|ui| {
        // Left panel - Categories
        ui.vertical(|ui| {
            ui.set_width(150.0);
            render_settings_categories(ui, &mut app.settings_state);
        });

        ui.separator();

        // Right panel - Content
        ui.vertical(|ui| {
            ui.set_min_width(300.0);
            render_settings_content_by_category(ui, app, current_category);
        });
    });
}

/// Render settings categories
fn render_settings_categories(ui: &mut egui::Ui, settings_state: &mut SettingsState) {
    ui.label(egui::RichText::new("设置分类").strong());
    ui.add_space(8.0);

    for category in SettingsCategory::all() {
        let is_selected = settings_state.current_category == category;
        if ui.selectable_label(is_selected, category.display_name()).clicked() {
            settings_state.current_category = category;
        }
    }
}

/// Render settings content based on selected category
fn render_settings_content_by_category(ui: &mut egui::Ui, app: &mut SeeUApp, current_category: SettingsCategory) {
    match current_category {
        SettingsCategory::Appearance => render_appearance_settings(ui, app),
        SettingsCategory::General => render_general_settings(ui, app),
        SettingsCategory::Search => render_search_settings(ui, app),
        SettingsCategory::Terminal => render_terminal_settings(ui, app),
        SettingsCategory::ITools => render_itools_settings(ui, app),
        SettingsCategory::AIAssistant => render_ai_assistant_settings(ui, app),
        SettingsCategory::Advanced => render_advanced_settings(ui, app),
    }
}

/// Render appearance settings
fn render_appearance_settings(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.heading("🎨 外观设置");
    ui.add_space(10.0);

    // Color Theme section
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Color Theme").strong());
            ui.add_space(5.0);

            let current_theme = app.get_theme();

            // Theme selection
            for theme in Theme::all() {
                let is_selected = current_theme == theme;
                let response = ui.selectable_label(is_selected, theme.display_name());

                if response.clicked() && !is_selected {
                    app.set_theme(ui.ctx(), theme);
                }

                // Add theme preview/description
                if is_selected {
                    ui.indent("theme_description", |ui| {
                        ui.label(egui::RichText::new(get_theme_description(theme)).weak());
                    });
                }
            }
        });
    });

    ui.add_space(15.0);

    // Font settings (placeholder for future implementation)
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("字体设置").strong());
            ui.add_space(5.0);
            ui.label("字体大小和样式设置将在未来版本中提供");
        });
    });

    ui.add_space(15.0);

    // UI Scale settings (placeholder for future implementation)
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("界面缩放").strong());
            ui.add_space(5.0);
            ui.label("界面缩放设置将在未来版本中提供");
        });
    });
}

/// Render general settings
fn render_general_settings(ui: &mut egui::Ui, _app: &mut SeeUApp) {
    ui.heading("🔧 常规设置");
    ui.add_space(10.0);

    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("启动设置").strong());
            ui.add_space(5.0);
            ui.checkbox(&mut false, "开机自动启动");
            ui.checkbox(&mut true, "启动时恢复上次会话");
        });
    });

    ui.add_space(15.0);

    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("数据设置").strong());
            ui.add_space(5.0);
            ui.checkbox(&mut true, "自动保存");
            ui.checkbox(&mut false, "定期备份数据");
        });
    });
}

/// Render search settings
fn render_search_settings(ui: &mut egui::Ui, app: &mut SeeUApp) {
    // Check for completed indexing operations to update UI
    app.isearch_state.check_reindex_results();

    ui.heading("🔍 搜索设置");
    ui.add_space(10.0);

    // Process directory dialog in settings
    app.isearch_state.process_directory_dialog();

    // Indexed directories management
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("索引目录管理").strong());
            ui.add_space(5.0);

            // Add directory button
            ui.horizontal(|ui| {
                if ui.button("+ 添加目录").clicked() {
                    app.isearch_state.open_directory_dialog();
                }

                // Remove directory button (only enabled if a directory is selected)
                if let Some(selected) = app.isearch_state.selected_directory {
                    if ui.button("- 移除目录").clicked() {
                        app.isearch_state.remove_directory(selected);
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("- 移除目录"));
                }
            });

            ui.add_space(8.0);

            // Directory list - full width
            ui.label("已索引的目录：");
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    if app.isearch_state.indexed_directories.is_empty() {
                        ui.label("暂无索引目录");
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("提示：添加目录后才能进行文件搜索").weak());
                    } else {
                        // Clone the directories to avoid borrowing issues
                        let directories = app.isearch_state.indexed_directories.clone();
                        let selected_directory = app.isearch_state.selected_directory;

                        for (i, directory) in directories.iter().enumerate() {
                            let is_selected = selected_directory == Some(i);

                            // Full width group for each directory
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(ui.available_width(), 0.0),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    ui.group(|ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.vertical(|ui| {
                                            // Directory path with wrapping
                                            let path_text = format!("📁 {}", directory.path);
                                            ui.allocate_ui_with_layout(
                                                egui::Vec2::new(ui.available_width(), 0.0),
                                                egui::Layout::top_down(egui::Align::LEFT),
                                                |ui| {
                                                    if ui.selectable_label(is_selected, &path_text).clicked() {
                                                        app.isearch_state.selected_directory = Some(i);
                                                    }
                                                }
                                            );

                                            // Directory stats in a horizontal layout
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new(format!("📄 {} 个文件", directory.file_count)).small().weak());
                                                ui.label(egui::RichText::new(format!("💾 {:.1} MB", directory.total_size_bytes as f64 / (1024.0 * 1024.0))).small().weak());
                                            });

                                            // Last indexed time
                                            if let Some(last_indexed) = directory.last_indexed {
                                                ui.label(egui::RichText::new(format!("🕒 最后索引: {}", last_indexed.format("%m-%d %H:%M"))).small().weak());
                                            } else {
                                                ui.label(egui::RichText::new("🕒 未索引").small().weak());
                                            }

                                            // Update button for this directory
                                            ui.horizontal(|ui| {
                                                if ui.small_button("🔄 更新此目录").on_hover_text("重新索引此目录").clicked() {
                                                    app.isearch_state.update_directory_index(i);
                                                }
                                            });
                                        });
                                    });
                                }
                            );

                            ui.add_space(4.0);
                        }
                    }
                });

            ui.add_space(8.0);

            // Directory management buttons
            ui.horizontal(|ui| {
                // Remove directory button
                if ui.button("➖ 移除目录").clicked() {
                    if let Some(selected) = app.isearch_state.selected_directory {
                        if selected < app.isearch_state.indexed_directories.len() {
                            let removed_dir = app.isearch_state.indexed_directories.remove(selected);
                            app.isearch_state.save_indexed_directories();
                            app.isearch_state.selected_directory = None;
                            log::info!("Removed directory from index: {}", removed_dir.path);
                        }
                    }
                }

                // Update all indexes button
                if ui.button("🔄 更新全部索引").on_hover_text("重新索引所有目录").clicked() {
                    app.isearch_state.update_all_indexes();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("选择目录后可移除或更新").weak());
                });
            });
        });
    });

    ui.add_space(15.0);

    // Index statistics
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("索引统计").strong());
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("已索引文件数:");
                ui.label(format!("{}", app.isearch_state.index_stats.total_files));
            });

            ui.horizontal(|ui| {
                ui.label("索引大小:");
                ui.label(format!("{:.1} MB", app.isearch_state.index_stats.total_size_bytes as f64 / (1024.0 * 1024.0)));
            });

            if let Some(last_updated) = app.isearch_state.index_stats.last_updated {
                ui.horizontal(|ui| {
                    ui.label("最后更新:");
                    ui.label(format!("{}", last_updated.format("%Y-%m-%d %H:%M")));
                });
            }

            if app.isearch_state.is_indexing {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("正在索引...");
                });
            }
        });
    });

    ui.add_space(15.0);

    // Search options
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("搜索选项").strong());
            ui.add_space(5.0);

            let mut options_changed = false;

            if ui.checkbox(&mut app.isearch_state.enable_content_preview, "启用内容预览").changed() {
                options_changed = true;
            }
            if ui.checkbox(&mut app.isearch_state.enable_file_type_filter, "启用文件类型筛选").changed() {
                options_changed = true;
            }
            if ui.checkbox(&mut app.isearch_state.search_hidden_files, "搜索隐藏文件").changed() {
                options_changed = true;
            }
            if ui.checkbox(&mut app.isearch_state.enable_file_monitoring, "实时文件监控").changed() {
                options_changed = true;
            }

            // Auto-save search options when changed
            if options_changed {
                app.isearch_state.save_search_options();
            }
        });
    });
}

/// Render terminal settings
fn render_terminal_settings(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.heading("💻 终端设置");
    ui.add_space(10.0);

    let mut config = app.iterminal_state.get_config().clone();
    let mut font_scale = app.iterminal_state.font_scale;
    let mut config_changed = false;

    // Appearance settings
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("外观设置").strong());
            ui.add_space(5.0);

            // Font size
            ui.horizontal(|ui| {
                ui.label("字体大小:");
                if ui.add(egui::Slider::new(&mut config.font_size, 8.0..=24.0).suffix("px")).changed() {
                    config_changed = true;
                }
            });

            // Font scale
            ui.horizontal(|ui| {
                ui.label("字体缩放:");
                ui.add(egui::Slider::new(&mut font_scale, 0.5..=3.0).step_by(0.1));
            });

            // Scrollback lines
            ui.horizontal(|ui| {
                ui.label("滚动缓冲行数:");
                if ui.add(egui::Slider::new(&mut config.scrollback_lines, 100..=50000).suffix("行")).changed() {
                    config_changed = true;
                }
            });
        });
    });

    ui.add_space(15.0);

    // Behavior settings
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("行为设置").strong());
            ui.add_space(5.0);

            // Shell command
            ui.horizontal(|ui| {
                ui.label("默认Shell:");
                if ui.text_edit_singleline(&mut config.shell_command).changed() {
                    config_changed = true;
                }
            });

            // Enable bell
            if ui.checkbox(&mut config.enable_bell, "启用响铃").changed() {
                config_changed = true;
            }

            // Cursor blink
            ui.horizontal(|ui| {
                ui.label("光标闪烁间隔:");
                if ui.add(egui::Slider::new(&mut config.cursor_blink_interval, 0..=2000).suffix("ms")).changed() {
                    config_changed = true;
                }
                if config.cursor_blink_interval == 0 {
                    ui.label("(0 = 禁用闪烁)");
                }
            });

            // Tab size
            ui.horizontal(|ui| {
                ui.label("Tab大小:");
                if ui.add(egui::Slider::new(&mut config.tab_size, 2..=8).suffix("空格")).changed() {
                    config_changed = true;
                }
            });
        });
    });

    ui.add_space(15.0);

    // Color settings
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("颜色设置").strong());
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("背景色:");
                let mut bg_color = egui::Color32::from_rgba_premultiplied(
                    (config.background_color[0] * 255.0) as u8,
                    (config.background_color[1] * 255.0) as u8,
                    (config.background_color[2] * 255.0) as u8,
                    (config.background_color[3] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut bg_color).changed() {
                    let [r, g, b, a] = bg_color.to_normalized_gamma_f32();
                    config.background_color = [r, g, b, a];
                    config_changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("文本色:");
                let mut text_color = egui::Color32::from_rgba_premultiplied(
                    (config.text_color[0] * 255.0) as u8,
                    (config.text_color[1] * 255.0) as u8,
                    (config.text_color[2] * 255.0) as u8,
                    (config.text_color[3] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut text_color).changed() {
                    let [r, g, b, a] = text_color.to_normalized_gamma_f32();
                    config.text_color = [r, g, b, a];
                    config_changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("光标色:");
                let mut cursor_color = egui::Color32::from_rgba_premultiplied(
                    (config.cursor_color[0] * 255.0) as u8,
                    (config.cursor_color[1] * 255.0) as u8,
                    (config.cursor_color[2] * 255.0) as u8,
                    (config.cursor_color[3] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut cursor_color).changed() {
                    let [r, g, b, a] = cursor_color.to_normalized_gamma_f32();
                    config.cursor_color = [r, g, b, a];
                    config_changed = true;
                }
            });
        });
    });

    ui.add_space(15.0);

    // Action buttons
    ui.horizontal(|ui| {
        if ui.button("💾 保存设置").clicked() || config_changed {
            app.iterminal_state.update_config(config.clone());
            app.iterminal_state.font_scale = font_scale;
            let _ = app.iterminal_state.save_config();
        }

        if ui.button("🔄 重置为默认").clicked() {
            let default_config = iterminal::config::TerminalConfig::default();
            app.iterminal_state.update_config(default_config);
            app.iterminal_state.font_scale = 1.0;
        }

        if ui.button("📋 导出配置").clicked() {
            // TODO: Implement config export
            log::info!("Export terminal config");
        }

        if ui.button("📁 导入配置").clicked() {
            // TODO: Implement config import
            log::info!("Import terminal config");
        }
    });

    // Update the state
    if config_changed {
        app.iterminal_state.update_config(config);
    }
    app.iterminal_state.font_scale = font_scale;
}

/// Render iTools settings
fn render_itools_settings(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.heading("🛠️ iTools 设置");
    ui.add_space(10.0);

    // Role settings
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("角色设置").strong());
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("当前角色:");
                ui.label(egui::RichText::new(app.itools_state.current_role.display_name()).strong());
            });

            ui.add_space(5.0);

            // Role selection
            egui::ComboBox::from_id_source("itools_role_selector")
                .selected_text(app.itools_state.current_role.display_name())
                .show_ui(ui, |ui| {
                    use itools::roles::UserRole;

                    let roles = [
                        UserRole::BusinessUser,
                        UserRole::Developer,
                        UserRole::Operations,
                        UserRole::Administrator,
                    ];

                    for role in &roles {
                        let selected = ui.selectable_value(
                            &mut app.itools_state.current_role,
                            role.clone(),
                            role.display_name(),
                        );

                        if selected.clicked() {
                            log::info!("Role changed to: {:?}", role);
                            // Log role change for audit
                            app.itools_state.log_audit(
                                format!("Role changed to {}", role.display_name()),
                                None,
                                itools::state::AuditResult::Success,
                            );
                        }
                    }
                });

            ui.add_space(5.0);
            ui.label(egui::RichText::new("角色说明:").weak());
            ui.label(egui::RichText::new(get_role_description(&app.itools_state.current_role)).weak());
        });
    });

    ui.add_space(15.0);

    // Plugin settings
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("插件设置").strong());
            ui.add_space(5.0);

            // Plugin statistics
            let installed_plugins = app.itools_state.plugin_manager.get_installed_plugins();
            let connected_plugins = app.itools_state.mcp_client.get_connected_plugins();

            egui::Grid::new("plugin_stats")
                .num_columns(2)
                .spacing([20.0, 5.0])
                .show(ui, |ui| {
                    ui.label("已安装插件:");
                    ui.label(format!("{}", installed_plugins.len()));
                    ui.end_row();

                    ui.label("已连接插件:");
                    ui.label(format!("{}", connected_plugins.len()));
                    ui.end_row();

                    ui.label("插件目录:");
                    ui.label("~/.seeu/itools/plugins");
                    ui.end_row();
                });

            ui.add_space(10.0);

            // Plugin management buttons
            ui.horizontal(|ui| {
                if ui.button("🔄 刷新插件列表").clicked() {
                    log::info!("Refreshing plugin list");
                    // TODO: Implement plugin refresh
                }

                if ui.button("📁 打开插件目录").clicked() {
                    log::info!("Opening plugin directory");
                    // TODO: Open plugin directory
                }

                if ui.button("🧹 清理缓存").clicked() {
                    log::info!("Cleaning plugin cache");
                    // TODO: Clean plugin cache
                }
            });
        });
    });

    ui.add_space(15.0);

    // Security settings
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("安全设置").strong());
            ui.add_space(5.0);

            // Permission settings
            ui.horizontal(|ui| {
                ui.label("权限级别:");
                let permission_level = match app.itools_state.current_role {
                    itools::roles::UserRole::BusinessUser => "受限",
                    itools::roles::UserRole::Developer => "中等",
                    itools::roles::UserRole::Operations => "运维",
                    itools::roles::UserRole::Administrator => "完整",
                    itools::roles::UserRole::Custom(_) => "自定义",
                };
                ui.label(egui::RichText::new(permission_level).strong());
            });

            ui.add_space(5.0);

            // Audit log summary
            let audit_entries = app.itools_state.security_context.audit_log.len();
            ui.horizontal(|ui| {
                ui.label("审计日志条目:");
                ui.label(format!("{}", audit_entries));
            });

            ui.add_space(10.0);

            // Security actions
            ui.horizontal(|ui| {
                if ui.button("📊 查看审计日志").clicked() {
                    log::info!("Opening audit log");
                    // Switch to iTools module and show audit view
                    app.active_module = crate::app::Module::ITools;
                    // TODO: Set specific view for audit log
                }

                if ui.button("🔒 安全检查").clicked() {
                    log::info!("Running security check");
                    // TODO: Run security check
                }
            });
        });
    });

    ui.add_space(15.0);

    // System information
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("系统信息").strong());
            ui.add_space(5.0);

            egui::Grid::new("system_info")
                .num_columns(2)
                .spacing([20.0, 5.0])
                .show(ui, |ui| {
                    ui.label("会话ID:");
                    ui.label(format!("{}", app.itools_state.security_context.session_id));
                    ui.end_row();

                    ui.label("MCP 协议版本:");
                    ui.label("1.0.0");
                    ui.end_row();

                    ui.label("插件API版本:");
                    ui.label("0.1.0");
                    ui.end_row();
                });

            ui.add_space(10.0);

            // System actions
            ui.horizontal(|ui| {
                if ui.button("🔄 重新初始化").clicked() {
                    log::info!("Reinitializing iTools");
                    app.itools_state.initialize();
                }

                if ui.button("📋 导出配置").clicked() {
                    log::info!("Exporting iTools configuration");
                    // TODO: Export configuration
                }

                if ui.button("📁 导入配置").clicked() {
                    log::info!("Importing iTools configuration");
                    // TODO: Import configuration
                }
            });
        });
    });
}

/// Get role description for display
fn get_role_description(role: &itools::roles::UserRole) -> &'static str {
    match role {
        itools::roles::UserRole::BusinessUser => "适用于业务分析和数据处理任务，具有受限的系统权限",
        itools::roles::UserRole::Developer => "适用于软件开发和代码管理任务，具有中等的系统权限",
        itools::roles::UserRole::Operations => "适用于系统运维和监控任务，具有运维相关的系统权限",
        itools::roles::UserRole::Administrator => "适用于系统管理和配置任务，具有完整的系统权限",
        itools::roles::UserRole::Custom(_) => "自定义角色，权限根据具体配置而定",
    }
}

/// Render AI assistant settings
fn render_ai_assistant_settings(ui: &mut egui::Ui, app: &mut SeeUApp) {
    ui.heading("🤖 AI助手设置");
    ui.add_space(10.0);

    // Get AI assistant settings from the app
    {
        let ai_state = &mut app.ai_assist_state;
        // API Configuration
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("API 配置").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Base URL:");
                    ui.text_edit_singleline(&mut ai_state.ai_settings.base_url);
                });

                ui.horizontal(|ui| {
                    ui.label("API Key:");
                    ui.text_edit_singleline(&mut ai_state.ai_settings.api_key);
                });

                ui.horizontal(|ui| {
                    ui.label("模型名称:");
                    ui.text_edit_singleline(&mut ai_state.ai_settings.model);
                });
            });
        });

        ui.add_space(15.0);

        // Model Parameters
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("模型参数").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Temperature:");
                    ui.add(egui::Slider::new(&mut ai_state.ai_settings.temperature, 0.0..=2.0)
                        .step_by(0.1)
                        .text("创造性"));
                });

                ui.horizontal(|ui| {
                    ui.label("Max Tokens:");
                    ui.add(egui::Slider::new(&mut ai_state.ai_settings.max_tokens, 100..=8000)
                        .step_by(100.0)
                        .text("最大长度"));
                });

                ui.checkbox(&mut ai_state.ai_settings.streaming, "启用流式输出");
            });
        });

        ui.add_space(15.0);

        // Chat Settings
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("对话设置").strong());
                ui.add_space(5.0);

                if ui.button("清除所有对话历史").clicked() {
                    ai_state.chat_sessions.clear();
                    ai_state.create_new_session();
                    ai_state.active_session_idx = 0;
                }

                if ui.button("重置为默认设置").clicked() {
                    ai_state.ai_settings = Default::default();
                }
            });
        });
    }
}

/// Render advanced settings
fn render_advanced_settings(ui: &mut egui::Ui, _app: &mut SeeUApp) {
    ui.heading("⚙️ 高级设置");
    ui.add_space(10.0);

    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("性能设置").strong());
            ui.add_space(5.0);
            ui.checkbox(&mut true, "启用硬件加速");
            ui.checkbox(&mut false, "调试模式");
        });
    });

    ui.add_space(15.0);

    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("开发者选项").strong());
            ui.add_space(5.0);
            if ui.button("打开日志目录").clicked() {
                log::info!("Opening log directory");
            }
            if ui.button("重置所有设置").clicked() {
                log::info!("Reset all settings");
            }
        });
    });
}

/// Get theme description
fn get_theme_description(theme: Theme) -> &'static str {
    match theme {
        Theme::DarkModern => "现代深色主题，适合长时间使用，减少眼部疲劳",
        Theme::LightModern => "现代浅色主题，清晰明亮，适合白天使用",
        Theme::Dark => "经典深色主题（兼容性保留）",
        Theme::Light => "经典浅色主题（兼容性保留）",
    }
}
