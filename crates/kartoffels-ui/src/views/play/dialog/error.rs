use super::DialogEvent;
use crate::{Button, Ui};
use ratatui::widgets::{Paragraph, Widget};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct ErrorDialog {
    pub error: String,
}

impl ErrorDialog {
    pub fn render(&self, ui: &mut Ui) -> Option<DialogEvent> {
        let text = Paragraph::new(self.error.as_str()).wrap(Default::default());

        let width = 50;
        let height = text.line_count(width) as u16;

        ui.error_dialog(width, height, Some(" whoopsie "), |ui| {
            text.render(ui.area(), ui.buf());

            if Button::new(KeyCode::Enter, "close").render(ui).activated {
                Some(DialogEvent::Close)
            } else {
                None
            }
        })
    }
}
