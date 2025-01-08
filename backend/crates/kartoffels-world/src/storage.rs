mod header;
mod migrations;
mod systems;

use self::header::*;
pub use self::systems::*;
use crate::{Bots, Map, Policy, Theme};
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedWorld<'a> {
    pub bots: MaybeOwned<'a, Bots>,
    pub map: MaybeOwned<'a, Map>,
    pub name: MaybeOwned<'a, String>,
    pub policy: MaybeOwned<'a, Policy>,
    pub theme: Option<MaybeOwned<'a, Theme>>,
}
