mod metronome;

pub use self::metronome::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
    pub(crate) const STEPS: u32 = 256;

    pub(crate) fn metronome(&self) -> Metronome {
        Metronome::new(Self::HZ, Self::STEPS)
    }

    pub(crate) fn steps(&self) -> u32 {
        match self {
            Clock::Normal | Clock::Fast | Clock::Faster | Clock::Unlimited => {
                Self::STEPS
            }
            Clock::Manual => 1024,
        }
    }

    fn speed(&self) -> Option<i64> {
        match self {
            Clock::Normal => Some(1),
            Clock::Fast => Some(2),
            Clock::Faster => Some(4),
            Clock::Unlimited | Clock::Manual => None,
        }
    }
}
