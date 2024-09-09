mod header;
mod menu;

use self::header::*;
use self::menu::*;
use super::Background;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{theme, Term};
use ratatui::layout::{Constraint, Layout};
use tokio::time;

pub async fn run(
    term: &mut Term,
    store: &Store,
    bg: &mut Background,
) -> Result<Response> {
    loop {
        let mut resp = None;

        term.draw(|ui| {
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
                Constraint::Length(Menu::height(ui, store)),
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
                resp = Menu::render(ui, store);
            });
        })
        .await?;

        if let Some(resp) = resp {
            time::sleep(theme::INTERACTION_TIME).await;

            return Ok(resp);
        }

        term.poll().await?;
    }
}

#[derive(Debug)]
pub enum Response {
    OnlinePlay,
    Tutorial,
    Sandbox,
    Challenges,
    Quit,
}
