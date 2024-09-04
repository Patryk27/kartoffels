#[derive(Default)]
pub struct Policy {
    pub can_pause_world: bool,
    pub can_configure_world: bool,
    pub can_manage_bots: bool,
    pub pause_is_propagated: bool,
}
