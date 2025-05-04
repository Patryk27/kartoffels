mod arena;
mod cave;

pub use self::arena::*;
pub use self::cave::*;
use crate::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Theme {
    #[serde(rename = "arena")]
    Arena(ArenaTheme),

    #[serde(rename = "cave")]
    Cave(CaveTheme),
}

impl Theme {
    pub async fn build(
        &self,
        rng: &mut impl RngCore,
        map: MapBuilder,
    ) -> Result<Map> {
        match self {
            Theme::Arena(this) => this.build(rng, map).await,
            Theme::Cave(this) => this.build(rng, map).await,
        }
    }
}
