//! `uor-harness loop report` (loop PR 7b, ORDER call 3).
//!
//! **`--check`** reads `<task folder>/REPORT.md` from the working tree and fails when
//! [`crate::loopprcheck::report_problems`] finds a problem, when a `<!-- fill:` template line is
//! left, or when the file is missing. It is a session-time check, so uncommitted text counts.
//!
//! **`--new`** writes `REPORT.md` from `docs/loop/templates/REPORT.md`, filling in the ORDER's item
//! id, task folder and lab. It never overwrites: an existing REPORT exits 2.
//!
//! **`--receipts`** copies this head's gate results from `local/<lab>/gates.jsonl` into
//! `<task folder>/receipts.jsonl`: the latest line per gate whose `commit` is `HEAD`, sorted by gate
//! name, each line exactly as `loop gates` recorded it.
//!
//! The task folder is the one holding the single `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md` that
//! this branch adds against `git merge-base origin/main HEAD`; `--order <path>` names it instead.
//! Exit codes: 0 ok, 1 a check failed, 2 cannot run.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str =
    "usage: uor-harness loop report (--check | --new | --receipts) [--order <path>]";
const TEMPLATE: &str = "docs/loop/templates/REPORT.md";
const FILL: &str = "<!-- fill:";

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Check,
    New,
    Receipts,
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    // A caller's GIT_DIR, GIT_WORK_TREE or GIT_INDEX_FILE would point git at another repository.
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(k);
        }
    }
    let out = cmd
        .arg("-C")
        .arg(root)
        .args(["-c", "core.quotepath=false"])
        .args(args)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The ORDER this branch adds (status `A`, or the new side of an `R`), relative to `root`.
pub(crate) fn find_order(root: &Path) -> Result<String, String> {
    let mb = git(root, &["merge-base", "origin/main", "HEAD"])?;
    let mb = mb.trim();
    let out = git(
        root,
        &[
            "diff",
            "-z",
            "--name-status",
            "-M",
            "--diff-filter=AR",
            mb,
            "HEAD",
        ],
    )?;
    let pattern = regex::Regex::new(r"^docs/loop/tasks/[^/]+/[^/]+-[0-9]+/ORDER\.md$")
        .expect("a valid pattern");
    let mut fields = out.split('\0').filter(|f| !f.is_empty());
    let mut found = Vec::new();
    while let Some(status) = fields.next() {
        let path = if status.starts_with('R') {
            fields.next();
            fields.next()
        } else {
            fields.next()
        };
        if let Some(p) = path.filter(|p| pattern.is_match(p)) {
            found.push(p.to_string());
        }
    }
    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(format!(
            "this branch adds no docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md against {mb}; pass --order <path>"
        )),
        _ => Err(format!(
            "this branch adds {} ORDERs ({}); pass --order <path>",
            found.len(),
            found.join(", ")
        )),
    }
}

/// Each problem `--check` reports for this REPORT text; empty when it passes.
fn check_problems(text: &str) -> Vec<String> {
    let mut out = crate::loopprcheck::report_problems(text);
    let mut section: Option<String> = None;
    for (line, can_head) in crate::loopprcheck::heading_lines(text) {
        if let Some(h) = line.strip_prefix("## ").filter(|_| can_head) {
            section = Some(h.trim().to_string());
        } else if can_head && line.trim_start().starts_with(FILL) {
            // A template prompt is a line of its own; prose that names the marker is not one.
            out.push(match &section {
                Some(s) => format!("a `{FILL}` template line is left in `## {s}`"),
                None => format!("a `{FILL}` template line is left before any `## ` heading"),
            });
        }
    }
    out
}

/// Each problem `--check` reports for `<folder>/REPORT.md` in the working tree (a missing file is
/// one); empty when it passes. `shown` is the folder as printed. `loop close` runs the same check.
pub(crate) fn report_file_problems(folder: &Path, shown: &str) -> Vec<String> {
    match fs::read_to_string(folder.join("REPORT.md")) {
        Ok(text) => check_problems(&text),
        Err(_) => vec![format!(
            "{shown}/REPORT.md is missing; `loop report --new` writes one"
        )],
    }
}

fn run_check(folder: &Path, shown: &str) -> u8 {
    let problems = report_file_problems(folder, shown);
    if problems.is_empty() {
        println!("PASS report {shown}/REPORT.md");
        return 0;
    }
    println!("FAIL report {shown}/REPORT.md");
    for p in problems {
        println!("  {p}");
    }
    1
}

fn run_new(root: &Path, order: &Path, folder: &Path, shown: &str) -> u8 {
    let template = match fs::read_to_string(root.join(TEMPLATE)) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("uor-harness: cannot read {TEMPLATE}: {e}");
            return 2;
        }
    };
    let order_text = match fs::read_to_string(order) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("uor-harness: cannot read {}: {e}", order.display());
            return 2;
        }
    };
    let item = regex::Regex::new(r"Item: \[([^\]\s]+)\]").expect("a valid pattern");
    let Some(id) = item.captures(&order_text).map(|c| c[1].to_string()) else {
        eprintln!("uor-harness: {} names no `Item: [<id>]`", order.display());
        return 2;
    };
    let task = folder
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    // `<lab>-<n>`: the lab is everything before the last `-`.
    let lab = task.rsplit_once('-').map(|(l, _)| l).unwrap_or(&task);
    // The template's first line says which command uses it; the REPORT does not keep it.
    let body = match template.split_once('\n') {
        Some((first, rest)) if first.trim_start().starts_with("<!--") && !first.contains(FILL) => {
            rest
        }
        _ => &template,
    };
    let text = body
        .replace("{id}", &id)
        .replace("{task}", &task)
        .replace("{lab}", lab);
    let path = folder.join("REPORT.md");
    let mut file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            eprintln!("uor-harness: {shown}/REPORT.md exists; refusing to overwrite it");
            return 2;
        }
        Err(e) => {
            eprintln!("uor-harness: cannot create {}: {e}", path.display());
            return 2;
        }
    };
    if let Err(e) = file.write_all(text.as_bytes()) {
        eprintln!("uor-harness: cannot write {}: {e}", path.display());
        return 2;
    }
    println!("wrote {shown}/REPORT.md from {TEMPLATE}");
    0
}

fn run_receipts(root: &Path, lab: &str, folder: &Path, shown: &str) -> u8 {
    if lab.trim().is_empty() {
        eprintln!("uor-harness: loop report --receipts needs a lab: set UOR_LAB or pass --lab");
        return 2;
    }
    let head = match git(root, &["rev-parse", "HEAD"]) {
        Ok(h) => h.trim().to_string(),
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return 2;
        }
    };
    let clean = match crate::loopgates::clean_head_tree_hash(root) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("uor-harness: {e}");
            return 2;
        }
    };
    let ledger = root
        .join("local")
        .join(crate::state::slug(lab))
        .join("gates.jsonl");
    let text = fs::read_to_string(&ledger).unwrap_or_default();
    // Later lines are later runs, so inserting in file order keeps the latest per gate.
    let mut latest: BTreeMap<String, &str> = BTreeMap::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let (Some(gate), Some(commit)) = (v["gate"].as_str(), v["commit"].as_str()) else {
            continue;
        };
        // A row from a dirty tree at HEAD is not a receipt for HEAD.
        if commit == head && v["tree"].as_str() == Some(clean.as_str()) {
            latest.insert(gate.to_string(), line);
        }
    }
    if latest.is_empty() {
        println!(
            "FAIL receipts: no gate results at HEAD ({head}) on a clean tree in {}; run `loop gates` first",
            ledger.display()
        );
        return 1;
    }
    let mut out = String::new();
    for line in latest.values() {
        out.push_str(line);
        out.push('\n');
    }
    let path = folder.join("receipts.jsonl");
    let tmp = folder.join(format!(".receipts.jsonl.{}.tmp", std::process::id()));
    if let Err(e) = fs::write(&tmp, out).and_then(|_| fs::rename(&tmp, &path)) {
        let _ = fs::remove_file(&tmp);
        eprintln!("uor-harness: cannot write {}: {e}", path.display());
        return 2;
    }
    println!(
        "wrote {shown}/receipts.jsonl: {} gate result(s) at {head}",
        latest.len()
    );
    0
}

/// `loop report ...`; `args` follow the action name.
pub fn cli(root: &Path, lab: &str, args: &[String]) -> u8 {
    let (mut mode, mut order) = (None, None);
    let mut i = 0;
    while i < args.len() {
        let m = match args[i].as_str() {
            "--check" => Mode::Check,
            "--new" => Mode::New,
            "--receipts" => Mode::Receipts,
            "--order" => match args.get(i + 1) {
                Some(p) => {
                    order = Some(p.clone());
                    i += 2;
                    continue;
                }
                None => {
                    eprintln!("{USAGE}");
                    return 2;
                }
            },
            other => {
                eprintln!("uor-harness: loop report: unknown argument {other:?}\n{USAGE}");
                return 2;
            }
        };
        if mode.is_some_and(|prev| prev != m) {
            eprintln!("uor-harness: loop report: give one of --check, --new, --receipts\n{USAGE}");
            return 2;
        }
        mode = Some(m);
        i += 1;
    }
    let Some(mode) = mode else {
        eprintln!("{USAGE}");
        return 2;
    };
    let order: PathBuf = match order {
        Some(p) => {
            let p = PathBuf::from(p);
            if p.is_absolute() {
                p
            } else {
                root.join(p)
            }
        }
        None => match find_order(root) {
            Ok(rel) => root.join(rel),
            Err(e) => {
                eprintln!("uor-harness: {e}");
                return 2;
            }
        },
    };
    if !order.is_file() {
        eprintln!("uor-harness: no ORDER at {}", order.display());
        return 2;
    }
    let folder = order.parent().unwrap_or(root).to_path_buf();
    let shown = folder
        .strip_prefix(root)
        .unwrap_or(&folder)
        .display()
        .to_string();
    match mode {
        Mode::Check => run_check(&folder, &shown),
        Mode::New => run_new(root, &order, &folder, &shown),
        Mode::Receipts => run_receipts(root, lab, &folder, &shown),
    }
}
