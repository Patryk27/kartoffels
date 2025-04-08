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
