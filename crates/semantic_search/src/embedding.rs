//! 向量化服务实现

use async_trait::async_trait;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use moka::future::Cache;
use std::time::Duration;

use crate::{EmbeddingConfig, EmbeddingError};

/// 向量化服务接口
#[async_trait]
pub trait EmbeddingService {
    /// 生成文本向量
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    
    /// 批量生成向量
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
    
    /// 获取向量维度
    fn dimension(&self) -> usize;
}

/// OpenAI向量化服务
pub struct OpenAIEmbeddingService {
    client: Client,
    config: EmbeddingConfig,
    cache: Cache<String, Vec<f32>>,
}

impl OpenAIEmbeddingService {
    pub fn new(config: EmbeddingConfig) -> Self {
        let cache = if config.cache_config.enabled {
            Cache::builder()
                .max_capacity(config.cache_config.max_size as u64)
                .time_to_live(Duration::from_secs(config.cache_config.ttl_seconds))
                .build()
        } else {
            Cache::builder().max_capacity(0).build()
        };

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(config.request_timeout))
                .build()
                .unwrap_or_default(),
            config,
            cache,
        }
    }

    /// 构建API请求
    async fn create_embedding_request(&self, texts: &[String]) -> Result<EmbeddingResponse, EmbeddingError> {
        let api_key = self.config.api_key
            .as_ref()
            .ok_or_else(|| EmbeddingError::ConfigError("API密钥未配置".to_string()))?;

        let request = EmbeddingRequest {
            input: texts.to_vec(),
            model: self.config.model.clone(),
            encoding_format: Some("float".to_string()),
        };

        let response = self.client
            .post(&self.config.get_embedding_endpoint())
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            
            return match status.as_u16() {
                401 => Err(EmbeddingError::AuthenticationError("API密钥无效".to_string())),
                429 => Err(EmbeddingError::QuotaExceeded("API配额已用完".to_string())),
                _ => Err(EmbeddingError::ApiError(format!("API错误 {}: {}", status, error_text))),
            };
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ApiError(format!("解析响应失败: {}", e)))?;

        Ok(embedding_response)
    }
}

#[async_trait]
impl EmbeddingService for OpenAIEmbeddingService {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        // 检查缓存
        if let Some(cached_embedding) = self.cache.get(text).await {
            return Ok(cached_embedding);
        }

        // 调用API
        let response = self.create_embedding_request(&[text.to_string()]).await?;
        
        if response.data.is_empty() {
            return Err(EmbeddingError::ApiError("响应数据为空".to_string()));
        }

        let embedding = response.data[0].embedding.clone();

        // 验证向量维度
        if embedding.len() != self.config.dimension {
            return Err(EmbeddingError::DimensionMismatch);
        }

        // 缓存结果
        self.cache.insert(text.to_string(), embedding.clone()).await;

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // 检查批处理大小
        if texts.len() > self.config.batch_size {
            // 分批处理
            let mut all_embeddings = Vec::new();
            for chunk in texts.chunks(self.config.batch_size) {
                let chunk_embeddings = self.embed_batch(chunk).await?;
                all_embeddings.extend(chunk_embeddings);
            }
            return Ok(all_embeddings);
        }

        // 检查缓存
        let mut uncached_texts = Vec::new();
        let mut uncached_indices = Vec::new();
        let mut results = vec![Vec::new(); texts.len()];

        for (i, text) in texts.iter().enumerate() {
            if let Some(cached_embedding) = self.cache.get(text).await {
                results[i] = cached_embedding;
            } else {
                uncached_texts.push(text.clone());
                uncached_indices.push(i);
            }
        }

        // 处理未缓存的文本
        if !uncached_texts.is_empty() {
            let response = self.create_embedding_request(&uncached_texts).await?;
            
            if response.data.len() != uncached_texts.len() {
                return Err(EmbeddingError::ApiError("响应数据数量不匹配".to_string()));
            }

            // 更新结果和缓存
            for (i, embedding_data) in response.data.iter().enumerate() {
                let embedding = embedding_data.embedding.clone();
                let original_index = uncached_indices[i];
                
                // 验证向量维度
                if embedding.len() != self.config.dimension {
                    return Err(EmbeddingError::DimensionMismatch);
                }

                results[original_index] = embedding.clone();
                self.cache.insert(uncached_texts[i].clone(), embedding).await;
            }
        }

        Ok(results)
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }
}

/// 本地向量化服务（使用Ollama等）
pub struct LocalEmbeddingService {
    client: Client,
    config: EmbeddingConfig,
    base_url: String,
}

impl LocalEmbeddingService {
    pub fn new(config: EmbeddingConfig, base_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(config.request_timeout))
                .build()
                .unwrap_or_default(),
            config,
            base_url,
        }
    }
}

#[async_trait]
impl EmbeddingService for LocalEmbeddingService {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let request = LocalEmbeddingRequest {
            model: self.config.model.clone(),
            prompt: text.to_string(),
        };

        let response = self.client
            .post(&format!("{}/api/embeddings", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ApiError(format!("本地服务错误: {}", error_text)));
        }

        let embedding_response: LocalEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ApiError(format!("解析本地服务响应失败: {}", e)))?;

        Ok(embedding_response.embedding)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        // 本地服务通常不支持批处理，逐个处理
        let mut embeddings = Vec::new();
        for text in texts {
            let embedding = self.embed_text(text).await?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.config.dimension
    }
}

/// 向量化服务工厂
pub struct EmbeddingServiceFactory;

impl EmbeddingServiceFactory {
    pub fn create_service(config: &EmbeddingConfig) -> Result<Arc<dyn EmbeddingService + Send + Sync>, EmbeddingError> {
        match config.provider.as_str() {
            "openai" => {
                let service = OpenAIEmbeddingService::new(config.clone());
                Ok(Arc::new(service))
            }
            "local" => {
                let base_url = config.get_api_base_url();
                let service = LocalEmbeddingService::new(config.clone(), base_url);
                Ok(Arc::new(service))
            }
            _ => Err(EmbeddingError::ConfigError(format!("不支持的向量化提供商: {}", config.provider)))
        }
    }
}

// API请求/响应结构

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: Vec<String>,
    model: String,
    encoding_format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    model: String,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize)]
struct LocalEmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct LocalEmbeddingResponse {
    embedding: Vec<f32>,
}
