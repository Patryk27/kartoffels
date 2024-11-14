use crate::views::game::Event;
use kartoffels_ui::{Button, Caret, Render, Ui};
use kartoffels_world::prelude::{BotId, Snapshot};
use ratatui::text::{Line, Span};
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug, Default)]
pub struct JoinBotModal {
    id: String,
    caret: Caret,
}

impl JoinBotModal {
    pub fn render(&mut self, ui: &mut Ui<Event>, world: &Snapshot) {
        ui.info_window(26, 4, Some(" join-bot "), |ui| {
            ui.line("enter bot id:");

            ui.line(Line::from_iter([
                Span::raw("> "),
                Span::raw(&self.id),
                self.caret.as_span(),
            ]));

            ui.space(1);

            if let Some(event) = ui.event {
                self.handle(ui, world, event);
            }

            ui.row(|ui| {
                Button::new(KeyCode::Escape, "cancel")
                    .throwing(Event::CloseModal)
                    .render(ui);

                if Button::new(KeyCode::Enter, "join")
                    .right_aligned()
                    .render(ui)
                    .pressed
                {
                    self.handle_confirm(ui, world);
                }
            });
        });
    }

    fn handle(
        &mut self,
        ui: &mut Ui<Event>,
        world: &Snapshot,
        event: &InputEvent,
    ) {
        match event {
            InputEvent::Key(event) => match (event.key, event.modifiers) {
                (KeyCode::Char(ch), Modifiers::NONE) => {
                    self.handle_insert(ch);
                }

                (KeyCode::Backspace, Modifiers::NONE) => {
                    self.id.pop();
                }

                _ => {}
            },

            InputEvent::Paste(payload) => {
                for ch in payload.chars() {
                    self.handle_insert(ch);
                }

                if self.id.len() == BotId::LENGTH {
                    self.handle_confirm(ui, world);
                }
            }

            _ => {}
        }
    }

    fn handle_insert(&mut self, ch: char) {
        if self.id.len() >= BotId::LENGTH {
            return;
        }

        if ch.is_alphanumeric() || ch == '-' {
            self.id.push(ch);
        }
    }

    fn handle_confirm(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        let id = self.id.trim();

        let Ok(id) = id.parse() else {
            ui.throw(Event::OpenErrorModal {
                error: format!("`{}` is not a valid bot id", self.id),
            });

            return;
        };

        if !world.bots().has(id) {
            ui.throw(Event::OpenErrorModal {
                error: format!("bot `{id}` was not found"),
            });

            return;
        }

        ui.throw(Event::JoinBot { id });
    }
}
