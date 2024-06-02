#![no_std]
#![no_main]

hellbots_vm_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv64im"

    _start:
        li x1, 10
        li x2, 20
        mul x3, x1, x2
        ebreak
    "#
}

/*
 * x1 = 10
 * x2 = 20
 * x3 = 200
 */
