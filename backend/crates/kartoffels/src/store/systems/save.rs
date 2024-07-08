use crate::{SerializedWorld, World};
use anyhow::{Context, Result};
use futures_util::FutureExt;
use maybe_owned::MaybeOwned;
use std::future::Future;
use tracing::{debug, info};
use web_time::{Duration, Instant};

pub struct State {
    task: Option<Box<dyn Future<Output = Result<()>> + Send + Unpin>>,
    next_tick_at: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            task: Default::default(),
            next_tick_at: next_tick(),
        }
    }
}

pub fn run(world: &mut World, state: &mut State) {
    let Some(path) = &world.path else {
        return;
    };

    if Instant::now() < state.next_tick_at {
        return;
    }

    debug!("saving world");

    if let Some(task) = state.task.take() {
        task.now_or_never()
            .expect(
                "the previous save is still in progress - has the I/O stalled?",
            )
            .unwrap();
    }

    let world = SerializedWorld {
        name: MaybeOwned::Borrowed(&world.name),
        mode: MaybeOwned::Borrowed(&world.mode),
        theme: MaybeOwned::Borrowed(&world.theme),
        policy: MaybeOwned::Borrowed(&world.policy),
        map: MaybeOwned::Borrowed(&world.map),
        bots: MaybeOwned::Borrowed(&world.bots),
    };

    let task = world.store(path).expect("couldn't save the world");

    let task = tokio::spawn(async move {
        let (tt_ser, tt_io) = task.await?;

        info!(?tt_ser, ?tt_io, "world saved");

        Ok(())
    })
    .map(|result| result.context("task crashed")?);

    state.task = Some(Box::new(task));
    state.next_tick_at = next_tick();
}

fn next_tick() -> Instant {
    Instant::now() + Duration::from_secs(15)
}
