#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64im"

    _start:
        li x1, 0x100000001
        li x2, 0x1
        divu x3, x1, x2

        li x1, 0xffffffffffffffff
        li x2, 0xffffffffffffffff
        divu x4, x1, x2

        li x1, 0x0
        li x2, 0x100000001
        divu x5, x1, x2

        li x1, 0xffffbfffffffffff
        li x2, 0x100000001
        divu x6, x1, x2

        li x1, 0x1
        li x2, 0x0
        divu x7, x1, x2
        ebreak
    "#
}

/*
 * x3 = 0x100000001
 * x4 = 0x1
 * x5 = 0x0
 * x6 = 0xffffbfff
 * x7 = -1
 */
