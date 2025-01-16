#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        addi x5, x0, 10
        addi x5, x0, 10
        addi x5, x5, 10
        ebreak
    "#
}

/*
 * x5 = 20
 */
