#![no_std]
#![no_main]

use core::panic::PanicInfo;

use riscv::asm::wfi;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    loop {
        unsafe { wfi() }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
