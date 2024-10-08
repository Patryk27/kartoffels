mod metronome;

pub use self::metronome::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Clock {
    #[serde(rename = "auto")]
    Auto { hz: u32, steps: u32 },

    #[serde(rename = "manual")]
    Manual { steps: u32 },
}

impl Clock {
    pub(crate) fn metronome(&self, bench: bool) -> Option<Metronome> {
        match self {
            Clock::Auto { hz, steps } => {
                if bench {
                    None
                } else {
                    Some(Metronome::new(*hz, *steps))
                }
            }
            Clock::Manual { .. } => None,
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Clock::Auto {
            hz: 64_000,
            steps: 1_000,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ClockSpeed {
    #[default]
    Normal,
    Faster,
    Fastest,
}
