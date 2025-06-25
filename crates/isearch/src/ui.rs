use eframe::egui;
use crate::{ISearchState, SortBy, ViewMode};
use crate::utils;

/// Render the iSearch module
pub fn render_isearch(ui: &mut egui::Ui, state: &mut ISearchState) {
    render_isearch_with_sidebar_info(ui, state, false, None);
}

/// Render the iSearch module with right sidebar awareness
pub fn render_isearch_with_sidebar_info(ui: &mut egui::Ui, state: &mut ISearchState, right_sidebar_open: bool, right_sidebar_width: Option<f32>) {
    // Process directory dialog
    state.process_directory_dialog();

    // Process file watcher events
    state.process_watcher_events();

    // Check for completed indexing operations (important for updating UI)
    state.check_reindex_results();

    // Process search results from background thread
    state.process_search_results();

    // Use available_rect to get the actual available space
    let available_rect = ui.available_rect_before_wrap();
    let content_height = available_rect.height() - 20.0; // Reserve space for status bar and padding

    ui.allocate_ui_with_layout(
        egui::Vec2::new(available_rect.width(), content_height),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
        // Search bar
        ui.vertical(|ui| {
            let search_id = ui.make_persistent_id("search_input");

            ui.horizontal(|ui| {
                // Directory panel toggle button (only show if there are indexed directories)
                if !state.indexed_directories.is_empty() {
                    let toggle_text = if state.show_directories_panel { "📁 ▼" } else { "📁 ▶" };
                    if ui.button(toggle_text).on_hover_text("显示/隐藏索引目录").clicked() {
                        state.show_directories_panel = !state.show_directories_panel;
                    }
                }

                ui.label("🔍");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut state.search_query)
                        .hint_text("搜索文件... (支持 filetype:pdf, filename:name, +必须, \"精确短语\")")
                        .desired_width(ui.available_width() - 100.0)
                        .id(search_id)
                );

                // Trigger search based on user settings
                let should_search = ui.button("搜索").clicked() ||
                   (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) ||
                   (state.search_on_typing && response.changed() && !state.search_query.trim().is_empty());

                if should_search {
                    state.search();
                }

                // File type filter button (only show if enabled)
                if state.enable_file_type_filter {
                    let filter_text = if state.show_file_type_filter { "🔽" } else { "🔼" };
                    if ui.button(format!("📁{}", filter_text)).on_hover_text("文件类型筛选").clicked() {
                        state.show_file_type_filter = !state.show_file_type_filter;
                    }
                }

                let help_button = ui.button("?").on_hover_text("点击查看高级搜索语法帮助");
                if help_button.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(ui.make_persistent_id("search_syntax_help")));
                }
            });

            // Show popup with search syntax help
            let popup_id = ui.make_persistent_id("search_syntax_help");
            if ui.memory(|mem| mem.is_popup_open(popup_id)) {
                egui::Window::new("高级搜索语法")
                    .id(popup_id)
                    .fixed_size([400.0, 200.0])
                    .show(ui.ctx(), |ui| {
                        ui.heading("高级搜索语法");
                        ui.separator();
                        ui.label("支持以下高级搜索语法：");
                        ui.label("• filetype:pdf - 按文件类型筛选");
                        ui.label("• filename:report - 按文件名筛选");
                        ui.label("• +关键词 - 必须包含该关键词");
                        ui.label("• \"精确短语\" - 精确匹配短语");
                        ui.label("示例：project +important filetype:pdf \"quarterly report\"");
                    });
            }
        });

        // File type filter panel
        if state.show_file_type_filter && state.enable_file_type_filter {
            ui.add_space(5.0);
            ui.horizontal_wrapped(|ui| {
                ui.label("文件类型:");

                let file_types = vec![
                    ("文档", vec!["pdf", "doc", "docx", "txt", "md", "rtf"]),
                    ("表格", vec!["xls", "xlsx", "csv", "ods"]),
                    ("演示", vec!["ppt", "pptx", "odp"]),
                    ("代码", vec!["rs", "py", "js", "ts", "java", "cpp", "c", "h", "go", "php"]),
                    ("图片", vec!["jpg", "jpeg", "png", "gif", "bmp", "svg", "webp"]),
                    ("音频", vec!["mp3", "wav", "flac", "aac", "ogg"]),
                    ("视频", vec!["mp4", "avi", "mkv", "mov", "wmv", "flv"]),
                    ("压缩", vec!["zip", "rar", "7z", "tar", "gz", "bz2"]),
                ];

                for (category, extensions) in file_types {
                    let is_selected = extensions.iter().any(|ext| state.selected_file_types.contains(&ext.to_string()));
                    let mut selected = is_selected;

                    if ui.checkbox(&mut selected, category).changed() {
                        if selected {
                            // Add all extensions in this category
                            for ext in extensions {
                                if !state.selected_file_types.contains(&ext.to_string()) {
                                    state.selected_file_types.push(ext.to_string());
                                }
                            }
                        } else {
                            // Remove all extensions in this category
                            state.selected_file_types.retain(|ext| !extensions.contains(&ext.as_str()));
                        }
                    }
                }

                if ui.button("清除").clicked() {
                    state.selected_file_types.clear();
                }
            });
        }

        ui.separator();

        // Main content - show directory info panel only if there are indexed directories and panel is enabled
        if !state.indexed_directories.is_empty() && state.show_directories_panel {
            egui::SidePanel::left("directories_panel")
                .resizable(true)
                .default_width(200.0)
                .show_inside(ui, |ui| {
                    ui.heading("索引目录");

                    ui.separator();

                    // Directory list with detailed info - full width
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Clone the directories to avoid borrowing issues
                        let directories = state.indexed_directories.clone();
                        let selected_directory = state.selected_directory;

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
                                                        state.selected_directory = Some(i);
                                                    }
                                                }
                                            );

                                            // Directory stats
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new(format!("📄 {} 个文件", directory.file_count)).small().weak());
                                                ui.label(egui::RichText::new(format!("💾 {:.1} MB", directory.total_size_bytes as f64 / (1024.0 * 1024.0))).small().weak());
                                            });

                                            // Last indexed time
                                            if let Some(last_indexed) = directory.last_indexed {
                                                ui.label(egui::RichText::new(format!("🕒 {}", last_indexed.format("%m-%d %H:%M"))).small().weak());
                                            } else {
                                                ui.label(egui::RichText::new("🕒 未索引").small().weak());
                                            }

                                            // Update button for this directory
                                            ui.horizontal(|ui| {
                                                if ui.small_button("🔄 更新此目录").on_hover_text("重新索引此目录").clicked() {
                                                    state.update_directory_index(i);
                                                }
                                            });
                                        });
                                    });
                                }
                            );

                            ui.add_space(4.0);
                        }
                    });

                    // Index stats
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.label(format!(
                            "已索引 {} 个文件 ({:.1} MB)",
                            state.index_stats.total_files,
                            state.index_stats.total_size_bytes as f64 / (1024.0 * 1024.0)
                        ));

                        if let Some(last_updated) = state.index_stats.last_updated {
                            ui.label(format!(
                                "最后更新: {}",
                                last_updated.format("%Y-%m-%d %H:%M")
                            ));
                        }

                        if state.is_indexing {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("正在索引...");
                            });
                        }

                        ui.separator();

                        // Reindex all directories button
                        if ui.button("🔄 重新索引全部").on_hover_text("重新索引所有目录，应用最新功能改进").clicked() {
                            state.reindex_all_directories();
                        }

                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("💡 在设置中管理索引目录").weak());
                    });
                });
        }

        
        // 正常情况下使用完整的中央面板
        egui::CentralPanel::default().show_inside(ui, |ui| {
            render_search_results_content(ui, state);
        });
       
        }
    );
}

/// Render the search results content area
fn render_search_results_content(ui: &mut egui::Ui, state: &mut ISearchState) {
    // Add a scroll area for the entire central panel content to prevent overflow
    let central_height = ui.available_height();
    egui::ScrollArea::vertical()
        .max_height(central_height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            // Dynamic heading with search statistics
            if !state.search_results.is_empty() && !state.is_searching {
                // Show results count and time in the heading
                let heading_text = if state.search_stats.total_matches > state.search_stats.total_results {
                    format!("搜索结果 - 找到 {} 个匹配（{} 个文件），耗时 {:.2} 秒",
                        state.search_stats.total_matches,
                        state.search_stats.total_results,
                        state.search_stats.search_time_ms as f64 / 1000.0)
                } else {
                    format!("搜索结果 - 找到 {} 个结果，耗时 {:.2} 秒",
                        state.search_stats.total_results,
                        state.search_stats.search_time_ms as f64 / 1000.0)
                };
                ui.heading(heading_text);
            } else {
                ui.heading("搜索结果");
            }

            // Check if there are no indexed directories
            if state.indexed_directories.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(egui::RichText::new("📂").size(48.0));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("暂无索引目录").heading());
                        ui.add_space(10.0);
                        ui.label("请先在设置中添加要搜索的目录");
                        ui.add_space(15.0);

                        if ui.button("🔧 前往设置").clicked() {
                            state.navigate_to_settings = true;
                            log::info!("Navigate to settings for directory indexing");
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.label(egui::RichText::new("💡 提示").strong());
                        ui.label("• 添加目录后系统会自动建立索引");
                        ui.label("• 支持多个目录同时索引");
                        ui.label("• 索引完成后即可进行快速搜索");
                    });
                });
            } else if state.search_query.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("🔍").size(32.0));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("开始搜索").heading());
                        ui.add_space(5.0);
                        ui.label("在上方搜索框中输入关键词开始搜索");
                    });
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(15.0);

                // Always show search syntax help when search is empty (not collapsible)
                ui.heading("🎯 高级搜索语法");
                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("支持以下高级搜索语法：").strong());
                        ui.add_space(8.0);

                        // File type filtering
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📄").size(16.0));
                            ui.label(egui::RichText::new("filetype:pdf").code());
                            ui.label("- 按文件类型筛选");
                        });

                        // Filename filtering
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📝").size(16.0));
                            ui.label(egui::RichText::new("filename:report").code());
                            ui.label("- 按文件名筛选");
                        });

                        // Required keywords
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("✅").size(16.0));
                            ui.label(egui::RichText::new("+关键词").code());
                            ui.label("- 必须包含该关键词");
                        });

                        // Exact phrases
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("💬").size(16.0));
                            ui.label(egui::RichText::new("\"精确短语\"").code());
                            ui.label("- 精确匹配短语");
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Example section
                        ui.label(egui::RichText::new("💡 示例：").strong());
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("🔍");
                            ui.label(egui::RichText::new("project +important filetype:pdf \"quarterly report\"").code());
                        });
                        ui.label(egui::RichText::new("查找包含 'project' 和 'important' 的 PDF 文件，且包含精确短语 'quarterly report'").weak());
                    });
                });

            } else if state.search_results.is_empty() && !state.is_searching {
                ui.centered_and_justified(|ui| {
                    ui.label("未找到匹配结果");
                });
            } else if state.is_searching {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label("正在搜索...");
                });
            } else {
                // Search statistics at the top
                ui.horizontal(|ui| {
                    

                    if state.has_more_results {
                        ui.label("(显示前 100 条结果)");
                    }
                });

                ui.separator();

                // Sort and view controls in one row
                ui.horizontal(|ui| {
                    // Sort controls
                    ui.label("排序：");

                    // Sort by buttons
                    let sort_options = [
                        SortBy::Relevance,
                        SortBy::FileName,
                        SortBy::DirectoryName,
                        SortBy::FileSize,
                        SortBy::ModifiedTime,
                    ];

                    for sort_option in &sort_options {
                        let is_current = state.sort_by == *sort_option;
                        let button_text = if is_current {
                            format!("{} {}", sort_option.display_name(), state.sort_order.icon())
                        } else {
                            sort_option.display_name().to_string()
                        };

                        let button = if is_current {
                            ui.add(egui::Button::new(button_text).fill(ui.visuals().selection.bg_fill))
                        } else {
                            ui.button(button_text)
                        };

                        if button.clicked() {
                            state.set_sort_by(sort_option.clone());
                        }
                    }

                    // Add some space between sort and view controls
                    ui.add_space(20.0);

                    // View mode controls
                    ui.label("视图：");

                    // View mode buttons
                    let view_options = [ViewMode::Detailed, ViewMode::List];

                    for view_option in &view_options {
                        let is_current = state.view_mode == *view_option;
                        let button_text = format!("{} {}", view_option.icon(), view_option.display_name());

                        let button = if is_current {
                            ui.add(egui::Button::new(button_text).fill(ui.visuals().selection.bg_fill))
                        } else {
                            ui.button(button_text)
                        };

                        if button.clicked() {
                            state.view_mode = view_option.clone();
                            // Save the view mode preference
                            state.save_search_options();
                        }
                    }
                });

                ui.add_space(5.0);
                ui.separator();

                // Search results with proper height constraint
                // Since we now have an outer scroll area, we can be more generous with the inner scroll area
                // but still need to reserve space for bottom statistics
                let remaining_height = ui.available_height(); // Reserve space for statistics
                egui::ScrollArea::vertical()
                    .max_height(remaining_height.max(200.0)) // Ensure minimum height of 200px
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        // Render results based on view mode
                        match state.view_mode {
                            ViewMode::Detailed => render_detailed_view(ui, state),
                            ViewMode::List => render_list_view(ui, state),
                        }
                    });
                        // Results will be rendered by the appropriate view function
            }
                });

    // Show file properties dialog if requested
    if state.show_properties_dialog {
        if let Some(file) = &state.properties_file.clone() {
            let file_path = file.path.clone();
            egui::Window::new("📋 文件属性")
                .collapsible(false)
                .resizable(false)
                .fixed_size([450.0, 500.0])
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.add_space(10.0);

                        // File icon and name with copy button
                        ui.horizontal(|ui| {
                            let icon = match file.file_type.as_str() {
                                "pdf" => "📄",
                                "doc" | "docx" => "📝",
                                "xls" | "xlsx" => "📊",
                                "ppt" | "pptx" => "📽",
                                "txt" | "md" => "📃",
                                "rs" | "js" | "py" | "cpp" => "💻",
                                "jpg" | "png" | "gif" => "🖼",
                                _ => "📁",
                            };
                            ui.label(egui::RichText::new(icon).size(24.0));
                            ui.add_space(8.0);

                            // File name with wrapping for long names
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&file.filename).heading());
                                    if ui.small_button("📋").on_hover_text("复制文件名").clicked() {
                                        state.copy_filename(file);
                                    }
                                });
                            });
                        });

                        ui.add_space(15.0);

                        // Properties grid with copy buttons
                        egui::Grid::new("file_properties")
                            .num_columns(3)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("名称:").strong());
                                ui.add(egui::Label::new(&file.filename).wrap());
                                if ui.small_button("📋").on_hover_text("复制文件名").clicked() {
                                    state.copy_filename(file);
                                }
                                ui.end_row();

                                ui.label(egui::RichText::new("路径:").strong());
                                ui.add(egui::Label::new(&file.path).wrap());
                                if ui.small_button("📋").on_hover_text("复制完整路径").clicked() {
                                    state.copy_path_and_name(file);
                                }
                                ui.end_row();

                                ui.label(egui::RichText::new("类型:").strong());
                                ui.label(&file.file_type.to_uppercase());
                                ui.label(""); // Empty cell for alignment
                                ui.end_row();

                                ui.label(egui::RichText::new("大小:").strong());
                                ui.label(format!("{:.1} KB ({} 字节)",
                                    file.size_bytes as f64 / 1024.0,
                                    file.size_bytes));
                                ui.label(""); // Empty cell for alignment
                                ui.end_row();

                                ui.label(egui::RichText::new("修改时间:").strong());
                                ui.label(file.modified.format("%Y-%m-%d %H:%M:%S").to_string());
                                ui.label(""); // Empty cell for alignment
                                ui.end_row();

                                ui.label(egui::RichText::new("搜索评分:").strong());
                                ui.label(format!("{:.2}", file.score));
                                ui.label(""); // Empty cell for alignment
                                ui.end_row();
                            });

                        ui.add_space(15.0);

                        // Content preview if available and enabled
                        if state.enable_content_preview && !file.content_preview.is_empty() {
                            ui.label(egui::RichText::new("内容预览:").strong());
                            ui.add_space(5.0);

                            egui::ScrollArea::vertical()
                                .max_height(100.0)
                                .show(ui, |ui| {
                                    ui.add(egui::Label::new(&file.content_preview).wrap());
                                });

                            ui.add_space(15.0);
                        }

                        // Action buttons
                        ui.horizontal(|ui| {
                            if ui.button("📂 打开文件夹").clicked() {
                                state.open_folder(&file_path);
                            }

                            if ui.button("📄 打开文件").clicked() {
                                state.open_file(&file_path);
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("关闭").clicked() {
                                    state.show_properties_dialog = false;
                                    state.properties_file = None;
                                }
                            });
                        });

                        ui.add_space(5.0);
                    });
                });
        }
    }

    // Directory input dialog (替代文件对话框)
    if state.show_directory_input_dialog {
        egui::Window::new("添加索引目录")
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("请输入要索引的目录路径：");
                    ui.add_space(5.0);

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut state.directory_input)
                            .hint_text("例如：/home/user/Documents")
                            .desired_width(ui.available_width())
                    );

                    // 自动聚焦输入框
                    if state.show_directory_input_dialog {
                        response.request_focus();
                    }

                    ui.add_space(10.0);

                    // 常用目录快捷按钮
                    ui.label("常用目录：");
                    ui.horizontal_wrapped(|ui| {
                        if let Some(home_dir) = dirs::home_dir() {
                            if ui.small_button("🏠 主目录").clicked() {
                                state.directory_input = home_dir.to_string_lossy().to_string();
                            }
                        }

                        if let Some(documents_dir) = dirs::document_dir() {
                            if ui.small_button("📄 文档").clicked() {
                                state.directory_input = documents_dir.to_string_lossy().to_string();
                            }
                        }

                        if let Some(downloads_dir) = dirs::download_dir() {
                            if ui.small_button("📥 下载").clicked() {
                                state.directory_input = downloads_dir.to_string_lossy().to_string();
                            }
                        }

                        if let Some(desktop_dir) = dirs::desktop_dir() {
                            if ui.small_button("🖥 桌面").clicked() {
                                state.directory_input = desktop_dir.to_string_lossy().to_string();
                            }
                        }
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(5.0);

                    // 按钮
                    ui.horizontal(|ui| {
                        if ui.button("添加").clicked() ||
                           (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                            state.add_directory_from_input();
                        }

                        if ui.button("取消").clicked() {
                            state.show_directory_input_dialog = false;
                            state.directory_input.clear();
                        }
                    });
                });
            });
    }
}

/// Render search results in detailed view (card-style)
fn render_detailed_view(ui: &mut egui::Ui, state: &mut ISearchState) {
    // Clone the results to avoid borrowing issues
    let results = state.search_results.clone();
    for result in &results {
        ui.push_id(&result.id, |ui| {
            // Create a frame for the search result item with hover effect
            let frame_response = egui::Frame::none()
                .inner_margin(egui::Margin::same(8.0))
                .rounding(egui::Rounding::same(4.0))
                .show(ui, |ui| {
                    // File name and icon
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

                        // File name with highlighting and copy button
                        let truncated_filename = utils::truncate_with_ellipsis(&result.filename, 40);

                        // Check if filename contains search terms for highlighting
                        if !state.search_query.trim().is_empty() {
                            let search_terms = utils::extract_search_terms(&state.search_query);
                            let filename_lower = truncated_filename.to_lowercase();
                            let has_match = search_terms.iter().any(|term| filename_lower.contains(&term.to_lowercase()));

                            if has_match && !search_terms.is_empty() {
                                // Create highlighted filename with heading style
                                let mut highlighted_job = utils::create_highlighted_rich_text(&truncated_filename, &search_terms);
                                // Apply heading style to the entire job
                                for section in &mut highlighted_job.sections {
                                    section.format.font_id = egui::FontId::new(18.0, egui::FontFamily::Proportional);
                                }
                                ui.add(egui::Label::new(highlighted_job));
                            } else {
                                ui.heading(truncated_filename);
                            }
                        } else {
                            ui.heading(truncated_filename);
                        }

                        if ui.small_button("📋").on_hover_text("复制文件名").clicked() {
                            state.copy_filename(result);
                        }

                        // File size, date, and score
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("{}", result.modified.format("%Y-%m-%d %H:%M")));
                            ui.label(format!("{:.1} KB", result.size_bytes as f64 / 1024.0));
                            // Uncomment the following line to display the score (for debugging)
                            // ui.label(format!("得分: {:.2}", result.score));
                        });
                    });

                    // File path with truncation and copy button
                    ui.horizontal(|ui| {
                        ui.label("📂");
                        let truncated_path = utils::truncate_with_ellipsis(&result.path, 60);
                        ui.label(truncated_path);
                        if ui.small_button("📋").on_hover_text("复制完整路径").clicked() {
                            state.copy_path_and_name(result);
                        }
                    });

                    // Content preview with truncation (only if enabled)
                    if state.enable_content_preview {
                        ui.add_space(4.0);
                        if result.content_preview.is_empty() {
                            ui.label(egui::RichText::new(format!("📝 [内容预览为空] - 文件类型: {}", result.file_type)).weak().italics());
                        } else if result.content_preview.contains("无法预览内容") {
                            // Special handling for non-previewable files
                            let icon = match result.file_type.as_str() {
                                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "ico" | "tiff" => "🖼",
                                "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" => "🎬",
                                "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" => "🎵",
                                "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => "📦",
                                "exe" | "msi" | "dmg" | "pkg" | "deb" | "rpm" => "⚙️",
                                _ => "📄",
                            };
                            ui.label(egui::RichText::new(format!("{} {}", icon, result.content_preview)).weak().italics());
                        } else {
                            // Regular content preview for previewable files with highlighting
                            let truncated_preview = utils::truncate_with_ellipsis(&result.content_preview, 300);

                            // Create highlighted rich text if we have search terms
                            if !state.search_query.trim().is_empty() {
                                let search_terms = utils::extract_search_terms(&state.search_query);
                                if !search_terms.is_empty() {
                                    // Create rich text with highlighting
                                    let highlighted_job = utils::create_highlighted_rich_text(&truncated_preview, &search_terms);

                                    ui.horizontal(|ui| {
                                        ui.label("📝");
                                        ui.add(egui::Label::new(highlighted_job).wrap());
                                        ui.label(format!("({}字符)", result.content_preview.chars().count()));
                                    });
                                } else {
                                    // Fallback to normal display
                                    ui.add(egui::Label::new(format!("📝 {} ({}字符)", truncated_preview, result.content_preview.chars().count())).wrap());
                                }
                            } else {
                                // No search terms, display normally
                                ui.add(egui::Label::new(format!("📝 {} ({}字符)", truncated_preview, result.content_preview.chars().count())).wrap());
                            }
                        }
                    } else {
                        // Debug: Show when content preview is disabled
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("📝 [内容预览已禁用]").weak().italics());
                    }

                    // Action buttons
                    ui.horizontal(|ui| {
                        // Open file button
                        if ui.button("打开文件").clicked() {
                            let path = result.path.clone();
                            state.open_file(&path);
                        }

                        // Open folder button
                        if ui.button("打开文件夹").clicked() {
                            let path = result.path.clone();
                            state.open_folder(&path);
                        }

                        // Add space to push the menu button to the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Context menu button - use a more reliable approach
                            ui.menu_button("...", |ui| {
                                ui.set_min_width(150.0);

                                // Properties button (first)
                                if ui.button("📋 属性").clicked() {
                                    state.show_file_properties(result);
                                    ui.close_menu();
                                }

                                ui.separator();

                                // Copy submenu (second)
                                ui.menu_button("📄 复制", |ui| {
                                    if ui.button("名称").clicked() {
                                        state.copy_filename(result);
                                        ui.close_menu();
                                    }
                                    if ui.button("路径").clicked() {
                                        state.copy_filepath(result);
                                        ui.close_menu();
                                    }
                                    if ui.button("路径+名称").clicked() {
                                        state.copy_path_and_name(result);
                                        ui.close_menu();
                                    }
                                });
                            });
                        });
                    });
                });

            // Add hover effect by painting a background when hovered
            if frame_response.response.hovered() {
                let rect = frame_response.response.rect;
                let hover_color = if ui.visuals().dark_mode {
                    egui::Color32::from_rgba_unmultiplied(100, 150, 255, 25) // Blue overlay for dark mode
                } else {
                    egui::Color32::from_rgba_unmultiplied(50, 100, 200, 20) // Blue overlay for light mode
                };
                ui.painter().rect_filled(rect, egui::Rounding::same(4.0), hover_color);

                // Add a subtle border when hovered
                let border_color = if ui.visuals().dark_mode {
                    egui::Color32::from_rgba_unmultiplied(150, 180, 255, 60)
                } else {
                    egui::Color32::from_rgba_unmultiplied(80, 120, 220, 80)
                };
                ui.painter().rect_stroke(rect, egui::Rounding::same(4.0), egui::Stroke::new(1.0, border_color));
            }

            ui.add_space(4.0);
            ui.separator();
        });
    }

    render_search_statistics(ui, state);
}

/// Render search results in list view (table-style) using egui_extras::TableBuilder
fn render_list_view(ui: &mut egui::Ui, state: &mut ISearchState) {
    // Clone the results to avoid borrowing issues
    let results = state.search_results.clone();

    if results.is_empty() {
        render_search_statistics(ui, state);
        return;
    }

    use egui_extras::{TableBuilder, Column};

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(180.0).at_most(250.0)) // 文件名列
        .column(Column::remainder().at_least(200.0)) // 路径列 - 使用剩余空间
        .column(Column::auto().at_least(80.0).at_most(100.0)) // 大小列
        .column(Column::auto().at_least(100.0).at_most(120.0)) // 修改时间列
        .column(Column::auto().at_least(120.0).at_most(150.0)) // 操作列
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("文件名");
            });
            header.col(|ui| {
                ui.strong("路径");
            });
            header.col(|ui| {
                ui.strong("大小");
            });
            header.col(|ui| {
                ui.strong("修改时间");
            });
            header.col(|ui| {
                ui.strong("操作");
            });
        })
        .body(|mut body| {
            for result in &results {
                body.row(22.0, |mut row| {
                    // File name column with icon
                    row.col(|ui| {
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

                            // File name with highlighting
                            let truncated_filename = utils::truncate_with_ellipsis(&result.filename, 25);
                            if !state.search_query.trim().is_empty() {
                                let search_terms = utils::extract_search_terms(&state.search_query);
                                let filename_lower = truncated_filename.to_lowercase();
                                let has_match = search_terms.iter().any(|term| filename_lower.contains(&term.to_lowercase()));

                                if has_match && !search_terms.is_empty() {
                                    let highlighted_job = utils::create_highlighted_rich_text(&truncated_filename, &search_terms);
                                    ui.add(egui::Label::new(highlighted_job));
                                } else {
                                    ui.label(truncated_filename);
                                }
                            } else {
                                ui.label(truncated_filename);
                            }
                        });
                    });

                    // File path column
                    row.col(|ui| {
                        let truncated_path = utils::truncate_with_ellipsis(&result.path, 50);
                        ui.label(truncated_path).on_hover_text(&result.path);
                    });

                    // File size column
                    row.col(|ui| {
                        let size_text = if result.size_bytes >= 1024 * 1024 {
                            format!("{:.1} MB", result.size_bytes as f64 / (1024.0 * 1024.0))
                        } else {
                            format!("{:.1} KB", result.size_bytes as f64 / 1024.0)
                        };
                        ui.label(size_text);
                    });

                    // Modified time column
                    row.col(|ui| {
                        ui.label(result.modified.format("%m-%d %H:%M").to_string());
                    });

                    // Action buttons column
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // Open file button
                            if ui.small_button("📂").on_hover_text("打开文件").clicked() {
                                let path = result.path.clone();
                                state.open_file(&path);
                            }

                            // Open folder button
                            if ui.small_button("📁").on_hover_text("打开文件夹").clicked() {
                                let path = result.path.clone();
                                state.open_folder(&path);
                            }

                            // Context menu
                            ui.menu_button("⋯", |ui| {
                                ui.set_min_width(150.0);

                                if ui.button("📋 属性").clicked() {
                                    state.show_file_properties(result);
                                    ui.close_menu();
                                }

                                ui.separator();

                                if ui.button("📄 复制名称").clicked() {
                                    state.copy_filename(result);
                                    ui.close_menu();
                                }

                                if ui.button("📂 复制路径").clicked() {
                                    state.copy_filepath(result);
                                    ui.close_menu();
                                }

                                if ui.button("📋 复制完整路径").clicked() {
                                    state.copy_path_and_name(result);
                                    ui.close_menu();
                                }
                            });
                        });
                    });
                });
            }
        });

    ui.add_space(10.0);
    render_search_statistics(ui, state);
}

/// Render search statistics at the bottom
fn render_search_statistics(ui: &mut egui::Ui, state: &ISearchState) {
    // Add some spacing before bottom statistics
    ui.add_space(10.0);

    // Only show additional info that's not in the title
    ui.horizontal(|ui| {
        if state.has_more_results {
            ui.label(egui::RichText::new("显示前 100 条结果").small().weak());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("请使用更精确的搜索条件缩小结果范围").small().weak());
            });
        } else {
            // Show query time when not showing "more results" message
            ui.label(egui::RichText::new(format!(
                "查询时间: {}",
                state.search_stats.query_time.format("%Y-%m-%d %H:%M:%S")
            )).small().weak());
        }
    });
}