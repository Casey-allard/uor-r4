//! Behaviour of the claim-wording gate on scratch trees and on this repository.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use uor_r4_claim_wording::{check, line_violates, report, Violation, PASS_LINE};

/// A uniquely named scratch tree under the system temp directory, removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "uor-r4-claim-wording-{}-{}-{}",
            name,
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        Scratch(dir)
    }

    fn write(&self, rel: &str, contents: &[u8]) {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        std::fs::write(p, contents).expect("write");
    }

    fn violations(&self) -> Vec<Violation> {
        check(&self.0).expect("scan")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn single(rel: &str, body: &str) -> Vec<Violation> {
    let s = Scratch::new("single");
    s.write(rel, body.as_bytes());
    s.violations()
}

#[test]
fn bare_machine_verified_fails() {
    let v = single("docs/a.md", "intro\n  This kernel is machine-verified.  \n");
    assert_eq!(
        v,
        vec![Violation {
            path: "docs/a.md".into(),
            line: 2,
            text: "This kernel is machine-verified.".into(),
        }]
    );
}

#[test]
fn allow_marker_passes() {
    assert!(single(
        "docs/a.md",
        "machine verified <!-- claim-wording: allow -->\n"
    )
    .is_empty());
}

#[test]
fn disavowal_words_pass() {
    for word in ["not", "without", "prohibited"] {
        let line = format!("This is {word} a machine-verified claim.\n");
        assert!(single("docs/a.md", &line).is_empty(), "{word}");
    }
}

#[test]
fn disavowal_is_whole_word() {
    // "nothing" and "knot" contain "no"/"not" but are not disavowals.
    assert!(line_violates("nothing here is machine-verified"));
    assert!(line_violates("a knot, machine-verified"));
}

#[test]
fn proof_link_passes() {
    assert!(single("docs/a.md", "machine-verified, see [x](proof/foo.md)\n").is_empty());
    assert!(single("docs/a.md", "machine-verified, see PROOF.md\n").is_empty());
}

#[test]
fn case_insensitive_variants_fail() {
    for phrase in [
        "Exactly-Equivalent",
        "provably equivalent",
        "exact teacher equivalence",
    ] {
        assert!(line_violates(&format!("The two are {phrase}.")), "{phrase}");
    }
}

#[test]
fn github_archive_is_excluded() {
    let s = Scratch::new("archive");
    s.write(
        "docs/history/github-archive-2026/x.md",
        b"machine-verified\n",
    );
    s.write("docs/history/other/x.md", b"machine-verified\n");
    let v = s.violations();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].path, "docs/history/other/x.md");
}

#[test]
fn non_utf8_file_is_skipped() {
    let s = Scratch::new("utf8");
    s.write("docs/bad.md", b"machine-verified \xff\xfe\n");
    s.write("docs/good.md", b"machine-verified\n");
    let v = s.violations();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].path, "docs/good.md");
}

#[test]
fn scan_set_and_order() {
    let s = Scratch::new("scope");
    s.write("README.md", b"machine-verified\n");
    s.write("crates/b/README.md", b"machine-verified\n");
    s.write("crates/a/README.md", b"machine-verified\n");
    s.write("crates/a/src/notes.md", b"machine-verified\n");
    s.write("docs/z.md", b"machine-verified\n");
    s.write("docs/a/x.md", b"machine-verified\n");
    s.write("docs/a-b/x.md", b"machine-verified\n");
    s.write("docs/a.txt", b"machine-verified\n");
    s.write("other/x.md", b"machine-verified\n");
    let paths: Vec<String> = s.violations().into_iter().map(|v| v.path).collect();
    // Component order, as Python sorts Path objects: "a" < "a-b" < "z.md".
    assert_eq!(
        paths,
        [
            "docs/a/x.md",
            "docs/a-b/x.md",
            "docs/z.md",
            "crates/a/README.md",
            "crates/b/README.md",
            "README.md",
        ]
    );
}

#[test]
fn report_format() {
    assert_eq!(report(&[]), format!("{PASS_LINE}\n"));
    let v = [Violation {
        path: "docs/a.md".into(),
        line: 3,
        text: "machine-verified".into(),
    }];
    assert_eq!(
        report(&v),
        "Claim-wording gate FAILED (docs/formal_vocabulary.md section 2.1):\n\
         Prohibited claim phrase without a linked proof artifact/certificate,\n\
         a same-line disavowal, or a 'claim-wording: allow' marker:\n\n  docs/a.md:3: machine-verified\n"
    );
}

#[test]
fn real_repo_passes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let v = check(&root).expect("scan repo");
    assert!(v.is_empty(), "{}", report(&v));
}
