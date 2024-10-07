#[derive(Debug)]
pub struct Perms {
    /// When active, user can upload at most one bot; useful for tutorial and
    /// various challenges.
    pub single_bot_mode: bool,

    /// When active, pause will be synchronized between UI and the underlying
    /// world.
    ///
    /// We keep it disabled for online play, but enabled for worlds created by
    /// user themself.
    pub sync_pause: bool,

    pub user_can_alter_bots: bool,
    pub user_can_alter_speed: bool,
    pub user_can_pause_world: bool,
    pub user_can_spawn_prefabs: bool,
    pub user_can_upload_bots: bool,
}

impl Perms {
    pub const ONLINE: Self = Self {
        single_bot_mode: false,
        sync_pause: false,
        user_can_alter_bots: false,
        user_can_alter_speed: false,
        user_can_pause_world: true,
        user_can_spawn_prefabs: false,
        user_can_upload_bots: true,
    };

    pub const SANDBOX: Self = Self {
        single_bot_mode: false,
        sync_pause: true,
        user_can_alter_bots: true,
        user_can_alter_speed: true,
        user_can_pause_world: true,
        user_can_spawn_prefabs: true,
        user_can_upload_bots: true,
    };

    pub const TUTORIAL: Self = Self {
        single_bot_mode: true,
        sync_pause: true,
        user_can_alter_bots: false,
        user_can_alter_speed: false,
        user_can_pause_world: false,
        user_can_spawn_prefabs: false,
        user_can_upload_bots: true,
    };

    pub const CHALLENGE: Self = Self {
        single_bot_mode: false,
        sync_pause: true,
        user_can_alter_bots: true,
        user_can_alter_speed: true,
        user_can_pause_world: true,
        user_can_spawn_prefabs: false,
        user_can_upload_bots: true,
    };

    pub const PENDING: Self = Self {
        single_bot_mode: false,
        sync_pause: false,
        user_can_alter_bots: false,
        user_can_alter_speed: false,
        user_can_pause_world: false,
        user_can_spawn_prefabs: false,
        user_can_upload_bots: false,
    };
}

impl Default for Perms {
    fn default() -> Self {
        Self::ONLINE
    }
}
