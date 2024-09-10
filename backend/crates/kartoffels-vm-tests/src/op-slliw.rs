#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0xffffffff
        slliw x2, x1, 0x1
        ebreak
    "#
}

/*
 * x1 = 0xffffffff
 * x2 = -0x2
 */
