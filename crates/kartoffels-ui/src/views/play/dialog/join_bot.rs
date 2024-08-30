use super::DialogEvent;
use crate::{theme, Button, Ui};
use kartoffels_world::prelude::{BotId, Snapshot};
use ratatui::layout::Offset;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use termwiz::input::{InputEvent, KeyCode, Modifiers};
use tokio::time::{self, Interval};

#[derive(Debug)]
pub struct JoinBotDialog {
    pub id: String,
    pub caret_visible: bool,
    pub caret_interval: Interval,
}

impl JoinBotDialog {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        snapshot: &Snapshot,
    ) -> Option<DialogEvent> {
        if ui.poll(self.caret_interval.tick()).is_ready() {
            self.caret_visible = !self.caret_visible;

            _ = ui.poll(self.caret_interval.tick());
        }

        ui.info_dialog(26, 4, Some(" joining bot "), |ui| {
            let mut event = None;

            Line::raw("enter bot id:").render(ui.area(), ui.buf());

            Line::from_iter([
                Span::raw("> "),
                Span::raw(&self.id),
                Span::raw(if self.caret_visible { "_" } else { "" })
                    .fg(theme::GREEN),
            ])
            .render(ui.area().offset(Offset { x: 0, y: 1 }), ui.buf());

            if let Some(ui_event) = ui.event() {
                event = self.handle(ui_event, snapshot);
            }

            if Button::new(KeyCode::Escape, "cancel").render(ui).activated {
                event = Some(DialogEvent::Close);
            }

            if Button::new(KeyCode::Enter, "join").render(ui).activated {
                event = self.handle_confirm(snapshot);
            }

            event
        })
    }

    fn handle(
        &mut self,
        event: &InputEvent,
        snapshot: &Snapshot,
    ) -> Option<DialogEvent> {
        match event {
            InputEvent::Key(event) => match (event.key, event.modifiers) {
                (KeyCode::Char(ch), Modifiers::NONE) => {
                    self.handle_insert(ch);
                }

                (KeyCode::Backspace, Modifiers::NONE) => {
                    self.id.pop();
                }

                _ => (),
            },

            InputEvent::Paste(payload) => {
                for ch in payload.chars() {
                    self.handle_insert(ch);
                }

                if self.id.len() == BotId::LENGTH {
                    return self.handle_confirm(snapshot);
                }
            }

            _ => (),
        }

        None
    }

    fn handle_insert(&mut self, ch: char) {
        if self.id.len() >= BotId::LENGTH {
            return;
        }

        if ch.is_alphanumeric() || ch == '-' {
            self.id.push(ch);
        }
    }

    fn handle_confirm(&self, snapshot: &Snapshot) -> Option<DialogEvent> {
        let id = self.id.trim();

        let Ok(id) = id.parse() else {
            return Some(DialogEvent::Throw(format!(
                "`{}` is not a valid bot id",
                self.id
            )));
        };

        if snapshot.bots.by_id(id).is_none() {
            return Some(DialogEvent::Throw(format!(
                "bot `{}` was not found\n\nmaybe it's dead?",
                id
            )));
        }

        Some(DialogEvent::JoinBot(id))
    }
}

impl Default for JoinBotDialog {
    fn default() -> Self {
        Self {
            id: Default::default(),
            caret_visible: false,
            caret_interval: time::interval(theme::CARET_INTERVAL),
        }
    }
}
