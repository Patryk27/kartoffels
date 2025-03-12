use std::env;
use std::path::Path;

fn main() {
    if env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("riscv32") {
        println!(
            "cargo:rustc-link-search={}",
            Path::new(file!()).parent().unwrap().display(),
        );

        println!("cargo:rerun-if-changed=kartoffel.ld");
    }
}
