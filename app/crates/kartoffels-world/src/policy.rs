use crate::spec;
use anyhow::{anyhow, Result};
use bevy_ecs::system::Resource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Resource)]
pub struct Policy {
    pub auto_respawn: bool,
    pub max_alive_bots: usize,
    pub max_queued_bots: usize,
}

impl Policy {
    pub fn create(spec: &str) -> Result<Self> {
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
