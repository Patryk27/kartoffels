use anyhow::Result;
use chrono::TimeDelta;
use std::thread;
use web_time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct Metronome {
    interval: Duration,
    backlog: TimeDelta,
    now: Instant,
}

impl Metronome {
    pub fn new(hz: u32, ticks: u32) -> Self {
        let interval = Duration::from_nanos(
            Duration::from_secs(1).as_nanos() as u64 / (hz as u64)
                * (ticks as u64),
        );

        Self {
            interval,
            backlog: Default::default(),
            now: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        self.backlog += TimeDelta::nanoseconds(
            self.interval.as_nanos() as i64
                - self.now.elapsed().as_nanos() as i64,
        );

        if self.backlog.num_seconds() != 0 {
            self.backlog =
                TimeDelta::seconds(self.backlog.num_seconds().signum());
        }
    }

    pub fn wait(&mut self) {
        if self.backlog.num_milliseconds() >= 2 {
            let (_, tt) = Self::measure(|| {
                thread::sleep(self.backlog.to_std().unwrap());
            });

            self.backlog -= TimeDelta::from_std(tt).unwrap();
            self.tick();
        }

        self.now = Instant::now();
    }

    pub fn interval(&self) -> Duration {
        self.interval
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
