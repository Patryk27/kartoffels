use crate::views::game::Event;
use kartoffels_ui::{Button, RectExt, Render, Ui};
use ratatui::widgets::Paragraph;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct ErrorDialog {
    error: String,
}

impl ErrorDialog {
    pub fn new(error: String) -> Self {
        Self { error }
    }

    pub fn render(&self, ui: &mut Ui<Event>) {
        let text = Paragraph::new(self.error.as_str()).wrap(Default::default());
        let width = 50;
        let height = text.line_count(width) as u16 + 2;

        ui.error_window(width, height, Some(" ouch "), |ui| {
            text.render(ui);

            ui.clamp(ui.area.footer(1), |ui| {
                Button::new(KeyCode::Enter, "close")
                    .throwing(Event::CloseDialog)
                    .right_aligned()
                    .render(ui);
            });
        });
    }
}
