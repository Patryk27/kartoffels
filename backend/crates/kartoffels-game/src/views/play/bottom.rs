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
            ui.space(2);

            Self::render_go_back_btn(ui, &mut resp);
            Self::render_pause_btn(ui, state, &mut resp);
            Self::render_help_btn(ui, state, &mut resp);
            Self::render_bots_btn(ui, state, &mut resp);
            Self::render_configure_btn(ui, state, &mut resp);
        });

        Self::render_status(ui, state);

        resp
    }

    fn render_go_back_btn(ui: &mut Ui, resp: &mut Option<BottomPanelResponse>) {
        if Button::new(KeyCode::Escape, "go back").render(ui).pressed {
            *resp = Some(BottomPanelResponse::GoBack)
        }
    }

    fn render_pause_btn(
        ui: &mut Ui,
        state: &State,
        resp: &mut Option<BottomPanelResponse>,
    ) {
        ui.space(2);

        let label = if state.paused { "resume" } else { "pause" };

        let enabled =
            state.handle.is_some() && state.perms.user_can_pause_world;

        if Button::new(KeyCode::Char(' '), label)
            .enabled(enabled)
            .render(ui)
            .pressed
        {
            *resp = Some(BottomPanelResponse::Pause);
        }
    }

    fn render_help_btn(
        ui: &mut Ui,
        state: &State,
        resp: &mut Option<BottomPanelResponse>,
    ) {
        ui.space(2);

        if Button::new(KeyCode::Char('h'), "help")
            .enabled(state.help.is_some())
            .render(ui)
            .pressed
        {
            *resp = Some(BottomPanelResponse::Help);
        }
    }

    fn render_bots_btn(
        ui: &mut Ui,
        state: &State,
        resp: &mut Option<BottomPanelResponse>,
    ) {
        if !state.perms.single_bot_mode {
            ui.space(2);

            if Button::new(KeyCode::Char('b'), "bots")
                .enabled(state.handle.is_some())
                .render(ui)
                .pressed
            {
                *resp = Some(BottomPanelResponse::ListBots);
            }
        }
    }

    fn render_configure_btn(
        ui: &mut Ui,
        state: &State,
        resp: &mut Option<BottomPanelResponse>,
    ) {
        if state.perms.user_can_configure_world {
            ui.space(2);

            if Button::new(KeyCode::Char('C'), "configure world")
                .render(ui)
                .pressed
            {
                *resp = Some(BottomPanelResponse::ConfigureWorld);
            }
        }
    }

    fn render_status(ui: &mut Ui, state: &State) {
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
