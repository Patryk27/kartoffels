use ratatui::layout::Rect;

pub trait RectExt
where
    Self: Sized,
{
    fn footer(self) -> Self;
}

impl RectExt for Rect {
    fn footer(self) -> Self {
        Rect {
            x: self.x,
            y: self.y + self.height - 1,
            width: self.width,
            height: 1,
        }
    }
}
