use clap::Parser;
use std::{ffi::OsString, fs, path::PathBuf};

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

    /// Enables raw mode when processing files,
    /// each input line will be parsed as a message and sent directly to the suite.
    /// The result file will contain any response messages from the suite.
    #[clap(short, long)]
    verbose: bool,

    /// List of files, each representing a benchmark that should be performed.
    /// A .result file will be generated for each benchmark.
    #[clap(short, long, multiple_values = true)]
    files: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    for file in args.files {
        if args.raw {
            fs::write(
                file.with_extension("result"),
                cli::benchmark_raw_file(&args.tty, file, args.verbose),
            )
            .expect("Failed to write output file");
        } else {
            fs::write(
                file.with_extension("result"),
                cli::benchmark_raw_file(&args.tty, file, args.verbose),
            )
            .expect("Failed to write output file");
        }
    }
}
