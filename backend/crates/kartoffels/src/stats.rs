use crate::{cfg, World};
use tracing::info;
use web_time::{Duration, Instant};

struct State {
    ticks: u32,
    next_tick_at: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ticks: 0,
            next_tick_at: next_tick(),
        }
    }
}

pub fn run(world: &mut World) {
    let state = world.systems.get_mut::<State>();

    state.ticks += cfg::SIM_TICKS;

    if Instant::now() < state.next_tick_at {
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
        format!("connections = {}", world.conns.len()),
        format!("vcpu = {} khz", state.ticks / 1_000),
    ];

    info!(target: "kartoffels", "status:");

    for msg in msgs {
        info!(target: "kartoffels", "> {}", msg);
    }

    state.ticks = 0;
    state.next_tick_at = next_tick();
}

fn next_tick() -> Instant {
    Instant::now() + Duration::from_secs(1)
}
