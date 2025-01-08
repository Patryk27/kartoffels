use crate::{Clock, Policy, Theme};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub clock: Clock,
    pub emit_events: bool,
    pub name: String,
    pub path: Option<PathBuf>,
    pub policy: Policy,
    pub seed: Option<<SmallRng as SeedableRng>::Seed>,
    pub theme: Option<Theme>,
}

impl Config {
    pub(crate) fn validate(&self) {
        assert!(
            !(self.path.is_some() && self.seed.is_some()),
            "setting both `config.path` and `config.seed` is not supported, \
             because rng state is currently not persisted",
        );

        // We store bot indices into map's tile metadata and since those are u8,
        // we can't have than 256 bots
        assert!(self.policy.max_alive_bots <= 256);
        assert!(self.policy.max_queued_bots <= 256);
    }
}
