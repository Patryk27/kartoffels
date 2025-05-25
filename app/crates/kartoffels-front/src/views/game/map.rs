use super::{Event, Mode, View};
use crate::{BotIdExt, Ui, theme};
use glam::{IVec2, Vec2, ivec2, vec2};
use kartoffels_world::prelude as w;
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::time::{Instant, SystemTime};
use termwiz::input::{KeyCode, Modifiers};

#[derive(Debug)]
pub struct Map {
    pub blink: Instant,
}

impl Map {
    pub fn render(&self, ui: &mut Ui<Event>, view: &View) {
        self.render_tiles(ui, view);
        self.render_cursor(ui, view);
        self.process_keys(ui);
    }

    fn render_tiles(&self, ui: &mut Ui<Event>, view: &View) {
        let offset = view.camera.pos()
            - ivec2(ui.area.width as i32, ui.area.height as i32) / 2;

        let time = {
            let time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();

            (time % 100_000_000) as f32 / 3500.0
        };

        for dy in 0..ui.area.height {
            for dx in 0..ui.area.width {
                let area = Rect {
                    x: ui.area.x + dx,
                    y: ui.area.y + dy,
                    width: 1,
                    height: 1,
                };

                ui.at(area, |ui| {
                    self.render_tile(
                        ui,
                        view,
                        time,
                        offset + ivec2(dx as i32, dy as i32),
                    );
                });
            }
        }
    }

    fn render_cursor(&self, ui: &mut Ui<Event>, view: &View) {
        if let Mode::SpawningBot {
            cursor_screen: Some(cursor_screen),
            cursor_valid,
            ..
        } = &view.mode
        {
            let cursor_screen = cursor_screen.as_ivec2()
                - ivec2(ui.area.x as i32, ui.area.y as i32);

            if cursor_screen.x >= 0
                && cursor_screen.y >= 0
                && cursor_screen.x < ui.area.width as i32
                && cursor_screen.y < ui.area.height as i32
            {
                let cursor_screen =
                    (cursor_screen.x as u16, cursor_screen.y as u16);

                let cursor_bg = if *cursor_valid {
                    theme::GREEN
                } else {
                    theme::RED
                };

                ui.buf[cursor_screen]
                    .set_char('@')
                    .set_fg(theme::BG)
                    .set_bg(cursor_bg);
            }
        }
    }

    fn render_tile(
        &self,
        ui: &mut Ui<Event>,
        view: &View,
        time: f32,
        pos: IVec2,
    ) {
        let tile = view.snapshot.map.get(pos);

        let ch;
        let mut fg;
        let mut bg;

        match tile.kind {
            w::TileKind::BOT => {
                ch = '@';

                fg = view
                    .snapshot
                    .bots
                    .alive
                    .get_by_idx(tile.meta[0])
                    .map(|bot| bot.id.color())
                    .unwrap();

                bg = theme::BG;
            }

            w::TileKind::BOT_CHEVRON => {
                ch = match w::AbsDir::from(tile.meta[1]) {
                    w::AbsDir::N => '↑',
                    w::AbsDir::E => '→',
                    w::AbsDir::S => '↓',
                    w::AbsDir::W => '←',
                };

                fg = view
                    .snapshot
                    .bots
                    .alive
                    .get_by_idx(tile.meta[0])
                    .map(|bot| bot.id.color())
                    .unwrap();

                bg = theme::BG;
            }

            w::TileKind::DOOR => {
                ch = '+';
                fg = theme::GRAY;
                bg = theme::BG;
            }

            w::TileKind::FLOOR => {
                ch = '.';
                fg = theme::DARK_GRAY;
                bg = theme::BG;
            }

            w::TileKind::WALL => {
                ch = '#';
                fg = theme::GRAY;
                bg = theme::BG;
            }

            w::TileKind::WALL_H => {
                ch = '-';
                fg = theme::GRAY;
                bg = theme::BG;
            }

            w::TileKind::WALL_V => {
                ch = '|';
                fg = theme::GRAY;
                bg = theme::BG;
            }

            w::TileKind::WATER => {
                let height = ocean(time, pos.as_vec2() / vec2(8.0, 4.0));
                let color = 0.4 + (height * 8.0) as u8 as f32 / 16.0;

                ch = '~';

                fg = Color::Rgb(
                    0,
                    64 + (32.0 * color) as u8,
                    128 + (128.0 * color) as u8,
                );

                bg = Color::Rgb(0, (64.0 * color) as u8, (255.0 * color) as u8);
            }

            w::ObjectKind::FLAG => {
                ch = '=';
                fg = theme::YELLOW;
                bg = theme::BG;
            }

            w::ObjectKind::GEM => {
                ch = '*';
                fg = theme::BLUE;
                bg = theme::BG;
            }

            _ => {
                ch = ' ';
                fg = theme::FG;
                bg = theme::BG;
            }
        };

        if ui.enabled {
            if view.status.is_paused() && tile.kind != w::TileKind::BOT {
                fg = theme::DARK_GRAY;
                bg = theme::BG;
            }

            if tile.kind == w::TileKind::BOT {
                let id = view
                    .snapshot
                    .bots
                    .alive
                    .get_by_idx(tile.meta[0])
                    .map(|bot| bot.id)
                    .unwrap();

                if ui.mouse_over(ui.area) && view.config.can_join_bots {
                    fg = theme::BG;
                    bg = theme::GREEN;

                    if ui.mouse_pressed() {
                        ui.throw(Event::JoinBot { id });
                    }
                } else {
                    #[allow(clippy::collapsible_else_if)]
                    if let Some(bot) = &view.bot
                        && bot.id == id
                        && self.blink.elapsed().as_millis() % 1000 <= 500
                    {
                        fg = theme::BG;
                        bg = theme::GREEN;
                    }
                }
            }
        }

        let pos = ui.area.as_position();

        ui.buf[pos].set_char(ch).set_fg(fg).set_bg(bg);
    }

    fn process_keys(&self, ui: &mut Ui<Event>) {
        if !ui.enabled {
            return;
        }

        let offset = ivec2(ui.area.width as i32, ui.area.height as i32) / 3;

        if ui.key(KeyCode::Char('w'), Modifiers::NONE)
            || ui.key(KeyCode::UpArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera {
                delta: ivec2(0, -offset.y),
            });
        }

        if ui.key(KeyCode::Char('a'), Modifiers::NONE)
            || ui.key(KeyCode::LeftArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera {
                delta: ivec2(-offset.x, 0),
            });
        }

        if ui.key(KeyCode::Char('s'), Modifiers::NONE)
            || ui.key(KeyCode::DownArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera {
                delta: ivec2(0, offset.y),
            });
        }

        if ui.key(KeyCode::Char('d'), Modifiers::NONE)
            || ui.key(KeyCode::RightArrow, Modifiers::NONE)
        {
            ui.throw(Event::MoveCamera {
                delta: ivec2(offset.x, 0),
            });
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

/// Inspired by https://www.shadertoy.com/view/MdXyzX.
fn ocean(time: f32, pos: Vec2) -> f32 {
    let pos = pos + vec2(128.0, 128.0);

    let mut h_sum = 0.0;
    let mut h_weight = 0.0;

    let mut wave_pos = pos;
    let mut wave_freq = 1.0;
    let mut wave_weight = 1.0;

    let mut noise = 0.0f32;

    for _ in 0..12 {
        let wave_dir = vec2(noise.cos(), noise.sin());

        let wave = wave_dir.dot(wave_pos) * wave_freq + time;
        let wave_h = (wave.sin() - 1.0).exp();
        let wave_dh = wave_h * wave.cos();

        h_sum += wave_h * wave_weight;
        h_weight += wave_weight;

        wave_pos -= 0.25 * wave_dh * wave_dir * wave_weight;
        wave_freq *= 1.18;
        wave_weight *= 0.82;

        noise += 1234.4321;
    }

    h_sum / h_weight
}
