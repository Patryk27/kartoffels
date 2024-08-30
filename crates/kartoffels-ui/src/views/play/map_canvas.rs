use crate::{theme, BotIdExt, Ui};
use glam::{ivec2, IVec2};
use kartoffels_world::prelude::{Dir, Snapshot, TileBase};
use termwiz::input::{KeyCode, Modifiers};

#[derive(Debug)]
pub struct MapCanvas;

impl MapCanvas {
    pub fn render(
        ui: &mut Ui,
        snapshot: &Snapshot,
        camera: IVec2,
        paused: bool,
        enabled: bool,
    ) -> Option<MapCanvasEvent> {
        let area = ui.area();
        let offset = camera - ivec2(area.width as i32, area.height as i32) / 2;

        for dy in 0..area.height {
            for dx in 0..area.width {
                let tile =
                    snapshot.map.get(offset + ivec2(dx as i32, dy as i32));

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

                        fg = snapshot
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

                        fg = snapshot
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

                if enabled {
                    if paused && tile.base != TileBase::BOT {
                        fg = theme::DARK_GRAY;
                        bg = theme::BG;
                    }
                } else {
                    fg = theme::DARK_GRAY;
                    bg = theme::BG;
                }

                ui.buf()[(area.left() + dx, area.top() + dy)]
                    .set_symbol(ch)
                    .set_fg(fg)
                    .set_bg(bg);
            }
        }

        if ui.key(KeyCode::Char('w'), Modifiers::NONE)
            || ui.key(KeyCode::UpArrow, Modifiers::NONE)
        {
            return Some(MapCanvasEvent::MoveCamera(ivec2(0, -offset.y)));
        }

        if ui.key(KeyCode::Char('a'), Modifiers::NONE)
            || ui.key(KeyCode::LeftArrow, Modifiers::NONE)
        {
            return Some(MapCanvasEvent::MoveCamera(ivec2(-offset.x, 0)));
        }

        if ui.key(KeyCode::Char('s'), Modifiers::NONE)
            || ui.key(KeyCode::DownArrow, Modifiers::NONE)
        {
            return Some(MapCanvasEvent::MoveCamera(ivec2(0, offset.y)));
        }

        if ui.key(KeyCode::Char('d'), Modifiers::NONE)
            || ui.key(KeyCode::RightArrow, Modifiers::NONE)
        {
            return Some(MapCanvasEvent::MoveCamera(ivec2(offset.x, 0)));
        }

        None
    }
}

#[derive(Debug)]
pub enum MapCanvasEvent {
    MoveCamera(IVec2),
}
