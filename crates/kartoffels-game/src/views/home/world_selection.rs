use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{theme, Button, Clear, Term};
use kartoffels_world::prelude::Handle as WorldHandle;
use ratatui::text::Line;
use termwiz::input::KeyCode;
use tokio::time;

pub async fn run(term: &mut Term, store: &Store) -> Result<Response> {
    loop {
        let mut resp = None;

        term.draw(|ui| {
            Clear::render(ui);

            let width = 40;
            let height = (store.worlds.len() + 4) as u16;

            ui.info_window(width, height, Some(" online play "), |ui| {
                ui.line(Line::raw("choose world:").centered());
                ui.space(1);

                for (idx, world) in store.worlds.iter().enumerate() {
                    let key = KeyCode::Char((b'1' + (idx as u8)) as char);

                    if Button::new(key, world.name())
                        .centered()
                        .render(ui)
                        .pressed
                    {
                        resp = Some(Response::OnlinePlay(world.to_owned()));
                    }
                }

                ui.space(1);

                if Button::new(KeyCode::Escape, "go back")
                    .centered()
                    .render(ui)
                    .pressed
                {
                    resp = Some(Response::GoBack);
                }
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
    OnlinePlay(WorldHandle),
    GoBack,
}
