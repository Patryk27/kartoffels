#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv32im"

    _start:
        li x1, 100
        li x2, 3
        divu x3, x1, x2

        li x1, 100
        li x2, -3
        divu x4, x1, x2

        li x1, -100
        li x2, 3
        divu x5, x1, x2

        li x1, -100
        li x2, -3
        divu x6, x1, x2

        li x1, 100
        li x2, 0
        divu x7, x1, x2

        li x1, 0
        li x2, 100
        divu x8, x1, x2

        ebreak
    "#
}

/*
 * x3 = 33
 * x4 = 0
 * x5 = 1431655732
 * x6 = 0
 * x7 = -1
 * x8 = 0
 */
