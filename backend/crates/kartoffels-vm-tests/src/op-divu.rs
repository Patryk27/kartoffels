#![no_std]
#![no_main]

kartoffels_vm_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64im"

    _start:
        li x1, -100
        li x2, 20
        divu x3, x1, x2
        divu x4, x2, x0
        ebreak
    "#
}

/*
 * x1 = -100
 * x2 = 20
 * x3 = 922337203685477575
 * x4 = -1
 */
