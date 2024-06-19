#![no_std]
#![no_main]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 10
        li x2, 20
        add x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 10
 * x2 = 20
 * x3 = 30
 */
