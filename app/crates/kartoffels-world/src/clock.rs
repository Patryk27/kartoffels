use anyhow::Result;
use bevy_ecs::system::{Res, ResMut, Resource};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Resource)]
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
    Manual,
}

impl Clock {
    pub(crate) const HZ: u32 = 64_000;

    pub(crate) fn metronome(&self) -> Metronome {
        Metronome::new(Self::HZ)
    }

    fn speed(&self) -> Option<u32> {
        match self {
            Clock::Normal => Some(1),
            Clock::Fast => Some(2),
            Clock::Faster => Some(4),
            Clock::Unlimited | Clock::Manual => None,
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

    pub fn sleep(&mut self, clock: Clock) {
        let Some(speed) = clock.speed() else {
            return;
        };

        self.deadline += self.interval / speed;

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
pub struct TickFuel {
    fuel: u32,
    callback: Option<oneshot::Sender<()>>,
}

impl TickFuel {
    pub fn set(&mut self, fuel: u32, callback: oneshot::Sender<()>) {
        assert!(self.fuel == 0 && self.callback.is_none());

        self.fuel = fuel;
        self.callback = Some(callback);
    }

    pub fn dec(&mut self) -> bool {
        if self.is_empty() {
            if let Some(callback) = self.callback.take() {
                _ = callback.send(());
            }

            false
        } else {
            self.fuel -= 1;
            true
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fuel == 0
    }
}

pub fn sleep(clock: Res<Clock>, mut mtr: ResMut<Metronome>) {
    mtr.sleep(*clock);
}
