use super::{Event, Focus};
use crate::Ui;
use termwiz::input::KeyCode;

#[derive(Clone, Debug)]
pub enum SandboxTheme {
    Arena,
    Cave,
}

impl SandboxTheme {
    pub fn render_btn(ui: &mut Ui<Event>, this: &Self) {
        ui.btn(
            format!("theme: {}", this.label()),
            KeyCode::Char('t'),
            |btn| btn.throwing(Event::FocusOn(Some(Focus::SandboxTheme))),
        );
    }

    pub fn render_form(ui: &mut Ui<Event>) {
        for (idx, this) in SandboxTheme::all().enumerate() {
            if idx > 0 {
                ui.space(1);
            }

            ui.btn(this.label(), this.key(), |btn| {
                btn.help(this.help()).throwing(Event::SetTheme(this))
            });
        }
    }

    pub fn height() -> u16 {
        3 * Self::all().count() as u16 - 1
    }

    fn all() -> impl Iterator<Item = Self> {
        [Self::Arena, Self::Cave].into_iter()
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Arena => "arena",
            Self::Cave => "cave",
        }
    }

    fn key(&self) -> KeyCode {
        KeyCode::Char(match self {
            Self::Arena => 'a',
            Self::Cave => 'c',
        })
    }

    fn help(&self) -> &'static str {
        match self {
            Self::Arena => "roundy boi, nowhere to hide",
            Self::Cave => "warped boi, plenty corners to hide in",
        }
    }
}
