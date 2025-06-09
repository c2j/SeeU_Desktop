use sysinfo::{System, SystemExt, CpuExt};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// System service for monitoring system resources
pub struct SystemService {
    system: Arc<Mutex<System>>,
    last_cpu_usage: f32,
    initialized: bool,
}

impl SystemService {
    /// Create a new system service
    pub fn new() -> Self {
        let mut system = System::new_all();
        
        // 初始化时先刷新一次
        system.refresh_all();
        
        // 创建服务实例
        let mut service = Self {
            system: Arc::new(Mutex::new(system)),
            last_cpu_usage: 0.0,
            initialized: false,
        };
        
        // 初始化CPU使用率 - 需要两次采样
        service.initialize_cpu_usage();
        
        service
    }
    
    /// 初始化CPU使用率，确保有有效值
    fn initialize_cpu_usage(&mut self) {
        if let Ok(mut system) = self.system.lock() {
            // 第一次刷新
            system.refresh_cpu();
            
            // 短暂等待
            drop(system);
            thread::sleep(Duration::from_millis(100));
            
            // 第二次刷新并获取使用率
            if let Ok(mut system) = self.system.lock() {
                system.refresh_cpu();
                let usage = system.global_cpu_info().cpu_usage();
                
                // 检查是否为NaN，如果是则使用0.0
                self.last_cpu_usage = if usage.is_nan() { 0.0 } else { usage };
                self.initialized = true;
            }
        }
    }

    /// Get CPU usage as a percentage
    pub fn get_cpu_usage(&self) -> f32 {
        // 如果CPU使用率为NaN，返回0.0
        if self.last_cpu_usage.is_nan() {
            0.0
        } else {
            self.last_cpu_usage
        }
    }

    /// Get memory usage as a percentage
    pub fn get_memory_usage(&self) -> f32 {
        if let Ok(mut system) = self.system.lock() {
            system.refresh_memory();

            let total_memory = system.total_memory() as f32;
            let used_memory = system.used_memory() as f32;

            if total_memory > 0.0 {
                (used_memory / total_memory) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Refresh system information
    pub fn refresh(&mut self) {
        if let Ok(mut system) = self.system.lock() {
            // 刷新CPU信息
            system.refresh_cpu();
            
            // 获取并保存CPU使用率
            let global_cpu = system.global_cpu_info();
            let usage = global_cpu.cpu_usage();
            
            // 检查是否为NaN，如果是则保持上次的有效值
            if !usage.is_nan() {
                self.last_cpu_usage = usage;
            }
            
            // 刷新其他系统信息
            system.refresh_memory();
        }
    }
}
