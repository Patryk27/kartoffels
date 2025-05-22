use super::{Event, Focus};
use crate::Ui;
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Clone, Debug)]
pub enum SandboxSize {
    Tiny,
    Small,
    Medium,
    Large,
}

impl SandboxSize {
    pub fn render_btn(ui: &mut Ui<Event>, val: &Self) {
        ui.btn(format!("size: {val}"), KeyCode::Char('s'), |btn| {
            btn.throwing(Event::FocusOn(Some(Focus::SandboxSize)))
        });
    }

    pub fn render_form(ui: &mut Ui<Event>) {
        for val in Self::all() {
            ui.btn(val.to_string(), val.key(), |btn| {
                btn.throwing(Event::SetSize(val))
            });
        }
    }

    pub fn height() -> u16 {
        Self::all().count() as u16
    }

    fn all() -> impl Iterator<Item = Self> {
        [Self::Tiny, Self::Small, Self::Medium, Self::Large].into_iter()
    }

    fn key(&self) -> KeyCode {
        KeyCode::Char(match self {
            Self::Tiny => 't',
            Self::Small => 's',
            Self::Medium => 'm',
            Self::Large => 'l',
        })
    }
}

impl fmt::Display for SandboxSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Tiny => "tiny",
                Self::Small => "small",
                Self::Medium => "medium",
                Self::Large => "large",
            }
        )
    }
}
