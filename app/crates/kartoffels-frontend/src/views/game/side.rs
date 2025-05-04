mod idle;
mod joined;

use self::idle::*;
use self::joined::*;
use super::{Event, State};
use kartoffels_ui::Ui;

#[derive(Debug)]
pub struct SidePanel;

impl SidePanel {
    pub const WIDTH: u16 = 26;

    pub fn render(ui: &mut Ui<Event>, state: &State) {
        ui.area.x += 1;
        ui.area.width -= 1;

        ui.enable(state.world.is_some(), |ui| {
            if let Some(bot) = &state.bot {
                JoinedSidePanel::render(ui, state, bot);
            } else {
                IdleSidePanel::render(ui, state);
            }
        });
    }
}
