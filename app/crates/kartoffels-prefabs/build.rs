use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_src = out.join("lib.rs");
    let out_target = out.join("target.riscv");

    if env::var("TARGET").unwrap() == "riscv32-kartoffel-bot" {
        fs::write(out_src, "").unwrap();
        return;
    }

    // SAFETY: We're single-threaded
    unsafe {
        env::set_var(
            "CARGO_ENCODED_RUSTFLAGS",
            "-Clink-arg=-Triscv32-kartoffel-bot.ld",
        );
    }

    println!("cargo:rerun-if-changed=./src");
    println!("cargo:rerun-if-changed=../kartoffel/src");

    let status = Command::new("cargo")
        .args([
            "build",
            "-p",
            "kartoffels-prefabs",
            "--bins",
            "--release",
            "--target",
            "../../riscv32-kartoffel-bot.json",
            "--target-dir",
            &out_target.display().to_string(),
            "-Z",
            "build-std=alloc,core",
            "-Z",
            "build-std-features=compiler-builtins-mem",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    if !status.success() {
        panic!("`cargo` failed");
    }
}
