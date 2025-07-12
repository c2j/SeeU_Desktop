//! 主UI渲染

use egui;
use crate::state::IFileEditorState;
use super::{file_tree, tabs};

/// 渲染文件编辑器主界面
pub fn render_file_editor(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 确保已初始化
    state.ensure_initialized();

    // 处理异步加载消息（每帧调用）
    state.process_async_messages();

    // 获取可用的完整高度
    let available_height = ui.available_height();

    // 主要布局：水平分割，确保撑满高度
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), available_height),
        egui::Layout::left_to_right(egui::Align::TOP),
        |ui| {
            // 左侧文件树
            if state.ui_state.show_file_tree {
                ui.allocate_ui_with_layout(
                    egui::vec2(state.ui_state.file_tree_width, available_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        // 设置固定宽度，防止被长文件名撑宽
                        ui.set_max_width(state.ui_state.file_tree_width);
                        ui.set_min_width(state.ui_state.file_tree_width);
                        ui.set_min_height(available_height);
                        file_tree::render_file_tree(ui, state);
                    }
                );

                // 分隔符
                ui.separator();
            }

            // 右侧编辑区域
            let remaining_width = ui.available_width();
            ui.allocate_ui_with_layout(
                egui::vec2(remaining_width, available_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    // 设置最小高度确保撑满
                    ui.set_min_height(available_height);
                    render_editor_area(ui, state);
                }
            );
        }
    );

    // 渲染对话框（在主UI之后，确保在最上层）
    let ctx = ui.ctx();

    // 渲染查找替换对话框
    if state.ui_state.show_find_replace {
        render_find_replace_dialog(ctx, state);
    }

    // 渲染保存确认对话框
    if state.ui_state.show_save_confirmation {
        render_save_confirmation_dialog(ctx, state);
    }

    // 渲染文件操作对话框
    if state.ui_state.show_new_file_dialog {
        render_new_file_dialog(ctx, state);
    }

    if state.ui_state.show_new_folder_dialog {
        render_new_folder_dialog(ctx, state);
    }

    if state.ui_state.show_rename_dialog {
        render_rename_dialog(ctx, state);
    }

    if state.ui_state.show_delete_confirmation {
        render_delete_confirmation_dialog(ctx, state);
    }

    // 渲染上下文菜单
    if state.ui_state.show_context_menu {
        render_context_menu_dialog(ctx, state);
    }
}

/// 渲染编辑器区域
fn render_editor_area(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    let available_height = ui.available_height();

    ui.vertical(|ui| {
        // 合并的工具栏和标签页
        tabs::render_tabs_with_toolbar(ui, state);
        ui.separator();

        // 计算编辑器内容区域的高度（减去合并工具栏的高度）
        let toolbar_height = 40.0; // 合并工具栏大概高度
        let content_height = available_height - toolbar_height;

        // 显示错误信息（如果有）
        let mut should_clear_error = false;
        if let Some(error) = &state.last_error {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::RED, "❌");
                ui.colored_label(egui::Color32::RED, format!("错误: {}", error));
                if ui.small_button("✕").on_hover_text("关闭错误提示").clicked() {
                    should_clear_error = true;
                }
            });
            ui.separator();
        }
        if should_clear_error {
            state.last_error = None;
        }

        // 编辑器内容区域，确保撑满剩余高度
        let error_height = if state.last_error.is_some() { 30.0 } else { 0.0 };
        let adjusted_content_height = content_height - error_height;

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), adjusted_content_height.max(100.0)),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.set_min_height(adjusted_content_height.max(100.0));
                let has_active_buffer = state.editor.get_active_buffer().is_some();
                log::debug!("WORD_WRAP_DIAGNOSIS: has_active_buffer={}, buffer_count={}",
                           has_active_buffer, state.editor.buffers.len());

                if has_active_buffer {
                    crate::ui::code_editor::render_code_editor(ui, state);
                } else {
                    render_welcome_screen(ui, state);
                }
            }
        );
    });
}

// 原工具栏函数已合并到标签页组件中
// 保留此函数以防需要单独使用
#[allow(dead_code)]
fn render_toolbar_legacy(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    ui.horizontal(|ui| {
        // 文件操作按钮
        if ui.button("📁 打开").clicked() {
            state.open_file_dialog();
        }

        if ui.button("💾 保存").clicked() {
            if let Err(e) = state.editor.save_active_file() {
                log::error!("Failed to save file: {}", e);
                state.last_error = Some(e);
            } else {
                log::info!("File saved successfully");
            }
        }

        if ui.button("📂 文件夹").clicked() {
            state.open_folder_dialog();
        }

        if ui.button("📄 新建").clicked() {
            create_new_file(state);
        }

        ui.separator();

        // 编辑操作按钮
        if ui.button("↶ 撤销").clicked() {
            undo_edit(state);
        }

        if ui.button("↷ 重做").clicked() {
            redo_edit(state);
        }

        if ui.button("🔍 查找").clicked() {
            state.ui_state.show_find_replace = !state.ui_state.show_find_replace;
        }

        ui.separator();

        // 视图切换按钮
        let tree_button_text = if state.ui_state.show_file_tree {
            "🗂️ 隐藏文件树"
        } else {
            "🗂️ 显示文件树"
        };

        if ui.button(tree_button_text).clicked() {
            state.ui_state.show_file_tree = !state.ui_state.show_file_tree;
            log::info!("File tree visibility toggled: {}", state.ui_state.show_file_tree);
        }

        ui.separator();

        // 搜索框
        ui.label("🔍");
        ui.text_edit_singleline(&mut state.ui_state.search_query);

        if ui.button("搜索").clicked() {
            state.ui_state.show_search = !state.ui_state.search_query.is_empty();
        }
    });

    ui.separator();
}

/// 渲染欢迎屏幕
fn render_welcome_screen(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);

            ui.heading("📝 文件编辑器");
            ui.add_space(20.0);

            // 根据是否首次使用显示不同的内容
            if state.is_first_time_use() {
                // 首次使用的欢迎界面
                ui.label("欢迎使用文件编辑器！");
                ui.add_space(10.0);
                ui.label("请选择一个工作目录开始使用：");
                ui.add_space(20.0);

                // 主要操作按钮
                if ui.add_sized([200.0, 40.0], egui::Button::new("📂 选择工作目录")).clicked() {
                    show_directory_picker_for_welcome(state);
                }

                ui.add_space(15.0);

                // 快速选择按钮
                ui.horizontal(|ui| {
                    if ui.button("🏠 用户目录").clicked() {
                        if let Some(home_dir) = dirs::home_dir() {
                            if let Err(e) = state.set_file_tree_root(home_dir) {
                                log::error!("Failed to set workspace: {}", e);
                            }
                        }
                    }

                    if ui.button("🖥️ 桌面").clicked() {
                        if let Some(desktop_dir) = dirs::desktop_dir() {
                            if let Err(e) = state.set_file_tree_root(desktop_dir) {
                                log::error!("Failed to set workspace: {}", e);
                            }
                        }
                    }

                    if ui.button("📄 文档").clicked() {
                        if let Some(documents_dir) = dirs::document_dir() {
                            if let Err(e) = state.set_file_tree_root(documents_dir) {
                                log::error!("Failed to set workspace: {}", e);
                            }
                        }
                    }
                });

            } else {
                // 已有工作区的界面
                ui.label("选择一个文件开始编辑");
                ui.add_space(10.0);

                if ui.button("📁 打开文件").clicked() {
                    state.open_file_dialog();
                }

                ui.add_space(20.0);

                // 显示当前工作区
                if let Some(workspace) = &state.workspace_root {
                    ui.label(format!("当前工作区: {}", workspace.display()));
                    ui.add_space(10.0);

                    if ui.button("📂 更换工作目录").clicked() {
                        show_directory_picker_for_welcome(state);
                    }
                }

                // 快速操作
                ui.collapsing("快速操作", |ui| {
                    if ui.button("设置工作区到当前目录").clicked() {
                        if let Ok(current_dir) = std::env::current_dir() {
                            if let Err(e) = state.set_workspace(current_dir) {
                                log::error!("Failed to set workspace: {}", e);
                            }
                        }
                    }

                    if ui.button("刷新文件树").clicked() {
                        if let Err(e) = state.file_tree.refresh(&state.settings) {
                            log::error!("Failed to refresh file tree: {}", e);
                        }
                    }
                });
            }
        });
    });
}

/// 显示目录选择器（用于欢迎屏幕）
fn show_directory_picker_for_welcome(state: &mut IFileEditorState) {
    log::info!("Opening directory picker from welcome screen...");

    // 尝试使用系统文件对话框
    match rfd::FileDialog::new().pick_folder() {
        Some(path) => {
            log::info!("Selected directory: {:?}", path);
            if let Err(e) = state.set_file_tree_root(path) {
                log::error!("Failed to set root directory: {}", e);
                state.last_error = Some(e);
            }
        }
        None => {
            log::info!("Directory selection cancelled or failed, showing input dialog");
            // 如果系统对话框失败或取消，显示输入框
            state.ui_state.show_directory_picker = true;
            // 设置当前目录作为默认值
            if let Some(current_dir) = std::env::current_dir().ok() {
                state.ui_state.directory_input = current_dir.to_string_lossy().to_string();
            }
        }
    }
}

/// 创建新文件
fn create_new_file(state: &mut IFileEditorState) {
    // 创建新文件对话框
    let initial_dir = state.workspace_root.clone()
        .or_else(|| dirs::home_dir())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let file_dialog = rfd::FileDialog::new()
        .set_title("新建文件")
        .set_directory(&initial_dir)
        .set_file_name("untitled.txt");

    if let Some(path) = file_dialog.save_file() {
        // 创建空文件
        if let Err(e) = std::fs::write(&path, "") {
            log::error!("Failed to create new file: {}", e);
            state.last_error = Some(crate::FileEditorError::IoError(e));
        } else {
            // 打开新创建的文件
            if let Err(e) = state.editor.open_file(path.clone(), &state.settings, state.async_load_sender.clone()) {
                log::error!("Failed to open new file: {}", e);
                state.last_error = Some(e);
            } else {
                log::info!("Created and opened new file: {:?}", path);
            }
        }
    }
}

/// 撤销编辑
fn undo_edit(state: &mut IFileEditorState) {
    if let Some(active_tab) = state.editor.active_tab {
        if let Some(path) = state.editor.tabs.get(active_tab).cloned() {
            if let Some(undo_stack) = state.editor.undo_stack.get_mut(&path) {
                if let Some(operation) = undo_stack.operations.pop() {
                    // 应用撤销操作
                    if let Some(buffer) = state.editor.buffers.get_mut(&path) {
                        apply_undo_operation(buffer, operation);
                        log::info!("Undo operation applied");
                    }
                } else {
                    log::info!("No more operations to undo");
                }
            }
        }
    }
}

/// 重做编辑
fn redo_edit(_state: &mut IFileEditorState) {
    // TODO: 实现重做功能（需要重做栈）
    log::info!("Redo operation requested (not implemented yet)");
}

/// 应用撤销操作
fn apply_undo_operation(buffer: &mut crate::state::TextBuffer, operation: crate::state::EditOperation) {
    use crate::state::OperationType;

    match operation.operation_type {
        OperationType::Insert => {
            // 撤销插入：删除文本
            buffer.delete_text(operation.range);
        }
        OperationType::Delete => {
            // 撤销删除：插入文本
            buffer.insert_text(operation.range.start, &operation.text);
        }
        OperationType::Replace => {
            // 撤销替换：恢复原文本
            buffer.replace_text(operation.range, &operation.text);
        }
    }

    // 恢复光标位置
    buffer.cursor = operation.cursor_before;
}

/// 渲染查找替换对话框
fn render_find_replace_dialog(ctx: &egui::Context, state: &mut IFileEditorState) {
    egui::Window::new("查找和替换")
        .collapsible(false)
        .resizable(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // 查找输入框
                ui.horizontal(|ui| {
                    ui.label("查找:");
                    ui.text_edit_singleline(&mut state.ui_state.find_query);
                });

                // 替换输入框
                ui.horizontal(|ui| {
                    ui.label("替换:");
                    ui.text_edit_singleline(&mut state.ui_state.replace_query);
                });

                ui.separator();

                // 操作按钮
                ui.horizontal(|ui| {
                    if ui.button("查找下一个").clicked() {
                        find_next(state);
                    }

                    if ui.button("替换").clicked() {
                        replace_current(state);
                    }

                    if ui.button("全部替换").clicked() {
                        replace_all(state);
                    }

                    if ui.button("关闭").clicked() {
                        state.ui_state.show_find_replace = false;
                    }
                });
            });
        });
}

/// 渲染保存确认对话框
fn render_save_confirmation_dialog(ctx: &egui::Context, state: &mut IFileEditorState) {
    egui::Window::new("保存确认")
        .collapsible(false)
        .resizable(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("以下文件有未保存的更改:");
                ui.separator();

                // 显示未保存的文件列表
                for file_path in &state.ui_state.save_confirmation_files.clone() {
                    ui.horizontal(|ui| {
                        ui.label("📄");
                        ui.label(file_path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("未知文件"));
                    });
                }

                ui.separator();

                // 操作按钮
                ui.horizontal(|ui| {
                    if ui.button("保存全部").clicked() {
                        save_all_and_close(state);
                    }

                    if ui.button("不保存").clicked() {
                        discard_changes_and_close(state);
                    }

                    if ui.button("取消").clicked() {
                        state.ui_state.show_save_confirmation = false;
                        state.ui_state.save_confirmation_files.clear();
                    }
                });
            });
        });
}

/// 查找下一个
fn find_next(state: &mut IFileEditorState) {
    let query = &state.ui_state.find_query;
    if query.is_empty() {
        return;
    }

    if let Some(buffer) = state.editor.get_active_buffer() {
        let text = buffer.rope.to_string();
        let current_pos = buffer.cursor.byte_offset;

        if let Some(pos) = text[current_pos..].find(query) {
            let found_pos = current_pos + pos;
            log::info!("Found '{}' at position {}", query, found_pos);
            // TODO: 移动光标到找到的位置
        } else {
            // 从头开始查找
            if let Some(pos) = text.find(query) {
                log::info!("Found '{}' at position {} (wrapped)", query, pos);
                // TODO: 移动光标到找到的位置
            } else {
                log::info!("'{}' not found", query);
            }
        }
    }
}

/// 替换当前选中的文本
fn replace_current(state: &mut IFileEditorState) {
    let find_query = &state.ui_state.find_query.clone();
    let replace_query = &state.ui_state.replace_query.clone();

    if find_query.is_empty() {
        return;
    }

    // TODO: 实现替换当前选中的文本
    log::info!("Replace '{}' with '{}'", find_query, replace_query);
}

/// 全部替换
fn replace_all(state: &mut IFileEditorState) {
    let find_query = &state.ui_state.find_query.clone();
    let replace_query = &state.ui_state.replace_query.clone();

    if find_query.is_empty() {
        return;
    }

    if let Some(active_tab) = state.editor.active_tab {
        if let Some(path) = state.editor.tabs.get(active_tab).cloned() {
            if let Some(buffer) = state.editor.buffers.get_mut(&path) {
                let text = buffer.rope.to_string();
                let new_text = text.replace(find_query, replace_query);

                if text != new_text {
                    // 替换整个文本内容
                    buffer.rope = crop::Rope::from(new_text);
                    buffer.modified = true;
                    log::info!("Replaced all occurrences of '{}' with '{}'", find_query, replace_query);
                } else {
                    log::info!("No occurrences of '{}' found", find_query);
                }
            }
        }
    }
}

/// 保存全部并关闭
fn save_all_and_close(state: &mut IFileEditorState) {
    for file_path in &state.ui_state.save_confirmation_files.clone() {
        if let Err(e) = state.editor.save_file(file_path) {
            log::error!("Failed to save file {:?}: {}", file_path, e);
        }
    }

    state.ui_state.show_save_confirmation = false;
    state.ui_state.save_confirmation_files.clear();
}

/// 丢弃更改并关闭
fn discard_changes_and_close(state: &mut IFileEditorState) {
    // 关闭所有未保存的文件
    for file_path in &state.ui_state.save_confirmation_files.clone() {
        if let Err(e) = state.editor.close_file(file_path) {
            log::error!("Failed to close file {:?}: {}", file_path, e);
        }
    }

    state.ui_state.show_save_confirmation = false;
    state.ui_state.save_confirmation_files.clear();
}

/// 渲染新建文件对话框
fn render_new_file_dialog(ctx: &egui::Context, state: &mut IFileEditorState) {
    egui::Window::new("新建文件")
        .collapsible(false)
        .resizable(false)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("文件名:");
                ui.text_edit_singleline(&mut state.ui_state.new_file_name);

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("创建").clicked() {
                        create_new_file_in_directory(state);
                    }

                    if ui.button("取消").clicked() {
                        state.ui_state.show_new_file_dialog = false;
                        state.ui_state.new_file_name.clear();
                        state.ui_state.operation_target_path = None;
                    }
                });
            });
        });
}

/// 渲染新建文件夹对话框
fn render_new_folder_dialog(ctx: &egui::Context, state: &mut IFileEditorState) {
    egui::Window::new("新建文件夹")
        .collapsible(false)
        .resizable(false)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("文件夹名:");
                ui.text_edit_singleline(&mut state.ui_state.new_folder_name);

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("创建").clicked() {
                        create_new_folder_in_directory(state);
                    }

                    if ui.button("取消").clicked() {
                        state.ui_state.show_new_folder_dialog = false;
                        state.ui_state.new_folder_name.clear();
                        state.ui_state.operation_target_path = None;
                    }
                });
            });
        });
}

/// 渲染重命名对话框
fn render_rename_dialog(ctx: &egui::Context, state: &mut IFileEditorState) {
    egui::Window::new("重命名")
        .collapsible(false)
        .resizable(false)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("新名称:");
                ui.text_edit_singleline(&mut state.ui_state.rename_new_name);

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("重命名").clicked() {
                        rename_file_or_folder(state);
                    }

                    if ui.button("取消").clicked() {
                        state.ui_state.show_rename_dialog = false;
                        state.ui_state.rename_new_name.clear();
                        state.ui_state.operation_target_path = None;
                    }
                });
            });
        });
}

/// 渲染删除确认对话框
fn render_delete_confirmation_dialog(ctx: &egui::Context, state: &mut IFileEditorState) {
    egui::Window::new("确认删除")
        .collapsible(false)
        .resizable(false)
        .default_width(350.0)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                if let Some(path) = &state.ui_state.operation_target_path {
                    ui.label(format!("确定要删除以下项目吗？"));
                    ui.separator();
                    ui.horizontal(|ui| {
                        if path.is_dir() {
                            ui.label("📁");
                        } else {
                            ui.label("📄");
                        }
                        ui.label(path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("未知"));
                    });
                    ui.small(format!("路径: {}", path.display()));
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("删除").clicked() {
                        delete_file_or_folder(state);
                    }

                    if ui.button("取消").clicked() {
                        state.ui_state.show_delete_confirmation = false;
                        state.ui_state.operation_target_path = None;
                    }
                });
            });
        });
}

/// 在指定目录中创建新文件
fn create_new_file_in_directory(state: &mut IFileEditorState) {
    if let Some(parent_dir) = &state.ui_state.operation_target_path.clone() {
        let file_name = &state.ui_state.new_file_name;

        if file_name.is_empty() {
            log::error!("File name cannot be empty");
            return;
        }

        let file_path = parent_dir.join(file_name);

        // 检查文件是否已存在
        if file_path.exists() {
            log::error!("File already exists: {:?}", file_path);
            return;
        }

        // 创建文件
        if let Err(e) = std::fs::write(&file_path, "") {
            log::error!("Failed to create file: {}", e);
            state.last_error = Some(crate::FileEditorError::IoError(e));
        } else {
            log::info!("Created new file: {:?}", file_path);

            // 刷新文件树
            if let Err(e) = state.file_tree.refresh(&state.settings) {
                log::error!("Failed to refresh file tree: {}", e);
            }

            // 打开新创建的文件
            if let Err(e) = state.editor.open_file(file_path, &state.settings, state.async_load_sender.clone()) {
                log::error!("Failed to open new file: {}", e);
            }
        }
    }

    // 关闭对话框
    state.ui_state.show_new_file_dialog = false;
    state.ui_state.new_file_name.clear();
    state.ui_state.operation_target_path = None;
}

/// 在指定目录中创建新文件夹
fn create_new_folder_in_directory(state: &mut IFileEditorState) {
    if let Some(parent_dir) = &state.ui_state.operation_target_path.clone() {
        let folder_name = &state.ui_state.new_folder_name;

        if folder_name.is_empty() {
            log::error!("Folder name cannot be empty");
            return;
        }

        let folder_path = parent_dir.join(folder_name);

        // 检查文件夹是否已存在
        if folder_path.exists() {
            log::error!("Folder already exists: {:?}", folder_path);
            return;
        }

        // 创建文件夹
        if let Err(e) = std::fs::create_dir(&folder_path) {
            log::error!("Failed to create folder: {}", e);
            state.last_error = Some(crate::FileEditorError::IoError(e));
        } else {
            log::info!("Created new folder: {:?}", folder_path);

            // 刷新文件树
            if let Err(e) = state.file_tree.refresh(&state.settings) {
                log::error!("Failed to refresh file tree: {}", e);
            }
        }
    }

    // 关闭对话框
    state.ui_state.show_new_folder_dialog = false;
    state.ui_state.new_folder_name.clear();
    state.ui_state.operation_target_path = None;
}

/// 重命名文件或文件夹
fn rename_file_or_folder(state: &mut IFileEditorState) {
    if let Some(old_path) = &state.ui_state.operation_target_path.clone() {
        let new_name = &state.ui_state.rename_new_name;

        if new_name.is_empty() {
            log::error!("New name cannot be empty");
            return;
        }

        if let Some(parent) = old_path.parent() {
            let new_path = parent.join(new_name);

            // 检查新路径是否已存在
            if new_path.exists() && new_path != *old_path {
                log::error!("Target already exists: {:?}", new_path);
                return;
            }

            // 执行重命名
            if let Err(e) = std::fs::rename(old_path, &new_path) {
                log::error!("Failed to rename: {}", e);
                state.last_error = Some(crate::FileEditorError::IoError(e));
            } else {
                log::info!("Renamed {:?} to {:?}", old_path, new_path);

                // 如果重命名的是已打开的文件，更新编辑器状态
                if old_path.is_file() && state.editor.buffers.contains_key(old_path) {
                    // 更新编辑器中的文件路径
                    if let Some(buffer) = state.editor.buffers.remove(old_path) {
                        state.editor.buffers.insert(new_path.clone(), buffer);
                    }

                    // 更新标签页
                    for tab_path in &mut state.editor.tabs {
                        if tab_path == old_path {
                            *tab_path = new_path.clone();
                            break;
                        }
                    }
                }

                // 刷新文件树
                if let Err(e) = state.file_tree.refresh(&state.settings) {
                    log::error!("Failed to refresh file tree: {}", e);
                }
            }
        }
    }

    // 关闭对话框
    state.ui_state.show_rename_dialog = false;
    state.ui_state.rename_new_name.clear();
    state.ui_state.operation_target_path = None;
}

/// 渲染上下文菜单对话框
fn render_context_menu_dialog(ctx: &egui::Context, state: &mut IFileEditorState) {
    if let Some(path) = &state.ui_state.context_menu_path.clone() {
        let title = if state.ui_state.context_menu_is_dir {
            "目录操作"
        } else {
            "文件操作"
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    if state.ui_state.context_menu_is_dir {
                        // 目录上下文菜单
                        if ui.button("📄 新建文件").clicked() {
                            state.ui_state.show_new_file_dialog = true;
                            state.ui_state.operation_target_path = Some(path.clone());
                            state.ui_state.new_file_name.clear();
                            state.ui_state.show_context_menu = false;
                        }

                        if ui.button("📁 新建文件夹").clicked() {
                            state.ui_state.show_new_folder_dialog = true;
                            state.ui_state.operation_target_path = Some(path.clone());
                            state.ui_state.new_folder_name.clear();
                            state.ui_state.show_context_menu = false;
                        }

                        ui.separator();
                    } else {
                        // 文件上下文菜单
                        if ui.button("📝 打开").clicked() {
                            if let Err(e) = state.editor.open_file(path.clone(), &state.settings, state.async_load_sender.clone()) {
                                log::error!("Failed to open file: {}", e);
                                state.last_error = Some(e);
                            }
                            state.ui_state.show_context_menu = false;
                        }

                        ui.separator();
                    }

                    if ui.button("📋 复制路径").clicked() {
                        copy_path_to_clipboard(path);
                        state.ui_state.show_context_menu = false;
                    }

                    if ui.button("✏️ 重命名").clicked() {
                        state.ui_state.show_rename_dialog = true;
                        state.ui_state.operation_target_path = Some(path.clone());
                        state.ui_state.rename_new_name = path.file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("")
                            .to_string();
                        state.ui_state.show_context_menu = false;
                    }

                    ui.separator();

                    if ui.button("🗑️ 删除").clicked() {
                        state.ui_state.show_delete_confirmation = true;
                        state.ui_state.operation_target_path = Some(path.clone());
                        state.ui_state.show_context_menu = false;
                    }

                    ui.separator();

                    if ui.button("取消").clicked() {
                        state.ui_state.show_context_menu = false;
                    }
                });
            });
    }
}

/// 复制路径到剪贴板
fn copy_path_to_clipboard(path: &std::path::PathBuf) {
    let path_str = path.to_string_lossy().to_string();

    // 使用 arboard 复制到剪贴板
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Err(e) = clipboard.set_text(&path_str) {
            log::error!("Failed to copy path to clipboard: {}", e);
        } else {
            log::info!("Copied path to clipboard: {}", path_str);
        }
    } else {
        log::error!("Failed to access clipboard");
    }
}

/// 删除文件或文件夹
fn delete_file_or_folder(state: &mut IFileEditorState) {
    if let Some(path) = &state.ui_state.operation_target_path.clone() {
        let result = if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };

        match result {
            Ok(()) => {
                log::info!("Deleted: {:?}", path);

                // 如果删除的是已打开的文件，关闭它
                if path.is_file() && state.editor.buffers.contains_key(path) {
                    if let Err(e) = state.editor.close_file(path) {
                        log::error!("Failed to close deleted file: {}", e);
                    }
                }

                // 刷新文件树
                if let Err(e) = state.file_tree.refresh(&state.settings) {
                    log::error!("Failed to refresh file tree: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to delete: {}", e);
                state.last_error = Some(crate::FileEditorError::IoError(e));
            }
        }
    }

    // 关闭对话框
    state.ui_state.show_delete_confirmation = false;
    state.ui_state.operation_target_path = None;
}
