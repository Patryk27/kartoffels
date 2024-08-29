use crate::{Action, Clear, LayoutExt, Prompt, RectExt, Term};
use anyhow::Result;
use itertools::Either;
use kartoffels_store::Store;
use kartoffels_world::prelude::Handle as WorldHandle;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Text};
use std::iter;
use termwiz::input::{InputEvent, KeyCode, Modifiers};
use tokio::select;

pub async fn run(term: &mut Term, store: &Store) -> Result<Outcome> {
    let mut prompt = Prompt::default();

    loop {
        term.draw(|f| {
            let menu = menu(store);

            let area = Layout::dialog(
                Constraint::Length(menu.width() as u16 + 4),
                Constraint::Length(menu.height() as u16 + 2),
                f.area(),
            );

            f.render_widget(Clear, f.area());
            f.render_widget(menu.centered(), area);
            f.render_widget(prompt.as_line().centered(), area.footer());
        })
        .await?;

        let event = select! {
            event = term.read() => Either::Left(event?),
            _ = prompt.tick() => Either::Right(()),
        };

        if let Either::Left(Some(InputEvent::Key(event))) = event {
            match (event.key, event.modifiers) {
                (KeyCode::Char(ch @ '1'..='9'), Modifiers::NONE) => {
                    let world_idx = (ch as u8 - b'1') as usize;

                    if let Some((_, world)) = store.worlds.get(world_idx) {
                        return Ok(Outcome::Play(world.to_owned()));
                    }
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
    Play(WorldHandle),
    Quit,
}

fn menu(store: &Store) -> Text {
    let worlds = store.worlds.iter().enumerate().map(|(idx, (_, world))| {
        Action::new((idx + 1).to_string(), world.name(), true).into()
    });

    iter::once(Line::raw("choose world:"))
        .chain([Line::raw("")])
        .chain(worlds)
        .chain([Line::raw("")])
        .chain([Action::new("esc", "go back", true).into()])
        .collect()
}
