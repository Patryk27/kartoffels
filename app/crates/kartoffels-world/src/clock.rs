use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde::Serialize;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub enum Clock {
    /// Simulates 64k bot-ticks per second
    #[default]
    Normal,

    /// Simulates 128k bot-ticks per second
    Fast,

    /// Simulates 256k bot-ticks per second
    Faster,

    /// Simulates as many ticks per second as the server can do
    Unlimited,

    /// Manual clock, requires calling [`Handle::tick()`] for the world to
    /// progress; useful for testing.
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

    pub(crate) fn ticks(&self) -> u32 {
        match self {
            Self::Manual { .. } => 1,
            _ => 16,
        }
    }

    fn speed(&self) -> Option<u32> {
        match self {
            Self::Normal => Some(1),
            Self::Fast => Some(2),
            Self::Faster => Some(4),
            Self::Unlimited | Self::Manual { .. } => None,
        }
    }
}

#[derive(Clone, Debug)]
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
}

#[derive(Debug, Default)]
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
