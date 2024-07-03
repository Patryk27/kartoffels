#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0xfffffffff
        srli x2, x1, 1
        ebreak
    "#
}

/*
 * x1 = 0xfffffffff
 * x2 = 0x7ffffffff
 */
