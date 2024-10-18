use crate::Background;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{Button, Render, Term};
use kartoffels_world::prelude::Handle as WorldHandle;
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(
    store: &Store,
    term: &mut Term,
    bg: &Background,
) -> Result<Response> {
    debug!("run()");

    loop {
        let resp = term
            .draw(|ui| {
                let width = store
                    .public_worlds()
                    .iter()
                    .map(|world| world.name().len() as u16 + 4)
                    .max()
                    .unwrap_or(0)
                    .max(11);

                let height = store.public_worlds().len() as u16 + 2;

                bg.render(ui);

                ui.info_window(width, height, Some(" play "), |ui| {
                    for (idx, world) in store.public_worlds().iter().enumerate()
                    {
                        let key = KeyCode::Char((b'1' + (idx as u8)) as char);

                        if Button::new(key, world.name()).render(ui).pressed {
                            ui.throw(Response::Play(world.to_owned()));
                        }
                    }

                    ui.space(1);

                    Button::new(KeyCode::Escape, "go back")
                        .throwing(Response::GoBack)
                        .render(ui);
                });
            })
            .await?;

        term.poll().await?;

        if let Some(resp) = resp {
            return Ok(resp);
        }
    }
}

#[derive(Debug)]
pub enum Response {
    Play(WorldHandle),
    GoBack,
}
