#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 10
        li x2, 25
        sub x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 10
 * x2 = 25
 * x3 = -15
 */
