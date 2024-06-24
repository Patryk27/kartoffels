#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 123
        slli x2, x1, 4
        ebreak
    "#
}

/*
 * x1 = 123
 * x2 = 1968
 */
