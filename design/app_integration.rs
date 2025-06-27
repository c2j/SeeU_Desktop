// 主应用集成 - 将语义搜索集成到SeeU Desktop

use std::sync::Arc;
use tokio::sync::RwLock;

/// 扩展主应用状态以支持语义搜索
pub struct AppStateExtension {
    /// 语义搜索状态
    pub semantic_search: SemanticSearchState,
    /// 是否启用语义搜索
    pub semantic_search_enabled: bool,
    /// 语义搜索配置界面状态
    pub show_semantic_config: bool,
}

impl Default for AppStateExtension {
    fn default() -> Self {
        Self {
            semantic_search: SemanticSearchState::default(),
            semantic_search_enabled: false,
            show_semantic_config: false,
        }
    }
}

/// 应用启动时的初始化逻辑
impl AppStateExtension {
    /// 应用启动时初始化语义搜索
    pub async fn initialize_on_startup(&mut self) -> Result<(), String> {
        log::info!("🚀 应用启动，检查语义搜索配置...");

        // 加载配置
        if let Ok(config) = self.load_semantic_search_config().await {
            self.semantic_search.config = config;
            
            if self.semantic_search.config.enabled {
                log::info!("语义搜索已启用，开始初始化...");
                match self.semantic_search.initialize().await {
                    Ok(_) => {
                        self.semantic_search_enabled = true;
                        log::info!("✅ 语义搜索初始化成功");
                    }
                    Err(e) => {
                        log::error!("❌ 语义搜索初始化失败: {}", e);
                        // 不阻止应用启动，只是禁用语义搜索功能
                        self.semantic_search_enabled = false;
                    }
                }
            } else {
                log::info!("语义搜索功能已禁用");
            }
        } else {
            log::info!("未找到语义搜索配置，使用默认设置");
        }

        Ok(())
    }

    /// 应用关闭时的清理逻辑
    pub async fn cleanup_on_shutdown(&mut self) -> Result<(), String> {
        log::info!("🛑 应用关闭，清理语义搜索资源...");

        if self.semantic_search_enabled {
            self.semantic_search.shutdown().await?;
            self.semantic_search_enabled = false;
        }

        log::info!("✅ 语义搜索资源清理完成");
        Ok(())
    }

    /// 笔记保存时的自动索引
    pub async fn on_note_saved(&mut self, note: &Note) -> Result<(), String> {
        if !self.semantic_search_enabled {
            return Ok(());
        }

        log::debug!("📝 笔记已保存，更新语义索引: {}", note.title);

        // 异步索引，不阻塞UI
        if let Err(e) = self.semantic_search.index_note(note).await {
            log::warn!("更新语义索引失败: {}", e);
            // 不返回错误，避免影响笔记保存
        }

        Ok(())
    }

    /// 笔记删除时的索引清理
    pub async fn on_note_deleted(&mut self, note_id: &str) -> Result<(), String> {
        if !self.semantic_search_enabled {
            return Ok(());
        }

        log::debug!("🗑️ 笔记已删除，清理语义索引: {}", note_id);

        // TODO: 实现删除索引的逻辑
        // self.semantic_search.delete_note_index(note_id).await?;

        Ok(())
    }

    /// 执行混合搜索（集成到现有搜索界面）
    pub async fn perform_hybrid_search(&mut self, query: &str, limit: usize) -> Result<Vec<HybridSearchResult>, String> {
        if !self.semantic_search_enabled {
            return Err("语义搜索功能未启用".to_string());
        }

        self.semantic_search.semantic_search(query, limit).await?;
        Ok(self.semantic_search.get_search_results().to_vec())
    }

    /// 切换语义搜索功能
    pub async fn toggle_semantic_search(&mut self) -> Result<(), String> {
        if self.semantic_search_enabled {
            // 禁用语义搜索
            self.semantic_search.shutdown().await?;
            self.semantic_search_enabled = false;
            
            // 更新配置
            self.semantic_search.config.enabled = false;
            self.save_semantic_search_config().await?;
            
            log::info!("语义搜索已禁用");
        } else {
            // 启用语义搜索
            self.semantic_search.config.enabled = true;
            self.semantic_search.initialize().await?;
            self.semantic_search_enabled = true;
            
            // 保存配置
            self.save_semantic_search_config().await?;
            
            log::info!("语义搜索已启用");
        }

        Ok(())
    }

    /// 批量重建索引
    pub async fn rebuild_semantic_index(&mut self, notes: &[Note]) -> Result<(), String> {
        if !self.semantic_search_enabled {
            return Err("语义搜索功能未启用".to_string());
        }

        log::info!("🔄 开始重建语义索引，共 {} 条笔记", notes.len());

        self.semantic_search.index_notes(notes).await?;

        log::info!("✅ 语义索引重建完成");
        Ok(())
    }

    /// 加载语义搜索配置
    async fn load_semantic_search_config(&self) -> Result<SemanticSearchConfig, String> {
        let config_path = self.get_config_path()?;
        
        if !config_path.exists() {
            return Ok(SemanticSearchConfig::default());
        }

        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        let config: SemanticSearchConfig = toml::from_str(&config_content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;

        Ok(config)
    }

    /// 保存语义搜索配置
    async fn save_semantic_search_config(&self) -> Result<(), String> {
        let config_path = self.get_config_path()?;
        
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }

        let config_content = toml::to_string_pretty(&self.semantic_search.config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;

        std::fs::write(&config_path, config_content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;

        log::debug!("✅ 语义搜索配置已保存");
        Ok(())
    }

    /// 获取配置文件路径
    fn get_config_path(&self) -> Result<std::path::PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or("无法获取配置目录")?
            .join("SeeU_Desktop");

        Ok(config_dir.join("semantic_search.toml"))
    }
}

/// 搜索界面集成
pub struct SearchUIIntegration;

impl SearchUIIntegration {
    /// 渲染语义搜索选项
    pub fn render_semantic_search_options(
        ui: &mut egui::Ui,
        app_state: &mut AppStateExtension,
    ) {
        ui.horizontal(|ui| {
            // 语义搜索开关
            let mut enabled = app_state.semantic_search_enabled;
            if ui.checkbox(&mut enabled, "启用语义搜索").changed() {
                // 异步切换语义搜索状态
                // 注意：这里需要在实际集成时处理异步调用
                log::info!("用户切换语义搜索状态: {}", enabled);
            }

            ui.separator();

            // 配置按钮
            if ui.button("⚙ 配置").clicked() {
                app_state.show_semantic_config = true;
            }

            // 重建索引按钮
            if ui.button("🔄 重建索引").clicked() && app_state.semantic_search_enabled {
                log::info!("用户请求重建语义索引");
                // 需要在实际集成时实现
            }
        });

        // 服务状态指示器
        ui.horizontal(|ui| {
            let status_text = match app_state.semantic_search.get_status().await {
                ServiceStatus::Running => ("🟢", "运行中"),
                ServiceStatus::Starting => ("🟡", "启动中"),
                ServiceStatus::Stopped => ("🔴", "已停止"),
                ServiceStatus::Error => ("❌", "错误"),
            };

            ui.label(format!("{} 语义搜索: {}", status_text.0, status_text.1));

            if let Some(error) = app_state.semantic_search.get_last_error() {
                ui.label(format!("错误: {}", error));
            }
        });
    }

    /// 渲染语义搜索结果
    pub fn render_semantic_search_results(
        ui: &mut egui::Ui,
        results: &[HybridSearchResult],
    ) {
        if results.is_empty() {
            ui.label("没有找到相关结果");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (index, result) in results.iter().enumerate() {
                ui.group(|ui| {
                    // 标题和分数
                    ui.horizontal(|ui| {
                        ui.heading(&result.title);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("相关度: {:.2}", result.combined_score));
                            
                            // 搜索类型标识
                            let type_text = match result.search_type {
                                SearchType::Semantic => "🧠 语义",
                                SearchType::Keyword => "🔍 关键词",
                                SearchType::Hybrid => "🔗 混合",
                            };
                            ui.label(type_text);
                        });
                    });

                    // 内容预览
                    ui.label(&result.content_preview);

                    // 匹配信息
                    if !result.matched_terms.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("匹配词:");
                            for term in &result.matched_terms {
                                ui.label(format!("#{}", term));
                            }
                        });
                    }

                    // 相关笔记
                    if !result.related_notes.is_empty() {
                        ui.collapsing("相关笔记", |ui| {
                            for related in &result.related_notes {
                                ui.horizontal(|ui| {
                                    if ui.link(&related.title).clicked() {
                                        // 打开相关笔记
                                        log::info!("打开相关笔记: {}", related.note_id);
                                    }
                                    ui.label(format!("({:.2})", related.similarity_score));
                                });
                            }
                        });
                    }
                });

                if index < results.len() - 1 {
                    ui.separator();
                }
            }
        });
    }

    /// 渲染语义搜索配置界面
    pub fn render_semantic_config_dialog(
        ctx: &egui::Context,
        app_state: &mut AppStateExtension,
    ) {
        if !app_state.show_semantic_config {
            return;
        }

        egui::Window::new("语义搜索配置")
            .default_width(500.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                // TODO: 实现配置界面
                ui.label("语义搜索配置界面");
                
                if ui.button("关闭").clicked() {
                    app_state.show_semantic_config = false;
                }
            });
    }
}

// 重新导出类型
pub use crate::{
    SemanticSearchState, SemanticSearchConfig, HybridSearchResult,
    SearchType, ServiceStatus, Note,
};
