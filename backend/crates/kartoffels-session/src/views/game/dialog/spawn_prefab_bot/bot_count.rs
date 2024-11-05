use super::{Event, Focus};
use kartoffels_ui::{Button, Render, Ui};
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct BotCount(u8);

impl BotCount {
    pub fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        Button::new(KeyCode::Char('c'), format!("count: {val}"))
            .throwing(Event::FocusOn(Some(Focus::BotCount)))
            .render(ui);
    }

    pub fn render_choice(ui: &mut Ui<Event>) {
        for n in 1..=9 {
            let key = KeyCode::Char((b'0' + n) as char);
            let label = Self(n).to_string();

            Button::new(key, label)
                .throwing(Event::SetBotCount(Self(n)))
                .render(ui);
        }
    }

    pub fn height() -> u16 {
        9
    }
}

impl Default for BotCount {
    fn default() -> Self {
        Self(1)
    }
}

impl fmt::Display for BotCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
