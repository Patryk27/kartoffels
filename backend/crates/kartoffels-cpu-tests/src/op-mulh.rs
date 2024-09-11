#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64im"

    _start:
        li x1, 0xb504f334
        li x2, -0xb504f332
        mulh x3, x1, x2

        li x1, -0xb504f332
        li x2, -0xb504f332
        mulh x4, x1, x2

        li x1, -0xb504f332
        li x2, 0xb504f334
        mulh x5, x1, x2

        li x1, -0x8000000000000000
        li x2, 0xb504f334
        mulh x6, x1, x2
        ebreak
    "#
}

/*
 * x3 = 0xffffffffffffffff
 * x4 = 0x0
 * x5 = 0xffffffffffffffff
 * x6 = 0xffffffffa57d8666
 */
