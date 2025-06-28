//! 性能监控和指标收集模块

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// 性能指标收集器
#[derive(Debug)]
pub struct MetricsCollector {
    // 查询统计
    query_count: AtomicU64,
    query_total_time: AtomicU64,
    query_errors: AtomicU64,
    
    // 缓存统计
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_size: AtomicUsize,
    
    // 内存统计
    memory_usage: AtomicUsize,
    peak_memory: AtomicUsize,
    
    // 向量操作统计
    vector_operations: AtomicU64,
    vector_total_time: AtomicU64,
    
    // 数据库操作统计
    db_operations: AtomicU64,
    db_total_time: AtomicU64,
    
    start_time: Instant,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self {
            query_count: AtomicU64::new(0),
            query_total_time: AtomicU64::new(0),
            query_errors: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_size: AtomicUsize::new(0),
            memory_usage: AtomicUsize::new(0),
            peak_memory: AtomicUsize::new(0),
            vector_operations: AtomicU64::new(0),
            vector_total_time: AtomicU64::new(0),
            db_operations: AtomicU64::new(0),
            db_total_time: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }
    
    /// 记录查询操作
    pub fn record_query(&self, duration: Duration, success: bool) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
        self.query_total_time.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        
        if !success {
            self.query_errors.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// 记录缓存命中
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录缓存未命中
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 更新缓存大小
    pub fn update_cache_size(&self, size: usize) {
        self.cache_size.store(size, Ordering::Relaxed);
    }
    
    /// 更新内存使用量
    pub fn update_memory_usage(&self, usage: usize) {
        self.memory_usage.store(usage, Ordering::Relaxed);
        
        // 更新峰值内存
        let current_peak = self.peak_memory.load(Ordering::Relaxed);
        if usage > current_peak {
            self.peak_memory.store(usage, Ordering::Relaxed);
        }
    }
    
    /// 记录向量操作
    pub fn record_vector_operation(&self, duration: Duration) {
        self.vector_operations.fetch_add(1, Ordering::Relaxed);
        self.vector_total_time.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
    }
    
    /// 记录数据库操作
    pub fn record_db_operation(&self, duration: Duration) {
        self.db_operations.fetch_add(1, Ordering::Relaxed);
        self.db_total_time.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
    }
    
    /// 获取性能快照
    pub fn get_snapshot(&self) -> MetricsSnapshot {
        let uptime = self.start_time.elapsed();
        let query_count = self.query_count.load(Ordering::Relaxed);
        let query_total_time = self.query_total_time.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        let vector_ops = self.vector_operations.load(Ordering::Relaxed);
        let vector_time = self.vector_total_time.load(Ordering::Relaxed);
        let db_ops = self.db_operations.load(Ordering::Relaxed);
        let db_time = self.db_total_time.load(Ordering::Relaxed);
        
        MetricsSnapshot {
            uptime_seconds: uptime.as_secs(),
            
            // 查询指标
            query_count,
            query_errors: self.query_errors.load(Ordering::Relaxed),
            avg_query_time_ms: if query_count > 0 { 
                query_total_time as f64 / query_count as f64 
            } else { 
                0.0 
            },
            queries_per_second: if uptime.as_secs() > 0 { 
                query_count as f64 / uptime.as_secs() as f64 
            } else { 
                0.0 
            },
            
            // 缓存指标
            cache_hits,
            cache_misses,
            cache_hit_rate: if cache_hits + cache_misses > 0 { 
                cache_hits as f64 / (cache_hits + cache_misses) as f64 
            } else { 
                0.0 
            },
            cache_size: self.cache_size.load(Ordering::Relaxed),
            
            // 内存指标
            memory_usage_bytes: self.memory_usage.load(Ordering::Relaxed),
            peak_memory_bytes: self.peak_memory.load(Ordering::Relaxed),
            
            // 向量操作指标
            vector_operations: vector_ops,
            avg_vector_time_ms: if vector_ops > 0 { 
                vector_time as f64 / vector_ops as f64 
            } else { 
                0.0 
            },
            
            // 数据库操作指标
            db_operations: db_ops,
            avg_db_time_ms: if db_ops > 0 { 
                db_time as f64 / db_ops as f64 
            } else { 
                0.0 
            },
        }
    }
    
    /// 重置所有指标
    pub fn reset(&self) {
        self.query_count.store(0, Ordering::Relaxed);
        self.query_total_time.store(0, Ordering::Relaxed);
        self.query_errors.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.cache_size.store(0, Ordering::Relaxed);
        self.memory_usage.store(0, Ordering::Relaxed);
        self.peak_memory.store(0, Ordering::Relaxed);
        self.vector_operations.store(0, Ordering::Relaxed);
        self.vector_total_time.store(0, Ordering::Relaxed);
        self.db_operations.store(0, Ordering::Relaxed);
        self.db_total_time.store(0, Ordering::Relaxed);
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能指标快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub uptime_seconds: u64,
    
    // 查询指标
    pub query_count: u64,
    pub query_errors: u64,
    pub avg_query_time_ms: f64,
    pub queries_per_second: f64,
    
    // 缓存指标
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
    pub cache_size: usize,
    
    // 内存指标
    pub memory_usage_bytes: usize,
    pub peak_memory_bytes: usize,
    
    // 向量操作指标
    pub vector_operations: u64,
    pub avg_vector_time_ms: f64,
    
    // 数据库操作指标
    pub db_operations: u64,
    pub avg_db_time_ms: f64,
}

impl MetricsSnapshot {
    /// 格式化为人类可读的字符串
    pub fn format_human_readable(&self) -> String {
        format!(
            "ZhushoudeDB 性能指标报告\n\
            ========================\n\
            运行时间: {}秒\n\
            \n\
            查询统计:\n\
            - 总查询数: {}\n\
            - 查询错误: {}\n\
            - 平均查询时间: {:.2}ms\n\
            - 查询速率: {:.2} QPS\n\
            \n\
            缓存统计:\n\
            - 缓存命中: {}\n\
            - 缓存未命中: {}\n\
            - 缓存命中率: {:.1}%\n\
            - 缓存大小: {}\n\
            \n\
            内存统计:\n\
            - 当前内存使用: {:.2} MB\n\
            - 峰值内存使用: {:.2} MB\n\
            \n\
            向量操作:\n\
            - 向量操作数: {}\n\
            - 平均向量操作时间: {:.2}ms\n\
            \n\
            数据库操作:\n\
            - 数据库操作数: {}\n\
            - 平均数据库操作时间: {:.2}ms",
            self.uptime_seconds,
            self.query_count,
            self.query_errors,
            self.avg_query_time_ms,
            self.queries_per_second,
            self.cache_hits,
            self.cache_misses,
            self.cache_hit_rate * 100.0,
            self.cache_size,
            self.memory_usage_bytes as f64 / 1024.0 / 1024.0,
            self.peak_memory_bytes as f64 / 1024.0 / 1024.0,
            self.vector_operations,
            self.avg_vector_time_ms,
            self.db_operations,
            self.avg_db_time_ms
        )
    }
    
    /// 导出为JSON格式
    pub fn to_json(&self) -> crate::Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| crate::Error::SerializationError(e))
    }
}

/// 性能计时器
pub struct Timer {
    start: Instant,
    metrics: Arc<MetricsCollector>,
    operation_type: TimerType,
}

#[derive(Debug, Clone)]
pub enum TimerType {
    Query,
    Vector,
    Database,
}

impl Timer {
    /// 创建新的计时器
    pub fn new(metrics: Arc<MetricsCollector>, operation_type: TimerType) -> Self {
        Self {
            start: Instant::now(),
            metrics,
            operation_type,
        }
    }
    
    /// 完成计时并记录结果
    pub fn finish(self, success: bool) {
        let duration = self.start.elapsed();
        
        match self.operation_type {
            TimerType::Query => self.metrics.record_query(duration, success),
            TimerType::Vector => self.metrics.record_vector_operation(duration),
            TimerType::Database => self.metrics.record_db_operation(duration),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_metrics_collector() {
        let metrics = MetricsCollector::new();
        
        // 记录一些操作
        metrics.record_query(Duration::from_millis(100), true);
        metrics.record_query(Duration::from_millis(200), false);
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        metrics.update_memory_usage(1024 * 1024);
        
        let snapshot = metrics.get_snapshot();
        
        assert_eq!(snapshot.query_count, 2);
        assert_eq!(snapshot.query_errors, 1);
        assert_eq!(snapshot.cache_hits, 1);
        assert_eq!(snapshot.cache_misses, 1);
        assert_eq!(snapshot.cache_hit_rate, 0.5);
        assert_eq!(snapshot.memory_usage_bytes, 1024 * 1024);
    }
    
    #[test]
    fn test_timer() {
        let metrics = Arc::new(MetricsCollector::new());
        
        {
            let timer = Timer::new(metrics.clone(), TimerType::Query);
            thread::sleep(Duration::from_millis(10));
            timer.finish(true);
        }
        
        let snapshot = metrics.get_snapshot();
        assert_eq!(snapshot.query_count, 1);
        assert!(snapshot.avg_query_time_ms >= 10.0);
    }
    
    #[test]
    fn test_snapshot_formatting() {
        let metrics = MetricsCollector::new();
        metrics.record_query(Duration::from_millis(100), true);
        metrics.record_cache_hit();
        
        let snapshot = metrics.get_snapshot();
        let formatted = snapshot.format_human_readable();
        
        assert!(formatted.contains("ZhushoudeDB 性能指标报告"));
        assert!(formatted.contains("总查询数: 1"));
        assert!(formatted.contains("缓存命中: 1"));
    }
    
    #[test]
    fn test_json_export() {
        let metrics = MetricsCollector::new();
        metrics.record_query(Duration::from_millis(50), true);

        let snapshot = metrics.get_snapshot();
        let json = snapshot.to_json().unwrap();

        assert!(json.contains("query_count"));
        assert!(json.contains("1")); // 移除引号，因为数字在JSON中不带引号
    }
}
