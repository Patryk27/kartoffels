#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv32im"

    _start:
        li x1, 100
        li x2, 3
        div x3, x1, x2

        li x1, 100
        li x2, -3
        div x4, x1, x2

        li x1, -100
        li x2, 3
        div x5, x1, x2

        li x1, -100
        li x2, -3
        div x6, x1, x2

        li x1, 100
        li x2, 0
        div x7, x1, x2

        li x1, 0
        li x2, 100
        div x8, x1, x2

        ebreak
    "#
}

/*
 * x3 = 33
 * x4 = -33
 * x5 = -33
 * x6 = 33
 * x7 = -1
 * x8 = 0
 */
