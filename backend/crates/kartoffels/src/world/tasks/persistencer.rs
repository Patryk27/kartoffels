use crate::world::SerializedWorld;
use crate::World;
use anyhow::{Context, Result};
use maybe_owned::MaybeOwned;
use std::time::{Duration, Instant};
use tracing::{debug, info};

#[derive(Debug)]
pub struct Persistencer {
    next_tick_at: Instant,
}

impl Persistencer {
    pub fn new() -> Self {
        Self {
            next_tick_at: Self::next_tick(),
        }
    }

    pub fn tick(&mut self, world: &World) {
        if Instant::now() < self.next_tick_at {
            return;
        }

        Self::save(world).unwrap();

        self.next_tick_at = Self::next_tick();
    }

    pub fn save(world: &World) -> Result<()> {
        let Some(path) = &world.path else {
            return Ok(());
        };

        debug!("saving world");

        let world = SerializedWorld {
            name: MaybeOwned::Borrowed(&world.name),
            mode: MaybeOwned::Borrowed(&world.mode),
            theme: MaybeOwned::Borrowed(&world.theme),
            policy: MaybeOwned::Borrowed(&world.policy),
            map: MaybeOwned::Borrowed(&world.map),
            bots: MaybeOwned::Borrowed(&world.bots),
        };

        let tt = Instant::now();

        world.store(path).context("couldn't save the world")?;

        info!(tt = ?tt.elapsed(), "world saved");

        Ok(())
    }

    fn next_tick() -> Instant {
        Instant::now() + Duration::from_secs(30)
    }
}
