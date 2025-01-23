#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64im"

    _start:
        li x1, 0xb504f334
        li x2, -0xb504f332
        div x3, x1, x2

        li x1, -0xb504f332
        li x2, -0xb504f332
        div x4, x1, x2

        li x1, -0xb504f332
        li x2, 0xb504f334
        div x5, x1, x2

        li x1, -0x8000000000000000
        li x2, 0xb504f334
        div x6, x1, x2

        li x1, 0x1
        li x2, 0x0
        div x7, x1, x2
        ebreak
    "#
}

/*
 * x3 = -0x1
 * x4 = 0x1
 * x5 = 0x0
 * x6 = -0xb504f333
 * x7 = -1
 */
