use bevy_ecs::system::Resource;
use kartoffels_utils::Id;

#[derive(Debug, PartialEq, Eq, Resource)]
pub struct WorldId(pub Id);
