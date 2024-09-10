#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64im"

    _start:
        li x1, 0x100000001
        li x2, 0x1
        mulhu x3, x1, x2

        li x1, 0xffffffffffffffff
        li x2, 0xffffffffffffffff
        mulhu x4, x1, x2

        li x1, 0x0
        li x2, 0x100000001
        mulhu x5, x1, x2

        li x1, 0x100000001
        li x2, 0x100000001
        mulhu x6, x1, x2
        ebreak
    "#
}

/*
 * x3 = 0x0
 * x4 = 0xfffffffffffffffe
 * x5 = 0x0
 * x6 = 0x1
 */
