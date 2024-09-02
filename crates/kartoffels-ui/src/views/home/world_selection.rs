use crate::{theme, Button, Clear, Term};
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_world::prelude::Handle as WorldHandle;
use ratatui::text::Line;
use termwiz::input::KeyCode;
use tokio::time;

pub async fn run(term: &mut Term, store: &Store) -> Result<Response> {
    loop {
        let mut response = None;

        term.draw(|ui| {
            Clear::render(ui);

            let width = 40;
            let height = (store.worlds.len() + 5) as u16;

            ui.info_dialog(width, height, Some(" play "), |ui| {
                ui.line(Line::raw("choose world:").centered());
                ui.space(1);

                for (idx, world) in store.worlds.iter().enumerate() {
                    let key = KeyCode::Char((b'1' + (idx as u8)) as char);

                    if Button::new(key, world.name())
                        .centered()
                        .block()
                        .render(ui)
                        .pressed
                    {
                        response = Some(Response::Play(world.to_owned()));
                    }
                }

                ui.space(1);

                if Button::new(KeyCode::Char('t'), "tutorial")
                    .centered()
                    .block()
                    .render(ui)
                    .pressed
                {
                    response = Some(Response::OpenTutorial);
                }

                if Button::new(KeyCode::Escape, "go back")
                    .centered()
                    .render(ui)
                    .pressed
                {
                    response = Some(Response::GoBack);
                }
            });
        })
        .await?;

        if let Some(response) = response {
            time::sleep(theme::INTERACTION_TIME).await;

            return Ok(response);
        }

        term.tick().await?;
    }
}

#[derive(Debug)]
pub enum Response {
    Play(WorldHandle),
    OpenTutorial,
    GoBack,
}
