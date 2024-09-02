use crate::{theme, BotIdExt, Ui};
use glam::{ivec2, IVec2};
use kartoffels_world::prelude::{BotId, Dir, Snapshot, Tile, TileBase};
use ratatui::layout::Rect;
use termwiz::input::{KeyCode, Modifiers};

#[derive(Debug)]
pub struct MapCanvas;

impl MapCanvas {
    pub fn render(
        ui: &mut Ui,
        world: &Snapshot,
        camera: IVec2,
        paused: bool,
        enabled: bool,
    ) -> Option<MapCanvasResponse> {
        let mut response = None;

        let area = ui.area();
        let offset = camera - ivec2(area.width as i32, area.height as i32) / 2;

        for dy in 0..area.height {
            for dx in 0..area.width {
                let area = Rect {
                    x: area.x + dx,
                    y: area.y + dy,
                    width: 1,
                    height: 1,
                };

                ui.clamp(area, |ui| {
                    let tile =
                        world.map.get(offset + ivec2(dx as i32, dy as i32));

                    Self::render_tile(
                        ui,
                        world,
                        tile,
                        paused,
                        enabled,
                        &mut response,
                    );
                });
            }
        }

        if enabled {
            let offset =
                ivec2(ui.area().width as i32, ui.area().height as i32) / 5;

            if ui.key(KeyCode::Char('w'), Modifiers::NONE)
                || ui.key(KeyCode::UpArrow, Modifiers::NONE)
            {
                response =
                    Some(MapCanvasResponse::MoveCamera(ivec2(0, -offset.y)));
            }

            if ui.key(KeyCode::Char('a'), Modifiers::NONE)
                || ui.key(KeyCode::LeftArrow, Modifiers::NONE)
            {
                response =
                    Some(MapCanvasResponse::MoveCamera(ivec2(-offset.x, 0)));
            }

            if ui.key(KeyCode::Char('s'), Modifiers::NONE)
                || ui.key(KeyCode::DownArrow, Modifiers::NONE)
            {
                response =
                    Some(MapCanvasResponse::MoveCamera(ivec2(0, offset.y)));
            }

            if ui.key(KeyCode::Char('d'), Modifiers::NONE)
                || ui.key(KeyCode::RightArrow, Modifiers::NONE)
            {
                response =
                    Some(MapCanvasResponse::MoveCamera(ivec2(offset.x, 0)));
            }
        }

        response
    }

    fn render_tile(
        ui: &mut Ui,
        world: &Snapshot,
        tile: Tile,
        paused: bool,
        enabled: bool,
        response: &mut Option<MapCanvasResponse>,
    ) {
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

                fg = world
                    .bots
                    .alive
                    .by_idx(tile.meta[0])
                    .map(|bot| bot.id.color())
                    .unwrap();

                bg = theme::BG;
            }

            TileBase::BOT_CHEVRON => {
                ch = match Dir::from(tile.meta[1]) {
                    Dir::Up => "⇡",
                    Dir::Right => "⇢",
                    Dir::Down => "⇣",
                    Dir::Left => "⇠",
                };

                fg = world
                    .bots
                    .alive
                    .by_idx(tile.meta[0])
                    .map(|bot| bot.id.color())
                    .unwrap();

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

            if tile.base == TileBase::BOT && ui.mouse_over(ui.area()) {
                fg = theme::BG;
                bg = theme::GREEN;

                if ui.mouse_pressed() {
                    let id = world
                        .bots
                        .alive
                        .by_idx(tile.meta[0])
                        .map(|bot| bot.id)
                        .unwrap();

                    *response = Some(MapCanvasResponse::JoinBot(id));
                }
            }
        }

        let pos = ui.area().as_position();

        ui.buf()[pos].set_symbol(ch).set_fg(fg).set_bg(bg);
    }
}

#[derive(Debug)]
pub enum MapCanvasResponse {
    MoveCamera(IVec2),
    JoinBot(BotId),
}