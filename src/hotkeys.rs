//src/hotkeys.rs

use crate::error::Result;
use eframe::egui::Key;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub enabled: bool,
    pub bindings: HashMap<String, Hotkey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    pub key: Key,
    pub modifiers: HotkeyModifiers,
    pub action: HotkeyAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HotkeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub command: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HotkeyAction {
    // 视图切换
    ShowOverview,
    ShowAppUsage,
    ShowPomodoro,
    ShowProjects,
    ShowStatistics,
    ShowSettings,

    // 番茄钟控制
    StartPomodoro,
    PausePomodoro,
    StopPomodoro,
    SkipBreak,

    // 项目相关
    NewProject,
    EditProject,
    DeleteProject,

    // 通用操作
    Save,
    Export,
    ShowHelp,
    Quit,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        // 视图切换快捷键
        bindings.insert(
            "overview".to_string(),
            Hotkey {
                key: Key::Num1,
                modifiers: HotkeyModifiers::ctrl(),
                action: HotkeyAction::ShowOverview,
            },
        );

        bindings.insert(
            "app_usage".to_string(),
            Hotkey {
                key: Key::Num2,
                modifiers: HotkeyModifiers::ctrl(),
                action: HotkeyAction::ShowAppUsage,
            },
        );

        bindings.insert(
            "pomodoro".to_string(),
            Hotkey {
                key: Key::Num3,
                modifiers: HotkeyModifiers::ctrl(),
                action: HotkeyAction::ShowPomodoro,
            },
        );

        // 番茄钟控制
        bindings.insert(
            "start_pomodoro".to_string(),
            Hotkey {
                key: Key::Space,
                modifiers: HotkeyModifiers::ctrl(),
                action: HotkeyAction::StartPomodoro,
            },
        );

        bindings.insert(
            "pause_pomodoro".to_string(),
            Hotkey {
                key: Key::P,
                modifiers: HotkeyModifiers::ctrl(),
                action: HotkeyAction::PausePomodoro,
            },
        );

        // 通用操作
        bindings.insert(
            "save".to_string(),
            Hotkey {
                key: Key::S,
                modifiers: HotkeyModifiers::ctrl(),
                action: HotkeyAction::Save,
            },
        );

        bindings.insert(
            "export".to_string(),
            Hotkey {
                key: Key::E,
                modifiers: HotkeyModifiers::ctrl(),
                action: HotkeyAction::Export,
            },
        );

        bindings.insert(
            "help".to_string(),
            Hotkey {
                key: Key::H,
                modifiers: HotkeyModifiers::ctrl(),
                action: HotkeyAction::ShowHelp,
            },
        );

        bindings.insert(
            "quit".to_string(),
            Hotkey {
                key: Key::Q,
                modifiers: HotkeyModifiers::ctrl(),
                action: HotkeyAction::Quit,
            },
        );

        Self {
            enabled: true,
            bindings,
        }
    }
}

impl HotkeyModifiers {
    pub fn none() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            command: false,
        }
    }

    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            shift: false,
            alt: false,
            command: false,
        }
    }

    pub fn shift() -> Self {
        Self {
            ctrl: false,
            shift: true,
            alt: false,
            command: false,
        }
    }

    pub fn alt() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: true,
            command: false,
        }
    }

    pub fn command() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            command: true,
        }
    }

    pub fn matches(&self, ctx: &eframe::egui::Context) -> bool {
        let modifiers = ctx.input().modifiers;
        modifiers.ctrl == self.ctrl
            && modifiers.shift == self.shift
            && modifiers.alt == self.alt
            && modifiers.command == self.command
    }
}

impl Hotkey {
    pub fn matches(&self, ctx: &eframe::egui::Context) -> bool {
        self.modifiers.matches(ctx) && ctx.input().key_pressed(self.key)
    }

    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.command {
            parts.push("Cmd");
        }

        parts.push(&format!("{:?}", self.key));
        parts.join("+")
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return None;
        }

        let mut modifiers = HotkeyModifiers::none();
        let mut key = None;

        for part in &parts[..parts.len()-1] {
            match *part {
                "Ctrl" => modifiers.ctrl = true,
                "Shift" => modifiers.shift = true,
                "Alt" => modifiers.alt = true,
                "Cmd" => modifiers.command = true,
                _ => return None,
            }
        }

        if let Ok(k) = parts.last()?.parse() {
            key = Some(k);
        }

        key.map(|k| Self {
            key: k,
            modifiers,
            action: HotkeyAction::Save, // 默认action，需要外部设置
        })
    }
}

pub struct HotkeyManager {
    config: HotkeyConfig,
    handler: Box<dyn Fn(HotkeyAction) -> Result<()> + Send + Sync>,
}

impl HotkeyManager {
    pub fn new<F>(config: HotkeyConfig, handler: F) -> Self 
    where
        F: Fn(HotkeyAction) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            config,
            handler: Box::new(handler),
        }
    }

    pub fn update(&self, ctx: &eframe::egui::Context) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        for hotkey in self.config.bindings.values() {
            if hotkey.matches(ctx) {
                (self.handler)(hotkey.action.clone())?;
            }
        }

        Ok(())
    }

    pub fn get_hotkey(&self, action: &HotkeyAction) -> Option<&Hotkey> {
        self.config.bindings.values()
            .find(|h| h.action == *action)
    }

    pub fn set_hotkey(&mut self, name: String, hotkey: Hotkey) {
        self.config.bindings.insert(name, hotkey);
    }

    pub fn remove_hotkey(&mut self, name: &str) {
        self.config.bindings.remove(name);
    }

    pub fn get_bindings(&self) -> &HashMap<String, Hotkey> {
        &self.config.bindings
    }
}

// 快捷键帮助对话框
pub struct HotkeyHelpDialog;

impl HotkeyHelpDialog {
    pub fn show(ui: &mut eframe::egui::Ui, hotkeys: &HashMap<String, Hotkey>) {
        ui.heading("快捷键帮助");

        egui::Grid::new("hotkey_help_grid")
            .striped(true)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.strong("操作");
                ui.strong("快捷键");
                ui.end_row();

                for (name, hotkey) in hotkeys {
                    ui.label(name);
                    ui.label(hotkey.to_string());
                    ui.end_row();
                }
            });
    }
}

// 快捷键设置对话框
pub struct HotkeySettingsDialog {
    editing_hotkey: Option<(String, Hotkey)>,
    waiting_for_input: bool,
}

impl HotkeySettingsDialog {
    pub fn new() -> Self {
        Self {
            editing_hotkey: None,
            waiting_for_input: false,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut eframe::egui::Ui,
        manager: &mut HotkeyManager,
    ) -> Option<(String, Hotkey)> {
        let mut result = None;

        ui.heading("快捷键设置");
        ui.checkbox(&mut manager.config.enabled, "启用快捷键");

        if manager.config.enabled {
            egui::Grid::new("hotkey_settings_grid")
                .striped(true)
                .spacing([40.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("操作");
                    ui.strong("快捷键");
                    ui.strong("操作");
                    ui.end_row();

                    for (name, hotkey) in manager.get_bindings() {
                        ui.label(name);
                        
                        if Some((name.clone(), hotkey.clone())) == self.editing_hotkey {
                            if self.waiting_for_input {
                                ui.label("按下新的快捷键组合...");
                                if let Some(new_hotkey) = Self::capture_hotkey(ui.ctx()) {
                                    let mut new_hotkey = new_hotkey;
                                    new_hotkey.action = hotkey.action.clone();
                                    result = Some((name.clone(), new_hotkey));
                                    self.editing_hotkey = None;
                                    self.waiting_for_input = false;
                                }
                            } else {
                                if ui.button("开始设置").clicked() {
                                    self.waiting_for_input = true;
                                }
                                if ui.button("取消").clicked() {
                                    self.editing_hotkey = None;
                                }
                            }
                        } else {
                            ui.label(hotkey.to_string());
                            if ui.button("修改").clicked() {
                                self.editing_hotkey = Some((name.clone(), hotkey.clone()));
                                self.waiting_for_input = false;
                            }
                        }
                        ui.end_row();
                    }
                });
        }

        result
    }

    fn capture_hotkey(ctx: &eframe::egui::Context) -> Option<Hotkey> {
        let mut pressed_keys = Vec::new();
        for key in Key::ALL {
            if ctx.input().key_pressed(key) {
                pressed_keys.push(key);
            }
        }

        if pressed_keys.is_empty() {
            return None;
        }

        let modifiers = ctx.input().modifiers;
        let key = pressed_keys[0];

        Some(Hotkey {
            key,
            modifiers: HotkeyModifiers {
                ctrl: modifiers.ctrl,
                shift: modifiers.shift,
                alt: modifiers.alt,
                command: modifiers.command,
            },
            action: HotkeyAction::Save, // 临时action
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;

    #[test]
    fn test_hotkey_string_conversion() {
        let hotkey = Hotkey {
            key: Key::S,
            modifiers: HotkeyModifiers::ctrl(),
            action: HotkeyAction::Save,
        };

        let hotkey_str = hotkey.to_string();
        assert_eq!(hotkey_str, "Ctrl+S");

        let parsed = Hotkey::from_string(&hotkey_str);
        assert!(parsed.is_some());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.key, hotkey.key);
        assert_eq!(parsed.modifiers.ctrl, hotkey.modifiers.ctrl);
    }

    #[test]
    fn test_hotkey_matching() {
        let ctx = Context::default();
        
        let hotkey = Hotkey {
            key: Key::S,
            modifiers: HotkeyModifiers::ctrl(),
            action: HotkeyAction::Save,
        };

        // 模拟按下 Ctrl+S
        ctx.run(|ctx| {
            ctx.input_mut().key_press(Key::S);
            ctx.input_mut().modifiers.ctrl = true;
            assert!(hotkey.matches(ctx));
        });

        // 模拟只按下 S
        ctx.run(|ctx| {
            ctx.input_mut().key_press(Key::S);
            ctx.input_mut().modifiers.ctrl = false;
            assert!(!hotkey.matches(ctx));
        });
    }

    #[test]
    fn test_hotkey_manager() {
        let ctx = Context::default();
        let config = HotkeyConfig::default();
        let mut called_action = None;

        let manager = HotkeyManager::new(config, move |action| {
            called_action = Some(action);
            Ok(())
        });

        ctx.run(|ctx| {
            // 模拟按下 Ctrl+S
            ctx.input_mut().key_press(Key::S);
            ctx.input_mut().modifiers.ctrl = true;

            manager.update(ctx).unwrap();
        });

        assert_eq!(called_action, Some(HotkeyAction::Save));
    }
}