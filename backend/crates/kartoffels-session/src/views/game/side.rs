mod idle;
mod joined;

use self::idle::*;
use self::joined::*;
use super::{Event, State};
use kartoffels_ui::{Clear, Ui};
use ratatui::layout::Rect;

#[derive(Debug)]
pub struct SidePanel;

impl SidePanel {
    pub const WIDTH: u16 = 25;

    pub fn render(ui: &mut Ui<Event>, state: &State) {
        let area = {
            let area = ui.area();

            Rect {
                x: area.x + 1,
                y: area.y,
                width: area.width - 1,
                height: area.height,
            }
        };

        Clear::render(ui);

        ui.enable(state.handle.is_some(), |ui| {
            ui.clamp(area, |ui| {
                if let Some(bot) = &state.bot {
                    JoinedSidePanel::render(ui, state, bot);
                } else {
                    IdleSidePanel::render(ui, state);
                }
            });
        });
    }
}
