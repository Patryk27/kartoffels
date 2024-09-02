use super::Response;
use crate::{Button, Ui};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct Menu;

impl Menu {
    pub fn height(ui: &Ui) -> u16 {
        if ui.is_over_ssh() {
            8
        } else {
            5
        }
    }

    pub fn render(ui: &mut Ui) -> Option<Response> {
        let mut response = None;

        if Button::new(KeyCode::Char('p'), "play")
            .centered()
            .render(ui)
            .pressed
        {
            response = Some(Response::Play);
        }

        ui.space(1);

        if Button::new(KeyCode::Char('t'), "tutorial")
            .centered()
            .render(ui)
            .pressed
        {
            response = Some(Response::OpenTutorial);
        }

        ui.space(1);

        if Button::new(KeyCode::Char('c'), "challenges")
            .centered()
            .render(ui)
            .pressed
        {
            response = Some(Response::OpenChallenges);
        }

        if ui.is_over_ssh() {
            ui.space(2);

            if Button::new(KeyCode::Escape, "quit")
                .centered()
                .render(ui)
                .pressed
            {
                response = Some(Response::Quit);
            }
        }

        response
    }
}
