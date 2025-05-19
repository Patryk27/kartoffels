mod idle;
mod joined;

use self::idle::*;
use self::joined::*;
use super::{Event, View};
use crate::Ui;

#[derive(Debug)]
pub struct SidePanel;

impl SidePanel {
    pub const WIDTH: u16 = 26;

    pub fn render(ui: &mut Ui<Event>, view: &View) {
        ui.area.x += 1;
        ui.area.width -= 1;

        ui.enabled(view.world.is_some(), |ui| {
            if let Some(bot) = &view.bot {
                JoinedSidePanel::render(ui, view, bot);
            } else {
                IdleSidePanel::render(ui, view);
            }
        });
    }
}
