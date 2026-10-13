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
    Mistake {
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
