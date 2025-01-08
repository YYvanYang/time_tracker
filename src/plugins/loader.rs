use crate::core::AppError;
use crate::core::AppResult;
use crate::plugins::traits::Plugin;
use libloading::{Library, Symbol};
use std::path::PathBuf;
use std::sync::Arc;

pub struct PluginLoader {
    plugin_dir: PathBuf,
    loaded_plugins: Vec<(Library, Arc<dyn Plugin>)>,
}

impl PluginLoader {
    pub fn new() -> Self {
        let plugin_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("./data"))
            .join("time_tracker")
            .join("plugins");
        
        std::fs::create_dir_all(&plugin_dir).unwrap_or_default();
        
        Self {
            plugin_dir,
            loaded_plugins: Vec::new(),
        }
    }

    pub fn load_plugin(&mut self, plugin_name: &str) -> AppResult<Arc<dyn Plugin>> {
        let plugin_path = self.plugin_dir.join(format!(
            "{}{}",
            plugin_name,
            std::env::consts::DLL_EXTENSION
        ));

        if !plugin_path.exists() {
            return Err(AppError::InvalidOperation(format!(
                "Plugin {} does not exist",
                plugin_name
            )));
        }

        unsafe {
            let lib = Library::new(&plugin_path).map_err(|e| AppError::Plugin(e))?;

            let create_plugin: Symbol<unsafe fn() -> *mut dyn Plugin> = 
                lib.get(b"create_plugin").map_err(|e| AppError::Plugin(e))?;

            let plugin = create_plugin();
            let plugin = Arc::new(Box::from_raw(plugin));
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

    pub fn get_loaded_plugins(&self) -> Vec<Arc<dyn Plugin>> {
        self.loaded_plugins.iter().map(|(_, p)| p.clone()).collect()
    }
} 