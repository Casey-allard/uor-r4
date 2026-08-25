//! Issue #946 binding cheap instrument.
//!
//! This ignored, teacher-free harness compiles at most 4,096 recorded train
//! positions and inspects at most 4,096 recorded held-out positions. The
//! treatment and context-only control have identical region, edge, ROUT,
//! EMIT, candidate, and active-node ceilings. A negative verdict is a valid
//! terminal measurement and deliberately does not fail the test process.

use std::path::Path;
use std::time::Instant;

use serde_json::json;
use uor_r4_core::transformerless::compiler::{self, STAGES};
use uor_r4_graph_compiler::induction as cover;
use uor_r4_graph_format::{GraphView, ScoreQ, SectionId};
use uor_r4_graph_runtime::R4G1Runtime;

const MAX_TRAIN_POSITIONS: usize = 4_096;
const MAX_HELD_OUT_POSITIONS: usize = 4_096;

fn cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn read(path: &Path, name: &str) -> Vec<u8> {
    std::fs::read(path.join(name))
        .unwrap_or_else(|error| panic!("read {}: {error}", path.join(name).display()))
}

fn candidates(
    runtime: &R4G1Runtime<'_>,
    context: &[u32],
    context_signature: &[u8],
    trajectory_signature: Option<&[u8]>,
) -> Vec<(u32, i32)> {
    let mut scores = vec![ScoreQ::MIN; runtime.node_count() as usize];
    let mut output = [(0u32, ScoreQ::MIN); 8];
    let count = runtime.predict_candidates_with_signature_lanes(
        context,
        Some(context_signature),
        trajectory_signature,
        &mut scores,
        &mut output,
    );
    output[..count]
        .iter()
        .map(|(token, score)| (*token, score.raw()))
        .collect()
}

#[test]
#[ignore = "teacher-free exact-input instrument; set R4_TRAJECTORY_PREFLIGHT_BUNDLE"]
fn exact_recorded_corpus_preflight() {
    let started = Instant::now();
    let bundle = std::env::var("R4_TRAJECTORY_PREFLIGHT_BUNDLE")
        .expect("R4_TRAJECTORY_PREFLIGHT_BUNDLE must name the exact #933 full bundle");
    let bundle = Path::new(&bundle);
    let artifact_bytes = read(bundle, "tless_artifacts.bin");
    let corpus_meta = read(bundle, "corpus.meta");
    let corpus_records = read(bundle, "corpus.records");
    let artifacts = compiler::parse_artifacts(&artifact_bytes).expect("TLA parses");
    let corpus = compiler::load_corpus_bytes(&corpus_meta, &corpus_records, None)
        .expect("recorded corpus parses");
    let (mut train_positions, mut held_out_positions) = cover::split_positions(&corpus);
    train_positions.truncate(MAX_TRAIN_POSITIONS);
    held_out_positions.truncate(MAX_HELD_OUT_POSITIONS);
    assert!(
        !train_positions.is_empty(),
        "preflight needs train positions"
    );
    assert!(
        !held_out_positions.is_empty(),
        "preflight needs held-out positions"
    );

    let extraction_started = Instant::now();
    let train = cover::build_observations_with_threads(&artifacts, &corpus, &train_positions, 1);
    let held_out =
        cover::build_observations_with_threads(&artifacts, &corpus, &held_out_positions, 1);
    let extraction_ms = extraction_started.elapsed().as_millis();

    let config = cover::CoverConfig {
        depths: 2,
        k0: 32,
        regions_budget: 64,
        memory_budget_bytes: 256 * 1024 * 1024,
        threads: 1,
        min_support: 16,
        entropy_gain_bits: 0.25,
        radius_quantile_numerator: 95,
        radius_quantile_denominator: 100,
        ..cover::CoverConfig::default()
    };
    let artifact_kappa = cid(&artifact_bytes);
    let mut corpus_hasher = blake3::Hasher::new();
    corpus_hasher.update(&corpus_meta);
    corpus_hasher.update(&corpus_records);
    let corpus_kappa = format!("blake3:{}", corpus_hasher.finalize().to_hex());

    let induction_started = Instant::now();
    let mut induced = cover::induce_cover(&train, &config, &artifact_kappa, &corpus_kappa)
        .expect("bounded cover induction succeeds");
    assert_eq!(
        cover::attach_trajectory_routing(
            &mut induced.cover,
            &train,
            config.radius_quantile_numerator,
            config.radius_quantile_denominator,
        ),
        induced.cover.regions.len(),
        "every fixed-budget region receives one trajectory prototype"
    );
    let induction_ms = induction_started.elapsed().as_millis();

    let mut control = induced.cover.clone();
    for region in &mut control.regions {
        region.trajectory_sig = Some(region.sig);
        region.trajectory_radius = Some(region.radius);
    }
    let reference = cover::ReferenceClassifier::freeze(&induced.cover);
    let edges = cover::build_edges(&induced.cover, &reference, &train, &corpus.story);
    let prior = cover::root_prior(&train);
    let vocab = (artifacts.token_codes.len() / STAGES) as u32;
    let emit = |graph: &cover::Cover| {
        cover::emit_r4g1(
            &artifact_bytes,
            (&corpus_meta, &corpus_records),
            vocab,
            graph,
            &edges,
            &prior,
            &train,
        )
        .expect("bounded in-memory artifact emits")
        .0
    };
    let emission_started = Instant::now();
    let treatment_bytes = emit(&induced.cover);
    let treatment_repeat = emit(&induced.cover);
    let control_bytes = emit(&control);
    let emission_ms = emission_started.elapsed().as_millis();
    assert_eq!(
        treatment_bytes, treatment_repeat,
        "double compilation identity"
    );
    assert_eq!(
        treatment_bytes.len(),
        control_bytes.len(),
        "equal byte budget"
    );
    let treatment_view = GraphView::parse(&treatment_bytes).expect("treatment admits");
    let control_view = GraphView::parse(&control_bytes).expect("control admits");
    assert_eq!(treatment_view.node_count(), control_view.node_count());
    assert_eq!(treatment_view.edge_count(), control_view.edge_count());
    assert_eq!(
        treatment_view.section(SectionId::ROUT).map(<[u8]>::len),
        control_view.section(SectionId::ROUT).map(<[u8]>::len)
    );
    assert_eq!(
        treatment_view.section(SectionId::EMIT),
        control_view.section(SectionId::EMIT)
    );
    let treatment = R4G1Runtime::parse(&treatment_bytes).expect("treatment runtime admission");
    let control = R4G1Runtime::parse(&control_bytes).expect("control runtime admission");

    let mut inspected = 0usize;
    let mut fallback_positions = 0usize;
    let mut trajectory_admissions = 0usize;
    let mut effects = 0usize;
    let mut first_effect = None;
    for observation in &held_out {
        inspected += 1;
        let context = cover::context_window(&corpus, observation.position as usize);
        let mut scores = vec![ScoreQ::MIN; treatment.node_count() as usize];
        let (_, trace) = treatment.predict_distribution_with_signature_lanes_traced(
            &context,
            Some(&observation.sig),
            Some(&observation.trajectory_sig),
            &mut scores,
        );
        if !trace.context_probe_attempted {
            continue;
        }
        fallback_positions += 1;
        if trace.session_admitted_nodes == 0 {
            continue;
        }
        trajectory_admissions += 1;
        let treatment_candidates = candidates(
            &treatment,
            &context,
            &observation.sig,
            Some(&observation.trajectory_sig),
        );
        let control_candidates =
            candidates(&control, &context, &observation.sig, Some(&observation.sig));
        if treatment_candidates != control_candidates {
            effects += 1;
            if first_effect.is_none() {
                first_effect = Some(json!({
                    "position": observation.position,
                    "context_admitted_nodes": trace.context_admitted_nodes,
                    "trajectory_admitted_nodes": trace.session_admitted_nodes,
                    "treatment_candidates": treatment_candidates,
                    "control_candidates": control_candidates,
                }));
            }
        }
    }

    let verdict = if effects > 0 {
        "PREFLIGHT_POSITIVE"
    } else {
        "REPRESENTATION_NOT_ESTABLISHED"
    };
    let report = json!({
        "schema": "uor-r4.trajectory-routing-preflight/1",
        "issue": 946,
        "verdict": verdict,
        "teacher_loaded": false,
        "limits": {
            "train_positions": MAX_TRAIN_POSITIONS,
            "held_out_positions": MAX_HELD_OUT_POSITIONS,
            "context_active_nodes": 4,
            "trajectory_active_nodes": 4,
            "total_active_nodes": 8,
        },
        "inputs": {
            "artifact_kappa": artifact_kappa,
            "corpus_kappa": corpus_kappa,
        },
        "budgets": {
            "regions": induced.cover.regions.len(),
            "nodes": treatment_view.node_count(),
            "edges": treatment_view.edge_count(),
            "treatment_bytes": treatment_bytes.len(),
            "control_bytes": control_bytes.len(),
            "rout_bytes": treatment_view.section(SectionId::ROUT).map(<[u8]>::len),
            "emit_bytes": treatment_view.section(SectionId::EMIT).map(<[u8]>::len),
        },
        "counts": {
            "inspected": inspected,
            "fallback_positions": fallback_positions,
            "trajectory_admissions": trajectory_admissions,
            "candidate_effects": effects,
        },
        "first_effect": first_effect,
        "outputs": {
            "treatment_cid": cid(&treatment_bytes),
            "control_cid": cid(&control_bytes),
        },
        "timing_ms": {
            "observation_extraction": extraction_ms,
            "induction": induction_ms,
            "emission": emission_ms,
            "total": started.elapsed().as_millis(),
        },
    });
    println!("ISSUE_946_PREFLIGHT_REPORT={report}");
}
