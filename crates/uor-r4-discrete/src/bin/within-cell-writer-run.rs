//! Within-cell writer qualification, Stage T error decomposition, and zero-update B0
//! evaluation and grading CLI (`m1-within-cell-writer`, `#3267`, referencing `#3128` and `#3129`).
//! Separate binary so `driver_source_sha256` is unchanged.
//!
//! Usage:
//!   within-cell-writer-run qualify OUT_JSON
//!   within-cell-writer-run decompose-stage-t STAGE_T_DIR DATA_ROOT OUT_JSON
//!   within-cell-writer-run b0 DATA_ROOT QUAL_JSON DECOMP_JSON OUT_JSON

#[cfg(feature = "cuda")]
use std::path::Path;
use std::process::ExitCode;

fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("qualify") if args.len() == 2 => {
            #[cfg(feature = "cuda")]
            {
                uor_r4_discrete::within_cell_writer::run_qualify(Path::new(&args[1]))
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
            #[cfg(not(feature = "cuda"))]
            {
                Err("within-cell-writer-run qualify requires --features cuda".into())
            }
        }
        Some("decompose-stage-t") if args.len() == 4 => {
            #[cfg(feature = "cuda")]
            {
                uor_r4_discrete::within_cell_writer::run_decompose_stage_t(
                    Path::new(&args[1]),
                    Path::new(&args[2]),
                    Path::new(&args[3]),
                )
                .map(|_| ())
                .map_err(|e| e.to_string())
            }
            #[cfg(not(feature = "cuda"))]
            {
                Err("within-cell-writer-run decompose-stage-t requires --features cuda".into())
            }
        }
        Some("b0") if args.len() == 5 => {
            #[cfg(feature = "cuda")]
            {
                uor_r4_discrete::within_cell_writer::run_b0(
                    Path::new(&args[1]),
                    Path::new(&args[2]),
                    Path::new(&args[3]),
                    Path::new(&args[4]),
                )
                .map(|_| ())
                .map_err(|e| e.to_string())
            }
            #[cfg(not(feature = "cuda"))]
            {
                Err("within-cell-writer-run b0 requires --features cuda".into())
            }
        }
        _ => Err(
            "usage:\n  within-cell-writer-run qualify OUT_JSON\n  within-cell-writer-run decompose-stage-t STAGE_T_DIR DATA_ROOT OUT_JSON\n  within-cell-writer-run b0 DATA_ROOT QUAL_JSON DECOMP_JSON OUT_JSON"
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
