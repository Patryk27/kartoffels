mod ctrl;

use crate::views::game;
use crate::Background;
use anyhow::Result;
use kartoffels_store::{Session, Store};
use kartoffels_ui::{
    Button, FadeCtrl, FadeCtrlEvent, Frame, KeyCode, UiWidget,
};
use kartoffels_world::prelude::Handle as WorldHandle;
use std::iter;
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    bg: &Background,
) -> Result<()> {
    let mut fade_in = false;

    loop {
        match run_once(store, frame, bg, fade_in).await? {
            Event::Play(world) => {
                game::run(store, sess, frame, |game| {
                    ctrl::run(sess, world, game)
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

async fn run_once(
    store: &Store,
    frame: &mut Frame,
    bg: &Background,
    fade_in: bool,
) -> Result<Event> {
    debug!("run()");

    let worlds = store.public_worlds();

    if worlds.is_empty() {
        return Ok(Event::GoBack);
    }

    let worlds: Vec<_> =
        worlds.iter().map(|world| (world, world.name())).collect();

    let mut world_btns: Vec<_> = worlds
        .iter()
        .enumerate()
        .map(|(idx, (world, world_name))| {
            let key = KeyCode::Char((b'1' + (idx as u8)) as char);

            Button::new(world_name.as_str(), key)
                .throwing(Event::Play((*world).clone()))
        })
        .collect();

    let mut go_back_btn =
        Button::new("go-back", KeyCode::Escape).throwing(Event::GoBack);

    let width = world_btns
        .iter()
        .chain(iter::once(&go_back_btn))
        .map(|btn| btn.width())
        .max()
        .unwrap();

    let height = world_btns.len() as u16 + 2;

    let mut fade = FadeCtrl::default()
        .animate(!store.testing())
        .fade_in(fade_in);

    loop {
        let event = frame
            .update(|ui| {
                fade.render(ui, |ui| {
                    bg.render(ui);

                    ui.info_window(width, height, Some(" play "), |ui| {
                        for btn in &mut world_btns {
                            ui.add(btn);
                        }

                        ui.space(1);
                        ui.add(&mut go_back_btn);
                    });
                });
            })
            .await?;

        if let Some(event) = event {
            return Ok(event);
        }
    }
}

#[derive(Clone, Debug)]
enum Event {
    Play(WorldHandle),
    GoBack,
}

impl FadeCtrlEvent for Event {
    fn needs_fade_out(&self) -> bool {
        matches!(self, Event::Play(_))
    }
}
