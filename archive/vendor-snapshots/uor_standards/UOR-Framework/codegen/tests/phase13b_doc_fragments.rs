//! Phase 13b verification: `emit::load_doc_fragment` resolves keys
//! against a Markdown phase-doc, panics on missing files / keys, and
//! honours the heading + explicit-terminator close rules.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use uor_codegen::emit::load_doc_fragment;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn loads_known_fragment() {
    let body = load_doc_fragment(
        &workspace_root(),
        "docs/orphan-closure/phase-13b-doc-fragments.md",
        "phase-13b:hello",
    );
    assert!(
        body.contains("hello world"),
        "expected `hello world` in fragment body; got: {body}"
    );
    // Heading terminator works — the fragment ends before the next
    // `## …` heading.
    assert!(
        !body.contains("## Migration"),
        "fragment leaked into following section heading; got: {body}"
    );
}

#[test]
fn explicit_terminator_works() {
    let body = load_doc_fragment(
        &workspace_root(),
        "docs/orphan-closure/phase-13b-doc-fragments.md",
        "phase-13b:multiline",
    );
    // Explicit `<!-- /doc-key -->` terminator caps the fragment.
    assert!(
        body.contains("end-of-multiline"),
        "expected `end-of-multiline` in fragment; got: {body}"
    );
    // Fenced code block is preserved verbatim.
    assert!(
        body.contains("fenced code preserved verbatim"),
        "fenced code block not preserved; got: {body}"
    );
}

#[test]
#[should_panic(expected = "load_doc_fragment: missing key 'definitely-not-present:abc'")]
fn missing_key_panics() {
    let _ = load_doc_fragment(
        &workspace_root(),
        "docs/orphan-closure/phase-13b-doc-fragments.md",
        "definitely-not-present:abc",
    );
}

#[test]
#[should_panic(expected = "load_doc_fragment: file not found")]
fn missing_file_panics() {
    let _ = load_doc_fragment(
        &workspace_root(),
        "docs/orphan-closure/this-file-does-not-exist.md",
        "any-key",
    );
}
