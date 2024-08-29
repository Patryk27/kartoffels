use crate::{theme, Clear};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Padding, Widget};

pub trait BlockExt {
    fn dialog(
        title: Option<&str>,
        border_fg: Color,
        area: Rect,
        buf: &mut Buffer,
    ) -> Rect;

    fn dialog_info(title: Option<&str>, area: Rect, buf: &mut Buffer) -> Rect {
        Self::dialog(title, theme::GREEN, area, buf)
    }

    fn dialog_error(title: Option<&str>, area: Rect, buf: &mut Buffer) -> Rect {
        Self::dialog(title, theme::RED, area, buf)
    }

    fn render_and_lay(self, area: Rect, buf: &mut Buffer) -> Rect;
}

impl BlockExt for Block<'_> {
    fn dialog(
        title: Option<&str>,
        border_fg: Color,
        area: Rect,
        buf: &mut Buffer,
    ) -> Rect {
        let mut block = Block::bordered()
            .border_style(Style::new().fg(border_fg).bg(theme::BG))
            .padding(Padding::horizontal(1));

        if let Some(title) = title {
            block = block.title(title).title_alignment(Alignment::Center)
        }

        Clear.render(area, buf);

        block.render_and_lay(area, buf)
    }

    fn render_and_lay(self, area: Rect, buf: &mut Buffer) -> Rect {
        let inner_area = self.inner(area);

        self.render(area, buf);

        inner_area
    }
}
