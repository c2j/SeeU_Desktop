use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Performance metrics for MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub server_id: Uuid,
    pub server_name: String,
    
    // Connection metrics
    pub connection_time: Option<Duration>,
    pub last_ping_time: Option<Duration>,
    pub uptime: Duration,
    pub connection_count: u64,
    pub disconnection_count: u64,
    
    // Request metrics
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub max_response_time: Duration,
    pub min_response_time: Duration,
    
    // Resource usage
    pub memory_usage: Option<u64>, // bytes
    pub cpu_usage: Option<f32>,    // percentage
    
    // Error metrics
    pub error_rate: f32,           // percentage
    pub last_error: Option<String>,
    pub error_count_24h: u64,
    
    // Capability metrics
    pub tools_count: u32,
    pub resources_count: u32,
    pub prompts_count: u32,
    
    // Timestamps
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Real-time performance data point
#[derive(Debug, Clone)]
pub struct PerformanceDataPoint {
    pub timestamp: Instant,
    pub response_time: Duration,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Performance monitor for MCP servers
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Server metrics
    metrics: HashMap<Uuid, ServerMetrics>,
    
    /// Real-time performance data (last 1000 points per server)
    performance_data: HashMap<Uuid, VecDeque<PerformanceDataPoint>>,
    
    /// Performance event sender
    event_sender: Option<mpsc::UnboundedSender<PerformanceEvent>>,
    
    /// Monitoring configuration
    config: MonitoringConfig,
}

/// Performance monitoring events
#[derive(Debug, Clone)]
pub enum PerformanceEvent {
    /// Server performance updated
    MetricsUpdated(Uuid, ServerMetrics),
    
    /// Performance threshold exceeded
    ThresholdExceeded(Uuid, ThresholdType, f64),
    
    /// Server health status changed
    HealthStatusChanged(Uuid, HealthStatus),
    
    /// Performance alert
    Alert(Uuid, AlertLevel, String),
}

/// Threshold types for monitoring
#[derive(Debug, Clone)]
pub enum ThresholdType {
    ResponseTime(Duration),
    ErrorRate(f32),
    MemoryUsage(u64),
    CpuUsage(f32),
}

/// Health status of a server
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Alert levels
#[derive(Debug, Clone)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Maximum data points to keep per server
    pub max_data_points: usize,
    
    /// Response time threshold for warnings (ms)
    pub response_time_warning: Duration,
    
    /// Response time threshold for errors (ms)
    pub response_time_error: Duration,
    
    /// Error rate threshold for warnings (%)
    pub error_rate_warning: f32,
    
    /// Error rate threshold for errors (%)
    pub error_rate_error: f32,
    
    /// Memory usage threshold for warnings (MB)
    pub memory_warning: u64,
    
    /// Memory usage threshold for errors (MB)
    pub memory_error: u64,
    
    /// CPU usage threshold for warnings (%)
    pub cpu_warning: f32,
    
    /// CPU usage threshold for errors (%)
    pub cpu_error: f32,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            performance_data: HashMap::new(),
            event_sender: None,
            config: MonitoringConfig::default(),
        }
    }

    /// Set event sender
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<PerformanceEvent>) {
        self.event_sender = Some(sender);
    }

    /// Start monitoring a server
    pub fn start_monitoring(&mut self, server_id: Uuid, server_name: String) {
        let metrics = ServerMetrics {
            server_id,
            server_name,
            connection_time: None,
            last_ping_time: None,
            uptime: Duration::ZERO,
            connection_count: 0,
            disconnection_count: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: Duration::ZERO,
            max_response_time: Duration::ZERO,
            min_response_time: Duration::MAX,
            memory_usage: None,
            cpu_usage: None,
            error_rate: 0.0,
            last_error: None,
            error_count_24h: 0,
            tools_count: 0,
            resources_count: 0,
            prompts_count: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.metrics.insert(server_id, metrics.clone());
        self.performance_data.insert(server_id, VecDeque::new());
        
        self.send_event(PerformanceEvent::MetricsUpdated(server_id, metrics));
    }

    /// Stop monitoring a server
    pub fn stop_monitoring(&mut self, server_id: Uuid) {
        self.metrics.remove(&server_id);
        self.performance_data.remove(&server_id);
    }

    /// Record server connection
    pub fn record_connection(&mut self, server_id: Uuid, connection_time: Duration) {
        let updated_metrics = if let Some(metrics) = self.metrics.get_mut(&server_id) {
            metrics.connection_time = Some(connection_time);
            metrics.connection_count += 1;
            metrics.updated_at = chrono::Utc::now();
            Some(metrics.clone())
        } else {
            None
        };

        if let Some(metrics) = updated_metrics {
            self.send_event(PerformanceEvent::MetricsUpdated(server_id, metrics));
        }
    }

    /// Record server disconnection
    pub fn record_disconnection(&mut self, server_id: Uuid) {
        let updated_metrics = if let Some(metrics) = self.metrics.get_mut(&server_id) {
            metrics.disconnection_count += 1;
            metrics.updated_at = chrono::Utc::now();
            Some(metrics.clone())
        } else {
            None
        };

        if let Some(metrics) = updated_metrics {
            self.send_event(PerformanceEvent::MetricsUpdated(server_id, metrics));
        }
    }

    /// Record request performance
    pub fn record_request(&mut self, server_id: Uuid, response_time: Duration, success: bool, error_message: Option<String>) {
        let (updated_metrics, should_check_thresholds) = if let Some(metrics) = self.metrics.get_mut(&server_id) {
            metrics.total_requests += 1;

            if success {
                metrics.successful_requests += 1;
            } else {
                metrics.failed_requests += 1;
                if let Some(error) = &error_message {
                    metrics.last_error = Some(error.clone());
                }
                metrics.error_count_24h += 1;
            }

            // Update response time statistics
            if response_time > metrics.max_response_time {
                metrics.max_response_time = response_time;
            }
            if response_time < metrics.min_response_time {
                metrics.min_response_time = response_time;
            }

            // Calculate average response time
            let total_time = metrics.average_response_time.as_nanos() as u64 * (metrics.total_requests - 1) + response_time.as_nanos() as u64;
            metrics.average_response_time = Duration::from_nanos(total_time / metrics.total_requests);

            // Calculate error rate
            metrics.error_rate = (metrics.failed_requests as f32 / metrics.total_requests as f32) * 100.0;

            metrics.updated_at = chrono::Utc::now();

            (Some(metrics.clone()), true)
        } else {
            (None, false)
        };

        // Add performance data point
        if let Some(data) = self.performance_data.get_mut(&server_id) {
            data.push_back(PerformanceDataPoint {
                timestamp: Instant::now(),
                response_time,
                success,
                error_message,
            });

            // Keep only the last N data points
            while data.len() > self.config.max_data_points {
                data.pop_front();
            }
        }

        // Check thresholds and send events
        if let Some(metrics) = updated_metrics {
            if should_check_thresholds {
                self.check_thresholds(server_id, &metrics);
            }
            self.send_event(PerformanceEvent::MetricsUpdated(server_id, metrics));
        }
    }

    /// Update server capabilities
    pub fn update_capabilities(&mut self, server_id: Uuid, tools_count: u32, resources_count: u32, prompts_count: u32) {
        let updated_metrics = if let Some(metrics) = self.metrics.get_mut(&server_id) {
            metrics.tools_count = tools_count;
            metrics.resources_count = resources_count;
            metrics.prompts_count = prompts_count;
            metrics.updated_at = chrono::Utc::now();
            Some(metrics.clone())
        } else {
            None
        };

        if let Some(metrics) = updated_metrics {
            self.send_event(PerformanceEvent::MetricsUpdated(server_id, metrics));
        }
    }

    /// Update resource usage
    pub fn update_resource_usage(&mut self, server_id: Uuid, memory_usage: Option<u64>, cpu_usage: Option<f32>) {
        let updated_metrics = if let Some(metrics) = self.metrics.get_mut(&server_id) {
            metrics.memory_usage = memory_usage;
            metrics.cpu_usage = cpu_usage;
            metrics.updated_at = chrono::Utc::now();
            Some(metrics.clone())
        } else {
            None
        };

        // Check resource thresholds
        if let Some(memory) = memory_usage {
            if memory > self.config.memory_error {
                self.send_event(PerformanceEvent::ThresholdExceeded(server_id, ThresholdType::MemoryUsage(memory), memory as f64));
            } else if memory > self.config.memory_warning {
                self.send_event(PerformanceEvent::Alert(server_id, AlertLevel::Warning, format!("High memory usage: {} MB", memory / 1024 / 1024)));
            }
        }

        if let Some(cpu) = cpu_usage {
            if cpu > self.config.cpu_error {
                self.send_event(PerformanceEvent::ThresholdExceeded(server_id, ThresholdType::CpuUsage(cpu), cpu as f64));
            } else if cpu > self.config.cpu_warning {
                self.send_event(PerformanceEvent::Alert(server_id, AlertLevel::Warning, format!("High CPU usage: {:.1}%", cpu)));
            }
        }

        if let Some(metrics) = updated_metrics {
            self.send_event(PerformanceEvent::MetricsUpdated(server_id, metrics));
        }
    }

    /// Get metrics for a server
    pub fn get_metrics(&self, server_id: Uuid) -> Option<&ServerMetrics> {
        self.metrics.get(&server_id)
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> Vec<&ServerMetrics> {
        self.metrics.values().collect()
    }

    /// Get performance data for a server
    pub fn get_performance_data(&self, server_id: Uuid) -> Option<&VecDeque<PerformanceDataPoint>> {
        self.performance_data.get(&server_id)
    }

    /// Get health status for a server
    pub fn get_health_status(&self, server_id: Uuid) -> HealthStatus {
        if let Some(metrics) = self.metrics.get(&server_id) {
            if metrics.error_rate > self.config.error_rate_error ||
               metrics.average_response_time > self.config.response_time_error {
                return HealthStatus::Critical;
            }
            
            if metrics.error_rate > self.config.error_rate_warning ||
               metrics.average_response_time > self.config.response_time_warning {
                return HealthStatus::Warning;
            }
            
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }

    /// Check performance thresholds
    fn check_thresholds(&self, server_id: Uuid, metrics: &ServerMetrics) {
        // Check response time thresholds
        if metrics.average_response_time > self.config.response_time_error {
            self.send_event(PerformanceEvent::ThresholdExceeded(
                server_id,
                ThresholdType::ResponseTime(metrics.average_response_time),
                metrics.average_response_time.as_millis() as f64
            ));
        } else if metrics.average_response_time > self.config.response_time_warning {
            self.send_event(PerformanceEvent::Alert(
                server_id,
                AlertLevel::Warning,
                format!("High response time: {}ms", metrics.average_response_time.as_millis())
            ));
        }

        // Check error rate thresholds
        if metrics.error_rate > self.config.error_rate_error {
            self.send_event(PerformanceEvent::ThresholdExceeded(
                server_id,
                ThresholdType::ErrorRate(metrics.error_rate),
                metrics.error_rate as f64
            ));
        } else if metrics.error_rate > self.config.error_rate_warning {
            self.send_event(PerformanceEvent::Alert(
                server_id,
                AlertLevel::Warning,
                format!("High error rate: {:.1}%", metrics.error_rate)
            ));
        }
    }

    /// Send performance event
    fn send_event(&self, event: PerformanceEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            max_data_points: 1000,
            response_time_warning: Duration::from_millis(1000),
            response_time_error: Duration::from_millis(5000),
            error_rate_warning: 5.0,
            error_rate_error: 15.0,
            memory_warning: 512 * 1024 * 1024, // 512 MB
            memory_error: 1024 * 1024 * 1024,  // 1 GB
            cpu_warning: 70.0,
            cpu_error: 90.0,
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}
