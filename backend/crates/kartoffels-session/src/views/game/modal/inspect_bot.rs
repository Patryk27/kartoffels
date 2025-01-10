use crate::views::game::Event as ParentEvent;
use chrono::Utc;
use kartoffels_ui::{theme, Button, RectExt, Render, Ui};
use kartoffels_world::prelude::{BotId, BotSnapshot, Snapshot};
use ratatui::style::Stylize;
use std::fmt;
use termwiz::input::KeyCode;

#[derive(Clone, Debug)]
pub struct InspectBotModal {
    id: BotId,
    tab: Tab,
}

impl InspectBotModal {
    pub fn new(id: BotId) -> Self {
        Self {
            id,
            tab: Tab::Events,
        }
    }

    pub fn render(&mut self, ui: &mut Ui<ParentEvent>, world: &Snapshot) {
        let event = ui.catch(|ui| {
            let width = ui.area.width - 8;
            let height = ui.area.height - 4;
            let title = format!(" bots › {} › {} ", self.id, self.tab);

            ui.info_window(width, height, Some(&title), |ui| {
                ui.clamp(ui.area.header(1), |ui| {
                    self.render_header(ui);
                });

                ui.space(2);

                match self.tab {
                    Tab::Events => {
                        self.render_body_events(ui, world);
                    }
                    Tab::Runs => {
                        self.render_body_runs(ui, world);
                    }
                }

                ui.clamp(ui.area.footer(1), |ui| {
                    self.render_footer(ui);
                });
            });
        });

        if let Some(event) = event {
            if let Some(event) = self.handle(event) {
                ui.throw(event);
            }
        }
    }

    fn render_header(&self, ui: &mut Ui<Event>) {
        ui.row(|ui| {
            Tab::Events.btn().render(ui);

            ui.space(1);

            Tab::Runs.btn().render(ui);
        });
    }

    fn render_footer(&self, ui: &mut Ui<Event>) {
        Button::new(KeyCode::Escape, "close")
            .throwing(Event::GoBack)
            .right_aligned()
            .render(ui);
    }

    fn render_body_events(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        let now = Utc::now();

        let events = world.bots.get(self.id).map(|bot| match bot {
            BotSnapshot::Alive(bot) => &bot.events,
            BotSnapshot::Dead(bot) => &bot.events,
            BotSnapshot::Queued(bot) => &bot.events,
        });

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
    }

    fn render_body_runs(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        for run in world.runs.get(self.id) {
            if ui.area.height <= 1 {
                break;
            }

            ui.row(|ui| {
                ui.line(format!(
                    "{}, {:?}, {:?}",
                    run.score, run.spawned_at, run.killed_at
                ));
            });

            ui.space(1);
        }
    }

    fn handle(&mut self, event: Event) -> Option<ParentEvent> {
        match event {
            Event::GoBack => Some(ParentEvent::CloseModal),

            Event::ChangeTab(tab) => {
                self.tab = tab;
                None
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Event {
    GoBack,
    ChangeTab(Tab),
}

#[derive(Clone, Copy, Debug, Default)]
enum Tab {
    #[default]
    Events,
    Runs,
}

impl Tab {
    fn btn(&self) -> Button<Event> {
        match self {
            Tab::Events => Button::new(KeyCode::Char('e'), "events")
                .throwing(Event::ChangeTab(Self::Events)),

            Tab::Runs => Button::new(KeyCode::Char('r'), "runs")
                .throwing(Event::ChangeTab(Self::Runs)),
        }
    }
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Events => write!(f, "events"),
            Self::Runs => write!(f, "runs"),
        }
    }
}
