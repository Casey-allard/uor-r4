//! `uor-harness loop pr-check`: the rules a loop PR must pass, read from git alone.
//!
//! The PR is the diff from `git merge-base <base> <head>` to `<head>`. Registries, items and
//! budgets are read at the merge base, never at the head, so a PR cannot loosen its own check;
//! the files the PR adds or changes are read at the head. Nothing reads the working tree; only
//! `review-record` and `required-checks` read GitHub, through the read-only [`PrGh`] and only
//! with `--pr <n>`. A rule is one function plus one entry in [`RULES`].

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

const USAGE: &str =
    "usage: uor-harness loop pr-check [--base <rev>] [--head <rev>] [--head-ref <branch>] [--pr <n> | --local | --ci] [--json]";

/// Every rule, in the order it runs and prints.
const RULES: &[(&str, Rule)] = &[
    ("order", rule_order),
    ("scope", rule_scope),
    ("banned-presence", rule_banned_presence),
    ("commit-order", rule_commit_order),
    ("report", rule_report),
    ("owner-paths", rule_owner_paths),
    ("rule-sources", rule_rule_sources),
    ("rendered-lock", rule_rendered_lock),
    ("read-path", rule_read_path),
    ("filemap", rule_filemap),
    ("review-record", rule_review_record),
    ("required-checks", rule_required_checks),
];

type Rule = fn(&Pr) -> RuleResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    Warn,
    NotApplicable,
}

impl Verdict {
    pub fn text(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Fail => "FAIL",
            Verdict::Warn => "WARN",
            Verdict::NotApplicable => "N/A",
        }
    }
}

#[derive(Debug)]
pub struct RuleResult {
    pub rule: &'static str,
    pub verdict: Verdict,
    pub summary: String,
    pub details: Vec<String>,
}

/// One changed file: its `--name-status` letter and its paths (old then new for a rename or copy).
#[derive(Debug)]
pub struct Change {
    pub status: char,
    pub paths: Vec<String>,
}

/// The PR under check: the merge base, the head and what changed between them.
pub struct Pr<'a> {
    root: &'a Path,
    pub merge_base: String,
    pub head: String,
    pub changes: Vec<Change>,
    /// The branch the head is on (`--head-ref`), `""` when unknown; only `rendered-lock` reads it.
    pub head_ref: String,
    /// Whether the PR head comes from a fork; `None` when the source did not say. The render
    /// sentinel path requires `Some(false)`.
    pub is_cross_repository: Option<bool>,
    /// How `review-record` and `required-checks` reach GitHub; [`Github::Missing`] until set.
    pub github: Github,
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
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
    Ok(out.stdout)
}

fn resolve(root: &Path, rev: &str) -> Result<String, String> {
    let spec = format!("{rev}^{{commit}}");
    let out = git(root, &["rev-parse", "--verify", "--quiet", &spec])
        .map_err(|_| format!("{rev:?} does not name a commit"))?;
    Ok(String::from_utf8_lossy(&out).trim().to_string())
}

fn z_fields(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect()
}

/// `--name-status -z` fields as changes: a rename or copy carries two paths.
fn name_status(fields: &[String]) -> Result<Vec<Change>, String> {
    let mut changes = Vec::new();
    let mut i = 0;
    while i < fields.len() {
        let status = fields[i].chars().next().unwrap_or('?');
        let n = if matches!(status, 'R' | 'C') { 2 } else { 1 };
        let paths = fields
            .get(i + 1..i + 1 + n)
            .ok_or("git --name-status output is truncated")?
            .to_vec();
        changes.push(Change { status, paths });
        i += 1 + n;
    }
    Ok(changes)
}

impl<'a> Pr<'a> {
    pub fn open(root: &'a Path, base: &str, head: &str) -> Result<Pr<'a>, String> {
        let base_sha = resolve(root, base)?;
        let head = resolve(root, head)?;
        let mb = git(root, &["merge-base", &base_sha, &head])?;
        let merge_base = String::from_utf8_lossy(&mb).trim().to_string();
        let raw = git(
            root,
            &[
                "diff",
                "--ignore-submodules=none",
                "-z",
                "--name-status",
                "-M",
                &merge_base,
                &head,
            ],
        )?;
        let changes = name_status(&z_fields(&raw))?;
        Ok(Pr {
            root,
            merge_base,
            head,
            changes,
            head_ref: String::new(),
            is_cross_repository: None,
            github: Github::Missing,
        })
    }

    /// A file's text at the merge base, `None` when it is not there.
    pub fn at_base(&self, path: &str) -> Option<String> {
        self.show(&self.merge_base, path)
    }

    /// A file's text at the head, `None` when it is not there.
    pub fn at_head(&self, path: &str) -> Option<String> {
        self.show(&self.head, path)
    }

    fn show(&self, rev: &str, path: &str) -> Option<String> {
        git(self.root, &["show", &format!("{rev}:{path}")])
            .ok()
            .map(|b| String::from_utf8_lossy(&b).into_owned())
    }

    /// Every path the PR touches, both sides of a rename or copy.
    pub fn changed_paths(&self) -> impl Iterator<Item = (&Change, &str)> {
        self.changes
            .iter()
            .flat_map(|c| c.paths.iter().map(move |p| (c, p.as_str())))
    }

    /// The ORDER files the PR adds (an add, or the new side of a rename or copy).
    pub fn added_orders(&self) -> Vec<&str> {
        self.changes
            .iter()
            .filter(|c| matches!(c.status, 'A' | 'R' | 'C'))
            .filter_map(|c| c.paths.last().map(String::as_str))
            .filter(|p| order_parts(p).is_some())
            .collect()
    }

    /// The ORDERs the PR adds as one repair chain, oldest first (see [`order_chain`]).
    fn the_chain(&self) -> Result<Vec<String>, String> {
        order_chain(&self.added_orders()).map_err(|e| format!("the PR adds {e}"))
    }

    /// The active ORDER the PR adds (see [`active_order`]), or why there is none.
    fn the_order(&self) -> Result<&str, String> {
        let added = self.added_orders();
        let active = active_order(&added).map_err(|e| format!("the PR adds {e}"))?;
        Ok(added
            .into_iter()
            .find(|p| *p == active)
            .expect("the active ORDER is one of the added ORDERs"))
    }
}

/// `(folder, lab, n)` of `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md`.
pub(crate) fn order_parts(path: &str) -> Option<(String, String, String)> {
    let re = regex::Regex::new(r"^docs/loop/tasks/([^/]+)/([A-Za-z0-9_-]+)-([0-9]+)/ORDER\.md$")
        .expect("a valid pattern");
    let c = re.captures(path)?;
    Some((c[1].to_string(), c[2].to_string(), c[3].to_string()))
}

/// The ORDERs a PR adds, oldest first, when they are one lab's repair chain in one task
/// folder (loop step 11): every path is `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md` with
/// the same `<folder>` and `<lab>`, ordered by `n` read as a number (`kimi-10` after
/// `kimi-9`). The last is the active ORDER; the rest are history. The error names the
/// count, so a caller can prefix who adds them ("the PR adds ...").
pub fn order_chain<S: AsRef<str>>(paths: &[S]) -> Result<Vec<String>, String> {
    let all: Vec<&str> = paths.iter().map(AsRef::as_ref).collect();
    if all.is_empty() {
        return Err("no docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md".into());
    }
    let mut chain = Vec::new();
    for p in &all {
        let parsed = order_parts(p).and_then(|(f, l, n)| Some((f, l, n.parse::<u64>().ok()?)));
        let Some((folder, lab, n)) = parsed else {
            return Err(format!(
                "{p}, which is not docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md"
            ));
        };
        chain.push((folder, lab, n, p.to_string()));
    }
    let set = |i: fn(&(String, String, u64, String)) -> &String| {
        let mut v: Vec<&str> = chain.iter().map(|c| i(c).as_str()).collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    let (folders, labs) = (set(|c| &c.0), set(|c| &c.1));
    if folders.len() > 1 || labs.len() > 1 {
        return Err(format!(
            "{} ORDERs that are not one lab's repairs in one task folder (folders: {}; labs: {}): {}",
            all.len(),
            folders.join(", "),
            labs.join(", "),
            all.join(", ")
        ));
    }
    chain.sort_by_key(|c| c.2);
    if chain.windows(2).any(|w| w[0].2 == w[1].2) {
        return Err(format!(
            "{} ORDERs, two with the same number: {}",
            all.len(),
            all.join(", ")
        ));
    }
    Ok(chain.into_iter().map(|c| c.3).collect())
}

/// The active ORDER of a PR's added ORDERs: the highest `n` of [`order_chain`].
pub fn active_order<S: AsRef<str>>(paths: &[S]) -> Result<String, String> {
    order_chain(paths).map(|mut c| c.pop().expect("a chain is never empty"))
}

fn result(rule: &'static str, details: Vec<String>, pass: &str, fail: &str) -> RuleResult {
    let verdict = if details.is_empty() {
        Verdict::Pass
    } else {
        Verdict::Fail
    };
    let summary = if details.is_empty() { pass } else { fail };
    RuleResult {
        rule,
        verdict,
        summary: summary.to_string(),
        details,
    }
}

fn fail(rule: &'static str, summary: String) -> RuleResult {
    RuleResult {
        rule,
        verdict: Verdict::Fail,
        summary,
        details: Vec::new(),
    }
}

/// The labs registered in `docs/harness/labs.json` at the merge base.
fn base_labs(pr: &Pr) -> Result<Vec<String>, String> {
    let text = pr
        .at_base("docs/harness/labs.json")
        .ok_or("docs/harness/labs.json is missing at the merge base")?;
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("docs/harness/labs.json at the merge base: {e}"))?;
    let labs = v
        .get("labs")
        .and_then(|l| l.as_object())
        .ok_or("docs/harness/labs.json at the merge base has no `labs` object")?;
    Ok(labs.keys().cloned().collect())
}

/// The lab an ORDER's `Lab:` line names (its value on the same line).
fn order_lab(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"\bLab:[ \t]*([A-Za-z0-9_-]+)").expect("a valid pattern");
    re.captures(text).map(|c| c[1].to_string())
}

/// The PR adds exactly one ORDER, by a registered lab, naming an item that exists at the base.
fn rule_order(pr: &Pr) -> RuleResult {
    const R: &str = "order";
    let path = match pr.the_order() {
        Ok(p) => p,
        Err(e) => return fail(R, e),
    };
    let (_, lab, _) = order_parts(path).expect("an added ORDER path parses");
    let Some(text) = pr.at_head(path) else {
        return fail(R, format!("{path} cannot be read at the head"));
    };
    let mut details = Vec::new();
    match base_labs(pr) {
        Ok(labs) if labs.contains(&lab) => {}
        Ok(_) => details.push(format!(
            "lab {lab} (from the folder) is not registered in docs/harness/labs.json at the merge base"
        )),
        Err(e) => details.push(e),
    }
    match order_lab(&text) {
        Some(l) if l == lab => {}
        Some(l) => details.push(format!(
            "the ORDER says `Lab: {l}` but its folder is lab {lab}"
        )),
        None => details.push("the ORDER has no `Lab: <lab>` line".into()),
    }
    match crate::loopclaim::order_item(&text) {
        None => {
            details.push("the ORDER names no item (`Item: [..](roadmap/items/<id>.md)`)".into())
        }
        Some(id) => {
            if pr.at_base(&format!("docs/roadmap/items/{id}.md")).is_none() {
                details.push(format!(
                    "item {id} does not exist at the merge base (docs/roadmap/items/{id}.md)"
                ));
            }
            let setup_re = regex::Regex::new(r"Setup row:[ \t]*\S").expect("a valid pattern");
            if id.starts_with("infra-loop-pr") && !setup_re.is_match(&text) {
                details.push(format!(
                    "item {id} is a loop setup PR, so the ORDER needs a `Setup row: <x>` line"
                ));
            }
        }
    }
    match pr.the_chain() {
        Ok(chain) => details.extend(chain_identity_problems(pr, &chain, path, &text)),
        Err(e) => details.push(e),
    }
    result(R, details, path, &format!("{path} is not a valid ORDER"))
}

/// What differs across a repair chain: every earlier ORDER's `Lab:` matches its folder's lab,
/// and its `Lab:`, `Item:` and `Setup row:` at the head equal the active ORDER's, so a repair
/// ORDER can never re-point the PR at another item, lab or setup row.
fn chain_identity_problems(pr: &Pr, chain: &[String], active: &str, text: &str) -> Vec<String> {
    let identity = |t: &str| {
        [
            ("Lab:", order_lab(t)),
            ("Item:", crate::loopclaim::order_item(t)),
            ("Setup row:", crate::loopreview::setup_row(t)),
        ]
    };
    let want = identity(text);
    let mut details = Vec::new();
    for order in chain.iter().map(String::as_str).filter(|p| *p != active) {
        let (_, lab, _) = order_parts(order).expect("a chain ORDER path parses");
        let Some(t) = pr.at_head(order) else {
            details.push(format!("{order} cannot be read at the head"));
            continue;
        };
        let got = identity(&t);
        if got[0].1.as_deref() != Some(lab.as_str()) {
            details.push(format!(
                "{order} says `Lab: {}` but its folder is lab {lab}",
                got[0].1.as_deref().unwrap_or("(none)")
            ));
        }
        for ((field, g), (_, w)) in got.iter().zip(&want) {
            if g != w {
                let show = |v: &Option<String>| v.clone().unwrap_or_else(|| "(none)".into());
                details.push(format!(
                    "{order} has `{field} {}` but the active ORDER {active} has `{field} {}`",
                    show(g),
                    show(w)
                ));
            }
        }
    }
    details
}

/// Every changed path lies inside the ORDER item's `scope:` (read at the base) or is always allowed.
fn rule_scope(pr: &Pr) -> RuleResult {
    const R: &str = "scope";
    let order = match pr.the_order() {
        Ok(p) => p,
        Err(e) => return fail(R, format!("cannot check scope: {e}")),
    };
    let Some(order_text) = pr.at_head(order) else {
        return fail(
            R,
            format!("cannot check scope: {order} cannot be read at the head"),
        );
    };
    let Some(id) = crate::loopclaim::order_item(&order_text) else {
        return fail(R, format!("cannot check scope: {order} names no item"));
    };
    let item_path = format!("docs/roadmap/items/{id}.md");
    let Some(item_text) = pr.at_base(&item_path) else {
        return fail(
            R,
            format!("cannot check scope: item {id} does not exist at the merge base"),
        );
    };
    let item = match crate::loopclaim::parse_item(&item_text) {
        Ok(i) => i,
        Err(e) => {
            return fail(
                R,
                format!("cannot check scope: item {id} at the merge base {e}"),
            )
        }
    };
    let Some(scope) = item.scope else {
        return fail(R, format!("item {id} has no scope: at the base"));
    };
    let bad_globs: Vec<String> = scope
        .iter()
        .filter_map(|g| crate::loopclaim::scope_glob_ok(g).err())
        .collect();
    if !bad_globs.is_empty() {
        return RuleResult {
            rule: R,
            verdict: Verdict::Fail,
            summary: format!("item {id} has an invalid scope glob at the base"),
            details: bad_globs,
        };
    }
    let new_notebooks: Vec<String> = pr
        .changed_paths()
        .filter(|(_, path)| path.starts_with("docs/roadmap/items/") && path.ends_with(".md"))
        .filter_map(|(_, path)| {
            let base_notebook = crate::loopclaim::parse_item(&pr.at_base(path)?)
                .ok()?
                .notebook;
            if base_notebook.is_some() {
                return None;
            }
            crate::loopclaim::parse_item(&pr.at_head(path)?)
                .ok()?
                .notebook
        })
        .collect();
    // The active ORDER's folder and its history ORDERs' folders (loop step 11).
    let folders: Vec<String> = pr
        .the_chain()
        .unwrap_or_default()
        .iter()
        .map(|o| o.trim_end_matches("ORDER.md").to_string())
        .collect();
    let mut details = Vec::new();
    for (change, path) in pr.changed_paths() {
        let allowed = scope.iter().any(|g| glob_match(g, path))
            || crate::loopclaim::SHARED_FILES.contains(&path)
            || folders.iter().any(|f| path.starts_with(f.as_str()))
            || (path == item_path && status_only_edit(pr, path))
            || (item.notebook.as_deref() == Some(path) && keeps_base_lines(pr, path))
            || (change.status == 'A'
                && new_notebooks.iter().any(|notebook| notebook == path)
                && keeps_base_lines(pr, path))
            || (change.status == 'A' && path.starts_with("docs/harness/mistakes/"))
            || (path == "docs/start/FILEMAP.md" && keeps_base_lines(pr, path));
        if !allowed {
            details.push(format!("{path} is outside item {id}'s scope"));
        }
    }
    details.dedup();
    let n = pr.changed_paths().count();
    result(
        R,
        details,
        &format!(
            "{n} changed path(s) inside item {id}'s scope [{}]",
            scope.join(", ")
        ),
        &format!("changes outside item {id}'s scope [{}]", scope.join(", ")),
    )
}

/// The PR's commits, oldest first (`rev-list --reverse --topo-order`), each with whether it is a
/// merge (more than one parent).
fn pr_commits(pr: &Pr) -> Result<Vec<(String, bool)>, String> {
    let range = format!("{}..{}", pr.merge_base, pr.head);
    let out = git(
        pr.root,
        &["rev-list", "--reverse", "--topo-order", "--parents", &range],
    )?;
    Ok(String::from_utf8_lossy(&out)
        .lines()
        .filter_map(|l| {
            let mut words = l.split_whitespace();
            let sha = words.next()?.to_string();
            Some((sha, words.count() > 1))
        })
        .collect())
}

/// The `(Lab:, Item:)` an ORDER text names.
fn order_identity(text: &str) -> (Option<String>, Option<String>) {
    (order_lab(text), crate::loopclaim::order_item(text))
}

/// What one non-merge commit changes against its parent.
fn commit_changes(pr: &Pr, sha: &str) -> Result<Vec<Change>, String> {
    let parent = format!("{sha}^");
    let raw = git(
        pr.root,
        &[
            "diff-tree",
            "--ignore-submodules=none",
            "--no-commit-id",
            "-r",
            "-M",
            "--name-status",
            "-z",
            &parent,
            sha,
        ],
    )?;
    name_status(&z_fields(&raw))
}

/// The source suffixes the D23/D25/D32 ratchets scan.
const BANNED_SUFFIXES: [&str; 4] = ["rs", "cu", "cuh", "wgsl"];

fn is_banned_source(path: &str) -> bool {
    let Some((_, ext)) = path.rsplit_once('.') else {
        return false;
    };
    BANNED_SUFFIXES.contains(&ext)
}

/// Whether a byte can be part of a Rust identifier or a numeric literal suffix.
fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Copy `slice` into `out` as spaces, keeping line breaks so line numbers hold.
fn blank_into(out: &mut String, slice: &str) {
    for c in slice.chars() {
        out.push(if c == '\n' { '\n' } else { ' ' });
    }
}

/// The byte offset just past the raw string starting at the `r` at `start`, `None` when no raw
/// string starts there.
fn raw_string_end(b: &[u8], start: usize) -> Option<usize> {
    if b.get(start) != Some(&b'r') {
        return None;
    }
    let mut j = start + 1;
    let mut hashes = 0usize;
    while j < b.len() && b[j] == b'#' {
        hashes += 1;
        j += 1;
    }
    if b.get(j) != Some(&b'"') {
        return None;
    }
    let mut k = j + 1;
    while k + 1 + hashes <= b.len() {
        if b[k] == b'"' && b[k + 1..k + 1 + hashes].iter().all(|&x| x == b'#') {
            return Some(k + 1 + hashes);
        }
        k += 1;
    }
    Some(b.len())
}

/// The byte offset just past a raw string (`r"..."`, `r#"..."#`, `br"..."`, `cr#"..."#`) starting
/// at `i`, `None` if none starts there or the prefix is part of a longer identifier.
fn raw_prefix_end(b: &[u8], i: usize) -> Option<usize> {
    if i > 0 && is_word_byte(b[i - 1]) {
        return None;
    }
    match b[i] {
        b'r' => raw_string_end(b, i),
        b'b' | b'c' if b.get(i + 1) == Some(&b'r') => raw_string_end(b, i + 1),
        _ => None,
    }
}

/// The byte offset just past a character literal starting at `i`, `None` for a lifetime or a
/// numeric separator.
fn char_literal_end(b: &[u8], i: usize) -> Option<usize> {
    if b.get(i) != Some(&b'\'') {
        return None;
    }
    if i > 0 {
        let prev = b[i - 1];
        if is_word_byte(prev) {
            let is_prefix =
                matches!(prev, b'b' | b'u' | b'U' | b'L') && (i < 2 || !is_word_byte(b[i - 2]));
            if !is_prefix {
                return None;
            }
        }
    }
    let next = *b.get(i + 1)?;
    if next == b'\'' || next == b'\n' || next == b'\r' {
        return None;
    }
    if next == b'\\' {
        let mut j = i + 2;
        while j < b.len() && b[j] != b'\n' && b[j] != b'\r' && j - i <= 14 {
            if b[j] == b'\\' {
                j += 2;
                continue;
            }
            if b[j] == b'\'' {
                return Some(j + 1);
            }
            j += 1;
        }
        return None;
    }
    let tail = std::str::from_utf8(&b[i + 1..]).ok()?;
    let ch = tail.chars().next()?;
    let close = i + 1 + ch.len_utf8();
    (b.get(close) == Some(&b'\'')).then_some(close + 1)
}

/// `text` with comments, plain strings, character literals and raw strings replaced by spaces,
/// line breaks kept. Raw strings are blanked here (unlike the ratchet's site strip) because this
/// view exists to find structure: a brace or a gate inside a raw string is text, not structure.
fn strip_structure(text: &str) -> String {
    let b = text.as_bytes();
    let n = b.len();
    let mut out = String::with_capacity(n);
    let mut copied = 0usize;
    let mut i = 0usize;
    while i < n {
        let c = b[i];
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            out.push_str(&text[copied..i]);
            let mut j = i + 2;
            while j < n && b[j] != b'\n' {
                j += 1;
            }
            blank_into(&mut out, &text[i..j]);
            i = j;
            copied = i;
        } else if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            out.push_str(&text[copied..i]);
            let mut depth = 0usize;
            let mut j = i;
            while j < n {
                if j + 1 < n && b[j] == b'/' && b[j + 1] == b'*' {
                    depth += 1;
                    j += 2;
                } else if j + 1 < n && b[j] == b'*' && b[j + 1] == b'/' {
                    depth = depth.saturating_sub(1);
                    j += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    j += 1;
                }
            }
            let end = j.min(n);
            blank_into(&mut out, &text[i..end]);
            i = end;
            copied = i;
        } else if let Some(end) = raw_prefix_end(b, i) {
            out.push_str(&text[copied..i]);
            blank_into(&mut out, &text[i..end]);
            i = end;
            copied = i;
        } else if c == b'"' {
            out.push_str(&text[copied..i]);
            let mut j = i + 1;
            while j < n {
                if b[j] == b'\\' {
                    j += 1;
                    if j < n {
                        j += text[j..].chars().next().map_or(1, char::len_utf8);
                    }
                    continue;
                }
                if b[j] == b'"' {
                    j += 1;
                    break;
                }
                j += 1;
            }
            let end = j.min(n);
            blank_into(&mut out, &text[i..end]);
            i = end;
            copied = i;
        } else if c == b'\'' {
            if let Some(end) = char_literal_end(b, i) {
                out.push_str(&text[copied..i]);
                blank_into(&mut out, &text[i..end]);
                i = end;
                copied = i;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    out.push_str(&text[copied..n]);
    out
}

/// Skip one attribute (`#[...]` or `#![...]`) at the start of `s`.
fn skip_attribute(s: &str) -> Option<&str> {
    let rest = s.strip_prefix("#![").or_else(|| s.strip_prefix("#["))?;
    let mut depth = 1i64;
    for (i, c) in rest.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&rest[i + 1..]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Skip the balanced `open`..`close` group starting at the first character of `s`.
fn skip_balanced(s: &str, open: char, close: char) -> Option<&str> {
    let mut depth = 0i64;
    for (i, c) in s.char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return Some(&s[i + c.len_utf8()..]);
            }
        }
    }
    None
}

/// The next identifier-like token at the start of `s`, and the text after it.
fn next_word(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    let end = s
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(s.len());
    (&s[..end], &s[end..])
}

/// The item keyword a line starts (after attributes, visibility and modifiers), and the byte
/// offset just past that keyword. This is the real item-start parser: it skips balanced
/// `pub(...)` visibility (including `pub(in path)`), attributes, and the modifiers that may
/// precede an item, so `use`, `extern crate`, `extern "C" { ... }`, `pub(crate) fn` and their
/// kind all read as items.
fn item_prefix(line: &str) -> Option<(&'static str, usize)> {
    let mut idx = 0usize;
    loop {
        let rest = &line[idx..];
        let s = rest.trim_start();
        idx += rest.len() - s.len();
        if s.is_empty() {
            return None;
        }
        if s.starts_with("#[") {
            let after = skip_attribute(s)?;
            idx += s.len() - after.len();
            continue;
        }
        let (token, after) = next_word(s);
        if token.is_empty() {
            return None;
        }
        idx += s.len() - after.len();
        match token {
            "pub" => {
                let rest = &line[idx..];
                let t = rest.trim_start();
                if t.starts_with('(') {
                    let after = skip_balanced(t, '(', ')')?;
                    idx += rest.len() - after.len();
                }
            }
            "default" | "async" | "unsafe" | "auto" => {}
            "macro_rules" => {
                let rest = &line[idx..];
                if rest.trim_start().starts_with('!') {
                    return Some(("macro_rules!", idx));
                }
                return None;
            }
            "use" => return Some(("use", idx)),
            "struct" => return Some(("struct", idx)),
            "enum" => return Some(("enum", idx)),
            "fn" => return Some(("fn", idx)),
            "mod" => return Some(("mod", idx)),
            "trait" => return Some(("trait", idx)),
            "union" => return Some(("union", idx)),
            "impl" => return Some(("impl", idx)),
            "type" => return Some(("type", idx)),
            "static" => return Some(("static", idx)),
            "const" => return Some(("const", idx)),
            "extern" => return Some(("extern", idx)),
            _ => return None,
        }
    }
}

/// Whether a structure-view line starts an item.
fn is_item_line(line: &str) -> bool {
    item_prefix(line).is_some()
}

/// The index of the line through the item starting at `start`: through its closing brace, or
/// through its semicolon when it declares without a body. The structure view has comments,
/// strings and raw strings blanked, so braces and semicolons in text cannot confuse the count.
fn item_end(structure: &[&str], start: usize) -> usize {
    let mut depth: i64 = 0;
    let mut opened = false;
    for (i, line) in structure.iter().enumerate().skip(start) {
        for c in line.chars() {
            match c {
                '{' => {
                    depth += 1;
                    opened = true;
                }
                '}' => depth -= 1,
                _ => {}
            }
        }
        if opened && depth <= 0 {
            return i;
        }
        if !opened && line.contains(';') {
            return i;
        }
    }
    structure.len().saturating_sub(1)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GateKind {
    Outer,
    Inner,
}

/// The cfg(any()) gate a structure-view line starts, if any. Whitespace is ignored, so
/// `#[cfg( any ( ) )] // note` matches; comments and strings are already blanked, so text that
/// merely looks like a gate is not one.
fn gate_kind(line: &str) -> Option<GateKind> {
    let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.starts_with("#![cfg(any())]") {
        Some(GateKind::Inner)
    } else if compact.starts_with("#[cfg(any())]") {
        Some(GateKind::Outer)
    } else {
        None
    }
}

/// The brace depth before each structure-view line.
fn line_depths(structure: &[&str]) -> Vec<i64> {
    let mut depths = Vec::with_capacity(structure.len());
    let mut depth = 0i64;
    for line in structure {
        depths.push(depth);
        for c in line.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        depth = depth.max(0);
    }
    depths
}

/// The last line of the block that encloses the inner gate on line `i`, or the last line of the
/// file when the gate is a crate-root inner attribute.
fn inner_gate_end(structure: &[&str], i: usize, depths: &[i64]) -> usize {
    let base = depths[i];
    if base == 0 {
        return structure.len().saturating_sub(1);
    }
    let mut depth = base;
    for (k, line) in structure.iter().enumerate().skip(i) {
        for c in line.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        if depth < base {
            return k;
        }
    }
    structure.len().saturating_sub(1)
}

/// The lines that start items in the structure view, in order, so a gate can find the next item
/// with a binary search instead of walking to the end of the file.
fn item_start_indices(structure: &[&str]) -> Vec<usize> {
    structure
        .iter()
        .enumerate()
        .filter_map(|(i, line)| is_item_line(line).then_some(i))
        .collect()
}

/// The gate spans of a source file: every cfg(any()) attribute through the end of the item it
/// gates. An outer gate on a module covers the items nested in it; an inner `#![cfg(any())]`
/// covers the module (or file) that contains it.
fn gated_spans(structure: &[&str]) -> Vec<(usize, usize)> {
    let starts = item_start_indices(structure);
    let depths = line_depths(structure);
    let mut spans = Vec::new();
    let mut i = 0;
    while i < structure.len() {
        match gate_kind(structure[i]) {
            Some(GateKind::Outer) => {
                let at = starts.partition_point(|&s| s < i);
                if let Some(&j) = starts.get(at) {
                    let end = item_end(structure, j);
                    spans.push((i, end));
                    i = end + 1;
                    continue;
                }
            }
            Some(GateKind::Inner) => {
                let end = inner_gate_end(structure, i, &depths);
                spans.push((if depths[i] == 0 { 0 } else { i }, end));
                i = end + 1;
                continue;
            }
            None => {}
        }
        i += 1;
    }
    spans
}

/// The file's code that compiles: every gated span blanked from the ratchet's site view. The
/// site view keeps raw strings, because a CUDA kernel or query text in a raw string is
/// executable; the structure view used to find gates and item ends blanks them.
fn ungated_code(text: &str) -> String {
    let structure_text = strip_structure(text);
    let structure: Vec<&str> = structure_text.split('\n').collect();
    let code_text = crate::ratchet::strip_comments_strings_and_char_literals(text);
    let code: Vec<&str> = code_text.split('\n').collect();
    let n = code.len().min(structure.len());
    if n == 0 {
        return String::new();
    }
    let mut live = vec![true; n];
    for (start, end) in gated_spans(&structure[..n]) {
        for slot in live
            .iter_mut()
            .take(end.min(n - 1) + 1)
            .skip(start.min(n - 1))
        {
            *slot = false;
        }
    }
    code[..n]
        .iter()
        .enumerate()
        .map(|(i, line)| if live[i] { *line } else { "" })
        .collect::<Vec<_>>()
        .join("\n")
}

fn ascii_segments(code: &str) -> impl Iterator<Item = &str> {
    code.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|seg| !seg.is_empty())
}

/// Whether an identifier segment matches the D23 families, with the ratchet's rope exception:
/// rope matches a segment starting with rope or containing Rope/RoPE/ROPE, while the other
/// families match inside the segment. English property/Proper/Properties stay clear.
fn is_d23_segment(seg: &str) -> bool {
    if seg.len() < 4 {
        return false;
    }
    let lower = seg.to_ascii_lowercase();
    if lower.starts_with(concat!("ro", "pe"))
        || seg.contains(concat!("Ro", "pe"))
        || seg.contains(concat!("Ro", "PE"))
        || seg.contains(concat!("RO", "PE"))
    {
        return true;
    }
    crate::ratchet::D23_IDENT_FAMILIES.iter().any(|family| {
        let family: &str = family;
        family != concat!("ro", "pe") && family != concat!("ro", "pei") && lower.contains(family)
    })
}

/// The D23 sites in already-stripped code: the banned word as an identifier segment and every
/// family identifier segment. Strings and comments are gone, so prose cannot trip it.
fn d23_sites(code: &str) -> u64 {
    ascii_segments(code)
        .filter(|seg| {
            seg.to_ascii_lowercase().contains(crate::ratchet::D23_WORD) || is_d23_segment(seg)
        })
        .count() as u64
}

/// The D25 dense-product and tensor-core sites in already-stripped code: `.matmul(`, the candle
/// op `MatMul`, `cublas`, the word `gemm` (word characters are alphanumeric or `_`), and the
/// tensor-core intrinsics. A bare `matmul` identifier, a `matmul_helper`/ `gemm_helper` name and
/// raw strings that merely spell `matmul` are not dense-product sites.
fn d25_sites(code: &str) -> u64 {
    let mut sites = code.match_indices(".matmul(").count() as u64;
    sites += code.match_indices("MatMul").count() as u64;
    let lower = code.to_ascii_lowercase();
    sites += lower.match_indices("cublas").count() as u64;
    for (at, _) in lower.match_indices("gemm") {
        let before = if at == 0 {
            None
        } else {
            lower[..at].chars().next_back()
        };
        let after = lower[at + 4..].chars().next();
        if before.is_none_or(|c| !is_word_char(c)) && after.is_none_or(|c| !is_word_char(c)) {
            sites += 1;
        }
    }
    let patterns: [&[u8]; 4] = [b"wmma", b"mma_sync", b"wgmma", b"tcgen05"];
    let bytes = lower.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if let Some(pattern) = patterns.iter().find(|p| bytes[i..].starts_with(p)) {
            sites += 1;
            i += pattern.len();
        } else {
            i += 1;
        }
    }
    sites
}

/// The exponent and float suffix that may follow a decimal mantissa, and whether either was
/// found.
fn decimal_exp_and_suffix(b: &[u8], mut k: usize, is_cuda: bool) -> (usize, bool) {
    let n = b.len();
    let mut float = false;
    if k < n && matches!(b[k], b'e' | b'E') {
        let mut e = k + 1;
        if e < n && matches!(b[e], b'+' | b'-') {
            e += 1;
        }
        let digits = e;
        let mut q = e;
        while q < n && (b[q].is_ascii_digit() || b[q] == b'_' || (is_cuda && b[q] == b'\'')) {
            q += 1;
        }
        if q > digits {
            float = true;
            k = q;
        }
    }
    let suffix_start = k;
    while k < n && is_word_byte(b[k]) {
        k += 1;
    }
    let suffix = &b[suffix_start..k];
    let suffix_is_float = ["f16", "bf16", "f32", "f64"]
        .iter()
        .any(|s| suffix.eq_ignore_ascii_case(s.as_bytes()))
        || (is_cuda && matches!(suffix, b"f" | b"F" | b"d" | b"D"));
    (k, float || suffix_is_float)
}

/// The end of the numeric literal starting at `start`, and whether it is a floating-point
/// literal. Handles decimal literals, `1.`, `1f64`, `.5` (at its dot), and hex floats such as
/// `0x1.8p3`; plain integers, radix integers and tuple field accesses are not floats.
fn number_end(b: &[u8], start: usize, is_cuda: bool) -> (usize, bool) {
    let n = b.len();
    if b[start] == b'0' && start + 1 < n && matches!(b[start + 1], b'x' | b'X') {
        let mut k = start + 2;
        while k < n && (b[k].is_ascii_hexdigit() || b[k] == b'_') {
            k += 1;
        }
        if k < n && b[k] == b'.' && b.get(k + 1) != Some(&b'.') {
            k += 1;
            while k < n && (b[k].is_ascii_hexdigit() || b[k] == b'_') {
                k += 1;
            }
        }
        let mut float = false;
        if k < n && matches!(b[k], b'p' | b'P') {
            let mut e = k + 1;
            if e < n && matches!(b[e], b'+' | b'-') {
                e += 1;
            }
            let digits = e;
            let mut q = e;
            while q < n && (b[q].is_ascii_digit() || b[q] == b'_') {
                q += 1;
            }
            if q > digits {
                float = true;
                k = q;
            }
        }
        while k < n && is_word_byte(b[k]) {
            k += 1;
        }
        return (k, float);
    }
    if b[start] == b'0' && start + 1 < n && matches!(b[start + 1], b'b' | b'B' | b'o' | b'O') {
        let mut k = start + 2;
        while k < n && is_word_byte(b[k]) {
            k += 1;
        }
        return (k, false);
    }
    let mut k = start + 1;
    while k < n && (b[k].is_ascii_digit() || b[k] == b'_') {
        k += 1;
    }
    let mut has_dot = false;
    if k < n && b[k] == b'.' && b.get(k + 1) != Some(&b'.') {
        let next = b.get(k + 1).copied();
        let next_is_ident = next.is_some_and(|c| c.is_ascii_alphabetic() || c == b'_');
        if !next_is_ident {
            has_dot = true;
            k += 1;
            while k < n && (b[k].is_ascii_digit() || b[k] == b'_' || (is_cuda && b[k] == b'\'')) {
                k += 1;
            }
        }
    }
    let (k, tail_float) = decimal_exp_and_suffix(b, k, is_cuda);
    (k, has_dot || tail_float)
}

/// The D32 sites in already-stripped code: float type tokens and floating-point literals. `_` is
/// an identifier character, so `__half` matches and `f32_buffer` does not.
fn d32_sites(code: &str, is_cuda: bool) -> u64 {
    let b = code.as_bytes();
    let n = b.len();
    let mut sites = 0u64;
    let mut i = 0usize;
    while i < n {
        let c = b[i];
        let prev_word = i > 0 && is_word_byte(b[i - 1]);
        let prev_single_dot = i > 0 && b[i - 1] == b'.' && (i < 2 || b[i - 2] != b'.');
        if (c.is_ascii_alphabetic() || c == b'_') && !prev_word {
            let mut k = i + 1;
            while k < n && is_word_byte(b[k]) {
                k += 1;
            }
            let word = &code[i..k];
            if crate::ratchet::D32_FLOAT_TYPES
                .iter()
                .any(|token| word.eq_ignore_ascii_case(token))
            {
                sites += 1;
            }
            i = k;
            continue;
        }
        if c == b'.'
            && !prev_word
            && !(i > 0 && b[i - 1] == b'.')
            && b.get(i + 1).is_some_and(u8::is_ascii_digit)
        {
            let mut k = i + 1;
            while k < n && (b[k].is_ascii_digit() || b[k] == b'_' || (is_cuda && b[k] == b'\'')) {
                k += 1;
            }
            let (k, _) = decimal_exp_and_suffix(b, k, is_cuda);
            sites += 1;
            i = k.max(i + 2);
            continue;
        }
        if c.is_ascii_digit() && !prev_word && !prev_single_dot {
            let (end, float) = number_end(b, i, is_cuda);
            if float {
                sites += 1;
            }
            i = end.max(i + 1);
            continue;
        }
        i += 1;
    }
    sites
}

/// Whether D32 scans this path (the discrete model or a standalone instrument), matching the
/// ratchet: crates/uor-r4-discrete/ or docs/labs/<lab>/instrument/.
fn d32_scope(path: &str) -> bool {
    if path.starts_with("crates/uor-r4-discrete/") {
        return true;
    }
    let parts: Vec<&str> = path.split('/').collect();
    parts.len() >= 4 && parts[0] == "docs" && parts[1] == "labs" && parts[3] == "instrument"
}

/// Per-kind banned-site counts in one file's compiled code.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct BannedCounts {
    d23: u64,
    d25: u64,
    d32: u64,
}

fn banned_counts(text: &str, path: &str) -> BannedCounts {
    let code = ungated_code(text);
    counts_in(&code, path)
}

/// Per-kind counts in already-stripped compiled code.
fn counts_in(code: &str, path: &str) -> BannedCounts {
    BannedCounts {
        d23: d23_sites(code),
        d25: d25_sites(code),
        d32: if d32_scope(path) {
            d32_sites(code, path.ends_with(".cu") || path.ends_with(".cuh"))
        } else {
            0
        },
    }
}

/// One out-of-line `mod NAME;` declaration, with the inline module names that enclose it and
/// whether a cfg(any()) gate covers it.
struct ModDecl {
    name: String,
    inline_names: Vec<String>,
    gated: bool,
    line: usize,
}

/// The directory in which `declaring`'s out-of-line child modules live: beside lib.rs, main.rs
/// and mod.rs, otherwise in the folder named after the declaring file's stem.
fn module_dir(declaring: &str) -> String {
    let (dir, file) = declaring.rsplit_once('/').unwrap_or(("", declaring));
    let stem = file.strip_suffix(".rs").unwrap_or(file);
    if matches!(stem, "lib" | "main" | "mod") {
        dir.to_string()
    } else if dir.is_empty() {
        stem.to_string()
    } else {
        format!("{dir}/{stem}")
    }
}

/// The two files `mod NAME;` may name under the declaring file's inline module path.
fn module_file_candidates(declaring: &str, inline_names: &[String], name: &str) -> [String; 2] {
    let mut owner = module_dir(declaring);
    for inline in inline_names {
        owner = if owner.is_empty() {
            inline.clone()
        } else {
            format!("{owner}/{inline}")
        };
    }
    let join = |rel: String| {
        if owner.is_empty() {
            rel
        } else {
            format!("{owner}/{rel}")
        }
    };
    [join(format!("{name}.rs")), join(format!("{name}/mod.rs"))]
}

/// The out-of-line `mod NAME;` declarations in one structure view, with the inline module names
/// that enclose each. Inline modules are intervals found through `item_end`, so
/// `mod outer { mod inner; }` resolves inner under `outer/`.
fn mod_decls(structure: &[&str], spans: &[(usize, usize)]) -> Vec<ModDecl> {
    let starts = item_start_indices(structure);
    let mut raw: Vec<(ModDecl, bool, usize)> = Vec::new();
    for &line in &starts {
        let Some((keyword, offset)) = item_prefix(structure[line]) else {
            continue;
        };
        if keyword != "mod" {
            continue;
        }
        let end = (line + 8).min(structure.len());
        let joined = structure[line..end].join("\n");
        let (name, after) = next_word(&joined[offset..]);
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let rest = after.trim_start();
        let (inline, span_end) = if rest.starts_with('{') {
            (true, item_end(structure, line))
        } else if rest.starts_with(';') {
            (false, line)
        } else {
            continue;
        };
        let gated = spans.iter().any(|&(s, e)| s <= line && line <= e);
        raw.push((
            ModDecl {
                name: name.to_string(),
                inline_names: Vec::new(),
                gated,
                line,
            },
            inline,
            span_end,
        ));
    }
    let intervals: Vec<(usize, usize, String)> = raw
        .iter()
        .filter(|(_, inline, _)| *inline)
        .map(|(decl, _, end)| (decl.line, *end, decl.name.clone()))
        .collect();
    let mut decls = Vec::new();
    for (mut decl, inline, _) in raw {
        if inline {
            continue;
        }
        let mut ancestors: Vec<&(usize, usize, String)> = intervals
            .iter()
            .filter(|(start, end, _)| *start < decl.line && decl.line <= *end)
            .collect();
        ancestors.sort_by_key(|(start, _, _)| *start);
        decl.inline_names = ancestors
            .into_iter()
            .map(|(_, _, name)| name.clone())
            .collect();
        decls.push(decl);
    }
    decls
}

/// The out-of-line module declarations of a source text.
fn text_mod_decls(text: &str) -> Vec<ModDecl> {
    let structure_text = strip_structure(text);
    let structure: Vec<&str> = structure_text.split('\n').collect();
    let spans = gated_spans(&structure);
    mod_decls(&structure, &spans)
}

/// The file a `mod NAME;` names at `rev`, if it exists.
fn resolve_module(
    pr: &Pr,
    rev: &str,
    declaring: &str,
    inline_names: &[String],
    name: &str,
) -> Option<String> {
    module_file_candidates(declaring, inline_names, name)
        .into_iter()
        .find(|path| pr.show(rev, path.as_str()).is_some())
}

/// The out-of-line module files a source file declares at `rev`, mapped to whether every
/// declaration of that file is gated. A declaration absent from the map is not present.
fn direct_mods(pr: &Pr, rev: &str, declaring: &str) -> HashMap<String, bool> {
    let mut mods = HashMap::new();
    if !declaring.ends_with(".rs") {
        return mods;
    }
    let Some(text) = pr.show(rev, declaring) else {
        return mods;
    };
    for decl in text_mod_decls(&text) {
        let Some(path) = resolve_module(pr, rev, declaring, &decl.inline_names, &decl.name) else {
            continue;
        };
        mods.entry(path)
            .and_modify(|gated: &mut bool| *gated = *gated && decl.gated)
            .or_insert(decl.gated);
    }
    mods
}

/// Mark `path` and every module file it declares as gated out at `rev`.
fn mark_gated_tree(pr: &Pr, rev: &str, path: &str, gated: &mut HashSet<String>, depth: usize) {
    if depth > 64 || !gated.insert(path.to_string()) || !path.ends_with(".rs") {
        return;
    }
    let Some(text) = pr.show(rev, path) else {
        return;
    };
    for decl in text_mod_decls(&text) {
        if let Some(child) = resolve_module(pr, rev, path, &decl.inline_names, &decl.name) {
            mark_gated_tree(pr, rev, &child, gated, depth + 1);
        }
    }
}

/// Candidate parent module files that could declare `path` as an out-of-line module.
fn parent_module_candidates(path: &str) -> Vec<String> {
    let Some((dir, file)) = path.rsplit_once('/') else {
        return Vec::new();
    };
    let stem = file.strip_suffix(".rs").unwrap_or(file);
    let parent = if stem == "mod" {
        let Some((grand, _)) = dir.rsplit_once('/') else {
            return Vec::new();
        };
        grand.to_string()
    } else if matches!(stem, "lib" | "main") {
        return Vec::new();
    } else {
        dir.to_string()
    };
    let join = |rel: &str| {
        if parent.is_empty() {
            rel.to_string()
        } else {
            format!("{parent}/{rel}")
        }
    };
    let beside = if parent.is_empty() {
        "mod.rs".to_string()
    } else {
        format!("{parent}.rs")
    };
    vec![beside, join("mod.rs"), join("lib.rs"), join("main.rs")]
}

/// Whether a parent module file's gated out-of-line `mod` declaration, or a gated parent above
/// it, keeps `path` out of the compiled tree at `rev`.
fn gated_by_ancestor(pr: &Pr, rev: &str, path: &str, depth: usize) -> bool {
    if depth > 64 {
        return false;
    }
    for parent in parent_module_candidates(path) {
        if parent == path {
            continue;
        }
        let Some(text) = pr.show(rev, &parent) else {
            continue;
        };
        let decls = text_mod_decls(&text);
        let mut declares = false;
        let mut directly_gated = false;
        for decl in decls {
            let Some(resolved) = resolve_module(pr, rev, &parent, &decl.inline_names, &decl.name)
            else {
                continue;
            };
            if resolved == path {
                declares = true;
                directly_gated |= decl.gated;
            }
        }
        if declares && (directly_gated || gated_by_ancestor(pr, rev, &parent, depth + 1)) {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy)]
enum RevSide {
    Base,
    Head,
}

fn side_rev<'a>(pr: &'a Pr, side: RevSide) -> &'a str {
    match side {
        RevSide::Base => &pr.merge_base,
        RevSide::Head => &pr.head,
    }
}

fn side_path(change: &Change, side: RevSide) -> &str {
    match side {
        RevSide::Base => change.paths.first().map(String::as_str).unwrap_or(""),
        RevSide::Head => change.paths.last().map(String::as_str).unwrap_or(""),
    }
}

/// Every module file that a changed `mod NAME;` declaration gates out at one revision, including
/// the files those modules declare. A changed file already gated by an unchanged parent is marked
/// too, so editing inside a permanently gated tree does not read as newly compiled code.
fn gated_files(pr: &Pr, side: RevSide) -> HashSet<String> {
    let rev = side_rev(pr, side);
    let mut gated = HashSet::new();
    for change in &pr.changes {
        let path = side_path(change, side);
        if !path.ends_with(".rs") {
            continue;
        }
        let Some(text) = pr.show(rev, path) else {
            continue;
        };
        for decl in text_mod_decls(&text) {
            if !decl.gated {
                continue;
            }
            if let Some(file) = resolve_module(pr, rev, path, &decl.inline_names, &decl.name) {
                mark_gated_tree(pr, rev, &file, &mut gated, 0);
            }
        }
        if gated_by_ancestor(pr, rev, path, 0) {
            mark_gated_tree(pr, rev, path, &mut gated, 0);
        }
    }
    gated
}

/// The banned sites that compile from `path` and every ungated out-of-line module it declares at
/// `rev`.
fn compiled_module_counts(pr: &Pr, rev: &str, path: &str, depth: usize) -> BannedCounts {
    if depth > 64 {
        return BannedCounts::default();
    }
    let Some(text) = pr.show(rev, path) else {
        return BannedCounts::default();
    };
    let mut counts = banned_counts(&text, path);
    if path.ends_with(".rs") {
        for decl in text_mod_decls(&text) {
            if decl.gated {
                continue;
            }
            if let Some(child) = resolve_module(pr, rev, path, &decl.inline_names, &decl.name) {
                let child_counts = compiled_module_counts(pr, rev, &child, depth + 1);
                counts.d23 += child_counts.d23;
                counts.d25 += child_counts.d25;
                counts.d32 += child_counts.d32;
            }
        }
    }
    counts
}

/// Details already emitted, so a module file reached from several changed declaring files is
/// reported once.
#[derive(Default)]
struct DetailSink {
    details: Vec<String>,
    seen: HashSet<(String, &'static str, u64, u64)>,
}

impl DetailSink {
    fn push(&mut self, file: &str, kind: &'static str, before: u64, after: u64, via: &str) {
        if after <= before || !self.seen.insert((file.to_string(), kind, before, after)) {
            return;
        }
        self.details.push(format!(
            "{file}: {kind} sites in compiled code rise from {before} at the merge base to {after} at the head{via} (a gate was removed, moved to a test-only gate, or the gated code changed); CHARTER section 3.4 forbids banned code in any compiled module"
        ));
    }
}

/// Under the ratchets, banned code that is gated out is counted but not compiled; CHARTER section
/// 3.4 says banned code may not exist in any compiled or loaded module, so un-gating it (or moving
/// it behind a test-only gate) is a presence change the ratchets cannot see. This rule compares the
/// banned sites that compile per changed file at the merge base and at the head and fails when the
/// head compiles more of any family. An out-of-line `mod NAME;` gate is followed into NAME's
/// file, so lifting or adding that gate charges the module file it compiles out. Deleting gated
/// code, keeping the gate, and moving a gate to a parent module all pass.
fn rule_banned_presence(pr: &Pr) -> RuleResult {
    const RULE: &str = "banned-presence";
    let base_gated = gated_files(pr, RevSide::Base);
    let head_gated = gated_files(pr, RevSide::Head);
    let mut sink = DetailSink::default();
    for change in &pr.changes {
        let old = change.paths.first().map(String::as_str).unwrap_or("");
        let new = change.paths.last().map(String::as_str).unwrap_or("");
        if change.status == 'A' || change.status == 'D' || !is_banned_source(new) {
            continue;
        }
        let (Some(base), Some(head)) = (pr.at_base(old), pr.at_head(new)) else {
            continue;
        };
        let mut was = banned_counts(&base, new);
        let mut now = banned_counts(&head, new);
        if base_gated.contains(old) {
            was = BannedCounts::default();
        }
        if head_gated.contains(new) {
            now = BannedCounts::default();
        }
        for (kind, before, after) in [
            ("D23", was.d23, now.d23),
            ("D25", was.d25, now.d25),
            ("D32", was.d32, now.d32),
        ] {
            sink.push(new, kind, before, after, "");
        }
        // A gate lifted off `mod NAME;` compiles the module's file, which this change need not
        // touch; compare that file's compiled sites at the base and the head too.
        let base_mods = if base_gated.contains(old) {
            HashMap::new()
        } else {
            direct_mods(pr, &pr.merge_base, old)
        };
        let head_mods = if head_gated.contains(new) {
            HashMap::new()
        } else {
            direct_mods(pr, &pr.head, new)
        };
        let mut modules: Vec<&String> = base_mods.keys().chain(head_mods.keys()).collect();
        modules.sort();
        modules.dedup();
        for module in modules {
            let base_compiled = matches!(base_mods.get(module), Some(false));
            let head_compiled = matches!(head_mods.get(module), Some(false));
            if base_compiled == head_compiled {
                continue;
            }
            let was = if base_compiled {
                compiled_module_counts(pr, &pr.merge_base, module, 0)
            } else {
                BannedCounts::default()
            };
            let now = if head_compiled {
                compiled_module_counts(pr, &pr.head, module, 0)
            } else {
                BannedCounts::default()
            };
            let via = if head_compiled && !base_compiled {
                format!(" (counting the newly compiled module file(s) {module})")
            } else {
                String::new()
            };
            for (kind, before, after) in [
                ("D23", was.d23, now.d23),
                ("D25", was.d25, now.d25),
                ("D32", was.d32, now.d32),
            ] {
                sink.push(new, kind, before, after, &via);
            }
        }
    }
    if sink.details.is_empty() {
        result(
            RULE,
            sink.details,
            "no changed file compiles more banned code than its base",
            "",
        )
    } else {
        let n = sink.details.len();
        result(
            RULE,
            sink.details,
            "",
            &format!("{n} changed file(s) compile more banned code than their base"),
        )
    }
}

/// The first non-merge commit adds the lowest-numbered ORDER, and that commit changes nothing
/// outside its folder; each later (repair) ORDER is added by a commit that changes nothing
/// outside its own folder.
fn rule_commit_order(pr: &Pr) -> RuleResult {
    const R: &str = "commit-order";
    let chain = match pr.the_chain() {
        Ok(c) => c,
        Err(e) => return fail(R, format!("cannot check the commit order: {e}")),
    };
    let order = chain[0].as_str();
    let folder = order.trim_end_matches("ORDER.md");
    let commits = match pr_commits(pr) {
        Ok(c) => c,
        Err(e) => return fail(R, format!("cannot list the PR's commits: {e}")),
    };
    let mut details = Vec::new();
    for (sha, merge) in &commits {
        let short = &sha[..sha.len().min(9)];
        if *merge {
            // A merge's own tree can carry work no single-parent diff shows.
            details.push(format!(
                "{short} is a merge before the commit that adds {order}"
            ));
            continue;
        }
        let changes = match commit_changes(pr, sha) {
            Ok(c) => c,
            Err(e) => return fail(R, format!("cannot read commit {short}: {e}")),
        };
        let paths = || {
            changes
                .iter()
                .flat_map(|c| c.paths.iter().map(String::as_str))
        };
        // An add, or the new side of a rename or copy, as in `added_orders`.
        let adds_order = changes.iter().any(|c| {
            matches!(c.status, 'A' | 'R' | 'C') && c.paths.last().map(String::as_str) == Some(order)
        });
        if !adds_order {
            let p: Vec<&str> = paths().collect();
            if !p.is_empty() {
                details.push(format!(
                    "{short} changes {} before the commit that adds {order}",
                    p.join(", ")
                ));
            }
            continue;
        }
        let outside: Vec<&str> = paths().filter(|p| !p.starts_with(folder)).collect();
        if !outside.is_empty() {
            details.push(format!(
                "{short} adds {order} but also changes {}, outside {folder}",
                outside.join(", ")
            ));
        }
        // Later commits may edit the ORDER (scope widening does), never re-point it.
        let first = pr.show(sha, order).map(|t| order_identity(&t));
        let at_head = pr.at_head(order).map(|t| order_identity(&t));
        if first != at_head {
            details.push(format!(
                "the ORDER's `Lab:` or `Item:` at the head differs from the ORDER as {short} added it"
            ));
        }
        match repair_order_problems(pr, &chain[1..], &commits) {
            Ok(p) => details.extend(p),
            Err(e) => return fail(R, e),
        }
        let pass = match chain.len() {
            1 => format!("{short} adds {order} first, alone in its folder"),
            k => format!(
                "{short} adds {order} first, alone in its folder; each of the {} repair ORDERs is added alone in its folder",
                k - 1
            ),
        };
        return result(
            R,
            details,
            &pass,
            "the ORDER commits do not come first, alone",
        );
    }
    fail(R, format!("no non-merge commit adds {order}"))
}

/// What is wrong with how each repair ORDER (every ORDER after the first) was added: one
/// non-merge commit adds it, that commit changes nothing outside its folder, and its `Lab:`
/// and `Item:` are the same at the head.
fn repair_order_problems(
    pr: &Pr,
    repairs: &[String],
    commits: &[(String, bool)],
) -> Result<Vec<String>, String> {
    let mut details = Vec::new();
    for order in repairs.iter().map(String::as_str) {
        let folder = order.trim_end_matches("ORDER.md");
        let mut found = false;
        for (sha, _) in commits.iter().filter(|(_, merge)| !merge) {
            let short = &sha[..sha.len().min(9)];
            let changes =
                commit_changes(pr, sha).map_err(|e| format!("cannot read commit {short}: {e}"))?;
            let adds = changes.iter().any(|c| {
                matches!(c.status, 'A' | 'R' | 'C')
                    && c.paths.last().map(String::as_str) == Some(order)
            });
            if !adds {
                continue;
            }
            found = true;
            let outside: Vec<&str> = changes
                .iter()
                .flat_map(|c| c.paths.iter().map(String::as_str))
                .filter(|p| !p.starts_with(folder))
                .collect();
            if !outside.is_empty() {
                details.push(format!(
                    "{short} adds the repair ORDER {order} but also changes {}, outside {folder}",
                    outside.join(", ")
                ));
            }
            let first = pr.show(sha, order).map(|t| order_identity(&t));
            if first != pr.at_head(order).map(|t| order_identity(&t)) {
                details.push(format!(
                    "the `Lab:` or `Item:` of {order} at the head differs from the ORDER as {short} added it"
                ));
            }
            break;
        }
        if !found {
            details.push(format!("no non-merge commit adds the repair ORDER {order}"));
        }
    }
    Ok(details)
}

/// The `## ` sections a REPORT needs, in this order (each heading starts with one, any case).
const REPORT_SECTIONS: [&str; 7] = [
    "What did not work",
    "What changed",
    "Measurements",
    "Checks run at the head",
    "Skills used",
    "Still open",
    "Plain summary",
];

/// Each line with whether it can be a heading: false inside a ``` or ~~~ fenced code block,
/// fence lines included.
pub(crate) fn heading_lines(text: &str) -> impl Iterator<Item = (&str, bool)> {
    let mut fence: Option<&str> = None;
    text.lines().map(move |line| {
        let t = line.trim_start();
        let marker = ["```", "~~~"].into_iter().find(|m| t.starts_with(m));
        match (fence, marker) {
            (None, Some(m)) => {
                fence = Some(m);
                (line, false)
            }
            (Some(open), Some(m)) if open == m => {
                fence = None;
                (line, false)
            }
            (Some(_), _) => (line, false),
            (None, None) => (line, true),
        }
    })
}

/// What is wrong with a REPORT's sections; empty when it is valid.
pub fn report_problems(text: &str) -> Vec<String> {
    // (lower-cased heading, whether a non-blank line follows before the next `## `)
    let mut sections: Vec<(String, bool)> = Vec::new();
    for (line, can_head) in heading_lines(text) {
        if let Some(h) = line.strip_prefix("## ").filter(|_| can_head) {
            sections.push((h.trim().to_lowercase(), false));
        } else if !line.trim().is_empty() {
            if let Some(last) = sections.last_mut() {
                last.1 = true;
            }
        }
    }
    let mut out = Vec::new();
    let mut prev: Option<(usize, &str)> = None;
    for name in REPORT_SECTIONS {
        let lower = name.to_lowercase();
        let Some(i) = sections.iter().position(|(h, _)| h.starts_with(&lower)) else {
            out.push(format!("no `## {name}` heading"));
            continue;
        };
        if let Some((pi, pname)) = prev {
            if i < pi {
                out.push(format!(
                    "out of order: `## {name}` comes before `## {pname}`"
                ));
            }
        }
        if !sections[i].1 {
            out.push(format!("the `## {name}` section is empty"));
        }
        prev = Some((i, name));
    }
    out
}

/// `<ORDER folder>/REPORT.md` exists at the head with every section, in order, none empty.
fn rule_report(pr: &Pr) -> RuleResult {
    const R: &str = "report";
    let order = match pr.the_order() {
        Ok(p) => p,
        Err(e) => return fail(R, format!("cannot find the REPORT: {e}")),
    };
    let path = format!("{}REPORT.md", order.trim_end_matches("ORDER.md"));
    let Some(text) = pr.at_head(&path) else {
        return fail(R, format!("{path} is missing at the head"));
    };
    result(
        R,
        report_problems(&text),
        &path,
        &format!("{path} is not a valid REPORT"),
    )
}

/// An owner-paths entry covers `path`: `<prefix>/**` covers the prefix and everything below it.
fn owner_match(entry: &str, path: &str) -> bool {
    match entry.strip_suffix("/**") {
        Some(prefix) => path == prefix || path.starts_with(&format!("{prefix}/")),
        None => path == entry,
    }
}

/// What the ORDER's `## Owner approval` section(s) lack for `entries`; empty when approved.
pub fn owner_approval_missing(order_text: &str, entries: &[&str]) -> Vec<String> {
    let mut section = String::new();
    let mut inside = false;
    for (line, can_head) in heading_lines(order_text) {
        if can_head && (line.starts_with("## ") || line.starts_with("# ")) {
            inside = line.to_lowercase().starts_with("## owner approval");
        }
        if inside {
            section.push_str(line);
            section.push('\n');
        }
    }
    if section.is_empty() {
        return vec!["the ORDER has no `## Owner approval` section".into()];
    }
    let mut out = Vec::new();
    for entry in entries {
        let named = entry.strip_suffix("/**").unwrap_or(entry);
        if !section.contains(named) {
            out.push(format!("the owner approval section does not name {entry}"));
        }
    }
    let quote = regex::Regex::new(r#""[^"\n]+""#).expect("a valid pattern");
    if !quote.is_match(&section) {
        out.push("the owner approval section holds no double-quoted answer".into());
    }
    let time =
        regex::Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}(:\d{2})?Z").expect("a valid pattern");
    if !time.is_match(&section) {
        out.push(
            "the owner approval section holds no ISO UTC time (YYYY-MM-DDTHH:MM[:SS]Z)".into(),
        );
    }
    out
}

/// A change to a path in `docs/harness/owner-paths.json` (read at the base) carries the owner's
/// approval in the ORDER: the entry named, a quoted answer and an ISO UTC time.
fn rule_owner_paths(pr: &Pr) -> RuleResult {
    const R: &str = "owner-paths";
    const LIST: &str = "docs/harness/owner-paths.json";
    let Some(text) = pr.at_base(LIST) else {
        return fail(R, format!("{LIST} is missing at the merge base"));
    };
    let v: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return fail(R, format!("{LIST} at the merge base: {e}")),
    };
    let Some(entries) = v.get("paths").and_then(|p| p.as_array()).and_then(|a| {
        a.iter()
            .map(|e| e.get("path").and_then(|p| p.as_str()))
            .collect::<Option<Vec<&str>>>()
    }) else {
        return fail(
            R,
            format!("{LIST} at the merge base has no `paths` list of {{\"path\": ..}} entries"),
        );
    };
    let mut details = Vec::new();
    let mut matched = Vec::new();
    for entry in entries {
        let mut hits: Vec<&str> = pr
            .changed_paths()
            .map(|(_, p)| p)
            .filter(|p| owner_match(entry, p))
            .collect();
        hits.dedup();
        if !hits.is_empty() {
            details.push(format!("{} changed (owner path {entry})", hits.join(", ")));
            matched.push(entry);
        }
    }
    if matched.is_empty() {
        return result(R, Vec::new(), "no owner path changed", "");
    }
    let missing = match pr.the_order() {
        Err(e) => vec![format!("no ORDER to carry the owner's approval: {e}")],
        Ok(order) => match pr.at_head(order) {
            None => vec![format!("{order} cannot be read at the head")],
            Some(t) => owner_approval_missing(&t, &matched),
        },
    };
    if missing.is_empty() {
        return RuleResult {
            rule: R,
            verdict: Verdict::Pass,
            summary: format!("owner approval recorded for {}", matched.join(", ")),
            details: Vec::new(),
        };
    }
    details.extend(missing);
    RuleResult {
        rule: R,
        verdict: Verdict::Fail,
        summary: "owner path(s) changed without the owner's approval in the ORDER".into(),
        details,
    }
}

fn not_applicable(rule: &'static str, summary: &str) -> RuleResult {
    RuleResult {
        rule,
        verdict: Verdict::NotApplicable,
        summary: summary.to_string(),
        details: Vec::new(),
    }
}

/// The markdown headings in `text`: each ATX heading line (up to 3 spaces, 1 to 6 `#`, then a
/// space, a tab or the line end), and the text line of each setext heading (a non-blank line
/// over a line of only `=` or only `-`, up to 3 leading spaces).
fn markdown_headings(text: &str) -> Vec<&str> {
    let atx = regex::Regex::new(r"^ {0,3}#{1,6}([ \t]|$)").expect("a valid pattern");
    let underline = regex::Regex::new(r"^ {0,3}(=+|-+)[ \t]*$").expect("a valid pattern");
    let lines: Vec<&str> = text.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(i, line)| {
            let setext = !line.trim().is_empty()
                && lines
                    .get(i + 1)
                    .is_some_and(|next| underline.is_match(next));
            atx.is_match(line) || setext
        })
        .map(|(_, line)| *line)
        .collect()
}

/// Outside the files in `docs/harness/rule-sources.json` (read at the base; absent counts as
/// empty), `archive/`, `docs/loop/tasks/` and `docs/roadmap/items/`, the PR adds no markdown
/// file (`.md`, `.markdown`, `.mdx`, any case) named for a rule and no markdown heading (ATX or
/// setext) containing the word `rule` or `rules` (`_` and other non-alphanumerics separate words).
/// A heading counts as added when the base file lacks it; a file renamed or copied out of a listed
/// or exempt file is compared with an empty base.
fn rule_rule_sources(pr: &Pr) -> RuleResult {
    const R: &str = "rule-sources";
    const LIST: &str = "docs/harness/rule-sources.json";
    let listed: Vec<String> = match pr.at_base(LIST) {
        None => Vec::new(),
        Some(text) => {
            let files = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| {
                    v.get("files")?
                        .as_array()?
                        .iter()
                        .map(|f| f.as_str().map(String::from))
                        .collect::<Option<Vec<String>>>()
                });
            match files {
                Some(f) => f,
                None => {
                    return fail(
                        R,
                        format!("{LIST} at the merge base has no `files` list of paths"),
                    )
                }
            }
        }
    };
    let exempt = |p: &str| {
        ["archive/", "docs/loop/tasks/", "docs/roadmap/items/"]
            .iter()
            .any(|d| p.starts_with(d))
            || listed.iter().any(|l| l == p)
    };
    let word = regex::Regex::new(r"(?i)(^|[^a-z0-9])rules?([^a-z0-9]|$)").expect("a valid pattern");
    let markdown = |p: &str| {
        let p = p.to_lowercase();
        [".md", ".markdown", ".mdx"].iter().any(|e| p.ends_with(e))
    };
    let mut details = Vec::new();
    for c in &pr.changes {
        let Some(path) = c.paths.last().map(String::as_str) else {
            continue;
        };
        if c.status == 'D' || !markdown(path) || exempt(path) {
            continue;
        }
        let name = path.rsplit('/').next().unwrap_or(path);
        if matches!(c.status, 'A' | 'R' | 'C') && name.to_lowercase().contains("rule") {
            details.push(format!("{path} adds a rule-named file outside {LIST}"));
        }
        let Some(head) = pr.at_head(path) else {
            continue;
        };
        // Text moved or copied out of a listed or exempt file counts as added in full.
        let base = if matches!(c.status, 'R' | 'C') && exempt(&c.paths[0]) {
            String::new()
        } else {
            pr.at_base(&c.paths[0]).unwrap_or_default()
        };
        let old: std::collections::HashSet<&str> = markdown_headings(&base).into_iter().collect();
        for line in markdown_headings(&head) {
            if word.is_match(line) && !old.contains(line) {
                details.push(format!(
                    "{path} adds the rule heading `{line}` outside {LIST}"
                ));
            }
        }
    }
    result(
        R,
        details,
        &format!("no rule stated outside {LIST}"),
        &format!("a rule is stated outside {LIST}"),
    )
}

const AGENTS_START: &str = "<!-- loop:lessons-top-ten:start -->";
const AGENTS_END: &str = "<!-- loop:lessons-top-ten:end -->";

/// The bytes of AGENTS.md between the lessons markers; `None` when neither marker is there. A
/// lone or misordered marker yields the whole text, so it still counts as a view.
fn agents_lessons(text: &str) -> Option<String> {
    match (text.find(AGENTS_START), text.find(AGENTS_END)) {
        (None, None) => None,
        (Some(s), Some(e)) if e >= s + AGENTS_START.len() => {
            Some(text[s + AGENTS_START.len()..e].to_string())
        }
        _ => Some(format!("unpaired marker\n{text}")),
    }
}

/// The rendered views change only on the `kimi/render` branch: `STATE.md`, `HANDOFF.md`,
/// `docs/harness/LESSONS.md` and AGENTS.md between the lessons-top-ten markers. The roadmap
/// README and `docs/loop/baseline.json` join this lock with loop PR 8.
fn rule_rendered_lock(pr: &Pr) -> RuleResult {
    const R: &str = "rendered-lock";
    let mut views: Vec<(&str, Option<String>, Option<String>)> =
        ["STATE.md", "HANDOFF.md", "docs/harness/LESSONS.md"]
            .iter()
            .map(|p| (*p, pr.at_base(p), pr.at_head(p)))
            .collect();
    views.push((
        "AGENTS.md (lessons-top-ten markers)",
        pr.at_base("AGENTS.md").as_deref().and_then(agents_lessons),
        pr.at_head("AGENTS.md").as_deref().and_then(agents_lessons),
    ));
    if views.iter().all(|(_, b, h)| b.is_none() && h.is_none()) {
        return not_applicable(R, "no rendered view exists yet (loop PR 8 commits them)");
    }
    // The views' first commit is the migration (loop PR 8): while none exists at the base,
    // the PR that commits them is allowed on any branch; once they exist at the base, only
    // kimi/render changes them (the steady-state render PR, loop page "The render PR").
    if views.iter().all(|(_, b, _)| b.is_none()) {
        return result(
            R,
            Vec::new(),
            "no rendered view at the base: the first commit of the views (loop PR 8)",
            "",
        );
    }
    let changed: Vec<String> = views
        .iter()
        .filter(|(_, b, h)| b != h)
        .map(|(p, _, _)| format!("{p} changed"))
        .collect();
    if matches!(pr.head_ref.as_str(), "kimi/render" | "origin/kimi/render") {
        return result(
            R,
            Vec::new(),
            "head ref kimi/render may change the views",
            "",
        );
    }
    let shown = if pr.head_ref.is_empty() {
        "(none)"
    } else {
        pr.head_ref.as_str()
    };
    result(
        R,
        changed,
        "no rendered view changed",
        &format!("rendered views change only on kimi/render, not head ref {shown}"),
    )
}

/// A read-path budget: name, (base, head) value, over budget at a value, the budget, its unit.
type Budget = (
    &'static str,
    (u64, u64),
    fn(u64) -> bool,
    &'static str,
    &'static str,
);

/// The read-path budgets, measured on the head and compared with the base. **Any budget overrun is
/// a failure since loop PR 11**: the loop relies on the read path, so the former warnings (an
/// over-budget file that did not grow, and a whole path above the 76,800 B line) are errors now.
/// The whole path's growth leaves out the ORDER, which the base never has. A read-path file that is
/// a symlink at the head fails.
fn rule_read_path(pr: &Pr) -> RuleResult {
    const R: &str = "read-path";
    let bytes = |t: Option<String>| t.map_or(0, |t| t.len() as u64);
    let lines = |t: Option<String>| t.map_or(0, |t| t.lines().count() as u64);
    let order = pr.the_order().ok();
    let pair = |p: &str| (bytes(pr.at_base(p)), bytes(pr.at_head(p)));
    let agents_lines = (
        lines(pr.at_base("AGENTS.md")),
        lines(pr.at_head("AGENTS.md")),
    );
    let (agents, charter, state, handoff) = (
        pair("AGENTS.md"),
        pair("CHARTER.md"),
        pair("STATE.md"),
        pair("HANDOFF.md"),
    );
    let ord = order.map_or((0, 0), pair);
    // Growth of the whole path is judged without the ORDER, which every PR adds.
    let core = (
        agents.0 + charter.0 + state.0 + handoff.0,
        agents.1 + charter.1 + state.1 + handoff.1,
    );
    let total = core.1 + ord.1;
    // (name, (base, head), over budget at this value, the budget as words, unit)
    let items: [Budget; 4] = [
        ("AGENTS.md", agents_lines, |v| v > 80, "at most 80", "lines"),
        ("STATE.md", state, |v| v >= 24_576, "under 24576", "B"),
        ("HANDOFF.md", handoff, |v| v >= 24_576, "under 24576", "B"),
        ("the ORDER", ord, |v| v >= 4_096, "under 4096", "B"),
    ];
    let mut fails = Vec::new();
    for path in ["AGENTS.md", "CHARTER.md", "STATE.md", "HANDOFF.md"]
        .into_iter()
        .chain(order)
    {
        let tree = git(pr.root, &["ls-tree", &pr.head, "--", path])
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default();
        if tree.starts_with("120000 ") {
            fails.push(format!("{path} is a symlink at the head"));
        }
    }
    for (name, (b, h), over, budget, unit) in items {
        if over(h) {
            let line = format!("{name} is {h} {unit} (base {b}), over its budget ({budget})");
            if h > b {
                fails.push(format!("{line} and grew"));
            } else {
                fails.push(format!("{line} but did not grow"));
            }
        }
    }
    if total > 128_000 {
        let line = format!(
            "the read path is {total} B, over its budget (stop above 128000); AGENTS + CHARTER + STATE + HANDOFF is {} B (base {})",
            core.1, core.0
        );
        if core.1 > core.0 {
            fails.push(format!("{line} and grew"));
        } else {
            fails.push(format!("{line} but did not grow"));
        }
    } else if total > 76_800 {
        fails.push(format!(
            "the read path is {total} B (AGENTS + CHARTER + STATE + HANDOFF base {}), above the 76800 B line",
            core.0
        ));
    }
    let summary = format!(
        "AGENTS.md {} lines, STATE.md {} B, HANDOFF.md {} B, ORDER {} B, read path {} B (AGENTS + CHARTER + STATE + HANDOFF + ORDER)",
        agents_lines.1, state.1, handoff.1, ord.1, total
    );
    let verdict = if fails.is_empty() {
        Verdict::Pass
    } else {
        Verdict::Fail
    };
    RuleResult {
        rule: R,
        verdict,
        summary,
        details: fails,
    }
}

/// `docs/start/FILEMAP.md` at the head has a row (a table line whose first cell is a backticked
/// path) for every depth-1 and depth-2 directory, and every row names a directory at the head.
fn rule_filemap(pr: &Pr) -> RuleResult {
    const R: &str = "filemap";
    const MAP: &str = "docs/start/FILEMAP.md";
    let Some(text) = pr.at_head(MAP) else {
        return not_applicable(
            R,
            "docs/start/FILEMAP.md does not exist yet (loop PR 8 creates it)",
        );
    };
    let raw = match git(
        pr.root,
        &["ls-tree", "-d", "-r", "--name-only", "-z", &pr.head],
    ) {
        Ok(r) => r,
        Err(e) => return fail(R, e),
    };
    let dirs = z_fields(&raw);
    let row = regex::Regex::new(r"^\|\s*`([^`]+)`\s*\|").expect("a valid pattern");
    let rows: Vec<String> = text
        .lines()
        .filter_map(|l| row.captures(l))
        .map(|c| c[1].trim().trim_end_matches('/').to_string())
        .collect();
    let mut details = Vec::new();
    for d in &dirs {
        let depth = d.split('/').count();
        if depth > 2 || d.split('/').any(|s| s.starts_with('.')) || d.starts_with("archive/") {
            continue;
        }
        if !rows.contains(d) {
            details.push(format!("missing row: {d}"));
        }
    }
    for r in &rows {
        if !dirs.contains(r) {
            details.push(format!("stale row: {r} is not a directory at the head"));
        }
    }
    result(
        R,
        details,
        &format!("{MAP} has a row for every depth-1 and depth-2 directory"),
        &format!("{MAP} has missing or stale rows"),
    )
}

/// Base and head differ only on `status:` lines.
fn status_only_edit(pr: &Pr, path: &str) -> bool {
    let (Some(b), Some(h)) = (pr.at_base(path), pr.at_head(path)) else {
        return false;
    };
    let (b, h): (Vec<&str>, Vec<&str>) = (b.lines().collect(), h.lines().collect());
    b.len() == h.len()
        && b.iter()
            .zip(&h)
            .all(|(x, y)| x == y || (x.starts_with("status:") && y.starts_with("status:")))
}

/// Every base line is still present at the head, in order (rows added only). Used for the
/// item's notebook page, whose results table is followed by prose, so a new row lands mid-file.
fn keeps_base_lines(pr: &Pr, path: &str) -> bool {
    let Some(h) = pr.at_head(path) else {
        return false;
    };
    let b = pr.at_base(path).unwrap_or_default();
    let mut head = h.lines();
    b.lines().all(|bl| head.any(|hl| hl == bl))
}

/// Separator-aware glob match: `*` and `?` never match `/`; a segment that is exactly `**`
/// matches zero or more whole segments.
pub fn glob_match(glob: &str, path: &str) -> bool {
    let g: Vec<&str> = glob.split('/').collect();
    let p: Vec<&str> = path.split('/').collect();
    match_segments(&g, &p)
}

fn match_segments(g: &[&str], p: &[&str]) -> bool {
    match g.split_first() {
        None => p.is_empty(),
        Some((&"**", rest)) => (0..=p.len()).any(|i| match_segments(rest, &p[i..])),
        Some((seg, rest)) => {
            !p.is_empty()
                && segment_match(seg.as_bytes(), p[0].as_bytes())
                && match_segments(rest, &p[1..])
        }
    }
}

fn segment_match(pat: &[u8], s: &[u8]) -> bool {
    match pat.split_first() {
        None => s.is_empty(),
        Some((b'*', rest)) => (0..=s.len()).any(|i| segment_match(rest, &s[i..])),
        Some((b'?', rest)) => !s.is_empty() && segment_match(rest, &s[1..]),
        Some((c, rest)) => s.first() == Some(c) && segment_match(rest, &s[1..]),
    }
}

/// The size class of a changed path.
pub(crate) fn size_class(path: &str) -> &'static str {
    let name = path.rsplit('/').next().unwrap_or(path);
    if path.split('/').any(|s| s == "tests")
        || name.ends_with("_test.rs")
        || name.ends_with("tests.rs")
    {
        "tests"
    } else if path == "Cargo.lock" || path == "docs/roadmap/README.md" {
        "generated"
    } else if [".rs", ".toml", ".yml", ".yaml", ".sh"]
        .iter()
        .any(|e| name.ends_with(e))
    {
        "code"
    } else if name.ends_with(".md") {
        "docs"
    } else {
        "other"
    }
}

const CLASSES: [&str; 5] = ["tests", "code", "docs", "generated", "other"];

/// `(files, added, removed)` per class, from `git diff --numstat` (binary files count 0 lines).
fn size(pr: &Pr) -> Result<Vec<(&'static str, u64, u64, u64)>, String> {
    size_rows(pr.root, &pr.merge_base, &pr.head)
}

/// The `size: …` line `pr-check` prints, for the diff `merge_base..head` in `root`.
pub(crate) fn size_line(root: &Path, merge_base: &str, head: &str) -> Result<String, String> {
    Ok(size_text(&size_rows(root, merge_base, head)?))
}

fn size_text(rows: &[(&'static str, u64, u64, u64)]) -> String {
    let parts: Vec<String> = rows
        .iter()
        .map(|(c, f, a, d)| format!("{c} {f} file(s) +{a}/-{d}"))
        .collect();
    format!("size: {}", parts.join(", "))
}

fn size_rows(
    root: &Path,
    merge_base: &str,
    head: &str,
) -> Result<Vec<(&'static str, u64, u64, u64)>, String> {
    let raw = git(
        root,
        &[
            "diff",
            "--ignore-submodules=none",
            "-z",
            "--numstat",
            "-M",
            merge_base,
            head,
        ],
    )?;
    let fields = z_fields(&raw);
    let mut out: Vec<(&'static str, u64, u64, u64)> =
        CLASSES.iter().map(|c| (*c, 0, 0, 0)).collect();
    let mut i = 0;
    while i < fields.len() {
        let mut cols = fields[i].splitn(3, '\t');
        let add = cols.next().unwrap_or("-").parse::<u64>().unwrap_or(0);
        let del = cols.next().unwrap_or("-").parse::<u64>().unwrap_or(0);
        let path = cols.next().unwrap_or("");
        // A rename prints an empty path, then the old and the new path as their own fields.
        let (path, step) = if path.is_empty() {
            (fields.get(i + 2).map(String::as_str).unwrap_or(""), 3)
        } else {
            (path, 1)
        };
        let class = size_class(path);
        let row = out
            .iter_mut()
            .find(|r| r.0 == class)
            .expect("a known class");
        row.1 += 1;
        row.2 += add;
        row.3 += del;
        i += step;
    }
    Ok(out)
}

/// Tests only: names a JSON file that replaces `gh` for the two GitHub-backed rules.
pub const PRCHECK_GH_FAKE_ENV: &str = "UOR_PRCHECK_GH_FAKE";

/// One PR comment: its author's login and its body.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Comment {
    pub author: String,
    pub body: String,
}

/// One check run: its id (GitHub numbers them in creation order), its name, the slug of the app
/// that posted it and its output summary.
#[derive(Debug, Clone)]
pub struct CheckRun {
    pub id: u64,
    pub name: String,
    pub app_slug: String,
    pub summary: String,
}

/// A check run as the fake file writes it; a missing `id` is its position in the list.
#[derive(serde::Deserialize)]
struct FakeCheckRun {
    #[serde(default)]
    id: Option<u64>,
    name: String,
    app: String,
    #[serde(default)]
    summary: String,
}

/// One commit status: its context and state.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Status {
    pub context: String,
    pub state: String,
}

/// Every GitHub read pr-check makes; it never writes.
pub trait PrGh {
    /// The issue comments on PR `pr`, oldest first.
    fn comments(&self, pr: u64) -> Result<Vec<Comment>, String>;
    /// Every check run on commit `sha`, every page read.
    fn check_runs(&self, sha: &str) -> Result<Vec<CheckRun>, String>;
    /// Every commit status on commit `sha`, every page read.
    fn statuses(&self, sha: &str) -> Result<Vec<Status>, String>;
    /// Whether the PR head comes from a fork; the render sentinel path requires `false`.
    fn is_cross_repository(&self, pr: u64) -> Result<bool, String>;
}

/// How the GitHub-backed rules run: `--pr <n>` reads GitHub, `--local` and `--ci` skip them,
/// and none of the three fails them.
pub enum Github {
    Missing,
    Local,
    Ci,
    Pr(u64, Box<dyn PrGh>),
}

fn comments_args(pr: u64) -> Vec<String> {
    vec![
        "api".into(),
        format!("repos/{{owner}}/{{repo}}/issues/{pr}/comments"),
        "--paginate".into(),
    ]
}

fn check_runs_args(sha: &str) -> Vec<String> {
    vec![
        "api".into(),
        format!("repos/{{owner}}/{{repo}}/commits/{sha}/check-runs?per_page=100"),
        "--paginate".into(),
    ]
}

fn statuses_args(sha: &str) -> Vec<String> {
    vec![
        "api".into(),
        format!("repos/{{owner}}/{{repo}}/commits/{sha}/statuses?per_page=100"),
        "--paginate".into(),
    ]
}

/// `gh`, run from the repository root with every `GIT_*` variable removed.
pub struct RealPrGh {
    pub root: std::path::PathBuf,
}

impl RealPrGh {
    fn api(&self, args: &[String]) -> Result<Vec<serde_json::Value>, String> {
        let mut cmd = Command::new("gh");
        for (k, _) in std::env::vars_os() {
            if k.to_string_lossy().starts_with("GIT_") {
                cmd.env_remove(k);
            }
        }
        let out = cmd
            .args(args)
            .current_dir(&self.root)
            .output()
            .map_err(|e| format!("gh {}: {e}", args.join(" ")))?;
        if !out.status.success() {
            return Err(format!(
                "gh {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        // `--paginate` prints one JSON document per page.
        let text = String::from_utf8_lossy(&out.stdout);
        serde_json::Deserializer::from_str(&text)
            .into_iter::<serde_json::Value>()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("gh {} printed unreadable JSON: {e}", args.join(" ")))
    }
}

fn str_at<'v>(v: &'v serde_json::Value, path: &[&str]) -> &'v str {
    path.iter()
        .try_fold(v, |v, k| v.get(k))
        .and_then(|v| v.as_str())
        .unwrap_or("")
}

impl PrGh for RealPrGh {
    fn is_cross_repository(&self, pr: u64) -> Result<bool, String> {
        let out = crate::loopclaim::run_gh(
            &self.root,
            &["pr", "view", &pr.to_string(), "--json", "isCrossRepository"],
        )?;
        let v: serde_json::Value = serde_json::from_str(&out)
            .map_err(|e| format!("gh pr view {pr} printed unreadable JSON: {e}"))?;
        v.get("isCrossRepository")
            .and_then(|b| b.as_bool())
            .ok_or_else(|| format!("gh pr view {pr} printed no isCrossRepository"))
    }
    fn comments(&self, pr: u64) -> Result<Vec<Comment>, String> {
        let pages = self.api(&comments_args(pr))?;
        Ok(pages
            .iter()
            .filter_map(|p| p.as_array())
            .flatten()
            .map(|c| Comment {
                author: str_at(c, &["user", "login"]).to_string(),
                body: str_at(c, &["body"]).to_string(),
            })
            .collect())
    }

    fn check_runs(&self, sha: &str) -> Result<Vec<CheckRun>, String> {
        let pages = self.api(&check_runs_args(sha))?;
        Ok(pages
            .iter()
            .filter_map(|p| p.get("check_runs")?.as_array())
            .flatten()
            .map(|r| CheckRun {
                id: r.get("id").and_then(serde_json::Value::as_u64).unwrap_or(0),
                name: str_at(r, &["name"]).to_string(),
                app_slug: str_at(r, &["app", "slug"]).to_string(),
                summary: str_at(r, &["output", "summary"]).to_string(),
            })
            .collect())
    }

    fn statuses(&self, sha: &str) -> Result<Vec<Status>, String> {
        let pages = self.api(&statuses_args(sha))?;
        Ok(pages
            .iter()
            .filter_map(|p| p.as_array())
            .flatten()
            .map(|s| Status {
                context: str_at(s, &["context"]).to_string(),
                state: str_at(s, &["state"]).to_string(),
            })
            .collect())
    }
}

/// The in-memory stand-in for `gh`, read from the file `UOR_PRCHECK_GH_FAKE` names; it answers
/// the same lists for every PR and sha.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FakePrGh {
    #[serde(default, rename = "is_cross_repository")]
    is_cross_repository: bool,
    #[serde(default)]
    comments: Vec<Comment>,
    #[serde(default)]
    check_runs: Vec<FakeCheckRun>,
    #[serde(default)]
    statuses: Vec<Status>,
}

impl FakePrGh {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "cannot read {PRCHECK_GH_FAKE_ENV} file {}: {e}",
                path.display()
            )
        })?;
        serde_json::from_str(&text).map_err(|e| {
            format!(
                "{PRCHECK_GH_FAKE_ENV} file {} is malformed: {e}",
                path.display()
            )
        })
    }
}

impl PrGh for FakePrGh {
    fn is_cross_repository(&self, _pr: u64) -> Result<bool, String> {
        Ok(self.is_cross_repository)
    }
    fn comments(&self, _pr: u64) -> Result<Vec<Comment>, String> {
        Ok(self.comments.clone())
    }
    fn check_runs(&self, _sha: &str) -> Result<Vec<CheckRun>, String> {
        Ok(self
            .check_runs
            .iter()
            .enumerate()
            .map(|(i, r)| CheckRun {
                id: r.id.unwrap_or(i as u64),
                name: r.name.clone(),
                app_slug: r.app.clone(),
                summary: r.summary.clone(),
            })
            .collect())
    }
    fn statuses(&self, _sha: &str) -> Result<Vec<Status>, String> {
        Ok(self.statuses.clone())
    }
}

/// The PR number and the GitHub reader, or the rule's N/A or FAIL when the mode skips GitHub.
fn github<'p>(pr: &'p Pr, rule: &'static str) -> Result<(u64, &'p dyn PrGh), RuleResult> {
    match &pr.github {
        Github::Pr(n, gh) => Ok((*n, gh.as_ref())),
        Github::Local => Err(not_applicable(rule, "local run")),
        Github::Ci => Err(not_applicable(
            rule,
            "the policy job runs before the review record and the checks exist",
        )),
        Github::Missing => Err(fail(rule, "needs --pr <n>, or --local".into())),
    }
}

/// The PR's `uor-review v1` records, judged by [`crate::loopreview::gate`]: it passes when a
/// record counts and no counted CHANGES stands (each lab's last counted record is an APPROVE;
/// another lab's APPROVE does not clear a CHANGES). What the gate reads
/// about the repository comes from the merge base (the build-plan rows of `docs/loop/README.md`,
/// the vendors of `docs/harness/labs.json`, D59 from `docs/harness/owner-paths.json` and the
/// switch); the ORDER lab, item and `Setup row:` come from the PR's ORDER at the head; the PR is
/// doc-only when every changed path is `docs` or `generated`. Each record's line, and why it
/// counted or not, is a detail line. Free-text records no longer count. The `[<lab>]` label is
/// self-declared: every lab posts from the same GitHub account (loop page, P-3/P-4), so the
/// comment author cannot bind it.
fn rule_review_record(pr: &Pr) -> RuleResult {
    const R: &str = "review-record";
    let (number, gh) = match github(pr, R) {
        Ok(g) => g,
        Err(r) => return r,
    };
    let ctx = match ReviewContext::of(pr) {
        Ok(c) => c,
        Err(e) => return fail(R, e),
    };
    let comments: Vec<String> = match gh.comments(number) {
        Ok(c) => c.into_iter().map(|c| c.body).collect(),
        Err(e) => return fail(R, e),
    };
    let outcome = crate::loopreview::gate(&ctx.input(number, &pr.head, &comments));
    let (verdict, summary) = if outcome.pass {
        (
            Verdict::Pass,
            format!("a counted uor-review v1 APPROVE at {} stands", pr.head),
        )
    } else {
        (
            Verdict::Fail,
            format!("no standing counted uor-review v1 APPROVE at {}", pr.head),
        )
    };
    RuleResult {
        rule: R,
        verdict,
        summary,
        details: outcome.details,
    }
}

/// Everything [`crate::loopreview::gate`] reads about a PR except its comments, read as
/// `review-record` reads it; `loop queue` builds the same input to find the counted APPROVE.
pub struct ReviewContext {
    pub order: String,
    pub order_lab: String,
    pub item: String,
    pub setup_row: Option<String>,
    pub rows: Vec<String>,
    pub vendors: std::collections::BTreeMap<String, String>,
    pub d59_in_force: bool,
    pub doc_only: bool,
}

impl ReviewContext {
    /// The context, or the reason `review-record` fails before it reads any comment.
    pub fn of(pr: &Pr) -> Result<ReviewContext, String> {
        let page = pr
            .at_base("docs/loop/README.md")
            .ok_or("docs/loop/README.md is missing at the merge base")?;
        let vendors = pr
            .at_base("docs/harness/labs.json")
            .ok_or("docs/harness/labs.json is missing at the merge base".to_string())
            .and_then(|t| crate::labs::parse_labs(&t))?
            .into_iter()
            .map(|(k, l)| (k, l.vendor))
            .collect();
        // Unreadable owner paths or a missing switch keep D59 in force.
        let d59_in_force = crate::loopreview::d59_in_force(
            &pr.at_base("docs/harness/owner-paths.json")
                .unwrap_or_default(),
            pr.at_base(crate::loopreview::D59_SWITCH).as_deref(),
        );
        let rows = crate::loopreview::build_rows(&page);
        let doc_only = crate::loopreview::doc_only(pr.changed_paths().map(|(_, p)| p));
        // The queue mirror learns fork provenance from the same GitHub handle it already has;
        // an unreadable source leaves it unknown, which the sentinel path refuses.
        let cross = pr.is_cross_repository.or_else(|| match &pr.github {
            Github::Pr(n, gh) => gh.is_cross_repository(*n).ok(),
            _ => None,
        });
        let order = match pr.the_order() {
            Ok(o) => o.to_string(),
            Err(e) => {
                // The render PR carries no ORDER by design; its record uses the sentinel path.
                let paths: Vec<String> = pr.changed_paths().map(|(_, p)| p.to_string()).collect();
                if crate::loopreview::render_sentinel(&pr.head_ref, &paths, cross) {
                    return Ok(ReviewContext {
                        order: crate::loopreview::RENDER_ORDER.to_string(),
                        order_lab: "kimi".to_string(),
                        item: String::new(),
                        setup_row: None,
                        rows,
                        vendors,
                        d59_in_force,
                        doc_only,
                    });
                }
                return Err(format!("cannot check the review record: {e}"));
            }
        };
        let (_, order_lab, _) = order_parts(&order).expect("an added ORDER path parses");
        let text = pr
            .at_head(&order)
            .ok_or_else(|| format!("{order} is unreadable at the head"))?;
        let item = crate::loopclaim::order_item(&text)
            .ok_or_else(|| format!("{order} names no item at the head"))?;
        Ok(ReviewContext {
            order,
            order_lab,
            item,
            setup_row: crate::loopreview::setup_row(&text),
            rows,
            vendors,
            d59_in_force,
            doc_only,
        })
    }
    /// The gate's input for PR `number` at `head` with `comments` (bodies, oldest first).
    pub fn input<'a>(
        &'a self,
        number: u64,
        head: &'a str,
        comments: &'a [String],
    ) -> crate::loopreview::GateInput<'a> {
        crate::loopreview::GateInput {
            pr: number,
            head,
            order_path: &self.order,
            order_lab: &self.order_lab,
            order_item: &self.item,
            setup_row: self.setup_row.as_deref(),
            rows: &self.rows,
            vendors: &self.vendors,
            d59_in_force: self.d59_in_force,
            doc_only: self.doc_only,
            comments,
        }
    }
}

/// The required checks, copied from the branch protection of `main`.
const REQUIRED_CHECKS: [&str; 5] = [
    "fmt / clippy / tests / no_std / κ",
    "cargo audit",
    "fuzz smoke (nightly libfuzzer)",
    "wasm-pack build (dashboard facade, deploy.yml parity)",
    "Gate C trend alarm",
];

/// For every required name, the newest check run at the head (the highest id, from any app) is
/// from the `github-actions` app and its summary ends `policy: pass (run <n>)`, and no commit status uses a required name, because any token
/// can post a status (loop page, P-4). The summary format is written by `required-transport` in
/// `.github/workflows/ci.yml`, which loop PR 5's commit 4 changes.
fn rule_required_checks(pr: &Pr) -> RuleResult {
    const R: &str = "required-checks";
    let (_, gh) = match github(pr, R) {
        Ok(g) => g,
        Err(r) => return r,
    };
    let statuses = match gh.statuses(&pr.head) {
        Ok(s) => s,
        Err(e) => return fail(R, e),
    };
    let runs = match gh.check_runs(&pr.head) {
        Ok(r) => r,
        Err(e) => return fail(R, e),
    };
    let policy = regex::Regex::new(r"policy: pass \(run \d+\)$").expect("a valid pattern");
    let mut details = Vec::new();
    for s in &statuses {
        if REQUIRED_CHECKS.contains(&s.context.as_str()) {
            details.push(format!(
                "a commit status named `{}` ({}) exists; a required name must come from a check run alone",
                s.context, s.state
            ));
        }
    }
    for name in REQUIRED_CHECKS {
        let newest = runs.iter().filter(|r| r.name == name).max_by_key(|r| r.id);
        match newest {
            None => details.push(format!("`{name}` has no check run at the head")),
            Some(r)
                if r.app_slug == "github-actions" && policy.is_match(r.summary.trim_end()) => {}
            Some(r) => details.push(format!(
                "`{name}`: the newest check run (id {}, app {}) is not a github-actions run whose summary ends `policy: pass (run <n>)`",
                r.id, r.app_slug
            )),
        }
    }
    result(
        R,
        details,
        &format!(
            "all {} required checks pass the policy",
            REQUIRED_CHECKS.len()
        ),
        "a required check is missing or not from the policy job",
    )
}

/// The branch `head` names (`refs/heads/<x>` or `refs/remotes/<x>`), else the current branch,
/// else `""`.
fn default_head_ref(root: &Path, head: &str) -> String {
    for prefix in ["refs/heads/", "refs/remotes/"] {
        let r = format!("{prefix}{head}");
        if git(root, &["rev-parse", "--verify", "--quiet", &r]).is_ok() {
            return head.to_string();
        }
    }
    git(root, &["symbolic-ref", "--short", "-q", "HEAD"])
        .map(|b| String::from_utf8_lossy(&b).trim().to_string())
        .unwrap_or_default()
}

/// Every rule in [`RULES`] run against `pr`, in order; `loop queue` runs the same list.
pub fn evaluate(pr: &Pr) -> Vec<RuleResult> {
    RULES
        .iter()
        .map(|(name, rule)| {
            let r = rule(pr);
            debug_assert_eq!(&r.rule, name);
            r
        })
        .collect()
}

/// `loop pr-check [--base <rev>] [--head <rev>] [--head-ref <branch>] [--pr <n> | --local | --ci]
/// [--json]`. Exit 0 every rule passes, 1 a rule failed, 2 the check could not run.
pub fn cli(root: &Path, args: &[String]) -> u8 {
    let (mut base, mut head, mut json) = ("origin/main".to_string(), "HEAD".to_string(), false);
    let mut head_ref: Option<String> = None;
    let (mut pr_number, mut modes): (Option<u64>, Vec<&str>) = (None, Vec::new());
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            m @ ("--local" | "--ci") => modes.push(m),
            "--pr" => {
                let Some(n) = args.get(i + 1).and_then(|v| v.parse::<u64>().ok()) else {
                    eprintln!("{USAGE}");
                    return 2;
                };
                pr_number = Some(n);
                modes.push("--pr");
                i += 1;
            }
            flag @ ("--base" | "--head" | "--head-ref") => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("{USAGE}");
                    return 2;
                };
                match flag {
                    "--base" => base = v.clone(),
                    "--head" => head = v.clone(),
                    _ => head_ref = Some(v.clone()),
                }
                i += 1;
            }
            _ => {
                eprintln!("{USAGE}");
                return 2;
            }
        }
        i += 1;
    }
    let mut pr = match Pr::open(root, &base, &head) {
        Ok(pr) => pr,
        Err(e) => {
            eprintln!("uor-harness: loop pr-check: {e}");
            return 2;
        }
    };
    pr.head_ref = head_ref.unwrap_or_else(|| default_head_ref(root, &head));
    let (github, cross) = match (modes.as_slice(), pr_number) {
        ([], _) => (Github::Missing, None),
        (["--local"], _) => (Github::Local, None),
        (["--ci"], _) => (Github::Ci, None),
        (["--pr"], Some(n)) => {
            let gh: Box<dyn PrGh> = match std::env::var_os(PRCHECK_GH_FAKE_ENV) {
                Some(p) => match FakePrGh::from_file(Path::new(&p)) {
                    Ok(f) => Box::new(f),
                    Err(e) => {
                        eprintln!("uor-harness: loop pr-check: {e}");
                        return 2;
                    }
                },
                None => Box::new(RealPrGh {
                    root: root.to_path_buf(),
                }),
            };
            let cross = gh.is_cross_repository(n).ok();
            (Github::Pr(n, gh), cross)
        }
        _ => {
            eprintln!("uor-harness: loop pr-check: give at most one of --pr <n>, --local, --ci");
            return 2;
        }
    };
    pr.github = github;
    pr.is_cross_repository = cross;
    let size = match size(&pr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("uor-harness: loop pr-check: {e}");
            return 2;
        }
    };
    let results = evaluate(&pr);
    let failed = results.iter().any(|r| r.verdict == Verdict::Fail);
    let fake = std::env::var_os(PRCHECK_GH_FAKE_ENV);
    if json {
        let size_obj: serde_json::Map<String, serde_json::Value> = size
            .iter()
            .map(|(c, f, a, d)| {
                (
                    c.to_string(),
                    serde_json::json!({"files": f, "added": a, "removed": d}),
                )
            })
            .collect();
        let rules: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "rule": r.rule,
                    "verdict": r.verdict.text().to_lowercase(),
                    "summary": r.summary,
                    "details": r.details,
                })
            })
            .collect();
        let out = serde_json::json!({
            "base": base,
            "merge_base": pr.merge_base,
            "head": pr.head,
            "verdict": if failed { "fail" } else { "pass" },
            "fake_github": fake.is_some(),
            "size": size_obj,
            "rules": rules,
        });
        println!("{out}");
    } else {
        println!("{}", size_text(&size));
        if let Some(p) = &fake {
            println!(
                "FAKE GitHub: {PRCHECK_GH_FAKE_ENV}={} (tests only)",
                p.to_string_lossy()
            );
        }
        for r in &results {
            println!("{} {} {}", r.verdict.text(), r.rule, r.summary);
            for d in &r.details {
                println!("  {d}");
            }
        }
        println!("verdict: {}", if failed { "FAIL" } else { "PASS" });
    }
    u8::from(failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn o(folder: &str, dir: &str) -> String {
        format!("docs/loop/tasks/{folder}/{dir}/ORDER.md")
    }

    #[test]
    fn active_order_is_the_highest_of_one_lab_in_one_folder() {
        let one = [o("f", "kimi-1")];
        assert_eq!(active_order(&one), Ok(o("f", "kimi-1")));
        let three = [o("f", "kimi-2"), o("f", "kimi-3"), o("f", "kimi-1")];
        assert_eq!(active_order(&three), Ok(o("f", "kimi-3")));
        assert_eq!(
            order_chain(&three).unwrap(),
            vec![o("f", "kimi-1"), o("f", "kimi-2"), o("f", "kimi-3")]
        );
        let numeric = [o("f", "kimi-10"), o("f", "kimi-9")];
        assert_eq!(active_order(&numeric), Ok(o("f", "kimi-10")));
        let labs = active_order(&[o("f", "kimi-1"), o("f", "deepseek-1")]).unwrap_err();
        assert!(labs.contains("labs: deepseek, kimi"), "{labs}");
        let folders = active_order(&[o("f", "kimi-1"), o("g", "kimi-2")]).unwrap_err();
        assert!(folders.contains("folders: f, g"), "{folders}");
        let same_n = active_order(&[o("f", "kimi-01"), o("f", "kimi-1")]).unwrap_err();
        assert!(same_n.contains("two with the same number"), "{same_n}");
        let none: [String; 0] = [];
        assert!(active_order(&none)
            .unwrap_err()
            .starts_with("no docs/loop/tasks/"));
    }

    #[test]
    fn gh_endpoints() {
        assert_eq!(
            comments_args(7),
            [
                "api",
                "repos/{owner}/{repo}/issues/7/comments",
                "--paginate"
            ]
        );
        let sha = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            check_runs_args(sha),
            [
                "api",
                &format!("repos/{{owner}}/{{repo}}/commits/{sha}/check-runs?per_page=100"),
                "--paginate"
            ]
        );
        assert_eq!(
            statuses_args(sha),
            [
                "api",
                &format!("repos/{{owner}}/{{repo}}/commits/{sha}/statuses?per_page=100"),
                "--paginate"
            ]
        );
    }

    #[test]
    fn size_classes() {
        assert_eq!(size_class("crates/h/tests/x.rs"), "tests");
        assert_eq!(size_class("crates/h/src/foo_test.rs"), "tests");
        assert_eq!(size_class("crates/h/src/lib.rs"), "code");
        assert_eq!(size_class("Cargo.lock"), "generated");
        assert_eq!(size_class("docs/roadmap/README.md"), "generated");
        assert_eq!(size_class("docs/a.md"), "docs");
        assert_eq!(size_class("docs/a.json"), "other");
    }
}
