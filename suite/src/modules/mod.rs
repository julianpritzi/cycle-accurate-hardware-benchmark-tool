//! A module represents functionality that is provided by some platform.
//!
//! A platform bundles the functionality it supports by including module implementations
//! and providing it with necessary information, like addresses of MMIOs.
//!
//! This file contains traits for all supported modules.
//! This folder includes module implementations that can be used and potentially reused by platforms.

use alloc::string::String;

/// Generic module trait, implemented by all modules.
pub trait Module {
    /// Initialize the current module.
    /// This function has to be called exactly once before using any of the modules functionality.
    /// If an error is encountered during initialization an error message is returned.
    ///
    /// # Safety:
    /// - only call once
    unsafe fn init(&mut self) -> Result<(), &'static str>;

    /// True if the Module has been initialized.
    fn initialized(&self) -> bool;
}

// Module for reading bytes
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
