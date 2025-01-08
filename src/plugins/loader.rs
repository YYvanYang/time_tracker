use crate::core::{AppResult, AppError};
use crate::plugins::Plugin;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub struct PluginLoader {
    plugin_dir: PathBuf,
    loaded_plugins: Vec<(Library, Arc<dyn Plugin>)>,
}

impl PluginLoader {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugin_dir,
            loaded_plugins: Vec::new(),
        }
    }

    pub fn load_plugin(&mut self, plugin_name: &str) -> AppResult<Arc<dyn Plugin>> {
        let plugin_path = self.plugin_dir.join(format!("lib{}.so", plugin_name));
        
        unsafe {
            let lib = Library::new(plugin_path)?;
            
            let constructor: Symbol<unsafe fn() -> *mut dyn Plugin> = lib.get(b"_plugin_create")?;
            let plugin = Arc::new(Box::from_raw(constructor()));
            
            self.loaded_plugins.push((lib, plugin.clone()));
            Ok(plugin)
        }
    }

    pub fn unload_plugin(&mut self, plugin_name: &str) -> AppResult<()> {
        if let Some(index) = self.loaded_plugins.iter().position(|(_, p)| p.name() == plugin_name) {
            self.loaded_plugins.remove(index);
        }
        Ok(())
    }

    pub fn get_loaded_plugins(&self) -> Vec<String> {
        self.loaded_plugins.iter()
            .map(|(_, p)| p.name().to_string())
            .collect()
    }
} 