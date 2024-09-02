use super::Response;
use crate::{Button, Ui};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct Menu;

impl Menu {
    pub const HEIGHT: u16 = 5;

    pub fn render(ui: &mut Ui) -> Option<Response> {
        if Button::new(KeyCode::Char('p'), "play")
            .centered()
            .render(ui)
            .pressed
        {
            return Some(Response::Play);
        }

        ui.space(1);

        if Button::new(KeyCode::Char('t'), "tutorial")
            .centered()
            .render(ui)
            .pressed
        {
            return Some(Response::OpenTutorial);
        }

        ui.space(1);

        if Button::new(KeyCode::Char('c'), "challenges")
            .centered()
            .render(ui)
            .pressed
        {
            return Some(Response::OpenChallenges);
        }

        ui.space(2);

        if Button::new(KeyCode::Escape, "quit")
            .centered()
            .render(ui)
            .pressed
        {
            return Some(Response::Quit);
        }

        None
    }
}
