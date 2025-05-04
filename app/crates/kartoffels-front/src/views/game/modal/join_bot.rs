use crate::views::game::Event;
use crate::{Button, Input, Ui, UiWidget};
use anyhow::anyhow;
use kartoffels_world::prelude::{BotId, Snapshot};
use termwiz::input::{InputEvent, KeyCode};

#[derive(Debug, Default)]
pub struct JoinBotModal {
    id: Input,
}

impl JoinBotModal {
    pub fn render(&mut self, ui: &mut Ui<Event>, world: &Snapshot) {
        ui.info_window(26, 4, Some(" join-bot "), |ui| {
            ui.line("enter bot id:");
            ui.add(&mut self.id);
            ui.space(1);

            ui.row(|ui| {
                Button::new("cancel", KeyCode::Escape)
                    .throwing(Event::CloseModal)
                    .render(ui);

                if Button::new("join", KeyCode::Enter)
                    .right_aligned()
                    .render(ui)
                    .pressed
                {
                    self.handle_confirm(ui, world);
                }
            });

            if let Some(InputEvent::Paste(_)) = ui.event
                && self.id.value().len() == BotId::LENGTH
            {
                self.handle_confirm(ui, world);
            }
        });
    }

    fn handle_confirm(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        let id = self.id.value().trim();

        if id.is_empty() {
            ui.throw(Event::CloseModal);
            return;
        }

        let Ok(id) = id.parse() else {
            ui.throw(Event::OpenErrorModal {
                error: anyhow!("`{id}` is not a valid bot id")
                    .context("couldn't join bot"),
            });

            return;
        };

        if !world.bots.has(id) {
            ui.throw(Event::OpenErrorModal {
                error: anyhow!("bot `{id}` not found")
                    .context("couldn't join bot"),
            });

            return;
        }

        ui.throw(Event::JoinBot { id });
    }
}
