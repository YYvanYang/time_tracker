// src/ui/theme.rs

use crate::config::{Config, Theme as AppTheme};
use eframe::egui::{self, Color32, Rounding, Stroke, Style, Visuals};
use crate::ui::TimeTrackerApp;

pub const SPACING: f32 = 8.0;
pub const PADDING: f32 = 6.0;
pub const ROUNDING: f32 = 4.0;

// 明亮主题颜色
pub const LIGHT_PRIMARY: Color32 = Color32::from_rgb(25, 118, 210);
pub const LIGHT_SECONDARY: Color32 = Color32::from_rgb(156, 39, 176);
pub const LIGHT_BACKGROUND: Color32 = Color32::from_rgb(255, 255, 255);
pub const LIGHT_SURFACE: Color32 = Color32::from_rgb(250, 250, 250);
pub const LIGHT_ERROR: Color32 = Color32::from_rgb(211, 47, 47);
pub const LIGHT_TEXT: Color32 = Color32::from_rgb(33, 33, 33);
pub const LIGHT_TEXT_SECONDARY: Color32 = Color32::from_rgb(117, 117, 117);

// 暗色主题颜色
pub const DARK_PRIMARY: Color32 = Color32::from_rgb(64, 169, 255);
pub const DARK_SECONDARY: Color32 = Color32::from_rgb(186, 104, 200);
pub const DARK_BACKGROUND: Color32 = Color32::from_rgb(18, 18, 18);
pub const DARK_SURFACE: Color32 = Color32::from_rgb(30, 30, 30);
pub const DARK_ERROR: Color32 = Color32::from_rgb(255, 82, 82);
pub const DARK_TEXT: Color32 = Color32::from_rgb(255, 255, 255);
pub const DARK_TEXT_SECONDARY: Color32 = Color32::from_rgb(189, 189, 189);

pub fn apply_theme(ctx: &egui::Context, app: &TimeTrackerApp) {
    let config = app.get_config();
    let visuals = match config.ui.theme {
        AppTheme::Light => light_theme(),
        AppTheme::Dark => dark_theme(),
        AppTheme::System => {
            if is_dark_mode_enabled() {
                dark_theme()
            } else {
                light_theme()
            }
        }
    };

    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(SPACING, SPACING);
    style.spacing.window_margin = egui::Margin::same(PADDING);
    style.visuals = visuals;

    ctx.set_style(style);
}

fn light_theme() -> Visuals {
    Visuals {
        dark_mode: false,
        override_text_color: Some(LIGHT_TEXT),
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                weak_bg_fill: LIGHT_SURFACE.linear_multiply(0.7),
                bg_fill: LIGHT_SURFACE,
                bg_stroke: Stroke::new(1.0, LIGHT_TEXT_SECONDARY),
                fg_stroke: Stroke::new(1.0, LIGHT_TEXT),
                rounding: Rounding::same(ROUNDING),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                weak_bg_fill: LIGHT_SURFACE.linear_multiply(0.7),
                bg_fill: LIGHT_SURFACE,
                bg_stroke: Stroke::new(1.0, LIGHT_PRIMARY),
                fg_stroke: Stroke::new(1.0, LIGHT_TEXT),
                rounding: Rounding::same(ROUNDING),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: LIGHT_PRIMARY.linear_multiply(0.9),
                bg_stroke: Stroke::new(1.0, LIGHT_PRIMARY),
                fg_stroke: Stroke::new(1.5, LIGHT_TEXT),
                rounding: Rounding::same(ROUNDING),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: LIGHT_PRIMARY,
                bg_stroke: Stroke::new(1.0, LIGHT_PRIMARY),
                fg_stroke: Stroke::new(2.0, LIGHT_BACKGROUND),
                rounding: Rounding::same(ROUNDING),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: LIGHT_PRIMARY.linear_multiply(0.8),
                bg_stroke: Stroke::new(1.0, LIGHT_PRIMARY),
                fg_stroke: Stroke::new(1.0, LIGHT_TEXT),
                rounding: Rounding::same(ROUNDING),
                expansion: 1.0,
            },
        },
        selection: egui::style::Selection {
            bg_fill: LIGHT_PRIMARY.linear_multiply(0.3),
            stroke: Stroke::new(1.0, LIGHT_PRIMARY),
        },
        ..Visuals::light()
    }
}

fn dark_theme() -> Visuals {
    Visuals {
        dark_mode: true,
        override_text_color: Some(DARK_TEXT),
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: DARK_SURFACE,
                bg_stroke: Stroke::new(1.0, DARK_TEXT_SECONDARY),
                fg_stroke: Stroke::new(1.0, DARK_TEXT),
                rounding: Rounding::same(ROUNDING),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: DARK_SURFACE,
                bg_stroke: Stroke::new(1.0, DARK_PRIMARY),
                fg_stroke: Stroke::new(1.0, DARK_TEXT),
                rounding: Rounding::same(ROUNDING),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: DARK_PRIMARY.linear_multiply(0.9),
                bg_stroke: Stroke::new(1.0, DARK_PRIMARY),
                fg_stroke: Stroke::new(1.5, DARK_TEXT),
                rounding: Rounding::same(ROUNDING),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: DARK_PRIMARY,
                bg_stroke: Stroke::new(1.0, DARK_PRIMARY),
                fg_stroke: Stroke::new(2.0, DARK_BACKGROUND),
                rounding: Rounding::same(ROUNDING),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: DARK_PRIMARY.linear_multiply(0.8),
                bg_stroke: Stroke::new(1.0, DARK_PRIMARY),
                fg_stroke: Stroke::new(1.0, DARK_TEXT),
                rounding: Rounding::same(ROUNDING),
                expansion: 1.0,
            },
        },
        selection: egui::style::Selection {
            bg_fill: DARK_PRIMARY.linear_multiply(0.3),
            stroke: Stroke::new(1.0, DARK_PRIMARY),
        },
        ..Visuals::dark()
    }
}

#[cfg(target_os = "windows")]
fn is_dark_mode_enabled() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"
    ) {
        if let Ok(dark_mode) = hkcu.get_value::<u32, _>("AppsUseLightTheme") {
            return dark_mode == 0;
        }
    }
    false
}

#[cfg(target_os = "macos")]
fn is_dark_mode_enabled() -> bool {
    use core_foundation::bundle::{CFBundle, CFBundleGetMainBundle};
    use core_foundation::string::CFString;
    use core_foundation::base::{TCFType, CFType};
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::number::CFNumber;

    unsafe {
        if let Some(bundle) = CFBundle::main_bundle() {
            if let Ok(dict) = bundle.info_dictionary() {
                let key = CFString::from_static_string("AppleInterfaceStyle");
                if let Some(value) = dict.find(&key as &CFType) {
                    if let Some(string) = value.downcast::<CFString>() {
                        return string.to_string() == "Dark";
                    }
                }
            }
        }
    }
    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn is_dark_mode_enabled() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_theme_colors() {
        // 测试亮色主题颜色对比度
        let contrast_ratio = color_contrast(LIGHT_TEXT, LIGHT_BACKGROUND);
        assert!(contrast_ratio >= 4.5, "Light theme text contrast too low");

        // 测试暗色主题颜色对比度
        let contrast_ratio = color_contrast(DARK_TEXT, DARK_BACKGROUND);
        assert!(contrast_ratio >= 4.5, "Dark theme text contrast too low");
    }

    #[test]
    fn test_theme_switching() {
        let ctx = egui::Context::default();

        // 测试亮色主题
        let mut config = Config::default();
        config.ui.theme = AppTheme::Light;
        apply_theme(&ctx, &config);
        assert!(!ctx.style().visuals.dark_mode);

        // 测试暗色主题
        config.ui.theme = AppTheme::Dark;
        apply_theme(&ctx, &config);
        assert!(ctx.style().visuals.dark_mode);
    }

    // 辅助函数：计算颜色对比度
    fn color_contrast(color1: Color32, color2: Color32) -> f32 {
        let l1 = relative_luminance(color1);
        let l2 = relative_luminance(color2);
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }

    fn relative_luminance(color: Color32) -> f32 {
        let r = srgb_to_linear(color.r() as f32 / 255.0) * 0.2126;
        let g = srgb_to_linear(color.g() as f32 / 255.0) * 0.7152;
        let b = srgb_to_linear(color.b() as f32 / 255.0) * 0.0722;
        r + g + b
    }

    fn srgb_to_linear(c: f32) -> f32 {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
}