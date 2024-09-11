use crate::views::play::Event;
use kartoffels_ui::{Button, RectExt, Ui};
use ratatui::widgets::{Paragraph, Widget};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct ErrorDialog {
    error: String,
}

impl ErrorDialog {
    pub fn new(error: String) -> Self {
        Self { error }
    }

    pub fn render(&self, ui: &mut Ui) {
        let text = Paragraph::new(self.error.as_str()).wrap(Default::default());
        let width = 50;
        let height = text.line_count(width) as u16 + 2;

        ui.error_window(width, height, Some(" whoopsie "), |ui| {
            text.render(ui.area(), ui.buf());

            ui.clamp(ui.area().footer(1), |ui| {
                Button::new(KeyCode::Enter, "close")
                    .throwing(Event::CloseDialog)
                    .right_aligned()
                    .render(ui);
            });
        });
    }
}
