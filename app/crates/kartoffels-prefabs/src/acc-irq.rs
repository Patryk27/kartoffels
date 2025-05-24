#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use kartoffel::*;

fn on_timer0a() {
    print!("tmr0 ");
}

fn on_timer1() {
    print!("tmr1 ");
}

fn on_timer2a() {
    print!("tmr2a ");

    irq_set(IRQ_TIMER2, irq!(on_timer2b));
}

fn on_timer2b() {
    print!("tmr2b ");

    irq_clear(IRQ_TIMER2);
}

fn on_motor_busy() {
    print!("mtr ");
}

fn motor_step_checked() -> Result<(), ()> {
    static mut OK: bool = false;
    static mut PREV: Option<IrqFn> = None;

    fn on_motor_busy(arg: u32) {
        unsafe {
            if let Some(prev) = PREV {
                prev.call(arg);
            }
        }

        print!("mtr2 ");

        unsafe {
            OK = true;
        }
    }

    unsafe {
        OK = false;
        PREV = irq_replace(IRQ_MOTOR_BUSY, irq!(on_motor_busy));
    }

    motor_step();

    unsafe {
        irq_replace(IRQ_MOTOR_BUSY, PREV.take());
    }

    if unsafe { OK } {
        Ok(())
    } else {
        Err(())
    }
}

#[no_mangle]
fn main() {
    print!("boot ");

    irq_set(IRQ_TIMER0, irq!(on_timer0a));
    irq_set(IRQ_TIMER1, irq!(on_timer1));
    irq_set(IRQ_TIMER2, irq!(on_timer2a));
    irq_set(IRQ_MOTOR_BUSY, irq!(on_motor_busy));

    timer_set(TIMER0, TIMER_PS_256, 250);
    timer_set(TIMER1, TIMER_PS_256, 125);
    timer_set(TIMER2, TIMER_PS_256, 40);

    motor_step_checked().unwrap();
    motor_step_checked().unwrap_err();
    motor_wait();
    motor_step_checked().unwrap();
    motor_step_checked().unwrap_err();

    print!("done");
}
