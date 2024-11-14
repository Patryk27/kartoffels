#[derive(Debug)]
pub struct Config {
    /// Whether the interface is enabled or not - when `false`, user can't do
    /// anything except using the `go back` button and interacting with modals.
    pub enabled: bool,

    /// When active, user can upload at most one bot - useful for tutorial and
    /// challenges.
    pub hero_mode: bool,

    /// When active, pause will be synchronized between UI and the underlying
    /// world.
    ///
    /// We keep it disabled for online play, but enabled for worlds created by
    /// user themself.
    pub sync_pause: bool,

    pub can_delete_bots: bool,
    pub can_join_bots: bool,
    pub can_overclock: bool,
    pub can_pause: bool,
    pub can_restart_bots: bool,
    pub can_spawn_bots: bool,
    pub can_upload_bots: bool,
}

impl Config {
    pub(super) const DEBUG: Self = Self {
        enabled: true,
        hero_mode: false,
        sync_pause: true,

        can_delete_bots: true,
        can_join_bots: true,
        can_overclock: true,
        can_pause: true,
        can_restart_bots: true,
        can_spawn_bots: true,
        can_upload_bots: true,
    };

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            hero_mode: false,
            sync_pause: false,

            can_delete_bots: false,
            can_join_bots: true,
            can_overclock: false,
            can_pause: true,
            can_restart_bots: false,
            can_spawn_bots: false,
            can_upload_bots: true,
        }
    }
}
