use std::{
    ffi::OsString,
    io::{self, BufRead},
    path::PathBuf,
};

use tty::{RawTerminal, SerialConnection};

pub mod tty;

pub fn create_raw_console(tty: &OsString) {
    let mut suite = SerialConnection::new(tty).expect("Failed to connect to serial");

    let raw_term = RawTerminal::new(&mut suite, Box::new(io::stdin().lock().lines()));

    for line in raw_term {
        match line {
            Ok(reply) => println!("{reply}"),
            Err(err) => eprintln!("! {err}"),
        }
    }
}

pub fn benchmark_file(_tty: &OsString, _file: PathBuf) {
    todo!()
}
