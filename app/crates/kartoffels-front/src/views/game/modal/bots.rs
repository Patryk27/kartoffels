use crate::views::game::Event as ParentEvent;
use crate::{theme, BotIdExt, Button, LineEdit, Ui, VRow};
use kartoffels_world::prelude::{AliveBotSnapshot, BotId, Snapshot};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::{
    Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
};
use termwiz::input::{KeyCode, Modifiers};

#[derive(Debug, Default)]
pub struct BotsModal {
    offset: usize,
    selected: usize,
    focus: Focus,
    table: Table,
    filter: LineEdit,
}

impl BotsModal {
    pub fn render(&mut self, ui: &mut Ui<ParentEvent>, world: &Snapshot) {
        let width = Table::COLS.iter().copied().sum::<u16>() + 1;
        let height = ui.area.height - 2;

        let event = ui.catching(|ui| {
            ui.imodal(width, height, Some(" bots "), |ui| {
                let [body_area, _, footer_area] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Length(2),
                ])
                .areas(ui.area);

                ui.at(body_area, |ui| {
                    self.render_body(ui, world);
                });

                ui.at(footer_area, |ui| {
                    self.render_footer(ui);
                });
            });
        });

        if let Some(event) = event {
            self.handle(ui, event);
        }
    }

    fn render_body(&mut self, ui: &mut Ui<Event>, world: &Snapshot) {
        let [table_area, scroll_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(2)])
                .areas(ui.area);

        self.table.update(world, self.filter.value());

        let max_offset = self
            .table
            .rows
            .len()
            .checked_sub(table_area.height as usize);

        self.offset = self.offset.min(max_offset.unwrap_or(0));

        self.selected = self
            .selected
            .min(table_area.height as usize)
            .min(self.table.rows.len().saturating_sub(1));

        // ---

        ui.at(table_area, |ui| {
            ui.focused(self.focus == Focus::Table, |ui| {
                self.table.render(ui, world, self.offset, self.selected);
            });
        });

        if let Some(max_offset) = max_offset {
            ui.at(scroll_area, |ui| {
                let mut state =
                    ScrollbarState::new(max_offset).position(self.offset);

                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .render(ui.area, ui.buf, &mut state);
            });
        }
    }

    fn render_footer(&mut self, ui: &mut Ui<Event>) {
        match self.focus {
            Focus::Table => {
                self.render_footer_table(ui);
            }
            Focus::Filter => {
                self.render_footer_filter(ui);
            }
        }
    }

    fn render_footer_table(&mut self, ui: &mut Ui<Event>) {
        ui.row(|ui| {
            ui.btn("scroll-up", KeyCode::Char('w'), |btn| {
                btn.throwing(Event::ScrollUp)
            });

            ui.space(4);

            ui.btn("select-up", KeyCode::UpArrow, |btn| {
                btn.throwing(Event::SelectUp)
            });

            ui.btn("filter", KeyCode::Char('/'), |btn| {
                btn.throwing(Event::FocusOn(Focus::Filter)).right_aligned()
            });
        });

        ui.row(|ui| {
            ui.btn("scroll-down", KeyCode::Char('s'), |btn| {
                btn.throwing(Event::ScrollDown)
            });

            ui.space(2);

            ui.btn("select-down", KeyCode::DownArrow, |btn| {
                btn.throwing(Event::SelectDown)
            });

            ui.btn("close", KeyCode::Escape, |btn| {
                btn.throwing(Event::Parent(ParentEvent::CloseModal))
                    .right_aligned()
            });
        });
    }

    fn render_footer_filter(&mut self, ui: &mut Ui<Event>) {
        ui.row(|ui| {
            ui.span("filter:");
        });

        ui.row(|ui| {
            ui.add(&mut self.filter);

            ui.btn("accept", KeyCode::Enter, |btn| {
                btn.throwing(Event::FocusOn(Focus::Table)).right_aligned()
            });

            // Accepting via enter makes more sense, but add a hidden escape
            // binding just in case
            if ui.key(KeyCode::Escape, Modifiers::NONE) {
                ui.throw(Event::FocusOn(Focus::Table));
            }
        });
    }

    fn handle(&mut self, ui: &mut Ui<ParentEvent>, event: Event) {
        match event {
            Event::ScrollUp => {
                self.offset = self.offset.saturating_sub(8);
            }
            Event::ScrollDown => {
                self.offset = self.offset.saturating_add(8);
            }
            Event::SelectUp => {
                self.selected = self.selected.saturating_sub(1);
            }
            Event::SelectDown => {
                self.selected = self.selected.saturating_add(1);
            }
            Event::FocusOn(focus) => {
                self.focus = focus;
            }
            Event::Parent(event) => {
                ui.throw(event);
            }
        }
    }
}

enum Event {
    ScrollUp,
    ScrollDown,
    SelectUp,
    SelectDown,
    FocusOn(Focus),
    Parent(ParentEvent),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Focus {
    #[default]
    Table,
    Filter,
}

#[derive(Clone, Debug, Default)]
struct Table {
    rows: Vec<(usize, BotId)>,
    filter: String,
    version: u64,
}

impl Table {
    const COLS: &[u16] = &[
        5,                        // nth
        BotId::LENGTH as u16 + 1, // id
        7,                        // age
        6,                        // score
        16,                       // actions
    ];

    fn update(&mut self, world: &Snapshot, filter: &str) {
        if world.version == self.version && filter == self.filter {
            return;
        }

        self.rows.clear();

        self.rows.extend(
            world
                .bots
                .alive
                .iter_by_scores()
                .enumerate()
                .filter(move |(_, bot)| {
                    if filter.is_empty() {
                        true
                    } else {
                        bot.id.to_string().contains(filter)
                    }
                })
                .map(|(idx, bot)| (idx, bot.id)),
        );

        self.filter = filter.into();
        self.version = world.version;
    }

    fn render(
        &self,
        ui: &mut Ui<Event>,
        world: &Snapshot,
        offset: usize,
        selected: usize,
    ) {
        let [thead_area, tbody_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
                .areas(ui.area);

        ui.at(thead_area, |ui| {
            VRow::new(ui, Self::COLS)
                .col(Span::raw("nth"))
                .col(Span::raw("id"))
                .col(Span::raw("age"))
                .col(Span::raw("score"));
        });

        ui.at(tbody_area, |ui| {
            for (idx, ((nth, id), area)) in self
                .rows
                .iter()
                .skip(offset)
                .take(tbody_area.height as usize)
                .zip(ui.area.rows())
                .enumerate()
            {
                ui.focused(idx == selected, |ui| {
                    ui.at(area, |ui| {
                        let bot = world.bots.alive.get(*id).unwrap();

                        self.render_row(ui, *nth, bot);
                    });
                });
            }
        });
    }

    fn render_row(
        &self,
        ui: &mut Ui<Event>,
        nth: usize,
        bot: &AliveBotSnapshot,
    ) {
        if ui.focused {
            ui.buf.set_style(ui.area, Style::new().bg(theme::DARK_GRAY));
        }

        let nth = Span::raw(format!("#{}", nth + 1));
        let id = Span::raw(bot.id.to_string()).fg(bot.id.color());
        let age = Span::raw(bot.age.as_time(6).to_string());
        let score = Span::raw(bot.score.to_string());

        let inspect = Button::new("inspect", KeyCode::Enter)
            .throwing(Event::Parent(ParentEvent::InspectBot { id: bot.id }));

        VRow::new(ui, Self::COLS)
            .col(nth)
            .col(id)
            .col(age)
            .col(score)
            .col(inspect);
    }
}
