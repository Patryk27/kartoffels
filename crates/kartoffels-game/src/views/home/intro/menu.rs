use super::Response;
use kartoffels_ui::{Button, Ui};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct Menu;

impl Menu {
    pub fn height(ui: &Ui) -> u16 {
        if ui.ty().is_ssh() {
            7
        } else {
            5
        }
    }

    pub fn render(ui: &mut Ui) -> Option<Response> {
        let mut resp = None;

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

        resp
    }
}
