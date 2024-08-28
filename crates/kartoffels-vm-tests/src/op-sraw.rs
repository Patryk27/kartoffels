#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 123
        li x2, 4
        sraw x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 123
 * x2 = 4
 * x3 = 7
 */
