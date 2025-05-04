use crate::{theme, Ui, UiWidget};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

#[derive(Debug)]
pub struct Clear;

impl Clear {
    pub fn render_ex(area: Rect, buf: &mut Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf[(x, y)].reset();
                buf[(x, y)].set_fg(theme::FG).set_bg(theme::BG);
            }
        }
    }
}

impl<T> UiWidget<T> for Clear {
    type Response = ();

    fn render(self, ui: &mut Ui<T>) -> Self::Response {
        Self::render_ex(ui.area, ui.buf);
    }
}
