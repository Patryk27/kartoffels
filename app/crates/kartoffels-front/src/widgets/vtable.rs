use crate::{Ui, UiWidget};
use ratatui::prelude::Rect;

#[derive(Debug)]
pub struct VRow<'a, 'b, T> {
    ui: &'a mut Ui<'b, T>,
    widths: &'static [u16],
    idx: usize,
    pos: u16,
}

impl<'a, 'b, T> VRow<'a, 'b, T> {
    pub fn new(ui: &'a mut Ui<'b, T>, widths: &'static [u16]) -> Self {
        Self {
            ui,
            widths,
            idx: 0,
            pos: 0,
        }
    }

    pub fn col(mut self, widget: impl UiWidget<T>) -> Self {
        let width = self.widths[self.idx];

        let area = Rect {
            x: self.ui.area.x + self.pos,
            y: self.ui.area.y,
            width,
            height: self.ui.area.height,
        };

        self.ui.at(area, |ui| {
            widget.render(ui);
        });

        self.idx += 1;
        self.pos += width;
        self
    }
}
