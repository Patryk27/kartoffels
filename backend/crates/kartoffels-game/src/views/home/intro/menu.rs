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

    pub fn render(ui: &mut Ui, store: &Store) -> Option<Response> {
        let mut resp = None;

        let block = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .padding(Padding::horizontal(1));

        ui.block(block, |ui| {
            if !store.worlds.is_empty() {
                if Button::new(KeyCode::Char('p'), "online play")
                    .centered()
                    .render(ui)
                    .pressed
                {
                    resp = Some(Response::OnlinePlay);
                }

                ui.space(1);
            }

            if Button::new(KeyCode::Char('t'), "tutorial")
                .centered()
                .render(ui)
                .pressed
            {
                resp = Some(Response::Tutorial);
            }

            if Button::new(KeyCode::Char('s'), "sandbox")
                .centered()
                .render(ui)
                .pressed
            {
                resp = Some(Response::Sandbox);
            }

            if Button::new(KeyCode::Char('c'), "challenges")
                .centered()
                .render(ui)
                .pressed
            {
                resp = Some(Response::Challenges);
            }

            if ui.ty().is_ssh() {
                ui.space(1);

                if Button::new(KeyCode::Escape, "quit")
                    .centered()
                    .render(ui)
                    .pressed
                {
                    resp = Some(Response::Quit);
                }
            }
        });

        resp
    }
}
