use crate::views::game::Event;
use kartoffels_ui::{Button, KeyCode, Ui, UiWidget};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Paragraph;

#[derive(Debug)]
pub struct ErrorModal {
    error: Paragraph<'static>,
}

impl ErrorModal {
    pub fn new(error: String) -> Self {
        Self {
            error: Paragraph::new(error).wrap(Default::default()),
        }
    }

    pub fn render(&self, ui: &mut Ui<Event>) {
        let width = 50;
        let height = self.error.line_count(width) as u16 + 2;

        ui.error_window(width, height, Some(" ouch "), |ui| {
            let [text_area, _, footer_area] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(ui.area);

            ui.clamp(text_area, |ui| {
                ui.render(&self.error);
            });

            ui.clamp(footer_area, |ui| {
                Button::new(KeyCode::Enter, "close")
                    .throwing(Event::CloseModal)
                    .right_aligned()
                    .render(ui);
            });
        });
    }
}
