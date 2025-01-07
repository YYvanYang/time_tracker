use eframe::egui;
use crate::ui::{TimeTrackerApp, styles};
use super::components::{Card, ProgressBar};
use crate::storage::app_state::Task;

pub fn render(_app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.heading("今日概览");
    ui.separator();

    // 显示当前状态
    Card::new()
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("工作中");
                    ui.label("剩余时间: 15:32");
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ProgressBar::new(0.35).show(ui);
                });
            });
        });

    ui.separator();

    // 显示今日统计
    ui.horizontal(|ui| {
        Card::new()
            .show(ui, |ui| {
                ui.label("完成番茄数");
                ui.heading("6");
            });

        Card::new()
            .show(ui, |ui| {
                ui.label("专注时长");
                ui.heading("3小时12分");
            });

        Card::new()
            .show(ui, |ui| {
                ui.label("效率得分");
                ui.heading("78%");
            });
    });

    ui.separator();

    // 显示今日任务
    ui.heading("今日任务");
    
    // 添加新任务的输入框
    ui.horizontal(|ui| {
        let mut new_task = String::new();
        let response = ui.text_edit_singleline(&mut new_task);
        
        let should_add = (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            || ui.button("添加任务").clicked();
            
        if should_add && !new_task.trim().is_empty() {
            _app.tasks.push(Task::new(new_task));
        }
    });

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            if _app.tasks.is_empty() {
                ui.label("今天还没有添加任务");
            } else {
                let mut tasks_to_remove = Vec::new();
                let mut changes_made = false;

                for (index, task) in _app.tasks.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let changed = ui.checkbox(&mut task.completed, "").changed();
                        
                        if task.completed {
                            ui.label(egui::RichText::new(&task.title)
                                .strikethrough()
                                .color(styles::GRAY));
                        } else {
                            ui.label(&task.title);
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑").clicked() {
                                tasks_to_remove.push(index);
                            }
                        });

                        if changed {
                            changes_made = true;
                        }
                    });
                }
                
                // 删除标记的任务
                for &index in tasks_to_remove.iter().rev() {
                    _app.tasks.remove(index);
                }
                
                // 保存所有更改
                if changes_made || !tasks_to_remove.is_empty() {
                    if let Err(e) = _app.save_state() {
                        _app.show_error(format!("保存任务状态失败: {}", e));
                    }
                }
            }
        });
}