use crate::{theme, Ui};

#[derive(Debug)]
pub struct Clear;

impl Clear {
    pub fn render(ui: &mut Ui) {
        let area = ui.area();
        let buf = ui.buf();

        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf[(x, y)]
                    .set_symbol(" ")
                    .set_fg(theme::FG)
                    .set_bg(theme::BG);
            }
        }
    }
}
