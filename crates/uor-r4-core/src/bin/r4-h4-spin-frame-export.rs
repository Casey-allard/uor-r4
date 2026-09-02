use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};

use uor_r4_core::h4_spin_frame_sidecar::H4SpinFrameSidecarV1;

const USAGE: &str = "usage: r4-h4-spin-frame-export <OUTPUT.json>\n\nBuild and atomically publish the canonical H4 spin-frame sidecar. OUTPUT.json is required; stdout export is not supported.";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("r4-h4-spin-frame-export: {error}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn run<I>(arguments: I) -> Result<(), String>
where
    I: IntoIterator<Item = OsString>,
{
    let output = parse_arguments(arguments)?;
    let Some(output) = output else {
        println!("{USAGE}");
        return Ok(());
    };
    let artifact = H4SpinFrameSidecarV1::build().map_err(|error| error.to_string())?;
    let bytes = artifact
        .canonical_bytes()
        .map_err(|error| error.to_string())?;
    H4SpinFrameSidecarV1::from_canonical_bytes(&bytes)
        .map_err(|error| format!("canonical round trip failed: {error}"))?;
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

fn parse_arguments<I>(arguments: I) -> Result<Option<PathBuf>, String>
where
    I: IntoIterator<Item = OsString>,
{
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    if arguments.len() == 1 && (arguments[0] == "--help" || arguments[0] == "-h") {
        return Ok(None);
    }
    if arguments.len() != 1 {
        return Err("exactly one output path is required".to_owned());
    }
    let output = PathBuf::from(&arguments[0]);
    if output.as_os_str().is_empty() || output == Path::new("-") {
        return Err("output must be an explicit filesystem path".to_owned());
    }
    Ok(Some(output))
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
    if let Ok(metadata) = std::fs::symlink_metadata(destination) {
        if !metadata.file_type().is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "output exists and is not a regular file",
            ));
        }
    }
    // Follow parent-directory symlinks (for example macOS `/tmp` ->
    // `/private/tmp`) while retaining the final-component symlink rejection
    // above for the destination itself.
    if !std::fs::metadata(parent)?.file_type().is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "output parent is not a directory",
        ));
    }

    let (temporary, mut file) = loop {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(
            ".{}.h4-spin-frame.{}.{}.tmp",
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

    #[cfg(unix)]
    fn unique_test_directory() -> PathBuf {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "uor-r4-h4-spin-frame-export-test-{}-{sequence}",
            std::process::id()
        ))
    }

    #[test]
    fn explicit_output_or_help_is_required() {
        assert_eq!(parse_arguments([OsString::from("--help")]).unwrap(), None);
        assert_eq!(
            parse_arguments([OsString::from("frames.json")]).unwrap(),
            Some(PathBuf::from("frames.json"))
        );
        assert!(parse_arguments(Vec::<OsString>::new()).is_err());
        assert!(parse_arguments([OsString::from("-")]).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn follows_parent_symlink_but_rejects_destination_symlink() {
        use std::os::unix::fs::symlink;

        let root = unique_test_directory();
        let actual_parent = root.join("actual-parent");
        let linked_parent = root.join("linked-parent");
        std::fs::create_dir_all(&actual_parent).unwrap();
        symlink(&actual_parent, &linked_parent).unwrap();

        let through_parent_link = linked_parent.join("frames.json");
        write_atomically(&through_parent_link, b"canonical frames").unwrap();
        assert_eq!(
            std::fs::read(actual_parent.join("frames.json")).unwrap(),
            b"canonical frames"
        );

        let actual_destination = actual_parent.join("existing.json");
        std::fs::write(&actual_destination, b"keep me").unwrap();
        let linked_destination = actual_parent.join("linked-output.json");
        symlink(&actual_destination, &linked_destination).unwrap();
        let error = write_atomically(&linked_destination, b"replacement").unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert_eq!(std::fs::read(actual_destination).unwrap(), b"keep me");

        std::fs::remove_dir_all(root).unwrap();
    }
}
