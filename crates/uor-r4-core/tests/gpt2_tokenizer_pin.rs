//! #669: the pinned GPT-2 tokenizer identity in `models/gpt2-124m.json`.
//!
//! The source descriptor pins `tokenizer_kappa` — the blake3 of the pinned
//! `openai-community/gpt2` `tokenizer.json` (revision `607a30d7…`) — so a
//! compiled GPT-2 bundle can be checked against the exact tokenizer the
//! #601 `hf-byte-bpe/1` adapter records, not merely the id family. The
//! `hf-byte-bpe` byte-level BPE ingestion and its adapter record are the
//! same machinery SmolLM2 uses (`tokenizer_adapter.rs`); this test binds
//! the SPECIFIC GPT-2 tokenizer to a pinned CID.
//!
//! Two-state coverage:
//! - [`gpt2_tokenizer_pin_is_well_formed`] runs in CI: the pin is present
//!   and structurally valid, no snapshot required.
//! - [`real_gpt2_tokenizer_matches_the_pin`] is presence-gated (#599
//!   three-state) on the real 548 MB snapshot, a dev/local compiler input
//!   that never downloads in CI: when present it must reproduce the pin
//!   byte-for-byte; when absent it reports UNAVAILABLE and passes
//!   vacuously — never a silent skip of a real failure.

use std::path::PathBuf;

use uor_r4_core::transformerless::hf_bpe::{HfBpeTokenizer, TokenizerAdapter};

/// Workspace root: `CARGO_MANIFEST_DIR` is `<root>/crates/uor-r4-core`.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn gpt2_descriptor() -> serde_json::Value {
    let path = repo_root().join("models").join("gpt2-124m.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_slice(&bytes).expect("models/gpt2-124m.json parses")
}

/// CI-run: the tokenizer pin is present and well-formed — a
/// `blake3:<64 hex>` address scoped to `tokenizer.json` with the pinned
/// byte length. Runs without the snapshot.
#[test]
fn gpt2_tokenizer_pin_is_well_formed() {
    let descriptor = gpt2_descriptor();

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
        Some("tokenizer.json"),
        "the pin is scoped to the tokenizer.json file"
    );
    assert_eq!(
        descriptor["tokenizer_bytes"].as_u64(),
        Some(1_355_256),
        "the tokenizer byte length is pinned"
    );
}

/// Presence-gated: the real pinned snapshot's `tokenizer.json` reproduces
/// the pinned `tokenizer_kappa`, and the #601 `hf-byte-bpe/1` adapter
/// built from it carries that SAME CID — the exact-tokenizer binding, not
/// just the id family. UNAVAILABLE (vacuous pass) when the snapshot is
/// absent.
#[test]
fn real_gpt2_tokenizer_matches_the_pin() {
    let descriptor = gpt2_descriptor();
    let kappa = descriptor["tokenizer_kappa"]
        .as_str()
        .expect("tokenizer_kappa is pinned")
        .to_owned();
    let source_dir = descriptor["source_directory"]
        .as_str()
        .expect("source_directory is declared");
    let dir = repo_root().join(source_dir);
    let tokenizer_json = dir.join("tokenizer.json");
    if !tokenizer_json.is_file() {
        eprintln!(
            "UNAVAILABLE: real gpt2 tokenizer absent at {} — presence-gated pin check skipped",
            tokenizer_json.display()
        );
        return;
    }

    // The pinned bytes reproduce the pinned κ.
    let bytes = std::fs::read(&tokenizer_json).expect("read tokenizer.json");
    let measured = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    assert_eq!(
        measured, kappa,
        "the snapshot tokenizer.json κ must reproduce the pin in models/gpt2-124m.json"
    );

    // The #601 adapter built from it binds the SAME CID under the
    // hf-byte-bpe/1 family — the record a compiled bundle carries.
    let tokenizer = HfBpeTokenizer::from_dir(&dir).expect("load hf byte-level bpe tokenizer");
    let adapter = tokenizer.adapter();
    assert_eq!(adapter.family, TokenizerAdapter::HF_BYTE_BPE_FAMILY);
    assert_eq!(adapter.version, TokenizerAdapter::HF_BYTE_BPE_VERSION);
    assert_eq!(
        adapter.tokenizer_cid, kappa,
        "the adapter binds the pinned tokenizer.json κ"
    );
}
