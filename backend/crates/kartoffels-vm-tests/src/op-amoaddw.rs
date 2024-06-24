#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64ia"

    _start:
        li x1, 0x00101000
        li x2, 1
        sh x2, 0(x1)
        amoadd.w x10, x2, 0(x1)
        amoadd.w x10, x2, 0(x1)
        amoadd.w x10, x2, 0(x1)
        amoadd.w x10, x2, 0(x1)
        lh x11, 0(x1)
        ebreak
    "#
}

/*
 * x1 = 1052672
 * x2 = 1
 * x10 = 4
 * x11 = 5
 */
