//! #846 final-certification access audit.
//!
//! The frozen #844 constitution says that final held-out composition and
//! topology cells are CID-bound, access-logged, and unopened until #846.  The
//! #846 run contract makes that property a binding cheap instrument: the final
//! grid must not run when the seal is absent, overlaps prior access, or lacks a
//! content identity/access record.
//!
//! This test audits committed metadata and predecessor source/records only.  It
//! deliberately does not call the task generator, so a failing audit cannot
//! itself open or manufacture a final partition.

use std::collections::BTreeSet;

use uor_r4_graph_compiler::compositional_planning::AXIS_CARDINALITY;

const N_PER_CELL: usize = 512;

const SUITE_MANIFEST: &str =
    include_str!("../../uor-r4-api/capability_suites/compositional_reasoning.json");
const GENERATOR_SOURCE: &str =
    include_str!("../../uor-r4-graph-compiler/src/compositional_planning.rs");
const INDUCTION_SOURCE: &str =
    include_str!("../../uor-r4-graph-compiler/src/semantic_transitions.rs");
const FORMAT_SOURCE: &str = include_str!("../../uor-r4-graph-format/src/plan_sections.rs");
const RUNTIME_SOURCE: &str = include_str!("../../uor-r4-graph-runtime/src/plan.rs");
const PRIOR_HARNESS: &str = include_str!("compositional_planning_measurement_843.rs");
const SHARED_EPISODE: &str = include_str!("support/episode.rs");
const PRIOR_RECORD: &str = include_str!("../../../docs/bounded_semantic_transitions_843.md");
const COMMITTED_RESULT: &str =
    include_str!("../../../docs/compositional_planning_certification_846_result.json");

#[derive(Debug, Clone, PartialEq, Eq)]
struct AccessAudit {
    declared_sealed: BTreeSet<u8>,
    fitting_topologies: BTreeSet<u8>,
    prior_evaluation_topologies: BTreeSet<u8>,
    untouched_topologies: BTreeSet<u8>,
    has_partition_cid: bool,
    has_access_log_binding: bool,
    has_materialized_composition_axis: bool,
}

impl AccessAudit {
    fn passes(&self) -> bool {
        !self.declared_sealed.is_empty()
            && self.declared_sealed == self.untouched_topologies
            && self.has_partition_cid
            && self.has_access_log_binding
            && self.has_materialized_composition_axis
    }

    fn final_sample_ceiling(&self) -> usize {
        if self.passes() {
            self.declared_sealed.len() * N_PER_CELL
        } else {
            0
        }
    }
}

fn axis_digits(seed: u64) -> [u64; 4] {
    let c = AXIS_CARDINALITY;
    [
        seed % c,
        (seed / c) % c,
        (seed / (c * c)) % c,
        (seed / (c * c * c)) % c,
    ]
}

/// Reproduce the predecessor joint-split seed walk without generating a task.
fn joint_topologies(high_half: bool, count: usize) -> BTreeSet<u8> {
    let half = AXIS_CARDINALITY / 2;
    let mut topologies = BTreeSet::new();
    let mut kept = 0usize;
    let mut seed = 0u64;
    while kept < count {
        let digits = axis_digits(seed);
        let selected = if high_half {
            digits.iter().all(|digit| *digit >= half)
        } else {
            digits.iter().all(|digit| *digit < half)
        };
        if selected {
            topologies.insert(digits[2] as u8);
            kept += 1;
        }
        seed += 1;
    }
    topologies
}

fn struct_body<'a>(source: &'a str, name: &str) -> &'a str {
    let start = source
        .find(name)
        .unwrap_or_else(|| panic!("{name} exists in the frozen generator"));
    let rest = &source[start..];
    let end = rest
        .find("\n}\n")
        .unwrap_or_else(|| panic!("{name} has a closing brace"));
    &rest[..end]
}

fn committed_audit() -> AccessAudit {
    let fitting_topologies = joint_topologies(false, N_PER_CELL / 4);
    let prior_evaluation_topologies = joint_topologies(true, N_PER_CELL);
    let universe: BTreeSet<u8> = (0..AXIS_CARDINALITY as u8).collect();
    let opened: BTreeSet<u8> = fitting_topologies
        .union(&prior_evaluation_topologies)
        .copied()
        .collect();
    let untouched_topologies = universe.difference(&opened).copied().collect();

    // The exact predecessor harnesses instantiate the seal as empty.  This is
    // source evidence, not an inference from prose.
    assert!(PRIOR_HARNESS.contains("sealed_topologies: std::collections::BTreeSet::new()"));
    assert!(SHARED_EPISODE.contains("sealed_topologies: std::collections::BTreeSet::new()"));
    assert!(PRIOR_RECORD.contains("full 300-cell grid"));

    let split_cell = struct_body(GENERATOR_SOURCE, "pub struct SplitCell");
    AccessAudit {
        declared_sealed: BTreeSet::new(),
        fitting_topologies,
        prior_evaluation_topologies,
        untouched_topologies,
        has_partition_cid: SUITE_MANIFEST.contains("sealed_partition_cid")
            || SUITE_MANIFEST.contains("slice_partition_cid"),
        has_access_log_binding: SUITE_MANIFEST.contains("access_log")
            || SUITE_MANIFEST.contains("access-log"),
        has_materialized_composition_axis: split_cell.contains("composition"),
    }
}

fn cid(parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        hasher.update(&((*part).len() as u64).to_le_bytes());
        hasher.update(part);
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn benchmark_cid() -> String {
    cid(&[SUITE_MANIFEST.as_bytes(), GENERATOR_SOURCE.as_bytes()])
}

fn candidate_cid() -> String {
    cid(&[
        b"RF-33/bounded-breadth-first",
        INDUCTION_SOURCE.as_bytes(),
        FORMAT_SOURCE.as_bytes(),
        RUNTIME_SOURCE.as_bytes(),
    ])
}

fn audit_inputs_cid() -> String {
    cid(&[
        SUITE_MANIFEST.as_bytes(),
        GENERATOR_SOURCE.as_bytes(),
        INDUCTION_SOURCE.as_bytes(),
        FORMAT_SOURCE.as_bytes(),
        RUNTIME_SOURCE.as_bytes(),
        PRIOR_HARNESS.as_bytes(),
        SHARED_EPISODE.as_bytes(),
        PRIOR_RECORD.as_bytes(),
    ])
}

#[test]
fn the_access_audit_can_pass_and_fail() {
    let valid = AccessAudit {
        declared_sealed: BTreeSet::from([8, 9]),
        fitting_topologies: BTreeSet::from([0, 1, 2, 3]),
        prior_evaluation_topologies: BTreeSet::from([4, 5, 6, 7]),
        untouched_topologies: BTreeSet::from([8, 9]),
        has_partition_cid: true,
        has_access_log_binding: true,
        has_materialized_composition_axis: true,
    };
    assert!(valid.passes(), "a real disjoint seal clears the instrument");
    assert_eq!(valid.final_sample_ceiling(), 2 * N_PER_CELL);

    let mut overlap = valid;
    overlap.declared_sealed = BTreeSet::from([7, 8]);
    assert!(
        !overlap.passes(),
        "a previously opened cell fails the audit"
    );
    assert_eq!(overlap.final_sample_ceiling(), 0);
}

#[test]
fn committed_partition_fails_closed_before_final_access() {
    let audit = committed_audit();
    assert_eq!(audit.fitting_topologies, BTreeSet::from([0, 1, 2, 3]));
    assert_eq!(
        audit.prior_evaluation_topologies,
        BTreeSet::from([4, 5, 6, 7])
    );
    assert!(audit.untouched_topologies.is_empty());
    assert!(audit.declared_sealed.is_empty());
    assert!(!audit.has_partition_cid);
    assert!(!audit.has_access_log_binding);
    assert!(!audit.has_materialized_composition_axis);
    assert!(!audit.passes());
    assert_eq!(audit.final_sample_ceiling(), 0);

    println!("benchmark_cid   : {}", benchmark_cid());
    println!("candidate_cid   : {}", candidate_cid());
    println!("audit_inputs_cid: {}", audit_inputs_cid());
    println!("verdict         : FAIL -> REASONING NOT ESTABLISHED");
    println!("final_grid      : NOT_RUN");
}

#[test]
fn committed_result_binds_the_audit_inputs_and_negative_verdict() {
    let result: serde_json::Value =
        serde_json::from_str(COMMITTED_RESULT).expect("the #846 result is valid JSON");
    assert_eq!(
        result["benchmark_cid"].as_str(),
        Some(benchmark_cid().as_str())
    );
    assert_eq!(
        result["candidate_cid"].as_str(),
        Some(candidate_cid().as_str())
    );
    assert_eq!(
        result["audit_inputs_cid"].as_str(),
        Some(audit_inputs_cid().as_str())
    );
    assert_eq!(result["instrument_verdict"], "FAIL");
    assert_eq!(result["final_grid"], "NOT_RUN");
    assert_eq!(result["verdict"], "REASONING NOT ESTABLISHED");
    assert_eq!(result["audit"]["sealed_sample_ceiling"], 0);
}
