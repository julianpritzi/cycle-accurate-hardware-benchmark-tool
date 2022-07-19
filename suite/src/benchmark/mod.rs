pub mod datasets;

use core::arch::asm;

use self::datasets::{aes::AesData, hashing::HashingData, rng::RngData};
use crate::{
    modules::{AESKeyLength, AESModule, AESOperation, HashingModule, RNGModule},
    platform::{self, Platform},
};
use alloc::vec::Vec;
use alloc::{string::String, vec};
use benchmark_common::{AesBlockResult, BenchmarkResult};
use seq_macro::seq;

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

/// Performs a sha2 benchmark using the provided dataset
pub fn sha2_benchmark_total(data_set: &HashingData) -> Option<BenchmarkResult> {
    if let Some(sha2_module) = platform::current().get_sha2_module() {
        let mut output = [0u32; 8];

        let cycle1 = get_cycle();
        sha2_module.init_hashing();
        let cycle2 = get_cycle();
        sha2_module.write_input(data_set.data);
        sha2_module.wait_for_completion();
        let cycle3 = get_cycle();
        sha2_module.read_digest(&mut output);
        let cycle4 = get_cycle();

        assert_eq!(&output, data_set.sha2_digest);

        Some(BenchmarkResult::SHA2Total {
            initialization: cycle2 - cycle1,
            computation: cycle3 - cycle2,
            reading_output: cycle4 - cycle3,
        })
    } else {
        None
    }
}

/// Performs a sha3 benchmark using the provided dataset
pub fn sha3_benchmark_total(data_set: &HashingData) -> Option<BenchmarkResult> {
    if let Some(sha3_module) = platform::current().get_sha3_module() {
        let mut output = [0u32; 8];

        let cycle1 = get_cycle();
        sha3_module.init_hashing();
        let cycle2 = get_cycle();
        sha3_module.write_input(data_set.data);
        sha3_module.wait_for_completion();
        let cycle3 = get_cycle();
        sha3_module.read_digest(&mut output);
        let cycle4 = get_cycle();

        assert_eq!(&output, data_set.sha3_digest);

        Some(BenchmarkResult::SHA3Total {
            initialization: cycle2 - cycle1,
            computation: cycle3 - cycle2,
            reading_output: cycle4 - cycle3,
        })
    } else {
        None
    }
}

/// Performs an aes benchmark with manual computation using the provided dataset and operation
pub fn aes_benchmark_per_block(
    data_set: &AesData,
    operation: AESOperation,
    tight_implementations: bool,
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
            if tight_implementations {
                bench_aes_block_tight(
                    input[i],
                    &mut buffer[i],
                    aes_module,
                    &mut block_timings,
                    &data_set.key_length,
                );
            } else {
                bench_aes_block_normal(input[i], &mut buffer[i], aes_module, &mut block_timings);
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

pub fn bench_aes_block_normal(
    data_in: u128,
    data_out: &mut u128,
    aes_module: &mut platform::module_types::AESModule,
    block_timings: &mut Vec<AesBlockResult>,
) {
    unsafe {
        let _c1 = get_cycle();
        aes_module.write_block(data_in);
        let _c2 = get_cycle();
        aes_module.trigger_start();
        aes_module.wait_for_output();
        let _c3 = get_cycle();
        aes_module.read_block(data_out);
        let _c4 = get_cycle();

        block_timings.push(AesBlockResult {
            write_input: _c2 - _c1,
            computation: _c3 - _c2,
            read_output: _c4 - _c3,
        })
    }
}

/// Use predetermined snippets to wait an exact number of cycles
/// Reading the output now should return a status that signals the output is ready, this will be verified later
pub fn bench_aes_block_tight(
    data_in: u128,
    data_out: &mut u128,
    aes_module: &mut platform::module_types::AESModule,
    block_timings: &mut Vec<AesBlockResult>,
    key_len: &AESKeyLength,
) {
    match key_len {
        AESKeyLength::Aes128 => unsafe {
            let _c1 = get_cycle();
            aes_module.write_block(data_in);
            let _c2 = get_cycle();
            aes_module.trigger_start();
            seq!(N in 0..8 { asm!("nop"); });
            #[cfg(feature = "opentitan_aes_masking")]
            {
                seq!(N in 0..44 { asm!("nop"); });
            }
            let status = aes_module.read_status();
            let _c3 = get_cycle();
            aes_module.read_block(data_out);
            let _c4 = get_cycle();

            // Verify that the output was signalled as being ready
            assert!(aes_module.check_if_output_ready(status));

            block_timings.push(AesBlockResult {
                write_input: _c2 - _c1,
                computation: _c3 - _c2,
                read_output: _c4 - _c3,
            })
        },
        AESKeyLength::Aes192 => unsafe {
            let _c1 = get_cycle();
            aes_module.write_block(data_in);
            let _c2 = get_cycle();
            aes_module.trigger_start();
            seq!(N in 0..10 { asm!("nop"); });
            #[cfg(feature = "opentitan_aes_masking")]
            {
                seq!(N in 0..52 { asm!("nop"); });
            }
            let status = aes_module.read_status();
            let _c3 = get_cycle();
            aes_module.read_block(data_out);
            let _c4 = get_cycle();

            // Verify that the output was signalled as being ready
            assert!(aes_module.check_if_output_ready(status));

            block_timings.push(AesBlockResult {
                write_input: _c2 - _c1,
                computation: _c3 - _c2,
                read_output: _c4 - _c3,
            })
        },
        AESKeyLength::Aes256 => unsafe {
            let _c1 = get_cycle();
            aes_module.write_block(data_in);
            let _c2 = get_cycle();
            aes_module.trigger_start();
            seq!(N in 0..12 { asm!("nop"); });
            #[cfg(feature = "opentitan_aes_masking")]
            {
                seq!(N in 0..56 { asm!("nop"); });
            }
            let status = aes_module.read_status();
            let _c3 = get_cycle();
            aes_module.read_block(data_out);
            let _c4 = get_cycle();

            // Verify that the output was signalled as being ready
            assert!(aes_module.check_if_output_ready(status));

            block_timings.push(AesBlockResult {
                write_input: _c2 - _c1,
                computation: _c3 - _c2,
                read_output: _c4 - _c3,
            })
        },
    }
}

/// Performs an aes benchmark with automatic computation using the provided dataset and operation
pub fn aes_benchmark_total(data_set: &AesData, operation: AESOperation) -> Option<BenchmarkResult> {
    if let Some(aes_module) = platform::current().get_aes_module() {
        let block_count = data_set.plaintext.len();
        let (input, output) = match operation {
            AESOperation::Encrypt => (data_set.plaintext, data_set.ciphertext),
            AESOperation::Decrypt => (data_set.ciphertext, data_set.plaintext),
        };
        let mut buffer = vec![0u128; block_count];

        // Benchmarking the Encryption
        let _init_start = get_cycle();
        aes_module.init_aes(
            &data_set.key_length,
            operation,
            &data_set.mode,
            data_set.key_share0,
            data_set.key_share1,
            false,
        );
        let _init_end = get_cycle();

        let _computation_start = get_cycle();
        unsafe {
            aes_module.write_block(input[0]);
            aes_module.wait_for_input_ready();
            aes_module.write_block(input[1]);

            for i in 2..block_count {
                aes_module.wait_for_output();
                aes_module.read_block(&mut buffer[i - 2]);
                aes_module.write_block(input[i]);
            }

            aes_module.wait_for_output();
            aes_module.read_block(&mut buffer[block_count - 2]);
            aes_module.wait_for_output();
            aes_module.read_block(&mut buffer[block_count - 1]);
        }
        let _computation_end = get_cycle();

        let _deinit_start = get_cycle();
        aes_module.deinitialize();
        let _deinit_end = get_cycle();

        assert_eq!(buffer, output);

        Some(BenchmarkResult::AESTotal {
            initialization: _init_end - _init_start,
            computation: _computation_end - _computation_start,
            deinitalization: _deinit_end - _deinit_start,
        })
    } else {
        None
    }
}

/// Performs an rng benchmark using the provided dataset and operation
pub fn rng_benchmark_total_seeded(data_set: &RngData) -> Option<BenchmarkResult> {
    if let Some(rng_module) = platform::current().get_rng_module() {
        let mut random_numbers1 = vec![0; data_set.values.len()];
        let mut random_numbers2 = vec![0; data_set.values.len()];
        let mut seeded_blocks: Vec<u64> = Vec::with_capacity(data_set.values.len());
        let mut seeded_wait_blocks: Vec<u64> = Vec::with_capacity(data_set.values.len());

        let seeded_init_s = get_cycle();
        rng_module.init_rng(Some(data_set.seed));
        let seeded_init_e = get_cycle();
        for num in &mut random_numbers1[..] {
            let c1 = get_cycle();
            *num = rng_module.generate();
            let c2 = get_cycle();

            seeded_blocks.push(c2 - c1);
        }

        let seeded_wait_init_s = get_cycle();
        rng_module.init_rng(Some(data_set.seed));
        let seeded_wait_init_e = get_cycle();
        unsafe {
            seq!(N in 0..53 { asm!("nop"); });
        }
        for num in &mut random_numbers2[..] {
            let c1 = get_cycle();
            *num = rng_module.generate();
            let c2 = get_cycle();

            seeded_wait_blocks.push(c2 - c1);
        }

        assert_eq!(random_numbers1, data_set.values);

        Some(BenchmarkResult::RNGTotalSeeded {
            seeded_initialization: seeded_init_e - seeded_init_s,
            seeded_generation: seeded_blocks,
            seeded_wait_initialization: seeded_wait_init_e - seeded_wait_init_s,
            seeded_wait_generation: seeded_wait_blocks,
        })
    } else {
        None
    }
}

/// Performs an rng benchmark using entropy seed
pub fn rng_benchmark_true_random(blocks: usize) -> Option<BenchmarkResult> {
    if let Some(rng_module) = platform::current().get_rng_module() {
        let mut random_numbers = vec![0; blocks];
        let mut unseeded_blocks: Vec<u64> = Vec::with_capacity(blocks);
        let mut unseeded_wait_blocks: Vec<u64> = Vec::with_capacity(blocks);

        let unseeded_init_s = get_cycle();
        rng_module.init_rng(None);
        let unseeded_init_e = get_cycle();
        for num in &mut random_numbers[..] {
            let c1 = get_cycle();
            *num = rng_module.generate();
            let c2 = get_cycle();

            unseeded_blocks.push(c2 - c1);
        }

        let unseeded_wait_init_s = get_cycle();
        rng_module.init_rng(None);
        let unseeded_wait_init_e = get_cycle();
        unsafe {
            seq!(N in 0..53 { asm!("nop"); });
        }
        for num in &mut random_numbers[..] {
            let c1 = get_cycle();
            *num = rng_module.generate();
            let c2 = get_cycle();

            unseeded_wait_blocks.push(c2 - c1);
        }

        Some(BenchmarkResult::RNGTotalTrueRandom {
            unseeded_initialization: unseeded_init_e - unseeded_init_s,
            unseeded_generation: unseeded_blocks,
            unseeded_wait_initialization: unseeded_wait_init_e - unseeded_wait_init_s,
            unseeded_wait_generation: unseeded_wait_blocks,
        })
    } else {
        None
    }
}

/// Runs an example benchmark for the ecdsa library
#[allow(dead_code)]
pub fn ecdsa_benchmark() -> Option<String> {
    #[cfg(any(feature = "platform_nexysvideo_earlgrey"))]
    {
        use crate::libs::ecdsa::{
            ecdsa_p256_message_digest_t, ecdsa_p256_private_key_t, ecdsa_p256_public_key_t,
            ecdsa_p256_sign, ecdsa_p256_signature_t, ecdsa_p256_verify, hardened_bool_t,
        };

        // public and private part of the ECDSA key was manually generated.
        let priv_key = ecdsa_p256_private_key_t {
            d: [
                0xe32ae325, 0xba720dd6, 0x7a61c7bf, 0x042a9ce2, 0x1caf1e98, 0xdada301d, 0x209ab209,
                0x69d57c5c,
            ],
        };
        let pub_key = ecdsa_p256_public_key_t {
            x: [
                0x2119818f, 0x4bf23e33, 0xa6730cc3, 0x7f88c59f, 0xd73e9dab, 0x0e28969b, 0x4560410e,
                0xda6152c2,
            ],
            y: [
                0x9dccc8a7, 0xf2f07fac, 0xb22c083e, 0xf519656d, 0x86ed498a, 0x9eceefab, 0x82219250,
                0x54b75d6a,
            ],
        };
        let digest = ecdsa_p256_message_digest_t {
            h: [
                0x9dccc8a7, 0xf2f07fac, 0xb22c083e, 0xf519656d, 0x86ed498a, 0x9eceefab, 0x82219250,
                0x54b75d6a,
            ],
        };
        let mut signed_digest_buffer = ecdsa_p256_signature_t {
            r: [0; 8],
            s: [0; 8],
        };
        let mut verification_result = hardened_bool_t::HardenedBoolInvalid;

        println!("start signing");
        let c_1 = get_cycle();
        unsafe {
            ecdsa_p256_sign(&digest, &priv_key, &mut signed_digest_buffer);
        }
        let c_2 = get_cycle();
        println!("stop signing");
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

        return Some(alloc::format!("sign: {}, verify {}", c_2 - c_1, c_3 - c_2));
    }
    #[allow(unreachable_code)]
    None
}

/// Performs a number of microbenchmarks for adjusting the other measurements
pub fn micro_benchmarks() -> Option<BenchmarkResult> {
    // Measure overhead of a getcycle operation
    let _get_cycle_overhead_s = get_cycle();
    let _get_cycle_overhead_e = get_cycle();

    // Measure overhead of a empty inlined function call
    let _empty_call_overhead_s = get_cycle();
    micro_benchmarks::do_nothing();
    let _empty_call_overhead_e = get_cycle();

    // Measure overhead of a function returning the single argument
    let _call_and_return_overhead_s = get_cycle();
    let x = micro_benchmarks::return_argument(42);
    let _call_and_return_overhead_e = get_cycle();
    assert!(x == 42);

    // Measure overhead of a function returning the a value
    let _return_only_overhead_s = get_cycle();
    let x = micro_benchmarks::return_42();
    let _return_only_overhead_e = get_cycle();
    assert!(x == 42);

    // Measure overhead of a function writing to u32 buffer
    let mut x = 0u32;
    let _write_u32_overhead_s = get_cycle();
    micro_benchmarks::write_42u32(&mut x);
    let _write_u32_overhead_e = get_cycle();
    assert!(x == 42);

    // Measure overhead of a function writing to u128 buffer
    let mut x = 0u128;
    let _write_u128_overhead_s = get_cycle();
    micro_benchmarks::write_u128(&mut x);
    let _write_u128_overhead_e = get_cycle();
    assert!(x == u128::MAX);

    Some(BenchmarkResult::MicroBenchmarks {
        get_cycle: _get_cycle_overhead_e - _get_cycle_overhead_s,
        empty_call: _empty_call_overhead_e - _empty_call_overhead_s,
        call_and_return: _call_and_return_overhead_e - _call_and_return_overhead_s,
        return_only: _return_only_overhead_e - _return_only_overhead_s,
        write_u32: _write_u32_overhead_e - _write_u32_overhead_s,
        write_u128: _write_u128_overhead_e - _write_u128_overhead_s,
    })
}

mod micro_benchmarks {
    #[inline]
    pub fn do_nothing() {}

    #[inline]
    pub fn return_argument(arg: u32) -> u32 {
        arg
    }

    #[inline]
    pub fn return_42() -> u32 {
        42
    }

    #[inline]
    pub fn write_42u32(arg: &mut u32) {
        *arg = 42;
    }

    #[inline]
    pub fn write_u128(arg: &mut u128) {
        *arg = u128::MAX;
    }
}
