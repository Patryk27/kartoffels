mod ctrl;

use crate::views::game;
use crate::Background;
use anyhow::Result;
use kartoffels_store::{SessionId, Store};
use kartoffels_ui::{Button, Fade, FadeDir, Render, Term};
use kartoffels_world::prelude::Handle as WorldHandle;
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: SessionId,
    term: &mut Term,
    bg: &Background,
) -> Result<()> {
    let mut fade_in = false;

    loop {
        match run_once(store, term, bg, fade_in).await? {
            Event::Play(world) => {
                game::run(store, sess, term, |game| {
                    ctrl::run(world.clone(), game)
                })
                .await?;

                fade_in = true;
            }

            Event::GoBack => {
                return Ok(());
            }
        }
    }
}

async fn run_once<'a>(
    store: &'a Store,
    term: &mut Term,
    bg: &Background,
    fade_in: bool,
) -> Result<Event<'a>> {
    debug!("run()");

    let mut fade_in = if fade_in && !store.testing() {
        Some(Fade::new(FadeDir::In))
    } else {
        None
    };

    let mut fade_out: Option<(Fade, Event)> = None;

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

                        Button::new(key, world.name())
                            .throwing(Event::Play(world))
                            .render(ui);
                    }

                    ui.space(1);

                    Button::new(KeyCode::Escape, "go back")
                        .throwing(Event::GoBack)
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
enum Event<'a> {
    Play(&'a WorldHandle),
    GoBack,
}

impl Event<'_> {
    fn fade_out(&self) -> bool {
        matches!(self, Event::Play(_))
    }
}
