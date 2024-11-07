use super::{Event, Focus};
use itertools::Itertools;
use kartoffels_ui::{Button, Render, Ui};
use kartoffels_world::prelude::prefabs;
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Clone, Copy, Debug)]
pub struct BotPrefab {
    idx: usize,
}

impl BotPrefab {
    pub(super) fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        Button::new(KeyCode::Char('p'), format!("prefab:{val}"))
            .throwing(Event::FocusOn(Some(Focus::BotPrefab)))
            .render(ui);
    }

    pub(super) fn render_choice(ui: &mut Ui<Event>) {
        for (idx, prefab) in Self::all().enumerate() {
            let key = KeyCode::Char((b'1' + (idx as u8)) as char);
            let label = prefab.to_string();

            Button::new(key, label)
                .throwing(Event::SetBotPrefab(prefab))
                .render(ui);
        }
    }

    pub(super) fn height() -> u16 {
        Self::all().count() as u16
    }

    pub fn src(&self) -> &'static [u8] {
        prefabs::ALL[self.idx].source
    }

    fn all() -> impl Iterator<Item = Self> {
        prefabs::ALL.iter().enumerate().map(|(idx, _)| Self { idx })
    }
}

impl Default for BotPrefab {
    fn default() -> Self {
        let idx = prefabs::ALL
            .iter()
            .find_position(|prefab| prefab.name == "roberto")
            .unwrap()
            .0;

        Self { idx }
    }
}

impl fmt::Display for BotPrefab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", prefabs::ALL[self.idx].name)
    }
}
