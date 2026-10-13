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
        errors.push(format!("id {id:?} must equal the file stem {stem:?}"));
    }
    if !id.is_empty() && !kebab_case_id(&id) {
        errors.push(format!("id {id:?} is not a kebab-case id"));
    }
    let sev = get("severity");
    if !sev.is_empty() && !SEVERITIES.contains(&sev.as_str()) {
        errors.push(format!(
            "severity must be one of {SEVERITIES:?}, got {sev:?}"
        ));
    }
    let class = get("class");
    if !class.is_empty() && !CLASSES.contains(&class.as_str()) {
        errors.push(format!("class must be one of {CLASSES:?}, got {class:?}"));
    }
    let status = get("status");
    if !status.is_empty() && !STATUSES.contains(&status.as_str()) {
        errors.push(format!(
            "status must be one of {STATUSES:?}, got {status:?}"
        ));
    }
    let date = get("date");
    if !date.is_empty() && !is_ymd(&date) {
        errors.push(format!("date must be YYYY-MM-DD, got {date:?}"));
    }
    let guard = get("guard");
    if !guard.is_empty() && !guard.starts_with("none") && !guard.contains('.') {
        errors.push(format!(
            "guard {guard:?} must be a dotted rule id or 'none: <reason>'"
        ));
    }
    errors
}

/// `^[a-z0-9][a-z0-9-]{2,63}$` — at least three characters, kebab-case.
fn kebab_case_id(id: &str) -> bool {
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
            out.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    out
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
