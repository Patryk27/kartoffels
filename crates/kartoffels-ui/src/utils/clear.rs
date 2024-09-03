use crate::{theme, Ui};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

#[derive(Debug)]
pub struct Clear;

impl Clear {
    pub fn render(ui: &mut Ui) {
        Self::render_ex(ui.area(), ui.buf())
    }

    pub fn render_ex(area: Rect, buf: &mut Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf[(x, y)].reset();
                buf[(x, y)].set_fg(theme::FG).set_bg(theme::BG);
            }
        }
    }
}
