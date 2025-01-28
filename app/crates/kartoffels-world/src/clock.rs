use anyhow::Result;
use bevy_ecs::system::{Res, ResMut, Resource};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde::Serialize;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

#[derive(Clone, Debug, Default, PartialEq, Eq, Resource, Serialize)]
pub enum Clock {
    /// Simulates at 64k bot-ticks per second
    #[default]
    Normal,

    /// Simulates 128k bot-ticks per second
    Fast,

    /// Simulates 256k bot-ticks per second
    Faster,

    /// Simulates as many ticks per second as the server can do
    Unlimited,

    /// Manual clock, requires calling [`Handle::tick()`] for world to progress;
    /// useful for testing.
    Manual { now: DateTime<Utc> },
}

impl Clock {
    pub(crate) const HZ: u32 = 64_000;

    pub fn manual() -> Self {
        Self::Manual {
            now: Utc.from_utc_datetime(&NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2018, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            )),
        }
    }

    pub(crate) fn now(&self) -> DateTime<Utc> {
        if let Self::Manual { now } = self {
            *now
        } else {
            Utc::now()
        }
    }

    pub(crate) fn metronome(&self) -> Metronome {
        Metronome::new(Self::HZ)
    }

    /// How many bot-ticks to simulate at once.
    ///
    /// This is a performance optimization that allows us to amortize the cost
    /// of other systems in relation to `bots::tick()` - i.e. in practice it
    /// makes sense to run `bots::tick()` more than once, since this allows us
    /// to utilize CPU caches etc. better.
    pub(crate) fn ticks(&self) -> u32 {
        match self {
            Clock::Manual { .. } => 1,
            _ => 32,
        }
    }

    fn speed(&self) -> Option<u32> {
        match self {
            Clock::Normal => Some(1),
            Clock::Fast => Some(2),
            Clock::Faster => Some(4),
            Clock::Unlimited | Clock::Manual { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Resource)]
pub struct Metronome {
    interval: Duration,
    deadline: Instant,
}

impl Metronome {
    pub fn new(hz: u32) -> Self {
        Self {
            interval: Duration::from_secs(1) / hz,
            deadline: Instant::now(),
        }
    }

    pub fn sleep(&mut self, clock: &Clock) {
        let Some(speed) = clock.speed() else {
            return;
        };

        self.deadline += clock.ticks() * self.interval / speed;

        let now = Instant::now();

        if let Some(delay) = self.deadline.checked_duration_since(now) {
            thread::sleep(delay);
        }
    }

    pub fn measure<T>(f: impl FnOnce() -> T) -> (T, Duration) {
        let tt = Instant::now();
        let result = f();

        (result, tt.elapsed())
    }

    pub fn try_measure<T>(
        f: impl FnOnce() -> Result<T>,
    ) -> Result<(T, Duration)> {
        let (result, tt) = Self::measure(f);

        Ok((result?, tt))
    }
}

#[derive(Debug, Default, Resource)]
pub struct Fuel {
    remaining: u32,
    callback: Option<oneshot::Sender<()>>,
}

impl Fuel {
    pub fn set(&mut self, fuel: u32, callback: oneshot::Sender<()>) {
        assert!(self.remaining == 0 && self.callback.is_none());

        self.remaining = fuel;
        self.callback = Some(callback);
    }

    pub fn tick(&mut self, clock: &Clock) {
        self.remaining = self.remaining.saturating_sub(clock.ticks());

        if self.is_empty()
            && let Some(callback) = self.callback.take()
        {
            _ = callback.send(());
        }
    }

    pub fn is_empty(&self) -> bool {
        self.remaining == 0
    }
}

pub fn sleep(clock: Res<Clock>, mut mtr: ResMut<Metronome>) {
    mtr.sleep(&clock);
}
