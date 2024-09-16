use crate::{Clock, ModeConfig, Policy, ThemeConfig};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub clock: Clock,
    pub mode: ModeConfig,
    pub name: String,
    pub path: Option<PathBuf>,
    pub policy: Policy,
    pub rng: Option<<SmallRng as SeedableRng>::Seed>,
    pub theme: ThemeConfig,
}
