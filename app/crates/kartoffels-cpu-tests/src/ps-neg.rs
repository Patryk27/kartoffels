#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x2, 123
        neg x3, x2
        neg x4, x3
        ebreak
    "#
}

/*
 * x2 = 123
 * x3 = -123
 * x4 = 123
 */
