use super::Response;
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

    pub fn height(ui: &Ui) -> u16 {
        if ui.ty().is_ssh() {
            9
        } else {
            7
        }
    }

    pub fn render(ui: &mut Ui) -> Option<Response> {
        let mut resp = None;

        let block = Block::bordered()
            .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
            .padding(Padding::horizontal(1));

        ui.block(block, |ui| {
            if Button::new(KeyCode::Char('p'), "online play")
                .centered()
                .render(ui)
                .pressed
            {
                resp = Some(Response::OnlinePlay);
            }

            ui.space(1);

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
