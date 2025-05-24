use super::{Event, Focus};
use crate::Ui;
use kartoffels_prefabs::{DUMMY, ROBERTO};
use termwiz::input::KeyCode;

#[derive(Clone, Copy, Debug)]
pub enum BotSource {
    Upload,
    Prefab(BotPrefab),
}

impl BotSource {
    pub(super) fn render_btn(ui: &mut Ui<Event>, this: &Self) {
        ui.btn(
            format!("source: {}", this.label()),
            KeyCode::Char('s'),
            |btn| btn.throwing(Event::FocusOn(Some(Focus::BotSource))),
        );
    }

    pub(super) fn render_form(ui: &mut Ui<Event>) {
        for (idx, this) in Self::all().enumerate() {
            if idx > 0 {
                ui.space(1);
            }

            ui.btn(this.label(), this.key(), |btn| {
                btn.help(this.help()).throwing(Event::SetBotSource(this))
            });
        }
    }

    pub(super) fn height() -> u16 {
        (Self::all().count() * 3 - 1) as u16
    }

    fn all() -> impl Iterator<Item = Self> {
        [
            Self::Upload,
            Self::Prefab(BotPrefab::Dummy),
            Self::Prefab(BotPrefab::Roberto),
        ]
        .into_iter()
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Upload => "upload",
            Self::Prefab(BotPrefab::Dummy) => "prefab.dummy",
            Self::Prefab(BotPrefab::Roberto) => "prefab.roberto",
        }
    }

    fn key(&self) -> KeyCode {
        KeyCode::Char(match self {
            Self::Upload => 'u',
            Self::Prefab(BotPrefab::Dummy) => 'd',
            Self::Prefab(BotPrefab::Roberto) => 'r',
        })
    }

    fn help(&self) -> &'static str {
        match self {
            Self::Upload => "upload your own bot",
            Self::Prefab(BotPrefab::Dummy) => {
                "the most simplest bot, does literally nothing"
            }
            Self::Prefab(BotPrefab::Roberto) => {
                "moderately challenging bot, likes to stab"
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BotPrefab {
    Dummy,
    Roberto,
}

impl BotPrefab {
    pub fn source(&self) -> Vec<u8> {
        match self {
            Self::Dummy => DUMMY,
            Self::Roberto => ROBERTO,
        }
        .to_vec()
    }
}
