use ratatui::layout::Rect;

pub trait RectExt
where
    Self: Sized,
{
    fn footer(self, height: u16) -> Self;
}

impl RectExt for Rect {
    fn footer(self, height: u16) -> Self {
        Rect {
            x: self.x,
            y: self.y + self.height - height,
            width: self.width,
            height,
        }
    }
}
