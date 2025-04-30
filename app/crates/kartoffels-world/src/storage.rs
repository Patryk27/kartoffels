mod header;
mod migrations;
mod systems;

use self::header::*;
pub use self::systems::*;
use crate::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedWorld<'a> {
    pub bots: MaybeOwned<'a, Bots>,
    pub lives: MaybeOwned<'a, Lives>,
    pub map: MaybeOwned<'a, Map>,
    pub name: MaybeOwned<'a, String>,
    pub policy: MaybeOwned<'a, Policy>,
    pub rng: MaybeOwned<'a, ChaCha8Rng>,
    pub theme: Option<MaybeOwned<'a, Theme>>,
}
