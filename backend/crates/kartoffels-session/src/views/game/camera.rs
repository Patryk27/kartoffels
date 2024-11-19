use glam::{ivec2, IVec2, UVec2, Vec2};
use kartoffels_store::Store;
use ratatui::layout::Rect;

#[derive(Debug, Default)]
pub struct Camera {
    pos: Vec2,
    src: Vec2,
    dst: Vec2,
    t: f32,
}

impl Camera {
    pub fn set(&mut self, pos: IVec2) {
        let pos = pos.as_vec2();

        self.pos = pos;
        self.src = pos;
        self.dst = pos;
        self.t = 0.0;
    }

    pub fn look_at(&mut self, pos: IVec2) {
        if self.dst.distance(pos.as_vec2()) <= 3.0 {
            self.dst = pos.as_vec2();
        } else {
            self.src = self.pos().as_vec2();
            self.dst = pos.as_vec2();
            self.t = 0.0;
        }
    }

    pub fn nudge_by(&mut self, delta: IVec2) {
        self.look_at((self.dst + delta.as_vec2()).as_ivec2());
    }

    pub fn screen_to_world(&self, pos: UVec2, map_area: Rect) -> IVec2 {
        assert_eq!(map_area.x, 0);
        assert_eq!(map_area.y, 0);

        self.pos() + pos.as_ivec2()
            - ivec2(map_area.width as i32, map_area.height as i32) / 2
    }

    pub fn tick(&mut self, dt: f32, store: &Store) {
        if store.testing() {
            // Don't bother animating camera during tests
            self.pos = self.dst;
            return;
        }

        self.t = (self.t + dt * 2.2).min(1.0);
        self.pos = self.src.lerp(self.dst, ease(self.t));
    }

    pub fn pos(&self) -> IVec2 {
        self.pos.round().as_ivec2()
    }
}

fn ease(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(2)
}
