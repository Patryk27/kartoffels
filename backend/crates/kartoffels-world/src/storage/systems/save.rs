use crate::{SerializedWorld, World};
use anyhow::{Context, Result};
use futures_util::FutureExt;
use maybe_owned::MaybeOwned;
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::{runtime, task};
use tracing::{debug, Instrument};

pub struct State {
    task: Option<Box<dyn Future<Output = Result<()>> + Send + Unpin>>,
    next_run_at: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            task: Default::default(),
            next_run_at: next_run_at(),
        }
    }
}

pub fn run(world: &mut World, state: &mut State) {
    if Instant::now() < state.next_run_at {
        return;
    }

    state.next_run_at = next_run_at();

    run_now(world, state, false);
}

pub fn run_now(world: &mut World, state: &mut State, wait: bool) {
    let Some(path) = &world.path else {
        return;
    };

    debug!("saving world");

    if let Some(task) = state.task.take() {
        if wait {
            runtime::Handle::current().block_on(task).unwrap();
        } else {
            task.now_or_never()
                .expect(
                    "the previous save is still in progress - has the I/O \
                     stalled?",
                )
                .unwrap();
        }
    }

    let world = SerializedWorld {
        bots: MaybeOwned::Borrowed(&world.bots),
        clock: MaybeOwned::Borrowed(&world.clock),
        map: MaybeOwned::Borrowed(&world.map),
        mode: MaybeOwned::Borrowed(&world.mode),
        name: MaybeOwned::Borrowed(&world.name),
        policy: MaybeOwned::Borrowed(&world.policy),
        theme: world.theme.as_ref().map(MaybeOwned::Borrowed),
    };

    let task = world.store(path).expect("couldn't save the world");

    let task = task::spawn(
        async move {
            let (tt_ser, tt_io) = task.await?;

            debug!(?tt_ser, ?tt_io, "saved");

            Ok(())
        }
        .in_current_span(),
    )
    .map(|result| result.context("task crashed")?);

    if wait {
        runtime::Handle::current().block_on(task).unwrap();
    } else {
        state.task = Some(Box::new(task));
    }
}

fn next_run_at() -> Instant {
    Instant::now() + Duration::from_secs(60)
}
