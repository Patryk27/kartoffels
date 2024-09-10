use crate::{theme, Ui};

#[derive(Debug)]
pub struct Backdrop;

impl Backdrop {
    pub fn render(ui: &mut Ui) {
        let area = ui.area();
        let buf = ui.buf();

        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf[(x, y)].set_fg(theme::DARK_GRAY).set_bg(theme::BG);
            }
        }
    }
}
