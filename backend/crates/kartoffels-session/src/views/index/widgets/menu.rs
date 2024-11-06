use super::super::Event;
use kartoffels_store::Store;
use kartoffels_ui::{theme, Button, Render, Ui};
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct Menu;

impl Menu {
    pub fn width() -> u16 {
        20
    }

    pub fn height<T>(store: &Store, ui: &Ui<T>) -> u16 {
        let height = if ui.ty.is_ssh() { 7 } else { 5 };

        if store.public_worlds().is_empty() {
            height
        } else {
            height + 1
        }
    }

    pub fn render(store: &Store, ui: &mut Ui<Event>) {
        let block = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .padding(Padding::horizontal(1));

        ui.block(block, |ui| {
            if !store.public_worlds().is_empty() {
                Button::new(KeyCode::Char('p'), "play")
                    .throwing(Event::Play)
                    .centered()
                    .render(ui);
            }

            Button::new(KeyCode::Char('s'), "sandbox")
                .throwing(Event::Sandbox)
                .centered()
                .render(ui);

            Button::new(KeyCode::Char('t'), "tutorial")
                .throwing(Event::Tutorial)
                .centered()
                .render(ui);

            Button::new(KeyCode::Char('c'), "challenges")
                .throwing(Event::Challenges)
                .centered()
                .render(ui);

            if ui.ty.is_ssh() {
                ui.space(1);

                Button::new(KeyCode::Escape, "quit")
                    .throwing(Event::Quit)
                    .centered()
                    .render(ui);
            }
        });
    }
}
