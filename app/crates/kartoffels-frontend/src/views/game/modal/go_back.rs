use crate::views::game::Event;
use kartoffels_ui::{theme, Button, KeyCode, Ui, UiWidget};
use ratatui::text::Line;

#[derive(Debug, Default)]
pub struct GoBackModal;

impl GoBackModal {
    pub fn render(&mut self, ui: &mut Ui<Event>) {
        ui.window(23, 3, Some(" go-back "), theme::YELLOW, |ui| {
            ui.line(Line::raw("do you want to go back?").centered());
            ui.line("");

            ui.row(|ui| {
                Button::new("no", KeyCode::Char('n'))
                    .throwing(Event::CloseModal)
                    .render(ui);

                Button::new("yes", KeyCode::Char('y'))
                    .throwing(Event::GoBack { confirm: false })
                    .right_aligned()
                    .render(ui);
            });
        });
    }
}
