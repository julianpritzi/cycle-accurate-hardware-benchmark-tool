#![no_std]
#![no_main]

use core::panic::PanicInfo;

use riscv_rt::entry;

mod modules;
mod platform;

use platform::Platform;

#[entry]
fn main() -> ! {
    if let Some(comm) = platform::current().get_communication_module() {
        unsafe { comm.init().unwrap() };
        writeln!(comm, "Hello World!").unwrap();
    }

    platform::current().suspend(0);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(comm) = platform::current().get_communication_module() {
        unsafe {
            let _ = comm.init();
        }
        let _ = writeln!(comm, "! {}", info);
    }

    platform::current().suspend(101);
}
