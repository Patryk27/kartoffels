mod connected;
mod connecting;
mod idle;

pub use self::connected::*;
pub use self::connecting::*;
pub use self::idle::*;
use kartoffels_world::prelude::BotUpdate;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use termwiz::input::InputEvent;

#[derive(Debug, Default)]
pub enum SidePanel {
    #[default]
    Idle,
    Connecting(ConnectingSidePanel),
    Connected(ConnectedSidePanel),
}

impl SidePanel {
    pub const WIDTH: u16 = 22;

    pub fn handle(&mut self, event: InputEvent) -> SidePanelOutcome {
        match self {
            SidePanel::Idle => match IdleSidePanel::handle(event) {
                IdleSidePanelOutcome::ConnectToBot => {
                    *self = SidePanel::Connecting(Default::default());
                    SidePanelOutcome::None
                }

                IdleSidePanelOutcome::UploadBot => SidePanelOutcome::UploadBot,

                IdleSidePanelOutcome::Forward(event) => {
                    SidePanelOutcome::Forward(event)
                }
            },

            SidePanel::Connecting(this) => match this.handle(event) {
                ConnectingSidePanelOutcome::ConnectToBot(id) => {
                    *self = SidePanel::Connected(ConnectedSidePanel { id });
                    SidePanelOutcome::None
                }

                ConnectingSidePanelOutcome::Abort => {
                    *self = SidePanel::Idle;
                    SidePanelOutcome::None
                }

                ConnectingSidePanelOutcome::None => SidePanelOutcome::None,

                ConnectingSidePanelOutcome::Forward(event) => {
                    SidePanelOutcome::Forward(event)
                }
            },

            SidePanel::Connected(this) => match this.handle(event) {
                ConnectedSidePanelOutcome::Disconnect => {
                    *self = SidePanel::Idle;
                    SidePanelOutcome::None
                }

                ConnectedSidePanelOutcome::Forward(event) => {
                    SidePanelOutcome::Forward(event)
                }
            },
        }
    }

    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        bots: &[BotUpdate],
        enabled: bool,
    ) {
        let area = Rect {
            x: area.x + 1,
            y: area.y,
            width: area.width - 1,
            height: area.height,
        };

        match self {
            SidePanel::Idle => {
                IdleSidePanel.render(area, buf, enabled);
            }
            SidePanel::Connecting(this) => {
                this.render(area, buf);
            }
            SidePanel::Connected(this) => {
                this.render(area, buf, bots);
            }
        }
    }
}

#[derive(Debug)]
pub enum SidePanelOutcome {
    UploadBot,
    None,
    Forward(InputEvent),
}
