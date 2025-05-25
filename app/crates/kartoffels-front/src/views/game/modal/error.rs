use crate::Ui;
use crate::views::game::Event;
use anyhow::Error;
use kartoffels_utils::ErrorExt;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Paragraph;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct ErrorModal {
    error: Box<Paragraph<'static>>,
}

impl ErrorModal {
    pub fn new(error: Error) -> Self {
        Self {
            error: Box::new(
                Paragraph::new(error.to_fmt_string()).wrap(Default::default()),
            ),
        }
    }

    pub fn render(&self, ui: &mut Ui<Event>) {
        let width = 60;
        let height = self.error.line_count(width) as u16 + 2;

        ui.emodal(width, height, Some("ouch"), |ui| {
            let [text_area, _, footer_area] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(ui.area);

            ui.add_at(text_area, &*self.error);

            ui.at(footer_area, |ui| {
                ui.btn("close", KeyCode::Enter, |btn| {
                    btn.throwing(Event::CloseModal).right_aligned()
                });
            });
        });
    }
}
