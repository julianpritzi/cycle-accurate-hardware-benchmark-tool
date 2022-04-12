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

use alloc::string::String;
use platform::Platform;
use riscv_rt::entry;

extern crate alloc;

fn main() {
    println!("Hello World!");

    let s = String::from("Hello allocated World!");
    for sub_str in s.split_whitespace() {
        println!("{}", sub_str)
    }
}

#[entry]
fn entry() -> ! {
    unsafe { runtime::init().expect("Runtime initialization failed") };

    #[cfg(test)]
    test_main();
    #[cfg(not(test))]
    main();

    platform::current().suspend(0);
}
