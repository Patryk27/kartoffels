use super::Modal;
use crate::views::game::Event as ParentEvent;
use chrono::Utc;
use kartoffels_ui::{theme, Button, KeyCode, Ui, UiWidget};
use kartoffels_world::cfg;
use kartoffels_world::prelude::{BotId, BotSnapshot, Snapshot};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::widgets::{Cell, Row, Table};
use std::fmt;

pub struct InspectBotModal {
    id: BotId,
    tab: Tab,
    parent: Option<Box<Modal>>,
}

impl InspectBotModal {
    pub fn new(id: BotId, parent: Option<Box<Modal>>) -> Self {
        Self {
            id,
            tab: Default::default(),
            parent,
        }
    }

    pub fn render(&mut self, ui: &mut Ui<ParentEvent>, world: &Snapshot) {
        let event = ui.catch(|ui| {
            let width = ui.area.width - 8;
            let height = ui.area.height - 4;
            let title = format!(" bots › {} ", self.id);

            ui.info_window(width, height, Some(&title), |ui| {
                let [body_area, _, footer_area] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .areas(ui.area);

                ui.clamp(body_area, |ui| {
                    self.render_body(ui, world);
                });

                ui.clamp(footer_area, |ui| {
                    self.render_footer(ui);
                });
            });
        });

        if let Some(event) = event
            && let Some(event) = self.handle(event)
        {
            ui.throw(event);
        }
    }

    fn render_body(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        match self.tab {
            Tab::Stats => {
                self.render_body_stats(ui, world);
            }
            Tab::Events => {
                self.render_body_events(ui, world);
            }
            Tab::Runs => {
                self.render_body_runs(ui, world);
            }
        }
    }

    fn render_body_stats(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        if let Some(BotSnapshot::Alive(bot)) = world.bots.get(self.id) {
            ui.line(format!("age = {} vcpu-cycles", bot.age));
            ui.line(format!("    = {}s", bot.age_seconds()));
            ui.space(1);
        }

        let Some(stats) = world.stats.get(self.id) else {
            return;
        };

        ui.line(format!("sum(scores) = {}", stats.scores_sum));
        ui.line(format!("len(scores) = {}", stats.scores_len));
        ui.line(format!("avg(scores) = {:.2}", stats.scores_avg));
        ui.line(format!("max(scores) = {}", stats.scores_max));

        if stats.scores_len >= (cfg::MAX_RUNS_PER_BOT as u32) {
            ui.line("");

            ui.line(format!(
                "note: only {} recent scores are saved - overall, your bot has \
                 been alive for {} rounds",
                cfg::MAX_RUNS_PER_BOT,
                world.runs.len(self.id),
            ));
        }
    }

    fn render_body_events(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        let events = world.bots.get(self.id).map(|bot| match bot {
            BotSnapshot::Alive(bot) => &bot.events,
            BotSnapshot::Dead(bot) => &bot.events,
            BotSnapshot::Queued(bot) => &bot.events,
        });

        let rows =
            events
                .into_iter()
                .flat_map(|event| event.iter())
                .map(|event| {
                    let date = event
                        .at
                        .format(theme::DATETIME_FORMAT)
                        .to_string()
                        .fg(theme::GRAY);

                    Row::new(vec![
                        Cell::new(date),
                        Cell::new(event.msg.to_string()),
                    ])
                });

        let widths = vec![
            Constraint::Length(theme::DATETIME_LENGTH),
            Constraint::Fill(1),
        ];

        let header =
            Row::new(vec![Cell::new("at"), Cell::new("message")]).underlined();

        Table::new(rows, widths).header(header).render(ui);
    }

    // TODO support custom sorting
    fn render_body_runs(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        let rows = world.runs.iter(self.id).map(|run| {
            let spawned_at = run
                .spawned_at
                .format(theme::DATETIME_FORMAT)
                .to_string()
                .fg(theme::GRAY);

            let killed_at = run
                .killed_at
                .map(|at| at.format(theme::DATETIME_FORMAT).to_string())
                .unwrap_or_else(|| "-".into())
                .fg(theme::GRAY);

            let age = run
                .killed_at
                .unwrap_or_else(Utc::now)
                .signed_duration_since(run.spawned_at);

            // TODO support minutes
            let age = format!("{}s", age.num_seconds());

            let score = run.score.to_string();

            Row::new(vec![
                Cell::new(spawned_at),
                Cell::new(killed_at),
                Cell::new(age),
                Cell::new(score),
            ])
        });

        let widths = vec![
            Constraint::Length(theme::DATETIME_LENGTH),
            Constraint::Length(theme::DATETIME_LENGTH),
            Constraint::Length(7),
            Constraint::Length(5),
        ];

        let header = Row::new(vec![
            Cell::new("spawned-at"),
            Cell::new("killed-at"),
            Cell::new("age"),
            Cell::new("score"),
        ])
        .underlined();

        Table::new(rows, widths).header(header).render(ui);
    }

    fn render_footer(&self, ui: &mut Ui<Event>) {
        ui.row(|ui| {
            for (idx, tab) in Tab::all().enumerate() {
                if idx > 0 {
                    ui.span(" • ");
                }

                ui.render(if self.tab == tab {
                    tab.btn().bold()
                } else {
                    tab.btn()
                });
            }

            Button::new(KeyCode::Escape, "close")
                .throwing(Event::GoBack)
                .right_aligned()
                .render(ui);
        });
    }

    fn handle(&mut self, event: Event) -> Option<ParentEvent> {
        match event {
            Event::GoBack => {
                if let Some(modal) = self.parent.take() {
                    Some(ParentEvent::OpenModal { modal })
                } else {
                    Some(ParentEvent::CloseModal)
                }
            }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Stats,
    Events,
    Runs,
}

impl Tab {
    fn all() -> impl Iterator<Item = Self> {
        [Self::Stats, Self::Events, Self::Runs].into_iter()
    }

    fn btn(&self) -> Button<Event> {
        let btn = match self {
            Tab::Stats => Button::new(KeyCode::Char('s'), "stats"),
            Tab::Events => Button::new(KeyCode::Char('e'), "events"),
            Tab::Runs => Button::new(KeyCode::Char('r'), "runs"),
        };

        btn.throwing(Event::ChangeTab(*self))
    }
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stats => write!(f, "stats"),
            Self::Events => write!(f, "events"),
            Self::Runs => write!(f, "runs"),
        }
    }
}
