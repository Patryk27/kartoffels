use super::{Event, Focus};
use crate::Ui;
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Clone, Copy, Debug)]
pub struct BotCount(u8);

impl BotCount {
    pub(super) fn render_btn(ui: &mut Ui<Event>, this: &Self) {
        ui.btn(format!("count: {this}"), KeyCode::Char('c'), |btn| {
            btn.throwing(Event::FocusOn(Some(Focus::BotCount)))
        });
    }

    pub(super) fn render_form(ui: &mut Ui<Event>) {
        for n in 1..=10 {
            let this = Self(n);
            let key = KeyCode::Char((b'0' + (n % 10)) as char);

            ui.btn(this.to_string(), key, |btn| {
                btn.throwing(Event::SetBotCount(this))
            });
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
