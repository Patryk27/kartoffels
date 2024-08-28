use crate::{theme, BotIdExt};
use glam::{ivec2, IVec2};
use kartoffels_world::prelude::{BotUpdate, Dir, Map, TileBase};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug)]
pub struct MapCanvas {
    pub camera: IVec2,
    pub camera_offset: IVec2,
}

impl MapCanvas {
    pub fn handle(&mut self, event: InputEvent) -> MapCanvasOutcome {
        if let InputEvent::Key(event) = &event {
            match (event.key, event.modifiers) {
                (KeyCode::Char('w') | KeyCode::UpArrow, Modifiers::NONE) => {
                    self.camera.y -= self.camera_offset.y;

                    return MapCanvasOutcome::None;
                }

                (KeyCode::Char('a') | KeyCode::LeftArrow, Modifiers::NONE) => {
                    self.camera.x -= self.camera_offset.x;

                    return MapCanvasOutcome::None;
                }

                (KeyCode::Char('s') | KeyCode::DownArrow, Modifiers::NONE) => {
                    self.camera.y += self.camera_offset.y;

                    return MapCanvasOutcome::None;
                }

                (KeyCode::Char('d') | KeyCode::RightArrow, Modifiers::NONE) => {
                    self.camera.x += self.camera_offset.x;

                    return MapCanvasOutcome::None;
                }

                _ => (),
            }
        }

        MapCanvasOutcome::Forward(event)
    }

    pub fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        map: &Map,
        bots: &[BotUpdate],
        paused: bool,
        enabled: bool,
    ) {
        let offset =
            self.camera - ivec2(area.width as i32, area.height as i32) / 2;

        for dy in 0..area.height {
            for dx in 0..area.width {
                let tile = map.get(offset + ivec2(dx as i32, dy as i32));
                let ch;
                let mut fg;
                let mut bg;

                match tile.base {
                    TileBase::FLOOR => {
                        ch = ".";
                        fg = theme::GRAY;
                        bg = theme::BG;
                    }

                    TileBase::WALL_H => {
                        ch = "-";
                        fg = theme::GRAY;
                        bg = theme::BG;
                    }

                    TileBase::WALL_V => {
                        ch = "|";
                        fg = theme::GRAY;
                        bg = theme::BG;
                    }

                    TileBase::BOT => {
                        ch = "@";
                        fg = bots[tile.meta[0] as usize].id.color();
                        bg = theme::BG;
                    }

                    TileBase::BOT_CHEVRON => {
                        ch = match Dir::from(tile.meta[1]) {
                            Dir::Up => "⇡",
                            Dir::Right => "⇢",
                            Dir::Down => "⇣",
                            Dir::Left => "⇠",
                        };

                        fg = bots[tile.meta[0] as usize].id.color();
                        bg = theme::BG;
                    }

                    _ => {
                        ch = " ";
                        fg = theme::FG;
                        bg = theme::BG;
                    }
                };

                if enabled {
                    if paused {
                        if tile.base == TileBase::BOT {
                            fg = theme::BG;
                            bg = theme::DARK_GRAY;
                        } else {
                            fg = theme::DARK_GRAY;
                            bg = theme::BG;
                        }
                    }
                } else {
                    fg = theme::DARK_GRAY;
                    bg = theme::BG;
                }

                buf[(area.left() + dx, area.top() + dy)]
                    .set_symbol(ch)
                    .set_fg(fg)
                    .set_bg(bg);
            }
        }
    }
}

#[derive(Debug)]
pub enum MapCanvasOutcome {
    None,
    Forward(InputEvent),
}
