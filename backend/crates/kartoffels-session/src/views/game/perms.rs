#[derive(Debug)]
pub struct Perms {
    /// Whether the interface is enabled or not - when `false`, user can't do
    /// anything except using the `go back` button and interacting with dialogs.
    pub ui_enabled: bool,

    /// When active, user can upload at most one bot - useful for tutorial and
    /// challenges.
    pub hero_mode: bool,

    /// When active, pause will be synchronized between UI and the underlying
    /// world.
    ///
    /// We keep it disabled for online play, but enabled for worlds created by
    /// user themself.
    pub sync_pause: bool,

    /// Whether the user can delete/restart bots not uploaded by them.
    pub can_user_manage_bots: bool,

    pub can_user_pause: bool,
    pub can_user_set_speed: bool,
    pub can_user_spawn_prefabs: bool,
    pub can_user_upload_bots: bool,
}

impl Perms {
    pub const ONLINE: Self = Self {
        ui_enabled: true,
        hero_mode: false,
        sync_pause: false,
        can_user_manage_bots: false,
        can_user_pause: true,
        can_user_set_speed: false,
        can_user_spawn_prefabs: false,
        can_user_upload_bots: true,
    };

    pub const SANDBOX: Self = Self {
        ui_enabled: true,
        hero_mode: false,
        sync_pause: true,
        can_user_manage_bots: true,
        can_user_pause: true,
        can_user_set_speed: false,
        can_user_spawn_prefabs: true,
        can_user_upload_bots: true,
    };

    pub const TUTORIAL: Self = Self {
        ui_enabled: true,
        hero_mode: true,
        sync_pause: true,
        can_user_manage_bots: false,
        can_user_pause: false,
        can_user_set_speed: false,
        can_user_spawn_prefabs: false,
        can_user_upload_bots: true,
    };

    pub const CHALLENGE: Self = Self {
        ui_enabled: true,
        hero_mode: true,
        sync_pause: true,
        can_user_manage_bots: true,
        can_user_pause: true,
        can_user_set_speed: true,
        can_user_spawn_prefabs: false,
        can_user_upload_bots: true,
    };

    pub const DEBUG: Self = Self {
        ui_enabled: true,
        hero_mode: false,
        sync_pause: true,
        can_user_manage_bots: true,
        can_user_pause: true,
        can_user_set_speed: true,
        can_user_spawn_prefabs: true,
        can_user_upload_bots: true,
    };

    pub fn disabled(mut self) -> Self {
        self.ui_enabled = false;
        self
    }
}

impl Default for Perms {
    fn default() -> Self {
        Self::ONLINE
    }
}
