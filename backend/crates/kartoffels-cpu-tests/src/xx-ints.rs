#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

extern crate alloc;
extern crate kartoffel;

use alloc::collections::BinaryHeap;
use core::hint::black_box;
use core::iter::Sum;
use core::ops::{ShlAssign, ShrAssign};
use num_traits::{FromPrimitive, Num, ToPrimitive};

#[cfg_attr(target_arch = "riscv64", no_mangle)]
fn main() {
    let mut rng = Rng { state: 0xcafebabe };

    let out = run::<i8>(&mut rng, 37, 29, 61)
        + run::<u8>(&mut rng, 47, 61, 17)
        + run::<i16>(&mut rng, 4021, 9857, 1667)
        + run::<u16>(&mut rng, 1607, 3187, 4801)
        + run::<i32>(&mut rng, 921563, 989579, 112831)
        + run::<u32>(&mut rng, 606791, 202949, 930737)
        + run::<i64>(&mut rng, 60309923, 21697201, 92053769)
        + run::<u64>(&mut rng, 60549949, 34133111, 35802097)
        + run::<i128>(&mut rng, 3138769811, 1875370859, 4863270187)
        + run::<u128>(&mut rng, 5836095589, 9761164103, 4784206519);

    kartoffels_cpu_tests::exit(out);
}

fn run<T>(rng: &mut Rng, p1: T, p2: T, p3: T) -> u32
where
    T: Num
        + ToPrimitive
        + FromPrimitive
        + ShlAssign<u32>
        + ShrAssign<u32>
        + Sum
        + Ord
        + Copy,
{
    let rng = black_box(rng);
    let mut items = BinaryHeap::new();

    for x in 0..128 {
        for y in 25..=52 {
            let mut val = p1 + T::from_u32(x).unwrap() * p2
                - T::from_u32(y).unwrap() * p3;

            match rng.next() % 6 {
                0 => {}
                1 => {
                    val <<= rng.next() % 64;
                }
                2 => {
                    val >>= rng.next() % 64;
                }
                3 => {
                    val = val * T::from_u8((rng.next() % 123) as u8).unwrap();
                }
                4 => {
                    val =
                        val / T::from_u8((1 + rng.next() % 123) as u8).unwrap();
                }
                _ => {
                    val =
                        val % T::from_u8(1 + (rng.next() % 123) as u8).unwrap();
                }
            }

            if rng.next() <= (u32::MAX / 2) {
                items.push(val);
            }
        }
    }

    items.drain().sum::<T>().to_u128().unwrap_or_default() as u32
}

#[derive(Clone, Debug)]
struct Rng {
    state: u32,
}

impl Rng {
    fn next(&mut self) -> u32 {
        self.state = 1664525 * self.state + 1013904223;
        self.state
    }
}

/*
 * x10 = 3113135538
 */
