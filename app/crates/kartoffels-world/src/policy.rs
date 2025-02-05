use crate::spec;
use anyhow::{anyhow, Error, Result};
use bevy_ecs::system::Resource;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Resource,
)]
pub struct Policy {
    pub auto_respawn: bool,
    pub max_alive_bots: usize,
    pub max_queued_bots: usize,
}

impl FromStr for Policy {
    type Err = Error;

    fn from_str(spec: &str) -> Result<Self> {
        let mut this = Self::default();

        for entry in spec::entries(spec) {
            let entry = entry?;

            match entry.key {
                "auto-respawn" => {
                    this.auto_respawn = entry.value()?;
                }
                "max-alive-bots" => {
                    this.max_alive_bots = entry.value()?;
                }
                "max-queued-bots" => {
                    this.max_queued_bots = entry.value()?;
                }
                key => {
                    return Err(anyhow!("unknown key: {key}"));
                }
            }
        }

        Ok(this)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let actual = Policy::from_str(
            "auto-respawn=true,max-alive-bots=100,max-queued-bots=200",
        )
        .unwrap();

        let expected = Policy {
            auto_respawn: true,
            max_alive_bots: 100,
            max_queued_bots: 200,
        };

        assert_eq!(expected, actual);
    }
}
