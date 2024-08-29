use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Widget};

pub trait BlockExt {
    fn render_and_measure(self, area: Rect, buf: &mut Buffer) -> Rect;
}

impl BlockExt for Block<'_> {
    fn render_and_measure(self, area: Rect, buf: &mut Buffer) -> Rect {
        let inner_area = self.inner(area);

        self.render(area, buf);

        inner_area
    }
}
