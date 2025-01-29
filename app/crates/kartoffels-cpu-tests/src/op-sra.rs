#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 123
        li x2, 4
        sra x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 123
 * x2 = 4
 * x3 = 7
 */
