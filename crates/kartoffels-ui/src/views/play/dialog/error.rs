use super::DialogResponse;
use crate::{Button, RectExt, Ui};
use ratatui::widgets::{Paragraph, Widget};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct ErrorDialog {
    pub error: String,
}

impl ErrorDialog {
    pub fn render(&self, ui: &mut Ui) -> Option<DialogResponse> {
        let text = Paragraph::new(self.error.as_str()).wrap(Default::default());
        let width = 50;
        let height = text.line_count(width) as u16 + 2;

        ui.error_dialog(width, height, Some(" whoopsie "), |ui| {
            text.render(ui.area(), ui.buf());

            ui.clamp(ui.area().footer(), |ui| {
                if Button::new(KeyCode::Enter, "close")
                    .right()
                    .render(ui)
                    .pressed
                {
                    Some(DialogResponse::Close)
                } else {
                    None
                }
            })
        })
    }
}
