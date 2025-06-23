/// Macro to generate WASM exports for a plugin
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        use std::sync::Mutex;
        use std::ffi::{CStr, CString};
        use std::os::raw::{c_char, c_int};
        
        static mut PLUGIN_INSTANCE: Option<Mutex<$plugin_type>> = None;
        static mut LAST_RESPONSE: Option<CString> = None;
        
        /// Initialize the plugin
        #[no_mangle]
        pub extern "C" fn plugin_init() -> c_int {
            let mut plugin = <$plugin_type>::default();
            match plugin.init() {
                Ok(_) => {
                    unsafe {
                        PLUGIN_INSTANCE = Some(Mutex::new(plugin));
                    }
                    0 // Success
                }
                Err(_) => -1 // Error
            }
        }
        
        /// Get plugin capabilities as JSON string
        #[no_mangle]
        pub extern "C" fn plugin_get_capabilities() -> *const c_char {
            unsafe {
                if let Some(ref plugin_mutex) = PLUGIN_INSTANCE {
                    if let Ok(plugin) = plugin_mutex.lock() {
                        let capabilities = plugin.get_capabilities();
                        if let Ok(json) = serde_json::to_string(&capabilities) {
                            if let Ok(c_string) = CString::new(json) {
                                let ptr = c_string.as_ptr();
                                LAST_RESPONSE = Some(c_string);
                                return ptr;
                            }
                        }
                    }
                }
                std::ptr::null()
            }
        }
        
        /// Get plugin metadata as JSON string
        #[no_mangle]
        pub extern "C" fn plugin_get_metadata() -> *const c_char {
            unsafe {
                if let Some(ref plugin_mutex) = PLUGIN_INSTANCE {
                    if let Ok(plugin) = plugin_mutex.lock() {
                        let metadata = plugin.get_metadata();
                        if let Ok(json) = serde_json::to_string(&metadata) {
                            if let Ok(c_string) = CString::new(json) {
                                let ptr = c_string.as_ptr();
                                LAST_RESPONSE = Some(c_string);
                                return ptr;
                            }
                        }
                    }
                }
                std::ptr::null()
            }
        }
        
        /// Get plugin permissions as JSON string
        #[no_mangle]
        pub extern "C" fn plugin_get_permissions() -> *const c_char {
            unsafe {
                if let Some(ref plugin_mutex) = PLUGIN_INSTANCE {
                    if let Ok(plugin) = plugin_mutex.lock() {
                        let permissions = plugin.get_permissions();
                        if let Ok(json) = serde_json::to_string(&permissions) {
                            if let Ok(c_string) = CString::new(json) {
                                let ptr = c_string.as_ptr();
                                LAST_RESPONSE = Some(c_string);
                                return ptr;
                            }
                        }
                    }
                }
                std::ptr::null()
            }
        }
        
        /// Handle a plugin request
        #[no_mangle]
        pub extern "C" fn plugin_handle_request(request_ptr: *const c_char) -> *const c_char {
            unsafe {
                if request_ptr.is_null() {
                    return std::ptr::null();
                }
                
                let c_str = CStr::from_ptr(request_ptr);
                if let Ok(request_str) = c_str.to_str() {
                    if let Ok(request) = serde_json::from_str::<$crate::PluginRequest>(request_str) {
                        if let Some(ref plugin_mutex) = PLUGIN_INSTANCE {
                            if let Ok(mut plugin) = plugin_mutex.lock() {
                                let response = plugin.handle_request(request);
                                if let Ok(json) = serde_json::to_string(&response) {
                                    if let Ok(c_string) = CString::new(json) {
                                        let ptr = c_string.as_ptr();
                                        LAST_RESPONSE = Some(c_string);
                                        return ptr;
                                    }
                                }
                            }
                        }
                    }
                }
                std::ptr::null()
            }
        }
        
        /// Cleanup plugin resources
        #[no_mangle]
        pub extern "C" fn plugin_cleanup() {
            unsafe {
                if let Some(plugin_mutex) = PLUGIN_INSTANCE.take() {
                    if let Ok(mut plugin) = plugin_mutex.lock() {
                        plugin.cleanup();
                    }
                }
                LAST_RESPONSE = None;
            }
        }
    };
}

/// Macro to create a simple tool plugin
#[macro_export]
macro_rules! simple_tool_plugin {
    (
        name: $name:expr,
        display_name: $display_name:expr,
        version: $version:expr,
        description: $description:expr,
        author: $author:expr,
        tools: [$($tool:expr),*],
        handler: $handler:expr
    ) => {
        use $crate::*;
        
        #[derive(Default)]
        pub struct SimpleToolPlugin;
        
        impl Plugin for SimpleToolPlugin {
            fn init(&mut self) -> Result<(), PluginError> {
                Ok(())
            }
            
            fn get_capabilities(&self) -> PluginCapabilities {
                PluginCapabilities {
                    provides_tools: true,
                    ..Default::default()
                }
            }
            
            fn get_metadata(&self) -> PluginMetadata {
                PluginMetadata {
                    name: $name.to_string(),
                    display_name: $display_name.to_string(),
                    version: $version.to_string(),
                    description: $description.to_string(),
                    author: $author.to_string(),
                    license: "MIT".to_string(),
                    homepage: None,
                    repository: None,
                    keywords: vec![],
                    categories: vec!["tools".to_string()],
                }
            }
            
            fn get_permissions(&self) -> Vec<PluginPermission> {
                vec![]
            }
            
            fn handle_request(&mut self, request: PluginRequest) -> PluginResponse {
                match request.method.as_str() {
                    "tools/list" => {
                        let tools = vec![$($tool),*];
                        utils::success_response(request.id, json!({"tools": tools}))
                    }
                    "tools/call" => {
                        if let Ok(params) = utils::parse_json::<serde_json::Value>(&request.params) {
                            if let Some(tool_name) = params.get("name").and_then(|n| n.as_str()) {
                                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                                match $handler(tool_name, arguments) {
                                    Ok(result) => utils::success_response(request.id, result),
                                    Err(e) => utils::error_response(request.id, e.code, e.message),
                                }
                            } else {
                                utils::error_response(request.id, -32602, "Missing tool name".to_string())
                            }
                        } else {
                            utils::error_response(request.id, -32602, "Invalid parameters".to_string())
                        }
                    }
                    _ => utils::error_response(request.id, -32601, "Method not found".to_string()),
                }
            }
            
            fn cleanup(&mut self) {}
        }
        
        export_plugin!(SimpleToolPlugin);
    };
}

/// Macro to create a simple resource plugin
#[macro_export]
macro_rules! simple_resource_plugin {
    (
        name: $name:expr,
        display_name: $display_name:expr,
        version: $version:expr,
        description: $description:expr,
        author: $author:expr,
        resources: [$($resource:expr),*],
        handler: $handler:expr
    ) => {
        use $crate::*;
        
        #[derive(Default)]
        pub struct SimpleResourcePlugin;
        
        impl Plugin for SimpleResourcePlugin {
            fn init(&mut self) -> Result<(), PluginError> {
                Ok(())
            }
            
            fn get_capabilities(&self) -> PluginCapabilities {
                PluginCapabilities {
                    provides_resources: true,
                    ..Default::default()
                }
            }
            
            fn get_metadata(&self) -> PluginMetadata {
                PluginMetadata {
                    name: $name.to_string(),
                    display_name: $display_name.to_string(),
                    version: $version.to_string(),
                    description: $description.to_string(),
                    author: $author.to_string(),
                    license: "MIT".to_string(),
                    homepage: None,
                    repository: None,
                    keywords: vec![],
                    categories: vec!["resources".to_string()],
                }
            }
            
            fn get_permissions(&self) -> Vec<PluginPermission> {
                vec![]
            }
            
            fn handle_request(&mut self, request: PluginRequest) -> PluginResponse {
                match request.method.as_str() {
                    "resources/list" => {
                        let resources = vec![$($resource),*];
                        utils::success_response(request.id, json!({"resources": resources}))
                    }
                    "resources/read" => {
                        if let Ok(params) = utils::parse_json::<serde_json::Value>(&request.params) {
                            if let Some(uri) = params.get("uri").and_then(|u| u.as_str()) {
                                match $handler(uri) {
                                    Ok(content) => utils::success_response(request.id, json!({"contents": [{"uri": uri, "text": content}]})),
                                    Err(e) => utils::error_response(request.id, e.code, e.message),
                                }
                            } else {
                                utils::error_response(request.id, -32602, "Missing URI".to_string())
                            }
                        } else {
                            utils::error_response(request.id, -32602, "Invalid parameters".to_string())
                        }
                    }
                    _ => utils::error_response(request.id, -32601, "Method not found".to_string()),
                }
            }
            
            fn cleanup(&mut self) {}
        }
        
        export_plugin!(SimpleResourcePlugin);
    };
}
