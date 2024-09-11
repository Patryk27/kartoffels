mod header;
mod menu;

use self::header::*;
use self::menu::*;
use crate::Background;
use anyhow::Result;
use kartoffels_ui::Term;
use ratatui::layout::{Constraint, Layout};

pub async fn run(term: &mut Term, bg: &mut Background) -> Result<Response> {
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
                    Constraint::Length(Menu::height(ui)),
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
                    Menu::render(ui);
                });

                ui.catch()
            })
            .await?
            .flatten();

        term.poll().await?;

        if let Some(resp) = resp {
            return Ok(resp);
        }
    }
}

#[derive(Debug)]
pub enum Response {
    Play,
    Tutorial,
    Quit,
}
