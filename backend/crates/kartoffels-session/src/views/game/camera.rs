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

    pub fn tick(&mut self, dt: f32, store: &Store) {
        if store.testing() {
            // Don't bother animating camera during tests, it makes them less
            // reproducible
            self.pos = self.target;
        } else {
            let vec = self.target - self.pos;
            let dir = vec.normalize();
            let len = vec.length();

            if len <= 0.5 {
                self.pos = self.target;
                return;
            }

            self.pos += dir * len.max(4.0) * (dt * 4.0).min(1.0);
        }
    }

    pub fn pos(&self) -> IVec2 {
        self.pos.as_ivec2()
    }
}
