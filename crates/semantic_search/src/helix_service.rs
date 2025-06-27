//! 嵌入式HelixDB服务封装

use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::PathBuf;
use serde::Serialize;

use crate::{
    HelixDBManager, EmbeddingService, SemanticSearchConfig, HelixDBError, 
    SemanticSearchError, Note, SemanticSearchResult, RelatedNote, 
    ConceptExtraction, SearchFilters, HelixDBStatus
};

/// 嵌入式HelixDB服务
pub struct EmbeddedHelixService {
    /// HelixDB管理器
    manager: Arc<RwLock<HelixDBManager>>,
    /// HelixDB客户端
    client: helix_db::HelixDB,
    /// 向量化服务
    embedding_service: Arc<dyn EmbeddingService + Send + Sync>,
    /// 配置
    config: SemanticSearchConfig,
    /// 是否已初始化
    initialized: Arc<RwLock<bool>>,
}

impl EmbeddedHelixService {
    /// 创建新的嵌入式HelixDB服务
    pub async fn new(
        data_dir: PathBuf,
        config: SemanticSearchConfig,
        embedding_service: Arc<dyn EmbeddingService + Send + Sync>,
    ) -> Result<Self, HelixDBError> {
        let manager = HelixDBManager::new(data_dir, config.helix_config.clone());
        let client = helix_db::HelixDB::new(Some(config.helix_config.port));

        Ok(Self {
            manager: Arc::new(RwLock::new(manager)),
            client,
            embedding_service,
            config,
            initialized: Arc::new(RwLock::new(false)),
        })
    }

    /// 初始化服务
    pub async fn initialize(&self) -> Result<(), HelixDBError> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            return Ok(());
        }

        log::info!("🔧 初始化嵌入式HelixDB服务...");

        // 启动HelixDB进程
        {
            let mut manager = self.manager.write().await;
            manager.start().await?;
        }

        // 等待服务就绪
        self.wait_for_ready().await?;

        *initialized = true;
        log::info!("✅ 嵌入式HelixDB服务初始化完成");

        Ok(())
    }

    /// 关闭服务
    pub async fn shutdown(&self) -> Result<(), HelixDBError> {
        log::info!("🛑 关闭嵌入式HelixDB服务...");

        {
            let mut manager = self.manager.write().await;
            manager.stop().await?;
        }

        {
            let mut initialized = self.initialized.write().await;
            *initialized = false;
        }

        log::info!("✅ 嵌入式HelixDB服务已关闭");
        Ok(())
    }

    /// 检查服务状态
    pub async fn is_healthy(&self) -> bool {
        let manager = self.manager.read().await;
        matches!(manager.status(), HelixDBStatus::Running)
    }

    /// 等待服务就绪
    async fn wait_for_ready(&self) -> Result<(), HelixDBError> {
        let max_attempts = 30;
        let mut attempts = 0;

        while attempts < max_attempts {
            {
                let mut manager = self.manager.write().await;
                if manager.health_check().await.unwrap_or(false) {
                    return Ok(());
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            attempts += 1;
        }

        Err(HelixDBError::StartupTimeout("服务就绪检查超时".to_string()))
    }

    /// 索引笔记
    pub async fn index_note(&self, note: &Note) -> Result<(), SemanticSearchError> {
        // 检查服务状态
        if !self.is_healthy().await {
            return Err(SemanticSearchError::ServiceUnavailable);
        }

        // 生成向量
        let title_embedding = self.embedding_service
            .embed_text(&note.title)
            .await?;

        let content_embedding = self.embedding_service
            .embed_text(&note.content)
            .await?;

        // 合并标题和内容向量（简单平均）
        let combined_embedding = combine_embeddings(&title_embedding, &content_embedding);

        // 构建查询参数
        #[derive(Serialize)]
        struct AddNoteParams {
            id: String,
            title: String,
            content: String,
            embedding: Vec<f32>,
            notebook_id: String,
        }

        let params = AddNoteParams {
            id: note.id.clone(),
            title: note.title.clone(),
            content: note.content.clone(),
            embedding: combined_embedding,
            notebook_id: note.notebook_id.clone(),
        };

        // 调用HelixDB
        let _result: serde_json::Value = self.client
            .query("addNote", &params)
            .await
            .map_err(|e| SemanticSearchError::DatabaseError(format!("索引笔记失败: {}", e)))?;

        log::debug!("✅ 笔记已索引: {}", note.title);
        Ok(())
    }

    /// 批量索引笔记
    pub async fn index_notes(&self, notes: &[Note]) -> Result<(), SemanticSearchError> {
        log::info!("📚 开始批量索引 {} 条笔记...", notes.len());

        let mut success_count = 0;
        let mut error_count = 0;

        for note in notes {
            match self.index_note(note).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    log::warn!("索引笔记失败 [{}]: {}", note.title, e);
                }
            }
        }

        log::info!("📊 批量索引完成: 成功 {}, 失败 {}", success_count, error_count);

        if error_count > 0 {
            Err(SemanticSearchError::IndexError(
                format!("部分笔记索引失败: {}/{}", error_count, notes.len())
            ))
        } else {
            Ok(())
        }
    }

    /// 语义搜索
    pub async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
        _filters: Option<SearchFilters>,
    ) -> Result<Vec<SemanticSearchResult>, SemanticSearchError> {
        // 检查服务状态
        if !self.is_healthy().await {
            return Err(SemanticSearchError::ServiceUnavailable);
        }

        // 生成查询向量
        let query_embedding = self.embedding_service
            .embed_text(query)
            .await?;

        // 构建查询参数
        #[derive(Serialize)]
        struct SearchParams {
            query_embedding: Vec<f32>,
            limit: i64,
        }

        let params = SearchParams {
            query_embedding,
            limit: limit as i64,
        };

        // 调用HelixDB搜索
        let results: Vec<serde_json::Value> = self.client
            .query("searchNotes", &params)
            .await
            .map_err(|e| SemanticSearchError::DatabaseError(format!("语义搜索失败: {}", e)))?;

        // 转换结果
        let search_results = results
            .into_iter()
            .filter_map(|result| self.parse_search_result(result, query))
            .collect();

        Ok(search_results)
    }

    /// 查找相似笔记
    pub async fn find_similar_notes(
        &self,
        note_id: &str,
        limit: usize,
    ) -> Result<Vec<RelatedNote>, SemanticSearchError> {
        // 检查服务状态
        if !self.is_healthy().await {
            return Err(SemanticSearchError::ServiceUnavailable);
        }

        // 构建查询参数
        #[derive(Serialize)]
        struct SimilarParams {
            note_id: String,
            limit: i64,
        }

        let params = SimilarParams {
            note_id: note_id.to_string(),
            limit: limit as i64,
        };

        // 调用HelixDB
        let results: Vec<serde_json::Value> = self.client
            .query("findSimilarNotes", &params)
            .await
            .map_err(|e| SemanticSearchError::DatabaseError(format!("查找相似笔记失败: {}", e)))?;

        // 转换结果
        let related_notes = results
            .into_iter()
            .filter_map(|result| self.parse_related_note(result))
            .collect();

        Ok(related_notes)
    }

    /// 概念提取
    pub async fn extract_concepts(&self, _text: &str) -> Result<ConceptExtraction, SemanticSearchError> {
        // TODO: 实现基于AI的概念提取
        // 可以使用现有的AI助手服务
        Ok(ConceptExtraction {
            concepts: vec![],
            entities: vec![],
            keywords: vec![],
        })
    }

    /// 更新笔记向量
    pub async fn update_note_embedding(&self, note_id: &str) -> Result<(), SemanticSearchError> {
        // TODO: 实现笔记向量更新
        log::debug!("更新笔记向量: {}", note_id);
        Ok(())
    }

    /// 删除笔记索引
    pub async fn delete_note_index(&self, note_id: &str) -> Result<(), SemanticSearchError> {
        // 检查服务状态
        if !self.is_healthy().await {
            return Err(SemanticSearchError::ServiceUnavailable);
        }

        // 构建查询参数
        #[derive(Serialize)]
        struct DeleteParams {
            note_id: String,
        }

        let params = DeleteParams {
            note_id: note_id.to_string(),
        };

        // 调用HelixDB删除
        let _result: serde_json::Value = self.client
            .query("deleteNote", &params)
            .await
            .map_err(|e| SemanticSearchError::DatabaseError(format!("删除笔记索引失败: {}", e)))?;

        log::debug!("✅ 笔记索引已删除: {}", note_id);
        Ok(())
    }

    /// 解析搜索结果
    fn parse_search_result(&self, result: serde_json::Value, _query: &str) -> Option<SemanticSearchResult> {
        let id = result.get("id")?.as_str()?.to_string();
        let title = result.get("title")?.as_str()?.to_string();
        let content = result.get("content")?.as_str()?.to_string();
        
        // 生成内容预览
        let content_preview = if content.len() > 200 {
            format!("{}...", &content[..200])
        } else {
            content
        };

        // TODO: 计算相似度分数
        let similarity_score = 0.8; // 临时值

        Some(SemanticSearchResult {
            note_id: id,
            title,
            content_preview,
            similarity_score,
            keyword_score: 0.0,
            graph_score: 0.0,
            combined_score: similarity_score,
            related_notes: vec![],
            matched_concepts: vec![],
        })
    }

    /// 解析相关笔记
    fn parse_related_note(&self, result: serde_json::Value) -> Option<RelatedNote> {
        let note_id = result.get("id")?.as_str()?.to_string();
        let title = result.get("title")?.as_str()?.to_string();
        
        Some(RelatedNote {
            note_id,
            title,
            relation_type: "similar".to_string(),
            similarity_score: 0.8, // TODO: 从结果中提取实际分数
        })
    }
}

/// 合并向量（简单平均）
fn combine_embeddings(embedding1: &[f32], embedding2: &[f32]) -> Vec<f32> {
    embedding1
        .iter()
        .zip(embedding2.iter())
        .map(|(a, b)| (a + b) / 2.0)
        .collect()
}
