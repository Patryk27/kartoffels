#[derive(Debug)]
pub struct Permissions {
    pub single_bot_mode: bool,
    pub sync_pause: bool,
    pub user_can_manage_bots: bool,
    pub user_can_pause_world: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            single_bot_mode: false,
            sync_pause: false,
            user_can_manage_bots: false,
            user_can_pause_world: true,
        }
    }
}
