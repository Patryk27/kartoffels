#[cfg(target_arch = "riscv64")]
core::arch::global_asm!(
    r#"
    .global _start
    .section .init, "ax"

    _start:
        la sp, _stack_end
        jal main
        ebreak
    "#,
);
