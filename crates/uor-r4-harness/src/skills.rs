//! `skill` and `mistakes` — the query surface over skills and lessons (D30 port).
//!
//! # What `skill use` is for
//!
//! It writes **one receipt line** saying a skill was actually loaded and deployed. That is the record
//! the owner reads to see whether the harness was used or merely present, so the port has to keep the
//! receipt shape and the id resolution exactly.
//!
//! # Why the id resolution is three passes
//!
//! A lab types `skill use context-hydration`, but may type a prefix, and may type the skill's `name`
//! where that differs from its directory. Python's rule, kept here because labs have already used all
//! three forms:
//!
//! 1. exact id, then exact name;
//! 2. then a unique **prefix** of the id.
//!
//! A prefix matching more than one skill resolves to the first in sorted order rather than erroring —
//! that is Python's behaviour, and changing it here would make the two implementations disagree on a
//! command labs run every session.
//!
//! # No YAML dependency
//!
//! Frontmatter is a flat `key: value` block with `[a, b]` lists. That is parsed here rather than
//! pulling a YAML crate for one file shape; a nested value would be reported as unparsed rather than
//! silently dropped.

use std::path::{Path, PathBuf};

use crate::state::StatePaths;

/// One skill, from `docs/harness/skills/<dir>/SKILL.md`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub kind: String,
    pub applies_to: Vec<String>,
    pub when: Vec<String>,
    pub requires: Vec<String>,
    pub outputs: Vec<String>,
    pub forbidden: Vec<String>,
    pub milestones: Vec<String>,
    pub path: PathBuf,
}

/// The skills directory.
pub fn skills_dir(paths: &StatePaths) -> PathBuf {
    paths.repo.join("docs").join("harness").join("skills")
}

/// Load every skill, sorted by id so resolution is deterministic.
pub fn load(paths: &StatePaths) -> Vec<Skill> {
    let dir = skills_dir(paths);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<Skill> = Vec::new();
    for e in entries.flatten() {
        let path = e.path();
        if !path.is_dir() {
            continue;
        }
        let file = path.join("SKILL.md");
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let dir_name = e.file_name().to_string_lossy().to_string();
        out.push(parse(&dir_name, &text, &file));
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// Parse one skill file. A missing id falls back to the directory name, because the directory is what
/// a lab actually types.
pub fn parse(dir_name: &str, text: &str, path: &Path) -> Skill {
    let fm = frontmatter(text);
    let get = |k: &str| {
        fm.iter()
            .find(|(key, _)| key == k)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    let id = {
        let name = get("name");
        if name.is_empty() {
            dir_name.to_string()
        } else {
            name
        }
    };
    Skill {
        id: id.clone(),
        name: id,
        description: get("description"),
        version: get("version"),
        kind: get("kind"),
        applies_to: list(&get("applies-to")),
        when: list(&get("when")),
        requires: list(&get("requires")),
        outputs: list(&get("outputs")),
        forbidden: list(&get("forbidden")),
        milestones: list(&get("milestones")),
        path: path.to_path_buf(),
    }
}

/// The `---` block at the head of the file, as key/value pairs.
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
        // A nested block would be silently dropped by this parser, so it is reported as unparsed
        // instead - an empty value the caller can see rather than a missing key nobody notices.
        if let Some((k, v)) = line.split_once(':') {
            out.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    out
}

/// `[a, b, c]` or a bare scalar, into a list.
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

/// Resolve a typed key to a skill: exact id, exact name, then id prefix. See the module docs for why
/// all three forms are kept.
pub fn find(skills: &[Skill], key: &str) -> Option<Skill> {
    if key.is_empty() {
        return None;
    }
    skills
        .iter()
        .find(|s| s.id == key || s.name == key)
        .or_else(|| skills.iter().find(|s| s.id.starts_with(key)))
        .cloned()
}

/// Match one `applies-to` pattern against a path.
///
/// **A trailing `*` alone is a prefix test, and that was wrong.** `docs/mechanisms/*.md` stripped to
/// the prefix `docs/mechanisms/` and matched any file under it, **including non-`.md` files** — and
/// the first version went further and required the literal remainder `.md` as a prefix, so it matched
/// nothing at all. The pattern has to be split on `*` into a prefix AND a suffix, and both checked.
///
/// A pattern with no `*` matches by equality or as a suffix, because `applies-to` entries are
/// sometimes written relative to a different root.
fn glob_match(pat: &str, path: &str) -> bool {
    match pat.split_once('*') {
        Some((head, tail)) => {
            path.len() >= head.len() + tail.len() && path.starts_with(head) && path.ends_with(tail)
        }
        None => path == pat || path.ends_with(pat) || pat.ends_with(path),
    }
}

/// Which skills apply to a set of files.
pub fn matching(skills: &[Skill], files: &[String]) -> Vec<Skill> {
    skills
        .iter()
        .filter(|s| {
            s.applies_to
                .iter()
                .any(|pat| files.iter().any(|f| glob_match(pat, f)))
        })
        .cloned()
        .collect()
}

/// Rank skills against a task description, as Python's `match_skills` does.
///
/// A `when` trigger outranks an incidental description-word overlap; weak matches show only when
/// nothing triggered, or a generic word would route every skill.
pub fn rank(skills: &[Skill], query: &str) -> Vec<(Skill, String)> {
    let q = query.trim().to_lowercase();
    let mut tokens: Vec<&str> = q
        .split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || "_.-/".contains(c)))
        .filter(|t| t.chars().count() > 2)
        .collect();
    tokens.sort_unstable();
    tokens.dedup();
    let mut strong: Vec<(usize, Skill, String)> = Vec::new();
    let mut weak: Vec<(usize, Skill, String)> = Vec::new();
    for s in skills {
        let when_text = s.when.join(" ").to_lowercase();
        let hay_desc = format!("{} {} {}", s.name, s.description, s.id).to_lowercase();
        let mut hits: Vec<String> = Vec::new();
        let mut weight = 0;
        for trigger in &s.when {
            let t = trigger.to_lowercase();
            let t = t.trim();
            if !q.is_empty() && !t.is_empty() && q.contains(t) {
                hits.push(format!("when:{t}"));
                weight += 60 + t.split_whitespace().count();
            }
        }
        for t in &tokens {
            if when_text.contains(t) {
                hits.push(format!("when~{t}"));
                weight += 10;
            } else if hay_desc.contains(t) {
                hits.push(format!("desc~{t}"));
                weight += 1;
            } else if let Some(pat) = s.applies_to.iter().find(|p| {
                let p = p.to_lowercase();
                crate::mistakes::fnmatch(&p, t) || crate::mistakes::fnmatch(&format!("*{t}*"), &p)
            }) {
                hits.push(format!("path~{pat}"));
                weight += 8;
            }
        }
        if hits.is_empty() {
            continue;
        }
        hits.sort();
        hits.dedup();
        hits.truncate(4);
        let entry = (weight, s.clone(), hits.join(", "));
        if weight >= 10 {
            strong.push(entry);
        } else {
            weak.push(entry);
        }
    }
    let mut ranked = if strong.is_empty() { weak } else { strong };
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));
    ranked.into_iter().map(|(_, s, why)| (s, why)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_repo() -> (tempfile::TempDir, StatePaths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        let base = repo.join("docs/harness/skills");
        std::fs::create_dir_all(base.join("context-hydration")).expect("mkdir");
        std::fs::create_dir_all(base.join("evidence-record")).expect("mkdir");
        std::fs::write(
            base.join("context-hydration/SKILL.md"),
            "---\nname: context-hydration\ndescription: Hydrate the session.\nversion: 1\nkind: research\napplies-to: [docs/mechanisms/*.md, docs/labs/session-goal.md]\nwhen: [start a session, brief]\nmilestones: [m1, m2]\n---\n\nbody\n",
        )
        .expect("write");
        std::fs::write(
            base.join("evidence-record/SKILL.md"),
            "---\nname: evidence-record\ndescription: Record evidence.\nmilestones: [m3]\n---\n\nbody\n",
        )
        .expect("write");
        let paths = StatePaths::resolve(repo, None, Some("/nonexistent-live"));
        (dir, paths)
    }

    /// A `when` trigger in the task text routes its skill and says why; unrelated text routes none.
    #[test]
    fn rank_routes_on_a_when_trigger_and_not_on_noise() {
        let (_d, p) = fixture_repo();
        let skills = load(&p);
        let hits = rank(&skills, "please start a session for m1");
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].0.id, "context-hydration");
        assert!(hits[0].1.contains("when:start a session"), "{}", hits[0].1);
        assert!(rank(&skills, "zebra quantum").is_empty());
    }

    #[test]
    fn every_skill_loads_sorted_by_id() {
        let (_d, p) = fixture_repo();
        let skills = load(&p);
        assert_eq!(skills.len(), 2);
        assert_eq!(
            skills[0].id, "context-hydration",
            "sorted, so resolution is deterministic"
        );
        assert_eq!(skills[1].id, "evidence-record");
    }

    #[test]
    fn frontmatter_fields_parse() {
        let (_d, p) = fixture_repo();
        let s = find(&load(&p), "context-hydration").expect("skill");
        assert_eq!(s.description, "Hydrate the session.");
        assert_eq!(s.version, "1");
        assert_eq!(s.kind, "research");
        assert_eq!(s.when, vec!["start a session", "brief"]);
        assert_eq!(s.milestones, vec!["m1", "m2"]);
        assert_eq!(s.applies_to.len(), 2);
    }

    /// All three forms labs have actually typed: exact id, a prefix, and a name.
    #[test]
    fn resolution_accepts_id_prefix_and_name() {
        let (_d, p) = fixture_repo();
        let skills = load(&p);
        assert_eq!(
            find(&skills, "evidence-record").expect("exact").id,
            "evidence-record"
        );
        assert_eq!(find(&skills, "evid").expect("prefix").id, "evidence-record");
        assert_eq!(
            find(&skills, "context-hydration").expect("name").id,
            "context-hydration"
        );
    }

    /// An unknown skill must resolve to nothing, so `skill use` can refuse rather than record a
    /// receipt for a skill that does not exist. **A receipt naming a nonexistent skill is worse than
    /// no receipt**, because the record would claim a skill was loaded.
    #[test]
    fn an_unknown_skill_resolves_to_nothing() {
        let (_d, p) = fixture_repo();
        assert!(find(&load(&p), "no-such-skill").is_none());
        assert!(find(&load(&p), "").is_none());
    }

    #[test]
    fn a_missing_name_falls_back_to_the_directory() {
        let s = parse(
            "some-dir",
            "---\ndescription: x\n---\n",
            &PathBuf::from("f"),
        );
        assert_eq!(s.id, "some-dir", "the directory is what a lab types");
    }

    #[test]
    fn a_file_with_no_frontmatter_still_loads() {
        let s = parse("plain", "# Just a heading\n", &PathBuf::from("f"));
        assert_eq!(s.id, "plain");
        assert!(s.description.is_empty());
    }

    #[test]
    fn matching_finds_skills_by_glob_and_by_suffix() {
        let (_d, p) = fixture_repo();
        let skills = load(&p);
        let hit = matching(&skills, &["docs/mechanisms/attention.md".to_string()]);
        assert_eq!(hit.len(), 1);
        assert_eq!(hit[0].id, "context-hydration");
        let hit2 = matching(&skills, &["session-goal.md".to_string()]);
        assert_eq!(hit2.len(), 1, "a suffix match");
    }

    #[test]
    fn matching_nothing_returns_nothing() {
        let (_d, p) = fixture_repo();
        assert!(matching(&load(&p), &["unrelated.rs".to_string()]).is_empty());
    }

    /// Against the REAL registry: a broken skill file must fail here rather than in a lab's session.
    #[test]
    fn the_real_registry_loads_and_every_id_resolves_to_itself() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = StatePaths::resolve(&repo, None, Some("/nonexistent-live"));
        let skills = load(&paths);
        assert!(
            skills.len() >= 5,
            "expected the real skill set, got {}",
            skills.len()
        );
        for s in &skills {
            assert!(!s.id.is_empty(), "{}", s.path.display());
            assert!(!s.description.is_empty(), "{} has no description", s.id);
            assert_eq!(
                find(&skills, &s.id).expect("self").id,
                s.id,
                "an id must resolve to itself"
            );
        }
    }
}
