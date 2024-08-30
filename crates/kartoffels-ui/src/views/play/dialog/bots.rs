use super::DialogEvent;
use crate::{BotIdExt, Ui};
use kartoffels_world::prelude::{BotId, Snapshot};
use ratatui::style::Stylize;
use ratatui::widgets::{Cell, Row, StatefulWidget, Table, TableState};

#[derive(Debug, Default)]
pub struct BotsDialog {
    pub table: TableState,
}

impl BotsDialog {
    const WIDTHS: [u16; 4] = [
        4,                    // #
        BotId::LENGTH as u16, // id
        6,                    // age
        7,                    // score
    ];

    pub fn render(
        &mut self,
        ui: &mut Ui,
        snapshot: &Snapshot,
    ) -> Option<DialogEvent> {
        let width = Self::WIDTHS.iter().copied().sum::<u16>() + 4;
        let height = ui.area().height - 2;

        ui.info_dialog(width, height, Some(" bots "), |ui| {
            let header = Row::new(vec!["#", "id", "age", "score ⯆"]);

            let rows =
                snapshot.bots.alive.iter_sorted_by_scores().enumerate().map(
                    |(place, (bot, score))| {
                        Row::new([
                            Cell::new(format!("#{}", place + 1)),
                            Cell::new(bot.id.to_string()).fg(bot.id.color()),
                            Cell::new(bot.age.to_string()),
                            Cell::new(score.to_string()),
                        ])
                    },
                );

            Table::new(rows, Self::WIDTHS).header(header).render(
                ui.area(),
                ui.buf(),
                &mut self.table,
            );

            None
        })
    }

    // pub fn handle(&mut self, event: InputEvent) -> Option<DialogEvent> {
    //     if let InputEvent::Key(event) = &event {
    //         match (event.key, event.modifiers) {
    //             (KeyCode::Char('w') | KeyCode::UpArrow, Modifiers::NONE) => {
    //                 *self.table.offset_mut() =
    //                     self.table.offset().saturating_sub(8);
    //             }

    //             (KeyCode::Char('s') | KeyCode::DownArrow, Modifiers::NONE) => {
    //                 *self.table.offset_mut() =
    //                     self.table.offset().saturating_add(8);
    //             }

    //             _ => (),
    //         }
    //     }

    //     None
    // }
}
