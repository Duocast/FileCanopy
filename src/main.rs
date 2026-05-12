use std::process::ExitCode;

use clap::Parser;

use filecanopy::cli::Cli;
use filecanopy::cli::commands;

fn main() -> ExitCode {
    filecanopy::telemetry::init();

    let cli = Cli::parse();

    match commands::dispatch(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!(error = ?err, "filecanopy failed");
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
