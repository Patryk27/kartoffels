use super::State;
use crate::BotIdExt;
use anyhow::Result;
use glam::{ivec2, IVec2};
use kartoffels_ui::{theme, Ui};
use kartoffels_world::prelude::{BotId, Dir, Tile, TileBase};
use ratatui::layout::Rect;
use std::ops::ControlFlow;
use std::time::{SystemTime, UNIX_EPOCH};
use termwiz::input::{KeyCode, Modifiers};

#[derive(Debug)]
pub struct Map;

impl Map {
    pub fn render(ui: &mut Ui, state: &State) -> Option<MapResponse> {
        let mut resp = None;

        Self::render_tiles(ui, state, &mut resp);
        Self::process_keys(ui, &mut resp);

        resp
    }

    fn render_tiles(
        ui: &mut Ui,
        state: &State,
        resp: &mut Option<MapResponse>,
    ) {
        let area = ui.area();

        let offset =
            state.camera - ivec2(area.width as i32, area.height as i32) / 2;

        for dy in 0..area.height {
            for dx in 0..area.width {
                let area = Rect {
                    x: area.x + dx,
                    y: area.y + dy,
                    width: 1,
                    height: 1,
                };

                ui.clamp(area, |ui| {
                    let tile = state
                        .snapshot
                        .map()
                        .get(offset + ivec2(dx as i32, dy as i32));

                    Self::render_tile(ui, state, tile, resp);
                });
            }
        }
    }

    fn render_tile(
        ui: &mut Ui,
        state: &State,
        tile: Tile,
        resp: &mut Option<MapResponse>,
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

                fg = state
                    .snapshot
                    .bots()
                    .alive()
                    .by_idx(tile.meta[0])
                    .map(|bot| bot.id.color())
                    .unwrap();

                bg = theme::BG;
            }

            TileBase::BOT_CHEVRON => {
                ch = match Dir::from(tile.meta[1]) {
                    Dir::Up => "↑",
                    Dir::Right => "→",
                    Dir::Down => "↓",
                    Dir::Left => "←",
                };

                fg = state
                    .snapshot
                    .bots()
                    .alive()
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

        if ui.enabled() {
            if state.paused && tile.base != TileBase::BOT {
                fg = theme::DARK_GRAY;
                bg = theme::BG;
            }

            if tile.base == TileBase::BOT {
                let id = state
                    .snapshot
                    .bots()
                    .alive()
                    .by_idx(tile.meta[0])
                    .map(|bot| bot.id)
                    .unwrap();

                if ui.mouse_over(ui.area()) && !state.perms.single_bot_mode {
                    fg = theme::BG;
                    bg = theme::GREEN;

                    if ui.mouse_pressed() {
                        *resp = Some(MapResponse::JoinBot(id));
                    }
                } else if let Some(bot) = &state.bot {
                    if bot.id == id {
                        let blink = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                            % 1000
                            >= 500;

                        if blink {
                            fg = theme::BG;
                            bg = theme::GREEN;
                        }
                    }
                }
            }
        }

        let pos = ui.area().as_position();

        ui.buf()[pos].set_symbol(ch).set_fg(fg).set_bg(bg);
    }

    fn process_keys(ui: &Ui, resp: &mut Option<MapResponse>) {
        if !ui.enabled() {
            return;
        }

        let offset = ivec2(ui.area().width as i32, ui.area().height as i32) / 5;

        if ui.key(KeyCode::Char('w'), Modifiers::NONE)
            || ui.key(KeyCode::UpArrow, Modifiers::NONE)
        {
            *resp = Some(MapResponse::MoveCamera(ivec2(0, -offset.y)));
        }

        if ui.key(KeyCode::Char('a'), Modifiers::NONE)
            || ui.key(KeyCode::LeftArrow, Modifiers::NONE)
        {
            *resp = Some(MapResponse::MoveCamera(ivec2(-offset.x, 0)));
        }

        if ui.key(KeyCode::Char('s'), Modifiers::NONE)
            || ui.key(KeyCode::DownArrow, Modifiers::NONE)
        {
            *resp = Some(MapResponse::MoveCamera(ivec2(0, offset.y)));
        }

        if ui.key(KeyCode::Char('d'), Modifiers::NONE)
            || ui.key(KeyCode::RightArrow, Modifiers::NONE)
        {
            *resp = Some(MapResponse::MoveCamera(ivec2(offset.x, 0)));
        }
    }
}

#[derive(Debug)]
pub enum MapResponse {
    MoveCamera(IVec2),
    JoinBot(BotId),
}

impl MapResponse {
    pub fn handle(self, state: &mut State) -> Result<ControlFlow<(), ()>> {
        match self {
            MapResponse::MoveCamera(delta) => {
                state.camera += delta;

                if let Some(bot) = &mut state.bot {
                    bot.is_followed = false;
                }
            }

            MapResponse::JoinBot(id) => {
                state.join_bot(id);
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}
