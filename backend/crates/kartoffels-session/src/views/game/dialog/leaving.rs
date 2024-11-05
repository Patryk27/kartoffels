use crate::views::game::Event;
use kartoffels_ui::{theme, Button, Render, Ui};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct GoBackDialog;

impl GoBackDialog {
    pub fn render(&mut self, ui: &mut Ui<Event>) {
        ui.window(42, 3, Some(" go back "), theme::YELLOW, |ui| {
            ui.line("do you want to go back to the main menu?");
            ui.space(1);

            ui.row(|ui| {
                Button::new(KeyCode::Char('n'), "no, continue game")
                    .throwing(Event::CloseDialog)
                    .render(ui);

                Button::new(KeyCode::Char('y'), "yes, leave game")
                    .throwing(Event::GoBack(false))
                    .right_aligned()
                    .render(ui);
            });
        });
    }
}
