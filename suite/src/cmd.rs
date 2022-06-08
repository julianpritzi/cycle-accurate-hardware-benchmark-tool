use alloc::{format, vec::Vec};
use benchmark_common::{BenchmarkResult, IncomingMessage, OutgoingMessage, SuiteStatus};

use crate::{
    benchmark::examples,
    platform::{self, Platform},
};

/// Takes an IncomingMessage and decides how to respond to it
///
/// # Arguments
///
/// * `cmd` - the message to produce a response to
pub fn run_cmd(cmd: IncomingMessage) -> Option<OutgoingMessage> {
    match cmd {
        IncomingMessage::Suspend(value) => platform::current().suspend(value),
        IncomingMessage::Invalid(msg) => {
            Some(OutgoingMessage::Error(format!("Invalid message: {msg}")))
        }
        IncomingMessage::Done => Some(OutgoingMessage::Status(SuiteStatus::Done)),
        IncomingMessage::GetStatus => Some(OutgoingMessage::Status(SuiteStatus::Ready)),
        IncomingMessage::Benchmark(info, n) => {
            let mut results: Vec<BenchmarkResult> = Vec::new();

            for _ in 0..n {
                let result = match info {
                    benchmark_common::BenchmarkInfo::ExampleSHA256 => examples::sha256_benchmark(),
                    benchmark_common::BenchmarkInfo::ExampleAES256 => examples::aes256_benchmark(),
                    benchmark_common::BenchmarkInfo::ExampleRNG => examples::rng_benchmark(),
                    benchmark_common::BenchmarkInfo::ExampleECDSA => examples::ecdsa_benchmark(),
                };

                if let Some(result) = result {
                    results.push(result)
                }
            }

            Some(OutgoingMessage::BenchmarkResults(results))
        }
    }
}
