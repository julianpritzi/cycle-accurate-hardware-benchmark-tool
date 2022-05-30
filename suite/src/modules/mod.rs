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

use alloc::{string::String, vec::Vec};

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
    fn init_aes(
        &self,
        key_len: AESKeyLength,
        operation: AESOperation,
        mode: AESMode,
        key_share0: &[u32; 8],
        key_share1: &[u32; 8],
    );

    /// Encrypts/Decrypts the input data.
    /// This function accepts &[u128] for performance reasons.
    /// One u128 is interpreted as 4 consecutive little endian u32s.
    /// If the data is present as &[u32] try transmuting it to &[u128].
    /// If the data is present as &[u8] try converting it to little endian &[u32] before transmuting.
    ///
    /// # Panics
    ///
    /// If the input and output slice do not have the same size.
    ///
    /// # Arguments
    ///
    /// * `input` - the data that should be encrypted
    /// * `output` - a buffer of the same size as input, used for storing the encrypted value
    fn execute(&self, input: &[u128], output: &mut [u128]);

    /// Encrypts/Decrypts the input data in place.
    /// This function accepts &[u128] for performance reasons.
    /// One u128 is interpreted as 4 consecutive little endian u32s.
    /// If the data is present as &[u32] try transmuting it to &[u128].
    /// If the data is present as &[u8] try converting it to little endian &[u32] before transmuting.
    ///
    /// # Panics
    ///
    /// If the input and output slice do not have the same size.
    ///
    /// # Arguments
    ///
    /// * `input` - the data that should be encrypted, will be overwritten with the encrypted message
    fn execute_inplace(&self, data: &mut [u128]);

    /// Blocks until the SHA256 module completed computation
    fn deinitialize(&self);
}

/// Module for random number generation
pub trait RNGModule: Module {
    /// Initialize the module, optionally provide a seed
    fn init_rng(&self, seed: Option<Vec<u32>>);

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
    use crate::{mark_test_as_skipped, platform, platform::Platform};

    use super::{AESKeyLength, AESMode, AESOperation};

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

    #[test_case]
    fn aes_enc_test1() {
        if let Some(aes_module) = platform::current().get_aes_module() {
            let key_share0: [u32; 8] = [
                0x0000_1111,
                0x2222_3333,
                0x4444_5555,
                0x6666_7777,
                0x0000_1111,
                0x2222_3333,
                0x4444_5555,
                0x6666_7777,
            ];
            let key_share1: [u32; 8] = [0; 8];
            let iv = 0x0000_1111_2222_3333_4444_5555_6666_7777u128;
            let plaintext: [u128; 4] = [
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
                0x0000_0000_0000_0000_0000_0000_0000_0000,
                0x0000_1111_2222_3333_4444_5555_6666_7777,
                0x1234_4321_abcd_dcba_affa_afaf_0100_0010,
            ];
            let mut enc_buffer: [u128; 4] = [0, 0, 0, 0];

            aes_module.init_aes(
                AESKeyLength::Aes256,
                AESOperation::Encrypt,
                AESMode::CTR { iv },
                &key_share0,
                &key_share1,
            );
            aes_module.execute(&plaintext, &mut enc_buffer);
            aes_module.deinitialize();

            assert_eq!(
                enc_buffer,
                [
                    // precomputed using the openssl crate
                    0xfd0dcbcab0d253425800853d7c871aa4,
                    0xce1192022849ba635a02a1b9efabe045,
                    0x477120db31cf4dfd849c565ff4f8e932,
                    0x1ac8141b6d63a496c015988d5ac71596,
                ]
            )
        } else {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_enc_test1_inplace() {
        if let Some(aes_module) = platform::current().get_aes_module() {
            let key_share0: [u32; 8] = [
                0x0000_1111,
                0x2222_3333,
                0x4444_5555,
                0x6666_7777,
                0x0000_1111,
                0x2222_3333,
                0x4444_5555,
                0x6666_7777,
            ];
            let key_share1: [u32; 8] = [0; 8];
            let iv = 0x0000_1111_2222_3333_4444_5555_6666_7777u128;
            let mut data: [u128; 4] = [
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
                0x0000_0000_0000_0000_0000_0000_0000_0000,
                0x0000_1111_2222_3333_4444_5555_6666_7777,
                0x1234_4321_abcd_dcba_affa_afaf_0100_0010,
            ];

            aes_module.init_aes(
                AESKeyLength::Aes256,
                AESOperation::Encrypt,
                AESMode::CTR { iv },
                &key_share0,
                &key_share1,
            );
            aes_module.execute_inplace(&mut data);
            aes_module.deinitialize();

            assert_eq!(
                data,
                [
                    // precomputed using the openssl crate
                    0xfd0dcbcab0d253425800853d7c871aa4,
                    0xce1192022849ba635a02a1b9efabe045,
                    0x477120db31cf4dfd849c565ff4f8e932,
                    0x1ac8141b6d63a496c015988d5ac71596,
                ]
            )
        } else {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_dec_test1() {
        if let Some(aes_module) = platform::current().get_aes_module() {
            let key_share0: [u32; 8] = [
                0x8561_6e27,
                0xfcd8_ab2d,
                0x6218_cd69,
                0xb876_335b,
                0xe75a_5245,
                0xaa1d_9e75,
                0x553f_3be1,
                0x4fd6_4b05,
            ];
            let key_share1: [u32; 8] = [0; 8];
            let iv = 0xfb12_60c5_8b69_93a7_b8c7_7c6e_464a_a903u128;
            let ciphertext: [u128; 5] = [
                0x7b436a4b7d3f339be5e7177bd8921e2f,
                0x1eefd500fd21234297170d075150b292,
                0xbaedb76067736877ac26e465251f1c3a,
                0xfb7511bf323f8851ee66e9c253a07f02,
                0x9255ff0a9b062e8759bd262ee56526bd,
            ];
            let mut enc_buffer: [u128; 5] = [0; 5];

            aes_module.init_aes(
                AESKeyLength::Aes256,
                AESOperation::Decrypt,
                AESMode::CTR { iv },
                &key_share0,
                &key_share1,
            );
            aes_module.execute(&ciphertext, &mut enc_buffer);
            aes_module.deinitialize();

            assert_eq!(
                enc_buffer,
                [
                    // precomputed using the openssl crate
                    0x12bb_b300_8e5d_392b_eeab_2332_be17_833eu128,
                    0xa1f0_6916_0d57_f83a_a0ba_1311_1e98_709f,
                    0x3d05_7c8c_6f2a_1b6e_bf50_2dcb_38cd_60d8,
                    0x7c6d_2b00_3232_d98b_a452_627a_fe2f_23dc,
                    0xb491_10e4_8ad8_3e04_20e4_348a_82ce_cf15,
                ]
            )
        } else {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_dec_test1_inplace() {
        if let Some(aes_module) = platform::current().get_aes_module() {
            let key_share0: [u32; 8] = [
                0x8561_6e27,
                0xfcd8_ab2d,
                0x6218_cd69,
                0xb876_335b,
                0xe75a_5245,
                0xaa1d_9e75,
                0x553f_3be1,
                0x4fd6_4b05,
            ];
            let key_share1: [u32; 8] = [0; 8];
            let iv = 0xfb12_60c5_8b69_93a7_b8c7_7c6e_464a_a903u128;
            let mut data: [u128; 5] = [
                0x7b436a4b7d3f339be5e7177bd8921e2f,
                0x1eefd500fd21234297170d075150b292,
                0xbaedb76067736877ac26e465251f1c3a,
                0xfb7511bf323f8851ee66e9c253a07f02,
                0x9255ff0a9b062e8759bd262ee56526bd,
            ];

            aes_module.init_aes(
                AESKeyLength::Aes256,
                AESOperation::Decrypt,
                AESMode::CTR { iv },
                &key_share0,
                &key_share1,
            );
            aes_module.execute_inplace(&mut data);
            aes_module.deinitialize();

            assert_eq!(
                data,
                [
                    // precomputed using the openssl crate
                    0x12bb_b300_8e5d_392b_eeab_2332_be17_833eu128,
                    0xa1f0_6916_0d57_f83a_a0ba_1311_1e98_709f,
                    0x3d05_7c8c_6f2a_1b6e_bf50_2dcb_38cd_60d8,
                    0x7c6d_2b00_3232_d98b_a452_627a_fe2f_23dc,
                    0xb491_10e4_8ad8_3e04_20e4_348a_82ce_cf15,
                ]
            )
        } else {
            mark_test_as_skipped!()
        }
    }

    #[test_case]
    fn aes_dec_of_enc_is_correct() {
        if let Some(aes_module) = platform::current().get_aes_module() {
            let key_share0: [u32; 8] = [
                0x0000_1111,
                0x2222_3333,
                0x4444_5555,
                0x6666_7777,
                0x0000_1111,
                0x2222_3333,
                0x4444_5555,
                0x6666_7777,
            ];
            let key_share1: [u32; 8] = [0; 8];
            let iv = 0xcccc_cccc_cccc_cccc_cccc_cccc_cccc_cccc;
            let plaintext: [u128; 4] = [
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
                0x0000_0000_0000_0000_0000_0000_0000_0000,
                0x0000_1111_2222_3333_4444_5555_6666_7777,
                0x1234_4321_abcd_dcba_affa_afaf_0100_0010,
            ];
            let mut enc_buffer: [u128; 4] = [0, 0, 0, 0];
            let mut dec_buffer: [u128; 4] = [0, 0, 0, 0];

            aes_module.init_aes(
                AESKeyLength::Aes256,
                AESOperation::Encrypt,
                AESMode::CTR { iv },
                &key_share0,
                &key_share1,
            );
            aes_module.execute(&plaintext, &mut enc_buffer);
            aes_module.deinitialize();

            aes_module.init_aes(
                AESKeyLength::Aes256,
                AESOperation::Decrypt,
                AESMode::CTR { iv },
                &key_share0,
                &key_share1,
            );
            aes_module.execute(&enc_buffer, &mut dec_buffer);
            aes_module.deinitialize();

            assert_eq!(plaintext, dec_buffer);
        } else {
            mark_test_as_skipped!()
        }
    }
}
