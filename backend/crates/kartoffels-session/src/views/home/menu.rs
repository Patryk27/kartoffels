use super::Response;
use kartoffels_ui::{theme, Button, Ui};
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct Menu;

impl Menu {
    pub fn width() -> u16 {
        20
    }

    pub fn height<T>(ui: &Ui<T>) -> u16 {
        if ui.ty().is_ssh() {
            5
        } else {
            4
        }
    }

    pub fn render(ui: &mut Ui<Response>) {
        let block = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .padding(Padding::horizontal(1));

        ui.block(block, |ui| {
            Button::new(KeyCode::Char('p'), "play")
                .throwing(Response::Play)
                .centered()
                .render(ui);

            Button::new(KeyCode::Char('t'), "tutorial")
                .throwing(Response::Tutorial)
                .centered()
                .render(ui);

            if ui.ty().is_ssh() {
                Button::new(KeyCode::Escape, "quit")
                    .throwing(Response::Quit)
                    .centered()
                    .render(ui);
            }
        });
    }
}
