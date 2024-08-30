use crate::{theme, BotIdExt, Term};
use glam::{ivec2, IVec2};
use kartoffels_world::prelude::{Dir, Snapshot, TileBase};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use termwiz::input::{InputEvent, KeyCode, Modifiers};

#[derive(Debug)]
pub struct MapCanvas<'a> {
    pub snapshot: &'a Snapshot,
    pub camera: IVec2,
    pub paused: bool,
    pub enabled: bool,
}

impl<'a> MapCanvas<'a> {
    pub fn render(self, area: Rect, buf: &mut Buffer) {
        let offset =
            self.camera - ivec2(area.width as i32, area.height as i32) / 2;

        for dy in 0..area.height {
            for dx in 0..area.width {
                let tile =
                    self.snapshot.map.get(offset + ivec2(dx as i32, dy as i32));

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

                        fg = self
                            .snapshot
                            .bots
                            .alive
                            .by_idx(tile.meta[0])
                            .map(|bot| bot.id.color())
                            .unwrap_or(theme::RED);

                        bg = theme::BG;
                    }

                    TileBase::BOT_CHEVRON => {
                        ch = match Dir::from(tile.meta[1]) {
                            Dir::Up => "⇡",
                            Dir::Right => "⇢",
                            Dir::Down => "⇣",
                            Dir::Left => "⇠",
                        };

                        fg = self
                            .snapshot
                            .bots
                            .alive
                            .by_idx(tile.meta[0])
                            .map(|bot| bot.id.color())
                            .unwrap_or(theme::RED);

                        bg = theme::BG;
                    }

                    _ => {
                        ch = " ";
                        fg = theme::FG;
                        bg = theme::BG;
                    }
                };

                if self.enabled {
                    if self.paused && tile.base != TileBase::BOT {
                        fg = theme::DARK_GRAY;
                        bg = theme::BG;
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

    pub fn handle(event: InputEvent, term: &Term) -> MapCanvasEvent {
        if let InputEvent::Key(event) = &event {
            let offset = term.size().as_ivec2() / 8;

            match (event.key, event.modifiers) {
                (KeyCode::Char('w') | KeyCode::UpArrow, Modifiers::NONE) => {
                    return MapCanvasEvent::MoveCamera(ivec2(0, -offset.y));
                }

                (KeyCode::Char('a') | KeyCode::LeftArrow, Modifiers::NONE) => {
                    return MapCanvasEvent::MoveCamera(ivec2(-offset.x, 0));
                }

                (KeyCode::Char('s') | KeyCode::DownArrow, Modifiers::NONE) => {
                    return MapCanvasEvent::MoveCamera(ivec2(0, offset.y));
                }

                (KeyCode::Char('d') | KeyCode::RightArrow, Modifiers::NONE) => {
                    return MapCanvasEvent::MoveCamera(ivec2(offset.x, 0));
                }

                _ => (),
            }
        }

        MapCanvasEvent::Forward(event)
    }
}

#[derive(Debug)]
pub enum MapCanvasEvent {
    MoveCamera(IVec2),
    Forward(InputEvent),
}
