#[derive(Debug)]
pub struct Perms {
    /// Whether the interface is enabled or not - when `false`, user can't do
    /// anything except using the `go back` button.
    pub enabled: bool,

    /// When active, user can upload at most one bot; useful for tutorial and
    /// various challenges.
    pub single_bot_mode: bool,

    /// When active, pause will be synchronized between UI and the underlying
    /// world.
    ///
    /// We keep it disabled for online play, but enabled for worlds created by
    /// user themself.
    pub sync_pause: bool,

    /// Whether the user can restart/destroy bots not uploaded by them.
    pub user_can_manage_bots: bool,

    pub user_can_pause: bool,
    pub user_can_set_speed: bool,
    pub user_can_spawn_prefabs: bool,
    pub user_can_upload_bots: bool,
}

impl Perms {
    pub const ONLINE: Self = Self {
        enabled: true,
        single_bot_mode: false,
        sync_pause: false,
        user_can_manage_bots: false,
        user_can_pause: true,
        user_can_set_speed: false,
        user_can_spawn_prefabs: false,
        user_can_upload_bots: true,
    };

    pub const SANDBOX: Self = Self {
        enabled: true,
        single_bot_mode: false,
        sync_pause: true,
        user_can_manage_bots: true,
        user_can_pause: true,
        user_can_set_speed: false,
        user_can_spawn_prefabs: true,
        user_can_upload_bots: true,
    };

    pub const TUTORIAL: Self = Self {
        enabled: true,
        single_bot_mode: true,
        sync_pause: true,
        user_can_manage_bots: false,
        user_can_pause: false,
        user_can_set_speed: false,
        user_can_spawn_prefabs: false,
        user_can_upload_bots: true,
    };

    pub const CHALLENGE: Self = Self {
        enabled: true,
        single_bot_mode: true,
        sync_pause: true,
        user_can_manage_bots: true,
        user_can_pause: true,
        user_can_set_speed: true,
        user_can_spawn_prefabs: false,
        user_can_upload_bots: true,
    };

    pub const PENDING: Self = Self {
        enabled: false,
        single_bot_mode: false,
        sync_pause: false,
        user_can_manage_bots: false,
        user_can_pause: false,
        user_can_set_speed: false,
        user_can_spawn_prefabs: false,
        user_can_upload_bots: false,
    };
}

impl Default for Perms {
    fn default() -> Self {
        Self::ONLINE
    }
}
