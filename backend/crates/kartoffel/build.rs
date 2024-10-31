use std::path::Path;

fn main() {
    println!(
        "cargo:rustc-link-search={}",
        Path::new(file!()).parent().unwrap().display(),
    );

    println!("cargo:rerun-if-changed=kartoffel.ld");
}
