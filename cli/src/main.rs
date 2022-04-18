use std::{ffi::OsString, path::PathBuf};

use clap::Parser;
mod lib;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    tty: OsString,

    #[clap(short, long)]
    raw: bool,

    #[clap(short, long)]
    files: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    for file in args.files {
        lib::benchmark_file(&args.tty, file);
    }

    if args.raw {
        lib::create_raw_console(&args.tty);
    }
}
