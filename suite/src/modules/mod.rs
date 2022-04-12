use alloc::string::String;

use crate::println;

/// Generic module trait, implemented by all modules
pub trait Module {
    /// Initialize module
    ///
    /// # Safety:
    /// - only call once
    unsafe fn init(&mut self) -> Result<(), &'static str>;

    fn initialized(&self) -> bool;
}

pub trait ByteRead {
    fn read_byte(&self) -> Option<u8>;

    fn read_byte_blocking(&self) -> u8 {
        loop {
            if let Some(val) = self.read_byte() {
                return val;
            }
            core::hint::spin_loop();
        }
    }

    fn read_line(&self) -> String {
        let mut line = String::new();
        loop {
            let c = self.read_byte_blocking() as char;
            if c == '\n' || c == '\r' {
                return line;
            }
            line.push(c);
        }
    }
}

/// Module for communicating with the Benchmarking-CLI
pub trait CommunicationModule: core::fmt::Write + Module + ByteRead {}
