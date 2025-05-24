use super::{Event, Focus};
use crate::Ui;
use termwiz::input::KeyCode;

#[derive(Clone, Debug)]
pub enum SandboxSize {
    Tiny,
    Small,
    Medium,
    Large,
}

impl SandboxSize {
    pub fn render_btn(ui: &mut Ui<Event>, this: &Self) {
        ui.btn(
            format!("size: {}", this.label()),
            KeyCode::Char('s'),
            |btn| btn.throwing(Event::FocusOn(Some(Focus::SandboxSize))),
        );
    }

    pub fn render_form(ui: &mut Ui<Event>) {
        for (idx, this) in Self::all().enumerate() {
            if idx > 0 {
                ui.space(1);
            }

            ui.btn(this.label(), this.key(), |btn| {
                btn.help(this.help()).throwing(Event::SetSize(this))
            });
        }
    }

    pub fn height() -> u16 {
        3 * Self::all().count() as u16 - 1
    }

    fn all() -> impl Iterator<Item = Self> {
        [Self::Tiny, Self::Small, Self::Medium, Self::Large].into_iter()
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Tiny => "tiny",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }

    fn key(&self) -> KeyCode {
        KeyCode::Char(match self {
            Self::Tiny => 't',
            Self::Small => 's',
            Self::Medium => 'm',
            Self::Large => 'l',
        })
    }

    fn help(&self) -> &'static str {
        match self {
            Self::Tiny => "crowdy",
            Self::Small => "less crowdy",
            Self::Medium => "even less crowdy",
            Self::Large => "whoooa man, careful",
        }
    }
}
