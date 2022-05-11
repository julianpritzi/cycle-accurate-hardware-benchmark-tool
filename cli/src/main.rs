use clap::Parser;
use std::{ffi::OsString, path::PathBuf};

#[derive(Parser)]
struct Args {
    /// A valid path to the tty that should be used to communicate with the suite.
    #[clap(short, long)]
    tty: OsString,

    /// Enables raw mode when processing files,
    /// each input line will be parsed as a message and sent directly to the suite.
    /// The result file will contain any response messages from the suite.
    #[clap(short, long)]
    raw: bool,

    /// List of files, each representing a benchmark that should be performed.
    /// A .result file will be generated for each benchmark.
    #[clap(short, long, multiple_values = true)]
    files: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    for file in args.files {
        if args.raw {
            cli::benchmark_raw_file(&args.tty, file);
        } else {
            cli::benchmark_file(&args.tty, file);
        }
    }
}
