use crate::views::game::Event;
use anyhow::Error;
use kartoffels_ui::{Button, KeyCode, Ui};
use kartoffels_utils::ErrorExt;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Paragraph;

#[derive(Debug)]
pub struct ErrorModal {
    error: Paragraph<'static>,
}

impl ErrorModal {
    pub fn new(error: Error) -> Self {
        Self {
            error: Paragraph::new(error.to_fmt_string())
                .wrap(Default::default()),
        }
    }

    pub fn render(&self, ui: &mut Ui<Event>) {
        let width = 60;
        let height = self.error.line_count(width) as u16 + 2;

        ui.error_window(width, height, Some(" ouch "), |ui| {
            let [text_area, _, footer_area] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(ui.area);

            ui.add_at(text_area, &self.error);

            ui.add_at(footer_area, {
                Button::new(KeyCode::Enter, "close")
                    .throwing(Event::CloseModal)
                    .right_aligned()
            });
        });
    }
}
