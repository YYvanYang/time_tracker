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

pub fn apply_theme(ctx: &egui::Context, is_dark: bool) {
    let mut style = (*ctx.style()).clone();
    style.visuals = if is_dark { dark_theme() } else { light_theme() };
    ctx.set_style(style);
}

fn create_widget_visuals(
    bg_fill: Color32,
    weak_bg_fill: Color32,
    bg_stroke: Stroke,
    fg_stroke: Stroke,
    rounding: f32,
    expansion: f32,
) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill,
        weak_bg_fill,
        bg_stroke,
        fg_stroke,
        rounding: rounding.into(),
        expansion,
    }
}

fn dark_theme() -> Visuals {
    let mut visuals = Visuals::dark();
    
    visuals.widgets.active = create_widget_visuals(
        Color32::from_rgb(70, 70, 70),      // bg_fill
        Color32::from_rgb(60, 60, 60),      // weak_bg_fill
        Stroke::new(1.0, Color32::from_rgb(140, 140, 140)), // bg_stroke
        Stroke::new(1.0, Color32::from_rgb(255, 255, 255)), // fg_stroke
        2.0,  // rounding
        1.0,  // expansion
    );

    visuals.widgets.open = create_widget_visuals(
        Color32::from_rgb(50, 50, 50),      // bg_fill
        Color32::from_rgb(40, 40, 40),      // weak_bg_fill
        Stroke::new(1.0, Color32::from_rgb(120, 120, 120)), // bg_stroke
        Stroke::new(1.0, Color32::from_rgb(200, 200, 200)), // fg_stroke
        2.0,  // rounding
        0.0,  // expansion
    );

    visuals
}

fn light_theme() -> Visuals {
    let mut visuals = Visuals::light();
    
    visuals.widgets.active = create_widget_visuals(
        Color32::from_rgb(220, 220, 220),   // bg_fill
        Color32::from_rgb(210, 210, 210),   // weak_bg_fill
        Stroke::new(1.0, Color32::from_rgb(160, 160, 160)), // bg_stroke
        Stroke::new(1.0, Color32::from_rgb(0, 0, 0)),       // fg_stroke
        2.0,  // rounding
        1.0,  // expansion
    );

    visuals.widgets.open = create_widget_visuals(
        Color32::from_rgb(230, 230, 230),   // bg_fill
        Color32::from_rgb(220, 220, 220),   // weak_bg_fill
        Stroke::new(1.0, Color32::from_rgb(180, 180, 180)), // bg_stroke
        Stroke::new(1.0, Color32::from_rgb(60, 60, 60)),    // fg_stroke
        2.0,  // rounding
        0.0,  // expansion
    );

    visuals
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