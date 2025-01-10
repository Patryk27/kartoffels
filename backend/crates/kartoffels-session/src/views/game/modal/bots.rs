use crate::views::game::Event;
use crate::BotIdExt;
use kartoffels_ui::{
    Button, RectExt, Render, Ui, VirtualRow, WidgetList, WidgetListState,
};
use kartoffels_world::prelude::{AliveBotSnapshot, BotId, Snapshot};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct BotsModal {
    state: WidgetListState,
}

impl BotsModal {
    const WIDTHS: &[u16] = &[
        5,                        // nth
        BotId::LENGTH as u16 + 1, // id
        7,                        // age
        8,                        // score
        6,                        // action
    ];

    pub fn render(&mut self, ui: &mut Ui<Event>, world: &Snapshot) {
        let width = Self::WIDTHS.iter().copied().sum();
        let height = ui.area.height - 2;

        ui.info_window(width, height, Some(" bots "), |ui| {
            VirtualRow::new(ui, Self::WIDTHS)
                .add(Span::raw("nth"))
                .add(Span::raw("id"))
                .add(Span::raw("age"))
                .add(Span::raw("score"))
                .add(Span::raw("action"));

            ui.space(1);

            let rows = world
                .bots
                .alive
                .iter_sorted_by_scores()
                .enumerate()
                .map(|(nth, bot)| BotsModalRow { nth, bot });

            let area = Rect {
                height: ui.area.height - 2,
                ..ui.area
            };

            ui.clamp(area, |ui| {
                WidgetList::new(rows, &mut self.state).render(ui);
            });

            ui.space(2);

            ui.clamp(ui.area.footer(1), |ui| {
                ui.row(|ui| {
                    if Button::new(KeyCode::Char('w'), "scroll-up")
                        .render(ui)
                        .pressed
                    {
                        self.state.offset = self.state.offset.saturating_sub(8);
                    }

                    ui.space(2);

                    if Button::new(KeyCode::Char('s'), "scroll-down")
                        .render(ui)
                        .pressed
                    {
                        self.state.offset = self.state.offset.saturating_add(8);
                    }

                    ui.space(2);

                    Button::new(KeyCode::Escape, "close")
                        .throwing(Event::CloseModal)
                        .right_aligned()
                        .render(ui);
                });
            });
        });
    }
}

#[derive(Clone, Debug)]
struct BotsModalRow<'a> {
    nth: usize,
    bot: &'a AliveBotSnapshot,
}

impl Render<Event> for BotsModalRow<'_> {
    fn render(self, ui: &mut Ui<Event>) {
        let nth = Span::raw(format!("#{}", self.nth + 1));
        let id = Span::raw(self.bot.id.to_string()).fg(self.bot.id.color());
        let age = Span::raw(format!("{}s", self.bot.age_seconds()));
        let score = Span::raw(self.bot.score.to_string());

        let join = Button::new(None, "join")
            .throwing(Event::JoinBot { id: self.bot.id });

        VirtualRow::new(ui, BotsModal::WIDTHS)
            .add(nth)
            .add(id)
            .add(age)
            .add(score)
            .add(join);
    }
}
