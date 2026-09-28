//! Explicit offline response-aware code-choice recipe; no built-in fit dose.
use std::{
    io,
    path::{Path, PathBuf},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let normalize = args.first().is_some_and(|p| p == Path::new("--normalize"));
    let evaluate = args.first().is_some_and(|p| p == Path::new("--evaluate"));
    if normalize || evaluate {
        args.remove(0);
    }
    if args.len() != if evaluate { 3 } else { 2 } {
        return Err(io::Error::other(
            "usage: dialogue-child-rounding [--normalize] CAMPAIGN NEW_REPORT_ROOT | --evaluate CAMPAIGN PACKED_ROOT NEW_REPORT_ROOT",
        )
        .into());
    }
    let worker_args = args.clone();
    let worker = std::thread::Builder::new()
        .name("dialogue-code-choices".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            if evaluate {
                uor_r4_training::dialogue_rounding::evaluate(
                    &worker_args[0],
                    &worker_args[1],
                    &worker_args[2],
                    option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or(""),
                )
            } else {
                uor_r4_training::dialogue_rounding::run(
                    &worker_args[0],
                    &worker_args[1],
                    option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or(""),
                    normalize,
                )
            }
        })?;
    let result = worker
        .join()
        .map_err(|_| io::Error::other("dialogue code-choice worker failed"))??;
    println!(
        "{}",
        serde_json::json!({"status":result["status"],"completed_updates":result["completed_updates"],"report_root":args[if evaluate { 2 } else { 1 }]})
    );
    Ok(())
}
