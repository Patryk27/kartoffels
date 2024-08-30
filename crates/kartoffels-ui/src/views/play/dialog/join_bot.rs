use super::DialogEvent;
use crate::{theme, Action, BlockExt, IntervalExt, LayoutExt, RectExt};
use kartoffels_world::prelude::{BotId, Update};
use ratatui::buffer::Buffer;
use ratatui::layout::{Layout, Offset, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Widget};
use termwiz::input::{InputEvent, KeyCode, Modifiers};
use tokio::time::{self, Interval};

#[derive(Debug)]
pub struct JoinBotDialog {
    pub id: String,
    pub caret_visible: bool,
    pub caret_interval: Interval,
}

impl JoinBotDialog {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let area = Block::dialog_info(
            Some(" joining bot "),
            Layout::dialog(26, 4, area),
            buf,
        );

        Line::raw("enter bot id:").render(area, buf);

        Line::from_iter([
            Span::raw("> "),
            Span::raw(&self.id),
            Span::raw(if self.caret_visible { "_" } else { "" })
                .fg(theme::GREEN),
        ])
        .render(area.offset(Offset { x: 0, y: 1 }), buf);

        Line::from(Action::new("esc", "cancel", true))
            .left_aligned()
            .render(area.footer(), buf);

        Line::from(Action::new("enter", "join", true))
            .right_aligned()
            .render(area.footer(), buf);
    }

    pub fn handle(
        &mut self,
        event: InputEvent,
        update: &Update,
    ) -> Option<DialogEvent> {
        match event {
            InputEvent::Key(event) => match (event.key, event.modifiers) {
                (KeyCode::Char(ch), Modifiers::NONE) => {
                    self.handle_insert(ch);
                }

                (KeyCode::Backspace, Modifiers::NONE) => {
                    self.id.pop();
                }

                (KeyCode::Enter, Modifiers::NONE) => {
                    return self.handle_confirm(update);
                }

                (KeyCode::Escape, Modifiers::NONE) => {
                    return Some(DialogEvent::Close);
                }

                _ => (),
            },

            InputEvent::Paste(payload) => {
                for ch in payload.chars() {
                    self.handle_insert(ch);
                }

                if self.id.len() == BotId::LENGTH {
                    return self.handle_confirm(update);
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

    fn handle_confirm(&self, update: &Update) -> Option<DialogEvent> {
        let id = self.id.trim();

        let Ok(id) = id.parse() else {
            return Some(DialogEvent::Throw(format!(
                "`{}` is not a valid bot id",
                self.id
            )));
        };

        if update.bots.by_id(id).is_none() {
            return Some(DialogEvent::Throw(format!(
                "bot `{}` was not found\n\nmaybe it's dead?",
                id
            )));
        }

        Some(DialogEvent::JoinBot(id))
    }

    pub async fn tick(&mut self) {
        self.caret_interval.tick().await;
        self.caret_visible = !self.caret_visible;
    }
}

impl Default for JoinBotDialog {
    fn default() -> Self {
        Self {
            id: Default::default(),
            caret_visible: true,
            caret_interval: time::interval(theme::CARET_INTERVAL)
                .skipping_first(),
        }
    }
}
