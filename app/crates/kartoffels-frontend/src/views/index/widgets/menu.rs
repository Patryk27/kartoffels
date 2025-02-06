use super::super::Event;
use kartoffels_store::Store;
use kartoffels_ui::{theme, Button, KeyCode, Ui, UiWidget};
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding};

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
