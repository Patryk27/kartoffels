#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64ia"

    _start:
        li x1, 0x00102000
        li x2, 123
        sh x2, 0(x1)
        li x2, 321
        amoand.w x10, x2, 0(x1)
        lh x11, 0(x1)
        ebreak
    "#
}

/*
 * x2 = 321
 * x10 = 123
 * x11 = 65
 */
