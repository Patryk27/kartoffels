use crate::views::game::Event as ParentEvent;
use crate::BotIdExt;
use itertools::Itertools;
use kartoffels_ui::{theme, Button, KeyCode, Ui, UiWidget, VRow};
use kartoffels_world::prelude::{AliveBotSnapshot, BotId, Snapshot};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::{
    Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
};

#[derive(Debug, Default)]
pub struct BotsModal {
    offset: usize,
    length: Option<usize>,
    height: usize,
    selected: Selected,
}

impl BotsModal {
    const WIDTHS: &[u16] = &[
        5,                        // nth
        BotId::LENGTH as u16 + 1, // id
        7,                        // age
        6,                        // score
        18,                       // actions
    ];

    pub fn render(&mut self, ui: &mut Ui<ParentEvent>, world: &Snapshot) {
        let width = Self::WIDTHS.iter().copied().sum();
        let height = ui.area.height - 2;

        let event = ui.catch(|ui| {
            ui.info_window(width, height, Some(" bots "), |ui| {
                let [body_area, _, footer_area] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Length(2),
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
            && let Some(event) = self.handle(event, world)
        {
            ui.throw(event);
        }
    }

    fn render_body(&mut self, ui: &mut Ui<Event>, world: &Snapshot) {
        let [table_area, scrollbar_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(2)])
                .areas(ui.area);

        let [thead_area, tbody_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
                .areas(table_area);

        // ---

        self.height = tbody_area.height as usize;

        match self.selected {
            Selected::Id(id) => {
                // If we're tracking a bot with specific id, recalculate the
                // offset so that this particular bot is always centered

                let nth = world
                    .bots
                    .alive
                    .iter_sorted_by_scores()
                    .enumerate()
                    .find_position(|(_, bot)| bot.id == id)
                    .map(|(nth, _)| nth);

                if let Some(nth) = nth {
                    self.offset = nth.saturating_sub(self.height / 2);
                } else {
                    // If our tracked bot is no longer on the list, it must have
                    // died - in that case let's reset the selection
                    self.selected = Selected::Nth(0);
                }
            }

            Selected::Nth(nth) => {
                // If we're tracking a row with specific ordinal, make sure it's
                // in the viewport; this prevents clipping out of the table when
                // pressing up/down arrow keys.

                self.selected = Selected::Nth(
                    nth.clamp(self.min_visible_nth(), self.max_visible_nth()),
                );
            }
        }

        self.length = world.bots.alive.len().checked_sub(self.height);
        self.offset = self.offset.min(self.length.unwrap_or(0));

        // ---

        ui.clamp(thead_area, |ui| {
            VRow::new(ui, Self::WIDTHS)
                .column(Span::raw("nth"))
                .column(Span::raw("id"))
                .column(Span::raw("age"))
                .column(Span::raw("score"));
        });

        ui.clamp(tbody_area, |ui| {
            let rows = world
                .bots
                .alive
                .iter_sorted_by_scores()
                .enumerate()
                .skip(self.offset)
                .take(self.height)
                .map(|(nth, bot)| Row {
                    nth,
                    bot,
                    selected: self.selected,
                });

            for (row, area) in rows.zip(ui.area.rows()) {
                ui.render_at(area, row);
            }
        });

        if let Some(length) = self.length {
            ui.clamp(scrollbar_area, |ui| {
                let mut state =
                    ScrollbarState::new(length).position(self.offset);

                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .render(ui.area, ui.buf, &mut state);
            });
        }
    }

    fn render_footer(&mut self, ui: &mut Ui<Event>) {
        ui.row(|ui| {
            Button::new(KeyCode::Char('w'), "scroll-up")
                .throwing(Event::ScrollUp)
                .enabled(self.selected.is_nth())
                .render(ui);

            ui.space(4);

            Button::new(KeyCode::UpArrow, "select-up")
                .throwing(Event::SelectUp)
                .enabled(self.selected.is_nth())
                .render(ui);

            if self.selected.is_nth() {
                Button::new(KeyCode::Char('t'), "track-id")
                    .throwing(Event::TrackId)
                    .right_aligned()
                    .render(ui);
            } else {
                Button::new(KeyCode::Char('t'), "track-nth")
                    .throwing(Event::TrackNth)
                    .right_aligned()
                    .render(ui);
            }
        });

        ui.space(1);

        ui.row(|ui| {
            Button::new(KeyCode::Char('s'), "scroll-down")
                .throwing(Event::ScrollDown)
                .enabled(self.selected.is_nth())
                .render(ui);

            ui.space(2);

            Button::new(KeyCode::DownArrow, "select-down")
                .throwing(Event::SelectDown)
                .enabled(self.selected.is_nth())
                .render(ui);

            Button::new(KeyCode::Escape, "close")
                .throwing(Event::Parent(ParentEvent::CloseModal))
                .right_aligned()
                .render(ui);
        });
    }

    fn handle(
        &mut self,
        event: Event,
        world: &Snapshot,
    ) -> Option<ParentEvent> {
        match event {
            Event::ScrollUp => {
                self.offset = self.offset.saturating_sub(8);
                None
            }

            Event::ScrollDown => {
                self.offset = self.offset.saturating_add(8);
                None
            }

            Event::SelectUp => {
                if let Selected::Nth(nth) = &mut self.selected {
                    *nth = nth.saturating_sub(1);

                    if *nth <= self.min_visible_nth() {
                        self.offset = self.offset.saturating_sub(1);
                    }
                }

                None
            }

            Event::SelectDown => {
                if let Selected::Nth(nth) = &mut self.selected {
                    *nth = nth.saturating_add(1).min(world.bots.alive.len());

                    if *nth >= self.max_visible_nth() {
                        self.offset += 1;
                    }
                }

                None
            }

            Event::TrackId => {
                if let Selected::Nth(nth) = self.selected
                    && let Some(bot) =
                        world.bots.alive.iter_sorted_by_scores().nth(nth)
                {
                    self.selected = Selected::Id(bot.id);
                }

                None
            }

            Event::TrackNth => {
                if let Selected::Id(id) = self.selected {
                    let nth = world
                        .bots
                        .alive
                        .iter_sorted_by_scores()
                        .enumerate()
                        .find_position(|(_, bot)| bot.id == id)
                        .map(|(nth, _)| nth)
                        .unwrap_or(0);

                    self.selected = Selected::Nth(nth);
                }

                None
            }

            Event::Parent(event) => Some(event),
        }
    }

    fn min_visible_nth(&self) -> usize {
        self.offset
    }

    fn max_visible_nth(&self) -> usize {
        self.offset + self.height - 1
    }
}

enum Event {
    ScrollUp,
    ScrollDown,
    SelectUp,
    SelectDown,
    TrackId,
    TrackNth,
    Parent(ParentEvent),
}

#[derive(Clone, Copy, Debug)]
enum Selected {
    Id(BotId),
    Nth(usize),
}

impl Selected {
    fn is_nth(&self) -> bool {
        matches!(self, Self::Nth(_))
    }

    fn matches(&self, nth: usize, bot: &AliveBotSnapshot) -> bool {
        match *self {
            Selected::Id(id2) => bot.id == id2,
            Selected::Nth(nth2) => nth2 == nth,
        }
    }
}

impl Default for Selected {
    fn default() -> Self {
        Self::Nth(0)
    }
}

#[derive(Clone, Debug)]
struct Row<'a> {
    nth: usize,
    bot: &'a AliveBotSnapshot,
    selected: Selected,
}

impl UiWidget<Event> for Row<'_> {
    fn render(self, ui: &mut Ui<Event>) {
        let is_selected = self.selected.matches(self.nth, self.bot);

        let nth = Span::raw(format!("#{}", self.nth + 1));
        let id = Span::raw(self.bot.id.to_string()).fg(self.bot.id.color());
        let age = Span::raw(format!("{}s", self.bot.age_seconds()));
        let score = Span::raw(self.bot.score.to_string());

        let inspect = {
            let key = is_selected.then_some(KeyCode::Enter);

            Button::new(key, "inspect").throwing(Event::Parent(
                ParentEvent::InspectBot { id: self.bot.id },
            ))
        };

        VRow::new(ui, BotsModal::WIDTHS)
            .column(nth)
            .column(id)
            .column(age)
            .column(score)
            .column(inspect);

        if is_selected {
            ui.buf.set_style(ui.area, Style::new().bg(theme::DARK_GRAY));
        }
    }
}
