//! Wider-code-carrier ($B \in \{6, 8\}$) finite CUDA qualification binary (#2847).
//! Separate binary so the training driver's source identity is unchanged.
//! Usage: wider-carrier-qualify REPORT_DIR (requires `--features cuda` and a GPU)

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!("usage: wider-carrier-qualify REPORT_DIR");
        return std::process::ExitCode::from(2);
    }
    #[cfg(feature = "cuda")]
    {
        match uor_r4_discrete::wider_carrier::cuda_runner::qualify(std::path::Path::new(&args[0])) {
            Ok(_) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        eprintln!("wider-carrier-qualify requires --features cuda");
        std::process::ExitCode::FAILURE
    }
}
