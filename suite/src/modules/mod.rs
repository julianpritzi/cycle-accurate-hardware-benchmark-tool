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

impl<T> CommunicationModule for T where T: core::fmt::Write + Module + ByteRead {}

/// Module for performing hash computation
pub trait HashingModule: Module {
    /// Setup the module for hashing
    fn init_hashing(&self);

    /// Input a single u32
    ///
    /// # Safety
    ///
    /// - input has to be ready
    unsafe fn write_input(&self, data: u32);

    /// Return true if input is ready
    fn input_ready(&self) -> bool;

    /// Return the number of elements in the internal fifo
    fn get_fifo_elements(&self) -> u32;

    /// Blocks until the module completed computation
    fn wait_for_completion(&self);

    /// Reads the output of the hashing module,
    /// if required also resets the module.
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

type AESStatus = u32;

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
        &mut self,
        key_len: &AESKeyLength,
        operation: AESOperation,
        mode: &AESMode,
        key_share0: &[u32; 8],
        key_share1: &[u32; 8],
        manual: bool,
    );

    /// Writes an input block
    ///
    /// # Safety
    ///
    /// Input has to be ready
    unsafe fn write_block(&self, block: u128);

    /// Waits until more input can be provided
    fn wait_for_input_ready(&self);

    /// Waits until the output can be read
    fn wait_for_output(&self);

    /// Returns a status value
    fn read_status(&self) -> AESStatus;

    /// Triggers the computation
    fn trigger_start(&self);

    /// Returns true if a status value signals that the output was ready
    fn check_if_output_ready(&self, status: AESStatus) -> bool;

    /// Reads an output block
    ///
    /// # Safety
    ///
    /// Output has to be valid
    unsafe fn read_block(&self, block: &mut u128);

    /// Clears the state of the aes module
    fn deinitialize(&self);
}

/// Module for random number generation
pub trait RNGModule: Module {
    /// Initialize the module, optionally provide a seed
    fn init_rng(&self, seed: Option<&[u32]>);

    /// Generate a random number
    fn generate(&self) -> u128;
}

/// An empty module implementing module traits
/// so the empty module can be used for platforms that do not support all modules
///
/// The empty module is impossible to construct and only serves as placeholders for
/// type values.
#[allow(dead_code)]
pub mod empty {
    pub struct EmptyModule {
        /// This field guarantees that EmptyModule can not be constructed
        _priv: (),
    }
    impl super::Module for EmptyModule {
        unsafe fn init(&mut self) -> Result<(), &'static str> {
            unreachable!()
        }

        fn initialized(&self) -> bool {
            unreachable!()
        }
    }
    impl super::HashingModule for EmptyModule {
        fn init_hashing(&self) {
            unreachable!()
        }

        unsafe fn write_input(&self, _: u32) {
            unreachable!()
        }

        fn wait_for_completion(&self) {
            unreachable!()
        }

        fn read_digest(&self, _: &mut [u32; 8]) {
            unreachable!()
        }

        fn input_ready(&self) -> bool {
            unreachable!()
        }

        fn get_fifo_elements(&self) -> u32 {
            unreachable!()
        }
    }
    impl super::AESModule for EmptyModule {
        fn init_aes(
            &mut self,
            _: &super::AESKeyLength,
            _: super::AESOperation,
            _: &super::AESMode,
            _: &[u32; 8],
            _: &[u32; 8],
            _: bool,
        ) {
            unreachable!()
        }

        unsafe fn write_block(&self, _: u128) {
            unreachable!()
        }

        fn wait_for_input_ready(&self) {
            unreachable!()
        }

        fn wait_for_output(&self) {
            unreachable!()
        }

        fn trigger_start(&self) {
            unreachable!()
        }

        fn check_if_output_ready(&self, _: super::AESStatus) -> bool {
            unreachable!()
        }

        unsafe fn read_block(&self, _: &mut u128) {
            unreachable!()
        }

        fn deinitialize(&self) {
            unreachable!()
        }

        fn read_status(&self) -> super::AESStatus {
            unreachable!()
        }
    }
    impl super::RNGModule for EmptyModule {
        fn init_rng(&self, _: Option<&[u32]>) {
            unreachable!()
        }

        fn generate(&self) -> u128 {
            unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        benchmark::{self, datasets},
        mark_test_as_skipped,
    };

    use super::AESOperation;

    #[test_case]
    fn sha2_digest_is_correct1() {
        if let None = benchmark::sha2_benchmark_total(&datasets::hashing::DATASETS[0]) {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn sha2_digest_is_correct2() {
        if let None = benchmark::sha2_benchmark_total(&datasets::hashing::DATASETS[1]) {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn sha3_digest_is_correct1() {
        if let None = benchmark::sha3_benchmark_total(&datasets::hashing::DATASETS[0]) {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn sha3_digest_is_correct2() {
        if let None = benchmark::sha3_benchmark_total(&datasets::hashing::DATASETS[1]) {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_encryption_256_correct1() {
        if let None =
            benchmark::aes_benchmark_total(&datasets::aes::DATASETS[0], AESOperation::Encrypt)
        {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_encryption_256_correct2() {
        if let None =
            benchmark::aes_benchmark_total(&datasets::aes::DATASETS[1], AESOperation::Encrypt)
        {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_encryption_128_correct1() {
        if let None =
            benchmark::aes_benchmark_total(&datasets::aes::DATASETS[2], AESOperation::Encrypt)
        {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_decryption_256_correct1() {
        if let None =
            benchmark::aes_benchmark_total(&datasets::aes::DATASETS[0], AESOperation::Decrypt)
        {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_decryption_256_correct2() {
        if let None =
            benchmark::aes_benchmark_total(&datasets::aes::DATASETS[1], AESOperation::Decrypt)
        {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_decryption_128_correct1() {
        if let None =
            benchmark::aes_benchmark_total(&datasets::aes::DATASETS[2], AESOperation::Decrypt)
        {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn rng_correct1() {
        if let None = benchmark::rng_benchmark_total(&datasets::rng::DATASETS[0]) {
            mark_test_as_skipped!()
        }
    }
}
