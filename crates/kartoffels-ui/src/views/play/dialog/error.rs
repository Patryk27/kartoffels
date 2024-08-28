use crate::{theme, BlockExt, Clear};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph, Widget};

#[derive(Debug)]
pub struct ErrorDialog {
    pub text: String,
}

impl ErrorDialog {
    // TODO show escape button
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let text = Paragraph::new(self.text.as_str()).wrap(Default::default());
        let height = text.line_count(area.width - 2) as u16 + 2;

        let [_, area, _] = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Fill(1),
            Constraint::Percentage(10),
        ])
        .areas(area);

        let [_, area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(2),
        ])
        .areas(area);

        let area = Block::bordered()
            .border_style(Style::new().fg(theme::RED).bg(theme::BG))
            .title("whoopsie")
            .title_alignment(Alignment::Center)
            .render_and_measure(area, buf);

        Clear.render(area, buf);

        text.render(area, buf);
    }
}
