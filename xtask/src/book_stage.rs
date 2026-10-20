//! Stage the mdBook source (`cargo xtask book-stage [--out DIR]`).
//!
//! One source, two outputs (owner decision, 19 October 2026): the markdown
//! GitHub renders is also the source of the documentation site. This task
//! copies `docs/SUMMARY.md`, every page it lists and `docs/figures/**` into
//! DIR (default `target/book-src`, the `src` of `book.toml`), at their paths
//! relative to `docs/`. A SUMMARY entry outside `docs/` (`../CHARTER.md`,
//! `../projects/x/README.md`) is staged at its repository path.
//!
//! The staged copies differ from the sources in two ways only:
//!
//! - **Links.** A relative link to a page or figure in the book stays
//!   relative (re-based if the page moved); any other relative link goes to
//!   the file on GitHub (`blob/main`, `tree/main` for a directory, `raw/main`
//!   for an image), keeping its `#anchor`.
//! - **Maths.** GitHub reads `$…$` and `$$…$$`; mdBook's MathJax reads
//!   `\(…\)` and `\[…\]`, so the delimiters are converted and the maths
//!   content is backslash-escaped so markdown passes it through unchanged.
//!   A single `$` is maths only as a delimiter pair on one line.
//!
//! Fenced code blocks and inline code spans are never changed.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::docs_links::{fence_marker, normalize};

const REPO_URL: &str = "https://github.com/UOR-Foundation/uor-r4";
const FIGURES: &str = "docs/figures/";
const IMAGE_EXTS: [&str; 6] = [".svg", ".png", ".jpg", ".jpeg", ".gif", ".webp"];

/// The book: each repository path it contains and where that path is staged.
#[derive(Debug)]
pub struct Book {
    pages: BTreeMap<String, String>,
}

impl Book {
    /// The book SUMMARY describes (`summary` is `docs/SUMMARY.md`'s text).
    pub fn from_summary(summary: &str) -> Result<Book, String> {
        let mut pages = BTreeMap::new();
        pages.insert("docs/SUMMARY.md".to_string(), "SUMMARY.md".to_string());
        for target in summary_targets(summary) {
            let repo = normalize(&format!("docs/{target}"))
                .map_err(|p| format!("SUMMARY entry {target} climbs above the repository: {p}"))?;
            let staged = repo.strip_prefix("docs/").unwrap_or(&repo).to_string();
            if let Some((other, _)) = pages.iter().find(|(r, s)| **s == staged && **r != repo) {
                return Err(format!(
                    "SUMMARY entries {other} and {repo} would both stage as {staged}"
                ));
            }
            pages.insert(repo, staged);
        }
        Ok(Book { pages })
    }

    /// Book pages for which `exists` says the repository file is absent.
    pub fn missing_pages(&self, exists: &dyn Fn(&str) -> bool) -> Vec<String> {
        self.pages.keys().filter(|r| !exists(r)).cloned().collect()
    }

    /// Where a repository path is staged, if it is part of the book.
    fn staged(&self, repo: &str) -> Option<String> {
        if let Some(s) = self.pages.get(repo) {
            return Some(s.clone());
        }
        repo.strip_prefix(FIGURES)
            .map(|rest| format!("figures/{rest}"))
    }
}

/// The link targets of SUMMARY's entries (relative to `docs/`), in order.
/// Draft chapters (empty targets) are skipped.
pub fn summary_targets(summary: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in summary.lines() {
        let mut rest = line;
        while let Some(i) = rest.find("](") {
            let after = &rest[i + 2..];
            let end = after.find(')').unwrap_or(after.len());
            let dest = after[..end].trim();
            if !dest.is_empty() {
                out.push(dest.split('#').next().unwrap_or(dest).to_string());
            }
            rest = &after[end..];
        }
    }
    out
}

/// A relative link from staged file `from` to staged path `to`.
fn relative(from: &str, to: &str) -> String {
    let mut from_dir: Vec<&str> = from.split('/').collect();
    from_dir.pop();
    let to_parts: Vec<&str> = to.split('/').collect();
    let common = from_dir
        .iter()
        .zip(&to_parts)
        .take_while(|(a, b)| a == b)
        .count()
        .min(to_parts.len() - 1);
    format!(
        "{}{}",
        "../".repeat(from_dir.len() - common),
        to_parts[common..].join("/")
    )
}

/// mdBook publishes every `README.md` chapter as `index.html`, and its index
/// preprocessor rewrites only SUMMARY entries, so body links name `index.md`.
fn index_page(rel: &str) -> String {
    match rel.strip_suffix("README.md") {
        Some(dir) if dir.is_empty() || dir.ends_with('/') => format!("{dir}index.md"),
        _ => rel.to_string(),
    }
}

/// The staged form of link destination `dest` in repository file `src`, or
/// `None` to leave it as written (external, same-page anchor, or unresolvable).
fn rewrite_dest(
    dest: &str,
    src: &str,
    book: &Book,
    is_dir: &dyn Fn(&str) -> bool,
) -> Option<String> {
    let lower = dest.to_ascii_lowercase();
    if dest.is_empty()
        || dest.starts_with('#')
        || dest.contains("://")
        || ["mailto:", "data:", "tel:"]
            .iter()
            .any(|s| lower.starts_with(s))
    {
        return None;
    }
    let (path, anchor) = match dest.find('#') {
        Some(i) => (&dest[..i], &dest[i..]),
        None => (dest, ""),
    };
    let joined = match path.strip_prefix('/') {
        Some(abs) => abs.to_string(),
        None => match src.rfind('/') {
            Some(i) => format!("{}/{path}", &src[..i]),
            None => path.to_string(),
        },
    };
    let repo = normalize(&joined).ok()?;
    let from = book.staged(src)?;
    let dir_link = path.ends_with('/') || path.is_empty() || is_dir(&repo);
    if !dir_link {
        if let Some(to) = book.staged(&repo) {
            return Some(format!("{}{anchor}", index_page(&relative(&from, &to))));
        }
        let kind = if IMAGE_EXTS
            .iter()
            .any(|e| lower.split('#').next().unwrap_or("").ends_with(e))
        {
            "raw"
        } else {
            "blob"
        };
        return Some(format!("{REPO_URL}/{kind}/main/{repo}{anchor}"));
    }
    let readme = if repo.is_empty() {
        "README.md".to_string()
    } else {
        format!("{repo}/README.md")
    };
    if let Some(to) = book.staged(&readme) {
        return Some(format!("{}{anchor}", index_page(&relative(&from, &to))));
    }
    Some(format!("{REPO_URL}/tree/main/{repo}{anchor}"))
}

/// Byte ranges of the inline code spans in `line`.
fn code_spans(line: &str) -> Vec<(usize, usize)> {
    let b = line.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'`' {
            i += 1;
            continue;
        }
        let n = b[i..].iter().take_while(|c| **c == b'`').count();
        let mut j = i + n;
        let mut close = None;
        while j < b.len() {
            if b[j] == b'`' {
                let m = b[j..].iter().take_while(|c| **c == b'`').count();
                if m == n {
                    close = Some(j + m);
                    break;
                }
                j += m;
            } else {
                j += 1;
            }
        }
        match close {
            Some(end) => {
                spans.push((i, end));
                i = end;
            }
            None => i += n,
        }
    }
    spans
}

/// End of an inline link destination starting at `s[0..]` (after `](`).
fn dest_range(s: &str) -> Option<(usize, usize)> {
    let lead = s.len() - s.trim_start_matches([' ', '\t']).len();
    let t = &s[lead..];
    if let Some(rest) = t.strip_prefix('<') {
        let end = rest.find('>')?;
        return Some((lead + 1, lead + 1 + end));
    }
    let mut depth = 0usize;
    let mut end = t.len();
    let mut chars = t.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '(' => depth += 1,
            ')' if depth == 0 => {
                end = i;
                break;
            }
            ')' => depth -= 1,
            c if c.is_whitespace() => {
                end = i;
                break;
            }
            _ => {}
        }
    }
    (end > 0).then_some((lead, lead + end))
}

/// Rewrite the link destinations in one non-code segment of a line.
/// `line_start` says whether the segment begins the line (for reference
/// definitions).
fn rewrite_links(
    seg: &str,
    line_start: bool,
    src: &str,
    book: &Book,
    is_dir: &dyn Fn(&str) -> bool,
) -> String {
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    // Reference definitions: `[id]: dest`.
    if line_start {
        let t = seg.trim_start();
        let lead = seg.len() - t.len();
        if lead <= 3 && t.starts_with('[') && !t.starts_with("[^") {
            if let Some(close) = t.find("]:") {
                let start = lead + close + 2;
                if let Some((a, b)) = dest_range(&seg[start..]) {
                    ranges.push((start + a, start + b));
                }
            }
        }
    }
    // Inline links and images: `](dest)`.
    let mut from = 0;
    while let Some(i) = seg[from..].find("](") {
        let start = from + i + 2;
        if let Some((a, b)) = dest_range(&seg[start..]) {
            ranges.push((start + a, start + b));
            from = start + b;
        } else {
            from = start;
        }
    }
    // HTML attributes: `src="dest"`, `href="dest"`.
    for attr in ["src=\"", "href=\""] {
        let mut from = 0;
        while let Some(i) = seg[from..].find(attr) {
            let start = from + i + attr.len();
            match seg[start..].find('"') {
                Some(len) => {
                    ranges.push((start, start + len));
                    from = start + len;
                }
                None => break,
            }
        }
    }
    ranges.sort_unstable();
    let mut out = String::with_capacity(seg.len());
    let mut last = 0;
    for (a, b) in ranges {
        if a < last {
            continue;
        }
        if let Some(new) = rewrite_dest(&seg[a..b], src, book, is_dir) {
            out.push_str(&seg[last..a]);
            out.push_str(&new);
            last = b;
        }
    }
    out.push_str(&seg[last..]);
    out
}

/// Maths content with every ASCII punctuation character backslash-escaped,
/// so markdown hands it to MathJax exactly as written.
fn escape_math(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        if c.is_ascii_punctuation() {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// The closing `$` of an inline maths span opened at `b[i]`, if any. The
/// opener must not be followed by whitespace; the first later `$` closes only
/// if it is not preceded by whitespace and not followed by a digit.
fn closing_dollar(b: &[u8], i: usize) -> Option<usize> {
    let first = *b.get(i + 1)?;
    if first.is_ascii_whitespace() || first == b'$' {
        return None;
    }
    let mut j = i + 1;
    while j < b.len() {
        match b[j] {
            b'\\' => j += 2,
            b'$' => {
                let ok = !b[j - 1].is_ascii_whitespace()
                    && b.get(j + 1).map_or(true, |c| !c.is_ascii_digit());
                return ok.then_some(j);
            }
            _ => j += 1,
        }
    }
    None
}

/// Convert `$$…$$` and `$…$` within one non-code segment of a line.
fn convert_inline_math(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let (mut i, mut last) = (0, 0);
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'$' if b.get(i + 1) == Some(&b'$') => match s[i + 2..].find("$$") {
                Some(len) if len > 0 => {
                    out.push_str(&s[last..i]);
                    out.push_str("\\\\[");
                    out.push_str(&escape_math(&s[i + 2..i + 2 + len]));
                    out.push_str("\\\\]");
                    i += len + 4;
                    last = i;
                }
                _ => i += 2,
            },
            b'$' => match closing_dollar(b, i) {
                Some(j) => {
                    out.push_str(&s[last..i]);
                    out.push_str("\\\\(");
                    out.push_str(&escape_math(&s[i + 1..j]));
                    out.push_str("\\\\)");
                    i = j + 1;
                    last = i;
                }
                None => i += 1,
            },
            _ => i += 1,
        }
    }
    out.push_str(&s[last.min(s.len())..]);
    out
}

/// Links and inline maths for one line outside fences and maths blocks.
fn stage_line(line: &str, src: &str, book: &Book, is_dir: &dyn Fn(&str) -> bool) -> String {
    let mut out = String::with_capacity(line.len());
    let mut last = 0;
    let text = |seg: &str, start: bool, out: &mut String| {
        out.push_str(&convert_inline_math(&rewrite_links(
            seg, start, src, book, is_dir,
        )));
    };
    for (a, b) in code_spans(line) {
        text(&line[last..a], last == 0, &mut out);
        out.push_str(&line[a..b]);
        last = b;
    }
    text(&line[last..], last == 0, &mut out);
    out
}

/// The staged copy of markdown `text` from repository file `src`.
pub fn stage_markdown(text: &str, src: &str, book: &Book, is_dir: &dyn Fn(&str) -> bool) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / 8);
    let mut fence: Option<(u8, usize)> = None;
    let mut math_block = false;
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        let nl = &line[body.len()..];
        if let Some((ch, len)) = fence {
            if let Some((c, l, only_ws)) = fence_marker(body) {
                if c == ch && l >= len && only_ws {
                    fence = None;
                }
            }
            out.push_str(line);
            continue;
        }
        if math_block {
            let t = body.trim_start();
            match t.find("$$") {
                Some(i) => {
                    out.push_str(&escape_math(&t[..i]));
                    out.push_str("\\\\]");
                    out.push_str(&t[i + 2..]);
                    math_block = false;
                }
                None => out.push_str(&escape_math(t)),
            }
            out.push_str(nl);
            continue;
        }
        if let Some((c, l, _)) = fence_marker(body) {
            fence = Some((c, l));
            out.push_str(line);
            continue;
        }
        let t = body.trim_start();
        if let Some(rest) = t.strip_prefix("$$") {
            if !rest.contains("$$") {
                out.push_str(&body[..body.len() - t.len()]);
                out.push_str("\\\\[");
                out.push_str(&escape_math(rest));
                out.push_str(nl);
                math_block = true;
                continue;
            }
        }
        out.push_str(&stage_line(body, src, book, is_dir));
        out.push_str(nl);
    }
    out
}

fn copy_tree(from: &Path, to: &Path, count: &mut usize) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("create {}: {e}", to.display()))?;
    let entries = fs::read_dir(from).map_err(|e| format!("read {}: {e}", from.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read {}: {e}", from.display()))?;
        let path = entry.path();
        let dest = to.join(entry.file_name());
        if path.is_dir() {
            copy_tree(&path, &dest, count)?;
        } else {
            fs::copy(&path, &dest).map_err(|e| format!("copy {}: {e}", path.display()))?;
            *count += 1;
        }
    }
    Ok(())
}

/// Written into a staged output so a later run knows the directory is its own.
const MARKER: &str = ".book-stage";

/// Refuses to clear `out` unless it is plainly a staging directory: never the
/// repository root or an ancestor of it, and otherwise only when it is absent,
/// empty, carries the marker of an earlier run, or lives under `target/` or `local/`.
fn check_clearable(root: &Path, out: &Path) -> Result<(), String> {
    let norm = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let (root, out_n) = (norm(root), norm(out));
    if root.starts_with(&out_n) {
        return Err(format!(
            "book-stage: refusing --out {}: it is the repository root or contains it",
            out.display()
        ));
    }
    if !out.exists() || out.join(MARKER).is_file() {
        return Ok(());
    }
    let empty = fs::read_dir(out)
        .map(|mut d| d.next().is_none())
        .unwrap_or(false);
    let scratch = ["target", "local"]
        .iter()
        .any(|d| out_n.starts_with(root.join(d)));
    if empty || scratch {
        return Ok(());
    }
    Err(format!(
        "book-stage: refusing to clear {}: it is not empty and has no {MARKER} marker from an earlier run",
        out.display()
    ))
}

pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let mut out = PathBuf::from("target/book-src");
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--out" => out = PathBuf::from(it.next().ok_or("--out needs a directory")?),
            other => return Err(format!("book-stage: unknown argument {other}")),
        }
    }
    let out = if out.is_absolute() {
        out
    } else {
        root.join(out)
    };
    let summary_path = root.join("docs/SUMMARY.md");
    let summary = fs::read_to_string(&summary_path)
        .map_err(|e| format!("read {}: {e}", summary_path.display()))?;
    let book = Book::from_summary(&summary)?;
    let missing = book.missing_pages(&|r| root.join(r).is_file());
    if !missing.is_empty() {
        let list: Vec<String> = missing.iter().map(|m| format!("  {m}")).collect();
        return Err(format!(
            "book-stage: {} SUMMARY entr{} missing:\n{}",
            missing.len(),
            if missing.len() == 1 {
                "y is"
            } else {
                "ies are"
            },
            list.join("\n")
        ));
    }
    check_clearable(root, &out)?;
    if out.exists() {
        fs::remove_dir_all(&out).map_err(|e| format!("clear {}: {e}", out.display()))?;
    }
    let is_dir = |p: &str| root.join(p).is_dir();
    for (repo, staged) in &book.pages {
        let text = fs::read_to_string(root.join(repo)).map_err(|e| format!("read {repo}: {e}"))?;
        let dest = out.join(staged);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        fs::write(&dest, stage_markdown(&text, repo, &book, &is_dir))
            .map_err(|e| format!("write {}: {e}", dest.display()))?;
    }
    let mut figures = 0;
    copy_tree(&root.join(FIGURES), &out.join("figures"), &mut figures)?;
    fs::write(out.join(MARKER), "staged by cargo xtask book-stage\n")
        .map_err(|e| format!("write marker: {e}"))?;
    println!(
        "book-stage: {} pages and {figures} figure files staged in {}",
        book.pages.len(),
        out.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUMMARY: &str = "# Summary\n\n[Docs](README.md)\n\n- [A](start/a.md)\n- [B](geometry/b.md)\n- [Charter](../CHARTER.md)\n- [P](../projects/p/README.md)\n- [Mech](mechanisms/README.md)\n";

    fn book() -> Book {
        Book::from_summary(SUMMARY).expect("summary")
    }

    fn dirs(p: &str) -> bool {
        ["crates/core", "docs/mechanisms", "archive/old"].contains(&p)
    }

    fn stage(text: &str, src: &str) -> String {
        stage_markdown(text, src, &book(), &dirs)
    }

    #[test]
    fn summary_maps_outside_entries_to_their_repo_paths() {
        let b = book();
        assert_eq!(b.staged("docs/start/a.md").as_deref(), Some("start/a.md"));
        assert_eq!(b.staged("CHARTER.md").as_deref(), Some("CHARTER.md"));
        assert_eq!(
            b.staged("projects/p/README.md").as_deref(),
            Some("projects/p/README.md")
        );
        assert_eq!(
            b.staged("docs/figures/x/y.svg").as_deref(),
            Some("figures/x/y.svg")
        );
        assert_eq!(b.staged("docs/labs/x.md"), None);
        assert_eq!(
            stage("[c](../CHARTER.md) [r](start/a.md)", "docs/SUMMARY.md"),
            "[c](CHARTER.md) [r](start/a.md)"
        );
    }

    #[test]
    fn link_to_code_goes_to_github_blob() {
        assert_eq!(
            stage("See [x](../../crates/core/src/lib.rs).", "docs/start/a.md"),
            "See [x](https://github.com/UOR-Foundation/uor-r4/blob/main/crates/core/src/lib.rs)."
        );
    }

    #[test]
    fn link_to_archive_goes_to_github_blob() {
        assert_eq!(
            stage("[old](archive/notes.md)", "CHARTER.md"),
            "[old](https://github.com/UOR-Foundation/uor-r4/blob/main/archive/notes.md)"
        );
        assert_eq!(
            stage("[old](../../archive/old/x.md)", "docs/start/a.md"),
            "[old](https://github.com/UOR-Foundation/uor-r4/blob/main/archive/old/x.md)"
        );
    }

    #[test]
    fn link_between_book_pages_stays_relative_and_is_rebased() {
        assert_eq!(
            stage("[b](../geometry/b.md)", "docs/start/a.md"),
            "[b](../geometry/b.md)"
        );
        // CHARTER.md sits at the repository root and links into docs/.
        assert_eq!(
            stage("[a](docs/start/a.md)", "CHARTER.md"),
            "[a](start/a.md)"
        );
        assert_eq!(
            stage("[c](../../CHARTER.md)", "docs/start/a.md"),
            "[c](../CHARTER.md)"
        );
        assert_eq!(
            stage("[a](../../docs/start/a.md)", "projects/p/README.md"),
            "[a](../../start/a.md)"
        );
    }

    #[test]
    fn directory_links_go_to_tree_or_the_book_readme() {
        assert_eq!(
            stage("[c](../../crates/core)", "docs/start/a.md"),
            "[c](https://github.com/UOR-Foundation/uor-r4/tree/main/crates/core)"
        );
        assert_eq!(
            stage("[c](../../crates/core/)", "docs/start/a.md"),
            "[c](https://github.com/UOR-Foundation/uor-r4/tree/main/crates/core)"
        );
        assert_eq!(
            stage("[m](../mechanisms/)", "docs/start/a.md"),
            "[m](../mechanisms/index.md)"
        );
    }

    #[test]
    fn direct_readme_links_point_at_index() {
        assert_eq!(
            stage("[p](../../projects/p/README.md#x)", "docs/start/a.md"),
            "[p](../projects/p/index.md#x)"
        );
    }

    #[test]
    fn staging_output_guard_refuses_the_repo_and_unmarked_content() {
        let base = std::env::temp_dir().join(format!("book-stage-guard-{}", std::process::id()));
        let root = base.join("repo");
        let docs = root.join("docs");
        fs::create_dir_all(&docs).expect("mkdir");
        fs::write(docs.join("x.md"), "x").expect("write");
        assert!(check_clearable(&root, &root).is_err());
        assert!(check_clearable(&root, &base).is_err());
        assert!(check_clearable(&root, &docs).is_err());
        fs::write(docs.join(MARKER), "").expect("marker");
        assert!(check_clearable(&root, &docs).is_ok());
        assert!(check_clearable(&root, &root.join("nope")).is_ok());
        fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn anchors_are_kept() {
        assert_eq!(
            stage("[d](../integration/DECISIONS.md#d37--float)", "docs/start/a.md"),
            "[d](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/integration/DECISIONS.md#d37--float)"
        );
        assert_eq!(
            stage("[b](../geometry/b.md#top)", "docs/start/a.md"),
            "[b](../geometry/b.md#top)"
        );
        assert_eq!(stage("[s](#local)", "docs/start/a.md"), "[s](#local)");
    }

    #[test]
    fn images_reference_definitions_and_html_attributes() {
        assert_eq!(
            stage("![f](../figures/x.svg)", "docs/start/a.md"),
            "![f](../figures/x.svg)"
        );
        assert_eq!(
            stage("![f](../labs/run/plot.png)", "docs/start/a.md"),
            "![f](https://github.com/UOR-Foundation/uor-r4/raw/main/docs/labs/run/plot.png)"
        );
        assert_eq!(
            stage("[ref]: ../labs/x.md", "docs/start/a.md"),
            "[ref]: https://github.com/UOR-Foundation/uor-r4/blob/main/docs/labs/x.md"
        );
        assert_eq!(
            stage("<img src=\"../figures/x.svg\"> <a href=\"../labs/x.md\">x</a>", "docs/start/a.md"),
            "<img src=\"../figures/x.svg\"> <a href=\"https://github.com/UOR-Foundation/uor-r4/blob/main/docs/labs/x.md\">x</a>"
        );
        assert_eq!(
            stage("[w](https://example.com/a.md)", "docs/start/a.md"),
            "[w](https://example.com/a.md)"
        );
    }

    #[test]
    fn code_fences_and_spans_are_untouched() {
        let text = "```md\n[x](../../crates/core/src/lib.rs) and $a$\n```\nafter `[y](../labs/x.md)` and `$b$`\n";
        assert_eq!(stage(text, "docs/start/a.md"), text);
        let tilde = "~~~~\n$$\nx\n~~~\n$$\n~~~~\n";
        assert_eq!(stage(tilde, "docs/start/a.md"), tilde);
    }

    #[test]
    fn inline_maths_is_converted_and_escaped() {
        assert_eq!(
            stage("so $x^2$ holds", "docs/geometry/b.md"),
            "so \\\\(x\\^2\\\\) holds"
        );
        assert_eq!(
            stage("$\\mathbb{Z}[\\phi]$ and $a_1*b_2$", "docs/geometry/b.md"),
            "\\\\(\\\\mathbb\\{Z\\}\\[\\\\phi\\]\\\\) and \\\\(a\\_1\\*b\\_2\\\\)"
        );
        assert_eq!(
            stage("inline $$E=mc^2$$ display", "docs/geometry/b.md"),
            "inline \\\\[E\\=mc\\^2\\\\] display"
        );
    }

    #[test]
    fn block_maths_is_converted() {
        let text = "Before\n\n$$\nh(q) = (a, b)\n$$\n\nAfter $$x$$\n";
        assert_eq!(
            stage(text, "docs/geometry/b.md"),
            "Before\n\n\\\\[\nh\\(q\\) \\= \\(a\\, b\\)\n\\\\]\n\nAfter \\\\[x\\\\]\n"
        );
        assert_eq!(
            stage("$$x+y$$\n", "docs/geometry/b.md"),
            "\\\\[x\\+y\\\\]\n"
        );
    }

    #[test]
    fn currency_is_untouched() {
        for s in [
            "It costs $5.",
            "between $5 and $10 per hour",
            "caps: $5–$8/h",
            "≤ $8/h and $4",
            "an escaped \\$x\\$ pair",
        ] {
            assert_eq!(stage(s, "docs/start/a.md"), s);
        }
        assert_eq!(
            stage("$5 then $x$", "docs/start/a.md"),
            "$5 then \\\\(x\\\\)"
        );
    }

    #[test]
    fn missing_pages_lists_absent_files_only() {
        let b = book();
        let present = |p: &str| p != "CHARTER.md" && p != "projects/p/README.md";
        assert_eq!(
            b.missing_pages(&present),
            vec!["CHARTER.md".to_string(), "projects/p/README.md".to_string()]
        );
        assert!(b.missing_pages(&|_| true).is_empty());
    }

    #[test]
    fn missing_or_escaping_summary_entries_are_errors() {
        assert!(Book::from_summary("- [x](../../outside.md)\n").is_err());
        assert_eq!(
            summary_targets("- [x]()\n- [y](a.md#top)\n"),
            vec!["a.md".to_string()]
        );
    }
}
