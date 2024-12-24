#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x1212121212121212
        li x2, 0x3434343434343434
        or x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 0x1212121212121212
 * x2 = 0x3434343434343434
 * x3 = 0x3636363636363636
 */
