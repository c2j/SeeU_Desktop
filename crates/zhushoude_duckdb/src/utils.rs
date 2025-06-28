//! 工具函数模块
//! 
//! 提供各种辅助工具函数

use std::time::{Duration, Instant};

/// 性能计时器
pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    /// 创建新的计时器
    pub fn new(name: &str) -> Self {
        Self {
            start: Instant::now(),
            name: name.to_string(),
        }
    }
    
    /// 获取经过的时间
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
    
    /// 获取经过的毫秒数
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }
    
    /// 记录并打印经过的时间
    pub fn log_elapsed(&self) {
        log::debug!("{} 耗时: {}ms", self.name, self.elapsed_ms());
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.log_elapsed();
    }
}

/// 内存使用监控
pub struct MemoryMonitor;

impl MemoryMonitor {
    /// 获取当前进程内存使用 (MB)
    pub fn get_memory_usage_mb() -> f64 {
        // 简化实现，实际应该使用系统API
        0.0
    }
    
    /// 检查内存使用是否超过限制
    pub fn check_memory_limit(limit_mb: usize) -> bool {
        Self::get_memory_usage_mb() > limit_mb as f64
    }
}

/// 文本处理工具
pub struct TextUtils;

impl TextUtils {
    /// 计算文本的字符数
    pub fn char_count(text: &str) -> usize {
        text.chars().count()
    }
    
    /// 截断文本到指定字符数
    pub fn truncate(text: &str, max_chars: usize) -> String {
        if text.chars().count() <= max_chars {
            text.to_string()
        } else {
            text.chars().take(max_chars).collect::<String>() + "..."
        }
    }
    
    /// 清理文本中的特殊字符
    pub fn clean_text(text: &str) -> String {
        text.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?;:\"'()[]{}".contains(*c))
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// 向量工具
pub struct VectorUtils;

impl VectorUtils {
    /// 计算两个向量的余弦相似度
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
    
    /// 向量L2归一化
    pub fn normalize_l2(vector: &mut [f32]) {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in vector.iter_mut() {
                *x /= norm;
            }
        }
    }
    
    /// 计算向量的L2范数
    pub fn l2_norm(vector: &[f32]) -> f32 {
        vector.iter().map(|x| x * x).sum::<f32>().sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timer() {
        let timer = Timer::new("test");
        std::thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed_ms() >= 10);
    }
    
    #[test]
    fn test_text_utils() {
        assert_eq!(TextUtils::char_count("hello"), 5);
        assert_eq!(TextUtils::char_count("你好"), 2);
        
        assert_eq!(TextUtils::truncate("hello world", 5), "hello...");
        assert_eq!(TextUtils::truncate("hi", 5), "hi");
        
        let cleaned = TextUtils::clean_text("  hello,   world!  ");
        assert_eq!(cleaned, "hello, world!");
    }
    
    #[test]
    fn test_vector_utils() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let c = vec![1.0, 0.0, 0.0];
        
        assert!((VectorUtils::cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
        assert!((VectorUtils::cosine_similarity(&a, &c) - 1.0).abs() < 1e-6);
        
        let mut vector = vec![3.0, 4.0];
        VectorUtils::normalize_l2(&mut vector);
        assert!((VectorUtils::l2_norm(&vector) - 1.0).abs() < 1e-6);
    }
}
