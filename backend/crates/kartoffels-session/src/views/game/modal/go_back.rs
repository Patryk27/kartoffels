use crate::views::game::Event;
use kartoffels_ui::{theme, Button, Ui, UiWidget};
use ratatui::text::Line;
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct GoBackModal;

impl GoBackModal {
    pub fn render(&mut self, ui: &mut Ui<Event>) {
        ui.window(23, 3, Some(" go-back "), theme::YELLOW, |ui| {
            ui.line(Line::raw("do you want to go back?").centered());
            ui.line("");

            ui.row(|ui| {
                Button::new(KeyCode::Char('n'), "no")
                    .throwing(Event::CloseModal)
                    .render(ui);

                Button::new(KeyCode::Char('y'), "yes")
                    .throwing(Event::GoBack {
                        needs_confirmation: false,
                    })
                    .right_aligned()
                    .render(ui);
            });
        });
    }
}
