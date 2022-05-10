//! A module represents functionality that is provided by some platform.
//!
//! A platform bundles the functionality it supports by including module implementations
//! and providing it with necessary information, like addresses of MMIOs.
//!
//! This file contains traits for all supported modules.
//! This folder includes module implementations that can be used and potentially reused by platforms.
use core::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

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

impl<T> CommunicationModule for T where T: core::fmt::Write + Module + ByteRead {}

/// Module for performing SHA265 hash computation
pub trait SHA256Module: Module {
    /// Setup the Module for SHA256 computation
    fn init_sha256(&self);

    /// Input data into the module, over which the sha hash should be computed
    /// This function accepts &[u32] for performance reasons.
    /// If the data is present as &[u8] try transmuting it to &[u32].
    ///
    /// # Arguments
    ///
    /// * `data` - the data to compute the hash of
    fn write_input(&self, data: &[u32]);

    /// Blocks until the SHA256 module completed computation
    fn wait_for_completion(&self);

    /// Reads the output of the SHA256 module
    ///
    /// # Arguments
    ///
    /// * `buffer` - the buffer into which the digest should be read
    fn read_digest(&self, buffer: &mut [u32; 8]);
}

/// Wrapper for a pointer to a Module
///
/// This wrapper is used as a guarantee that the underlying
/// pointer is valid and to have cleaner return values.
///
/// Automatic dereferencing is implemented, so the reference can be used like the module.
pub struct ModuleRef<T: Module + ?Sized>(NonNull<T>);

impl<T: Module + ?Sized> ModuleRef<T> {
    /// Safety:
    ///  - only call with a valid pointer
    ///  - the pointer has to stay valid
    #[allow(dead_code)]
    pub unsafe fn new(module: *mut T) -> ModuleRef<T> {
        ModuleRef(NonNull::new_unchecked(module))
    }
}

impl<T: Module + ?Sized> Deref for ModuleRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T: Module + ?Sized> DerefMut for ModuleRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

#[cfg(test)]
mod tests {
    use crate::{mark_test_as_skipped, platform, platform::Platform};

    #[test_case]
    fn sha256_digest_is_correct1() {
        if let Some(hmac_module) = platform::current().get_sha256_module() {
            let input = [0u32; 1];
            let mut output = [0u32; 8];

            hmac_module.init_sha256();
            hmac_module.write_input(&input);
            hmac_module.wait_for_completion();
            hmac_module.read_digest(&mut output);

            assert_eq!(
                output,
                [
                    // Precomputed value by sha2 crate
                    0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c,
                    0xe80524c0, 0x14b81119,
                ]
            )
        } else {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn sha256_digest_is_correct2() {
        if let Some(hmac_module) = platform::current().get_sha256_module() {
            let input = [
                0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c, 0xe80524c0,
                0x14b81119, 0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c,
                0xe80524c0, 0x14b81119, 0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc,
                0x52bc498c, 0xe80524c0, 0x14b81119, 0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748,
                0xea778adc, 0x52bc498c, 0xe80524c0, 0x14b81119, 0xdf3f6198, 0x04a92fdb, 0x4057192d,
                0xc43dd748, 0xea778adc, 0x52bc498c, 0xe80524c0, 0x14b81119, 0xdf3f6198, 0x04a92fdb,
                0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c, 0xe80524c0, 0x14b81119, 0xdf3f6198,
                0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c, 0xe80524c0, 0x14b81119,
                0xdf3f6198, 0x04a92fdb, 0x4057192d, 0xc43dd748, 0xea778adc, 0x52bc498c, 0xe80524c0,
            ];
            let mut output = [0u32; 8];

            hmac_module.init_sha256();
            hmac_module.write_input(&input);
            hmac_module.wait_for_completion();
            hmac_module.read_digest(&mut output);

            assert_eq!(
                output,
                [
                    // Precomputed value by sha2 crate
                    0x572ad168, 0x273d0dce, 0x05a098a5, 0x5509de23, 0x70110f07, 0x0b57a5ed,
                    0x910bf83c, 0xbc6c1496,
                ]
            )
        } else {
            mark_test_as_skipped!()
        }
    }
}
