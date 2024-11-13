mod metronome;

pub use self::metronome::*;

#[derive(Clone, Copy, Debug, Default)]
pub enum Clock {
    #[default]
    Auto,
    Manual {
        ticks: u32,
    },
}

impl Clock {
    pub(crate) const HZ: u32 = 64_000;
    pub(crate) const TICKS: u32 = 256;

    pub(crate) fn metronome(&self) -> Option<Metronome> {
        match self {
            Clock::Auto => Some(Metronome::new(Self::HZ, Self::TICKS)),
            Clock::Manual { .. } => None,
        }
    }

    pub(crate) fn ticks(&self) -> u32 {
        match self {
            Clock::Auto => Self::TICKS,
            Clock::Manual { ticks } => *ticks,
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
