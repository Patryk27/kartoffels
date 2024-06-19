#![no_std]
#![no_main]

kartoffels_vm_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 112233445566778899
        li x2, 1
        add x3, x1, x2
        addw x4, x1, x2
        ebreak
    "#
}

/*
 * x1 = 112233445566778899
 * x2 = 1
 * x3 = 112233445566778900
 * x4 = 1592593940
 */
