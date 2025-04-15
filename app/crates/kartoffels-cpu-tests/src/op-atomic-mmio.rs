#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv32ia"

    _start:
        li x1, 0x08000002
        amoadd.w x0, x0, 0(x1)
    "#
}

/*
 * err = invalid access on 0x08000002+4
 */
