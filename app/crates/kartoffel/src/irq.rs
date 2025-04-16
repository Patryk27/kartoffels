use crate::*;

pub const IRQ_TIMER0: u8 = 0;

pub const IRQ_TIMER1: u8 = 1;

pub const IRQ_TIMER2: u8 = 2;

pub const IRQ_MOTOR_BUSY: u8 = 3;

pub const IRQ_MOTOR_IDLE: u8 = 4;

pub const IRQ_ARM_BUSY: u8 = 5;

pub const IRQ_ARM_IDLE: u8 = 6;

pub const IRQ_ARM_STABBED: u8 = 7;

pub const IRQ_ARM_PICKED: u8 = 8;

pub const IRQ_ARM_DROPPED: u8 = 9;

pub const IRQ_RADAR_BUSY: u8 = 10;

pub const IRQ_RADAR_IDLE: u8 = 11;

pub const IRQ_COMPASS_BUSY: u8 = 12;

pub const IRQ_COMPASS_IDLE: u8 = 13;

pub type IrqFn = extern "C" fn(u32);

pub fn irq_on() {
    wri(MEM_IRQ, 0, cmd(0x01, 0x01, 0x00, 0x00));
}

pub fn irq_clear_on() {
    wri(MEM_IRQ, 0, cmd(0x01, 0x02, 0x00, 0x00));
}

pub fn irq_off() {
    wri(MEM_IRQ, 0, cmd(0x01, 0x00, 0x00, 0x00));
}

pub fn irq_set(irq: u8, fun: IrqFn) {
    wri(MEM_IRQ, irq_idx(irq), fun as usize as u32);
}

pub fn irq_get(irq: u8) -> Option<IrqFn> {
    irq_fn(rdi(MEM_IRQ, irq_idx(irq)))
}

pub fn irq_take(irq: u8) -> Option<IrqFn> {
    irq_fn(swi(MEM_IRQ, irq_idx(irq), 0))
}

pub fn irq_replace(irq: u8, fun: IrqFn) -> Option<IrqFn> {
    irq_fn(swi(MEM_IRQ, irq_idx(irq), fun as usize as u32))
}

fn irq_idx(irq: u8) -> usize {
    1 + irq as usize
}

fn irq_fn(ptr: u32) -> Option<IrqFn> {
    if ptr == 0 {
        None
    } else {
        Some(unsafe { core::mem::transmute(ptr as usize) })
    }
}
