use super::{Event, Focus};
use crate::Ui;
use termwiz::input::KeyCode;

#[derive(Clone, Copy, Debug, Default)]
pub enum BotPosition {
    Manual,
    #[default]
    Random,
}

impl BotPosition {
    pub(super) fn render_btn(ui: &mut Ui<Event>, this: &Self) {
        ui.btn(
            format!("position: {}", this.label()),
            KeyCode::Char('p'),
            |btn| btn.throwing(Event::FocusOn(Some(Focus::BotPosition))),
        );
    }

    pub(super) fn render_form(ui: &mut Ui<Event>) {
        for (idx, this) in Self::all().enumerate() {
            if idx > 0 {
                ui.space(1);
            }

            ui.btn(this.label(), this.key(), |btn| {
                btn.help(this.help()).throwing(Event::SetBotPosition(this))
            });
        }
    }

    pub fn height() -> u16 {
        (Self::all().count() * 3 - 1) as u16
    }

    fn all() -> impl Iterator<Item = Self> {
        [Self::Manual, Self::Random].into_iter()
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Random => "random",
        }
    }

    fn key(&self) -> KeyCode {
        KeyCode::Char(match self {
            Self::Manual => 'm',
            Self::Random => 'r',
        })
    }

    fn help(&self) -> &'static str {
        match self {
            Self::Manual => "use mouse to place your bot wherever you like",
            Self::Random => "it's, well, random",
        }
    }
}
