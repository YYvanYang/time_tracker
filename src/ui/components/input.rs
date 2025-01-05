use eframe::egui;
use super::styles;

pub struct Input {
    value: String,
    placeholder: String,
    password: bool,
}

impl Input {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            placeholder: String::new(),
            password: false,
        }
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    pub fn show(mut self, ui: &mut egui::Ui) -> Option<String> {
        let response = if self.password {
            ui.add(egui::TextEdit::singleline(&mut self.value)
                .password(true)
                .hint_text(&self.placeholder))
        } else {
            ui.add(egui::TextEdit::singleline(&mut self.value)
                .hint_text(&self.placeholder))
        };

        if response.changed() {
            Some(self.value)
        } else {
            None
        }
    }
} 