use std::{fs, path::PathBuf, process, str::FromStr};

use analysis::Info;
use clap::Parser;

mod analysis;

#[derive(Parser)]
struct Cli {
    input_file_path: PathBuf,

    #[clap(short, long, value_name = "FILE")]
    output_file_path: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let content = fs::read_to_string(&cli.input_file_path).unwrap_or_else(|err| {
        eprintln!(
            "ERROR: Failed to read file `{}` ({err})",
            cli.input_file_path.as_path().display(),
        );
        process::exit(1);
    });

    let info: Info = serde_json::from_str(content.as_str()).unwrap_or_else(|err| {
        eprintln!("ERROR: Invalid JSON format ({err})");
        process::exit(1);
    });

    println!("Analyzing...");

    let analysis_result = info.analyze();

    println!("Finished analyzing!");

    let output_path = cli
        .output_file_path
        .unwrap_or_else(|| PathBuf::from_str("output.json").unwrap());

    fs::write(
        &output_path,
        serde_json::to_string_pretty(&analysis_result).unwrap(),
    )
    .unwrap_or_else(|err| {
        eprintln!(
            "ERROR: Failed to write to `{}` ({err})",
            output_path.to_string_lossy()
        );
        process::exit(1);
    });
}
