//! #605 route-fit tests, compiler side: the versioned method registry,
//! the eight-identity fit manifest, the production-boundary trace
//! corpus, and the deterministic double-run of the fit itself. The
//! replacement-ladder tests live in
//! `crates/uor-r4-graph-certify/tests/route_fit_605.rs` (the ladder is
//! certify-side).
//!
//! Fixture discipline: the synthetic teacher and its mini-corpus are
//! integer-seeded and clock-free; the ONLY nondeterminism in this file
//! is the temp-directory naming (process-unique, never part of any
//! measured byte).

use std::time::{SystemTime, UNIX_EPOCH};

use uor_r4_graph_compiler::observation::{
    ObservationManifest, merge_shards, merge_trace_rows, shard_file_name, trace_sidecar_name,
};
use uor_r4_graph_compiler::route_fit::{
    FIT_MANIFEST_FORMAT, ROUTE_FIT_ID, ROUTE_FIT_VERSION, RouteFitMethod, SYNTH_CORPUS_TOKENS,
    SYNTH_SEQ_LEN, SYNTH_SHARD_BITS, SyntheticRouteTeacher, fit_method_spec, fit_route_codes,
    generate_synthetic_route_trace, load_route_trace_corpus, route_fit_v1_parameter_labels,
    synthetic_capture_geometry, synthetic_fit_manifest, synthetic_trace_profile,
};
use uor_r4_model_source::TeacherOracle;

fn unique_path(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("uor-r4-{name}-{nanos}"))
}

/// Test 6 (task list): the registry refuses every unknown
/// `(id, version)` by name — never guesses, never resolves a "closest"
/// method.
#[test]
fn registry_refuses_unknown_id_and_version_by_name() {
    let known = fit_method_spec(ROUTE_FIT_ID, ROUTE_FIT_VERSION).expect("registered method");
    assert_eq!(known, RouteFitMethod::route_fit_v1());
    for (id, version) in [
        (ROUTE_FIT_ID, 2u32),
        (ROUTE_FIT_ID, 0),
        ("mystery-fit", 1),
        ("route-fit-2", 1),
    ] {
        let error =
            fit_method_spec(id, version).expect_err("unknown (id, version) is not a product");
        assert!(error.reason.contains(id), "reason names the id: {error}");
        assert!(
            error.reason.contains(&version.to_string()),
            "reason names the version: {error}"
        );
    }
}

/// The v1 parameter labeling separates compiled from declared from
/// absent, and no v1 parameter claims source-weight provenance.
#[test]
fn parameter_labels_classify_compiled_vs_declared_vs_absent() {
    let labels = route_fit_v1_parameter_labels();
    let class_of = |name: &str| {
        labels
            .iter()
            .find(|label| label.name == name)
            .map(|label| label.class.as_str())
            .expect("label present")
    };
    assert_eq!(class_of("route_codes"), "compiled");
    assert_eq!(class_of("thresholds"), "compiled");
    assert_eq!(class_of("mask"), "declared");
    assert_eq!(class_of("contributions"), "declared");
    assert_eq!(class_of("top_m"), "declared");
    assert_eq!(class_of("radii"), "absent");
    assert_eq!(class_of("residual_projection"), "absent");
    assert_eq!(class_of("source_weights"), "absent");
}

fn corpus_file_bytes(dir: &std::path::Path) -> Vec<Vec<u8>> {
    let manifest = ObservationManifest::load(dir)
        .expect("manifest io")
        .expect("manifest present");
    let mut files = Vec::new();
    for shard in 0..manifest.shard_count() {
        let name = shard_file_name(SYNTH_SHARD_BITS, shard);
        files.push(std::fs::read(dir.join(&name)).unwrap_or_default());
        files.push(std::fs::read(dir.join(format!("{name}.prob"))).unwrap_or_default());
        files.push(
            std::fs::read(dir.join(trace_sidecar_name(SYNTH_SHARD_BITS, shard)))
                .unwrap_or_default(),
        );
    }
    files
}

/// Test 1 (task list), fit half: two independent generate → load → fit
/// runs produce byte-identical trace corpora, byte-identical fitted
/// parameters, and equal manifest κs.
#[test]
fn deterministic_double_run_fit_is_byte_identical() {
    let dir_a = unique_path("route-fit-double-a");
    let dir_b = unique_path("route-fit-double-b");
    let summary_a = generate_synthetic_route_trace(&dir_a).expect("corpus a");
    let summary_b = generate_synthetic_route_trace(&dir_b).expect("corpus b");
    assert!(summary_a.done, "corpus a reached its target");
    assert!(summary_b.done, "corpus b reached its target");
    assert_eq!(
        corpus_file_bytes(&dir_a),
        corpus_file_bytes(&dir_b),
        "the production #603 pipeline must write byte-identical corpora"
    );

    let teacher = SyntheticRouteTeacher::new();
    let geometry = synthetic_capture_geometry();
    let corpus_a = load_route_trace_corpus(&dir_a, geometry, teacher.bos_token() as u32)
        .expect("corpus a loads");
    let corpus_b = load_route_trace_corpus(&dir_b, geometry, teacher.bos_token() as u32)
        .expect("corpus b loads");
    assert_eq!(corpus_a.records, SYNTH_CORPUS_TOKENS);
    assert_eq!(corpus_a.records_kappa, corpus_b.records_kappa);
    assert_eq!(corpus_a.trace_kappa, corpus_b.trace_kappa);
    assert_eq!(
        corpus_a.identity_bundle_digest,
        corpus_b.identity_bundle_digest
    );

    let fitted_a = fit_route_codes(&corpus_a).expect("fit a");
    let fitted_b = fit_route_codes(&corpus_b).expect("fit b");
    assert_eq!(
        fitted_a.canonical_bytes(),
        fitted_b.canonical_bytes(),
        "double-run fitted parameters must be byte-identical"
    );
    assert_eq!(fitted_a.kappa(), fitted_b.kappa());
    // And an in-process re-fit over the SAME corpus is byte-identical
    // too (no hidden iteration-order dependence).
    let fitted_again = fit_route_codes(&corpus_a).expect("fit a again");
    assert_eq!(fitted_a.canonical_bytes(), fitted_again.canonical_bytes());

    let manifest_a = synthetic_fit_manifest(&corpus_a, &teacher.kappa()).expect("manifest a");
    let manifest_b = synthetic_fit_manifest(&corpus_b, &teacher.kappa()).expect("manifest b");
    assert_eq!(manifest_a.canonical_bytes(), manifest_b.canonical_bytes());
    assert_eq!(manifest_a.kappa(), manifest_b.kappa());
    assert_eq!(manifest_a.format, FIT_MANIFEST_FORMAT);
    // The eight identity fields: five present with real values, the
    // tokenizer genuinely absent (typed None, not an empty string).
    assert!(manifest_a.source_snapshot.is_some());
    assert!(manifest_a.tokenizer.is_none());
    assert!(manifest_a.adapter.is_some());
    assert!(manifest_a.trace.is_some());
    assert!(manifest_a.geometry_identity.is_some());
    assert!(manifest_a.operator_identity.is_some());
    assert!(manifest_a.corpus.is_some());
    assert!(manifest_a.compiler.is_some());

    for dir in [&dir_a, &dir_b] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// The fit input boundary is the production one: the corpus loads back
/// through the #603 merge surfaces, every story fits the deployed
/// candidate bound, the declared profile is the registered `full/1`,
/// and the q/k + support lanes are populated.
#[test]
fn trace_corpus_is_production_shaped_and_bounded() {
    let dir = unique_path("route-fit-corpus-shape");
    generate_synthetic_route_trace(&dir).expect("corpus");
    // Production merge surfaces read the same directory.
    let records = merge_shards(&dir).expect("records merge");
    let trace = merge_trace_rows(&dir).expect("trace merge");
    assert_eq!(records.len() % 88, 0);
    assert!(!trace.is_empty());

    let teacher = SyntheticRouteTeacher::new();
    let corpus = load_route_trace_corpus(
        &dir,
        synthetic_capture_geometry(),
        teacher.bos_token() as u32,
    )
    .expect("corpus loads");
    assert_eq!(corpus.trace_profile, synthetic_trace_profile());
    assert_eq!(corpus.declared_layers, vec![0, 1]);
    let mut total_steps = 0usize;
    for story in &corpus.stories {
        assert!(story.steps.len() <= SYNTH_SEQ_LEN);
        assert_eq!(story.tokens.len(), story.steps.len());
        assert_eq!(story.tokens[0], teacher.bos_token() as u32);
        for (pos, step) in story.steps.iter().enumerate() {
            assert_eq!(step.pos as usize, pos);
            assert_eq!(step.q_rows.len(), 2);
            assert_eq!(step.k_rows.len(), 2);
            assert_eq!(step.q_rows[0].len(), corpus.geometry.residual_width);
            // The support lane is bounded: at most min(pos + 1, cap)
            // real entries — absent slots were decoded as absence,
            // never as zero entries.
            for lane in &step.supports {
                for head_support in lane {
                    assert_eq!(
                        head_support.len(),
                        (pos + 1).min(corpus.support_size as usize)
                    );
                    for &(position, weight) in head_support {
                        assert!(position as usize <= pos);
                        assert!(weight.is_finite());
                    }
                }
            }
            total_steps += 1;
        }
    }
    assert_eq!(total_steps, corpus.records);
    let _ = std::fs::remove_dir_all(&dir);
}
