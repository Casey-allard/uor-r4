//! Exclusive report directories for experiment drivers.
//!
//! A report root is claimed once, with exclusive creation semantics, before any
//! model work; an existing path fails instead of being reused or overwritten.
//! A completed attempt is sealed with a manifest naming every file and its
//! BLAKE3 digest, so retries and interrupted attempts keep distinct directories,
//! logs and charges. This is a small executable guard, not an evidence framework.
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Sentinel written into a freshly claimed report directory.
pub const ATTEMPT_FILE: &str = "attempt.json";
/// Manifest written once when an attempt is sealed.
pub const MANIFEST_FILE: &str = "manifest.json";

/// Create `path` exclusively as a new report directory and write its attempt
/// sentinel. Parents are created as containers; the leaf must not exist. An
/// existing leaf, including an existing completed report, fails before any
/// caller work and is left byte-identical.
pub fn claim(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "report directory {} was not created exclusively ({e}); choose a new attempt path instead of reusing or overwriting an existing report",
                path.display()
            ),
        )
    })?;
    let argv: Vec<String> = std::env::args().collect();
    let sentinel = serde_json::json!({
        "schema": "uor-r4.report-attempt/1",
        "claimed_at": humantime_now(),
        "pid": std::process::id(),
        "argv": argv,
        "rule": "exclusively created before model work; never reused for a retry"
    });
    let mut file = fs::File::create_new(path.join(ATTEMPT_FILE))?;
    io::Write::write_all(
        &mut file,
        serde_json::to_string_pretty(&sentinel)?.as_bytes(),
    )?;
    Ok(())
}

/// Seal a completed attempt: write `manifest.json` (exclusively) listing every
/// regular file below `path` with its size and BLAKE3 digest. Sealing twice fails.
pub fn seal(path: &Path) -> io::Result<PathBuf> {
    let manifest_path = path.join(MANIFEST_FILE);
    if manifest_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} is already sealed", path.display()),
        ));
    }
    let mut files = Vec::new();
    collect(path, path, &mut files)?;
    files.sort();
    let mut entries = Vec::with_capacity(files.len());
    for relative in files {
        let bytes = fs::read(path.join(&relative))?;
        entries.push(serde_json::json!({
            "path": relative,
            "bytes": bytes.len(),
            "blake3": blake3::hash(&bytes).to_hex().to_string(),
        }));
    }
    let manifest = serde_json::json!({
        "schema": "uor-r4.report-manifest/1",
        "sealed_at": humantime_now(),
        "root": path,
        "files": entries,
    });
    let mut file = fs::File::create_new(&manifest_path)?;
    io::Write::write_all(
        &mut file,
        serde_json::to_string_pretty(&manifest)?.as_bytes(),
    )?;
    Ok(manifest_path)
}

/// Verify a sealed attempt: every listed file must exist with its recorded size and
/// BLAKE3 digest, and the directory must hold no unlisted regular file besides the
/// manifest. Returns the sorted list of unlisted files on success (empty) or an error
/// naming the first listed mismatch; unlisted files are reported as an error too, so
/// a complete file set is checked, not only the listed hashes.
pub fn verify(path: &Path) -> io::Result<Vec<String>> {
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(path.join(MANIFEST_FILE))?)?;
    let listed = manifest["files"]
        .as_array()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "manifest without files"))?;
    let mut names = std::collections::BTreeSet::new();
    for entry in listed {
        let relative = entry["path"].as_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "manifest entry without path")
        })?;
        let bytes = fs::read(path.join(relative))?;
        if entry["bytes"].as_u64() != Some(bytes.len() as u64)
            || entry["blake3"].as_str() != Some(blake3::hash(&bytes).to_hex().as_str())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("sealed file changed: {relative}"),
            ));
        }
        names.insert(relative.to_owned());
    }
    let mut actual = Vec::new();
    collect(path, path, &mut actual)?;
    let unlisted: Vec<String> = actual
        .into_iter()
        .filter(|f| f != MANIFEST_FILE && !names.contains(f))
        .collect();
    if !unlisted.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "sealed directory holds unlisted files: {}",
                unlisted.join(", ")
            ),
        ));
    }
    Ok(unlisted)
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() {
            collect(root, &path, out)?;
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            out.push(relative.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

fn humantime_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{now}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-report-output-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn report_output_new_path_is_claimed_and_sealed_normally() {
        let base = scratch("new");
        let out = base.join("candidate-a").join("evaluate").join("attempt-1");
        claim(&out).unwrap();
        assert!(out.join(ATTEMPT_FILE).is_file());
        fs::write(out.join("result.json"), b"{\"exact\":1}").unwrap();
        fs::create_dir(out.join("nested")).unwrap();
        fs::write(out.join("nested").join("rows.jsonl"), b"{}\n").unwrap();
        let manifest_path = seal(&out).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        let files = manifest["files"].as_array().unwrap();
        let names: Vec<_> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
        assert_eq!(
            names,
            vec![ATTEMPT_FILE, "nested/rows.jsonl", "result.json"]
        );
        assert_eq!(
            files[2]["blake3"],
            blake3::hash(b"{\"exact\":1}").to_hex().to_string()
        );
        assert!(seal(&out).is_err(), "sealing twice must fail");
        assert!(
            verify(&out).unwrap().is_empty(),
            "a complete sealed set verifies"
        );
        // A file added beneath a sealed directory is detected as unlisted.
        fs::write(out.join("derived-input.json"), b"[]").unwrap();
        let error = verify(&out).unwrap_err().to_string();
        assert!(
            error.contains("unlisted") && error.contains("derived-input.json"),
            "{error}"
        );
        fs::remove_file(out.join("derived-input.json")).unwrap();
        // A changed listed file is detected before the file set is judged.
        fs::write(out.join("result.json"), b"{\"exact\":0}").unwrap();
        assert!(verify(&out)
            .unwrap_err()
            .to_string()
            .contains("result.json"));
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn report_output_existing_report_fails_before_work_and_stays_byte_identical() {
        let base = scratch("collision");
        let out = base.join("evaluate").join("attempt-1");
        claim(&out).unwrap();
        let report = out.join("result.json");
        fs::write(&report, b"original rejected-candidate rows").unwrap();
        let sentinel_before = fs::read(out.join(ATTEMPT_FILE)).unwrap();
        let error = claim(&out).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert!(error.to_string().contains("new attempt path"));
        assert_eq!(
            fs::read(&report).unwrap(),
            b"original rejected-candidate rows"
        );
        assert_eq!(fs::read(out.join(ATTEMPT_FILE)).unwrap(), sentinel_before);
        // A plain pre-existing directory without a sentinel is also refused.
        let plain = base.join("plain");
        fs::create_dir(&plain).unwrap();
        fs::write(plain.join("result.json"), b"keep").unwrap();
        assert_eq!(
            claim(&plain).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(plain.join("result.json")).unwrap(), b"keep");
        // A distinct attempt path beside it completes normally.
        let next = base.join("evaluate").join("attempt-2");
        claim(&next).unwrap();
        assert!(next.join(ATTEMPT_FILE).is_file());
        fs::remove_dir_all(base).unwrap();
    }
}
