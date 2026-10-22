//! `uor-harness loop claim <item-id> [--lab-topic <topic>]`: the refusal checks (loop PR 7a).
//!
//! **Why this exists.** Claims made with `team claim` checked neither that an item was ready nor
//! that two labs were about to edit the same files. The loop page claims an item by opening a
//! draft PR carrying its ORDER, and this module refuses the claim first when it must not happen.
//!
//! **The checks, in order.** Each refusal exits 2 with its reason. First `git fetch origin main`;
//! if it fails, nothing is checked or written. Every item is then read from `origin/main`
//! (`git show origin/main:docs/roadmap/items/<id>.md`), never from the working tree.
//! 1. The item exists on `origin/main` and its `status:` is `ready`.
//! 2. No open order names it. Open orders are the ORDER files changed on this lab's worktrees under
//!    the main checkout's `.worktrees/<lab>-*` whose commit is not in `origin/main`, plus the ORDER
//!    files changed in open PRs, read through `refs/pull/<n>/head`. A PR ORDER that cannot be read
//!    warns and counts as an open order of unknown item. An ORDER names its item in the `Item:`
//!    line link `roadmap/items/<id>.md`.
//! 3. The item has a `scope:` line. Without one the claim is refused; it never guesses. Each of its
//!    globs passes [`scope_glob_ok`] and covers none of the [`SHARED_FILES`].
//! 4. No scope glob overlaps a glob of an open order's item (see [`globs_overlap`]); that item's
//!    globs must pass [`scope_glob_ok`] too. An open order whose item is missing or has no `scope:`
//!    is skipped here with a printed warning (it still counts for check 2). Globs under
//!    `docs/research/notebook/` never count, nor does a glob that is exactly one of the
//!    [`SHARED_FILES`].
//! 5. The lab's open PRs (head branch `<lab>/...`) are fewer than its `open_pr_limit` in
//!    `docs/harness/labs.json`.
//!
//! A worktree's order also stops being open when its ORDER is on `origin/main` byte for byte,
//! since a squash-merged branch is never an ancestor of `main`.
//!
//! **The writes** (ORDER call 5) happen only after every check has passed, from the `origin/main`
//! the checks read, in this order: `git worktree add .worktrees/<lab>-<topic> -b <lab>/<topic>
//! origin/main`, under the main checkout even when the claim runs from a linked worktree; the
//! ORDER `docs/loop/tasks/<item>/<lab>-<n>/ORDER.md` committed there; a push leased on the branch
//! being absent from `origin`; a draft PR; a re-list. A branch of that name already on `origin`,
//! or an `origin` that cannot be checked, refuses before any write. Any failure after `worktree
//! add`, and a lost race (the re-list shows a lower-numbered open PR, draft or not, whose ORDER
//! names the same item), runs one rollback, which attempts every step: close our PR; delete our
//! branch from `origin`, leased on the commit we pushed, so a branch someone else moved is never
//! deleted; remove our worktree and branch by exact path and name. Nothing else is touched. A
//! clean lost race exits 1; a failure, or a rollback that left anything, exits 2 and names what
//! is left.
//!
//! **GitHub.** All access goes through the [`Gh`] trait. The real one runs `gh`; tests point the
//! binary at a JSON file through `UOR_LOOP_GH_FAKE`.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

/// TEST-ONLY: a path to a JSON file standing in for `gh`. It holds either the array `gh pr list
/// --json number,headRefName,isDraft,files` prints, or an object `{"prs": [...], "nextPrNumber":
/// n, "afterCreate": [...]}`: `afterCreate` PRs join the list once a draft PR has been created,
/// and `nextPrNumber` is the number that draft PR gets. Each PR may add `"orderTexts": {path:
/// text}` for the ORDER files it changes. Each write is appended as one JSON line to
/// `<file>.calls.jsonl`. Never set it outside tests: it replaces GitHub entirely.
pub const GH_FAKE_ENV: &str = "UOR_LOOP_GH_FAKE";

const NOTEBOOK: &str = "docs/research/notebook/";

/// The shared registration files: any loop PR may change them, and no item's `scope:` may list
/// them. `pr-check`'s `scope` rule always allows them, and `loop claim` refuses an item whose scope
/// has a glob covering one. Casey's decisions in Claude's chat (ORDER
/// `docs/loop/tasks/infra-loop-harness/claude-22/ORDER.md`): 2026-10-21T23:43Z, "Exempt them
/// (Recommended)", for the first six; 2026-10-22T00:05Z, "Add all three (Recommended)", for
/// `loopprcheck.rs`, `gates.json` and `docs/roadmap/README.md`.
pub const SHARED_FILES: [&str; 9] = [
    "crates/uor-r4-harness/src/lib.rs",
    "crates/uor-r4-harness/src/main.rs",
    "crates/uor-r4-harness/src/loopgates.rs",
    "crates/uor-r4-harness/src/doccheck.rs",
    "crates/uor-r4-harness/src/loopprcheck.rs",
    "docs/harness/README.md",
    "docs/loop/README.md",
    "docs/harness/gates.json",
    "docs/roadmap/README.md",
];

/// The first shared file `glob` covers, matched as `pr-check`'s `scope` rule matches paths.
pub fn covered_shared_file(glob: &str) -> Option<&'static str> {
    SHARED_FILES
        .into_iter()
        .find(|f| crate::loopprcheck::glob_match(glob, f))
}

/// One file a PR changes, as `gh pr list --json files` prints it.
#[derive(Debug, Clone, Deserialize)]
pub struct PrFile {
    pub path: String,
}

/// One open PR, as `gh pr list --json number,headRefName,isDraft,files` prints it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPr {
    pub number: u64,
    pub head_ref_name: String,
    #[serde(default)]
    pub is_draft: bool,
    #[serde(default)]
    pub files: Vec<PrFile>,
    /// Fake only: ORDER texts keyed by path. `gh` never prints this field.
    #[serde(default)]
    pub order_texts: BTreeMap<String, String>,
}

impl OpenPr {
    /// The `docs/loop/tasks/**/ORDER.md` files this PR changes.
    pub fn order_paths(&self) -> Vec<&str> {
        self.files
            .iter()
            .map(|f| f.path.as_str())
            .filter(|p| is_order_path(p))
            .collect()
    }
}

/// Every GitHub call the claim makes.
pub trait Gh {
    /// Every open PR with its head branch and changed files.
    fn list_open_prs(&self) -> Result<Vec<OpenPr>, String>;
    /// The text of `path` on the PR's head branch.
    fn order_text(&self, pr: &OpenPr, path: &str) -> Result<String, String>;
    /// Open a draft PR against `main` from `branch`, which the claim has already pushed; returns
    /// the PR number. Pushing and deleting the branch are plain `git` calls, outside this trait.
    fn create_draft_pr(&self, branch: &str, title: &str, body: &str) -> Result<u64, String>;
    /// Close PR `number`.
    fn close_pr(&self, number: u64) -> Result<(), String>;
}

/// `gh`, run from the repository root.
pub struct RealGh {
    pub root: PathBuf,
}

/// The `GIT_*` variables among `names` that a child must not inherit: every one but those that only
/// say how to reach a remote. A `GIT_DIR` from a hook would otherwise point `gh` at another repository.
fn git_vars_to_strip(names: impl IntoIterator<Item = String>) -> Vec<String> {
    names
        .into_iter()
        .filter(|k| {
            k.starts_with("GIT_")
                && !["GIT_SSH_COMMAND", "GIT_SSH", "GIT_ASKPASS"].contains(&k.as_str())
        })
        .collect()
}

pub(crate) fn run_gh(root: &Path, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("gh");
    for k in git_vars_to_strip(std::env::vars_os().map(|(k, _)| k.to_string_lossy().into_owned())) {
        cmd.env_remove(k);
    }
    let out = cmd
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("gh {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "gh {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Percent-encode every byte of `s` outside `A-Za-z0-9-._~` and the bytes in `keep`.
fn percent_encode(s: &str, keep: &[u8]) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) || keep.contains(&b) {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// The contents API endpoint for `path` on PR `pr`'s head, read through `refs/pull/<pr>/head` so a
/// PR from a fork, or a branch name that needs escaping, still resolves.
pub(crate) fn contents_endpoint(path: &str, pr: u64) -> String {
    format!(
        "repos/{{owner}}/{{repo}}/contents/{}?ref={}",
        percent_encode(path, b"/"),
        percent_encode(&format!("refs/pull/{pr}/head"), b"")
    )
}

impl Gh for RealGh {
    fn list_open_prs(&self) -> Result<Vec<OpenPr>, String> {
        let text = run_gh(
            &self.root,
            &[
                "pr",
                "list",
                "--state",
                "open",
                "--limit",
                "1000",
                "--json",
                "number,headRefName,isDraft,files",
            ],
        )?;
        serde_json::from_str(&text).map_err(|e| format!("gh pr list printed unreadable JSON: {e}"))
    }

    fn order_text(&self, pr: &OpenPr, path: &str) -> Result<String, String> {
        let endpoint = contents_endpoint(path, pr.number);
        run_gh(
            &self.root,
            &[
                "api",
                "-H",
                "Accept: application/vnd.github.raw+json",
                &endpoint,
            ],
        )
    }

    fn create_draft_pr(&self, branch: &str, title: &str, body: &str) -> Result<u64, String> {
        let url = run_gh(
            &self.root,
            &[
                "pr", "create", "--draft", "--base", "main", "--head", branch, "--title", title,
                "--body", body,
            ],
        )?;
        url.trim()
            .rsplit('/')
            .next()
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| format!("gh pr create printed no PR URL: {}", url.trim()))
    }

    fn close_pr(&self, number: u64) -> Result<(), String> {
        run_gh(&self.root, &["pr", "close", &number.to_string()]).map(|_| ())
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FakeFile {
    List(Vec<OpenPr>),
    #[serde(rename_all = "camelCase")]
    Full {
        #[serde(default)]
        prs: Vec<OpenPr>,
        #[serde(default)]
        next_pr_number: Option<u64>,
        #[serde(default)]
        after_create: Vec<OpenPr>,
        /// When true, `create_draft_pr` records nothing and fails.
        #[serde(default, rename = "createFails")]
        create_fails: bool,
    },
}

/// The in-memory stand-in for `gh`, read from the file `UOR_LOOP_GH_FAKE` names.
pub struct FakeGh {
    pub prs: Vec<OpenPr>,
    pub after_create: Vec<OpenPr>,
    pub next_pr_number: u64,
    pub create_fails: bool,
    created: Cell<bool>,
    calls: RefCell<Option<std::fs::File>>,
    calls_path: PathBuf,
}

impl FakeGh {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {GH_FAKE_ENV} file {}: {e}", path.display()))?;
        let file = serde_json::from_str(&text)
            .map_err(|e| format!("{GH_FAKE_ENV} file {} is malformed: {e}", path.display()))?;
        let (prs, next, after_create, create_fails) = match file {
            FakeFile::List(prs) => (prs, None, Vec::new(), false),
            FakeFile::Full {
                prs,
                next_pr_number,
                after_create,
                create_fails,
            } => (prs, next_pr_number, after_create, create_fails),
        };
        let mut calls_path = path.as_os_str().to_owned();
        calls_path.push(".calls.jsonl");
        Ok(FakeGh {
            prs,
            after_create,
            next_pr_number: next.unwrap_or(1),
            create_fails,
            created: Cell::new(false),
            calls: RefCell::new(None),
            calls_path: calls_path.into(),
        })
    }

    fn record(&self, call: serde_json::Value) -> Result<(), String> {
        let mut slot = self.calls.borrow_mut();
        if slot.is_none() {
            *slot = Some(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.calls_path)
                    .map_err(|e| format!("cannot open {}: {e}", self.calls_path.display()))?,
            );
        }
        let f = slot.as_mut().expect("opened above");
        writeln!(f, "{call}")
            .map_err(|e| format!("cannot write {}: {e}", self.calls_path.display()))
    }
}

impl Gh for FakeGh {
    fn list_open_prs(&self) -> Result<Vec<OpenPr>, String> {
        let mut prs = self.prs.clone();
        if self.created.get() {
            prs.extend(self.after_create.iter().cloned());
        }
        Ok(prs)
    }

    fn order_text(&self, pr: &OpenPr, path: &str) -> Result<String, String> {
        pr.order_texts
            .get(path)
            .cloned()
            .ok_or_else(|| format!("fake PR #{} has no text for {path}", pr.number))
    }

    fn create_draft_pr(&self, branch: &str, title: &str, body: &str) -> Result<u64, String> {
        if self.create_fails {
            return Err("the fake gh pr create failed".into());
        }
        self.record(serde_json::json!({
            "call": "create_draft_pr", "branch": branch, "title": title, "body": body,
            "number": self.next_pr_number
        }))?;
        self.created.set(true);
        Ok(self.next_pr_number)
    }

    fn close_pr(&self, number: u64) -> Result<(), String> {
        self.record(serde_json::json!({"call": "close_pr", "number": number}))
    }
}

fn is_order_path(p: &str) -> bool {
    p.starts_with("docs/loop/tasks/") && p.ends_with("/ORDER.md")
}

/// The frontmatter fields of a roadmap item that the claim reads.
#[derive(Debug, Default)]
pub struct ItemFields {
    pub status: Option<String>,
    /// `None` when the item has no `scope:` line.
    pub scope: Option<Vec<String>>,
    /// The item's notebook page, from an unquoted-or-quoted `notebook:` line.
    pub notebook: Option<String>,
    /// The ids in the inline-list `depends_on:` line; empty when there is none.
    pub depends_on: Vec<String>,
    /// Why the `depends_on:` line is not an inline list. It is recorded, not an error, so that only
    /// `loop next`, which reads dependencies, acts on it.
    pub depends_on_error: Option<String>,
}

/// The entries of an inline `[a, b]` list, or `None` when `v` is not one.
fn inline_list(v: &str) -> Option<Vec<String>> {
    let inner = v.trim().strip_prefix('[')?.strip_suffix(']')?;
    Some(
        inner
            .split(',')
            .map(|s| unquote(s).to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

fn unquote(s: &str) -> &str {
    let s = s.trim();
    for q in ['"', '\''] {
        if let Some(inner) = s.strip_prefix(q).and_then(|v| v.strip_suffix(q)) {
            return inner;
        }
    }
    s
}

/// Read `status:`, `notebook:` and the inline-list `scope:` and `depends_on:` from an item's `---`
/// frontmatter.
pub fn parse_item(text: &str) -> Result<ItemFields, String> {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Err("has no `---` frontmatter".into());
    }
    let mut fields = ItemFields::default();
    for line in lines {
        if line.trim() == "---" {
            return Ok(fields);
        }
        if let Some(v) = line.strip_prefix("status:") {
            fields.status = Some(unquote(v).to_string());
        } else if let Some(v) = line.strip_prefix("notebook:") {
            fields.notebook = Some(unquote(v).to_string());
        } else if let Some(v) = line.strip_prefix("scope:") {
            fields.scope = Some(
                inline_list(v)
                    .ok_or_else(|| format!("scope `{}` is not a `[a, b]` list", v.trim()))?,
            );
        } else if let Some(v) = line.strip_prefix("depends_on:") {
            match inline_list(v) {
                Some(ids) => fields.depends_on = ids,
                None => {
                    fields.depends_on_error =
                        Some(format!("depends_on `{}` is not a `[a, b]` list", v.trim()))
                }
            }
        }
    }
    Err("frontmatter is not closed by `---`".into())
}

/// The item parsed from `origin/main`, the same source `check` reads; never the working tree.
pub(crate) fn read_item(root: &Path, id: &str) -> Result<ItemFields, String> {
    read_item_opt(root, id)
        .map_err(|e| format!("item {id}: {e}"))?
        .ok_or_else(|| format!("item {id} does not exist on origin/main"))
}

/// The item parsed from `origin/main`: `Ok(None)` when it is not there, `Err` when it does not parse.
pub(crate) fn read_item_opt(root: &Path, id: &str) -> Result<Option<ItemFields>, String> {
    main_item_text(root, id).map(|t| parse_item(&t)).transpose()
}

/// The item's text on `origin/main`, or `None` when it is not there.
fn main_item_text(root: &Path, id: &str) -> Option<String> {
    let spec = format!("origin/main:docs/roadmap/items/{id}.md");
    crate::loopgates::git(root, &["show", &spec])
        .ok()
        .map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// A scope glob both `loop claim` and `loop suborder` accept: not empty, relative, no `.` or `..`
/// segment (so no `./` prefix), no `//`, and none of `[ ] { } \`. Overlap and containment are
/// decided on the literal text, so anything that could name a path two ways is refused.
pub fn scope_glob_ok(glob: &str) -> Result<(), String> {
    let why = if glob.is_empty() {
        "is empty".to_string()
    } else if glob.starts_with('/') {
        "starts with /".to_string()
    } else if glob.split('/').any(|s| s == "." || s == "..") {
        "has a . or .. segment".to_string()
    } else if glob.contains("//") {
        "contains //".to_string()
    } else if let Some(c) = glob.chars().find(|c| "[]{}\\".contains(*c)) {
        format!("contains {c}")
    } else {
        return Ok(());
    };
    Err(format!("scope glob `{glob}` {why}"))
}

/// The item id an ORDER names: the `roadmap/items/<id>.md` link on its `Item:` line.
pub fn order_item(order_text: &str) -> Option<String> {
    let re = regex::Regex::new(r"roadmap/items/([A-Za-z0-9._-]+)\.md").expect("a valid pattern");
    let line = order_text.lines().find(|l| l.contains("Item:"))?;
    let after = &line[line.find("Item:")?..];
    re.captures(after).map(|c| c[1].to_string())
}

/// An order that is open: where it was found and the item it names, `None` when its ORDER could
/// not be read.
#[derive(Debug, Clone)]
pub struct OpenOrder {
    pub source: String,
    pub item: Option<String>,
    /// The PR carrying the order, when it was found on a PR.
    pub pr: Option<u64>,
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    crate::loopgates::git(root, args).map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// The main checkout of the repository at `root`: the parent of its common git directory, so a
/// claim run from a linked worktree still places and finds worktrees under the main checkout.
pub(crate) fn main_checkout(root: &Path) -> Result<PathBuf, String> {
    let common = git(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let common = PathBuf::from(common.trim());
    common
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| format!("git common dir {} has no parent", common.display()))
}

/// `a` and `b` name the same directory: equal as given, or equal once both canonicalise (macOS
/// `/var` is a symlink to `/private/var`).
fn same_dir(a: &Path, b: &Path) -> bool {
    a == b
        || matches!(
            (a.canonicalize(), b.canonicalize()),
            (Ok(x), Ok(y)) if x == y
        )
}

/// The ORDER files changed on this lab's worktrees under `worktrees` (the main checkout's
/// `.worktrees`) whose commit is not in `origin/main`.
fn worktree_orders(root: &Path, worktrees: &Path, lab: &str) -> Result<Vec<OpenOrder>, String> {
    let list = git(root, &["worktree", "list", "--porcelain"])?;
    let prefix = format!("{}-", crate::state::slug(lab));
    let mut orders = Vec::new();
    for block in list.split("\n\n") {
        let path = block.lines().find_map(|l| l.strip_prefix("worktree "));
        let head = block.lines().find_map(|l| l.strip_prefix("HEAD "));
        let (Some(path), Some(head)) = (path, head) else {
            continue;
        };
        let p = Path::new(path);
        let in_worktrees = p.parent().is_some_and(|d| same_dir(d, worktrees));
        let ours = p
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(&prefix));
        if !in_worktrees || !ours {
            continue;
        }
        let merged = Command::new("git")
            .args(["merge-base", "--is-ancestor", head, "origin/main"])
            .current_dir(root)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .status()
            .map_err(|e| format!("git merge-base: {e}"))?
            .success();
        if merged {
            continue;
        }
        let range = format!("origin/main...{head}");
        let changed = git(root, &["diff", "--name-only", &range])?;
        for file in changed.lines().filter(|f| is_order_path(f)) {
            let bytes = crate::loopgates::git(root, &["show", &format!("{head}:{file}")])?;
            // Squash-merged: the same ORDER bytes are on origin/main.
            let on_main = crate::loopgates::git(root, &["show", &format!("origin/main:{file}")]);
            if on_main.as_deref() == Ok(bytes.as_slice()) {
                continue;
            }
            let text = String::from_utf8_lossy(&bytes);
            if let Some(item) = order_item(&text) {
                orders.push(OpenOrder {
                    source: format!("worktree {path} ({file})"),
                    item: Some(item),
                    pr: None,
                });
            }
        }
    }
    Ok(orders)
}

/// The orders on open PRs. An ORDER that cannot be read warns and counts as an open order of
/// unknown item, so one unreadable PR never blocks every claim.
fn pr_orders(gh: &dyn Gh, prs: &[OpenPr]) -> Vec<OpenOrder> {
    let mut orders = Vec::new();
    for pr in prs {
        for path in pr.order_paths() {
            let source = format!("PR #{} ({path})", pr.number);
            let item = match gh.order_text(pr, path) {
                Ok(text) => match order_item(&text) {
                    Some(item) => Some(item),
                    None => continue,
                },
                Err(e) => {
                    eprintln!(
                        "uor-harness: loop claim: warning: cannot read PR #{}'s ORDER {path}: {e}; treated as an open order of unknown item",
                        pr.number
                    );
                    None
                }
            };
            orders.push(OpenOrder {
                source,
                item,
                pr: Some(pr.number),
            });
        }
    }
    orders
}

/// The text of a glob before its first `*`, `?` or `[`, and whether a wildcard follows it.
fn literal(glob: &str) -> (&str, bool) {
    match glob.find(['*', '?', '[']) {
        Some(i) => (&glob[..i], true),
        None => (glob, false),
    }
}

/// `a`'s literal is a path prefix of `b`'s literal. When a wildcard follows `a`'s literal, a plain
/// string prefix is enough, since the wildcard can complete the segment.
fn prefix_of(a: &str, b: &str) -> bool {
    let (la, wild) = literal(a);
    let (lb, _) = literal(b);
    if !lb.starts_with(la) {
        return false;
    }
    wild || la.is_empty()
        || la.ends_with('/')
        || lb.len() == la.len()
        || lb[la.len()..].starts_with('/')
}

/// Two scope globs overlap when the literal part of one is a path prefix of the other's (ORDER call
/// 3). Conservative: it may refuse a claim that would not collide, but never misses one that would.
pub fn globs_overlap(a: &str, b: &str) -> bool {
    prefix_of(a, b) || prefix_of(b, a)
}

/// A glob counts for overlap unless it is under the notebook or is exactly a shared file.
fn counts(glob: &str) -> bool {
    !literal(glob).0.starts_with(NOTEBOOK) && !SHARED_FILES.contains(&glob)
}

/// A refusal: the reason printed after `loop claim: refused: `.
pub type Refusal = String;

/// Run every check for `item` claimed by `lab`, in order, stopping at the first refusal.
pub fn check(root: &Path, lab: &str, item: &str, gh: &dyn Gh) -> Result<(), Refusal> {
    // Fetch first: every check below reads origin/main.
    crate::loopgates::git(root, &["fetch", "-q", "origin", "main"])
        .map_err(|e| format!("cannot fetch origin main, so nothing was checked or written: {e}"))?;
    // (a) the item exists on origin/main and is ready.
    let text = main_item_text(root, item).ok_or_else(|| {
        format!("item {item} does not exist (no docs/roadmap/items/{item}.md on origin/main)")
    })?;
    let fields = parse_item(&text).map_err(|e| format!("item {item}: {e}"))?;
    match fields.status.as_deref() {
        Some("ready") => {}
        Some(s) => return Err(format!("item {item} has status {s:?}, not \"ready\"")),
        None => return Err(format!("item {item} has no status: line")),
    }
    // (b) no open order names it.
    let prs = gh
        .list_open_prs()
        .map_err(|e| format!("cannot list open PRs: {e}"))?;
    let mut orders = worktree_orders(root, &main_checkout(root)?.join(".worktrees"), lab)?;
    orders.extend(pr_orders(gh, &prs));
    if let Some(o) = orders.iter().find(|o| o.item.as_deref() == Some(item)) {
        return Err(format!("item {item} is already claimed by {}", o.source));
    }
    // (c) the item has a scope.
    let Some(scope) = fields.scope else {
        return Err(format!("item {item} has no scope: line"));
    };
    for g in &scope {
        scope_glob_ok(g).map_err(|e| format!("item {item}: {e}"))?;
        if let Some(f) = covered_shared_file(g) {
            return Err(format!(
                "item {item}: scope glob `{g}` covers the shared file {f}; Casey (21-22 Oct, \"Exempt them (Recommended)\", \"Add all three (Recommended)\") made the nine shared registration files open to every loop PR, so no item scope may list them"
            ));
        }
    }
    // (d) no glob overlaps an open order's scope; notebook pages never count. An open order whose
    // item is missing or has no scope: is skipped with a warning (it still counted in (b)); one whose
    // ORDER could not be read (no item) is skipped, since its scope is unknown.
    for o in &orders {
        let Some(their_item) = o.item.as_deref() else {
            continue;
        };
        let theirs = match main_item_text(root, their_item) {
            None => {
                eprintln!(
                    "uor-harness: loop claim: warning: open order {} names {}, which is not on origin/main; overlap not checked against it",
                    o.source, their_item
                );
                continue;
            }
            Some(t) => {
                parse_item(&t)
                    .map_err(|e| format!("open order {}: item {}: {e}", o.source, their_item))?
                    .scope
            }
        };
        let Some(theirs) = theirs else {
            eprintln!(
                "uor-harness: loop claim: warning: open order {} names {}, which has no scope: line; overlap not checked against it",
                o.source, their_item
            );
            continue;
        };
        for g in &theirs {
            scope_glob_ok(g)
                .map_err(|e| format!("open order {}: item {}: {e}", o.source, their_item))?;
        }
        for mine in scope.iter().filter(|g| counts(g)) {
            if let Some(other) = theirs
                .iter()
                .filter(|g| counts(g))
                .find(|g| globs_overlap(mine, g))
            {
                return Err(format!(
                    "scope {mine} overlaps {other} of item {} ({})",
                    their_item, o.source
                ));
            }
        }
    }
    // (e) the lab is under its open-PR limit.
    let labs = crate::labs::load_labs(root)?;
    let slug = crate::state::slug(lab);
    let limit = labs
        .get(&slug)
        .map(|l| l.open_pr_limit)
        .ok_or_else(|| format!("lab {lab} is not in docs/harness/labs.json"))?;
    let branch_prefix = format!("{slug}/");
    let open = prs
        .iter()
        .filter(|p| p.head_ref_name.starts_with(&branch_prefix))
        .count();
    if open >= limit as usize {
        return Err(format!(
            "lab {slug} has {open} open PRs, at its open_pr_limit of {limit}"
        ));
    }
    Ok(())
}

/// The text of an item's `## Done when` section, trimmed.
fn done_when(item_text: &str) -> String {
    let mut out = Vec::new();
    let mut inside = false;
    for line in item_text.lines() {
        if line.starts_with("## ") {
            if inside {
                break;
            }
            inside = line.trim() == "## Done when";
            continue;
        }
        if inside {
            out.push(line);
        }
    }
    out.join("\n").trim().to_string()
}

/// `7a` for `infra-loop-pr07a-...`: the build-plan row a loop setup item is.
fn setup_row(item: &str) -> Option<String> {
    let re = regex::Regex::new(r"^infra-loop-pr0*([0-9]+[a-z]?)(-|$)").expect("a valid pattern");
    re.captures(item).map(|c| c[1].to_string())
}

/// One more than the highest `docs/loop/tasks/*/<lab>-<k>/` on `origin/main`, across all items.
fn next_order_number(root: &Path, slug: &str) -> Result<u64, String> {
    let files = git(
        root,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            "origin/main",
            "docs/loop/tasks",
        ],
    )?;
    let prefix = format!("{slug}-");
    let highest = files
        .lines()
        .filter_map(|f| f.strip_prefix("docs/loop/tasks/"))
        .filter_map(|f| f.split('/').nth(1))
        .filter_map(|dir| dir.strip_prefix(&prefix)?.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    Ok(highest + 1)
}

/// The skeleton ORDER a claim commits. Its `Item:` line is what [`order_item`] reads.
pub fn order_skeleton(
    lab: &str,
    n: u64,
    item: &str,
    base: &str,
    done: &str,
    scope: &[String],
) -> String {
    let mut s = format!("# ORDER: {item}, {lab}-{n}\n\n");
    s += &format!("Lab: {lab}. Item: [{item}](../../../../roadmap/items/{item}.md).");
    if let Some(row) = setup_row(item) {
        s += &format!(" Setup row: {row}.");
    }
    s += &format!(" Base: `{base}`.\n\n## Goal\n\nThe item's done-when reads:\n\n");
    for line in done.lines() {
        s += &format!(">{}{line}\n", if line.is_empty() { "" } else { " " });
    }
    s += "\n## Cancel probe (run first)\n\nTo fill in.\n\n## Files in scope\n\n";
    for g in scope {
        s += &format!("- `{g}`\n");
    }
    s += "\n## Done when\n\nTo fill in.\n\n## Stop and ask if\n\nTo fill in.\n";
    s
}

/// What a claim created, so that a rollback removes exactly that and nothing else.
struct Created {
    worktree: PathBuf,
    branch: String,
    /// The commit we pushed to `origin` as `branch`, once the push succeeded.
    pushed: Option<String>,
    /// Our draft PR, once it exists.
    pr: Option<u64>,
}

impl Created {
    /// Undo everything this claim created, newest first: close our PR, delete our pushed branch
    /// from `origin` only while it is still at the commit we pushed, remove our worktree by its
    /// exact path and our branch by its exact name. Every step is attempted; the result names
    /// what is left, and is empty when the rollback is clean.
    fn rollback(&self, root: &Path, gh: &dyn Gh) -> Vec<String> {
        let mut left = Vec::new();
        if let Some(pr) = self.pr {
            if let Err(e) = gh.close_pr(pr) {
                left.push(format!("draft PR #{pr} is still open ({e})"));
            }
        }
        if let Some(sha) = &self.pushed {
            // The lease: a branch someone else has moved since our push is never deleted.
            let lease = format!("--force-with-lease=refs/heads/{}:{sha}", self.branch);
            let args = ["push", "-q", "origin", &lease, "--delete", &self.branch];
            if let Err(e) = crate::loopgates::git(root, &args) {
                left.push(format!("origin still has {} ({e})", self.branch));
            }
        }
        let wt = self.worktree.to_string_lossy();
        if let Err(e) = crate::loopgates::git(root, &["worktree", "remove", "--force", &wt]) {
            left.push(format!("worktree {wt} is left ({e})"));
        }
        if let Err(e) = crate::loopgates::git(root, &["branch", "-D", &self.branch]) {
            left.push(format!("branch {} is left ({e})", self.branch));
        }
        left
    }
}

/// Whether `branch` exists on `origin`. `git ls-remote --exit-code` exits 0 when it does and 2
/// when it does not; any other outcome means we cannot tell, which refuses the claim.
fn on_origin(root: &Path, branch: &str) -> Result<bool, String> {
    let r = format!("refs/heads/{branch}");
    let out = std::process::Command::new("git")
        .args(["ls-remote", "--exit-code", "origin", &r])
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .map_err(|e| format!("cannot check origin for {branch}: {e}"))?;
    match out.status.code() {
        Some(0) => Ok(true),
        Some(2) => Ok(false),
        _ => Err(format!(
            "cannot check origin for {branch}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )),
    }
}

/// `reason`, followed by one line for each thing a rollback left behind.
fn with_leftovers(reason: String, left: &[String]) -> String {
    if left.is_empty() {
        return format!("{reason}; everything this claim created was removed");
    }
    let mut s = format!("{reason}; the rollback left things a person must remove:");
    for l in left {
        s += &format!("\n  left: {l}");
    }
    s
}

/// The result of the writes: how the claim ended.
pub enum Claimed {
    Won {
        worktree: PathBuf,
        order: String,
        pr: u64,
    },
    Lost {
        pr: u64,
        lower: u64,
        /// Our branch, which the rollback deleted from `origin`.
        branch: String,
        /// What the rollback could not remove; empty when it was clean.
        left: Vec<String>,
    },
}

/// The writes after [`check`] passed (ORDER call 5). `Err` means exit 2.
pub fn write(
    root: &Path,
    lab: &str,
    item: &str,
    topic: &str,
    gh: &dyn Gh,
) -> Result<Claimed, String> {
    // No second fetch: the writes start from the origin/main that `check` fetched and read.
    let slug = crate::state::slug(lab);
    let mut created = Created {
        worktree: main_checkout(root)?
            .join(".worktrees")
            .join(format!("{slug}-{topic}")),
        branch: format!("{slug}/{topic}"),
        pushed: None,
        pr: None,
    };
    if created.worktree.exists() {
        return Err(format!(
            "worktree folder {} already exists",
            created.worktree.display()
        ));
    }
    let branch_ref = format!("refs/heads/{}", created.branch);
    if crate::loopgates::git(root, &["rev-parse", "--verify", "--quiet", &branch_ref]).is_ok() {
        return Err(format!("branch {} already exists", created.branch));
    }
    if on_origin(root, &created.branch)? {
        return Err(format!(
            "branch {} already exists on origin",
            created.branch
        ));
    }
    let item_text = main_item_text(root, item)
        .ok_or_else(|| format!("cannot read item {item} on origin/main"))?;
    let scope = parse_item(&item_text)?.scope.unwrap_or_default();
    let n = next_order_number(root, &slug)?;
    let base = git(root, &["rev-parse", "--short=9", "origin/main"])?;
    let wt = created.worktree.to_string_lossy().into_owned();
    crate::loopgates::git(
        root,
        &[
            "worktree",
            "add",
            "-q",
            "--no-track",
            "-b",
            &created.branch,
            &wt,
            "origin/main",
        ],
    )?;
    // From here on every failure runs the rollback of what this claim created.
    let order = format!("docs/loop/tasks/{item}/{slug}-{n}/ORDER.md");
    let text = order_skeleton(&slug, n, item, base.trim(), &done_when(&item_text), &scope);
    let outcome = (|| -> Result<Option<u64>, String> {
        let path = created.worktree.join(&order);
        std::fs::create_dir_all(path.parent().expect("an ORDER has a folder"))
            .and_then(|()| std::fs::write(&path, &text))
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        let wt_root = created.worktree.clone();
        crate::loopgates::git(&wt_root, &["add", "--", &order])?;
        let msg = format!("[{slug}] loop claim: ORDER for {item} ({slug}-{n})");
        crate::loopgates::git(&wt_root, &["commit", "-q", "-m", &msg, "--", &order])?;
        let sha = git(&wt_root, &["rev-parse", "HEAD"])?.trim().to_string();
        // An empty lease value: the push succeeds only while `origin` has no such branch.
        let lease = format!("--force-with-lease=refs/heads/{}:", created.branch);
        let branch = created.branch.clone();
        crate::loopgates::git(&wt_root, &["push", "-q", "-u", "origin", &lease, &branch])?;
        created.pushed = Some(sha);
        let title = format!("[{slug}] {item} ({slug}-{n})");
        let body =
            format!("Draft claim of `{item}` by `uor-harness loop claim`.\n\nORDER: `{order}`\n");
        let pr = gh.create_draft_pr(&branch, &title, &body)?;
        created.pr = Some(pr);
        // Re-list: a lower-numbered open PR, draft or not, whose ORDER names the same item wins the
        // race. An ORDER that cannot be read names no item, so it is never a rival.
        let prs = gh
            .list_open_prs()
            .map_err(|e| format!("draft PR #{pr} is open but the re-list failed: {e}"))?;
        let rivals: Vec<OpenPr> = prs.into_iter().filter(|p| p.number < pr).collect();
        Ok(pr_orders(gh, &rivals)
            .into_iter()
            .filter(|o| o.item.as_deref() == Some(item))
            .filter_map(|o| o.pr)
            .min())
    })();
    match outcome {
        Err(e) => Err(with_leftovers(e, &created.rollback(root, gh))),
        Ok(Some(lower)) => Ok(Claimed::Lost {
            pr: created
                .pr
                .expect("the re-list ran after the PR was created"),
            lower,
            left: created.rollback(root, gh),
            branch: created.branch,
        }),
        Ok(None) => Ok(Claimed::Won {
            pr: created
                .pr
                .expect("the re-list ran after the PR was created"),
            worktree: created.worktree,
            order,
        }),
    }
}

const USAGE: &str = "usage: uor-harness loop claim <item-id> [--lab-topic <topic>]";

/// `loop claim ...`, with `args` starting after `claim`. Returns the exit code.
pub fn cli(root: &Path, lab: &str, args: &[String]) -> u8 {
    let (mut item, mut topic) = (None, None);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--lab-topic" => match args.get(i + 1) {
                Some(t) => {
                    topic = Some(t.clone());
                    i += 1;
                }
                None => {
                    eprintln!("{USAGE}");
                    return 2;
                }
            },
            a if a.starts_with('-') || item.is_some() => {
                eprintln!("uor-harness: loop claim: unexpected argument {a:?}\n{USAGE}");
                return 2;
            }
            a => item = Some(a.to_string()),
        }
        i += 1;
    }
    let Some(item) = item else {
        eprintln!("{USAGE}");
        return 2;
    };
    let id_re = regex::Regex::new("^[a-z0-9][a-z0-9.-]*$").expect("a valid pattern");
    if !id_re.is_match(&item) || topic.as_deref().is_some_and(|t| !id_re.is_match(t)) {
        eprintln!("uor-harness: loop claim: item and topic must match ^[a-z0-9][a-z0-9.-]*$");
        return 2;
    }
    if lab.trim().is_empty() {
        eprintln!("uor-harness: loop claim needs a lab: set UOR_LAB or pass --lab");
        return 2;
    }
    let gh: Box<dyn Gh> = match std::env::var_os(GH_FAKE_ENV) {
        Some(p) => match FakeGh::from_file(Path::new(&p)) {
            Ok(f) => Box::new(f),
            Err(e) => {
                eprintln!("uor-harness: loop claim: {e}");
                return 2;
            }
        },
        None => Box::new(RealGh {
            root: root.to_path_buf(),
        }),
    };
    if let Err(reason) = check(root, lab, &item, gh.as_ref()) {
        eprintln!("uor-harness: loop claim: refused: {reason}");
        return 2;
    }
    let topic = topic.unwrap_or_else(|| item.clone());
    match write(root, lab, &item, &topic, gh.as_ref()) {
        Ok(Claimed::Won {
            worktree,
            order,
            pr,
        }) => {
            println!("worktree: {}", worktree.display());
            println!("order: {order}");
            println!("draft PR: #{pr}");
            0
        }
        Ok(Claimed::Lost {
            pr,
            lower,
            branch,
            left,
        }) => {
            if left.is_empty() {
                eprintln!(
                    "uor-harness: loop claim: lost the race to #{lower}; closed #{pr}, deleted {branch} from origin and removed its worktree and branch"
                );
                return 1;
            }
            eprintln!(
                "uor-harness: loop claim: lost the race to #{lower}; the rollback of #{pr} left things a person must remove:"
            );
            for l in left {
                eprintln!("  left: {l}");
            }
            2
        }
        Err(e) => {
            eprintln!("uor-harness: loop claim: {e}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_gh_strips_git_vars_but_the_remote_ones() {
        let names = [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_INDEX_FILE",
            "GIT_SSH_COMMAND",
            "GIT_SSH",
            "GIT_ASKPASS",
            "HOME",
            "GH_TOKEN",
        ];
        assert_eq!(
            super::git_vars_to_strip(names.iter().map(|s| s.to_string())),
            ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"]
        );
    }

    #[test]
    fn contents_endpoint_reads_the_pull_ref_url_encoded() {
        assert_eq!(
            contents_endpoint("docs/loop/tasks/a b#c/ORDER.md", 7),
            "repos/{owner}/{repo}/contents/docs/loop/tasks/a%20b%23c/ORDER.md?ref=refs%2Fpull%2F7%2Fhead"
        );
    }

    #[test]
    fn overlap_is_by_literal_path_prefix() {
        assert!(globs_overlap("crates/a/src/**", "crates/a/src/x.rs"));
        assert!(globs_overlap("crates/a/**", "crates/a/src/*.rs"));
        assert!(globs_overlap("crates/a-*", "crates/a-b/x.rs"));
        assert!(globs_overlap("docs/x.md", "docs/x.md"));
        assert!(!globs_overlap("docs/x.md", "docs/x.md.bak"));
        assert!(!globs_overlap("crates/a/**", "crates/ab/**"));
        assert!(!globs_overlap("xtask/**", "crates/**"));
    }

    #[test]
    fn an_order_names_its_item_on_the_item_line() {
        let t = "# ORDER\n\nLab: claude. Item: [x](../../../../roadmap/items/infra-x.md) (#1).\n";
        assert_eq!(order_item(t).as_deref(), Some("infra-x"));
        assert_eq!(order_item("see roadmap/items/y.md\n"), None);
    }

    #[test]
    fn the_skeleton_order_names_its_item_and_setup_row() {
        let item = "---\nstatus: ready\n---\n\n## Done when\n\nIt works.\n\n## How to start\n\nx\n";
        let scope = vec!["crates/a/**".to_string()];
        let s = order_skeleton(
            "claude",
            3,
            "infra-loop-pr07a-claim",
            "abc",
            &done_when(item),
            &scope,
        );
        assert_eq!(order_item(&s).as_deref(), Some("infra-loop-pr07a-claim"));
        assert!(
            s.contains("Setup row: 7a.") && s.contains("> It works.\n"),
            "{s}"
        );
        assert!(
            s.contains("- `crates/a/**`") && !s.contains("How to start"),
            "{s}"
        );
        assert!(!order_skeleton("claude", 1, "m1-x", "abc", "", &[]).contains("Setup row"));
    }
}
