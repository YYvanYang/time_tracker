use super::base::{Dialog, DialogContext};
use crate::ui::styles;
use crate::error::Result;
use crate::ui::TimeTrackerApp;
use eframe::egui;
use crate::ui::components::Button;

pub struct ProjectDialog {
    pub title: String,
    pub name: String,
    pub description: String,
    pub color: egui::Color32,
    pub on_save: Option<Box<dyn FnOnce(&mut TimeTrackerApp, String, String, egui::Color32) -> Result<()> + Send>>,
}

impl ProjectDialog {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            name: String::new(),
            description: String::new(),
            color: styles::COLOR_PRIMARY,
            on_save: None,
        }
    }
}

impl Dialog for ProjectDialog {
    fn show(&mut self, ctx: &egui::Context, dialog_ctx: &mut DialogContext) -> bool {
        let mut is_open = true;
        
        egui::Window::new(&self.title)
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(styles::SPACING_MEDIUM, styles::SPACING_MEDIUM);

                // 项目名称
                ui.label("名称");
                ui.text_edit_singleline(&mut self.name);

                // 项目描述
                ui.label("描述");
                ui.text_edit_multiline(&mut self.description);

                // 项目颜色
                ui.horizontal(|ui| {
                    ui.label("颜色");
                    egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut self.color,
                        egui::color_picker::Alpha::Opaque,
                    );
                });

                ui.add_space(ui.spacing().item_spacing.y);

                // 按钮区域
                ui.horizontal(|ui| {
                    if Button::new("取消")
                        .with_style(styles::ButtonStyle::outlined())
                        .show(ui)
                        .clicked()
                    {
                        dialog_ctx.pop_dialog();
                    }

                    if Button::new("保存")
                        .enabled(!self.name.is_empty())
                        .show(ui)
                        .clicked()
                    {
                        if let Some(on_save) = self.on_save.take() {
                            if let Err(e) = on_save(
                                dialog_ctx.app,
                                self.name.clone(),
                                self.description.clone(),
                                self.color,
                            ) {
                                dialog_ctx.show_error(e.to_string());
                            }
                        }
                        dialog_ctx.pop_dialog();
                    }
                });
            });

        is_open
    }

    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(crate::error::TimeTrackerError::Dialog(
                "项目名称不能为空".into()
            ));
        }
        Ok(())
    }
} 