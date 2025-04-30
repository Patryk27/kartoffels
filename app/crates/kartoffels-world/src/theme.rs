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

impl FromStr for Theme {
    type Err = Error;

    fn from_str(spec: &str) -> Result<Self> {
        if let Some(spec) = spec.strip_prefix("arena:") {
            return ArenaTheme::from_str(spec).map(Theme::Arena);
        }

        if let Some(spec) = spec.strip_prefix("cave:") {
            return CaveTheme::from_str(spec).map(Theme::Cave);
        }

        Err(anyhow!("unknown theme"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::uvec2;

    #[test]
    fn from_str() {
        assert_eq!(
            Theme::Arena(ArenaTheme::new(123)),
            Theme::from_str("arena:radius=123").unwrap(),
        );

        assert_eq!(
            Theme::Cave(CaveTheme::new(uvec2(12, 34))),
            Theme::from_str("cave:width=12,height=34").unwrap(),
        );
    }
}
