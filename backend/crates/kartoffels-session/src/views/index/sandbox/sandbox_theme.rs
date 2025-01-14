use super::{Event, Focus};
use kartoffels_ui::{Button, Ui, UiWidget};
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Clone, Debug)]
pub enum SandboxTheme {
    Arena,
    Dungeon,
}

impl SandboxTheme {
    pub fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        Button::new(KeyCode::Char('t'), format!("theme: {val}"))
            .throwing(Event::FocusOn(Some(Focus::SandboxTheme)))
            .render(ui);
    }

    pub fn render_choice(ui: &mut Ui<Event>) {
        for val in SandboxTheme::all() {
            Button::new(val.key(), val.to_string())
                .throwing(Event::SetTheme(val))
                .render(ui);
        }
    }

    pub fn height() -> u16 {
        Self::all().count() as u16
    }

    fn all() -> impl Iterator<Item = Self> {
        [Self::Arena, Self::Dungeon].into_iter()
    }

    fn key(&self) -> KeyCode {
        KeyCode::Char(match self {
            Self::Arena => 'a',
            Self::Dungeon => 'd',
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
                Self::Dungeon => "dungeon",
            }
        )
    }
}
