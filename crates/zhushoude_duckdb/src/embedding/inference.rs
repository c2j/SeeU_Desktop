//! 异步推理引擎模块

use crate::{Result, EmbeddingEngine, Error};
use std::sync::Arc;
use tokio::sync::{Semaphore, RwLock, oneshot};
use tokio::task::JoinHandle;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 推理请求
#[derive(Debug)]
struct InferenceRequest {
    text: String,
    response_sender: oneshot::Sender<Result<Vec<f32>>>,
    priority: InferencePriority,
    created_at: Instant,
}

/// 推理优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InferencePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// 异步推理引擎
pub struct InferenceEngine {
    embedding_engine: Arc<EmbeddingEngine>,
    request_queue: Arc<RwLock<VecDeque<InferenceRequest>>>,
    semaphore: Arc<Semaphore>,
    worker_handle: Option<JoinHandle<()>>,
    max_batch_size: usize,
    batch_timeout: Duration,
}

impl InferenceEngine {
    /// 创建新的推理引擎
    pub fn new(embedding_engine: Arc<EmbeddingEngine>) -> Self {
        let request_queue = Arc::new(RwLock::new(VecDeque::new()));
        let semaphore = Arc::new(Semaphore::new(100)); // 最多100个并发请求

        let mut engine = Self {
            embedding_engine,
            request_queue,
            semaphore,
            worker_handle: None,
            max_batch_size: 8,
            batch_timeout: Duration::from_millis(50),
        };

        // 启动后台工作线程
        engine.start_worker();
        engine
    }

    /// 启动后台工作线程
    fn start_worker(&mut self) {
        let embedding_engine = self.embedding_engine.clone();
        let request_queue = self.request_queue.clone();
        let max_batch_size = self.max_batch_size;
        let batch_timeout = self.batch_timeout;

        let handle = tokio::spawn(async move {
            loop {
                let batch = Self::collect_batch(&request_queue, max_batch_size, batch_timeout).await;

                if batch.is_empty() {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    continue;
                }

                Self::process_batch(&embedding_engine, batch).await;
            }
        });

        self.worker_handle = Some(handle);
    }

    /// 收集批处理请求
    async fn collect_batch(
        request_queue: &Arc<RwLock<VecDeque<InferenceRequest>>>,
        max_batch_size: usize,
        batch_timeout: Duration,
    ) -> Vec<InferenceRequest> {
        let start_time = Instant::now();
        let mut batch = Vec::new();

        loop {
            // 尝试从队列中取出请求
            {
                let mut queue = request_queue.write().await;
                while batch.len() < max_batch_size {
                    if let Some(request) = queue.pop_front() {
                        batch.push(request);
                    } else {
                        break;
                    }
                }
            }

            // 如果批次已满或超时，返回批次
            if batch.len() >= max_batch_size || start_time.elapsed() >= batch_timeout {
                break;
            }

            // 如果有部分请求但未满，等待一小段时间
            if !batch.is_empty() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            } else {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        // 按优先级排序
        batch.sort_by(|a, b| b.priority.cmp(&a.priority));
        batch
    }

    /// 处理批次请求
    async fn process_batch(embedding_engine: &Arc<EmbeddingEngine>, batch: Vec<InferenceRequest>) {
        if batch.is_empty() {
            return;
        }

        // 提取文本
        let texts: Vec<String> = batch.iter().map(|req| req.text.clone()).collect();

        // 批量编码
        match embedding_engine.encode_batch(&texts).await {
            Ok(embeddings) => {
                // 发送结果
                for (request, embedding) in batch.into_iter().zip(embeddings.into_iter()) {
                    let _ = request.response_sender.send(Ok(embedding));
                }
            }
            Err(e) => {
                // 发送错误
                for request in batch {
                    let _ = request.response_sender.send(Err(e.clone()));
                }
            }
        }
    }

    /// 异步推理单个文本
    pub async fn infer_async(&self, text: &str, priority: InferencePriority) -> Result<Vec<f32>> {
        // 获取信号量许可
        let _permit = self.semaphore.acquire().await
            .map_err(|_| Error::ModelError("获取推理许可失败".to_string()))?;

        // 创建响应通道
        let (sender, receiver) = oneshot::channel();

        // 创建请求
        let request = InferenceRequest {
            text: text.to_string(),
            response_sender: sender,
            priority,
            created_at: Instant::now(),
        };

        // 添加到队列
        {
            let mut queue = self.request_queue.write().await;
            queue.push_back(request);
        }

        // 等待结果
        receiver.await
            .map_err(|_| Error::ModelError("推理请求被取消".to_string()))?
    }

    /// 同步推理（兼容性接口）
    pub async fn infer(&self, text: &str) -> Result<Vec<f32>> {
        self.infer_async(text, InferencePriority::Normal).await
    }

    /// 高优先级推理
    pub async fn infer_high_priority(&self, text: &str) -> Result<Vec<f32>> {
        self.infer_async(text, InferencePriority::High).await
    }

    /// 获取队列统计
    pub async fn get_queue_stats(&self) -> (usize, usize) {
        let queue = self.request_queue.read().await;
        let total_requests = queue.len();
        let high_priority_requests = queue.iter()
            .filter(|req| req.priority >= InferencePriority::High)
            .count();

        (total_requests, high_priority_requests)
    }

    /// 清空队列
    pub async fn clear_queue(&self) {
        let mut queue = self.request_queue.write().await;

        // 发送取消信号给所有等待的请求
        while let Some(request) = queue.pop_front() {
            let _ = request.response_sender.send(
                Err(Error::ModelError("推理队列被清空".to_string()))
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EmbeddingConfig;

    #[tokio::test]
    async fn test_inference_engine() {
        let config = EmbeddingConfig::default();
        let embedding_engine = Arc::new(EmbeddingEngine::new(config).await.unwrap());
        let inference_engine = InferenceEngine::new(embedding_engine);

        let result = inference_engine.infer("测试文本").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 512); // 默认向量维度
    }

    #[tokio::test]
    async fn test_priority_inference() {
        let config = EmbeddingConfig::default();
        let embedding_engine = Arc::new(EmbeddingEngine::new(config).await.unwrap());
        let inference_engine = InferenceEngine::new(embedding_engine);

        // 测试不同优先级的推理
        let normal_result = inference_engine.infer_async("普通优先级", InferencePriority::Normal).await;
        let high_result = inference_engine.infer_high_priority("高优先级").await;

        assert!(normal_result.is_ok());
        assert!(high_result.is_ok());
    }

    #[tokio::test]
    async fn test_queue_stats() {
        let config = EmbeddingConfig::default();
        let embedding_engine = Arc::new(EmbeddingEngine::new(config).await.unwrap());
        let inference_engine = InferenceEngine::new(embedding_engine);

        // 获取初始统计
        let (total, high_priority) = inference_engine.get_queue_stats().await;
        assert_eq!(total, 0);
        assert_eq!(high_priority, 0);
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let config = EmbeddingConfig::default();
        let embedding_engine = Arc::new(EmbeddingEngine::new(config).await.unwrap());
        let inference_engine = Arc::new(InferenceEngine::new(embedding_engine));

        // 并发发送多个请求
        let mut handles = Vec::new();
        for i in 0..5 {
            let engine = inference_engine.clone();
            let handle = tokio::spawn(async move {
                engine.infer(&format!("测试文本{}", i)).await
            });
            handles.push(handle);
        }

        // 等待所有请求完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }
}
