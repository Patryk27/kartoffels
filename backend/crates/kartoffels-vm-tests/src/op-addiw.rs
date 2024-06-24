#![cfg_attr(target_arch = "riscv64", no_std, no_main)]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x2, 112233445566778899
        addi x3, x2, 1
        addiw x4, x2, 1
        ebreak
    "#
}

/*
 * x2 = 112233445566778899
 * x3 = 112233445566778900
 * x4 = 1592593940
 */
