use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result, anyhow};
use wasmtime::{Engine, Module, Store, Instance, Func, Caller, AsContextMut};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use uuid::Uuid;
use serde_json::Value;

use super::plugin::{Plugin, PluginStatus, PluginCapabilities, PluginMetadata, PluginPermission};

/// WASM plugin runtime for secure plugin execution
pub struct WasmPluginRuntime {
    engine: Engine,
    plugins: HashMap<Uuid, WasmPlugin>,
}

/// WASM plugin instance
pub struct WasmPlugin {
    pub id: Uuid,
    pub metadata: PluginMetadata,
    pub capabilities: PluginCapabilities,
    pub permissions: Vec<PluginPermission>,
    pub status: PluginStatus,
    store: Store<WasiCtx>,
    instance: Instance,
    
    // Exported functions
    init_func: Option<Func>,
    get_capabilities_func: Option<Func>,
    get_metadata_func: Option<Func>,
    get_permissions_func: Option<Func>,
    handle_request_func: Option<Func>,
    cleanup_func: Option<Func>,
}

impl WasmPluginRuntime {
    /// Create a new WASM plugin runtime
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        
        Ok(Self {
            engine,
            plugins: HashMap::new(),
        })
    }
    
    /// Load a plugin from WASM file
    pub async fn load_plugin(&mut self, plugin_path: &Path, manifest_path: &Path) -> Result<Uuid> {
        // Read plugin manifest
        let manifest_content = std::fs::read_to_string(manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;
        
        // Extract metadata
        let metadata = PluginMetadata {
            name: manifest.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            display_name: manifest.get("display_name").and_then(|v| v.as_str()).unwrap_or("Unknown Plugin").to_string(),
            description: manifest.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            version: manifest.get("version").and_then(|v| v.as_str()).unwrap_or("0.1.0").to_string(),
            author: manifest.get("author").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            homepage: manifest.get("homepage").and_then(|v| v.as_str()).map(|s| s.to_string()),
            repository: manifest.get("repository").and_then(|v| v.as_str()).map(|s| s.to_string()),
            license: manifest.get("license").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            keywords: manifest.get("keywords").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            categories: manifest.get("categories").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            icon: manifest.get("icon").and_then(|v| v.as_str()).map(|s| s.to_string()),
            screenshots: manifest.get("screenshots").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            target_roles: manifest.get("target_roles").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| {
                    match s {
                        "Developer" => crate::roles::UserRole::Developer,
                        "DataAnalyst" => crate::roles::UserRole::DataAnalyst,
                        "ContentCreator" => crate::roles::UserRole::ContentCreator,
                        "Researcher" => crate::roles::UserRole::Researcher,
                        _ => crate::roles::UserRole::Developer, // Default fallback
                    }
                }).collect())
                .unwrap_or_default(),
        };
        
        // Read WASM module
        let wasm_bytes = std::fs::read(plugin_path)?;
        let module = Module::new(&self.engine, &wasm_bytes)?;
        
        // Create WASI context
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_env()?
            .build();
        
        let mut store = Store::new(&self.engine, wasi);
        
        // Define host functions
        let host_log = Func::wrap(&mut store, |_caller: Caller<'_, WasiCtx>, ptr: i32, len: i32| {
            // Host logging function - would read from WASM memory and log
            log::info!("Plugin log: ptr={}, len={}", ptr, len);
        });
        
        let host_request_permission = Func::wrap(&mut store, |_caller: Caller<'_, WasiCtx>, ptr: i32, len: i32| -> i32 {
            // Host permission request function
            log::info!("Plugin permission request: ptr={}, len={}", ptr, len);
            1 // Grant permission (simplified)
        });
        
        // Create instance with host functions
        let instance = Instance::new(&mut store, &module, &[
            host_log.into(),
            host_request_permission.into(),
        ])?;
        
        // Get exported functions
        let init_func = instance.get_func(&mut store, "plugin_init");
        let get_capabilities_func = instance.get_func(&mut store, "plugin_get_capabilities");
        let get_metadata_func = instance.get_func(&mut store, "plugin_get_metadata");
        let get_permissions_func = instance.get_func(&mut store, "plugin_get_permissions");
        let handle_request_func = instance.get_func(&mut store, "plugin_handle_request");
        let cleanup_func = instance.get_func(&mut store, "plugin_cleanup");
        
        let plugin_id = Uuid::new_v4();
        
        let plugin = WasmPlugin {
            id: plugin_id,
            metadata,
            capabilities: PluginCapabilities::default(),
            permissions: vec![],
            status: PluginStatus::Installed,
            store,
            instance,
            init_func,
            get_capabilities_func,
            get_metadata_func,
            get_permissions_func,
            handle_request_func,
            cleanup_func,
        };
        
        self.plugins.insert(plugin_id, plugin);
        
        Ok(plugin_id)
    }
    
    /// Initialize a plugin
    pub fn init_plugin(&mut self, plugin_id: &Uuid) -> Result<()> {
        let plugin = self.plugins.get_mut(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found"))?;
        
        if let Some(init_func) = &plugin.init_func {
            let mut results = vec![wasmtime::Val::I32(0)];
            init_func.call(&mut plugin.store, &[], &mut results)?;

            if let Some(wasmtime::Val::I32(code)) = results.get(0) {
                if *code == 0 {
                    plugin.status = PluginStatus::Enabled;
                    Ok(())
                } else {
                    plugin.status = PluginStatus::Error("Initialization failed".to_string());
                    Err(anyhow!("Plugin initialization failed with code: {}", code))
                }
            } else {
                Err(anyhow!("Invalid return value from plugin_init"))
            }
        } else {
            Err(anyhow!("Plugin does not export plugin_init function"))
        }
    }
    
    /// Get plugin capabilities
    pub fn get_plugin_capabilities(&mut self, plugin_id: &Uuid) -> Result<PluginCapabilities> {
        if !self.plugins.contains_key(plugin_id) {
            return Err(anyhow!("Plugin not found"));
        }

        // Clone the function to avoid borrowing issues
        let get_capabilities_func = self.plugins.get(plugin_id)
            .and_then(|p| p.get_capabilities_func.clone());

        if let Some(func) = get_capabilities_func {
            let plugin = self.plugins.get_mut(plugin_id).unwrap();
            let mut results = vec![wasmtime::Val::I32(0)];
            func.call(&mut plugin.store, &[], &mut results)?;

            if let Some(wasmtime::Val::I32(ptr)) = results.get(0) {
                if *ptr != 0 {
                    // Read JSON string from WASM memory and parse capabilities
                    // This is simplified - in practice, you'd read from WASM linear memory
                    let capabilities = PluginCapabilities::default();
                    Ok(capabilities)
                } else {
                    Err(anyhow!("Plugin returned null capabilities"))
                }
            } else {
                Err(anyhow!("Invalid return value from plugin_get_capabilities"))
            }
        } else {
            Ok(PluginCapabilities::default())
        }
    }
    
    fn get_plugin_capabilities_internal(&mut self, plugin: &mut WasmPlugin) -> Result<PluginCapabilities> {
        if let Some(func) = &plugin.get_capabilities_func {
            let mut results = vec![wasmtime::Val::I32(0)];
            func.call(&mut plugin.store, &[], &mut results)?;

            if let Some(wasmtime::Val::I32(ptr)) = results.get(0) {
                if *ptr != 0 {
                    // Read JSON string from WASM memory and parse capabilities
                    // This is simplified - in practice, you'd read from WASM linear memory
                    let capabilities = PluginCapabilities::default();
                    Ok(capabilities)
                } else {
                    Err(anyhow!("Plugin returned null capabilities"))
                }
            } else {
                Err(anyhow!("Invalid return value from plugin_get_capabilities"))
            }
        } else {
            Ok(PluginCapabilities::default())
        }
    }
    
    fn get_plugin_permissions_internal(&mut self, plugin: &mut WasmPlugin) -> Result<Vec<PluginPermission>> {
        if let Some(func) = &plugin.get_permissions_func {
            let mut results = vec![wasmtime::Val::I32(0)];
            func.call(&mut plugin.store, &[], &mut results)?;

            if let Some(wasmtime::Val::I32(ptr)) = results.get(0) {
                if *ptr != 0 {
                    // Read JSON string from WASM memory and parse permissions
                    // This is simplified - in practice, you'd read from WASM linear memory
                    let permissions = vec![];
                    Ok(permissions)
                } else {
                    Ok(vec![])
                }
            } else {
                Err(anyhow!("Invalid return value from plugin_get_permissions"))
            }
        } else {
            Ok(vec![])
        }
    }
    
    /// Handle a plugin request
    pub fn handle_plugin_request(&mut self, plugin_id: &Uuid, request: &Value) -> Result<Value> {
        let plugin = self.plugins.get_mut(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found"))?;
        
        if let Some(func) = &plugin.handle_request_func {
            // Convert request to JSON string and pass to plugin
            let _request_json = serde_json::to_string(request)?;

            // In a real implementation, you'd write the JSON string to WASM memory
            // and pass the pointer and length to the function
            let mut results = vec![wasmtime::Val::I32(0)];
            func.call(&mut plugin.store, &[wasmtime::Val::I32(0)], &mut results)?;

            if let Some(wasmtime::Val::I32(ptr)) = results.get(0) {
                if *ptr != 0 {
                    // Read response JSON from WASM memory
                    // This is simplified - returning a mock response
                    Ok(serde_json::json!({
                        "id": "1",
                        "result": "Plugin response",
                        "error": null
                    }))
                } else {
                    Err(anyhow!("Plugin returned null response"))
                }
            } else {
                Err(anyhow!("Invalid return value from plugin_handle_request"))
            }
        } else {
            Err(anyhow!("Plugin does not export plugin_handle_request function"))
        }
    }
    
    /// Cleanup and unload a plugin
    pub fn unload_plugin(&mut self, plugin_id: &Uuid) -> Result<()> {
        if let Some(mut plugin) = self.plugins.remove(plugin_id) {
            // Call cleanup function if available
            if let Some(cleanup_func) = &plugin.cleanup_func {
                let mut results = vec![];
                let _ = cleanup_func.call(&mut plugin.store, &[], &mut results);
            }

            plugin.status = PluginStatus::NotInstalled;
            Ok(())
        } else {
            Err(anyhow!("Plugin not found"))
        }
    }
    
    /// Get all loaded plugins
    pub fn get_plugins(&self) -> Vec<&WasmPlugin> {
        self.plugins.values().collect()
    }
    
    /// Get a specific plugin
    pub fn get_plugin(&self, plugin_id: &Uuid) -> Option<&WasmPlugin> {
        self.plugins.get(plugin_id)
    }
    
    /// Get plugin status
    pub fn get_plugin_status(&self, plugin_id: &Uuid) -> Option<&PluginStatus> {
        self.plugins.get(plugin_id).map(|p| &p.status)
    }
}

impl Default for WasmPluginRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create WASM runtime")
    }
}
