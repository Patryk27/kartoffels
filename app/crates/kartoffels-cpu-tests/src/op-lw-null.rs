#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        lw x0, 0(x0)
    "#
}

/*
 * err = null-pointer access on 0x00000000+4
 */
