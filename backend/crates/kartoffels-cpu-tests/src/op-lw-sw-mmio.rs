#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x08000000
        li x2, 123
        sw x2, 0(x1)
        lw x2, 0(x1)
        ebreak
    "#
}

/*
 * x1 = 134217728
 * x2 = 15129
 */
