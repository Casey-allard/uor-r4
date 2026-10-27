//! The lab registry, and the check that refuses a lab it does not list.
//!
//! **Why this exists.** Every recording command attributes its record to `--lab` or `$UOR_LAB`, and
//! until this module nothing checked the name. `UOR_LAB=claud` wrote a heartbeat, a receipt and a
//! claim owned by a lab that does not exist, and no reader could tell it from a real one. The only
//! roster was the set of labs that had already published a presence file, which is a list of who
//! *has* written, not of who *may*.
//!
//! **Why a file and not a constant.** `onboard::KNOWN_LABS` was a roster in code, and it drifted
//! from the roster in the rules (four names against five) the day a fifth lab onboarded. The
//! registry is `docs/harness/labs.json`, which only the owner changes (`owner-paths.json`), so
//! adding a lab is an owner decision recorded in one place rather than a code change.
//!
//! **Why a missing registry refuses.** A check that cannot run is a refusal, not a pass. If
//! `labs.json` is absent or malformed, every name would otherwise be accepted, which is exactly the
//! silent failure this module exists to remove.
//!
//! Names are compared after `state::slug`, because the harness slugs a lab into every path it
//! writes: `DeepSeek` and `deepseek` land in one shard, so refusing on case would be nagging.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// One registered lab.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Lab {
    pub vendor: String,
    pub client: String,
    pub headless: String,
    pub open_pr_limit: u32,
    /// `pm` for the project manager; absent for every other lab.
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Deserialize)]
struct LabsFile {
    labs: BTreeMap<String, Lab>,
}

fn registry_in(root: &Path) -> PathBuf {
    root.join("docs").join("harness").join("labs.json")
}

/// `docs/harness/labs.json` for the repository containing `repo`.
///
/// `repo` is the working directory unless `--repo` was given, so it may be a subdirectory. Inside a
/// git work tree the registry is **only** the one at that tree's top level (`state::toplevel`, the
/// resolution the state paths use). The walk up the ancestors is for a checkout that is not a git
/// work tree, and it never runs inside one: a worktree under `.worktrees/` branched before the
/// registry existed would otherwise climb out of itself and borrow the primary checkout's file, so a
/// missing registry would pass instead of refusing (round-2 review of #2898). With neither, the path
/// under `repo` is returned so the error names where the file was expected.
pub fn labs_path(repo: &Path) -> PathBuf {
    if let Some(top) = crate::state::toplevel(repo) {
        return registry_in(&top);
    }
    repo.ancestors()
        .map(registry_in)
        .find(|p| p.is_file())
        .unwrap_or_else(|| registry_in(repo))
}

/// Every registered lab, keyed and sorted by name.
///
/// An unreadable or malformed file is an error naming the file, never an empty registry: an empty
/// registry would refuse everyone, and a lenient reader would accept everyone. Both hide the cause.
pub fn load_labs(repo: &Path) -> Result<BTreeMap<String, Lab>, String> {
    let path = labs_path(repo);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read the lab registry {}: {e}", path.display()))?;
    let doc: LabsFile = serde_json::from_str(&text)
        .map_err(|e| format!("the lab registry {} is malformed: {e}", path.display()))?;
    if doc.labs.is_empty() {
        return Err(format!("the lab registry {} lists no labs", path.display()));
    }
    Ok(doc.labs)
}

/// The labs in registry `text` (the same checks as [`load_labs`], for text read from elsewhere,
/// such as `git show origin/main:docs/harness/labs.json`).
pub(crate) fn parse_labs(text: &str) -> Result<BTreeMap<String, Lab>, String> {
    let doc: LabsFile =
        serde_json::from_str(text).map_err(|e| format!("the lab registry is malformed: {e}"))?;
    if doc.labs.is_empty() {
        return Err("the lab registry lists no labs".to_string());
    }
    Ok(doc.labs)
}

/// Accept `name` only if the registry lists it.
///
/// The refusal names the unknown lab and lists every registered one, so the fix is on screen.
pub fn check_lab(repo: &Path, name: &str) -> Result<(), String> {
    let labs = load_labs(repo).map_err(|e| format!("cannot check lab {name:?}: {e}"))?;
    let slug = crate::state::slug(name);
    if labs.contains_key(&slug) {
        return Ok(());
    }
    Err(format!(
        "lab {name:?} is not in docs/harness/labs.json; the registered labs are {}. \
         A new lab is an owner decision recorded in that file.",
        labs.keys().cloned().collect::<Vec<_>>().join(", ")
    ))
}

/// The one resolver every recording command goes through: `--lab` first, then `$UOR_LAB`.
///
/// No lab at all resolves to an empty string, so each command keeps its own "no lab identity"
/// handling (some warn and record unattributed, some refuse). A lab that IS given must be
/// registered; that is the case nothing checked before.
pub fn resolve_lab(
    repo: &Path,
    flag: Option<String>,
    env: Option<String>,
) -> Result<String, String> {
    let lab = flag.or(env).unwrap_or_default();
    if lab.trim().is_empty() {
        return Ok(lab);
    }
    check_lab(repo, &lab)?;
    Ok(lab)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FOUR: [&str; 4] = ["antigravity", "claude", "deepseek", "kimi"];

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn with_registry(text: Option<&str>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        if let Some(text) = text {
            let harness = dir.path().join("docs").join("harness");
            std::fs::create_dir_all(&harness).expect("mkdir");
            std::fs::write(harness.join("labs.json"), text).expect("write");
        }
        dir
    }

    /// The planted fault the loop ORDER requires: an unregistered lab is refused by the check and
    /// by the resolver every recording command uses, and the refusal lists the four.
    #[test]
    fn an_unregistered_lab_is_refused() {
        let err = check_lab(&repo(), "gemini").expect_err("gemini is not registered");
        assert!(err.contains("\"gemini\""), "{err}");
        assert!(err.contains("antigravity, claude, deepseek, kimi"), "{err}");
        // Codex left the loop on 26 Oct 2026 (claude-138): its name is refused like any other.
        let err = check_lab(&repo(), "codex").expect_err("codex is not registered");
        assert!(err.contains("\"codex\""), "{err}");
        assert!(resolve_lab(&repo(), None, Some("codex".into())).is_err());
        let err = resolve_lab(&repo(), None, Some("gemini".into()))
            .expect_err("the resolver refuses it too");
        assert!(err.contains("\"gemini\""), "{err}");
        assert!(resolve_lab(&repo(), Some("gemini".into()), Some("claude".into())).is_err());
    }

    #[test]
    fn each_registered_lab_is_accepted() {
        for lab in FOUR {
            check_lab(&repo(), lab).unwrap_or_else(|e| panic!("{lab}: {e}"));
            assert_eq!(
                resolve_lab(&repo(), Some(lab.into()), None).as_deref(),
                Ok(lab)
            );
        }
        // Slugged, as every path the harness writes is.
        assert!(check_lab(&repo(), "DeepSeek").is_ok());
    }

    /// No lab given is not this module's call: each command keeps its own warning or refusal.
    #[test]
    fn no_lab_resolves_empty_rather_than_refusing() {
        assert_eq!(resolve_lab(&repo(), None, None).as_deref(), Ok(""));
        assert_eq!(
            resolve_lab(&repo(), None, Some("  ".into())).as_deref(),
            Ok("  ")
        );
    }

    #[test]
    fn the_real_registry_lists_exactly_the_four() {
        let labs = load_labs(&repo()).expect("labs.json loads");
        assert_eq!(labs.keys().map(String::as_str).collect::<Vec<_>>(), FOUR);
        assert_eq!(labs["kimi"].role.as_deref(), Some("pm"));
        assert!(labs.values().filter(|l| l.role.is_some()).count() == 1);
    }

    /// A check that cannot run is a refusal, not a pass.
    #[test]
    fn a_missing_or_malformed_registry_refuses() {
        let missing = with_registry(None);
        assert!(load_labs(missing.path()).is_err());
        let err = check_lab(missing.path(), "claude").expect_err("missing registry refuses");
        assert!(err.contains("cannot read the lab registry"), "{err}");

        let malformed = with_registry(Some("{not json"));
        let err = check_lab(malformed.path(), "claude").expect_err("malformed refuses");
        assert!(err.contains("malformed"), "{err}");

        let empty = with_registry(Some(r#"{"schema": 1, "labs": {}}"#));
        assert!(check_lab(empty.path(), "claude").is_err());
    }

    /// Every owner path that is not a glob exists, so the list cannot name a file that moved.
    #[test]
    fn every_owner_path_and_rule_source_exists() {
        let root = repo();
        let read = |name: &str| -> serde_json::Value {
            let p = root.join("docs/harness").join(name);
            serde_json::from_str(&std::fs::read_to_string(&p).expect("reads")).expect("parses")
        };
        let owner = read("owner-paths.json");
        let paths = owner["paths"].as_array().expect("paths list");
        assert!(!paths.is_empty());
        for p in paths {
            let path = p["path"].as_str().expect("path");
            assert!(
                !p["why"].as_str().unwrap_or("").is_empty(),
                "{path}: no why"
            );
            if path.contains('*') {
                let base = path.split('*').next().unwrap_or("");
                assert!(root.join(base).exists(), "{path}: glob base missing");
            } else {
                assert!(root.join(path).exists(), "{path} does not exist");
            }
        }
        for name in ["labs.json", "owner-paths.json", "rule-sources.json"] {
            let listed = format!("docs/harness/{name}");
            assert!(
                paths.iter().any(|p| p["path"] == listed.as_str()),
                "{listed} must be owner-only"
            );
        }
        let sources = read("rule-sources.json");
        let files = sources["files"].as_array().expect("files list");
        assert!(!files.is_empty());
        for f in files {
            let f = f.as_str().expect("a path");
            assert!(root.join(f).exists(), "rule source {f} does not exist");
        }
    }
}
