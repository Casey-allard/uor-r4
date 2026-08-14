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
//! Increment 2 (`gpt2_cover_certifies_source_parity_rows`) extends the canary
//! through the cover stage: the compiled teacher drives cover induction to the
//! R4G1 route artifact + report, and the report carries the #606 source-parity
//! rows — the source-manifest κ (#597), the geometry projection (#600), and the
//! learned-absolute attention operator (#602) the GPT-2 oracle declares — with
//! held-out routing recall measured (Gate C) and the induced cover
//! deterministic across runs. (The serialized R4G1 embeds a per-run
//! HashMap-seeded provenance digest, so the *induced structure* — regions and
//! recall — is the reproducibility invariant, not the raw artifact bytes.)
//!
//! This canary is additive: it adds no SmolLM2 path, so SmolLM2 compiles
//! byte-unchanged.

use std::path::{Path, PathBuf};

use uor_r4_core::transformerless::compiler;
use uor_r4_graph_cli::{compile_hugging_face_with_progress, cover_command};
use uor_r4_model_source::attention::AttentionOperatorSpec;
use uor_r4_model_source::geometry::{self, GeometryProjection};

/// GPT-2 (124M) source hidden width; the geometry projection the oracle
/// declares is `bucket_average(768 → COMPILED_WIDTH)`.
const GPT2_N_EMBD: u32 = 768;

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

/// A bounded compile that emits enough teacher corpus for cover induction to
/// split into train + held-out (so held-out routing recall is measured).
fn cover_compile_args(source: &Path, out: &Path) -> Vec<String> {
    vec![
        "--source".to_owned(),
        source.to_string_lossy().into_owned(),
        "--output".to_owned(),
        out.to_string_lossy().into_owned(),
        "--target".to_owned(),
        "128".to_owned(),
        "--sequence-length".to_owned(),
        "16".to_owned(),
    ]
}

/// Bounded cover over a compiled teacher container, carrying the GPT-2 source
/// rows (#597 κ, #600 geometry, #602 attention operator) into the report.
fn cover_args(
    compiled: &Path,
    out: &Path,
    kappa: &str,
    geometry_json: &str,
    operator_json: &str,
) -> Vec<String> {
    vec![
        "--artifacts".to_owned(),
        compiled
            .join("tless_artifacts.bin")
            .to_string_lossy()
            .into_owned(),
        "--corpus-meta".to_owned(),
        compiled.join("corpus.meta").to_string_lossy().into_owned(),
        "--corpus-recs".to_owned(),
        compiled
            .join("corpus.records")
            .to_string_lossy()
            .into_owned(),
        "--out".to_owned(),
        out.to_string_lossy().into_owned(),
        "--source-manifest-kappa".to_owned(),
        kappa.to_owned(),
        "--geometry-projection".to_owned(),
        geometry_json.to_owned(),
        "--attention-operator".to_owned(),
        operator_json.to_owned(),
        "--regions-budget".to_owned(),
        "16".to_owned(),
        "--memory-budget".to_owned(),
        "256".to_owned(),
    ]
}

/// #670 increment 2: the pinned GPT-2 teacher, once compiled, drives cover
/// induction to the R4G1 route artifact + report, and that report carries the
/// #606 source-parity rows — the source-manifest κ (#597), the geometry
/// projection (#600), and the learned-absolute attention operator (#602) the
/// GPT-2 oracle declares — with held-out routing recall measured (Gate C).
/// Presence-gated: vacuous UNAVAILABLE pass when the snapshot is absent.
#[test]
fn gpt2_cover_certifies_source_parity_rows() {
    let source = gpt2_source();
    if !source.join("model.safetensors").exists() {
        eprintln!(
            "UNAVAILABLE: real gpt2 snapshot absent at {} — presence-gated #670 cover canary skipped",
            source.display()
        );
        return;
    }

    // The pinned source-snapshot κ (#597) from the model descriptor (#669).
    let descriptor: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo_root().join("models/gpt2-124m.json")).expect("read gpt2 descriptor"),
    )
    .expect("descriptor parses");
    let source_kappa = descriptor["source_kappa"]
        .as_str()
        .expect("descriptor carries source_kappa")
        .to_owned();

    // The source rows the GPT-2 oracle declares (see `uor-r4-model-source`
    // gpt2.rs `attention_operator_spec` / `geometry_projection`).
    let operator = AttentionOperatorSpec::learned_absolute_source_attention();
    let geometry_proj = GeometryProjection::bucket_average(GPT2_N_EMBD, geometry::COMPILED_WIDTH);
    let operator_json = serde_json::to_string(&operator).expect("operator serializes");
    let geometry_json = serde_json::to_string(&geometry_proj).expect("geometry serializes");

    let compiled = unique_out("cover-src");
    let cover_a = unique_out("cover-a");
    let cover_b = unique_out("cover-b");
    for dir in [&compiled, &cover_a, &cover_b] {
        let _ = std::fs::remove_dir_all(dir);
    }

    compile_hugging_face_with_progress(&cover_compile_args(&source, &compiled), |_, _| {})
        .expect("gpt2 source compiles for cover");
    assert!(
        compiled.join("corpus.meta").is_file() && compiled.join("corpus.records").is_file(),
        "compile emits the teacher corpus streams for cover"
    );

    cover_command(&cover_args(
        &compiled,
        &cover_a,
        &source_kappa,
        &geometry_json,
        &operator_json,
    ))
    .expect("cover induction over the GPT-2 teacher");

    let r4g1_a = cover_a.join("cover.r4g1");
    assert!(r4g1_a.is_file(), "cover emits the R4G1 route artifact");
    let bytes_a = std::fs::read(&r4g1_a).expect("read cover.r4g1");
    assert!(!bytes_a.is_empty(), "the R4G1 artifact is non-empty");

    let report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(cover_a.join("cover_report.json")).expect("read cover_report.json"),
    )
    .expect("report parses");

    // #597: the source-manifest κ is carried verbatim.
    assert_eq!(
        report["source_manifest_kappa"].as_str(),
        Some(source_kappa.as_str()),
        "report carries the source-manifest κ"
    );
    // #602: the attention operator row round-trips to GPT-2 learned-absolute.
    let report_operator: AttentionOperatorSpec =
        serde_json::from_value(report["attention_operator"].clone())
            .expect("report carries an attention-operator row");
    assert_eq!(
        report_operator, operator,
        "the report's operator is the GPT-2 learned-absolute source attention"
    );
    // #600: the geometry row round-trips to bucket-average(768 → compiled).
    let report_geometry: GeometryProjection =
        serde_json::from_value(report["geometry"].clone()).expect("report carries a geometry row");
    assert_eq!(
        report_geometry, geometry_proj,
        "the report's geometry is the GPT-2 bucket-average projection"
    );
    // Gate C: held-out routing recall is measured (non-empty).
    assert!(
        report["recall"]
            .as_array()
            .map(|r| !r.is_empty())
            .unwrap_or(false),
        "the report measures held-out routing recall"
    );

    // Deterministic induction: a second run over the same compiled teacher
    // induces the identical cover — same regions and same held-out recall.
    // (The serialized R4G1 carries a provenance digest seeded per-run by the
    // std HashMap's random state, so raw artifact bytes are not the invariant;
    // the induced structure is.)
    cover_command(&cover_args(
        &compiled,
        &cover_b,
        &source_kappa,
        &geometry_json,
        &operator_json,
    ))
    .expect("cover induction (second run)");
    let bytes_b = std::fs::read(cover_b.join("cover.r4g1")).expect("read second-run cover.r4g1");
    assert_eq!(
        bytes_a.len(),
        bytes_b.len(),
        "the induced R4G1 has a stable size across runs"
    );
    let report_b: serde_json::Value = serde_json::from_slice(
        &std::fs::read(cover_b.join("cover_report.json")).expect("read second cover_report.json"),
    )
    .expect("second report parses");
    assert_eq!(
        report["recall"], report_b["recall"],
        "held-out routing recall is deterministic across runs"
    );
    assert_eq!(
        report["regions"], report_b["regions"],
        "the induced region cover is deterministic across runs"
    );

    for dir in [&compiled, &cover_a, &cover_b] {
        let _ = std::fs::remove_dir_all(dir);
    }
}
