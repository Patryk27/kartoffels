use glam::{IVec2, UVec2, ivec2};
use ratatui::layout::Rect;

#[derive(Debug, Default)]
pub struct Camera {
    pos: IVec2,
}

impl Camera {
    pub fn look_at(&mut self, pos: IVec2) {
        self.pos = pos;
    }

    pub fn screen_to_world(&self, pos: UVec2, map_area: Rect) -> IVec2 {
        assert_eq!(map_area.x, 0);
        assert_eq!(map_area.y, 0);

        self.pos() + pos.as_ivec2()
            - ivec2(map_area.width as i32, map_area.height as i32) / 2
    }

    pub fn pos(&self) -> IVec2 {
        self.pos
    }
}
