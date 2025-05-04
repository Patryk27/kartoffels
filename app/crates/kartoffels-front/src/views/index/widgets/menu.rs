use super::super::Event;
use crate::{theme, Button, Ui, UiWidget};
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
        let mut height = if ui.ty.is_ssh() { 7 } else { 5 };

        if has_public_worlds {
            height += 1;
        }

        height
    }

    pub fn render(ui: &mut Ui<Event>, has_public_worlds: bool) {
        let block = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .padding(Padding::horizontal(1));

        ui.block(block, |ui| {
            if has_public_worlds {
                Button::new("play", KeyCode::Char('p'))
                    .throwing(Event::Play)
                    .centered()
                    .render(ui);
            }

            Button::new("sandbox", KeyCode::Char('s'))
                .throwing(Event::Sandbox)
                .centered()
                .render(ui);

            Button::new("tutorial", KeyCode::Char('t'))
                .throwing(Event::Tutorial)
                .centered()
                .render(ui);

            Button::new("challenges", KeyCode::Char('c'))
                .throwing(Event::Challenges)
                .centered()
                .render(ui);

            if ui.ty.is_ssh() {
                ui.space(1);

                Button::new("quit", KeyCode::Escape)
                    .throwing(Event::Quit)
                    .centered()
                    .render(ui);
            }
        });
    }
}
