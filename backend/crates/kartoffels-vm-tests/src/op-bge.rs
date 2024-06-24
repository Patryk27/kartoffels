#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 321
        li x2, 123
        li x3, 50
        bge x1, x2, _branch
        ebreak

    _branch:
        li x3, 60
        ebreak
    "#
}

/*
 * x1 = 321
 * x2 = 123
 * x3 = 60
 */
