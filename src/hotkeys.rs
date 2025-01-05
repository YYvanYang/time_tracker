//src/hotkeys.rs

use crate::error::Result;
use eframe::egui::Key;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::de::Error as DeError;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Logo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub enabled: bool,
    pub bindings: HashMap<String, Hotkey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    #[serde(serialize_with = "serialize_key", deserialize_with = "deserialize_key")]
    pub key: Key,
    pub modifiers: Vec<Modifier>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bindings: Self::default_bindings(),
        }
    }
}

impl HotkeyConfig {
    pub fn default_bindings() -> HashMap<String, Hotkey> {
        let bindings = HashMap::new();
        // ... 添加默认快捷键绑定
        bindings
    }
}

pub struct HotkeyManager {
    config: HotkeyConfig,
    // ... 其他字段
}

impl HotkeyManager {
    pub fn new(config: HotkeyConfig) -> Self {
        Self {
            config,
            // ... 初始化其他字段
        }
    }
    // ... 其他方法
}

fn serialize_key<S>(key: &Key, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u32(*key as u32)
}

fn deserialize_key<'de, D>(deserializer: D) -> std::result::Result<Key, D::Error>
where
    D: Deserializer<'de>,
{
    let value = u32::deserialize(deserializer)?;
    match value {
        0..=255 => {
            let key_u8 = value as u8;
            Ok(unsafe { std::mem::transmute::<u8, Key>(key_u8) })
        }
        _ => Err(D::Error::custom("Invalid key value")),
    }
}