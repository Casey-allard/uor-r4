use std::path::PathBuf;
use uor_r4_workbench::{comparison, host, worker, BoxError};

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Serve {
        configuration: PathBuf,
        configuration_sha256: String,
    },
    InternalWorker,
    PrivateCompareHost {
        release: PathBuf,
        release_sha256: String,
        admission: PathBuf,
        admission_sha256: String,
    },
    PrivateMetadata,
}

fn parse_mode(args: impl IntoIterator<Item = String>) -> Result<Mode, String> {
    let args: Vec<String> = args.into_iter().collect();
    match args.as_slice() {
        [mode] if mode == "--internal-worker" => Ok(Mode::InternalWorker),
        [mode] if mode == "--private-metadata" => Ok(Mode::PrivateMetadata),
        [mode, release, release_sha256, admission, admission_sha256]
            if mode == "--private-compare-host" =>
        {
            Ok(Mode::PrivateCompareHost {
                release: PathBuf::from(release),
                release_sha256: release_sha256.clone(),
                admission: PathBuf::from(admission),
                admission_sha256: admission_sha256.clone(),
            })
        }
        [config, path, digest] if config == "--config" => Ok(Mode::Serve {
            configuration: PathBuf::from(path),
            configuration_sha256: digest.clone(),
        }),
        _ => Err("usage: r4-workbench --config ABSOLUTE_PATH CONFIG_SHA256\n\
             private modes are reserved for accepted local releases"
            .to_owned()),
    }
}

fn run() -> Result<(), BoxError> {
    let mode = parse_mode(std::env::args().skip(1)).map_err(|e| -> BoxError { e.into() })?;
    match mode {
        Mode::Serve {
            configuration,
            configuration_sha256,
        } => host::serve(&configuration, &configuration_sha256),
        Mode::InternalWorker => worker::run(),
        Mode::PrivateCompareHost {
            release,
            release_sha256,
            admission,
            admission_sha256,
        } => comparison::run(&release, &release_sha256, &admission, &admission_sha256),
        Mode::PrivateMetadata => comparison::emit_private_metadata(),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("r4-workbench: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_modes_are_exact_and_take_no_incidental_arguments() {
        assert_eq!(
            parse_mode(["--internal-worker".to_owned()]),
            Ok(Mode::InternalWorker)
        );
        assert_eq!(
            parse_mode(["--private-metadata".to_owned()]),
            Ok(Mode::PrivateMetadata)
        );
        assert!(parse_mode(["--internal-worker".to_owned(), "extra".to_owned()]).is_err());
    }

    #[test]
    fn private_comparison_requires_both_trusted_identities() {
        let parsed = parse_mode([
            "--private-compare-host".to_owned(),
            "/release".to_owned(),
            "a".repeat(64),
            "/review".to_owned(),
            "b".repeat(64),
        ]);
        assert!(matches!(parsed, Ok(Mode::PrivateCompareHost { .. })));
        assert!(parse_mode([
            "--private-compare-host".to_owned(),
            "/release".to_owned(),
            "a".repeat(64),
        ])
        .is_err());
    }

    #[test]
    fn service_mode_requires_explicit_configuration_identity() {
        assert!(matches!(
            parse_mode([
                "--config".to_owned(),
                "/config.json".to_owned(),
                "c".repeat(64)
            ]),
            Ok(Mode::Serve { .. })
        ));
        assert!(parse_mode(["--config".to_owned(), "/config.json".to_owned()]).is_err());
    }
}
