use super::{Event, Focus};
use crate::{Button, Ui, UiWidget};
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
        Button::new(format!("source: {val}"), KeyCode::Char('s'))
            .throwing(Event::FocusOn(Some(Focus::BotSource)))
            .render(ui);
    }

    pub(super) fn render_choice(ui: &mut Ui<Event>) {
        for (idx, val) in Self::all().enumerate() {
            if idx > 0 {
                ui.space(1);
            }

            Button::new(val.to_string(), val.key())
                .help(val.desc())
                .throwing(Event::SetBotSource(val))
                .render(ui);
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
