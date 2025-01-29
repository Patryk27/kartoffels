#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv32im"

    _start:
        li x1, 0x12121212
        li x2, -0x34343434
        mul x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 0x12121212
 * x2 = -0x34343434
 * x3 = -0xaaff53a8
 */
