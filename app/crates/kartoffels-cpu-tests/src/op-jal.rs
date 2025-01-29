#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _one:
        li x4, 104
        jal _two

    _start:
        li x2, 102
        jal _one

    _two:
        li x3, 103
        ebreak
    "#
}

/*
 * x2 = 102
 * x3 = 103
 * x4 = 104
 */
