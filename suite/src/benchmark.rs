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

    use crate::platform::{self, Platform};

    /// Runs an example benchmark for the SHA256 module
    ///
    /// # Arguments
    /// * `n` - the number of times the benchmark should be run
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
}
