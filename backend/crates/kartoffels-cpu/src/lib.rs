#![allow(clippy::result_unit_err)]
#![feature(test)]

extern crate test;

mod fw;
mod mem;
mod mmio;
mod tick;

use self::fw::*;
pub use self::mmio::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Cpu {
    fw: Firmware,
    pc: u64,
    #[serde(with = "serde_bytes")]
    ram: Box<[u8]>,
    regs: Box<[i64; 32]>,
}

impl Cpu {
    const RAM_BASE: u32 = 0x00100000;
    const RAM_SIZE: u32 = 128 * 1024;
    const MMIO_BASE: u32 = 0x08000000;

    pub fn new(src: &[u8]) -> Result<Self> {
        let fw = Firmware::new(src)?;

        Ok(Self::from_fw(fw))
    }

    fn from_fw(fw: Firmware) -> Self {
        let pc = fw.entry_pc;

        let ram = {
            let mut ram = vec![0; Self::RAM_SIZE as usize].into_boxed_slice();

            for seg in &fw.segments {
                // Unwrap-safety: `Firmware::new()` already checks the bounds
                ram[seg.addr..seg.addr + seg.data.len()]
                    .copy_from_slice(&seg.data);
            }

            ram
        };

        let regs = Box::new([0; 32]);

        Self { fw, pc, ram, regs }
    }

    pub fn tick(&mut self, mmio: &mut dyn Mmio) -> Result<(), Box<str>> {
        self.do_tick(mmio)
    }

    pub fn try_tick(&mut self, mmio: &mut dyn Mmio) -> Result<bool, Box<str>> {
        match self.tick(mmio) {
            Ok(()) => Ok(true),
            Err(err) if err.contains("got `ebreak`") => Ok(false),
            Err(err) => Err(err),
        }
    }

    pub fn reset(self) -> Self {
        Self::from_fw(self.fw)
    }
}

#[cfg(test)]
mod tests {
    use super::{Cpu, Mmio};
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use test::Bencher;

    #[test]
    fn test() {
        build_tests();

        let tests = find_tests();

        assert!(!tests.is_empty());

        let rs_dir = Path::new("..").join("kartoffels-cpu-tests").join("src");

        let elf_dir = Path::new("..")
            .join("..")
            .join("target.riscv")
            .join("riscv64-kartoffel-bot")
            .join("release");

        for test in tests {
            run_test(
                &test,
                rs_dir.join(&test).with_extension("rs"),
                elf_dir.join(&test),
            );
        }
    }

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
        let mut cpu = Some(Cpu::new(&elf).unwrap());
        let mut mmio = TestMmio::default();

        b.iter(|| {
            let cpu_ref = cpu.as_mut().unwrap();

            while cpu_ref.try_tick(&mut mmio).unwrap() {
                //
            }

            cpu = Some(cpu.take().unwrap().reset());
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

    fn find_tests() -> Vec<String> {
        let manifest = Path::new("..")
            .join("kartoffels-cpu-tests")
            .join("Cargo.toml");

        let manifest = fs::read_to_string(manifest).unwrap();

        manifest
            .lines()
            .flat_map(|line| {
                let line = line.strip_prefix("    { name = \"")?;
                let (name, _) = line.split_once('"')?;

                Some(name.to_owned())
            })
            .collect()
    }

    fn run_test(test: &str, rs_path: PathBuf, elf_path: PathBuf) {
        println!("running {}", test);

        if !rs_path.exists() {
            panic!("file not found: {}", rs_path.display());
        }

        if !elf_path.exists() {
            panic!("file not found: {}", elf_path.display());
        }

        // ---

        let elf = fs::read(&elf_path).unwrap();
        let mut cpu = Cpu::new(&elf).unwrap();
        let mut mmio = TestMmio::default();

        while cpu.try_tick(&mut mmio).unwrap() {
            //
        }

        // ---

        for (reg_id, reg_val_exp) in extract_expected_regs(&rs_path) {
            let reg_val_act = cpu.regs[reg_id];

            assert!(
                reg_val_exp == reg_val_act,
                "assertion failed: x{} = {} != {} (0x{:x} != 0x{:x})",
                reg_id,
                reg_val_exp,
                reg_val_act,
                reg_val_exp,
                reg_val_act,
            );
        }

        // ---

        println!();
    }

    fn extract_expected_regs(path: &Path) -> Vec<(usize, i64)> {
        let mut out = Vec::new();
        let src = fs::read_to_string(path).unwrap();
        let mut lines = src.lines();

        while let Some(mut line) = lines.next() {
            if line == "/*" {
                loop {
                    line = lines.next().unwrap();

                    if line == " */" {
                        break;
                    }

                    out.push({
                        let (id, val) = line.split_once('=').unwrap();
                        let id = parse_reg_id(id);
                        let val = parse_reg_val(val);

                        (id, val)
                    });
                }
            }
        }

        out
    }

    fn parse_reg_id(s: &str) -> usize {
        s.trim().strip_prefix("* x").unwrap().parse().unwrap()
    }

    fn parse_reg_val(s: &str) -> i64 {
        let mut s = s.trim();
        let mut neg = false;

        if let Some(s2) = s.strip_prefix("-") {
            s = s2;
            neg = true;
        }

        let val = s
            .strip_prefix("0x")
            .map(|val| u64::from_str_radix(val, 16).unwrap() as i64)
            .unwrap_or_else(|| s.parse().unwrap());

        if neg {
            -val
        } else {
            val
        }
    }

    #[derive(Debug, Default)]
    struct TestMmio {
        mem: HashMap<u32, u32>,
    }

    impl Mmio for TestMmio {
        fn load(&self, addr: u32) -> Result<u32, ()> {
            self.mem.get(&addr).copied().ok_or(())
        }

        fn store(&mut self, addr: u32, val: u32) -> Result<(), ()> {
            self.mem.insert(addr, val * val);

            Ok(())
        }
    }
}
