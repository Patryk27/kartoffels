use super::{Event, Focus};
use kartoffels_ui::{Button, KeyCode, Ui, UiWidget};
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct BotCount(u8);

impl BotCount {
    pub(super) fn render_focus(ui: &mut Ui<Event>, val: &Self) {
        Button::new(KeyCode::Char('c'), format!("count: {val}"))
            .throwing(Event::FocusOn(Some(Focus::BotCount)))
            .render(ui);
    }

    pub(super) fn render_choice(ui: &mut Ui<Event>) {
        for n in 1..=10 {
            let key = KeyCode::Char((b'0' + (n % 10)) as char);
            let val = Self(n);

            Button::new(key, val.to_string())
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
