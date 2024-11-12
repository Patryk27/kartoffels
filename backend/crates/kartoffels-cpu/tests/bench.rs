#![feature(test)]

extern crate test;

use kartoffels_cpu::{Cpu, Firmware, Mmio};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use test::Bencher;

#[bench]
fn bench(b: &mut Bencher) {
    build_tests();

    let elf_path = Path::new("..")
        .join("..")
        .join("target.riscv")
        .join("riscv64-kartoffel-bot")
        .join("release")
        .join("xx-ints");

    let elf = fs::read(&elf_path).unwrap();
    let fw = Firmware::from_elf(&elf).unwrap();
    let mut cpu = Cpu::new(&fw);

    b.iter(|| {
        while cpu.try_tick(TestMmio).unwrap() {
            //
        }

        cpu = Cpu::new(&fw);
    });
}

fn build_tests() {
    let status = Command::new("cargo")
        .arg("build-cpu-tests")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .unwrap();

    if !status.success() {
        panic!("couldn't compile test fixtures");
    }
}

#[derive(Debug, Default)]
struct TestMmio;

impl Mmio for TestMmio {
    fn load(self, _: u32) -> Result<u32, ()> {
        Err(())
    }

    fn store(self, _: u32, _: u32) -> Result<(), ()> {
        Err(())
    }
}
