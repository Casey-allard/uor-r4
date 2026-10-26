//! Wider-code-carrier ($B \in \{6, 8\}$) zero-training GPU evaluation and grading CLI (#2847).
//! Separate binary so the training driver's source identity is unchanged.
//!
//! Usage:
//!   wider-carrier-run arm LABEL CONFIG_JSON OUT_DIR    (requires `--features cuda` and a GPU)
//!   wider-carrier-run grade RUN_DIR QUAL_JSON OUT_JSON (host grading of stored GPU outputs)

use std::path::Path;
use std::process::ExitCode;

fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("arm") if args.len() == 4 => {
            #[cfg(feature = "cuda")]
            {
                uor_r4_discrete::wider_carrier::cuda_runner::evaluate_arm(
                    &args[1],
                    Path::new(&args[2]),
                    Path::new(&args[3]),
                )
                .map(|_| ())
                .map_err(|e| e.to_string())
            }
            #[cfg(not(feature = "cuda"))]
            {
                Err("wider-carrier-run arm requires --features cuda".into())
            }
        }
        Some("grade") if args.len() == 4 => {
            let out = uor_r4_discrete::wider_carrier::grade_wider_suite(
                Path::new(&args[1]),
                Path::new(&args[2]),
                Path::new(&args[3]),
            )
            .map_err(|e| e.to_string())?;
            println!("{out}");
            Ok(())
        }
        _ => Err(
            "usage:\n  wider-carrier-run arm LABEL CONFIG_JSON OUT_DIR\n  wider-carrier-run grade RUN_DIR QUAL_JSON OUT_JSON"
                .into(),
        ),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
