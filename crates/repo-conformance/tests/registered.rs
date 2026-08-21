//! One marker test per registered conformance ID (#273 migration).
//!
//! Each asserts its Gherkin suite is present, non-empty, tagged with the ID at
//! the registered level, and free of pending steps -- so the register row is
//! bound to real, well-formed scenarios (CM-02), and the test fails if that
//! suite regresses. The function names are explicit (not macro-generated) and
//! end in the ID slug, which is how the source-scanning meta-gate finds them.

use repo_conformance::scenarios_in;
use std::path::PathBuf;

fn root() -> PathBuf {
    // #788: runtime resolution, never compile-time env!() — a cached test
    // binary carries the baked path of whatever checkout built it, which
    // made all 29 markers fail (`features/suites` NotFound) after a
    // sibling worktree was deleted (AUD-VER-001). repo_model::repo_root
    // walks up from the runtime CARGO_MANIFEST_DIR (cwd fallback) to the
    // ancestor holding model/ledger.toml.
    repo_model::repo_root()
}

fn check(id: &str, suite: &str) {
    let report = scenarios_in(&root().join("features/suites")).expect("suites read");
    let mine: Vec<_> = report
        .scenarios
        .iter()
        .filter(|s| s.suite == suite)
        .collect();
    assert!(
        !mine.is_empty(),
        "{suite}.feature has no scenarios for {id}"
    );
    for s in &mine {
        assert_eq!(
            s.id, id,
            "scenario `{}` in {suite} is tagged `{}`, not {id}",
            s.statement, s.id
        );
        assert_eq!(
            s.level, "build",
            "scenario `{}` level is `{}`, not build",
            s.statement, s.level
        );
        assert!(
            !s.steps.is_empty(),
            "scenario `{}` has no steps",
            s.statement
        );
    }
}

#[test]
fn behavioral_probes_rf_01() {
    check("RF-01", "behavioral_probes");
}

#[test]
fn compiler_executor_rf_02() {
    check("RF-02", "compiler_executor");
}

#[test]
fn compiler_jobs_config_rf_03() {
    check("RF-03", "compiler_jobs_config");
}

#[test]
fn compiler_memory_budget_rf_04() {
    check("RF-04", "compiler_memory_budget");
}

#[test]
fn compiler_stage_dag_rf_05() {
    check("RF-05", "compiler_stage_dag");
}

#[test]
fn expand_proof_model_rf_06() {
    check("RF-06", "expand_proof_model");
}

#[test]
fn formal_monograph_rf_07() {
    check("RF-07", "formal_monograph");
}

#[test]
fn future_state_planner_rf_08() {
    check("RF-08", "future_state_planner");
}

#[test]
fn graph_invariant_ownership_rf_09() {
    check("RF-09", "graph_invariant_ownership");
}

#[test]
fn inference_contract_rf_10() {
    check("RF-10", "inference_contract");
}

#[test]
fn inference_operation_contract_rf_11() {
    check("RF-11", "inference_operation_contract");
}

#[test]
fn lower_semantic_regions_rf_12() {
    check("RF-12", "lower_semantic_regions");
}

#[test]
fn packed_kernels_rf_13() {
    check("RF-13", "packed_kernels");
}

#[test]
fn parallel_observation_shards_rf_14() {
    check("RF-14", "parallel_observation_shards");
}

#[test]
fn parallel_reproducibility_rf_15() {
    check("RF-15", "parallel_reproducibility");
}

#[test]
fn pdf_traceability_matrix_rf_16() {
    check("RF-16", "pdf_traceability_matrix");
}

#[test]
fn performance_certificate_rf_17() {
    check("RF-17", "performance_certificate");
}

#[test]
fn quantum_cover_rf_18() {
    check("RF-18", "quantum_cover");
}

#[test]
fn quantum_facade_benchmarks_rf_19() {
    check("RF-19", "quantum_facade_benchmarks");
}

#[test]
fn quantum_lie_jordan_rf_20() {
    check("RF-20", "quantum_lie_jordan");
}

#[test]
fn r4g1_compile_quality_rf_21() {
    check("RF-21", "r4g1_compile_quality");
}

#[test]
fn r4g1_quality_rf_22() {
    check("RF-22", "r4g1_quality");
}

#[test]
fn r4g1_runtime_rf_23() {
    check("RF-23", "r4g1_runtime");
}

#[test]
fn rate_distortion_compression_rf_24() {
    check("RF-24", "rate_distortion_compression");
}

#[test]
fn reference_compiler_ir_rf_25() {
    check("RF-25", "reference_compiler_ir");
}

#[test]
fn scoring_semantics_rf_26() {
    check("RF-26", "scoring_semantics");
}

#[test]
fn semantic_state_space_rf_27() {
    check("RF-27", "semantic_state_space");
}

#[test]
fn separate_semantic_emission_rf_28() {
    check("RF-28", "separate_semantic_emission");
}

#[test]
fn teacher_parity_benchmarks_rf_29() {
    check("RF-29", "teacher_parity_benchmarks");
}

#[test]
fn selective_prediction_surfaces_rf_30() {
    check("RF-30", "selective_prediction_surfaces");
}

#[test]
fn skipmix_serving_lane_rf_31() {
    check("RF-31", "skipmix_serving_lane");
}
