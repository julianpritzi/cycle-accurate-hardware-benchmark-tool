use core::arch::asm;

/// Returns the machine cycle counter
///
/// Adapted from Fig. 10.1 on Page 61 of ["Volume I: RISC-V Unprivileged ISA V20191213"]
/// (https://riscv.org/wp-content/uploads/2019/12/riscv-spec-20191213.pdf)
///
/// The `rdcycle x` and `rdcycleh x` instructions have been replaced by
/// `csrr x, mcycle` and `csrr x, mcycleh` respectively, so this function can be called
/// while being in machine mode.
#[inline]
#[allow(dead_code)]
pub fn get_cycle() -> u64 {
    let counter_lo: u32;
    let counter_hi: u32;

    unsafe {
        asm!(
            "1: csrr {hi_old}, mcycleh",
            "csrr {lo}, mcycle",
            "csrr {hi}, mcycleh",
            "bne {hi_old}, {hi}, 1b",
            hi_old = out(reg) _,
            lo = out(reg) counter_lo,
            hi = out(reg) counter_hi
        );
    }
    ((counter_hi as u64) << 32u64) + counter_lo as u64
}

pub mod examples {
    #![allow(dead_code)]
    use alloc::vec;
    use benchmark_common::BenchmarkResult;

    #[cfg(any(feature = "platform_nexysvideo_earlgrey"))]
    use crate::libs::ecdsa::{
        ecdsa_p256_message_digest_t, ecdsa_p256_private_key_t, ecdsa_p256_public_key_t,
        ecdsa_p256_sign, ecdsa_p256_signature_t, ecdsa_p256_verify, hardened_bool_t,
    };
    use crate::{
        modules::{AESKeyLength, AESMode, AESOperation},
        platform::{self, Platform},
    };

    use super::get_cycle;

    /// Runs an example benchmark for the SHA256 module
    pub fn sha256_benchmark() -> Option<BenchmarkResult> {
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
                0x14b81119,
            ];

            let mut output = [0u32; 8];

            let cycle1 = super::get_cycle();
            hmac_module.init_sha256();
            let cycle2 = super::get_cycle();
            hmac_module.write_input(&input);
            hmac_module.wait_for_completion();
            let cycle3 = super::get_cycle();
            hmac_module.read_digest(&mut output);
            let cycle4 = super::get_cycle();

            assert_eq!(
                output,
                [
                    // precomputed by sha2 crate
                    0xa24ef743, 0xed238e92, 0x8f5fe495, 0x7959a1fa, 0x06b1d250, 0x147ed98d,
                    0xd817e3b2, 0xb32854ae,
                ]
            );

            Some(BenchmarkResult::ExampleSHA256 {
                initialization: cycle2 - cycle1,
                computation: cycle3 - cycle2,
                reading_output: cycle4 - cycle3,
            })
        } else {
            None
        }
    }

    /// Runs an example benchmark for the AES module
    pub fn aes256_benchmark() -> Option<BenchmarkResult> {
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

            let enc_c_1 = get_cycle();
            aes_module.init_aes(
                AESKeyLength::Aes256,
                AESOperation::Encrypt,
                AESMode::CTR { iv },
                &key_share0,
                &key_share1,
            );
            let enc_c_2 = get_cycle();
            aes_module.execute(&plaintext, &mut enc_buffer);
            let enc_c_3 = get_cycle();
            aes_module.deinitialize();
            let enc_c_4 = get_cycle();

            let dec_c_1 = get_cycle();
            aes_module.init_aes(
                AESKeyLength::Aes256,
                AESOperation::Decrypt,
                AESMode::CTR { iv },
                &key_share0,
                &key_share1,
            );
            let dec_c_2 = get_cycle();
            aes_module.execute(&enc_buffer, &mut dec_buffer);
            let dec_c_3 = get_cycle();
            aes_module.deinitialize();
            let dec_c_4 = get_cycle();

            assert_eq!(plaintext, dec_buffer);

            Some(BenchmarkResult::ExampleAES256 {
                enc_initialization: enc_c_2 - enc_c_1,
                enc_computation: enc_c_3 - enc_c_2,
                enc_deinitalization: enc_c_4 - enc_c_3,
                dec_initialization: dec_c_2 - dec_c_1,
                dec_computation: dec_c_3 - dec_c_2,
                dec_deinitalization: dec_c_4 - dec_c_3,
            })
        } else {
            None
        }
    }

    /// Runs an example benchmark for the AES module
    pub fn aes128_benchmark() -> Option<BenchmarkResult> {
        if let Some(aes_module) = platform::current().get_aes_module() {
            let key_share0: [u32; 8] = [
                0x0000_1111,
                0x2222_3333,
                0x4444_5555,
                0x6666_7777,
                0x0000_0000,
                0x0000_0000,
                0x0000_0000,
                0x0000_0000,
            ];
            let key_share1: [u32; 8] = [0; 8];
            let iv = 0xcccc_cccc_cccc_cccc_cccc_cccc_cccc_cccc;
            let plaintext: [u128; 4] = [
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
                0x0000_0000_0000_0000_0000_0000_0000_0000,
                0x0000_1111_2222_3333_4444_5555_6666_7777,
                0x1234_4321_abcd_dcba_affa_afaf_0100_0010,
            ];
            let mut enc_buffer: [u128; 4] = [0; 4];
            let mut dec_buffer: [u128; 4] = [0; 4];

            let enc_c_1 = get_cycle();
            aes_module.init_aes(
                AESKeyLength::Aes128,
                AESOperation::Encrypt,
                AESMode::CTR { iv },
                &key_share0,
                &key_share1,
            );
            let enc_c_2 = get_cycle();
            aes_module.execute(&plaintext, &mut enc_buffer);
            let enc_c_3 = get_cycle();
            aes_module.deinitialize();
            let enc_c_4 = get_cycle();

            let dec_c_1 = get_cycle();
            aes_module.init_aes(
                AESKeyLength::Aes128,
                AESOperation::Decrypt,
                AESMode::CTR { iv },
                &key_share0,
                &key_share1,
            );
            let dec_c_2 = get_cycle();
            aes_module.execute(&enc_buffer, &mut dec_buffer);
            let dec_c_3 = get_cycle();
            aes_module.deinitialize();
            let dec_c_4 = get_cycle();

            assert_eq!(plaintext, dec_buffer);

            Some(BenchmarkResult::ExampleAES128 {
                enc_initialization: enc_c_2 - enc_c_1,
                enc_computation: enc_c_3 - enc_c_2,
                enc_deinitalization: enc_c_4 - enc_c_3,
                dec_initialization: dec_c_2 - dec_c_1,
                dec_computation: dec_c_3 - dec_c_2,
                dec_deinitalization: dec_c_4 - dec_c_3,
            })
        } else {
            None
        }
    }

    /// Runs an example benchmark for the rng module
    pub fn rng_benchmark() -> Option<BenchmarkResult> {
        if let Some(rng_module) = platform::current().get_rng_module() {
            let seed = Some(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            let mut random_numbers1 = [0; 32];
            let mut random_numbers2 = [0; 32];

            let cycle1 = get_cycle();
            rng_module.init_rng(seed);
            let cycle2 = get_cycle();
            for num in &mut random_numbers1[..] {
                *num = rng_module.generate();
            }
            let cycle3 = get_cycle();
            rng_module.init_rng(None);
            let cycle4 = get_cycle();
            for num in &mut random_numbers2[..] {
                *num = rng_module.generate();
            }
            let cycle5 = get_cycle();

            Some(BenchmarkResult::ExampleRNG {
                seeded_initialization: cycle2 - cycle1,
                seeded_generation: cycle3 - cycle2,
                unseeded_initialization: cycle4 - cycle3,
                unseeded_generation: cycle5 - cycle4,
            })
        } else {
            None
        }
    }

    /// Runs an example benchmark for the ecdsa library
    pub fn ecdsa_benchmark() -> Option<BenchmarkResult> {
        #[cfg(any(feature = "platform_nexysvideo_earlgrey"))]
        {
            // public and private part of the ECDSA key was manually generated.
            let priv_key = ecdsa_p256_private_key_t {
                d: [
                    0xe32ae325, 0xba720dd6, 0x7a61c7bf, 0x042a9ce2, 0x1caf1e98, 0xdada301d,
                    0x209ab209, 0x69d57c5c,
                ],
            };
            let pub_key = ecdsa_p256_public_key_t {
                x: [
                    0x2119818f, 0x4bf23e33, 0xa6730cc3, 0x7f88c59f, 0xd73e9dab, 0x0e28969b,
                    0x4560410e, 0xda6152c2,
                ],
                y: [
                    0x9dccc8a7, 0xf2f07fac, 0xb22c083e, 0xf519656d, 0x86ed498a, 0x9eceefab,
                    0x82219250, 0x54b75d6a,
                ],
            };
            let digest = ecdsa_p256_message_digest_t {
                h: [
                    0x9dccc8a7, 0xf2f07fac, 0xb22c083e, 0xf519656d, 0x86ed498a, 0x9eceefab,
                    0x82219250, 0x54b75d6a,
                ],
            };
            let mut signed_digest_buffer = ecdsa_p256_signature_t {
                r: [0; 8],
                s: [0; 8],
            };
            let mut verification_result = hardened_bool_t::HardenedBoolInvalid;

            let c_1 = get_cycle();
            unsafe {
                ecdsa_p256_sign(&digest, &priv_key, &mut signed_digest_buffer);
            }
            let c_2 = get_cycle();
            unsafe {
                ecdsa_p256_verify(
                    &signed_digest_buffer,
                    &digest,
                    &pub_key,
                    &mut verification_result,
                );
            }
            let c_3 = get_cycle();

            assert_eq!(verification_result, hardened_bool_t::HardenedBoolTrue);

            return Some(BenchmarkResult::ExampleECDSA {
                signing: c_2 - c_1,
                verifying: c_3 - c_2,
            });
        }
        #[allow(unreachable_code)]
        None
    }
}
