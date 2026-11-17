#![forbid(unsafe_code)]

use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

const USAGE: &str = "Usage: uor-mvm status | uor-mvm collatz <1..512>";

fn run(args: &[OsString]) -> Result<(), String> {
    let limit = match args {
        [command] if command == "status" => None,
        [command, value] if command == "collatz" => {
            let limit = value
                .to_str()
                .and_then(|text| text.parse::<u16>().ok())
                .ok_or_else(|| "Collatz limit must be an integer from 1 to 512".to_string())?;
            Some(limit)
        }
        _ => return Err(USAGE.into()),
    };

    if let Some(limit) = limit {
        print!("{}", uor_r4_mvm::run_collatz(limit)?);
    } else {
        let (architecture, guest_cid) = uor_r4_mvm::guest_identity()?;
        println!("architecture={architecture} guest_cid={guest_cid}");
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(&env::args_os().skip(1).collect::<Vec<_>>()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
