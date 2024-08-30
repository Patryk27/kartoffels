use super::Outcome;
use crate::{Button, Ui};
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct Menu;

impl Menu {
    pub const HEIGHT: u16 = 5;

    pub fn render(ui: &mut Ui) -> Option<Outcome> {
        if Button::new(KeyCode::Char('p'), "play")
            .centered()
            .render(ui)
            .activated
        {
            return Some(Outcome::Play);
        }

        ui.step(1);

        if Button::new(KeyCode::Char('t'), "tutorial")
            .centered()
            .render(ui)
            .activated
        {
            return Some(Outcome::OpenTutorial);
        }

        ui.step(1);

        if Button::new(KeyCode::Char('c'), "challenges")
            .centered()
            .render(ui)
            .activated
        {
            return Some(Outcome::OpenChallenges);
        }

        ui.step(2);

        if Button::new(KeyCode::Escape, "quit")
            .centered()
            .render(ui)
            .activated
        {
            return Some(Outcome::Quit);
        }

        None
    }
}
