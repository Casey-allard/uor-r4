//! Native support surface of the graph compiler (legacy-crate repair, 19 Oct 2026).
//!
//! The historical `lib.rs` compiles an R4G1 graph from a transformer teacher:
//! it links `uor-r4-model-source` (normalized-exponential attention and dense
//! matrix products), samples teacher logits through the retired
//! `transformerless::compiler` top-8 normalized-exponential sampler, and assigns
//! contexts through the retired `transformerless::runtime` dense dot-argmax.
//! Those paths are banned in every compiled module (D23/D25/D29/D30)
//! and stay preserved in place as uncompiled evidence; the R4G1 mechanism stays
//! PARKED (D22). This surface compiles only the modules that need none of them.
//! No Cargo feature re-enables the historical surface.

pub mod behavioral_probes;
pub mod compositional_planning;
pub mod dependency_audit;
pub mod executor;
pub mod future_state_planner;
#[cfg(feature = "graph-construction")]
pub mod graph;
pub mod jobs_config;
pub mod lower_semantic_regions;
pub mod memory_budget;
pub mod monograph;
pub mod observation_shards;
pub mod pack;
#[cfg(feature = "patch-induction")]
pub mod patch_induction;
#[cfg(feature = "perturbation")]
pub mod perturbation;
pub mod rate_distortion_compression;
pub mod reference_compiler_ir;
pub mod reproducibility;
pub mod residual;
pub mod semantic_emission_decoupling;
pub mod semantic_state;
pub mod semantic_transitions;
pub mod stage_dag;
