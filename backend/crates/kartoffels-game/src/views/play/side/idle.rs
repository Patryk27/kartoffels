use super::SidePanelResponse;
use crate::play::State;
use kartoffels_ui::{Button, Ui};
use ratatui::layout::{Constraint, Layout};
use termwiz::input::KeyCode;

#[derive(Debug, Default)]
pub struct IdleSidePanel;

impl IdleSidePanel {
    pub fn render(ui: &mut Ui, state: &State) -> Option<SidePanelResponse> {
        let mut resp = None;

        let [_, join_area, upload_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(ui.area());

        if !state.perms.single_bot_mode {
            ui.enable(!state.snapshot.bots().is_empty(), |ui| {
                ui.clamp(join_area, |ui| {
                    if Button::new(KeyCode::Char('j'), "join bot")
                        .render(ui)
                        .pressed
                    {
                        resp = Some(SidePanelResponse::JoinBot);
                    }
                });
            });
        }

        ui.clamp(upload_area, |ui| {
            if Button::new(KeyCode::Char('u'), "upload bot")
                .render(ui)
                .pressed
            {
                resp = Some(SidePanelResponse::UploadBot);
            }
        });

        resp
    }
}
