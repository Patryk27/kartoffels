mod ctrl;

use crate::views::game;
use crate::views::index::WINDOW_WIDTH;
use crate::{BgMap, Button, Fade, Frame, UiWidget};
use anyhow::Result;
use itertools::Itertools;
use kartoffels_store::{Session, Store, World, WorldVis};
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
        debug!("run()");

        match main(store, frame, bg, fade_in).await? {
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

async fn main(
    store: &Store,
    frame: &mut Frame,
    bg: &BgMap,
    fade_in: bool,
) -> Result<Event> {
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
        Button::new("exit", KeyCode::Escape).throwing(Event::GoBack);

    let width = WINDOW_WIDTH;
    let height = world_btns.len() as u16 + 2;

    let mut fade = Fade::new(store, fade_in);

    loop {
        let event = frame
            .render(|ui| {
                bg.render(ui);

                ui.imodal(width, height, Some("play"), |ui| {
                    for btn in &mut world_btns {
                        ui.add(btn);
                    }

                    ui.space(1);
                    ui.add(&mut go_back_btn);
                });

                fade.render(ui);
            })
            .await?;

        if let Some(event @ Event::Play(_)) = event {
            fade.out(event);
            continue;
        }

        if let Some(event) = fade.poll().or(event) {
            return Ok(event);
        }
    }
}

#[derive(Debug)]
enum Event {
    Play(World),
    GoBack,
}
