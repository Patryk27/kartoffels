mod header;
mod menu;

use self::header::*;
use self::menu::*;
use crate::{Clear, Prompt, Term};
use anyhow::Result;
use ratatui::layout::{Constraint, Layout};

pub async fn run(term: &mut Term) -> Result<Outcome> {
    let mut prompt = Prompt::default();

    loop {
        let mut outcome = None;

        term.draw(|ui| {
            let [_, area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(Header::WIDTH),
                Constraint::Fill(1),
            ])
            .areas(ui.area());

            let [_, header_area, _, menu_area, _, prompt_area, _] =
                Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(Header::HEIGHT),
                    Constraint::Length(1),
                    Constraint::Length(Menu::HEIGHT),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Fill(2),
                ])
                .areas(area);

            Clear::render(ui);

            ui.clamp(header_area, |ui| {
                Header::render(ui);
            });

            ui.clamp(menu_area, |ui| {
                outcome = Menu::render(ui);
            });

            ui.clamp(prompt_area, |ui| {
                prompt.render(ui);
            });
        })
        .await?;

        if let Some(outcome) = outcome {
            return Ok(outcome);
        }

        term.tick().await?;
    }
}

#[derive(Debug)]
pub enum Outcome {
    Play,
    OpenTutorial,
    OpenChallenges,
    Quit,
}
