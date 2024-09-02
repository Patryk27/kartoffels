mod idle;
mod joined;

use self::idle::*;
use self::joined::*;
use super::JoinedBot;
use crate::{Clear, Ui};
use kartoffels_world::prelude::Snapshot;
use ratatui::layout::Rect;

#[derive(Debug)]
pub struct SidePanel;

impl SidePanel {
    pub const WIDTH: u16 = 25;

    pub fn render(
        ui: &mut Ui,
        world: &Snapshot,
        bot: Option<&JoinedBot>,
        enabled: bool,
    ) -> Option<SidePanelResponse> {
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

        ui.clamp(area, |ui| {
            if let Some(bot) = bot {
                JoinedSidePanel::render(ui, world, bot, enabled)
            } else {
                IdleSidePanel::render(ui, enabled)
            }
        })
    }
}

#[derive(Debug)]
pub enum SidePanelResponse {
    UploadBot,
    JoinBot,
    LeaveBot,
    ShowBotHistory,
}
