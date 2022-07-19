//! This crate documents the messages that are used in the communication between the
//! Suite and the CLI.
//!
//! It also provides functions for (de)serializing messages.
//! The current implementation for (de)serialization uses serde_json, which may be changed
//! in the future.

#![no_std]

extern crate alloc;
#[allow(unused_imports)]
use alloc::string::String;

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Messages sent from the CLI to the Suite
#[derive(Debug, Serialize, Deserialize)]
pub enum _CliToSuiteMessage {
    /// Requests the current status of the Suite
    GetStatus,
    /// Signals to the Suite that the CLI is done sending requests,
    /// and that the Suite should respond with a status update,
    /// once it completed all requests
    Done,
    /// Requests the Suite to suspend with the given code
    Suspend(u32),
    /// Requests the Suite to perform a benchmark and return the result
    Benchmark(BenchmarkInfo),
    /// Represents an Invalid message, it should not be sent intentionally,
    /// rather it is returned when an invalid message is deserialized
    ///
    /// The first parameter is the unparsed message
    Invalid(String),
}

/// Represents all the information necessary to perform a benchmark
#[derive(Debug, Serialize, Deserialize)]
pub enum BenchmarkInfo {
    /// Perform an aes benchmark of the specified type using the dataset with the provided id
    AESDataSet(AESBenchmarkType, usize),
    /// Perform an rng benchmark using the dataset with the provided id
    RNGDataSet(usize),
    /// Perform an rng benchmark using true number generation, with provided number of data blocks
    RNGTrueRandom(usize),
    /// Perform a hashing benchmark of the specified type using the dataset with the provided id
    HashDataSet(HashBenchmarkType, usize),
    /// Perform a set of microbenchmarks
    MicroBenchmarks,
}

/// Represents all the possible types of benchmarks for the AES module
#[derive(Debug, Serialize, Deserialize)]
pub enum AESBenchmarkType {
    // Perform a per block encryption benchmark, if true use tighter waiting for more accurate results.
    EncryptionPerBlock(bool),
    // Perform a per block decryption benchmark, if true use tighter waiting for more accurate results.
    DecryptionPerBlock(bool),
    EncryptionTotal,
    DecryptionTotal,
}

/// Represents all the possible types of benchmarks for the hashing module
#[derive(Debug, Serialize, Deserialize)]
pub enum HashBenchmarkType {
    SHA2,
    SHA3,
}

/// Messages sent from the Suite to the CLI
#[derive(Debug, Serialize, Deserialize)]
pub enum _SuiteToCliMessage {
    /// Signals the current status of the Suite
    Status(SuiteStatus),
    /// Notifies the CLI that an error occurred on the Suite
    Error(String),
    /// Returns a benchmarking result if the suite was able to perform the benchmark
    BenchmarkResults(Option<BenchmarkResult>),
    /// Represents an Invalid message, it should not be sent intentionally,
    /// rather it is returned when an invalid message is deserialized
    ///
    /// The first parameter is the unparsed message
    Invalid(String),
}

/// Represents all the results of a single benchmark
#[derive(Debug, Serialize, Deserialize)]
pub enum BenchmarkResult {
    SHA2Total {
        initialization: u64,
        computation: u64,
        reading_output: u64,
    },
    SHA3Total {
        initialization: u64,
        computation: u64,
        reading_output: u64,
    },
    RNGTotalSeeded {
        seeded_initialization: u64,
        seeded_generation: Vec<u64>,
        seeded_wait_initialization: u64,
        seeded_wait_generation: Vec<u64>,
    },
    RNGTotalTrueRandom {
        unseeded_initialization: u64,
        unseeded_generation: Vec<u64>,
        unseeded_wait_initialization: u64,
        unseeded_wait_generation: Vec<u64>,
    },
    ECDSATotal {
        signing: u64,
        verifying: u64,
    },
    AESPerBlock {
        initialization: u64,
        blocks: Vec<AesBlockResult>,
        deinitalization: u64,
    },
    AESTotal {
        initialization: u64,
        computation: u64,
        deinitalization: u64,
    },
    MicroBenchmarks {
        get_cycle: u64,
        empty_call: u64,
        call_and_return: u64,
        return_only: u64,
        write_u32: u64,
        write_u128: u64,
    },
}

/// Represents the benchmarked time of a single block in aes
#[derive(Debug, Serialize, Deserialize)]
pub struct AesBlockResult {
    pub write_input: u64,
    pub computation: u64,
    pub read_output: u64,
}

/// Represents the status of the Suite
#[derive(Debug, Serialize, Deserialize)]
pub enum SuiteStatus {
    Ready,
    Done,
}

/// Alias for messages sent from the CLI to the Suite, when building the CLI
#[cfg(feature = "cli")]
pub type OutgoingMessage = _CliToSuiteMessage;
/// Alias for messages sent from the Suite to the CLI, when building the CLI
#[cfg(feature = "cli")]
pub type IncomingMessage = _SuiteToCliMessage;

/// Alias for messages sent from the Suite to the CLI, when building the Suite
#[cfg(feature = "suite")]
pub type OutgoingMessage = _SuiteToCliMessage;
/// Alias for messages sent from the CLI to the Suite, when building the Suite
#[cfg(feature = "suite")]
pub type IncomingMessage = _CliToSuiteMessage;

/// Serializes an outgoing message to a String that can be exchanged
///
/// # Arguments
///
/// * `value` - the message that should be serialized
#[cfg(any(feature = "cli", feature = "suite"))]
pub fn serialize(value: &OutgoingMessage) -> String {
    serde_json::to_string(&value).expect("Can not serialize struct")
}

/// Deserializes a String to an incoming message
///
/// # Arguments
///
/// * `value` - the String that should be deserialized
#[cfg(any(feature = "cli", feature = "suite"))]
pub fn deserialize(value: String) -> IncomingMessage {
    if let Ok(value) = serde_json::from_str(&value) {
        value
    } else {
        IncomingMessage::Invalid(value)
    }
}

/// Parses a String from raw benchmarking files to a CliToSuiteMessage
///
/// # Arguments
///
/// * `value` - the String that should be parsed
#[cfg(feature = "cli")]
pub fn parse_raw(value: &str) -> _CliToSuiteMessage {
    use crate::alloc::string::ToString;

    if let Ok(value) = serde_json::from_str(&value) {
        value
    } else {
        _CliToSuiteMessage::Invalid(value.to_string())
    }
}
