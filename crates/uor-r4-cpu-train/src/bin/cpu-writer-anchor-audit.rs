use std::path::PathBuf;

use uor_r4_cpu_train::CpuFbsForward;
use uor_r4_discrete::corpus::sha256;

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-writer-anchor-audit: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let (artifact, expected_sha256, output) = parse_args()?;
    let bytes =
        std::fs::read(&artifact).map_err(|error| format!("{}: {error}", artifact.display()))?;
    reject_output_aliasing_artifact(&artifact, output.as_deref())?;
    let actual_sha256 = sha256(&bytes);
    verify_expected_sha256(&actual_sha256, expected_sha256.as_deref())?;
    let report =
        CpuFbsForward::audit_writer_port_anchors(&bytes).map_err(|error| error.to_string())?;
    let json = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    if let Some(output) = output {
        std::fs::write(&output, &json).map_err(|error| format!("{}: {error}", output.display()))?;
    } else {
        println!(
            "{}",
            String::from_utf8(json).map_err(|error| error.to_string())?
        );
    }
    if !report.passes() {
        return Err(format!(
            "anchor mismatch in {} port coordinates across {} tokens",
            report.mismatch_count, report.mismatch_tokens
        ));
    }
    Ok(())
}

fn verify_expected_sha256(actual: &str, expected: Option<&str>) -> Result<(), String> {
    if let Some(expected) = expected {
        if actual != expected {
            return Err(format!(
                "artifact SHA-256 differs: expected {expected}, got {actual}"
            ));
        }
    }
    Ok(())
}

fn reject_output_aliasing_artifact(
    artifact: &std::path::Path,
    output: Option<&std::path::Path>,
) -> Result<(), String> {
    let Some(output) = output else {
        return Ok(());
    };
    let artifact = std::fs::canonicalize(artifact)
        .map_err(|error| format!("{}: {error}", artifact.display()))?;
    let output = match std::fs::canonicalize(output) {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{}: {error}", output.display())),
    };
    if artifact == output {
        return Err("--output must not overwrite --artifact".to_owned());
    }
    Ok(())
}

fn parse_args() -> Result<(PathBuf, Option<String>, Option<PathBuf>), String> {
    let mut artifact = None;
    let mut expected_sha256 = None;
    let mut output = None;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--artifact" => artifact = Some(path(&mut args, "--artifact")?),
            "--expected-artifact-sha256" => {
                expected_sha256 = Some(parse_expected_sha256(value(
                    &mut args,
                    "--expected-artifact-sha256",
                )?)?);
            }
            "--output" => output = Some(path(&mut args, "--output")?),
            "--help" | "-h" => return Err(usage()),
            _ => return Err(format!("unknown argument {argument}\n{}", usage())),
        }
    }
    Ok((artifact.ok_or_else(usage)?, expected_sha256, output))
}

fn parse_expected_sha256(value: String) -> Result<String, String> {
    let value = value.to_ascii_lowercase();
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("--expected-artifact-sha256 needs 64 hexadecimal digits".to_owned());
    }
    Ok(value)
}

fn value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{flag} needs a value\n{}", usage()))
}

fn path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(value(args, flag)?))
}

fn usage() -> String {
    "usage: cpu-writer-anchor-audit --artifact PATH \
     [--expected-artifact-sha256 HEX] [--output PATH]"
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_names_the_audit_inputs() {
        let usage = usage();
        for value in ["--artifact", "--expected-artifact-sha256", "--output"] {
            assert!(usage.contains(value));
        }
    }

    #[test]
    fn expected_sha_requires_64_hexadecimal_digits_and_normalizes_case() {
        assert_eq!(
            parse_expected_sha256("AB".repeat(32)).unwrap(),
            "ab".repeat(32)
        );
        assert!(parse_expected_sha256("ab".repeat(31)).is_err());
        assert!(parse_expected_sha256("ab".repeat(33)).is_err());
        assert!(parse_expected_sha256("gg".repeat(32)).is_err());
    }

    #[test]
    fn expected_sha_mismatch_fails_closed() {
        let error = verify_expected_sha256(&"ab".repeat(32), Some(&"cd".repeat(32))).unwrap_err();
        assert!(error.contains("artifact SHA-256 differs"));
    }

    #[test]
    fn output_cannot_alias_the_artifact() {
        let root = std::env::temp_dir().join(format!(
            "uor-writer-anchor-audit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let artifact = root.join("artifact.uorlut");
        std::fs::write(&artifact, b"artifact").unwrap();
        assert!(reject_output_aliasing_artifact(&artifact, Some(&artifact)).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
