#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 10
        li x2, 20
        sltu x3, x1, x2
        sltu x4, x2, x1
        ebreak
    "#
}

/*
 * x1 = 10
 * x2 = 20
 * x3 = 1
 * x4 = 0
 */
