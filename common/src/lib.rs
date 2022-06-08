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
    /// Requests the Suite to perform a benchmark, n times and return the result
    Benchmark(BenchmarkInfo, u8),
    /// Represents an Invalid message, it should not be sent intentionally,
    /// rather it is returned when an invalid message is deserialized
    ///
    /// The first parameter is the unparsed message
    Invalid(String),
}

/// Represents all the information necessary to perform a benchmark
#[derive(Debug, Serialize, Deserialize)]
pub enum BenchmarkInfo {
    /// Run the example SHA256 benchmark
    ExampleSHA256,
    ExampleAES256,
    ExampleRNG,
    ExampleECDSA,
}

/// Messages sent from the Suite to the CLI
#[derive(Debug, Serialize, Deserialize)]
pub enum _SuiteToCliMessage {
    /// Signals the current status of the Suite
    Status(SuiteStatus),
    /// Notifies the CLI that an error occurred on the Suite
    Error(String),
    /// Requests the Suite to perform a benchmark, n times and return the result
    BenchmarkResults(Vec<BenchmarkResult>),
    /// Represents an Invalid message, it should not be sent intentionally,
    /// rather it is returned when an invalid message is deserialized
    ///
    /// The first parameter is the unparsed message
    Invalid(String),
}

/// Represents all the results of a single benchmark
#[derive(Debug, Serialize, Deserialize)]
pub enum BenchmarkResult {
    ExampleSHA256 {
        initialization: u64,
        computation: u64,
        reading_output: u64,
    },
    ExampleAES256 {
        enc_initialization: u64,
        enc_computation: u64,
        enc_deinitalization: u64,
        dec_initialization: u64,
        dec_computation: u64,
        dec_deinitalization: u64,
    },
    ExampleRNG {
        initialization: u64,
        generation: u64,
    },
    ExampleECDSA {
        signing: u64,
        verifying: u64,
    },
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
