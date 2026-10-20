//! Rust implementation standards validators.
//!
//! These validators check that the Rust source code meets the project's
//! conventions — items that clippy/rustfmt don't cover automatically.

pub mod all_features_build_check;
pub mod alloc_build_check;
pub mod api;
pub mod blanket_impls_exempt;
pub mod bridge_enforcement;
pub mod bridge_namespace_completion;
pub mod calibration_presets_valid;
pub mod const_fn_frontier;
pub mod const_ring_eval_coverage;
pub mod correctness;
pub mod driver_must_use;
pub mod driver_shape;
pub mod ebnf_constraint_decl;
pub mod endpoint_coverage;
pub mod error_trait_completeness;
pub mod escape_hatch_lint;
pub mod feature_flag_layout;
pub mod grammar_surface_coverage;
pub mod grounding_combinator_check;
pub mod host_types_discipline;
pub mod kernel_enforcement;
pub mod libm_dependency;
pub mod multiplication_resolver;
pub mod no_hardcoded_f64;
pub mod no_std_build_check;
pub mod orphan_counts;
pub mod parametric_constraints;
pub mod phantom_tag;
pub mod phase12_no_stubs;
pub mod pipeline_run_threads_input;
pub mod public_api_functional;
pub mod public_api_snapshot;
pub mod resolver_tower;
pub mod style;
pub mod target_doc;
pub mod taxonomy_coverage;
pub mod test_assertion_depth;
pub mod theory_deferred_register;
pub mod trace_byte_layout_pinned;
pub mod uor_foundation_verify_build;
pub mod uor_time_surface;
pub mod verify_trace_round_trip;
pub mod w4_closure;
pub mod witness_scaffold_surface;
pub mod witt_tower_completeness;
