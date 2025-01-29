#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 10
        li x2, 20
        sltiu x3, x1, 20
        sltiu x4, x2, 10
        ebreak
    "#
}

/*
 * x1 = 10
 * x2 = 20
 * x3 = 1
 * x4 = 0
 */
