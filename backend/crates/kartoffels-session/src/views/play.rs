use crate::Background;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{Button, Term};
use kartoffels_world::prelude::Handle as WorldHandle;
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(
    term: &mut Term,
    store: &Store,
    bg: &Background,
) -> Result<Response> {
    debug!("run()");

    loop {
        let resp = term
            .draw(|ui| {
                let width = 40;
                let mut height = 3;

                if !store.worlds.is_empty() {
                    height += store.worlds.len() as u16 + 1;
                }

                bg.render(ui);

                ui.info_window(width, height, Some(" play "), |ui| {
                    if !store.worlds.is_empty() {
                        for (idx, world) in store.worlds.iter().enumerate() {
                            let key =
                                KeyCode::Char((b'1' + (idx as u8)) as char);

                            if Button::new(key, world.name())
                                .centered()
                                .render(ui)
                                .pressed
                            {
                                ui.throw(Response::Play(world.to_owned()));
                            }
                        }

                        ui.space(1);
                    }

                    Button::new(KeyCode::Char('s'), "sandbox")
                        .throwing(Response::Sandbox)
                        .centered()
                        .render(ui);

                    Button::new(KeyCode::Char('c'), "challenges")
                        .throwing(Response::Challenges)
                        .centered()
                        .render(ui);

                    Button::new(KeyCode::Escape, "go back")
                        .throwing(Response::GoBack)
                        .centered()
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
    Sandbox,
    Challenges,
    GoBack,
}
