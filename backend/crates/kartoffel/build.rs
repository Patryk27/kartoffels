use std::env;

fn main() {
    println!(
        "cargo:rustc-link-search={}",
        env::current_dir().unwrap().display(),
    );

    println!("cargo:rerun-if-changed=kartoffel.ld");
}
