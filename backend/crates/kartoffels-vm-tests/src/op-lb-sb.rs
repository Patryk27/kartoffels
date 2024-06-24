#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x00101000
        li x2, 125
        sb x2, 0(x1)
        lb x3, -1(x1)
        lb x4, 0(x1)
        lb x5, 1(x1)
        ebreak
    "#
}

/*
 * x1 = 1052672
 * x2 = 125
 * x4 = 125
 */
