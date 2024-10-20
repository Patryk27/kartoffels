#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64ia"

    _start:
        li x1, 0x08000000
        li x2, 1
        amoadd.w x10, x2, 0(x1)
    "#
}

/*
 * err = unsupported atomic mmio operation on 0x08000000+4
 */
