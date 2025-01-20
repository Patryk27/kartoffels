use crate::Clock;
use serde::{Deserialize, Serialize};
use std::fmt;

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

    pub fn ticks(&self) -> u64 {
        self.0
    }

    pub fn seconds(&self) -> u64 {
        self.ticks() / (Clock::HZ as u64)
    }

    pub fn time(&self) -> impl fmt::Display {
        let s = self.seconds();

        fmt::from_fn(move |f| {
            if s >= 60 {
                write!(f, "{}m{}s", s / 60, s % 60)
            } else {
                write!(f, "{s}s")
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(0, "0s")]
    #[test_case(1, "1s")]
    #[test_case(30, "30s")]
    #[test_case(59, "59s")]
    #[test_case(60, "1m0s")]
    #[test_case(61, "1m1s")]
    #[test_case(120, "2m0s")]
    #[test_case(123, "2m3s")]
    fn time(given: u64, expected: &str) {
        let actual = Ticks::new(given * (Clock::HZ as u64)).time().to_string();

        assert_eq!(expected, actual);
    }
}
