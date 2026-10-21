//! `uor-harness loop suborder` and `loop ask` (loop PR 7a, ORDER calls 6 and 7).
//!
//! **`loop suborder --parent <ORDER> --scope <glob>[,<glob>...]`** gives a subagent a narrower
//! order. Every child glob must be inside the parent's scope, which is the `scope:` of the item the
//! parent ORDER names (the same source `loop claim` reads). A child glob is inside when some parent
//! glob contains it (see [`glob_contains`]). On success it writes
//! `local/<lab>/<topic>/orders/<n>.md`, `<topic>` being the parent ORDER's folder name and `<n>` one
//! more than the highest number there, and prints the subagent's block.
//!
//! **`loop ask --scope|--blocked --order <ORDER> ...`** writes a stop-and-ask card to
//! `local/<lab>/<topic>/asks/<n>.md`. It posts nothing. A missing field is refused with exit 2.

use std::path::{Path, PathBuf};

/// The text of a glob before its first `*`, `?` or `[`, and whether a wildcard follows it.
fn literal(glob: &str) -> (&str, bool) {
    match glob.find(['*', '?', '[']) {
        Some(i) => (&glob[..i], true),
        None => (glob, false),
    }
}

/// `parent` contains `child` only when both pass [`scope_glob_ok`] and either they are equal, or the
/// parent is `<dir>/**` (its only wildcard that trailing `/**`) and the child's literal part starts
/// with `<dir>/`. Any other parent wildcard contains only an identical child, so `docs/*.md` does
/// not contain `docs/secret.rs` and `crates/a*` does not contain `crates/ab/deep/x.rs`.
///
/// [`scope_glob_ok`]: crate::loopclaim::scope_glob_ok
pub fn glob_contains(parent: &str, child: &str) -> bool {
    use crate::loopclaim::scope_glob_ok;
    if scope_glob_ok(parent).is_err() || scope_glob_ok(child).is_err() {
        return false;
    }
    if parent == child {
        return true;
    }
    match parent.strip_suffix("/**") {
        Some(dir) if !literal(dir).1 => literal(child).0.starts_with(&format!("{dir}/")),
        _ => false,
    }
}

/// One more than the highest `<n>.md` in `dir` (1 when there is none).
fn next_number(dir: &Path) -> u64 {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            e.file_name()
                .to_str()?
                .strip_suffix(".md")?
                .parse::<u64>()
                .ok()
        })
        .max()
        .unwrap_or(0)
        + 1
}

/// How many numbers `write_numbered` tries before it gives up.
const CARD_ATTEMPTS: u64 = 100;

/// Create `<dir>/<n>.md` from `start` upward with `create_new`, so two writers never share a file
/// and an existing card is never overwritten. Gives up after `CARD_ATTEMPTS` numbers.
fn write_numbered(dir: &Path, start: u64, text: &str) -> Result<PathBuf, String> {
    use std::io::Write;
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    for n in start..start + CARD_ATTEMPTS {
        let path = dir.join(format!("{n}.md"));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut f) => {
                f.write_all(text.as_bytes())
                    .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
                return Ok(path);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(format!("cannot write {}: {e}", path.display())),
        }
    }
    Err(format!(
        "no free card number in {} after {CARD_ATTEMPTS} tries",
        dir.display()
    ))
}

/// Write `text` as the next card under `local/<lab>/<topic>/<kind>/`; returns its path.
fn write_card(
    root: &Path,
    lab: &str,
    topic: &str,
    kind: &str,
    text: &str,
) -> Result<PathBuf, String> {
    let dir = root.join("local").join(lab).join(topic).join(kind);
    write_numbered(&dir, next_number(&dir), text)
}

/// The parent ORDER's folder name, its path as given, and its text.
fn read_order(root: &Path, order: &str) -> Result<(String, String), String> {
    let path = root.join(order);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read ORDER {}: {e}", path.display()))?;
    let topic = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("ORDER {order} has no folder"))?
        .to_string();
    Ok((topic, text))
}

/// Parse `--flag value` pairs and bare switches. `switches` take no value.
fn parse(
    args: &[String],
    values: &[&str],
    switches: &[&str],
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut out = std::collections::BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if switches.contains(&a) {
            out.insert(a.to_string(), String::new());
        } else if values.contains(&a) {
            let v = args
                .get(i + 1)
                .ok_or_else(|| format!("{a} needs a value"))?;
            out.insert(a.to_string(), v.clone());
            i += 1;
        } else {
            return Err(format!("unexpected argument {a:?}"));
        }
        i += 1;
    }
    Ok(out)
}

fn need<'a>(
    m: &'a std::collections::BTreeMap<String, String>,
    flag: &str,
) -> Result<&'a str, String> {
    m.get(flag)
        .map(String::as_str)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| format!("missing {flag}"))
}

const SUB_USAGE: &str =
    "usage: uor-harness loop suborder --parent <ORDER> --scope <glob>[,<glob>...]";

/// The subagent's stop rules, printed in its block and written to its order.
const STOP_RULES: &str = "Stop and ask on any file outside this scope. Never claim, open, review or merge a PR; never post as the lab or push; never decide (§5.2); report to the parent.";

fn suborder(root: &Path, lab: &str, args: &[String]) -> Result<String, String> {
    let m = parse(args, &["--parent", "--scope"], &[])?;
    let parent = need(&m, "--parent")?;
    let globs: Vec<&str> = need(&m, "--scope")?
        .split(',')
        .map(str::trim)
        .filter(|g| !g.is_empty())
        .collect();
    let (topic, text) = read_order(root, parent)?;
    let item = crate::loopclaim::order_item(&text)
        .ok_or_else(|| format!("ORDER {parent} names no item on its Item: line"))?;
    let theirs = crate::loopclaim::read_item(root, &item)?
        .scope
        .ok_or_else(|| format!("item {item} has no scope: line"))?;
    for g in &theirs {
        crate::loopclaim::scope_glob_ok(g).map_err(|e| format!("parent item {item}: {e}"))?;
    }
    for g in &globs {
        crate::loopclaim::scope_glob_ok(g)?;
    }
    if let Some(out) = globs
        .iter()
        .find(|g| !theirs.iter().any(|p| glob_contains(p, g)))
    {
        return Err(format!(
            "scope {out} is outside the parent's scope [{}]",
            theirs.join(", ")
        ));
    }
    let slug = crate::state::slug(lab);
    let mut block = format!(
        "# Suborder of {parent}\n\nLab: {slug}. Parent ORDER: `{parent}`. Item: {item}.\n\n## Scope\n\n"
    );
    for g in &globs {
        block += &format!("- `{g}`\n");
    }
    block += &format!("\n## Stop rules\n\n{STOP_RULES}\n");
    let path = write_card(root, &slug, &topic, "orders", &block)?;
    Ok(format!("suborder: {}\n\n{block}", path.display()))
}

const ASK_USAGE: &str = "usage: uor-harness loop ask --scope --order <ORDER> --file <path> --reason <text> --lines <n> --own-code <yes|no>\n       uor-harness loop ask --blocked --order <ORDER> --limit <text> --measured <text> --command <text> --state <refused|not-reached>";

fn ask(root: &Path, lab: &str, args: &[String]) -> Result<String, String> {
    let values = [
        "--order",
        "--file",
        "--reason",
        "--lines",
        "--own-code",
        "--limit",
        "--measured",
        "--command",
        "--state",
    ];
    let m = parse(args, &values, &["--scope", "--blocked"])?;
    let (scope, blocked) = (m.contains_key("--scope"), m.contains_key("--blocked"));
    if scope == blocked {
        return Err("pass exactly one of --scope or --blocked".into());
    }
    let order = need(&m, "--order")?;
    let mut card = format!(
        "# Stop and ask: {}\n\nORDER: `{order}`.\n\n",
        if scope { "scope" } else { "blocked" }
    );
    if scope {
        let file = need(&m, "--file")?;
        let reason = need(&m, "--reason")?;
        let lines = need(&m, "--lines")?;
        lines
            .parse::<u64>()
            .map_err(|_| format!("--lines {lines:?} is not a whole number"))?;
        let own = need(&m, "--own-code")?;
        if own != "yes" && own != "no" {
            return Err(format!("--own-code {own:?} is not yes or no"));
        }
        card += &format!("- File: `{file}`\n- Reason: {reason}\n- Lines, roughly: {lines}\n- Could the lab's own code change instead: {own}\n");
    } else {
        let limit = need(&m, "--limit")?;
        let measured = need(&m, "--measured")?;
        let command = need(&m, "--command")?;
        let state = need(&m, "--state")?;
        if state != "refused" && state != "not-reached" {
            return Err(format!("--state {state:?} is not refused or not-reached"));
        }
        card += &format!("- Limit: {limit}\n- Measured by: {measured}\n- Checked by: `{command}`\n- State: {state}\n");
    }
    card += "\nThe order pauses here; the supervisor moves on. Nothing was posted.\n";
    let (topic, _) = read_order(root, order)?;
    let path = write_card(root, &crate::state::slug(lab), &topic, "asks", &card)?;
    Ok(format!("ask: {}\n", path.display()))
}

/// One action: what it prints on success, or why it was refused.
type Action = fn(&Path, &str, &[String]) -> Result<String, String>;

/// `loop suborder ...` or `loop ask ...`, with `args` starting after the action. Exit code 0 or 2.
pub fn cli(root: &Path, lab: &str, action: &str, args: &[String]) -> u8 {
    let (run, usage): (Action, &str) = match action {
        "suborder" => (suborder, SUB_USAGE),
        _ => (ask, ASK_USAGE),
    };
    if lab.trim().is_empty() {
        eprintln!("uor-harness: loop {action} needs a lab: set UOR_LAB or pass --lab");
        return 2;
    }
    match run(root, lab, args) {
        Ok(out) => {
            print!("{out}");
            0
        }
        Err(e) => {
            eprintln!("uor-harness: loop {action}: refused: {e}\n{usage}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{glob_contains, write_numbered};

    #[test]
    fn a_card_is_created_new_and_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path().join("cards");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("1.md"), "one").unwrap();
        std::fs::write(d.join("2.md"), "two").unwrap();
        let p = write_numbered(&d, 1, "new").unwrap();
        assert_eq!(p, d.join("3.md"));
        assert_eq!(std::fs::read_to_string(d.join("1.md")).unwrap(), "one");
        assert_eq!(std::fs::read_to_string(d.join("2.md")).unwrap(), "two");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "new");
    }

    #[test]
    fn a_card_gives_up_after_100_taken_numbers() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path().join("cards");
        std::fs::create_dir_all(&d).unwrap();
        for n in 1..=100 {
            std::fs::write(d.join(format!("{n}.md")), "x").unwrap();
        }
        assert!(write_numbered(&d, 1, "new").is_err());
        assert!(!d.join("101.md").exists());
    }

    #[test]
    fn containment_is_by_literal_path_prefix_under_a_wildcard() {
        assert!(glob_contains("crates/a/**", "crates/a/src/x.rs"));
        assert!(glob_contains("crates/a/**", "crates/a/src/*.rs"));
        assert!(glob_contains("crates/a/**", "crates/a/**"));
        assert!(glob_contains("crates/a/**", "crates/a/x/**"));
        assert!(glob_contains("crates/a/**", "crates/a/y.rs"));
        assert!(glob_contains("docs/x.md", "docs/x.md"));
        assert!(glob_contains("docs/*.md", "docs/*.md"));
        assert!(!glob_contains("crates/a/**", "crates/ab/x.rs"));
        assert!(!glob_contains("crates/a/src/*.rs", "crates/a/**"));
        assert!(!glob_contains("docs/x.md", "docs/x.md.bak"));
        assert!(!glob_contains("docs/x", "docs/x/y.md"));
    }

    #[test]
    fn containment_cannot_be_escaped() {
        assert!(!glob_contains("crates/a/**", "crates/a/../b/x.rs"));
        assert!(!glob_contains("docs/*.md", "docs/secret.rs"));
        assert!(!glob_contains("crates/a*", "crates/ab/deep/x.rs"));
        assert!(!glob_contains("crates/a-*", "crates/a-b/x.rs"));
        assert!(!glob_contains("crates/a/**", "crates/a/./x.rs"));
        assert!(!glob_contains("crates/*/**", "crates/b/x.rs"));
    }
}
