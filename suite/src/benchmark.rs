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
    use benchmark_common::BenchmarkResult;

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
}
