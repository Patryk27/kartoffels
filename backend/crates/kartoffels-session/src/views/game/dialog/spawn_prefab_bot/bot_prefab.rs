use super::{Event, Focus};
use kartoffels_ui::{Button, Render, Ui};
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub enum BotPrefab {
    ChlAcyclicMaze,
    ChlFlightSyndromeEnemy,
    Dummy,
    #[default]
    Roberto,
}

impl BotPrefab {
    pub fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        Button::new(KeyCode::Char('p'), format!("prefab: {val}"))
            .throwing(Event::FocusOn(Some(Focus::BotPrefab)))
            .render(ui);
    }

    pub fn render_choice(ui: &mut Ui<Event>) {
        for (idx, prefab) in Self::all().enumerate() {
            let key = KeyCode::Char((b'1' + (idx as u8)) as char);
            let label = prefab.to_string();

            Button::new(key, label)
                .throwing(Event::SetBotPrefab(prefab))
                .render(ui);
        }
    }

    pub fn height() -> u16 {
        Self::all().count() as u16
    }

    fn all() -> impl Iterator<Item = Self> {
        [
            Self::ChlAcyclicMaze,
            Self::ChlFlightSyndromeEnemy,
            Self::Dummy,
            Self::Roberto,
        ]
        .into_iter()
    }
}

impl fmt::Display for BotPrefab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ChlAcyclicMaze => "chl-acyclic-maze",
                Self::ChlFlightSyndromeEnemy => "chl-flight-syndrome-enemy",
                Self::Dummy => "dummy",
                Self::Roberto => "roberto",
            }
        )
    }
}
