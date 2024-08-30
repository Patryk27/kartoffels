mod idle;
mod joined;

use self::idle::*;
use self::joined::*;
use super::JoinedBot;
use kartoffels_world::prelude::Update;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use termwiz::input::InputEvent;

#[derive(Debug)]
pub struct SidePanel<'a> {
    pub update: &'a Update,
    pub bot: Option<&'a JoinedBot>,
    pub enabled: bool,
}

impl<'a> SidePanel<'a> {
    pub const WIDTH: u16 = 22;

    pub fn handle(is_joined: bool, event: InputEvent) -> SidePanelEvent {
        if is_joined {
            JoinedSidePanel::handle(event)
        } else {
            IdleSidePanel::handle(event)
        }
    }
}

impl Widget for SidePanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = Rect {
            x: area.x + 1,
            y: area.y,
            width: area.width - 1,
            height: area.height,
        };

        if let Some(bot) = &self.bot {
            JoinedSidePanel {
                update: self.update,
                bot,
                enabled: self.enabled,
            }
            .render(area, buf);
        } else {
            IdleSidePanel {
                enabled: self.enabled,
            }
            .render(area, buf);
        }
    }
}

#[derive(Debug)]
pub enum SidePanelEvent {
    UploadBot,
    JoinBot,
    LeaveBot,
    Forward(InputEvent),
}
