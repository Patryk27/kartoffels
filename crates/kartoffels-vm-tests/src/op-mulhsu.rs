#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64im"

    _start:
        li x1, 0xb504f334
        li x2, 0x100000001
        mulhsu x3, x1, x2

        li x1, -0xb504f332
        li x2, -0xb504f332
        mulhsu x4, x1, x2

        li x1, -0x8000000000000000
        li x2, 0x100000001
        mulhsu x5, x1, x2
        ebreak
    "#
}

/*
 * x3 = 0x0
 * x4 = 0xffffffff4afb0cce
 * x5 = 0xffffffff7fffffff
 */
