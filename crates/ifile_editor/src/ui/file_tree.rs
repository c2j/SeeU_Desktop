//! 文件树UI组件

use egui;
use egui_ltreeview::{TreeView, TreeViewBuilder, NodeBuilder, Action};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::state::{IFileEditorState, FileNodeId};



/// 渲染文件树
pub fn render_file_tree(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    ui.vertical(|ui| {
        // 文件树标题和操作
        render_file_tree_header(ui, state);

        ui.separator();

        // 显示当前根目录
        if let Some(root) = &state.file_tree.root_path {
            ui.label(format!("📂 {}", root.display()));
            ui.separator();
        }

        // 文件树内容
        if state.file_tree.file_entries.is_empty() {
            render_empty_tree(ui, state);
        } else {
            render_ltree_view(ui, state);
        }

        // 调试信息
        ui.separator();
        ui.small(format!("文件数: {}", state.file_tree.file_entries.len()));
        if let Some(root) = &state.file_tree.root_path {
            ui.small(format!("根目录: {}", root.display()));
        }
    });

    // 渲染目录选择器
    render_directory_picker(ui, state);
}

/// 渲染文件树标题栏
fn render_file_tree_header(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    ui.horizontal(|ui| {
        ui.heading("📁 文件");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("🔄").on_hover_text("刷新").clicked() {
                if let Err(e) = state.file_tree.refresh() {
                    log::error!("Failed to refresh file tree: {}", e);
                }
            }

            if ui.small_button("📁").on_hover_text("设置根目录").clicked() {
                show_directory_picker(state);
            }
        });
    });
}

/// 渲染空文件树
fn render_empty_tree(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label("📂");
            ui.label("没有文件");
            ui.add_space(10.0);

            if ui.button("选择目录").clicked() {
                show_directory_picker(state);
            }
        });
    });
}

/// 使用egui_ltreeview渲染文件树
fn render_ltree_view(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    let tree_id = ui.make_persistent_id("file_tree");

    // 确保根目录已加载
    ensure_root_loaded(state);

    // 确保所有展开的目录都已加载子项
    ensure_expanded_directories_loaded(ui, state);

    // 提前克隆需要的数据以避免借用冲突
    let root_path = state.file_tree.root_path.clone();
    let file_entries = state.file_tree.file_entries.clone();
    let directory_children = state.file_tree.directory_children.clone();

    // 由于闭包借用问题，我们需要分离右键菜单的处理
    let _context_menu_request: Option<(PathBuf, bool)> = None;

    // 将TreeView包装在ScrollArea中以支持滚动
    let actions = egui::ScrollArea::vertical()
        .id_source("file_tree_scroll")
        .auto_shrink([false, false])  // 不自动收缩，保持固定大小
        .show(ui, |ui| {
            let (_response, actions) = TreeView::new(tree_id)
                .allow_multi_selection(true)
                .fallback_context_menu(|ui, selected_nodes: &Vec<FileNodeId>| {
                    // 简化的右键菜单处理
                    if let Some(first_node) = selected_nodes.first() {
                        let path = &first_node.0;
                        ui.label(format!("右键菜单: {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("未知")));

                        if ui.button("📝 打开").clicked() {
                            // 设置请求标志，在外部处理
                            ui.close_menu();
                        }

                        if ui.button("📋 复制路径").clicked() {
                            copy_path_to_clipboard(path);
                            ui.close_menu();
                        }
                    }
                })
                .show_state(ui, &mut state.file_tree.tree_view_state, |builder| {
                    if let Some(root) = &root_path {
                        build_tree_nodes_simple(builder, &file_entries, &directory_children, root);
                    }
                });
            actions
        }).inner;

    // 处理树视图操作
    handle_tree_actions(actions, state);

    // 检查并加载新展开的目录
    check_and_load_expanded_directories(state);
}

/// 检查并加载新展开的目录
fn check_and_load_expanded_directories(state: &mut IFileEditorState) {
    // 获取当前展开的目录列表
    let current_expanded_dirs = get_currently_expanded_directories(state);

    // 检查是否有新展开的目录需要加载内容
    for path in current_expanded_dirs {
        if let Some(entry) = state.file_tree.get_file_entry(&path) {
            if entry.is_dir && !state.file_tree.directory_children.contains_key(&path) {
                log::info!("Loading children for newly expanded directory: {:?}", path);
                if let Err(e) = state.file_tree.load_directory_children(&path) {
                    log::error!("Failed to load directory children: {}", e);
                    state.last_error = Some(e);
                } else {
                    log::info!("Successfully loaded children for expanded directory: {:?}", path);
                }
            }
        }
    }
}

/// 获取当前展开的目录列表
fn get_currently_expanded_directories(state: &IFileEditorState) -> Vec<PathBuf> {
    let mut expanded_dirs = Vec::new();

    // 遍历所有已知的目录条目
    for (path, entry) in state.file_tree.file_entries.iter() {
        if entry.is_dir {
            let node_id = FileNodeId(path.clone());
            // 检查这个目录是否在TreeView中被标记为展开状态
            if let Some(is_open) = state.file_tree.tree_view_state.is_open(&node_id) {
                if is_open {
                    expanded_dirs.push(path.clone());
                }
            }
        }
    }

    expanded_dirs
}

/// 确保根目录已加载
fn ensure_root_loaded(state: &mut IFileEditorState) {
    if let Some(root) = state.file_tree.root_path.clone() {
        // 如果根目录还没有加载，则加载它
        if !state.file_tree.directory_children.contains_key(&root) {
            if let Err(e) = state.file_tree.load_directory_children(&root) {
                log::error!("Failed to load root directory: {}", e);
            }
        }
    }
}

/// 确保所有展开的目录都已加载子项（真正的懒加载）
fn ensure_expanded_directories_loaded(_ui: &mut egui::Ui, _state: &mut IFileEditorState) {
    // 这个函数现在只确保根目录已加载
    // 子目录只在用户主动展开时才加载，实现真正的懒加载

    // 不再预加载所有目录，避免性能问题
    // 子目录的加载将在 Action::Activate 中处理
}

/// 截断文件名以防止目录树过宽（Unicode安全）
fn truncate_filename(name: &str, max_chars: usize) -> String {
    // 使用字符数而不是字节数来计算长度，确保Unicode安全
    let char_count = name.chars().count();

    if char_count <= max_chars {
        name.to_string()
    } else {
        // 保留文件扩展名
        if let Some(dot_pos) = name.rfind('.') {
            let chars: Vec<char> = name.chars().collect();
            let dot_char_pos = name[..dot_pos].chars().count();

            if dot_char_pos > max_chars.saturating_sub(6) { // 为"..."和扩展名预留空间
                let base_chars = max_chars.saturating_sub(6);
                let base: String = chars[..base_chars].iter().collect();
                let ext: String = chars[dot_char_pos..].iter().collect();
                format!("{}...{}", base, ext)
            } else {
                name.to_string()
            }
        } else {
            // 没有扩展名的情况
            let truncate_chars = max_chars.saturating_sub(3);
            let chars: Vec<char> = name.chars().collect();
            let base: String = chars[..truncate_chars].iter().collect();
            format!("{}...", base)
        }
    }
}

/// 构建树节点（懒加载模式）
fn build_tree_nodes_simple(
    builder: &mut TreeViewBuilder<FileNodeId>,
    file_entries: &HashMap<PathBuf, crate::state::FileEntry>,
    directory_children: &HashMap<PathBuf, Vec<PathBuf>>,
    path: &PathBuf
) {
    let children = directory_children.get(path).cloned().unwrap_or_default();

    for child_path in children {
        if let Some(entry) = file_entries.get(&child_path) {
            let node_id = FileNodeId(child_path.clone());

            if entry.is_dir {
                // 目录节点 - 不设置为可激活，让TreeView处理展开/折叠
                let truncated_name = truncate_filename(&entry.name, 25); // 限制目录名长度
                let node_builder = NodeBuilder::dir(node_id)
                    .activatable(false)  // 目录不可激活，只能通过箭头展开/折叠
                    .default_open(false)  // 默认折叠状态
                    .label(format!("{} {}", entry.icon, truncated_name));

                if builder.node(node_builder) {
                    // 只有在目录已经加载了子项时才递归构建子节点
                    // 这实现了真正的懒加载：只有用户展开目录时才加载和显示子项
                    if directory_children.contains_key(&child_path) {
                        build_tree_nodes_simple(builder, file_entries, directory_children, &child_path);
                    }
                    // 如果目录还没有加载子项，就不显示任何子节点
                    // 用户点击箭头时会触发SetSelected事件，我们在那里加载内容
                }
                builder.close_dir();
            } else {
                // 文件节点 - 默认已经是可激活的
                let truncated_name = truncate_filename(&entry.name, 25); // 限制文件名长度
                let node_builder = NodeBuilder::leaf(node_id)
                    .label(format!("{} {}", entry.icon, truncated_name));

                builder.node(node_builder);
            }
        }
    }
}

/// 处理树视图操作
fn handle_tree_actions(actions: Vec<Action<FileNodeId>>, state: &mut IFileEditorState) {
    for action in actions {
        match action {
            Action::Activate(activate) => {
                // 双击激活：只处理文件打开
                for node_id in activate.selected {
                    let path = node_id.0;
                    if let Some(entry) = state.file_tree.get_file_entry(&path) {
                        if !entry.is_dir {
                            // 文件：打开文件
                            log::info!("Opening file: {:?}", path);
                            match state.editor.open_file(path.clone(), &state.settings) {
                                Ok(()) => {
                                    if let Some(buffer) = state.editor.get_active_buffer() {
                                        let mode = if buffer.read_only { "只读" } else { "编辑" };
                                        log::info!("File opened successfully in {} mode: {:?}", mode, path);
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to open file: {}", e);
                                    state.last_error = Some(e);
                                }
                            }
                        }
                        // 目录的双击不做任何处理，让TreeView自己管理展开/折叠
                    }
                }
            }
            Action::SetSelected(node_ids) => {
                // 选择操作（单击）
                if let Some(first_node) = node_ids.first() {
                    let path = &first_node.0;
                    log::info!("Selected: {:?}", path);

                    // 更新UI状态中的选中项
                    state.ui_state.file_tree_selected = Some(path.clone());

                    // 如果选中的是文件，立即打开它
                    if let Some(entry) = state.file_tree.get_file_entry(path) {
                        if !entry.is_dir {
                            // 文件：立即打开
                            log::info!("Opening selected file: {:?}", path);
                            match state.editor.open_file(path.clone(), &state.settings) {
                                Ok(()) => {
                                    if let Some(buffer) = state.editor.get_active_buffer() {
                                        let mode = if buffer.read_only { "只读" } else { "编辑" };
                                        log::info!("File opened successfully in {} mode: {:?}", mode, path);
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to open selected file: {}", e);
                                    state.last_error = Some(e);
                                }
                            }
                        } else {
                            // 目录：只是选中，不自动切换展开状态
                            // 让TreeView自己处理展开/折叠逻辑
                            log::info!("Directory selected: {:?}", path);

                            // 确保目录内容已加载（懒加载）
                            if !state.file_tree.directory_children.contains_key(path) {
                                if let Err(e) = state.file_tree.load_directory_children(path) {
                                    log::error!("Failed to load directory children: {}", e);
                                    state.last_error = Some(e);
                                } else {
                                    log::info!("Loaded directory children for: {:?}", path);
                                }
                            }
                        }
                    }
                }
            }

            _ => {
                // 其他操作（拖拽等）
                log::debug!("Tree action received");
            }
        }
    }
}













/// 显示目录选择器
fn show_directory_picker(state: &mut IFileEditorState) {
    log::info!("Opening directory picker...");

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

/// 渲染目录选择器对话框
fn render_directory_picker(ui: &mut egui::Ui, state: &mut IFileEditorState) {
    if !state.ui_state.show_directory_picker {
        return;
    }

    egui::Window::new("📁 选择目录")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.vertical(|ui| {
                ui.label("请输入目录路径:");

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut state.ui_state.directory_input);

                    if ui.button("📁").on_hover_text("浏览").clicked() {
                        // 再次尝试打开系统对话框
                        if let Some(path) = rfd::FileDialog::new()
                            .set_directory(&state.ui_state.directory_input)
                            .pick_folder()
                        {
                            state.ui_state.directory_input = path.to_string_lossy().to_string();
                        }
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("确定").clicked() {
                        let path = std::path::PathBuf::from(&state.ui_state.directory_input);
                        if path.exists() && path.is_dir() {
                            if let Err(e) = state.set_file_tree_root(path) {
                                log::error!("Failed to set root directory: {}", e);
                                state.last_error = Some(e);
                            } else {
                                state.ui_state.show_directory_picker = false;
                                state.ui_state.directory_input.clear();
                            }
                        } else {
                            log::error!("Invalid directory path: {}", state.ui_state.directory_input);
                            // 可以在这里显示错误消息
                        }
                    }

                    if ui.button("取消").clicked() {
                        state.ui_state.show_directory_picker = false;
                        state.ui_state.directory_input.clear();
                    }
                });

                // 显示一些常用目录的快捷按钮
                ui.separator();
                ui.label("快捷选择:");
                ui.horizontal_wrapped(|ui| {
                    if let Some(home_dir) = dirs::home_dir() {
                        if ui.small_button("🏠 主目录").clicked() {
                            state.ui_state.directory_input = home_dir.to_string_lossy().to_string();
                        }
                    }

                    if let Some(desktop_dir) = dirs::desktop_dir() {
                        if ui.small_button("🖥️ 桌面").clicked() {
                            state.ui_state.directory_input = desktop_dir.to_string_lossy().to_string();
                        }
                    }

                    if let Some(documents_dir) = dirs::document_dir() {
                        if ui.small_button("📄 文档").clicked() {
                            state.ui_state.directory_input = documents_dir.to_string_lossy().to_string();
                        }
                    }

                    if let Ok(current_dir) = std::env::current_dir() {
                        if ui.small_button("📂 当前目录").clicked() {
                            state.ui_state.directory_input = current_dir.to_string_lossy().to_string();
                        }
                    }
                });
            });
        });
}

/// 渲染文件上下文菜单（直接处理）
fn render_file_context_menu_direct(ui: &mut egui::Ui, path: &PathBuf, state: &mut IFileEditorState) {
    if ui.button("📝 打开").clicked() {
        log::info!("Opening file from context menu: {:?}", path);
        match state.editor.open_file(path.clone(), &state.settings) {
            Ok(()) => {
                if let Some(buffer) = state.editor.get_active_buffer() {
                    let mode = if buffer.read_only { "只读" } else { "编辑" };
                    log::info!("File opened successfully in {} mode: {:?}", mode, path);
                }
            }
            Err(e) => {
                log::error!("Failed to open file: {}", e);
                state.last_error = Some(e);
            }
        }
        ui.close_menu();
    }

    if ui.button("📋 复制路径").clicked() {
        copy_path_to_clipboard(path);
        ui.close_menu();
    }

    if ui.button("✏️ 重命名").clicked() {
        state.ui_state.show_rename_dialog = true;
        state.ui_state.operation_target_path = Some(path.clone());
        state.ui_state.rename_new_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
        ui.close_menu();
    }

    ui.separator();

    if ui.button("🗑️ 删除").clicked() {
        state.ui_state.show_delete_confirmation = true;
        state.ui_state.operation_target_path = Some(path.clone());
        ui.close_menu();
    }
}

/// 渲染目录上下文菜单（直接处理）
fn render_dir_context_menu_direct(ui: &mut egui::Ui, path: &PathBuf, state: &mut IFileEditorState) {
    if ui.button("📄 新建文件").clicked() {
        state.ui_state.show_new_file_dialog = true;
        state.ui_state.operation_target_path = Some(path.clone());
        state.ui_state.new_file_name.clear();
        ui.close_menu();
    }

    if ui.button("📁 新建文件夹").clicked() {
        state.ui_state.show_new_folder_dialog = true;
        state.ui_state.operation_target_path = Some(path.clone());
        state.ui_state.new_folder_name.clear();
        ui.close_menu();
    }

    ui.separator();

    if ui.button("📋 复制路径").clicked() {
        copy_path_to_clipboard(path);
        ui.close_menu();
    }

    if ui.button("✏️ 重命名").clicked() {
        state.ui_state.show_rename_dialog = true;
        state.ui_state.operation_target_path = Some(path.clone());
        state.ui_state.rename_new_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
        ui.close_menu();
    }

    ui.separator();

    if ui.button("🗑️ 删除").clicked() {
        state.ui_state.show_delete_confirmation = true;
        state.ui_state.operation_target_path = Some(path.clone());
        ui.close_menu();
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
