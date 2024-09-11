#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 123
        or x2, x1, 321
        ebreak
    "#
}

/*
 * x1 = 123
 * x2 = 379
 */
