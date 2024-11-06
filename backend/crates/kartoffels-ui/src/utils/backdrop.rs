use crate::{theme, Ui};

#[derive(Debug)]
pub struct Backdrop;

impl Backdrop {
    pub fn render<T>(ui: &mut Ui<T>) {
        for x in ui.area.left()..ui.area.right() {
            for y in ui.area.top()..ui.area.bottom() {
                ui.buf[(x, y)].set_fg(theme::DARK_GRAY).set_bg(theme::BG);
            }
        }
    }
}
