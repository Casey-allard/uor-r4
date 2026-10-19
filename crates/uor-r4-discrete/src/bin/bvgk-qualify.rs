//! BVG-K Stage C2 block path: finite device-level controls (V0, RC, UT, JR,
//! DC with V4/V5, V3, RF, C1R) and the TP diagnostic. Opt-in path; its applies
//! touch fixture devices only. A separate binary, so the training driver's
//! source identity is unchanged.
//! Usage: bvgk-qualify REPORT_DIR   (requires `--features cuda` and a GPU)

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!("usage: bvgk-qualify REPORT_DIR");
        return std::process::ExitCode::from(2);
    }
    #[cfg(feature = "cuda")]
    {
        match uor_r4_discrete::bvgk_qualification::run(std::path::Path::new(&args[0])) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        eprintln!("bvgk-qualify requires --features cuda");
        std::process::ExitCode::FAILURE
    }
}
