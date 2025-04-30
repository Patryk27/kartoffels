use crate::*;

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
