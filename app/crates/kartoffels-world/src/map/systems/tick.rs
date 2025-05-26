use crate::World;
use glam::{Vec2, ivec2, vec2};

const UPDATE_EVERY_TICKS: u64 = 8_000;

struct State {
    next_run_at: u64,
}

impl Default for State {
    fn default() -> Self {
        Self { next_run_at: 0 }
    }
}

pub fn tick(world: &mut World) {
    let state = world.states.get_mut::<State>();

    if world.ticks < state.next_run_at {
        return;
    }

    let time = world.ticks as f32 / (UPDATE_EVERY_TICKS as f32) / 25.0;
    let map = &mut world.map;

    for y in 0..map.size().y {
        for x in 0..map.size().x {
            let pos = ivec2(x as i32, y as i32);
            let tile = map.get_mut(pos);

            if tile.is_water() {
                let height = eval_ocean(time, pos.as_vec2() / vec2(8.0, 4.0));
                let depth = 0.4 + (height * 8.0) as u8 as f32 / 16.0;

                tile.meta[0] = (depth.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
    }

    state.next_run_at = world.ticks + UPDATE_EVERY_TICKS;
}

/// Inspired by https://www.shadertoy.com/view/MdXyzX.
fn eval_ocean(time: f32, pos: Vec2) -> f32 {
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
