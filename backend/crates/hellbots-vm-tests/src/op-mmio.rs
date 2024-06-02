#![no_std]
#![no_main]

hellbots_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 0x08000000
        li x2, 123
        sw x2, 0(x1)
        lw x2, 0(x1)
        ebreak
    "#
}

/*
 * x1 = 134217728
 * x2 = 15129
 */
