use crate::{SerializedWorld, Shutdown, World};
use anyhow::{Context, Result};
use futures_util::FutureExt;
use maybe_owned::MaybeOwned;
use std::future::Future;
use tracing::{debug, info};
use web_time::{Duration, Instant};

struct State {
    task: Option<Box<dyn Future<Output = Result<()>> + Send + Unpin>>,
    next_run_at: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            task: Default::default(),
            next_run_at: next_tick(),
        }
    }
}

pub fn run(world: &mut World) {
    let state = world.systems.get_mut::<State>();
    let shutdown = world.events.recv::<Shutdown>();

    if shutdown.is_some() {
        // HACK intercepting shutdown within this system feels pretty icky, but
        //      currently there's no better place to do this
        #[cfg(target_arch = "wasm32")]
        if let Some(interval) =
            world.platform.interval_handle.borrow_mut().take()
        {
            info!("clearing interval");

            web_sys::window()
                .expect("couldn't find window")
                .clear_interval_with_handle(interval);
        }
    }

    let Some(path) = &world.path else {
        if let Some(shutdown) = shutdown {
            _ = shutdown.tx.send(());
        }

        return;
    };

    if Instant::now() < state.next_run_at && shutdown.is_none() {
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

        if let Some(shutdown) = shutdown {
            info!("completing shutdown");

            _ = shutdown.tx.send(());
        }

        Ok(())
    })
    .map(|result| result.context("task crashed")?);

    state.task = Some(Box::new(task));
    state.next_run_at = next_tick();
}

fn next_tick() -> Instant {
    Instant::now() + Duration::from_secs(60)
}
