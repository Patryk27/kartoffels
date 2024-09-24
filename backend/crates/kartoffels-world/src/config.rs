use crate::{Clock, Mode, Policy, Theme};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub clock: Clock,
    pub mode: Mode,
    pub name: String,
    pub path: Option<PathBuf>,
    pub policy: Policy,
    pub rng: Option<<SmallRng as SeedableRng>::Seed>,
    pub theme: Theme,
}
