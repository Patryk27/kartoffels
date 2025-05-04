use super::{Event, Focus};
use crate::{Button, Ui, UiWidget};
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Clone, Copy, Debug)]
pub struct BotCount(u8);

impl BotCount {
    pub(super) fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        Button::new(format!("count: {val}"), KeyCode::Char('c'))
            .throwing(Event::FocusOn(Some(Focus::BotCount)))
            .render(ui);
    }

    pub(super) fn render_choice(ui: &mut Ui<Event>) {
        for n in 1..=10 {
            let val = Self(n);
            let key = KeyCode::Char((b'0' + (n % 10)) as char);

            Button::new(val.to_string(), key)
                .throwing(Event::SetBotCount(val))
                .render(ui);
        }
    }

    pub fn height() -> u16 {
        10
    }

    pub fn get(&self) -> u8 {
        self.0
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
