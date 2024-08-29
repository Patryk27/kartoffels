use crate::{theme, Action, BlockExt, Clear, Prompt, Term};
use anyhow::Result;
use itertools::Either;
use kartoffels_store::Store;
use kartoffels_world::prelude::Handle as WorldHandle;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Padding};
use std::iter;
use termwiz::input::{InputEvent, KeyCode, Modifiers};
use tokio::select;

pub async fn run(term: &mut Term, store: &Store) -> Result<Outcome> {
    let mut prompt = Prompt::new();

    loop {
        term.draw(|f| {
            let area = f.area();
            let menu = menu(store);

            let [_, area, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(menu.height() as u16 + 2),
                Constraint::Fill(2),
            ])
            .areas(area);

            let [_, area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(menu.width() as u16 + 2 + 2),
                Constraint::Fill(1),
            ])
            .areas(area);

            f.render_widget(Clear, f.area());

            let area = Block::bordered()
                .border_style(Style::new().fg(theme::GREEN).bg(theme::BG))
                .padding(Padding::horizontal(1))
                .render_and_measure(area, f.buffer_mut());

            f.render_widget(menu.centered(), area);
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
