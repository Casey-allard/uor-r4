//! Pure-integer D11 recurrence export and receipt verifier CLI (`#2890`).

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use recurrence_gates_six_recurrence_fit_instrument::{
    verify_d11_integer_export_exactness, verify_receipt_file,
};

fn main() -> ExitCode {
    let exactness = match verify_d11_integer_export_exactness() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("D11 integer export exactness failed: {e}");
            return ExitCode::from(1);
        }
    };
    if !exactness.passed {
        eprintln!("D11 integer export exactness did not pass");
        return ExitCode::from(1);
    }

    let mut args = std::env::args().skip(1);
    if let Some(arg) = args.next() {
        if arg == "--receipt" {
            let Some(p) = args.next() else {
                eprintln!("missing path after --receipt");
                return ExitCode::from(2);
            };
            match verify_receipt_file(&PathBuf::from(p)) {
                Ok(sha) => {
                    println!("verified receipt sha256={sha}");
                    return ExitCode::SUCCESS;
                }
                Err(e) => {
                    eprintln!("receipt verification failed: {e}");
                    return ExitCode::from(1);
                }
            }
        }
    }

    println!(
        "D11 integer export exactness verified: csd_entries={} q8_pairs={} max_lsb_diff={}",
        exactness.csd_keep_q12_exact_entries,
        exactness.shift_add_q8_exact_pairs,
        exactness.max_scan_lsb_diff
    );
    ExitCode::SUCCESS
}
