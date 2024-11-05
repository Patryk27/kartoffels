use super::{Event, Focus};
use kartoffels_ui::{Button, Render, Ui};
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub enum BotLocation {
    Manual,
    #[default]
    Random,
}

impl BotLocation {
    pub fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        Button::new(KeyCode::Char('l'), format!("location: {val}"))
            .throwing(Event::FocusOn(Some(Focus::BotLocation)))
            .render(ui);
    }

    pub fn render_choice(ui: &mut Ui<Event>) {
        Button::new(KeyCode::Char('m'), Self::Manual.to_string())
            .throwing(Event::SetBotLocation(Self::Manual))
            .render(ui);

        Button::new(KeyCode::Char('r'), Self::Random.to_string())
            .throwing(Event::SetBotLocation(Self::Random))
            .render(ui);
    }

    pub fn height() -> u16 {
        2
    }
}

impl fmt::Display for BotLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Manual => "manual (using mouse)",
                Self::Random => "random",
            }
        )
    }
}
