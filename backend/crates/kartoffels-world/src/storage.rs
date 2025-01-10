mod header;
mod migrations;
mod systems;

use self::header::*;
pub use self::systems::*;
use crate::{Bots, Map, Policy, Runs, Theme};
use maybe_owned::MaybeOwned;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedWorld<'a> {
    pub bots: MaybeOwned<'a, Bots>,
    pub map: MaybeOwned<'a, Map>,
    pub name: MaybeOwned<'a, String>,
    pub policy: MaybeOwned<'a, Policy>,
    pub rng: MaybeOwned<'a, ChaCha8Rng>,
    pub runs: MaybeOwned<'a, Runs>,
    pub theme: Option<MaybeOwned<'a, Theme>>,
}
