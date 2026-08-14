//! #639-1: the pinned T5 SentencePiece Unigram source in
//! `models/t5-base-tokenizer.json`.
//!
//! Casey chose T5 (`google-t5/t5-base`, revision `a9723ea7…`) as the
//! `sentencepiece-unigram` source for the #639 tokenizer adapter. This
//! descriptor pins `spiece.model` by its blake3 κ so the later slices
//! (639-2 registry generalization, 639-3 the real Unigram adapter, 639-4
//! differential fixtures) build against a fixed, verifiable input rather
//! than whatever a fresh download happens to return.
//!
//! This slice is descriptor + pin only — there is no adapter yet, so the
//! test binds bytes to κ, not a `(family, version)` record. That binding
//! arrives with the adapter in 639-3.
//!
//! Two-state coverage (mirrors `gpt2_tokenizer_pin.rs`, #669):
//! - [`t5_tokenizer_pin_is_well_formed`] runs in CI: the pin is present
//!   and structurally valid, no snapshot required.
//! - [`real_t5_spiece_matches_the_pin`] is presence-gated (#599
//!   three-state) on the real `spiece.model`, a dev/local input that never
//!   downloads in CI: when present it must reproduce the pin byte-for-byte;
//!   when absent it reports UNAVAILABLE and passes vacuously — never a
//!   silent skip of a real failure.

use std::path::PathBuf;

/// Workspace root: `CARGO_MANIFEST_DIR` is `<root>/crates/uor-r4-core`.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn t5_descriptor() -> serde_json::Value {
    let path = repo_root().join("models").join("t5-base-tokenizer.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_slice(&bytes).expect("models/t5-base-tokenizer.json parses")
}

/// CI-run: the source pin is present and well-formed — a `blake3:<64 hex>`
/// address scoped to `spiece.model` with the pinned byte length, the
/// `sentencepiece-unigram` family, the pinned HF repo/revision, and the
/// Apache-2.0 license. Runs without the snapshot.
#[test]
fn t5_tokenizer_pin_is_well_formed() {
    let descriptor = t5_descriptor();

    assert_eq!(
        descriptor["repository"].as_str(),
        Some("google-t5/t5-base"),
        "the source repository is pinned"
    );
    assert_eq!(
        descriptor["revision"].as_str(),
        Some("a9723ea7f1b39c1eae772870f3b547bf6ef7e6c1"),
        "the source revision is pinned"
    );
    assert_eq!(
        descriptor["tokenizer_family"].as_str(),
        Some("sentencepiece-unigram"),
        "the tokenizer family is recorded for 639-3"
    );
    assert_eq!(
        descriptor["license"].as_str(),
        Some("Apache-2.0"),
        "the license permitting the pin is captured with the descriptor"
    );

    let kappa = descriptor["tokenizer_kappa"]
        .as_str()
        .expect("tokenizer_kappa is pinned");
    let hex = kappa
        .strip_prefix("blake3:")
        .expect("tokenizer_kappa is a blake3 address");
    assert_eq!(hex.len(), 64, "blake3 hex is 64 chars: {kappa}");
    assert!(
        hex.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "tokenizer_kappa is hex: {kappa}"
    );
    assert_eq!(
        descriptor["tokenizer_kappa_scope"].as_str(),
        Some("spiece.model"),
        "the pin is scoped to the spiece.model file"
    );
    assert_eq!(
        descriptor["tokenizer_bytes"].as_u64(),
        Some(791_656),
        "the tokenizer byte length is pinned"
    );
}

/// Presence-gated: the real pinned `spiece.model` reproduces the pinned
/// `tokenizer_kappa` byte-for-byte, and its length matches the pin.
/// UNAVAILABLE (vacuous pass) when the snapshot is absent.
#[test]
fn real_t5_spiece_matches_the_pin() {
    let descriptor = t5_descriptor();
    let kappa = descriptor["tokenizer_kappa"]
        .as_str()
        .expect("tokenizer_kappa is pinned")
        .to_owned();
    let expected_bytes = descriptor["tokenizer_bytes"]
        .as_u64()
        .expect("tokenizer_bytes is pinned");
    let source_dir = descriptor["source_directory"]
        .as_str()
        .expect("source_directory is declared");
    let spiece = repo_root().join(source_dir).join("spiece.model");
    if !spiece.is_file() {
        eprintln!(
            "UNAVAILABLE: real T5 spiece.model absent at {} — presence-gated pin check skipped",
            spiece.display()
        );
        return;
    }

    let bytes = std::fs::read(&spiece).expect("read spiece.model");
    assert_eq!(
        bytes.len() as u64,
        expected_bytes,
        "the snapshot spiece.model byte length must match the pin"
    );
    let measured = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    assert_eq!(
        measured, kappa,
        "the snapshot spiece.model κ must reproduce the pin in models/t5-base-tokenizer.json"
    );
}
