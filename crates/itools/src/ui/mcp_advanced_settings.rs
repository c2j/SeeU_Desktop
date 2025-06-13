use std::collections::HashMap;
use uuid::Uuid;
use egui::{Context, Ui, RichText, Color32, Button, TextEdit, ComboBox};
use tokio::sync::mpsc;

use crate::mcp::{
    McpServerManager, ServerTemplate, ServerTemplateManager, TemplateCategory,
    PerformanceMonitor, ServerMetrics, HealthStatus,
    BatchOperationsManager, BatchOperation, BatchOperationResult,
    server_manager::TransportConfig
};

/// Advanced MCP Settings UI with templates, monitoring, and batch operations
#[derive(Debug)]
pub struct AdvancedMcpSettingsUi {
    /// Server manager
    server_manager: McpServerManager,
    
    /// Template manager
    template_manager: ServerTemplateManager,
    
    /// Performance monitor
    performance_monitor: PerformanceMonitor,
    
    /// Batch operations manager
    batch_manager: BatchOperationsManager,
    
    /// UI state
    ui_state: AdvancedMcpUiState,
    
    /// Selected servers for batch operations
    selected_servers: Vec<Uuid>,
}

/// Advanced UI state
#[derive(Debug)]
struct AdvancedMcpUiState {
    /// Current tab
    current_tab: McpTab,
    
    /// Show template browser
    show_template_browser: bool,
    
    /// Show performance dashboard
    show_performance_dashboard: bool,
    
    /// Show batch operations panel
    show_batch_operations: bool,
    
    /// Selected template category
    selected_template_category: Option<String>,
    
    /// Template search query
    template_search_query: String,
    
    /// Performance filter
    performance_filter: PerformanceFilter,
    
    /// Batch operation type
    selected_batch_operation: BatchOperationType,
    
    /// Status messages
    status_message: Option<String>,
    error_message: Option<String>,
}

/// MCP settings tabs
#[derive(Debug, Clone, PartialEq)]
enum McpTab {
    Servers,
    Templates,
    Performance,
    BatchOps,
    Settings,
}

/// Performance filter options
#[derive(Debug, Clone)]
enum PerformanceFilter {
    All,
    Healthy,
    Warning,
    Critical,
    HighLatency,
    HighErrorRate,
}

/// Batch operation types for UI
#[derive(Debug, Clone)]
enum BatchOperationType {
    Connect,
    Disconnect,
    Enable,
    Disable,
    Test,
    Restart,
    Delete,
    Export,
}

impl AdvancedMcpSettingsUi {
    /// Create a new advanced MCP settings UI
    pub fn new(config_path: std::path::PathBuf) -> Self {
        Self {
            server_manager: McpServerManager::new(config_path),
            template_manager: ServerTemplateManager::new(),
            performance_monitor: PerformanceMonitor::new(),
            batch_manager: BatchOperationsManager::new(),
            ui_state: AdvancedMcpUiState::default(),
            selected_servers: Vec::new(),
        }
    }

    /// Initialize the advanced UI
    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        self.server_manager.initialize().await?;
        
        // Setup event channels for monitoring and batch operations
        let (perf_sender, _perf_receiver) = mpsc::unbounded_channel();
        let (batch_sender, _batch_receiver) = mpsc::unbounded_channel();
        
        self.performance_monitor.set_event_sender(perf_sender);
        self.batch_manager.set_event_sender(batch_sender);
        
        Ok(())
    }

    /// Render the advanced MCP settings UI
    pub fn render(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.heading("🚀 Advanced MCP Settings");
        ui.separator();

        // Tab bar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.ui_state.current_tab, McpTab::Servers, "🖥️ Servers");
            ui.selectable_value(&mut self.ui_state.current_tab, McpTab::Templates, "📋 Templates");
            ui.selectable_value(&mut self.ui_state.current_tab, McpTab::Performance, "📊 Performance");
            ui.selectable_value(&mut self.ui_state.current_tab, McpTab::BatchOps, "⚡ Batch Ops");
            ui.selectable_value(&mut self.ui_state.current_tab, McpTab::Settings, "⚙️ Settings");
        });

        ui.separator();

        // Status messages
        if let Some(message) = &self.ui_state.status_message {
            ui.colored_label(Color32::GREEN, message);
        }
        if let Some(error) = &self.ui_state.error_message {
            ui.colored_label(Color32::RED, error);
        }

        // Tab content
        match self.ui_state.current_tab {
            McpTab::Servers => self.render_servers_tab(ui),
            McpTab::Templates => self.render_templates_tab(ui),
            McpTab::Performance => self.render_performance_tab(ui),
            McpTab::BatchOps => self.render_batch_operations_tab(ui),
            McpTab::Settings => self.render_settings_tab(ui),
        }

        // Dialogs
        self.render_template_browser(ctx);
        self.render_performance_dashboard(ctx);
        self.render_batch_operations_panel(ctx);
    }

    /// Render servers tab with enhanced features
    fn render_servers_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("➕ Add Server").clicked() {
                // TODO: Show add server dialog
            }
            
            if ui.button("📋 From Template").clicked() {
                self.ui_state.show_template_browser = true;
            }
            
            if ui.button("📊 Performance").clicked() {
                self.ui_state.show_performance_dashboard = true;
            }
            
            if ui.button("⚡ Batch Ops").clicked() {
                self.ui_state.show_batch_operations = true;
            }
            
            ui.separator();
            
            if ui.button("🔄 Refresh").clicked() {
                // TODO: Refresh server list
            }
        });

        ui.separator();

        // Server selection controls
        ui.horizontal(|ui| {
            if ui.button("☑️ Select All").clicked() {
                self.selected_servers = self.server_manager.list_servers()
                    .into_iter()
                    .map(|info| info.id)
                    .collect();
            }
            
            if ui.button("☐ Select None").clicked() {
                self.selected_servers.clear();
            }
            
            ui.label(format!("Selected: {}", self.selected_servers.len()));
        });

        ui.separator();

        // Enhanced server list with selection and performance indicators
        egui::ScrollArea::vertical().show(ui, |ui| {
            let directories = self.server_manager.get_server_directories();
            
            for directory in directories {
                ui.collapsing(&directory.name, |ui| {
                    for server in &directory.servers {
                        self.render_enhanced_server_item(ui, server);
                    }
                });
            }
        });
    }

    /// Render enhanced server item with selection and performance
    fn render_enhanced_server_item(&mut self, ui: &mut Ui, server: &crate::mcp::McpServerConfig) {
        // Generate a mock server ID for demonstration
        let server_id = Uuid::new_v4();
        let is_selected = self.selected_servers.contains(&server_id);
        
        ui.horizontal(|ui| {
            // Selection checkbox
            let mut selected = is_selected;
            if ui.checkbox(&mut selected, "").changed() {
                if selected && !is_selected {
                    self.selected_servers.push(server_id);
                } else if !selected && is_selected {
                    self.selected_servers.retain(|&id| id != server_id);
                }
            }
            
            // Health indicator
            let health_status = self.performance_monitor.get_health_status(server_id);
            let (health_color, health_icon) = match health_status {
                HealthStatus::Healthy => (Color32::GREEN, "🟢"),
                HealthStatus::Warning => (Color32::YELLOW, "🟡"),
                HealthStatus::Critical => (Color32::RED, "🔴"),
                HealthStatus::Unknown => (Color32::GRAY, "⚪"),
            };
            ui.colored_label(health_color, health_icon);
            
            // Status indicator
            let status_color = if server.enabled { Color32::GREEN } else { Color32::GRAY };
            ui.colored_label(status_color, "●");

            // Server info
            ui.vertical(|ui| {
                ui.label(RichText::new(&server.name).strong());
                if let Some(desc) = &server.description {
                    ui.label(RichText::new(desc).small().color(Color32::GRAY));
                }
                
                // Performance metrics
                if let Some(metrics) = self.performance_monitor.get_metrics(server_id) {
                    ui.horizontal(|ui| {
                        ui.label(format!("⏱️ {}ms", metrics.average_response_time.as_millis()));
                        ui.label(format!("❌ {:.1}%", metrics.error_rate));
                        ui.label(format!("📊 {} reqs", metrics.total_requests));
                    });
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Action buttons
                if ui.small_button("📊").on_hover_text("Performance").clicked() {
                    // TODO: Show server performance details
                }
                
                if ui.small_button("🔧").on_hover_text("Settings").clicked() {
                    // TODO: Open server settings
                }

                if ui.small_button("🧪").on_hover_text("Test").clicked() {
                    // TODO: Test server connection
                }

                let connect_button = if server.enabled {
                    Button::new("🔌").fill(Color32::from_rgb(100, 200, 100))
                } else {
                    Button::new("⚡").fill(Color32::from_rgb(200, 100, 100))
                };

                if ui.add(connect_button).on_hover_text("Connect/Disconnect").clicked() {
                    // TODO: Toggle server connection
                }
            });
        });
        ui.separator();
    }

    /// Render templates tab
    fn render_templates_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.add(TextEdit::singleline(&mut self.ui_state.template_search_query).desired_width(200.0));
            
            ui.label("Category:");
            ComboBox::from_id_source("template_category")
                .selected_text(
                    self.ui_state.selected_template_category
                        .as_deref()
                        .unwrap_or("All")
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.ui_state.selected_template_category, None, "All");
                    for category in self.template_manager.get_categories() {
                        ui.selectable_value(
                            &mut self.ui_state.selected_template_category,
                            Some(category.id.clone()),
                            &format!("{} {}", category.icon, category.name)
                        );
                    }
                });
        });

        ui.separator();

        // Template grid
        egui::ScrollArea::vertical().show(ui, |ui| {
            let templates = if self.ui_state.template_search_query.is_empty() {
                if let Some(category_id) = &self.ui_state.selected_template_category {
                    self.template_manager.get_templates_by_category(category_id)
                } else {
                    self.template_manager.get_templates()
                }
            } else {
                self.template_manager.search_templates(&self.ui_state.template_search_query)
            };

            for template in templates {
                self.render_template_card(ui, template);
            }
        });
    }

    /// Render template card
    fn render_template_card(&mut self, ui: &mut Ui, template: &ServerTemplate) {
        egui::Frame::none()
            .fill(ui.style().visuals.faint_bg_color)
            .inner_margin(egui::Margin::same(8.0))
            .outer_margin(egui::Margin::same(4.0))
            .rounding(egui::Rounding::same(4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&template.icon).size(24.0));
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&template.name).strong());
                        ui.label(RichText::new(&template.description).small());
                        
                        // Tags
                        ui.horizontal(|ui| {
                            for tag in &template.tags {
                                ui.small_button(tag);
                            }
                        });
                        
                        // Requirements
                        if !template.requirements.is_empty() {
                            ui.label(format!("Requirements: {}", template.requirements.join(", ")));
                        }
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📋 Use Template").clicked() {
                            // TODO: Create server from template
                        }
                        
                        if ui.small_button("ℹ️").on_hover_text("More Info").clicked() {
                            // TODO: Show template details
                        }
                    });
                });
            });
    }

    /// Render performance tab
    fn render_performance_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ComboBox::from_id_source("performance_filter")
                .selected_text(format!("{:?}", self.ui_state.performance_filter))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.ui_state.performance_filter, PerformanceFilter::All, "All");
                    ui.selectable_value(&mut self.ui_state.performance_filter, PerformanceFilter::Healthy, "Healthy");
                    ui.selectable_value(&mut self.ui_state.performance_filter, PerformanceFilter::Warning, "Warning");
                    ui.selectable_value(&mut self.ui_state.performance_filter, PerformanceFilter::Critical, "Critical");
                    ui.selectable_value(&mut self.ui_state.performance_filter, PerformanceFilter::HighLatency, "High Latency");
                    ui.selectable_value(&mut self.ui_state.performance_filter, PerformanceFilter::HighErrorRate, "High Error Rate");
                });
        });

        ui.separator();

        // Performance overview
        let all_metrics = self.performance_monitor.get_all_metrics();
        
        ui.horizontal(|ui| {
            ui.label(format!("Total Servers: {}", all_metrics.len()));
            
            let healthy_count = all_metrics.iter()
                .filter(|m| self.performance_monitor.get_health_status(m.server_id) == HealthStatus::Healthy)
                .count();
            ui.colored_label(Color32::GREEN, format!("Healthy: {}", healthy_count));
            
            let warning_count = all_metrics.iter()
                .filter(|m| self.performance_monitor.get_health_status(m.server_id) == HealthStatus::Warning)
                .count();
            ui.colored_label(Color32::YELLOW, format!("Warning: {}", warning_count));
            
            let critical_count = all_metrics.iter()
                .filter(|m| self.performance_monitor.get_health_status(m.server_id) == HealthStatus::Critical)
                .count();
            ui.colored_label(Color32::RED, format!("Critical: {}", critical_count));
        });

        ui.separator();

        // Performance metrics table
        egui::ScrollArea::vertical().show(ui, |ui| {
            for metrics in all_metrics {
                self.render_performance_metrics_row(ui, metrics);
            }
        });
    }

    /// Render performance metrics row
    fn render_performance_metrics_row(&mut self, ui: &mut Ui, metrics: &ServerMetrics) {
        let health_status = self.performance_monitor.get_health_status(metrics.server_id);
        let (health_color, health_icon) = match health_status {
            HealthStatus::Healthy => (Color32::GREEN, "🟢"),
            HealthStatus::Warning => (Color32::YELLOW, "🟡"),
            HealthStatus::Critical => (Color32::RED, "🔴"),
            HealthStatus::Unknown => (Color32::GRAY, "⚪"),
        };

        ui.horizontal(|ui| {
            ui.colored_label(health_color, health_icon);
            ui.label(&metrics.server_name);
            ui.label(format!("{}ms", metrics.average_response_time.as_millis()));
            ui.label(format!("{:.1}%", metrics.error_rate));
            ui.label(format!("{}", metrics.total_requests));
            ui.label(format!("{}", metrics.successful_requests));
            ui.label(format!("{}", metrics.failed_requests));
            
            if let Some(memory) = metrics.memory_usage {
                ui.label(format!("{}MB", memory / 1024 / 1024));
            } else {
                ui.label("-");
            }
            
            if let Some(cpu) = metrics.cpu_usage {
                ui.label(format!("{:.1}%", cpu));
            } else {
                ui.label("-");
            }
        });
        ui.separator();
    }

    /// Render batch operations tab
    fn render_batch_operations_tab(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Operation:");
            ComboBox::from_id_source("batch_operation")
                .selected_text(format!("{:?}", self.ui_state.selected_batch_operation))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.ui_state.selected_batch_operation, BatchOperationType::Connect, "Connect");
                    ui.selectable_value(&mut self.ui_state.selected_batch_operation, BatchOperationType::Disconnect, "Disconnect");
                    ui.selectable_value(&mut self.ui_state.selected_batch_operation, BatchOperationType::Enable, "Enable");
                    ui.selectable_value(&mut self.ui_state.selected_batch_operation, BatchOperationType::Disable, "Disable");
                    ui.selectable_value(&mut self.ui_state.selected_batch_operation, BatchOperationType::Test, "Test");
                    ui.selectable_value(&mut self.ui_state.selected_batch_operation, BatchOperationType::Restart, "Restart");
                    ui.selectable_value(&mut self.ui_state.selected_batch_operation, BatchOperationType::Delete, "Delete");
                    ui.selectable_value(&mut self.ui_state.selected_batch_operation, BatchOperationType::Export, "Export");
                });
            
            if ui.button("▶️ Execute").clicked() && !self.selected_servers.is_empty() {
                // TODO: Execute batch operation
                self.execute_batch_operation();
            }
            
            ui.label(format!("Selected: {} servers", self.selected_servers.len()));
        });

        ui.separator();

        // Active operations
        ui.label(RichText::new("Active Operations").strong());
        for operation in self.batch_manager.get_active_operations() {
            self.render_batch_operation_status(ui, operation);
        }

        ui.separator();

        // Operation history
        ui.label(RichText::new("Recent Operations").strong());
        for operation in self.batch_manager.get_operation_history().iter().rev().take(10) {
            self.render_batch_operation_history(ui, operation);
        }
    }

    /// Render batch operation status
    fn render_batch_operation_status(&mut self, ui: &mut Ui, operation: &BatchOperationResult) {
        ui.horizontal(|ui| {
            ui.label(format!("{:?}", operation.operation));
            ui.label(format!("Progress: {}/{}", operation.successful.len() + operation.failed.len(), operation.total_servers));
            ui.label(format!("Status: {:?}", operation.status));
            
            if ui.small_button("❌").on_hover_text("Cancel").clicked() {
                // TODO: Cancel operation
            }
        });
    }

    /// Render batch operation history
    fn render_batch_operation_history(&mut self, ui: &mut Ui, operation: &BatchOperationResult) {
        ui.horizontal(|ui| {
            let status_color = match operation.status {
                crate::mcp::batch_operations::BatchOperationStatus::Completed => Color32::GREEN,
                crate::mcp::batch_operations::BatchOperationStatus::Failed => Color32::RED,
                crate::mcp::batch_operations::BatchOperationStatus::Cancelled => Color32::YELLOW,
                _ => Color32::GRAY,
            };
            
            ui.colored_label(status_color, "●");
            ui.label(format!("{:?}", operation.operation));
            ui.label(format!("✅ {} ❌ {}", operation.successful.len(), operation.failed.len()));
            ui.label(operation.started_at.format("%H:%M:%S").to_string());
        });
    }

    /// Render settings tab
    fn render_settings_tab(&mut self, ui: &mut Ui) {
        ui.label("MCP Settings Configuration");
        ui.separator();
        
        // TODO: Add configuration options
        ui.label("Coming soon: Advanced configuration options");
    }

    /// Execute batch operation
    fn execute_batch_operation(&mut self) {
        if self.selected_servers.is_empty() {
            self.ui_state.error_message = Some("No servers selected".to_string());
            return;
        }

        let operation = match self.ui_state.selected_batch_operation {
            BatchOperationType::Connect => BatchOperation::Connect(self.selected_servers.clone()),
            BatchOperationType::Disconnect => BatchOperation::Disconnect(self.selected_servers.clone()),
            BatchOperationType::Enable => BatchOperation::Enable(self.selected_servers.clone()),
            BatchOperationType::Disable => BatchOperation::Disable(self.selected_servers.clone()),
            BatchOperationType::Test => BatchOperation::Test(self.selected_servers.clone()),
            BatchOperationType::Restart => BatchOperation::Restart(self.selected_servers.clone()),
            BatchOperationType::Delete => BatchOperation::Delete(self.selected_servers.clone()),
            BatchOperationType::Export => BatchOperation::Export(self.selected_servers.clone(), "export.json".to_string()),
        };

        // TODO: Execute the operation asynchronously
        self.ui_state.status_message = Some(format!("Executing {:?} on {} servers", operation, self.selected_servers.len()));
    }

    /// Render template browser dialog
    fn render_template_browser(&mut self, ctx: &Context) {
        if !self.ui_state.show_template_browser {
            return;
        }

        egui::Window::new("📋 Template Browser")
            .collapsible(false)
            .resizable(true)
            .default_size([800.0, 600.0])
            .show(ctx, |ui| {
                // TODO: Implement template browser
                ui.label("Template browser coming soon...");
                
                if ui.button("Close").clicked() {
                    self.ui_state.show_template_browser = false;
                }
            });
    }

    /// Render performance dashboard dialog
    fn render_performance_dashboard(&mut self, ctx: &Context) {
        if !self.ui_state.show_performance_dashboard {
            return;
        }

        egui::Window::new("📊 Performance Dashboard")
            .collapsible(false)
            .resizable(true)
            .default_size([1000.0, 700.0])
            .show(ctx, |ui| {
                // TODO: Implement performance dashboard with charts
                ui.label("Performance dashboard coming soon...");
                
                if ui.button("Close").clicked() {
                    self.ui_state.show_performance_dashboard = false;
                }
            });
    }

    /// Render batch operations panel dialog
    fn render_batch_operations_panel(&mut self, ctx: &Context) {
        if !self.ui_state.show_batch_operations {
            return;
        }

        egui::Window::new("⚡ Batch Operations")
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 400.0])
            .show(ctx, |ui| {
                // TODO: Implement batch operations panel
                ui.label("Batch operations panel coming soon...");
                
                if ui.button("Close").clicked() {
                    self.ui_state.show_batch_operations = false;
                }
            });
    }
}

impl Default for AdvancedMcpUiState {
    fn default() -> Self {
        Self {
            current_tab: McpTab::Servers,
            show_template_browser: false,
            show_performance_dashboard: false,
            show_batch_operations: false,
            selected_template_category: None,
            template_search_query: String::new(),
            performance_filter: PerformanceFilter::All,
            selected_batch_operation: BatchOperationType::Connect,
            status_message: None,
            error_message: None,
        }
    }
}
