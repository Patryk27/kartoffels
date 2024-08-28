use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::widgets::{Paragraph, Widget};

#[derive(Debug)]
pub struct HelpDialog;

impl HelpDialog {
    pub const WIDTH: u16 = 32;
    pub const HEIGHT: u16 = 32;

    pub fn render(self, area: Rect, buf: &mut Buffer) {
        let [_, area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(Self::WIDTH),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [_, area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(Self::HEIGHT),
            Constraint::Fill(1),
        ])
        .areas(area);

        Paragraph::new("hello, world!").render(area, buf);
    }
}
