#[derive(Debug)]
pub struct Permissions {
    pub single_bot_mode: bool,
    pub sync_pause: bool,
    pub user_can_manage_bots: bool,
    pub user_can_pause_world: bool,
    pub user_can_spawn_prefabs: bool,
}

impl Permissions {
    pub const ONLINE: Self = Self {
        single_bot_mode: false,
        sync_pause: false,
        user_can_manage_bots: false,
        user_can_pause_world: true,
        user_can_spawn_prefabs: false,
    };

    pub const SANDBOX: Self = Self {
        single_bot_mode: false,
        sync_pause: true,
        user_can_manage_bots: true,
        user_can_pause_world: true,
        user_can_spawn_prefabs: true,
    };

    pub const TUTORIAL: Self = Self {
        single_bot_mode: true,
        sync_pause: true,
        user_can_manage_bots: false,
        user_can_pause_world: false,
        user_can_spawn_prefabs: false,
    };

    pub const CHALLENGE: Self = Self {
        single_bot_mode: false,
        sync_pause: true,
        user_can_manage_bots: true,
        user_can_pause_world: true,
        user_can_spawn_prefabs: false,
    };

    pub const PENDING: Self = Self {
        single_bot_mode: false,
        sync_pause: false,
        user_can_manage_bots: false,
        user_can_pause_world: false,
        user_can_spawn_prefabs: false,
    };
}

impl Default for Permissions {
    fn default() -> Self {
        Self::ONLINE
    }
}
