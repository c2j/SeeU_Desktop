//! 缓存管理模块

use crate::types::CacheStats;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{atomic::{AtomicU64, Ordering}, RwLock};

/// 向量缓存
pub struct EmbeddingCache {
    cache: RwLock<LruCache<String, Vec<f32>>>,
    hit_count: AtomicU64,
    miss_count: AtomicU64,
}

impl EmbeddingCache {
    /// 创建新的缓存
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(LruCache::new(NonZeroUsize::new(capacity).unwrap())),
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
        }
    }
    
    /// 获取缓存值
    pub async fn get(&self, key: &str) -> Option<Vec<f32>> {
        let mut cache = self.cache.write().unwrap();
        if let Some(value) = cache.get(key) {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            Some(value.clone())
        } else {
            self.miss_count.fetch_add(1, Ordering::Relaxed);
            None
        }
    }
    
    /// 插入缓存值
    pub async fn insert(&self, key: String, value: Vec<f32>) {
        let mut cache = self.cache.write().unwrap();
        cache.put(key, value);
    }
    
    /// 获取缓存统计
    pub fn get_stats(&self) -> CacheStats {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        let total = hits + misses;
        let cache = self.cache.read().unwrap();
        
        CacheStats {
            hits,
            misses,
            hit_rate: if total > 0 { hits as f64 / total as f64 } else { 0.0 },
            size: cache.len(),
        }
    }
    
    /// 清空缓存
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
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
