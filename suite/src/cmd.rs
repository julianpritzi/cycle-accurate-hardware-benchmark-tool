use alloc::format;
use benchmark_common::{IncomingMessage, OutgoingMessage, SuiteStatus};

use crate::{
    benchmark::{
        aes_benchmark_per_block, aes_benchmark_total, datasets, micro_benchmarks,
        rng_benchmark_total, sha2_benchmark_total, sha3_benchmark_total,
    },
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
        IncomingMessage::Benchmark(info) => {
            let result = match info {
                benchmark_common::BenchmarkInfo::AESDataSet(bench_type, id) => {
                    if id > datasets::aes::DATASETS.len() {
                        return Some(OutgoingMessage::Error(format!(
                            "No aes dataset with id {}",
                            id
                        )));
                    }
                    let dataset = &datasets::aes::DATASETS[id];

                    match bench_type {
                        benchmark_common::AESBenchmarkType::EncryptionPerBlock => {
                            aes_benchmark_per_block(dataset, crate::modules::AESOperation::Encrypt)
                        }
                        benchmark_common::AESBenchmarkType::DecryptionPerBlock => {
                            aes_benchmark_per_block(dataset, crate::modules::AESOperation::Decrypt)
                        }
                        benchmark_common::AESBenchmarkType::EncryptionTotal => {
                            aes_benchmark_total(dataset, crate::modules::AESOperation::Encrypt)
                        }
                        benchmark_common::AESBenchmarkType::DecryptionTotal => {
                            aes_benchmark_total(dataset, crate::modules::AESOperation::Decrypt)
                        }
                    }
                }
                benchmark_common::BenchmarkInfo::RNGDataSet(id) => {
                    if id > datasets::rng::DATASETS.len() {
                        return Some(OutgoingMessage::Error(format!(
                            "No rng dataset with id {}",
                            id
                        )));
                    }
                    let dataset = &datasets::rng::DATASETS[id];

                    rng_benchmark_total(dataset)
                }
                benchmark_common::BenchmarkInfo::HashDataSet(bench_type, id) => {
                    if id > datasets::hashing::DATASETS.len() {
                        return Some(OutgoingMessage::Error(format!(
                            "No rng dataset with id {}",
                            id
                        )));
                    }
                    let dataset = &datasets::hashing::DATASETS[id];

                    match bench_type {
                        benchmark_common::HashBenchmarkType::SHA2 => sha2_benchmark_total(dataset),
                        benchmark_common::HashBenchmarkType::SHA3 => sha3_benchmark_total(dataset),
                    }
                }
                benchmark_common::BenchmarkInfo::MicroBenchmarks => micro_benchmarks(),
            };

            Some(OutgoingMessage::BenchmarkResults(result))
        }
    }
}
