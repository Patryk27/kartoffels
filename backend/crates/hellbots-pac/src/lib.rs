#![no_std]

use core::{array, ptr};

const MEM: *mut u32 = 0x08000000 as *mut u32;
const MEM_TIMER: *mut u32 = MEM;
const MEM_BATTERY: *mut u32 = MEM.wrapping_byte_add(1024);
const MEM_UART: *mut u32 = MEM.wrapping_byte_add(2 * 1024);
const MEM_MOTOR: *mut u32 = MEM.wrapping_byte_add(3 * 1024);
const MEM_ARM: *mut u32 = MEM.wrapping_byte_add(4 * 1024);
const MEM_RADAR: *mut u32 = MEM.wrapping_byte_add(5 * 1024);

#[inline(always)]
pub fn timer_seed() -> u32 {
    rdi(MEM_TIMER, 0)
}

#[inline(always)]
pub fn timer_ticks() -> u32 {
    rdi(MEM_TIMER, 1)
}

#[inline(always)]
pub fn battery_energy() -> u32 {
    rdi(MEM_BATTERY, 0)
}

#[inline(always)]
pub fn uart_send(ch: char) {
    wri(MEM_UART, 0, ch as u32);
}

#[inline(always)]
pub fn uart_send_str(str: &str) {
    for ch in str.chars() {
        uart_send(ch);
    }
}

#[inline(always)]
pub fn is_motor_ready() -> bool {
    rdi(MEM_MOTOR, 0) == 1
}

#[inline(always)]
pub fn motor_step() {
    wri(MEM_MOTOR, 0, 1);
}

#[inline(always)]
pub fn motor_turn(angle: i32) {
    wri(MEM_MOTOR, 1, angle as u32);
}

#[inline(always)]
pub fn is_arm_ready() -> bool {
    rdi(MEM_ARM, 0) == 1
}

#[inline(always)]
pub fn arm_stab() {
    wri(MEM_ARM, 0, 1)
}

#[inline(always)]
pub fn is_radar_ready() -> bool {
    rdi(MEM_RADAR, 0) == 1
}

#[inline(always)]
pub fn radar_scan(r: u32) {
    wri(MEM_RADAR, 0, r);
}

#[inline(always)]
pub fn radar_read<const D: usize>() -> [[char; D]; D] {
    array::from_fn(|y| {
        array::from_fn(|x| rdi(MEM_RADAR, y * D + x + 1) as u8 as char)
    })
}

#[inline(always)]
fn rdi(ptr: *mut u32, off: usize) -> u32 {
    unsafe { ptr::read_volatile(ptr.wrapping_add(off)) }
}

#[inline(always)]
fn wri(ptr: *mut u32, off: usize, val: u32) {
    unsafe {
        ptr::write_volatile(ptr.wrapping_add(off), val);
    }
}

#[cfg(target_arch = "riscv64")]
#[macro_export]
macro_rules! init {
    (@ $(,)?) => {
        //
    };

    (@ +alloc($size:literal) $(, $($tt:tt)*)?) => {
        use spin::Mutex;
        use talc::*;
        use core::ptr::addr_of;

        static mut ARENA: [u8; $size] = [0; $size];

        #[global_allocator]
        static ALLOCATOR: Talck<Mutex<()>, ClaimOnOom> = Talc::new(unsafe {
            ClaimOnOom::new(Span::from_const_array(addr_of!(ARENA)))
        })
        .lock();

        $crate::init!(@ $($($tt)*)?);
    };

    (@ +panic $(, $($tt:tt)*)?) => {
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            loop {}
        }

        $crate::init!(@ $($($tt)*)?);
    };

    ($($tt:tt)*) => {
        core::arch::global_asm!(
            r#"
            .global _start

            _start:
                la sp, _stack_start
                jal {}
                ebreak
            "#,
            sym main
        );

        $crate::init!(@ $($tt)*);
    };
}

#[cfg(not(target_arch = "riscv64"))]
#[macro_export]
macro_rules! init {
    ($($tt:tt)*) => {
        //
    };
}
