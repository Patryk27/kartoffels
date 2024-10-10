use super::{Event, State};
use crate::BotIdExt;
use glam::ivec2;
use kartoffels_ui::{theme, Ui};
use kartoffels_world::prelude::{Dir, Tile, TileBase};
use ratatui::layout::Rect;
use std::time::Instant;
use termwiz::input::{KeyCode, Modifiers};

#[derive(Debug)]
pub struct Map {
    pub blink: Instant,
}

impl Map {
    pub fn render(ui: &mut Ui<Event>, state: &mut State) {
        Self::render_tiles(ui, state);
        Self::process_keys(ui);
    }

    fn render_tiles(ui: &mut Ui<Event>, state: &mut State) {
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

                    Self::render_tile(ui, state, tile);
                });
            }
        }
    }

    fn render_tile(ui: &mut Ui<Event>, state: &mut State, tile: Tile) {
        let ch;
        let mut fg;
        let mut bg;

        match tile.base {
            TileBase::BOT => {
                ch = '@';

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
                    Dir::N => '↑',
                    Dir::E => '→',
                    Dir::S => '↓',
                    Dir::W => '←',
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

            TileBase::FLOOR => {
                ch = '.';
                fg = theme::DARK_GRAY;
                bg = theme::BG;
            }

            TileBase::WALL_H => {
                ch = '-';
                fg = theme::GRAY;
                bg = theme::BG;
            }

            TileBase::WALL_V => {
                ch = '|';
                fg = theme::GRAY;
                bg = theme::BG;
            }

            _ => {
                ch = ' ';
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
                        ui.throw(Event::JoinBot(id));
                    }
                } else {
                    #[allow(clippy::collapsible_else_if)]
                    if let Some(bot) = &state.bot
                        && bot.id == id
                        && state.map.blink.elapsed().as_millis() % 1000 <= 500
                    {
                        fg = theme::BG;
                        bg = theme::GREEN;
                    }
                }
            }
        }

        let pos = ui.area().as_position();

        ui.buf()[pos].set_char(ch).set_fg(fg).set_bg(bg);
    }

    fn process_keys(ui: &mut Ui<Event>) {
        if !ui.enabled() {
            return;
        }

        let offset = ivec2(ui.area().width as i32, ui.area().height as i32) / 5;

        if ui.key(KeyCode::Char('w'), Modifiers::NONE)
            || ui.key(KeyCode::UpArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera(ivec2(0, -offset.y)));
        }

        if ui.key(KeyCode::Char('a'), Modifiers::NONE)
            || ui.key(KeyCode::LeftArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera(ivec2(-offset.x, 0)));
        }

        if ui.key(KeyCode::Char('s'), Modifiers::NONE)
            || ui.key(KeyCode::DownArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera(ivec2(0, offset.y)));
        }

        if ui.key(KeyCode::Char('d'), Modifiers::NONE)
            || ui.key(KeyCode::RightArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera(ivec2(offset.x, 0)));
        }
    }
}

impl Default for Map {
    fn default() -> Self {
        Self {
            blink: Instant::now(),
        }
    }
}
