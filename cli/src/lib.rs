pub mod tty;

use benchmark_common::{parse_raw, IncomingMessage, OutgoingMessage, SuiteStatus};
use std::{ffi::OsString, fs, path::PathBuf};
use tty::{SerialConnection, SuiteConnection};

/// Benchmark the suite using the file provided, interpreted in raw mode.
///
/// Raw mode means that the files lines are parsed line by line, each representing a
/// message that the CLI sends directly to the Suite.
/// Implicitly sends OutgoingMessage::Done at the end.
///
/// For information about supported messages an their (de)serialization check the common crate.
///
/// # Arguments
///
/// * `tty` - path to the tty used to communicate with the suite
/// * `input_file` - path to the file containing the messages that should be sent
pub fn benchmark_raw_file(tty: &OsString, input_file: PathBuf, verbose: bool) -> String {
    let mut suite = SuiteConnection::new(
        SerialConnection::new(tty).expect("Failed to connect to serial"),
        verbose,
    )
    .expect("Failed to establish valid connection with suite");

    let input_msg = fs::read_to_string(&input_file).expect("Failed to read input file");
    let mut output_msg = String::new();
    for (line_num, line) in input_msg.lines().enumerate() {
        let line = line.trim();

        if line.starts_with('#') || line.is_empty() {
            output_msg.push_str(&format!("{line}\n"));
            continue;
        }

        let msg = parse_raw(line);

        if matches!(
            msg,
            OutgoingMessage::Invalid(_) | OutgoingMessage::Done | OutgoingMessage::GetStatus
        ) {
            panic!(
                "Input file contains invalid messages on line {}",
                line_num + 1
            )
        }

        suite.send_message(&msg);
        match suite.read_message() {
            Ok(msg) => {
                output_msg.push_str(&format!("{msg:#?}\n"));
            }
            Err(_) => {
                output_msg.push_str(&format!("-- Connection lost --\n"));
                return output_msg;
            }
        }
    }

    loop {
        suite.send_message(&OutgoingMessage::Done);
        match suite.read_message() {
            Ok(msg) => {
                if matches!(msg, IncomingMessage::Status(SuiteStatus::Done)) {
                    return output_msg;
                }
                output_msg.push_str(&format!("{msg:#?}\n"));
            }
            Err(_) => {
                output_msg.push_str(&format!("-- Connection lost --\n"));
                return output_msg;
            }
        }
    }
}

/// Benchmark the suite using the file provided.
///
/// The CLI will read the description of the benchmark from the file and
/// determine the messages that should be sent to the suite in order to
/// perform said benchmark.
///
/// # Arguments
///
/// * `_tty` - path to the tty used to communicate with the suite
/// * `_input_file` - path to the file containing a description of the benchmark that should be performed
pub fn benchmark_file(_tty: &OsString, _input_file: PathBuf) {
    // TODO: implement normal benchmarking function, including better output
    todo!()
}
