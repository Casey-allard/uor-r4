//! Row-parallel counterfactual parity qualification (opt-in path; zero apply).
//! A separate binary so the training driver's source identity is unchanged.
//! Usage: rows-qualify REPORT_DIR   (requires `--features cuda` and a GPU)

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!("usage: rows-qualify REPORT_DIR");
        return std::process::ExitCode::from(2);
    }
    #[cfg(feature = "cuda")]
    {
        match uor_r4_discrete::rows_qualification::run(std::path::Path::new(&args[0])) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        eprintln!("rows-qualify requires the cuda feature; nothing ran");
        std::process::ExitCode::from(2)
    }
}
