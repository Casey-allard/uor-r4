use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};

use uor_r4_core::r4_group_addressed_retention::{R4GroupGeometryArtifactV1, R4_GROUP_MAX_TOKEN_ID};

const USAGE: &str =
    "usage: r4-group-geometry-export <OUTPUT.json> [--max-token-id 4095]\n\nBuild and atomically publish the canonical R4GroupAddressedRetentionLMV1 geometry artifact. OUTPUT.json is required; stdout export is not supported.";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Help,
    Export { output: PathBuf, max_token_id: u16 },
}

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("r4-group-geometry-export: {error}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn run<I>(arguments: I) -> Result<(), String>
where
    I: IntoIterator<Item = OsString>,
{
    match parse_arguments(arguments)? {
        Command::Help => {
            println!("{USAGE}");
            Ok(())
        }
        Command::Export {
            output,
            max_token_id,
        } => {
            let artifact = R4GroupGeometryArtifactV1::build(max_token_id)
                .map_err(|error| error.to_string())?;
            let bytes = artifact
                .canonical_bytes()
                .map_err(|error| error.to_string())?;
            let verified = R4GroupGeometryArtifactV1::from_canonical_bytes(&bytes)
                .map_err(|error| format!("canonical round-trip failed: {error}"))?;
            if verified != artifact {
                return Err("canonical round-trip changed the geometry artifact".to_owned());
            }
            write_atomically(&output, &bytes)
                .map_err(|error| format!("cannot publish {}: {error}", output.display()))?;
            println!(
                "wrote {} bytes to {} ({})",
                bytes.len(),
                output.display(),
                artifact.artifact_cid
            );
            Ok(())
        }
    }
}

fn parse_arguments<I>(arguments: I) -> Result<Command, String>
where
    I: IntoIterator<Item = OsString>,
{
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    if arguments
        .iter()
        .any(|argument| argument == "--help" || argument == "-h")
    {
        if arguments.len() == 1 {
            return Ok(Command::Help);
        }
        return Err("--help cannot be combined with export arguments".to_owned());
    }

    let mut output = None;
    let mut max_token_id = R4_GROUP_MAX_TOKEN_ID;
    let mut max_token_seen = false;
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        if argument == "--max-token-id" {
            if max_token_seen {
                return Err("--max-token-id may be provided only once".to_owned());
            }
            let raw = arguments
                .get(index + 1)
                .ok_or_else(|| "--max-token-id requires a value".to_owned())?;
            let raw = raw
                .to_str()
                .ok_or_else(|| "--max-token-id must be valid UTF-8".to_owned())?;
            max_token_id = raw
                .parse::<u16>()
                .map_err(|_| format!("invalid --max-token-id value {raw:?}"))?;
            max_token_seen = true;
            index += 2;
            continue;
        }
        if argument.to_string_lossy().starts_with('-') {
            return Err(format!("unknown option {:?}", argument.to_string_lossy()));
        }
        if output.replace(PathBuf::from(argument)).is_some() {
            return Err("exactly one output path is required".to_owned());
        }
        index += 1;
    }
    let output = output.ok_or_else(|| "an explicit output path is required".to_owned())?;
    if output.as_os_str().is_empty() || output == Path::new("-") {
        return Err("output must be an explicit filesystem path".to_owned());
    }
    if max_token_id != R4_GROUP_MAX_TOKEN_ID {
        return Err(format!(
            "the frozen contract requires --max-token-id {R4_GROUP_MAX_TOKEN_ID}"
        ));
    }
    Ok(Command::Export {
        output,
        max_token_id,
    })
}

fn write_atomically(destination: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = destination.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "output path has no file name",
        )
    })?;
    match std::fs::symlink_metadata(destination) {
        Ok(metadata) if !metadata.file_type().is_file() => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "output exists and is not a regular file",
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let parent_metadata = std::fs::symlink_metadata(parent)?;
    if !parent_metadata.file_type().is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "output parent is not a directory",
        ));
    }

    let (temporary, mut file) = loop {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(
            ".{}.r4-group-geometry.{}.{}.tmp",
            file_name.to_string_lossy(),
            std::process::id(),
            sequence
        ));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => break (temporary, file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };

    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    drop(file);
    if let Err(error) = std::fs::rename(&temporary, destination) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    std::fs::File::open(parent)?.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn help_and_frozen_export_usage_parse() {
        assert_eq!(parse_arguments(args(&["--help"])).unwrap(), Command::Help);
        assert_eq!(
            parse_arguments(args(&["geometry.json"])).unwrap(),
            Command::Export {
                output: PathBuf::from("geometry.json"),
                max_token_id: 4095,
            }
        );
        assert_eq!(
            parse_arguments(args(&["--max-token-id", "4095", "geometry.json"])).unwrap(),
            Command::Export {
                output: PathBuf::from("geometry.json"),
                max_token_id: 4095,
            }
        );
    }

    #[test]
    fn missing_path_and_nonfrozen_bound_fail_closed() {
        assert!(parse_arguments(args(&[])).is_err());
        assert!(parse_arguments(args(&["-"])).is_err());
        assert!(parse_arguments(args(&["geometry.json", "--max-token-id", "4094"])).is_err());
    }
}
