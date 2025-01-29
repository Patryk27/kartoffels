#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv32ia"

    _start:
        li x1, 0x00102000
        li x2, 0x34343434
        sw x2, 0(x1)

        li x2, 0x12121212
        amomax.w x3, x2, 0(x1)
        lw x4, 0(x1)

        li x5, 0x56565656
        amomax.w x6, x5, 0(x1)
        lw x7, 0(x1)

        ebreak
    "#
}

/*
 * x1 = 0x00102000
 * x2 = 0x12121212
 * x3 = 0x34343434
 * x4 = 0x34343434
 * x5 = 0x56565656
 * x6 = 0x34343434
 * x7 = 0x56565656
 */
