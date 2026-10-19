//! `archive-verify`: prove checksum manifests still hold and `archive/` is immutable.
//!
//! `cargo xtask archive-verify [--json OUT] [--baseline FILE] [--base REV]`
//!
//! Part A hashes every file named by a tracked `SHA256SUMS`-style manifest and
//! reports `ok`, `mismatch`, `missing` or `unparsable`; with `--baseline` it
//! fails only on an entry that was `ok` and no longer is. Part B (`--base REV`)
//! fails when a file under `archive/` at REV was modified or deleted.

use std::collections::BTreeMap;
use std::io::Write;

use crate::docs_links::{git as git_links, normalize, tracked_files, PathMap, Universe};
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const SCHEMA: &str = "uor-r4.archive-verify/1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    pub manifest: String,
    pub path: String,
    pub status: String,
}

/// A manifest is a tracked file whose basename is or contains `SHA256SUMS`,
/// or matches `*sha256sums*.txt` case-insensitively.
pub fn is_manifest(path: &str) -> bool {
    let base = path.rsplit('/').next().unwrap_or(path);
    let lower = base.to_ascii_lowercase();
    base.contains("SHA256SUMS") || (lower.contains("sha256sums") && lower.ends_with(".txt"))
}

/// Parse `<64 hex><spaces>[*]<relpath>` (CRLF tolerated).
pub fn parse_line(line: &str) -> Option<(String, String)> {
    let line = line.trim_end_matches(['\r', '\n']);
    if line.len() < 66 || !line.is_char_boundary(64) {
        return None;
    }
    let (hash, rest) = line.split_at(64);
    if !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let path = rest.trim_start_matches(' ');
    if path.len() == rest.len() {
        return None;
    }
    let path = path.strip_prefix('*').unwrap_or(path);
    if path.is_empty() {
        return None;
    }
    Some((hash.to_ascii_lowercase(), path.to_string()))
}

fn file_sha256(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(format!("{:x}", Sha256::digest(&bytes)))
}

/// Verify one manifest's text. `hash_of` maps a repo-relative path to its hash.
pub fn verify_manifest(
    manifest: &str,
    text: &str,
    hash_of: &dyn Fn(&str) -> Option<String>,
) -> Vec<Entry> {
    let dir = manifest.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let (path, status) = match parse_line(line) {
            None => (t.to_string(), "unparsable"),
            Some((want, rel)) => {
                let full = if dir.is_empty() {
                    rel.clone()
                } else {
                    format!("{dir}/{rel}")
                };
                let status = match hash_of(&full) {
                    None => "missing",
                    Some(got) if got == want => "ok",
                    Some(_) => "mismatch",
                };
                (rel, status)
            }
        };
        out.push(Entry {
            manifest: manifest.to_string(),
            path,
            status: status.to_string(),
        });
    }
    out
}

fn counts(entries: &[Entry]) -> [usize; 4] {
    let mut c = [0usize; 4];
    for e in entries {
        match e.status.as_str() {
            "ok" => c[0] += 1,
            "mismatch" => c[1] += 1,
            "missing" => c[2] += 1,
            _ => c[3] += 1,
        }
    }
    c
}

fn entry_key(map: &PathMap, manifest: &str, path: &str) -> (String, String) {
    let rel = normalize(path).unwrap_or_else(|_| path.to_string());
    (map.map(manifest), rel)
}

/// Baseline entries that were `ok` and are not now. Each is matched against
/// exactly the current entry with the same key (mapped full manifest path,
/// normalized relpath). An `ok` entry with no current counterpart is reported
/// as `vanished`.
pub fn regressions(baseline: &[Entry], current: &[Entry], map: &PathMap) -> Vec<Entry> {
    let mut now: BTreeMap<(String, String), Vec<&Entry>> = BTreeMap::new();
    for c in current {
        now.entry(entry_key(map, &c.manifest, &c.path))
            .or_default()
            .push(c);
    }
    let mut out = Vec::new();
    for b in baseline.iter().filter(|b| b.status == "ok") {
        match now.get(&entry_key(map, &b.manifest, &b.path)) {
            None => out.push(Entry {
                manifest: map.map(&b.manifest),
                path: b.path.clone(),
                status: "vanished".to_string(),
            }),
            Some(cs) => {
                if cs.iter().all(|c| c.status != "ok") {
                    out.push((*cs[0]).clone());
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn to_json(manifests: usize, entries: &[Entry]) -> Value {
    let c = counts(entries);
    json!({
        "schema": SCHEMA,
        "manifests": manifests,
        "entries": entries.iter().map(|e| json!({
            "manifest": e.manifest, "path": e.path, "status": e.status,
        })).collect::<Vec<_>>(),
        "counts": {"ok": c[0], "mismatch": c[1], "missing": c[2], "unparsable": c[3]},
    })
}

fn read_baseline(path: &str) -> Result<Vec<Entry>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))?;
    if v["schema"] != SCHEMA {
        return Err(format!("{path}: not a {SCHEMA} file"));
    }
    let arr = v["entries"]
        .as_array()
        .ok_or_else(|| format!("{path}: no entries"))?;
    let field = |e: &Value, k: &str| e[k].as_str().map(str::to_string);
    arr.iter()
        .map(|e| {
            Ok(Entry {
                manifest: field(e, "manifest").ok_or("entry without manifest")?,
                path: field(e, "path").ok_or("entry without path")?,
                status: field(e, "status").ok_or("entry without status")?,
            })
        })
        .collect::<Result<_, &str>>()
        .map_err(|e| format!("{path}: {e}"))
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}

fn part_a(
    root: &Path,
    json_out: Option<&str>,
    baseline: Option<&str>,
    renames_from: Option<&str>,
) -> Result<(), String> {
    let listing = git(root, &["ls-files", "-z"])?;
    let mut manifests: Vec<String> = listing
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .filter(|p| is_manifest(p))
        .collect();
    manifests.sort();
    let mut entries = Vec::new();
    for m in &manifests {
        let bytes = std::fs::read(root.join(m)).map_err(|e| format!("{m}: {e}"))?;
        let text = String::from_utf8_lossy(&bytes);
        entries.extend(verify_manifest(m, &text, &|p| file_sha256(&root.join(p))));
    }
    entries.sort();
    if let Some(out) = json_out {
        if let Some(parent) = Path::new(out)
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        let body = serde_json::to_string_pretty(&to_json(manifests.len(), &entries))
            .map_err(|e| e.to_string())?;
        std::fs::write(out, body + "\n").map_err(|e| format!("{out}: {e}"))?;
    }
    let c = counts(&entries);
    let regs = match baseline {
        Some(b) => {
            let map = match renames_from {
                Some(rev) => {
                    let out = git_links(
                        root,
                        &["diff", "--cached", "-M100%", "--name-status", "-z", rev],
                    )?;
                    let mut m = PathMap::parse_renames_z(&out);
                    m.add_dir_renames(&Universe::new(tracked_files(root)?));
                    m
                }
                None => PathMap::default(),
            };
            regressions(&read_baseline(b)?, &entries, &map)
        }
        None => Vec::new(),
    };
    for r in &regs {
        eprintln!(
            "regression: {} :: {} is now {}",
            r.manifest, r.path, r.status
        );
    }
    println!(
        "manifests {}, ok {}, mismatch {}, missing {}, regressions {}",
        manifests.len(),
        c[0],
        c[1],
        c[2],
        regs.len()
    );
    if regs.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} checksum regression(s) against the baseline",
            regs.len()
        ))
    }
}

/// True when the working-tree executable bit disagrees with the mode git
/// recorded (`100755` executable, `100644` not).
fn mode_changed(rev_mode: &str, executable_now: bool) -> bool {
    match rev_mode {
        "100755" => !executable_now,
        "100644" => executable_now,
        _ => false,
    }
}

#[cfg(unix)]
fn executable(meta: &std::fs::Metadata) -> Option<bool> {
    use std::os::unix::fs::PermissionsExt;
    Some(meta.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn executable(_meta: &std::fs::Metadata) -> Option<bool> {
    None
}

/// The git blob hash of `bytes`.
fn blob_hash(root: &Path, bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("git")
        .args(["hash-object", "--stdin"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git hash-object: {e}"))?;
    child
        .stdin
        .take()
        .ok_or("git hash-object: no stdin")?
        .write_all(bytes)
        .map_err(|e| format!("git hash-object: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("git hash-object: {e}"))?;
    if !out.status.success() {
        return Err("git hash-object failed".to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn part_b(root: &Path, rev: &str) -> Result<(), String> {
    let listing = git(root, &["ls-tree", "-r", "-z", rev, "--", "archive/"])?;
    // "<mode> <type> <hash>\t<path>"
    let mut files: Vec<(String, String, String)> = Vec::new();
    for rec in listing.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        let rec = String::from_utf8_lossy(rec);
        if let Some((meta, path)) = rec.split_once('\t') {
            let mut f = meta.split(' ');
            let (mode, _ty, hash) = (f.next(), f.next(), f.next());
            if let (Some(mode), Some(hash)) = (mode, hash) {
                files.push((mode.to_string(), hash.to_string(), path.to_string()));
            }
        }
    }
    if files.is_empty() {
        println!("archive immutability: {rev} has no archive/ tree; nothing to compare");
        return Ok(());
    }
    let (mut deleted, mut modified) = (Vec::new(), Vec::new());
    let mut to_hash: Vec<&(String, String, String)> = Vec::new();
    let mut submodules = 0usize;
    for f in &files {
        let path = root.join(&f.2);
        match path.symlink_metadata() {
            Err(_) => deleted.push(f.2.clone()),
            Ok(_) if f.0 == "160000" => submodules += 1,
            Ok(meta) if f.0 == "120000" => {
                // A symlink: its blob is the link target text.
                let target = if meta.file_type().is_symlink() {
                    std::fs::read_link(&path).ok()
                } else {
                    None
                };
                match target {
                    Some(t) => {
                        if blob_hash(root, t.as_os_str().as_encoded_bytes())? != f.1 {
                            modified.push(f.2.clone());
                        }
                    }
                    None => modified.push(f.2.clone()),
                }
            }
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    modified.push(f.2.clone());
                } else if executable(&meta).is_some_and(|x| mode_changed(&f.0, x)) {
                    modified.push(f.2.clone());
                } else {
                    to_hash.push(f);
                }
            }
        }
    }
    if submodules > 0 {
        println!("archive immutability: {submodules} submodule entr(ies) skipped");
    }
    let mut child = Command::new("git")
        .args(["hash-object", "--stdin-paths"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git hash-object: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("git hash-object: no stdin")?;
    let input: String = to_hash.iter().map(|f| format!("{}\n", f.2)).collect();
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let out = child
        .wait_with_output()
        .map_err(|e| format!("git hash-object: {e}"))?;
    writer
        .join()
        .map_err(|_| "git hash-object writer panicked".to_string())?
        .map_err(|e| format!("git hash-object: {e}"))?;
    if !out.status.success() {
        return Err("git hash-object failed".to_string());
    }
    let hashes: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    if hashes.len() != to_hash.len() {
        return Err(format!(
            "git hash-object returned {} hashes for {} paths",
            hashes.len(),
            to_hash.len()
        ));
    }
    for (f, h) in to_hash.iter().zip(&hashes) {
        if *h != f.1 {
            modified.push(f.2.clone());
        }
    }
    deleted.sort();
    modified.sort();
    for p in &deleted {
        eprintln!("archive deleted: {p}");
    }
    for p in &modified {
        eprintln!("archive modified: {p}");
    }
    println!(
        "archive immutability vs {rev}: {} files, {} modified, {} deleted",
        files.len(),
        modified.len(),
        deleted.len()
    );
    if modified.is_empty() && deleted.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "archive/ changed: {} modified, {} deleted",
            modified.len(),
            deleted.len()
        ))
    }
}

/// Entry point: `args` are the arguments after the task name.
pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let opt = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .map(String::as_str)
    };
    let a = part_a(
        root,
        opt("--json"),
        opt("--baseline"),
        opt("--renames-from"),
    );
    let b = match opt("--base") {
        Some(rev) => part_b(root, rev),
        None => Ok(()),
    };
    match (a, b) {
        (Ok(()), Ok(())) => Ok(()),
        (a, b) => Err([a.err(), b.err()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("; ")),
    }
}

#[cfg(test)]
mod archive_verify_tests {
    use super::*;

    // T4: an executable-bit flip either way is a modification.
    #[test]
    fn executable_bit_flip_is_a_modification() {
        assert!(mode_changed("100644", true));
        assert!(mode_changed("100755", false));
        assert!(!mode_changed("100644", false));
        assert!(!mode_changed("100755", true));
        assert!(!mode_changed("120000", true));
    }

    const H: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn e(m: &str, p: &str, s: &str) -> Entry {
        Entry {
            manifest: m.into(),
            path: p.into(),
            status: s.into(),
        }
    }

    #[test]
    fn parses_two_spaces_star_and_crlf() {
        assert_eq!(
            parse_line(&format!("{H}  a/b.txt")),
            Some((H.into(), "a/b.txt".into()))
        );
        assert_eq!(
            parse_line(&format!("{H} *b.bin\r\n")),
            Some((H.into(), "b.bin".into()))
        );
        let upper = H.to_ascii_uppercase();
        assert_eq!(parse_line(&format!("{upper}  x")).unwrap().0, H);
    }

    #[test]
    fn rejects_bad_hex_short_and_unspaced() {
        assert!(parse_line(&format!("{}g  x", &H[..63])).is_none());
        assert!(parse_line("abc  x").is_none());
        assert!(parse_line(&format!("{H}x")).is_none());
        assert!(parse_line(&format!("{H}  ")).is_none());
    }

    #[test]
    fn manifest_names() {
        assert!(is_manifest("a/SHA256SUMS"));
        assert!(is_manifest("a/SHA256SUMS.txt"));
        assert!(is_manifest("a/run-Sha256Sums-1.TXT"));
        assert!(!is_manifest("a/sha256sums.md"));
        assert!(!is_manifest("a/README.txt"));
    }

    #[test]
    fn verify_statuses() {
        let text = format!("{H}  ok.txt\n{H}  bad.txt\n{H}  gone.txt\nnot a line\n\n");
        let hash = |p: &str| match p {
            "d/ok.txt" => Some(H.to_string()),
            "d/bad.txt" => Some("f".repeat(64)),
            _ => None,
        };
        let got: Vec<_> = verify_manifest("d/SHA256SUMS", &text, &hash)
            .into_iter()
            .map(|x| (x.path, x.status))
            .collect();
        assert_eq!(
            got,
            vec![
                ("ok.txt".to_string(), "ok".to_string()),
                ("bad.txt".to_string(), "mismatch".to_string()),
                ("gone.txt".to_string(), "missing".to_string()),
                ("not a line".to_string(), "unparsable".to_string()),
            ]
        );
    }

    fn id() -> PathMap {
        PathMap::default()
    }

    #[test]
    fn ok_to_mismatch_is_a_regression() {
        let r = regressions(
            &[e("d/SHA256SUMS", "a", "ok")],
            &[e("d/SHA256SUMS", "a", "mismatch")],
            &id(),
        );
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn missing_to_missing_is_not_a_regression() {
        let r = regressions(
            &[e("d/SHA256SUMS", "a", "missing")],
            &[e("d/SHA256SUMS", "a", "missing")],
            &id(),
        );
        assert!(r.is_empty());
    }

    // T1: same basename in two directories must not share a baseline key.
    #[test]
    fn same_basename_manifests_are_keyed_apart() {
        let base = [
            e("docs/a/SHA256SUMS", "README.md", "ok"),
            e("docs/b/SHA256SUMS", "README.md", "missing"),
        ];
        let cur = [
            e("docs/a/SHA256SUMS", "README.md", "mismatch"),
            e("docs/b/SHA256SUMS", "README.md", "missing"),
        ];
        let r = regressions(&base, &cur, &id());
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].manifest, "docs/a/SHA256SUMS");
    }

    // T2: an ok entry with no current counterpart is `vanished`, including a
    // `./x` -> `x` rewrite without normalization.
    #[test]
    fn ok_entries_that_vanish_are_regressions() {
        let base = [e("d/SHA256SUMS", "a", "ok"), e("d/SHA256SUMS", "b", "ok")];
        let r = regressions(&base, &[], &id());
        assert_eq!(r.len(), 2);
        assert!(r.iter().all(|x| x.status == "vanished"));
        let dot = regressions(
            &[e("d/SHA256SUMS", "./x", "ok")],
            &[e("d/SHA256SUMS", "x", "ok")],
            &id(),
        );
        assert!(dot.is_empty());
    }

    // T3: a manifest moved by the rename map still matches.
    #[test]
    fn moved_manifest_matches_through_the_rename_map() {
        let map = PathMap::parse_tsv("docs/x\tarchive/x\n").expect("map");
        let base = [e("docs/x/SHA256SUMS", "a", "ok")];
        let good = [e("archive/x/SHA256SUMS", "a", "ok")];
        assert!(regressions(&base, &good, &map).is_empty());
        let bad = [e("archive/x/SHA256SUMS", "a", "missing")];
        assert_eq!(regressions(&base, &bad, &map).len(), 1);
        assert_eq!(regressions(&base, &[], &map).len(), 1);
    }
}
