//! The `ix-trace-run` command line.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let options = match ix_trace_run::run::Options::parse(&args) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("ix-trace-run: {err}");
            return ExitCode::from(2);
        }
    };
    match ix_trace_run::run::run(&options) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(err) => {
            eprintln!("ix-trace-run: {err}");
            ExitCode::from(2)
        }
    }
}
