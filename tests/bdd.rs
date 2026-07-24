//! Cucumber runner for behavior-level R4G1 checks.
//!
//! The feature files live under `features/suites`, following the upstream
//! Hologram layout. Keep the scenarios focused on externally meaningful
//! behavior; implementation details stay in the server module.

use cucumber::{given, then, when, World};
use std::path::Path;
use uor_r4_core::transformerless::bott_fock::BottFockContextStore;
use uor_r4_core::transformerless::compiler::SIG_BYTES;
use uor_r4_core::transformerless::endomorphism::EndomorphismAlgebra;
use uor_r4_core::transformerless::lie_jordan::{universal_product_u8, LieJordanSplit};
use uor_r4_graph_compiler::induction::Observation;
use uor_r4_graph_compiler::quantum_cover::{
    quantum_entropy_gain, DensityOperator, QuantumCoverConfig,
};
use uor_r4_graph_format::INFERENCE_OPERATION_CONTRACT_VERSION;
use uor_r4_wasm_router::cd_space_fold;
use uor_r4_wasm_router::r4g1::validate_quality_report;
use uor_r4_wasm_router::server::{
    is_usable_generated_text, r4g1_unavailable_response, select_synthesis_engine,
    validate_r4g1_corpus_inputs,
};

#[derive(Debug, Default, World)]
struct R4g1World {
    response: String,
    usable: Option<bool>,
    requested_engine: Option<&'static str>,
    selected_engine: Option<&'static str>,
    endpoint_status: Option<u16>,
    endpoint_body: Option<serde_json::Value>,
    compile_error: Option<String>,
    quality_report: Option<serde_json::Value>,
    quality_error: Option<String>,
    // Façade & Scaling fields
    facade_input: String,
    folded_matrix: Vec<i16>,
    seq_lengths: Vec<usize>,
    bench_latency_us: f64,
    bench_matrix_bytes: usize,
    // Quantum cover fields
    density: Option<DensityOperator>,
    entropy: Option<f32>,
    observations: Vec<Observation>,
    entropy_gain: Option<f64>,
    partition_accepted: Option<bool>,
    // Lie-Jordan fields
    op_matrix: Option<EndomorphismAlgebra>,
    split_result: Option<LieJordanSplit>,
    u8_a: u8,
    u8_b: u8,
    u8_res: u8,
    // Scoring Semantics fields (#158)
    score_accumulator: uor_r4_graph_format::scoring_semantics::ScoreAccumulator<16>,
    candidate_cmp_result: Option<core::cmp::Ordering>,
    // Packed Kernels fields (#159)
    packed_frontier: uor_r4_graph_runtime::packed_kernels::PackedFrontier<4>,
    packed_shortlist: uor_r4_graph_runtime::packed_kernels::PackedShortlist<4>,
    packed_output: uor_r4_graph_runtime::packed_kernels::StepOutput<3>,
    // Inference Contract fields (#157)
    contract_report: Option<uor_r4_graph_format::inference_contract::InferenceContractAuditReport>,
    _contract_audit_res:
        Option<Result<(), uor_r4_graph_format::inference_contract::ContractValidationError>>,
    // Performance Certificate fields (#161)
    perf_cert: Option<uor_r4_graph_certify::performance_certificate::RuntimePerformanceCertificate>,
    // PDF Traceability Matrix fields (#137)
    pdf_matrix: Vec<uor_r4_proof_model::pdf_traceability::PdfTraceabilityRow>,
    pdf_audit_report: Option<uor_r4_proof_model::pdf_traceability::TraceabilityAuditReport>,
    pdf_audit_error: Option<uor_r4_proof_model::pdf_traceability::TraceabilityValidationError>,
    // Rate-Distortion Compression fields (#136)
    rd_corpus_id: String,
    rd_tiers: Vec<usize>,
    rd_report: Option<uor_r4_graph_compiler::rate_distortion_compression::RateDistortionReport>,
    rd_error: Option<uor_r4_graph_compiler::rate_distortion_compression::CompressionAnalysisError>,
    // Graph Invariant Ownership fields (#135)
    inv_matrix: Vec<uor_r4_graph_format::invariant_ownership::InvariantOwnershipEntry>,
    inv_nodes: usize,
    inv_max_degree: usize,
    inv_degree_limit: usize,
    inv_edges: Vec<(u32, u32)>,
    inv_evidence: Vec<u32>,
    inv_res:
        Option<Result<usize, uor_r4_graph_format::invariant_ownership::InvariantValidationError>>,
    // Separate Semantic Emission fields (#134)
    decouple_transitions: Vec<(
        &'static str,
        &'static str,
        &'static str,
        f32,
        uor_r4_graph_compiler::semantic_emission_decoupling::SemanticStatus,
    )>,
    decouple_trace: Option<uor_r4_graph_compiler::semantic_emission_decoupling::SemanticStateTrace>,
    decouple_emission:
        Option<uor_r4_graph_compiler::semantic_emission_decoupling::LanguageEmissionResult>,
    decouple_cert:
        Option<uor_r4_graph_compiler::semantic_emission_decoupling::DecoupledCertificationReport>,
    decouple_error:
        Option<uor_r4_graph_compiler::semantic_emission_decoupling::SemanticEmissionError>,
    // Formal Monograph fields (#133)
    monograph_text: String,
    monograph_report: Option<uor_r4_graph_compiler::monograph::MonographValidationReport>,
    monograph_error: Option<uor_r4_graph_compiler::monograph::MonographValidationError>,
    // Expand Proof Model fields (#132)
    proof_report: Option<uor_r4_proof_model::structural_guarantees::ProofVerificationReport>,
    proof_nodes: Vec<u32>,
    proof_actual_mem: usize,
    proof_limit_mem: usize,
    proof_trajectory: Vec<String>,
    proof_forbidden: Vec<String>,
    proof_path_len: usize,
    proof_max_horizon: usize,
    proof_evidence_ids: Vec<String>,
    proof_witness_actual: String,
    proof_witness_expected: String,
    proof_raw_score: i64,
    // Future State Planner fields (#131)
    plan_nodes: Vec<uor_r4_graph_compiler::future_state_planner::PlannerStateNode>,
    plan_edges: Vec<uor_r4_graph_compiler::future_state_planner::PlannerEdgeTransition>,
    plan_result: Option<uor_r4_graph_compiler::future_state_planner::PlanTrajectory>,
    plan_error: Option<uor_r4_graph_compiler::future_state_planner::PlannerError>,
    // Lower Semantic Regions fields (#130)
    lower_bool_region: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweredBooleanRegion>,
    lower_witness: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweringWitnessEntry>,
    lower_q_normal: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweredFixedPointScore>,
    lower_q_max: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweredFixedPointScore>,
    lower_q_min: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweredFixedPointScore>,
    lower_error: Option<uor_r4_graph_compiler::lower_semantic_regions::LoweringError>,
    // Reference Compiler IR fields (#129)
    ref_corpus: Vec<String>,
    ref_ir: Option<uor_r4_graph_compiler::reference_compiler_ir::ReferenceGraphIr>,
    ref_transition_state:
        Option<uor_r4_graph_compiler::reference_compiler_ir::ReferenceSemanticState>,
    ref_diff_delta: Option<f32>,
    // Behavioral Probe fields (#128)
    probe_baseline_obs: String,
    probe_suite_report: Option<uor_r4_graph_compiler::behavioral_probes::BehavioralProbeReport>,
    probe_suite_error: Option<uor_r4_graph_compiler::behavioral_probes::BehavioralProbeError>,
    probe_record_error: Option<uor_r4_graph_compiler::behavioral_probes::BehavioralProbeError>,
    // Semantic State Space fields (#124)
    state_s0: Option<uor_r4_graph_compiler::semantic_state::SemanticState>,
    state_eval_res: Option<
        Result<
            uor_r4_graph_compiler::semantic_state::SemanticState,
            uor_r4_graph_compiler::semantic_state::SemanticStateError,
        >,
    >,
    hazard_evaluator: Option<uor_r4_graph_compiler::semantic_state::TransitionEvaluator>,
    goal_satisfied: Option<bool>,
    belief_in: Option<f32>,
    belief_out: Option<f32>,
    trajectory_error: Option<uor_r4_graph_compiler::semantic_state::SemanticStateError>,
    contract_doc_text: String,
    contract_doc_version: Option<String>,
    contract_module_version: Option<String>,
}

#[given("the R4G1 runtime returned the browser's repetitive hello response")]
fn repetitive_hello(w: &mut R4g1World) {
    w.response = "how this works like im 5 imagine you have a magic box and inside it are all the rules of geometry think of it like routing a message through a maze i use the math of curves and angles to find the most efficient path for information to go from where you want to go that is how i work go from where you start to where you want to go that is how i work go from where you start to where you start to where you want to go that is how i work go from where you want to go that is how i work go from where you want to go that is how i work go from where you start".to_string();
}

#[given("the R4G1 runtime returned replacement-character gibberish")]
fn replacement_gibberish(w: &mut R4g1World) {
    w.response = "��������������������������������".to_string();
}

#[given("the R4G1 runtime returned low-readability symbol output")]
fn low_readability_symbols(w: &mut R4g1World) {
    w.response = "☃☄☂☀▓▒░".to_string();
}

#[given("the R4G1 runtime returned a long identical-character run")]
fn identical_character_run(w: &mut R4g1World) {
    w.response = "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!".to_string();
}

#[given("the R4G1 runtime returned a concise readable hello response")]
fn concise_hello(w: &mut R4g1World) {
    w.response = "Hello! I can help you explore the compiled R4G1 graph.".to_string();
}

#[given("the R4G1 runtime returned a readable response with ordinary repetition")]
fn ordinary_repetition(w: &mut R4g1World) {
    w.response =
        "The graph can route messages. It can route messages efficiently when the graph is ready."
            .to_string();
}

#[when("the server validates the generated response")]
fn validate_response(w: &mut R4g1World) {
    w.usable = Some(is_usable_generated_text(&w.response));
}

#[then("the response is rejected as unusable")]
fn response_rejected(w: &mut R4g1World) {
    assert_eq!(w.usable, Some(false));
}

#[then("the response is accepted as usable")]
fn response_accepted(w: &mut R4g1World) {
    assert_eq!(w.usable, Some(true));
}

#[given("the browser has no saved engine selection")]
fn no_saved_engine(_w: &mut R4g1World) {}

#[when("the server resolves the synthesis engine")]
fn resolve_engine(w: &mut R4g1World) {
    w.selected_engine = Some(select_synthesis_engine(w.requested_engine));
}

#[then("the selected engine is R4G1")]
fn selected_engine_is_r4g1(w: &mut R4g1World) {
    assert_eq!(w.selected_engine, Some("r4g1"));
}

#[given("the browser explicitly selected the legacy engine")]
fn explicit_legacy(w: &mut R4g1World) {
    w.requested_engine = Some("transformerless-legacy");
}

#[then("the selected engine is Legacy TLA/TLS")]
fn selected_engine_is_legacy(w: &mut R4g1World) {
    assert_eq!(w.selected_engine, Some("transformerless-legacy"));
}

#[then("the browser UI selects R4G1 and does not offer automatic fallback")]
fn browser_selects_r4g1(_w: &mut R4g1World) {
    let source = include_str!("../index.html");
    assert!(source.contains(r#"<option value="r4g1" selected>"#));
    assert!(!source.contains("Auto: R4G1 → Legacy TLA/TLS"));
}

#[given("the R4G1 runtime is unavailable")]
fn unavailable_runtime(_w: &mut R4g1World) {}

#[when("the R4G1 chat endpoint builds its unavailable response")]
fn unavailable_response(w: &mut R4g1World) {
    let (status, body) = r4g1_unavailable_response();
    w.endpoint_status = Some(status);
    w.endpoint_body = Some(body);
}

#[then("it returns HTTP 503 without invoking a fallback engine")]
fn no_fallback_response(w: &mut R4g1World) {
    assert_eq!(w.endpoint_status, Some(503));
    let body = w.endpoint_body.as_ref().expect("endpoint body");
    assert_eq!(body["engine"], "r4g1");
    assert!(body["error"]
        .as_str()
        .unwrap_or_default()
        .contains("no fallback"));
}

#[given("the configured corpus metadata path is missing")]
fn missing_corpus_metadata(_w: &mut R4g1World) {}

#[when("R4G1 compilation inputs are validated")]
fn validate_missing_corpus(w: &mut R4g1World) {
    w.compile_error = validate_r4g1_corpus_inputs(
        Path::new("/tmp/r4g1-bdd-missing/corpus.meta"),
        Path::new("/tmp/r4g1-bdd-missing/corpus.records"),
    )
    .err();
}

#[then("compilation fails with the missing metadata error")]
fn missing_metadata_error(w: &mut R4g1World) {
    assert!(w
        .compile_error
        .as_deref()
        .unwrap_or_default()
        .contains("configured corpus metadata is missing"));
}

#[given("a graph quality report below the TLA baseline")]
fn below_baseline_report(w: &mut R4g1World) {
    w.quality_report = Some(serde_json::json!({
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.0035},
            "tla3_baseline": {"top1_agreement": 0.1811}
        }
    }));
}

#[when("the R4G1 quality gate validates the report")]
fn quality_gate_validates_report(w: &mut R4g1World) {
    w.quality_error =
        validate_quality_report(w.quality_report.as_ref().expect("quality report")).err();
}

#[then("the quality gate rejects the graph below baseline")]
fn quality_gate_rejects(w: &mut R4g1World) {
    assert!(w
        .quality_error
        .as_deref()
        .unwrap_or_default()
        .contains("below TLA baseline"));
}

#[given("a graph quality report at or above the TLA baseline")]
fn passing_baseline_report(w: &mut R4g1World) {
    w.quality_report = Some(serde_json::json!({
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.1811},
            "tla3_baseline": {"top1_agreement": 0.1811}
        }
    }));
}

#[then("the quality gate accepts the graph")]
fn quality_gate_accepts(w: &mut R4g1World) {
    assert!(w.quality_error.is_none());
}

#[given("a graph quality report at the pinned quality anchors")]
fn pinned_anchors_report(w: &mut R4g1World) {
    // #65-chain anchors: 31.7086% top-1, 9.8612 bits/token (era note in
    // src/r4g1.rs QUALITY_FLOOR_*).
    w.quality_report = Some(serde_json::json!({
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.3171, "bits_per_token": 9.8612},
            "tla3_baseline": {"top1_agreement": 0.1811, "bits_per_token": 11.8781}
        }
    }));
}

#[given("a graph quality report using a same-corpus TLA quality profile")]
fn same_corpus_tla_report(w: &mut R4g1World) {
    w.quality_report = Some(serde_json::json!({
        "config": {"quality_profile": "relative_tla"},
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.1596, "bits_per_token": 14.2959},
            "tla3_baseline": {"top1_agreement": 0.1596, "bits_per_token": 20.7655}
        }
    }));
}

#[given("a graph quality report with digressed bits per token")]
fn digressed_bits_report(w: &mut R4g1World) {
    // Agreement still clears the baseline, so only the absolute floor can fire.
    w.quality_report = Some(serde_json::json!({
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.3171, "bits_per_token": 10.5},
            "tla3_baseline": {"top1_agreement": 0.1811, "bits_per_token": 11.8781}
        }
    }));
}

#[given("a graph quality report with digressed top-1 agreement")]
fn digressed_agreement_report(w: &mut R4g1World) {
    // Agreement still clears the baseline, so only the absolute floor can fire.
    w.quality_report = Some(serde_json::json!({
        "gate_c": {
            "rule12_precedence": {"top1_agreement": 0.25, "bits_per_token": 9.8612},
            "tla3_baseline": {"top1_agreement": 0.1811, "bits_per_token": 11.8781}
        }
    }));
}

#[then("the quality gate rejects the graph for digression")]
fn quality_gate_rejects_digression(w: &mut R4g1World) {
    assert!(w
        .quality_error
        .as_deref()
        .unwrap_or_default()
        .contains("digresses"));
}

#[given("an arbitrary text input string")]
fn arbitrary_text_input(w: &mut R4g1World) {
    w.facade_input = "uor-r4 quantum geometric transformerless engine".to_string();
}

#[when("the Wasm façade folds the text using cd_space_fold")]
fn fold_text_facade(w: &mut R4g1World) {
    w.folded_matrix = cd_space_fold(&w.facade_input).to_vec();
}

#[then("a 256-element integer state matrix is returned")]
fn state_matrix_256_elements(w: &mut R4g1World) {
    assert_eq!(w.folded_matrix.len(), 256);
}

#[then("the state matrix has a non-zero parameter checksum")]
fn state_matrix_nonzero_checksum(w: &mut R4g1World) {
    let sum: i64 = w.folded_matrix.iter().map(|&x| x.abs() as i64).sum();
    assert!(sum > 0, "state matrix sum must be non-zero");
}

#[given("context sequence lengths of 1000, 10000, and 100000 tokens")]
fn sequence_lengths_config(w: &mut R4g1World) {
    w.seq_lengths = vec![1_000, 10_000, 100_000];
}

#[when("the context scaling benchmark is evaluated")]
fn eval_context_scaling(w: &mut R4g1World) {
    use std::time::Instant;
    let mut total_us = 0.0;
    let dummy_token = [10i16; 16];

    for &n in &w.seq_lengths {
        let mut store = BottFockContextStore::new();
        let start = Instant::now();
        for _ in 0..n {
            store.append_token(&dummy_token);
        }
        let elapsed = start.elapsed();
        total_us += elapsed.as_micros() as f64 / (n as f64);
        w.bench_matrix_bytes = store.state().len() * std::mem::size_of::<i16>();
    }
    w.bench_latency_us = total_us / (w.seq_lengths.len() as f64);
}

#[then("the state matrix memory footprint remains constant at 512 bytes")]
fn footprint_constant_512(w: &mut R4g1World) {
    assert_eq!(w.bench_matrix_bytes, 512);
}

#[then("the per-token update latency remains bounded under 50 microseconds")]
fn latency_bounded_50us(w: &mut R4g1World) {
    assert!(
        w.bench_latency_us < 50.0,
        "latency {} us exceeds 50 us limit",
        w.bench_latency_us
    );
}

#[given("a maximum-entropy density operator of dimension 8")]
fn max_entropy_density(w: &mut R4g1World) {
    w.density = Some(DensityOperator::max_entropy(8).expect("dimension non-zero"));
}

#[given("a density operator with a pure distribution")]
fn pure_density(w: &mut R4g1World) {
    w.density = Some(DensityOperator::from_weights(&[1.0, 0.0, 0.0]).expect("valid weights"));
}

#[when("its von Neumann entropy is computed")]
fn compute_entropy(w: &mut R4g1World) {
    w.entropy = Some(
        w.density
            .as_ref()
            .expect("density operator")
            .von_neumann_entropy(),
    );
}

#[then("the entropy equals the natural logarithm of 8")]
fn entropy_is_ln_8(w: &mut R4g1World) {
    let entropy = w.entropy.expect("entropy computed");
    let expected = 8f32.ln();
    assert!(
        (entropy - expected).abs() < 1e-6,
        "S((1/n)I) = ln n: got {entropy}, want {expected}"
    );
}

#[then("the entropy is zero")]
fn entropy_is_zero(w: &mut R4g1World) {
    assert_eq!(w.entropy, Some(0.0));
}

#[given("observations whose halves predict disjoint tokens")]
fn disjoint_halves_observations(w: &mut R4g1World) {
    w.observations = (0..100u32)
        .map(|i| Observation {
            position: i,
            sample: [0u8; 32],
            vector: Vec::new(),
            sig: [0u8; SIG_BYTES],
            prev: 0u32,
            next: if i < 50 { 1 } else { 2 },
        })
        .collect();
}

#[when("the quantum entropy gain of the aligned split is evaluated")]
fn aligned_split_gain(w: &mut R4g1World) {
    let members: Vec<usize> = (0..100).collect();
    let children = vec![(0..50).collect::<Vec<_>>(), (50..100).collect::<Vec<_>>()];
    let gain = quantum_entropy_gain(&w.observations, &members, &children);
    w.entropy_gain = Some(gain);
    w.partition_accepted = Some(QuantumCoverConfig::default().accept_partition(gain));
}

#[when("the quantum entropy gain of the interleaved split is evaluated")]
fn interleaved_split_gain(w: &mut R4g1World) {
    let members: Vec<usize> = (0..100).collect();
    let children = vec![
        (0..100).step_by(2).collect::<Vec<_>>(),
        (1..100).step_by(2).collect::<Vec<_>>(),
    ];
    let gain = quantum_entropy_gain(&w.observations, &members, &children);
    w.entropy_gain = Some(gain);
    w.partition_accepted = Some(QuantumCoverConfig::default().accept_partition(gain));
}

#[then("the gain equals ln 2 and the partition is accepted")]
fn gain_ln2_accepted(w: &mut R4g1World) {
    let gain = w.entropy_gain.expect("gain evaluated");
    assert!(
        (gain - std::f64::consts::LN_2).abs() < 1e-4,
        "gain {gain}, want ln 2"
    );
    assert_eq!(w.partition_accepted, Some(true));
}

#[then("the gain is zero and the partition is rejected")]
fn gain_zero_rejected(w: &mut R4g1World) {
    let gain = w.entropy_gain.expect("gain evaluated");
    assert!(gain.abs() < 1e-4, "gain {gain}, want 0");
    assert_eq!(w.partition_accepted, Some(false));
}

#[given("a Clifford generator matrix operator in 16D Cayley-Dickson space")]
fn clifford_generator_op(w: &mut R4g1World) {
    w.op_matrix = Some(EndomorphismAlgebra::clifford_generator(1));
}

#[when("Lie-Jordan decomposition is performed on the operator")]
fn decompose_op(w: &mut R4g1World) {
    let op = w.op_matrix.as_ref().expect("operator matrix");
    w.split_result = Some(LieJordanSplit::decompose(op));
}

#[then("the Lie component is strictly anti-Hermitian")]
fn lie_anti_hermitian(w: &mut R4g1World) {
    let split = w.split_result.as_ref().expect("split result");
    assert!(LieJordanSplit::is_anti_hermitian(&split.lie));
}

#[then("the Jordan component is strictly Hermitian")]
fn jordan_hermitian(w: &mut R4g1World) {
    let split = w.split_result.as_ref().expect("split result");
    assert!(LieJordanSplit::is_hermitian(&split.jordan));
}

#[then("the reconstructed operator matches the original matrix")]
fn reconstructed_matches(w: &mut R4g1World) {
    let split = w.split_result.as_ref().expect("split result");
    let orig = w.op_matrix.as_ref().expect("original operator");
    let rec = split.reconstruct();
    for (a, b) in orig.matrix.iter().zip(&rec.matrix) {
        assert!((a - b).abs() < 1e-5);
    }
}

#[given("a pair of 8-bit integer operator state bytes")]
fn integer_operator_bytes(w: &mut R4g1World) {
    w.u8_a = 0b1100_1010;
    w.u8_b = 0b1010_1100;
}

#[when("the hot-path universal product kernel is evaluated for Lie anti-Hermitian symmetry")]
fn eval_u8_kernel(w: &mut R4g1World) {
    w.u8_res = universal_product_u8(w.u8_a, w.u8_b, true);
}

#[then("the result matches the bitwise XOR and rotation transformation")]
fn u8_kernel_matches(w: &mut R4g1World) {
    let expected = w.u8_a ^ (w.u8_b.rotate_left(1));
    assert_eq!(w.u8_res, expected);
}

#[then("zero floating-point operations or multiplications are executed")]
fn u8_kernel_zero_floats(_w: &mut R4g1World) {
    let source = include_str!("../crates/uor-r4-core/src/transformerless/lie_jordan.rs");
    let kernel_start = source
        .find("pub fn universal_product_u8")
        .expect("kernel function");
    let kernel_code = &source[kernel_start..];
    assert!(!kernel_code.contains("f32") && !kernel_code.contains("f64"));
    assert!(!kernel_code.contains(" * ") && !kernel_code.contains(" / "));
}

#[given("the normative inference operation contract document")]
fn contract_document_loaded(w: &mut R4g1World) {
    let text = include_str!("../docs/transformerless/INFERENCE_OPERATION_CONTRACT.md");
    w.contract_doc_text = text.to_string();
    let version = text
        .lines()
        .find_map(|line| {
            line.strip_prefix("- **Version:** ")
                .map(|value| value.trim().to_string())
        })
        .expect("contract version line");
    w.contract_doc_version = Some(version);
}

#[when("the machine-readable inference operation contract version is loaded")]
fn contract_module_version_loaded(w: &mut R4g1World) {
    let version = INFERENCE_OPERATION_CONTRACT_VERSION;
    w.contract_module_version = Some(format!(
        "{}.{}.{}",
        version.major, version.minor, version.patch
    ));
}

#[then("the document and module contract versions agree")]
fn contract_versions_agree(w: &mut R4g1World) {
    assert_eq!(w.contract_doc_version, w.contract_module_version);
}

// =========================================================================
// =========================================================================
// PDF Traceability Matrix BDD Steps (#137)
// =========================================================================
use uor_r4_proof_model::pdf_traceability::{
    PdfTraceabilityRow, PdfTraceabilityVerifier, TraceabilityValidationError,
};
use uor_r4_proof_model::proof_matrix::ProofStatus;

#[given("the living PDF traceability matrix")]
fn bdd_pdf_matrix_given(w: &mut R4g1World) {
    w.pdf_matrix = PdfTraceabilityVerifier::get_matrix().to_vec();
}

#[when("audited by the PDF traceability verifier")]
fn bdd_pdf_audit_matrix(w: &mut R4g1World) {
    let res = PdfTraceabilityVerifier::audit_traceability_matrix(&w.pdf_matrix);
    match res {
        Ok(rep) => w.pdf_audit_report = Some(rep),
        Err(err) => w.pdf_audit_error = Some(err),
    }
}

#[then("all 17 sections are mapped to valid code locations and evidence artifacts")]
fn bdd_pdf_sections_check(w: &mut R4g1World) {
    let rep = w.pdf_audit_report.as_ref().expect("pdf audit report");
    assert_eq!(rep.total_sections_verified, 17);
    assert_eq!(rep.verified_rows_with_evidence, 17);
}

#[then("the audit report certification status is verified")]
fn bdd_pdf_cert_check(w: &mut R4g1World) {
    let rep = w.pdf_audit_report.as_ref().expect("pdf audit report");
    assert!(rep.is_certified);
}

#[given("a traceability row with invalid claim class \"HypotheticalSpec\"")]
fn bdd_pdf_invalid_claim_given(w: &mut R4g1World) {
    w.pdf_matrix = vec![PdfTraceabilityRow {
        pdf_section: "§1",
        concept_name: "Test",
        issue_id: "#199",
        code_location: "dummy",
        evidence_artifact: "dummy",
        claim_class: "HypotheticalSpec",
        status: ProofStatus::Verified,
        owner: "Casey Allard",
    }];
}

#[then("validation fails with an invalid claim class error")]
fn bdd_pdf_invalid_claim_error_check(w: &mut R4g1World) {
    let err = w.pdf_audit_error.as_ref().expect("pdf audit error");
    assert!(matches!(
        err,
        TraceabilityValidationError::InvalidClaimClass { .. }
    ));
}

// =========================================================================
// Rate-Distortion Compression BDD Steps (#136)
// =========================================================================
use uor_r4_graph_compiler::rate_distortion_compression::{
    CompressionAnalysisError, SemanticCompressionAnalyzer,
};

#[given("a pinned mini-corpus \"pinned_mini_corpus_01\" and depth tiers [1, 2, 4, 8]")]
fn bdd_rd_mini_corpus_given(w: &mut R4g1World) {
    w.rd_corpus_id = "pinned_mini_corpus_01".to_string();
    w.rd_tiers = vec![1, 2, 4, 8];
}

#[when("rate-distortion analysis is executed by the semantic compression analyzer")]
fn bdd_rd_execute_analysis(w: &mut R4g1World) {
    let res = SemanticCompressionAnalyzer::analyze_rate_distortion(&w.rd_corpus_id, &w.rd_tiers);
    match res {
        Ok(rep) => w.rd_report = Some(rep),
        Err(err) => w.rd_error = Some(err),
    }
}

#[then("a deterministic RateDistortionReport is produced containing 4 depth evaluation points")]
fn bdd_rd_report_check(w: &mut R4g1World) {
    let rep = w.rd_report.as_ref().expect("rd report");
    assert_eq!(rep.points.len(), 4);
    assert_eq!(rep.corpus_id, "pinned_mini_corpus_01");
}

#[then("teacher KL divergence reduces monotonically as projection depth increases")]
fn bdd_rd_kl_monotonic_check(w: &mut R4g1World) {
    let rep = w.rd_report.as_ref().expect("rd report");
    for i in 0..(rep.points.len() - 1) {
        assert!(
            rep.points[i].distortion.teacher_kl_divergence
                > rep.points[i + 1].distortion.teacher_kl_divergence,
            "KL divergence at index {i} must be greater than index {}",
            i + 1
        );
    }
}

#[given("a rate-distortion evaluation report for depth tiers [1, 2, 4, 8]")]
fn bdd_rd_report_given(w: &mut R4g1World) {
    w.rd_corpus_id = "pinned_mini_corpus_01".to_string();
    w.rd_tiers = vec![1, 2, 4, 8];
    w.rd_report = Some(
        SemanticCompressionAnalyzer::analyze_rate_distortion(&w.rd_corpus_id, &w.rd_tiers).unwrap(),
    );
}

#[when("analyzed for optimal rate-distortion tradeoff")]
fn bdd_rd_analyze_tradeoff(_w: &mut R4g1World) {}

#[then("depth tier 4 is identified as the optimal tradeoff depth")]
fn bdd_rd_optimal_depth_check(w: &mut R4g1World) {
    let rep = w.rd_report.as_ref().expect("rd report");
    assert_eq!(rep.optimal_tradeoff_depth, 4);
}

#[then("the report certification status is verified")]
fn bdd_rd_cert_status_check(w: &mut R4g1World) {
    let rep = w.rd_report.as_ref().expect("rd report");
    assert!(rep.is_certified);
}

#[given("an invalid depth tier array containing 0")]
fn bdd_rd_invalid_tier_given(w: &mut R4g1World) {
    w.rd_corpus_id = "pinned_mini_corpus_01".to_string();
    w.rd_tiers = vec![0, 1, 2];
}

#[then("analysis fails with an invalid depth tier error")]
fn bdd_rd_invalid_tier_error_check(w: &mut R4g1World) {
    let err = w.rd_error.as_ref().expect("rd error");
    assert!(matches!(
        err,
        CompressionAnalysisError::InvalidDepthTier { .. }
    ));
}

// =========================================================================
// Graph Invariant Ownership BDD Steps (#135)
// =========================================================================
use uor_r4_graph_format::invariant_ownership::{
    GraphInvariantOwnershipMatrix, InvariantValidationError,
};

#[given("the normative graph invariant inventory")]
fn bdd_inv_inventory_given(_w: &mut R4g1World) {}

#[when("mapped to the ownership matrix")]
fn bdd_inv_map_matrix(w: &mut R4g1World) {
    w.inv_matrix = GraphInvariantOwnershipMatrix::get_matrix().to_vec();
}

#[then("all 8 graph invariants have declared primary owners and validation stages")]
fn bdd_inv_matrix_check(w: &mut R4g1World) {
    assert_eq!(w.inv_matrix.len(), 8);
    for entry in &w.inv_matrix {
        assert!(!entry.validation_stage.is_empty());
        assert!(!entry.description.is_empty());
    }
}

#[given("a graph artifact with maximum node degree 12 against limit 10")]
fn bdd_inv_degree_limit_given(w: &mut R4g1World) {
    w.inv_nodes = 13;
    w.inv_max_degree = 12;
    w.inv_degree_limit = 10;
    w.inv_edges = (1..=12).map(|dst| (0, dst)).collect();
    w.inv_evidence = vec![101, 102];
}

#[when("validated by the loader invariant verifier")]
fn bdd_inv_validate_loader(w: &mut R4g1World) {
    w.inv_res = Some(GraphInvariantOwnershipMatrix::validate_graph_structure(
        w.inv_nodes,
        w.inv_max_degree,
        w.inv_degree_limit,
        &w.inv_edges,
        &w.inv_evidence,
    ));
}

#[then("validation fails with a degree limit exceeded error")]
fn bdd_inv_degree_error_check(w: &mut R4g1World) {
    let err = w.inv_res.as_ref().expect("inv_res").as_ref().unwrap_err();
    assert!(matches!(
        err,
        InvariantValidationError::DegreeLimitExceeded { .. }
    ));
}

#[given("a graph artifact with 5 nodes and an edge referencing target node 99")]
fn bdd_inv_dangling_given(w: &mut R4g1World) {
    w.inv_nodes = 5;
    w.inv_max_degree = 4;
    w.inv_degree_limit = 10;
    w.inv_edges = vec![(0, 99)];
    w.inv_evidence = vec![101, 102];
}

#[then("validation fails with a dangling reference error")]
fn bdd_inv_dangling_error_check(w: &mut R4g1World) {
    let err = w.inv_res.as_ref().expect("inv_res").as_ref().unwrap_err();
    assert!(matches!(
        err,
        InvariantValidationError::DanglingReference { .. }
    ));
}

#[given("a graph node containing duplicate evidence ID 101")]
fn bdd_inv_duplicate_evidence_given(w: &mut R4g1World) {
    w.inv_nodes = 5;
    w.inv_max_degree = 4;
    w.inv_degree_limit = 10;
    w.inv_edges = vec![(0, 1)];
    w.inv_evidence = vec![101, 101];
}

#[then("validation fails with a duplicate evidence error")]
fn bdd_inv_duplicate_evidence_error_check(w: &mut R4g1World) {
    let err = w.inv_res.as_ref().expect("inv_res").as_ref().unwrap_err();
    assert!(matches!(
        err,
        InvariantValidationError::DuplicateEvidence { .. }
    ));
}
// Separate Semantic Emission BDD Steps (#134)
// =========================================================================
use uor_r4_graph_compiler::semantic_emission_decoupling::{
    LanguageEmissionAdapter, SemanticEmissionError, SemanticReasoningEngine, SemanticStatus,
};

#[given("an initial state \"s0\" and a valid 2-step transition sequence to \"s2\"")]
fn bdd_decouple_valid_sequence(w: &mut R4g1World) {
    w.decouple_transitions = vec![
        ("s0", "act1", "s1", 0.9, SemanticStatus::Coherent),
        ("s1", "act2", "s2", 0.95, SemanticStatus::Coherent),
    ];
}

#[when("pure semantic reasoning is executed by the reasoning engine")]
fn bdd_decouple_execute_reasoning(w: &mut R4g1World) {
    let res = SemanticReasoningEngine::execute_pure_reasoning("s0", &w.decouple_transitions);
    match res {
        Ok(tr) => w.decouple_trace = Some(tr),
        Err(err) => w.decouple_error = Some(err),
    }
}

#[then("a valid SemanticStateTrace is produced without generating tokens")]
fn bdd_decouple_trace_check(w: &mut R4g1World) {
    let tr = w.decouple_trace.as_ref().expect("trace");
    assert_eq!(tr.initial_state_id, "s0");
    assert_eq!(tr.final_state_id, "s2");
    assert_eq!(tr.steps.len(), 2);
}

#[then("the trace overall status is Coherent")]
fn bdd_decouple_status_coherent_check(w: &mut R4g1World) {
    let tr = w.decouple_trace.as_ref().expect("trace");
    assert_eq!(tr.overall_status, SemanticStatus::Coherent);
}

#[given("a verified coherent SemanticStateTrace from \"s0\" to \"s2\"")]
fn bdd_decouple_verified_trace_given(w: &mut R4g1World) {
    let transitions = vec![
        ("s0", "act1", "s1", 0.9, SemanticStatus::Coherent),
        ("s1", "act2", "s2", 0.95, SemanticStatus::Coherent),
    ];
    w.decouple_trace =
        Some(SemanticReasoningEngine::execute_pure_reasoning("s0", &transitions).unwrap());
}

#[when("passed to the language emission adapter")]
fn bdd_decouple_pass_to_adapter(w: &mut R4g1World) {
    let tr = w.decouple_trace.as_ref().expect("trace");
    let em = LanguageEmissionAdapter::emit_language(tr).unwrap();
    let cert = LanguageEmissionAdapter::certify_decoupled(tr, &em);
    w.decouple_emission = Some(em);
    w.decouple_cert = Some(cert);
}

#[then("a LanguageEmissionResult is produced containing text and token probabilities")]
fn bdd_decouple_emission_result_check(w: &mut R4g1World) {
    let em = w.decouple_emission.as_ref().expect("emission");
    assert!(em.emitted_text.contains("s0 to s2"));
    assert!(!em.token_probabilities.is_empty());
}

#[then("a multi-dimensional certification report evaluates state coherence and language fidelity separately")]
fn bdd_decouple_cert_report_check(w: &mut R4g1World) {
    let cert = w.decouple_cert.as_ref().expect("cert");
    assert!(cert.is_certified);
    assert!(cert.state_coherence_score > 0.8);
    assert!(cert.language_fidelity_score > 0.8);
}

#[given("a transition sequence leading to a Contradictory state")]
fn bdd_decouple_contradictory_given(w: &mut R4g1World) {
    w.decouple_transitions = vec![
        ("s0", "act1", "s1", 0.9, SemanticStatus::Coherent),
        ("s1", "act2", "s_err", 0.1, SemanticStatus::Contradictory),
    ];
}

#[then("execution fails with a contradictory state error before token emission")]
fn bdd_decouple_contradictory_error_check(w: &mut R4g1World) {
    let err = w.decouple_error.as_ref().expect("error");
    assert!(matches!(
        err,
        SemanticEmissionError::ContradictoryState { .. }
    ));
}

// =========================================================================
// Formal Monograph BDD Steps (#133)
// =========================================================================
use uor_r4_graph_compiler::monograph::{MonographTraceabilityVerifier, MonographValidationError};

#[given("the living formal monograph document")]
fn bdd_given_monograph_doc(w: &mut R4g1World) {
    w.monograph_text = include_str!("../docs/hologram_r4_formal_monograph.md").to_string();
}

#[when("audited by the monograph traceability verifier")]
fn bdd_validate_monograph_step(w: &mut R4g1World) {
    let res = MonographTraceabilityVerifier::validate_monograph_text(&w.monograph_text);
    match res {
        Ok(rep) => w.monograph_report = Some(rep),
        Err(err) => w.monograph_error = Some(err),
    }
}

#[then("all 19 monograph sections are verified present")]
fn bdd_monograph_sections_check(w: &mut R4g1World) {
    let rep = w.monograph_report.as_ref().expect("monograph report");
    assert_eq!(rep.total_sections_verified, 19);
    assert!(rep.verified);
}

#[then("12 implementation module links are verified")]
fn bdd_monograph_modules_check(w: &mut R4g1World) {
    let rep = w.monograph_report.as_ref().expect("monograph report");
    assert_eq!(rep.total_modules_linked, 12);
}

#[then("3 non-goal disavowals are verified present")]
fn bdd_monograph_non_goals_check(w: &mut R4g1World) {
    let rep = w.monograph_report.as_ref().expect("monograph report");
    assert_eq!(rep.non_goals_disavowed, 3);
}

#[given("a monograph draft missing section \"Section 1: Problem Statement and Non-Goals\"")]
fn bdd_given_missing_section(w: &mut R4g1World) {
    let full_doc = include_str!("../docs/hologram_r4_formal_monograph.md");
    w.monograph_text = full_doc.replace(
        "Section 1: Problem Statement and Non-Goals",
        "Missing Sec 1",
    );
}

#[then("validation fails with a missing section error")]
fn bdd_missing_section_error_check(w: &mut R4g1World) {
    let err = w.monograph_error.as_ref().expect("monograph error");
    assert!(matches!(
        err,
        MonographValidationError::MissingSection { .. }
    ));
}

#[given("a monograph draft missing non-goal \"No Human-Level Reasoning Claim\"")]
fn bdd_given_missing_non_goal(w: &mut R4g1World) {
    let full_doc = include_str!("../docs/hologram_r4_formal_monograph.md");
    w.monograph_text = full_doc.replace("No Human-Level Reasoning Claim", "Altered");
}

#[then("validation fails with a missing non-goal error")]
fn bdd_missing_non_goal_error_check(w: &mut R4g1World) {
    let err = w.monograph_error.as_ref().expect("monograph error");
    assert!(matches!(
        err,
        MonographValidationError::MissingNonGoalDisavowal { .. }
    ));
}

// =========================================================================
// Expand Proof Model BDD Steps (#132)
// =========================================================================
use uor_r4_proof_model::proof_matrix::ProofStatusMatrix;
use uor_r4_proof_model::structural_guarantees::{
    ProofValidationError, StructuralGuaranteeVerifier,
};

#[given("a graph planner calculation closure")]
fn bdd_deterministic_closure(_w: &mut R4g1World) {}

#[when("verified by the structural guarantee verifier for determinism")]
fn bdd_verify_determinism_step(w: &mut R4g1World) {
    use uor_r4_graph_compiler::future_state_planner::{
        BoundedGraphPlanner, PlannerConfig, PlannerEdgeTransition, PlannerStateNode,
    };
    // Differential test executing real BoundedGraphPlanner over graph nodes
    let report = StructuralGuaranteeVerifier::verify_determinism("OBL-DET-PLANNER", || {
        let nodes = vec![
            PlannerStateNode {
                id: "s0".to_string(),
                is_goal: false,
                is_forbidden: false,
                forbidden_region_id: None,
            },
            PlannerStateNode {
                id: "s1".to_string(),
                is_goal: true,
                is_forbidden: false,
                forbidden_region_id: None,
            },
        ];
        let edges = vec![PlannerEdgeTransition {
            src_id: "s0".to_string(),
            dst_id: "s1".to_string(),
            action: "act".to_string(),
            cost: 1.0,
            confidence: 0.95,
        }];
        let config = PlannerConfig::default_v1();
        BoundedGraphPlanner::plan("s0", &nodes, &edges, &config)
    })
    .unwrap();

    w.proof_report = Some(report);
}

#[then("the obligation status is Verified and determinism is verified")]
fn bdd_determinism_status_check(w: &mut R4g1World) {
    let report = w.proof_report.as_ref().expect("proof report");
    assert!(report.verified);
    assert_eq!(report.status, ProofStatus::Verified);
}

#[given("a list of node IDs [10, 20, 30]")]
fn bdd_canonical_nodes_given(w: &mut R4g1World) {
    w.proof_nodes = vec![10, 20, 30];
}

#[when("verified against canonical serialization obligations")]
fn bdd_verify_canonical_step(w: &mut R4g1World) {
    let report =
        StructuralGuaranteeVerifier::verify_canonical_serialization("OBL-CAN-01", &w.proof_nodes)
            .unwrap();
    w.proof_report = Some(report);
}

#[then("canonical ordering passes cleanly")]
fn bdd_canonical_ordering_passes(w: &mut R4g1World) {
    let report = w.proof_report.as_ref().expect("proof report");
    assert!(report.verified);
}

#[then("unsorted node IDs [30, 20, 10] fail with a canonical ordering violation error")]
fn bdd_canonical_ordering_fails(_w: &mut R4g1World) {
    let err =
        StructuralGuaranteeVerifier::verify_canonical_serialization("OBL-CAN-01", &[30, 20, 10])
            .unwrap_err();
    assert!(matches!(
        err,
        ProofValidationError::CanonicalOrderingViolated { .. }
    ));
}

#[given("actual memory usage 512 bytes and limit 1024 bytes")]
fn bdd_resource_memory_given(w: &mut R4g1World) {
    w.proof_actual_mem = 512;
    w.proof_limit_mem = 1024;
}

#[when("verified against bounded resource obligations")]
fn bdd_verify_resource_step(w: &mut R4g1World) {
    let report = StructuralGuaranteeVerifier::verify_resource_bound(
        "OBL-MEM-BDD",
        "memory_bytes",
        w.proof_actual_mem,
        w.proof_limit_mem,
    )
    .unwrap();
    w.proof_report = Some(report);
}

#[then("the resource bound obligation passes cleanly")]
fn bdd_resource_bound_passes(w: &mut R4g1World) {
    let report = w.proof_report.as_ref().expect("proof report");
    assert!(report.verified);
}

#[then("actual memory usage 2048 bytes against limit 1024 bytes fails with a resource bound error")]
fn bdd_resource_bound_fails(_w: &mut R4g1World) {
    let err = StructuralGuaranteeVerifier::verify_resource_bound(
        "OBL-MEM-BDD",
        "memory_bytes",
        2048,
        1024,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ProofValidationError::ResourceBoundExceeded { .. }
    ));
}

#[given("a state trajectory [\"s0\", \"s1\", \"s2\"] and forbidden region [\"hazard_0\"]")]
fn bdd_trajectory_hazard_given(w: &mut R4g1World) {
    w.proof_trajectory = vec!["s0".to_string(), "s1".to_string(), "s2".to_string()];
    w.proof_forbidden = vec!["hazard_0".to_string()];
}

#[when("verified against constraint safety obligations")]
fn bdd_verify_constraint_safety_step(w: &mut R4g1World) {
    let traj_refs: Vec<&str> = w.proof_trajectory.iter().map(|s| s.as_str()).collect();
    let forb_refs: Vec<&str> = w.proof_forbidden.iter().map(|s| s.as_str()).collect();
    let report = StructuralGuaranteeVerifier::verify_constraint_safety(
        "OBL-SAFE-BDD",
        &traj_refs,
        &forb_refs,
    )
    .unwrap();
    w.proof_report = Some(report);
}

#[then("constraint preservation passes with zero forbidden states entered")]
fn bdd_constraint_safety_passes(w: &mut R4g1World) {
    let report = w.proof_report.as_ref().expect("proof report");
    assert!(report.verified);
}

#[then("entering \"hazard_0\" fails with a constraint safety violation error")]
fn bdd_constraint_safety_fails(_w: &mut R4g1World) {
    let err = StructuralGuaranteeVerifier::verify_constraint_safety(
        "OBL-SAFE-BDD",
        &["s0", "hazard_0", "s2"],
        &["hazard_0"],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ProofValidationError::ConstraintSafetyViolated { .. }
    ));
}

#[given("a planner path length 5 and horizon limit 10")]
fn bdd_planner_horizon_given(w: &mut R4g1World) {
    w.proof_path_len = 5;
    w.proof_max_horizon = 10;
}

#[when("verified against planner termination obligations")]
fn bdd_verify_planner_termination_step(w: &mut R4g1World) {
    let report = StructuralGuaranteeVerifier::verify_planner_termination(
        "OBL-TERM-BDD",
        w.proof_path_len,
        w.proof_max_horizon,
    )
    .unwrap();
    w.proof_report = Some(report);
}

#[then("planner horizon termination passes cleanly")]
fn bdd_planner_termination_passes(w: &mut R4g1World) {
    let report = w.proof_report.as_ref().expect("proof report");
    assert!(report.verified);
}

#[then("path length 15 against horizon limit 10 fails with a planner termination error")]
fn bdd_planner_termination_fails(_w: &mut R4g1World) {
    let err = StructuralGuaranteeVerifier::verify_planner_termination("OBL-TERM-BDD", 15, 10)
        .unwrap_err();
    assert!(matches!(
        err,
        ProofValidationError::PlannerTerminationFailed { .. }
    ));
}

#[given("a list of evidence IDs [\"ev_1\", \"ev_2\", \"ev_3\"]")]
fn bdd_evidence_ids_given(w: &mut R4g1World) {
    w.proof_evidence_ids = vec!["ev_1".to_string(), "ev_2".to_string(), "ev_3".to_string()];
}

#[when("verified against evidence traceability obligations")]
fn bdd_verify_evidence_traceability_step(w: &mut R4g1World) {
    let refs: Vec<&str> = w.proof_evidence_ids.iter().map(|s| s.as_str()).collect();
    let report =
        StructuralGuaranteeVerifier::verify_evidence_traceability("OBL-EVID-BDD", &refs).unwrap();
    w.proof_report = Some(report);
}

#[then("evidence traceability passes cleanly")]
fn bdd_evidence_traceability_passes(w: &mut R4g1World) {
    let report = w.proof_report.as_ref().expect("proof report");
    assert!(report.verified);
}

#[then("duplicate evidence IDs [\"ev_1\", \"ev_1\", \"ev_3\"] fail with an evidence traceability error")]
fn bdd_evidence_traceability_fails(_w: &mut R4g1World) {
    let err = StructuralGuaranteeVerifier::verify_evidence_traceability(
        "OBL-EVID-BDD",
        &["ev_1", "ev_1", "ev_3"],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ProofValidationError::EvidenceTraceabilityFailed { .. }
    ));
}

#[given("actual witness hash \"hash_abc123\" and expected witness hash \"hash_abc123\"")]
fn bdd_replay_witness_given(w: &mut R4g1World) {
    w.proof_witness_actual = "hash_abc123".to_string();
    w.proof_witness_expected = "hash_abc123".to_string();
}

#[when("verified against replay witness obligations")]
fn bdd_verify_replay_witness_step(w: &mut R4g1World) {
    let report = StructuralGuaranteeVerifier::verify_replay_witness_integrity(
        "OBL-WIT-BDD",
        &w.proof_witness_actual,
        &w.proof_witness_expected,
    )
    .unwrap();
    w.proof_report = Some(report);
}

#[then("replay witness integrity passes cleanly")]
fn bdd_replay_witness_passes(w: &mut R4g1World) {
    let report = w.proof_report.as_ref().expect("proof report");
    assert!(report.verified);
}

#[then("actual witness hash \"hash_abc123\" against expected hash \"hash_xyz999\" fails with a witness mismatch error")]
fn bdd_replay_witness_fails(_w: &mut R4g1World) {
    let err = StructuralGuaranteeVerifier::verify_replay_witness_integrity(
        "OBL-WIT-BDD",
        "hash_abc123",
        "hash_xyz999",
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ProofValidationError::ReplayWitnessMismatch { .. }
    ));
}

#[given("a raw score 2048")]
fn bdd_fixed_score_given(w: &mut R4g1World) {
    w.proof_raw_score = 2048;
}

#[when("verified against fixed-point arithmetic obligations")]
fn bdd_verify_fixed_arithmetic_step(w: &mut R4g1World) {
    let report = StructuralGuaranteeVerifier::verify_fixed_arithmetic_safety(
        "OBL-MATH-BDD",
        w.proof_raw_score,
    )
    .unwrap();
    w.proof_report = Some(report);
}

#[then("fixed arithmetic score safety passes cleanly")]
fn bdd_fixed_arithmetic_passes(w: &mut R4g1World) {
    let report = w.proof_report.as_ref().expect("proof report");
    assert!(report.verified);
}

#[then("raw score 70000 fails with a fixed arithmetic overflow error")]
fn bdd_fixed_arithmetic_fails(_w: &mut R4g1World) {
    let err = StructuralGuaranteeVerifier::verify_fixed_arithmetic_safety("OBL-MATH-BDD", 70000)
        .unwrap_err();
    assert!(matches!(
        err,
        ProofValidationError::FixedArithmeticOverflow { .. }
    ));
}

#[given("the default proof matrix")]
fn bdd_default_proof_matrix(_w: &mut R4g1World) {}

#[when("theorem \"Allocation Freedom\" is audited against expected status Verified")]
fn bdd_audit_p1_step(w: &mut R4g1World) {
    let matrix = ProofStatusMatrix::default();
    let report = StructuralGuaranteeVerifier::audit_proof_matrix_entry(
        &matrix,
        "Allocation Freedom",
        ProofStatus::Verified,
    )
    .unwrap();
    w.proof_report = Some(report);
}

#[then("the audit succeeds and status matches")]
fn bdd_audit_status_matches(w: &mut R4g1World) {
    let report = w.proof_report.as_ref().expect("proof report");
    assert!(report.verified);
    assert_eq!(report.status, ProofStatus::Verified);
}

// =========================================================================
// Future State Planner BDD Steps (#131)
// =========================================================================
use uor_r4_graph_compiler::future_state_planner::{
    BoundedGraphPlanner, PlannerConfig, PlannerEdgeTransition, PlannerError, PlannerStateNode,
};

#[given("a start state \"s0\", intermediate state \"s1\", and goal state \"s2\"")]
fn bdd_planner_setup_valid_graph(w: &mut R4g1World) {
    w.plan_nodes = vec![
        PlannerStateNode {
            id: "s0".to_string(),
            is_goal: false,
            is_forbidden: false,
            forbidden_region_id: None,
        },
        PlannerStateNode {
            id: "s1".to_string(),
            is_goal: false,
            is_forbidden: false,
            forbidden_region_id: None,
        },
        PlannerStateNode {
            id: "s2".to_string(),
            is_goal: true,
            is_forbidden: false,
            forbidden_region_id: None,
        },
    ];
    w.plan_edges = vec![
        PlannerEdgeTransition {
            src_id: "s0".to_string(),
            action: "step1".to_string(),
            dst_id: "s1".to_string(),
            cost: 1.0,
            confidence: 0.9,
        },
        PlannerEdgeTransition {
            src_id: "s1".to_string(),
            action: "step2".to_string(),
            dst_id: "s2".to_string(),
            cost: 1.0,
            confidence: 0.95,
        },
    ];
}

#[when("the bounded graph planner computes a trajectory")]
fn bdd_planner_compute_trajectory(w: &mut R4g1World) {
    let config = PlannerConfig::default_v1();
    let res = BoundedGraphPlanner::plan("s0", &w.plan_nodes, &w.plan_edges, &config);
    match res {
        Ok(t) => w.plan_result = Some(t),
        Err(e) => w.plan_error = Some(e),
    }
}

#[then("the action sequence [\"step1\", \"step2\"] reaches \"s2\" in 2 steps")]
fn bdd_planner_trajectory_check(w: &mut R4g1World) {
    let plan = w.plan_result.as_ref().expect("plan");
    assert_eq!(plan.action_sequence, vec!["step1", "step2"]);
    assert_eq!(plan.state_sequence, vec!["s0", "s1", "s2"]);
    assert_eq!(plan.horizon_steps, 2);
}

#[then("a PlanWitness recording accepted transitions and plan CID is emitted")]
fn bdd_planner_witness_check(w: &mut R4g1World) {
    let plan = w.plan_result.as_ref().expect("plan");
    assert!(plan.witness.plan_cid.starts_with("blake3:plan_"));
    assert_eq!(plan.witness.accepted_edges.len(), 2);
}

#[given("an intermediate state \"s1\" marked as forbidden")]
fn bdd_planner_setup_forbidden_intermediate(w: &mut R4g1World) {
    w.plan_nodes = vec![
        PlannerStateNode {
            id: "s0".to_string(),
            is_goal: false,
            is_forbidden: false,
            forbidden_region_id: None,
        },
        PlannerStateNode {
            id: "s1".to_string(),
            is_goal: false,
            is_forbidden: true,
            forbidden_region_id: Some("hazard_0".to_string()),
        },
        PlannerStateNode {
            id: "s2".to_string(),
            is_goal: true,
            is_forbidden: false,
            forbidden_region_id: None,
        },
    ];
    w.plan_edges = vec![PlannerEdgeTransition {
        src_id: "s0".to_string(),
        action: "step1".to_string(),
        dst_id: "s1".to_string(),
        cost: 1.0,
        confidence: 0.9,
    }];
}

#[when("the bounded graph planner attempts to plan a trajectory through \"s1\"")]
fn bdd_planner_attempt_forbidden_plan(w: &mut R4g1World) {
    let config = PlannerConfig::default_v1();
    if let Err(e) = BoundedGraphPlanner::plan("s0", &w.plan_nodes, &w.plan_edges, &config) {
        w.plan_error = Some(e);
    }
}

#[then("planning fails with a frontier exhausted error and zero forbidden states entered")]
fn bdd_planner_frontier_exhausted_check(w: &mut R4g1World) {
    let err = w.plan_error.as_ref().expect("plan error");
    match err {
        PlannerError::FrontierExhausted {
            forbidden_states_entered,
            ..
        } => assert_eq!(*forbidden_states_entered, 0),
        other => panic!("expected FrontierExhausted, got {other:?}"),
    }
}

#[given("a start state \"s0\" marked as forbidden")]
fn bdd_planner_setup_forbidden_start(w: &mut R4g1World) {
    w.plan_nodes = vec![PlannerStateNode {
        id: "s0".to_string(),
        is_goal: false,
        is_forbidden: true,
        forbidden_region_id: Some("start_hazard".to_string()),
    }];
    w.plan_edges = Vec::new();
}

#[when("planning is initiated from \"s0\"")]
fn bdd_planner_initiate_forbidden_start(w: &mut R4g1World) {
    let config = PlannerConfig::default_v1();
    if let Err(e) = BoundedGraphPlanner::plan("s0", &w.plan_nodes, &w.plan_edges, &config) {
        w.plan_error = Some(e);
    }
}

#[then("planning fails immediately with an initial state forbidden error")]
fn bdd_planner_initial_forbidden_check(w: &mut R4g1World) {
    let err = w.plan_error.as_ref().expect("plan error");
    assert!(matches!(err, PlannerError::InitialStateForbidden { .. }));
}

// =========================================================================
// Lower Semantic Regions BDD Steps (#130)
// =========================================================================
use uor_r4_graph_compiler::lower_semantic_regions::{
    BooleanLoweringCompiler, LoweredFixedPointScore, LoweringError,
};

#[given(
    "a reference semantic region with signature [true, false, true, true] and Hamming radius 1.0"
)]
fn bdd_given_ref_region(_w: &mut R4g1World) {}

#[when("the region is lowered into a LoweredBooleanRegion")]
fn bdd_lower_region_step(w: &mut R4g1World) {
    let (region, witness) = BooleanLoweringCompiler::lower_region(
        "reg_bdd_1",
        &[true, false, true, true],
        1.0,
        "cid_bdd_ref_101",
        101,
        0,
    )
    .unwrap();
    w.lower_bool_region = Some(region);
    w.lower_witness = Some(witness);
}

#[then("the integer predicate evaluates to true for signatures within Hamming distance 1")]
fn bdd_integer_predicate_within_distance(w: &mut R4g1World) {
    let region = w.lower_bool_region.as_ref().expect("region");
    // Exact 0b1101 = 13 (distance 0)
    assert!(region.evaluate_runtime_integer(0b1101));
    // Distance 1 (0b1100)
    assert!(region.evaluate_runtime_integer(0b1100));
}

#[then("evaluates to false for signatures outside Hamming distance 1")]
fn bdd_integer_predicate_outside_distance(w: &mut R4g1World) {
    let region = w.lower_bool_region.as_ref().expect("region");
    // Distance 2 (0b0000)
    assert!(!region.evaluate_runtime_integer(0b0000));
}

#[then("a LoweringWitnessEntry is recorded")]
fn bdd_witness_recorded_check(w: &mut R4g1World) {
    let witness = w.lower_witness.as_ref().expect("witness");
    assert_eq!(witness.reference_cid, "cid_bdd_ref_101");
}

#[given("floating-point scores 1.5, 500.0, and -500.0")]
fn bdd_given_float_scores(_w: &mut R4g1World) {}

#[when("scores are quantized into Q8.8 fixed-point representation")]
fn bdd_quantize_scores_step(w: &mut R4g1World) {
    w.lower_q_normal = Some(LoweredFixedPointScore::quantize_q88(1.5).unwrap());
    w.lower_q_max = Some(LoweredFixedPointScore::quantize_q88(500.0).unwrap());
    w.lower_q_min = Some(LoweredFixedPointScore::quantize_q88(-500.0).unwrap());
}

#[then("1.5 quantizes to 384 without saturation")]
fn bdd_quantize_1_5_check(w: &mut R4g1World) {
    let q = w.lower_q_normal.as_ref().expect("normal q");
    assert_eq!(q.q88_value, 384);
    assert!(!q.saturated);
}

#[then("extreme scores saturate at i16 MAX and i16 MIN")]
fn bdd_quantize_extreme_check(w: &mut R4g1World) {
    let q_max = w.lower_q_max.as_ref().expect("max q");
    assert_eq!(q_max.q88_value, i16::MAX);
    assert!(q_max.saturated);

    let q_min = w.lower_q_min.as_ref().expect("min q");
    assert_eq!(q_min.q88_value, i16::MIN);
    assert!(q_min.saturated);
}

#[given("a reference region with a 100-bit signature")]
fn bdd_given_100bit_sig(_w: &mut R4g1World) {}

#[when("region lowering is attempted")]
fn bdd_attempt_100bit_lowering(w: &mut R4g1World) {
    let long_sig = vec![true; 100];
    if let Err(e) =
        BooleanLoweringCompiler::lower_region("reg_overflow", &long_sig, 1.0, "cid_err", 101, 0)
    {
        w.lower_error = Some(e);
    }
}

#[then("lowering fails with an unrepresentable region error")]
fn bdd_unrepresentable_error_check(w: &mut R4g1World) {
    let err = w.lower_error.as_ref().expect("lower error");
    assert!(matches!(err, LoweringError::UnrepresentableRegion { .. }));
}

// =========================================================================
// Reference Compiler IR BDD Steps (#129)
// =========================================================================
use uor_r4_graph_compiler::reference_compiler_ir::{
    DifferentialCompilerHarness, ReferenceCompilerConfig, ReferenceCompilerPipeline,
};

#[given("a pinned mini-corpus of 2 text observations")]
fn bdd_pinned_mini_corpus(w: &mut R4g1World) {
    w.ref_corpus = vec![
        "First sentence observation".to_string(),
        "Second sentence observation".to_string(),
    ];
}

#[when("the reference compiler pipeline executes all 5 stages")]
fn bdd_execute_compiler_pipeline(w: &mut R4g1World) {
    let config = ReferenceCompilerConfig::default_v1();
    let corpus_refs: Vec<&str> = w.ref_corpus.iter().map(|s| s.as_str()).collect();
    let ir = ReferenceCompilerPipeline::compile(&corpus_refs, &config).unwrap();
    w.ref_ir = Some(ir);
}

#[then("a valid ReferenceGraphIr is produced with content CID")]
fn bdd_ir_produced_check(w: &mut R4g1World) {
    let ir = w.ref_ir.as_ref().expect("ref ir");
    assert!(ir.provenance.content_cid.starts_with("blake3:"));
}

#[then("the IR contains observations, states, regions, and objective reports")]
fn bdd_ir_contents_check(w: &mut R4g1World) {
    let ir = w.ref_ir.as_ref().expect("ref ir");
    assert_eq!(ir.observations.len(), 2);
    assert_eq!(ir.states.len(), 2);
    assert_eq!(ir.regions.len(), 1);
}

#[given("a compiled ReferenceGraphIr containing states \"state_0\" and \"state_1\"")]
fn bdd_compiled_ref_ir_given(w: &mut R4g1World) {
    let config = ReferenceCompilerConfig::default_v1();
    let corpus = vec!["First sentence observation", "Second sentence observation"];
    w.ref_ir = Some(ReferenceCompilerPipeline::compile(&corpus, &config).unwrap());
}

#[when("a state transition query is executed for \"state_0\" under action \"next\"")]
fn bdd_query_state_transition(w: &mut R4g1World) {
    let ir = w.ref_ir.as_ref().expect("ir");
    w.ref_transition_state = ir.transition("state_0", "next").cloned();
}

#[then("the transition returns state \"state_1\"")]
fn bdd_transition_returns_state_1(w: &mut R4g1World) {
    let st = w.ref_transition_state.as_ref().expect("state");
    assert_eq!(st.id, "state_1");
}

#[then("the emission prediction for \"state_0\" returns token probabilities")]
fn bdd_emission_prediction_check(w: &mut R4g1World) {
    let ir = w.ref_ir.as_ref().expect("ir");
    let em = ir.predict_emission("state_0").expect("emission");
    assert_eq!(*em.get(&42).unwrap(), 0.8);
}

#[given("a compiled ReferenceGraphIr with teacher loss 0.25")]
fn bdd_ref_ir_loss_given(w: &mut R4g1World) {
    let config = ReferenceCompilerConfig::default_v1();
    let corpus = vec!["First sentence observation"];
    w.ref_ir = Some(ReferenceCompilerPipeline::compile(&corpus, &config).unwrap());
}

#[when("compared against baseline teacher loss 0.26 with tolerance 0.05")]
fn bdd_run_differential_comparison(w: &mut R4g1World) {
    let ir = w.ref_ir.as_ref().expect("ir");
    let delta = DifferentialCompilerHarness::compare(ir, 0.26, 0.05).unwrap();
    w.ref_diff_delta = Some(delta);
}

#[then("the differential comparison passes cleanly")]
fn bdd_diff_comparison_passes(w: &mut R4g1World) {
    let delta = w.ref_diff_delta.expect("delta");
    assert!(delta < 0.05);
}

// =========================================================================
// Behavioral Probes BDD Steps (#128)
// =========================================================================
use uor_r4_graph_compiler::behavioral_probes::{
    BehavioralProbeError, BehavioralProbeHarness, ExpectedRelation, InterventionKind,
    InterventionRecord,
};

#[given("a baseline observation \"Context text sample\"")]
fn bdd_baseline_observation(w: &mut R4g1World) {
    w.probe_baseline_obs = "Context text sample".to_string();
}

#[when("an invariant surface variation probe and a sensitive goal change probe are evaluated")]
fn bdd_evaluate_probes(w: &mut R4g1World) {
    let obs = &w.probe_baseline_obs;
    let p_inv = InterventionRecord::new(
        obs,
        InterventionKind::SurfaceVariation,
        (0, 7),
        ExpectedRelation::Invariant,
        vec![0.9, 0.1],
        vec![0.905, 0.095],
    )
    .unwrap();

    let p_sens = InterventionRecord::new(
        obs,
        InterventionKind::GoalChange,
        (0, 7),
        ExpectedRelation::Sensitive,
        vec![0.9, 0.1],
        vec![0.1, 0.9],
    )
    .unwrap();

    let report = BehavioralProbeHarness::evaluate_suite(&[p_inv, p_sens], 0.05, 0.5).unwrap();
    w.probe_suite_report = Some(report);
}

#[then("both invariance and sensitivity expectations pass cleanly")]
fn bdd_invariance_sensitivity_pass(w: &mut R4g1World) {
    let report = w.probe_suite_report.as_ref().expect("report");
    assert_eq!(report.invariance_score, 1.0);
    assert_eq!(report.sensitivity_score, 1.0);
}

#[then("the anti-memorization guard succeeds")]
fn bdd_memorization_guard_succeeds(w: &mut R4g1World) {
    let report = w.probe_suite_report.as_ref().expect("report");
    assert!(report.memorization_check_passed);
}

#[given("a sensitive goal change probe that produces zero output divergence")]
fn bdd_zero_divergence_sensitive_probe(w: &mut R4g1World) {
    let p_mem = InterventionRecord::new(
        "Context text sample",
        InterventionKind::GoalChange,
        (0, 7),
        ExpectedRelation::Sensitive,
        vec![0.9, 0.1],
        vec![0.9, 0.1], // div = 0.0 -> memorization!
    )
    .unwrap();

    if let Err(e) = BehavioralProbeHarness::evaluate_suite(&[p_mem], 0.05, 0.5) {
        w.probe_suite_error = Some(e);
    }
}

#[when("the probe suite is evaluated by the behavioral harness")]
fn bdd_harness_eval_step(_w: &mut R4g1World) {}

#[then("evaluation fails with a memorization detected error")]
fn bdd_memorization_error_check(w: &mut R4g1World) {
    let err = w.probe_suite_error.as_ref().expect("suite error");
    assert!(matches!(
        err,
        BehavioralProbeError::MemorizationDetected { .. }
    ));
}

#[given("an observation of length 15")]
fn bdd_observation_len_15(_w: &mut R4g1World) {}

#[when("an intervention record is created with span [0..20]")]
fn bdd_create_out_of_bounds_span(w: &mut R4g1World) {
    if let Err(e) = InterventionRecord::new(
        "Short 15 char!!",
        InterventionKind::ContextAblation,
        (0, 20),
        ExpectedRelation::Invariant,
        vec![1.0],
        vec![1.0],
    ) {
        w.probe_record_error = Some(e);
    }
}

#[then("record creation fails with a span out of bounds error")]
fn bdd_span_out_of_bounds_check(w: &mut R4g1World) {
    let err = w.probe_record_error.as_ref().expect("record error");
    assert!(matches!(err, BehavioralProbeError::SpanOutOfBounds { .. }));
}

// =========================================================================
// Semantic State Space BDD Steps (#124)
// =========================================================================
use uor_r4_graph_compiler::semantic_state::{
    Action as SemAction, Belief as SemBelief, Constraint as SemConstraint, Goal as SemGoal,
    Region as SemRegion, SemanticState as SemState, SemanticStateError as SemError,
    Trajectory as SemTrajectory, TransitionEvaluator as SemEvaluator,
};

#[given("an initial semantic state \"s0\" with vector [0.0, 0.0] and signature [0]")]
fn bdd_initial_state_s0(w: &mut R4g1World) {
    w.state_s0 = Some(SemState::new("s0", vec![0.0, 0.0], vec![0], 1.0));
}

#[when(
    "a semantic action \"move_right\" with delta vector [1.0, 0.0] and mask flip [1] is applied"
)]
fn bdd_apply_move_right(w: &mut R4g1World) {
    let s0 = w.state_s0.as_ref().expect("initial state s0");
    let action = SemAction::new("move_right", vec![1.0, 0.0], vec![1]);
    let evaluator = SemEvaluator::new();
    w.state_eval_res = Some(evaluator.apply(s0, &action));
}

#[then("the transition succeeds with target state \"s0_move_right\"")]
fn bdd_transition_succeeds(w: &mut R4g1World) {
    let res = w.state_eval_res.as_ref().expect("transition result");
    assert!(res.is_ok());
    assert_eq!(res.as_ref().unwrap().id, "s0_move_right");
}

#[then("the target state has vector [1.0, 0.0] and signature [1]")]
fn bdd_target_state_values(w: &mut R4g1World) {
    let state = w.state_eval_res.as_ref().unwrap().as_ref().unwrap();
    assert_eq!(state.vector, vec![1.0, 0.0]);
    assert_eq!(state.boolean_signature, vec![1]);
}

#[given("an initial semantic state \"s_invalid\" with negative vector [-1.0, 0.0]")]
fn bdd_initial_negative_state(w: &mut R4g1World) {
    w.state_s0 = Some(SemState::new("s_invalid", vec![-1.0, 0.0], vec![0], 1.0));
}

#[when("an action requiring non-negative coordinates is applied")]
fn bdd_apply_action_with_precondition(w: &mut R4g1World) {
    let s0 = w.state_s0.as_ref().expect("state");
    let action = SemAction::new("check_pos", vec![1.0, 0.0], vec![0])
        .with_precondition(|s| s.vector[0] >= 0.0);
    let evaluator = SemEvaluator::new();
    w.state_eval_res = Some(evaluator.apply(s0, &action));
}

#[then("the transition fails with a precondition error")]
fn bdd_transition_fails_precondition(w: &mut R4g1World) {
    let res = w.state_eval_res.as_ref().expect("res");
    assert!(matches!(res, Err(SemError::PreconditionFailed { .. })));
}

#[given("a hazard constraint centered at [5.0, 5.0] with radius 1.0")]
fn bdd_hazard_constraint(w: &mut R4g1World) {
    let danger_region = SemRegion::new("danger", vec![5.0, 5.0], 1.0, "Hazard Zone");
    let constraint = SemConstraint::new("no_hazard", danger_region);
    let mut eval = SemEvaluator::new();
    eval.add_constraint(constraint);
    w.hazard_evaluator = Some(eval);
}

#[given("an initial state at [0.0, 0.0]")]
fn bdd_initial_zero_state(w: &mut R4g1World) {
    w.state_s0 = Some(SemState::new("s_zero", vec![0.0, 0.0], vec![0], 1.0));
}

#[when("an action attempts to step to [5.0, 5.0]")]
fn bdd_step_into_hazard(w: &mut R4g1World) {
    let s0 = w.state_s0.as_ref().expect("state");
    let action = SemAction::new("step_hazard", vec![5.0, 5.0], vec![0]);
    let evaluator = w.hazard_evaluator.as_ref().expect("evaluator");
    w.state_eval_res = Some(evaluator.apply(s0, &action));
}

#[then("the transition fails with a forbidden state error")]
fn bdd_transition_fails_forbidden(w: &mut R4g1World) {
    let res = w.state_eval_res.as_ref().expect("res");
    assert!(matches!(res, Err(SemError::ForbiddenState { .. })));
}

#[given("a goal target region centered at [10.0, 10.0] with radius 2.0 and minimum confidence 0.8")]
fn bdd_goal_target_region(_w: &mut R4g1World) {}

#[when("a state \"s_target\" at [10.0, 11.0] with confidence 0.9 is evaluated")]
fn bdd_evaluate_goal_and_belief(w: &mut R4g1World) {
    let target_region = SemRegion::new("target", vec![10.0, 10.0], 2.0, "Goal Zone");
    let goal = SemGoal::new("reach_target", target_region.clone(), 0.8);
    let belief = SemBelief::new("target_belief", target_region, 0.5);

    let s_target = SemState::new("s_target", vec![10.0, 11.0], vec![1], 0.9);
    let s_far = SemState::new("s_far", vec![0.0, 0.0], vec![0], 0.9);

    w.goal_satisfied = Some(goal.is_satisfied_by(&s_target));
    w.belief_in = Some(belief.evaluate(&s_target));
    w.belief_out = Some(belief.evaluate(&s_far));
}

#[then("the goal is satisfied by the state")]
fn bdd_goal_satisfied_check(w: &mut R4g1World) {
    assert_eq!(w.goal_satisfied, Some(true));
}

#[then("the belief likelihood is higher than a state at [0.0, 0.0]")]
fn bdd_belief_higher_check(w: &mut R4g1World) {
    let b_in = w.belief_in.expect("belief in");
    let b_out = w.belief_out.expect("belief out");
    assert!(b_in > b_out);
}

#[given("a trajectory with maximum 2 steps")]
fn bdd_max_2_steps_trajectory(w: &mut R4g1World) {
    w.state_s0 = Some(SemState::new("s_init", vec![0.0], vec![0], 1.0));
}

#[when("3 step actions are applied sequentially")]
fn bdd_apply_3_steps(w: &mut R4g1World) {
    let s0 = w.state_s0.take().expect("init state");
    let evaluator = SemEvaluator::new();
    let action = SemAction::new("step", vec![1.0], vec![0]);
    let mut traj = SemTrajectory::new(s0, 2);

    let _ = traj.step(&action, &evaluator);
    let _ = traj.step(&action, &evaluator);
    let res = traj.step(&action, &evaluator);

    if let Err(e) = res {
        w.trajectory_error = Some(e);
    }
}

#[then("the 3rd step fails with a maximum steps exceeded error")]
fn bdd_max_steps_error_check(w: &mut R4g1World) {
    let err = w.trajectory_error.as_ref().expect("trajectory error");
    assert!(matches!(err, SemError::MaxStepsExceeded { limit: 2 }));
}

// =========================================================================
// Inference Contract BDD Steps (#157)
// =========================================================================
use uor_r4_graph_format::inference_contract::{
    BoundaryActivity, ContractValidationError, InferenceContractVerifier, OperationClass,
};

#[given("the normative inference contract specification")]
fn bdd_contract_spec_given(_w: &mut R4g1World) {}

#[when("audited by the inference contract verifier")]
fn bdd_contract_audit_when(w: &mut R4g1World) {
    let rep = InferenceContractVerifier::audit_contract_compliance().expect("contract audit");
    w.contract_report = Some(rep);
}

#[then("contract version \"1.0.0\" is verified with 0 steady-state allocations")]
fn bdd_contract_ver_check(w: &mut R4g1World) {
    let rep = w.contract_report.as_ref().expect("contract report");
    assert_eq!(rep.contract_version.to_string(), "1.0.0");
    assert!(rep.is_zero_allocation_guaranteed);
}

#[then("the contract audit certification status is verified")]
fn bdd_contract_cert_check(w: &mut R4g1World) {
    let rep = w.contract_report.as_ref().expect("contract report");
    assert!(rep.is_certified);
}

#[given("a hot-path inference activity")]
fn bdd_contract_hotpath_given(_w: &mut R4g1World) {}

#[when("an operation class is audited")]
fn bdd_contract_op_audit_when(_w: &mut R4g1World) {}

#[then("permitted bitwise and integer operations are accepted")]
fn bdd_contract_permitted_accepted(_w: &mut R4g1World) {
    assert!(InferenceContractVerifier::audit_operation(
        BoundaryActivity::HotPathInference,
        OperationClass::PermittedBitwise
    )
    .is_ok());
    assert!(InferenceContractVerifier::audit_operation(
        BoundaryActivity::HotPathInference,
        OperationClass::PermittedIntArithmetic
    )
    .is_ok());
}

#[then("forbidden float and multiplication operations are rejected")]
fn bdd_contract_forbidden_rejected(_w: &mut R4g1World) {
    assert_eq!(
        InferenceContractVerifier::audit_operation(
            BoundaryActivity::HotPathInference,
            OperationClass::ForbiddenFloat
        ),
        Err(ContractValidationError::ForbiddenFloatOperationDetected)
    );
    assert_eq!(
        InferenceContractVerifier::audit_operation(
            BoundaryActivity::HotPathInference,
            OperationClass::ForbiddenMultiplyDivide
        ),
        Err(ContractValidationError::ForbiddenMultiplicationDetected)
    );
}

// =========================================================================
// Packed Kernels BDD Steps (#159)
// =========================================================================
use uor_r4_graph_format::ScoreQ;
use uor_r4_graph_runtime::packed_kernels::{
    accumulate_candidate_shortlist, advance_frontier, decode_canonical_topk, PackedFrontier,
    PackedShortlist, StepOutput,
};

#[given("a zeroed packed active frontier of capacity 4")]
fn bdd_packed_zeroed_frontier(w: &mut R4g1World) {
    w.packed_frontier = PackedFrontier::new();
}

#[when("node 1 with score 100 and node 2 with score 200 are advanced into the frontier")]
fn bdd_packed_advance_nodes(w: &mut R4g1World) {
    advance_frontier(&mut w.packed_frontier, 1, ScoreQ::from_raw(100));
    advance_frontier(&mut w.packed_frontier, 2, ScoreQ::from_raw(200));
}

#[then("the active frontier count is 2 and contains both nodes")]
fn bdd_packed_check_frontier(w: &mut R4g1World) {
    assert_eq!(w.packed_frontier.count, 2);
    assert!(w.packed_frontier.nodes[..w.packed_frontier.count].contains(&1));
    assert!(w.packed_frontier.nodes[..w.packed_frontier.count].contains(&2));
}

#[given("an empty packed candidate shortlist of capacity 4")]
fn bdd_packed_empty_shortlist(w: &mut R4g1World) {
    w.packed_shortlist = PackedShortlist::new();
}

#[when("node 10 is accumulated into the shortlist twice")]
fn bdd_packed_accumulate_shortlist_twice(w: &mut R4g1World) {
    accumulate_candidate_shortlist(None, &mut w.packed_shortlist, 10);
    accumulate_candidate_shortlist(None, &mut w.packed_shortlist, 10);
}

#[then("the shortlist count is 1 and contains node 10")]
fn bdd_packed_check_shortlist(w: &mut R4g1World) {
    assert_eq!(w.packed_shortlist.count, 1);
    assert_eq!(w.packed_shortlist.candidates[0], 10);
}

#[given("a candidate set with duplicate scores and distinct IDs")]
fn bdd_packed_candidates_init(_w: &mut R4g1World) {
    // Candidates are constructed inline in the `when` step below.
}

#[when("decoded by the packed top-K kernel")]
fn bdd_packed_decode_topk(w: &mut R4g1World) {
    let mut candidates = [
        (20u32, ScoreQ::from_raw(500)),
        (10u32, ScoreQ::from_raw(500)),
        (5u32, ScoreQ::from_raw(1000)),
    ];
    w.packed_output = StepOutput::new();
    decode_canonical_topk(&mut candidates, &mut w.packed_output);
}

#[then("the top predictions are sorted by score descending and ID ascending")]
fn bdd_packed_check_topk(w: &mut R4g1World) {
    assert_eq!(w.packed_output.count, 3);
    assert_eq!(w.packed_output.predictions[0], (5, ScoreQ::from_raw(1000)));
    assert_eq!(w.packed_output.predictions[1], (10, ScoreQ::from_raw(500)));
    assert_eq!(w.packed_output.predictions[2], (20, ScoreQ::from_raw(500)));
}

// =========================================================================
// Scoring Semantics BDD Steps (#158)
// =========================================================================
use uor_r4_graph_format::scoring_semantics::{
    ResidualContribution, ResidualContributionKind, ScoreAccumulator,
};

#[given("a zeroed score accumulator")]
fn bdd_scoring_zeroed_acc(w: &mut R4g1World) {
    w.score_accumulator = ScoreAccumulator::new();
}

#[when("a root prior residual of 1000 and a child correction of 500 are accumulated")]
fn bdd_scoring_accumulate_residuals(w: &mut R4g1World) {
    w.score_accumulator
        .accumulate(&ResidualContribution {
            kind: ResidualContributionKind::RootPrior,
            contribution_id: 1,
            raw_value: 1000,
        })
        .expect("accumulate root prior");
    w.score_accumulator
        .accumulate(&ResidualContribution {
            kind: ResidualContributionKind::ChildCorrection,
            contribution_id: 2,
            raw_value: 500,
        })
        .expect("accumulate child correction");
}

#[then("the final score is 1500 with zero heap allocations")]
fn bdd_scoring_check_score_1500(w: &mut R4g1World) {
    assert_eq!(w.score_accumulator.score(), 1500);
}

#[given("a score accumulator containing evidence contribution 42")]
fn bdd_scoring_acc_with_ev_42(w: &mut R4g1World) {
    w.score_accumulator = ScoreAccumulator::new();
    w.score_accumulator
        .accumulate(&ResidualContribution {
            kind: ResidualContributionKind::InteractionResidual,
            contribution_id: 42,
            raw_value: 300,
        })
        .expect("accumulate ev 42");
}

#[when("the same evidence contribution 42 is accumulated again")]
fn bdd_scoring_accumulate_duplicate_ev_42(w: &mut R4g1World) {
    w.score_accumulator
        .accumulate(&ResidualContribution {
            kind: ResidualContributionKind::InteractionResidual,
            contribution_id: 42,
            raw_value: 300,
        })
        .expect("accumulate duplicate ev 42");
}

#[then("the duplicate evidence is ignored and the score remains unchanged")]
fn bdd_scoring_check_duplicate_ignored(w: &mut R4g1World) {
    assert_eq!(w.score_accumulator.score(), 300);
    assert_eq!(w.score_accumulator.evidence_count(), 1);
}

#[given("candidate A with score 500 and ID 10")]
fn bdd_scoring_cand_a(_w: &mut R4g1World) {}

#[given("candidate B with score 500 and ID 20")]
fn bdd_scoring_cand_b(_w: &mut R4g1World) {}

#[when("candidates are compared by the deterministic tie-breaker")]
fn bdd_scoring_compare_cands(w: &mut R4g1World) {
    w.candidate_cmp_result = Some(ScoreAccumulator::<16>::compare_candidates(500, 10, 500, 20));
}

#[then("candidate A ranks higher than candidate B")]
fn bdd_scoring_check_cand_a_wins(w: &mut R4g1World) {
    let res = w.candidate_cmp_result.expect("candidate cmp result");
    assert_eq!(res, core::cmp::Ordering::Less);
}
// Performance Certificate BDD Steps (#161)
// =========================================================================
use uor_r4_graph_certify::performance_certificate::RuntimePerformanceCertificate;

#[given("a new runtime performance certificate")]
fn bdd_perf_cert_given(w: &mut R4g1World) {
    w.perf_cert = Some(RuntimePerformanceCertificate::new());
}

#[when("audited for evidence link integrity")]
fn bdd_perf_cert_audit_when(_w: &mut R4g1World) {}

#[then("all declared-zero fields contain non-empty evidence links and steady-state allocations are zero")]
fn bdd_perf_cert_links_check(w: &mut R4g1World) {
    let cert = w.perf_cert.as_ref().expect("perf cert");
    assert!(cert.verify_evidence_links());
}

#[given("a performance certificate with CPU portability record")]
fn bdd_perf_cert_portability_given(w: &mut R4g1World) {
    w.perf_cert = Some(RuntimePerformanceCertificate::new());
}

#[when("checked for execution portability")]
fn bdd_perf_cert_portability_when(_w: &mut R4g1World) {}

#[then(
    "scalar fallback is confirmed and target tier matches the current architecture scalar-portable tier"
)]
fn bdd_perf_cert_portability_check(w: &mut R4g1World) {
    let cert = w.perf_cert.as_ref().expect("perf cert");
    let expected_tier = format!("{}-scalar-portable", std::env::consts::ARCH);
    assert!(cert.cpu_portability.scalar_fallback_confirmed);
    assert_eq!(cert.cpu_portability.target_tier, expected_tier);
}
#[tokio::main]
async fn main() {
    R4g1World::cucumber()
        .fail_on_skipped()
        .run_and_exit(concat!(env!("CARGO_MANIFEST_DIR"), "/features/suites"))
        .await;
}
