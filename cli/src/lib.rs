use std::{
    ffi::OsString,
    fmt::Write,
    io::{self, BufRead},
    path::PathBuf,
};

use tty::SerialTerminal;

mod tty;

pub fn create_raw_console(tty: &OsString) {
    let mut suite = SerialTerminal::new(tty).expect("Failed to connect to serial");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        writeln!(suite, "{}", line.unwrap()).expect("Writing to serial failed");

        match suite.read_line() {
            Ok(reply) => println!("{}", reply),
            Err(err) => {
                eprintln!("! {err}");
                break;
            }
        }
    }
}

pub fn benchmark_file(tty: &OsString, file: PathBuf) {
    todo!()
}
