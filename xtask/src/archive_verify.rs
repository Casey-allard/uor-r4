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
        // The repository is `root`; an inherited GIT_DIR (set inside git hooks)
        // would point every call at another repository.
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
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
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
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

/// A base-revision file that is a generated pointer stub (exactly the
/// `docs_stubs` template) replaced by the very record it pointed to: the stub's
/// `docs/` target no longer exists in the working tree, and the new content is
/// byte-identical to that target as it was at the base revision (`rev_blob`
/// gives the target's blob id at REV). Anything else written over a stub is a
/// modification, so a stub can never become an unrelated "archived record".
fn stub_replaced(
    base: &[u8],
    new_blob: &str,
    target_exists: &dyn Fn(&str) -> bool,
    rev_blob: &dyn Fn(&str) -> Option<String>,
) -> bool {
    std::str::from_utf8(base)
        .ok()
        .and_then(crate::docs_stubs::stub_target)
        .is_some_and(|t| {
            t.starts_with("docs/")
                && !target_exists(t)
                && rev_blob(t).is_some_and(|b| b == new_blob)
        })
}

/// The archive catalog's data file; it may only be appended to (#2982).
const CATALOG_DATA: &str = "archive/catalog.jsonl";

/// A pass for a modified catalog file under the append-only rule (#2982).
#[derive(Debug, PartialEq, Eq)]
enum CatalogChange {
    /// `archive/catalog.jsonl` gained this many lines and nothing else changed.
    Appended(usize),
    /// A page equal to `docs-catalog`'s output for the catalog now.
    Regenerated,
}

/// `archive/CATALOG.md` or `archive/catalog/<x>.md`, the only shapes
/// `docs_catalog::render` writes.
fn is_catalog_page(path: &str) -> bool {
    path == "archive/CATALOG.md"
        || path
            .strip_prefix("archive/catalog/")
            .and_then(|x| x.strip_suffix(".md"))
            .is_some_and(|x| !x.contains('/'))
}

/// Lines appended when `now` is `base` followed by new lines, the cut falls
/// on a line boundary and the whole of `now` parses as catalog records.
/// Anything else (an edited, removed or reordered row, no growth) is `None`.
fn catalog_appended(base: &[u8], now: &[u8]) -> Option<usize> {
    if now.len() <= base.len() || !now.starts_with(base) {
        return None;
    }
    if !(base.is_empty() || base.ends_with(b"\n")) {
        return None;
    }
    let text = std::str::from_utf8(now).ok()?;
    crate::docs_catalog::parse_records(text).ok()?;
    Some(text[base.len()..].lines().count())
}

/// `now` is byte-identical to the page `render` produces at `path`; a path
/// `render` does not produce, or not shaped like a catalog page, never passes.
fn catalog_page_regenerated(path: &str, now: &[u8], rendered: &[(String, String)]) -> bool {
    is_catalog_page(path)
        && rendered
            .iter()
            .any(|(p, c)| p == path && c.as_bytes() == now)
}

/// The append-only rule for one modified base-revision archive file: `base`
/// is its blob at REV, `now` its bytes in the working tree, and `pages` the
/// rendered catalog for the catalog data now (`None` when that does not parse).
fn catalog_change(
    path: &str,
    base: &[u8],
    now: &[u8],
    pages: Option<&[(String, String)]>,
) -> Option<CatalogChange> {
    if path == CATALOG_DATA {
        catalog_appended(base, now).map(CatalogChange::Appended)
    } else if pages.is_some_and(|p| catalog_page_regenerated(path, now, p)) {
        Some(CatalogChange::Regenerated)
    } else {
        None
    }
}

/// Every page `docs-catalog --check` expects for the catalog data `text`,
/// over the same tracked-file universe it uses.
fn render_catalog(root: &Path, text: &[u8]) -> Option<Vec<(String, String)>> {
    let recs = crate::docs_catalog::parse_records(std::str::from_utf8(text).ok()?).ok()?;
    // The same universe `docs-catalog --check` builds (docs_catalog::git_universe).
    let uni = Universe::new(tracked_files(root).ok()?);
    Some(crate::docs_catalog::render(&recs, &|p| uni.exists(p)))
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
    let mut replaced: Vec<String> = Vec::new();
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
                if meta.file_type().is_symlink()
                    || executable(&meta).is_some_and(|x| mode_changed(&f.0, x))
                {
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
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
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
    // The catalog data and its rendered pages, read once and only when needed.
    let catalog_now: std::cell::OnceCell<Option<Vec<u8>>> = std::cell::OnceCell::new();
    let catalog_pages: std::cell::OnceCell<Option<Vec<(String, String)>>> =
        std::cell::OnceCell::new();
    let read_catalog = || {
        catalog_now
            .get_or_init(|| std::fs::read(root.join(CATALOG_DATA)).ok())
            .as_deref()
    };
    let pages = || {
        catalog_pages
            .get_or_init(|| render_catalog(root, read_catalog()?))
            .as_deref()
    };
    let mut appended: Vec<(String, usize)> = Vec::new();
    let mut regenerated: Vec<String> = Vec::new();
    for (f, h) in to_hash.iter().zip(&hashes) {
        if *h != f.1 {
            let base_blob = git(root, &["cat-file", "blob", &f.1]);
            // A base that cannot be read never earns a catalog pass.
            let catalog = match &base_blob {
                Ok(b) if f.2 == CATALOG_DATA => {
                    read_catalog().and_then(|now| catalog_change(&f.2, b, now, None))
                }
                Ok(b) if is_catalog_page(&f.2) => std::fs::read(root.join(&f.2))
                    .ok()
                    .and_then(|now| catalog_change(&f.2, b, &now, pages())),
                _ => None,
            };
            match catalog {
                Some(CatalogChange::Appended(n)) => {
                    appended.push((f.2.clone(), n));
                    continue;
                }
                Some(CatalogChange::Regenerated) => {
                    regenerated.push(f.2.clone());
                    continue;
                }
                None => {}
            }
            let base = base_blob.unwrap_or_default();
            let rev_blob = |t: &str| {
                git(
                    root,
                    &["rev-parse", "--verify", "-q", &format!("{rev}:{t}")],
                )
                .ok()
                .map(|o| String::from_utf8_lossy(&o).trim().to_string())
            };
            if stub_replaced(
                &base,
                h,
                &|t| root.join(t).symlink_metadata().is_ok(),
                &rev_blob,
            ) {
                replaced.push(f.2.clone());
            } else {
                modified.push(f.2.clone());
            }
        }
    }
    deleted.sort();
    modified.sort();
    replaced.sort();
    regenerated.sort();
    for p in &replaced {
        println!("archive stub replaced: {p}");
    }
    for (p, n) in &appended {
        println!("archive catalog appended: {p} (+{n} lines)");
    }
    for p in &regenerated {
        println!("archive catalog regenerated: {p}");
    }
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

    // Stub replacement: a generated pointer stub may be replaced by the record
    // it points to, once its docs/ target is gone; nothing else may change.
    fn git_in(dir: &Path, args: &[&str]) {
        let mut c = Command::new("git");
        c.args(args).current_dir(dir);
        for (k, _) in std::env::vars() {
            if k.starts_with("GIT_") {
                c.env_remove(k);
            }
        }
        assert!(c.output().expect("git").status.success(), "git {args:?}");
    }

    /// The record `docs/t.md` holds at the base commit of every `stub_repo`.
    const RECORD: &str = "the real record\n";

    /// A throwaway repo under the gitignored `local/`, committed with
    /// `archive/stub.md` (a stub of `docs/t.md`), `archive/other.md` and the
    /// record `docs/t.md`. Unless `target_exists`, `docs/t.md` is then removed
    /// from the working tree, as a move of the record into the archive would.
    fn stub_repo(name: &str, target_exists: bool) -> std::path::PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../local/archive-verify-tests")
            .join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("archive")).expect("mkdir");
        std::fs::create_dir_all(dir.join("docs")).expect("mkdir");
        let stub = crate::docs_stubs::stub_text("docs/t.md", "../docs/t.md");
        std::fs::write(dir.join("archive/stub.md"), stub).expect("w");
        std::fs::write(dir.join("archive/other.md"), "record\n").expect("w");
        std::fs::write(dir.join("docs/t.md"), RECORD).expect("w");
        git_in(&dir, &["init", "-q"]);
        git_in(&dir, &["add", "-A"]);
        git_in(
            &dir,
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-qm",
                "base",
            ],
        );
        let top = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(&dir)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("git");
        assert_eq!(
            std::fs::canonicalize(String::from_utf8_lossy(&top.stdout).trim()).expect("top"),
            std::fs::canonicalize(&dir).expect("dir"),
            "the test repo must be its own repository"
        );
        if !target_exists {
            std::fs::remove_file(dir.join("docs/t.md")).expect("rm");
        }
        dir
    }

    #[test]
    fn replaced_stub_passes() {
        let d = stub_repo("replaced", false);
        std::fs::write(d.join("archive/stub.md"), RECORD).expect("w");
        assert!(part_b(&d, "HEAD").is_ok());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// A stub overwritten with anything but the record it pointed to is a
    /// modification, even when the record's live path is gone (review of #2801).
    #[test]
    fn stub_replaced_by_other_content_fails() {
        let d = stub_repo("other-content", false);
        std::fs::write(d.join("archive/stub.md"), "arbitrary\n").expect("w");
        assert!(part_b(&d, "HEAD").is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn modified_non_stub_fails() {
        let d = stub_repo("non-stub", false);
        std::fs::write(d.join("archive/other.md"), "edited\n").expect("w");
        assert!(part_b(&d, "HEAD").is_err());
    }

    #[test]
    fn stub_whose_target_still_exists_fails() {
        let d = stub_repo("target-lives", true);
        std::fs::write(d.join("archive/stub.md"), RECORD).expect("w");
        assert!(part_b(&d, "HEAD").is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn deleted_stub_fails() {
        let d = stub_repo("deleted", false);
        std::fs::remove_file(d.join("archive/stub.md")).expect("rm");
        assert!(part_b(&d, "HEAD").is_err());
    }

    #[test]
    fn near_stub_is_not_a_stub() {
        let exact = crate::docs_stubs::stub_text("docs/a.md", "../docs/a.md");
        assert_eq!(crate::docs_stubs::stub_target(&exact), Some("docs/a.md"));
        assert!(crate::docs_stubs::stub_target(&format!("{exact}extra\n")).is_none());
        let at_rev = |_: &str| Some("b1".to_string());
        assert!(stub_replaced(exact.as_bytes(), "b1", &|_| false, &at_rev));
        assert!(!stub_replaced(exact.as_bytes(), "b1", &|_| true, &at_rev));
        assert!(!stub_replaced(exact.as_bytes(), "b2", &|_| false, &at_rev));
        assert!(!stub_replaced(exact.as_bytes(), "b1", &|_| false, &|_| {
            None
        }));
        let near = format!("{exact}extra\n");
        assert!(!stub_replaced(near.as_bytes(), "b1", &|_| false, &at_rev));
    }

    // The archive catalog is append-only (#2982): catalog.jsonl may only gain
    // lines, and its pages may only equal docs-catalog's output.

    /// One catalog row on research line `l`.
    fn row(path: &str) -> String {
        format!(
            "{{\"path\":\"{path}\",\"title\":\"T {path}\",\"date\":\"2026-10-01\",\"lab\":\"claude\",\"line\":\"l\",\"mechanisms\":[\"m\"],\"outcome\":\"positive\",\"one_line\":\"x\",\"prs\":[1]}}\n"
        )
    }

    fn rendered(text: &str) -> Vec<(String, String)> {
        let recs = crate::docs_catalog::parse_records(text).expect("parse");
        crate::docs_catalog::render(&recs, &|_| true)
    }

    fn page<'a>(pages: &'a [(String, String)], path: &str) -> &'a str {
        &pages.iter().find(|(p, _)| p == path).expect("page").1
    }

    #[test]
    fn appended_row_passes() {
        let (a, b, c) = (
            row("archive/a.md"),
            row("archive/b.md"),
            row("archive/c.md"),
        );
        let base = format!("{a}{b}");
        assert_eq!(
            catalog_appended(base.as_bytes(), format!("{base}{c}").as_bytes()),
            Some(1)
        );
        assert_eq!(catalog_appended(b"", a.as_bytes()), Some(1));
        assert_eq!(
            catalog_change(
                CATALOG_DATA,
                base.as_bytes(),
                format!("{base}{c}{c}").as_bytes(),
                None
            ),
            Some(CatalogChange::Appended(2))
        );
    }

    #[test]
    fn rewritten_rows_are_not_appends() {
        let (a, b, c) = (
            row("archive/a.md"),
            row("archive/b.md"),
            row("archive/c.md"),
        );
        let base = format!("{a}{b}");
        let edited = format!("{a}{}{c}", row("archive/b2.md"));
        assert_eq!(
            catalog_appended(base.as_bytes(), edited.as_bytes()),
            None,
            "edited"
        );
        let removed = format!("{a}{c}{c}");
        assert_eq!(
            catalog_appended(base.as_bytes(), removed.as_bytes()),
            None,
            "removed"
        );
        assert_eq!(
            catalog_appended(base.as_bytes(), a.as_bytes()),
            None,
            "truncated"
        );
        let reordered = format!("{b}{a}{c}");
        assert_eq!(
            catalog_appended(base.as_bytes(), reordered.as_bytes()),
            None,
            "reordered"
        );
        assert_eq!(
            catalog_appended(base.as_bytes(), base.as_bytes()),
            None,
            "no growth"
        );
    }

    #[test]
    fn append_after_a_base_without_final_newline_fails() {
        let a = row("archive/a.md");
        let base = a.trim_end_matches('\n');
        let now = format!("{a}{}", row("archive/b.md"));
        assert_eq!(catalog_appended(base.as_bytes(), now.as_bytes()), None);
    }

    #[test]
    fn unparseable_append_fails() {
        let a = row("archive/a.md");
        let bad = format!("{a}not json\n");
        assert_eq!(catalog_appended(a.as_bytes(), bad.as_bytes()), None);
        let no_path = format!("{a}{{\"title\":\"x\"}}\n");
        assert_eq!(catalog_appended(a.as_bytes(), no_path.as_bytes()), None);
    }

    #[test]
    fn page_equal_to_render_passes_and_hand_edit_fails() {
        let text = format!("{}{}", row("archive/a.md"), row("archive/b.md"));
        let pages = rendered(&text);
        for path in ["archive/CATALOG.md", "archive/catalog/l.md"] {
            let good = page(&pages, path);
            assert!(
                catalog_page_regenerated(path, good.as_bytes(), &pages),
                "{path}"
            );
            assert_eq!(
                catalog_change(path, b"old", good.as_bytes(), Some(&pages)),
                Some(CatalogChange::Regenerated)
            );
            let hand = format!("{good}hand edit\n");
            assert!(
                !catalog_page_regenerated(path, hand.as_bytes(), &pages),
                "{path}"
            );
            assert_eq!(
                catalog_change(path, b"old", hand.as_bytes(), Some(&pages)),
                None
            );
            // An unparseable catalog renders nothing, so nothing passes.
            assert_eq!(catalog_change(path, b"old", good.as_bytes(), None), None);
        }
        // Another page's content at this path is still a hand edit.
        let index = page(&pages, "archive/CATALOG.md");
        assert!(!catalog_page_regenerated(
            "archive/catalog/l.md",
            index.as_bytes(),
            &pages
        ));
    }

    #[test]
    fn paths_render_does_not_produce_get_no_pass() {
        let text = row("archive/a.md");
        let mut pages = rendered(&text);
        let body = page(&pages, "archive/catalog/l.md").to_string();
        assert!(!catalog_page_regenerated(
            "archive/catalog/bogus.md",
            body.as_bytes(),
            &pages
        ));
        // Even a listed entry must have a catalog page's shape.
        pages.push(("archive/catalog/sub/l.md".to_string(), body.clone()));
        pages.push(("archive/other.md".to_string(), body.clone()));
        for p in ["archive/catalog/sub/l.md", "archive/other.md"] {
            assert!(!catalog_page_regenerated(p, body.as_bytes(), &pages), "{p}");
        }
        assert!(is_catalog_page("archive/CATALOG.md"));
        assert!(is_catalog_page("archive/catalog/l.md"));
        assert!(!is_catalog_page("archive/catalog.md"));
        assert!(!is_catalog_page("archive/x/CATALOG.md"));
    }

    #[test]
    fn lookalike_catalog_paths_get_no_pass() {
        let a = row("archive/a.md");
        let now = format!("{a}{}", row("archive/b.md"));
        let pages = rendered(&now);
        for p in [
            "archive/catalogue.jsonl",
            "archive/x/catalog.jsonl",
            "archive/catalog.jsonl.bak",
            "docs/catalog.jsonl",
            "archive/other.md",
        ] {
            assert_eq!(
                catalog_change(p, a.as_bytes(), now.as_bytes(), Some(&pages)),
                None,
                "{p}"
            );
        }
    }

    /// A throwaway repo under the gitignored `local/` whose base commit holds a
    /// rendered catalog of one row plus the record and a non-catalog file.
    fn catalog_repo(name: &str) -> std::path::PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../local/archive-verify-tests")
            .join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("archive/catalog")).expect("mkdir");
        std::fs::write(dir.join("archive/a.md"), "record a\n").expect("w");
        std::fs::write(dir.join("archive/b.md"), "record b\n").expect("w");
        std::fs::write(dir.join(CATALOG_DATA), row("archive/a.md")).expect("w");
        git_in(&dir, &["init", "-q"]);
        let mut top = Command::new("git");
        top.args(["rev-parse", "--show-toplevel"]).current_dir(&dir);
        for (k, _) in std::env::vars() {
            if k.starts_with("GIT_") {
                top.env_remove(k);
            }
        }
        let top = top.output().expect("git");
        assert_eq!(
            std::fs::canonicalize(String::from_utf8_lossy(&top.stdout).trim()).expect("top"),
            std::fs::canonicalize(&dir).expect("dir"),
            "the test repo must be its own repository"
        );
        git_in(&dir, &["add", "-A"]);
        write_pages(&dir);
        git_in(&dir, &["add", "-A"]);
        git_in(
            &dir,
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-qm",
                "base",
            ],
        );
        dir
    }

    /// Write the pages `docs-catalog` renders for the repo's catalog now.
    fn write_pages(dir: &Path) {
        let text = std::fs::read(dir.join(CATALOG_DATA)).expect("read");
        for (p, c) in render_catalog(dir, &text).expect("render") {
            std::fs::write(dir.join(p), c).expect("w");
        }
    }

    #[test]
    fn part_b_passes_an_append_with_regenerated_pages() {
        let d = catalog_repo("catalog-append");
        let text = format!("{}{}", row("archive/a.md"), row("archive/b.md"));
        std::fs::write(d.join(CATALOG_DATA), text).expect("w");
        write_pages(&d);
        assert!(part_b(&d, "HEAD").is_ok());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn part_b_fails_a_hand_edited_page_after_an_append() {
        let d = catalog_repo("catalog-hand-edit");
        let text = format!("{}{}", row("archive/a.md"), row("archive/b.md"));
        std::fs::write(d.join(CATALOG_DATA), text).expect("w");
        write_pages(&d);
        let index = d.join("archive/CATALOG.md");
        let mut body = std::fs::read_to_string(&index).expect("r");
        body.push_str("hand edit\n");
        std::fs::write(&index, body).expect("w");
        assert!(part_b(&d, "HEAD").is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn part_b_fails_an_edited_row_a_deleted_page_and_other_files() {
        let d = catalog_repo("catalog-edit");
        std::fs::write(d.join(CATALOG_DATA), row("archive/b.md")).expect("w");
        assert!(part_b(&d, "HEAD").is_err(), "edited row");
        let _ = std::fs::remove_dir_all(&d);

        let d = catalog_repo("catalog-page-deleted");
        std::fs::remove_file(d.join("archive/catalog/l.md")).expect("rm");
        assert!(part_b(&d, "HEAD").is_err(), "deleted page");
        let _ = std::fs::remove_dir_all(&d);

        let d = catalog_repo("catalog-other-file");
        std::fs::write(d.join("archive/a.md"), "edited\n").expect("w");
        assert!(part_b(&d, "HEAD").is_err(), "non-catalog file");
        let _ = std::fs::remove_dir_all(&d);
    }
}
