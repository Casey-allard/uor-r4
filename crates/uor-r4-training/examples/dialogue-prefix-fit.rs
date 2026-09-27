//! Bounded offline native dialogue learning, reload and actual output.
use std::{io, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let prepare = args
        .first()
        .is_some_and(|p| p == std::path::Path::new("--prepare"));
    if prepare {
        args.remove(0);
    }
    if args.len() != 2 {
        return Err(io::Error::other(
            "usage: dialogue-prefix-fit [--prepare] CAMPAIGN NEW_REPORT_ROOT",
        )
        .into());
    }
    let worker_args = args.clone();
    let worker = std::thread::Builder::new()
        .name("dialogue-prefix-training".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let run = if prepare {
                uor_r4_training::dialogue_learning::prepare
            } else {
                uor_r4_training::dialogue_learning::run
            };
            run(
                &worker_args[0],
                &worker_args[1],
                option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or(""),
            )
        })?;
    let result = worker
        .join()
        .map_err(|_| io::Error::other("dialogue training worker failed"))??;
    println!(
        "{}",
        serde_json::json!({"status":result["status"],"completed_step":result["completed_step"],"report_root":args[1]})
    );
    Ok(())
}
