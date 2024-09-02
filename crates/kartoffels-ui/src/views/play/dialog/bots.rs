use super::DialogResponse;
use crate::{BotIdExt, Button, RectExt, Ui};
use kartoffels_world::prelude::{BotId, Snapshot};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::widgets::{Cell, Row, StatefulWidget, Table, TableState};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct BotsDialog {
    pub table: TableState,
}

impl BotsDialog {
    const WIDTHS: [u16; 4] = [
        4,                    // nth
        BotId::LENGTH as u16, // id
        6,                    // age
        7,                    // score
    ];

    pub fn render(
        &mut self,
        ui: &mut Ui,
        world: &Snapshot,
    ) -> Option<DialogResponse> {
        let mut resp = None;

        let width = Self::WIDTHS.iter().copied().sum::<u16>() + 7;
        let height = ui.area().height - 2;

        ui.info_dialog(width, height, Some(" bots "), |ui| {
            let header = Row::new(vec!["nth", "id", "age", "score ⯆"]);

            let rows =
                world.bots.alive.iter_sorted_by_scores().enumerate().map(
                    |(place, (bot, score))| {
                        Row::new([
                            Cell::new(format!("#{}", place + 1)),
                            Cell::new(bot.id.to_string()).fg(bot.id.color()),
                            Cell::new(format!("{}s", bot.age)),
                            Cell::new(score.to_string()),
                        ])
                    },
                );

            let area = Rect {
                height: ui.area().height - 2,
                ..ui.area()
            };

            Table::new(rows, Self::WIDTHS).header(header).render(
                area,
                ui.buf(),
                &mut self.table,
            );

            ui.space(2);

            ui.clamp(ui.area().footer(1), |ui| {
                ui.row(|ui| {
                    if Button::new(KeyCode::Char('w'), "scroll up")
                        .block()
                        .render(ui)
                        .pressed
                    {
                        *self.table.offset_mut() =
                            self.table.offset().saturating_sub(8);
                    }

                    ui.space(2);

                    if Button::new(KeyCode::Char('s'), "scroll down")
                        .block()
                        .render(ui)
                        .pressed
                    {
                        *self.table.offset_mut() =
                            self.table.offset().saturating_add(8);
                    }

                    ui.space(2);

                    if Button::new(KeyCode::Escape, "close")
                        .block()
                        .render(ui)
                        .pressed
                    {
                        resp = Some(DialogResponse::Close);
                    }
                });
            });
        });

        resp
    }
}
