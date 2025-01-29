use core::ptr::addr_of;
use spin::Mutex;
use talc::*;

extern "C" {
    #[link_name = "_heap_start"]
    static HEAP_START: u32;

    #[link_name = "_heap_end"]
    static HEAP_END: u32;
}

#[allow(dead_code)]
#[cfg_attr(target_arch = "riscv32", global_allocator)]
static ALLOCATOR: Talck<Mutex<()>, ClaimOnOom> = unsafe {
    Talc::new(ClaimOnOom::new(Span::new(
        addr_of!(HEAP_START) as *mut u8,
        addr_of!(HEAP_END) as *mut u8,
    )))
    .lock()
};
