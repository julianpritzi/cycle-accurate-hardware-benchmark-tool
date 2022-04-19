use clap::Parser;
use std::{ffi::OsString, path::PathBuf};

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    tty: OsString,

    #[clap(short, long)]
    raw: bool,

    #[clap(short, long, multiple_values = true)]
    files: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    for file in args.files {
        cli::benchmark_file(&args.tty, file);
    }

    if args.raw {
        cli::create_raw_console(&args.tty);
    }
}
