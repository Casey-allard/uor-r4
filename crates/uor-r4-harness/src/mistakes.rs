//! `mistakes` — lessons from what went wrong here before (D30 port).
//!
//! # Why this exists
//!
//! `mistakes find -f <files>` is run before editing, and it answers one question: **what has already
//! gone wrong in code like this?** A lesson that does not surface at the moment someone is about to
//! repeat it is a lesson nobody reads.
//!
//! # The matching rules, and why each is there
//!
//! A lesson carries `paths` (globs), `trigger` (phrases), `milestones` and sometimes a `mechanism`.
//! [`match`] selects on any of them, and the `why` it returns names **which rule fired** — because a
//! lesson surfaced for the wrong reason teaches the wrong thing.
//!
//! * **`status` gates everything.** Only `active` and `warn` lessons are matched; a `closed` lesson is
//!   history, and resurfacing it would train readers to skim.
//! * **Query tokens are longer than three characters.** A token like `the` or `a` appears in almost
//!   every trigger, so admitting it would match everything and mean nothing.
//! * **A path matches by `fnmatch` or as a substring.** Lessons are written against paths that move,
//!   so an exact-only match would silently stop firing after a rename — and a guard that stops firing
//!   is indistinguishable from a guard that passes.
//! * **Results sort by severity, then id.** `block` before `warn` before `info`, because the reader
//!   is about to edit code and needs the stopping condition first.

use std::path::{Path, PathBuf};

use crate::preprules::Finding;

use crate::state::StatePaths;

/// One lesson from `docs/harness/mistakes/`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mistake {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub class: String,
    pub status: String,
    pub guard: String,
    pub lesson: String,
    pub mechanism: String,
    pub paths: Vec<String>,
    pub trigger: Vec<String>,
    pub milestones: Vec<String>,
    pub evidence: Vec<String>,
    /// Why this file is not a valid lesson, if it is not.
    ///
    /// **The Rust parser was permissive and never rejected anything, so `mistakes.registry_valid`
    /// had nothing to report** — the port could not carry the rule that catches a malformed lesson.
    /// The checks here mirror the Python reader's, including the reason each one exists.
    ///
    /// A lesson with errors is still returned: **reporting the file is the point, and dropping it
    /// would make a broken lesson invisible instead of loud.**
    pub errors: Vec<String>,
    pub path: PathBuf,
}

/// Why a lesson matched, for the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub mistake: Mistake,
    pub why: String,
}

/// The lessons directory.
pub fn dir(paths: &StatePaths) -> PathBuf {
    paths.repo.join("docs").join("harness").join("mistakes")
}

/// Load every lesson, sorted by id so resolution is deterministic.
pub fn load(paths: &StatePaths) -> Vec<Mistake> {
    let Ok(entries) = std::fs::read_dir(dir(paths)) else {
        return Vec::new();
    };
    let mut out: Vec<Mistake> = Vec::new();
    for e in entries.flatten() {
        let path = e.path();
        if path.extension().and_then(|x| x.to_str()) != Some("md") {
            continue;
        }
        // `README.md` indexes the set; it is not a lesson.
        if path.file_name().and_then(|f| f.to_str()) == Some("README.md") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        out.push(parse(&stem, &text, &path));
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// Parse one lesson file.
/// `severity` values a lesson may declare.
pub const SEVERITIES: [&str; 3] = ["block", "warn", "info"];
/// `class` values a lesson may declare.
pub const CLASSES: [&str; 7] = [
    "process",
    "evidence",
    "compute",
    "measurement",
    "delivery",
    "coordination",
    "assumption",
];
/// `status` values a lesson may declare.
pub const STATUSES: [&str; 3] = ["active", "resolved", "superseded"];

/// Why `text` is not a valid lesson file, in the Python reader's own terms.
///
/// **Every check here has a reason, and the reasons are the rule.** The id must equal the file stem
/// because the registry is navigated by filename; `trigger` needs two phrases because one phrase
/// matches everything and therefore retrieves nothing; `paths` must be non-empty because **a lesson
/// that no path matches can never be recalled** — it is a note filed where nothing looks.
pub fn validation_errors(stem: &str, text: &str, fm: &[(String, String)]) -> Vec<String> {
    let mut errors = Vec::new();
    let get = |k: &str| -> String {
        fm.iter()
            .find(|(key, _)| key == k)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    if !text.trim_start().starts_with("---") {
        errors.push("no front matter: the file must start with '---'".to_string());
        return errors;
    }
    for key in [
        "id", "title", "date", "severity", "class", "trigger", "paths", "evidence", "status",
    ] {
        if get(key).trim().is_empty() || get(key).trim() == "[]" {
            errors.push(format!("front matter is missing '{key}'"));
        }
    }
    let id = get("id");
    if !id.is_empty() && id.to_lowercase() != stem.to_lowercase() {
        errors.push(format!(
            "id {} must equal the file stem {}",
            crate::prgates::py_repr_public(&id),
            crate::prgates::py_repr_public(stem)
        ));
    }
    if !id.is_empty() && !kebab_case_id(&id) {
        errors.push(format!(
            "id {} is not a kebab-case id",
            crate::prgates::py_repr_public(&id)
        ));
    }
    let sev = get("severity");
    if !sev.is_empty() && !SEVERITIES.contains(&sev.as_str()) {
        errors.push(format!(
            "severity must be one of {}, got {}",
            py_tuple(&SEVERITIES),
            crate::prgates::py_repr_public(&sev)
        ));
    }
    let class = get("class");
    if !class.is_empty() && !CLASSES.contains(&class.as_str()) {
        errors.push(format!(
            "class must be one of {}, got {}",
            py_tuple(&CLASSES),
            crate::prgates::py_repr_public(&class)
        ));
    }
    let status = get("status");
    if !status.is_empty() && !STATUSES.contains(&status.as_str()) {
        errors.push(format!(
            "status must be one of {}, got {}",
            py_tuple(&STATUSES),
            crate::prgates::py_repr_public(&status)
        ));
    }
    let date = get("date");
    if !date.is_empty() && !is_ymd(&date) {
        errors.push(format!(
            "date must be YYYY-MM-DD, got {}",
            crate::prgates::py_repr_public(&date)
        ));
    }
    let guard = get("guard");
    if !guard.is_empty() && !guard.starts_with("none") && !guard.contains('.') {
        errors.push(format!(
            "guard {} must be a dotted rule id or 'none: <reason>'",
            crate::prgates::py_repr_public(&guard)
        ));
    }
    // **Three checks Python has and this port did not.** They were invisible until `capture`
    // started validating its own drafts: a lesson with one trigger is retrievable by nothing, and
    // the port was accepting lessons the registry rejects.
    for key in ["trigger", "paths", "evidence"] {
        let raw = get(key);
        let trimmed = raw.trim();
        // An inline list is the only list form this reader parses; an empty value is how a block
        // list arrives here, and Python accepts a block list, so neither is an error.
        if !trimmed.is_empty() && !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
            errors.push(format!("'{key}' must be an inline or block list"));
        }
    }
    let trigger = get("trigger");
    if trigger.trim().starts_with('[') && list(&trigger).len() < 2 {
        errors.push(
            "'trigger' needs at least two phrases, or the lesson is not retrievable".to_string(),
        );
    }
    let paths = get("paths");
    if paths.trim().starts_with('[') && list(&paths).is_empty() {
        errors.push("'paths' is empty: name a glob so the lesson is recalled".to_string());
    }
    errors
}

/// `^[a-z0-9][a-z0-9-]{2,63}$` — at least three characters, kebab-case.
pub fn kebab_case_id(id: &str) -> bool {
    let b = id.as_bytes();
    if b.len() < 3 || b.len() > 64 {
        return false;
    }
    if !(b[0].is_ascii_lowercase() || b[0].is_ascii_digit()) {
        return false;
    }
    b[1..]
        .iter()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

/// Resolve an evidence/record path relative to the repo, then the lesson directory.
///
/// **Two bases, because a lesson cites a path the way a reader would**: `docs/labs/...` from the repo
/// root, or a sibling file in the lesson's own directory. An absolute path is taken as-is. A path that
/// resolves nowhere is what `mistakes.evidence_exists` reports — **a citation to a file that does not
/// exist is worse than no citation, because it reads as evidence.**
pub fn resolve_path(paths: &StatePaths, value: &str) -> Option<PathBuf> {
    if value.trim().is_empty() {
        return None;
    }
    let p = Path::new(value);
    if p.is_absolute() {
        return if p.exists() {
            Some(p.to_path_buf())
        } else {
            None
        };
    }
    let direct = paths.repo.join(p);
    if direct.exists() {
        return Some(direct);
    }
    let in_dir = dir(paths).join(p);
    if in_dir.exists() {
        return Some(in_dir);
    }
    None
}

/// `mistakes.registry_valid` — every lesson file parses, ids are unique, and the front matter is valid.
///
/// **Two distinct failures, kept apart because the fixes differ:** a file whose *front matter* is
/// wrong (the reader's own errors, reported verbatim) and **a duplicate id** across files, which makes
/// the registry ambiguous — a lookup by id returns whichever file was read first.
pub fn registry_valid(paths: &StatePaths) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut seen: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for lesson in load(paths) {
        let name = lesson
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        for error in &lesson.errors {
            out.push(Finding {
                rule_id: "mistakes.registry_valid",
                severity: "error",
                message: format!("{name}: {error}"),
                path: lesson.path.display().to_string(),
                fix: "fix the front matter: see docs/harness/mistakes/README.md for the exact template"
                    .into(),
            });
        }
        if let Some(prior) = seen.get(&lesson.id) {
            out.push(Finding {
                rule_id: "mistakes.registry_valid",
                severity: "error",
                message: format!("duplicate lesson id {:?} in {name} and {prior}", lesson.id),
                path: lesson.path.display().to_string(),
                fix: "give one of them a unique id (the file stem must equal the id)".into(),
            });
        } else {
            seen.insert(lesson.id.clone(), name);
        }
    }
    out
}

/// `mistakes.evidence_exists` — every cited evidence path exists on disk.
pub fn evidence_exists(paths: &StatePaths) -> Vec<Finding> {
    let mut out = Vec::new();
    for lesson in load(paths) {
        // An unreadable file is `registry_valid`'s finding; reporting its (empty) evidence here would
        // be a second finding for one fault.
        if !lesson.errors.is_empty() && lesson.evidence.is_empty() {
            continue;
        }
        for value in &lesson.evidence {
            if resolve_path(paths, value).is_none() {
                out.push(Finding {
                    rule_id: "mistakes.evidence_exists",
                    severity: "error",
                    message: format!("{}: evidence path {value} does not exist", lesson.id),
                    path: lesson.path.display().to_string(),
                    fix: "cite a path that exists (repo-relative), or remove the claim".into(),
                });
            }
        }
    }
    out
}

/// Every rule id the engine registers, read from `rules/registered-rules.json`.
///
/// **This is the file that fixes the bug my first attempt had.** The first implementation resolved a
/// guard against `ruleinventory::all_ids()` — the rules the **port** has implemented — while the rule
/// must resolve against the rules the **engine** registers. A lesson guarding on a rule that exists
/// and is enforced was reported as *"not a rule this engine registers"*: **Rust said 4, Python said
/// 0.**
///
/// **A guard check against an incomplete port is not a weaker check, it is a wrong one** — it would
/// tell authors to delete guards on real rules, and it would stop being wrong only when the port
/// finished, **which is exactly when the check stops mattering.**
///
/// The set lives as data because **the check engine is Python and the port is not**: the port cannot
/// enumerate the engine's rules, so it reads them.
pub fn registered_rules(paths: &StatePaths) -> Vec<String> {
    let file = paths
        .repo
        .join("crates/uor-r4-harness/rules/registered-rules.json");
    let Ok(text) = std::fs::read_to_string(&file) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    v.get("rules")
        .and_then(|r| r.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// `mistakes.guard_covered` — every **block**-severity lesson names a guard rule that exists.
///
/// **Only `block` severity.** A `warn` lesson is advice and may legitimately have no machine behind
/// it; a `block` lesson is a stopping condition, and **a stopping condition with no rule is a
/// preference** — it will be forgotten, which is the whole reason the registry exists.
///
/// `none: <reason>` is accepted **only with a reason of at least four characters**, because `none:`
/// alone is the escape hatch used without saying why, and that is indistinguishable from forgetting.
pub fn guard_covered(paths: &StatePaths) -> Vec<Finding> {
    let known = registered_rules(paths);
    let mut out = Vec::new();
    for lesson in load(paths) {
        if lesson.severity != "block" || !lesson.errors.is_empty() {
            continue;
        }
        let guard = lesson.guard.trim();
        let problem = if guard.is_empty() {
            Some(format!("{} is severity block with no guard", lesson.id))
        } else if let Some(rest) = guard.strip_prefix("none") {
            let reason = rest.trim_start_matches(':').trim();
            if reason.chars().count() < 4 {
                Some(format!("{} guard {guard:?} gives no reason", lesson.id))
            } else {
                None
            }
        } else if !known.iter().any(|k| k == guard) {
            Some(format!(
                "{} guard {guard:?} is not a rule this engine registers",
                lesson.id
            ))
        } else {
            None
        };
        if let Some(message) = problem {
            out.push(Finding {
                rule_id: "mistakes.guard_covered",
                severity: "warn",
                message,
                path: lesson.path.display().to_string(),
                fix: "name a rule id, or 'none: <reason why no machine can decide it>'".into(),
            });
        }
    }
    out
}

/// `^\d{4}-\d{2}-\d{2}$`./// `^\d{4}-\d{2}-\d{2}$`.
fn is_ymd(date: &str) -> bool {
    let b = date.as_bytes();
    b.len() == 10
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[7] == b'-'
        && b[8..].iter().all(u8::is_ascii_digit)
}

pub fn parse(stem: &str, text: &str, path: &Path) -> Mistake {
    let fm = frontmatter(text);
    let get = |k: &str| {
        fm.iter()
            .find(|(key, _)| key == k)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    let id = {
        let v = get("id");
        if v.is_empty() {
            stem.to_string()
        } else {
            v
        }
    };
    // `severity` defaults to `warn`, matching the Python reader: an unlabelled lesson should still
    // surface, and defaulting to `info` would bury it below everything.
    let severity = {
        let v = get("severity");
        if v.is_empty() {
            "warn".to_string()
        } else {
            v
        }
    };
    let errors = validation_errors(stem, text, &fm);
    Mistake {
        errors,
        id,
        title: get("title"),
        severity,
        class: get("class"),
        status: {
            let v = get("status");
            if v.is_empty() {
                "active".to_string()
            } else {
                v
            }
        },
        guard: get("guard"),
        lesson: get("lesson"),
        mechanism: get("mechanism"),
        paths: list(&get("paths")),
        trigger: list(&get("trigger")),
        milestones: list(&get("milestones")),
        evidence: list(&get("evidence")),
        path: path.to_path_buf(),
    }
}

fn frontmatter(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return out;
    }
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            out.push((k.trim().to_string(), coerce(v)));
        }
    }
    out
}

/// `core._coerce`, for the one case this reader needs: **a quoted scalar loses its quotes.**
///
/// **Without it, `capture` refused its own output.** `build_lesson_text` writes the default guard
/// as `"none: captured by hand on …"` — quoted because it contains a colon — and Python's YAML
/// subset strips those quotes while this reader kept them. The guard then began with a quote
/// instead of `none`, failed the guard rule, and every capture with an auto-generated guard was
/// refused. Found by running the command, not by reading either parser.
///
/// Python's `_coerce` also turns `true`/`null` into non-strings and an inline list into a real
/// list; this returns the raw text for those and leaves the list to [`list`], because the
/// callers here compare text. **Block lists are not parsed at all** — nothing in the corpus uses
/// one, and the gap is stated rather than hidden.
fn coerce(raw: &str) -> String {
    let s = raw.trim();
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let (first, last) = (bytes[0], bytes[bytes.len() - 1]);
        // Both quotes are ASCII, so the slice boundaries are character boundaries.
        if first == last && (first == b'"' || first == b'\'') {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.to_string()
}

/// Python's tuple repr, for every message that quotes a closed list.
pub fn py_tuple(items: &[&str]) -> String {
    let body: Vec<String> = items.iter().map(|s| format!("'{s}'")).collect();
    format!("({})", body.join(", "))
}

fn list(raw: &str) -> Vec<String> {
    let t = raw.trim();
    let inner = t
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(t);
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Tokens worth matching on: longer than three characters.
///
/// A token like `the` or `a` appears in almost every trigger, so admitting it would match everything
/// and mean nothing.
fn query_tokens(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
        .filter(|t| t.chars().count() > 3)
        .map(str::to_string)
        .collect()
}

/// Does a lesson's path pattern match a file?
///
/// `fnmatch` semantics, implemented for `*`, `?` and `**`, plus a substring fallback. Lessons are
/// written against paths that move, so an exact-only match would silently stop firing after a rename
/// — **and a guard that stops firing is indistinguishable from a guard that passes.**
pub fn path_matches(pattern: &str, file: &str) -> bool {
    if fnmatch(pattern, file) {
        return true;
    }
    // The `*{pat}*` form the Python matcher also accepts, so a lesson written against a bare filename
    // still fires for a path containing it.
    fnmatch(&format!("*{pattern}*"), file)
}

/// Glob match with **Python's `fnmatch` semantics, not shell globbing**.
///
/// `*` matches **any** characters **including `/`**, and `?` matches any single character including
/// `/`. That is the opposite of a shell glob, and assuming shell behaviour was a real bug: with
/// separator-aware `*`, the pattern `*team.rs*` did not match `crates/x/src/team.rs`, so **a lesson
/// written against a bare filename silently stopped firing**. A guard that stops firing is
/// indistinguishable from a guard that passes.
///
/// Verified against Python: `fnmatch('src/*.rs', 'src/deep/team.rs')` is True, and
/// `fnmatch('a?c', 'a/c')` is True.
fn fnmatch(pattern: &str, text: &str) -> bool {
    fn go(p: &[u8], t: &[u8]) -> bool {
        if p.is_empty() {
            return t.is_empty();
        }
        match p[0] {
            b'*' => {
                // Try every split. `**` behaves like `*` here, which matches fnmatch.
                let rest = &p[1..];
                if go(rest, t) {
                    return true;
                }
                for i in 0..t.len() {
                    if go(rest, &t[i + 1..]) {
                        return true;
                    }
                }
                false
            }
            b'?' => !t.is_empty() && go(&p[1..], &t[1..]),
            c => !t.is_empty() && t[0] == c && go(&p[1..], &t[1..]),
        }
    }
    go(pattern.as_bytes(), text.as_bytes())
}

/// Select lessons that apply to these files, this query, or this milestone.
pub fn match_lessons(
    mistakes: &[Mistake],
    files: &[String],
    query: &str,
    milestones: &[String],
) -> Vec<Match> {
    let q = query.trim().to_lowercase();
    let tokens = query_tokens(query);
    let mut out: Vec<Match> = Vec::new();

    for m in mistakes {
        // Only active and warn lessons surface. A closed lesson is history, and resurfacing it would
        // train readers to skim.
        if m.status != "active" && m.status != "warn" {
            continue;
        }
        let mut why: Vec<String> = Vec::new();
        for f in files {
            if m.paths.iter().any(|pat| path_matches(pat, f)) {
                why.push(format!("path {f}"));
            }
        }
        for pat in &m.trigger {
            let low = pat.to_lowercase();
            if !q.is_empty() && (q.contains(&low) || tokens.iter().any(|t| low.contains(t))) {
                why.push(format!("trigger {pat}"));
            }
        }
        if !milestones.is_empty() && m.milestones.iter().any(|x| milestones.contains(x)) {
            why.push("same milestone".to_string());
        }
        if !q.is_empty() && !m.mechanism.is_empty() && q.contains(&m.mechanism.to_lowercase()) {
            why.push(format!("mechanism {}", m.mechanism));
        }
        if !why.is_empty() {
            // Deduplicated and capped, then sorted so the report is stable between runs.
            why.sort();
            why.dedup();
            why.truncate(3);
            out.push(Match {
                mistake: m.clone(),
                why: why.join("; "),
            });
        }
    }

    // `block` before `warn` before `info`: the reader is about to edit code and needs the stopping
    // condition first.
    let rank = |s: &str| match s {
        "block" => 0,
        "warn" => 1,
        "info" => 2,
        _ => 3,
    };
    out.sort_by(|a, b| {
        rank(&a.mistake.severity)
            .cmp(&rank(&b.mistake.severity))
            .then_with(|| a.mistake.id.cmp(&b.mistake.id))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lesson(id: &str, severity: &str, status: &str) -> Mistake {
        Mistake {
            id: id.into(),
            title: format!("title {id}"),
            severity: severity.into(),
            status: status.into(),
            paths: vec!["crates/uor-r4-harness/src/*.rs".into()],
            trigger: vec!["stale binary".into()],
            milestones: vec!["m1".into()],
            ..Default::default()
        }
    }

    #[test]
    fn a_closed_lesson_never_surfaces() {
        let ms = vec![lesson("a", "block", "closed")];
        assert!(match_lessons(&ms, &[], "stale binary", &[]).is_empty());
    }

    #[test]
    fn active_and_warn_lessons_surface() {
        let ms = vec![lesson("a", "block", "active"), lesson("b", "warn", "warn")];
        assert_eq!(match_lessons(&ms, &[], "stale binary", &[]).len(), 2);
    }

    /// The reader is about to edit code; the stopping condition comes first.
    #[test]
    fn results_sort_block_then_warn_then_info() {
        let ms = vec![
            lesson("z-info", "info", "active"),
            lesson("a-block", "block", "active"),
            lesson("m-warn", "warn", "active"),
        ];
        let hits = match_lessons(&ms, &[], "stale binary", &[]);
        let ids: Vec<&str> = hits.iter().map(|h| h.mistake.id.as_str()).collect();
        assert_eq!(ids, vec!["a-block", "m-warn", "z-info"]);
    }

    #[test]
    fn the_why_names_which_rule_fired() {
        let ms = vec![lesson("a", "block", "active")];
        let hits = match_lessons(&ms, &["crates/uor-r4-harness/src/team.rs".into()], "", &[]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].why.contains("path"), "{}", hits[0].why);
        // And for a trigger, the trigger is named - a lesson surfaced for the wrong reason teaches the
        // wrong thing.
        let hits = match_lessons(&ms, &[], "why is my stale binary wrong", &[]);
        assert!(hits[0].why.contains("trigger"), "{}", hits[0].why);
    }

    /// A short token appears in almost every trigger, so admitting it would match everything and mean
    /// nothing.
    /// Four characters or fewer is dropped: a token like `the` appears in almost every trigger, so
    /// admitting it would match everything and mean nothing.
    ///
    /// **`-` and `_` are TOKEN characters, not separators.** `stale-binary_x` is one token. My first
    /// test asserted it split into three, which is the opposite of the rule - the test was wrong, not
    /// the code.
    #[test]
    fn query_tokens_shorter_than_four_characters_are_ignored() {
        assert!(query_tokens("the a of").is_empty());
        assert_eq!(query_tokens("stale binary"), vec!["stale", "binary"]);
        assert_eq!(
            query_tokens("stale-binary_x"),
            vec!["stale-binary_x"],
            "hyphen and underscore join"
        );
        // The length test is on the TOKEN, so two-letter words are dropped too. My first two
        // versions of this line were wrong in the same direction: I checked the word length I had in
        // mind rather than what the rule says, and the rule is "token longer than three".
        assert_eq!(
            query_tokens("ab cd"),
            Vec::<String>::new(),
            "both tokens are two characters, so neither survives"
        );
        assert_eq!(
            query_tokens("abcd ef"),
            vec!["abcd"],
            "four survives, two does not"
        );
    }

    /// A guard that stops firing after a rename is indistinguishable from a guard that passes.
    #[test]
    fn a_path_matches_by_glob_and_by_substring() {
        assert!(path_matches(
            "crates/uor-r4-harness/src/*.rs",
            "crates/uor-r4-harness/src/team.rs"
        ));
        assert!(
            path_matches("team.rs", "crates/uor-r4-harness/src/team.rs"),
            "a bare filename still fires for a path containing it"
        );
        assert!(!path_matches("*.py", "a.rs"));
    }

    /// **Python `fnmatch` semantics, verified against Python.** `*` and `?` both cross `/`, which is
    /// the opposite of a shell glob - and assuming shell behaviour meant `*team.rs*` did not match
    /// `crates/x/src/team.rs`, so a lesson written against a bare filename silently stopped firing.
    #[test]
    fn stars_and_questions_cross_separators_like_python_fnmatch() {
        assert!(
            fnmatch("src/*.rs", "src/deep/team.rs"),
            "fnmatch: '*' crosses '/'"
        );
        assert!(
            fnmatch("*team.rs*", "crates/x/src/team.rs"),
            "a bare filename still fires"
        );
        assert!(fnmatch("a?c", "a/c"), "fnmatch: '?' matches '/'");
        assert!(!fnmatch("*.py", "a.rs"));
    }

    #[test]
    fn a_milestone_hit_is_reported_as_such() {
        let ms = vec![lesson("a", "warn", "active")];
        let hits = match_lessons(&ms, &[], "", &["m1".into()]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].why, "same milestone");
    }

    #[test]
    fn a_severity_and_status_default_so_an_unlabelled_lesson_still_surfaces() {
        let m = parse("my-lesson", "---\ntitle: x\n---\n", Path::new("f.md"));
        assert_eq!(m.id, "my-lesson", "the filename is the fallback id");
        assert_eq!(
            m.severity, "warn",
            "defaulting to info would bury it below everything"
        );
        assert_eq!(m.status, "active");
    }

    #[test]
    fn frontmatter_lists_parse() {
        let m = parse(
            "x",
            "---\npaths: [a.rs, b.rs]\ntrigger: [one, two]\nmilestones: [m1, m2]\n---\n",
            Path::new("f.md"),
        );
        assert_eq!(m.paths, vec!["a.rs", "b.rs"]);
        assert_eq!(m.trigger, vec!["one", "two"]);
        assert_eq!(m.milestones, vec!["m1", "m2"]);
    }

    /// Against the REAL registry: a broken lesson file must fail here rather than never surfacing.
    #[test]
    fn the_real_registry_loads_and_every_lesson_has_an_id() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let ms = load(&paths);
        assert!(
            ms.len() >= 10,
            "expected the real lesson set, got {}",
            ms.len()
        );
        for m in &ms {
            assert!(!m.id.is_empty(), "{}", m.path.display());
            assert!(!m.title.is_empty(), "{} has no title", m.id);
            assert_ne!(m.id, "README", "the index is not a lesson");
        }
    }

    /// `find` must actually surface something for a real file, or the command is decorative.
    #[test]
    fn a_real_file_surfaces_real_lessons() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let ms = load(&paths);
        let hits = match_lessons(
            &ms,
            &["crates/uor-r4-harness/src/status.rs".to_string()],
            "",
            &[],
        );
        assert!(
            !hits.is_empty(),
            "no lesson applies to the harness source - the registry or the matcher is wrong"
        );
    }
}

#[cfg(test)]
mod rule_tests {
    use super::*;
    use crate::state::StatePaths;

    fn fixture() -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let d = dir.path().join("docs/harness/mistakes");
        std::fs::create_dir_all(&d).expect("mkdir");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        (dir, p)
    }

    fn lesson(stem: &str, body: &str) -> String {
        format!(
            "---\nid: {stem}\ntitle: t\ndate: 2026-10-13\nseverity: block\nclass: process\n\
             status: active\nguard: prlint.record\ntrigger: [one, two]\npaths: [\"crates/*.rs\"]\n\
             evidence: [Cargo.toml]\n---\n\n{body}\n"
        )
    }

    /// The real registry is valid — the validator must not be stricter than Python's.
    #[test]
    fn the_real_registry_has_no_validation_errors() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let p = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let f = registry_valid(&p);
        assert!(f.is_empty(), "the real registry must be valid: {f:?}");
    }

    /// **A duplicate id makes the registry ambiguous** — a lookup by id returns whichever file was
    /// read first, which is not a rule anyone can reason about.
    #[test]
    fn a_duplicate_lesson_id_is_an_error_naming_both_files() {
        let (d, p) = fixture();
        let dir = d.path().join("docs/harness/mistakes");
        std::fs::write(dir.join("dup.md"), lesson("dup", "one")).expect("write");
        std::fs::write(dir.join("other.md"), lesson("dup", "two")).expect("write");
        let f = registry_valid(&p);
        assert!(
            f.iter().any(|x| x.message.contains("duplicate lesson id")),
            "a duplicate id must be reported: {f:?}"
        );
        assert!(f.iter().any(|x| x.message.contains("dup.md")), "{f:?}");
    }

    /// A file whose id does not equal its stem is invalid: the registry is navigated by filename.
    #[test]
    fn an_id_that_does_not_match_the_stem_is_reported() {
        let (d, p) = fixture();
        std::fs::write(
            d.path().join("docs/harness/mistakes/wrong-stem.md"),
            lesson("a-different-id", "x"),
        )
        .expect("write");
        let f = registry_valid(&p);
        assert!(
            f.iter()
                .any(|x| x.message.contains("must equal the file stem")),
            "{f:?}"
        );
    }

    /// **A citation to a file that does not exist is worse than no citation, because it reads as
    /// evidence.**
    #[test]
    fn a_missing_evidence_path_is_an_error() {
        let (d, p) = fixture();
        std::fs::write(
            d.path().join("docs/harness/mistakes/cite.md"),
            lesson("cite", "x")
                .replace("evidence: [Cargo.toml]", "evidence: [docs/nowhere/none.md]"),
        )
        .expect("write");
        let f = evidence_exists(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("does not exist"), "{}", f[0].message);
    }

    #[test]
    fn an_existing_evidence_path_is_accepted() {
        let (d, p) = fixture();
        std::fs::write(d.path().join("Cargo.toml"), "[workspace]\n").expect("write");
        std::fs::write(
            d.path().join("docs/harness/mistakes/ok.md"),
            lesson("ok", "x"),
        )
        .expect("write");
        assert!(evidence_exists(&p).is_empty(), "{:?}", evidence_exists(&p));
    }
}

#[cfg(test)]
mod guard_rule_tests {
    use super::*;
    use crate::state::StatePaths;
    use std::path::Path;

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn fixture(body: &str) -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let d = dir.path().join("docs/harness/mistakes");
        std::fs::create_dir_all(&d).expect("mkdir");
        std::fs::write(d.join("probe-lesson.md"), body).expect("write");
        let p = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        (dir, p)
    }

    fn lesson(severity: &str, guard: &str) -> String {
        format!(
            "---\nid: probe-lesson\ntitle: t\ndate: 2026-10-13\nseverity: {severity}\nclass: process\n\
             status: active\nguard: {guard}\ntrigger: [one, two]\npaths: [\"crates/*.rs\"]\n\
             evidence: [Cargo.toml]\n---\n\nbody\n"
        )
    }

    /// **The file the bug produced.** 53 ids, not 40: the engine resolves guards from three sources,
    /// and taking only `checkrules.rules()` reported four real lessons as unregistered — **the same
    /// false positive, in a new place.**
    #[test]
    fn the_data_file_holds_every_id_the_engine_registers() {
        let ids = registered_rules(&paths());
        assert_eq!(
            ids.len(),
            53,
            "the engine's set is 53 ids across three sources"
        );
        assert!(
            ids.iter().any(|i| i == "prlint.no_cpu_model_compute"),
            "prlint GATE_RULES are part of the set - omitting them was the false positive"
        );
        assert!(
            ids.iter().any(|i| i.starts_with("arena.")),
            "checkrules ids too"
        );
    }

    fn paths() -> StatePaths {
        StatePaths::resolve(repo(), None, Some("/nonexistent-live"))
    }

    /// **The real registry passes**, which is the property that matters: four real lessons guard on
    /// `prlint.*` gates, and reporting those as unregistered would tell authors to delete them.
    #[test]
    fn the_real_lessons_all_resolve() {
        let f = guard_covered(&paths());
        assert!(f.is_empty(), "a real guard failed to resolve: {f:?}");
    }

    /// A guard naming something that genuinely is not a rule is still reported.
    #[test]
    fn an_invented_guard_is_still_reported() {
        let (_d, p) = fixture(&lesson("block", "made.up.nonsense"));
        let f = guard_covered(&p);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message.contains("not a rule this engine registers"),
            "{}",
            f[0].message
        );
    }

    /// `none:` with no reason is the escape used without saying why.
    #[test]
    fn none_needs_a_reason() {
        let (_d, p) = fixture(&lesson("block", "none:"));
        assert!(guard_covered(&p)
            .iter()
            .any(|x| x.message.contains("gives no reason")));
        let (_d2, p2) = fixture(&lesson("block", "none: no machine can decide this"));
        assert!(
            guard_covered(&p2).is_empty(),
            "a reasoned none: is accepted"
        );
    }

    /// **Only `block` severity.** A `warn` lesson is advice and may have no machine behind it.
    #[test]
    fn a_warn_lesson_needs_no_guard() {
        let (_d, p) = fixture(&lesson("warn", "made.up.nonsense"));
        assert!(guard_covered(&p).is_empty(), "warn lessons are advice");
    }
}

// ---------------------------------------------------------------------------------------------
// `mistakes.stale_check` - every lesson's paths still match a file in the repo.
// ---------------------------------------------------------------------------------------------

/// The repo's file index, from **`git ls-files` rather than a filesystem walk**.
///
/// Python's original walked ~11k files and ran three `fnmatch` calls per (file, pattern) pair for
/// every lesson, which took **~46 seconds and dominated the whole `check`**. `git ls-files` is the
/// already-built index of exactly the files that matter, and it excludes ignored build output and
/// scratch state for free.
///
/// **`-z` because paths can contain newlines**, which the newline-separated form splits incorrectly.
fn file_index(repo: &std::path::Path) -> Vec<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .output();
    let Ok(out) = out else { return Vec::new() };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// `mistakes._patterns_match_index` - does any pattern match any indexed path?
///
/// **Ported for its semantics rather than its optimisation.** Python splits the patterns into
/// literals (resolved with `in`/`startswith`) and wildcards (resolved with `fnmatch`) because the
/// hot path was tens of millions of calls; **Rust does the same in one pass with the same outcomes**,
/// and the split is kept because the literal branch has *different* matching rules, not merely
/// faster ones: a literal matches by suffix or substring, a glob only by `fnmatch`.
fn patterns_match_index(index: &[String], patterns: &[String]) -> bool {
    let mut literals: Vec<(String, Option<String>)> = Vec::new();
    let mut globs: Vec<String> = Vec::new();
    for raw in patterns {
        let pat = raw.trim().trim_start_matches("./").to_string();
        if pat.is_empty() {
            continue;
        }
        if pat.contains('*') || pat.contains('?') || pat.contains('[') {
            if let Some(base) = pat.strip_suffix("/**") {
                literals.push((
                    pat.clone(),
                    Some(format!("{}/", base.trim_end_matches('/'))),
                ));
            } else {
                globs.push(pat);
            }
        } else {
            literals.push((pat, None));
        }
    }
    // **An empty pattern list is not a staleness finding**, and that is a deliberate early return
    // rather than a vacuous truth that happens to agree.
    if literals.is_empty() && globs.is_empty() {
        return true;
    }
    for (pat, prefix) in &literals {
        for rel in index {
            if rel == pat
                || rel.ends_with(&format!("/{pat}"))
                || prefix.as_ref().is_some_and(|p| rel.starts_with(p))
                || rel.contains(pat.as_str())
            {
                return true;
            }
        }
    }
    for glob in &globs {
        for rel in index {
            if fnmatch(glob, rel) {
                return true;
            }
        }
    }
    false
}

/// `mistakes.stale_check` - **an `info` finding, because a stale glob is not a fault.**
///
/// **A lesson whose paths match nothing has stopped being recallable**, and that is the failure the
/// rule exists for: **a stale glob silently stops firing, which is indistinguishable from a guard
/// that passes.** The severity is Python's.
pub fn stale_check(paths: &StatePaths) -> Vec<Finding> {
    let index = file_index(&paths.repo);
    if index.is_empty() {
        // **No index means nothing to compare, and reporting every lesson as stale would be a
        // finding that is wrong rather than missing.** Python's `git ls-files` on a real repo always
        // returns paths; an empty list means git is unavailable, which is not a lesson's fault.
        return Vec::new();
    }
    let mut out = Vec::new();
    for lesson in load(paths) {
        if !lesson.errors.is_empty() || lesson.paths.is_empty() {
            continue;
        }
        if !patterns_match_index(&index, &lesson.paths) {
            out.push(Finding {
                rule_id: "mistakes.stale_check",
                severity: "info",
                message: format!(
                    "{}: paths {:?} match no file in the repo",
                    lesson.id, lesson.paths
                ),
                path: lesson.path.display().to_string(),
                fix: "update or delete the lesson: a pattern that matches nothing never recalls it"
                    .into(),
            });
        }
    }
    out
}

#[cfg(test)]
mod stale_check_tests {
    use super::*;
    use std::path::Path;

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn idx() -> Vec<String> {
        file_index(&repo())
    }

    /// **The matcher's two branches have DIFFERENT rules, not merely different speeds.** A literal
    /// matches by exact path, suffix **or substring**; a glob matches only by `fnmatch`. Porting only
    /// the fast path would have made a lesson written as a bare filename stop firing.
    #[test]
    fn literals_and_globs_match_by_their_own_rules() {
        let index = idx();
        assert!(!index.is_empty(), "the real repo has a file index");
        assert!(
            patterns_match_index(&index, &["Cargo.toml".into()]),
            "a literal that exists"
        );
        assert!(
            patterns_match_index(&index, &["crates/*.rs".into()]),
            "a glob that matches"
        );
        // A literal matches as a SUBSTRING, which a glob would not.
        assert!(
            patterns_match_index(&index, &["crates/uor-r4-harness/src/team".into()]),
            "a literal matches by substring"
        );
        assert!(
            !patterns_match_index(&index, &["nothing/here/**".into()]),
            "a pattern matching nothing"
        );
        assert!(!patterns_match_index(
            &index,
            &["zzz-no-such-file.zzz".into()]
        ));
    }

    /// **`<dir>/**` does not become a glob**: Python rewrites it to a literal directory prefix, so it
    /// matches by `startswith`. Asserted because the rewrite is easy to drop and the glob it would
    /// otherwise become matches differently.
    #[test]
    fn a_double_star_suffix_becomes_a_prefix_literal() {
        let index = idx();
        assert!(patterns_match_index(&index, &["crates/**".into()]));
        assert!(patterns_match_index(&index, &["docs/harness/**".into()]));
        assert!(!patterns_match_index(&index, &["no-such-dir/**".into()]));
    }

    /// **An empty pattern list is not a staleness finding** - a deliberate early return, not a
    /// vacuous truth that happens to agree.
    #[test]
    fn an_empty_pattern_list_is_not_stale() {
        assert!(patterns_match_index(&idx(), &[]));
        assert!(
            patterns_match_index(&idx(), &["".into(), "  ".into()]),
            "blank patterns are skipped"
        );
    }

    /// **The real repo has no stale lessons**, which is Python's own answer - so this asserts the
    /// port agrees on real data rather than only on constructed cases.
    #[test]
    fn the_real_repo_has_no_stale_lessons() {
        let paths = StatePaths::resolve(repo(), None, Some("/nonexistent-live"));
        let f = stale_check(&paths);
        assert!(
            f.is_empty(),
            "python reports 0 on this repo; rust found: {f:?}"
        );
    }

    /// **A stale lesson IS reported, with Python's message and severity.** Constructed rather than
    /// found, because the real repo has none - **and a check that has only ever been green is a check
    /// nobody has tested.**
    #[test]
    fn a_lesson_matching_nothing_is_reported() {
        let dir = tempfile::tempdir().expect("tempdir");
        let d = dir.path().join("docs/harness/mistakes");
        std::fs::create_dir_all(&d).expect("mkdir");
        std::fs::write(
            d.join("stale-lesson.md"),
            "---\nid: stale-lesson\ntitle: t\ndate: 2026-10-14\nseverity: info\nclass: process\n\
             status: active\nguard: none: no machine can decide this\ntrigger: [one, two]\n\
             paths: [\"no/such/path/anywhere/**\"]\nevidence: [Cargo.toml]\n---\n\nbody\n",
        )
        .expect("write");
        // **A git repo with something in it, because the index comes from `git ls-files`** - an empty
        // index means "no evidence", and the check deliberately reports nothing in that case rather
        // than reporting every lesson. I hit that here first: the fixture reported 0 findings and the
        // check was right.
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "t@t"],
            vec!["config", "user.name", "t"],
            vec!["add", "docs/harness/mistakes/stale-lesson.md"],
        ] {
            let _ = std::process::Command::new("git")
                .arg("-C")
                .arg(dir.path())
                .args(&args)
                .output();
        }
        let paths = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        let f = stale_check(&paths);
        assert_eq!(f.len(), 1, "one stale lesson: {f:?}");
        assert_eq!(f[0].rule_id, "mistakes.stale_check");
        assert_eq!(f[0].severity, "info", "a stale glob is not a fault");
        assert!(f[0].message.contains("stale-lesson"), "{}", f[0].message);
        assert!(
            f[0].message.contains("match no file in the repo"),
            "{}",
            f[0].message
        );
    }

    /// **No file index means no findings**, because reporting every lesson as stale would be a
    /// finding that is WRONG rather than missing - and git being unavailable is not a lesson's fault.
    #[test]
    fn no_index_reports_nothing_rather_than_everything() {
        let dir = tempfile::tempdir().expect("tempdir");
        let d = dir.path().join("docs/harness/mistakes");
        std::fs::create_dir_all(&d).expect("mkdir");
        std::fs::write(
            d.join("a-lesson.md"),
            "---\nid: a-lesson\ntitle: t\ndate: 2026-10-14\nseverity: info\nclass: process\n\
             status: active\nguard: none: no machine can decide this\ntrigger: [one, two]\n\
             paths: [\"**/*.rs\"]\nevidence: [Cargo.toml]\n---\n\nbody\n",
        )
        .expect("write");
        // A directory that is not a git repository, so `git ls-files` fails.
        let paths = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        assert!(
            stale_check(&paths).is_empty(),
            "no index must not read as every lesson being stale"
        );
    }
}

// ------------------------------------------------------------------------------- show, capture

/// `registries.find_mistake`: an exact id first, then the first prefix match.
pub fn find_mistake<'a>(lessons: &'a [Mistake], key: &str) -> Option<&'a Mistake> {
    lessons
        .iter()
        .find(|m| m.id == key)
        .or_else(|| lessons.iter().find(|m| m.id.starts_with(key)))
}

/// The four body sections a lesson is read for. `mistakes.SECTIONS`.
pub const SECTIONS: [&str; 4] = ["What happened", "Symptom", "Lesson", "How to apply"];

/// One lesson's fields, as `mistakes capture` assembles them.
#[derive(Debug, Clone)]
pub struct LessonDraft {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub class: String,
    pub status: String,
    pub trigger: Vec<String>,
    pub paths: Vec<String>,
    pub milestones: Vec<String>,
    pub mechanism: String,
    pub guard: String,
    pub evidence: Vec<String>,
    pub date: String,
    pub what: String,
    pub symptom: String,
    pub lesson: String,
    pub apply: String,
}

impl Default for LessonDraft {
    /// Python's defaults, which are also the CLI's: `warn`, `process`, `active`.
    fn default() -> Self {
        LessonDraft {
            id: String::new(),
            title: String::new(),
            severity: "warn".to_string(),
            class: "process".to_string(),
            status: "active".to_string(),
            trigger: Vec::new(),
            paths: Vec::new(),
            milestones: Vec::new(),
            mechanism: String::new(),
            guard: String::new(),
            evidence: Vec::new(),
            date: String::new(),
            what: String::new(),
            symptom: String::new(),
            lesson: String::new(),
            apply: String::new(),
        }
    }
}

/// `util.truncate`: collapse whitespace, then cut at `limit` CHARACTERS with an ellipsis.
pub fn truncate(text: &str, limit: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= limit {
        return collapsed;
    }
    let cut: String = collapsed.chars().take(limit.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}

fn non_blank(items: &[String]) -> Vec<String> {
    items
        .iter()
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .collect()
}

/// `_scalar`: a plain YAML scalar when it is safe, quoted otherwise.
///
/// The plain set is Python's own regex `[A-Za-z0-9_./*<>#-]+`, which is why `docs/harness/**` is
/// written bare and `none: captured by hand on …` is not: **the colon would end the scalar.**
fn yaml_scalar(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    let plain = !value.contains(':')
        && !value.starts_with('-')
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_./*<>#-".contains(c));
    if plain {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('\"', "\\\""))
    }
}

fn yaml_list(items: &[String]) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }
    let parts: Vec<String> = items.iter().map(|s| yaml_scalar(s)).collect();
    format!("[{}]", parts.join(", "))
}

/// `build_lesson_text`: render a lesson that is registry-valid and honest about what is missing.
///
/// **The placeholders are the point.** A capture with no `--symptom` still produces a lesson whose
/// `## Symptom` section says it was not recorded, so `registry_valid` reads a complete file and a
/// reader sees a gap rather than an empty heading.
pub fn build_lesson_text(d: &LessonDraft) -> String {
    let date = if d.date.trim().is_empty() {
        crate::prep::today()
    } else {
        d.date.clone()
    };
    let mut triggers = non_blank(&d.trigger);
    let mut path_globs = non_blank(&d.paths);
    let mut evidence = non_blank(&d.evidence);
    if triggers.is_empty() {
        // The id with spaces, and the title cut to 40 characters: enough for `find` to match on
        // without inventing a trigger nobody wrote.
        triggers = vec![d.id.replace('-', " "), truncate(&d.title, 40)];
    }
    if path_globs.is_empty() {
        path_globs = vec!["docs/harness/**".to_string()];
    }
    if evidence.is_empty() {
        evidence = vec!["docs/labs/README.md".to_string()];
    }
    let guard = if d.guard.trim().is_empty() {
        // A capture with no guard says so, in the field a reader looks at to find out what stops it.
        format!("none: captured by hand on {date}; no rule decides it yet")
    } else {
        d.guard.clone()
    };
    let milestones = non_blank(&d.milestones);
    // **Every value goes through the YAML renderer, not only the ones with punctuation in them.**
    // Python writes `f"{key}: {_yaml(fm[key])}"` for all twelve, and skipping it for `title` meant a
    // title with a space was written bare while Python quoted it - a one-character difference in
    // every captured lesson, found by diffing the written file.
    let fm: Vec<(&str, String)> = vec![
        ("id", yaml_scalar(&d.id)),
        ("title", yaml_scalar(&d.title)),
        ("date", yaml_scalar(&date)),
        ("severity", yaml_scalar(&d.severity)),
        ("class", yaml_scalar(&d.class)),
        ("trigger", yaml_list(&triggers)),
        ("paths", yaml_list(&path_globs)),
        ("milestones", yaml_list(&milestones)),
        ("mechanism", yaml_scalar(&d.mechanism)),
        ("guard", yaml_scalar(&guard)),
        ("evidence", yaml_list(&evidence)),
        ("status", yaml_scalar(&d.status)),
    ];
    let front_matter: String = fm.iter().map(|(k, v)| format!("{k}: {v}\n")).collect();
    let what = placeholder(
        &d.what,
        "(not recorded yet: what the run or delivery actually did)",
    );
    let symptom = placeholder(
        &d.symptom,
        "(not recorded yet: what someone would see that made them look this up)",
    );
    let lesson = placeholder(
        &d.lesson,
        "(not recorded yet: the transferable rule this lesson teaches)",
    );
    let apply = placeholder(
        &d.apply,
        "(not recorded yet: what to do differently, with the exact command or check)",
    );
    // One `format!` rather than chained `replace` calls: a substituted value containing `{what}`
    // must not be substituted again, which is what Python's single `.format()` does.
    format!(
        "---\n{front_matter}---\n\n## What happened\n\n{what}\n\n## Symptom\n\n{symptom}\n\n## Lesson\n\n{lesson}\n\n## How to apply\n\n{apply}\n"
    )
}

fn placeholder(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod show_capture_tests {
    use super::*;

    /// **Python's own bytes**, from `mistakes.build_lesson_text` with a fixed date. A fixture that
    /// encoded the moment it was captured would be the trap this port has hit three times, so the
    /// date is passed in and the test asserts FORMAT.
    #[test]
    fn the_generated_lesson_is_pythons_bytes() {
        let draft = LessonDraft {
            id: "fixture-lesson".into(),
            title: "A fixture lesson".into(),
            severity: "block".into(),
            class: "delivery".into(),
            trigger: vec!["alpha".into(), "beta".into()],
            paths: vec!["src/**".into()],
            milestones: vec!["m1".into()],
            mechanism: "zeta".into(),
            guard: "check.rule".into(),
            evidence: vec!["docs/labs/README.md".into()],
            date: "2026-10-14".into(),
            symptom: "it broke".into(),
            lesson: "do not do it".into(),
            ..Default::default()
        };
        assert_eq!(
            build_lesson_text(&draft),
            concat!(
                "---\n",
                "id: fixture-lesson\n",
                "title: \"A fixture lesson\"\n",
                "date: 2026-10-14\n",
                "severity: block\n",
                "class: delivery\n",
                "trigger: [alpha, beta]\n",
                "paths: [src/**]\n",
                "milestones: [m1]\n",
                "mechanism: zeta\n",
                "guard: check.rule\n",
                "evidence: [docs/labs/README.md]\n",
                "status: active\n",
                "---\n\n## What happened\n\n",
                "(not recorded yet: what the run or delivery actually did)\n\n",
                "## Symptom\n\nit broke\n\n",
                "## Lesson\n\ndo not do it\n\n",
                "## How to apply\n\n",
                "(not recorded yet: what to do differently, with the exact command or check)\n"
            )
        );
    }

    /// **The defect `capture` found: a quoted front-matter value kept its quotes**, so the lesson the
    /// command had just written failed the guard rule and the write was refused.
    #[test]
    fn a_quoted_front_matter_value_loses_its_quotes() {
        let text =
            "---\nid: x\nguard: \"none: captured by hand on 2026-10-14\"\n---\n\n## Lesson\n\nz\n";
        let m = parse("x", text, Path::new("x.md"));
        assert_eq!(m.guard, "none: captured by hand on 2026-10-14");
        assert!(
            !m.errors.iter().any(|e| e.contains("dotted rule id")),
            "a quoted `none:` guard must not be read as a rule id: {:?}",
            m.errors
        );
    }

    /// Every message here was compared against Python on the same lesson file; the tuple and `!r`
    /// forms are Python's, not Rust's debug formatting.
    #[test]
    fn the_validation_messages_are_pythons() {
        let text = concat!(
            "---\n",
            "id: bad-lesson\ntitle: bad lesson\ndate: 14-10-2026\nseverity: warn\nclass: nope\n",
            "trigger: [only-one]\npaths: []\nevidence: docs/labs/README.md\nstatus: nope\n",
            "guard: \"some guard\"\n---\n\n## Lesson\n\nz\n"
        );
        let fm = frontmatter(text);
        let errors = validation_errors("bad-lesson", text, &fm);
        assert_eq!(
            errors,
            vec![
                "front matter is missing 'paths'",
                "class must be one of ('process', 'evidence', 'compute', 'measurement', 'delivery', 'coordination', 'assumption'), got 'nope'",
                "status must be one of ('active', 'resolved', 'superseded'), got 'nope'",
                "date must be YYYY-MM-DD, got '14-10-2026'",
                "guard 'some guard' must be a dotted rule id or 'none: <reason>'",
                "'evidence' must be an inline or block list",
                "'trigger' needs at least two phrases, or the lesson is not retrievable",
                "'paths' is empty: name a glob so the lesson is recalled",
            ]
        );
    }

    #[test]
    fn find_mistake_takes_an_exact_id_then_a_prefix() {
        let mut a = Mistake {
            id: "alpha-one".into(),
            ..Default::default()
        };
        a.path = PathBuf::from("alpha-one.md");
        let mut b = Mistake {
            id: "alphabet".into(),
            ..Default::default()
        };
        b.path = PathBuf::from("alphabet.md");
        let lessons = vec![a, b];
        assert_eq!(
            find_mistake(&lessons, "alphabet").expect("exact").id,
            "alphabet"
        );
        assert_eq!(
            find_mistake(&lessons, "alph").expect("prefix").id,
            "alpha-one"
        );
        assert!(find_mistake(&lessons, "nope").is_none());
    }

    /// The warning that every capture without `--mechanism` produces, plus the evidence problem.
    #[test]
    fn draft_problems_reports_pythons_warning_and_evidence_problem() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join("docs").join("harness")).expect("mkdir");
        let paths = StatePaths::resolve(dir.path(), None, Some("/nonexistent-live"));
        let target = dir
            .path()
            .join("docs")
            .join("harness")
            .join("mistakes")
            .join("x.md");
        let draft = LessonDraft {
            id: "x".into(),
            title: "X".into(),
            date: "2026-10-14".into(),
            ..Default::default()
        };
        let text = build_lesson_text(&draft);
        let (problems, warnings) = draft_problems(&paths, &target, &text);
        assert!(
            warnings.iter().any(|w| w.contains("'mechanism' is empty")),
            "{warnings:?}"
        );
        assert!(
            problems.iter().any(|p| p.contains("evidence path")),
            "the scratch repo has no docs/labs: {problems:?}"
        );
    }

    #[test]
    fn py_tuple_and_yaml_quoting_are_pythons() {
        assert_eq!(py_tuple(&["a", "b"]), "('a', 'b')");
        assert_eq!(yaml_scalar("plain"), "plain");
        assert_eq!(yaml_scalar("has space"), "\"has space\"");
        assert_eq!(yaml_scalar("has: colon"), "\"has: colon\"");
        assert_eq!(yaml_scalar(""), "\"\"");
        assert_eq!(yaml_scalar("docs/harness/**"), "docs/harness/**");
    }

    /// The assumption behind `draft_problems` implementing only one warning source: every draft the
    /// command writes carries all four sections with text in them, so Python's body warnings cannot
    /// fire. If the template changes, this fails rather than the warning going missing.
    #[test]
    fn the_template_carries_every_section_with_text() {
        let draft = LessonDraft {
            id: "x".into(),
            title: "X".into(),
            date: "2026-10-14".into(),
            ..Default::default()
        };
        let text = build_lesson_text(&draft);
        for section in SECTIONS {
            let heading = format!("## {section}");
            let at = text
                .find(&heading)
                .unwrap_or_else(|| panic!("missing {heading}"));
            let body = &text[at + heading.len()..];
            let first = body.trim_start().lines().next().unwrap_or_default();
            assert!(!first.trim().is_empty(), "{heading} is empty");
        }
    }
}

/// `_draft_problems`: what the registry rules would say about a draft BEFORE it is written.
///
/// Errors refuse the write; warnings are reported and allowed - a captured lesson with no mechanism
/// is honest, just less useful. **The check exists so `capture` cannot write a file that
/// `mistakes.registry_valid` rejects**, which would turn a lab's own record into a failing `check`.
///
/// **One warning source is implemented and the others cannot fire here.** Python's reader also warns
/// about a missing or empty body section; every draft this module writes comes from
/// `build_lesson_text`, which always emits all four [`SECTIONS`] with non-empty placeholders, and
/// `the_template_carries_every_section_with_text` asserts exactly that - so the assumption is
/// checked rather than assumed.
pub fn draft_problems(paths: &StatePaths, target: &Path, text: &str) -> (Vec<String>, Vec<String>) {
    let name = target
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let stem = target
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let lesson = parse(&stem, text, target);
    let mut problems: Vec<String> = lesson
        .errors
        .iter()
        .map(|e| format!("{name}: {e}"))
        .collect();
    let mut warnings: Vec<String> = Vec::new();
    if lesson.mechanism.trim().is_empty() {
        warnings.push(format!(
            "{name}: 'mechanism' is empty: the lesson is not recalled by mechanism name"
        ));
    }
    for value in &lesson.evidence {
        if resolve_path(paths, value).is_none() {
            problems.push(format!("{name}: evidence path {value} does not exist"));
        }
    }
    if load(paths)
        .iter()
        .any(|m| m.path != target && m.id == lesson.id)
    {
        problems.push(format!(
            "{name}: id {} is already used by another lesson",
            crate::prgates::py_repr_public(&lesson.id)
        ));
    }
    (problems, warnings)
}
