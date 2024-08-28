use std::env;

fn main() {
    println!(
        "cargo:rustc-link-search={}",
        env::current_dir().unwrap().join("misc").display()
    );

    println!("cargo:rerun-if-changed=misc/kartoffel.ld");
}
