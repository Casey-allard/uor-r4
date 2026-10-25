//! W2 read-code capacity capture on the imported Q4 device (read-only; no apply).
//! A separate binary so the training driver's source identity is unchanged.
//! Usage: w2-capacity CHECKPOINT SEALED_EVALUATION NEW_OUT_DIR
//! (requires `--features cuda` and a GPU; run from the root that holds `local/driver/data`)

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        eprintln!("usage: w2-capacity CHECKPOINT SEALED_EVALUATION NEW_OUT_DIR");
        return std::process::ExitCode::from(2);
    }
    #[cfg(feature = "cuda")]
    {
        use std::path::Path;
        match uor_r4_discrete::w2_capture::run(
            Path::new(&args[0]),
            Path::new(&args[1]),
            Path::new(&args[2]),
        ) {
            Ok(capture) => {
                eprintln!(
                    "w2-capacity: wrote {} ({} TRAIN rows, {} DEV rows)",
                    args[2],
                    capture.train_stream.rows,
                    capture.dev_rows.len()
                );
                std::process::ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        eprintln!("w2-capacity requires --features cuda; nothing ran");
        std::process::ExitCode::from(2)
    }
}
