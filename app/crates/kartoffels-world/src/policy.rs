use crate::spec;
use anyhow::{anyhow, Error, Result};
use bevy_ecs::system::Resource;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Resource)]
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
                "auto_respawn" => {
                    this.auto_respawn = entry.value()?;
                }

                "max_alive_bots" => {
                    this.max_alive_bots = entry.value()?;
                }

                "max_queued_bots" => {
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
