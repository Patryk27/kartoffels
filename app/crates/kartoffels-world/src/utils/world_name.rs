use arc_swap::ArcSwap;
use bevy_ecs::system::Resource;
use std::sync::Arc;

#[derive(Debug, Resource)]
pub struct WorldName(pub Arc<ArcSwap<String>>);
