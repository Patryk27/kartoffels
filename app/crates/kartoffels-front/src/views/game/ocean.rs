use glam::{IVec2, Vec2, ivec2, vec2};
use kartoffels_world::prelude as w;

#[derive(Clone, Debug, Default)]
pub struct Ocean {
    n: i32,
    min: IVec2,
    max: IVec2,
    size: IVec2,
    version: u64,
    heightmap: Vec<f32>,
}

impl Ocean {
    pub fn update(&mut self, min: IVec2, max: IVec2, world: &w::Snapshot) {
        let size = max - min;

        if size == ivec2(0, 0) {
            return;
        }

        if min == self.min && max == self.max && world.version == self.version {
            return;
        }

        if min != self.min || max != self.max {
            let mut heightmap = vec![0.0; (size.x * size.y) as usize];

            for y in min.y..max.y {
                for x in min.x..max.x {
                    let at = ivec2(x, y);
                    let off = at - min;
                    let idx = (off.y * size.x + off.x) as usize;

                    heightmap[idx] = self.height(at);
                }
            }

            self.heightmap = heightmap;
        }

        self.min = min;
        self.max = max;
        self.size = self.max - self.min;
        self.version = world.version;

        let time = world.ticks as f32 / 8000.0 / 25.0;

        for _ in 0..256 {
            self.n += 1;

            // let g = 1.32471795724474602596;
            // let a1 = 1.0 / g;
            // let a2 = 1.0 / (g * g);

            // let x = (0.5 + a1 * self.n) % 1.0;
            // let y = (0.5 + a2 * self.n) % 1.0;

            let x = halton(2, self.n);
            let y = halton(3, self.n);

            let x = (x * size.x as f32) as i32;
            let y = (y * size.y as f32) as i32;

            let off = ivec2(x, y);
            let at = self.min + off;
            let idx = (off.y * size.x + off.x) as usize;

            let height = Self::sample(at.as_vec2() / vec2(8.0, 4.0), time);
            let height = 0.4 + (height * 8.0) as u8 as f32 / 16.0;

            self.heightmap[idx] = height;
        }
    }

    pub fn height(&self, at: IVec2) -> f32 {
        let off = at - self.min;

        if off.x >= self.size.x || off.y >= self.size.y {
            return 0.0;
        }

        self.heightmap
            .get((off.y * self.size.x + off.x) as usize)
            .copied()
            .unwrap_or(0.0)
    }

    /// Inspired by https://www.shadertoy.com/view/MdXyzX.
    fn sample(pos: Vec2, time: f32) -> f32 {
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
}

fn halton(b: i32, mut i: i32) -> f32 {
    let mut r = 0.0;
    let mut f = 1.0;

    while i > 0 {
        f = f / (b as f32);
        r = r + f * ((i % b) as f32);
        i = ((i as f32) / (b as f32)).floor() as i32;
    }

    r
}
// fn halton(base: f32, mut index: f32) -> f32 {
//     let mut result = 0.0;
//     let mut f = 1.0;

//     while index > 0.0 {
//         f = f / base;
//         result += f * index % base;
//         index /= base;
//     }

//     result
// }
