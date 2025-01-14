use crate::views::game::Event as ParentEvent;
use chrono::Utc;
use kartoffels_ui::{theme, Button, RectExt, Ui, UiWidget};
use kartoffels_world::prelude::{BotId, BotSnapshot, Snapshot};
use ratatui::layout::Constraint;
use ratatui::style::Stylize;
use ratatui::widgets::{Cell, Row, Table};
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

                self.render_body(ui, world);

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
            for (idx, tab) in Tab::all().enumerate() {
                if idx > 0 {
                    ui.space(2);
                }

                ui.render(if self.tab == tab {
                    tab.btn().bold()
                } else {
                    tab.btn()
                });
            }
        });
    }

    fn render_body(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        match self.tab {
            Tab::Events => {
                self.render_body_events(ui, world);
            }
            Tab::Runs => {
                self.render_body_runs(ui, world);
            }
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
        let rows = world.runs.get(self.id).map(|run| {
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
        Button::new(KeyCode::Escape, "close")
            .throwing(Event::GoBack)
            .right_aligned()
            .render(ui);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Events,
    Runs,
}

impl Tab {
    fn all() -> impl Iterator<Item = Self> {
        [Self::Events, Self::Runs].into_iter()
    }

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
