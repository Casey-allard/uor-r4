//! `uor-harness loop pr-check`: the rules a loop PR must pass, read from git alone.
//!
//! The PR is the diff from `git merge-base <base> <head>` to `<head>`. Registries, items and
//! budgets are read at the merge base, never at the head, so a PR cannot loosen its own check;
//! the files the PR adds or changes are read at the head. Nothing reads the working tree; only
//! `review-record` and `required-checks` read GitHub, through the read-only [`PrGh`] and only
//! with `--pr <n>`. A rule is one function plus one entry in [`RULES`].

use std::path::Path;
use std::process::Command;

const USAGE: &str =
    "usage: uor-harness loop pr-check [--base <rev>] [--head <rev>] [--head-ref <branch>] [--pr <n> | --local | --ci] [--json]";

/// Every rule, in the order it runs and prints.
const RULES: &[(&str, Rule)] = &[
    ("order", rule_order),
    ("scope", rule_scope),
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
            || (item.notebook.as_deref() == Some(path) && append_only(pr, path))
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

/// The head text starts with the base text; a page absent at the base counts as empty.
fn append_only(pr: &Pr, path: &str) -> bool {
    let Some(h) = pr.at_head(path) else {
        return false;
    };
    h.starts_with(&pr.at_base(path).unwrap_or_default())
}

/// Every base line is still present at the head, in order (rows added only).
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
/// counted record is an APPROVE and the last counted record is an APPROVE. What the gate reads
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
        let order = pr
            .the_order()
            .map_err(|e| format!("cannot check the review record: {e}"))?;
        let (_, order_lab, _) = order_parts(order).expect("an added ORDER path parses");
        let text = pr
            .at_head(order)
            .ok_or_else(|| format!("{order} is unreadable at the head"))?;
        let item = crate::loopclaim::order_item(&text)
            .ok_or_else(|| format!("{order} names no item at the head"))?;
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
        Ok(ReviewContext {
            order: order.to_string(),
            order_lab,
            item,
            setup_row: crate::loopreview::setup_row(&text),
            rows: crate::loopreview::build_rows(&page),
            vendors,
            d59_in_force,
            doc_only: crate::loopreview::doc_only(pr.changed_paths().map(|(_, p)| p)),
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
    pr.github = match (modes.as_slice(), pr_number) {
        ([], _) => Github::Missing,
        (["--local"], _) => Github::Local,
        (["--ci"], _) => Github::Ci,
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
            Github::Pr(n, gh)
        }
        _ => {
            eprintln!("uor-harness: loop pr-check: give at most one of --pr <n>, --local, --ci");
            return 2;
        }
    };
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
