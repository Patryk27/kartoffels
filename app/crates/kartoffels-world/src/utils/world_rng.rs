use bevy_ecs::system::Resource;
use rand_chacha::ChaCha8Rng;

#[derive(Debug, Resource)]
pub struct WorldRng(pub ChaCha8Rng);
