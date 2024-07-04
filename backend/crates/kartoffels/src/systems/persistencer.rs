use crate::{SerializedWorld, World};
use anyhow::{Context, Result};
use futures_util::FutureExt;
use maybe_owned::MaybeOwned;
use std::future::Future;
use std::time::{Duration, Instant};
use tracing::{debug, info};

pub struct Persistencer {
    task: Option<Box<dyn Future<Output = Result<()>> + Unpin>>,
    next_tick_at: Instant,
}

impl Persistencer {
    pub fn new() -> Self {
        Self {
            task: None,
            next_tick_at: Self::next_tick(),
        }
    }

    pub fn tick(&mut self, world: &World) {
        let Some(path) = &world.path else {
            return;
        };

        if Instant::now() < self.next_tick_at {
            return;
        }

        debug!("saving world");

        if let Some(task) = self.task.take() {
            task.now_or_never()
                .expect("the previous save is still in progress - has the I/O stalled?")
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

        self.task = Some(Box::new(task));
        self.next_tick_at = Self::next_tick();
    }

    fn next_tick() -> Instant {
        Instant::now() + Duration::from_secs(15)
    }
}
