#[derive(Debug)]
pub struct Policy {
    pub can_pause_world: bool,
    pub can_configure_world: bool,
    pub can_manage_bots: bool,
    pub pause_is_propagated: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            can_pause_world: true,
            can_configure_world: false,
            can_manage_bots: false,
            pause_is_propagated: false,
        }
    }
}
