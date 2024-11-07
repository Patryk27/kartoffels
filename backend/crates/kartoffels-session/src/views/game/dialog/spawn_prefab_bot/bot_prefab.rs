use super::{Event, Focus};
use kartoffels_ui::{theme, Button, Render, Ui};
use kartoffels_world::prelude::prefabs;
use ratatui::style::Stylize;
use ratatui::text::Text;
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Clone, Copy, Debug, Default)]
pub enum BotPrefab {
    Dummy,
    #[default]
    Roberto,
}

impl BotPrefab {
    pub(super) fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        Button::new(KeyCode::Char('p'), format!("prefab:{val}"))
            .throwing(Event::FocusOn(Some(Focus::BotPrefab)))
            .render(ui);
    }

    pub(super) fn render_choice(ui: &mut Ui<Event>) {
        for (idx, val) in Self::all().enumerate() {
            if idx > 0 {
                ui.space(1);
            }

            Button::new(val.key(), val.to_string())
                .throwing(Event::SetBotPrefab(val))
                .render(ui);

            ui.row(|ui| {
                ui.space(4);
                ui.line(Text::raw(val.desc()).fg(theme::GRAY));
            });

            ui.space(1);
        }
    }

    pub(super) fn height() -> u16 {
        (Self::all().count() * 3 - 1) as u16
    }

    fn all() -> impl Iterator<Item = Self> {
        [Self::Dummy, Self::Roberto].into_iter()
    }

    fn key(&self) -> KeyCode {
        KeyCode::Char(match self {
            Self::Dummy => 'd',
            Self::Roberto => 'r',
        })
    }

    fn desc(&self) -> &'static str {
        match self {
            Self::Dummy => "the most simplest bot, does literally nothing",
            Self::Roberto => "moderately challenging bot - likes to stab",
        }
    }

    pub fn source(&self) -> Vec<u8> {
        match self {
            Self::Dummy => prefabs::DUMMY,
            Self::Roberto => prefabs::ROBERTO,
        }
        .to_vec()
    }
}

impl fmt::Display for BotPrefab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Dummy => "dummy",
                Self::Roberto => "roberto",
            }
        )
    }
}
