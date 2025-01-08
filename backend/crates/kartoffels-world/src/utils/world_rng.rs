use bevy_ecs::system::Resource;
use rand::rngs::SmallRng;

#[derive(Debug, Resource)]
pub struct WorldRng(pub SmallRng);
