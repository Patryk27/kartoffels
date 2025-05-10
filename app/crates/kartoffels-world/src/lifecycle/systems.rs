use crate::{Clock, World};
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

pub fn log(world: &mut World) {
    let state = world.states.get_mut::<State>();

    state.ticks += world.clock.ticks();

    if let Clock::Manual { .. } = world.clock {
        return;
    }

    if Instant::now() < state.next_run_at {
        return;
    }

    let alive = format!(
        "{}/{}",
        world.bots.alive.count(),
        world.policy.max_alive_bots
    );

    let queued = format!(
        "{}/{}",
        world.bots.queued.len(),
        world.policy.max_queued_bots
    );

    let conns = world.snapshots.tx.receiver_count();
    let vcpu = format!("{} khz", state.ticks / 1_000);

    debug!(?alive, ?queued, ?conns, ?vcpu);

    state.ticks = 0;
    state.next_run_at = next_run_at();
}

fn next_run_at() -> Instant {
    Instant::now() + Duration::from_secs(1)
}
