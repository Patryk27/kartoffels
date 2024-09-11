use crate::views::game::Event;
use kartoffels_ui::{Button, Caret, Ui};
use kartoffels_world::prelude::{BotId, Snapshot};
use ratatui::text::{Line, Span};
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug, Default)]
pub struct JoinBotDialog {
    id: String,
    caret: Caret,
}

impl JoinBotDialog {
    pub fn render(&mut self, ui: &mut Ui, world: &Snapshot) {
        ui.info_window(26, 4, Some(" joining bot "), |ui| {
            ui.line("enter bot id:");

            ui.line(Line::from_iter([
                Span::raw("> "),
                Span::raw(&self.id),
                self.caret.as_span(),
            ]));

            ui.space(1);

            // TODO avoid cloning
            if let Some(event) = ui.event().cloned() {
                self.handle(ui, world, &event);
            }

            ui.row(|ui| {
                Button::new(KeyCode::Escape, "cancel")
                    .throwing(Event::CloseDialog)
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

    fn handle(&mut self, ui: &mut Ui, world: &Snapshot, event: &InputEvent) {
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
                    self.handle_confirm(ui, world);
                }
            }

            _ => (),
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

    fn handle_confirm(&self, ui: &mut Ui, world: &Snapshot) {
        let id = self.id.trim();

        let Ok(id) = id.parse() else {
            ui.throw(Event::ShowErrorDialog(format!(
                "`{}` is not a valid bot id",
                self.id
            )));

            return;
        };

        if world.bots().by_id(id).is_none() {
            ui.throw(Event::ShowErrorDialog(format!(
                "bot `{}` was not found\n\nmaybe it's dead?",
                id
            )));

            return;
        }

        ui.throw(Event::JoinBot(id));
    }
}
