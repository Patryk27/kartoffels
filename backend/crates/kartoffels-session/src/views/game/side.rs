mod idle;
mod joined;

use self::idle::*;
use self::joined::*;
use super::{Event, State};
use kartoffels_ui::{theme, Ui};

#[derive(Debug)]
pub struct SidePanel;

impl SidePanel {
    pub const WIDTH: u16 = 27;

    pub fn render(ui: &mut Ui<Event>, state: &State) {
        for y in ui.area.top()..ui.area.bottom() {
            ui.buf[(ui.area.x, y)].set_bg(theme::DARKER_GRAY);
        }

        ui.area.x += 2;
        ui.area.width -= 3;

        ui.enable(state.handle.is_some(), |ui| {
            if let Some(bot) = &state.bot {
                JoinedSidePanel::render(ui, state, bot);
            } else {
                IdleSidePanel::render(ui, state);
            }
        });
    }
}
