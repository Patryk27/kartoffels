mod footer;
mod header;
mod menu;

use self::footer::*;
use self::header::*;
use self::menu::*;
use crate::Background;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{Fade, FadeDir, Render, Term};
use ratatui::layout::{Constraint, Layout};
use tracing::debug;

pub async fn run(
    store: &Store,
    term: &mut Term,
    bg: &Background,
    fade_in: bool,
) -> Result<Response> {
    debug!("run()");

    let mut fade_in = if fade_in && !store.testing() {
        Some(Fade::new(FadeDir::In))
    } else {
        None
    };

    let mut fade_out: Option<(Fade, Response)> = None;

    loop {
        let resp = term
            .draw(|ui| {
                let [_, area, _] = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(Header::width()),
                    Constraint::Fill(1),
                ])
                .areas(ui.area());

                let [_, header_area, _, menu_area, _] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(Header::height()),
                    Constraint::Fill(1),
                    Constraint::Length(Menu::height(store, ui)),
                    Constraint::Fill(1),
                ])
                .areas(area);

                let [_, menu_area, _] = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(Menu::width()),
                    Constraint::Fill(1),
                ])
                .areas(menu_area);

                bg.render(ui);

                ui.clamp(header_area, |ui| {
                    Header::render(ui);
                });

                ui.clamp(menu_area, |ui| {
                    Menu::render(store, ui);
                });

                Footer::render(store, ui);

                if let Some(fade) = &fade_in {
                    if fade.render(ui).is_completed() {
                        fade_in = None;
                    }
                }

                if let Some((fade, _)) = &fade_out {
                    fade.render(ui);
                }
            })
            .await?;

        term.poll().await?;

        if let Some((fade, resp)) = &fade_out {
            if fade.is_completed() {
                return Ok(*resp);
            }
        }

        if let Some(resp) = resp {
            if resp.fade_out() && !store.testing() {
                fade_out = Some((Fade::new(FadeDir::Out), resp));
            } else {
                return Ok(resp);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Response {
    Play,
    Sandbox,
    Tutorial,
    Challenges,
    Quit,
}

impl Response {
    fn fade_out(&self) -> bool {
        matches!(self, Response::Sandbox | Response::Tutorial)
    }
}
