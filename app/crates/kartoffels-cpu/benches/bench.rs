#![feature(test)]

extern crate test;

use kartoffels_cpu::{Cpu, Firmware, TickError};
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
        .join("riscv32-kartoffel-bot")
        .join("release")
        .join("xx-ints");

    let elf = fs::read(&elf_path).unwrap();
    let fw = Firmware::from_elf(&elf).unwrap();

    b.iter(|| {
        let mut cpu = Cpu::new(&fw);

        while cpu.tick(()) != Err(TickError::GotEbreak) {
            //
        }
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
