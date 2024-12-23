#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64ia"

    _start:
        li x1, 0x00101000
        li x2, 125
        li x4, 250
        sw x2, 0(x1)
        lr.w x3, 0(x1)
        sc.w x2, x4, 0(x1)
        lw x4, 0(x1)
        ebreak
    "#
}

/*
 * x1 = 1052672
 * x2 = 0
 * x3 = 125
 * x4 = 125
 */
