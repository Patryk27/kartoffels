use super::{Dialog, Policy, State};
use anyhow::Result;
use kartoffels_ui::{theme, Button, Ui};
use kartoffels_world::prelude::Handle as WorldHandle;
use ratatui::prelude::Rect;
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::widgets::Widget;
use std::ops::ControlFlow;
use termwiz::input::KeyCode;

#[derive(Debug)]
pub struct BottomPanel;

impl BottomPanel {
    pub fn render(
        ui: &mut Ui,
        policy: &Policy,
        handle: Option<&WorldHandle>,
        has_help: bool,
        paused: bool,
        enabled: bool,
    ) -> Option<BottomPanelResponse> {
        let mut resp = None;

        ui.row(|ui| {
            if Button::new(KeyCode::Escape, "go back")
                .enabled(enabled)
                .block()
                .render(ui)
                .pressed
            {
                resp = Some(BottomPanelResponse::GoBack);
            }

            ui.space(2);

            let label = if paused { "resume" } else { "pause" };

            if Button::new(KeyCode::Char(' '), label)
                .enabled(
                    enabled && handle.is_some() && policy.user_can_pause_world,
                )
                .block()
                .render(ui)
                .pressed
            {
                resp = Some(BottomPanelResponse::Pause);
            }

            ui.space(2);

            if has_help {
                if Button::new(KeyCode::Char('h'), "help")
                    .enabled(enabled)
                    .block()
                    .render(ui)
                    .pressed
                {
                    resp = Some(BottomPanelResponse::Help);
                }

                ui.space(2);
            }

            if Button::new(KeyCode::Char('b'), "bots")
                .enabled(enabled && handle.is_some())
                .block()
                .render(ui)
                .pressed
            {
                resp = Some(BottomPanelResponse::ListBots);
            }

            if policy.user_can_configure_world {
                ui.space(2);

                if Button::new(KeyCode::Char('C'), "configure world")
                    .enabled(enabled)
                    .block()
                    .render(ui)
                    .pressed
                {
                    resp = Some(BottomPanelResponse::ConfigureWorld);
                }
            }
        });

        if paused {
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
