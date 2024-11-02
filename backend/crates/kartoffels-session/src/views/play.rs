use crate::Background;
use anyhow::Result;
use kartoffels_store::Store;
use kartoffels_ui::{Button, Fade, FadeDir, Render, Term};
use kartoffels_world::prelude::Handle as WorldHandle;
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(
    store: &Store,
    term: &mut Term,
    bg: &Background,
    fade_in: bool,
) -> Result<Response> {
    debug!("run()");

    let mut fade_in = if fade_in && !store.testing() {
        Some(Fade::new(FadeDir::In))
    } else {
        None
    };

    let mut fade_out: Option<(Fade, Response)> = None;

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
                            ui.throw(Response::Play(world.clone()));
                        }
                    }

                    ui.space(1);

                    Button::new(KeyCode::Escape, "go back")
                        .throwing(Response::GoBack)
                        .render(ui);
                });

                if let Some(fade) = &fade_in {
                    if fade.render(ui).is_completed() {
                        fade_in = None;
                    }
                }

                if let Some((fade, _)) = &fade_out {
                    fade.render(ui);
                }
            })
            .await?;

        term.poll().await?;

        if let Some((fade, resp)) = &fade_out {
            if fade.is_completed() {
                return Ok(resp.clone());
            }
        }

        if let Some(resp) = resp {
            if resp.fade_out() && !store.testing() {
                fade_out = Some((Fade::new(FadeDir::Out), resp));
            } else {
                return Ok(resp);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Response {
    Play(WorldHandle),
    GoBack,
}

impl Response {
    fn fade_out(&self) -> bool {
        matches!(self, Response::Play(_))
    }
}
