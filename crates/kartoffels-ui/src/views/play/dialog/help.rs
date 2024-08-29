use super::DialogEvent;
use ratatui::prelude::{Buffer, Rect};
use termwiz::input::InputEvent;

#[derive(Debug, Default)]
pub struct HelpDialog;

impl HelpDialog {
    pub const WIDTH: u16 = 32;
    pub const HEIGHT: u16 = 32;

    pub fn render(&self, _area: Rect, _buf: &mut Buffer) {
        //
    }

    pub fn handle(&mut self, _event: InputEvent) -> Option<DialogEvent> {
        None
    }
}
