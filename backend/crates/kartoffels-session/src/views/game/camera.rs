use glam::{ivec2, IVec2, UVec2, Vec2};
use kartoffels_store::Store;
use ratatui::layout::Rect;

#[derive(Debug, Default)]
pub struct Camera {
    pos: Vec2,
    target: Vec2,
}

impl Camera {
    pub fn set_at(&mut self, target: IVec2) {
        let target = target.as_vec2();

        self.pos = target;
        self.target = target;
    }

    pub fn move_at(&mut self, target: IVec2) {
        self.target = target.as_vec2();
    }

    pub fn move_by(&mut self, delta: IVec2) {
        self.move_at((self.target + delta.as_vec2()).as_ivec2());
    }

    pub fn screen_to_world(&self, pos: UVec2, map_area: Rect) -> IVec2 {
        assert_eq!(map_area.x, 0);
        assert_eq!(map_area.y, 0);

        self.pos() + pos.as_ivec2()
            - ivec2(map_area.width as i32, map_area.height as i32) / 2
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
