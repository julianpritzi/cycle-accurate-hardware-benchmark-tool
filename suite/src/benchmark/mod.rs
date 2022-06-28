pub mod datasets;

use core::arch::asm;

use self::datasets::{aes::AesData, rng::RngData, sha256::Sha256Data};
use crate::{
    modules::AESOperation,
    platform::{self, Platform},
};
use alloc::vec;
use alloc::vec::Vec;
use benchmark_common::{AesBlockResult, BenchmarkResult};

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

pub fn sha2_benchmark_total(data_set: &Sha256Data) -> Option<BenchmarkResult> {
    if let Some(hmac_module) = platform::current().get_sha256_module() {
        let mut output = [0u32; 8];

        let cycle1 = get_cycle();
        hmac_module.init_sha256();
        let cycle2 = get_cycle();
        hmac_module.write_input(data_set.data);
        hmac_module.wait_for_completion();
        let cycle3 = get_cycle();
        hmac_module.read_digest(&mut output);
        let cycle4 = get_cycle();

        assert_eq!(&output, data_set.digest);

        Some(BenchmarkResult::SHA2Total {
            initialization: cycle2 - cycle1,
            computation: cycle3 - cycle2,
            reading_output: cycle4 - cycle3,
        })
    } else {
        None
    }
}

pub fn aes_benchmark_per_block(
    data_set: &AesData,
    operation: AESOperation,
) -> Option<BenchmarkResult> {
    if let Some(aes_module) = platform::current().get_aes_module() {
        let block_count = data_set.plaintext.len();
        let (input, output) = match operation {
            AESOperation::Encrypt => (data_set.plaintext, data_set.ciphertext),
            AESOperation::Decrypt => (data_set.ciphertext, data_set.plaintext),
        };
        let mut buffer = vec![0u128; block_count];
        let mut block_timings: Vec<AesBlockResult> = Vec::with_capacity(block_count);

        // Benchmarking the Encryption
        let _init_start = get_cycle();
        aes_module.init_aes(
            &data_set.key_length,
            operation,
            &data_set.mode,
            data_set.key_share0,
            data_set.key_share1,
            true,
        );
        let _init_end = get_cycle();

        for i in 0..block_count {
            unsafe {
                let _c1 = get_cycle();
                aes_module.write_block(input[i]);
                let _c2 = get_cycle();
                aes_module.wait_for_manual_output();
                let _c3 = get_cycle();
                aes_module.read_block(&mut buffer[i]);
                let _c4 = get_cycle();

                block_timings.push(AesBlockResult {
                    write_input: _c2 - _c1,
                    computation: _c3 - _c2,
                    read_output: _c4 - _c3,
                })
            }
        }

        let _deinit_start = get_cycle();
        aes_module.deinitialize();
        let _deinit_end = get_cycle();

        assert_eq!(buffer, output);

        Some(BenchmarkResult::AESPerBlock {
            initialization: _init_end - _init_start,
            blocks: block_timings,
            deinitalization: _deinit_end - _deinit_start,
        })
    } else {
        None
    }
}

pub fn rng_benchmark_total(data_set: &RngData) -> Option<BenchmarkResult> {
    if let Some(rng_module) = platform::current().get_rng_module() {
        let mut random_numbers1 = vec![0; data_set.values.len()];
        let mut random_numbers2 = vec![0; data_set.values.len()];

        let cycle1 = get_cycle();
        rng_module.init_rng(Some(data_set.seed));
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

        assert_eq!(random_numbers1, data_set.values);

        Some(BenchmarkResult::RNGTotal {
            seeded_initialization: cycle2 - cycle1,
            seeded_generation: cycle3 - cycle2,
            unseeded_initialization: cycle4 - cycle3,
            unseeded_generation: cycle5 - cycle4,
        })
    } else {
        None
    }
}
