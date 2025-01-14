use super::{Event, Focus};
use kartoffels_ui::{Button, Ui, UiWidget};
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
    pub fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        Button::new(KeyCode::Char('s'), format!("size: {val}"))
            .throwing(Event::FocusOn(Some(Focus::SandboxSize)))
            .render(ui);
    }

    pub fn render_choice(ui: &mut Ui<Event>) {
        for val in Self::all() {
            Button::new(val.key(), val.to_string())
                .throwing(Event::SetSize(val))
                .render(ui);
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
