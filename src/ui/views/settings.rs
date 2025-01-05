//src/ui/views/settings.rs

use crate::error::Result;
use crate::ui::{styles, components::*};
use crate::ui::TimeTrackerApp;
use crate::config::{Config, Theme};
use eframe::egui;
use std::path::PathBuf;

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(styles::SPACING_LARGE, styles::SPACING_LARGE);

    ui.heading("设置");

    egui::ScrollArea::vertical().show(ui, |ui| {
        // 常规设置
        render_general_settings(app, ui);
        ui.add_space(styles::SPACING_MEDIUM);
        ui.separator();

        // 番茄钟设置
        render_pomodoro_settings(app, ui);
        ui.add_space(styles::SPACING_MEDIUM);
        ui.separator();

        // 通知设置
        render_notification_settings(app, ui);
        ui.add_space(styles::SPACING_MEDIUM);
        ui.separator();

        // 数据设置
        render_data_settings(app, ui);
        ui.add_space(styles::SPACING_MEDIUM);
        ui.separator();

        // 关于
        render_about_section(ui);
    });
}

fn render_general_settings(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("常规");
    
    // 自启动设置
    let mut autostart = app.config.general.autostart;
    if ui.checkbox(&mut autostart, "开机自动启动").changed() {
        if let Err(err) = app.set_autostart(autostart) {
            app.show_error(format!("设置自启动失败: {}", err));
        }
    }

    // 最小化到托盘
    let mut minimize_to_tray = app.config.general.minimize_to_tray;
    if ui.checkbox(&mut minimize_to_tray, "最小化到托盘").changed() {
        app.config.general.minimize_to_tray = minimize_to_tray;
        app.save_config();
    }

    // 主题设置
    ui.horizontal(|ui| {
        ui.label("主题");
        egui::ComboBox::from_id_source("theme")
            .selected_text(match app.config.ui.theme {
                Theme::Light => "浅色",
                Theme::Dark => "深色",
                Theme::System => "跟随系统",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.config.ui.theme, Theme::Light, "浅色");
                ui.selectable_value(&mut app.config.ui.theme, Theme::Dark, "深色");
                ui.selectable_value(&mut app.config.ui.theme, Theme::System, "跟随系统");
            });
    });

    // 语言设置
    ui.horizontal(|ui| {
        ui.label("语言");
        egui::ComboBox::from_id_source("language")
            .selected_text(get_language_name(&app.config.general.language))
            .show_ui(ui, |ui| {
                if ui.selectable_value(
                    &mut app.config.general.language,
                    "zh-CN".to_string(),
                    "简体中文"
                ).changed() {
                    app.save_config();
                }
                if ui.selectable_value(
                    &mut app.config.general.language,
                    "en".to_string(),
                    "English"
                ).changed() {
                    app.save_config();
                }
            });
    });

    // 界面设置
    ui.horizontal(|ui| {
        ui.label("字体大小");
        if ui.add(
            egui::DragValue::new(&mut app.config.ui.font_size)
                .clamp_range(12..=24)
                .speed(1)
        ).changed() {
            app.save_config();
        }
    });

    ui.checkbox(&mut app.config.ui.compact_mode, "紧凑模式");
}

fn render_pomodoro_settings(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("番茄钟");

    // 时长设置
    ui.horizontal(|ui| {
        ui.label("工作时长（分钟）");
        if ui.add(
            egui::DragValue::new(
                &mut app.config.pomodoro.work_duration_minutes())
                .clamp_range(1..=120)
        ).changed() {
            app.save_config();
        }
    });

    ui.horizontal(|ui| {
        ui.label("短休息时长（分钟）");
        if ui.add(
            egui::DragValue::new(
                &mut app.config.pomodoro.short_break_duration_minutes())
                .clamp_range(1..=30)
        ).changed() {
            app.save_config();
        }
    });

    ui.horizontal(|ui| {
        ui.label("长休息时长（分钟）");
        if ui.add(
            egui::DragValue::new(
                &mut app.config.pomodoro.long_break_duration_minutes())
                .clamp_range(5..=60)
        ).changed() {
            app.save_config();
        }
    });

    ui.horizontal(|ui| {
        ui.label("长休息间隔（番茄数）");
        if ui.add(
            egui::DragValue::new(&mut app.config.pomodoro.long_break_interval)
                .clamp_range(1..=10)
        ).changed() {
            app.save_config();
        }
    });

    // 自动化设置
    ui.checkbox(&mut app.config.pomodoro.auto_start_breaks, "自动开始休息");
    ui.checkbox(&mut app.config.pomodoro.auto_start_pomodoros, "休息后自动开始下一个番茄钟");
}

fn render_notification_settings(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("通知");

    // 通知开关
    ui.checkbox(&mut app.config.notification.enabled, "启用系统通知");
    
    if app.config.notification.enabled {
        ui.indent("notification_settings", |ui| {
            ui.checkbox(&mut app.config.notification.break_reminder, "休息提醒");
            ui.checkbox(&mut app.config.notification.pomodoro_reminder, "番茄钟提醒");
            ui.checkbox(&mut app.config.notification.productivity_reminder, "生产力提醒");
        });
    }

    // 声音设置
    ui.checkbox(&mut app.config.pomodoro.sound_enabled, "启用声音");

    if app.config.pomodoro.sound_enabled {
        ui.horizontal(|ui| {
            ui.label("音量");
            if ui.add(
                egui::Slider::new(&mut app.config.pomodoro.sound_volume, 0..=100)
            ).changed() {
                app.save_config();
            }
        });

        // 声音测试
        if Button::new("测试声音")
            .with_style(styles::ButtonStyle::outlined())
            .show(ui)
            .clicked()
        {
            app.play_test_sound();
        }
    }
}

fn render_data_settings(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("数据");

    // 数据目录
    ui.horizontal(|ui| {
        ui.label("数据存储位置");
        ui.add(
            egui::TextEdit::singleline(&mut app.config.storage.data_dir.to_string_lossy().to_string())
                .interactive(false)
        );
        if Button::new("更改")
            .with_style(styles::ButtonStyle::outlined())
            .show(ui)
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("选择数据存储位置")
                .pick_folder()
            {
                app.config.storage.data_dir = path;
                app.save_config();
            }
        }
    });

    // 自动备份
    ui.checkbox(&mut app.config.storage.backup_enabled, "启用自动备份");

    if app.config.storage.backup_enabled {
        ui.indent("backup_settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("备份间隔（小时）");
                if ui.add(
                    egui::DragValue::new(&mut app.config.storage.backup_interval_hours)
                        .clamp_range(1..=168)
                ).changed() {
                    app.save_config();
                }
            });

            ui.horizontal(|ui| {
                ui.label("保留备份数量");
                if ui.add(
                    egui::DragValue::new(&mut app.config.storage.max_backup_count)
                        .clamp_range(1..=100)
                ).changed() {
                    app.save_config();
                }
            });
        });

        // 立即备份
        if Button::new("立即备份")
            .show(ui)
            .clicked()
        {
            if let Err(err) = app.create_backup() {
                app.show_error(format!("备份失败: {}", err));
            }
        }
    }

    ui.add_space(styles::SPACING_SMALL);

    // 数据清理
    ui.horizontal(|ui| {
        ui.label("保留数据时长（天）");
        if ui.add(
            egui::DragValue::new(&mut app.config.storage.keep_days)
                .clamp_range(7..=365)
        ).changed() {
            app.save_config();
        }
    });

    if Button::new("清理旧数据")
        .with_style(styles::ButtonStyle::danger())
        .show(ui)
        .clicked()
    {
        app.show_confirmation(
            "清理数据".to_string(),
            format!(
                "确定要清理{}天前的数据吗？此操作不可恢复。",
                app.config.storage.keep_days
            ),
            Box::new(|app| {
                app.storage.cleanup_old_data(app.config.storage.keep_days)
            }),
        );
    }

    ui.add_space(styles::SPACING_SMALL);

    // 数据导出
    if Button::new("导出数据")
        .show(ui)
        .clicked()
    {
        app.push_dialog(Dialog::Export(ExportDialog::default()));
    }

    if Button::new("导入数据")
        .with_style(styles::ButtonStyle::outlined())
        .show(ui)
        .clicked()
    {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("选择要导入的备份文件")
            .add_filter("备份文件", &["db"])
            .pick_file()
        {
            app.show_confirmation(
                "导入数据".to_string(),
                "导入数据将覆盖当前的所有数据，是否继续？".to_string(),
                Box::new(move |app| {
                    app.storage.import_data(&path)
                }),
            );
        }
    }
}

fn render_about_section(ui: &mut egui::Ui) {
    ui.heading("关于");

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("版本 ");
        ui.label(env!("CARGO_PKG_VERSION"));
        ui.label(" (");
        ui.hyperlink_to(
            "检查更新",
            "https://github.com/yourusername/timetracker/releases",
        );
        ui.label(")");
    });

    ui.horizontal_wrapped(|ui| {
        ui.label("Made with ❤️ by ");
        ui.hyperlink_to("Your Name", "https://github.com/yourusername");
    });

    if Button::new("查看使用说明")
        .show(ui)
        .clicked()
    {
        // TODO: 显示使用说明对话框
    }
}

fn get_language_name(code: &str) -> &'static str {
    match code {
        "zh-CN" => "简体中文",
        "en" => "English",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;

    #[test]
    fn test_settings_view_rendering() {
        let ctx = Context::default();
        let mut app = TimeTrackerApp::test_new();

        ctx.run(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render(&mut app, ui);
            });
        });
    }

    #[test]
    fn test_language_name() {
        assert_eq!(get_language_name("zh-CN"), "简体中文");
        assert_eq!(get_language_name("en"), "English");
        assert_eq!(get_language_name("unknown"), "Unknown");
    }

    #[test]
    fn test_config_changes() {
        let ctx = Context::default();
        let mut app = TimeTrackerApp::test_new();
        let original_font_size = app.config.ui.font_size;

        ctx.run(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render(&mut app, ui);
                
                // 修改设置
                app.config.ui.font_size = original_font_size + 2;
                app.save_config();
            });
        });

        // 验证设置是否被保存
        let new_config = Config::load().unwrap();
        assert_eq!(new_config.ui.font_size, original_font_size + 2);
    }
}