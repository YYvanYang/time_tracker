use eframe::egui;
use crate::ui::{TimeTrackerApp, styles};
use super::components::{Button, Card, Dialog, dialog::ProjectDialog};
use crate::storage::Project;
use chrono::{Utc, Local};
use std::sync::Arc;
use std::cell::RefCell;

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("项目管理");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::new("添加项目")
                .with_style(styles::ButtonStyle::primary())
                .show(ui)
                .clicked()
            {
                app.show_add_project_dialog = true;
            }
        });
    });
    ui.separator();

    // 显示项目列表
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for project in &app.projects {
                Card::new().show(ui, |ui| {
                    ui.heading(&project.name);
                    ui.horizontal(|ui| {
                        if let Some(desc) = &project.description {
                            ui.label(desc);
                        } else {
                            ui.label("暂无描述");
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if Button::new("编辑")
                                .show(ui)
                                .clicked()
                            {
                                let project = project.clone();
                                ProjectDialog::new("编辑项目").show(ui.ctx(), |ui| {
                                    let mut project_name = project.name.clone();
                                    let mut project_description = project.description.clone().unwrap_or_default();

                                    ui.horizontal(|ui| {
                                        ui.label("项目名称：");
                                        ui.text_edit_singleline(&mut project_name);
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("项目描述：");
                                        ui.text_edit_multiline(&mut project_description);
                                    });

                                    ui.horizontal(|ui| {
                                        if Button::new("取消")
                                            .show(ui)
                                            .clicked()
                                        {
                                            return;
                                        }

                                        if Button::new("确定")
                                            .with_style(styles::ButtonStyle::primary())
                                            .show(ui)
                                            .clicked()
                                        {
                                            if !project_name.is_empty() {
                                                let mut updated_project = project.clone();
                                                updated_project.name = project_name;
                                                updated_project.description = Some(project_description);
                                                updated_project.color = project.color.clone();
                                                updated_project.updated_at = Utc::now().with_timezone(&Local);

                                                if let Ok(mut storage) = app.storage.lock() {
                                                    if let Err(e) = storage.update_project(&updated_project) {
                                                        app.show_error(format!("更新项目失败：{}", e));
                                                    } else {
                                                        // 更新内存中的项目列表
                                                        if let Some(p) = app.projects.iter_mut().find(|p| p.id == project.id) {
                                                            *p = updated_project;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    });
                                });
                            }
                            if Button::new("删除")
                                .with_style(styles::ButtonStyle::danger())
                                .show(ui)
                                .clicked()
                            {
                                let project_id = project.id;
                                app.show_confirmation_dialog(
                                    "删除项目".to_string(),
                                    format!("确定要删除项目{}吗？", project.name),
                                    Box::new(move || {
                                        if let Ok(mut storage) = app.storage.lock() {
                                            if let Err(e) = storage.delete_project(project_id) {
                                                app.show_error(format!("删除项目失败：{}", e));
                                            } else {
                                                // 从列表中移除项目
                                                app.projects.retain(|p| p.id != project_id);
                                            }
                                        }
                                    }),
                                );
                            }
                        });
                    });
                });
                ui.add_space(8.0);
            }
        });

    // 添加项目对话框
    if app.show_add_project_dialog {
        let mut project_name = String::new();
        let mut project_description = String::new();

        ProjectDialog::new("添加项目").show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label("项目名称：");
                ui.text_edit_singleline(&mut project_name);
            });

            ui.horizontal(|ui| {
                ui.label("项目描述：");
                ui.text_edit_multiline(&mut project_description);
            });

            ui.horizontal(|ui| {
                if Button::new("取消")
                    .show(ui)
                    .clicked()
                {
                    app.show_add_project_dialog = false;
                }

                if Button::new("确定")
                    .with_style(styles::ButtonStyle::primary())
                    .show(ui)
                    .clicked()
                {
                    if !project_name.is_empty() {
                        let now = Utc::now().with_timezone(&Local);
                        let new_project = Project {
                            id: None,
                            name: project_name.clone(),
                            description: Some(project_description.clone()),
                            color: Some("#6495ED".to_string()),
                            created_at: now,
                            updated_at: now,
                        };
                        
                        if let Ok(mut storage) = app.storage.lock() {
                            match storage.add_project(&new_project) {
                                Ok(id) => {
                                    let mut project = new_project;
                                    project.id = Some(id);
                                    app.projects.push(project);
                                    app.show_add_project_dialog = false;
                                }
                                Err(e) => {
                                    app.show_error(format!("添加项目失败：{}", e));
                                }
                            }
                        }
                    }
                }
            });
        });
    }
} 