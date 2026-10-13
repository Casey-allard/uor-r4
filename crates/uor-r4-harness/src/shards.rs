//! Shard integrity: a ledger shard may never shrink.
//!
//! The ledger is the project's cumulative resource and activity record. It is sharded per lab
//! (`docs/harness/ledger/<lab>-<YYYY-MM>.jsonl`) so concurrent labs never contend on one file — but
//! sharding only removes *write* contention. It does nothing about **merge** contention, which is
//! where the loss actually happened: PR #2305's merge dropped two of Kimi's receipts from the kimi
//! shard, and they had to be restored by hand as an append-only union. Kimi's flag on #2028 records
//! this as at least the fourth shared-store clobber in one day.
//!
//! The rule this module enforces: **a shard may grow, and may never lose a line.** That is weaker
//! than "append-only" in the letter — a branch that appends and also reorders is allowed — and
//! deliberately so: the property that matters is that no receipt disappears. Requiring strict
//! prefix-append would fail on a shard that a lab legitimately resorted, and a guard that fires on
//! safe changes gets disabled.
//!
//! ## Why the comparison is a MULTISPECT and not a set
//!
//! Duplicate lines are normal here, not corruption. Measured on main: the kimi shard holds 8
//! duplicate rows, `unknown` holds 18 of its 24. Two receipts can be byte-identical when the same
//! action is recorded twice in the same second — `harness skill used` twice in a second produces the
//! same JSON. A set comparison would therefore MISS a real deletion: drop one of two identical rows
//! and the set is unchanged. Counting occurrences catches it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::state::StatePaths;

/// The ledger directory, relative to the repository root.
pub const LEDGER_REL: &str = "docs/harness/ledger";

/// One line that exists fewer times in the head than in the base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LostLine {
    /// The line's content, canonicalised (parsed and re-serialised with sorted keys) when it is JSON.
    pub content: String,
    /// How many copies the base held.
    pub base_count: usize,
    /// How many the head holds.
    pub head_count: usize,
}

impl LostLine {
    pub fn missing(&self) -> usize {
        self.base_count.saturating_sub(self.head_count)
    }
}

/// The verdict on one shard.
#[derive(Debug, Clone)]
pub struct ShardReport {
    pub path: PathBuf,
    pub base_lines: usize,
    pub head_lines: usize,
    pub lost: Vec<LostLine>,
    /// Lines the head added. Not a problem — reported so a grow is visible, not inferred.
    pub added: usize,
}

impl ShardReport {
    pub fn ok(&self) -> bool {
        self.lost.is_empty()
    }

    /// Total receipts that disappeared.
    pub fn lost_count(&self) -> usize {
        self.lost.iter().map(LostLine::missing).sum()
    }
}

/// Canonicalise a ledger line so two textual renderings of the same receipt compare equal.
///
/// A receipt is JSON written with sorted keys, but a merge resolution, a hand edit or a different
/// serialiser can reorder keys without changing the receipt. Comparing raw text would call that a
/// loss of one line and an addition of another, which is a false alarm on a no-op. Falls back to the
/// trimmed raw text when the line is not JSON, so a malformed line is still compared rather than
/// silently ignored — a corrupt line is not a licence to skip the check.
pub fn canonical(line: &str) -> String {
    let trimmed = line.trim();
    match serde_json::from_str::<Value>(trimmed) {
        Ok(v) => serde_json::to_string(&sort_keys(&v)).unwrap_or_else(|_| trimmed.to_string()),
        Err(_) => trimmed.to_string(),
    }
}

fn sort_keys(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for k in keys {
                out.insert(k.clone(), sort_keys(&map[k]));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(sort_keys).collect()),
        other => other.clone(),
    }
}

/// Count occurrences of each canonical line.
fn multiset(text: &str) -> BTreeMap<String, usize> {
    let mut out: BTreeMap<String, usize> = BTreeMap::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        *out.entry(canonical(line)).or_insert(0) += 1;
    }
    out
}

/// Compare a shard's base text against its head text.
///
/// Blank lines are ignored on both sides: they carry no receipt, and a merge that drops a trailing
/// newline is not a loss of record.
pub fn compare(path: &Path, base: &str, head: &str) -> ShardReport {
    let b = multiset(base);
    let h = multiset(head);
    let mut lost = Vec::new();
    for (content, base_count) in &b {
        let head_count = h.get(content).copied().unwrap_or(0);
        if head_count < *base_count {
            lost.push(LostLine {
                content: content.clone(),
                base_count: *base_count,
                head_count,
            });
        }
    }
    // Deterministic order, so a report reads the same on every run.
    lost.sort_by(|a, b| a.content.cmp(&b.content));
    let added: usize = h
        .iter()
        .map(|(c, n)| n.saturating_sub(b.get(c).copied().unwrap_or(0)))
        .sum();
    ShardReport {
        path: path.to_path_buf(),
        base_lines: b.values().sum(),
        head_lines: h.values().sum(),
        lost,
        added,
    }
}

/// Read the ledger's base version from git and compare it against the working tree.
///
/// `base_ref` is what the branch is measured against — `origin/main` for a PR, or a merge-base for a
/// narrower check. Only files under the ledger directory are considered; a shard that did not exist
/// at the base has nothing to lose, so it is skipped rather than reported as a 100% loss.
pub fn check_ledger(paths: &StatePaths, base_ref: &str) -> Result<Vec<ShardReport>, String> {
    let repo = paths.repo.as_path();
    // The base ref must EXIST. Without this, an unresolvable ref makes `git show` fail for every
    // shard, every shard is skipped as "absent at the base", and the check reports a clean ledger
    // from an empty comparison. That is the worst failure a guard can have - it passes exactly when
    // it could not look - and it is what happened the first time this ran against a ref that a
    // shallow fetch had deleted. A guard that cannot see must say so.
    let ok = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{base_ref}^{{commit}}"),
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !ok {
        return Err(format!(
            "base ref {base_ref:?} does not resolve in {} - refusing to report a ledger as clean \
             without a comparison point (`git fetch origin main` first, or pass --base HEAD)",
            repo.display()
        ));
    }
    let dir = repo.join(LEDGER_REL);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        // No ledger at all is not a shard-integrity failure; it is a different problem.
        return Ok(Vec::new());
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
        .collect();
    files.sort();

    let mut reports = Vec::new();
    for f in files {
        let rel = match f.strip_prefix(repo) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        let base = git_show(repo, base_ref, &rel);
        let Some(base) = base else {
            // Absent at the base: a new shard cannot shrink.
            continue;
        };
        let Ok(head) = std::fs::read_to_string(&f) else {
            continue;
        };
        reports.push(compare(Path::new(&rel), &base, &head));
    }
    Ok(reports)
}

/// `git show <ref>:<path>`, or `None` when the path does not exist at that ref.
fn git_show(repo: &Path, git_ref: &str, rel: &str) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .arg("show")
        .arg(format!("{git_ref}:{rel}"))
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Every failing shard, or empty when the ledger only grew.
pub fn violations(reports: &[ShardReport]) -> Vec<&ShardReport> {
    reports.iter().filter(|r| !r.ok()).collect()
}

/// Render a human report, naming each lost line so it can be restored rather than hunted.
pub fn render(reports: &[ShardReport]) -> String {
    let bad = violations(reports);
    if bad.is_empty() {
        let total: usize = reports.iter().map(|r| r.head_lines).sum();
        return format!(
            "shard integrity: ok - {} shard(s), {total} receipts, none lost",
            reports.len()
        );
    }
    let mut out = format!(
        "shard integrity: FAIL - {} of {} shard(s) lost receipts.\n",
        bad.len(),
        reports.len()
    );
    out.push_str(
        "A ledger shard may never shrink: the ledger is the cumulative record, and a merge that \
         drops a receipt is silent data loss.\n",
    );
    for r in bad {
        out.push_str(&format!(
            "\n  {}: {} -> {} lines ({} receipt(s) lost)\n",
            r.path.display(),
            r.base_lines,
            r.head_lines,
            r.lost_count()
        ));
        for l in r.lost.iter().take(5) {
            let shown: String = l.content.chars().take(120).collect();
            out.push_str(&format!(
                "    missing {} copy(ies) of: {shown}\n",
                l.missing()
            ));
        }
        if r.lost.len() > 5 {
            out.push_str(&format!(
                "    (+{} more distinct line(s))\n",
                r.lost.len() - 5
            ));
        }
        out.push_str("    fix: restore the missing line(s) - re-append them from the base ref\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_appended_shard_is_clean() {
        let r = compare(
            Path::new("x.jsonl"),
            "{\"a\":1}\n",
            "{\"a\":1}\n{\"b\":2}\n",
        );
        assert!(r.ok());
        assert_eq!(r.base_lines, 1);
        assert_eq!(r.head_lines, 2);
        assert_eq!(r.added, 1);
    }

    /// The exact failure #2305 caused: lines present at the base and absent after a merge.
    #[test]
    fn a_dropped_line_is_a_violation() {
        let r = compare(
            Path::new("x.jsonl"),
            "{\"a\":1}\n{\"b\":2}\n",
            "{\"a\":1}\n",
        );
        assert!(!r.ok());
        assert_eq!(r.lost_count(), 1);
        assert_eq!(r.lost[0].base_count, 1);
        assert_eq!(r.lost[0].head_count, 0);
    }

    /// Why the comparison is a multiset. Two identical receipts in one second are normal — kimi's
    /// shard holds 8 such duplicates and `unknown` holds 18 of 24 — and a SET comparison would miss
    /// the deletion of one of a pair entirely.
    #[test]
    fn losing_one_of_two_identical_lines_is_detected() {
        let dup = "{\"kind\":\"skill\",\"ts\":\"2026-10-12T22:10:42-04:00\"}\n";
        let r = compare(Path::new("x.jsonl"), &format!("{dup}{dup}"), dup);
        assert!(!r.ok(), "a set comparison would have called this clean");
        assert_eq!(r.lost_count(), 1);
        assert_eq!(r.lost[0].base_count, 2);
        assert_eq!(r.lost[0].head_count, 1);
    }

    /// A reordered but complete shard is NOT a loss. Rejecting it would be a guard that fires on safe
    /// changes, and a guard that cries wolf gets disabled.
    #[test]
    fn reordering_without_loss_is_clean() {
        let r = compare(
            Path::new("x.jsonl"),
            "{\"a\":1}\n{\"b\":2}\n",
            "{\"b\":2}\n{\"a\":1}\n",
        );
        assert!(r.ok(), "{:?}", r.lost);
        assert_eq!(r.added, 0);
    }

    /// Key order is a serialisation detail, not a different receipt.
    #[test]
    fn key_order_does_not_count_as_a_change() {
        let r = compare(
            Path::new("x.jsonl"),
            "{\"a\":1,\"b\":2}\n",
            "{\"b\":2,\"a\":1}\n",
        );
        assert!(r.ok(), "reordered keys are the same receipt: {:?}", r.lost);
    }

    #[test]
    fn blank_lines_are_not_receipts() {
        let r = compare(Path::new("x.jsonl"), "{\"a\":1}\n\n", "{\"a\":1}\n");
        assert!(r.ok(), "a dropped blank line is not a lost receipt");
    }

    /// A malformed line is still compared. Skipping it would let a corrupt shard hide a deletion.
    #[test]
    fn a_non_json_line_is_compared_rather_than_ignored() {
        let r = compare(Path::new("x.jsonl"), "not json\n", "");
        assert!(!r.ok());
        assert_eq!(r.lost_count(), 1);
    }

    #[test]
    fn a_shrink_of_many_lines_reports_the_total() {
        let base = "{\"i\":1}\n{\"i\":2}\n{\"i\":3}\n{\"i\":4}\n";
        let head = "{\"i\":1}\n";
        let r = compare(Path::new("x.jsonl"), base, head);
        assert_eq!(r.lost_count(), 3);
        assert_eq!(r.base_lines, 4);
        assert_eq!(r.head_lines, 1);
    }

    #[test]
    fn the_report_names_the_missing_lines() {
        let r = compare(
            Path::new("kimi-2026-10.jsonl"),
            "{\"ts\":\"dropped\"}\n",
            "",
        );
        let text = render(&[r]);
        assert!(text.contains("FAIL"), "{text}");
        assert!(text.contains("kimi-2026-10.jsonl"), "{text}");
        assert!(
            text.contains("dropped"),
            "the content is shown so it can be restored: {text}"
        );
        assert!(text.contains("restore"), "{text}");
    }

    #[test]
    fn a_clean_report_says_so_without_alarm() {
        let r = compare(
            Path::new("x.jsonl"),
            "{\"a\":1}\n",
            "{\"a\":1}\n{\"b\":2}\n",
        );
        let text = render(&[r]);
        assert!(text.contains("ok"), "{text}");
        assert!(!text.contains("FAIL"), "{text}");
    }

    /// A base ref that does not resolve is an ERROR, not an empty pass.
    ///
    /// This is the bug the first version had: an unresolvable ref made every `git show` fail, every
    /// shard was skipped, and the command printed "no ledger shards present" and exited 0. A guard
    /// that reports success because it could not look is worse than no guard at all.
    #[test]
    fn an_unresolvable_base_ref_is_an_error_not_a_clean_pass() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let err = check_ledger(&paths, "no-such-ref-anywhere")
            .expect_err("an unresolvable base must not return Ok");
        assert!(err.contains("does not resolve"), "{err}");
        assert!(err.contains("no-such-ref-anywhere"), "{err}");
    }

    /// Against the REAL ledger: main's shards must satisfy the rule relative to their own history.
    /// A shard compared with itself is trivially clean, so this checks the comparison runs end to end
    /// on real data rather than asserting a property that cannot fail.
    #[test]
    fn the_real_ledger_runs_end_to_end() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let reports = check_ledger(&paths, "HEAD").expect("the ledger is readable");
        assert!(!reports.is_empty(), "the ledger has shards to check");
        for r in &reports {
            assert!(
                r.ok(),
                "{} shrank against HEAD, which is itself",
                r.path.display()
            );
        }
    }
}
