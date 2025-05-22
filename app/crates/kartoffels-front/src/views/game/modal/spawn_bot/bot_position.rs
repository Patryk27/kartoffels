use super::{Event, Focus};
use crate::Ui;
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Clone, Copy, Debug, Default)]
pub enum BotPosition {
    Manual,
    #[default]
    Random,
}

impl BotPosition {
    pub(super) fn render_btn(ui: &mut Ui<Event>, val: &Self) {
        ui.btn(format!("position: {val}"), KeyCode::Char('p'), |btn| {
            btn.throwing(Event::FocusOn(Some(Focus::BotPosition)))
        });
    }

    pub(super) fn render_form(ui: &mut Ui<Event>) {
        for (idx, val) in Self::all().enumerate() {
            if idx > 0 {
                ui.space(1);
            }

            ui.btn(val.to_string(), val.key(), |btn| {
                btn.help(val.desc()).throwing(Event::SetBotPosition(val))
            });
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
