#![no_std]
#![no_main]

hellbots_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x2, 123
        not x3, x2
        not x4, x3
        ebreak
    "#
}

/*
 * x2 = 123
 * x3 = -124
 * x4 = 123
 */
