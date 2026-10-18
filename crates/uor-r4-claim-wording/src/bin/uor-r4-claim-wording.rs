//! `uor-r4-claim-wording [REPO_ROOT]`: exit 0 clean, 1 on a violation, 2 on an IO error.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    match uor_r4_claim_wording::check(&root) {
        Ok(violations) => {
            print!("{}", uor_r4_claim_wording::report(&violations));
            if violations.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("uor-r4-claim-wording: {e}");
            ExitCode::from(2)
        }
    }
}
