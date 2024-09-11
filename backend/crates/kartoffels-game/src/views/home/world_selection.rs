use super::Background;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{theme, Button, Term};
use kartoffels_world::prelude::Handle as WorldHandle;
use ratatui::text::Line;
use termwiz::input::KeyCode;
use tokio::time;

pub async fn run(
    term: &mut Term,
    store: &Store,
    bg: &mut Background,
) -> Result<Response> {
    loop {
        let resp = term
            .draw(|ui| {
                let width = 40;
                let height = (store.worlds.len() + 4) as u16;

                bg.render(ui);

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
                            ui.throw(Response::Some(world.to_owned()));
                        }
                    }

                    ui.space(1);

                    Button::new(KeyCode::Escape, "go back")
                        .throwing(Response::None)
                        .centered()
                        .render(ui);
                });

                ui.catch()
            })
            .await?
            .flatten();

        if let Some(resp) = resp {
            time::sleep(theme::INTERACTION_TIME).await;

            return Ok(resp);
        }

        term.poll().await?;
    }
}

#[derive(Debug)]
pub enum Response {
    Some(WorldHandle),
    None,
}
