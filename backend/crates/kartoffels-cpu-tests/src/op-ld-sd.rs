#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x00102000
        li x2, 0x1234567890abcdef
        sd x2, 0(x1)
        ld x3, -1(x1)
        ld x4, 0(x1)
        ld x5, 1(x1)
        ebreak
    "#
}

/*
 * x1 = 0x00102000
 * x2 = 0x1234567890abcdef
 * x3 = 0x34567890abcdef00
 * x4 = 0x1234567890abcdef
 * x5 = 0x001234567890abcd
 */
