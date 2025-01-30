use crate::{Clock, Policy, Theme};
use kartoffels_utils::Id;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub clock: Clock,
    pub events: bool,
    pub id: Option<Id>,
    pub name: String,
    pub path: Option<PathBuf>,
    pub policy: Policy,
    pub seed: Option<<ChaCha8Rng as SeedableRng>::Seed>,
    pub theme: Option<Theme>,
}

impl Config {
    pub(crate) fn validate(&self) {
        // We store bot indices into map's tile metadata and since those are u8,
        // we can't have than 256 bots
        assert!(self.policy.max_alive_bots <= 256);
        assert!(self.policy.max_queued_bots <= 256);
    }
}
