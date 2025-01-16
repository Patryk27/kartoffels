#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64ia"

    _start:
        li x1, 0x00102000
        li x2, 0x1212121212121212
        sd x2, 0(x1)
        lr.w x3, 0(x1)
        li x4, 0x34343434
        sc.w x5, x4, 0(x1)
        ld x6, 0(x1)
        ebreak
    "#
}

/*
 * x1 = 0x00102000
 * x2 = 0x1212121212121212
 * x3 = 0x0000000012121212
 * x4 = 0x0000000034343434
 * x5 = 0x0000000000000000
 * x6 = 0x1212121234343434
 */
