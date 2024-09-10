use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policy {
    pub auto_respawn: bool,
    pub max_alive_bots: usize,
    pub max_queued_bots: usize,
}
