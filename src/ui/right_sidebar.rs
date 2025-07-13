use eframe::egui;
use crate::app::SeeUApp;
use aiAssist;

/// Render the right sidebar
pub fn render_right_sidebar(ui: &mut egui::Ui, app: &mut SeeUApp) {
    // Header
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("👁️ AI助手").strong());
    });

    ui.separator();

    // 在渲染AI助手之前，强制更新所有上下文
    update_all_contexts_for_ai_assistant(app);

    // AI Assistant content
    aiAssist::render_ai_assist(ui, &mut app.ai_assist_state);
}

/// 为AI助手实时更新所有上下文
fn update_all_contexts_for_ai_assistant(app: &mut SeeUApp) {
    // log::info!("🔄 开始更新AI助手所有上下文");

    // 更新终端上下文
    let active_session_id = app.iterminal_state.egui_terminal_manager.get_active_session_id();
    // log::info!("🔄 当前活动终端会话ID: {:?}", active_session_id);

    let terminal_content = if let Some(session_id) = active_session_id {
        if let Some(session) = app.iterminal_state.egui_terminal_manager.get_sessions().get(&session_id) {
            // log::info!("🔄 找到终端会话，尝试获取内容");
            match session.get_text_content() {
                Ok(content) => {
                    // log::info!("🔄 成功获取终端内容，原始长度: {}", content.len());
                    // 限制输出长度，避免过长的内容
                    if content.len() > 2000 {
                        Some(format!("{}...\n[输出已截断，总长度: {} 字符]",
                            &content[content.len().saturating_sub(2000)..], content.len()))
                    } else {
                        Some(content)
                    }
                },
                Err(e) => {
                    // log::info!("🔄 获取终端内容失败: {:?}", e);
                    None
                }
            }
        } else {
            // log::info!("🔄 没有找到对应的终端会话");
            None
        }
    } else {
        // log::info!("🔄 没有活动的终端会话");
        None
    };

    // 更新AI助手的终端上下文
    if let Some(content) = terminal_content {
        // log::info!("🔄 更新AI助手终端上下文，内容长度: {}", content.len());
        aiAssist::update_terminal_output(&mut app.ai_assist_state, content);
    } else {
        // 如果没有终端内容，清空上下文
        // log::info!("🔄 清空AI助手终端上下文");
        aiAssist::update_terminal_output(&mut app.ai_assist_state, String::new());
    }

    // 更新笔记上下文
    if let Some(note_id) = &app.inote_state.current_note {
        if let Some(note) = app.inote_state.notes.get(note_id) {
            aiAssist::update_note_context(&mut app.ai_assist_state,
                note.title.clone(), note.content.clone());
        }
    } else {
        aiAssist::clear_note_context(&mut app.ai_assist_state);
    }

    // 更新编辑器上下文
    if let Some(file_context) = app.ifile_editor_state.get_current_file_context() {
        aiAssist::update_file_context(&mut app.ai_assist_state,
            file_context.file_name.clone(), file_context.content.clone());
    } else {
        aiAssist::clear_file_context(&mut app.ai_assist_state);
    }
}
