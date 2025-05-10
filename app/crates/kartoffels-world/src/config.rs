use crate::{Clock, Policy, Theme};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub const EVENT_STREAM_CAPACITY: usize = 128;
pub const REQUEST_STREAM_CAPACITY: usize = 128;
pub const MAX_LIVES_PER_BOT: usize = 128;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub clock: Clock,
    pub events: bool,
    pub name: String,
    pub policy: Policy,
    pub seed: Option<<ChaCha8Rng as SeedableRng>::Seed>,
    pub theme: Option<Theme>,
}
