use super::{Event, Focus};
use crate::Ui;
use kartoffels_prefabs::{DUMMY, ROBERTO};
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Clone, Copy, Debug)]
pub enum BotSource {
    Upload,
    Prefab(BotPrefabType),
}

impl BotSource {
    pub(super) fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        ui.btn(format!("source: {val}"), KeyCode::Char('s'), |btn| {
            btn.throwing(Event::FocusOn(Some(Focus::BotSource)))
        });
    }

    pub(super) fn render_choice(ui: &mut Ui<Event>) {
        for (idx, val) in Self::all().enumerate() {
            if idx > 0 {
                ui.space(1);
            }

            ui.btn(val.to_string(), val.key(), |btn| {
                btn.help(val.desc()).throwing(Event::SetBotSource(val))
            });
        }
    }

    pub(super) fn height() -> u16 {
        (Self::all().count() * 3 - 1) as u16
    }

    fn all() -> impl Iterator<Item = Self> {
        [
            Self::Upload,
            Self::Prefab(BotPrefabType::Dummy),
            Self::Prefab(BotPrefabType::Roberto),
        ]
        .into_iter()
    }

    fn key(&self) -> KeyCode {
        KeyCode::Char(match self {
            Self::Upload => 'u',
            Self::Prefab(BotPrefabType::Dummy) => 'd',
            Self::Prefab(BotPrefabType::Roberto) => 'r',
        })
    }

    fn desc(&self) -> &'static str {
        match self {
            Self::Upload => "upload your own bot",
            Self::Prefab(BotPrefabType::Dummy) => {
                "the most simplest bot, does literally nothing"
            }
            Self::Prefab(BotPrefabType::Roberto) => {
                "moderately challenging bot, likes to stab"
            }
        }
    }
}

impl fmt::Display for BotSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Upload => write!(f, "upload"),
            Self::Prefab(prefab) => write!(f, "prefab.{prefab}"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BotPrefabType {
    Dummy,
    Roberto,
}

impl fmt::Display for BotPrefabType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Dummy => write!(f, "dummy"),
            Self::Roberto => write!(f, "roberto"),
        }
    }
}

impl BotPrefabType {
    pub fn source(&self) -> Vec<u8> {
        match self {
            Self::Dummy => DUMMY,
            Self::Roberto => ROBERTO,
        }
        .to_vec()
    }
}
