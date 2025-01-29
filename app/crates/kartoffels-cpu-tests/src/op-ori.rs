#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x12121212
        ori x2, x1, 0x34
        ebreak
    "#
}

/*
 * x1 = 0x12121212
 * x2 = 0x12121236
 */
