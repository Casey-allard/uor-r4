//! #670: presence-gated end-to-end compile canary for the pinned GPT-2 source.
//!
//! Increment 1 of #670: proves a pinned `openai-community/gpt2` source compiles
//! END-TO-END through the one-shot `compile_hugging_face_with_progress` — the
//! same entry the CLI `compile` uses, which loads the source via the `Teacher`
//! enum (GPT-2 dispatch, #671), self-generates a bounded teacher corpus, and
//! emits the table-native artifact bundle. This exercises, on the real GPT-2
//! teacher, the learned-absolute attention operator (#672), the batched
//! executor (#675), and the tokenizer pin (#673) end to end.
//!
//! Presence-gated (#599 three-state): the 548 MB snapshot is a dev/local
//! compiler input, never a CI download, so when it is absent this reports
//! UNAVAILABLE and passes vacuously — never a silent skip of a real failure.
//!
//! Checks when the snapshot is present:
//! - the compile succeeds and emits a table-native artifact bundle whose
//!   `tless_artifacts.bin` parses through `compiler::parse_artifacts`;
//! - a deterministic double-run produces byte-identical artifact bytes (the
//!   corpus generator is integer-seeded; per-machine matmul is deterministic).
//!
//! The R4G1 route artifact + Gate C parity + the #606 source-parity
//! certificate rows (operator #602 / geometry #600 / tokenizer #601) are the
//! remaining extension of #670 — they drive the separate cover -> score ->
//! certify stages and are best built and verified with the snapshot present
//! (pipeline map recorded in project memory `gpt2_canary_670.md`). This canary
//! is additive: it adds no SmolLM2 path, so SmolLM2 compiles byte-unchanged.

use std::path::{Path, PathBuf};

use uor_r4_core::transformerless::compiler;
use uor_r4_graph_cli::compile_hugging_face_with_progress;

/// Workspace root: `CARGO_MANIFEST_DIR` is `<root>/crates/uor-r4-graph-cli`.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn gpt2_source() -> PathBuf {
    repo_root().join(".uor-models/sources/gpt2-124m")
}

/// Test-only temp output directory; its name never reaches a measured byte.
fn unique_out(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("uor-r4-gpt2-670-{tag}-{}", std::process::id()))
}

/// A bounded compile: a small target-record count is enough to prove the
/// pipeline runs end to end; the heavy corpus-scale run is a separate job.
fn compile_args(source: &Path, out: &Path) -> Vec<String> {
    vec![
        "--source".to_owned(),
        source.to_string_lossy().into_owned(),
        "--output".to_owned(),
        out.to_string_lossy().into_owned(),
        "--target".to_owned(),
        "16".to_owned(),
        "--sequence-length".to_owned(),
        "16".to_owned(),
    ]
}

#[test]
fn gpt2_source_compiles_end_to_end() {
    let source = gpt2_source();
    if !source.join("model.safetensors").exists() {
        eprintln!(
            "UNAVAILABLE: real gpt2 snapshot absent at {} — presence-gated #670 compile canary skipped",
            source.display()
        );
        return;
    }

    let out_a = unique_out("a");
    let out_b = unique_out("b");
    let _ = std::fs::remove_dir_all(&out_a);
    let _ = std::fs::remove_dir_all(&out_b);

    // First compile: the pinned GPT-2 source through the real pipeline.
    compile_hugging_face_with_progress(&compile_args(&source, &out_a), |_, _| {})
        .expect("gpt2 source compiles end-to-end");
    let artifacts_a = out_a.join("tless_artifacts.bin");
    assert!(
        artifacts_a.is_file(),
        "compile emitted the table-native artifact at {}",
        artifacts_a.display()
    );
    let bytes_a = std::fs::read(&artifacts_a).expect("read tless_artifacts.bin");
    assert!(
        compiler::parse_artifacts(&bytes_a).is_some(),
        "the emitted artifact is a valid table-native bundle"
    );

    // Deterministic double-run: a second compile yields byte-identical bytes.
    compile_hugging_face_with_progress(&compile_args(&source, &out_b), |_, _| {})
        .expect("gpt2 source compiles end-to-end (second run)");
    let bytes_b =
        std::fs::read(out_b.join("tless_artifacts.bin")).expect("read second-run artifact");
    assert_eq!(
        bytes_a, bytes_b,
        "the GPT-2 compile is deterministic — byte-identical artifact across runs"
    );

    let _ = std::fs::remove_dir_all(&out_a);
    let _ = std::fs::remove_dir_all(&out_b);
}
