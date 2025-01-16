use crate::Ui;
use ratatui::widgets::Widget;

pub trait UiWidget<T> {
    type Response = ();

    fn render(self, ui: &mut Ui<T>) -> Self::Response;
}

impl<T, W> UiWidget<T> for W
where
    W: Widget,
{
    type Response = ();

    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        W::render(self, ui.area, ui.buf)
    }
}

#[derive(Clone, Debug)]
pub struct FnUiWidget<F>(pub F);

impl FnUiWidget<()> {
    pub fn new<T, F, R>(f: F) -> FnUiWidget<F>
    where
        F: FnOnce(&mut Ui<T>) -> R,
    {
        FnUiWidget(f)
    }
}

impl<T, F, R> UiWidget<T> for FnUiWidget<F>
where
    F: FnOnce(&mut Ui<T>) -> R,
{
    type Response = R;

    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        (self.0)(ui)
    }
}
