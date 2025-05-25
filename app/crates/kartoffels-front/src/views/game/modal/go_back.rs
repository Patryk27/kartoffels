use crate::Ui;
use crate::views::game::Event;
use ratatui::text::Line;
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct GoBackModal;

impl GoBackModal {
    pub fn render(&mut self, ui: &mut Ui<Event>) {
        ui.wmodal(37, 3, Some("exit"), |ui| {
            ui.line(
                Line::raw("do you want to exit to the main menu?").centered(),
            );

            ui.line("");

            ui.row(|ui| {
                ui.btn("no", KeyCode::Char('n'), |btn| {
                    btn.throwing(Event::CloseModal)
                });

                ui.btn("yes", KeyCode::Char('y'), |btn| {
                    btn.throwing(Event::GoBack { confirm: false })
                        .right_aligned()
                });
            });
        });
    }
}
