use crate::Clock;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Duration, counted in world-ticks.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub struct Ticks(u64);

impl Ticks {
    pub fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Returns this duration in ticks.
    pub fn as_ticks(&self) -> u64 {
        self.0
    }

    /// Returns this duration in seconds.
    pub fn as_seconds(&self) -> u64 {
        self.as_ticks() / (Clock::HZ as u64)
    }

    /// Returns this duration in human-readable time, like `2d3h`.
    ///
    /// Optional `width` parameter can be used to constrain the string into a
    /// maximum width, e.g. `12d5h` limited to `width = 3` will say just `12d`.
    pub fn as_time(&self, width: impl Into<Option<u32>>) -> impl fmt::Display {
        let s = self.as_seconds();
        let width = width.into().unwrap_or(99);

        fmt::from_fn(move |f| {
            if s == 0 {
                write!(f, "{s}s")?;
                return Ok(());
            }

            let m = s / 60;
            let h = m / 60;
            let d = h / 24;
            let parts = [(d, 'd'), (h % 24, 'h'), (m % 60, 'm'), (s % 60, 's')];

            let mut width = width;

            for (amt, unit) in parts {
                if amt == 0 {
                    continue;
                }

                width = if let Some(w) = width.checked_sub(amt.ilog10() + 2) {
                    w
                } else if unit == 'd' {
                    0
                } else {
                    return Ok(());
                };

                write!(f, "{amt}{unit}")?;
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn ts(d: u64, h: u64, m: u64, s: u64) -> u64 {
        d * 24 * 60 * 60 + h * 60 * 60 + m * 60 + s
    }

    #[test_case(0, None, "0s")]
    // Seconds:
    #[test_case(ts(0, 0, 0, 1), None, "1s")]
    #[test_case(ts(0, 0, 0, 30), None, "30s")]
    #[test_case(ts(0, 0, 0, 59), None, "59s")]
    #[test_case(ts(0, 0, 0, 60), None, "1m")]
    // Minutes:
    #[test_case(ts(0, 0, 1, 0), None, "1m")]
    #[test_case(ts(0, 0, 30, 0), None, "30m")]
    #[test_case(ts(0, 0, 59, 0), None, "59m")]
    #[test_case(ts(0, 0, 60, 0), None, "1h")]
    // Hours:
    #[test_case(ts(0, 1, 0, 0), None, "1h")]
    #[test_case(ts(0, 16, 0, 0), None, "16h")]
    #[test_case(ts(0, 23, 0, 0), None, "23h")]
    #[test_case(ts(0, 24, 0, 0), None, "1d")]
    // Days:
    #[test_case(ts(1, 0, 0, 0), None, "1d")]
    #[test_case(ts(6, 0, 0, 0), None, "6d")]
    #[test_case(ts(123, 0, 0, 0), None, "123d")]
    // Mixed:
    #[test_case(ts(1, 0, 0, 2), None, "1d2s")]
    #[test_case(ts(1, 0, 2, 0), None, "1d2m")]
    #[test_case(ts(1, 2, 0, 0), None, "1d2h")]
    #[test_case(ts(1, 2, 3, 4), None, "1d2h3m4s")]
    // Mixed, with limited width:
    #[test_case(ts(1, 2, 3, 4), Some(0), "1d")]
    #[test_case(ts(1, 2, 3, 4), Some(1), "1d")]
    #[test_case(ts(1, 2, 3, 4), Some(2), "1d")]
    #[test_case(ts(1, 2, 3, 4), Some(3), "1d")]
    #[test_case(ts(1, 2, 3, 4), Some(4), "1d2h")]
    #[test_case(ts(1, 2, 3, 4), Some(5), "1d2h")]
    #[test_case(ts(1, 2, 3, 4), Some(6), "1d2h3m")]
    #[test_case(ts(1, 2, 3, 4), Some(7), "1d2h3m")]
    #[test_case(ts(1, 2, 3, 4), Some(8), "1d2h3m4s")]
    #[test_case(ts(1, 20, 3, 4), Some(4), "1d")]
    #[test_case(ts(1, 20, 3, 4), Some(5), "1d20h")]
    fn as_time(given: u64, width: Option<u32>, expected: &str) {
        let actual = Ticks::new(given * (Clock::HZ as u64))
            .as_time(width)
            .to_string();

        assert_eq!(expected, actual);
    }
}
