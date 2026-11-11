#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use uor_r4_studio_agent::schema_document;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("export-contract: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let first = args.next();
    match first.as_deref() {
        None | Some("--print") => {
            if first.is_some() && args.next().is_some() {
                return Err(usage());
            }
            print_schema()
        }
        Some("--write") => {
            let path = args.next().ok_or_else(usage)?;
            if args.next().is_some() {
                return Err(usage());
            }
            let document = schema_document();
            fs::write(Path::new(&path), format!("{document}\n"))
                .map_err(|error| format!("cannot write {path}: {error}"))
        }
        Some(other) => Err(format!("unknown argument {other:?}; {}", usage())),
    }
}

fn print_schema() -> Result<(), String> {
    let document = schema_document();
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{document}").map_err(|error| format!("cannot write stdout: {error}"))
}

fn usage() -> String {
    "usage: export-contract [--print | --write <path>]".to_string()
}
