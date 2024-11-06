use crate::Ui;
use ratatui::widgets::Widget;

pub trait Render<T> {
    type Response = ();

    fn render(self, ui: &mut Ui<T>) -> Self::Response;
}

impl<T, W> Render<T> for W
where
    W: Widget,
{
    type Response = ();

    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        W::render(self, ui.area, ui.buf)
    }
}
