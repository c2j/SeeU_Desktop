//! 标签页UI组件

use egui;
use crate::state::IFileEditorState;

/// 渲染标签页
pub fn render_tabs(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    if state.editor.tabs.is_empty() {
        return;
    }
    
    ui.horizontal(|ui| {
        let mut tab_to_close = None;
        let mut new_active_tab = state.editor.active_tab;
        
        for (index, tab_path) in state.editor.tabs.iter().enumerate() {
            let is_active = state.editor.active_tab == Some(index);
            let buffer = state.editor.buffers.get(tab_path);
            
            // 获取文件名
            let file_name = tab_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("未知文件");
            
            // 检查是否已修改
            let is_modified = buffer.map(|b| b.modified).unwrap_or(false);
            
            // 标签页按钮
            let tab_text = if is_modified {
                format!("● {}", file_name)
            } else {
                file_name.to_string()
            };
            
            let tab_button = egui::Button::new(&tab_text)
                .selected(is_active)
                .min_size(egui::vec2(80.0, 24.0));
            
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
            
            // 标签页分隔符
            if index < state.editor.tabs.len() - 1 {
                ui.separator();
            }
        }
        
        // 应用标签页切换
        if new_active_tab != state.editor.active_tab {
            state.editor.active_tab = new_active_tab;
        }
        
        // 处理标签页关闭
        if let Some(close_index) = tab_to_close {
            close_tab(state, close_index);
        }
        
        // 右侧空间和操作按钮
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // 关闭所有标签页按钮
            if ui.small_button("×").on_hover_text("关闭所有").clicked() {
                close_all_tabs(state);
            }
            
            // 关闭其他标签页按钮
            if ui.small_button("⊗").on_hover_text("关闭其他").clicked() {
                close_other_tabs(state);
            }
        });
    });
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
