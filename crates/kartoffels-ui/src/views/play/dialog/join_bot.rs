use super::DialogResponse;
use crate::{theme, Button, Ui};
use kartoffels_world::prelude::{BotId, Snapshot};
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
        world: &Snapshot,
    ) -> Option<DialogResponse> {
        if ui.poll_interval(&mut self.caret_interval) {
            self.caret_visible = !self.caret_visible;
        }

        let mut response = None;

        ui.info_dialog(26, 4, Some(" joining bot "), |ui| {
            Line::raw("enter bot id:").render(ui.area(), ui.buf());

            ui.space(1);

            Line::from_iter([
                Span::raw("> "),
                Span::raw(&self.id),
                Span::raw(if self.caret_visible { "_" } else { "" })
                    .fg(theme::GREEN),
            ])
            .render(ui.area(), ui.buf());

            ui.space(2);

            if let Some(event) = ui.event() {
                response = self.handle(event, world);
            }

            if Button::new(KeyCode::Escape, "cancel").render(ui).pressed {
                response = Some(DialogResponse::Close);
            }

            if Button::new(KeyCode::Enter, "join")
                .right()
                .render(ui)
                .pressed
            {
                response = self.handle_confirm(world);
            }
        });

        response
    }

    fn handle(
        &mut self,
        event: &InputEvent,
        world: &Snapshot,
    ) -> Option<DialogResponse> {
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
                    return self.handle_confirm(world);
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

    fn handle_confirm(&self, world: &Snapshot) -> Option<DialogResponse> {
        let id = self.id.trim();

        let Ok(id) = id.parse() else {
            return Some(DialogResponse::Throw(format!(
                "`{}` is not a valid bot id",
                self.id
            )));
        };

        if world.bots.by_id(id).is_none() {
            return Some(DialogResponse::Throw(format!(
                "bot `{}` was not found\n\nmaybe it's dead?",
                id
            )));
        }

        Some(DialogResponse::JoinBot(id))
    }
}

impl Default for JoinBotDialog {
    fn default() -> Self {
        Self {
            id: Default::default(),
            caret_visible: false,
            caret_interval: time::interval(theme::CARET_TIME),
        }
    }
}
