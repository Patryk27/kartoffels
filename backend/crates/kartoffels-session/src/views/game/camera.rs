use glam::{IVec2, Vec2};
use kartoffels_store::Store;

#[derive(Debug, Default)]
pub struct Camera {
    pos: Vec2,
    target: Vec2,
}

impl Camera {
    pub fn move_to(&mut self, target: IVec2) {
        let target = target.as_vec2();

        self.pos = target;
        self.target = target;
    }

    pub fn animate_to(&mut self, target: IVec2) {
        self.target = target.as_vec2();
    }

    pub fn animate_by(&mut self, delta: IVec2) {
        self.animate_to((self.target + delta.as_vec2()).as_ivec2());
    }

    pub fn tick(&mut self, store: &Store) {
        if store.is_testing() {
            // Don't bother animating camera during tests, it makes them less
            // reproducible

            self.pos = self.target;
        } else {
            self.pos = (self.pos * 4.0 + self.target) / 5.0;
        }
    }

    pub fn pos(&self) -> IVec2 {
        self.pos.as_ivec2()
    }
}
