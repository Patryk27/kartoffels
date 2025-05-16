mod ctrl;

use crate::views::game;
use crate::{BgMap, Button, FadeCtrl, FadeCtrlEvent, Frame, UiWidget};
use anyhow::Result;
use itertools::Itertools;
use kartoffels_store::{Session, Store, World, WorldVis};
use std::iter;
use termwiz::input::KeyCode;
use tracing::debug;

pub async fn run(
    store: &Store,
    sess: &Session,
    frame: &mut Frame,
    bg: &BgMap,
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
    bg: &BgMap,
    fade_in: bool,
) -> Result<Event> {
    debug!("run()");

    let worlds: Vec<_> = store
        .find_worlds(WorldVis::Public)
        .await?
        .into_iter()
        .map(|world| (world.name().clone(), world))
        .sorted_by(|(lhs_name, _), (rhs_name, _)| lhs_name.cmp(rhs_name))
        .collect();

    if worlds.is_empty() {
        return Ok(Event::GoBack);
    }

    let mut world_btns: Vec<_> = worlds
        .iter()
        .enumerate()
        .map(|(idx, (name, world))| {
            let key = KeyCode::Char((b'1' + (idx as u8)) as char);

            Button::new(name.as_str(), key)
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
            .tick(|ui| {
                fade.render(ui, |ui| {
                    bg.render(ui);

                    ui.imodal(width, height, Some(" play "), |ui| {
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
    Play(World),
    GoBack,
}

impl FadeCtrlEvent for Event {
    fn needs_fade_out(&self) -> bool {
        matches!(self, Event::Play(_))
    }
}
