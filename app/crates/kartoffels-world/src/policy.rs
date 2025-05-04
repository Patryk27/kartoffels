use crate::*;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    pub auto_respawn: bool,
    pub max_alive_bots: u8,
    pub max_queued_bots: u16,
}
