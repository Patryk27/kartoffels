use super::super::Event;
use crate::{theme, Ui};
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct Menu;

impl Menu {
    pub fn width() -> u16 {
        20
    }

    pub fn height<T>(ui: &Ui<T>, has_public_worlds: bool) -> u16 {
        let mut height = 5;

        if has_public_worlds {
            height += 1;
        }

        if ui.ty.is_ssh() {
            height += 2;
        }

        height
    }

    pub fn render(ui: &mut Ui<Event>, has_public_worlds: bool) {
        let block = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .padding(Padding::horizontal(1));

        ui.block(block, |ui| {
            if has_public_worlds {
                ui.btn("play", KeyCode::Char('p'), |btn| {
                    btn.throwing(Event::Play).centered()
                });
            }

            ui.btn("sandbox", KeyCode::Char('s'), |btn| {
                btn.throwing(Event::Sandbox).centered()
            });

            ui.btn("tutorial", KeyCode::Char('t'), |btn| {
                btn.throwing(Event::Tutorial).centered()
            });

            ui.btn("challenges", KeyCode::Char('c'), |btn| {
                btn.throwing(Event::Challenges).centered()
            });

            if ui.ty.is_ssh() {
                ui.space(1);

                ui.btn("quit", KeyCode::Escape, |btn| {
                    btn.throwing(Event::Quit).centered()
                });
            }
        });
    }
}
