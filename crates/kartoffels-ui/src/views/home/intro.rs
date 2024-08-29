mod header;
mod menu;

use self::header::*;
use self::menu::*;
use crate::{Clear, Prompt, Term};
use anyhow::Result;
use itertools::Either;
use ratatui::layout::{Constraint, Layout};
use termwiz::input::{InputEvent, KeyCode, Modifiers};
use tokio::select;

pub async fn run(term: &mut Term) -> Result<Outcome> {
    let mut prompt = Prompt::default();

    loop {
        term.draw(|f| {
            let area = f.area();

            let [_, area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(Header::WIDTH),
                Constraint::Fill(1),
            ])
            .areas(area);

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

            f.render_widget(Clear, f.area());
            f.render_widget(Header, header_area);
            f.render_widget(Menu, menu_area);
            f.render_widget(prompt.as_line().centered(), prompt_area);
        })
        .await?;

        let event = select! {
            event = term.read() => Either::Left(event?),
            _ = prompt.tick() => Either::Right(()),
        };

        if let Either::Left(Some(InputEvent::Key(event))) = event {
            match (event.key, event.modifiers) {
                (KeyCode::Char('p'), Modifiers::NONE) => {
                    return Ok(Outcome::Play);
                }
                (KeyCode::Char('t'), Modifiers::NONE) => {
                    return Ok(Outcome::SeeTutorial);
                }
                (KeyCode::Char('c'), Modifiers::NONE) => {
                    return Ok(Outcome::SeeChallenges);
                }
                (KeyCode::Escape, _) => {
                    return Ok(Outcome::Quit);
                }

                _ => (),
            }
        }
    }
}

#[derive(Debug)]
pub enum Outcome {
    Play,
    SeeTutorial,
    SeeChallenges,
    Quit,
}
