#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64ia"

    _start:
        li x1, 0x00102000
        li x2, 0x3434343434343434
        sd x2, 0(x1)

        li x2, -1
        amomax.d x3, x2, 0(x1)
        ld x4, 0(x1)

        li x5, -1
        amomaxu.d x6, x5, 0(x1)
        ld x7, 0(x1)

        ebreak
    "#
}

/*
 * x1 = 0x00102000
 * x2 = -1
 * x3 = 0x3434343434343434
 * x4 = 0x3434343434343434
 * x5 = -1
 * x6 = 0x3434343434343434
 * x7 = -1
 */
