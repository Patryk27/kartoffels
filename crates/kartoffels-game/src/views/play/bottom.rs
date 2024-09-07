use super::{Dialog, State};
use anyhow::Result;
use kartoffels_ui::{theme, Button, Ui};
use ratatui::prelude::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::widgets::Widget;
use std::ops::ControlFlow;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct BottomPanel;

impl BottomPanel {
    pub fn render(ui: &mut Ui, state: &State) -> Option<BottomPanelResponse> {
        let mut resp = None;

        ui.row(|ui| {
            if Button::new(KeyCode::Escape, "go back").render(ui).pressed {
                resp = Some(BottomPanelResponse::GoBack);
            }

            ui.space(2);

            // ---

            let label = if state.paused { "resume" } else { "pause" };

            if Button::new(KeyCode::Char(' '), label)
                .enabled(
                    state.handle.is_some() && state.perms.user_can_pause_world,
                )
                .render(ui)
                .pressed
            {
                resp = Some(BottomPanelResponse::Pause);
            }

            ui.space(2);

            // ---

            if state.help.is_some() {
                if Button::new(KeyCode::Char('h'), "help").render(ui).pressed {
                    resp = Some(BottomPanelResponse::Help);
                }

                ui.space(2);
            }

            // ---

            if Button::new(KeyCode::Char('b'), "bots")
                .enabled(state.handle.is_some())
                .render(ui)
                .pressed
            {
                resp = Some(BottomPanelResponse::ListBots);
            }

            // ---

            if state.perms.user_can_configure_world {
                ui.space(2);

                if Button::new(KeyCode::Char('C'), "configure world")
                    .render(ui)
                    .pressed
                {
                    resp = Some(BottomPanelResponse::ConfigureWorld);
                }
            }
        });

        if state.paused {
            let area = Rect {
                x: ui.area().width - 6,
                y: ui.area().y,
                width: 6,
                height: 1,
            };

            Span::raw("PAUSED")
                .fg(theme::FG)
                .bg(theme::RED)
                .render(area, ui.buf());
        } else if let Some(status) = &state.status {
            let width = status.len() as u16;

            let area = Rect {
                x: ui.area().width - width,
                y: ui.area().y,
                width,
                height: 1,
            };

            Span::raw(status)
                .fg(theme::BG)
                .bg(theme::YELLOW)
                .render(area, ui.buf());
        }

        resp
    }
}

#[derive(Debug)]
pub enum BottomPanelResponse {
    GoBack,
    Help,
    Pause,
    ListBots,
    ConfigureWorld,
}

impl BottomPanelResponse {
    pub async fn handle(
        self,
        state: &mut State,
    ) -> Result<ControlFlow<(), ()>> {
        match self {
            BottomPanelResponse::GoBack => {
                return Ok(ControlFlow::Break(()));
            }

            BottomPanelResponse::Help => {
                if let Some(dialog) = state.help {
                    state.dialog = Some(Dialog::Help(dialog));
                }
            }

            BottomPanelResponse::Pause => {
                state.pause(!state.paused).await?;
            }

            BottomPanelResponse::ListBots => {
                state.dialog = Some(Dialog::Bots(Default::default()));
            }

            BottomPanelResponse::ConfigureWorld => {
                state.dialog = Some(Dialog::ConfigureWorld(Default::default()));
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}
