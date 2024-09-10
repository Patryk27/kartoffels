#[derive(Debug)]
pub struct Permissions {
    pub user_can_pause_world: bool,
    pub user_can_configure_world: bool,
    pub user_can_manage_bots: bool,
    pub sync_pause_mode: bool,
    pub single_bot_mode: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            user_can_pause_world: true,
            user_can_configure_world: false,
            user_can_manage_bots: false,
            sync_pause_mode: false,
            single_bot_mode: false,
        }
    }
}
