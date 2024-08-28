#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 10
        li x2, 25
        subw x3, x1, x2
        ebreak
    "#
}

/*
 * x3 = -15
 */
