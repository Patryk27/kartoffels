#![no_std]
#![no_main]

hellbots_vm_tests::test! {
    r#"
    .global _start

    _start:
        lui x2, 1234
        li x3, 123456789
        ebreak
    "#
}

/*
 * x2 = 5054464
 * x3 = 123456789
 */
