use ratatui::layout::Rect;

pub trait RectExt
where
    Self: Sized,
{
    fn header(self, height: u16) -> Self;
    fn footer(self, height: u16) -> Self;
}

impl RectExt for Rect {
    fn header(self, height: u16) -> Self {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height,
        }
    }

    fn footer(self, height: u16) -> Self {
        Rect {
            x: self.x,
            y: self.y + self.height - height,
            width: self.width,
            height,
        }
    }
}
