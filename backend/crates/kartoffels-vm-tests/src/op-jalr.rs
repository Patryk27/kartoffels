#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _fun:
        add x4, x2, x3
        jalr x1, 0

    _start:
        li x2, 10
        li x3, 20
        jal _fun
        ebreak
    "#
}

/*
 * x1 = 1048584
 * x2 = 10
 * x3 = 20
 * x4 = 30
 */
