use super::Response;
use kartoffels_store::Store;
use kartoffels_ui::{theme, Button, Ui};
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct Menu;

impl Menu {
    pub fn width() -> u16 {
        24
    }

    pub fn height(ui: &Ui, store: &Store) -> u16 {
        let mut height = if ui.ty().is_ssh() { 7 } else { 5 };

        if !store.worlds.is_empty() {
            height += 2;
        }

        height
    }

    pub fn render(ui: &mut Ui, store: &Store) {
        let block = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .padding(Padding::horizontal(1));

        ui.block(block, |ui| {
            if !store.worlds.is_empty() {
                Button::new(KeyCode::Char('o'), "online play")
                    .throwing(Response::Online)
                    .centered()
                    .render(ui);

                ui.space(1);
            }

            Button::new(KeyCode::Char('s'), "sandbox")
                .throwing(Response::Sandbox)
                .centered()
                .render(ui);

            Button::new(KeyCode::Char('t'), "tutorial")
                .throwing(Response::Tutorial)
                .centered()
                .render(ui);

            Button::new(KeyCode::Char('c'), "challenges")
                .throwing(Response::Challenges)
                .centered()
                .render(ui);

            if ui.ty().is_ssh() {
                ui.space(1);

                Button::new(KeyCode::Escape, "quit")
                    .throwing(Response::Quit)
                    .centered()
                    .render(ui);
            }
        });
    }
}
