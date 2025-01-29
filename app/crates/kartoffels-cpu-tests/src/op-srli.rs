#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0xffffffff
        srli x2, x1, 1
        ebreak
    "#
}

/*
 * x1 = 0xffffffff
 * x2 = 0x7fffffff
 */
