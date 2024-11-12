use kartoffels_cpu::{Cpu, Firmware, Mmio};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
        println!("running `{test}`");

        let rs_path = rs_dir.join(&test).with_extension("rs");
        let elf_path = elf_dir.join(&test);

        run_test(rs_path, elf_path);

        println!();
    }
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

fn run_test(rs_path: PathBuf, elf_path: PathBuf) {
    if !rs_path.exists() {
        panic!("file not found: {}", rs_path.display());
    }

    if !elf_path.exists() {
        panic!("file not found: {}", elf_path.display());
    }

    // ---

    let actual = {
        let elf = fs::read(&elf_path).unwrap();
        let fw = Firmware::from_elf(&elf).unwrap();
        let mut cpu = Cpu::new(&fw);
        let mut mmio = TestMmio::default();

        loop {
            match cpu.try_tick(&mut mmio) {
                Ok(true) => continue,
                Ok(false) => break Ok(cpu.regs().to_owned()),
                Err(err) => break Err(err),
            }
        }
    };

    // ---

    let expected = TestExpectation::new(&rs_path);

    match actual {
        Ok(regs) => {
            assert!(expected.err.is_none());

            for (reg_id, reg_val_exp) in expected.regs {
                let reg_val_act = regs[reg_id];

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
        }

        Err(err) => {
            assert_eq!(expected.err.unwrap_or_default(), err.to_string());
            assert!(expected.regs.is_empty());
        }
    }
}

#[derive(Debug, Default)]
struct TestMmio {
    mem: HashMap<u32, u32>,
}

impl Mmio for &mut TestMmio {
    fn load(self, addr: u32) -> Result<u32, ()> {
        self.mem.get(&addr).copied().ok_or(())
    }

    fn store(self, addr: u32, val: u32) -> Result<(), ()> {
        self.mem.insert(addr, val * val);

        Ok(())
    }
}

struct TestExpectation {
    err: Option<String>,
    regs: Vec<(usize, i64)>,
}

impl TestExpectation {
    fn new(path: &Path) -> Self {
        let mut err = None;
        let mut regs = Vec::new();

        let src = fs::read_to_string(path).unwrap();
        let mut lines = src.lines();

        while let Some(mut line) = lines.next() {
            if line == "/*" {
                loop {
                    line = lines.next().unwrap();

                    if line == " */" {
                        break;
                    }

                    let (key, val) = line.split_once('=').unwrap();
                    let key = key.trim().strip_prefix("*").unwrap().trim();
                    let val = val.trim();

                    if key == "err" {
                        err = Some(val.into());
                    } else if key.starts_with("x") {
                        let id = Self::parse_reg_id(key);
                        let val = Self::parse_reg_val(val);

                        regs.push((id, val));
                    } else {
                        panic!("unexpected assertion: {line}");
                    }
                }
            }
        }

        Self { regs, err }
    }

    fn parse_reg_id(s: &str) -> usize {
        s.strip_prefix("x").unwrap().parse().unwrap()
    }

    fn parse_reg_val(mut s: &str) -> i64 {
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
}
