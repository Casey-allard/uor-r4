//! `uor-harness loop gates`: the gate runner, its ledger and its planted breaks (loop PR 4).
//!
//! **Why this exists.** The loop page's run rules say a gate that cannot run is REFUSED, later
//! gates read UNKNOWN after a red, the true exit code is kept, and a change to a gate's read files
//! during the run voids it. Before this module each check ran by hand and none of that held. The
//! gates are data in `docs/harness/gates.json`; every entry declares a break, a fault that must
//! turn the gate red with its own failure text, so a gate that has gone blind is caught.
//!
//! **Verdicts.** GREEN (exit 0), RED (non-zero exit), REFUSED (could not run, or its reads match
//! no file), UNKNOWN (not run after an earlier non-green result), VOID (a read file changed while
//! it ran). A break run is CAUGHT (green at HEAD, then red with the declared text), REFUSED
//! (already not green before the break, or red for another reason) or DEFECT (the gate stayed
//! green).
//!
//! **Working-tree hash.** `git hash-object --stdin` (git's content hash, so no new dependency) over
//! `HEAD`, `git status --porcelain=v1 -z --untracked-files=all`, `git diff HEAD --binary
//! --no-ext-diff --no-textconv --no-color`, and the path and bytes of every untracked file that is
//! not ignored, each section prefixed by its `u64` length and a missing file marked apart from an
//! empty one. A tracked edit, a staged edit or a new file each changes it; `local/` is ignored, so
//! the ledger and logs never do.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// `docs/harness/gates.json`.
#[derive(Debug, Deserialize)]
pub struct GateFile {
    pub version: u32,
    pub gates: Vec<Gate>,
}

/// One gate: what it proves, how it runs, what it reads, and its declared break.
#[derive(Debug, Clone, Deserialize)]
pub struct Gate {
    pub id: String,
    pub proves: String,
    /// argv, run from the repository root. A leading `cargo` resolves as [`resolve_program`] says.
    pub command: Vec<String>,
    /// Globs over repository paths (`*` within a segment, `**` across segments).
    pub reads: Vec<String>,
    pub quick: bool,
    #[serde(rename = "break")]
    pub brk: Break,
}

/// The planted fault: append text to `file`, or replace one exact text with another, in a scratch
/// worktree (repository-relative). The gate must then go red with `expect` somewhere in its output.
///
/// **A `replace` break exists so a fault can remove behaviour** - delete the branch whose absence the
/// gate should notice - instead of appending a case that can only make a test pass. Exactly one of
/// `append` and `replace` is given.
#[derive(Debug, Clone, Deserialize)]
pub struct Break {
    pub file: String,
    /// Text appended verbatim to the file.
    #[serde(default)]
    pub append: String,
    /// One literal replacement: `from` must occur exactly once, and is replaced by `to`.
    #[serde(default)]
    pub replace: Option<Replace>,
    pub expect: String,
}

/// One literal replacement for a `replace` break.
#[derive(Debug, Clone, Deserialize)]
pub struct Replace {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Verdict {
    Green,
    Red,
    Refused,
    Unknown,
    Void,
    Caught,
    Defect,
}

impl Verdict {
    pub fn word(self) -> &'static str {
        match self {
            Verdict::Green => "GREEN",
            Verdict::Red => "RED",
            Verdict::Refused => "REFUSED",
            Verdict::Unknown => "UNKNOWN",
            Verdict::Void => "VOID",
            Verdict::Caught => "CAUGHT",
            Verdict::Defect => "DEFECT",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub id: String,
    pub verdict: Verdict,
    pub exit: Option<i32>,
    /// The signal that killed the gate, when it did not exit.
    pub signal: Option<i32>,
    pub seconds: f64,
    pub reason: String,
}

/// One ledger line in `local/<lab>/gates.jsonl`.
#[derive(Debug, Serialize, Deserialize)]
pub struct LedgerRow {
    pub gate: String,
    pub verdict: Verdict,
    pub seconds: f64,
    pub commit: String,
    pub tree: String,
    pub exit: Option<i32>,
    /// Set (and `exit` null) when a signal killed the gate.
    #[serde(default)]
    pub signal: Option<i32>,
    pub ts: u64,
}

pub fn gates_path(root: &Path) -> PathBuf {
    root.join("docs/harness/gates.json")
}

/// Load and validate `gates.json`. Every entry must declare a complete break.
pub fn load(root: &Path) -> Result<GateFile, String> {
    let path = gates_path(root);
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let doc: GateFile =
        serde_json::from_str(&text).map_err(|e| format!("{} is malformed: {e}", path.display()))?;
    if doc.version != 1 {
        return Err(format!(
            "{}: version {} is not 1",
            path.display(),
            doc.version
        ));
    }
    if doc.gates.is_empty() {
        return Err(format!("{} lists no gates", path.display()));
    }
    let id_re = regex::Regex::new("^[a-z0-9][a-z0-9-]*$").expect("a valid pattern");
    let mut seen = std::collections::BTreeSet::new();
    for g in &doc.gates {
        if !id_re.is_match(&g.id) {
            return Err(format!(
                "gate id {:?} must match ^[a-z0-9][a-z0-9-]*$",
                g.id
            ));
        }
        if !seen.insert(g.id.clone()) {
            return Err(format!("gate {:?} is listed twice", g.id));
        }
        if g.command.is_empty() || g.reads.is_empty() || g.proves.trim().is_empty() {
            return Err(format!("gate {:?} needs proves, command and reads", g.id));
        }
        let b = &g.brk;
        let has_append = !b.append.is_empty();
        let has_replace = b.replace.is_some();
        if b.file.trim().is_empty() || b.expect.trim().is_empty() || has_append == has_replace {
            return Err(format!(
                "gate {:?} must declare a break: file, exactly one of append or replace, expect",
                g.id
            ));
        }
        if let Some(r) = &b.replace {
            if r.from.is_empty() {
                return Err(format!(
                    "gate {:?}: break.replace.from must not be empty",
                    g.id
                ));
            }
        }
        if !contained_rel(&b.file) {
            return Err(format!(
                "gate {:?}: break.file {:?} must be a relative path with no `.`, `..` or root part",
                g.id, b.file
            ));
        }
    }
    Ok(doc)
}

/// A repository-relative path that cannot leave the tree: not absolute, and no `.`, `..`, root
/// or prefix part.
fn contained_rel(file: &str) -> bool {
    let p = Path::new(file);
    !p.is_absolute()
        && !file.starts_with('/')
        && !file.split('/').any(|s| s == "." || s == "..")
        && p.components()
            .all(|c| matches!(c, std::path::Component::Normal(_)))
}

/// A leading `cargo` is `$CARGO` when set (it is under `cargo run`), else `~/.cargo/bin/cargo`
/// when that file exists (on this project's Mac a Homebrew `cargo` shadows it on `PATH`), else
/// `cargo` from `PATH`. Any other program is looked up on `PATH` as given.
pub fn resolve_program(argv0: &str) -> PathBuf {
    if argv0 == "cargo" {
        if let Some(c) = std::env::var_os("CARGO") {
            return PathBuf::from(c);
        }
        if let Some(home) = std::env::var_os("HOME") {
            let p = PathBuf::from(home).join(".cargo/bin/cargo");
            if p.is_file() {
                return p;
            }
        }
    }
    PathBuf::from(argv0)
}

pub(crate) fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
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

fn split_z(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect()
}

fn glob_regex(glob: &str) -> regex::Regex {
    let mut re = String::from("^");
    let mut chars = glob.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    chars.next();
                    re.push_str("(?:.*/)?");
                } else {
                    re.push_str(".*");
                }
            }
            '*' => re.push_str("[^/]*"),
            '?' => re.push_str("[^/]"),
            c => re.push_str(&regex::escape(&c.to_string())),
        }
    }
    re.push('$');
    regex::Regex::new(&re).expect("an escaped glob is a valid regex")
}

/// Append one hashed section: its length as a little-endian `u64`, then its bytes, so that no two
/// sequences of sections can run together into the same input.
fn section(h: &mut Vec<u8>, bytes: &[u8]) {
    h.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    h.extend_from_slice(bytes);
}

/// Append a file's path and contents. A file that cannot be read (listed but deleted) is marked
/// absent with a `0` byte; a present file is a `1` byte and then its contents, so a missing file
/// and an empty one hash differently.
fn file_section(h: &mut Vec<u8>, root: &Path, rel: &str) {
    section(h, rel.as_bytes());
    match fs::read(root.join(rel)) {
        Ok(bytes) => {
            h.push(1);
            section(h, &bytes);
        }
        Err(_) => h.push(0),
    }
}

/// Content hash of every tracked or untracked-unignored file matching one of `globs`, and how
/// many files matched.
pub fn reads_hash(root: &Path, globs: &[String]) -> Result<(String, usize), String> {
    let res: Vec<regex::Regex> = globs.iter().map(|g| glob_regex(g)).collect();
    let mut files = split_z(&git(
        root,
        &["ls-files", "-z", "-co", "--exclude-standard"],
    )?);
    files.sort();
    files.dedup();
    let mut h: Vec<u8> = Vec::new();
    let mut matched = 0;
    for f in files.iter().filter(|f| res.iter().any(|r| r.is_match(f))) {
        file_section(&mut h, root, f);
        matched += 1;
    }
    Ok((git_hash(root, &h)?, matched))
}

/// The working-tree hash recorded in the ledger (see the module comment).
pub fn tree_hash(root: &Path) -> Result<String, String> {
    let mut h: Vec<u8> = Vec::new();
    section(&mut h, &git(root, &["rev-parse", "HEAD"])?);
    section(
        &mut h,
        &git(
            root,
            &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        )?,
    );
    section(
        &mut h,
        &git(
            root,
            &[
                "diff",
                "HEAD",
                "--binary",
                "--no-ext-diff",
                "--no-textconv",
                "--no-color",
            ],
        )?,
    );
    for f in split_z(&git(root, &["ls-files", "-z", "-o", "--exclude-standard"])?) {
        file_section(&mut h, root, &f);
    }
    git_hash(root, &h)
}

/// The hash [`tree_hash`] records for a clean working tree at HEAD: no status lines, an empty diff
/// and no untracked files. `loop report --receipts` keeps only ledger rows with this tree.
pub(crate) fn clean_head_tree_hash(root: &Path) -> Result<String, String> {
    let mut h: Vec<u8> = Vec::new();
    section(&mut h, &git(root, &["rev-parse", "HEAD"])?);
    section(&mut h, b"");
    section(&mut h, b"");
    git_hash(root, &h)
}

fn git_hash(root: &Path, bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("git")
        .args(["hash-object", "--stdin"])
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git hash-object: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(bytes)
            .map_err(|e| format!("git hash-object: {e}"))?;
    }
    let out = child
        .wait_with_output()
        .map_err(|e| format!("git hash-object: {e}"))?;
    if !out.status.success() {
        return Err("git hash-object failed".into());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Run one gate in `root`, logging to `<logdir>/<id>.log` and `<id>.exit`.
pub fn run_one(root: &Path, gate: &Gate, logdir: &Path) -> Outcome {
    run_logged(root, gate, logdir, &gate.id)
}

/// [`run_one`], logging to `<logdir>/<name>.log` and `<name>.exit`.
fn run_logged(root: &Path, gate: &Gate, logdir: &Path, name: &str) -> Outcome {
    let start = Instant::now();
    let mut signal = None;
    let done = |verdict, exit, signal, reason: String| Outcome {
        id: gate.id.clone(),
        verdict,
        exit,
        signal,
        seconds: start.elapsed().as_secs_f64(),
        reason,
    };
    let before = match reads_hash(root, &gate.reads) {
        Ok((_, 0)) => return done(Verdict::Refused, None, None, "reads match no file".into()),
        Ok((h, _)) => h,
        Err(e) => {
            return done(
                Verdict::Refused,
                None,
                None,
                format!("cannot hash reads: {e}"),
            )
        }
    };
    let log_path = logdir.join(format!("{name}.log"));
    let log = match fs::File::create(&log_path) {
        Ok(f) => f,
        Err(e) => {
            return done(
                Verdict::Refused,
                None,
                None,
                format!("cannot open log: {e}"),
            )
        }
    };
    let err = match log.try_clone() {
        Ok(f) => f,
        Err(e) => {
            return done(
                Verdict::Refused,
                None,
                None,
                format!("cannot open log: {e}"),
            )
        }
    };
    let status = Command::new(resolve_program(&gate.command[0]))
        .args(&gate.command[1..])
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(err)
        .status();
    let status = match status {
        Ok(s) => s,
        Err(e) => {
            return done(
                Verdict::Refused,
                None,
                None,
                format!("cannot run {:?}: {e}", gate.command[0]),
            )
        }
    };
    let code = status.code();
    if code.is_none() {
        signal = std::os::unix::process::ExitStatusExt::signal(&status);
    }
    let exit_text = match (code, signal) {
        (Some(c), _) => c.to_string(),
        (None, Some(s)) => format!("signal {s}"),
        (None, None) => "signal".to_string(),
    };
    let _ = fs::write(logdir.join(format!("{name}.exit")), &exit_text);
    match reads_hash(root, &gate.reads) {
        Ok((after, _)) if after == before => {}
        Ok(_) => {
            return done(
                Verdict::Void,
                code,
                signal,
                "a read file changed during the run".into(),
            )
        }
        Err(e) => {
            return done(
                Verdict::Refused,
                code,
                signal,
                format!("cannot hash reads: {e}"),
            )
        }
    }
    match code {
        Some(0) => done(Verdict::Green, code, None, String::new()),
        Some(c) => done(Verdict::Red, code, None, format!("exit {c}")),
        None => done(Verdict::Red, None, signal, format!("killed by {exit_text}")),
    }
}

/// Run `gates` in order. After the first non-green verdict the rest are UNKNOWN unless
/// `keep_going`.
pub fn run_gates(root: &Path, gates: &[Gate], keep_going: bool, logdir: &Path) -> Vec<Outcome> {
    run_gates_with(root, gates, keep_going, logdir, &mut |_| {})
}

/// One step of a run, for the progress window: a gate is about to start, or it just finished.
pub enum Progress<'a> {
    Start { index: usize, id: &'a str },
    Finished { index: usize, outcome: &'a Outcome },
}

/// [`run_gates`] with a progress callback. The callback only observes; it never changes a verdict.
pub fn run_gates_with(
    root: &Path,
    gates: &[Gate],
    keep_going: bool,
    logdir: &Path,
    on_progress: &mut dyn FnMut(Progress<'_>),
) -> Vec<Outcome> {
    let mut out = Vec::new();
    let mut stopped = false;
    for (index, g) in gates.iter().enumerate() {
        if stopped {
            out.push(Outcome {
                id: g.id.clone(),
                verdict: Verdict::Unknown,
                exit: None,
                signal: None,
                seconds: 0.0,
                reason: "not run after an earlier non-green result".into(),
            });
            continue;
        }
        on_progress(Progress::Start { index, id: &g.id });
        let o = run_one(root, g, logdir);
        if o.verdict != Verdict::Green && !keep_going {
            stopped = true;
        }
        on_progress(Progress::Finished { index, outcome: &o });
        out.push(o);
    }
    out
}

/// Removes the scratch worktree on drop, so an early return or a panic cannot leave it behind.
struct Scratch<'a> {
    root: &'a Path,
    path: PathBuf,
}

impl Drop for Scratch<'_> {
    fn drop(&mut self) {
        let p = self.path.to_string_lossy().into_owned();
        let _ = git(self.root, &["worktree", "remove", "--force", &p]);
        if self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

/// Plant `gate`'s break in a detached scratch worktree of HEAD under `scratch_dir` and run it.
pub fn run_break(root: &Path, gate: &Gate, scratch_dir: &Path, logdir: &Path) -> Outcome {
    let refused = |reason: String| Outcome {
        id: gate.id.clone(),
        verdict: Verdict::Refused,
        exit: None,
        signal: None,
        seconds: 0.0,
        reason,
    };
    let path = scratch_dir.join(format!("wt-{}", gate.id));
    if let Err(e) = fs::create_dir_all(scratch_dir) {
        return refused(format!("cannot create {}: {e}", scratch_dir.display()));
    }
    let p = path.to_string_lossy().into_owned();
    if let Err(e) = git(
        root,
        &["worktree", "add", "--detach", "--quiet", &p, "HEAD"],
    ) {
        return refused(e);
    }
    let _guard = Scratch {
        root,
        path: path.clone(),
    };
    // The gate must be green at HEAD, or a red after planting proves nothing about the fault. Its
    // log is `<id>.pre.log`, so the break run's own log does not overwrite that evidence.
    let pre = run_logged(&path, gate, logdir, &format!("{}.pre", gate.id));
    if pre.verdict != Verdict::Green {
        let mut o = refused(format!(
            "gate not green before the break ({}: {})",
            pre.verdict.word(),
            pre.reason
        ));
        o.exit = pre.exit;
        o.signal = pre.signal;
        o.seconds = pre.seconds;
        return o;
    }
    // Containment is checked here, after the pre-check and immediately before the write, because
    // the gate's own run may have changed the scratch tree.
    let target = path.join(&gate.brk.file);
    let inside = match (
        target.parent().map(fs::canonicalize),
        fs::canonicalize(&path),
    ) {
        (Some(Ok(parent)), Ok(wt)) => parent.starts_with(&wt),
        _ => false,
    };
    if !inside {
        return refused(format!(
            "break.file {} resolves outside the scratch worktree",
            gate.brk.file
        ));
    }
    if fs::symlink_metadata(&target).is_ok_and(|m| m.file_type().is_symlink()) {
        return refused(format!("break.file {} is a symlink", gate.brk.file));
    }
    let planted = if let Some(r) = &gate.brk.replace {
        match fs::read_to_string(&target) {
            Ok(text) => {
                let count = text.matches(r.from.as_str()).count();
                if count != 1 {
                    return refused(format!(
                        "break.replace.from occurs {count} times in {}; it must occur exactly once",
                        gate.brk.file
                    ));
                }
                fs::write(&target, text.replacen(r.from.as_str(), &r.to, 1))
            }
            Err(e) => Err(e),
        }
    } else {
        fs::OpenOptions::new()
            .append(true)
            .open(&target)
            .and_then(|mut f| f.write_all(gate.brk.append.as_bytes()))
    };
    if let Err(e) = planted {
        return refused(format!("cannot plant the break in {}: {e}", gate.brk.file));
    }
    let mut o = run_one(&path, gate, logdir);
    let log = fs::read_to_string(logdir.join(format!("{}.log", gate.id))).unwrap_or_default();
    match o.verdict {
        Verdict::Red if log.contains(&gate.brk.expect) => {
            o.verdict = Verdict::Caught;
            o.reason = format!("red with {:?}", gate.brk.expect);
        }
        Verdict::Red => {
            o.verdict = Verdict::Refused;
            o.reason = format!("red without {:?}: red for another reason", gate.brk.expect);
        }
        Verdict::Green => {
            o.verdict = Verdict::Defect;
            o.reason = "the gate stayed green with its break planted: add the case".into();
        }
        _ => {}
    }
    o
}

/// Whether `pid` is certainly not a live process: `kill -0 <pid>` fails with "No such process".
/// Anything else (live, owned by another user, out of range, `kill` not runnable) counts as live.
fn pid_is_dead(pid: u32) -> bool {
    if pid == 0 || i32::try_from(pid).is_err() {
        return false;
    }
    match Command::new("kill")
        .args(["-0", &pid.to_string()])
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(o) => {
            !o.status.success() && String::from_utf8_lossy(&o.stderr).contains("No such process")
        }
        Err(_) => false,
    }
}

/// Remove scratch worktrees an interrupted earlier run of this lab left registered. An entry is
/// removed, by its own path, only when all three hold: its path is exactly
/// `<root>/local/<lab_slug>/gates/<ts>-<pid>/scratch/wt-<id>` for this lab, `<pid>` is not a
/// live process, and `<pid>` is not this process. Every other lab's and every live run's worktree
/// is left alone, and there is never a global prune.
pub fn sweep_stale_scratch(root: &Path, lab_slug: &str) {
    let Ok(list) = git(root, &["worktree", "list", "--porcelain"]) else {
        return;
    };
    let pat = format!(
        "^{}/local/{}/gates/[0-9]+-([0-9]+)/scratch/wt-[a-z0-9][a-z0-9-]*$",
        regex::escape(&root.to_string_lossy()),
        regex::escape(lab_slug)
    );
    let re = regex::Regex::new(&pat).expect("escaped paths make a valid regex");
    for line in String::from_utf8_lossy(&list).lines() {
        let Some(p) = line.strip_prefix("worktree ") else {
            continue;
        };
        let Some(pid) = re.captures(p).and_then(|c| c[1].parse::<u32>().ok()) else {
            continue;
        };
        if pid == std::process::id() || !pid_is_dead(pid) {
            continue;
        }
        let _ = git(root, &["worktree", "remove", "--force", p]);
    }
}

fn append_ledger(path: &Path, rows: &[LedgerRow]) -> Result<(), String> {
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    // One write_all per row, newline included, so concurrent runs cannot interleave a row.
    for r in rows {
        let mut line = serde_json::to_string(r).map_err(|e| e.to_string())?;
        line.push('\n');
        f.write_all(line.as_bytes())
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    Ok(())
}

/// The latest row per gate, each with whether it is STALE against `current_tree`.
pub fn status_rows(ledger: &Path, current_tree: &str) -> Vec<(LedgerRow, bool)> {
    let text = fs::read_to_string(ledger).unwrap_or_default();
    let mut latest: BTreeMap<String, LedgerRow> = BTreeMap::new();
    for line in text.lines() {
        if let Ok(r) = serde_json::from_str::<LedgerRow>(line) {
            latest.insert(r.gate.clone(), r);
        }
    }
    latest
        .into_values()
        .map(|r| {
            let stale = r.tree != current_tree;
            (r, stale)
        })
        .collect()
}

const USAGE: &str = "usage: uor-harness loop gates [--quick|--full] [--keep-going] [--status] \
     [--watch] [--break G|all]";

/// `loop <action> ...`. Returns the process exit code: 0 all green or caught, 1 otherwise,
/// 2 for a usage error or a refusal before any gate ran.
pub fn cli(root: &Path, lab: &str, args: &[String]) -> u8 {
    // The top-level parser forwards `--repo <path>` ahead of the action; drop it and `--lab`.
    let mut args: Vec<String> = args.to_vec();
    while let Some(p) = args.iter().position(|a| a == "--repo" || a == "--lab") {
        args.drain(p..(p + 2).min(args.len()));
    }
    match args.first().map(String::as_str) {
        Some("gates") => {}
        Some("arrive") => return crate::looparrive::cli(root, lab, &args[1..]),
        Some("board") if args.iter().any(|a| a == "--bench") => {
            return crate::loopbench::cli(root, &args[1..])
        }
        Some("board") => return crate::loopboard::cli(root, &args[1..]),
        Some("claim") => return crate::loopclaim::cli(root, lab, &args[1..]),
        Some("close") => return crate::loopclose::cli(root, lab, &args[1..]),
        Some("next") => return crate::loopnext::cli(root, lab, &args[1..]),
        Some("review") => return crate::loopreview::cli(root, lab, &args[1..]),
        Some(a @ ("suborder" | "ask")) => return crate::loopsub::cli(root, lab, a, &args[1..]),
        Some("render") => return crate::looprender::cli(root, &args[1..]),
        Some("pr-check") => return crate::loopprcheck::cli(root, &args[1..]),
        Some("queue") => return crate::loopqueue::cli(root, lab, &args[1..]),
        Some("report") => return crate::loopreport::cli(root, lab, &args[1..]),
        Some(other) => {
            eprintln!("uor-harness: loop {other}: not built yet (see the build plan)");
            return 2;
        }
        None => {
            eprintln!("{USAGE}");
            return 2;
        }
    }
    let (mut quick, mut keep_going, mut status, mut watch, mut brk) =
        (false, false, false, false, None);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--full" => quick = false,
            "--keep-going" => keep_going = true,
            "--status" => status = true,
            "--watch" => watch = true,
            "--break" => match args.get(i + 1) {
                Some(g) => {
                    brk = Some(g.clone());
                    i += 1;
                }
                None => {
                    eprintln!("{USAGE}");
                    return 2;
                }
            },
            other => {
                eprintln!("uor-harness: loop gates: unknown argument {other:?}\n{USAGE}");
                return 2;
            }
        }
        i += 1;
    }
    if lab.trim().is_empty() {
        eprintln!("uor-harness: loop gates needs a lab: set UOR_LAB or pass --lab");
        return 2;
    }
    if watch && (quick || status || brk.is_some()) {
        eprintln!(
            "uor-harness: loop gates --watch needs a full run: drop --quick, --status and --break"
        );
        return 2;
    }
    let decision = watch_decision(watch, !quick, at_terminal(), in_ci());
    if watch {
        if let WatchDecision::Off(why) = decision {
            eprintln!("uor-harness: loop gates: watch off ({why})");
        }
    }
    let lab_dir = root.join("local").join(crate::state::slug(lab));
    let ledger = lab_dir.join("gates.jsonl");
    if status {
        let tree = match tree_hash(root) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("uor-harness: {e}");
                return 2;
            }
        };
        let rows = status_rows(&ledger, &tree);
        if rows.is_empty() {
            println!("no gate results in {}", ledger.display());
        }
        for (r, stale) in rows {
            let short = r.commit.get(..12).unwrap_or(&r.commit);
            let mark = if stale { "  STALE" } else { "" };
            println!(
                "{:<22} {:<8} {:>8.1}s  {short}{mark}",
                r.gate,
                r.verdict.word(),
                r.seconds
            );
        }
        return 0;
    }
    let file = match load(root) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return 2;
        }
    };
    // Every refusal below comes before the run folder exists, so a refusal leaves no empty folder.
    let run_id = format!("{}-{}", now(), std::process::id());
    let logdir = lab_dir.join("gates").join(&run_id);
    let make_logdir = || match fs::create_dir_all(&logdir) {
        Ok(()) => true,
        Err(e) => {
            eprintln!("uor-harness: cannot create {}: {e}", logdir.display());
            false
        }
    };
    let outcomes = if let Some(which) = brk {
        let chosen: Vec<&Gate> = file
            .gates
            .iter()
            .filter(|g| which == "all" || g.id == which)
            .collect();
        if chosen.is_empty() {
            eprintln!("uor-harness: no gate {which:?} in gates.json");
            return 2;
        }
        // Breaks run in a scratch worktree of HEAD, so they use the committed gates.json.
        let rel = "docs/harness/gates.json";
        let differs = git(root, &["diff", "HEAD", "--quiet", "--", rel]).is_err();
        let untracked = git(root, &["ls-files", "--error-unmatch", "--", rel]).is_err();
        if differs || untracked {
            eprintln!(
                "uor-harness: {rel} is uncommitted; --break runs against the committed tree, so commit it first"
            );
            return 2;
        }
        if !make_logdir() {
            return 2;
        }
        sweep_stale_scratch(root, &crate::state::slug(lab));
        let scratch = logdir.join("scratch");
        let out: Vec<Outcome> = chosen
            .into_iter()
            .map(|g| run_break(root, g, &scratch, &logdir))
            .collect();
        let _ = fs::remove_dir(&scratch);
        out
    } else {
        let chosen: Vec<Gate> = file
            .gates
            .into_iter()
            .filter(|g| !quick || g.quick)
            .collect();
        if chosen.is_empty() {
            eprintln!("uor-harness: the selection runs no gate (--quick matches none)");
            return 2;
        }
        let (commit, tree) = match (git(root, &["rev-parse", "HEAD"]), tree_hash(root)) {
            (Ok(c), Ok(t)) => (String::from_utf8_lossy(&c).trim().to_string(), t),
            (Err(e), _) | (_, Err(e)) => {
                eprintln!("uor-harness: {e}");
                return 2;
            }
        };
        if !make_logdir() {
            return 2;
        }
        let mut writer = None;
        let mut watcher = None;
        if matches!(decision, WatchDecision::Open) {
            let path = logdir.join("status.json");
            let last = last_seconds(&ledger);
            writer = Some(WatchWriter::start(path.clone(), &run_id, lab, &chosen));
            watcher = Some(thread::spawn(move || watch_draw(path, last)));
        }
        let out = match writer.as_mut() {
            Some(w) => run_gates_with(root, &chosen, keep_going, &logdir, &mut |step| {
                w.progress(step)
            }),
            None => run_gates(root, &chosen, keep_going, &logdir),
        };
        if let Some(mut w) = writer {
            w.done();
        }
        if let Some(handle) = watcher {
            let _ = handle.join();
        }
        let ts = now();
        let rows: Vec<LedgerRow> = out
            .iter()
            .map(|o| LedgerRow {
                gate: o.id.clone(),
                verdict: o.verdict,
                seconds: o.seconds,
                commit: commit.clone(),
                tree: tree.clone(),
                exit: o.exit,
                signal: o.signal,
                ts,
            })
            .collect();
        if let Err(e) = append_ledger(&ledger, &rows) {
            eprintln!("uor-harness: {e}");
            return 2;
        }
        out
    };
    let mut ok = true;
    for o in &outcomes {
        ok &= matches!(o.verdict, Verdict::Green | Verdict::Caught);
        let exit = o.exit.map_or("-".to_string(), |c| c.to_string());
        println!(
            "{:<22} {:<8} exit {exit:<4} {:>7.1}s  {}",
            o.id,
            o.verdict.word(),
            o.seconds,
            o.reason
        );
    }
    println!("logs: {}", logdir.display());
    if ok {
        0
    } else {
        1
    }
}

// --- The progress window (loop PR 11) ---------------------------------------------------------

/// How often the watcher reads the status file.
pub const WATCH_POLL: Duration = Duration::from_millis(500);
/// The window gives up after five minutes without a change to the status file.
pub const WATCH_GIVE_UP: Duration = Duration::from_secs(300);

/// Whether the progress window opens, and when it does not, why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchDecision {
    Open,
    Off(&'static str),
}

/// The window opens only for a full run in a terminal, never in CI and never in a headless run.
pub fn watch_decision(asked: bool, full_run: bool, terminal: bool, ci: bool) -> WatchDecision {
    if !asked {
        return WatchDecision::Off("not asked");
    }
    if !full_run {
        return WatchDecision::Off("not a full run");
    }
    if ci {
        return WatchDecision::Off("CI");
    }
    if !terminal {
        return WatchDecision::Off("not a terminal");
    }
    WatchDecision::Open
}

/// Whether this process is at a terminal: both stdin and stdout, so a piped or headless run is not.
pub fn at_terminal() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// Whether the environment says CI; the window never opens there.
pub fn in_ci() -> bool {
    std::env::var_os("CI").is_some()
}

/// One finished gate in the status file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchResult {
    pub id: String,
    pub verdict: Verdict,
    pub seconds: f64,
}

/// The runner's status file, written after every gate: the window reads it, and nothing waits for
/// a reader.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WatchStatus {
    pub run: String,
    pub lab: String,
    pub gates: Vec<String>,
    /// The gate that is running now, if one is.
    pub current: Option<usize>,
    /// The gates that finished, in order.
    pub results: Vec<WatchResult>,
    pub done: bool,
}

impl WatchStatus {
    pub fn new(run: impl Into<String>, lab: impl Into<String>, gates: Vec<String>) -> Self {
        WatchStatus {
            run: run.into(),
            lab: lab.into(),
            gates,
            current: None,
            results: Vec::new(),
            done: false,
        }
    }
}

/// The window's one-line status: gate N of M, the gate running now, the reds so far, and the
/// finish estimate.
pub fn watch_line(status: &WatchStatus, estimate: Option<f64>) -> String {
    let total = status.gates.len();
    let finished = status.results.len();
    let n = match status.current {
        Some(i) => i + 1,
        None => finished,
    }
    .clamp(1, total.max(1));
    let mut line = format!("gate {n} of {total}");
    if let Some(id) = status.current.and_then(|i| status.gates.get(i)) {
        line.push_str(&format!(": {id}"));
    }
    let reds: Vec<&str> = status
        .results
        .iter()
        .filter(|r| r.verdict == Verdict::Red)
        .map(|r| r.id.as_str())
        .collect();
    if !reds.is_empty() {
        line.push_str(&format!(" | RED: {}", reds.join(", ")));
    }
    match estimate {
        Some(secs) => line.push_str(&format!(" | estimate {secs:.1}s left")),
        None => line.push_str(" | no estimate yet"),
    }
    line
}

/// The latest seconds per gate in a ledger, for the finish estimate.
///
/// **UNKNOWN rows are skipped**: they are the rows written for gates not run after an earlier red,
/// and their `seconds` is always `0.0`. Letting one overwrite a real timing would make the next
/// estimate read "0.0s left" precisely after an early stop, when the estimate is most useful.
pub fn last_seconds(ledger: &Path) -> BTreeMap<String, f64> {
    let mut last = BTreeMap::new();
    for line in fs::read_to_string(ledger).unwrap_or_default().lines() {
        if let Ok(r) = serde_json::from_str::<LedgerRow>(line) {
            if r.verdict == Verdict::Unknown {
                continue;
            }
            last.insert(r.gate, r.seconds);
        }
    }
    last
}

/// The finish estimate: the sum of the remaining gates' latest ledger seconds. None when no
/// remaining gate has a ledger row, so a first run says there is no estimate yet.
pub fn estimate_left(status: &WatchStatus, last: &BTreeMap<String, f64>) -> Option<f64> {
    let from = status.results.len().min(status.gates.len());
    let mut sum = 0.0;
    let mut known = false;
    for id in &status.gates[from..] {
        if let Some(secs) = last.get(id) {
            sum += secs;
            known = true;
        }
    }
    known.then_some(sum)
}

/// Whether the window must give up: five minutes without a change to the status file.
pub fn watch_gives_up(idle: Duration) -> bool {
    idle >= WATCH_GIVE_UP
}

/// Writes the status file after every gate, atomically.
struct WatchWriter {
    path: PathBuf,
    status: WatchStatus,
}

impl WatchWriter {
    fn start(path: PathBuf, run: &str, lab: &str, gates: &[Gate]) -> WatchWriter {
        let mut w = WatchWriter {
            path,
            status: WatchStatus::new(run, lab, gates.iter().map(|g| g.id.clone()).collect()),
        };
        w.write();
        w
    }

    fn write(&mut self) {
        let tmp = self.path.with_extension("json.tmp");
        if let Ok(bytes) = serde_json::to_vec(&self.status) {
            if fs::write(&tmp, bytes).is_ok() {
                let _ = fs::rename(&tmp, &self.path);
            }
        }
    }

    fn progress(&mut self, step: Progress<'_>) {
        match step {
            Progress::Start { index, .. } => self.status.current = Some(index),
            Progress::Finished { outcome, .. } => {
                self.status.current = None;
                self.status.results.push(WatchResult {
                    id: outcome.id.clone(),
                    verdict: outcome.verdict,
                    seconds: outcome.seconds,
                });
            }
        }
        self.write();
    }

    fn done(&mut self) {
        self.status.current = None;
        self.status.done = true;
        self.write();
    }
}

/// The window loop, split from the file and the clock so it can be tested with a fake reader, a
/// fake wait and a short give-up. It writes one line per change and returns when the run is done or
/// when the status file has not changed for `give_up`; it never touches the run.
pub fn watch_loop(
    mut read: impl FnMut() -> String,
    out: &mut dyn Write,
    last: &BTreeMap<String, f64>,
    give_up: Duration,
    mut wait: impl FnMut(),
) -> std::io::Result<()> {
    let mut seen = String::new();
    let mut changed_at = Instant::now();
    let mut announced: HashSet<String> = HashSet::new();
    let mut opened = false;
    loop {
        wait();
        let text = read();
        if !text.is_empty() && text != seen {
            seen = text;
            changed_at = Instant::now();
            let Ok(status) = serde_json::from_str::<WatchStatus>(&seen) else {
                continue;
            };
            if !opened {
                opened = true;
                writeln!(out, "watch | opened: {} gate(s)", status.gates.len())?;
            }
            for r in &status.results {
                if r.verdict == Verdict::Red && announced.insert(r.id.clone()) {
                    writeln!(out, "watch | RED {} after {:.1}s", r.id, r.seconds)?;
                }
            }
            if status.done {
                writeln!(out, "watch | {}", watch_line(&status, None))?;
                writeln!(out, "watch | closed after the run")?;
                return Ok(());
            }
            writeln!(
                out,
                "watch | {}",
                watch_line(&status, estimate_left(&status, last))
            )?;
        } else if changed_at.elapsed() >= give_up {
            // The give-up is timed here, not by `watch_gives_up`, so a test can inject its own
            // threshold; `WATCH_GIVE_UP` is the five-minute value production uses.
            writeln!(out, "watch | giving up after 5 minutes without change")?;
            return Ok(());
        }
    }
}

/// Draws the window on stderr: the real reader, the real poll and the five-minute give-up.
fn watch_draw(path: PathBuf, last: BTreeMap<String, f64>) {
    let mut read = move || fs::read_to_string(&path).unwrap_or_default();
    let stderr = std::io::stderr();
    let mut out = stderr.lock();
    let _ = watch_loop(&mut read, &mut out, &last, WATCH_GIVE_UP, || {
        thread::sleep(WATCH_POLL)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_and_an_empty_file_hash_differently() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("empty"), b"").unwrap();
        let (mut present, mut missing) = (Vec::new(), Vec::new());
        file_section(&mut present, tmp.path(), "empty");
        // Same name, but no such file in this directory.
        let other = tempfile::tempdir().unwrap();
        file_section(&mut missing, other.path(), "empty");
        assert_ne!(present, missing);
    }

    #[test]
    fn sections_cannot_run_together() {
        let (mut a, mut b) = (Vec::new(), Vec::new());
        section(&mut a, b"ab");
        section(&mut a, b"c");
        section(&mut b, b"a");
        section(&mut b, b"bc");
        assert_ne!(a, b);
    }
}
