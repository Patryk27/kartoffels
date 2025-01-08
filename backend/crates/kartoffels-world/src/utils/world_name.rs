use bevy_ecs::system::Resource;

#[derive(Debug, PartialEq, Eq, Resource)]
pub struct WorldName(pub String);
