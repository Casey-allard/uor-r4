//! UOR Framework conformance suite.
//!
//! This crate is the workspace-wide standard index and validator. It documents
//! the professional specifications that every component must satisfy and
//! implements automated validators to enforce them.
//!
//! # Conformance Scope
//!
//! | Component | Standard |
//! |-----------|----------|
//! | Rust source | Rust API Guidelines, edition 2021, clippy deny list |
//! | Ontology (JSON-LD) | JSON-LD 1.1, OWL 2 DL |
//! | Ontology (Turtle/N-Triples) | RDF 1.1, Turtle 1.1 |
//! | Ontology (EBNF) | ISO/IEC 14977 EBNF |
//! | Ontology (OWL RDF/XML) | OWL 2 RDF/XML |
//! | Ontology (JSON Schema) | JSON Schema Draft 2020-12 |
//! | Ontology (SHACL shapes) | SHACL W3C shapes |
//! | Instance graphs | SHACL W3C spec |
//! | Documentation | Diataxis framework, completeness, accuracy |
//! | Website | HTML5, WCAG 2.1 AA, CSS |
//!
//! # Entry Point
//!
//! ```no_run
//! use uor_conformance::{WorkspacePaths, run_all};
//! use std::path::PathBuf;
//!
//! let paths = WorkspacePaths {
//!     workspace: PathBuf::from("."),
//!     artifacts: PathBuf::from("public"),
//! };
//! let report = run_all(&paths).expect("Failed to run conformance");
//! assert!(report.all_passed());
//! ```

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

pub mod report;
pub mod tests;
pub mod validators;

pub use report::{ConformanceReport, Severity, TestResult};

/// Paths required by the conformance runner.
pub struct WorkspacePaths {
    /// Root of the Rust workspace (contains Cargo.toml, spec/, clients/, etc.)
    pub workspace: std::path::PathBuf,
    /// Directory containing built artifacts (uor.foundation.jsonld, docs/, etc.)
    pub artifacts: std::path::PathBuf,
}

/// Runs all conformance validators and returns the aggregated report.
///
/// Validators are run in this order:
/// 1. Rust source standards (style, API surface)
/// 2. Ontology inventory (counts must match `uor_ontology::counts`)
/// 3. Ontology JSON-LD 1.1
/// 4. Ontology OWL 2 DL
/// 5. Ontology RDF 1.1 / Turtle 1.1
/// 6. Ontology EBNF grammar
/// 7. SHACL instance conformance
/// 8. Documentation completeness and accuracy
/// 9. Website HTML5, WCAG, CSS, coverage
///
/// # Errors
///
/// Returns an error only if a file system operation fails.
pub fn run_all(paths: &WorkspacePaths) -> anyhow::Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    // 1. Rust source standards
    report.extend(validators::rust::style::validate(&paths.workspace)?);
    report.extend(validators::rust::api::validate(&paths.workspace)?);
    // v0.2.2 W6: public-API snapshot pin (drift detector).
    report.extend(validators::rust::public_api_snapshot::validate(
        &paths.workspace,
    )?);
    // v0.2.2 Phase A: UorTime infrastructure surface check.
    report.extend(validators::rust::uor_time_surface::validate(
        &paths.workspace,
    )?);
    // v0.2.2 Phase B: phantom Tag on Grounded surface check.
    report.extend(validators::rust::phantom_tag::validate(&paths.workspace)?);
    // v0.2.2 Phase C: Witt tower completeness — one marker struct + ring-op
    // impls per `schema:WittLevel` individual.
    report.extend(validators::rust::witt_tower_completeness::validate(
        &paths.workspace,
    )?);
    // v0.2.2 Phase C.4: multiplication resolver surface check.
    report.extend(validators::rust::multiplication_resolver::validate(
        &paths.workspace,
    )?);
    // v0.2.2 Phase D (Q4): parametric constraint surface check.
    report.extend(validators::rust::parametric_constraints::validate(
        &paths.workspace,
    )?);
    // v0.2.2 Phase E: bridge namespace completion check.
    report.extend(validators::rust::bridge_namespace_completion::validate(
        &paths.workspace,
    )?);
    // v0.2.2 Phase F: driver shape check.
    report.extend(validators::rust::driver_shape::validate(&paths.workspace)?);
    // v0.2.2 Phase G: widened const-fn frontier check.
    report.extend(validators::rust::const_fn_frontier::validate(
        &paths.workspace,
    )?);
    // v0.2.2 Phase J: combinator-only grounding check.
    report.extend(validators::rust::grounding_combinator_check::validate(
        &paths.workspace,
    )?);
    // Phase H (target §1.6): libm always-on dependency + transcendentals module.
    report.extend(validators::rust::libm_dependency::validate(
        &paths.workspace,
    )?);
    // Phase G (target §1.5): grammar-surface coverage.
    report.extend(validators::rust::grammar_surface_coverage::validate(
        &paths.workspace,
    )?);
    // Phase D (target §4.2): resolver-tower completion.
    report.extend(validators::rust::resolver_tower::validate(
        &paths.workspace,
    )?);
    // Phase E (target §4.6): bridge namespace enforcement additions.
    report.extend(validators::rust::bridge_enforcement::validate(
        &paths.workspace,
    )?);
    // Phase F (target §4.7): kernel namespace sealed witnesses + closed enumerations.
    report.extend(validators::rust::kernel_enforcement::validate(
        &paths.workspace,
    )?);
    // Phase K (target §4.3 / §9 criterion 1): W4 closure via combinator program.
    report.extend(validators::rust::w4_closure::validate(&paths.workspace)?);
    // Phase L.2 (target §4.5 / §9 criterion 5): const-ring-eval coverage.
    report.extend(validators::rust::const_ring_eval_coverage::validate(
        &paths.workspace,
    )?);
    // Phase M.3 (target §5): driver must-use discipline.
    report.extend(validators::rust::driver_must_use::validate(
        &paths.workspace,
    )?);
    // Correctness Layer 2: behavioral tests invoked as conformance checks.
    // Each behavior_*.rs test contributes one pass/fail line here, so an
    // incorrect endpoint fails conformance.
    report.extend(validators::rust::correctness::validate(&paths.workspace)?);
    // Correctness Layer 3: test-quality gate. Reject vacuous-assertion
    // patterns in behavior_*.rs so weak tests can't slip through.
    report.extend(validators::rust::test_assertion_depth::validate(
        &paths.workspace,
    )?);
    // Correctness Layer 4: endpoint-coverage gate. Every public symbol
    // in public-api.snapshot must map to a behavior test or an audited
    // exemption in endpoint_coverage.toml.
    report.extend(validators::rust::endpoint_coverage::validate(
        &paths.workspace,
    )?);
    // Phase 6 (orphan-closure): Path-4 theory-deferred register parity
    // — every class classified Path4TheoryDeferred has a row in
    // docs/theory_deferred.md with a non-empty research question, and
    // every row corresponds to a Path-4 class.
    report.extend(validators::rust::theory_deferred_register::validate(
        &paths.workspace,
    )?);
    // Phase 7e (orphan-closure): minimum-viable orphan-count validator.
    // Counts `pub trait {Name}<H: HostTypes>` declarations without any
    // `impl {Name}<H> for ...` site in the workspace.
    report.extend(validators::rust::orphan_counts::validate(&paths.workspace)?);
    // Phase 9d (orphan-closure): no_hardcoded_f64 gate. Ensures
    // `foundation/src/**/*.rs` contains zero `: f64` / `-> f64` outside
    // `#[cfg(test)]` blocks; every decimal value flows through
    // `H::Decimal: DecimalTranscendental`.
    report.extend(validators::rust::no_hardcoded_f64::validate(
        &paths.workspace,
    )?);
    // Phase 9e (orphan-closure): HostTypesDiscipline category. Asserts the
    // bounds and impl shape introduced by Phase 9 — DecimalTranscendental
    // supertrait, libm impls for f64 / f32, generic transcendentals dispatch.
    report.extend(validators::rust::host_types_discipline::validate(
        &paths.workspace,
    )?);
    // Phase 10 (orphan-closure): VerifiedMint witness scaffold surface.
    // Asserts every Path-2 class has a Mint{Foo} + Mint{Foo}Inputs<H> +
    // Certificate + OntologyVerifiedMint scaffold and a per-family
    // primitive stub module emitted to foundation/src/.
    report.extend(validators::rust::witness_scaffold_surface::validate(
        &paths.workspace,
    )?);
    // Phase 11c (orphan-closure): blanket_impls.rs presence + banner
    // discipline. Hand-written file lives at
    // `foundation/src/blanket_impls.rs`; `// @codegen-exempt` banner
    // gates emit::write_file's preservation logic.
    report.extend(validators::rust::blanket_impls_exempt::validate(
        &paths.workspace,
    )?);
    // Phase 12 (orphan-closure): no `WITNESS_UNIMPLEMENTED_STUB:*`
    // markers remain in foundation/src/primitives/*.rs. Every verify_*
    // returns Ok(witness) or a typed GenericImpossibilityWitness.
    report.extend(validators::rust::phase12_no_stubs::validate(
        &paths.workspace,
    )?);
    // Phase 13c (orphan-closure): TaxonomyCoverage. Asserts the
    // Phase-0 classification report matches the live classify_all
    // output, and that spec::counts::CLASSIFICATION_* constants are
    // in sync.
    report.extend(validators::rust::taxonomy_coverage::validate(
        &paths.workspace,
    )?);
    // v0.2.2 Phase H: lints + cross-cutting.
    report.extend(validators::rust::feature_flag_layout::validate(
        &paths.workspace,
    )?);
    report.extend(validators::rust::escape_hatch_lint::validate(
        &paths.workspace,
    )?);
    report.extend(validators::rust::no_std_build_check::validate(
        &paths.workspace,
    )?);
    report.extend(validators::rust::alloc_build_check::validate(
        &paths.workspace,
    )?);
    report.extend(validators::rust::all_features_build_check::validate(
        &paths.workspace,
    )?);
    report.extend(validators::rust::uor_foundation_verify_build::validate(
        &paths.workspace,
    )?);
    // v0.2.2 T6.18: calibration preset literals validity.
    report.extend(validators::rust::calibration_presets_valid::validate(
        &paths.workspace,
    )?);
    // v0.2.2 T6.19: pipeline entry points thread H: Hasher.
    report.extend(validators::rust::pipeline_run_threads_input::validate(
        &paths.workspace,
    )?);
    // v0.2.2 T6.20: verify-trace round-trip discipline.
    report.extend(validators::rust::verify_trace_round_trip::validate(
        &paths.workspace,
    )?);
    // v0.2.2 T6.21: trace byte layout pinned.
    report.extend(validators::rust::trace_byte_layout_pinned::validate(
        &paths.workspace,
    )?);
    // v0.2.2 T6.22: error trait completeness.
    report.extend(validators::rust::error_trait_completeness::validate(
        &paths.workspace,
    )?);

    // v0.2.2 structural cross-reference validators (Workstream A):
    // static-snapshot checks of the sealed surface, resolver signature
    // shape, closed enumerations, and trait-shape invariants. Catch
    // structural drift that behavioral tests cannot observe.
    report.extend(validators::rust::target_doc::sealed_type_coverage::validate(&paths.workspace)?);
    report.extend(
        validators::rust::target_doc::resolver_signature_shape::validate(&paths.workspace)?,
    );
    report.extend(
        validators::rust::target_doc::constraint_encoder_completeness::validate(&paths.workspace)?,
    );
    report.extend(validators::rust::target_doc::w4_grounding_closure::validate(&paths.workspace)?);
    report
        .extend(validators::rust::target_doc::spectral_sequence_walk::validate(&paths.workspace)?);

    // 2. Ontology inventory
    report.extend(validators::ontology::inventory::validate(&paths.artifacts)?);

    // 2b. Workspace-level inventory (shapes count)
    report.extend(validators::ontology::inventory::validate_workspace(
        &paths.workspace,
    )?);

    // 2c. Required-property coverage: fail if any `required: true`
    //     property is missing an assertion on any matching individual.
    //     Fast in-memory drift guard — runs before any codegen.
    let ontology = uor_ontology::Ontology::full();
    report.extend(validators::ontology::required_property_coverage::validate(
        ontology,
    )?);

    // 3. JSON-LD 1.1
    report.extend(validators::ontology::jsonld::validate(&paths.artifacts)?);

    // 4. OWL 2 DL (operates on live spec, no file I/O)
    report.extend(validators::ontology::owl::validate());

    // 5. RDF 1.1 / Turtle 1.1
    report.extend(validators::ontology::rdf::validate(&paths.artifacts)?);

    // 5b. EBNF grammar (Amendment 42)
    report.extend(validators::ontology::ebnf::validate(&paths.artifacts)?);

    // 5c. OWL RDF/XML artifact
    report.extend(validators::ontology::owl_xml::validate(&paths.artifacts)?);

    // 5d. JSON Schema artifact
    report.extend(validators::ontology::json_schema::validate(
        &paths.artifacts,
    )?);

    // 5e. SHACL shapes artifact
    report.extend(validators::ontology::shacl_shapes::validate(
        &paths.artifacts,
    )?);

    // 6. SHACL instance conformance
    report.extend(validators::ontology::shacl::validate());

    // 6b. Generated crate conformance
    report.extend(validators::ontology::crate_::validate(&paths.workspace)?);

    // 6b2. Declarative enforcement module
    report.extend(validators::ontology::enforcement::validate(
        &paths.workspace,
    )?);

    // 6c. Standards document counts
    report.extend(validators::ontology::standards::validate(&paths.workspace)?);

    // 7. Documentation
    report.extend(validators::docs::completeness::validate(&paths.artifacts)?);
    report.extend(validators::docs::accuracy::validate(&paths.artifacts)?);
    report.extend(validators::docs::structure::validate(&paths.artifacts)?);
    report.extend(validators::docs::links::validate(&paths.artifacts)?);
    // v0.2.2 W5: ψ vocabulary leak gate (consumer-facing surface).
    report.extend(validators::docs::psi_leakage::validate(&paths.workspace)?);
    // v0.2.2 T1.5 (cleanup): concept page count matches CONCEPT_PAGES constant.
    report.extend(validators::docs::concept_pages_count::validate(
        &paths.workspace,
    )?);
    // v0.2.2 T2.3 (cleanup): EBNF constraint-decl production check.
    report.extend(validators::rust::ebnf_constraint_decl::validate(
        &paths.workspace,
    )?);
    // v0.2.2 T2.0 (cleanup): public API functional verification gate —
    // shells to the foundation and verify-crate test binaries to assert
    // every previously-hardcoded endpoint is functional and input-dependent.
    report.extend(validators::rust::public_api_functional::validate(
        &paths.workspace,
    )?);

    // 8. Website
    report.extend(validators::website::html::validate(&paths.artifacts)?);
    report.extend(validators::website::accessibility::validate(
        &paths.artifacts,
    )?);
    report.extend(validators::website::coverage::validate(&paths.artifacts)?);
    report.extend(validators::website::css::validate(&paths.artifacts)?);
    report.extend(validators::website::links::validate(&paths.artifacts)?);

    // 8b. Website nav structure
    report.extend(validators::website::nav::validate(&paths.artifacts)?);
    // 8c. Website design system
    report.extend(validators::website::design::validate(&paths.artifacts)?);
    // 8d. New page existence
    report.extend(validators::website::pages::validate(&paths.artifacts)?);
    // 8e. Visual elements (SVG)
    report.extend(validators::website::visual::validate(&paths.artifacts)?);
    // 8f. Bootstrap framework integration
    report.extend(validators::website::bootstrap::validate(&paths.artifacts)?);

    // 9. Lean 4 formalization
    report.extend(validators::lean4::structure::validate(&paths.workspace)?);
    report.extend(validators::lean4::build::validate(&paths.workspace)?);
    report.extend(validators::lean4::individual_proof::validate(
        &paths.workspace,
    )?);

    Ok(report)
}

#[cfg(test)]
mod tests_unit {
    use super::*;

    #[test]
    fn spec_inventory_passes() {
        let ontology = uor_ontology::Ontology::full();
        assert_eq!(ontology.namespaces.len(), uor_ontology::counts::NAMESPACES);
        assert_eq!(ontology.class_count(), uor_ontology::counts::CLASSES);
        assert_eq!(ontology.property_count(), uor_ontology::counts::PROPERTIES);
        assert_eq!(
            ontology.individual_count(),
            uor_ontology::counts::INDIVIDUALS
        );
    }

    #[test]
    fn owl_dl_constraints_pass() {
        let report = validators::ontology::owl::validate();
        let failures: Vec<_> = report.results.iter().filter(|r| r.is_failure()).collect();
        assert!(
            failures.is_empty(),
            "OWL 2 DL constraint failures: {:#?}",
            failures
        );
    }

    #[test]
    fn shacl_instances_pass() {
        let report = validators::ontology::shacl::validate();
        let failures: Vec<_> = report.results.iter().filter(|r| r.is_failure()).collect();
        assert!(
            failures.is_empty(),
            "SHACL conformance failures: {:#?}",
            failures
        );
    }
}
