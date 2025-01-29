#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x00102000
        li x2, 0x12345678
        sh x2, 0(x1)
        lh x3, -1(x1)
        lh x4, 0(x1)
        lh x5, 1(x1)
        ebreak
    "#
}

/*
 * x1 = 0x00102000
 * x2 = 0x12345678
 * x3 = 0x00007800
 * x4 = 0x00005678
 * x5 = 0x00000056
 */
