use super::{Event, Focus};
use crate::{Button, Ui, UiWidget};
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Clone, Copy, Debug, Default)]
pub enum BotPosition {
    Manual,
    #[default]
    Random,
}

impl BotPosition {
    pub(super) fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        Button::new(format!("position: {val}"), KeyCode::Char('p'))
            .throwing(Event::FocusOn(Some(Focus::BotPosition)))
            .render(ui);
    }

    pub(super) fn render_choice(ui: &mut Ui<Event>) {
        for (idx, val) in Self::all().enumerate() {
            if idx > 0 {
                ui.space(1);
            }

            Button::new(val.to_string(), val.key())
                .help(val.desc())
                .throwing(Event::SetBotPosition(val))
                .render(ui);
        }
    }

    pub fn height() -> u16 {
        (Self::all().count() * 3 - 1) as u16
    }

    fn all() -> impl Iterator<Item = Self> {
        [Self::Manual, Self::Random].into_iter()
    }

    fn key(&self) -> KeyCode {
        KeyCode::Char(match self {
            Self::Manual => 'm',
            Self::Random => 'r',
        })
    }

    fn desc(&self) -> &'static str {
        match self {
            Self::Manual => "use mouse to place your bot wherever you like",
            Self::Random => "it's, well, random",
        }
    }
}

impl fmt::Display for BotPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Manual => "manual",
                Self::Random => "random",
            }
        )
    }
}
