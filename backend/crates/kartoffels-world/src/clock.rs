mod metronome;

pub use self::metronome::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Clock {
    Auto { hz: u32, steps: u32 },
    Manual { steps: u32 },
}

impl Clock {
    pub(crate) fn metronome(&self) -> Option<Metronome> {
        match self {
            Clock::Auto { hz, steps } => Some(Metronome::new(*hz, *steps)),
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
