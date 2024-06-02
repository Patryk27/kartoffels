#![allow(clippy::result_unit_err)]

mod fw;
mod mem;
mod mmio;
mod tick;

pub use self::fw::*;
pub use self::mmio::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Runtime {
    fw: Firmware,
    pc: u64,
    #[serde(with = "serde_bytes")]
    ram: Box<[u8]>,
    regs: Box<[i64; 32]>,
}

impl Runtime {
    const RAM_BASE: u32 = 0x00100000;
    const RAM_SIZE: u32 = 128 * 1024;
    const MMIO_BASE: u32 = 0x08000000;

    pub fn new(fw: Firmware) -> Self {
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

    pub fn tick(&mut self, mmio: &mut impl Mmio) -> Result<bool> {
        self.do_tick(mmio)
    }

    pub fn reset(self) -> Self {
        Self::new(self.fw)
    }
}

#[cfg(test)]
mod tests {
    use super::{Firmware, Mmio, Runtime};
    use itertools::Itertools;
    use pretty_assertions as pa;
    use std::collections::HashMap;
    use std::fmt::Write;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    #[test]
    fn test() {
        let status = Command::new("cargo")
            .arg("build-vm-tests")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .unwrap();

        if !status.success() {
            panic!("couldn't compile tests");
        }

        let tests = extract_tests();

        assert!(!tests.is_empty());

        let rs_dir = Path::new("..").join("hellbots-vm-tests").join("src");

        let elf_dir = Path::new("..")
            .join("..")
            .join("target")
            .join("riscv64-unknown-bot")
            .join("release");

        for test in tests {
            run(
                &test,
                rs_dir.join(&test).with_extension("rs"),
                elf_dir.join(&test),
            );
        }
    }

    fn run(test: &str, rs_path: PathBuf, elf_path: PathBuf) {
        println!("running {}", test);

        if !rs_path.exists() {
            panic!("file not found: {}", rs_path.display());
        }

        if !elf_path.exists() {
            panic!("file not found: {}", elf_path.display());
        }

        // ---

        let elf = fs::read(&elf_path).unwrap();
        let fw = Firmware::new(&elf).unwrap();
        let mut vm = Runtime::new(fw);
        let mut mmio = TestMmio::default();

        while vm.tick(&mut mmio).unwrap() {
            //
        }

        // ---

        let actual = extract_actual(&vm);
        let expected = extract_expected(&rs_path);

        pa::assert_eq!(expected, actual, "test failed: {}", test);

        // ---

        println!();
    }

    fn extract_tests() -> Vec<String> {
        let manifest =
            Path::new("..").join("hellbots-vm-tests").join("Cargo.toml");

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

    fn extract_actual(vm: &Runtime) -> String {
        let regs = vm
            .regs
            .iter()
            .enumerate()
            .filter(|(_, val)| **val != 0)
            .map(|(idx, val)| format!(" * x{} = {}", idx, *val))
            .join("\n");

        let mut out = String::default();

        _ = writeln!(out, "/*");
        _ = writeln!(out, "{}", regs);
        _ = writeln!(out, " */");

        out
    }

    fn extract_expected(path: &Path) -> String {
        let mut out = String::new();
        let src = fs::read_to_string(path).unwrap();
        let mut lines = src.lines();

        while let Some(mut line) = lines.next() {
            if line == "/*" {
                loop {
                    _ = writeln!(out, "{}", line);

                    if line == " */" {
                        break;
                    }

                    line = lines.next().unwrap();
                }
            }
        }

        out
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
