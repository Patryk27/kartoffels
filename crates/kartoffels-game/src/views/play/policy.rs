#[derive(Default)]
pub struct Policy {
    pub can_configure_world: bool,
    pub can_manage_bots: bool,
    pub propagate_pause: bool,
}
