#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0xffffffff
        slli x2, x1, 1
        ebreak
    "#
}

/*
 * x1 = 0xffffffff
 * x2 = 0x1fffffffe
 */
