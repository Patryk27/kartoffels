use crate::views::game::Event;
use kartoffels_ui::{theme, Button, Render, Ui};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct LeavingDialog;

impl LeavingDialog {
    pub fn render(&mut self, ui: &mut Ui<Event>) {
        ui.window(50, 3, Some(" leaving "), theme::YELLOW, |ui| {
            ui.line("do you want to go back to the main menu?");
            ui.space(1);

            ui.row(|ui| {
                Button::new(KeyCode::Char('n'), "no, continue game")
                    .throwing(Event::CloseDialog)
                    .render(ui);

                Button::new(KeyCode::Char('y'), "yes, go to main menu")
                    .throwing(Event::GoBack)
                    .right_aligned()
                    .render(ui);
            });
        });
    }
}
