use std::{ffi::OsString, path::PathBuf};

pub fn create_raw_console(tty: &OsString) -> ! {
    println!("raw console on {tty:?}");
    todo!()
}

pub fn benchmark_file(tty: &OsString, file: PathBuf) {
    todo!()
}
