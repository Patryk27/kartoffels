use crate::views::game::Event;
use crate::BotIdExt;
use kartoffels_ui::{
    Button, RectExt, Render, Ui, VirtualRow, WidgetList, WidgetListState,
};
use kartoffels_world::prelude::{BotId, Snapshot, SnapshotAliveBot};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct BotsDialog {
    state: WidgetListState,
}

impl BotsDialog {
    const WIDTHS: &[u16] = &[
        5,                        // nth
        BotId::LENGTH as u16 + 1, // id
        7,                        // age
        8,                        // score
        8,                        // join button
    ];

    pub fn render(&mut self, ui: &mut Ui<Event>, world: &Snapshot) {
        let width = Self::WIDTHS.iter().copied().sum::<u16>() + 7;
        let height = ui.area().height - 2;

        ui.info_window(width, height, Some(" bots "), |ui| {
            VirtualRow::new(ui, Self::WIDTHS)
                .add(Span::raw("nth"))
                .add(Span::raw("id"))
                .add(Span::raw("age"))
                .add(Span::raw("score"));

            ui.space(1);

            let rows = world
                .bots()
                .alive()
                .iter_sorted_by_scores()
                .enumerate()
                .map(|(nth, bot)| BotsDialogRow { nth, bot });

            let area = Rect {
                height: ui.area().height - 2,
                ..ui.area()
            };

            ui.clamp(area, |ui| {
                WidgetList::new(rows, &mut self.state).render(ui);
            });

            ui.space(2);

            ui.clamp(ui.area().footer(1), |ui| {
                ui.row(|ui| {
                    if Button::new(KeyCode::Char('w'), "scroll up")
                        .render(ui)
                        .pressed
                    {
                        self.state.offset = self.state.offset.saturating_sub(8);
                    }

                    ui.space(2);

                    if Button::new(KeyCode::Char('s'), "scroll down")
                        .render(ui)
                        .pressed
                    {
                        self.state.offset = self.state.offset.saturating_add(8);
                    }

                    ui.space(2);

                    Button::new(KeyCode::Escape, "close")
                        .throwing(Event::CloseDialog)
                        .render(ui);
                });
            });
        });
    }
}

#[derive(Clone, Debug)]
struct BotsDialogRow<'a> {
    nth: usize,
    bot: &'a SnapshotAliveBot,
}

impl Render<Event> for BotsDialogRow<'_> {
    fn render(self, ui: &mut Ui<Event>) {
        let nth = Span::raw(format!("#{}", self.nth + 1));
        let id = Span::raw(self.bot.id.to_string()).fg(self.bot.id.color());
        let age = Span::raw(format!("{}s", self.bot.age));
        let score = Span::raw(self.bot.score.to_string());

        let join =
            Button::new(None, "join").throwing(Event::JoinBot(self.bot.id));

        VirtualRow::new(ui, BotsDialog::WIDTHS)
            .add(nth)
            .add(id)
            .add(age)
            .add(score)
            .add(join);
    }
}
