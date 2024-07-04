use anyhow::Result;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct Metronome {
    max_iter_tt: i64,
    backlog: i64,
}

impl Metronome {
    const ONE_SECOND_NS: i64 = Duration::from_secs(1).as_nanos() as i64;

    pub fn new(hz: u32, ticks: u32) -> Self {
        let max_iter_tt =
            Duration::from_secs(1).as_nanos() / (hz as u128) * (ticks as u128);

        Self {
            max_iter_tt: max_iter_tt as i64,
            backlog: 0,
        }
    }

    pub fn iter<T>(&mut self, f: impl FnOnce(&Self) -> T) -> T {
        let (result, tt) = Self::measure_ns(|| f(self));

        self.backlog += tt - self.max_iter_tt;

        while self.backlog <= -2_000_000 {
            let (_, tt) = Self::measure_ns(|| {
                thread::sleep(Duration::from_millis(1));
            });

            self.backlog += tt;
        }

        self.backlog = self
            .backlog
            .clamp(-Self::ONE_SECOND_NS, Self::ONE_SECOND_NS);

        result
    }

    pub fn backlog_ms(&self) -> i64 {
        if self.backlog.abs() > 2_000_000 {
            self.backlog / 1_000_000
        } else {
            0
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

    fn measure_ns<T>(f: impl FnOnce() -> T) -> (T, i64) {
        let (result, tt) = Self::measure(f);

        (result, tt.as_nanos() as i64)
    }
}
