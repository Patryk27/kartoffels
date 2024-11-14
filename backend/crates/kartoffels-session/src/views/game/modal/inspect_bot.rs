use crate::views::game::Event;
use chrono::Utc;
use kartoffels_ui::{theme, Button, RectExt, Render, Ui};
use kartoffels_world::prelude::{BotId, Snapshot, SnapshotBot};
use ratatui::style::Stylize;
use termwiz::input::KeyCode;

#[derive(Clone, Debug)]
pub struct InspectBotModal {
    id: BotId,
}

impl InspectBotModal {
    pub fn new(id: BotId) -> Self {
        Self { id }
    }

    pub fn render(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        let now = Utc::now();
        let width = ui.area.width - 8;
        let height = ui.area.height - 4;
        let title = format!(" bot {} ", self.id);

        let events = world.bots().get(self.id).map(|bot| match bot {
            SnapshotBot::Alive(bot) => &bot.events,
            SnapshotBot::Dead(bot) => &bot.events,
            SnapshotBot::Queued(bot) => &bot.events,
        });

        ui.info_window(width, height, Some(&title), |ui| {
            for event in events.into_iter().flat_map(|e| e.iter()) {
                if ui.area.height <= 1 {
                    break;
                }

                ui.row(|ui| {
                    let date = if event.at.date_naive() == now.date_naive() {
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

            ui.clamp(ui.area.footer(1), |ui| {
                Button::new(KeyCode::Escape, "close")
                    .throwing(Event::CloseModal)
                    .right_aligned()
                    .render(ui);
            });
        });
    }
}
