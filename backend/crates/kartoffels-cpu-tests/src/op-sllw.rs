#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0xffffffff
        li x2, 0x1
        sllw x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 0xffffffff
 * x2 = 0x1
 * x3 = -0x2
 */
