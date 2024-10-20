#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x08000000
        sh x0, 0(x1)
    "#
}

/*
 * err = invalid-sized mmio store on 0x08000000+2
 */
