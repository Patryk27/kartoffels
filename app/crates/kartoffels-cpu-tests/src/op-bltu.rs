#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 123
        li x2, 321
        li x3, 50
        bltu x1, x2, _branch
        ebreak

    _branch:
        li x3, 60
        ebreak
    "#
}

/*
 * x1 = 123
 * x2 = 321
 * x3 = 60
 */
