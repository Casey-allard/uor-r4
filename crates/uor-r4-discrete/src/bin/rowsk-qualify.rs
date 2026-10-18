//! BVG-K Stage C: K-coordinate scoring parity (V2), block refusals (V6) and the
//! K latency curve. Opt-in path, zero apply. A separate binary, so the training
//! driver's source identity is unchanged.
//! Usage: rowsk-qualify REPORT_DIR   (requires `--features cuda` and a GPU)

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!("usage: rowsk-qualify REPORT_DIR");
        return std::process::ExitCode::from(2);
    }
    #[cfg(feature = "cuda")]
    {
        match uor_r4_discrete::rowsk_qualification::run(std::path::Path::new(&args[0])) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        eprintln!("rowsk-qualify requires --features cuda");
        std::process::ExitCode::FAILURE
    }
}
