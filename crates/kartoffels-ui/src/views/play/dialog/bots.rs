use super::DialogEvent;
use crate::{BlockExt, BotIdExt, LayoutExt};
use kartoffels_world::prelude::{BotId, Snapshot};
use ratatui::layout::Layout;
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Cell, Row, StatefulWidget, Table, TableState};
use termwiz::input::{InputEvent, KeyCode, Modifiers};

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
        area: Rect,
        buf: &mut Buffer,
        snapshot: &Snapshot,
    ) {
        let area = {
            let width = Self::WIDTHS.iter().copied().sum::<u16>() + 4;
            let height = area.height - 2;

            Block::dialog_info(
                Some(" bots "),
                Layout::dialog(width, height, area),
                buf,
            )
        };

        let header = Row::new(vec!["#", "id", "age", "score ⯆"]);

        let rows = snapshot.bots.alive.iter_sorted_by_scores().enumerate().map(
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
            area,
            buf,
            &mut self.table,
        );
    }

    pub fn handle(&mut self, event: InputEvent) -> Option<DialogEvent> {
        if let InputEvent::Key(event) = &event {
            match (event.key, event.modifiers) {
                (KeyCode::Char('w') | KeyCode::UpArrow, Modifiers::NONE) => {
                    *self.table.offset_mut() =
                        self.table.offset().saturating_sub(8);
                }

                (KeyCode::Char('s') | KeyCode::DownArrow, Modifiers::NONE) => {
                    *self.table.offset_mut() =
                        self.table.offset().saturating_add(8);
                }

                _ => (),
            }
        }

        None
    }
}
