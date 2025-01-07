use crate::core::AppResult;
use crate::plugins::traits::Plugin;
use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

pub struct PluginLoader {
    plugin_dir: PathBuf,
    loaded_libraries: Vec<Library>,
}

impl PluginLoader {
    pub fn new<P: AsRef<Path>>(plugin_dir: P) -> Self {
        Self {
            plugin_dir: plugin_dir.as_ref().to_path_buf(),
            loaded_libraries: Vec::new(),
        }
    }

    pub async fn load_plugin(&mut self, plugin_name: &str) -> AppResult<Arc<dyn Plugin>> {
        let plugin_path = self.plugin_dir.join(format!("lib{}.so", plugin_name));
        
        // 检查插件文件是否存在
        if !plugin_path.exists() {
            return Err(crate::core::AppError::NotFound(format!(
                "插件 {} 不存在",
                plugin_name
            )));
        }

        unsafe {
            // 加载动态库
            let lib = Library::new(plugin_path)?;
            
            // 获取插件创建函数
            let constructor: Symbol<fn() -> Box<dyn Plugin>> = lib.get(b"create_plugin")?;
            
            // 创建插件实例
            let plugin = constructor();
            
            // 保存库引用
            self.loaded_libraries.push(lib);
            
            Ok(Arc::new(*plugin))
        }
    }

    pub async fn unload_plugin(&mut self, plugin_name: &str) -> AppResult<()> {
        // 卸载插件时需要小心处理,确保没有正在使用的引用
        // TODO: 实现安全的插件卸载逻辑
        Ok(())
    }

    pub async fn reload_plugin(&mut self, plugin_name: &str) -> AppResult<Arc<dyn Plugin>> {
        // 先卸载旧版本
        self.unload_plugin(plugin_name).await?;
        
        // 加载新版本
        self.load_plugin(plugin_name).await
    }

    pub async fn scan_plugins(&self) -> AppResult<Vec<String>> {
        let mut plugins = Vec::new();
        let mut entries = fs::read_dir(&self.plugin_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.starts_with("lib") && filename.ends_with(".so") {
                    // 提取插件名称
                    let plugin_name = filename[3..filename.len()-3].to_string();
                    plugins.push(plugin_name);
                }
            }
        }
        
        Ok(plugins)
    }
} 