//! 标签页UI组件

use egui;
use crate::state::IFileEditorState;

/// 渲染标签页和工具栏（合并到一行）
pub fn render_tabs_with_toolbar(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 检查是否有活动文件
    let has_active_file = state.editor.get_active_buffer().is_some();

    ui.horizontal(|ui| {
        // 左侧：文件操作按钮
        if has_active_file {
            render_file_operation_buttons_icon_only(ui, state);
        } else {
            render_basic_file_operations(ui, state);
        }

        // 中间：标签页（如果有打开的文件）
        if !state.editor.tabs.is_empty() {
            ui.separator();

            // 使用可用空间的一部分来显示标签页
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width() * 0.6, ui.available_height()),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    render_tab_buttons(ui, state);
                }
            );
        }

        // 右侧：编辑和视图操作按钮
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            render_view_operation_buttons(ui, state);

            // 只有在有活动文件时才显示编辑操作按钮
            if has_active_file {
                ui.separator();
                render_edit_operation_buttons_icon_only(ui, state);
            }
        });
    });
}

/// 渲染基本文件操作按钮（没有文件打开时）
fn render_basic_file_operations(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 只显示打开和新建按钮
    if ui.button("📁 打开").clicked() {
        state.open_file_dialog();
    }

    if ui.button("📂 文件夹").clicked() {
        state.open_folder_dialog();
    }

    if ui.button("📄 新建").clicked() {
        create_new_file(state);
    }
}

/// 渲染文件操作按钮（仅图标版本，有文件打开时）
fn render_file_operation_buttons_icon_only(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 文件操作按钮（仅图标风格）
    if ui.small_button("💾").on_hover_text("保存").clicked() {
        if let Err(e) = state.editor.save_active_file() {
            log::error!("Failed to save file: {}", e);
            state.last_error = Some(e);
        } else {
            log::info!("File saved successfully");
        }
    }

    if ui.small_button("📁").on_hover_text("打开").clicked() {
        state.open_file_dialog();
    }

    if ui.small_button("📄").on_hover_text("新建").clicked() {
        create_new_file(state);
    }
}

/// 渲染文件操作按钮（原版本，保留以备用）
#[allow(dead_code)]
fn render_file_operation_buttons(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 文件操作按钮（图标+文字风格）
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
}

/// 渲染编辑操作按钮（仅图标版本）
fn render_edit_operation_buttons_icon_only(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 编辑操作按钮（仅图标）
    if ui.small_button("↶").on_hover_text("撤销").clicked() {
        undo_edit(state);
    }

    if ui.small_button("↷").on_hover_text("重做").clicked() {
        redo_edit(state);
    }

    if ui.small_button("🔍").on_hover_text("查找").clicked() {
        state.ui_state.show_find_replace = !state.ui_state.show_find_replace;
    }
}

/// 渲染编辑操作按钮（原版本，保留以备用）
#[allow(dead_code)]
fn render_edit_operation_buttons(ui: &mut egui::Ui, state: &mut IFileEditorState) {
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
}

/// 渲染视图操作按钮
fn render_view_operation_buttons(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    // 视图切换按钮
    let tree_button_text = if state.ui_state.show_file_tree {
        "🗂️ 隐藏树"
    } else {
        "🗂️ 显示树"
    };

    if ui.button(tree_button_text).clicked() {
        state.ui_state.show_file_tree = !state.ui_state.show_file_tree;
        log::info!("File tree visibility toggled: {}", state.ui_state.show_file_tree);
    }

    // 标签页操作按钮
    if !state.editor.tabs.is_empty() {
        if ui.small_button("×").on_hover_text("关闭所有").clicked() {
            close_all_tabs(state);
        }

        if ui.small_button("⊗").on_hover_text("关闭其他").clicked() {
            close_other_tabs(state);
        }
    }
}

/// 渲染标签页按钮
fn render_tab_buttons(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    let mut tab_to_close = None;
    let mut new_active_tab = state.editor.active_tab;

    // 使用滚动区域来处理大量标签页，但更紧凑
    egui::ScrollArea::horizontal()
        .id_source("tab_scroll")
        .auto_shrink([true, true])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (index, tab_path) in state.editor.tabs.iter().enumerate() {
                    let is_active = state.editor.active_tab == Some(index);
                    let buffer = state.editor.buffers.get(tab_path);

                    // 获取文件名
                    let file_name = tab_path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("未知文件");

                    // 检查是否已修改
                    let is_modified = buffer.map(|b| b.modified).unwrap_or(false);

                    // 更紧凑的标签页按钮
                    ui.horizontal(|ui| {
                        // 标签页按钮
                        let tab_text = if is_modified {
                            format!("● {}", file_name)
                        } else {
                            file_name.to_string()
                        };

                        let tab_button = egui::Button::new(&tab_text)
                            .selected(is_active)
                            .min_size(egui::vec2(50.0, 18.0));

                        let tab_response = ui.add(tab_button);

                        // 处理标签页点击
                        if tab_response.clicked() {
                            new_active_tab = Some(index);
                        }

                        // 标签页右键菜单
                        tab_response.context_menu(|ui| {
                            render_tab_context_menu(ui, index, tab_path, &mut tab_to_close);
                        });

                        // 关闭按钮
                        let close_button = egui::Button::new("×")
                            .small()
                            .fill(egui::Color32::TRANSPARENT);

                        if ui.add(close_button).clicked() {
                            tab_to_close = Some(index);
                        }
                    });

                    // 标签页之间的小间距
                    if index < state.editor.tabs.len() - 1 {
                        ui.add_space(1.0);
                    }
                }
            });
        });

    // 应用标签页切换
    if new_active_tab != state.editor.active_tab {
        state.editor.active_tab = new_active_tab;
    }

    // 处理标签页关闭
    if let Some(close_index) = tab_to_close {
        close_tab(state, close_index);
    }
}

/// 渲染标签页（保持向后兼容）
pub fn render_tabs(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    if state.editor.tabs.is_empty() {
        return;
    }

    render_tab_buttons(ui, state);
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
            if let Err(e) = state.editor.open_file(path.clone(), &state.settings) {
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
fn redo_edit(state: &mut IFileEditorState) {
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

/// 渲染标签页右键菜单
fn render_tab_context_menu(
    ui: &mut egui::Ui,
    tab_index: usize,
    tab_path: &std::path::PathBuf,
    tab_to_close: &mut Option<usize>,
) {
    if ui.button("关闭").clicked() {
        *tab_to_close = Some(tab_index);
        ui.close_menu();
    }
    
    if ui.button("关闭其他").clicked() {
        // TODO: 实现关闭其他标签页
        log::info!("Close other tabs except: {:?}", tab_path);
        ui.close_menu();
    }
    
    if ui.button("关闭所有").clicked() {
        // TODO: 实现关闭所有标签页
        log::info!("Close all tabs");
        ui.close_menu();
    }
    
    ui.separator();
    
    if ui.button("复制路径").clicked() {
        copy_path_to_clipboard(tab_path);
        ui.close_menu();
    }

    if ui.button("在文件管理器中显示").clicked() {
        show_in_file_manager(tab_path);
        ui.close_menu();
    }
}

/// 关闭指定标签页
fn close_tab(state: &mut IFileEditorState, tab_index: usize) {
    if tab_index >= state.editor.tabs.len() {
        return;
    }
    
    let tab_path = state.editor.tabs[tab_index].clone();
    
    // 检查文件是否已修改
    if let Some(buffer) = state.editor.buffers.get(&tab_path) {
        if buffer.modified {
            // 显示保存确认对话框
            state.ui_state.show_save_confirmation = true;
            state.ui_state.save_confirmation_files = vec![tab_path];
            return; // 不立即关闭，等待用户确认
        }
    }
    
    // 移除标签页和缓冲区
    state.editor.tabs.remove(tab_index);
    state.editor.buffers.remove(&tab_path);
    state.editor.undo_stack.remove(&tab_path);
    
    // 调整活动标签页索引
    if let Some(active_index) = state.editor.active_tab {
        if active_index == tab_index {
            // 关闭的是当前活动标签页
            if state.editor.tabs.is_empty() {
                state.editor.active_tab = None;
            } else if tab_index > 0 {
                state.editor.active_tab = Some(tab_index - 1);
            } else {
                state.editor.active_tab = Some(0);
            }
        } else if active_index > tab_index {
            // 活动标签页在关闭标签页之后，需要调整索引
            state.editor.active_tab = Some(active_index - 1);
        }
    }
    
    log::info!("Closed tab: {:?}", tab_path);
}

/// 关闭所有标签页
fn close_all_tabs(state: &mut IFileEditorState) {
    // 检查是否有未保存的文件
    let mut has_unsaved = false;
    for buffer in state.editor.buffers.values() {
        if buffer.modified {
            has_unsaved = true;
            break;
        }
    }
    
    if has_unsaved {
        // 收集所有未保存的文件
        let unsaved_files: Vec<_> = state.editor.buffers.iter()
            .filter(|(_, buffer)| buffer.modified)
            .map(|(path, _)| path.clone())
            .collect();

        // 显示保存确认对话框
        state.ui_state.show_save_confirmation = true;
        state.ui_state.save_confirmation_files = unsaved_files;
        return; // 不立即关闭，等待用户确认
    }
    
    // 清空所有标签页和缓冲区
    state.editor.tabs.clear();
    state.editor.buffers.clear();
    state.editor.undo_stack.clear();
    state.editor.active_tab = None;
    
    log::info!("Closed all tabs");
}

/// 关闭其他标签页
fn close_other_tabs(state: &mut IFileEditorState) {
    if let Some(active_index) = state.editor.active_tab {
        if active_index < state.editor.tabs.len() {
            let active_path = state.editor.tabs[active_index].clone();
            
            // 检查其他文件是否有未保存的更改
            let mut has_unsaved = false;
            for (path, buffer) in &state.editor.buffers {
                if path != &active_path && buffer.modified {
                    has_unsaved = true;
                    break;
                }
            }
            
            if has_unsaved {
                // 收集其他未保存的文件
                let unsaved_files: Vec<_> = state.editor.buffers.iter()
                    .filter(|(path, buffer)| path != &&active_path && buffer.modified)
                    .map(|(path, _)| path.clone())
                    .collect();

                if !unsaved_files.is_empty() {
                    // 显示保存确认对话框
                    state.ui_state.show_save_confirmation = true;
                    state.ui_state.save_confirmation_files = unsaved_files;
                    return; // 不立即关闭，等待用户确认
                }
            }
            
            // 保留当前活动的标签页，移除其他所有标签页
            let active_buffer = state.editor.buffers.remove(&active_path);
            let active_undo_stack = state.editor.undo_stack.remove(&active_path);
            
            // 清空所有标签页和缓冲区
            state.editor.tabs.clear();
            state.editor.buffers.clear();
            state.editor.undo_stack.clear();
            
            // 重新添加活动标签页
            state.editor.tabs.push(active_path.clone());
            if let Some(buffer) = active_buffer {
                state.editor.buffers.insert(active_path.clone(), buffer);
            }
            if let Some(undo_stack) = active_undo_stack {
                state.editor.undo_stack.insert(active_path.clone(), undo_stack);
            }

            state.editor.active_tab = Some(0);

            log::info!("Closed other tabs, kept: {:?}", active_path);
        }
    }
}

/// 复制文件路径到剪贴板
fn copy_path_to_clipboard(path: &std::path::PathBuf) {
    let path_str = path.to_string_lossy().to_string();

    // 使用 egui 的剪贴板功能
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

/// 在文件管理器中显示文件
fn show_in_file_manager(path: &std::path::PathBuf) {
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = std::process::Command::new("explorer")
            .args(["/select,", &path.to_string_lossy()])
            .spawn()
        {
            log::error!("Failed to show file in explorer: {}", e);
        } else {
            log::info!("Opened file in explorer: {:?}", path);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Err(e) = std::process::Command::new("open")
            .args(["-R", &path.to_string_lossy()])
            .spawn()
        {
            log::error!("Failed to show file in finder: {}", e);
        } else {
            log::info!("Opened file in finder: {:?}", path);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 尝试使用 xdg-open 打开父目录
        if let Some(parent) = path.parent() {
            if let Err(e) = std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
            {
                log::error!("Failed to show file in file manager: {}", e);
            } else {
                log::info!("Opened parent directory: {:?}", parent);
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        log::warn!("Show in file manager not supported on this platform");
    }
}
