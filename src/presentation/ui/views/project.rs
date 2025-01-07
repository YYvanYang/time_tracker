//src/ui/views/project.rs

use crate::error::Result;
use crate::ui::{styles, components::*};
use crate::ui::TimeTrackerApp;
use eframe::egui;
use chrono::Local;

pub fn render(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(styles::SPACING_LARGE, styles::SPACING_LARGE);

    // 标题栏
    ui.horizontal(|ui| {
        ui.heading("项目管理");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::new("新建项目")
                .show(ui)
                .clicked()
            {
                let dialog = ProjectDialog::new("新建项目")
                    .with_on_save(|app, name, description, color| {
                        app.storage.create_project(name, description, color)
                    });
                app.push_dialog(Dialog::AddProject(dialog));
            }
        });
    });

    // 项目列表和详情
    ui.horizontal(|ui| {
        // 左侧项目列表
        ui.vertical(|ui| {
            render_project_list(app, ui);
        }).min_width(250.0);

        ui.separator();

        // 右侧项目详情
        ui.vertical(|ui| {
            if let Some(project) = app.get_current_project() {
                render_project_details(app, ui, &project);
            } else {
                render_empty_state(ui);
            }
        });
    });
}

fn render_project_list(app: &mut TimeTrackerApp, ui: &mut egui::Ui) {
    // 搜索框
    let mut search = String::new();
    ui.horizontal(|ui| {
        ui.label("🔍");
        ui.text_edit_singleline(&mut search);
    });

    ui.add_space(styles::SPACING_SMALL);

    // 项目列表
    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 40.0)
        .show(ui, |ui| {
            if let Ok(projects) = app.storage.get_projects() {
                for project in projects {
                    let is_selected = app.current_project_id == Some(project.id);
                    let response = ui.add(
                        ProjectListItem::new(&project)
                            .selected(is_selected)
                    );

                    if response.clicked() {
                        app.current_project_id = Some(project.id);
                    }

                    if response.double_clicked() {
                        let dialog = ProjectDialog::new("编辑项目")
                            .with_project(&project)
                            .with_on_save(move |app, name, description, color| {
                                app.storage.update_project(project.id, name, description, color)
                            });
                        app.push_dialog(Dialog::EditProject(dialog));
                    }
                }
            }
        });
}

fn render_project_details(app: &mut TimeTrackerApp, ui: &mut egui::Ui, project: &Project) {
    // 项目标题和操作按钮
    ui.horizontal(|ui| {
        ui.heading(&project.name);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::new("删除")
                .with_style(styles::ButtonStyle::danger())
                .show(ui)
                .clicked()
            {
                app.show_confirmation(
                    "删除项目".to_string(),
                    format!("确定要删除项目"{}"吗？此操作不可恢复。", project.name),
                    Box::new(move |app| {
                        app.storage.delete_project(project.id)
                    }),
                );
            }

            if Button::new("编辑")
                .with_style(styles::ButtonStyle::outlined())
                .show(ui)
                .clicked()
            {
                let dialog = ProjectDialog::new("编辑项目")
                    .with_project(project)
                    .with_on_save(move |app, name, description, color| {
                        app.storage.update_project(project.id, name, description, color)
                    });
                app.push_dialog(Dialog::EditProject(dialog));
            }
        });
    });

    // 项目描述
    if let Some(description) = &project.description {
        ui.label(description);
    }

    ui.separator();

    // 项目统计
    if let Ok(stats) = app.storage.get_project_stats(project) {
        ui.horizontal(|ui| {
            Card::new()
                .show(ui, |ui| {
                    ui.label("工作时长");
                    ui.heading(format_duration(stats.total_time));
                });

            Card::new()
                .show(ui, |ui| {
                    ui.label("完成番茄数");
                    ui.heading(format!("{}", stats.completed_pomodoros));
                });

            Card::new()
                .show(ui, |ui| {
                    ui.label("专注度");
                    ui.heading(format!("{:.1}%", stats.focus_score * 100.0));
                });
        });
    }

    ui.add_space(styles::SPACING_MEDIUM);

    // 任务列表
    ui.horizontal(|ui| {
        ui.heading("任务");
        if Button::new("添加任务")
            .show(ui)
            .clicked()
        {
            let dialog = TaskDialog::new()
                .with_project_id(project.id)
                .with_on_save(move |app, title, estimated_pomodoros| {
                    app.storage.create_task(project.id, title, estimated_pomodoros)
                });
            app.push_dialog(Dialog::AddTask(dialog));
        }
    });

    if let Ok(tasks) = app.storage.get_project_tasks(project.id) {
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for task in tasks {
                    Card::new()
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if task.completed {
                                    ui.checkbox(&mut true, "✓");
                                } else {
                                    let mut completed = false;
                                    if ui.checkbox(&mut completed, "").changed() && completed {
                                        app.storage.complete_task(task.id).ok();
                                    }
                                }

                                ui.label(&task.title);

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if !task.completed {
                                        ui.label(format!("{}🍅", task.remaining_pomodoros));
                                    }
                                });
                            });
                        });
                }
            });
    }

    ui.add_space(styles::SPACING_MEDIUM);

    // 标签管理
    ui.horizontal(|ui| {
        ui.heading("标签");
        if Button::new("添加标签")
            .show(ui)
            .clicked()
        {
            let dialog = TagDialog::new()
                .with_on_save(move |app, name, color| {
                    app.storage.add_project_tag(project.id, name, color)
                });
            app.push_dialog(Dialog::AddTag(dialog));
        }
    });

    if let Ok(tags) = app.storage.get_project_tags(project.id) {
        ui.horizontal(|ui| {
            for tag in tags {
                Tag::new(&tag.name)
                    .with_color(tag.color)
                    .removable()
                    .show(ui)
                    .map(|_| {
                        app.storage.remove_project_tag(project.id, &tag.name).ok();
                    });
            }
        });
    }

    ui.add_space(styles::SPACING_MEDIUM);

    // 时间统计图表
    ui.heading("时间统计");
    if let Ok(time_data) = app.storage.get_project_time_distribution(project.id) {
        Chart::new(time_data)
            .with_size(ui.available_width(), 200.0)
            .with_color(project.color)
            .show(ui);
    }

    // 进度预测
    if let Ok(prediction) = app.storage.get_project_completion_prediction(project.id) {
        ui.add_space(styles::SPACING_MEDIUM);
        Card::new()
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("📊");
                    match prediction {
                        ProjectPrediction::OnTrack { estimated_completion } => {
                            ui.colored_label(
                                styles::COLOR_SUCCESS,
                                format!(
                                    "按当前进度，预计将于 {} 完成项目",
                                    estimated_completion.format("%Y-%m-%d")
                                ),
                            );
                        }
                        ProjectPrediction::Delayed { delay_days } => {
                            ui.colored_label(
                                styles::COLOR_WARNING,
                                format!(
                                    "项目进度落后 {} 天，建议增加工作时间",
                                    delay_days
                                ),
                            );
                        }
                        ProjectPrediction::NeedsMoreData => {
                            ui.colored_label(
                                styles::COLOR_INFO,
                                "需要更多数据来进行进度预测",
                            );
                        }
                    }
                });
            });
    }
}

fn render_empty_state(ui: &mut egui::Ui) {
    styles::centered_column(ui, |ui| {
        ui.add_space(100.0);
        ui.label("👈 请选择或创建一个项目");
    });
}

// 自定义项目列表项组件
struct ProjectListItem<'a> {
    project: &'a Project,
    selected: bool,
}

impl<'a> ProjectListItem<'a> {
    fn new(project: &'a Project) -> Self {
        Self {
            project,
            selected: false,
        }
    }

    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl<'a> egui::Widget for ProjectListItem<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let ProjectListItem { project, selected } = self;

        let padding = egui::vec2(8.0, 6.0);
        let total_extra = padding * 2.0;

        let mut text = project.name.clone();
        if let Ok(stats) = ui.memory_mut(|mem| {
            // 使用内存缓存项目统计信息
            let cache_key = format!("project_stats_{}", project.id);
            if let Some(stats) = mem.data.get_temp::<ProjectStats>(&cache_key) {
                Ok(stats.clone())
            } else {
                Err(()) // 需要重新获取统计信息
            }
        }) {
            text = format!(
                "{} ({} 🍅)",
                project.name,
                stats.completed_pomodoros
            );
        }

        let galley = ui.painter().layout_no_wrap(
            text,
            styles::body().into(),
            if selected {
                styles::COLOR_PRIMARY
            } else {
                ui.style().visuals.text_color()
            },
        );

        let desired_size = galley.size() + total_extra;
        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            // 绘制背景
            if selected || response.hovered() {
                ui.painter().rect_filled(
                    rect,
                    styles::BORDER_RADIUS,
                    if selected {
                        styles::COLOR_PRIMARY.linear_multiply(0.1)
                    } else {
                        visuals.bg_fill
                    },
                );
            }

            // 绘制文本
            let text_pos = rect.min + padding;
            ui.painter().galley(text_pos, galley);
        }

        response
    }
}

// 数据结构
#[derive(Debug, Clone)]
struct Project {
    id: i64,
    name: String,
    description: Option<String>,
    color: egui::Color32,
    created_at: chrono::DateTime<Local>,
    updated_at: chrono::DateTime<Local>,
}

#[derive(Debug, Clone)]
struct ProjectStats {
    total_time: std::time::Duration,
    completed_pomodoros: u32,
    focus_score: f32,
}

#[derive(Debug)]
enum ProjectPrediction {
    OnTrack {
        estimated_completion: chrono::DateTime<Local>,
    },
    Delayed {
        delay_days: u32,
    },
    NeedsMoreData,
}

fn format_duration(duration: std::time::Duration) -> String {
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;

    #[test]
    fn test_project_view_rendering() {
        let ctx = Context::default();
        let mut app = TimeTrackerApp::test_new();

        ctx.run(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render(&mut app, ui);
            });
        });
    }

    #[test]
    fn test_project_list_item() {
        let ctx = Context::default();
        let project = Project {
            id: 1,
            name: "Test Project".to_string(),
            description: None,
            color: styles::COLOR_PRIMARY,
            created_at: Local::now(),
            updated_at: Local::now(),
        };

        ctx.run(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = ui.add(ProjectListItem::new(&project));
                assert!(response.clicked().is_some());
            });
        });
    }

    #[test]
    fn test_empty_state() {
        let ctx = Context::default();
        
        ctx.run(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_empty_state(ui);
            });
        });
    }
}