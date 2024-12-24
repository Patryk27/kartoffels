#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x00102000
        li x2, 0x1234567890abcdef
        sh x2, 0(x1)
        lh x3, -1(x1)
        lh x4, 0(x1)
        lh x5, 1(x1)
        ebreak
    "#
}

/*
 * x1 = 0x00102000
 * x2 = 0x1234567890abcdef
 * x3 = 0xffffffffffffef00
 * x4 = 0xffffffffffffcdef
 * x5 = 0x00000000000000cd
 */
