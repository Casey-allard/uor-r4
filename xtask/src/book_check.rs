//! `cargo xtask book-check [--dir book]`: every relative `href`/`src` in the
//! built mdBook site must resolve to a file on disk. mdBook itself does not
//! check links in the output, so a chapter link that names a page the build
//! never produced (such as `README.html` for an `index.html` chapter) would
//! otherwise only be found by a reader.

use std::fs;
use std::path::{Path, PathBuf};

fn html_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))? {
        let path = entry
            .map_err(|e| format!("read {}: {e}", dir.display()))?
            .path();
        if path.is_dir() {
            html_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "html") {
            out.push(path);
        }
    }
    Ok(())
}

/// Every `href="…"` and `src="…"` value in `html`.
fn attribute_values(html: &str) -> Vec<&str> {
    let mut found = Vec::new();
    for attr in ["href=\"", "src=\""] {
        let mut rest = html;
        while let Some(i) = rest.find(attr) {
            let after = &rest[i + attr.len()..];
            // `data-href="` and `xlink:href="` end in `href="`; only whole attributes count.
            let boundary = i == 0 || rest.as_bytes()[i - 1].is_ascii_whitespace();
            match after.find('"') {
                Some(j) => {
                    if boundary {
                        found.push(&after[..j]);
                    }
                    rest = &after[j..];
                }
                None => break,
            }
        }
    }
    found
}

fn is_checked(value: &str) -> bool {
    !(value.is_empty()
        || ["http:", "https:", "mailto:", "#", "/", "data:"]
            .iter()
            .any(|p| value.starts_with(p))
        || value.contains("${"))
}

/// `<file>: <href>` for each relative link in `file` whose target is missing.
pub fn broken_links(file: &Path, html: &str) -> Vec<String> {
    let dir = file.parent().unwrap_or(Path::new("."));
    attribute_values(html)
        .into_iter()
        .filter(|v| is_checked(v))
        .filter(|v| {
            let target = v.split(['#', '?']).next().unwrap_or("");
            !target.is_empty() && !dir.join(target).exists()
        })
        .map(|v| format!("{}: {v}", file.display()))
        .collect()
}

pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let mut dir = PathBuf::from("book");
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dir" => dir = PathBuf::from(it.next().ok_or("--dir needs a directory")?),
            other => return Err(format!("book-check: unknown argument {other}")),
        }
    }
    let dir = if dir.is_absolute() {
        dir
    } else {
        root.join(dir)
    };
    let mut files = Vec::new();
    html_files(&dir, &mut files)?;
    if files.is_empty() {
        return Err(format!(
            "book-check: no .html files under {}",
            dir.display()
        ));
    }
    let mut broken = Vec::new();
    for file in &files {
        let html = fs::read_to_string(file).map_err(|e| format!("read {}: {e}", file.display()))?;
        broken.extend(broken_links(file, &html));
    }
    if broken.is_empty() {
        println!("book-check: {} pages, 0 broken links", files.len());
        return Ok(());
    }
    for b in &broken {
        eprintln!("{b}");
    }
    Err(format!("book-check: {} broken links", broken.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("book-check-{name}-{}", std::process::id()));
        fs::create_dir_all(&d).expect("mkdir");
        d
    }

    #[test]
    fn existing_target_passes_and_missing_fails() {
        let d = scratch("a");
        fs::write(d.join("index.html"), "").expect("write");
        let page = d.join("page.html");
        assert!(broken_links(&page, r#"<a href="index.html#x">i</a>"#).is_empty());
        assert_eq!(
            broken_links(&page, r#"<a href="README.html">r</a> <img src="gone.png">"#).len(),
            2
        );
        fs::remove_dir_all(&d).expect("cleanup");
    }

    #[test]
    fn anchors_external_absolute_and_templates_are_ignored() {
        let page = Path::new("/nonexistent/page.html");
        let html = r##"<a href="#top"></a><a href="https://x.y/z"></a><a href="mailto:a@b"></a>
            <a href="/abs"></a><img src="data:image/png;base64,AA"><a href="${base}/x"></a>"##;
        assert!(broken_links(page, html).is_empty());
    }

    #[test]
    fn run_fails_on_a_broken_link() {
        let d = scratch("b");
        fs::write(d.join("p.html"), r#"<a href="README.html"></a>"#).expect("write");
        assert!(run(&d, &["--dir".into(), ".".into()]).is_err());
        fs::write(d.join("p.html"), r#"<a href="p.html"></a>"#).expect("write");
        assert!(run(&d, &["--dir".into(), ".".into()]).is_ok());
        fs::remove_dir_all(&d).expect("cleanup");
    }
}
