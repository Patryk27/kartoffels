use crate::{theme, Ui, UiWidget};
use ratatui::style::Stylize;
use ratatui::text::Span;
use std::time::Instant;
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Clone, Debug)]
pub struct Input {
    value: String,
    caret: Instant,
    secret: bool,
}

impl Input {
    pub const MAX_LENGTH: usize = 128;

    pub fn value(&self) -> &str {
        &self.value
    }

    fn handle(&mut self, event: &InputEvent) {
        match event {
            InputEvent::Key(event) => match (event.key, event.modifiers) {
                (KeyCode::Char(ch), Modifiers::NONE) => {
                    self.add(ch);
                }

                (KeyCode::Backspace, Modifiers::NONE) => {
                    self.value.pop();
                }

                _ => {}
            },

            InputEvent::Paste(payload) => {
                for ch in payload.chars() {
                    self.add(ch);
                }
            }

            _ => (),
        }
    }

    fn add(&mut self, ch: char) {
        if !ch.is_ascii_control() && self.value.len() < Self::MAX_LENGTH {
            self.value.push(ch);
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self {
            value: Default::default(),
            caret: Instant::now(),
            secret: Default::default(),
        }
    }
}

impl<T> UiWidget<T> for &mut Input {
    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        ui.row(|ui| {
            ui.span("> ");

            let value_width = (ui.area.width - 1) as usize;

            if self.secret {
                for _ in 0..self.value.len().min(value_width) {
                    ui.buf[(ui.area.x, ui.area.y)].set_char('*');
                    ui.area.x += 1;
                }
            } else {
                let offset = self.value.len().saturating_sub(value_width);

                for ch in self.value.chars().skip(offset) {
                    ui.buf[(ui.area.x, ui.area.y)].set_char(ch);
                    ui.area.x += 1;
                }
            }

            if self.caret.elapsed().as_millis() % 1000 <= 500 {
                ui.span(Span::raw("_").fg(theme::GREEN));
            }
        });

        if let Some(event) = ui.event {
            self.handle(event);
        }
    }
}
