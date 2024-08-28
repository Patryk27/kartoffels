mod intro;
mod menu;

use self::intro::*;
use self::menu::*;
use crate::{Clear, Term};
use anyhow::Result;
use itertools::{Either, Itertools};
use kartoffels_store::Store;
use kartoffels_world::prelude::Handle as WorldHandle;
use ratatui::layout::{Constraint, Layout};
use std::time::Duration;
use termwiz::input::{InputEvent, KeyCode, Modifiers};
use tokio::{select, time};

#[derive(Debug)]
pub enum HomeOutcome {
    OpenTutorial,
    OpenChallenges,
    Play(WorldHandle),
    Quit,
}

pub async fn home(term: &mut Term, store: &Store) -> Result<HomeOutcome> {
    let worlds: Vec<_> = store
        .worlds
        .values()
        .sorted_by_key(|world| world.name())
        .collect();

    let mut blink = false;

    let mut blink_int = {
        let mut int = time::interval(Duration::from_millis(500));

        int.tick().await;
        int
    };

    loop {
        term.draw(|f| {
            let area = f.area();

            let menu = Menu {
                blink,
                worlds: &worlds,
            };

            let [_, main_area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(Intro::WIDTH),
                Constraint::Fill(1),
            ])
            .areas(area);

            let [_, intro_area, _, menu_area, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(Intro::HEIGHT),
                Constraint::Length(1),
                Constraint::Length(menu.height()),
                Constraint::Fill(2),
            ])
            .areas(main_area);

            f.render_widget(Clear, area);
            f.render_widget(Intro, intro_area);
            f.render_widget(menu, menu_area);
        })
        .await?;

        let event = select! {
            event = term.read() => Either::Left(event?),
            _ = blink_int.tick() => Either::Right(()),
        };

        match event {
            Either::Left(Some(InputEvent::Key(event))) => {
                match (event.key, event.modifiers) {
                    (KeyCode::Escape, _) => {
                        return Ok(HomeOutcome::Quit);
                    }

                    (KeyCode::Char('t'), Modifiers::NONE) => {
                        return Ok(HomeOutcome::OpenTutorial);
                    }

                    (KeyCode::Char('c'), Modifiers::NONE) => {
                        return Ok(HomeOutcome::OpenChallenges);
                    }

                    (KeyCode::Char(n @ '1'..='9'), Modifiers::NONE) => {
                        let idx = (n as u8) - b'1';

                        if let Some(world) = worlds.get(idx as usize) {
                            return Ok(HomeOutcome::Play((*world).clone()));
                        }
                    }

                    _ => (),
                }
            }

            Either::Left(_) => {
                //
            }

            Either::Right(_) => {
                blink = !blink;
            }
        }
    }
}
