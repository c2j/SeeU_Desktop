//! 缓存管理模块

use crate::types::CacheStats;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 缓存项
#[derive(Clone)]
struct CacheItem {
    vector: Vec<f32>,
    timestamp: u64,
    access_count: u64,
}

/// 高性能向量缓存
pub struct EmbeddingCache {
    cache: DashMap<String, CacheItem>,
    hit_count: AtomicU64,
    miss_count: AtomicU64,
    max_size: usize,
}

impl EmbeddingCache {
    /// 创建新的缓存
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: DashMap::new(),
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            max_size: capacity,
        }
    }

    /// 获取缓存值
    pub async fn get(&self, key: &str) -> Option<Vec<f32>> {
        if let Some(mut item) = self.cache.get_mut(key) {
            // 更新访问统计
            item.access_count += 1;
            item.timestamp = Self::current_timestamp();

            self.hit_count.fetch_add(1, Ordering::Relaxed);
            Some(item.vector.clone())
        } else {
            self.miss_count.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// 插入缓存值
    pub async fn insert(&self, key: String, value: Vec<f32>) {
        // 检查是否需要清理缓存
        if self.cache.len() >= self.max_size {
            self.evict_old_entries().await;
        }

        let item = CacheItem {
            vector: value,
            timestamp: Self::current_timestamp(),
            access_count: 1,
        };

        self.cache.insert(key, item);
    }

    /// 清理旧的缓存项
    async fn evict_old_entries(&self) {
        let target_size = self.max_size * 3 / 4; // 清理到75%容量

        if self.cache.len() <= target_size {
            return;
        }

        // 收集所有项目的键和时间戳
        let mut items: Vec<(String, u64, u64)> = self.cache
            .iter()
            .map(|entry| {
                let key = entry.key().clone();
                let item = entry.value();
                (key, item.timestamp, item.access_count)
            })
            .collect();

        // 按访问频率和时间排序（LFU + LRU）
        items.sort_by(|a, b| {
            let score_a = a.2 as f64 / (Self::current_timestamp() - a.1 + 1) as f64;
            let score_b = b.2 as f64 / (Self::current_timestamp() - b.1 + 1) as f64;
            score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 删除评分最低的项目
        let to_remove = self.cache.len() - target_size;
        for (key, _, _) in items.iter().take(to_remove) {
            self.cache.remove(key);
        }
    }

    /// 获取当前时间戳
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// 获取缓存统计
    pub fn get_stats(&self) -> CacheStats {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        let total = hits + misses;

        CacheStats {
            hits,
            misses,
            hit_rate: if total > 0 { hits as f64 / total as f64 } else { 0.0 },
            size: self.cache.len(),
        }
    }

    /// 清空缓存
    pub fn clear(&self) {
        self.cache.clear();
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
    }

    /// 获取缓存使用的内存大小（估算）
    pub fn memory_usage_bytes(&self) -> usize {
        let item_count = self.cache.len();
        let avg_vector_size = 512; // BGE模型的向量维度
        let vector_memory = item_count * avg_vector_size * std::mem::size_of::<f32>();
        let metadata_memory = item_count * (
            std::mem::size_of::<String>() +
            std::mem::size_of::<CacheItem>()
        );
        vector_memory + metadata_memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cache_operations() {
        let cache = EmbeddingCache::new(10);
        
        // 测试缓存未命中
        let result = cache.get("key1").await;
        assert!(result.is_none());
        
        // 测试缓存插入和命中
        let vector = vec![1.0, 2.0, 3.0];
        cache.insert("key1".to_string(), vector.clone()).await;
        
        let result = cache.get("key1").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vector);
        
        // 测试统计
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
        assert_eq!(stats.size, 1);
    }
    
    #[tokio::test]
    async fn test_cache_capacity() {
        let cache = EmbeddingCache::new(2);
        
        // 插入超过容量的项目
        cache.insert("key1".to_string(), vec![1.0]).await;
        cache.insert("key2".to_string(), vec![2.0]).await;
        cache.insert("key3".to_string(), vec![3.0]).await;
        
        // 验证LRU行为
        let stats = cache.get_stats();
        assert_eq!(stats.size, 2);
        
        // key1应该被淘汰
        let result = cache.get("key1").await;
        assert!(result.is_none());
        
        // key2和key3应该还在
        let result2 = cache.get("key2").await;
        assert!(result2.is_some());
        
        let result3 = cache.get("key3").await;
        assert!(result3.is_some());
    }
    
    #[test]
    fn test_cache_clear() {
        let cache = EmbeddingCache::new(10);
        
        // 添加一些数据
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            cache.insert("key1".to_string(), vec![1.0]).await;
            cache.get("key1").await;
        });
        
        // 清空缓存
        cache.clear();
        
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size, 0);
    }
}
