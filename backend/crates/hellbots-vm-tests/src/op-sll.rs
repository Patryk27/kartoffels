#![no_std]
#![no_main]

hellbots_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 123
        li x2, 4
        sll x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 123
 * x2 = 4
 * x3 = 1968
 */
