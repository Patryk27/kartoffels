use super::Broadcaster;
use crate::world::Metronome;
use crate::World;
use std::time::{Duration, Instant};
use tracing::{info, warn};

#[derive(Debug)]
pub struct Statistician {
    ticks: u32,
    next_tick_at: Instant,
}

impl Statistician {
    pub fn new() -> Self {
        Self {
            ticks: 0,
            next_tick_at: Self::next_tick(),
        }
    }

    pub fn tick(
        &mut self,
        world: &World,
        mtr: &Metronome,
        bcaster: &Broadcaster,
    ) {
        self.ticks += World::SIM_TICKS;

        if Instant::now() < self.next_tick_at {
            return;
        }

        let msgs = [
            format!(
                "alive-bots = {} / {}",
                world.bots.alive.len(),
                world.policy.max_alive_bots
            ),
            format!(
                "queued-bots = {} / {}",
                world.bots.queued.len(),
                world.policy.max_queued_bots
            ),
            format!("connections = {}", bcaster.len()),
            format!("vcpu = {} khz", self.ticks / 1_000),
        ];

        info!(target: "kartoffels", "status:");

        for msg in msgs {
            info!(target: "kartoffels", "> {}", msg);
        }

        if mtr.backlog_ms() >= 500 {
            warn!(
                target: "kartoffels",
                "simulation is falling behind (vcpu = {} khz)",
                self.ticks / 1_000,
            );
        }

        self.ticks = 0;
        self.next_tick_at = Self::next_tick();
    }

    fn next_tick() -> Instant {
        Instant::now() + Duration::from_secs(1)
    }
}
