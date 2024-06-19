#![no_std]
#![no_main]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 123
        li x2, 321
        and x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 123
 * x2 = 321
 * x3 = 65
 */
