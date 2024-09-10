#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0xfffffffff
        li x2, 0x1
        srl x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 0xfffffffff
 * x2 = 0x1
 * x3 = 0x7ffffffff
 */
