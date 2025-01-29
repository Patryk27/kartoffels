#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x08000000
        lh x2, 0(x1)
    "#
}

/*
 * err = missized mmio load on 0x08000000+2
 */
