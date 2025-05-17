use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    #[serde(default = "default_auto_respawn")]
    pub auto_respawn: bool,
    #[serde(default = "default_max_alive_bots")]
    pub max_alive_bots: u8,
    #[serde(default = "default_max_queued_bots")]
    pub max_queued_bots: u16,
}

fn default_auto_respawn() -> bool {
    true
}

fn default_max_alive_bots() -> u8 {
    64
}

fn default_max_queued_bots() -> u16 {
    64
}
