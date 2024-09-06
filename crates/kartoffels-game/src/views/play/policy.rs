#[derive(Debug)]
pub struct Policy {
    pub ui_enabled: bool,
    pub user_can_pause_world: bool,
    pub user_can_configure_world: bool,
    pub user_can_manage_bots: bool,
    pub pause_is_propagated: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            ui_enabled: true,
            user_can_pause_world: true,
            user_can_configure_world: false,
            user_can_manage_bots: false,
            pause_is_propagated: false,
        }
    }
}
