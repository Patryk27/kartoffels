use crate::{cfg, World};
use std::time::{Duration, Instant};
use tracing::debug;

struct State {
    ticks: u32,
    next_run_at: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ticks: 0,
            next_run_at: next_run_at(),
        }
    }
}

pub fn run(world: &mut World) {
    let state = world.systems.get_mut::<State>();

    state.ticks += cfg::SIM_TICKS;

    if Instant::now() < state.next_run_at {
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
        format!("connections = {}", world.updates.receiver_count()),
        format!("vcpu = {} khz", state.ticks / 1_000),
    ];

    debug!(target: "kartoffels", "status:");

    for msg in msgs {
        debug!(target: "kartoffels", "> {}", msg);
    }

    state.ticks = 0;
    state.next_run_at = next_run_at();
}

fn next_run_at() -> Instant {
    Instant::now() + Duration::from_secs(1)
}
