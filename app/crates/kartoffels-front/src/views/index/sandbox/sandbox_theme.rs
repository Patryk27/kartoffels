use super::{Event, Focus};
use crate::Ui;
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Clone, Debug)]
pub enum SandboxTheme {
    Arena,
    Cave,
}

impl SandboxTheme {
    pub fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        ui.btn(format!("theme: {val}"), KeyCode::Char('t'), |btn| {
            btn.throwing(Event::FocusOn(Some(Focus::SandboxTheme)))
        });
    }

    pub fn render_choice(ui: &mut Ui<Event>) {
        for val in SandboxTheme::all() {
            ui.btn(val.to_string(), val.key(), |btn| {
                btn.throwing(Event::SetTheme(val))
            });
        }
    }

    pub fn height() -> u16 {
        Self::all().count() as u16
    }

    fn all() -> impl Iterator<Item = Self> {
        [Self::Arena, Self::Cave].into_iter()
    }

    fn key(&self) -> KeyCode {
        KeyCode::Char(match self {
            Self::Arena => 'a',
            Self::Cave => 'c',
        })
    }
}

impl fmt::Display for SandboxTheme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Arena => "arena",
                Self::Cave => "cave",
            }
        )
    }
}
