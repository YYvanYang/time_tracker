use std::collections::HashMap;
use std::sync::Arc;
use libloading::{Library, Symbol};
use crate::core::AppResult;
use crate::plugins::traits::Plugin;

pub struct PluginLoader {
    libraries: HashMap<String, Library>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
        }
    }

    pub fn load_plugin(&mut self, plugin_name: &str) -> AppResult<Arc<dyn Plugin>> {
        let lib_path = format!("plugins/{}.so", plugin_name);
        let lib = unsafe { Library::new(&lib_path)? };

        let constructor: Symbol<unsafe fn() -> *mut dyn Plugin> = unsafe {
            lib.get(b"_plugin_create")?
        };

        let plugin = unsafe {
            let raw = constructor();
            Arc::from_raw(raw as *mut dyn Plugin)
        };

        self.libraries.insert(plugin_name.to_string(), lib);
        Ok(plugin)
    }

    pub fn unload_plugin(&mut self, plugin_name: &str) -> AppResult<()> {
        if let Some(lib) = self.libraries.remove(plugin_name) {
            drop(lib);
        }
        Ok(())
    }

    pub fn list_plugins(&self) -> AppResult<Vec<String>> {
        // TODO: 实现插件扫描逻辑
        Ok(Vec::new())
    }
} 