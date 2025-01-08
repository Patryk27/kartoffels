use crate::{Bots, Clock, Policy, Snapshots};
use bevy_ecs::system::{Local, Res};
use std::time::{Duration, Instant};
use tracing::debug;

pub struct State {
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

pub fn log(
    mut state: Local<State>,
    clock: Res<Clock>,
    bots: Res<Bots>,
    snapshots: Res<Snapshots>,
    policy: Res<Policy>,
) {
    state.ticks += 1;

    if let Clock::Manual = &*clock {
        return;
    }

    if Instant::now() < state.next_run_at {
        return;
    }

    let alive = format!("{}/{}", bots.alive.count(), policy.max_alive_bots);
    let queued = format!("{}/{}", bots.queued.len(), policy.max_queued_bots);
    let conns = snapshots.tx.receiver_count();
    let vcpu = format!("{} khz", state.ticks / 1_000);

    debug!(?alive, ?queued, ?conns, ?vcpu);

    state.ticks = 0;
    state.next_run_at = next_run_at();
}

fn next_run_at() -> Instant {
    Instant::now() + Duration::from_secs(1)
}
