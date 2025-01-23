#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x1212121212121212
        andi x2, x1, 0x34
        ebreak
    "#
}

/*
 * x1 = 0x1212121212121212
 * x2 = 0x0000000000000010
 */
