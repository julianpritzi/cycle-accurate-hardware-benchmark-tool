#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(default_alloc_error_handler)]
#![test_runner(crate::runtime::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod modules;
mod platform;
#[macro_use]
mod runtime;

use platform::Platform;
use riscv_rt::entry;

extern crate alloc;

fn main() {}

/// First function that is called once riscv_rt finished setting up the rust runtime.
/// (initialize stack pointer, zero bss, initialize data, ...)
///
/// This function then does the following:
/// 1. Initialize the suite specific runtime (heap, modules)
/// 2. Call main() or test_main() depending on the compilation
/// 3. In case main or test_main finish, signal the end of execution to the platform
#[entry]
fn entry() -> ! {
    unsafe { runtime::init().expect("Runtime initialization failed") };

    #[cfg(test)]
    test_main();
    #[cfg(not(test))]
    main();

    platform::current().suspend(0);
}
