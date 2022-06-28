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

/// Configuration of the key length used by the aes module
#[allow(dead_code)]
pub enum AESKeyLength {
    Aes128,
    Aes192,
    Aes256,
}

/// Configuration of the aes mode used by the aes module
#[allow(dead_code)]
pub enum AESMode {
    ECB,
    /// The iv corresponds to 4 consecutive little endian u32s
    CBC {
        iv: u128,
    },
    CFB,
    OFB,
    /// The iv corresponds to 4 consecutive little endian u32s
    CTR {
        iv: u128,
    },
}

/// Configuration of the operation performed by the aes module
#[allow(dead_code)]
pub enum AESOperation {
    Encrypt,
    Decrypt,
}

/// Module for performing AES en- and decryption
pub trait AESModule: Module {
    /// Setup the AESModule with the provided configuration.
    /// The key used is computed by XORing key_share0 and key_share1.
    ///
    /// # Arguments
    ///
    /// * `key_len` - specifies whether to use AES-128/192/256
    /// * `operation` - whether to encrypt or decrypt the
    /// * `mode` - specifies the AES mode of operation
    /// * `key_share0` - first share of the key
    /// * `key_share1` - second share of the key
    /// * `manual` - true if the aes module does not start computing before `wait_for_manual_output` is called
    fn init_aes(
        &self,
        key_len: &AESKeyLength,
        operation: AESOperation,
        mode: &AESMode,
        key_share0: &[u32; 8],
        key_share1: &[u32; 8],
        manual: bool,
    );

    /// # Safety
    ///
    /// Input has to be ready
    unsafe fn write_block(&self, block: u128);

    fn wait_for_input_ready(&self);

    fn wait_for_output(&self);

    fn wait_for_manual_output(&self);

    /// # Safety
    ///
    /// Output has to be valid
    unsafe fn read_block(&self, block: &mut u128);

    /// Blocks until the SHA256 module completed computation
    fn deinitialize(&self);
}

/// Module for random number generation
pub trait RNGModule: Module {
    /// Initialize the module, optionally provide a seed
    fn init_rng(&self, seed: Option<&[u32]>);

    /// Generate a random number
    fn generate(&self) -> u128;
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
    use alloc::vec;

    use crate::{mark_test_as_skipped, platform, platform::Platform};

    use super::{AESKeyLength, AESMode, AESOperation};

    #[test_case]
    fn sha256_digest_is_correct1() {
        if let Some(hmac_module) = platform::current().get_sha256_module() {
            // TODO: use datasets

            mark_test_as_skipped!()
        } else {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_enc_test() {
        if let Some(aes_module) = platform::current().get_aes_module() {
            // TODO: use datasets

            mark_test_as_skipped!()
        } else {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_dec_test() {
        if let Some(aes_module) = platform::current().get_aes_module() {
            // TODO: use datasets

            mark_test_as_skipped!()
        } else {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn csrng_seeded_is_correct() {
        if let Some(rng) = platform::current().get_rng_module() {
            if cfg!(feature = "platform_nexysvideo_earlgrey") {
                // TODO: use datasets

                mark_test_as_skipped!()
            } else {
                // This test can only be run on nexysvideo because verilator does not simulate the CSRNG module correctly
                mark_test_as_skipped!()
            }
        } else {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn csrng_unseeded_terminates() {
        if let Some(rng) = platform::current().get_rng_module() {
            if cfg!(feature = "platform_nexysvideo_earlgrey") {
                rng.init_rng(None);

                rng.generate();
            } else {
                // This test can only be run on nexysvideo because verilator does not simulate the CSRNG module correctly
                mark_test_as_skipped!()
            }
        } else {
            mark_test_as_skipped!()
        }
    }
}
