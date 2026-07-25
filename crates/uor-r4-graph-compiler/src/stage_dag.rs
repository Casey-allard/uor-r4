//! Normative Compiler Stage Ownership and Parallelization DAG (#166).
//!
//! Classifies compiler pipeline stages into four concurrency classes:
//! - `ParallelSafe`
//! - `ParallelWithDeterministicMerge`
//! - `BoundedParallel`
//! - `SequentialCanonicalFinalization`

use std::fmt;

/// Concurrency class classification for a compiler stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConcurrencyClass {
    /// Embarrassingly parallel over independent shards/inputs.
    ParallelSafe,
    /// Parallel discovery + stable-sort/dedup/ordered reduction.
    ParallelWithDeterministicMerge,
    /// Bounded by memory budget or external teacher probe rate limits.
    BoundedParallel,
    /// Strictly single-threaded to protect canonical artifact form & byte equality.
    SequentialCanonicalFinalization,
}

impl fmt::Display for ConcurrencyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParallelSafe => write!(f, "Parallel-Safe"),
            Self::ParallelWithDeterministicMerge => write!(f, "Parallel with Deterministic Merge"),
            Self::BoundedParallel => write!(f, "Bounded Parallel"),
            Self::SequentialCanonicalFinalization => write!(f, "Sequential Canonical Finalization"),
        }
    }
}

/// Metadata entry for a single compiler pipeline stage node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageNode {
    pub stage_id: &'static str,
    pub name: &'static str,
    pub class: ConcurrencyClass,
    pub module: &'static str,
    pub boundary_owner_issue: &'static str,
}

/// Programmatic registry of all classified compiler pipeline stages.
pub struct CompilerStageDag;

impl CompilerStageDag {
    pub fn all_stages() -> &'static [StageNode] {
        &[
            StageNode {
                stage_id: "S01",
                name: "Corpus Partitioning",
                class: ConcurrencyClass::ParallelSafe,
                module: "observation_text",
                boundary_owner_issue: "#170",
            },
            StageNode {
                stage_id: "S02",
                name: "Teacher-Probe Request Prep",
                class: ConcurrencyClass::BoundedParallel,
                module: "observation",
                boundary_owner_issue: "#170",
            },
            StageNode {
                stage_id: "S03",
                name: "Trace Normalization",
                class: ConcurrencyClass::ParallelSafe,
                module: "observation",
                boundary_owner_issue: "#170",
            },
            StageNode {
                stage_id: "S04",
                name: "Contextual Feature Extraction",
                class: ConcurrencyClass::ParallelSafe,
                module: "observation",
                boundary_owner_issue: "#170",
            },
            StageNode {
                stage_id: "S05",
                name: "Behavioral Fingerprinting",
                class: ConcurrencyClass::ParallelSafe,
                module: "behavioral_probes",
                boundary_owner_issue: "#170",
            },
            StageNode {
                stage_id: "S06",
                name: "Paraphrase & Counterfactual Analysis",
                class: ConcurrencyClass::ParallelWithDeterministicMerge,
                module: "perturbation",
                boundary_owner_issue: "#170",
            },
            StageNode {
                stage_id: "S07",
                name: "Distance & Divergence Calculation",
                class: ConcurrencyClass::ParallelSafe,
                module: "quantum_cover",
                boundary_owner_issue: "#171",
            },
            StageNode {
                stage_id: "S08",
                name: "Nearest-Neighbor Search",
                class: ConcurrencyClass::ParallelWithDeterministicMerge,
                module: "quantum_cover",
                boundary_owner_issue: "#171",
            },
            StageNode {
                stage_id: "S09",
                name: "Recursive Clustering",
                class: ConcurrencyClass::ParallelWithDeterministicMerge,
                module: "quantum_cover",
                boundary_owner_issue: "#171",
            },
            StageNode {
                stage_id: "S10",
                name: "Region Proposal",
                class: ConcurrencyClass::ParallelWithDeterministicMerge,
                module: "induction",
                boundary_owner_issue: "#171",
            },
            StageNode {
                stage_id: "S11",
                name: "Overlap Discovery",
                class: ConcurrencyClass::ParallelWithDeterministicMerge,
                module: "induction",
                boundary_owner_issue: "#171",
            },
            StageNode {
                stage_id: "S12",
                name: "Parent/Child Discovery",
                class: ConcurrencyClass::SequentialCanonicalFinalization,
                module: "induction",
                boundary_owner_issue: "#171",
            },
            StageNode {
                stage_id: "S13",
                name: "Transition Discovery",
                class: ConcurrencyClass::ParallelWithDeterministicMerge,
                module: "semantic_state",
                boundary_owner_issue: "#171",
            },
            StageNode {
                stage_id: "S14",
                name: "XOR-Polynomial Search",
                class: ConcurrencyClass::BoundedParallel,
                module: "routing",
                boundary_owner_issue: "#172",
            },
            StageNode {
                stage_id: "S15",
                name: "Routing-Program Search",
                class: ConcurrencyClass::BoundedParallel,
                module: "routing",
                boundary_owner_issue: "#172",
            },
            StageNode {
                stage_id: "S16",
                name: "Mask & Threshold Search",
                class: ConcurrencyClass::ParallelSafe,
                module: "lower_semantic_regions",
                boundary_owner_issue: "#172",
            },
            StageNode {
                stage_id: "S17",
                name: "Radius Calibration",
                class: ConcurrencyClass::ParallelSafe,
                module: "lower_semantic_regions",
                boundary_owner_issue: "#172",
            },
            StageNode {
                stage_id: "S18",
                name: "Collision Analysis",
                class: ConcurrencyClass::BoundedParallel,
                module: "lower_semantic_regions",
                boundary_owner_issue: "#172",
            },
            StageNode {
                stage_id: "S19",
                name: "Shortlist-Recall Evaluation",
                class: ConcurrencyClass::ParallelWithDeterministicMerge,
                module: "shortlist_evaluator",
                boundary_owner_issue: "#173",
            },
            StageNode {
                stage_id: "S20",
                name: "Region Emission Compilation",
                class: ConcurrencyClass::ParallelWithDeterministicMerge,
                module: "pack",
                boundary_owner_issue: "#173",
            },
            StageNode {
                stage_id: "S21",
                name: "Residual Compilation",
                class: ConcurrencyClass::ParallelWithDeterministicMerge,
                module: "residual",
                boundary_owner_issue: "#173",
            },
            StageNode {
                stage_id: "S22",
                name: "Quantization Analysis",
                class: ConcurrencyClass::BoundedParallel,
                module: "rate_distortion_compression",
                boundary_owner_issue: "#173",
            },
            StageNode {
                stage_id: "S23",
                name: "Empirical Certification",
                class: ConcurrencyClass::BoundedParallel,
                module: "performance_certificate",
                boundary_owner_issue: "#173",
            },
            StageNode {
                stage_id: "S24",
                name: "Graph-Fragment Construction",
                class: ConcurrencyClass::SequentialCanonicalFinalization,
                module: "graph",
                boundary_owner_issue: "#173",
            },
            StageNode {
                stage_id: "S25",
                name: "Artifact Section Construction",
                class: ConcurrencyClass::SequentialCanonicalFinalization,
                module: "pack",
                boundary_owner_issue: "#173",
            },
            StageNode {
                stage_id: "S26",
                name: "Canonical Sorting & ID Assignment",
                class: ConcurrencyClass::SequentialCanonicalFinalization,
                module: "pack",
                boundary_owner_issue: "#167",
            },
            StageNode {
                stage_id: "S27",
                name: "Final Offset Calculation & Packing",
                class: ConcurrencyClass::SequentialCanonicalFinalization,
                module: "pack",
                boundary_owner_issue: "#167",
            },
            StageNode {
                stage_id: "S28",
                name: "Root Hashing & Signing",
                class: ConcurrencyClass::SequentialCanonicalFinalization,
                module: "pack",
                boundary_owner_issue: "#167",
            },
        ]
    }

    pub fn finalization_spine() -> Vec<&'static StageNode> {
        Self::all_stages()
            .iter()
            .filter(|s| s.class == ConcurrencyClass::SequentialCanonicalFinalization)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_dag_completeness_and_spine() {
        let stages = CompilerStageDag::all_stages();
        assert_eq!(stages.len(), 28);

        let spine = CompilerStageDag::finalization_spine();
        assert_eq!(spine.len(), 6);
        let spine_ids: Vec<&str> = spine.iter().map(|s| s.stage_id).collect();
        assert_eq!(spine_ids, vec!["S12", "S24", "S25", "S26", "S27", "S28"]);
    }
}
