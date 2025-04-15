use crate::*;

pub const IRQ_TIMER0: u8 = 0;

pub const IRQ_TIMER1: u8 = 1;

pub const IRQ_TIMER2: u8 = 2;

pub const IRQ_MOTOR_BUSY: u8 = 3;

pub const IRQ_ARM_BUSY: u8 = 4;

pub const IRQ_ARM_STABBED: u8 = 5;

pub const IRQ_ARM_PICKED: u8 = 6;

pub const IRQ_ARM_DROPPED: u8 = 7;

pub const IRQ_RADAR_BUSY: u8 = 8;

pub const IRQ_COMPASS_BUSY: u8 = 9;

pub const IRQ_LEVEL: u8 = 0;

pub const IRQ_RISING: u8 = 1;

pub const IRQ_FALLING: u8 = 2;

pub type IrqFn = extern "C" fn(u32);

pub fn irq_on() {
    wri(MEM_IRQ, 0, cmd(0x01, 0x01, 0x00, 0x00));
}

pub fn irq_off() {
    wri(MEM_IRQ, 0, cmd(0x01, 0x00, 0x00, 0x00));
}

pub fn irq_ack(irq: u8) {
    wri(MEM_IRQ, 0, cmd(0x02, irq, 0x00, 0x00));
}

pub fn irq_set(irq: u8, edge: u8, fun: IrqFn) {
    wri(MEM_IRQ, irq_idx(irq, edge), fun as usize as u32);
}

pub fn irq_get(irq: u8, edge: u8) -> Option<IrqFn> {
    irq_fn(rdi(MEM_IRQ, irq_idx(irq, edge)))
}

pub fn irq_take(irq: u8, edge: u8) -> Option<IrqFn> {
    irq_fn(swi(MEM_IRQ, irq_idx(irq, edge), 0))
}

pub fn irq_replace(irq: u8, edge: u8, fun: IrqFn) -> Option<IrqFn> {
    irq_fn(swi(MEM_IRQ, irq_idx(irq, edge), fun as usize as u32))
}

pub fn irq_idx(irq: u8, edge: u8) -> usize {
    1 + 3 * irq as usize + edge as usize
}

fn irq_fn(ptr: u32) -> Option<IrqFn> {
    if ptr == 0 {
        None
    } else {
        Some(unsafe { core::mem::transmute(ptr as usize) })
    }
}
