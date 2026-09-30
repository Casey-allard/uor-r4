//! Job spec schema `uor-r4.lab-runner-job/1` and its validation.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::{invalid, Result};

pub const JOB_SCHEMA: &str = "uor-r4.lab-runner-job/1";
pub const STOP_ENFORCEMENT_MARGIN_MS: u64 = 2_000;
fn default_stop_grace_ms() -> u64 {
    250
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KillKind {
    /// Kill once `wall_s` has elapsed.
    Wall,
    /// Additionally kill when `value` (a literal substring) appears in the
    /// job's stdout or stderr log.
    LogMatch,
    /// Additionally run `value` as a shell check command (`sh -c value`);
    /// a nonzero exit status kills the job.
    Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KillCriterion {
    pub kind: KillKind,
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorageReservation {
    pub volume: String,
    pub additional_bytes: u64,
    pub checkpoint_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinationClaim {
    pub issue: u64,
    pub session: String,
    pub epoch: u64,
    pub work_card: String,
    pub attempt_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileIdentity {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobProvenance {
    pub source_sha: String,
    pub executable: FileIdentity,
    /// Every data/configuration/model input, or an independently verified
    /// sealed input manifest, belongs here. An inert pilot may use no inputs.
    pub inputs: Vec<FileIdentity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobSpec {
    #[serde(default)]
    pub schema: Option<String>,
    pub id: String,
    pub lab: String,
    pub cwd: PathBuf,
    pub argv: Vec<String>,
    #[serde(default)]
    pub env: Option<BTreeMap<String, String>>,
    pub threads: u32,
    pub rss_gib: f64,
    #[serde(default)]
    pub gpu: bool,
    pub wall_s: u64,
    /// Grace after verified TERM before KILL; this permits payload-owned
    /// checkpoint handlers, but does not certify a saved checkpoint.
    #[serde(default = "default_stop_grace_ms")]
    pub stop_grace_ms: u64,
    pub kill_criterion: KillCriterion,
    #[serde(default)]
    pub exclusive: bool,
    #[serde(default)]
    pub cargo: bool,
    #[serde(default)]
    pub storage: Vec<StorageReservation>,
    #[serde(default)]
    pub coordination: Option<CoordinationClaim>,
    #[serde(default)]
    pub provenance: Option<JobProvenance>,
}

fn hex_digest(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

impl FileIdentity {
    pub fn verify(&self) -> Result<()> {
        self.verify_until(None)
    }

    fn verify_until(&self, deadline: Option<std::time::Instant>) -> Result<()> {
        use sha2::{Digest, Sha256};
        use std::io::Read;
        if !self.path.is_absolute() || !hex_digest(&self.sha256, 64) {
            return Err(invalid(
                "file provenance requires an absolute path and lowercase SHA256",
            ));
        }
        let mut file = std::fs::File::open(&self.path)?;
        let before = file.metadata()?;
        if !before.is_file() {
            return Err(invalid("provenance input must be a regular file"));
        }
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            if deadline.is_some_and(|end| std::time::Instant::now() >= end) {
                return Err(invalid(
                    "provenance hashing exceeded reserved job wall time",
                ));
            }
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let after = file.metadata()?;
        let path_after = std::fs::metadata(&self.path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if (before.dev(), before.ino()) != (path_after.dev(), path_after.ino()) {
                return Err(invalid("provenance file path changed during verification"));
            }
        }
        if before.len() != after.len()
            || before.modified()? != after.modified()?
            || after.len() != path_after.len()
            || after.modified()? != path_after.modified()?
            || format!("{:x}", hasher.finalize()) != self.sha256
        {
            return Err(invalid("provenance file changed or SHA256 does not match"));
        }
        Ok(())
    }
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 180
        && id != "."
        && id != ".."
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}

impl JobSpec {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let spec: JobSpec = serde_json::from_slice(bytes)?;
        spec.validate()?;
        Ok(spec)
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(schema) = &self.schema {
            if schema != JOB_SCHEMA {
                return Err(invalid(format!(
                    "spec schema must be {JOB_SCHEMA}, found {schema}"
                )));
            }
        }
        if !valid_id(&self.id) {
            return Err(invalid("spec id must be a non-empty single path component"));
        }
        if self.argv.is_empty() || self.argv[0].is_empty() {
            return Err(invalid("spec argv must be non-empty"));
        }
        if !self.cwd.is_dir() {
            return Err(invalid(format!(
                "spec cwd {} does not exist or is not a directory",
                self.cwd.display()
            )));
        }
        if self.threads == 0 {
            return Err(invalid("spec threads must be at least 1"));
        }
        if !(self.rss_gib > 0.0) || !self.rss_gib.is_finite() {
            return Err(invalid("spec rss_gib must be a positive finite number"));
        }
        if self.wall_s == 0 || self.wall_s > u64::MAX / 1000 {
            return Err(invalid("spec wall_s must be at least 1"));
        }
        if self.stop_grace_ms > 30_000 {
            return Err(invalid("stop_grace_ms must be at most 30000"));
        }
        self.reserved_ms()?;
        if self.lab.is_empty() || !self.cwd.is_absolute() {
            return Err(invalid("lab must be named and cwd absolute"));
        }
        if self
            .env
            .as_ref()
            .is_some_and(|env| env.keys().any(|key| key.starts_with("UOR_RUNNER_")))
        {
            return Err(invalid(
                "UOR_RUNNER_ environment is reserved for process ownership",
            ));
        }
        if self.storage.iter().any(|r| {
            r.volume.is_empty() || r.additional_bytes.checked_add(r.checkpoint_bytes).is_none()
        }) {
            return Err(invalid("invalid storage reservation"));
        }
        if std::path::Path::new(&self.argv[0])
            .file_name()
            .is_some_and(|n| n == "cargo")
            && !self.cargo
        {
            return Err(invalid("Cargo jobs must declare cargo=true"));
        }
        if let Some(claim) = &self.coordination {
            crate::coord::work_card_digest(&claim.work_card)?;
            if claim.attempt_id != self.id
                || claim.attempt_id.len() > 128
                || !claim
                    .attempt_id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
            {
                return Err(invalid(
                    "coordinated job id must equal its reserved immutable attempt id",
                ));
            }
        }
        if let Some(provenance) = &self.provenance {
            if !hex_digest(&provenance.source_sha, 40)
                || !provenance.executable.path.is_absolute()
                || !hex_digest(&provenance.executable.sha256, 64)
                || provenance
                    .inputs
                    .iter()
                    .any(|input| !input.path.is_absolute() || !hex_digest(&input.sha256, 64))
            {
                return Err(invalid("invalid source/executable/input provenance"));
            }
        }
        match self.kill_criterion.kind {
            KillKind::Wall => {}
            KillKind::LogMatch | KillKind::Command => {
                if self.kill_criterion.value.is_empty() {
                    return Err(invalid(
                        "kill_criterion value must be non-empty for log_match and command",
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn reserved_ms(&self) -> Result<u64> {
        self.wall_s
            .checked_mul(1000)
            .and_then(|ms| ms.checked_add(self.stop_grace_ms))
            .and_then(|ms| ms.checked_add(STOP_ENFORCEMENT_MARGIN_MS))
            .ok_or_else(|| invalid("wall plus stop reservation overflow"))
    }

    /// Expensive hashing happens only at an otherwise idle production host,
    /// immediately before opening the launch gate. Paths remain cooperative
    /// immutable inputs; this is not isolation against a hostile same-user writer.
    pub fn verify_provenance(&self) -> Result<()> {
        use std::process::Command;
        self.validate()?;
        let provenance = self
            .provenance
            .as_ref()
            .ok_or_else(|| invalid("production job requires source/executable/input provenance"))?;
        if !hex_digest(&provenance.source_sha, 40)
            || !std::path::Path::new(&self.argv[0]).is_absolute()
            || std::fs::canonicalize(&self.argv[0])?
                != std::fs::canonicalize(&provenance.executable.path)?
        {
            return Err(invalid(
                "production argv executable/source identity mismatch",
            ));
        }
        let git = |args: &[&str]| -> Result<std::process::Output> {
            Ok(Command::new("/usr/bin/git")
                .arg("-C")
                .arg(&self.cwd)
                .args(args)
                .output()?)
        };
        let head = git(&["rev-parse", "--verify", "HEAD"])?;
        if !head.status.success()
            || String::from_utf8_lossy(&head.stdout).trim() != provenance.source_sha
        {
            return Err(invalid("source HEAD does not match reserved provenance"));
        }
        if !git(&["diff", "--quiet", "HEAD", "--"])?.status.success() {
            return Err(invalid("tracked source differs from reserved clean commit"));
        }
        let untracked = git(&["ls-files", "--others", "--exclude-standard", "-z"])?;
        if !untracked.status.success() || !untracked.stdout.is_empty() {
            return Err(invalid(
                "source worktree has untracked files outside ignored outputs",
            ));
        }
        let deadline = std::time::Instant::now()
            .checked_add(std::time::Duration::from_secs(self.wall_s))
            .ok_or_else(|| invalid("provenance verification wall limit overflows clock"))?;
        provenance.executable.verify_until(Some(deadline))?;
        for input in &provenance.inputs {
            input.verify_until(Some(deadline))?;
        }
        // Empty inputs are reserved for explicit inert pilot programs; a model,
        // compiler or research executable must bind its declared input files.
        if provenance.inputs.is_empty() {
            let executable = std::fs::canonicalize(&provenance.executable.path)?;
            let inert = ["/usr/bin/true", "/usr/bin/false", "/bin/sleep"]
                .iter()
                .filter_map(|path| std::fs::canonicalize(path).ok())
                .any(|path| path == executable);
            if !inert {
                return Err(invalid("production research job requires input provenance"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_spec_json() -> serde_json::Value {
        serde_json::json!({
            "schema": JOB_SCHEMA,
            "id": "job-1",
            "lab": "lab-test",
            "cwd": std::env::temp_dir(),
            "argv": ["sh", "-c", "true"],
            "threads": 1,
            "rss_gib": 0.5,
            "gpu": false,
            "wall_s": 10,
            "kill_criterion": {"kind": "wall", "value": ""},
            "exclusive": false
        })
    }

    #[test]
    fn valid_spec_parses() {
        let spec = JobSpec::parse(&serde_json::to_vec(&base_spec_json()).unwrap()).unwrap();
        assert_eq!(spec.id, "job-1");
        assert_eq!(spec.kill_criterion.kind, KillKind::Wall);
    }

    #[test]
    fn missing_wall_s_names_the_field() {
        let mut value = base_spec_json();
        value.as_object_mut().unwrap().remove("wall_s");
        let error = JobSpec::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(
            error.to_string().contains("wall_s"),
            "error must name wall_s: {error}"
        );
    }

    #[test]
    fn missing_kill_criterion_names_the_field() {
        let mut value = base_spec_json();
        value.as_object_mut().unwrap().remove("kill_criterion");
        let error = JobSpec::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(
            error.to_string().contains("kill_criterion"),
            "error must name kill_criterion: {error}"
        );
    }

    #[test]
    fn empty_argv_and_missing_cwd_are_rejected() {
        let mut value = base_spec_json();
        value["argv"] = serde_json::json!([]);
        let error = JobSpec::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(error.to_string().contains("argv"), "{error}");

        let mut value = base_spec_json();
        value["cwd"] = serde_json::json!("/definitely/not/a/real/dir-lab-runner");
        let error = JobSpec::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(error.to_string().contains("cwd"), "{error}");
    }

    #[test]
    fn log_match_requires_a_value() {
        let mut value = base_spec_json();
        value["kill_criterion"] = serde_json::json!({"kind": "log_match", "value": ""});
        let error = JobSpec::parse(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(error.to_string().contains("kill_criterion"), "{error}");
    }

    #[test]
    fn provenance_rejects_changed_executable_input_and_untracked_source() {
        use std::{fs, process::Command};
        let root = std::env::temp_dir().join(format!(
            "uor-provenance-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let git = |args: &[&str]| {
            let out = Command::new("/usr/bin/git")
                .arg("-C")
                .arg(&root)
                .args(args)
                .output()
                .unwrap();
            assert!(out.status.success(), "fixture Git command failed");
            String::from_utf8(out.stdout).unwrap().trim().to_string()
        };
        git(&["init", "--quiet"]);
        fs::write(root.join("source.rs"), b"fn main() {}\n").unwrap();
        fs::write(root.join(".gitignore"), b"/artifact.bin\n/input.bin\n").unwrap();
        git(&["add", "source.rs", ".gitignore"]);
        git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ]);
        let head = git(&["rev-parse", "HEAD"]);
        fs::write(root.join("artifact.bin"), b"executable v1").unwrap();
        fs::write(root.join("input.bin"), b"input v1").unwrap();
        let mut value = base_spec_json();
        value["cwd"] = serde_json::json!(root);
        value["argv"] = serde_json::json!([root.join("artifact.bin")]);
        value["provenance"] = serde_json::json!({"source_sha":head,
            "executable":{"path":root.join("artifact.bin"),"sha256":crate::coord::digest(b"executable v1")},
            "inputs":[{"path":root.join("input.bin"),"sha256":crate::coord::digest(b"input v1")}]});
        let spec = JobSpec::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        spec.verify_provenance().unwrap();
        fs::write(root.join("artifact.bin"), b"executable v2").unwrap();
        assert!(spec.verify_provenance().is_err());
        fs::write(root.join("artifact.bin"), b"executable v1").unwrap();
        fs::write(root.join("input.bin"), b"input v2").unwrap();
        assert!(spec.verify_provenance().is_err());
        fs::write(root.join("input.bin"), b"input v1").unwrap();
        fs::write(root.join("build.rs"), b"fn main() {}\n").unwrap();
        assert!(spec
            .verify_provenance()
            .unwrap_err()
            .to_string()
            .contains("untracked"));
        fs::remove_file(root.join("build.rs")).unwrap();
        fs::write(root.join("source.rs"), b"fn main() { changed(); }\n").unwrap();
        assert!(spec
            .verify_provenance()
            .unwrap_err()
            .to_string()
            .contains("tracked source"));
        fs::remove_dir_all(root).unwrap();
    }
}
