use crate::views::game::Event;
use chrono::Utc;
use kartoffels_ui::{theme, Button, RectExt, Render, Ui};
use kartoffels_world::prelude::{BotId, Snapshot};
use ratatui::style::Stylize;
use termwiz::input::KeyCode;

// TODO
#[derive(Clone, Debug)]
pub struct InspectBotDialog {
    id: BotId,
}

impl InspectBotDialog {
    pub fn new(id: BotId) -> Self {
        Self { id }
    }

    pub fn render(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        let now = Utc::now();
        let width = ui.area.width - 8;
        let height = ui.area.height - 4;
        let title = format!(" bot {} ", self.id);

        ui.info_window(width, height, Some(&title), |ui| {
            let events = if let Some(bot) = world.bots().alive().get(self.id) {
                Some(&bot.events)
            } else if let Some(bot) = world.bots().dead().get(self.id) {
                Some(&bot.events)
            } else if let Some(bot) = world.bots().queued().get(self.id) {
                Some(&bot.events)
            } else {
                None
            };

            if let Some(events) = events {
                for event in events.iter().take(16) {
                    ui.row(|ui| {
                        let date = if event.at.date_naive() == now.date_naive()
                        {
                            event.at.format("%H:%M:%S")
                        } else {
                            event.at.format("%Y-%m-%d %H:%M:%S")
                        };

                        ui.span(date.to_string().fg(theme::GRAY));
                        ui.span(" | ".fg(theme::GRAY));
                        ui.span(&event.msg);
                    });

                    ui.space(1);
                }
            }

            ui.clamp(ui.area.footer(1), |ui| {
                Button::new(KeyCode::Escape, "close")
                    .throwing(Event::CloseDialog)
                    .right_aligned()
                    .render(ui);
            });
        });
    }
}
