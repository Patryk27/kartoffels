use super::DialogEvent;
use kartoffels_world::prelude::Update;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::widgets::{Row, StatefulWidget, Table, TableState};
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug, Default)]
pub struct BotsDialog {
    pub table: TableState,
}

impl BotsDialog {
    pub const WIDTH: u16 = 32;

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, _state: &Update) {
        let [_, area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(Self::WIDTH),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [_, area, _] = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Fill(1),
            Constraint::Percentage(10),
        ])
        .areas(area);

        let rows = [
            Row::new(vec!["Cell1", "Cell2"]),
            Row::new(vec!["Cell3", "Cell4"]),
        ];

        let widths = [Constraint::Length(5), Constraint::Length(5)];

        Table::new(rows, widths).render(area, buf, &mut self.table);
    }

    pub fn handle(&mut self, event: InputEvent) -> Option<DialogEvent> {
        if let InputEvent::Key(event) = &event {
            match (event.key, event.modifiers) {
                (KeyCode::Char('w') | KeyCode::UpArrow, Modifiers::NONE) => {
                    *self.table.offset_mut() =
                        self.table.offset().saturating_add(8);
                }

                (KeyCode::Char('s') | KeyCode::DownArrow, Modifiers::NONE) => {
                    *self.table.offset_mut() =
                        self.table.offset().saturating_sub(8);
                }

                _ => (),
            }
        }

        None
    }
}
