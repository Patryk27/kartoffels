//! This is the ultimate integer arithmetic test - it goes through most of the
//! supported operators, like multiplication or shifts, executes them randomly
//! and then compares with the final result.
//!
//! The final number was gotten by running this application on a x86_64
//! machine.

#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

extern crate alloc;
extern crate kartoffel;

use alloc::collections::BinaryHeap;
use core::iter::Sum;
use core::ops::{ShlAssign, ShrAssign};
use num_traits::{
    CheckedAdd, CheckedMul, CheckedSub, FromPrimitive, Num, ToPrimitive,
};
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[cfg_attr(target_arch = "riscv32", no_mangle)]
fn main() {
    let mut rng = ChaCha8Rng::from_seed(Default::default());

    let out = run::<i8>(&mut rng)
        + run::<u8>(&mut rng)
        + run::<i16>(&mut rng)
        + run::<u16>(&mut rng)
        + run::<i32>(&mut rng)
        + run::<u32>(&mut rng)
        + run::<i64>(&mut rng)
        + run::<u64>(&mut rng)
        + run::<i128>(&mut rng)
        + run::<u128>(&mut rng);

    kartoffels_cpu_tests::exit(out);
}

#[inline(never)]
fn run<T>(rng: &mut dyn RngCore) -> u32
where
    T: Num
        + ToPrimitive
        + FromPrimitive
        + CheckedAdd
        + CheckedSub
        + CheckedMul
        + ShlAssign<u32>
        + ShrAssign<u32>
        + Sum
        + Ord
        + Copy,
    Standard: Distribution<T>,
{
    // BinaryHeap was chosen for extra benchmarking juice for the memory
    // allocator - Vec would be equally good for testing the operators, but
    // seizing the day let's test everything
    let mut items = BinaryHeap::new();

    for _ in 0..2048 {
        let mut val = rng.gen::<T>();

        match rng.gen::<u8>() % 10 {
            0 => {
                if rng.gen_bool(0.33) {
                    val = val.checked_add(&rng.gen()).unwrap_or(val);
                } else {
                    val = val + rng.gen::<T>();
                }
            }

            1 => {
                if rng.gen_bool(0.33) {
                    val = val.checked_sub(&rng.gen()).unwrap_or(val);
                } else {
                    val = val - rng.gen::<T>();
                }
            }

            2 => {
                if rng.gen_bool(0.33) {
                    val = val.checked_mul(&rng.gen()).unwrap_or(val);
                } else {
                    val = val * rng.gen::<T>();
                }
            }

            3 => {
                val = val / rng.gen::<T>().max(T::one());
            }

            4 => {
                val = val % rng.gen::<T>().max(T::one());
            }

            5 => {
                let rhs = rng.gen::<T>() % T::from_u32(16).unwrap();

                if let Some(rhs) = rhs.to_u32() {
                    val <<= rhs;
                }
            }

            6 => {
                let rhs = rng.gen::<T>() % T::from_u32(16).unwrap();

                if let Some(rhs) = rhs.to_u32() {
                    val >>= rhs;
                }
            }

            _ => (),
        }

        if rng.gen::<bool>() {
            items.push(val);
        }
    }

    items.drain().sum::<T>().to_u128().unwrap_or_default() as u32
}

/*
 * x10 = 1770358021
 */
