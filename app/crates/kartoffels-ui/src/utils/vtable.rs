use crate::{Ui, UiWidget};
use ratatui::prelude::Rect;

#[derive(Debug)]
pub struct VRow<'a, 'b, T> {
    ui: &'a mut Ui<'b, T>,
    widths: &'static [u16],
    col: usize,
    offset: u16,
}

impl<'a, 'b, T> VRow<'a, 'b, T> {
    pub fn new(ui: &'a mut Ui<'b, T>, widths: &'static [u16]) -> Self {
        Self {
            ui,
            widths,
            col: 0,
            offset: 0,
        }
    }

    pub fn column(mut self, widget: impl UiWidget<T>) -> Self {
        let width = self.widths[self.col];

        let area = Rect {
            x: self.ui.area.x + self.offset,
            y: self.ui.area.y,
            width,
            height: self.ui.area.height,
        };

        self.ui.clamp(area, |ui| {
            widget.render(ui);
        });

        self.col += 1;
        self.offset += width;
        self
    }
}
