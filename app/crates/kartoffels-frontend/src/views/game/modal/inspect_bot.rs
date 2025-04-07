use super::Modal;
use crate::views::game::Event as ParentEvent;
use kartoffels_ui::{theme, Button, KeyCode, Ui, UiWidget};
use kartoffels_world::cfg;
use kartoffels_world::prelude::{BotId, BotSnapshot, Snapshot};
use ordinal::Ordinal;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::symbols;
use ratatui::widgets::{Axis, Cell, Chart, Dataset, GraphType, Row, Table};
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
            Tab::Lives => {
                self.render_body_lives(ui, world);
            }
        }
    }

    fn render_body_stats(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        let Some(stats) = world.stats.get(self.id) else {
            return;
        };

        let Some(lives) = world.lives.get(self.id) else {
            return;
        };

        // ---

        let [col1, col2, col3] =
            Layout::horizontal([Constraint::Fill(1); 3]).areas(ui.area);

        ui.clamp(col1, |ui| {
            ui.line(format!("curr-life = #{}", world.lives.len(self.id)));
            ui.space(1);

            match world.bots.get(self.id) {
                Some(BotSnapshot::Alive(bot)) => {
                    ui.line(format!("age = {} ticks", bot.age.as_ticks()));
                    ui.line(format!("    = {}", bot.age.as_time(None)));
                }

                Some(BotSnapshot::Queued(bot)) => {
                    ui.line(format!(
                        "status = queued ({})",
                        Ordinal(bot.place)
                    ));
                }

                Some(BotSnapshot::Dead(_)) => {
                    ui.line("status = dead");
                }

                None => (),
            }
        });

        ui.clamp(col2, |ui| {
            ui.line(format!("avg(ages) = {:.2}s", stats.ages.avg));
            ui.line(format!("sum(ages) = {}s", stats.ages.sum));
            ui.line(format!("min(ages) = {}s", stats.ages.min));
            ui.line(format!("max(ages) = {}s", stats.ages.max));
        });

        ui.clamp(col3, |ui| {
            ui.line(format!("avg(scores) = {:.2}", stats.scores.avg));
            ui.line(format!("sum(scores) = {}", stats.scores.sum));
            ui.line(format!("min(scores) = {}", stats.scores.min));
            ui.line(format!("max(scores) = {}", stats.scores.max));
        });

        ui.space(5);

        if stats.lives >= (cfg::MAX_LIVES_PER_BOT as u32) {
            ui.line(format!(
                "note: this machine has gone through {} lives, showing only \
                 the recent {} ones",
                world.lives.len(self.id),
                cfg::MAX_LIVES_PER_BOT,
            ));

            ui.space(1);
        }

        // ---

        let dataset: Vec<_> = lives
            .iter()
            .rev()
            .enumerate()
            .map(|(idx, life)| (idx as f64, life.score as f64))
            .collect();

        let dataset = Dataset::default()
            .name("scores")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme::YELLOW))
            .data(&dataset);

        let x_axis = Axis::default()
            .title("life".fg(theme::GRAY))
            .style(Style::default().white())
            .bounds([0.0, lives.len() as f64])
            .labels(["past-lives".fg(theme::GRAY), "curr-life".fg(theme::GRAY)])
            .labels_alignment(Alignment::Right);

        let y_axis = Axis::default()
            .title("score".fg(theme::GRAY))
            .style(Style::default().white())
            .bounds([0.0, stats.scores.max as f64])
            .labels([
                "0".fg(theme::GRAY),
                stats.scores.max.to_string().fg(theme::GRAY),
            ]);

        Chart::new(vec![dataset])
            .x_axis(x_axis)
            .y_axis(y_axis)
            .style(Style::default().bg(theme::BG))
            .legend_position(None)
            .render(ui);
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
    fn render_body_lives(&self, ui: &mut Ui<Event>, world: &Snapshot) {
        let age = world
            .bots
            .alive
            .get(self.id)
            .map(|bot| bot.age)
            .unwrap_or_default();

        let rows = world.lives.iter(self.id).map(|life| {
            let born_at = life
                .born_at
                .format(theme::DATETIME_FORMAT)
                .to_string()
                .fg(theme::GRAY);

            let died_at = life
                .died_at
                .map(|at| at.format(theme::DATETIME_FORMAT).to_string())
                .unwrap_or_else(|| "-".into())
                .fg(theme::GRAY);

            let age = life.age.unwrap_or(age);

            Row::new(vec![
                Cell::new(born_at),
                Cell::new(died_at),
                Cell::new(age.as_time(None).to_string()),
                Cell::new(life.score.to_string()),
            ])
        });

        let widths = vec![
            Constraint::Length(theme::DATETIME_LENGTH),
            Constraint::Length(theme::DATETIME_LENGTH),
            Constraint::Length(7),
            Constraint::Length(5),
        ];

        let header = Row::new(vec![
            Cell::new("born-at"),
            Cell::new("died-at"),
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

                ui.add(if self.tab == tab {
                    tab.btn().bold()
                } else {
                    tab.btn()
                });
            }

            let join =
                Button::new("join", KeyCode::Enter).throwing(Event::JoinBot);

            let close =
                Button::new("close", KeyCode::Escape).throwing(Event::GoBack);

            let [_, join_area, _, close_area] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(join.width()),
                Constraint::Length(2),
                Constraint::Length(close.width()),
            ])
            .areas(ui.area);

            ui.add_at(join_area, join);
            ui.add_at(close_area, close);
        });
    }

    fn handle(&mut self, event: Event) -> Option<ParentEvent> {
        match event {
            Event::ChangeTab(tab) => {
                self.tab = tab;
                None
            }

            Event::JoinBot => Some(ParentEvent::JoinBot { id: self.id }),

            Event::GoBack => {
                if let Some(modal) = self.parent.take() {
                    Some(ParentEvent::OpenModal { modal })
                } else {
                    Some(ParentEvent::CloseModal)
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Event {
    ChangeTab(Tab),
    JoinBot,
    GoBack,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Stats,
    Events,
    Lives,
}

impl Tab {
    fn all() -> impl Iterator<Item = Self> {
        [Self::Stats, Self::Events, Self::Lives].into_iter()
    }

    fn btn(&self) -> Button<Event> {
        let btn = match self {
            Tab::Stats => Button::new("stats", KeyCode::Char('s')),
            Tab::Events => Button::new("events", KeyCode::Char('e')),
            Tab::Lives => Button::new("lives", KeyCode::Char('l')),
        };

        btn.throwing(Event::ChangeTab(*self))
    }
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stats => write!(f, "stats"),
            Self::Events => write!(f, "events"),
            Self::Lives => write!(f, "lives"),
        }
    }
}
