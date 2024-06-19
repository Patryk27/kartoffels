#![no_std]
#![no_main]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 123
        or x2, x1, 321
        ebreak
    "#
}

/*
 * x1 = 123
 * x2 = 379
 */
