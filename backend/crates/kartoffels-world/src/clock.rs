mod metronome;

pub use self::metronome::*;

#[derive(Clone, Copy, Debug, Default)]
pub enum Clock {
    #[default]
    Auto,
    Manual {
        steps: u32,
    },
}

impl Clock {
    pub(crate) const HZ: u32 = 64_000;
    pub(crate) const STEPS: u32 = 256;

    pub(crate) fn metronome(&self) -> Option<Metronome> {
        match self {
            Clock::Auto => Some(Metronome::new(Self::HZ, Self::STEPS)),
            Clock::Manual { .. } => None,
        }
    }

    pub(crate) fn steps(&self) -> u32 {
        match self {
            Clock::Auto => Self::STEPS,
            Clock::Manual { steps } => *steps,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ClockSpeed {
    #[default]
    Normal,
    Faster,
    Fastest,
    Unlimited,
}
