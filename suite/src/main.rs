#![no_std]
#![no_main]

mod modules;
mod platform;
#[macro_use]
mod runtime;
mod test;

use platform::Platform;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    unsafe { runtime::init().expect("Runtime initialization failed") };

    println!("Hello World");

    platform::current().suspend(0);
}
