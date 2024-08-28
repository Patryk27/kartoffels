use kartoffels_world::prelude::BotId;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::{Block, Widget};

pub trait BotIdExt {
    fn color(&self) -> Color;
}

impl BotIdExt for BotId {
    fn color(&self) -> Color {
        let hue = (self.get().get() % 360) as f64;

        Color::from_hsl(hue, 100.0, 50.0)
    }
}

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
