#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64im"

    _start:
        li x1, 112233445566778899
        li x2, 123456
        mulh x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 112233445566778899
 * x2 = 123456
 * x3 = 751
 */
