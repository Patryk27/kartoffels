#![no_std]
#![no_main]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x2, -123
        sext.w x3, x2
        ebreak
    "#
}

/*
 * x2 = -123
 * x3 = -123
 */
