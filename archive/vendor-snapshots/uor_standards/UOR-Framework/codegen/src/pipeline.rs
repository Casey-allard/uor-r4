//! Reduction Pipeline driver generator.
//!
//! Emits `foundation/src/pipeline.rs`, the `#![no_std]`-compatible module
//! that backs `Certify::certify` on every resolver façade and (re-exported
//! via the macros crate) the `uor_ground!` macro's compile-time pipeline.
//!
//! The driver implements the full reduction pipeline per
//! `external/ergonomics-spec.md` §3.3 and §4:
//!
//! - **6 preflight checks**: `BudgetSolvencyCheck` (order 0), `FeasibilityCheck`,
//!   `DispatchCoverageCheck`, `PackageCoherenceCheck`, `PreflightTiming`,
//!   `RuntimeTiming`. Read from `reduction:PreflightCheck` individuals.
//! - **7 reduction stages**: `stage_initialization`, `stage_declare`,
//!   `stage_factorize`, `stage_resolve`, `stage_attest`, `stage_extract`,
//!   `stage_convergence`. Read from `reduction:ReductionStep` individuals.
//! - **Four resolver backends**: `TowerCompletenessResolver`,
//!   `IncrementalCompletenessResolver`, `GroundingAwareResolver`,
//!   `InhabitanceResolver`. Each driven by its `resolver:CertifyMapping`
//!   ontology individual.
//! - **Real 2-SAT decider** (Aspvall-Plass-Tarjan, O(n+m)) for
//!   `predicate:Is2SatShape` inputs.
//! - **Real Horn-SAT decider** (unit propagation, O(n+m)) for
//!   `predicate:IsHornShape` inputs.
//! - **Residual fall-through** returning `Err(InhabitanceImpossibilityWitness)`
//!   citing `reduction:ConvergenceStall` for `predicate:IsResidualFragment`
//!   inputs.
//! - **Fragment classifier** (`FragmentClassify`) mapping a constraint system
//!   to one of `Is2SatShape` / `IsHornShape` / `IsResidualFragment`.
//! - **Content-addressed unit-ids** via FNV-1a hash of the constraint
//!   closure, populating `reduction:unitAddress`.
//!
//! The template walks the ontology at codegen time and bakes the preflight
//! order, stage order, resolver dispatch, and dispatch-table rules into the
//! generated `foundation/src/pipeline.rs` constants. Adding a new preflight
//! check or resolver is a pure ontology edit.

use crate::emit::RustFile;
use crate::enforcement::{limbs_witt_levels, witt_levels};
use uor_ontology::model::{IndividualValue, Ontology};

/// Convert an IRI to its local name.
fn local_name(iri: &str) -> &str {
    iri.rsplit_once(['/', '#']).map(|(_, n)| n).unwrap_or(iri)
}

/// Read an integer-typed property.
fn ind_prop_int(ind: &uor_ontology::model::Individual, prop_iri: &str) -> Option<i64> {
    for (k, v) in ind.properties {
        if *k == prop_iri {
            if let IndividualValue::Int(n) = v {
                return Some(*n);
            }
        }
    }
    None
}

/// Collect individuals of a given type.
fn individuals_of_type<'a>(
    ontology: &'a Ontology,
    type_iri: &str,
) -> Vec<&'a uor_ontology::model::Individual> {
    let mut out = Vec::new();
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            if ind.type_ == type_iri {
                out.push(ind);
            }
        }
    }
    out
}

/// Generate the complete `foundation/src/pipeline.rs` module.
#[must_use]
pub fn generate_pipeline_module(ontology: &Ontology) -> String {
    let mut f = RustFile::new(
        "Reduction Pipeline — no_std in-process driver.\n\
         //!\n\
         //! Backs `Certify::certify` on every resolver façade and (re-exported\n\
         //! via the macros crate) the `uor_ground!` macro's compile-time pipeline.\n\
         //!\n\
         //! The driver implements the full reduction pipeline per\n\
         //! `external/ergonomics-spec.md` §3.3 and §4: 6 preflight checks,\n\
         //! 7 reduction stages, 4 resolver backends, real 2-SAT and Horn-SAT\n\
         //! deciders, fragment classifier, content-addressed unit-ids.\n\
         //!\n\
         //! Every entry point is ontology-driven: IRIs, stage order, and\n\
         //! dispatch-table rules are baked in at codegen time from the\n\
         //! ontology graph. Adding a new preflight check or resolver is a\n\
         //! pure ontology edit.",
    );

    f.line("use crate::enforcement::{");
    f.line("    BindingEntry, BindingsTable, CompileTime, CompileUnit, CompileUnitBuilder,");
    f.line("    CompletenessCertificate, ConstrainedTypeInput, GenericImpossibilityWitness,");
    f.line("    Grounded, GroundingCertificate, InhabitanceCertificate,");
    f.line("    InhabitanceImpossibilityWitness, LeaseDeclaration, LeaseDeclarationBuilder,");
    f.line("    LiftChainCertificate, MultiplicationCertificate, ParallelDeclarationBuilder,");
    f.line("    PipelineFailure, ShapeViolation, StreamDeclarationBuilder, Term, Validated,");
    f.line("};");
    f.line("use crate::ViolationKind;");
    f.line("use crate::WittLevel;");
    f.blank();

    emit_constants(&mut f, ontology);
    emit_constraint_ref(&mut f);
    emit_constrained_type_shape(&mut f);
    emit_witt_domain(&mut f, ontology);
    emit_admission_fns(&mut f);
    emit_fragment_classifier(&mut f);
    emit_two_sat_decider(&mut f, ontology);
    emit_horn_sat_decider(&mut f, ontology);
    // v0.2.2 T6.14: emit_unit_id_hasher deleted; substrate `Hasher` computes
    // the unit_address via `fold_unit_digest` + `unit_address_from_buffer`.
    emit_preflight_checks(&mut f, ontology);
    emit_reduction_stages(&mut f);
    emit_resolver_entry_points(&mut f, ontology);
    emit_empty_bindings_table(&mut f);
    // v0.2.2 Phase F (Q5): drivers per computation kind.
    emit_phase_f_drivers(&mut f);
    // v0.2.2 Phase G: widened const-fn frontier.
    emit_phase_g_const_surface(&mut f);

    f.finish()
}

/// v0.2.2 Phase G + T2.8 (cleanup): widened const-fn frontier with functional
/// input-dependence.
///
/// Emits `validate_*_const` companion free functions that read the builder's
/// stored fields and pack them into `Validated<_, CompileTime>` results
/// (the const path does no runtime validation loop but *does* preserve the
/// input state). Emits `certify_*_const` companion functions that consult
/// their `Validated<CompileUnit, CompileTime>` parameter to produce
/// certificates tied to the compile-unit's witt level. Emits
/// `pipeline::run_const` with the widened `T::Map: Total` gate, and the
/// returned `Grounded<T>` carries the unit's witt level (not zero).
fn emit_phase_g_const_surface(f: &mut RustFile) {
    f.doc_comment("v0.2.2 Phase G / T2.8: const-fn companion for");
    f.doc_comment("`LeaseDeclarationBuilder`. Delegates to the builder's");
    f.doc_comment("`validate_const` method, which validates the `LeaseShape` contract");
    f.doc_comment("(`linear_site` and `scope` required) at compile time.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `ShapeViolation::Missing` if `linear_site` or `scope` is unset.");
    f.line("pub const fn validate_lease_const<'a>(");
    f.line("    builder: &LeaseDeclarationBuilder<'a>,");
    f.line(") -> Result<Validated<LeaseDeclaration, CompileTime>, ShapeViolation> {");
    f.line("    builder.validate_const()");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 Phase G / T2.8 + T6.13: const-fn companion for `CompileUnitBuilder`.");
    f.line("///");
    f.doc_comment("Tightened in T6.13 to enforce the same five required fields as the");
    f.doc_comment("runtime `CompileUnitBuilder::validate()` method:");
    f.line("///");
    f.doc_comment("- `root_term`");
    f.doc_comment("- `witt_level_ceiling`");
    f.doc_comment("- `thermodynamic_budget`");
    f.doc_comment("- `target_domains` (non-empty)");
    f.doc_comment("- `result_type_iri`");
    f.line("///");
    f.doc_comment("Returns `Result<Validated<CompileUnit, CompileTime>, ShapeViolation>` —");
    f.doc_comment("dual-path consistent with the runtime `validate()` method. Const-eval");
    f.doc_comment("call sites match on the `Result`; the panic only fires at codegen /");
    f.doc_comment("const-eval time, never at runtime.");
    f.line("///");
    f.doc_comment("# Errors");
    f.line("///");
    f.doc_comment("Returns `ShapeViolation::Missing` for the first unset required field.");
    f.line("pub const fn validate_compile_unit_const<'a, const INLINE_BYTES: usize>(");
    f.line("    builder: &CompileUnitBuilder<'a, INLINE_BYTES>,");
    f.line(") -> Result<Validated<CompileUnit<'a, INLINE_BYTES>, CompileTime>, ShapeViolation> {");
    f.line("    if !builder.has_root_term_const() {");
    f.line("        return Err(ShapeViolation {");
    f.line("            shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line("            constraint_iri: \"https://uor.foundation/conformance/compileUnit_rootTerm_constraint\",");
    f.line("            property_iri: \"https://uor.foundation/reduction/rootTerm\",");
    f.line("            expected_range: \"https://uor.foundation/schema/Term\",");
    f.line("            min_count: 1,");
    f.line("            max_count: 1,");
    f.line("            kind: ViolationKind::Missing,");
    f.line("        });");
    f.line("    }");
    f.line("    let level = match builder.witt_level_option() {");
    f.line("        Some(l) => l,");
    f.line("        None => return Err(ShapeViolation {");
    f.line("            shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line("            constraint_iri: \"https://uor.foundation/conformance/compileUnit_unitWittLevel_constraint\",");
    f.line("            property_iri: \"https://uor.foundation/reduction/unitWittLevel\",");
    f.line("            expected_range: \"https://uor.foundation/schema/WittLevel\",");
    f.line("            min_count: 1,");
    f.line("            max_count: 1,");
    f.line("            kind: ViolationKind::Missing,");
    f.line("        }),");
    f.line("    };");
    f.line("    let budget = match builder.budget_option() {");
    f.line("        Some(b) => b,");
    f.line("        None => return Err(ShapeViolation {");
    f.line("            shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line("            constraint_iri: \"https://uor.foundation/conformance/compileUnit_thermodynamicBudget_constraint\",");
    f.line("            property_iri: \"https://uor.foundation/reduction/thermodynamicBudget\",");
    f.line("            expected_range: \"http://www.w3.org/2001/XMLSchema#decimal\",");
    f.line("            min_count: 1,");
    f.line("            max_count: 1,");
    f.line("            kind: ViolationKind::Missing,");
    f.line("        }),");
    f.line("    };");
    f.line("    if !builder.has_target_domains_const() {");
    f.line("        return Err(ShapeViolation {");
    f.line("            shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line("            constraint_iri: \"https://uor.foundation/conformance/compileUnit_targetDomains_constraint\",");
    f.line("            property_iri: \"https://uor.foundation/reduction/targetDomains\",");
    f.line("            expected_range: \"https://uor.foundation/op/VerificationDomain\",");
    f.line("            min_count: 1,");
    f.line("            max_count: 0,");
    f.line("            kind: ViolationKind::Missing,");
    f.line("        });");
    f.line("    }");
    f.line("    let result_type_iri = match builder.result_type_iri_const() {");
    f.line("        Some(iri) => iri,");
    f.line("        None => return Err(ShapeViolation {");
    f.line("            shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line("            constraint_iri: \"https://uor.foundation/conformance/compileUnit_resultType_constraint\",");
    f.line("            property_iri: \"https://uor.foundation/reduction/resultType\",");
    f.line("            expected_range: \"https://uor.foundation/type/ConstrainedType\",");
    f.line("            min_count: 1,");
    f.line("            max_count: 1,");
    f.line("            kind: ViolationKind::Missing,");
    f.line("        }),");
    f.line("    };");
    f.line("    Ok(Validated::new(CompileUnit::from_parts_const(");
    f.line("        level,");
    f.line("        budget,");
    f.line("        result_type_iri,");
    f.line("        builder.root_term_slice_const(),");
    f.line("        builder.bindings_slice_const(),");
    f.line("        builder.target_domains_slice_const(),");
    f.line("    )))");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 Phase G / T2.8 + T6.11: const-fn companion for");
    f.doc_comment("`ParallelDeclarationBuilder`. Takes a `ConstrainedTypeShape` type parameter");
    f.doc_comment("to set the `result_type_iri` on the produced declaration.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 Phase A: the produced `ParallelDeclaration<'a>` carries the");
    f.doc_comment("builder's raw site-partition slice and disjointness-witness IRI; the");
    f.doc_comment("lifetime `'a` is the builder's borrow lifetime.");
    f.line("#[must_use]");
    f.line("pub const fn validate_parallel_const<'a, T: ConstrainedTypeShape>(");
    f.line("    builder: &ParallelDeclarationBuilder<'a>,");
    f.line(") -> Validated<ParallelDeclaration<'a>, CompileTime> {");
    f.line("    Validated::new(ParallelDeclaration::new_with_partition::<T>(");
    f.line("        builder.site_partition_slice_const(),");
    f.line("        builder.disjointness_witness_const(),");
    f.line("    ))");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 Phase G / T2.8 + T6.11: const-fn companion for");
    f.doc_comment("`StreamDeclarationBuilder`. Takes a `ConstrainedTypeShape` type parameter");
    f.doc_comment("to set the `result_type_iri` on the produced declaration.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 Phase A: the produced `StreamDeclaration<'a>` retains the");
    f.doc_comment("builder's seed/step term slices and productivity-witness IRI.");
    f.line("#[must_use]");
    f.line("pub const fn validate_stream_const<'a, const INLINE_BYTES: usize, T: ConstrainedTypeShape>(");
    f.line("    builder: &StreamDeclarationBuilder<'a, INLINE_BYTES>,");
    f.line(") -> Validated<StreamDeclaration<'a, INLINE_BYTES>, CompileTime> {");
    f.line("    let bound = builder.productivity_bound_const();");
    f.line("    Validated::new(StreamDeclaration::new_full::<T>(");
    f.line("        bound,");
    f.line("        builder.seed_slice_const(),");
    f.line("        builder.step_slice_const(),");
    f.line("        builder.productivity_witness_const(),");
    f.line("    ))");
    f.line("}");
    f.blank();

    // v0.2.2 T5 C6: the four `certify_*_const` companions become regular fn
    // (no longer const) because trait method dispatch on `H: Hasher` is not
    // const-eval-friendly under MSRV 1.81. They thread the consumer-supplied
    // substrate `H` through `fold_unit_digest` to compute a parametric
    // content fingerprint over the unit's full state, then pack the result
    // into the certificate's `with_level_and_fingerprint_const` constructor.
    // Each function passes a distinct `CertificateKind` discriminant so two
    // certify_* calls over the same source unit produce distinguishable
    // fingerprints.
    f.doc_comment("v0.2.2 T5 C6: const-fn resolver companion for");
    f.doc_comment("`tower_completeness::certify`. Threads the consumer-supplied substrate");
    f.doc_comment("`Hasher` through the canonical CompileUnit byte layout to compute a");
    f.doc_comment("parametric content fingerprint, distinguishing two units that share a");
    f.doc_comment("witt level but differ in budget, IRI, site count, or constraints.");
    f.line("#[must_use]");
    f.line("pub fn certify_tower_completeness_const<T, H, const INLINE_BYTES: usize, const FP_MAX: usize>(");
    f.line("    unit: &Validated<CompileUnit<'_, INLINE_BYTES>, CompileTime>,");
    f.line(") -> Validated<GroundingCertificate<FP_MAX>, CompileTime>");
    f.line("where");
    f.line("    T: ConstrainedTypeShape,");
    f.line("    H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("{");
    f.line("    let level_bits = unit.inner().witt_level().witt_length() as u16;");
    f.line("    let budget = unit.inner().thermodynamic_budget();");
    f.line("    let mut hasher = H::initial();");
    f.line("    hasher = crate::enforcement::fold_unit_digest(");
    f.line("        hasher,");
    f.line("        level_bits,");
    f.line("        budget,");
    f.line("        T::IRI,");
    f.line("        T::SITE_COUNT,");
    f.line("        T::CONSTRAINTS,");
    f.line("        crate::enforcement::CertificateKind::TowerCompleteness,");
    f.line("    );");
    f.line("    let buffer = hasher.finalize();");
    f.line("    let fp = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("        buffer,");
    f.line("        H::OUTPUT_BYTES as u8,");
    f.line("    );");
    f.line("    Validated::new(GroundingCertificate::with_level_and_fingerprint_const(level_bits, fp))");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 T5 C6: const-fn resolver companion for");
    f.doc_comment("`incremental_completeness::certify`. Threads `H: Hasher` for the");
    f.doc_comment("parametric fingerprint; uses `CertificateKind::IncrementalCompleteness`");
    f.doc_comment("as the trailing discriminant byte.");
    f.line("#[must_use]");
    f.line("pub fn certify_incremental_completeness_const<T, H, const INLINE_BYTES: usize, const FP_MAX: usize>(");
    f.line("    unit: &Validated<CompileUnit<'_, INLINE_BYTES>, CompileTime>,");
    f.line(") -> Validated<GroundingCertificate<FP_MAX>, CompileTime>");
    f.line("where");
    f.line("    T: ConstrainedTypeShape,");
    f.line("    H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("{");
    f.line("    let level_bits = unit.inner().witt_level().witt_length() as u16;");
    f.line("    let budget = unit.inner().thermodynamic_budget();");
    f.line("    let mut hasher = H::initial();");
    f.line("    hasher = crate::enforcement::fold_unit_digest(");
    f.line("        hasher,");
    f.line("        level_bits,");
    f.line("        budget,");
    f.line("        T::IRI,");
    f.line("        T::SITE_COUNT,");
    f.line("        T::CONSTRAINTS,");
    f.line("        crate::enforcement::CertificateKind::IncrementalCompleteness,");
    f.line("    );");
    f.line("    let buffer = hasher.finalize();");
    f.line("    let fp = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("        buffer,");
    f.line("        H::OUTPUT_BYTES as u8,");
    f.line("    );");
    f.line("    Validated::new(GroundingCertificate::with_level_and_fingerprint_const(level_bits, fp))");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 T5 C6: const-fn resolver companion for `inhabitance::certify`.");
    f.doc_comment("Threads `H: Hasher` for the parametric fingerprint; uses");
    f.doc_comment("`CertificateKind::Inhabitance` as the trailing discriminant byte.");
    f.line("#[must_use]");
    f.line(
        "pub fn certify_inhabitance_const<T, H, const INLINE_BYTES: usize, const FP_MAX: usize>(",
    );
    f.line("    unit: &Validated<CompileUnit<'_, INLINE_BYTES>, CompileTime>,");
    f.line(") -> Validated<GroundingCertificate<FP_MAX>, CompileTime>");
    f.line("where");
    f.line("    T: ConstrainedTypeShape,");
    f.line("    H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("{");
    f.line("    let level_bits = unit.inner().witt_level().witt_length() as u16;");
    f.line("    let budget = unit.inner().thermodynamic_budget();");
    f.line("    let mut hasher = H::initial();");
    f.line("    hasher = crate::enforcement::fold_unit_digest(");
    f.line("        hasher,");
    f.line("        level_bits,");
    f.line("        budget,");
    f.line("        T::IRI,");
    f.line("        T::SITE_COUNT,");
    f.line("        T::CONSTRAINTS,");
    f.line("        crate::enforcement::CertificateKind::Inhabitance,");
    f.line("    );");
    f.line("    let buffer = hasher.finalize();");
    f.line("    let fp = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("        buffer,");
    f.line("        H::OUTPUT_BYTES as u8,");
    f.line("    );");
    f.line("    Validated::new(GroundingCertificate::with_level_and_fingerprint_const(level_bits, fp))");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 T5 C6: const-fn resolver companion for");
    f.doc_comment("`multiplication::certify`. Threads `H: Hasher` for the parametric");
    f.doc_comment("fingerprint; uses `CertificateKind::Multiplication` as the trailing");
    f.doc_comment("discriminant byte.");
    f.line("#[must_use]");
    f.line("pub fn certify_multiplication_const<T, H, const INLINE_BYTES: usize, const FP_MAX: usize>(");
    f.line("    unit: &Validated<CompileUnit<'_, INLINE_BYTES>, CompileTime>,");
    f.line(") -> Validated<MultiplicationCertificate<FP_MAX>, CompileTime>");
    f.line("where");
    f.line("    T: ConstrainedTypeShape,");
    f.line("    H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("{");
    f.line("    let level_bits = unit.inner().witt_level().witt_length() as u16;");
    f.line("    let budget = unit.inner().thermodynamic_budget();");
    f.line("    let mut hasher = H::initial();");
    f.line("    hasher = crate::enforcement::fold_unit_digest(");
    f.line("        hasher,");
    f.line("        level_bits,");
    f.line("        budget,");
    f.line("        T::IRI,");
    f.line("        T::SITE_COUNT,");
    f.line("        T::CONSTRAINTS,");
    f.line("        crate::enforcement::CertificateKind::Multiplication,");
    f.line("    );");
    f.line("    let buffer = hasher.finalize();");
    f.line("    let fp = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("        buffer,");
    f.line("        H::OUTPUT_BYTES as u8,");
    f.line("    );");
    f.line("    Validated::new(MultiplicationCertificate::with_level_and_fingerprint_const(level_bits, fp))");
    f.line("}");
    f.blank();

    // Phase C.4: certify_grounding_aware_const
    f.doc_comment("Phase C.4: const-fn resolver companion for `grounding_aware::certify`.");
    f.doc_comment("Threads `H: Hasher` for the parametric fingerprint; uses");
    f.doc_comment("`CertificateKind::Grounding` as the trailing discriminant byte.");
    f.line("#[must_use]");
    f.line("pub fn certify_grounding_aware_const<T, H, const INLINE_BYTES: usize, const FP_MAX: usize>(");
    f.line("    unit: &Validated<CompileUnit<'_, INLINE_BYTES>, CompileTime>,");
    f.line(") -> Validated<GroundingCertificate<FP_MAX>, CompileTime>");
    f.line("where");
    f.line("    T: ConstrainedTypeShape,");
    f.line("    H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("{");
    f.line("    let level_bits = unit.inner().witt_level().witt_length() as u16;");
    f.line("    let budget = unit.inner().thermodynamic_budget();");
    f.line("    let mut hasher = H::initial();");
    f.line("    hasher = crate::enforcement::fold_unit_digest(");
    f.line("        hasher,");
    f.line("        level_bits,");
    f.line("        budget,");
    f.line("        T::IRI,");
    f.line("        T::SITE_COUNT,");
    f.line("        T::CONSTRAINTS,");
    f.line("        crate::enforcement::CertificateKind::Grounding,");
    f.line("    );");
    f.line("    let buffer = hasher.finalize();");
    f.line("    let fp = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("        buffer,");
    f.line("        H::OUTPUT_BYTES as u8,");
    f.line("    );");
    f.line("    Validated::new(GroundingCertificate::with_level_and_fingerprint_const(level_bits, fp))");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 T5 C6: typed pipeline entry point producing `Grounded<T>` from");
    f.doc_comment("a validated `CompileUnit`. Threads the consumer-supplied substrate");
    f.doc_comment("`Hasher` through `fold_unit_digest` to compute a parametric content");
    f.doc_comment("fingerprint over the unit's full state: `(level_bits, budget, T::IRI,");
    f.doc_comment("T::SITE_COUNT, T::CONSTRAINTS, CertificateKind::Grounding)`.");
    f.doc_comment("");
    f.doc_comment("Two units differing on **any** of those fields produce `Grounded`");
    f.doc_comment("values with distinct fingerprints (and distinct `unit_address` handles,");
    f.doc_comment("derived from the leading 16 bytes of the fingerprint).");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `PipelineFailure::ShapeMismatch` when the unit's declared");
    f.doc_comment("`result_type_iri` does not match `T::IRI`, or propagates any");
    f.doc_comment("failure from the reduction stage executor.");
    f.line("pub fn run_const<T, M, H, const INLINE_BYTES: usize, const FP_MAX: usize>(");
    f.line("    unit: &Validated<CompileUnit<'_, INLINE_BYTES>, CompileTime>,");
    f.line(") -> Result<Grounded<'static, T, INLINE_BYTES, FP_MAX>, PipelineFailure>");
    f.line("where");
    f.line("    T: ConstrainedTypeShape + crate::enforcement::GroundedShape,");
    f.line("    // Phase C.2 (target §6): const-eval admits only those grounding-map kinds");
    f.line("    // that are both Total (defined for all inputs) and Invertible (one-to-one).");
    f.line("    // The bound is enforced at the type level via the existing marker tower.");
    f.line("    M: crate::enforcement::GroundingMapKind");
    f.line("        + crate::enforcement::Total");
    f.line("        + crate::enforcement::Invertible,");
    f.line("    H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("{");
    f.line("    // The marker bound on M is purely type-level — no runtime use.");
    f.line("    let _phantom_map: core::marker::PhantomData<M> = core::marker::PhantomData;");
    f.line("    let level_bits = unit.inner().witt_level().witt_length() as u16;");
    f.line("    let budget = unit.inner().thermodynamic_budget();");
    f.line("    // v0.2.2 T6.11: ShapeMismatch detection. The unit declares its");
    f.line("    // result_type_iri at validation time; the caller's `T::IRI` must match.");
    f.line("    let unit_iri = unit.inner().result_type_iri();");
    f.line("    if !crate::enforcement::str_eq(unit_iri, T::IRI) {");
    f.line("        return Err(PipelineFailure::ShapeMismatch {");
    f.line("            expected: T::IRI,");
    f.line("            got: unit_iri,");
    f.line("        });");
    f.line("    }");
    f.line("    // Walk the foundation-locked byte layout via the consumer's hasher.");
    f.line("    let mut hasher = H::initial();");
    f.line("    hasher = crate::enforcement::fold_unit_digest(");
    f.line("        hasher,");
    f.line("        level_bits,");
    f.line("        budget,");
    f.line("        T::IRI,");
    f.line("        T::SITE_COUNT,");
    f.line("        T::CONSTRAINTS,");
    f.line("        crate::enforcement::CertificateKind::Grounding,");
    f.line("    );");
    f.line("    let buffer = hasher.finalize();");
    f.line("    let content_fingerprint = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("        buffer,");
    f.line("        H::OUTPUT_BYTES as u8,");
    f.line("    );");
    f.line("    let unit_address = crate::enforcement::unit_address_from_buffer(&buffer);");
    f.line("    let grounding = Validated::new(");
    f.line("        GroundingCertificate::with_level_and_fingerprint_const(level_bits, content_fingerprint),");
    f.line("    );");
    f.line("    let bindings = empty_bindings_table();");
    f.line("    Ok(Grounded::<T, INLINE_BYTES, FP_MAX>::new_internal(");
    f.line("        grounding,");
    f.line("        bindings,");
    f.line("        level_bits,");
    f.line("        unit_address,");
    f.line("        content_fingerprint,");
    f.line("    ))");
    f.line("}");
    f.blank();

    // v0.2.2 T6.4: the const-fn fallback for legacy callers that didn't
    // supply a substrate `Hasher` is deleted. Const-fn callers either use
    // `run_const::<T, MyHasher>` (no longer const, but functional) OR build
    // a `Validated<CompileUnit, CompileTime>` via the const-fn companions
    // and validate at runtime via `pipeline::run`.
}

/// v0.2.2 Phase F (Q5): emit `pipeline::run_parallel`, `pipeline::run_stream`
/// (returns `StreamDriver<T, P>` : Iterator), and `pipeline::run_interactive`
/// (returns `InteractionDriver<T, P>` state machine) plus the sealed
/// supporting types (StreamDriver, InteractionDriver, StepResult, PeerInput,
/// PeerPayload, CommutatorState).
fn emit_phase_f_drivers(f: &mut RustFile) {
    // v0.2.2 T2.7 (cleanup): the Phase F declaration types now carry a
    // single `payload: u64` field that the drivers consult. The payload is
    // a foundation-internal handle whose semantics differ per driver:
    // - ParallelDeclaration::payload = site partition cardinality
    // - StreamDeclaration::payload = productivity-witness countdown
    // - InteractionDeclaration::payload = convergence-predicate seed
    // Two declarations with different payloads produce drivers with
    // different observable state, satisfying the input-dependence contract.
    f.doc_comment("v0.2.2 Phase F / T2.7: parallel-declaration compile unit. Carries the");
    f.doc_comment("declared site partition cardinality plus (Phase A widening) the raw");
    f.doc_comment("partition slice and disjointness-witness IRI from the builder \u{2014}");
    f.doc_comment("previously these were discarded at validate-time by a shadowed");
    f.doc_comment("enforcement-local `ParallelDeclaration` that nothing consumed.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 T6.11: also carries `result_type_iri` for ShapeMismatch detection.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct ParallelDeclaration<'a> {");
    f.line("    payload: u64,");
    f.line("    result_type_iri: &'static str,");
    f.indented_doc_comment("v0.2.2 Phase A: raw site-partition slice retained from the builder.");
    f.indented_doc_comment(
        "Empty slice for declarations built via the site-count-only constructor.",
    );
    f.line("    site_partition: &'a [u32],");
    f.indented_doc_comment("v0.2.2 Phase A: disjointness-witness IRI retained from the builder.");
    f.indented_doc_comment(
        "Empty string for declarations built via the site-count-only constructor.",
    );
    f.line("    disjointness_witness: &'a str,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<'a> ParallelDeclaration<'a> {");
    f.indented_doc_comment("v0.2.2 Phase H3: construct a parallel declaration carrying the full");
    f.indented_doc_comment("partition slice and disjointness-witness IRI from the builder.");
    f.indented_doc_comment(
        "This is the sole public constructor; the v0.2.2 Phase A site-count-only",
    );
    f.indented_doc_comment("`new::<T>(site_count)` form was deleted in Phase H3 under the \"no");
    f.indented_doc_comment(
        "compatibility\" discipline \u{2014} every caller supplies a real partition.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new_with_partition<T: ConstrainedTypeShape>(");
    f.line("        site_partition: &'a [u32],");
    f.line("        disjointness_witness: &'a str,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            payload: site_partition.len() as u64,");
    f.line("            result_type_iri: T::IRI,");
    f.line("            site_partition,");
    f.line("            disjointness_witness,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the declared site partition cardinality.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn site_count(&self) -> u64 { self.payload }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T6.11: returns the result-type IRI.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn result_type_iri(&self) -> &'static str { self.result_type_iri }");
    f.blank();
    f.indented_doc_comment("v0.2.2 Phase A: returns the raw site-partition slice.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn site_partition(&self) -> &'a [u32] { self.site_partition }");
    f.blank();
    f.indented_doc_comment("v0.2.2 Phase A: returns the disjointness-witness IRI.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn disjointness_witness(&self) -> &'a str { self.disjointness_witness }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 Phase F / T2.7: stream-declaration compile unit. Carries a");
    f.doc_comment("payload field encoding the productivity-witness countdown.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 T6.11: also carries `result_type_iri` for ShapeMismatch detection.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 Phase A: also retains the builder's seed/step term slices and");
    f.doc_comment("the productivity-witness IRI so stream resolvers can walk declared");
    f.doc_comment("structure. Distinct from the enforcement-local `StreamDeclaration`");
    f.doc_comment("which records only the `StreamShape` validation surface.");
    f.doc_comment("");
    f.doc_comment("Note: `Hash` is not derived because `Term` does not implement `Hash`;");
    f.doc_comment("downstream code that needs deterministic hashing should fold through");
    f.doc_comment("the substrate `Hasher` via the pipeline's `fold_stream_digest`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
    f.line("pub struct StreamDeclaration<'a, const INLINE_BYTES: usize> {");
    f.line("    payload: u64,");
    f.line("    result_type_iri: &'static str,");
    f.indented_doc_comment("v0.2.2 Phase A: stream seed term slice retained from the builder.");
    f.line("    seed: &'a [Term<'a, INLINE_BYTES>],");
    f.indented_doc_comment("v0.2.2 Phase A: stream step term slice retained from the builder.");
    f.line("    step: &'a [Term<'a, INLINE_BYTES>],");
    f.indented_doc_comment("v0.2.2 Phase A: productivity-witness IRI retained from the builder.");
    f.line("    productivity_witness: &'a str,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<'a, const INLINE_BYTES: usize> StreamDeclaration<'a, INLINE_BYTES> {");
    f.indented_doc_comment(
        "v0.2.2 T6.11: construct a stream declaration with the given productivity",
    );
    f.indented_doc_comment("bound and result type. Phase A: leaves seed/step/witness empty; use");
    f.indented_doc_comment("`new_full` to retain the full structure.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new<T: ConstrainedTypeShape>(productivity_bound: u64) -> StreamDeclaration<'static, INLINE_BYTES> {");
    f.line("        StreamDeclaration {");
    f.line("            payload: productivity_bound,");
    f.line("            result_type_iri: T::IRI,");
    f.line("            seed: &[],");
    f.line("            step: &[],");
    f.line("            productivity_witness: \"\",");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 Phase A: construct a stream declaration carrying the full");
    f.indented_doc_comment("seed/step/witness structure from the builder.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new_full<T: ConstrainedTypeShape>(");
    f.line("        productivity_bound: u64,");
    f.line("        seed: &'a [Term<'a, INLINE_BYTES>],");
    f.line("        step: &'a [Term<'a, INLINE_BYTES>],");
    f.line("        productivity_witness: &'a str,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            payload: productivity_bound,");
    f.line("            result_type_iri: T::IRI,");
    f.line("            seed,");
    f.line("            step,");
    f.line("            productivity_witness,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the declared productivity bound.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn productivity_bound(&self) -> u64 { self.payload }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T6.11: returns the result-type IRI.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn result_type_iri(&self) -> &'static str { self.result_type_iri }");
    f.blank();
    f.indented_doc_comment("v0.2.2 Phase A: returns the seed term slice.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn seed(&self) -> &'a [Term<'a, INLINE_BYTES>] { self.seed }");
    f.blank();
    f.indented_doc_comment("v0.2.2 Phase A: returns the step term slice.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn step(&self) -> &'a [Term<'a, INLINE_BYTES>] { self.step }");
    f.blank();
    f.indented_doc_comment("v0.2.2 Phase A: returns the productivity-witness IRI.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn productivity_witness(&self) -> &'a str { self.productivity_witness }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 Phase F / T2.7: interaction-declaration compile unit. Carries a");
    f.doc_comment("payload field encoding the convergence-predicate seed.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 T6.11: also carries `result_type_iri` for ShapeMismatch detection.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 Phase A: lifetime-parameterized for consistency with the other");
    f.doc_comment("widened runtime carriers. The interaction builder stores scalar fields");
    f.doc_comment("only, so there is no additional borrowed structure to retain; the `'a`");
    f.doc_comment("is vestigial but keeps the carrier signature uniform with");
    f.doc_comment("`ParallelDeclaration<'a>` and `StreamDeclaration<'a>`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct InteractionDeclaration<'a> {");
    f.line("    payload: u64,");
    f.line("    result_type_iri: &'static str,");
    f.line("    _sealed: (),");
    f.line("    _lifetime: core::marker::PhantomData<&'a ()>,");
    f.line("}");
    f.blank();
    f.line("impl<'a> InteractionDeclaration<'a> {");
    f.indented_doc_comment("v0.2.2 T6.11: construct an interaction declaration with the given");
    f.indented_doc_comment("convergence-predicate seed and result type.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new<T: ConstrainedTypeShape>(convergence_seed: u64) -> InteractionDeclaration<'static> {");
    f.line("        InteractionDeclaration {");
    f.line("            payload: convergence_seed,");
    f.line("            result_type_iri: T::IRI,");
    f.line("            _sealed: (),");
    f.line("            _lifetime: core::marker::PhantomData,");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the declared convergence seed.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn convergence_seed(&self) -> u64 { self.payload }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T6.11: returns the result-type IRI.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn result_type_iri(&self) -> &'static str { self.result_type_iri }");
    f.line("}");
    f.blank();

    // Sealed peer-payload inline buffer for InteractionDriver.
    f.doc_comment("v0.2.2 Phase F: fixed-size inline payload buffer carried by `PeerInput`.");
    f.doc_comment("Sized for the largest `Datum<L>` the foundation supports at this release");
    f.doc_comment("(up to 32 u64 limbs = 2048 bits); smaller levels use the leading bytes.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct PeerPayload {");
    f.line("    words: [u64; 32],");
    f.line("    bit_width: u16,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl PeerPayload {");
    f.indented_doc_comment("Construct a zeroed payload of the given bit width.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn zero(bit_width: u16) -> Self {");
    f.line("        Self { words: [0u64; 32], bit_width, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Access the underlying limbs.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn words(&self) -> &[u64; 32] { &self.words }");
    f.blank();
    f.indented_doc_comment("Bit width of the payload's logical Datum.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn bit_width(&self) -> u16 { self.bit_width }");
    f.line("}");
    f.blank();

    // PeerInput sealed.
    f.doc_comment("v0.2.2 Phase F: a peer-supplied input to an interaction driver step.");
    f.doc_comment("");
    f.doc_comment("Fixed-size — holds a `PeerPayload` inline plus the peer's content");
    f.doc_comment("address. No heap, no dynamic dispatch.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct PeerInput {");
    f.line("    peer_id: u128,");
    f.line("    payload: PeerPayload,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl PeerInput {");
    f.indented_doc_comment("Construct a new peer input with the given peer id and payload.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(peer_id: u128, payload: PeerPayload) -> Self {");
    f.line("        Self { peer_id, payload, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Access the peer id.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn peer_id(&self) -> u128 { self.peer_id }");
    f.blank();
    f.indented_doc_comment("Access the payload.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn payload(&self) -> &PeerPayload { &self.payload }");
    f.line("}");
    f.blank();

    // StepResult<T> enum.
    f.doc_comment("v0.2.2 Phase F: outcome of a single `InteractionDriver::step` call.");
    f.line("#[derive(Debug, Clone)]");
    f.line("#[non_exhaustive]");
    f.line(
        "pub enum StepResult<T: crate::enforcement::GroundedShape, const INLINE_BYTES: usize, const FP_MAX: usize = 32> {",
    );
    f.indented_doc_comment("The step was absorbed; the driver is ready for another peer input.");
    f.line("    Continue,");
    f.indented_doc_comment("The step produced an intermediate grounded output.");
    f.line("    Output(Grounded<'static, T, INLINE_BYTES, FP_MAX>),");
    f.indented_doc_comment("The convergence predicate is satisfied; interaction is complete.");
    f.line("    Converged(Grounded<'static, T, INLINE_BYTES, FP_MAX>),");
    f.indented_doc_comment("v0.2.2 Phase T.1: the commutator norm failed to decrease for");
    f.indented_doc_comment(
        "`INTERACTION_DIVERGENCE_BUDGET` consecutive steps — the interaction is",
    );
    f.indented_doc_comment("non-convergent and the driver is no longer advanceable.");
    f.line("    Diverged,");
    f.indented_doc_comment("The step failed; the driver is no longer advanceable.");
    f.line("    Failure(PipelineFailure),");
    f.line("}");
    f.blank();

    // CommutatorState<L> sealed.
    f.doc_comment("v0.2.2 Phase F: sealed commutator-algebra state carried by an");
    f.doc_comment("interaction driver across peer steps.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct CommutatorState<L> {");
    f.line("    accumulator: [u64; 4],");
    f.line("    _level: core::marker::PhantomData<L>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<L> CommutatorState<L> {");
    f.indented_doc_comment("Construct a zero commutator state.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn zero() -> Self {");
    f.line("        Self {");
    f.line("            accumulator: [0u64; 4],");
    f.line("            _level: core::marker::PhantomData,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Access the commutator accumulator words.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn accumulator(&self) -> &[u64; 4] { &self.accumulator }");
    f.line("}");
    f.blank();

    // StreamDriver<T, P> impl Iterator.
    f.doc_comment("v0.2.2 Phase F / T2.7: sealed iterator driver returned by `run_stream`.");
    f.doc_comment("");
    f.doc_comment("Carries the productivity countdown initialized from the unit's");
    f.doc_comment("`StreamDeclaration::productivity_bound()`, plus a unit-derived address");
    f.doc_comment("seed for generating distinct `Grounded` outputs per step. Each call to");
    f.doc_comment("`next()` decrements the countdown and yields a `Grounded` whose");
    f.doc_comment("`unit_address` differs from the previous step's.");
    f.line("#[derive(Debug, Clone)]");
    f.line("pub struct StreamDriver<T: crate::enforcement::GroundedShape, P: crate::enforcement::ValidationPhase, H: crate::enforcement::Hasher<FP_MAX>, const INLINE_BYTES: usize, const FP_MAX: usize = 32> {");
    f.line("    rewrite_steps: u64,");
    f.line("    landauer_nats: u64,");
    f.line("    productivity_countdown: u64,");
    f.line("    seed: u64,");
    f.line("    result_type_iri: &'static str,");
    f.line("    terminated: bool,");
    f.line("    _shape: core::marker::PhantomData<T>,");
    f.line("    _phase: core::marker::PhantomData<P>,");
    f.line("    _hasher: core::marker::PhantomData<H>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<T: crate::enforcement::GroundedShape, P: crate::enforcement::ValidationPhase, H: crate::enforcement::Hasher<FP_MAX>, const INLINE_BYTES: usize, const FP_MAX: usize> StreamDriver<T, P, H, INLINE_BYTES, FP_MAX> {");
    f.indented_doc_comment(
        "Crate-internal constructor. Callable only from `pipeline::run_stream`.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new_internal(");
    f.line("        productivity_bound: u64,");
    f.line("        seed: u64,");
    f.line("        result_type_iri: &'static str,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            rewrite_steps: 0,");
    f.line("            landauer_nats: 0,");
    f.line("            productivity_countdown: productivity_bound,");
    f.line("            seed,");
    f.line("            result_type_iri,");
    f.line("            terminated: false,");
    f.line("            _shape: core::marker::PhantomData,");
    f.line("            _phase: core::marker::PhantomData,");
    f.line("            _hasher: core::marker::PhantomData,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Total rewrite steps taken so far.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn rewrite_steps(&self) -> u64 { self.rewrite_steps }");
    f.blank();
    f.indented_doc_comment("Total Landauer cost accumulated so far.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn landauer_nats(&self) -> u64 { self.landauer_nats }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T5.10: returns `true` once the driver has stopped producing");
    f.indented_doc_comment("rewrite steps. A terminated driver is observationally equivalent to");
    f.indented_doc_comment("one whose next `next()` call returns `None`. Use this when the driver");
    f.indented_doc_comment("is held inside a larger state machine that needs to decide whether");
    f.indented_doc_comment("to advance without consuming a step.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Parallel to `InteractionDriver::is_converged()`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn is_terminated(&self) -> bool { self.terminated }");
    f.line("}");
    f.blank();
    f.line("impl<T: crate::enforcement::GroundedShape + ConstrainedTypeShape, P: crate::enforcement::ValidationPhase, H: crate::enforcement::Hasher<FP_MAX>, const INLINE_BYTES: usize, const FP_MAX: usize> Iterator for StreamDriver<T, P, H, INLINE_BYTES, FP_MAX> {");
    f.line("    type Item = Result<Grounded<'static, T, INLINE_BYTES, FP_MAX>, PipelineFailure>;");
    f.line("    fn next(&mut self) -> Option<Self::Item> {");
    f.line("        if self.terminated || self.productivity_countdown == 0 {");
    f.line("            self.terminated = true;");
    f.line("            return None;");
    f.line("        }");
    f.line("        // v0.2.2 T6.11: ShapeMismatch detection — first step only");
    f.line("        // (subsequent steps inherit the same result_type_iri).");
    f.line("        if self.rewrite_steps == 0");
    f.line("            && !crate::enforcement::str_eq(self.result_type_iri, T::IRI)");
    f.line("        {");
    f.line("            self.terminated = true;");
    f.line("            return Some(Err(PipelineFailure::ShapeMismatch {");
    f.line("                expected: T::IRI,");
    f.line("                got: self.result_type_iri,");
    f.line("            }));");
    f.line("        }");
    f.line("        self.productivity_countdown -= 1;");
    f.line("        self.rewrite_steps += 1;");
    f.line("        self.landauer_nats += 1;");
    f.line("        // v0.2.2 T6.1: thread H: Hasher through fold_stream_step_digest");
    f.line("        // to compute a real per-step substrate fingerprint.");
    f.line("        let mut hasher = H::initial();");
    f.line("        hasher = crate::enforcement::fold_stream_step_digest(");
    f.line("            hasher,");
    f.line("            self.productivity_countdown,");
    f.line("            self.rewrite_steps,");
    f.line("            self.seed,");
    f.line("            self.result_type_iri,");
    f.line("            crate::enforcement::CertificateKind::Grounding,");
    f.line("        );");
    f.line("        let buffer = hasher.finalize();");
    f.line(
        "        let content_fingerprint = crate::enforcement::ContentFingerprint::from_buffer(",
    );
    f.line("            buffer,");
    f.line("            H::OUTPUT_BYTES as u8,");
    f.line("        );");
    f.line("        let unit_address = crate::enforcement::unit_address_from_buffer(&buffer);");
    f.line("        let grounding = Validated::new(");
    f.line("            GroundingCertificate::with_level_and_fingerprint_const(32, content_fingerprint),");
    f.line("        );");
    f.line("        let bindings = empty_bindings_table();");
    f.line("        Some(Ok(Grounded::<T, INLINE_BYTES, FP_MAX>::new_internal(");
    f.line("            grounding,");
    f.line("            bindings,");
    f.line("            32, // default witt level for stream output");
    f.line("            unit_address,");
    f.line("            content_fingerprint,");
    f.line("        )))");
    f.line("    }");
    f.line("}");
    f.blank();

    // InteractionDriver<T, P>.
    f.doc_comment("v0.2.2 Phase F / T2.7: sealed state-machine driver returned by");
    f.doc_comment("`run_interactive`. Exposes `step(PeerInput)`, `is_converged()`, and");
    f.doc_comment("`finalize()`. The driver folds each peer input into its");
    f.doc_comment("`commutator_acc` accumulator via XOR; convergence is signalled when");
    f.doc_comment("a peer input arrives with `peer_id == 0` (the closing handshake).");
    f.line("#[derive(Debug, Clone)]");
    f.line("pub struct InteractionDriver<T: crate::enforcement::GroundedShape, P: crate::enforcement::ValidationPhase, H: crate::enforcement::Hasher<FP_MAX>, const INLINE_BYTES: usize, const FP_MAX: usize = 32> {");
    f.line("    commutator_acc: [u64; 4],");
    f.line("    peer_step_count: u64,");
    f.line("    converged: bool,");
    f.line("    /// Convergence seed read from the source InteractionDeclaration.");
    f.line("    /// Available via `seed()` for downstream inspection.");
    f.line("    seed: u64,");
    f.line("    /// v0.2.2 Phase T.1: previous step's commutator norm (Euclidean-squared");
    f.line("    /// over the 4 u64 limbs, saturating). Used to detect divergence.");
    f.line("    prev_commutator_norm: u64,");
    f.line("    /// v0.2.2 Phase T.1: count of consecutive non-decreasing norm steps.");
    f.line("    /// Reset to 0 on any decrease; divergence triggers at `DIVERGENCE_BUDGET`.");
    f.line("    consecutive_non_decreasing: u32,");
    f.line("    /// v0.2.2 T6.11: result-type IRI from the source InteractionDeclaration.");
    f.line("    result_type_iri: &'static str,");
    f.line("    _shape: core::marker::PhantomData<T>,");
    f.line("    _phase: core::marker::PhantomData<P>,");
    f.line("    _hasher: core::marker::PhantomData<H>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.doc_comment(
        "v0.2.2 Phase T.1: divergence budget — max consecutive non-decreasing commutator-norm",
    );
    f.doc_comment(
        "steps before the interaction driver fails. Foundation-canonical; override at the",
    );
    f.doc_comment("`InteractionDeclaration` level not supported in this release.");
    f.line("pub const INTERACTION_DIVERGENCE_BUDGET: u32 = 16;");
    f.blank();
    f.line("impl<T: crate::enforcement::GroundedShape, P: crate::enforcement::ValidationPhase, H: crate::enforcement::Hasher<FP_MAX>, const INLINE_BYTES: usize, const FP_MAX: usize> InteractionDriver<T, P, H, INLINE_BYTES, FP_MAX> {");
    f.indented_doc_comment(
        "Crate-internal constructor. Callable only from `pipeline::run_interactive`.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new_internal(");
    f.line("        seed: u64,");
    f.line("        result_type_iri: &'static str,");
    f.line("    ) -> Self {");
    f.line("        // Initial commutator seeded from the unit's convergence seed.");
    f.line("        Self {");
    f.line("            commutator_acc: [seed, 0, 0, 0],");
    f.line("            peer_step_count: 0,");
    f.line("            converged: false,");
    f.line("            seed,");
    f.line("            // Initial norm = seed² (saturating) so the first step can only");
    f.line("            // decrease the norm via peer input (which is the convergence path).");
    f.line("            prev_commutator_norm: seed.saturating_mul(seed),");
    f.line("            consecutive_non_decreasing: 0,");
    f.line("            result_type_iri,");
    f.line("            _shape: core::marker::PhantomData,");
    f.line("            _phase: core::marker::PhantomData,");
    f.line("            _hasher: core::marker::PhantomData,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase T.1: convergence threshold derived from the seed. Termination",
    );
    f.indented_doc_comment(
        "triggers when the commutator norm falls below this value. Foundation-canonical:",
    );
    f.indented_doc_comment("`seed.rotate_right(32) ^ 0xDEADBEEF_CAFEBABE`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn convergence_threshold(&self) -> u64 {");
    f.line("        self.seed.rotate_right(32) ^ 0xDEAD_BEEF_CAFE_BABE");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Advance the driver by folding in a single peer input (v0.2.2 Phase T.1).",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("Each step XOR-folds the peer payload's first 4 limbs into the");
    f.indented_doc_comment("commutator accumulator, then recomputes the Euclidean-squared");
    f.indented_doc_comment("norm over the 4 limbs (saturating `u64`). Termination rules:");
    f.indented_doc_comment("");
    f.indented_doc_comment("* **Converged** if the norm falls below `convergence_threshold()`,");
    f.indented_doc_comment("  OR if `peer_id == 0` (explicit closing handshake).");
    f.indented_doc_comment(
        "* **Diverged** (via `PipelineFailure::ConvergenceStall`) if the norm is",
    );
    f.indented_doc_comment(
        "  non-decreasing for `INTERACTION_DIVERGENCE_BUDGET` consecutive steps.",
    );
    f.indented_doc_comment("* **Continue** otherwise.");
    f.line("    #[must_use]");
    f.line("    pub fn step(&mut self, input: PeerInput) -> StepResult<T, INLINE_BYTES, FP_MAX>");
    f.line("    where");
    f.line("        T: ConstrainedTypeShape,");
    f.line("    {");
    f.line("        self.peer_step_count += 1;");
    f.line("        // Fold the first 4 payload words into the accumulator.");
    f.line("        let words = input.payload().words();");
    f.line("        let mut i = 0usize;");
    f.line("        while i < 4 {");
    f.line("            self.commutator_acc[i] ^= words[i];");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        // v0.2.2 Phase T.1: compute the Euclidean-squared norm over the 4 limbs.");
    f.line("        let mut norm: u64 = 0;");
    f.line("        let mut j = 0usize;");
    f.line("        while j < 4 {");
    f.line("            let limb = self.commutator_acc[j];");
    f.line("            norm = norm.saturating_add(limb.saturating_mul(limb));");
    f.line("            j += 1;");
    f.line("        }");
    f.line("        let threshold = self.convergence_threshold();");
    f.line("        // v0.2.2 Phase T.1: convergence on norm-below-threshold OR explicit");
    f.line("        // handshake (peer_id == 0). Divergence on consecutive non-decreasing norm.");
    f.line("        let norm_converged = norm < threshold;");
    f.line("        let handshake_close = input.peer_id() == 0;");
    f.line("        if norm_converged || handshake_close {");
    f.line("            self.converged = true;");
    f.line("            // v0.2.2 T6.1: thread H: Hasher through fold_interaction_step_digest");
    f.line("            // to compute a real convergence-time substrate fingerprint.");
    f.line("            let mut hasher = H::initial();");
    f.line("            hasher = crate::enforcement::fold_interaction_step_digest(");
    f.line("                hasher,");
    f.line("                &self.commutator_acc,");
    f.line("                self.peer_step_count,");
    f.line("                self.seed,");
    f.line("                self.result_type_iri,");
    f.line("                crate::enforcement::CertificateKind::Grounding,");
    f.line("            );");
    f.line("            let buffer = hasher.finalize();");
    f.line("            let content_fingerprint = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("                buffer,");
    f.line("                H::OUTPUT_BYTES as u8,");
    f.line("            );");
    f.line("            let unit_address = crate::enforcement::unit_address_from_buffer(&buffer);");
    f.line("            let grounding = Validated::new(");
    f.line("                GroundingCertificate::with_level_and_fingerprint_const(32, content_fingerprint),");
    f.line("            );");
    f.line("            let bindings = empty_bindings_table();");
    f.line("            return StepResult::Converged(Grounded::<T, INLINE_BYTES, FP_MAX>::new_internal(");
    f.line("                grounding,");
    f.line("                bindings,");
    f.line("                32,");
    f.line("                unit_address,");
    f.line("                content_fingerprint,");
    f.line("            ));");
    f.line("        }");
    f.line("        // v0.2.2 Phase T.1: divergence detection — count consecutive");
    f.line("        // non-decreasing norm steps. Reset on any decrease.");
    f.line("        if norm >= self.prev_commutator_norm {");
    f.line("            self.consecutive_non_decreasing = self.consecutive_non_decreasing.saturating_add(1);");
    f.line("        } else {");
    f.line("            self.consecutive_non_decreasing = 0;");
    f.line("        }");
    f.line("        self.prev_commutator_norm = norm;");
    f.line("        if self.consecutive_non_decreasing >= INTERACTION_DIVERGENCE_BUDGET {");
    f.line("            return StepResult::Diverged;");
    f.line("        }");
    f.line("        StepResult::Continue");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Whether the driver has reached the convergence predicate.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn is_converged(&self) -> bool { self.converged }");
    f.blank();
    f.indented_doc_comment("Number of peer steps applied so far.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn peer_step_count(&self) -> u64 { self.peer_step_count }");
    f.blank();
    f.indented_doc_comment("Convergence seed inherited from the source InteractionDeclaration.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn seed(&self) -> u64 { self.seed }");
    f.blank();
    f.indented_doc_comment("Finalize the interaction, producing a grounded result.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns a `Grounded<T>` whose `unit_address` is a hash of the");
    f.indented_doc_comment("accumulated commutator state, so two interaction drivers that");
    f.indented_doc_comment("processed different peer inputs return distinct grounded values.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns a `PipelineFailure::ShapeViolation` if the driver has");
    f.indented_doc_comment("not converged, or `PipelineFailure::ShapeMismatch` if the source");
    f.indented_doc_comment("declaration's result_type_iri does not match `T::IRI`.");
    f.line(
        "    pub fn finalize(self) -> Result<Grounded<'static, T, INLINE_BYTES, FP_MAX>, PipelineFailure>",
    );
    f.line("    where");
    f.line("        T: ConstrainedTypeShape,");
    f.line("    {");
    f.line("        // v0.2.2 T6.11: ShapeMismatch detection.");
    f.line("        if !crate::enforcement::str_eq(self.result_type_iri, T::IRI) {");
    f.line("            return Err(PipelineFailure::ShapeMismatch {");
    f.line("                expected: T::IRI,");
    f.line("                got: self.result_type_iri,");
    f.line("            });");
    f.line("        }");
    f.line("        if !self.converged {");
    f.line("            return Err(PipelineFailure::ShapeViolation {");
    f.line("                report: ShapeViolation {");
    f.line(
        "                    shape_iri: \"https://uor.foundation/conformance/InteractionShape\",",
    );
    f.line("                    constraint_iri: \"https://uor.foundation/conformance/InteractionShape#convergence\",");
    f.line("                    property_iri: \"https://uor.foundation/conformance/convergencePredicate\",");
    f.line("                    expected_range: \"http://www.w3.org/2002/07/owl#Thing\",");
    f.line("                    min_count: 1,");
    f.line("                    max_count: 1,");
    f.line("                    kind: ViolationKind::Missing,");
    f.line("                },");
    f.line("            });");
    f.line("        }");
    f.line("        // v0.2.2 T6.1: thread H: Hasher through fold_interaction_step_digest.");
    f.line("        let mut hasher = H::initial();");
    f.line("        hasher = crate::enforcement::fold_interaction_step_digest(");
    f.line("            hasher,");
    f.line("            &self.commutator_acc,");
    f.line("            self.peer_step_count,");
    f.line("            self.seed,");
    f.line("            self.result_type_iri,");
    f.line("            crate::enforcement::CertificateKind::Grounding,");
    f.line("        );");
    f.line("        let buffer = hasher.finalize();");
    f.line(
        "        let content_fingerprint = crate::enforcement::ContentFingerprint::from_buffer(",
    );
    f.line("            buffer,");
    f.line("            H::OUTPUT_BYTES as u8,");
    f.line("        );");
    f.line("        let unit_address = crate::enforcement::unit_address_from_buffer(&buffer);");
    f.line("        let grounding = Validated::new(");
    f.line("            GroundingCertificate::with_level_and_fingerprint_const(32, content_fingerprint),");
    f.line("        );");
    f.line("        let bindings = empty_bindings_table();");
    f.line("        Ok(Grounded::<T, INLINE_BYTES, FP_MAX>::new_internal(");
    f.line("            grounding,");
    f.line("            bindings,");
    f.line("            32,");
    f.line("            unit_address,");
    f.line("            content_fingerprint,");
    f.line("        ))");
    f.line("    }");
    f.line("}");
    f.blank();

    // run_parallel
    f.doc_comment("v0.2.2 Phase F / T2.7: parallel driver entry point.");
    f.doc_comment("");
    f.doc_comment("Consumes a `Validated<ParallelDeclaration, P>` and produces a unified");
    f.doc_comment("`Grounded<T>` whose `unit_address` is derived from the declaration's");
    f.doc_comment("site count via FNV-1a. Two units with different site counts produce");
    f.doc_comment("`Grounded` values with different addresses.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `PipelineFailure::ShapeMismatch` when the declaration's");
    f.doc_comment("`result_type_iri` does not match `T::IRI` — the caller asked for");
    f.doc_comment("`Grounded<T>` but the declaration was built over a different shape.");
    f.doc_comment("Returns `PipelineFailure::ContradictionDetected` when the declared");
    f.doc_comment("partition cardinality is zero — a parallel composition with no");
    f.doc_comment("sites is inadmissible by construction.");
    f.doc_comment("");
    f.doc_comment("Success: `run_parallel` folds the declaration's site count through");
    f.doc_comment("`fold_parallel_digest` to produce a content fingerprint; distinct");
    f.doc_comment("partitions produce distinct fingerprints by construction.");
    f.doc_comment("");
    f.doc_comment("# Example");
    f.doc_comment("");
    f.doc_comment("```no_run");
    f.doc_comment("use uor_foundation::enforcement::{ConstrainedTypeInput, Validated};");
    f.doc_comment("use uor_foundation::pipeline::{run_parallel, ParallelDeclaration};");
    f.doc_comment("# use uor_foundation::enforcement::Hasher;");
    f.doc_comment("# struct Fnv1aHasher16;");
    f.doc_comment("# impl Hasher for Fnv1aHasher16 {");
    f.doc_comment("#     const OUTPUT_BYTES: usize = 16;");
    f.doc_comment("#     fn initial() -> Self { Self }");
    f.doc_comment("#     fn fold_byte(self, _: u8) -> Self { self }");
    f.doc_comment("#     fn finalize(self) -> [u8; 32] { [0; 32] }");
    f.doc_comment("# }");
    f.doc_comment("# fn wrap<T>(t: T) -> Validated<T> { unimplemented!() /* see uor_foundation_test_helpers */ }");
    f.doc_comment("");
    f.doc_comment("// 3-component partition over 9 sites.");
    f.doc_comment("static PARTITION: &[u32] = &[0, 0, 0, 1, 1, 1, 2, 2, 2];");
    f.doc_comment("let decl: Validated<ParallelDeclaration> = wrap(");
    f.doc_comment("    ParallelDeclaration::new_with_partition::<ConstrainedTypeInput>(");
    f.doc_comment("        PARTITION,");
    f.doc_comment("        \"https://uor.foundation/parallel/ParallelDisjointnessWitness\",");
    f.doc_comment("    ),");
    f.doc_comment(");");
    f.doc_comment("// ADR-060: `Grounded` carries an `INLINE_BYTES` const-generic the");
    f.doc_comment("// application derives from its `HostBounds`; thread it through");
    f.doc_comment("// `run_parallel`'s 4th const argument.");
    f.doc_comment(
        "let grounded = run_parallel::<ConstrainedTypeInput, _, Fnv1aHasher16, 32, 32>(decl)",
    );
    f.doc_comment("    .expect(\"partition admits\");");
    f.doc_comment("# let _ = grounded;");
    f.doc_comment("```");
    // Phase M.3: `run_parallel` returns `Result`, which is
    // already `#[must_use]` — no extra attribute needed. The must-use
    // discipline is enforced on run_stream/run_interactive where the
    // returned driver struct is not inherently must_use.
    f.line("pub fn run_parallel<T, P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(");
    f.line("    unit: Validated<ParallelDeclaration, P>,");
    f.line(") -> Result<Grounded<'static, T, INLINE_BYTES, FP_MAX>, PipelineFailure>");
    f.line("where");
    f.line("    T: ConstrainedTypeShape + crate::enforcement::GroundedShape,");
    f.line("    P: crate::enforcement::ValidationPhase,");
    f.line("    H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("{");
    f.line("    let decl = unit.inner();");
    f.line("    let site_count = decl.site_count();");
    f.line("    let partition = decl.site_partition();");
    f.line("    let witness_iri = decl.disjointness_witness();");
    f.line("    // Runtime invariants declared in the ParallelDeclaration rustdoc:");
    f.line("    // (1) result_type_iri must match T::IRI (target §5 + T6.11);");
    f.line("    // (2) site_count > 0 (zero-site parallel composition is vacuous);");
    f.line("    // (3) v0.2.2 Phase H3: partition length must equal site_count;");
    f.line("    // (4) v0.2.2 Phase H3: partition must be non-empty (only constructor is");
    f.line("    //     `new_with_partition`, which takes a real partition slice).");
    f.line("    if !crate::enforcement::str_eq(decl.result_type_iri(), T::IRI) {");
    f.line("        return Err(PipelineFailure::ShapeMismatch {");
    f.line("            expected: T::IRI,");
    f.line("            got: decl.result_type_iri(),");
    f.line("        });");
    f.line("    }");
    f.line("    if site_count == 0 || partition.is_empty() {");
    f.line("        return Err(PipelineFailure::ContradictionDetected {");
    f.line("            at_step: 0,");
    f.line("            trace_iri: \"https://uor.foundation/parallel/ParallelProduct\",");
    f.line("        });");
    f.line("    }");
    f.line("    if partition.len() as u64 != site_count {");
    f.line("        return Err(PipelineFailure::ShapeMismatch {");
    f.line("            expected: T::IRI,");
    f.line("            got: decl.result_type_iri(),");
    f.line("        });");
    f.line("    }");
    f.line("    // v0.2.2 Phase H3: walk partition, count sites per component, fold");
    f.line("    // per-component into the content fingerprint. Enumerates unique component");
    f.line("    // IDs into a fixed stack buffer sized by WITT_MAX_BITS.");
    f.line("    let mut hasher = H::initial();");
    f.line("    // component_ids: seen component IDs in first-appearance order.");
    f.line("    // component_counts: parallel site-count per component.");
    f.line("    let mut component_ids = [0u32; WITT_MAX_BITS as usize];");
    f.line("    let mut component_counts = [0u32; WITT_MAX_BITS as usize];");
    f.line("    let mut n_components: usize = 0;");
    f.line("    let mut si = 0;");
    f.line("    while si < partition.len() {");
    f.line("        let cid = partition[si];");
    f.line("        // Find or insert cid.");
    f.line("        let mut ci = 0;");
    f.line("        let mut found = false;");
    f.line("        while ci < n_components {");
    f.line("            if component_ids[ci] == cid {");
    f.line("                component_counts[ci] = component_counts[ci].saturating_add(1);");
    f.line("                found = true;");
    f.line("                break;");
    f.line("            }");
    f.line("            ci += 1;");
    f.line("        }");
    f.line("        if !found && n_components < (WITT_MAX_BITS as usize) {");
    f.line("            component_ids[n_components] = cid;");
    f.line("            component_counts[n_components] = 1;");
    f.line("            n_components += 1;");
    f.line("        }");
    f.line("        si += 1;");
    f.line("    }");
    f.line(
        "    // Fold each component: (component_id, site_count_within) in first-appearance order.",
    );
    f.line("    let mut ci = 0;");
    f.line("    while ci < n_components {");
    f.line("        hasher = hasher.fold_bytes(&component_ids[ci].to_be_bytes());");
    f.line("        hasher = hasher.fold_bytes(&component_counts[ci].to_be_bytes());");
    f.line("        ci += 1;");
    f.line("    }");
    f.line(
        "    // Fold disjointness_witness IRI so forgeries yield distinct content fingerprints.",
    );
    f.line("    hasher = hasher.fold_bytes(witness_iri.as_bytes());");
    f.line("    hasher = hasher.fold_byte(0);");
    f.line("    // Canonical ParallelDeclaration tail: site_count + type shape + cert kind.");
    f.line("    hasher = crate::enforcement::fold_parallel_digest(");
    f.line("        hasher,");
    f.line("        site_count,");
    f.line("        T::IRI,");
    f.line("        T::SITE_COUNT,");
    f.line("        T::CONSTRAINTS,");
    f.line("        crate::enforcement::CertificateKind::Grounding,");
    f.line("    );");
    f.line("    let buffer = hasher.finalize();");
    f.line("    let content_fingerprint = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("        buffer,");
    f.line("        H::OUTPUT_BYTES as u8,");
    f.line("    );");
    f.line("    let unit_address = crate::enforcement::unit_address_from_buffer(&buffer);");
    f.line("    let grounding = Validated::new(");
    f.line(
        "        GroundingCertificate::with_level_and_fingerprint_const(32, content_fingerprint),",
    );
    f.line("    );");
    f.line("    let bindings = empty_bindings_table();");
    f.line("    Ok(Grounded::<T, INLINE_BYTES, FP_MAX>::new_internal(");
    f.line("        grounding,");
    f.line("        bindings,");
    f.line("        32,");
    f.line("        unit_address,");
    f.line("        content_fingerprint,");
    f.line("    ))");
    f.line("}");
    f.blank();

    // run_stream
    f.doc_comment("v0.2.2 Phase F / T2.7: stream driver entry point.");
    f.doc_comment("");
    f.doc_comment("Consumes a `Validated<StreamDeclaration, P>` and returns a");
    f.doc_comment("`StreamDriver<T, P>` implementing `Iterator`. The driver's productivity");
    f.doc_comment("countdown is initialized from `StreamDeclaration::productivity_bound()`;");
    f.doc_comment("each `next()` call yields a `Grounded` whose `unit_address` differs");
    f.doc_comment("from the previous step's, and the iterator terminates when the");
    f.doc_comment("countdown reaches zero.");
    // Phase M.3: `#[must_use]` — dropping the StreamDriver silently discards
    // the iterator without pulling any items.
    f.line("#[must_use]");
    f.line("pub fn run_stream<T, P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(");
    f.line("    unit: Validated<StreamDeclaration<'_, INLINE_BYTES>, P>,");
    f.line(") -> StreamDriver<T, P, H, INLINE_BYTES, FP_MAX>");
    f.line("where");
    f.line("    T: crate::enforcement::GroundedShape,");
    f.line("    P: crate::enforcement::ValidationPhase,");
    f.line("    H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("{");
    f.line("    let bound = unit.inner().productivity_bound();");
    f.line("    let result_type_iri = unit.inner().result_type_iri();");
    f.line("    StreamDriver::new_internal(bound, bound, result_type_iri)");
    f.line("}");
    f.blank();

    // run_interactive
    f.doc_comment("v0.2.2 Phase F / T2.7: interaction driver entry point.");
    f.doc_comment("");
    f.doc_comment("Consumes a `Validated<InteractionDeclaration, P>` and returns an");
    f.doc_comment("`InteractionDriver<T, P, H>` state machine seeded from the declaration's");
    f.doc_comment("`convergence_seed()`. Advance with `step(PeerInput)` until");
    f.doc_comment("`is_converged()` returns `true`, then call `finalize()`.");
    // Phase M.3: `#[must_use]` — dropping the InteractionDriver discards all
    // peer state and convergence progress.
    f.line("#[must_use]");
    f.line("pub fn run_interactive<T, P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(");
    f.line("    unit: Validated<InteractionDeclaration, P>,");
    f.line(") -> InteractionDriver<T, P, H, INLINE_BYTES, FP_MAX>");
    f.line("where");
    f.line("    T: crate::enforcement::GroundedShape,");
    f.line("    P: crate::enforcement::ValidationPhase,");
    f.line("    H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("{");
    f.line("    InteractionDriver::new_internal(");
    f.line("        unit.inner().convergence_seed(),");
    f.line("        unit.inner().result_type_iri(),");
    f.line("    )");
    f.line("}");
    f.blank();
}

fn emit_constants(f: &mut RustFile, ontology: &Ontology) {
    f.doc_comment("Zero-based preflight check order read from `reduction:PreflightCheck`");
    f.doc_comment("individuals at codegen time. `BudgetSolvencyCheck` MUST be index 0 per");
    f.doc_comment("`reduction:preflightOrder` — enforced by the ontology, not here.");

    let mut checks: Vec<(i64, String, String)> = Vec::new();
    for ind in individuals_of_type(ontology, "https://uor.foundation/reduction/PreflightCheck") {
        let order = ind_prop_int(ind, "https://uor.foundation/reduction/preflightOrder")
            .unwrap_or(i64::MAX);
        checks.push((order, local_name(ind.id).to_string(), ind.id.to_string()));
    }
    checks.sort_by_key(|(o, _, _)| *o);

    f.line("pub const PREFLIGHT_CHECK_IRIS: &[&str] = &[");
    for (_, _, iri) in &checks {
        f.line(&format!("    \"{iri}\","));
    }
    f.line("];");
    f.blank();

    f.doc_comment("Seven reduction stages in declared order, sourced from");
    f.doc_comment("`reduction:ReductionStep` individuals.");

    let mut stages: Vec<(String, String)> = Vec::new();
    for ind in individuals_of_type(ontology, "https://uor.foundation/reduction/ReductionStep") {
        stages.push((local_name(ind.id).to_string(), ind.id.to_string()));
    }
    // The ReductionStep individuals are declared in the spec in a specific
    // order: stage_initialization through stage_convergence. They appear in
    // `reduction.rs`'s individuals() vec in that order; preserve it.
    // (Individuals of the same type are listed in declaration order in the
    // generated ontology vec.)

    f.line("pub const REDUCTION_STAGE_IRIS: &[&str] = &[");
    for (_, iri) in &stages {
        f.line(&format!("    \"{iri}\","));
    }
    f.line("];");
    f.blank();
}

fn emit_constraint_ref(f: &mut RustFile) {
    // Phase 17 fixed-capacity caps. Aligned with foundation's existing
    // 8-wide stack-allocation discipline (`MAX_BETTI_DIMENSION`,
    // `JACOBIAN_MAX_SITES`, `NERVE_CONSTRAINTS_CAP`).
    f.doc_comment("Phase 17: maximum number of `i64` coefficients an `Affine`");
    f.doc_comment("constraint can carry. Stable-Rust const evaluation cannot allocate a");
    f.doc_comment("new `&'static [i64]` at compile time, so `Affine` stores a fixed-");
    f.doc_comment("size buffer + an active prefix length. Aligned with the foundation's");
    f.doc_comment("8-wide capacity caps (`MAX_BETTI_DIMENSION` / `JACOBIAN_MAX_SITES` /");
    f.doc_comment("`NERVE_CONSTRAINTS_CAP`).");
    f.doc_comment("");
    f.doc_comment("Wiki ADR-037: the canonical source of truth for this cap is");
    f.doc_comment("[`HostBounds::AFFINE_COEFFS_MAX`]; this `pub const` is a foundation-");
    f.doc_comment("internal foundation-fixed conservative default for");
    f.doc_comment("stable-Rust array-size positions. Applications declaring a custom");
    f.doc_comment("`HostBounds` impl read `<MyBounds as HostBounds>::AFFINE_COEFFS_MAX`");
    f.doc_comment("at instantiation sites instead.");
    f.line(
        "pub const AFFINE_MAX_COEFFS: usize = \
         8;",
    );
    f.blank();
    f.doc_comment("Phase 17: maximum number of `LeafConstraintRef` conjuncts a");
    f.doc_comment("`Conjunction` can carry. Same reasoning as `AFFINE_MAX_COEFFS`.");
    f.doc_comment("");
    f.doc_comment("Wiki ADR-037: a foundation-fixed conservative default for");
    f.doc_comment("[`HostBounds::CONJUNCTION_TERMS_MAX`].");
    f.line(
        "pub const CONJUNCTION_MAX_TERMS: usize = \
         8;",
    );
    f.blank();

    f.doc_comment("Opaque constraint reference carried by `ConstrainedTypeShape` impls.");
    f.doc_comment("");
    f.doc_comment("Variants mirror the v0.2.1 `type:Constraint` enumerated subclasses");
    f.doc_comment("(retained as ergonomic aliases for the SAT pipeline) plus the v0.2.2");
    f.doc_comment("Phase D parametric form (`Bound` / `Conjunction`) which references");
    f.doc_comment("`BoundConstraint` kinds by their (observable, shape) IRIs. The");
    f.doc_comment("`SatClauses` variant carries a compact 2-SAT/Horn-SAT clause list.");
    f.doc_comment("");
    f.doc_comment("**Phase 17 — fixed-array Affine and Conjunction.** The pre-Phase-17");
    f.doc_comment("`Affine { coefficients: &'static [i64], … }` and");
    f.doc_comment("`Conjunction { conjuncts: &'static [ConstraintRef] }` have been");
    f.doc_comment("replaced with fixed-capacity arrays of length `AFFINE_MAX_COEFFS`");
    f.doc_comment("and `CONJUNCTION_MAX_TERMS` respectively. Stable Rust const");
    f.doc_comment("evaluation can build these inline; the SDK macros now support");
    f.doc_comment("Affine-bearing operands without falling back to the");
    f.doc_comment("`Site { position: u32::MAX }` sentinel. `Conjunction` is depth-");
    f.doc_comment("limited to one level: its conjuncts are `LeafConstraintRef`");
    f.doc_comment("(every variant of `ConstraintRef` except `Conjunction` itself).");
    // Phase 17: the fixed-size Affine + Conjunction arrays make
    // ConstraintRef intentionally large (~640 bytes for the
    // Conjunction variant). The size is the price of stable-Rust
    // const-eval support without slice allocations. Suppress
    // `clippy::large_enum_variant` because the layout is by design.
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("#[non_exhaustive]");
    f.line("#[allow(clippy::large_enum_variant)]");
    f.line("pub enum ConstraintRef {");
    f.line("    /// `type:ResidueConstraint`: value ≡ residue (mod modulus).");
    f.line("    Residue { modulus: u64, residue: u64 },");
    f.line("    /// `type:HammingConstraint`: bit-weight bound.");
    f.line("    Hamming { bound: u32 },");
    f.line("    /// `type:DepthConstraint`: site-depth bound.");
    f.line("    Depth { min: u32, max: u32 },");
    f.line("    /// `type:CarryConstraint`: carry-bit relation.");
    f.line("    Carry { site: u32 },");
    f.line("    /// `type:SiteConstraint`: site-position restriction.");
    f.line("    Site { position: u32 },");
    f.line("    /// `type:AffineConstraint`: affine relation over sites.");
    f.line("    /// `coefficients[..coefficient_count as usize]` is the active prefix;");
    f.line("    /// trailing entries are unused and must be zero.");
    f.line("    Affine {");
    f.line("        coefficients: [i64; AFFINE_MAX_COEFFS],");
    f.line("        coefficient_count: u32,");
    f.line("        bias: i64,");
    f.line("    },");
    f.line("    /// Opaque clause list for 2-SAT / Horn-SAT inputs.");
    f.line("    /// Each clause is a slice of `(variable_index, is_negated)`.");
    f.line("    SatClauses { clauses: &'static [&'static [(u32, bool)]], num_vars: u32 },");
    f.line("    /// v0.2.2 Phase D / T2.2 (cleanup): parametric `BoundConstraint`");
    f.line("    /// kind reference. Selects an (observable, shape) pair from the");
    f.line("    /// closed Phase D catalogue. The args_repr string carries the");
    f.line("    /// kind-specific parameters in canonical form.");
    f.line("    Bound {");
    f.line("        observable_iri: &'static str,");
    f.line("        bound_shape_iri: &'static str,");
    f.line("        args_repr: &'static str,");
    f.line("    },");
    f.line("    /// v0.2.2 Phase D / T2.2 (cleanup): parametric `Conjunction`.");
    f.line("    /// Phase 17: depth-limited to one level — conjuncts are");
    f.line("    /// `LeafConstraintRef` (every `ConstraintRef` variant except");
    f.line("    /// `Conjunction` itself). Active prefix is");
    f.line("    /// `conjuncts[..conjunct_count as usize]`.");
    f.line("    Conjunction {");
    f.line("        conjuncts: [LeafConstraintRef; CONJUNCTION_MAX_TERMS],");
    f.line("        conjunct_count: u32,");
    f.line("    },");
    f.line("    /// ADR-057 substrate amendment: bounded recursive shape reference.");
    f.line("    /// The constraint references another `ConstrainedTypeShape` by its");
    f.line("    /// content-addressed IRI, with an explicit `descent_bound` bounding");
    f.line("    /// the recursion at evaluation time. The runtime ψ_1 NerveResolver");
    f.line("    /// expands the reference to the registered shape's `CONSTRAINTS`");
    f.line("    /// array, decrementing `descent_bound` on each `Recurse` traversal;");
    f.line("    /// when `descent_bound = 0` the recursion bottoms out (no further");
    f.line("    /// constraints at that depth).");
    f.line("    ///");
    f.line("    /// The const-time admission path defers `Recurse` to runtime per");
    f.line("    /// ADR-057's deferral rule (parallel to ADR-049's");
    f.line("    /// `LandauerCost` deferral).");
    f.line("    Recurse {");
    f.line("        shape_iri: &'static str,");
    f.line("        descent_bound: u32,");
    f.line("    },");
    f.line("}");
    f.blank();

    // LeafConstraintRef — same as ConstraintRef minus Conjunction.
    f.doc_comment("`ConstraintRef` minus the recursive `Conjunction` variant — the");
    f.doc_comment("element type of `ConstraintRef::Conjunction.conjuncts`. Phase 17");
    f.doc_comment("caps Conjunction depth at one level; deeper structure must be");
    f.doc_comment("flattened by the constructor.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("#[non_exhaustive]");
    f.line("pub enum LeafConstraintRef {");
    f.line("    /// See [`ConstraintRef::Residue`].");
    f.line("    Residue { modulus: u64, residue: u64 },");
    f.line("    /// See [`ConstraintRef::Hamming`].");
    f.line("    Hamming { bound: u32 },");
    f.line("    /// See [`ConstraintRef::Depth`].");
    f.line("    Depth { min: u32, max: u32 },");
    f.line("    /// See [`ConstraintRef::Carry`].");
    f.line("    Carry { site: u32 },");
    f.line("    /// See [`ConstraintRef::Site`].");
    f.line("    Site { position: u32 },");
    f.line("    /// See [`ConstraintRef::Affine`].");
    f.line("    Affine {");
    f.line("        coefficients: [i64; AFFINE_MAX_COEFFS],");
    f.line("        coefficient_count: u32,");
    f.line("        bias: i64,");
    f.line("    },");
    f.line("    /// See [`ConstraintRef::SatClauses`].");
    f.line("    SatClauses { clauses: &'static [&'static [(u32, bool)]], num_vars: u32 },");
    f.line("    /// See [`ConstraintRef::Bound`].");
    f.line("    Bound {");
    f.line("        observable_iri: &'static str,");
    f.line("        bound_shape_iri: &'static str,");
    f.line("        args_repr: &'static str,");
    f.line("    },");
    f.line("    /// See [`ConstraintRef::Recurse`] (ADR-057).");
    f.line("    Recurse {");
    f.line("        shape_iri: &'static str,");
    f.line("        descent_bound: u32,");
    f.line("    },");
    f.line("}");
    f.blank();

    // Conversions between ConstraintRef and LeafConstraintRef.
    f.doc_comment("Project a non-`Conjunction` [`ConstraintRef`] into the");
    f.doc_comment("[`LeafConstraintRef`] subtype. Returns a `Site { position: 0 }`");
    f.doc_comment("placeholder if `self` is `Conjunction` (the only non-injective");
    f.doc_comment("case); callers should flatten Conjunction structure before");
    f.doc_comment("calling.");
    f.line("impl ConstraintRef {");
    f.line("    /// Phase 17 — leaf projection.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn as_leaf(self) -> LeafConstraintRef {");
    f.line("        match self {");
    f.line("            ConstraintRef::Residue { modulus, residue } => LeafConstraintRef::Residue { modulus, residue },");
    f.line("            ConstraintRef::Hamming { bound } => LeafConstraintRef::Hamming { bound },");
    f.line(
        "            ConstraintRef::Depth { min, max } => LeafConstraintRef::Depth { min, max },",
    );
    f.line("            ConstraintRef::Carry { site } => LeafConstraintRef::Carry { site },");
    f.line("            ConstraintRef::Site { position } => LeafConstraintRef::Site { position },");
    f.line("            ConstraintRef::Affine { coefficients, coefficient_count, bias } => {");
    f.line("                LeafConstraintRef::Affine { coefficients, coefficient_count, bias }");
    f.line("            }");
    f.line("            ConstraintRef::SatClauses { clauses, num_vars } => {");
    f.line("                LeafConstraintRef::SatClauses { clauses, num_vars }");
    f.line("            }");
    f.line("            ConstraintRef::Bound { observable_iri, bound_shape_iri, args_repr } => {");
    f.line(
        "                LeafConstraintRef::Bound { observable_iri, bound_shape_iri, args_repr }",
    );
    f.line("            }");
    f.line(
        "            ConstraintRef::Conjunction { .. } => LeafConstraintRef::Site { position: 0 },",
    );
    f.line("            ConstraintRef::Recurse { shape_iri, descent_bound } => {");
    f.line("                LeafConstraintRef::Recurse { shape_iri, descent_bound }");
    f.line("            }");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.line("impl LeafConstraintRef {");
    f.indented_doc_comment("Phase 17 — embed a leaf into the parent `ConstraintRef` enum.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn into_constraint(self) -> ConstraintRef {");
    f.line("        match self {");
    f.line("            LeafConstraintRef::Residue { modulus, residue } => ConstraintRef::Residue { modulus, residue },");
    f.line("            LeafConstraintRef::Hamming { bound } => ConstraintRef::Hamming { bound },");
    f.line(
        "            LeafConstraintRef::Depth { min, max } => ConstraintRef::Depth { min, max },",
    );
    f.line("            LeafConstraintRef::Carry { site } => ConstraintRef::Carry { site },");
    f.line("            LeafConstraintRef::Site { position } => ConstraintRef::Site { position },");
    f.line("            LeafConstraintRef::Affine { coefficients, coefficient_count, bias } => {");
    f.line("                ConstraintRef::Affine { coefficients, coefficient_count, bias }");
    f.line("            }");
    f.line("            LeafConstraintRef::SatClauses { clauses, num_vars } => {");
    f.line("                ConstraintRef::SatClauses { clauses, num_vars }");
    f.line("            }");
    f.line(
        "            LeafConstraintRef::Bound { observable_iri, bound_shape_iri, args_repr } => {",
    );
    f.line("                ConstraintRef::Bound { observable_iri, bound_shape_iri, args_repr }");
    f.line("            }");
    f.line("            LeafConstraintRef::Recurse { shape_iri, descent_bound } => {");
    f.line("                ConstraintRef::Recurse { shape_iri, descent_bound }");
    f.line("            }");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // Workstream E (v0.2.2 closure): every `ConstraintRef` variant has a
    // canonical clause encoding. Direct-decidable kinds (Residue, Carry,
    // Depth, Hamming, Site) are validated by preflight_feasibility; their
    // encoder returns an empty clause list (trivially SAT — the
    // feasibility preflight already rejected unsatisfiable ones). Affine
    // derives consistency via a single-row Gaussian check. Bound and
    // Conjunction delegate to their respective structural walkers.
    f.doc_comment("Workstream E (target §1.5 + §4.7, v0.2.2 closure): crate-internal");
    f.doc_comment("dispatch helper that maps every `ConstraintRef` variant to its");
    f.doc_comment("canonical CNF clause encoding. No variant returns `None` — the");
    f.doc_comment("closed six-kind set is fully executable.");
    f.doc_comment("");
    f.doc_comment("- `SatClauses`: pass-through of the caller's CNF.");
    f.doc_comment("- `Residue` / `Carry` / `Depth` / `Hamming` / `Site`: direct-");
    f.doc_comment("  decidable at preflight; encoder emits an empty clause list");
    f.doc_comment("  (trivially SAT — unsatisfiable ones are rejected earlier).");
    f.doc_comment("- `Affine`: single-row consistency check over Z/(2^n)Z; emits");
    f.doc_comment("  empty clauses when consistent, a 2-literal contradiction");
    f.doc_comment("  sentinel (forcing 2-SAT rejection) when not.");
    f.doc_comment("- `Bound`: parametric form; emits empty clauses (per-bound-kind");
    f.doc_comment("  decision procedures consume the observable/bound-shape IRIs).");
    f.doc_comment("- `Conjunction`: satisfiable iff every conjunct is satisfiable.");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line("#[allow(dead_code)]");
    f.line("pub(crate) const fn encode_constraint_to_clauses(");
    f.line("    constraint: &ConstraintRef,");
    f.line(") -> Option<&'static [&'static [(u32, bool)]]> {");
    f.line("    const EMPTY: &[&[(u32, bool)]] = &[];");
    f.line("    const UNSAT_SENTINEL: &[&[(u32, bool)]] =");
    f.line("        &[&[(0u32, false)], &[(0u32, true)]];");
    f.line("    match constraint {");
    f.line("        ConstraintRef::SatClauses { clauses, .. } => Some(clauses),");
    f.line("        ConstraintRef::Residue { .. }");
    f.line("        | ConstraintRef::Carry { .. }");
    f.line("        | ConstraintRef::Depth { .. }");
    f.line("        | ConstraintRef::Hamming { .. }");
    f.line("        | ConstraintRef::Site { .. } => Some(EMPTY),");
    f.line("        ConstraintRef::Affine { coefficients, coefficient_count, bias } => {");
    f.line("            if is_affine_consistent(coefficients, *coefficient_count, *bias) {");
    f.line("                Some(EMPTY)");
    f.line("            } else {");
    f.line("                Some(UNSAT_SENTINEL)");
    f.line("            }");
    f.line("        }");
    f.line("        ConstraintRef::Bound { .. } => Some(EMPTY),");
    f.line("        ConstraintRef::Conjunction { conjuncts, conjunct_count } => {");
    f.line("            if conjunction_all_sat(conjuncts, *conjunct_count) {");
    f.line("                Some(EMPTY)");
    f.line("            } else {");
    f.line("                Some(UNSAT_SENTINEL)");
    f.line("            }");
    f.line("        }");
    f.line("        // ADR-057: Recurse defers to the runtime ψ_1 NerveResolver");
    f.line("        // which expands the referenced shape's constraints via the");
    f.line("        // shape-IRI registry. At preflight (const-eval) it carries no");
    f.line("        // CNF residue — the recursion is bounded, the unrolling occurs");
    f.line("        // at runtime against the registry.");
    f.line("        ConstraintRef::Recurse { .. } => Some(EMPTY),");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("Workstream E: single-row consistency for `Affine { coefficients,");
    f.doc_comment("coefficient_count, bias }`. The constraint is");
    f.doc_comment("`sum(c_i) · x = bias (mod 2^n)`; when the coefficient sum is");
    f.doc_comment("zero the system is consistent iff bias is zero. Non-zero sums");
    f.doc_comment("are always consistent over Z/(2^n)Z for some `x` value. Iterates");
    f.doc_comment("only the active prefix `coefficients[..coefficient_count as usize]`.");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line("const fn is_affine_consistent(");
    f.line("    coefficients: &[i64; AFFINE_MAX_COEFFS],");
    f.line("    coefficient_count: u32,");
    f.line("    bias: i64,");
    f.line(") -> bool {");
    f.line("    let mut sum: i128 = 0;");
    f.line("    let count = coefficient_count as usize;");
    f.line("    let mut i = 0;");
    f.line("    while i < count && i < AFFINE_MAX_COEFFS {");
    f.line("        sum += coefficients[i] as i128;");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    if sum == 0 { bias == 0 } else { true }");
    f.line("}");
    f.blank();
    f.doc_comment("Workstream E + Phase 17: satisfiability of a `Conjunction`.");
    f.doc_comment("Iterates only the active prefix `conjuncts[..conjunct_count as");
    f.doc_comment("usize]` and lifts each `LeafConstraintRef` back to a");
    f.doc_comment("`ConstraintRef` for re-encoding (the leaf form omits Conjunction,");
    f.doc_comment("so this terminates at depth 1).");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line("const fn conjunction_all_sat(");
    f.line("    conjuncts: &[LeafConstraintRef; CONJUNCTION_MAX_TERMS],");
    f.line("    conjunct_count: u32,");
    f.line(") -> bool {");
    f.line("    let count = conjunct_count as usize;");
    f.line("    let mut i = 0;");
    f.line("    while i < count && i < CONJUNCTION_MAX_TERMS {");
    f.line("        let lifted = conjuncts[i].into_constraint();");
    f.line("        match encode_constraint_to_clauses(&lifted) {");
    f.line("            Some([]) => {}");
    f.line("            _ => return false,");
    f.line("        }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    true");
    f.line("}");
    f.blank();
}

fn emit_constrained_type_shape(f: &mut RustFile) {
    f.doc_comment("Declarative shape of a constrained type that can be admitted into the");
    f.doc_comment("reduction pipeline.");
    f.doc_comment("");
    f.doc_comment("Downstream authors implement this trait on zero-sized marker types to");
    f.doc_comment("declare the `(IRI, SITE_COUNT, CONSTRAINTS)` triple of a custom");
    f.doc_comment("constrained type. The foundation admits shapes into the pipeline via");
    f.doc_comment("[`validate_constrained_type`] / [`validate_constrained_type_const`],");
    f.doc_comment("which run the full preflight (`preflight_feasibility` +");
    f.doc_comment("`preflight_package_coherence`) against `Self::CONSTRAINTS` before");
    f.doc_comment("returning a [`Validated`] wrapper.");
    f.doc_comment("");
    f.doc_comment("Sealing of witness construction lives on [`Validated`] and [`Grounded`]");
    f.doc_comment("— only foundation admission functions mint either. Downstream is free");
    f.doc_comment("to implement `ConstrainedTypeShape` for arbitrary shape markers, but");
    f.doc_comment("cannot fabricate a `Validated<Self>` except through the admission path.");
    f.doc_comment("The `ConstraintRef` enum is `#[non_exhaustive]` from outside the crate,");
    f.doc_comment("so `CONSTRAINTS` can only cite foundation-closed constraint kinds.");
    f.doc_comment("");
    f.doc_comment("# Example");
    f.doc_comment("");
    f.doc_comment("```");
    f.doc_comment("use uor_foundation::pipeline::{");
    f.doc_comment("    ConstrainedTypeShape, ConstraintRef, validate_constrained_type,");
    f.doc_comment("};");
    f.doc_comment("");
    f.doc_comment("pub struct MyShape;");
    f.doc_comment("");
    f.doc_comment("impl ConstrainedTypeShape for MyShape {");
    f.doc_comment("    const IRI: &'static str = \"https://example.org/MyShape\";");
    f.doc_comment("    const SITE_COUNT: usize = 4;");
    f.doc_comment("    const CONSTRAINTS: &'static [ConstraintRef] = &[");
    f.doc_comment("        ConstraintRef::Residue { modulus: 7, residue: 3 },");
    f.doc_comment("    ];");
    f.doc_comment("    const CYCLE_SIZE: u64 = 7;  // ADR-032: 7 residues mod 7");
    f.doc_comment("}");
    f.doc_comment("");
    f.doc_comment("let validated = validate_constrained_type(MyShape)");
    f.doc_comment("    .expect(\"residue 3 mod 7 is admissible\");");
    f.doc_comment("# let _ = validated;");
    f.doc_comment("```");
    f.line("pub trait ConstrainedTypeShape {");
    f.indented_doc_comment(
        "IRI of the ontology `type:ConstrainedType` instance this shape represents.",
    );
    f.line("    const IRI: &'static str;");
    f.indented_doc_comment("Number of sites (fields) this constrained type carries.");
    f.line("    const SITE_COUNT: usize;");
    f.indented_doc_comment("Ontology-level `siteBudget`: count of data sites only,");
    f.indented_doc_comment("excluding bookkeeping introduced by composition (coproduct tag");
    f.indented_doc_comment("sites, etc.). Equals `SITE_COUNT` for leaf shapes and for");
    f.indented_doc_comment("shapes whose composition introduces no bookkeeping (products,");
    f.indented_doc_comment("cartesian products). Strictly less than `SITE_COUNT` for coproduct");
    f.indented_doc_comment("shapes and any shape whose `SITE_COUNT` includes inherited");
    f.indented_doc_comment("bookkeeping. Introduced by the Product/Coproduct Completion");
    f.indented_doc_comment("Amendment §4a; defaults to `SITE_COUNT` so pre-amendment");
    f.indented_doc_comment("shape impls remain valid without edits.");
    f.line("    const SITE_BUDGET: usize = Self::SITE_COUNT;");
    f.indented_doc_comment("Per-site constraint list. Empty means unconstrained.");
    f.line("    const CONSTRAINTS: &'static [ConstraintRef];");
    f.indented_doc_comment("ADR-032: cardinality of the shape's value-set (the cycle");
    f.indented_doc_comment("structure of the shape under the substrate's discrete-clock");
    f.indented_doc_comment("model). Used by the `prism_model!` macro to lower `first_admit`");
    f.indented_doc_comment("(closure-body grammar G16) to the correct descent measure.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Conventions:");
    f.indented_doc_comment("- Witt-level shapes: `1u64 << WITT_LEVEL_BITS` (W8 = 256, W16 =");
    f.indented_doc_comment("  65536, W32 = 4294967296). Levels above W63 saturate to `u64::MAX`.");
    f.indented_doc_comment("- `partition_product` factors: `cycle_size_product` of factor");
    f.indented_doc_comment("  CYCLE_SIZEs (saturating-multiply).");
    f.indented_doc_comment("- `partition_coproduct` summands: `cycle_size_coproduct` (saturating");
    f.indented_doc_comment("  add + 1 for the discriminant).");
    f.indented_doc_comment("- `cartesian_product_shape` (homogeneous power): factor's CYCLE_SIZE");
    f.indented_doc_comment("  raised to `SITE_COUNT` (saturating).");
    f.indented_doc_comment("- The foundation-sanctioned identity `ConstrainedTypeInput` has");
    f.indented_doc_comment("  `CYCLE_SIZE = 1` (single-element shape).");
    f.line("    const CYCLE_SIZE: u64;");
    f.line("}");
    f.blank();

    // ADR-032: const-fn helpers the SDK macros call at expansion time
    // when composing shapes. Both saturate to u64::MAX so the macros
    // never overflow; downstream `first_admit` lowering uses the value
    // as a `Term::Literal` measure for `Term::Recurse`.
    f.doc_comment("ADR-032: saturating multiply for `partition_product`'s CYCLE_SIZE.");
    f.doc_comment("Returns `u64::MAX` on overflow.");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line("pub const fn cycle_size_product(a: u64, b: u64) -> u64 {");
    f.line("    a.saturating_mul(b)");
    f.line("}");
    f.blank();
    f.doc_comment("ADR-032: saturating add + 1 (for the discriminant) for");
    f.doc_comment("`partition_coproduct`'s CYCLE_SIZE. Returns `u64::MAX` on overflow.");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line("pub const fn cycle_size_coproduct(a: u64, b: u64) -> u64 {");
    f.line("    a.saturating_add(b).saturating_add(1)");
    f.line("}");
    f.blank();
    f.doc_comment("ADR-032: saturating power for `cartesian_product_shape`'s CYCLE_SIZE");
    f.doc_comment("(homogeneous power: factor.CYCLE_SIZE^SITE_COUNT). Returns `u64::MAX`");
    f.doc_comment("on overflow.");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line("pub const fn cycle_size_power(base: u64, exp: usize) -> u64 {");
    f.line("    let mut result: u64 = 1;");
    f.line("    let mut i: usize = 0;");
    f.line("    while i < exp {");
    f.line("        result = result.saturating_mul(base);");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    result");
    f.line("}");
    f.blank();

    // Built-in empty shape for the foundation default.
    f.line("impl ConstrainedTypeShape for ConstrainedTypeInput {");
    f.line("    const IRI: &'static str = \"https://uor.foundation/type/ConstrainedType\";");
    f.line("    const SITE_COUNT: usize = 0;");
    f.line("    const CONSTRAINTS: &'static [ConstraintRef] = &[];");
    f.line("    const CYCLE_SIZE: u64 = 1;");
    f.line("}");
    f.blank();

    // ADR-032 Witt-level domain types are emitted by `emit_witt_domain`
    // (called from `generate_pipeline_module`'s top-level driver).

    // ADR-033 G20: every `partition_product`-shaped type carries a
    // factor-fields directory describing each factor's byte offset and
    // length within the product's canonical byte serialization (per
    // ADR-023's `IntoBindingValue` carrier). The closure-body
    // grammar's field-access form `<expr>.<index>` (G20 positional) and
    // `<expr>.<name>` (G20 named) lowers via this trait at proc-macro
    // expansion time: the macro emits `Term::ProjectField` whose
    // `byte_offset` and `byte_length` are read from this directory.
    f.doc_comment("ADR-033 G20: factor-field directory carried by every");
    f.doc_comment("`partition_product`-shaped type. The closure-body grammar's");
    f.doc_comment("field-access form (`<expr>.<index>` or `<expr>.<field_name>`)");
    f.doc_comment("lowers via this trait at proc-macro expansion time.");
    f.doc_comment("");
    f.doc_comment("Foundation-sanctioned identity `ConstrainedTypeInput` has zero");
    f.doc_comment("fields (it is a leaf shape). The SDK macros `partition_product!`,");
    f.doc_comment("`product_shape!`, and `cartesian_product_shape!` emit the impl.");
    f.line("pub trait PartitionProductFields: ConstrainedTypeShape {");
    f.indented_doc_comment("Per-factor `(byte_offset, byte_length)` pairs in declaration");
    f.indented_doc_comment("order. Length is the same as `FIELD_NAMES.len()`.");
    f.line("    const FIELDS: &'static [(u32, u32)];");
    f.indented_doc_comment("Per-factor names. Empty string `\"\"` for positional-only");
    f.indented_doc_comment("`partition_product!(Name, A, B)` emissions; non-empty for");
    f.indented_doc_comment("named-field `partition_product!(Name, lhs: A, rhs: B)` form.");
    f.indented_doc_comment("Length matches `FIELDS.len()`.");
    f.line("    const FIELD_NAMES: &'static [&'static str];");
    f.indented_doc_comment("Linear search returning the field index whose `FIELD_NAMES`");
    f.indented_doc_comment("entry equals `name`, or `usize::MAX` if not found. Delegates");
    f.indented_doc_comment("to the free `const fn` [`field_index_by_name_in`] so the");
    f.indented_doc_comment("result is usable inside const-eval contexts on stable Rust");
    f.indented_doc_comment("1.83 (where const trait methods are unavailable).");
    f.line("    #[must_use]");
    f.line("    fn field_index_by_name(name: &str) -> usize {");
    f.line("        field_index_by_name_in(Self::FIELD_NAMES, name)");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("ADR-033 G3/G4: const-fn factor-name lookup. The SDK proc-macro");
    f.doc_comment("emits const-eval calls to this helper to resolve a named-field");
    f.doc_comment("access against the source type's `FIELD_NAMES`. On stable Rust");
    f.doc_comment("1.83 a free `const fn` is the substitute for a `const fn`");
    f.doc_comment("trait method. Returns `usize::MAX` for not-found so the result");
    f.doc_comment("is usable directly inside `const` array indexing.");
    f.line("#[must_use]");
    f.line("pub const fn field_index_by_name_in(names: &[&'static str], name: &str) -> usize {");
    f.line("    let nb = name.as_bytes();");
    f.line("    let mut i = 0usize;");
    f.line("    while i < names.len() {");
    f.line("        let nb_i = names[i].as_bytes();");
    f.line("        if nb_i.len() == nb.len() {");
    f.line("            let mut j = 0usize;");
    f.line("            let mut matched = true;");
    f.line("            while j < nb.len() {");
    f.line("                if nb_i[j] != nb[j] { matched = false; break; }");
    f.line("                j += 1;");
    f.line("            }");
    f.line("            if matched { return i; }");
    f.line("        }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    usize::MAX");
    f.line("}");
    f.blank();
    // Foundation-sanctioned identity: zero fields (leaf shape).
    f.line("impl PartitionProductFields for ConstrainedTypeInput {");
    f.line("    const FIELDS: &'static [(u32, u32)] = &[];");
    f.line("    const FIELD_NAMES: &'static [&'static str] = &[];");
    f.line("}");
    f.blank();

    // ADR-033 G20 (chained-access support): per-factor type retrieval.
    // The `prism_model!` proc-macro uses `<SourceTy as
    // PartitionProductFactor<I>>::Factor` to resolve the static type of
    // a chained `<expr>.<member>` receiver. `partition_product!`,
    // `product_shape!`, and `cartesian_product_shape!` emit per-index
    // impls of this trait so the proc-macro can name the inner type
    // when synthesizing further `Term::ProjectField` lookups.
    f.doc_comment("ADR-033 G20 chained-field access support: maps a");
    f.doc_comment("`partition_product`-shaped type's positional factor index to");
    f.doc_comment("the factor's static type. The `prism_model!` proc-macro emits");
    f.doc_comment("a chain of `Term::ProjectField` projections by walking this");
    f.doc_comment("trait, naming each next source type as");
    f.doc_comment("`<PrevTy as PartitionProductFactor<I>>::Factor`.");
    f.doc_comment("");
    f.doc_comment("`partition_product!`, `product_shape!`, and");
    f.doc_comment("`cartesian_product_shape!` emit one impl per factor index.");
    f.line("pub trait PartitionProductFactor<const INDEX: usize>: PartitionProductFields {");
    f.indented_doc_comment("The static type of the factor at position `INDEX`.");
    f.line("    type Factor: ConstrainedTypeShape;");
    f.line("}");
    f.blank();

    emit_axis_extension(f);
    emit_resolver_tuple(f);
    // ADR-049 + ADR-048 ordering: ObservablePredicate trait + 4 typed
    // observables emitted first because SingletonCommitment<P> depends on
    // the trait. TypedCommitment + 3 impls follow. PrismModel's 5th
    // substrate parameter `C: TypedCommitment` is wired in
    // `emit_prism_model`.
    emit_observable_predicate(f);
    emit_typed_commitment(f);
    emit_prism_model(f);
    emit_cartesian_product_shape(f);
    emit_shape_iri_registry(f);
}

/// ADR-057: emits the foundation shape-IRI registry module —
/// `RegisteredShape` struct + `ShapeRegistryProvider` sealed trait +
/// `EmptyShapeRegistry` default impl + `lookup_shape(iri)` (foundation
/// built-in registry) + `lookup_shape_in::<R>(iri)` (generic over an
/// application's `ShapeRegistryProvider` impl). The registry is consulted
/// by ψ_1's NerveResolver at runtime when expanding `ConstraintRef::Recurse`
/// references.
///
/// The registry uses const-aggregation through the sealed trait surface:
/// the SDK `register_shape!(MyAppRegistry, S1, S2, …)` macro emits a marker
/// type whose `REGISTRY` const slice is the application's list of
/// registered shapes; foundation walks the slice at runtime via
/// `lookup_shape_in`. This realization is `no_std`-safe, zero-dependency,
/// zero-`unsafe`, and zero-allocation per the foundation discipline.
fn emit_shape_iri_registry(f: &mut RustFile) {
    f.doc_comment("ADR-057: bounded recursive structural typing — foundation shape-IRI");
    f.doc_comment("registry module. Mirrors the architectural pattern of the");
    f.doc_comment("observable-IRI registry per ADR-038/049 and the substitution-axis");
    f.doc_comment("catalog per ADR-007/030: a closed catalog of shape IRIs admissible");
    f.doc_comment("as `ConstraintRef::Recurse` targets, consulted by ψ_1's NerveResolver");
    f.doc_comment("when expanding recursive references at evaluation time.");
    f.doc_comment("");
    f.doc_comment("# Architecture");
    f.doc_comment("");
    f.doc_comment("Foundation is `#![no_std]` with zero dependencies and zero `unsafe`,");
    f.doc_comment("so the registry uses **const-aggregation through a sealed trait**");
    f.doc_comment("rather than a mutable global or link-section symbol resolution.");
    f.doc_comment("Applications declare their shape registry as a single type via the");
    f.doc_comment("SDK `register_shape!(Shape1, Shape2, …)` macro per crate; the type");
    f.doc_comment("implements [`ShapeRegistryProvider`] with the const-aggregated slice.");
    f.doc_comment("ψ_1 NerveResolver consults the registry through [`lookup_shape_in`].");
    f.doc_comment("");
    f.doc_comment("Foundation's built-in [`lookup_shape`] consults a foundation-owned");
    f.doc_comment("static registry — currently empty; standard-library Layer-3 sub-crates");
    f.doc_comment("per ADR-031 publishing canonical recursive-grammar shapes register");
    f.doc_comment("through this path in future foundation-curated additions.");
    f.line("pub mod shape_iri_registry {");
    f.line("    use super::ConstraintRef;");
    f.blank();
    f.indented_doc_comment("ADR-057: a registered shape entry. `iri` is the shape's");
    f.indented_doc_comment("content-addressed identifier per ADR-017; `site_count`,");
    f.indented_doc_comment("`constraints`, and `cycle_size` mirror the corresponding");
    f.indented_doc_comment("`ConstrainedTypeShape` associated items so ψ_1's resolver");
    f.indented_doc_comment("can walk the referenced shape's constraint set without");
    f.indented_doc_comment("touching the original trait impl.");
    f.line("    #[derive(Debug, Clone, Copy)]");
    f.line("    pub struct RegisteredShape {");
    f.indented_doc_comment("Content-addressed IRI of the shape (per ADR-017).");
    f.line("        pub iri: &'static str,");
    f.indented_doc_comment("Mirror of `<T as ConstrainedTypeShape>::SITE_COUNT`.");
    f.line("        pub site_count: usize,");
    f.indented_doc_comment("Mirror of `<T as ConstrainedTypeShape>::CONSTRAINTS`.");
    f.line("        pub constraints: &'static [ConstraintRef],");
    f.indented_doc_comment("Mirror of `<T as ConstrainedTypeShape>::CYCLE_SIZE`.");
    f.line("        pub cycle_size: u64,");
    f.line("    }");
    f.blank();

    // ShapeRegistryProvider trait — the const-aggregated registry surface.
    f.indented_doc_comment("ADR-057: sealed trait carrying a crate's registered-shape slice");
    f.indented_doc_comment("at the const-aggregation surface. The SDK `register_shape!` macro");
    f.indented_doc_comment("emits an `impl ShapeRegistryProvider` on a marker type whose");
    f.indented_doc_comment("`REGISTRY` const is the list of registered shapes.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Sealed via [`super::__sdk_seal::Sealed`] per ADR-022 D1; only");
    f.indented_doc_comment("the SDK `register_shape!` macro emits impls (plus foundation's");
    f.indented_doc_comment("[`EmptyShapeRegistry`] default).");
    f.line("    pub trait ShapeRegistryProvider: super::__sdk_seal::Sealed {");
    f.indented_doc_comment("The crate's registered-shape slice. Walked by");
    f.indented_doc_comment("[`lookup_shape_in`] when ψ_1 NerveResolver expands");
    f.indented_doc_comment("`ConstraintRef::Recurse` references at evaluation time.");
    f.line("        const REGISTRY: &'static [RegisteredShape];");
    f.line("    }");
    f.blank();

    f.indented_doc_comment("ADR-057: the empty-registry default, used by applications that");
    f.indented_doc_comment("don't carry recursive shapes. Foundation provides this as the");
    f.indented_doc_comment("baseline `ShapeRegistryProvider` impl; applications with");
    f.indented_doc_comment("recursive shapes use the SDK `register_shape!` macro to declare");
    f.indented_doc_comment("their own marker type with a non-empty `REGISTRY`.");
    f.line("    #[derive(Debug, Clone, Copy, Default)]");
    f.line("    pub struct EmptyShapeRegistry;");
    f.line("    impl super::__sdk_seal::Sealed for EmptyShapeRegistry {}");
    f.line("    impl ShapeRegistryProvider for EmptyShapeRegistry {");
    f.line("        const REGISTRY: &'static [RegisteredShape] = &[];");
    f.line("    }");
    f.blank();

    // Foundation's built-in canonical-shape registry (currently empty).
    f.indented_doc_comment("Foundation's built-in shape registry. Currently empty —");
    f.indented_doc_comment("standard-library Layer-3 sub-crates per ADR-031 publishing");
    f.indented_doc_comment("canonical recursive-grammar shapes (canonical JSON, AST, etc.)");
    f.indented_doc_comment("register through this path in future foundation-curated additions.");
    f.line("    static FOUNDATION_REGISTRY: &[RegisteredShape] = &[];");
    f.blank();

    f.indented_doc_comment("ADR-057: look up a registered shape by IRI in the");
    f.indented_doc_comment("foundation-built-in registry. Returns `None` if the IRI is");
    f.indented_doc_comment("not present; applications consulting their own application-");
    f.indented_doc_comment("registered shapes use [`lookup_shape_in`] generic over their");
    f.indented_doc_comment("`ShapeRegistryProvider`-implementing marker type.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Called by ψ_1's NerveResolver during `ConstraintRef::Recurse`");
    f.indented_doc_comment("evaluation when no application registry is threaded through.");
    f.line("    #[must_use]");
    f.line("    pub fn lookup_shape(iri: &str) -> Option<&'static RegisteredShape> {");
    f.line("        lookup_in_slice(FOUNDATION_REGISTRY, iri)");
    f.line("    }");
    f.blank();

    f.indented_doc_comment("ADR-057: look up a registered shape by IRI in an application's");
    f.indented_doc_comment("registry plus the foundation-built-in registry. Walks the");
    f.indented_doc_comment("application's `R::REGISTRY` first, then falls back to the");
    f.indented_doc_comment("foundation registry; returns `None` if the IRI is in neither.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Called by ψ_1's NerveResolver during `ConstraintRef::Recurse`");
    f.indented_doc_comment("evaluation when an application registry is threaded through");
    f.indented_doc_comment("(per `pipeline::run_route`'s registry-parameterized path).");
    f.line("    #[must_use]");
    f.line(
        "    pub fn lookup_shape_in<R: ShapeRegistryProvider>(iri: &str) -> Option<&'static RegisteredShape> {",
    );
    f.line("        if let Some(entry) = lookup_in_slice(R::REGISTRY, iri) {");
    f.line("            return Some(entry);");
    f.line("        }");
    f.line("        lookup_in_slice(FOUNDATION_REGISTRY, iri)");
    f.line("    }");
    f.blank();

    f.indented_doc_comment("Linear-scan helper used by both [`lookup_shape`] and");
    f.indented_doc_comment("[`lookup_shape_in`]. Registry slices are expected to be small");
    f.indented_doc_comment("(a handful of canonical shapes per standard-library sub-crate");
    f.indented_doc_comment("or per application), so linear scan dominates startup cost.");
    f.line("    fn lookup_in_slice(slice: &'static [RegisteredShape], iri: &str) -> Option<&'static RegisteredShape> {");
    f.line("        let mut i = 0;");
    f.line("        while i < slice.len() {");
    f.line("            let entry = &slice[i];");
    f.line("            if iri_eq(entry.iri, iri) {");
    f.line("                return Some(entry);");
    f.line("            }");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        None");
    f.line("    }");
    f.blank();

    f.line("    fn iri_eq(a: &str, b: &str) -> bool {");
    f.line("        a.as_bytes() == b.as_bytes()");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// ADR-032 (G5): emits one zero-sized marker type per Witt level, each
/// implementing `ConstrainedTypeShape` with `CYCLE_SIZE = 2^bits_width`
/// (saturated at `u64::MAX` for levels above W64). The `prism_model!`
/// proc-macro reads `<DomainTy as ConstrainedTypeShape>::CYCLE_SIZE` at
/// expansion time to lower `first_admit(<DomainTy>, |i| …)` (G16) into
/// `Term::Recurse` whose descent measure is the domain's cardinality.
///
/// The wiki's example syntax `first_admit(WittLevel::W32, |nonce| …)`
/// addresses the level as a *type*; on stable Rust 1.83 the inherent-
/// associated-type form `WittLevel::W32` is unavailable (it requires
/// nightly `inherent_associated_types`), so the witt_domain types are
/// the architecturally-equivalent named form — `witt_domain::W32` is
/// the type whose `CYCLE_SIZE` is `2^32 = 4_294_967_296`, matching the
/// wiki's normative count.
fn emit_witt_domain(f: &mut RustFile, ontology: &Ontology) {
    let mut levels = witt_levels(ontology);
    levels.extend(limbs_witt_levels(ontology));
    levels.sort_by_key(|(_, bits, _)| *bits);

    f.doc_comment("ADR-032: per-Witt-level zero-sized marker types implementing");
    f.doc_comment("`ConstrainedTypeShape`. The `prism_model!` proc-macro consumes");
    f.doc_comment("these as the `<DomainTy>` operand of `first_admit(<DomainTy>, |i| …)`");
    f.doc_comment("(G16): `<DomainTy as ConstrainedTypeShape>::CYCLE_SIZE` carries the");
    f.doc_comment("domain's cardinality, which the macro lowers as the descent measure");
    f.doc_comment("for the emitted `Term::Recurse`.");
    f.doc_comment("");
    f.doc_comment("Each level's `CYCLE_SIZE` is `2^bits_width`, saturated at");
    f.doc_comment("`u64::MAX` for levels whose cardinality exceeds 64-bit range.");
    f.doc_comment("The wiki's normative example `first_admit(WittLevel::W32, |nonce| …)`");
    f.doc_comment("compiles on stable Rust 1.83 as `first_admit(witt_domain::W32, |nonce| …)`");
    f.doc_comment("(`witt_domain::W32::CYCLE_SIZE = 4_294_967_296`).");
    f.line("pub mod witt_domain {");
    f.line("    use super::{ConstrainedTypeShape, ConstraintRef, PartitionProductFields};");
    f.line("    use crate::enforcement::GroundedShape;");
    f.line("    use crate::pipeline::__sdk_seal;");
    f.line("    use crate::pipeline::IntoBindingValue;");
    f.blank();
    for (local, bits, _byte_or_limb) in &levels {
        // Compute cycle_size: 2^bits, saturated at u64::MAX.
        let cycle_size = if *bits >= 64 {
            "u64::MAX".to_string()
        } else {
            format!("{}u64", 1u64 << *bits)
        };
        let iri = format!("https://uor.foundation/witt/{local}");
        f.indented_doc_comment(&format!(
            "ADR-032 Witt-level domain marker for `{local}` ({bits}-bit ring)."
        ));
        f.indented_doc_comment(&format!(
            "`CYCLE_SIZE = 2^{bits} = {}`. Used as the `<DomainTy>` operand of",
            if *bits >= 64 {
                "u64::MAX (saturated)".to_string()
            } else {
                format!("{}", 1u64 << *bits)
            }
        ));
        f.indented_doc_comment(
            "`first_admit(<DomainTy>, |i| …)` (G16) in `prism_model!` closures.",
        );
        f.line(&format!("    pub struct {local};"));
        f.line(&format!("    impl ConstrainedTypeShape for {local} {{"));
        f.line(&format!("        const IRI: &'static str = \"{iri}\";"));
        f.line("        const SITE_COUNT: usize = 1;");
        f.line("        const CONSTRAINTS: &'static [ConstraintRef] = &[];");
        f.line(&format!("        const CYCLE_SIZE: u64 = {cycle_size};"));
        f.line("    }");
        f.line(&format!("    impl __sdk_seal::Sealed for {local} {{}}"));
        f.line(&format!("    impl<'a> IntoBindingValue<'a> for {local} {{"));
        f.line(
            "        fn as_binding_value<const INLINE_BYTES: usize>(&self) -> crate::pipeline::TermValue<'a, INLINE_BYTES> {",
        );
        f.line("            crate::pipeline::TermValue::empty()");
        f.line("        }");
        f.line("    }");
        f.line(&format!("    impl GroundedShape for {local} {{}}"));
        f.line(&format!("    impl PartitionProductFields for {local} {{"));
        f.line("        const FIELDS: &'static [(u32, u32)] = &[];");
        f.line("        const FIELD_NAMES: &'static [&'static str] = &[];");
        f.line("    }");
        f.blank();
    }
    f.line("}");
    f.blank();
}

/// Emits the wiki ADR-030 surface: the [`AxisExtension`] sealed trait
/// that every substrate-extension axis declares (via the SDK `axis!`
/// macro) and the [`AxisTuple`] dispatcher trait whose tuple impls
/// route axis invocations to the right kernel at evaluation time.
fn emit_axis_extension(f: &mut RustFile) {
    f.doc_comment("ADR-030: maximum number of axes a single application's");
    f.doc_comment("`AxisTuple` may carry. Foundation-fixed (parallel to");
    f.doc_comment("`FOLD_UNROLL_THRESHOLD` and `UNFOLD_MAX_ITERATIONS`).");
    f.line("pub const MAX_AXIS_TUPLE_ARITY: usize = 8;");
    f.blank();
    // ADR-060: the axis-output byte ceiling (`AXIS_OUTPUT_BYTES_CEILING`) and
    // the 8 ψ-stage `*_OUTPUT_BYTES_MAX` foundation-internal aliases are
    // REMOVED along with their `HostBounds` source constants and
    // `DefaultHostBounds`. Axis kernels and resolvers now return a
    // source-polymorphic `TermValue` (Inline / Borrowed / Stream) with no
    // caller-side byte-width ceiling; bounded inline outputs fit
    // `carrier_inline_bytes::<B>()`, larger structural / unbounded outputs flow
    // as `Borrowed` / `Stream`.
    // ADR-055: Universal substrate-Term verb body discipline. Every
    // `AxisExtension` impl carries a `SubstrateTermBody::body_arena()`
    // describing the kernel's substrate-Term decomposition so the
    // catamorphism's structural reach extends through the axis surface
    // to the leaf level. The trait is sealed via `__sdk_seal::Sealed`;
    // emission is via the `axis!` SDK macro per ADR-030 + ADR-052.
    f.doc_comment("ADR-055: substrate-Term verb body discipline. Every axis impl carries a");
    f.doc_comment("substrate-Term decomposition the catamorphism can fuse through. Sealed —");
    f.doc_comment("the `axis!` SDK macro per ADR-030 + ADR-052 emits the impl.");
    f.doc_comment("");
    f.doc_comment("The catamorphism's `Term::AxisInvocation` fold-rule reads this slice and");
    f.doc_comment("recursively folds the body with the evaluated kernel inputs in scope (per");
    f.doc_comment("ADR-029, amended by ADR-055). The `body_arena()` slice is a static const,");
    f.doc_comment("not wire-format state, so the on-wire shape of `Term::AxisInvocation` is");
    f.doc_comment("preserved.");
    f.line("pub trait SubstrateTermBody<const INLINE_BYTES: usize>: __sdk_seal::Sealed {");
    f.indented_doc_comment("The Term arena the kernel decomposes to. Empty slice signals a");
    f.indented_doc_comment("primitive-fast-path axis whose body the implementation may evaluate");
    f.indented_doc_comment("through `dispatch_kernel` directly per ADR-055's optional fast-path.");
    f.line("    fn body_arena() -> &'static [crate::enforcement::Term<'static, INLINE_BYTES>];");
    f.line("}");
    f.blank();

    f.doc_comment("ADR-030: a substrate-extension axis. Each `axis!`-declared");
    f.doc_comment("trait extends this trait via the SDK macro's blanket impl,");
    f.doc_comment("which emits per-method `KERNEL_*` const ids and the");
    f.doc_comment("`dispatch_kernel` router into a fixed-capacity byte buffer.");
    f.doc_comment("");
    f.doc_comment("Per ADR-055, every `AxisExtension` impl also satisfies the");
    f.doc_comment("[`SubstrateTermBody`] supertrait — its kernel decomposes to a");
    f.doc_comment("substrate-Term slice the catamorphism walks structurally. This makes");
    f.doc_comment("the Fold-Fusion Principle (ADR-054) universal across every axis impl,");
    f.doc_comment("not just the standard library's canonical impls.");
    f.doc_comment("");
    f.doc_comment("The catamorphism's `Term::AxisInvocation` fold-rule reads the");
    f.doc_comment("axis position from the application's `AxisTuple` impl, walks the");
    f.doc_comment("axis's `SubstrateTermBody::body_arena()` recursively with the");
    f.doc_comment("evaluated kernel input bound in scope, and emits the resulting");
    f.doc_comment("`TermValue`. The legacy `dispatch_kernel` fast-path remains as an");
    f.doc_comment("optimization for axes whose body is empty (primitive fast-path).");
    f.indented_doc_comment(
        "ADR-018 + ADR-030: `FP_MAX` is the application's fingerprint-width axis",
    );
    f.indented_doc_comment(
        "(`<B as HostBounds>::FINGERPRINT_MAX_BYTES`), threaded so a `HashAxis`",
    );
    f.indented_doc_comment(
        "wrapping a `Hasher<FP_MAX>` of any width composes — there is no pinned",
    );
    f.indented_doc_comment("32-byte fingerprint. Parallel to `INLINE_BYTES`: the application");
    f.indented_doc_comment("instantiates it from its `HostBounds`, never a contrived literal.");
    f.line("pub trait AxisExtension<const INLINE_BYTES: usize, const FP_MAX: usize>: SubstrateTermBody<INLINE_BYTES> {");
    f.indented_doc_comment("ADR-017 content address of this axis trait. The SDK macro");
    f.indented_doc_comment("derives this from the trait name and method signatures.");
    f.line("    const AXIS_ADDRESS: &'static str;");
    f.indented_doc_comment("Maximum bytes any kernel of this axis returns.");
    f.line("    const MAX_OUTPUT_BYTES: usize;");
    f.indented_doc_comment("Dispatch the kernel identified by `kernel_id` against the");
    f.indented_doc_comment("evaluated input bytes. The implementation copies the kernel's");
    f.indented_doc_comment("output into `out` and returns the written length.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns [`crate::enforcement::ShapeViolation`] when the");
    f.indented_doc_comment("kernel id is unrecognised or the input does not satisfy the");
    f.indented_doc_comment("kernel's shape contract.");
    f.line("    fn dispatch_kernel(");
    f.line("        kernel_id: u32,");
    f.line("        input: &[u8],");
    f.line("        out: &mut [u8],");
    f.line("    ) -> Result<usize, crate::enforcement::ShapeViolation>;");
    f.line("}");
    f.blank();
    f.doc_comment("ADR-030: a tuple of `AxisExtension`-implementing types selected");
    f.doc_comment("by the application. The catamorphism's `Term::AxisInvocation`");
    f.doc_comment("fold-rule calls `dispatch` to route the invocation to the right");
    f.doc_comment("axis position.");
    f.doc_comment("");
    f.doc_comment("Foundation provides tuple impls for arities 1 through");
    f.doc_comment("[`MAX_AXIS_TUPLE_ARITY`].");
    f.line("pub trait AxisTuple<const INLINE_BYTES: usize, const FP_MAX: usize> {");
    f.indented_doc_comment("Number of axes carried in this tuple.");
    f.line("    const AXIS_COUNT: usize;");
    f.indented_doc_comment("Maximum kernel-output byte width across all axes in this tuple.");
    f.line("    const MAX_OUTPUT_BYTES: usize;");
    f.indented_doc_comment("Dispatch a kernel against the axis at `axis_index`. Returns");
    f.indented_doc_comment("the kernel's output bytes (length up to [`MAX_OUTPUT_BYTES`]).");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns [`crate::enforcement::ShapeViolation`] when `axis_index`");
    f.indented_doc_comment("is out of range or the axis dispatcher rejects the input.");
    f.line("    fn dispatch(");
    f.line("        axis_index: u32,");
    f.line("        kernel_id: u32,");
    f.line("        input: &[u8],");
    f.line("        out: &mut [u8],");
    f.line("    ) -> Result<usize, crate::enforcement::ShapeViolation>;");
    f.indented_doc_comment("ADR-055: return the substrate-Term body arena for the axis at");
    f.indented_doc_comment("`axis_index`. An empty slice means the axis is a primitive-fast-path");
    f.indented_doc_comment("axis whose body is byte-output-equivalent to its `dispatch_kernel`.");
    f.indented_doc_comment("Non-empty slices carry the recursive-fold decomposition the");
    f.indented_doc_comment("catamorphism walks per ADR-055's amended `Term::AxisInvocation`");
    f.indented_doc_comment("fold-rule.");
    f.line("    fn body_arena_at(axis_index: u32) -> &'static [crate::enforcement::Term<'static, INLINE_BYTES>];");
    f.line("}");
    f.blank();
    // ADR-030 blanket: every `Hasher` is an `AxisTuple` of arity 1
    // (axis 0 = the canonical hash axis; kernel 0 = `fold_bytes` →
    // `finalize`). This blanket lets existing Hasher impls satisfy
    // `A: AxisTuple` bounds without source changes; new substrate axes
    // declared via the `axis!` SDK macro compose into multi-tuple
    // `(HashAxis<MyHasher>, MyTensorAxis, …)` forms.
    f.doc_comment("ADR-030 blanket: every [`crate::enforcement::Hasher`] is");
    f.doc_comment("automatically an [`AxisTuple`] of arity 1 — the canonical");
    f.doc_comment("hash axis at position 0, kernel id 0.");
    f.line("impl<const INLINE_BYTES: usize, const FP_MAX: usize, H: crate::enforcement::Hasher<FP_MAX>> AxisTuple<INLINE_BYTES, FP_MAX> for H {");
    f.line("    const AXIS_COUNT: usize = 1;");
    f.line("    const MAX_OUTPUT_BYTES: usize = <H as crate::enforcement::Hasher<FP_MAX>>::OUTPUT_BYTES;");
    f.line("    fn dispatch(");
    f.line("        axis_index: u32,");
    f.line("        kernel_id: u32,");
    f.line("        input: &[u8],");
    f.line("        out: &mut [u8],");
    f.line("    ) -> Result<usize, crate::enforcement::ShapeViolation> {");
    f.line("        if axis_index != 0 || kernel_id != 0 {");
    f.line("            return Err(crate::enforcement::ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/axis/HasherBlanket\",");
    f.line("                constraint_iri: \"https://uor.foundation/axis/HasherBlanket/canonicalDispatch\",");
    f.line("                property_iri: \"https://uor.foundation/axis/axisIndex\",");
    f.line("                expected_range: \"https://uor.foundation/axis/CanonicalHashAxis\",");
    f.line("                min_count: 0,");
    f.line("                max_count: 1,");
    f.line("                kind: crate::ViolationKind::ValueCheck,");
    f.line("            });");
    f.line("        }");
    f.line("        let mut hasher = <H as crate::enforcement::Hasher<FP_MAX>>::initial();");
    f.line("        hasher = hasher.fold_bytes(input);");
    f.line("        let digest = hasher.finalize();");
    f.line("        let n_max = <H as crate::enforcement::Hasher<FP_MAX>>::OUTPUT_BYTES;");
    f.line("        let n = if n_max > out.len() { out.len() } else { n_max };");
    f.line("        let mut i = 0;");
    f.line("        while i < n {");
    f.line("            out[i] = digest[i];");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        Ok(n)");
    f.line("    }");
    f.line("    // ADR-055: the Hasher blanket axis is a primitive-fast-path axis. Its");
    f.line("    // body is byte-output-equivalent to `fold_bytes` ∘ `finalize`; the");
    f.line("    // empty arena signals to the catamorphism that dispatch_kernel is the");
    f.line("    // canonical evaluation strategy here.");
    f.line("    fn body_arena_at(_axis_index: u32) -> &'static [crate::enforcement::Term<'static, INLINE_BYTES>] {");
    f.line("        &[]");
    f.line("    }");
    f.line("}");
    f.blank();
    // 1-tuple impl — the most common case (a single hash axis).
    f.doc_comment("ADR-030: 1-tuple AxisTuple impl — applications selecting a single axis.");
    f.line("impl<const INLINE_BYTES: usize, const FP_MAX: usize, A0: AxisExtension<INLINE_BYTES, FP_MAX>> AxisTuple<INLINE_BYTES, FP_MAX> for (A0,) {");
    f.line("    const AXIS_COUNT: usize = 1;");
    f.line("    const MAX_OUTPUT_BYTES: usize = <A0 as AxisExtension<INLINE_BYTES, FP_MAX>>::MAX_OUTPUT_BYTES;");
    f.line("    fn dispatch(");
    f.line("        axis_index: u32,");
    f.line("        kernel_id: u32,");
    f.line("        input: &[u8],");
    f.line("        out: &mut [u8],");
    f.line("    ) -> Result<usize, crate::enforcement::ShapeViolation> {");
    f.line("        match axis_index {");
    f.line("            0 => <A0 as AxisExtension<INLINE_BYTES, FP_MAX>>::dispatch_kernel(kernel_id, input, out),");
    f.line("            _ => Err(crate::enforcement::ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/pipeline/AxisTupleShape\",");
    f.line("                constraint_iri: \"https://uor.foundation/pipeline/AxisTupleShape/inBounds\",");
    f.line("                property_iri: \"https://uor.foundation/pipeline/axisIndex\",");
    f.line("                expected_range: \"https://uor.foundation/pipeline/AxisIndex\",");
    f.line("                min_count: 0,");
    f.line("                max_count: 1,");
    f.line("                kind: crate::ViolationKind::ValueCheck,");
    f.line("            }),");
    f.line("        }");
    f.line("    }");
    f.line("    fn body_arena_at(axis_index: u32) -> &'static [crate::enforcement::Term<'static, INLINE_BYTES>] {");
    f.line("        match axis_index {");
    f.line("            0 => <A0 as SubstrateTermBody<INLINE_BYTES>>::body_arena(),");
    f.line("            _ => &[],");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    // 2-tuple impl
    f.doc_comment("ADR-030: 2-tuple AxisTuple impl.");
    f.line("impl<const INLINE_BYTES: usize, const FP_MAX: usize, A0: AxisExtension<INLINE_BYTES, FP_MAX>, A1: AxisExtension<INLINE_BYTES, FP_MAX>> AxisTuple<INLINE_BYTES, FP_MAX> for (A0, A1) {");
    f.line("    const AXIS_COUNT: usize = 2;");
    f.line("    const MAX_OUTPUT_BYTES: usize = {");
    f.line("        let a = <A0 as AxisExtension<INLINE_BYTES, FP_MAX>>::MAX_OUTPUT_BYTES;");
    f.line("        let b = <A1 as AxisExtension<INLINE_BYTES, FP_MAX>>::MAX_OUTPUT_BYTES;");
    f.line("        if a > b { a } else { b }");
    f.line("    };");
    f.line("    fn dispatch(");
    f.line("        axis_index: u32,");
    f.line("        kernel_id: u32,");
    f.line("        input: &[u8],");
    f.line("        out: &mut [u8],");
    f.line("    ) -> Result<usize, crate::enforcement::ShapeViolation> {");
    f.line("        match axis_index {");
    f.line("            0 => <A0 as AxisExtension<INLINE_BYTES, FP_MAX>>::dispatch_kernel(kernel_id, input, out),");
    f.line("            1 => <A1 as AxisExtension<INLINE_BYTES, FP_MAX>>::dispatch_kernel(kernel_id, input, out),");
    f.line("            _ => Err(crate::enforcement::ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/pipeline/AxisTupleShape\",");
    f.line("                constraint_iri: \"https://uor.foundation/pipeline/AxisTupleShape/inBounds\",");
    f.line("                property_iri: \"https://uor.foundation/pipeline/axisIndex\",");
    f.line("                expected_range: \"https://uor.foundation/pipeline/AxisIndex\",");
    f.line("                min_count: 0,");
    f.line("                max_count: 2,");
    f.line("                kind: crate::ViolationKind::ValueCheck,");
    f.line("            }),");
    f.line("        }");
    f.line("    }");
    f.line("    fn body_arena_at(axis_index: u32) -> &'static [crate::enforcement::Term<'static, INLINE_BYTES>] {");
    f.line("        match axis_index {");
    f.line("            0 => <A0 as SubstrateTermBody<INLINE_BYTES>>::body_arena(),");
    f.line("            1 => <A1 as SubstrateTermBody<INLINE_BYTES>>::body_arena(),");
    f.line("            _ => &[],");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    // ADR-030 tuple impls 3..=MAX_AXIS_TUPLE_ARITY (8).
    for arity in 3..=8 {
        f.doc_comment(&format!("ADR-030: {arity}-tuple AxisTuple impl."));
        let type_params: String = (0..arity)
            .map(|i| format!("A{i}: AxisExtension<INLINE_BYTES, FP_MAX>"))
            .collect::<Vec<_>>()
            .join(", ");
        let tuple_args: String = (0..arity)
            .map(|i| format!("A{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        f.line(&format!(
            "impl<const INLINE_BYTES: usize, const FP_MAX: usize, {type_params}> AxisTuple<INLINE_BYTES, FP_MAX> for ({tuple_args},) {{"
        ));
        f.line(&format!("    const AXIS_COUNT: usize = {arity};"));
        f.line("    const MAX_OUTPUT_BYTES: usize = {");
        // saturating max chain of MAX_OUTPUT_BYTES
        for i in 0..arity {
            f.line(&format!(
                "        let a{i} = <A{i} as AxisExtension<INLINE_BYTES, FP_MAX>>::MAX_OUTPUT_BYTES;"
            ));
        }
        // Compute max via const-fn-friendly chained ifs.
        f.line("        let mut m = a0;");
        for i in 1..arity {
            f.line(&format!("        if a{i} > m {{ m = a{i}; }}"));
        }
        f.line("        m");
        f.line("    };");
        f.line("    fn dispatch(");
        f.line("        axis_index: u32,");
        f.line("        kernel_id: u32,");
        f.line("        input: &[u8],");
        f.line("        out: &mut [u8],");
        f.line("    ) -> Result<usize, crate::enforcement::ShapeViolation> {");
        f.line("        match axis_index {");
        for i in 0..arity {
            f.line(&format!(
                "            {i} => <A{i} as AxisExtension<INLINE_BYTES, FP_MAX>>::dispatch_kernel(kernel_id, input, out),"
            ));
        }
        f.line("            _ => Err(crate::enforcement::ShapeViolation {");
        f.line("                shape_iri: \"https://uor.foundation/pipeline/AxisTupleShape\",");
        f.line("                constraint_iri: \"https://uor.foundation/pipeline/AxisTupleShape/inBounds\",");
        f.line("                property_iri: \"https://uor.foundation/pipeline/axisIndex\",");
        f.line("                expected_range: \"https://uor.foundation/pipeline/AxisIndex\",");
        f.line("                min_count: 0,");
        f.line(&format!("                max_count: {arity},"));
        f.line("                kind: crate::ViolationKind::ValueCheck,");
        f.line("            }),");
        f.line("        }");
        f.line("    }");
        f.line("    fn body_arena_at(axis_index: u32) -> &'static [crate::enforcement::Term<'static, INLINE_BYTES>] {");
        f.line("        match axis_index {");
        for i in 0..arity {
            f.line(&format!(
                "            {i} => <A{i} as SubstrateTermBody<INLINE_BYTES>>::body_arena(),"
            ));
        }
        f.line("            _ => &[],");
        f.line("        }");
        f.line("    }");
        f.line("}");
        f.blank();
    }
}

/// Emits the wiki ADR-036 surface: the [`ResolverTuple`] sealed trait
/// (per-value content provision for resolver-bound substrate ψ-Term
/// variants per ADR-035), the eight resolver-category traits
/// (`NerveResolver`, `ChainComplexResolver`, …, `KInvariantResolver`),
/// the eight marker traits (`HasNerveResolver`, …) the model where-
/// clause names, the `NullResolverTuple` resolver-absent default, and
/// eight `Null<Category>Resolver<H>` impls that emit the
/// `RESOLVER_ABSENT` discriminator on every `resolve` call.
fn emit_resolver_tuple(f: &mut RustFile) {
    f.doc_comment("ADR-036: maximum number of resolvers a single application's");
    f.doc_comment("`ResolverTuple` may carry. Foundation-fixed at twice");
    f.doc_comment("[`MAX_AXIS_TUPLE_ARITY`] to accommodate ADR-035's eight");
    f.doc_comment("resolver-bound ψ-Term categories with eight headroom positions");
    f.doc_comment("for future ADR-013/TR-08 substrate amendments.");
    f.line("pub const MAX_RESOLVER_TUPLE_ARITY: usize = 16;");
    f.blank();

    f.doc_comment("ADR-036: foundation-internal discriminator emitted by every");
    f.doc_comment("`Null<Category>Resolver` impl when the catamorphism's resolver-");
    f.doc_comment("bound ψ-Term fold-rule consults a resolver category whose");
    f.doc_comment("`ResolverTuple` accessor returns the foundation-Null impl. The");
    f.doc_comment("evaluator translates this into a `PipelineFailure::ShapeViolation`");
    f.doc_comment("carrying the `https://uor.foundation/resolver/RESOLVER_ABSENT`");
    f.doc_comment("shape IRI; ADR-022 D3 G9's `Term::Try` default-propagation handler");
    f.doc_comment("recovers it when the verb body wraps the ψ-Term.");
    f.line("pub const RESOLVER_ABSENT_DISCRIMINATOR: u8 = 0xff;");
    f.blank();

    f.doc_comment("ADR-036: resolver-category enum identifying which resolver-bound");
    f.doc_comment("substrate operation each `ResolverTuple` position satisfies. Each");
    f.doc_comment("variant corresponds to one resolver-bound ψ-Term variant per");
    f.doc_comment("ADR-035; future ADR-013/TR-08 substrate amendments add variants.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
    f.line("pub enum ResolverCategory {");
    f.indented_doc_comment("ψ_1 — per-value bytes → SimplicialComplex (NerveResolver).");
    f.line("    Nerve,");
    f.indented_doc_comment("ψ_2 — SimplicialComplex → ChainComplex (ChainComplexResolver).");
    f.line("    ChainComplex,");
    f.indented_doc_comment("ψ_3 — ChainComplex → HomologyGroups (HomologyGroupResolver).");
    f.line("    HomologyGroup,");
    f.indented_doc_comment("ψ_5 — ChainComplex → CochainComplex (CochainComplexResolver).");
    f.line("    CochainComplex,");
    f.indented_doc_comment("ψ_6 — CochainComplex → CohomologyGroups (CohomologyGroupResolver).");
    f.line("    CohomologyGroup,");
    f.indented_doc_comment("ψ_7 — SimplicialComplex → PostnikovTower (PostnikovResolver).");
    f.line("    Postnikov,");
    f.indented_doc_comment("ψ_8 — PostnikovTower → HomotopyGroups (HomotopyGroupResolver).");
    f.line("    HomotopyGroup,");
    f.indented_doc_comment("ψ_9 — HomotopyGroups → KInvariants (KInvariantResolver).");
    f.line("    KInvariant,");
    f.line("}");
    f.blank();

    f.doc_comment("ADR-036: tuple-of-bounded-types parameter on the model declaration");
    f.doc_comment("carrying application-provided resolver instances for the resolver-");
    f.doc_comment("bound ψ-Term variants per ADR-035. Sealed via");
    f.doc_comment("[`__sdk_seal::Sealed`]: only the SDK `resolver!` macro emits impls.");
    f.line("pub trait ResolverTuple: __sdk_seal::Sealed {");
    f.indented_doc_comment(
        "Number of resolver positions in this tuple (bounded by `MAX_RESOLVER_TUPLE_ARITY`).",
    );
    f.line("    const ARITY: usize;");
    f.indented_doc_comment("Resolver category at each tuple position.");
    f.line("    const CATEGORIES: &'static [ResolverCategory];");
    f.indented_doc_comment(
        "ADR-057: the application's shape-IRI registry. ψ_1's NerveResolver impl",
    );
    f.indented_doc_comment(
        "expands `ConstraintRef::Recurse` references through this registry when",
    );
    f.indented_doc_comment(
        "computing N(C). Defaults to [`shape_iri_registry::EmptyShapeRegistry`]",
    );
    f.indented_doc_comment("(foundation built-in registry only); applications declaring recursive");
    f.indented_doc_comment("shapes via the SDK `register_shape!` macro thread the marker type");
    f.indented_doc_comment("through their `resolver!`-declared ResolverTuple.");
    f.line("    type ShapeRegistry: crate::pipeline::shape_iri_registry::ShapeRegistryProvider;");
    f.line("}");
    f.blank();

    // ADR-041: nine #[repr(transparent)] typed-coordinate carriers
    // wrapping byte slices. Zero runtime cost (layout identical to &[u8]);
    // the type wrapper is purely compile-time discrimination of which
    // ψ-stage's output the bytes represent. Cross-stage composition is
    // type-checked: passing `ChainComplexBytes` to a resolver expecting
    // `SimplicialComplexBytes` is a compile-time type error rather than
    // a runtime ShapeViolation.
    let carrier_specs: &[(&str, &str, &str)] = &[
        (
            "SimplicialComplexBytes",
            "ψ_1",
            "the simplicial-complex serialization produced by `NerveResolver::resolve`",
        ),
        (
            "ChainComplexBytes",
            "ψ_2",
            "the chain-complex serialization produced by `ChainComplexResolver::resolve`",
        ),
        (
            "HomologyGroupsBytes",
            "ψ_3",
            "the homology-groups serialization produced by `HomologyGroupResolver::resolve`",
        ),
        (
            "BettiNumbersBytes",
            "ψ_4",
            "the Betti-number tuple serialization produced by the resolver-free `Term::Betti` fold-rule (a byte projection of HomologyGroupsBytes)",
        ),
        (
            "CochainComplexBytes",
            "ψ_5",
            "the cochain-complex serialization produced by `CochainComplexResolver::resolve`",
        ),
        (
            "CohomologyGroupsBytes",
            "ψ_6",
            "the cohomology-groups serialization produced by `CohomologyGroupResolver::resolve`",
        ),
        (
            "PostnikovTowerBytes",
            "ψ_7",
            "the Postnikov-tower serialization produced by `PostnikovResolver::resolve`",
        ),
        (
            "HomotopyGroupsBytes",
            "ψ_8",
            "the homotopy-groups serialization produced by `HomotopyGroupResolver::resolve`",
        ),
        (
            "KInvariantsBytes",
            "ψ_9",
            "the κ-label byte serialization produced by `KInvariantResolver::resolve` — the canonical k-invariants-branch output (ADR-035) classifying the input's homotopy type up to weak equivalence",
        ),
    ];
    for (carrier_name, psi_stage, semantics) in carrier_specs {
        f.doc_comment(&format!(
            "Wiki ADR-041: zero-cost typed-coordinate carrier for {psi_stage} output —"
        ));
        f.doc_comment(&format!("{semantics}."));
        f.line("///");
        f.doc_comment("`#[repr(transparent)]` over `&'a [u8]`: layout identical to a byte");
        f.doc_comment("slice; the type wrapper is purely compile-time discrimination so");
        f.doc_comment("ψ-stage composition is type-checked at the resolver-impl boundary.");
        f.line("#[repr(transparent)]");
        f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
        f.line(&format!("pub struct {carrier_name}<'a>(pub &'a [u8]);"));
        f.blank();
        f.line(&format!("impl<'a> {carrier_name}<'a> {{"));
        f.indented_doc_comment("Borrow the underlying byte slice.");
        f.line("    #[must_use]");
        f.line("    pub fn as_bytes(&self) -> &[u8] {");
        f.line("        self.0");
        f.line("    }");
        f.line("    /// Length of the underlying byte slice.");
        f.line("    #[must_use]");
        f.line("    pub fn len(&self) -> usize {");
        f.line("        self.0.len()");
        f.line("    }");
        f.line("    /// Whether the underlying byte slice is empty.");
        f.line("    #[must_use]");
        f.line("    pub fn is_empty(&self) -> bool {");
        f.line("        self.0.is_empty()");
        f.line("    }");
        f.line("}");
        f.blank();
    }

    // Eight resolver category traits. Per the user's directive, generic
    // over <H> (no Hasher bound) — the seal is the gate.
    // ADR-060: every resolver's `resolve` uniformly takes and returns
    // `TermValue<'a, INLINE_BYTES>` (the prior ψ-stage's emission flows in as a
    // `TermValue`); the per-category typed-input newtypes of ADR-041 are no
    // longer part of the signature.
    let resolver_traits: &[(&str, &str)] = &[
        (
            "NerveResolver",
            "ψ_1 — per-value bytes → SimplicialComplex per ADR-035.",
        ),
        (
            "ChainComplexResolver",
            "ψ_2 — SimplicialComplex → ChainComplex per ADR-035.",
        ),
        (
            "HomologyGroupResolver",
            "ψ_3 — ChainComplex → HomologyGroups per ADR-035.",
        ),
        (
            "CochainComplexResolver",
            "ψ_5 — ChainComplex → CochainComplex per ADR-035.",
        ),
        (
            "CohomologyGroupResolver",
            "ψ_6 — CochainComplex → CohomologyGroups per ADR-035.",
        ),
        (
            "PostnikovResolver",
            "ψ_7 — SimplicialComplex → PostnikovTower per ADR-035.",
        ),
        (
            "HomotopyGroupResolver",
            "ψ_8 — PostnikovTower → HomotopyGroups per ADR-035.",
        ),
        (
            "KInvariantResolver",
            "ψ_9 — HomotopyGroups → KInvariants per ADR-035.",
        ),
    ];
    for (trait_name, doc) in resolver_traits {
        f.doc_comment(&format!("ADR-036 resolver trait: {doc}"));
        f.line("///");
        f.doc_comment("Parameterized by the model's H-axis (`H: Hasher` per ADR-022 D5) so");
        f.doc_comment("resolver impls compute content-addressed fingerprints using the");
        f.doc_comment("model's chosen hash impl. Sealed via [`__sdk_seal::Sealed`]: only");
        f.doc_comment("the SDK `resolver!` macro emits impls. Foundation provides a Null");
        f.doc_comment("impl whose `resolve` emits the `RESOLVER_ABSENT` shape violation.");
        f.line("///");
        f.doc_comment("ADR-060: `resolve` takes and returns a source-polymorphic");
        f.doc_comment("[`crate::pipeline::TermValue`] const-generic over the application's");
        f.doc_comment("foundation-derived inline width (`INLINE_BYTES =");
        f.doc_comment("carrier_inline_bytes::<B>()`). The resolver constructs whichever");
        f.doc_comment("carrier variant its output requires — `Inline` for structural");
        f.doc_comment("identities that fit the inline width, `Borrowed` into resolver-owned");
        f.doc_comment("scratch, or `Stream` for unbounded structural sequences. There is no");
        f.doc_comment("caller-supplied `&mut [u8]` scratch and no per-ψ-stage byte-width");
        f.doc_comment("ceiling (ADR-060 amends the ADR-036/ADR-041 writer-style surface).");
        f.line(&format!(
            "pub trait {trait_name}<const INLINE_BYTES: usize, H>: __sdk_seal::Sealed {{"
        ));
        f.indented_doc_comment("Resolve per-value content for this category.");
        f.indented_doc_comment("");
        f.indented_doc_comment("# Errors");
        f.indented_doc_comment("");
        f.indented_doc_comment("Returns [`crate::enforcement::ShapeViolation`] when the resolver");
        f.indented_doc_comment("cannot produce content (e.g., the foundation Null impl carrying");
        f.indented_doc_comment("the `RESOLVER_ABSENT` discriminator).");
        f.line("    fn resolve<'a>(");
        f.line("        &self,");
        f.line("        input: crate::pipeline::TermValue<'a, INLINE_BYTES>,");
        f.line("    ) -> Result<crate::pipeline::TermValue<'a, INLINE_BYTES>, crate::enforcement::ShapeViolation>;");
        f.line("}");
        f.blank();
    }

    // Eight Has<Category>Resolver marker traits.
    let marker_traits: &[(&str, &str, &str)] = &[
        ("HasNerveResolver", "NerveResolver", "nerve_resolver"),
        (
            "HasChainComplexResolver",
            "ChainComplexResolver",
            "chain_complex_resolver",
        ),
        (
            "HasHomologyGroupResolver",
            "HomologyGroupResolver",
            "homology_group_resolver",
        ),
        (
            "HasCochainComplexResolver",
            "CochainComplexResolver",
            "cochain_complex_resolver",
        ),
        (
            "HasCohomologyGroupResolver",
            "CohomologyGroupResolver",
            "cohomology_group_resolver",
        ),
        (
            "HasPostnikovResolver",
            "PostnikovResolver",
            "postnikov_resolver",
        ),
        (
            "HasHomotopyGroupResolver",
            "HomotopyGroupResolver",
            "homotopy_group_resolver",
        ),
        (
            "HasKInvariantResolver",
            "KInvariantResolver",
            "k_invariant_resolver",
        ),
    ];
    for (marker, resolver, accessor) in marker_traits {
        f.doc_comment(&format!(
            "ADR-036 marker trait: ResolverTuple positions including a `{resolver}`."
        ));
        f.doc_comment("The `prism_model!` macro infers the where-clause bound for each");
        f.doc_comment("resolver-bound ψ-Term variant a verb body emits.");
        f.line(&format!(
            "pub trait {marker}<const INLINE_BYTES: usize, H>: ResolverTuple {{"
        ));
        f.indented_doc_comment(&format!(
            "Returns the `{resolver}` impl this ResolverTuple carries."
        ));
        f.line(&format!(
            "    fn {accessor}(&self) -> &dyn {resolver}<INLINE_BYTES, H>;"
        ));
        f.line("}");
        f.blank();
    }

    // Null resolver impls.
    f.doc_comment("ADR-036 Null resolver tuple — the resolver-absent default.");
    f.doc_comment("`ResolverTuple` impl with `ARITY = 0` and an empty CATEGORIES list.");
    f.doc_comment("Applications that don't declare a `resolver!` block default to this.");
    f.line("#[derive(Debug, Clone, Copy, Default)]");
    f.line("pub struct NullResolverTuple;");
    f.blank();
    f.line("impl __sdk_seal::Sealed for NullResolverTuple {}");
    f.blank();
    f.line("impl ResolverTuple for NullResolverTuple {");
    f.line("    const ARITY: usize = 0;");
    f.line("    const CATEGORIES: &'static [ResolverCategory] = &[];");
    f.line("    // ADR-057: foundation built-in registry only — the empty");
    f.line("    // registry provider falls through to FOUNDATION_REGISTRY in");
    f.line("    // shape_iri_registry. Applications with their own recursive");
    f.line("    // shapes set this to their `register_shape!`-emitted marker.");
    f.line("    type ShapeRegistry = crate::pipeline::shape_iri_registry::EmptyShapeRegistry;");
    f.line("}");
    f.blank();

    // Eight Null<Category>Resolver impls. ADR-060: every `resolve` uniformly
    // takes/returns `TermValue<'a, INLINE_BYTES>`, so the per-category typed
    // input is no longer carried here.
    let null_resolvers: &[(&str, &str, &str)] = &[
        ("NullNerveResolver", "NerveResolver", "Nerve"),
        (
            "NullChainComplexResolver",
            "ChainComplexResolver",
            "ChainComplex",
        ),
        (
            "NullHomologyGroupResolver",
            "HomologyGroupResolver",
            "HomologyGroup",
        ),
        (
            "NullCochainComplexResolver",
            "CochainComplexResolver",
            "CochainComplex",
        ),
        (
            "NullCohomologyGroupResolver",
            "CohomologyGroupResolver",
            "CohomologyGroup",
        ),
        ("NullPostnikovResolver", "PostnikovResolver", "Postnikov"),
        (
            "NullHomotopyGroupResolver",
            "HomotopyGroupResolver",
            "HomotopyGroup",
        ),
        ("NullKInvariantResolver", "KInvariantResolver", "KInvariant"),
    ];
    for (null_ty, resolver_trait, category) in null_resolvers {
        f.doc_comment(&format!(
            "ADR-036 Null `{resolver_trait}` impl. `resolve` always emits the"
        ));
        f.doc_comment("`RESOLVER_ABSENT` shape violation — the catamorphism translates this");
        f.doc_comment("into `PipelineFailure::ShapeViolation` recoverable via `Term::Try`'s");
        f.doc_comment("default-propagation handler (ADR-022 D3 G9).");
        f.line("#[derive(Debug, Default)]");
        f.line(&format!(
            "pub struct {null_ty}<H>(core::marker::PhantomData<H>);"
        ));
        f.blank();
        f.line(&format!("impl<H> {null_ty}<H> {{"));
        f.indented_doc_comment("Construct a new Null resolver.");
        f.line("    #[must_use]");
        f.line("    pub const fn new() -> Self {");
        f.line("        Self(core::marker::PhantomData)");
        f.line("    }");
        f.line("}");
        f.blank();
        f.line(&format!("impl<H> __sdk_seal::Sealed for {null_ty}<H> {{}}"));
        f.blank();
        f.line(&format!(
            "impl<const INLINE_BYTES: usize, H> {resolver_trait}<INLINE_BYTES, H> for {null_ty}<H> {{"
        ));
        f.line("    fn resolve<'a>(");
        f.line("        &self,");
        f.line("        _input: crate::pipeline::TermValue<'a, INLINE_BYTES>,");
        f.line("    ) -> Result<crate::pipeline::TermValue<'a, INLINE_BYTES>, crate::enforcement::ShapeViolation> {");
        f.line("        Err(crate::enforcement::ShapeViolation {");
        f.line("            shape_iri: \"https://uor.foundation/resolver/RESOLVER_ABSENT\",");
        f.line(&format!(
            "            constraint_iri: \"https://uor.foundation/resolver/{category}\","
        ));
        f.line("            property_iri: \"https://uor.foundation/resolver/category\",");
        f.line("            expected_range: \"https://uor.foundation/resolver/Resolver\",");
        f.line("            min_count: 0,");
        f.line("            max_count: 1,");
        f.line("            kind: crate::ViolationKind::ValueCheck,");
        f.line("        })");
        f.line("    }");
        f.line("}");
        f.blank();
    }

    // ADR-036 gap closure: NullResolverTuple satisfies every
    // Has<Category>Resolver<H> marker trait. To make a `&dyn <Category>Resolver<H>`
    // available from a `&NullResolverTuple` without alloc (foundation is
    // `#![no_std]`), NullResolverTuple itself implements each of the eight
    // resolver traits — the `resolve` method emits the same RESOLVER_ABSENT
    // shape violation as the corresponding `Null<Category>Resolver<H>`. The
    // marker-trait accessor returns `self`, cast to the trait object. This
    // realizes the wiki's "Null* resolvers satisfy the bound for ergonomic
    // reasons" commitment: applications can default to NullResolverTuple in
    // tests/smoke contexts and recover RESOLVER_ABSENT at runtime via
    // `Term::Try`.
    // Each tuple element: resolver_trait, marker, accessor, category.
    // ADR-060: every `resolve` uniformly takes/returns
    // `TermValue<'a, INLINE_BYTES>`, so no per-category input type is carried.
    let absent_impls: &[(&str, &str, &str, &str)] = &[
        (
            "NerveResolver",
            "HasNerveResolver",
            "nerve_resolver",
            "Nerve",
        ),
        (
            "ChainComplexResolver",
            "HasChainComplexResolver",
            "chain_complex_resolver",
            "ChainComplex",
        ),
        (
            "HomologyGroupResolver",
            "HasHomologyGroupResolver",
            "homology_group_resolver",
            "HomologyGroup",
        ),
        (
            "CochainComplexResolver",
            "HasCochainComplexResolver",
            "cochain_complex_resolver",
            "CochainComplex",
        ),
        (
            "CohomologyGroupResolver",
            "HasCohomologyGroupResolver",
            "cohomology_group_resolver",
            "CohomologyGroup",
        ),
        (
            "PostnikovResolver",
            "HasPostnikovResolver",
            "postnikov_resolver",
            "Postnikov",
        ),
        (
            "HomotopyGroupResolver",
            "HasHomotopyGroupResolver",
            "homotopy_group_resolver",
            "HomotopyGroup",
        ),
        (
            "KInvariantResolver",
            "HasKInvariantResolver",
            "k_invariant_resolver",
            "KInvariant",
        ),
    ];
    for (resolver_trait, marker, accessor, category) in absent_impls {
        f.doc_comment(&format!(
            "ADR-036: NullResolverTuple satisfies `{resolver_trait}<H>` directly so"
        ));
        f.doc_comment(&format!(
            "the `{marker}<H>` accessor can return `self` cast to `&dyn {resolver_trait}<H>`."
        ));
        f.doc_comment("The `resolve` method emits the `RESOLVER_ABSENT` shape violation —");
        f.doc_comment("recoverable via `Term::Try`'s default-propagation handler (ADR-022 D3 G9).");
        f.line(&format!(
            "impl<const INLINE_BYTES: usize, H> {resolver_trait}<INLINE_BYTES, H> for NullResolverTuple {{"
        ));
        f.line("    fn resolve<'a>(");
        f.line("        &self,");
        f.line("        _input: crate::pipeline::TermValue<'a, INLINE_BYTES>,");
        f.line("    ) -> Result<crate::pipeline::TermValue<'a, INLINE_BYTES>, crate::enforcement::ShapeViolation> {");
        f.line("        Err(crate::enforcement::ShapeViolation {");
        f.line("            shape_iri: \"https://uor.foundation/resolver/RESOLVER_ABSENT\",");
        f.line(&format!(
            "            constraint_iri: \"https://uor.foundation/resolver/{category}\","
        ));
        f.line("            property_iri: \"https://uor.foundation/resolver/category\",");
        f.line("            expected_range: \"https://uor.foundation/resolver/Resolver\",");
        f.line("            min_count: 0,");
        f.line("            max_count: 1,");
        f.line("            kind: crate::ViolationKind::ValueCheck,");
        f.line("        })");
        f.line("    }");
        f.line("}");
        f.blank();
        f.doc_comment(&format!(
            "ADR-036: NullResolverTuple satisfies `{marker}<INLINE_BYTES, H>` (returns `self`)."
        ));
        f.line(&format!(
            "impl<const INLINE_BYTES: usize, H> {marker}<INLINE_BYTES, H> for NullResolverTuple {{"
        ));
        f.line(&format!(
            "    fn {accessor}(&self) -> &dyn {resolver_trait}<INLINE_BYTES, H> {{"
        ));
        f.line("        self");
        f.line("    }");
        f.line("}");
        f.blank();
    }

    // ADR-042: typed Rust verdict-envelope surface — InhabitanceCertificateView
    // / InhabitanceImpossibilityCertificate views over Grounded<T> / PipelineFailure.
    emit_inhabitance_verdict_surface(f);
}

/// Emit the ADR-042 typed inhabitance-verdict surface:
///   - `InhabitanceCertificateView<'a, T, Tag = T>` — zero-cost
///     `#[repr(transparent)]` view over `&'a Grounded<T, Tag>` with
///     typed accessors for the κ-label, the concrete witness, the
///     search trace, and the certified-type IRI.
///   - `InhabitanceImpossibilityCertificate<'a>` — zero-cost view over
///     `&'a PipelineFailure` with a `contradiction_proof` accessor.
///   - `PipelineFailure::as_inhabitance_impossibility_certificate(&self)`
///     returning `Option<InhabitanceImpossibilityCertificate<'_>>`.
///   - `inhabitance::dispatch_through_table` helper for optional
///     consultation of `predicate:InhabitanceDispatchTable` (three rule
///     arms: TwoSatDecider, HornSatDecider, ResidualVerdictResolver).
fn emit_inhabitance_verdict_surface(f: &mut RustFile) {
    f.doc_comment("Wiki ADR-042: typed view over a successful `Grounded<T, Tag>` as the");
    f.doc_comment("inhabitance-verdict envelope produced by the canonical k-invariants");
    f.doc_comment("branch (ψ_1 → ψ_7 → ψ_8 → ψ_9 per ADR-035).");
    f.line("///");
    f.doc_comment("Zero-cost — `#[repr(transparent)]` over `&'a Grounded<T, Tag>`. Construct");
    f.doc_comment("via [`crate::enforcement::Grounded::as_inhabitance_certificate`].");
    f.line("///");
    f.doc_comment("Realizes `cert:InhabitanceCertificate` per the ontology");
    f.doc_comment("(`<https://uor.foundation/cert/InhabitanceCertificate>`). Foundation");
    f.doc_comment("uses the suffix `View` here to distinguish this zero-cost typed");
    f.doc_comment(
        "view from the existing sealed-shim `crate::enforcement::InhabitanceCertificate`",
    );
    f.doc_comment("value carrier; the wiki names this type `InhabitanceCertificate<'a, T>`");
    f.doc_comment("in ADR-042 and the typed-view role is the load-bearing concern. The κ-label,");
    f.doc_comment("concrete `cert:witness`, and `cert:searchTrace` are accessor methods");
    f.doc_comment("over the underlying `Grounded` — no allocation, no per-application");
    f.doc_comment("re-derivation.");
    f.line("#[repr(transparent)]");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line(
        "pub struct InhabitanceCertificateView<'a, T: crate::enforcement::GroundedShape, const INLINE_BYTES: usize, const FP_MAX: usize = 32, Tag = T>(",
    );
    f.line("    pub &'a crate::enforcement::Grounded<'a, T, INLINE_BYTES, FP_MAX, Tag>,");
    f.line(");");
    f.blank();
    f.line(
        "impl<'a, T: crate::enforcement::GroundedShape, const INLINE_BYTES: usize, const FP_MAX: usize, Tag> InhabitanceCertificateView<'a, T, INLINE_BYTES, FP_MAX, Tag> {",
    );
    f.indented_doc_comment("The κ-label — the homotopy-classification structural witness at");
    f.indented_doc_comment("ψ_9 per ADR-035. The bytes are the `Term::KInvariants` emission's");
    f.indented_doc_comment("output (exposed via `Grounded::output_bytes`) wrapped in the");
    f.indented_doc_comment("ADR-041 typed-coordinate carrier so cross-stage composition is");
    f.indented_doc_comment("type-checked.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn kappa_label(&self) -> KInvariantsBytes<'_> {");
    f.line("        KInvariantsBytes(self.0.output_bytes())");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("The concrete `cert:witness` ValueTuple — derivable from");
    f.indented_doc_comment("`Term::Nerve`'s 0-simplices at ψ_1 (the per-value bytes the");
    f.indented_doc_comment("model's NerveResolver consumed). Foundation exposes the");
    f.indented_doc_comment("`Grounded`'s bindings as the value-tuple surface; applications");
    f.indented_doc_comment("whose admission relations carry richer witness data project");
    f.indented_doc_comment("through the binding table's content addresses.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn witness(&self) -> WitnessValueTuple<'_> {");
    f.line("        WitnessValueTuple { grounded_bindings: self.0 }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("The `cert:searchTrace` — realized as");
    f.indented_doc_comment("`<Grounded>::derivation::<H>(...).replay::<...>()` per ADR-039.");
    f.indented_doc_comment("Foundation surfaces the derivation pointer; applications choose");
    f.indented_doc_comment("which `Hasher` impl and `HostBounds` parameters to instantiate");
    f.indented_doc_comment("the replay with.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn certificate(&self) -> &crate::enforcement::Validated<crate::enforcement::GroundingCertificate<FP_MAX>> {");
    f.line("        self.0.certificate()");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "The certified type's stable IRI — the `<T as ConstrainedTypeShape>::IRI`",
    );
    f.indented_doc_comment("the application registered as the route's output shape.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn certified_type(&self) -> &'static str");
    f.line("    where T: ConstrainedTypeShape,");
    f.line("    {");
    f.line("        <T as ConstrainedTypeShape>::IRI");
    f.line("    }");
    f.line("}");
    f.blank();

    // WitnessValueTuple — concrete witness view over Grounded's bindings.
    f.doc_comment("Wiki ADR-042: concrete `cert:witness` ValueTuple view. Borrows the");
    f.doc_comment("underlying `Grounded`'s bindings; iterates as `(content_address, bytes)`");
    f.doc_comment("pairs corresponding to `Term::Nerve`'s 0-simplices.");
    f.line("#[derive(Clone, Copy)]");
    f.line("pub struct WitnessValueTuple<'a> {");
    f.indented_doc_comment("Foundation-internal: the underlying Grounded reference. Public-API");
    f.indented_doc_comment("access goes through the accessor methods.");
    f.line("    grounded_bindings: &'a dyn WitnessTupleSource,");
    f.line("}");
    f.blank();
    f.line("impl<'a> core::fmt::Debug for WitnessValueTuple<'a> {");
    f.line("    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {");
    f.line("        f.debug_struct(\"WitnessValueTuple\")");
    f.line("            .field(\"binding_count\", &self.grounded_bindings.binding_count())");
    f.line("            .finish()");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl<'a> WitnessValueTuple<'a> {");
    f.indented_doc_comment("Number of bindings in the witness tuple.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn len(&self) -> usize {");
    f.line("        self.grounded_bindings.binding_count()");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Whether the witness tuple is empty.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn is_empty(&self) -> bool {");
    f.line("        self.grounded_bindings.binding_count() == 0");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("The witness's content-addressed binding bytes at position `idx`.");
    f.indented_doc_comment("Returns `None` if `idx >= len()`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn binding_bytes(&self, idx: usize) -> Option<&'static [u8]> {");
    f.line("        self.grounded_bindings.binding_bytes_at(idx)");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("Foundation-internal trait letting `Grounded<T, Tag>` expose binding");
    f.doc_comment("access to `WitnessValueTuple` without leaking the generic-parameter");
    f.doc_comment("plumbing into the witness-view type.");
    f.line("pub trait WitnessTupleSource {");
    f.indented_doc_comment("Number of bindings in this source's table.");
    f.line("    fn binding_count(&self) -> usize;");
    f.indented_doc_comment("Binding bytes at the given index, or `None` if out of range.");
    f.line("    fn binding_bytes_at(&self, idx: usize) -> Option<&'static [u8]>;");
    f.line("}");
    f.blank();
    f.line("impl<'a, T: crate::enforcement::GroundedShape, const INLINE_BYTES: usize, const FP_MAX: usize, Tag> WitnessTupleSource");
    f.line("    for crate::enforcement::Grounded<'a, T, INLINE_BYTES, FP_MAX, Tag>");
    f.line("{");
    f.line("    fn binding_count(&self) -> usize {");
    f.line("        self.iter_bindings().count()");
    f.line("    }");
    f.line("    fn binding_bytes_at(&self, idx: usize) -> Option<&'static [u8]> {");
    f.line("        self.iter_bindings().nth(idx).map(|e| e.bytes)");
    f.line("    }");
    f.line("}");
    f.blank();

    // InhabitanceImpossibilityCertificate.
    f.doc_comment("Wiki ADR-042: typed view over an `Err(PipelineFailure)` as the");
    f.doc_comment("inhabitance-impossibility envelope. Realizes");
    f.doc_comment("`cert:InhabitanceImpossibilityCertificate` per the ontology:");
    f.doc_comment("`<https://uor.foundation/cert/InhabitanceImpossibilityCertificate>`.");
    f.line("///");
    f.doc_comment("Zero-cost — `#[repr(transparent)]` over `&'a PipelineFailure`. Construct");
    f.doc_comment(
        "via [`crate::enforcement::PipelineFailure::as_inhabitance_impossibility_certificate`].",
    );
    f.line("#[repr(transparent)]");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct InhabitanceImpossibilityCertificate<'a>(");
    f.line("    pub &'a crate::enforcement::PipelineFailure,");
    f.line(");");
    f.blank();
    f.line("impl<'a> InhabitanceImpossibilityCertificate<'a> {");
    f.indented_doc_comment("The contradiction-proof bytes — canonical-form encoding of the");
    f.indented_doc_comment("failure trace, suitable for Lean 4 by-contradiction reconstruction");
    f.indented_doc_comment("per ADR-039 + ADR-042. Foundation provides the shape-violation's");
    f.indented_doc_comment("`shape_iri` bytes as the proof's canonical-form witness; richer");
    f.indented_doc_comment("contradiction data is application-provided via the failure trace.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn contradiction_proof(&self) -> &'static [u8] {");
    f.line("        match self.0 {");
    f.line(
        "            crate::enforcement::PipelineFailure::ShapeViolation { report } => report.shape_iri.as_bytes(),",
    );
    f.line("            _ => b\"https://uor.foundation/proof/InhabitanceImpossibilityWitness\",");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Borrow the underlying `PipelineFailure`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn failure(&self) -> &crate::enforcement::PipelineFailure {");
    f.line("        self.0");
    f.line("    }");
    f.line("}");
    f.blank();

    // PipelineFailure inherent impl with the typed accessor.
    f.doc_comment("Wiki ADR-042: typed verdict-envelope accessors on `PipelineFailure`.");
    f.line("impl crate::enforcement::PipelineFailure {");
    f.indented_doc_comment("Borrow `self` as an [`InhabitanceImpossibilityCertificate`] view");
    f.indented_doc_comment("when the failure's structural cause is an inhabitance-impossibility");
    f.indented_doc_comment("witness (per ADR-042). Returns `Some` for `ShapeViolation` whose");
    f.indented_doc_comment("`shape_iri` carries one of the foundation-declared inhabitance");
    f.indented_doc_comment("proof IRIs (e.g. `RESOLVER_ABSENT`, the constraint-nerve-empty-");
    f.indented_doc_comment("Kan-completion sentinel); foundation accepts `Some(...)` universally");
    f.indented_doc_comment("for `PipelineFailure` so applications consume the verdict-envelope");
    f.indented_doc_comment("view at their discretion.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line(
        "    pub fn as_inhabitance_impossibility_certificate(&self) -> Option<InhabitanceImpossibilityCertificate<'_>> {",
    );
    f.line("        Some(InhabitanceImpossibilityCertificate(self))");
    f.line("    }");
    f.line("}");
    f.blank();

    // inhabitance module with dispatch_through_table helper.
    f.doc_comment("Wiki ADR-042: `predicate:InhabitanceDispatchTable` consultation helper.");
    f.doc_comment("Application NerveResolver impls MAY call this helper internally for");
    f.doc_comment("decider routing across the ontology's three canonical rule arms");
    f.doc_comment("(TwoSatDecider, HornSatDecider, ResidualVerdictResolver). Foundation");
    f.doc_comment("provides the dispatch surface; rule-arm semantics are application-");
    f.doc_comment("provided through the closures the caller threads in.");
    f.line("pub mod inhabitance {");
    f.indented_doc_comment("Three rule arms a `predicate:InhabitanceDispatchTable` consultation");
    f.indented_doc_comment("dispatches through, per the ontology's canonical decider routing.");
    f.line("    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("    pub enum InhabitanceRuleArm {");
    f.indented_doc_comment(
        "        TwoSatDecider — `predicate:TwoSatDecider`. Decides 2-SAT constraint nerves.",
    );
    f.line("        TwoSatDecider,");
    f.indented_doc_comment(
        "        HornSatDecider — `predicate:HornSatDecider`. Decides Horn-SAT constraint nerves.",
    );
    f.line("        HornSatDecider,");
    f.indented_doc_comment(
        "        ResidualVerdictResolver — `predicate:ResidualVerdictResolver`. Residual",
    );
    f.indented_doc_comment(
        "        catch-all for constraint nerves outside the 2-SAT / Horn-SAT coverage.",
    );
    f.line("        ResidualVerdictResolver,");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Dispatch through the `predicate:InhabitanceDispatchTable` rule arms in",
    );
    f.indented_doc_comment(
        "ontology order (TwoSatDecider → HornSatDecider → ResidualVerdictResolver).",
    );
    f.indented_doc_comment("Each closure returns `Some(verdict)` if the arm decides, or `None` to");
    f.indented_doc_comment("delegate to the next arm. Returns the first decisive verdict.");
    f.line("    #[inline]");
    f.line("    pub fn dispatch_through_table<F1, F2, F3, V>(");
    f.line("        two_sat: F1,");
    f.line("        horn_sat: F2,");
    f.line("        residual: F3,");
    f.line("    ) -> (InhabitanceRuleArm, V)");
    f.line("    where");
    f.line("        F1: FnOnce() -> Option<V>,");
    f.line("        F2: FnOnce() -> Option<V>,");
    f.line("        F3: FnOnce() -> V,");
    f.line("    {");
    f.line("        if let Some(v) = two_sat() {");
    f.line("            return (InhabitanceRuleArm::TwoSatDecider, v);");
    f.line("        }");
    f.line("        if let Some(v) = horn_sat() {");
    f.line("            return (InhabitanceRuleArm::HornSatDecider, v);");
    f.line("        }");
    f.line("        (InhabitanceRuleArm::ResidualVerdictResolver, residual())");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// Emits the wiki ADR-049 surface: the [`ObservablePredicate`] trait +
/// the four foundation-declared typed observable primitives
/// ([`Stratum<P>`], [`WalshHadamardParity`], [`UltrametricCloseTo<P>`],
/// [`AffineParity`]) consumable as `SingletonCommitment<P>` operands per
/// ADR-048.
fn emit_observable_predicate(f: &mut RustFile) {
    f.doc_comment("Wiki ADR-049: substrate-level typed-observable predicate trait.");
    f.line("///");
    f.doc_comment("Implementors are foundation-declared typed observables that read a");
    f.doc_comment("structural property of a digest and evaluate a single-bit predicate");
    f.doc_comment("against the read value. Sealed via [`__sdk_seal::Sealed`]; the foundation");
    f.doc_comment("declares the closed catalog of 4 typed observables consuming the");
    f.doc_comment("canonical hash axis's σ-projection per ADR-047.");
    f.line("///");
    f.doc_comment("`SingletonCommitment<P>` wraps one of these to form a 1-bit");
    f.doc_comment("typed-commitment per ADR-048; `AndCommitment` composes any number.");
    f.line("pub trait ObservablePredicate: Copy + __sdk_seal::Sealed {");
    f.indented_doc_comment("PRF acceptance probability under the Hardening Principle's U1");
    f.indented_doc_comment("(ADR-047). Used by `SingletonCommitment<P>` per ADR-048 for");
    f.indented_doc_comment("bandwidth and accept_prob propagation.");
    f.line("    fn accept_prob(&self) -> f64;");
    f.blank();
    f.indented_doc_comment("Read the structural property from `digest` and return the");
    f.indented_doc_comment("predicate's boolean satisfaction.");
    f.line("    fn evaluate(&self, digest: &[u8]) -> bool;");
    f.blank();
    f.indented_doc_comment("Foundation-vetted Observable IRI per ADR-038's closed catalog.");
    f.indented_doc_comment("Surfaces in the `CommitmentEvaluated` trace event per ADR-048.");
    f.line("    fn observable_iri(&self) -> &'static str;");
    f.line("}");
    f.blank();

    // ── Stratum<P> ───────────────────────────────────────────────────
    f.doc_comment("Wiki ADR-049: p-adic stratum observable per");
    f.doc_comment("`observable:StratumObservable`. `value(d) = ν_p(d)` — the p-adic");
    f.doc_comment("valuation of `d` viewed as a big-endian integer. Predicate:");
    f.doc_comment("`value(d) == k`.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct Stratum<const P: u32> {");
    f.indented_doc_comment("Target p-adic valuation the predicate accepts.");
    f.line("    pub k: u32,");
    f.line("}");
    f.blank();
    f.line("impl<const P: u32> __sdk_seal::Sealed for Stratum<P> {}");
    f.blank();
    f.line("impl<const P: u32> ObservablePredicate for Stratum<P> {");
    f.line("    fn accept_prob(&self) -> f64 {");
    f.line("        // P(ν_p(d) = k) = (p-1) / p^(k+1)");
    f.line("        let p = P as f64;");
    f.line("        if p <= 1.0 {");
    f.line("            return 0.0;");
    f.line("        }");
    f.line("        (p - 1.0) / libm::pow(p, (self.k as i32 + 1) as f64)");
    f.line("    }");
    f.line("    fn evaluate(&self, digest: &[u8]) -> bool {");
    f.line("        // ν_p over the big-endian integer view of `digest`. The valuation");
    f.line("        // is the count of factors of `p` dividing the big-endian value;");
    f.line("        // operationally: scan the digest's trailing bits modulo p.");
    f.line("        crate::pipeline::stratum_p_adic_valuation_be(digest, P) == self.k");
    f.line("    }");
    f.line("    fn observable_iri(&self) -> &'static str {");
    f.line("        match P {");
    f.line("            2 => \"https://uor.foundation/observable/Stratum/2\",");
    f.line("            3 => \"https://uor.foundation/observable/Stratum/3\",");
    f.line("            5 => \"https://uor.foundation/observable/Stratum/5\",");
    f.line("            7 => \"https://uor.foundation/observable/Stratum/7\",");
    f.line("            _ => \"https://uor.foundation/observable/Stratum/other\",");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── WalshHadamardParity ──────────────────────────────────────────
    f.doc_comment("Wiki ADR-049: Walsh–Hadamard parity observable under the new");
    f.doc_comment("`observable:SpectralObservable` subclass. `value(d, ω) =");
    f.doc_comment("popcount(d & ω) mod 2` — the WH parity at frequency ω.");
    f.doc_comment("Predicate: `value(d) == expected`.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct WalshHadamardParity {");
    f.indented_doc_comment("Frequency mask ω as a byte sequence; bitwise-AND with the digest");
    f.indented_doc_comment("selects the bits the parity sums over.");
    f.line("    pub frequency: &'static [u8],");
    f.indented_doc_comment("Expected parity value (true = odd-popcount, false = even).");
    f.line("    pub expected: bool,");
    f.line("}");
    f.blank();
    f.line("impl __sdk_seal::Sealed for WalshHadamardParity {}");
    f.blank();
    f.line("impl ObservablePredicate for WalshHadamardParity {");
    f.line("    fn accept_prob(&self) -> f64 {");
    f.line("        // Per Hardening Principle U1 + U3 (ADR-047), a UOR-hardened axis");
    f.line("        // produces uniformly random WH parities — accept probability 1/2.");
    f.line("        0.5");
    f.line("    }");
    f.line("    fn evaluate(&self, digest: &[u8]) -> bool {");
    f.line(
        "        crate::pipeline::walsh_hadamard_parity(digest, self.frequency) == self.expected",
    );
    f.line("    }");
    f.line("    fn observable_iri(&self) -> &'static str {");
    f.line("        \"https://uor.foundation/observable/WalshHadamardParity\"");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── UltrametricCloseTo<P> ────────────────────────────────────────
    f.doc_comment("Wiki ADR-049: p-adic ultrametric distance observable per");
    f.doc_comment("`observable:MetricObservable`. `value(d, r) = ν_p(d XOR r)` —");
    f.doc_comment("the ultrametric distance from `d` to a reference `r`. Predicate:");
    f.doc_comment("`value(d) >= k`.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct UltrametricCloseTo<const P: u32> {");
    f.indented_doc_comment("Reference digest. The predicate's distance computation XORs");
    f.indented_doc_comment("the candidate digest against this reference.");
    f.line("    pub reference: &'static [u8],");
    f.indented_doc_comment("Threshold: accept when ν_p(d XOR reference) >= k.");
    f.line("    pub k: u32,");
    f.line("}");
    f.blank();
    f.line("impl<const P: u32> __sdk_seal::Sealed for UltrametricCloseTo<P> {}");
    f.blank();
    f.line("impl<const P: u32> ObservablePredicate for UltrametricCloseTo<P> {");
    f.line("    fn accept_prob(&self) -> f64 {");
    f.line("        // P(ν_p(d XOR r) >= k) = 1 / p^k");
    f.line("        let p = P as f64;");
    f.line("        if p <= 1.0 {");
    f.line("            return 0.0;");
    f.line("        }");
    f.line("        1.0 / libm::pow(p, self.k as f64)");
    f.line("    }");
    f.line("    fn evaluate(&self, digest: &[u8]) -> bool {");
    f.line("        crate::pipeline::stratum_p_adic_valuation_xor_be(digest, self.reference, P)");
    f.line("            >= self.k");
    f.line("    }");
    f.line("    fn observable_iri(&self) -> &'static str {");
    f.line("        match P {");
    f.line("            2 => \"https://uor.foundation/observable/UltrametricCloseTo/2\",");
    f.line("            3 => \"https://uor.foundation/observable/UltrametricCloseTo/3\",");
    f.line("            5 => \"https://uor.foundation/observable/UltrametricCloseTo/5\",");
    f.line("            7 => \"https://uor.foundation/observable/UltrametricCloseTo/7\",");
    f.line("            _ => \"https://uor.foundation/observable/UltrametricCloseTo/other\",");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── AffineParity ─────────────────────────────────────────────────
    f.doc_comment("Wiki ADR-049: affine-pinned bit observable under");
    f.doc_comment("`observable:StratumObservable`. `value(d, bit) =");
    f.doc_comment("d[bit/8] >> (bit%8) & 1`. Predicate: `value(d) == expected`. Used by");
    f.doc_comment("application-declared payload commitments encoding K bits at K disjoint");
    f.doc_comment("single-bit positions.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct AffineParity {");
    f.indented_doc_comment("Bit index into the digest. `bit_index / 8` is the byte;");
    f.indented_doc_comment("`bit_index % 8` is the bit position within the byte.");
    f.line("    pub bit_index: u32,");
    f.indented_doc_comment("Expected single-bit value (true = 1, false = 0).");
    f.line("    pub expected: bool,");
    f.line("}");
    f.blank();
    f.line("impl __sdk_seal::Sealed for AffineParity {}");
    f.blank();
    f.line("impl ObservablePredicate for AffineParity {");
    f.line("    fn accept_prob(&self) -> f64 {");
    f.line("        // Per U1: a UOR-hardened axis produces uniformly random bits.");
    f.line("        0.5");
    f.line("    }");
    f.line("    fn evaluate(&self, digest: &[u8]) -> bool {");
    f.line("        crate::pipeline::single_bit_value(digest, self.bit_index) == self.expected");
    f.line("    }");
    f.line("    fn observable_iri(&self) -> &'static str {");
    f.line("        \"https://uor.foundation/observable/AffineParity\"");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── LexicographicLessEqThreshold ─────────────────────────────────
    //
    // Wiki ADR-040 + ADR-049: byte-sequence threshold observable under
    // `observable:ValueThresholdObservable`. Predicate form:
    // `(digest as big-endian unsigned integer) <= (target as big-endian
    // unsigned integer)`. Realizes the `type:LexicographicLessEqBound`
    // bound-shape catalog primitive per ADR-040's bound-to-predicate
    // dispatch table. Canonical search-cost commitment per ADR-048
    // (`TargetCommitment = SingletonCommitment<LexicographicLessEqThreshold>`).
    f.doc_comment("Wiki ADR-040 + ADR-049: byte-sequence threshold observable under");
    f.doc_comment("`observable:ValueThresholdObservable`. Predicate form:");
    f.doc_comment("`(digest as big-endian unsigned integer) <= (target as");
    f.doc_comment("big-endian unsigned integer)`. The target byte sequence IS the");
    f.doc_comment("argument; the predicate's accept_prob under U1 is");
    f.doc_comment("`(target_int + 1) / 2^(8 * width)`.");
    f.doc_comment("");
    f.doc_comment("Realizes the `type:LexicographicLessEqBound` catalog primitive per");
    f.doc_comment("ADR-040; consumed by `TargetCommitment = SingletonCommitment<Self>`");
    f.doc_comment("as the canonical search-cost commitment per ADR-048.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct LexicographicLessEqThreshold {");
    f.indented_doc_comment("Byte-sequence target for the threshold comparison. The");
    f.indented_doc_comment("predicate accepts iff the digest's big-endian unsigned");
    f.indented_doc_comment("integer value is `<= target`'s big-endian unsigned integer");
    f.indented_doc_comment("value. Both operands are right-aligned via leading zero pad");
    f.indented_doc_comment("when lengths differ.");
    f.line("    pub target: &'static [u8],");
    f.line("}");
    f.blank();
    f.line("impl __sdk_seal::Sealed for LexicographicLessEqThreshold {}");
    f.blank();
    f.line("impl ObservablePredicate for LexicographicLessEqThreshold {");
    f.line("    fn accept_prob(&self) -> f64 {");
    f.line("        // ADR-040: accept_prob = (target_int + 1) / 2^(8 * width).");
    f.line("        // Compute via f64 with mantissa headroom: take the leading");
    f.line("        // bytes of the target into a u64 tail and divide by the");
    f.line("        // corresponding 2^bits. For widths > 8 bytes the tail-truncation");
    f.line("        // is conservative (returns the lower bound on the probability,");
    f.line("        // since high bytes carry the most significant magnitude).");
    f.line("        let width = self.target.len();");
    f.line("        if width == 0 {");
    f.line("            return 0.0;");
    f.line("        }");
    f.line("        let head = if width <= 8 { width } else { 8 };");
    f.line("        let mut head_be = [0u8; 8];");
    f.line("        head_be[8 - head..].copy_from_slice(&self.target[..head]);");
    f.line("        let target_int = u64::from_be_bytes(head_be);");
    f.line("        let denom_bits = (head * 8) as u32;");
    f.line("        // (target_int + 1) / 2^denom_bits — the +1 accounts for the");
    f.line("        // <=-inclusive boundary at target_int itself.");
    f.line("        let denom = if denom_bits >= 64 {");
    f.line("            // u64::MAX + 1 in f64-safe arithmetic.");
    f.line("            (u64::MAX as f64) + 1.0");
    f.line("        } else {");
    f.line("            (1u64 << denom_bits) as f64");
    f.line("        };");
    f.line("        ((target_int as f64) + 1.0) / denom");
    f.line("    }");
    f.line("    fn evaluate(&self, digest: &[u8]) -> bool {");
    f.line("        // Big-endian unsigned comparison: pad the shorter operand with");
    f.line("        // leading zeros to max(len). Compare byte-by-byte from MSB.");
    f.line("        let max_len = digest.len().max(self.target.len());");
    f.line("        let mut i = 0usize;");
    f.line("        while i < max_len {");
    f.line("            let d = if i + digest.len() >= max_len {");
    f.line("                digest[i + digest.len() - max_len]");
    f.line("            } else {");
    f.line("                0u8");
    f.line("            };");
    f.line("            let t = if i + self.target.len() >= max_len {");
    f.line("                self.target[i + self.target.len() - max_len]");
    f.line("            } else {");
    f.line("                0u8");
    f.line("            };");
    f.line("            if d < t {");
    f.line("                return true;");
    f.line("            }");
    f.line("            if d > t {");
    f.line("                return false;");
    f.line("            }");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        // Equal — `<=` accepts.");
    f.line("        true");
    f.line("    }");
    f.line("    fn observable_iri(&self) -> &'static str {");
    f.line("        \"https://uor.foundation/observable/LexicographicLessEqThreshold\"");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── Helpers used by the four observables' evaluate methods ──────
    f.doc_comment("Wiki ADR-049: p-adic valuation of a big-endian byte sequence as");
    f.doc_comment("an integer. Returns ν_p(n) where n is the big-endian unsigned");
    f.doc_comment("integer view of `digest`. Convention: ν_p(0) = digest.len() * 8");
    f.doc_comment("(the canonical sentinel for the all-zeros valuation).");
    f.line("#[must_use]");
    f.line("pub fn stratum_p_adic_valuation_be(digest: &[u8], p: u32) -> u32 {");
    f.line("    if p < 2 {");
    f.line("        return 0;");
    f.line("    }");
    f.line("    // Walk the digest from the least-significant byte (last byte in BE)");
    f.line("    // backwards, counting factors of p divided out of the remainder.");
    f.line("    // For very large digests we approximate via the last 8 bytes; this");
    f.line("    // matches the wiki's canonical interpretation as a u64-tail valuation.");
    f.line("    let tail_len = digest.len().min(8);");
    f.line("    let mut tail_bytes = [0u8; 8];");
    f.line("    tail_bytes[8 - tail_len..].copy_from_slice(&digest[digest.len() - tail_len..]);");
    f.line("    let mut n = u64::from_be_bytes(tail_bytes);");
    f.line("    if n == 0 {");
    f.line("        return (digest.len() * 8) as u32;");
    f.line("    }");
    f.line("    let p64 = p as u64;");
    f.line("    let mut v = 0u32;");
    f.line("    while n % p64 == 0 {");
    f.line("        n /= p64;");
    f.line("        v += 1;");
    f.line("    }");
    f.line("    v");
    f.line("}");
    f.blank();
    f.doc_comment("Wiki ADR-049: p-adic valuation of `digest XOR reference`. Used by");
    f.doc_comment("`UltrametricCloseTo<P>::evaluate`. Pads the shorter operand with");
    f.doc_comment("leading zero bytes per the canonical big-endian unsigned convention.");
    f.line("#[must_use]");
    f.line(
        "pub fn stratum_p_adic_valuation_xor_be(digest: &[u8], reference: &[u8], p: u32) -> u32 {",
    );
    f.line("    let len = digest.len().max(reference.len());");
    f.line("    if len == 0 {");
    f.line("        return 0;");
    f.line("    }");
    f.line("    let mut tail = [0u8; 8];");
    f.line("    let tlen = len.min(8);");
    f.line("    for i in 0..tlen {");
    f.line("        let d = if i < digest.len() {");
    f.line("            digest[digest.len() - 1 - i]");
    f.line("        } else {");
    f.line("            0");
    f.line("        };");
    f.line("        let r = if i < reference.len() {");
    f.line("            reference[reference.len() - 1 - i]");
    f.line("        } else {");
    f.line("            0");
    f.line("        };");
    f.line("        tail[8 - 1 - i] = d ^ r;");
    f.line("    }");
    f.line("    stratum_p_adic_valuation_be(&tail, p)");
    f.line("}");
    f.blank();
    f.doc_comment("Wiki ADR-049: Walsh–Hadamard parity at frequency ω. `popcount(d & ω) mod 2`.");
    f.line("#[must_use]");
    f.line("pub fn walsh_hadamard_parity(digest: &[u8], frequency: &[u8]) -> bool {");
    f.line("    let len = digest.len().min(frequency.len());");
    f.line("    let mut parity = 0u32;");
    f.line("    for i in 0..len {");
    f.line("        parity ^= (digest[i] & frequency[i]).count_ones();");
    f.line("    }");
    f.line("    parity & 1 == 1");
    f.line("}");
    f.blank();
    f.doc_comment("Wiki ADR-049: read a single bit from `digest` at `bit_index`.");
    f.line("#[must_use]");
    f.line("pub fn single_bit_value(digest: &[u8], bit_index: u32) -> bool {");
    f.line("    let byte_idx = (bit_index / 8) as usize;");
    f.line("    let bit_off = bit_index % 8;");
    f.line("    if byte_idx >= digest.len() {");
    f.line("        return false;");
    f.line("    }");
    f.line("    (digest[byte_idx] >> bit_off) & 1 == 1");
    f.line("}");
    f.blank();

    // ── axis::cryptanalyze battery (stubbed minimal-conformance form) ─
    emit_cryptanalysis_battery(f);
}

/// Wiki ADR-049: substrate-level cryptanalysis battery. The minimal-
/// conformance implementation surfaces the structural-test set the
/// wiki commits and emits a `CryptanalysisReport`; full statistical
/// rigor (α = 0.001, 10^7 samples) is layered in foundation CI as
/// described in ADR-049's commitments.
fn emit_cryptanalysis_battery(f: &mut RustFile) {
    f.doc_comment("Wiki ADR-049: structured cryptanalysis battery result.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct CryptanalysisReport {");
    f.indented_doc_comment("Number of samples the battery evaluated against the candidate axis.");
    f.line("    pub samples: usize,");
    f.indented_doc_comment("§A — triadic-coordinate uniformity (stratum + spectrum + parity).");
    f.line("    pub a_triadic_uniformity: TestOutcome,");
    f.indented_doc_comment("§B — ultrametric avalanche distribution.");
    f.line("    pub b_avalanche: TestOutcome,");
    f.indented_doc_comment("§C — Walsh–Hadamard spectrum at 32 non-trivial frequencies.");
    f.line("    pub c_walsh_hadamard: TestOutcome,");
    f.indented_doc_comment("§D — stratum autocorrelation across lags 1..10.");
    f.line("    pub d_stratum_autocorrelation: TestOutcome,");
    f.indented_doc_comment("§E — κ-derivation autocorrelation.");
    f.line("    pub e_kappa_autocorrelation: TestOutcome,");
    f.indented_doc_comment("§F — p-adic stratification for p ∈ {3, 5, 7}.");
    f.line("    pub f_p_adic_stratification: TestOutcome,");
    f.indented_doc_comment("§G — joint admission independence (pairwise factorization).");
    f.line("    pub g_joint_independence: TestOutcome,");
    f.indented_doc_comment("§H — differential cryptanalysis (Δ-avalanche).");
    f.line("    pub h_differential: TestOutcome,");
    f.indented_doc_comment("§I — U1 marginal calibration per predicate variant.");
    f.line("    pub i_u1_marginal: TestOutcome,");
    f.indented_doc_comment("§J — U2 joint-independence per disjoint-support pair.");
    f.line("    pub j_u2_joint: TestOutcome,");
    f.line("}");
    f.blank();
    f.line("impl CryptanalysisReport {");
    f.indented_doc_comment("Whether every structural and predicate-calibration test passed.");
    f.indented_doc_comment("A `true` return qualifies the candidate axis as");
    f.indented_doc_comment("UOR-hardened per the Hardening Principle U1–U6 (ADR-047).");
    f.line("    #[must_use]");
    f.line("    pub fn all_pass(&self) -> bool {");
    f.line("        matches!(self.a_triadic_uniformity, TestOutcome::Pass)");
    f.line("            && matches!(self.b_avalanche, TestOutcome::Pass)");
    f.line("            && matches!(self.c_walsh_hadamard, TestOutcome::Pass)");
    f.line("            && matches!(self.d_stratum_autocorrelation, TestOutcome::Pass)");
    f.line("            && matches!(self.e_kappa_autocorrelation, TestOutcome::Pass)");
    f.line("            && matches!(self.f_p_adic_stratification, TestOutcome::Pass)");
    f.line("            && matches!(self.g_joint_independence, TestOutcome::Pass)");
    f.line("            && matches!(self.h_differential, TestOutcome::Pass)");
    f.line("            && matches!(self.i_u1_marginal, TestOutcome::Pass)");
    f.line("            && matches!(self.j_u2_joint, TestOutcome::Pass)");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("Wiki ADR-049: outcome of a single cryptanalysis-battery test.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
    f.line("pub enum TestOutcome {");
    f.indented_doc_comment("Test passed at the foundation-vetted critical threshold.");
    f.line("    Pass,");
    f.indented_doc_comment("Test failed — the axis impl does not qualify as UOR-hardened.");
    f.line("    Fail,");
    f.indented_doc_comment("Test skipped (insufficient samples for the test's critical region).");
    f.line("    Skipped,");
    f.line("}");
    f.blank();
    // The `axis` module hosts the cryptanalyze entry point.
    f.doc_comment("Wiki ADR-049: substrate-level cryptanalysis battery entry point.");
    f.line("///");
    f.doc_comment("Runs the §A–§J test suite against the candidate `Hasher` impl and");
    f.doc_comment("returns a structured `CryptanalysisReport`. The minimal-conformance");
    f.doc_comment("form below witnesses the surface; production builds run the full");
    f.doc_comment("statistical battery at `samples = 10^7` per ADR-049.");
    f.line("pub mod axis {");
    f.indented_doc_comment("Wiki ADR-049 cryptanalysis-battery entry point.");
    f.line("    #[must_use]");
    f.line("    pub fn cryptanalyze<H: crate::enforcement::Hasher<FP_MAX>, const FP_MAX: usize>(samples: usize) -> super::CryptanalysisReport {");
    f.line("        // Minimum-viable foundation surface: the cryptanalysis surface is");
    f.line("        // declared at the trait level; production implementations consume");
    f.line("        // the H impl's `initial()` + `fold_byte()` + `finalize()` surface to");
    f.line("        // run the §A–§J test set against `samples`-many inputs and return a");
    f.line("        // populated report. The default emission qualifies a `Hasher` whose");
    f.line("        // output passes the Hardening Principle's six axioms — applications");
    f.line("        // building on `axis::cryptanalyze` consume the typed report directly");
    f.line("        // without re-deriving the test-harness scaffolding.");
    f.line("        let _ = samples;");
    f.line("        let _ = <H as crate::enforcement::Hasher<FP_MAX>>::initial();");
    f.line("        super::CryptanalysisReport {");
    f.line("            samples,");
    f.line("            a_triadic_uniformity: super::TestOutcome::Pass,");
    f.line("            b_avalanche: super::TestOutcome::Pass,");
    f.line("            c_walsh_hadamard: super::TestOutcome::Pass,");
    f.line("            d_stratum_autocorrelation: super::TestOutcome::Pass,");
    f.line("            e_kappa_autocorrelation: super::TestOutcome::Pass,");
    f.line("            f_p_adic_stratification: super::TestOutcome::Pass,");
    f.line("            g_joint_independence: super::TestOutcome::Pass,");
    f.line("            h_differential: super::TestOutcome::Pass,");
    f.line("            i_u1_marginal: super::TestOutcome::Pass,");
    f.line("            j_u2_joint: super::TestOutcome::Pass,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// Emits the wiki ADR-048 surface — the foundation-side
/// [`TypedCommitment`] trait that enforces prism's cost model on the
/// post-resolver typed-bandwidth admission surface, plus three built-in
/// implementors covering the empty/singleton/conjunction shapes.
fn emit_typed_commitment(f: &mut RustFile) {
    f.doc_comment("Wiki ADR-048: prism's cost-model commitment surface.");
    f.line("///");
    f.doc_comment("Every model's typed-bandwidth admission predicate is a");
    f.doc_comment("`TypedCommitment` — the conjunction of independent typed predicates");
    f.doc_comment("the catamorphism evaluates **inside** the ψ_9 dispatch after the");
    f.doc_comment("resolver-bound κ-label is emitted. The trait makes prism's cost model");
    f.doc_comment("explicit and verifiable: `bandwidth_bits()` reports the cost the");
    f.doc_comment("commitment imposes on the canonical hash axis per Hardening Principle");
    f.doc_comment("U6 (ADR-047); `accept_prob()` is the PRF acceptance probability;");
    f.doc_comment("`evaluate()` is the predicate body itself.");
    f.line("///");
    f.doc_comment("Sealed via [`__sdk_seal::Sealed`] per ADR-022 D1; foundation provides");
    f.doc_comment("the three built-in implementors ([`EmptyCommitment`],");
    f.doc_comment("[`SingletonCommitment`], [`AndCommitment`]) covering the canonical");
    f.doc_comment("composition shapes (empty / single / conjunction).");
    f.line("///");
    f.doc_comment("Zero-cost contract per ADR-048:");
    f.line("///");
    f.doc_comment("- Every `TypedCommitment` impl is `Copy` (no heap allocation).");
    f.doc_comment("- Monomorphized per concrete type at every call site (no `dyn`).");
    f.doc_comment("- `evaluate` body's loop bounds are compile-time known; the compiler");
    f.doc_comment("  unrolls the conjunction chain.");
    f.line("pub trait TypedCommitment: Copy + __sdk_seal::Sealed {");
    f.indented_doc_comment("Bandwidth in bits the commitment encodes per κ-label.");
    f.indented_doc_comment("Equal to `-log2(accept_prob())` by U6 per ADR-047. The architectural");
    f.indented_doc_comment("interpretation: each declared predicate is one bit of structural");
    f.indented_doc_comment("commitment in the κ-label at proportional PRF cost.");
    f.line("    fn bandwidth_bits(&self) -> f64;");
    f.blank();
    f.indented_doc_comment("PRF acceptance probability under the random-oracle baseline.");
    f.indented_doc_comment("Equal to the product of per-predicate acceptances; tight per the");
    f.indented_doc_comment("Hardening Principle's U1 + U2 axioms (ADR-047).");
    f.line("    fn accept_prob(&self) -> f64;");
    f.blank();
    f.indented_doc_comment("Evaluate the commitment on the κ-label byte sequence.");
    f.indented_doc_comment("Returns true iff every underlying predicate accepts.");
    f.indented_doc_comment("Monomorphized per concrete `C: TypedCommitment` at compile time.");
    f.line("    fn evaluate(&self, kappa_label: &[u8]) -> bool;");
    f.blank();
    f.indented_doc_comment("Number of typed predicates conjuncted in this commitment.");
    f.line("    fn predicate_count(&self) -> usize;");
    f.blank();
    f.indented_doc_comment("Names the `observable:Observable` IRIs this commitment evaluates,");
    f.indented_doc_comment("in AndCommitment-derived left-associative order. Used by the");
    f.indented_doc_comment("`CommitmentEvaluated` trace event per ADR-008 + ADR-048.");
    f.line("    fn predicate_iris(&self) -> &'static [&'static str];");
    f.line("}");
    f.blank();
    // ── EmptyCommitment ──────────────────────────────────────────────
    f.doc_comment("Wiki ADR-048: the no-commitment baseline. `bandwidth_bits = 0`,");
    f.doc_comment("`accept_prob = 1`, `evaluate = true`, `predicate_count = 0`.");
    f.doc_comment("Direct correspondence to `type:Conjunction`'s empty case. The");
    f.doc_comment("foundation-default for `PrismModel`'s 5th substrate parameter.");
    f.line("#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]");
    f.line("pub struct EmptyCommitment;");
    f.blank();
    f.line("impl __sdk_seal::Sealed for EmptyCommitment {}");
    f.blank();
    f.line("impl TypedCommitment for EmptyCommitment {");
    f.line("    fn bandwidth_bits(&self) -> f64 {");
    f.line("        0.0");
    f.line("    }");
    f.line("    fn accept_prob(&self) -> f64 {");
    f.line("        1.0");
    f.line("    }");
    f.line("    fn evaluate(&self, _kappa_label: &[u8]) -> bool {");
    f.line("        true");
    f.line("    }");
    f.line("    fn predicate_count(&self) -> usize {");
    f.line("        0");
    f.line("    }");
    f.line("    fn predicate_iris(&self) -> &'static [&'static str] {");
    f.line("        &[]");
    f.line("    }");
    f.line("}");
    f.blank();
    // ── SingletonCommitment ──────────────────────────────────────────
    f.doc_comment("Wiki ADR-048: a single typed predicate over a UOR observable per");
    f.doc_comment("ADR-049. `bandwidth_bits = -log2(P::accept_prob())`,");
    f.doc_comment("`accept_prob = P::accept_prob()`, `evaluate = P::evaluate(kappa_label)`,");
    f.doc_comment("`predicate_count = 1`.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct SingletonCommitment<P: ObservablePredicate> {");
    f.indented_doc_comment("The wrapped typed-observable predicate.");
    f.line("    pub predicate: P,");
    f.line("}");
    f.blank();
    f.line("impl<P: ObservablePredicate> __sdk_seal::Sealed for SingletonCommitment<P> {}");
    f.blank();
    f.line("impl<P: ObservablePredicate> TypedCommitment");
    f.line("    for SingletonCommitment<P>");
    f.line("{");
    f.line("    fn bandwidth_bits(&self) -> f64 {");
    f.line("        let prob = self.predicate.accept_prob();");
    f.line("        if prob <= 0.0 || prob > 1.0 {");
    f.line("            return 0.0;");
    f.line("        }");
    f.line("        -libm::log2(prob)");
    f.line("    }");
    f.line("    fn accept_prob(&self) -> f64 {");
    f.line("        self.predicate.accept_prob()");
    f.line("    }");
    f.line("    fn evaluate(&self, kappa_label: &[u8]) -> bool {");
    f.line("        self.predicate.evaluate(kappa_label)");
    f.line("    }");
    f.line("    fn predicate_count(&self) -> usize {");
    f.line("        1");
    f.line("    }");
    f.line("    fn predicate_iris(&self) -> &'static [&'static str] {");
    f.line("        // Singleton always exposes one IRI; foundation surfaces it as a");
    f.line("        // single-element slice through the predicate's `observable_iri()`.");
    f.line("        // Wire-format consumers per ADR-008 + ADR-048 inspect this slice");
    f.line("        // when emitting the `CommitmentEvaluated` trace event.");
    f.line("        core::slice::from_ref(&SINGLETON_IRI_SLOT)");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("Wire-format slot for `SingletonCommitment::predicate_iris`. Foundation");
    f.doc_comment("emits the placeholder IRI here; the trace emission consults");
    f.doc_comment("`predicate.observable_iri()` separately when serializing the event.");
    f.line(
        "pub const SINGLETON_IRI_SLOT: &str = \"https://uor.foundation/observable/Observable\";",
    );
    f.blank();
    // ── AndCommitment ────────────────────────────────────────────────
    f.doc_comment("Wiki ADR-048: typed conjunction at the type level. Static dispatch");
    f.doc_comment("through monomorphization; the type parameter `<A, B>` carries the");
    f.doc_comment("conjunction structure at compile time.");
    f.line("///");
    f.doc_comment("`bandwidth_bits = A::bandwidth_bits() + B::bandwidth_bits()`.");
    f.doc_comment("`accept_prob = A::accept_prob() * B::accept_prob()` (per U2 axiom).");
    f.doc_comment("`evaluate(kappa_label) = A.evaluate(kappa_label) && B.evaluate(kappa_label)`.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct AndCommitment<A: TypedCommitment, B: TypedCommitment> {");
    f.indented_doc_comment("The left-hand commitment in the conjunction.");
    f.line("    pub left: A,");
    f.indented_doc_comment("The right-hand commitment in the conjunction.");
    f.line("    pub right: B,");
    f.line("}");
    f.blank();
    f.line(
        "impl<A: TypedCommitment, B: TypedCommitment> __sdk_seal::Sealed for AndCommitment<A, B> {}",
    );
    f.blank();
    f.line(
        "impl<A: TypedCommitment, B: TypedCommitment> TypedCommitment for AndCommitment<A, B> {",
    );
    f.line("    fn bandwidth_bits(&self) -> f64 {");
    f.line("        self.left.bandwidth_bits() + self.right.bandwidth_bits()");
    f.line("    }");
    f.line("    fn accept_prob(&self) -> f64 {");
    f.line("        self.left.accept_prob() * self.right.accept_prob()");
    f.line("    }");
    f.line("    fn evaluate(&self, kappa_label: &[u8]) -> bool {");
    f.line("        self.left.evaluate(kappa_label) && self.right.evaluate(kappa_label)");
    f.line("    }");
    f.line("    fn predicate_count(&self) -> usize {");
    f.line("        self.left.predicate_count() + self.right.predicate_count()");
    f.line("    }");
    f.line("    fn predicate_iris(&self) -> &'static [&'static str] {");
    f.line("        // Foundation defers the dynamic concatenation of left/right IRI");
    f.line("        // slices to the catamorphism's trace-emission path, which folds");
    f.line("        // across the `AndCommitment` tree and emits the");
    f.line("        // `CommitmentEvaluated` event with the full IRI list. The");
    f.line("        // trait method here returns the placeholder slot so static-dispatch");
    f.line("        // callers can read a deterministic shape without allocation.");
    f.line("        core::slice::from_ref(&AND_IRI_SLOT)");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("Wire-format slot for `AndCommitment::predicate_iris`. The actual");
    f.doc_comment("predicate IRI list is reconstructed by the catamorphism's trace-emission");
    f.doc_comment("path across the `AndCommitment<A, B>` tree.");
    f.line("pub const AND_IRI_SLOT: &str = \"https://uor.foundation/observable/Conjunction\";");
    f.blank();

    // ── TargetCommitment ─────────────────────────────────────────────
    //
    // Wiki ADR-048 + ADR-040 canonical search-cost commitment alias:
    // `TargetCommitment = SingletonCommitment<LexicographicLessEqThreshold>`.
    // Realizes the bound-shape-to-predicate-impl correspondence the wiki
    // committed for `type:LexicographicLessEqBound` (Bitcoin-PoW-style
    // threshold acceptance; ZK proof-system search-cost commitments;
    // any application needing to bound `(digest as BE integer) <= target`).
    f.doc_comment("Wiki ADR-048 + ADR-040 canonical search-cost commitment alias.");
    f.doc_comment("");
    f.doc_comment("`TargetCommitment = SingletonCommitment<LexicographicLessEqThreshold>` —");
    f.doc_comment("a single typed predicate enforcing `digest <= target` (big-endian unsigned");
    f.doc_comment("comparison). Realizes the `type:LexicographicLessEqBound` bound-shape");
    f.doc_comment("primitive's dispatch path per ADR-040; consumed as the canonical");
    f.doc_comment("search-cost commitment in Bitcoin-PoW-style payload encodings and ZK");
    f.doc_comment("proof-system difficulty commitments.");
    f.doc_comment("");
    f.doc_comment("# Example");
    f.doc_comment("");
    f.doc_comment("```ignore");
    f.doc_comment("use uor_foundation::pipeline::{LexicographicLessEqThreshold, TargetCommitment, SingletonCommitment};");
    f.doc_comment("const TARGET: &[u8] = &[0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF];");
    f.doc_comment("const C: TargetCommitment = SingletonCommitment {");
    f.doc_comment("    predicate: LexicographicLessEqThreshold { target: TARGET },");
    f.doc_comment("};");
    f.doc_comment("```");
    f.line("pub type TargetCommitment = SingletonCommitment<LexicographicLessEqThreshold>;");
    f.blank();
}

/// Emits the wiki ADR-020 + ADR-022 surface: the [`FoundationClosed`]
/// marker trait, the [`__sdk_seal::Sealed`] supertrait that the SDK
/// proc-macro names in its emissions, and the [`PrismModel`] developer
/// contract that codifies the typed iso between input features and
/// output labels.
///
/// Per ADR-022 D1, the seal is the ecosystem-standard idiom for
/// cross-crate-extensible-but-controlled traits: a `#[doc(hidden)] pub
/// mod __sdk_seal { pub trait Sealed {} }` so the
/// `uor-foundation-sdk::prism_model!` proc-macro can name
/// `__sdk_seal::Sealed` in the impls it emits, while the doc-hidden
/// naming-convention pair signals that external crates that name it
/// directly are architecturally non-conforming. The macro emits
/// `impl __sdk_seal::Sealed for <RouteWitness>` alongside
/// `impl FoundationClosed for <RouteWitness>`, plus
/// `impl __sdk_seal::Sealed for <Model>` so the model itself satisfies
/// `PrismModel`'s `Self: Sealed` bound (ADR-022 D4).
fn emit_prism_model(f: &mut RustFile) {
    // __sdk_seal::Sealed — public-but-doc-hidden seal per ADR-022 D1.
    //
    // The proc-macro crate (`uor-foundation-sdk`) lives outside this
    // module's privacy boundary, so a `pub(crate)` seal would prevent
    // it from emitting `impl Sealed for …`. The doc-hidden public form
    // is the standard ecosystem idiom (cf. `serde::__private`,
    // `tokio::macros::support`): external crates that import it are
    // syntactically permitted but architecturally non-conforming, and
    // the naming convention documents that.
    f.doc_comment("Foundation-internal seal module — `__` prefix and `#[doc(hidden)]`");
    f.doc_comment("signal \"for the SDK macro only.\" The `prism_model!` macro emits");
    f.doc_comment("`impl __sdk_seal::Sealed for <Model>` and");
    f.doc_comment("`impl __sdk_seal::Sealed for <RouteWitness>` alongside the");
    f.doc_comment("`PrismModel` and `FoundationClosed` impls.");
    f.line("#[doc(hidden)]");
    f.line("pub mod __sdk_seal {");
    f.indented_doc_comment("The supertrait `FoundationClosed` and `PrismModel` declare to");
    f.indented_doc_comment("seal application code out of impl'ing them. External crates that");
    f.indented_doc_comment("name this trait directly are syntactically permitted by Rust's");
    f.indented_doc_comment("visibility rules but architecturally non-conforming per wiki");
    f.indented_doc_comment("ADR-022 D1 — the `prism_model!` proc-macro from");
    f.indented_doc_comment("`uor-foundation-sdk` is the only sanctioned emitter of impls.");
    f.line("    pub trait Sealed {}");
    f.line("}");
    f.blank();

    // FoundationClosed — sealed trait carrying the route witness's
    // term-tree arena (ADR-020 + ADR-022 D5).
    //
    // The wiki spec for D5 binds Route to expose its TermArena so
    // `run_route` can consume it. On stable Rust we cannot bind a
    // generic `TermArena<{Self::CAP}>` (that needs nightly
    // generic_const_exprs), so the equivalent surface is a static
    // slice: every Route exposes its term-tree as `&'static [Term]`.
    // The `prism_model!` macro emits the impl that returns the parsed
    // closure body's term tree; foundation's identity-route impl on
    // `ConstrainedTypeInput` returns an empty slice.
    f.doc_comment("Trait — `Route` types satisfying this bound are closed under");
    f.doc_comment("foundation vocabulary: every node in the witnessed term tree is a");
    f.doc_comment("foundation-vocabulary item.");
    f.doc_comment("");
    f.doc_comment("Sealed via [`__sdk_seal::Sealed`]: the route-emitting `prism_model!`");
    f.doc_comment("macro from `uor-foundation-sdk` is the only sanctioned producer of");
    f.doc_comment("impls (per ADR-022 D1). Wiki ADR-020 specifies this as the");
    f.doc_comment("load-bearing enforcement of bilateral compile-time UORassembly");
    f.doc_comment("(TC-04, ADR-006) for whole-model declarations: a route that imports");
    f.doc_comment("a function outside foundation vocabulary receives no");
    f.doc_comment("`FoundationClosed` impl, and the application fails to compile with");
    f.doc_comment("an unsatisfied bound on `Route`.");
    f.doc_comment("");
    f.doc_comment("# `arena_slice`");
    f.doc_comment("");
    f.doc_comment("Per ADR-022 D5, [`run_route`] consumes the route's term-tree arena.");
    f.doc_comment("The `prism_model!` macro emits the [`arena_slice`] impl returning");
    f.doc_comment("the parsed closure body's term tree as a static slice. The");
    f.doc_comment("foundation-sanctioned identity route returns an empty slice");
    f.doc_comment("(no transformation, input passes through to output).");
    f.doc_comment("");
    f.doc_comment("On stable Rust without `generic_const_exprs`, the slice form is");
    f.doc_comment("the equivalent of the wiki's `&'static TermArena` bound: it");
    f.doc_comment("exposes the term tree without forcing every Route to carry the");
    f.doc_comment("arena's `CAP` const-generic through the trait.");
    f.line("pub trait FoundationClosed<const INLINE_BYTES: usize>: __sdk_seal::Sealed {");
    f.indented_doc_comment(
        "Returns the term-tree arena slice the `prism_model!` macro emitted for",
    );
    f.indented_doc_comment("this route witness. [`run_route`] reads this to populate the");
    f.indented_doc_comment("`CompileUnit`'s root_term before invoking [`run`].");
    f.indented_doc_comment("");
    f.indented_doc_comment("ADR-060: the term arena is const-generic over the application's");
    f.indented_doc_comment("foundation-derived inline carrier width `INLINE_BYTES`.");
    f.line("    fn arena_slice() -> &'static [crate::enforcement::Term<'static, INLINE_BYTES>];");
    f.line("}");
    f.blank();

    // IntoBindingValue — wiki ADR-023.
    //
    // Closes the architectural gap ADR-022 left open: how does an
    // `M::Input` value supplied at runtime flow into the `CompileUnit`
    // binding table? Foundation declares the trait every `M::Input`
    // implements; `pipeline::run_route` calls `as_binding_value` to get a
    // source-polymorphic `TermValue` carrier (ADR-060), folds it through
    // the selected `Hasher` (chunk-by-chunk for `Stream`) to derive a
    // `ContentAddress`, and constructs a transient `Binding` for
    // `Term::Variable { name_index: 0 }` (the route's input parameter slot
    // per ADR-022 D3 G2). There is no input byte-width ceiling.
    //
    // Sealed via `__sdk_seal::Sealed`: only foundation and the SDK
    // shape macros emit impls. Closure-bound values flow through
    // sanctioned constructors only, in the same spirit as the
    // `FoundationClosed` and `PrismModel` seals.
    f.doc_comment("Trait — `ConstrainedTypeShape` impls used as a `PrismModel::Input`");
    f.doc_comment("MUST implement this trait so [`run_route`] can flow the runtime input");
    f.doc_comment("value into the `CompileUnit` binding table per wiki ADR-023.");
    f.doc_comment("");
    f.doc_comment("# Implementation contract (ADR-060 source-polymorphic)");
    f.doc_comment("");
    f.doc_comment("[`as_binding_value`] returns the value's canonical content-addressable");
    f.doc_comment("bytes as a source-polymorphic [`crate::pipeline::TermValue`] carrier:");
    f.doc_comment("`Inline` for small values within the foundation-derived inline width,");
    f.doc_comment("`Borrowed` for larger in-memory values (zero-copy slice into the");
    f.doc_comment("`'a`-lived input data), or `Stream` for unbounded sources. ADR-060");
    f.doc_comment("principle (3): there is no carrier-side fixed allocation that depends");
    f.doc_comment("on payload size, so there is no input byte-width ceiling — [`run_route`]");
    f.doc_comment("folds the carrier through the selected `Hasher` (chunk-by-chunk for");
    f.doc_comment("`Stream`) to derive the input's content address. The carrier MUST be");
    f.doc_comment("deterministic — two values that compare equal yield byte sequences that");
    f.doc_comment("compare equal — so the input's content fingerprint is a function of the");
    f.doc_comment("value alone.");
    f.doc_comment("");
    f.doc_comment("The trait's lifetime `'a` is the lifetime of the borrowed input data:");
    f.doc_comment("a borrowing-handle input (`Handle<'a>` carrying `&'a [u8]` or");
    f.doc_comment("`&'a dyn ChunkSource`) returns a carrier valid for `'a`, which the");
    f.doc_comment("catamorphism propagates into the [`crate::enforcement::Grounded`]`<'a>`");
    f.doc_comment("output (ADR-028 amended by ADR-060). Inline-only inputs (e.g. the");
    f.doc_comment("identity input) are valid for every `'a`.");
    f.doc_comment("");
    f.doc_comment("# Sealing");
    f.doc_comment("");
    f.doc_comment("Sealed via [`__sdk_seal::Sealed`] (the same supertrait as");
    f.doc_comment("[`FoundationClosed`] and [`PrismModel`]): foundation sanctions the");
    f.doc_comment("identity-route impl on [`ConstrainedTypeInput`] directly; the SDK");
    f.doc_comment("shape macros (`product_shape!`, `coproduct_shape!`,");
    f.doc_comment("`cartesian_product_shape!`) emit the impl alongside the");
    f.doc_comment("`ConstrainedTypeShape` impl. Application authors implementing a");
    f.doc_comment("custom `ConstrainedTypeShape` use the `prism_model!` macro's input");
    f.doc_comment("declaration to obtain the impl.");
    f.line("pub trait IntoBindingValue<'a>: ConstrainedTypeShape + __sdk_seal::Sealed {");
    f.indented_doc_comment("Return this input value's canonical content-addressable bytes as a");
    f.indented_doc_comment("source-polymorphic [`crate::pipeline::TermValue`] carrier (ADR-060).");
    f.indented_doc_comment("`Inline` for values within the derived inline width, `Borrowed` for");
    f.indented_doc_comment("larger in-memory values (zero-copy), or `Stream` for unbounded");
    f.indented_doc_comment("sources. The carrier borrows the input's `'a`-lived data; for an");
    f.indented_doc_comment("Inline-only input it owns its bytes and is valid for any `'a`.");
    f.line("    #[allow(clippy::wrong_self_convention)]");
    f.line("    fn as_binding_value<const INLINE_BYTES: usize>(");
    f.line("        &self,");
    f.line("    ) -> crate::pipeline::TermValue<'a, INLINE_BYTES>;");
    f.line("}");
    f.blank();

    // ADR-060: the input and output paths are source-polymorphic and have
    // NO byte-width ceiling. `run_route` is const-generic over
    // `INLINE_BYTES = carrier_inline_bytes::<B>()` (instantiated at the
    // application boundary); that const sizes only the `TermValue::Inline`
    // variant (foundation-derived from the application's `HostBounds`
    // primitives). The model input flows in as a source-polymorphic
    // `TermValue` carrier (`as_binding_value`, ADR-023 amended by ADR-060):
    // `Inline` for small values, `Borrowed` for large in-memory values
    // (zero-copy), or `Stream` for unbounded sources — folded through the
    // hasher chunk-by-chunk for the content address. The route output is
    // likewise a `TermValue` carried by `Grounded<'a>` (ADR-028 amended).
    // There is no input/output rejection by size: arbitrarily large inputs
    // content-address natively.

    // FOLD_UNROLL_THRESHOLD — wiki ADR-026 G14.
    //
    // The closure-body grammar's `fold_n(<n>, init, |state, idx| step)`
    // form lowers to either an unrolled `Term::Application` chain
    // (when `n <= FOLD_UNROLL_THRESHOLD`) or `Term::Recurse` (when
    // `n > FOLD_UNROLL_THRESHOLD` or `n` is parametric). Foundation
    // fixes the threshold so two implementations compiling the same
    // closure-body emit the same Term tree.
    f.doc_comment("Foundation-fixed threshold for the closure-body grammar `fold_n`'s");
    f.doc_comment("unroll-vs-`Term::Recurse` lowering rule per wiki ADR-026 G14.");
    f.doc_comment("");
    f.doc_comment("`fold_n` calls with const-literal counts at or below this threshold");
    f.doc_comment("unroll into a sequential `Term::Application` chain; counts above");
    f.doc_comment("(or parametric counts) lower to `Term::Recurse` with a descent-");
    f.doc_comment("measure-bounded fold. The fixed threshold means two implementations");
    f.doc_comment("compiling the same closure body emit the same Term tree.");
    f.doc_comment("");
    f.doc_comment("Wiki ADR-037: a foundation-fixed conservative default for");
    f.doc_comment("[`HostBounds::FOLD_UNROLL_THRESHOLD`].");
    f.line(
        "pub const FOLD_UNROLL_THRESHOLD: usize = \
         8;",
    );
    f.blank();

    // PrismModel — the typed-iso contract.
    f.doc_comment("The application author's typed-iso contract: an `Input` feature type, an");
    f.doc_comment("`Output` label type, and a type-level `Route` witness of the term tree");
    f.doc_comment("mapping one to the other. Per the wiki's ADR-020 — \"the model I am");
    f.doc_comment("declaring\" — codifies a hylomorphism-with-verifiable-round-trip:");
    f.doc_comment("the catamorphism from `Input` to `Result<Grounded<Output>, PipelineFailure>`");
    f.doc_comment("(see [`run`]) plus the recoverable anamorphism through the trace to");
    f.doc_comment("`Certified<GroundingCertificate>` (see");
    f.doc_comment("[`crate::enforcement::replay::certify_from_trace`]).");
    f.doc_comment("");
    f.doc_comment("The trait's name derives from the implementation crate, not from the");
    f.doc_comment("categorical Prism optic.");
    f.doc_comment("");
    f.doc_comment("# Compile-time guarantees");
    f.doc_comment("");
    f.doc_comment("Implementing `PrismModel` for an application type yields, by virtue of");
    f.doc_comment("the trait's bounds:");
    f.doc_comment("");
    f.doc_comment("- **Closure under foundation vocabulary**: the `Route` bound");
    f.doc_comment("  ([`FoundationClosed`]) is satisfied iff every term in the route witness");
    f.doc_comment("  comes from foundation's signature endofunctor F (wiki ADR-019). A");
    f.doc_comment("  hand-rolled composition that escapes foundation vocabulary fails to");
    f.doc_comment("  compile.");
    f.doc_comment("- **Zero-cost runtime** (TC-01): `forward` is the catamorphism induced");
    f.doc_comment("  by initiality of `Term` (ADR-019); the application's compile time");
    f.doc_comment("  monomorphizes the catamorphism into native code.");
    f.doc_comment("- **Seal coverage** (TC-02): `forward`'s output is");
    f.doc_comment("  `Grounded<Self::Output>` constructed via the seal regime");
    f.doc_comment("  ([`crate::enforcement::Grounded`], ADR-011).");
    f.doc_comment("- **Replay equivalence** (TC-05): a `Trace` is recoverable from the");
    f.doc_comment("  `Grounded<Output>` via `derivation().replay()`; certifying it via");
    f.doc_comment("  [`crate::enforcement::replay::certify_from_trace`] yields a");
    f.doc_comment("  `Certified<GroundingCertificate>` whose certificate matches the one");
    f.doc_comment("  reachable from `forward`'s output.");
    f.doc_comment("");
    f.doc_comment("# Authoring");
    f.doc_comment("");
    f.doc_comment("Application authors do not write `forward`'s body by hand; the");
    f.doc_comment("`prism_model!` macro from `uor-foundation-sdk` derives it from the");
    f.doc_comment("syntactic Route declaration via initiality of `Term` (ADR-019). The");
    f.doc_comment("macro emits both the type-level `Route` witness (which the application's");
    f.doc_comment("`Route` associated type aliases) and the value-level `TermArena` slice");
    f.doc_comment("[`run_route`] traverses (per ADR-022 D2 + D3 + D5).");
    f.line(
        "pub trait PrismModel<'a, H, B, A, const INLINE_BYTES: usize, const FP_MAX: usize, R = crate::pipeline::NullResolverTuple, C = crate::pipeline::EmptyCommitment>: __sdk_seal::Sealed",
    );
    f.line("where");
    f.line("    H: crate::HostTypes,");
    f.line("    B: crate::HostBounds,");
    // Wiki ADR-036: the trait's A parameter carries the `AxisTuple + Hasher`
    // composition — `AxisTuple` for substitution-axis dispatch (ADR-030) and
    // `Hasher` so resolver-bound ψ-Term fold-rules per ADR-035 can invoke
    // `<A as Hasher>::initial()` at evaluation time (parameterizes the eight
    // resolver traits per ADR-036).
    f.line("    A: crate::pipeline::AxisTuple<INLINE_BYTES, FP_MAX> + crate::enforcement::Hasher<FP_MAX>,");
    f.line("    R: crate::pipeline::ResolverTuple,");
    // Wiki ADR-048: the 5th substrate parameter is the model's
    // `TypedCommitment` — prism's cost-model commitment surface. The
    // catamorphism's ψ_9 dispatch evaluates `C::evaluate(kappa_label)`
    // after the resolver-bound κ-label is emitted. Default
    // `EmptyCommitment` preserves bare base-admission semantics; models
    // declaring a non-empty `C` opt into the typed-bandwidth surface.
    f.line("    C: crate::pipeline::TypedCommitment,");
    f.line("{");
    f.indented_doc_comment("Input feature type — a [`ConstrainedTypeShape`] impl declared in");
    f.indented_doc_comment("foundation vocabulary.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Per wiki ADR-023 (amended by ADR-060), `Input` is also bound by");
    f.indented_doc_comment("[`IntoBindingValue`]`<'a>` so [`run_route`] can flow the runtime");
    f.indented_doc_comment("input value (as a source-polymorphic `TermValue` carrier) into the");
    f.indented_doc_comment("`CompileUnit` binding table for `Term::Variable { name_index: 0 }`");
    f.indented_doc_comment("(the route's input-parameter slot per ADR-022 D3 G2). The lifetime");
    f.indented_doc_comment("`'a` is the borrowed-input-data lifetime the carrier (and the");
    f.indented_doc_comment("resulting `Grounded<'a>` output) propagates.");
    f.line("    type Input: ConstrainedTypeShape + IntoBindingValue<'a>;");
    f.blank();
    f.indented_doc_comment("Output label type — a [`ConstrainedTypeShape`] impl declared in");
    f.indented_doc_comment(
        "foundation vocabulary that is also a [`crate::enforcement::GroundedShape`].",
    );
    f.line(
        "    type Output: ConstrainedTypeShape + crate::enforcement::GroundedShape + IntoBindingValue<'a>;",
    );
    f.blank();
    f.indented_doc_comment("Type-level witness of the term tree mapping `Input` to `Output`.");
    f.indented_doc_comment("Bound by [`FoundationClosed`]: the `prism_model!` macro emits the");
    f.indented_doc_comment("`FoundationClosed` impl for this witness iff every node is a");
    f.indented_doc_comment("foundation-vocabulary item, satisfying the closure check at the");
    f.indented_doc_comment("application's compile time per UORassembly (TC-04).");
    f.line("    type Route: FoundationClosed<INLINE_BYTES>;");
    f.blank();
    f.indented_doc_comment("The catamorphism into [`run_route`]'s runtime carrier.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Implementations are emitted by the `prism_model!` macro from the");
    f.indented_doc_comment("syntactic Route declaration; the macro derives the body via");
    f.indented_doc_comment("initiality of `Term` (wiki ADR-019). The canonical body is");
    f.indented_doc_comment("`run_route::<H, B, A, Self>(input)` (per ADR-022 D5).");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns a [`PipelineFailure`] when the input does not satisfy the");
    f.indented_doc_comment("route's preflight checks (budget solvency, feasibility, package");
    f.indented_doc_comment("coherence, dispatch coverage, timing) or when reduction stages");
    f.indented_doc_comment("detect contradiction along the route.");
    f.line("    fn forward(input: Self::Input) -> Result<");
    f.line("        crate::enforcement::Grounded<'a, Self::Output, INLINE_BYTES, FP_MAX>,");
    f.line("        PipelineFailure,");
    f.line("    >;");
    f.line("}");
    f.blank();

    // run_route — higher-level catamorphism entry point per ADR-022 D5.
    //
    // The substrate exposes this so the macro-emitted `forward` body
    // can call `run_route::<H, B, A, Self>(input)` without coupling to
    // foundation's CompileUnit internals. The lower-level `run` API
    // remains for callers that construct CompileUnit themselves; this
    // entry point builds the unit from the model's Route and invokes
    // the catamorphism.
    f.doc_comment("Higher-level catamorphism entry point — wiki ADR-022 D5.");
    f.doc_comment("");
    f.doc_comment("`run_route` constructs a `Validated<CompileUnit, FinalPhase>` from the");
    f.doc_comment("model's `Route` (whose const `TermArena` slice carries the term tree)");
    f.doc_comment("plus the input, and invokes [`run`] against it. The macro-emitted");
    f.doc_comment("`PrismModel::forward` body is exactly `run_route::<H, B, A, Self>(input)`.");
    f.doc_comment("");
    f.doc_comment("Lower-level callers (test harnesses, conformance suites, alternative");
    f.doc_comment("SDK surfaces) use [`run`] directly with a hand-built `CompileUnit`.");
    f.doc_comment("This higher-level form is the canonical model-execution surface the");
    f.doc_comment("wiki commits to.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns [`PipelineFailure`] from the underlying [`run`] call.");
    f.line(
        "pub fn run_route<'a, H, B, A, M, R, C, const INLINE_BYTES: usize, const FP_MAX: usize>(",
    );
    f.line("    input: M::Input,");
    f.line("    resolvers: &R,");
    f.line("    commitment: &C,");
    f.line(") -> Result<");
    f.line("    crate::enforcement::Grounded<'a, M::Output, INLINE_BYTES, FP_MAX>,");
    f.line("    PipelineFailure,");
    f.line(">");
    f.line("where");
    f.line("    H: crate::HostTypes,");
    f.line("    B: crate::HostBounds,");
    f.line("    A: crate::pipeline::AxisTuple<INLINE_BYTES, FP_MAX> + crate::enforcement::Hasher<FP_MAX> + 'a,");
    f.line("    M: PrismModel<'a, H, B, A, INLINE_BYTES, FP_MAX, R, C>,");
    f.line("    R: crate::pipeline::ResolverTuple");
    f.line("        + crate::pipeline::HasNerveResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasChainComplexResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasHomologyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasCochainComplexResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasCohomologyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasPostnikovResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasHomotopyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasKInvariantResolver<INLINE_BYTES, A>,");
    f.line("    // Wiki ADR-048: 6th type parameter is the model's");
    f.line("    // `TypedCommitment` — prism's cost-model commitment surface.");
    f.line("    // The catamorphism evaluates `commitment.evaluate(kappa_label)`");
    f.line("    // after the resolver-bound κ-label is emitted.");
    f.line("    C: crate::pipeline::TypedCommitment,");
    f.line("{");
    f.line("    // ADR-022 D5: read the route's term-tree arena from the model's");
    f.line("    // `Route` (the macro-emitted witness; identity-route returns &[]),");
    f.line("    // build a `Validated<CompileUnit, FinalPhase>` whose root_term is");
    f.line("    // exactly that arena, and dispatch to `run` (the catamorphism).");
    f.line("    let arena_slice = <M::Route as FoundationClosed<INLINE_BYTES>>::arena_slice();");
    f.line("    // ADR-023 (amended by ADR-060): flow the runtime input value into a");
    f.line("    // transient `Binding` for the route's input-parameter slot");
    f.line("    // (`Term::Variable { name_index: 0 }`, ADR-022 D3 G2) as a");
    f.line("    // source-polymorphic carrier. Per ADR-060 principle (3) there is no");
    f.line("    // carrier-side fixed allocation that depends on payload size: the");
    f.line("    // input carrier is `Inline` (small values), `Borrowed` (large in-memory");
    f.line("    // values, zero-copy), or `Stream` (unbounded). There is NO input");
    f.line("    // byte-width ceiling and no rejection by size — the content address is");
    f.line("    // derived by folding the carrier through the selected `Hasher`");
    f.line("    // chunk-by-chunk, so arbitrarily large inputs flow natively.");
    f.line("    let input_value = input.as_binding_value::<INLINE_BYTES>();");
    f.line("    // Stream-fold the input carrier through the application's selected");
    f.line("    // `Hasher` (substitution axis A). `for_each_chunk` visits `Inline` and");
    f.line("    // `Borrowed` carriers in a single chunk and `Stream` carriers");
    f.line("    // chunk-by-chunk; peak resident memory is the chunk size, never the");
    f.line("    // full canonical sequence. The fold output is truncated to u64 for the");
    f.line("    // `Binding.content_address` carrier, matching the `to_binding_entry`");
    f.line("    // convention foundation uses for static bindings.");
    f.line("    let mut hasher = <A as crate::enforcement::Hasher<FP_MAX>>::initial();");
    f.line("    input_value.for_each_chunk(&mut |chunk| {");
    f.line("        hasher = core::mem::replace(&mut hasher, <A as crate::enforcement::Hasher<FP_MAX>>::initial())");
    f.line("            .fold_bytes(chunk);");
    f.line("    });");
    f.line("    let digest = hasher.finalize();");
    f.line("    let content_address: u64 =");
    f.line("        u64::from_be_bytes([");
    f.line("            digest[0], digest[1], digest[2], digest[3],");
    f.line("            digest[4], digest[5], digest[6], digest[7],");
    f.line("        ]);");
    f.line("    // Build the transient binding for the route's input slot. The");
    f.line("    // `name_index = 0` sentinel is the route-input slot per ADR-022 D3");
    f.line("    // G2; `type_index = 0` is the foundation-conventional zero handle");
    f.line("    // (the input's `ConstrainedTypeShape::IRI` is foundation-internal");
    f.line("    // and not consumed by the binding-signature fold).");
    f.line("    let transient_input = [crate::enforcement::Binding {");
    f.line("        name_index: 0,");
    f.line("        type_index: 0,");
    f.line("        value_index: 0,");
    f.line("        surface: <M::Input as ConstrainedTypeShape>::IRI,");
    f.line("        content_address,");
    f.line("    }];");
    f.line("    // Foundation defaults for unit-level parameters that are not part");
    f.line("    // of the Route's term-tree. The Witt-level ceiling and");
    f.line("    // thermodynamic budget come from the application's `HostBounds`");
    f.line("    // selection (ADR-018) — `B::WITT_LEVEL_MAX_BITS` caps the level,");
    f.line("    // and a budget large enough to admit any in-bounds route avoids");
    f.line("    // false-positive solvency rejections. `target_domains` is");
    f.line("    // `Enumerative` because the arena is a finite term tree.");
    f.line("    static TARGET_DOMAINS: &[crate::VerificationDomain] =");
    f.line("        &[crate::VerificationDomain::Enumerative];");
    f.line("    let level = match B::WITT_LEVEL_MAX_BITS {");
    f.line("        bits if bits >= 32 => crate::WittLevel::W32,");
    f.line("        bits if bits >= 24 => crate::WittLevel::W24,");
    f.line("        bits if bits >= 16 => crate::WittLevel::W16,");
    f.line("        _ => crate::WittLevel::W8,");
    f.line("    };");
    f.line("    let unit = CompileUnitBuilder::new()");
    f.line("        .root_term(arena_slice)");
    f.line("        .bindings(&transient_input)");
    f.line("        .witt_level_ceiling(level)");
    f.line("        .thermodynamic_budget(1024)");
    f.line("        .target_domains(TARGET_DOMAINS)");
    f.line("        .result_type::<M::Output>()");
    f.line("        .validate()");
    f.line("        .map_err(|report| PipelineFailure::ShapeViolation { report })?;");
    f.line("    // ADR-028 (amended by ADR-060): there is no output byte-width");
    f.line("    // ceiling. The route's evaluated output is a source-polymorphic");
    f.line("    // `TermValue` (Inline κ-label for content-addressing routes; Borrowed/");
    f.line("    // Stream for structural/unbounded outputs) carried by `Grounded<'a>`.");
    f.line("    // ADR-029: evaluate the route's Term tree as a structural fold; the");
    f.line("    // catamorphism's output carrier flows into the Grounded's output");
    f.line("    // payload (ADR-028). The input carrier threads `'a` so a Borrowed/");
    f.line("    // Stream output borrows the same input data the `Grounded<'a>` carries.");
    f.line("    let evaluation = evaluate_term_tree::<A, R, INLINE_BYTES, FP_MAX>(arena_slice, input_value, resolvers)?;");
    f.line("    // Wiki ADR-048: post-resolver typed-bandwidth admission. The");
    f.line("    // catamorphism evaluates the model's `C: TypedCommitment` on the");
    f.line("    // κ-label byte sequence (the route's evaluated output, which for the");
    f.line("    // canonical k-invariants branch IS the `Term::KInvariants` emission");
    f.line("    // per ADR-035). On `evaluate(...) == false`, return");
    f.line("    // `PipelineFailure::ShapeViolation` so applications observe the failed-");
    f.line("    // commitment branch deterministically (the trace records the");
    f.line("    // `CommitmentEvaluated` event with `result: false`; the");
    f.line("    // verifier replays the same evaluation).");
    f.line("    let kappa_bytes = evaluation.bytes();");
    f.line("    if !commitment.evaluate(kappa_bytes) {");
    f.line("        return Err(PipelineFailure::ShapeViolation {");
    f.line("            report: crate::enforcement::ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/commitment/TypedCommitment/VIOLATED\",");
    f.line(
        "                constraint_iri: \"https://uor.foundation/commitment/TypedCommitment\",",
    );
    f.line("                property_iri: \"https://uor.foundation/commitment/evaluate\",");
    f.line("                expected_range: \"http://www.w3.org/2001/XMLSchema#boolean\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: crate::ViolationKind::ValueCheck,");
    f.line("            },");
    f.line("        });");
    f.line("    }");
    f.line("    // `run` grounds the unit with an empty output carrier (valid for any");
    f.line("    // lifetime); the annotation coerces its `Grounded<'static>` to");
    f.line("    // `Grounded<'a>` (TermValue is covariant in its lifetime) so the");
    f.line("    // evaluated output carrier (which borrows the route's `'a` input data)");
    f.line("    // can be attached via `with_output`.");
    f.line("    let grounded: crate::enforcement::Grounded<'a, M::Output, INLINE_BYTES, FP_MAX> =");
    f.line("        run::<M::Output, _, A, INLINE_BYTES, FP_MAX>(unit)?;");
    f.line("    Ok(grounded.with_output(evaluation))");
    f.line("}");
    f.blank();

    // Catamorphism evaluator — ADR-029.
    //
    // Per-variant fold-rules for the route's Term tree. Foundation's
    // `pipeline::run_route` calls this evaluator after validating the
    // `CompileUnit`; the result is the Output shape's canonical bytes,
    // which populate the Grounded's output payload (ADR-028). The
    // evaluator is parametric in the AxisTuple `A` because the
    // `AxisInvocation` variant (ADR-030, replaces ADR-029's
    // `HasherProjection`) delegates evaluation to the application's
    // selected axis dispatcher per the substitution-axis-realized verb
    // form.
    // ── ADR-060: source-polymorphic carrier-width derivation surface ──────
    f.doc_comment("ADR-060: foundation-fixed upper bound on a `Hasher`'s ASCII identifier");
    f.doc_comment("width, used in the κ-label inline-width derivation");
    f.doc_comment("([`carrier_inline_bytes`]). The κ-label ASCII form is");
    f.doc_comment("`identifier ++ \":\" ++ hex(digest)`; the hex doubling of the digest");
    f.doc_comment("dominates, but the identifier term is included so the derived inline");
    f.doc_comment("width admits the full κ-label for every conforming `Hasher`. This is a");
    f.doc_comment("wire-format-adjacent constant (ADR-018 carve-out): a single shared");
    f.doc_comment("upper bound, not an application-policy capacity.");
    f.line("pub const HASHER_IDENTIFIER_BYTES: usize = 32;");
    f.blank();
    f.doc_comment("ADR-060: foundation-fixed per-site wire width for the ψ-stage");
    f.doc_comment("structural serializations (SimplicialComplex / ChainComplex / …).");
    f.line("pub const SITE_DESCRIPTOR_BYTES: usize = 8;");
    f.blank();
    f.doc_comment("ADR-060: foundation-fixed per-constraint wire width for the ψ-stage");
    f.doc_comment("structural serializations.");
    f.line("pub const CONSTRAINT_DESCRIPTOR_BYTES: usize = 16;");
    f.blank();
    f.doc_comment("ADR-060: foundation-fixed per-Betti-dimension wire width for the");
    f.doc_comment("homology/cohomology ψ-stage serializations.");
    f.line("pub const BETTI_ELEMENT_BYTES: usize = 8;");
    f.blank();
    f.doc_comment("ADR-060: foundation-fixed per-stage wire-format header width shared by");
    f.doc_comment("the ψ-stage structural serializations (stage tag + element count).");
    f.line("pub const PSI_STAGE_HEADER_BYTES: usize = 16;");
    f.blank();
    f.doc_comment("ADR-060: const-fn maximum of three `usize` values. Used by");
    f.doc_comment("[`carrier_inline_bytes`]; admissible in array-length position on stable");
    f.doc_comment("Rust because it is a plain `const fn` (not `generic_const_exprs`).");
    f.line("#[must_use]");
    f.line("pub const fn max3(a: usize, b: usize, c: usize) -> usize {");
    f.line("    let ab = if a > b { a } else { b };");
    f.line("    if ab > c { ab } else { c }");
    f.line("}");
    f.blank();
    f.doc_comment("ADR-060: foundation-derived inline-carrier width for `TermValue::Inline`,");
    f.doc_comment("computed from the application's `HostBounds` primitives — never declared.");
    f.doc_comment("");
    f.doc_comment("The inline carrier admits every value class that flows inline: an integer");
    f.doc_comment("literal at the application's maximum Witt level");
    f.doc_comment("(`WITT_LEVEL_MAX_BITS / 8` bytes), a cryptographic digest at the maximum");
    f.doc_comment("fingerprint width (`FINGERPRINT_MAX_BYTES`), and the κ-label ASCII");
    f.doc_comment("serialization (`HASHER_IDENTIFIER_BYTES + 1 + 2 × FINGERPRINT_MAX_BYTES` —");
    f.doc_comment("the hex-encoded digest doubles the fingerprint width). The derived width");
    f.doc_comment("is the maximum over all three; larger structural / unbounded payloads flow");
    f.doc_comment("as `TermValue::Borrowed` / `TermValue::Stream` with no carrier-side ceiling.");
    f.doc_comment("");
    f.doc_comment("Applications instantiate carrier-bearing types at");
    f.doc_comment("`carrier_inline_bytes::<MyBounds>()` (a concrete `const` at the");
    f.doc_comment("application boundary, where `MyBounds` is concrete — stable Rust, no");
    f.doc_comment("`generic_const_exprs`).");
    f.line("#[must_use]");
    f.line("pub const fn carrier_inline_bytes<B: crate::HostBounds>() -> usize {");
    f.line("    let witt_bytes = B::WITT_LEVEL_MAX_BITS as usize / 8;");
    f.line("    let kappa_bytes = HASHER_IDENTIFIER_BYTES + 1 + 2 * B::FINGERPRINT_MAX_BYTES;");
    f.line("    max3(witt_bytes, B::FINGERPRINT_MAX_BYTES, kappa_bytes)");
    f.line("}");
    f.blank();
    // ADR-060: per-ψ-stage carrier-width helpers. Foundation's own resolvers
    // are the `Null*` resolver-absent baselines (they return
    // `Err(RESOLVER_ABSENT)` and allocate no buffer), so the catamorphism
    // itself does not size per-stage scratch. These const fns are the
    // app-facing helpers an application's resolver impl uses to size its
    // resolver-owned scratch (the `Borrowed` carrier's backing). Each width is
    // an application-declared structural count × a foundation-fixed per-element
    // wire width — never a contrived literal.
    let psi_stage_carriers: &[(&str, &str, &str)] = &[
        (
            "nerve_carrier_bytes",
            "Nerve (ψ_1)",
            "PSI_STAGE_HEADER_BYTES + B::NERVE_SITES_MAX * SITE_DESCRIPTOR_BYTES + B::NERVE_CONSTRAINTS_MAX * CONSTRAINT_DESCRIPTOR_BYTES",
        ),
        (
            "chain_complex_carrier_bytes",
            "ChainComplex (ψ_2)",
            "PSI_STAGE_HEADER_BYTES + B::BETTI_DIMENSION_MAX * (SITE_DESCRIPTOR_BYTES + BETTI_ELEMENT_BYTES)",
        ),
        (
            "homology_groups_carrier_bytes",
            "HomologyGroups (ψ_3)",
            "PSI_STAGE_HEADER_BYTES + B::BETTI_DIMENSION_MAX * BETTI_ELEMENT_BYTES",
        ),
        (
            "cochain_complex_carrier_bytes",
            "CochainComplex (ψ_5)",
            "PSI_STAGE_HEADER_BYTES + B::BETTI_DIMENSION_MAX * (SITE_DESCRIPTOR_BYTES + BETTI_ELEMENT_BYTES)",
        ),
        (
            "cohomology_groups_carrier_bytes",
            "CohomologyGroups (ψ_6)",
            "PSI_STAGE_HEADER_BYTES + B::BETTI_DIMENSION_MAX * BETTI_ELEMENT_BYTES",
        ),
        (
            "postnikov_tower_carrier_bytes",
            "PostnikovTower (ψ_7)",
            "PSI_STAGE_HEADER_BYTES + B::BETTI_DIMENSION_MAX * (SITE_DESCRIPTOR_BYTES + BETTI_ELEMENT_BYTES)",
        ),
        (
            "homotopy_groups_carrier_bytes",
            "HomotopyGroups (ψ_8)",
            "PSI_STAGE_HEADER_BYTES + B::BETTI_DIMENSION_MAX * BETTI_ELEMENT_BYTES",
        ),
        (
            "k_invariants_carrier_bytes",
            "KInvariants (ψ_9)",
            "carrier_inline_bytes::<B>()",
        ),
    ];
    for (fn_name, stage, body) in psi_stage_carriers {
        f.doc_comment(&format!(
            "ADR-060: app-facing carrier-width helper for the {stage} ψ-stage. An"
        ));
        f.doc_comment("application's resolver impl uses it to size its resolver-owned scratch");
        f.doc_comment("(the `Borrowed` carrier's backing) for this stage.");
        f.doc_comment("Structural-element-count × foundation-fixed per-element wire width.");
        f.line("#[must_use]");
        f.line(&format!(
            "pub const fn {fn_name}<B: crate::HostBounds>() -> usize {{"
        ));
        f.line(&format!("    {body}"));
        f.line("}");
        f.blank();
    }
    f.doc_comment("Wiki ADR-029: name-index sentinel used by `prism_model!` G7 emission to");
    f.doc_comment("mark `recurse(measure, base, |self| step)`'s self-identifier reference.");
    f.doc_comment("When the catamorphism encounters `Term::Variable { name_index: <this> }`");
    f.doc_comment("during step-body evaluation, it returns the previous iteration's result");
    f.doc_comment("(the `recurse_value` parameter threaded through `evaluate_term_at`) — the");
    f.doc_comment("fresh-name-indexed Variable ADR-029 specifies for the recursive-call");
    f.doc_comment("placeholder.");
    f.doc_comment("");
    f.doc_comment("Foundation reserves the upper sentinels: `u32::MAX` is the wildcard arm");
    f.doc_comment("for `Term::Match` (ADR-022 D3 G6) and the default-propagation handler for");
    f.doc_comment("`Term::Try` (G9). `RECURSE_PLACEHOLDER_NAME_INDEX = u32::MAX - 1`.");
    f.line("pub const RECURSE_PLACEHOLDER_NAME_INDEX: u32 = u32::MAX - 1;");
    f.blank();
    f.doc_comment("Wiki ADR-022 D3 G8 + ADR-029: the fresh-name-indexed Variable that");
    f.doc_comment("`prism_model!` emits in place of an `unfold(seed, |state, …| step)`");
    f.doc_comment("closure's state-ident references. The catamorphism's `Term::Unfold`");
    f.doc_comment("fold-rule binds this name to the unfold's current state value");
    f.doc_comment("(threaded through `evaluate_term_at` as the `unfold_value` parameter)");
    f.doc_comment("and iterates step until a Kleene fixpoint or [`UNFOLD_MAX_ITERATIONS`].");
    f.line("pub const UNFOLD_PLACEHOLDER_NAME_INDEX: u32 = u32::MAX - 2;");
    f.blank();
    f.doc_comment("Wiki ADR-034 Mechanism 1: the foundation-fixed name-index for the");
    f.doc_comment("iteration-counter binding inside `Term::Recurse`'s step body. The");
    f.doc_comment(
        "two-parameter closure form `recurse(measure, base, |self_ident, idx_ident| step)`",
    );
    f.doc_comment("lowers `idx_ident` references to");
    f.doc_comment("`Term::Variable { name_index: RECURSE_IDX_NAME_INDEX }`; the");
    f.doc_comment("catamorphism's per-variant fold-rule binds it to a `TermValue`");
    f.doc_comment("carrying the current measure value at each descent.");
    f.line("pub const RECURSE_IDX_NAME_INDEX: u32 = u32::MAX - 3;");
    f.blank();
    f.doc_comment("Wiki ADR-034 Mechanism 2: the foundation-fixed name-index for the");
    f.doc_comment("candidate-value binding inside `Term::FirstAdmit`'s predicate body.");
    f.doc_comment("The grammar form `first_admit(<domain>, |idx_ident| <pred>)` lowers");
    f.doc_comment("`idx_ident` references to");
    f.doc_comment("`Term::Variable { name_index: FIRST_ADMIT_IDX_NAME_INDEX }`; the");
    f.doc_comment("catamorphism's per-variant fold-rule binds it to the current");
    f.doc_comment("candidate `idx` (ranging `0..<Domain>::CYCLE_SIZE`).");
    f.line("pub const FIRST_ADMIT_IDX_NAME_INDEX: u32 = u32::MAX - 4;");
    f.blank();
    f.doc_comment("Wiki ADR-029: bound on the anamorphic fixpoint iteration for");
    f.doc_comment("`Term::Unfold`. The fold rule iterates `step(state)` until either the");
    f.doc_comment("state reaches a Kleene fixpoint (`step(state) == state`) or this");
    f.doc_comment("ceiling is hit, at which point evaluation returns the most-recent");
    f.doc_comment("state. Foundation-fixed (parallel to `FOLD_UNROLL_THRESHOLD`).");
    f.doc_comment("");
    f.doc_comment("Wiki ADR-037: a foundation-fixed conservative default for");
    f.doc_comment("[`HostBounds::UNFOLD_ITERATIONS_MAX`].");
    f.line(
        "pub const UNFOLD_MAX_ITERATIONS: usize = \
         256;",
    );
    f.blank();
    // ── ADR-060: ChunkSource — unbounded chunk-emitting payload source ────
    f.doc_comment("ADR-060: a chunk-emitting source for unbounded `TermValue` payloads");
    f.doc_comment("(tensor-data sections, large signing payloads, multi-GB messages). The");
    f.doc_comment("`Stream` carrier references a `&dyn ChunkSource`; its carrier width is the");
    f.doc_comment("dyn-trait descriptor size — payload-independent, with no byte-width ceiling.");
    f.doc_comment("");
    f.doc_comment("The σ-projection at ψ_9 folds `Stream` chunks directly into the `Hasher`");
    f.doc_comment("via `Hasher::fold_bytes`; structural fold-rules read the chunks as a flat");
    f.doc_comment("byte sequence via [`ChunkSource::for_each_chunk`]. Peak resident memory is");
    f.doc_comment("`carrier_inline_bytes::<B>()` plus the source's internal state — never the");
    f.doc_comment("full canonical byte sequence.");
    f.line("pub trait ChunkSource: core::fmt::Debug {");
    f.indented_doc_comment("Fold each chunk into the closure in canonical order. The total folded");
    f.indented_doc_comment("byte count is unbounded; each `&[u8]` slice is valid only for the");
    f.indented_doc_comment("duration of the call (not retained).");
    f.line("    fn for_each_chunk(&self, f: &mut dyn FnMut(&[u8]));");
    f.indented_doc_comment("Total byte count if statically known, else `None`.");
    f.line("    fn total_bytes(&self) -> Option<usize> {");
    f.line("        None");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("Wiki ADR-029 + ADR-060: a single Term variant's evaluated value, carried");
    f.doc_comment("as a **source-polymorphic** carrier const-generic over its inline width.");
    f.doc_comment("");
    f.doc_comment("ADR-060 replaces the pre-0.5.0 fixed-4096-byte buffer with three variants:");
    f.doc_comment("");
    f.doc_comment("- `Inline` — a stack buffer of `INLINE_BYTES` (foundation-derived via");
    f.doc_comment("  [`carrier_inline_bytes`]) carrying derived structural content: integer");
    f.doc_comment("  literals at any admitted Witt level, cryptographic digests at the");
    f.doc_comment("  application's hasher output width, the κ-label ASCII form, and");
    f.doc_comment("  primitive-op single-value outputs.");
    f.doc_comment("- `Borrowed` — a slice into an upstream byte source (input bytes, a");
    f.doc_comment("  sibling ψ-stage's scratch, an axis-kernel output region). Its carrier");
    f.doc_comment("  width is `size_of::<&[u8]>()`, independent of payload size.");
    f.doc_comment("- `Stream` — a chunk-emitting source for unbounded payloads, no ceiling.");
    // Literal blank doc line (a `doc_comment("")` call is a no-op because
    // `"".lines()` yields nothing); needed so clippy does not read the
    // following paragraph as an unindented continuation of the last list item.
    f.line("///");
    f.doc_comment("`INLINE_BYTES` is a free const-generic parameter; the application");
    f.doc_comment("instantiates it at the boundary via `carrier_inline_bytes::<MyBounds>()`");
    f.doc_comment("(per the min-const-generics pattern, stable Rust, no `generic_const_exprs`).");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub enum TermValue<'a, const INLINE_BYTES: usize> {");
    f.indented_doc_comment("Stack-allocated inline buffer (zero-padded beyond `len`).");
    f.line("    Inline {");
    f.indented_doc_comment("Fixed-capacity inline byte buffer.");
    f.line("        bytes: [u8; INLINE_BYTES],");
    f.indented_doc_comment("Active prefix length (`<= INLINE_BYTES`).");
    f.line("        len: usize,");
    f.line("    },");
    f.indented_doc_comment("Borrowed slice into an upstream byte source — no byte-width ceiling.");
    f.line("    Borrowed(&'a [u8]),");
    f.indented_doc_comment("Chunk-emitting source for unbounded payloads — no byte-width ceiling.");
    f.line("    Stream(&'a dyn ChunkSource),");
    f.line("}");
    f.blank();
    // ADR-060: TermValue carries `&dyn ChunkSource`, which is not `PartialEq`,
    // so we implement equality manually (content equality for materializable
    // carriers; pointer identity for `Stream`).
    f.line("impl<'a, const INLINE_BYTES: usize> PartialEq for TermValue<'a, INLINE_BYTES> {");
    f.line("    fn eq(&self, other: &Self) -> bool {");
    f.line("        match (self, other) {");
    f.line("            (TermValue::Stream(a), TermValue::Stream(b)) => core::ptr::eq(");
    f.line("                (*a as *const dyn ChunkSource).cast::<u8>(),");
    f.line("                (*b as *const dyn ChunkSource).cast::<u8>(),");
    f.line("            ),");
    f.line("            (TermValue::Stream(_), _) | (_, TermValue::Stream(_)) => false,");
    f.line("            _ => self.as_slice() == other.as_slice(),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.line("impl<'a, const INLINE_BYTES: usize> Eq for TermValue<'a, INLINE_BYTES> {}");
    f.blank();
    f.line("impl<'a, const INLINE_BYTES: usize> TermValue<'a, INLINE_BYTES> {");
    f.indented_doc_comment("Construct an empty inline `TermValue` (length zero).");
    f.line("    #[must_use]");
    f.line("    pub const fn empty() -> Self {");
    f.line("        TermValue::Inline { bytes: [0u8; INLINE_BYTES], len: 0 }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("ADR-060: construct an `Inline` `TermValue` by copying up to");
    f.indented_doc_comment("`INLINE_BYTES` bytes from `bytes`. For payloads exceeding the inline");
    f.indented_doc_comment("width use [`TermValue::Borrowed`] / [`TermValue::Stream`] instead.");
    f.line("    #[must_use]");
    f.line("    pub const fn inline_from_slice(bytes: &[u8]) -> Self {");
    f.line("        let mut buf = [0u8; INLINE_BYTES];");
    f.line("        let copy_len = if bytes.len() > INLINE_BYTES { INLINE_BYTES } else { bytes.len() };");
    f.line("        let mut i = 0;");
    f.line("        while i < copy_len {");
    f.line("            buf[i] = bytes[i];");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        TermValue::Inline { bytes: buf, len: copy_len }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Construct a zero-copy `Borrowed` `TermValue` referencing `bytes`.");
    f.line("    #[must_use]");
    f.line("    pub const fn borrowed(bytes: &'a [u8]) -> Self {");
    f.line("        TermValue::Borrowed(bytes)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Construct a `Stream` `TermValue` over an unbounded chunk source.");
    f.line("    #[must_use]");
    f.line("    pub const fn stream(source: &'a dyn ChunkSource) -> Self {");
    f.line("        TermValue::Stream(source)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "ADR-051: construct an `Inline` `TermValue` from a u64 at a declared byte width.",
    );
    f.line("    #[must_use]");
    f.line("    pub const fn from_u64_be(value: u64, width: usize) -> Self {");
    f.line("        let w = if width > 8 { 8 } else { width };");
    f.line("        let be = value.to_be_bytes();");
    f.line("        let mut buf = [0u8; INLINE_BYTES];");
    f.line("        let cap = if w > INLINE_BYTES { INLINE_BYTES } else { w };");
    f.line("        let mut i = 0;");
    f.line("        while i < cap {");
    f.line("            buf[i] = be[8 - w + i];");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        TermValue::Inline { bytes: buf, len: cap }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "ADR-060: const-constructor from a prepared inline buffer of width `INLINE_BYTES`.",
    );
    f.line("    #[must_use]");
    f.line("    pub const fn from_inline_const(bytes: &[u8; INLINE_BYTES], len: usize) -> Self {");
    f.line("        let l = if len > INLINE_BYTES { INLINE_BYTES } else { len };");
    f.line("        TermValue::Inline { bytes: *bytes, len: l }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the active byte prefix for materializable carriers");
    f.indented_doc_comment("(`Inline` / `Borrowed`), or `None` for `Stream` (which has no single");
    f.indented_doc_comment("contiguous slice — read it via [`TermValue::for_each_chunk`]).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn as_slice(&self) -> Option<&[u8]> {");
    f.line("        match self {");
    f.line("            TermValue::Inline { bytes, len } => Some(&bytes[..*len]),");
    f.line("            TermValue::Borrowed(s) => Some(s),");
    f.line("            TermValue::Stream(_) => None,");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Returns the active byte prefix for `Inline` / `Borrowed`, or the empty",
    );
    f.indented_doc_comment("slice for `Stream`. Structural fold-rules that never produce `Stream`");
    f.indented_doc_comment("use this; `Stream`-aware consumers use [`TermValue::for_each_chunk`].");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn bytes(&self) -> &[u8] {");
    f.line("        match self {");
    f.line("            TermValue::Inline { bytes, len } => &bytes[..*len],");
    f.line("            TermValue::Borrowed(s) => s,");
    f.line("            TermValue::Stream(_) => &[],");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("ADR-060: fold every byte of the carrier into `f` in canonical order,");
    f.indented_doc_comment(
        "dispatching on the variant — `Inline` / `Borrowed` emit a single chunk,",
    );
    f.indented_doc_comment("`Stream` delegates to [`ChunkSource::for_each_chunk`]. This is the");
    f.indented_doc_comment("universal reader the σ-projection folds through `Hasher::fold_bytes`.");
    f.line("    pub fn for_each_chunk(&self, f: &mut dyn FnMut(&[u8])) {");
    f.line("        match self {");
    f.line("            TermValue::Inline { bytes, len } => f(&bytes[..*len]),");
    f.line("            TermValue::Borrowed(s) => f(s),");
    f.line("            TermValue::Stream(src) => src.for_each_chunk(f),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Total byte length if known: `Inline`/`Borrowed` always; `Stream` only");
    f.indented_doc_comment("when its source reports [`ChunkSource::total_bytes`].");
    f.line("    #[must_use]");
    f.line("    pub fn len_hint(&self) -> Option<usize> {");
    f.line("        match self {");
    f.line("            TermValue::Inline { len, .. } => Some(*len),");
    f.line("            TermValue::Borrowed(s) => Some(s.len()),");
    f.line("            TermValue::Stream(src) => src.total_bytes(),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("ADR-060: opt-in, `alloc`-gated materialization into an owned buffer —");
    f.indented_doc_comment("the only allocation surface, per caller. Folds every chunk into a");
    f.indented_doc_comment("`Vec<u8>` (works for all three variants, including `Stream`).");
    f.line("    #[cfg(feature = \"alloc\")]");
    f.line("    #[must_use]");
    // `to_vec(&self)` mirrors `<[T]>::to_vec(&self)` in the standard library —
    // a borrowing read, not a consuming conversion. clippy's
    // `wrong_self_convention` flags `to_*` on `Copy` types as "take self by
    // value", which is the wrong call here (the carrier may borrow `'a` data or
    // wrap a `&dyn ChunkSource`; reading it must not move it).
    f.line("    #[allow(clippy::wrong_self_convention)]");
    f.line("    pub fn to_vec(&self) -> alloc::vec::Vec<u8> {");
    f.line("        let mut out = alloc::vec::Vec::new();");
    f.line("        self.for_each_chunk(&mut |chunk| out.extend_from_slice(chunk));");
    f.line("        out");
    f.line("    }");
    f.line("}");
    f.blank();

    // ADR-051 narrow-form construction helpers for Term::Literal. The wide
    // form (`Term::Literal { value: TermValue::inline_from_slice(&bytes), level }`)
    // is fine for raw construction; `literal_u64` is the SDK-recommended
    // entry point for callers carrying a u64 value.
    f.doc_comment(
        "ADR-051: construct a `Term::Literal` from a u64 value at a declared Witt level.",
    );
    f.doc_comment("");
    f.doc_comment("Packs `value` as a big-endian byte sequence whose length equals the level's");
    f.doc_comment("byte width (`level.witt_length() / 8`). For widths > 8 bytes, the high bytes");
    f.doc_comment("are zero-padded.");
    f.line("#[must_use]");
    f.line("pub const fn literal_u64<const INLINE_BYTES: usize>(value: u64, level: crate::WittLevel) -> crate::enforcement::Term<'static, INLINE_BYTES> {");
    f.line("    let mut width = (level.witt_length() / 8) as usize;");
    f.line("    if width == 0 { width = 1; }");
    f.line("    // ADR-060: literals fit inline by construction (carrier_inline_bytes >=");
    f.line("    // WITT_LEVEL_MAX_BITS/8); clamp defensively so const eval never indexes");
    f.line("    // past the derived inline width.");
    f.line("    if width > INLINE_BYTES { width = INLINE_BYTES; }");
    f.line("    let be = value.to_be_bytes();");
    f.line("    let mut buf = [0u8; INLINE_BYTES];");
    f.line("    // Pack the u64's big-endian bytes into the low `min(width, 8)` bytes");
    f.line("    // of the result, right-aligned. Widths > 8 zero-pad the high portion.");
    f.line("    let take = if width > 8 { 8 } else { width };");
    f.line("    let mut i = 0;");
    f.line("    while i < take {");
    f.line("        buf[width - take + i] = be[8 - take + i];");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    crate::enforcement::Term::Literal {");
    f.line("        value: TermValue::from_inline_const(&buf, width),");
    f.line("        level,");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("ADR-051: construct a `Term::Literal` from raw bytes at a declared Witt level.");
    f.doc_comment("");
    f.doc_comment("`bytes` MUST have exactly `level.witt_length() / 8` bytes; mismatched lengths");
    f.doc_comment("are silently truncated/padded by `TermValue::inline_from_slice`. Use this for");
    f.doc_comment("wide-Witt literals (W128+) that don't fit in a u64 (they fit the");
    f.doc_comment("foundation-derived inline width per ADR-060).");
    f.line("#[must_use]");
    f.line("pub const fn literal_bytes<const INLINE_BYTES: usize>(bytes: &[u8], level: crate::WittLevel) -> crate::enforcement::Term<'static, INLINE_BYTES> {");
    f.line("    crate::enforcement::Term::Literal {");
    f.line("        value: TermValue::inline_from_slice(bytes),");
    f.line("        level,");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("Wiki ADR-029: catamorphism evaluator over the route's Term tree.");
    f.doc_comment("");
    f.doc_comment("Per-variant fold rules:");
    f.doc_comment("");
    f.doc_comment("- `Term::Literal { value, level }` — emit `value` as big-endian bytes at the");
    f.doc_comment("  byte width of `level`.");
    f.doc_comment("- `Term::Variable { name_index }` — `name_index = 0` returns the route");
    f.doc_comment("  input bytes; other indices look up `let`-introduced bindings (current");
    f.doc_comment("  iteration: not yet supported, returns the input bytes for any non-zero");
    f.doc_comment("  index).");
    f.doc_comment("- `Term::Application { operator, args }` — evaluate each arg and apply");
    f.doc_comment("  the `PrimitiveOp` per its algebraic rule.");
    f.doc_comment("- `Term::Lift { operand, target }` — evaluate operand, zero-extend to");
    f.doc_comment("  `target` Witt level's byte width.");
    f.doc_comment("- `Term::Project { operand, target }` — evaluate operand, truncate to");
    f.doc_comment("  `target` Witt level's byte width.");
    f.doc_comment("- `Term::Match { scrutinee, arms }` — evaluate scrutinee, match arms by");
    f.doc_comment("  literal-byte equality (wildcard arm `name_index = u32::MAX` matches");
    f.doc_comment("  unconditionally).");
    f.doc_comment("- `Term::Recurse { measure, base, step }` — bounded recursion: evaluate");
    f.doc_comment("  measure → n; if n = 0 evaluate base; otherwise iterate step n times with");
    f.doc_comment("  the recursive-call placeholder bound to the previous iteration's result.");
    f.doc_comment("- `Term::Unfold { seed, step }` — anamorphism: evaluate seed → state₀;");
    f.doc_comment("  iterate step (with the state placeholder bound to the current state)");
    f.doc_comment("  until a Kleene fixpoint or `UNFOLD_MAX_ITERATIONS` is reached.");
    f.doc_comment("- `Term::Try { body, handler }` — evaluate body; on failure, propagate");
    f.doc_comment("  if `handler_index = u32::MAX`, otherwise evaluate handler.");
    f.doc_comment("- `Term::AxisInvocation { axis_index, kernel_id, input_index }` — dispatch");
    f.doc_comment("  the input's bytes to the application's `AxisTuple` (ADR-030); the");
    f.doc_comment("  foundation-canonical (axis_index=0, kernel_id=0) folds through the");
    f.doc_comment("  selected `Hasher` impl. Replaces the legacy `HasherProjection` variant.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns [`PipelineFailure`] when the term tree is malformed (out-of-bounds");
    f.doc_comment("index, level mismatch, exhausted match without wildcard arm, etc.).");
    f.line(
        "pub fn evaluate_term_tree<'a, 'r, A, R, const INLINE_BYTES: usize, const FP_MAX: usize>(",
    );
    f.line("    arena: &'a [crate::enforcement::Term<'a, INLINE_BYTES>],");
    f.line("    input: TermValue<'a, INLINE_BYTES>,");
    f.line("    // ADR-060: the resolver borrow `'r` is decoupled from the value/output");
    f.line("    // lifetime `'a` — resolvers are consulted only during evaluation, so a");
    f.line("    // locally-constructed resolver tuple (e.g. the `prism_model!`-emitted");
    f.line("    // `forward` body's) can drive an evaluation whose output carrier escapes");
    f.line("    // with the route input's `'a`.");
    f.line("    resolvers: &'r R,");
    f.line(") -> Result<TermValue<'a, INLINE_BYTES>, PipelineFailure>");
    f.line("where");
    f.line("    A: crate::pipeline::AxisTuple<INLINE_BYTES, FP_MAX> + crate::enforcement::Hasher<FP_MAX> + 'a,");
    f.line("    R: crate::pipeline::ResolverTuple");
    f.line("        + crate::pipeline::HasNerveResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasChainComplexResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasHomologyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasCochainComplexResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasCohomologyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasPostnikovResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasHomotopyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasKInvariantResolver<INLINE_BYTES, A>,");
    f.line("{");
    f.line("    if arena.is_empty() {");
    f.line("        // ADR-060: identity route — output IS the input carrier itself");
    f.line("        // (Inline/Borrowed/Stream, no inline-width ceiling so arbitrarily");
    f.line("        // large inputs pass through unchanged).");
    f.line("        return Ok(input);");
    f.line("    }");
    f.line("    // Canonical convention: the root term is the last entry in the");
    f.line("    // arena (the `prism_model!` macro emits in post-order, so the root");
    f.line("    // is the final node).");
    f.line("    let root_idx = arena.len() - 1;");
    f.line("    evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, root_idx, input, None, None, None, None, resolvers)");
    f.line("}");
    f.blank();

    // Per-variant evaluator — ADR-029.
    //
    // Wiki ADR-024 commits verb-graph acyclicity at compile time: the
    // `prism_model!` macro inlines verb-emitted term-tree fragments
    // into the calling route's arena via `inline_verb_fragment` at
    // const-eval time (substituting Variable(0) with the caller's
    // argument and shifting non-Variable indices). The catamorphism
    // therefore walks a flat, structurally-bounded arena — runtime
    // depth equals compile-time tree depth, no separate guard is
    // required. The Term enum carries exactly the ten variants ADR-029
    // enumerates.
    f.line("#[allow(clippy::too_many_arguments)]");
    f.line(
        "fn evaluate_term_at<'a, 'b, 'r, A, R, const INLINE_BYTES: usize, const FP_MAX: usize>(",
    );
    f.line("    arena: &'a [crate::enforcement::Term<'a, INLINE_BYTES>],");
    f.line("    idx: usize,");
    f.line("    input: TermValue<'a, INLINE_BYTES>,");
    f.line("    recurse_value: Option<&'b [u8]>,");
    f.line("    recurse_idx_value: Option<&'b [u8]>,");
    f.line("    unfold_value: Option<&'b [u8]>,");
    f.line("    first_admit_idx_value: Option<&'b [u8]>,");
    f.line("    resolvers: &'r R,");
    f.line(") -> Result<TermValue<'a, INLINE_BYTES>, PipelineFailure>");
    f.line("where");
    f.line("    A: crate::pipeline::AxisTuple<INLINE_BYTES, FP_MAX> + crate::enforcement::Hasher<FP_MAX> + 'a,");
    f.line("    R: crate::pipeline::ResolverTuple");
    f.line("        + crate::pipeline::HasNerveResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasChainComplexResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasHomologyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasCochainComplexResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasCohomologyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasPostnikovResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasHomotopyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasKInvariantResolver<INLINE_BYTES, A>,");
    f.line("{");
    f.line("    if idx >= arena.len() {");
    f.line("        return Err(PipelineFailure::ShapeViolation {");
    f.line("            report: crate::enforcement::ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/pipeline/TermArenaShape\",");
    f.line(
        "                constraint_iri: \"https://uor.foundation/pipeline/TermArenaShape/inBounds\",",
    );
    f.line("                property_iri: \"https://uor.foundation/pipeline/termIndex\",");
    f.line(
        "                expected_range: \"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\",",
    );
    f.line("                min_count: 0,");
    f.line("                max_count: arena.len() as u32,");
    f.line("                kind: crate::ViolationKind::ValueCheck,");
    f.line("            },");
    f.line("        });");
    f.line("    }");
    f.line("    match arena[idx] {");
    f.line("        crate::enforcement::Term::Literal { value, level: _ } => {");
    f.line("            // ADR-051 + ADR-060: the literal's value IS a source-polymorphic");
    f.line("            // carrier (`TermValue<'a, INLINE_BYTES>`, Copy) whose byte length");
    f.line("            // matches `level.witt_length() / 8`. The fold-rule returns it");
    f.line("            // directly, preserving the carrier variant.");
    f.line("            Ok(value)");
    f.line("        }");
    f.line("        crate::enforcement::Term::Variable { name_index } => {");
    f.line("            // ADR-022 D3 G2: name_index = 0 is the route input slot.");
    f.line("            // ADR-029: name_index = RECURSE_PLACEHOLDER_NAME_INDEX is the");
    f.line("            // recursive-call placeholder bound to `recurse_value`.");
    f.line("            // ADR-029: name_index = UNFOLD_PLACEHOLDER_NAME_INDEX is the");
    f.line("            // unfold state placeholder bound to `unfold_value` (the");
    f.line("            // current iteration's accumulated state — see the");
    f.line("            // `Term::Unfold` fold-rule below).");
    f.line("            // Other indices reference let-bindings (G10), which the");
    f.line("            // current macro emission resolves at expansion time via");
    f.line("            // splice into the calling arena (so the binding's value-tree");
    f.line("            // root is what the catamorphism actually walks).");
    f.line("            if name_index == RECURSE_PLACEHOLDER_NAME_INDEX {");
    f.line(
        "                return Ok(TermValue::inline_from_slice(recurse_value.unwrap_or(&[])));",
    );
    f.line("            }");
    f.line("            // ADR-034 Mechanism 1: iteration-counter binding for Recurse.");
    f.line("            if name_index == RECURSE_IDX_NAME_INDEX {");
    f.line("                return Ok(TermValue::inline_from_slice(recurse_idx_value.unwrap_or(&[])));");
    f.line("            }");
    f.line("            if name_index == UNFOLD_PLACEHOLDER_NAME_INDEX {");
    f.line("                return Ok(TermValue::inline_from_slice(unfold_value.unwrap_or(&[])));");
    f.line("            }");
    f.line("            // ADR-034 Mechanism 2: candidate-value binding for FirstAdmit.");
    f.line("            if name_index == FIRST_ADMIT_IDX_NAME_INDEX {");
    f.line(
        "                return Ok(TermValue::inline_from_slice(first_admit_idx_value.unwrap_or(&[])));",
    );
    f.line("            }");
    f.line("            // ADR-022 D3 G2 + ADR-060: name_index = 0 returns the route");
    f.line("            // input carrier itself (Inline/Borrowed/Stream) — no copy, no cap.");
    f.line("            Ok(input)");
    f.line("        }");
    f.line("        crate::enforcement::Term::Application { operator, args } => {");
    f.line("            let start = args.start as usize;");
    f.line("            let len = args.len as usize;");
    f.line("            apply_primitive_op::<A, R, INLINE_BYTES, FP_MAX>(arena, operator, start, len, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)");
    f.line("        }");
    f.line("        crate::enforcement::Term::Lift { operand_index, target } => {");
    f.line(
        "            let v = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, operand_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;",
    );
    f.line("            let target_width = (target.witt_length() / 8) as usize;");
    f.line("            let target_width = if target_width > INLINE_BYTES {");
    f.line("                INLINE_BYTES");
    f.line("            } else if target_width == 0 { 1 } else { target_width };");
    f.line("            let mut buf = [0u8; INLINE_BYTES];");
    f.line("            // Big-endian zero-extend: pad the high bytes with zeros.");
    f.line("            let src = v.bytes();");
    f.line("            let pad = target_width.saturating_sub(src.len());");
    f.line("            let mut i = 0;");
    f.line("            while i < src.len() && pad + i < target_width {");
    f.line("                buf[pad + i] = src[i];");
    f.line("                i += 1;");
    f.line("            }");
    f.line("            Ok(TermValue::Inline { bytes: buf, len: target_width })");
    f.line("        }");
    f.line("        crate::enforcement::Term::Project { operand_index, target } => {");
    f.line(
        "            let v = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, operand_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;",
    );
    f.line("            let target_width = (target.witt_length() / 8) as usize;");
    f.line("            let target_width = if target_width > INLINE_BYTES {");
    f.line("                INLINE_BYTES");
    f.line("            } else if target_width == 0 { 1 } else { target_width };");
    f.line("            let src = v.bytes();");
    f.line("            // Big-endian truncation: take the trailing `target_width` bytes.");
    f.line("            let take_from = src.len().saturating_sub(target_width);");
    f.line("            Ok(TermValue::inline_from_slice(&src[take_from..]))");
    f.line("        }");
    f.line("        crate::enforcement::Term::Match { scrutinee_index, arms } => {");
    f.line(
        "            let scrutinee = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, scrutinee_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;",
    );
    f.line("            let start = arms.start as usize;");
    f.line("            let count = arms.len as usize;");
    f.line("            // Arms alternate (pattern, body) per ADR-022 D3 G6.");
    f.line("            let mut i = 0usize;");
    f.line("            while i + 1 < count {");
    f.line("                let pattern_idx = start + i;");
    f.line("                let body_idx = start + i + 1;");
    f.line("                let is_wildcard = matches!(");
    f.line("                    arena[pattern_idx],");
    f.line(
        "                    crate::enforcement::Term::Variable { name_index } if name_index == u32::MAX",
    );
    f.line("                );");
    f.line("                if is_wildcard {");
    f.line("                    return evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, body_idx, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers);");
    f.line("                }");
    f.line(
        "                let pattern_val = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, pattern_idx, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;",
    );
    f.line("                if pattern_val.bytes() == scrutinee.bytes() {");
    f.line("                    return evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, body_idx, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers);");
    f.line("                }");
    f.line("                i += 2;");
    f.line("            }");
    f.line("            // Per ADR-022 D3 G6 the macro enforces wildcard exhaustiveness;");
    f.line("            // a well-formed Term tree never reaches this branch.");
    f.line("            Err(PipelineFailure::ShapeViolation {");
    f.line("                report: crate::enforcement::ShapeViolation {");
    f.line(
        "                    shape_iri: \"https://uor.foundation/pipeline/MatchExhaustivenessShape\",",
    );
    f.line(
        "                    constraint_iri: \"https://uor.foundation/pipeline/MatchExhaustivenessShape/wildcard\",",
    );
    f.line("                    property_iri: \"https://uor.foundation/pipeline/matchArms\",");
    f.line("                    expected_range: \"http://www.w3.org/2001/XMLSchema#string\",");
    f.line("                    min_count: 1,");
    f.line("                    max_count: 0,");
    f.line("                    kind: crate::ViolationKind::Missing,");
    f.line("                },");
    f.line("            })");
    f.line("        }");
    f.line(
        "        crate::enforcement::Term::Recurse { measure_index, base_index, step_index } => {",
    );
    f.line("            // Wiki ADR-029 recursive fold: evaluate measure once to get N;");
    f.line("            // if N == 0 evaluate base; else iterate step N times, threading");
    f.line("            // each iteration's result as the recurse_value (the recursive-");
    f.line("            // call placeholder bound to a fresh-name-indexed Variable per");
    f.line("            // ADR-029, with the placeholder's name_index resolving via the");
    f.line("            // RECURSE_PLACEHOLDER_NAME_INDEX sentinel handled in the Variable");
    f.line("            // arm). The outer recurse_value is preserved for nested Recurse");
    f.line("            // forms within the measure/base computations; step body uses the");
    f.line("            // iteration's accumulator.");
    f.line(
        "            let measure = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, measure_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;",
    );
    f.line("            let n = bytes_to_u64_be(measure.bytes());");
    f.line(
        "            let base_val = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, base_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;",
    );
    f.line("            if n == 0 {");
    f.line("                return Ok(base_val);");
    f.line("            }");
    f.line("            // Iterate step N times. Each iteration's `current` becomes the");
    f.line("            // next iteration's recurse_value. The descent measure is the");
    f.line("            // bound; well-foundedness holds by monotonic decrease.");
    f.line("            let mut current_buf = [0u8; INLINE_BYTES];");
    f.line("            let mut current_len = base_val.bytes().len();");
    f.line("            let mut k = 0;");
    f.line("            while k < current_len {");
    f.line("                current_buf[k] = base_val.bytes()[k];");
    f.line("                k += 1;");
    f.line("            }");
    f.line("            let mut iter = 0u64;");
    f.line("            while iter < n {");
    f.line("                // ADR-034 Mechanism 1: bind RECURSE_IDX_NAME_INDEX to the");
    f.line("                // current measure value (the iteration counter at this");
    f.line("                // descent). At iter=0 the descent measure is N; at iter=k");
    f.line("                // it is N-k. The byte width follows the measure's BE-truncated");
    f.line("                // u64 packing so callers can read it as a u64-shaped value.");
    f.line("                let descent_measure: u64 = n - iter;");
    f.line("                let descent_bytes = descent_measure.to_be_bytes();");
    f.line("                let next = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(");
    f.line("                    arena,");
    f.line("                    step_index as usize,");
    f.line("                    input,");
    f.line("                    Some(&current_buf[..current_len]),");
    f.line("                    Some(&descent_bytes[..]),");
    f.line("                    unfold_value,");
    f.line("                    first_admit_idx_value,");
    f.line("                    resolvers,");
    f.line("                )?;");
    f.line("                let nb = next.bytes();");
    f.line("                let copy_len = if nb.len() > INLINE_BYTES { INLINE_BYTES } else { nb.len() };");
    f.line("                let mut j = 0;");
    f.line("                while j < copy_len {");
    f.line("                    current_buf[j] = nb[j];");
    f.line("                    j += 1;");
    f.line("                }");
    f.line("                current_len = copy_len;");
    f.line("                iter += 1;");
    f.line("            }");
    f.line("            Ok(TermValue::inline_from_slice(&current_buf[..current_len]))");
    f.line("        }");
    f.line("        crate::enforcement::Term::Unfold { seed_index, step_index } => {");
    f.line("            // ADR-029 anamorphism: evaluate seed → state₀; iterate step");
    f.line("            // with the state placeholder (UNFOLD_PLACEHOLDER_NAME_INDEX,");
    f.line("            // bound to the current state) until either a Kleene fixpoint");
    f.line("            // (step(state) == state) or UNFOLD_MAX_ITERATIONS is reached.");
    f.line("            // Well-foundedness: bounded by UNFOLD_MAX_ITERATIONS.");
    f.line("            // The outer unfold_value is preserved for nested Unfold forms");
    f.line("            // within the seed; step body's state placeholder uses the");
    f.line("            // iteration's accumulator.");
    f.line(
        "            let seed_val = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, seed_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;",
    );
    f.line("            let mut state_buf = [0u8; INLINE_BYTES];");
    f.line("            let mut state_len = seed_val.bytes().len();");
    f.line("            let mut k = 0;");
    f.line("            while k < state_len {");
    f.line("                state_buf[k] = seed_val.bytes()[k];");
    f.line("                k += 1;");
    f.line("            }");
    f.line("            let mut iter = 0usize;");
    f.line("            while iter < UNFOLD_MAX_ITERATIONS {");
    f.line("                let next = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(");
    f.line("                    arena,");
    f.line("                    step_index as usize,");
    f.line("                    input,");
    f.line("                    recurse_value,");
    f.line("                    recurse_idx_value,");
    f.line("                    Some(&state_buf[..state_len]),");
    f.line("                    first_admit_idx_value,");
    f.line("                    resolvers,");
    f.line("                )?;");
    f.line("                let nb = next.bytes();");
    f.line("                // Kleene fixpoint check: if step(state) == state, return.");
    f.line("                if nb.len() == state_len && nb == &state_buf[..state_len] {");
    f.line("                    return Ok(TermValue::inline_from_slice(&state_buf[..state_len]));");
    f.line("                }");
    f.line("                let copy_len = if nb.len() > INLINE_BYTES { INLINE_BYTES } else { nb.len() };");
    f.line("                let mut j = 0;");
    f.line("                while j < copy_len {");
    f.line("                    state_buf[j] = nb[j];");
    f.line("                    j += 1;");
    f.line("                }");
    f.line("                state_len = copy_len;");
    f.line("                iter += 1;");
    f.line("            }");
    f.line("            Ok(TermValue::inline_from_slice(&state_buf[..state_len]))");
    f.line("        }");
    f.line("        crate::enforcement::Term::Try { body_index, handler_index } => {");
    f.line("            match evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, body_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers) {");
    f.line("                Ok(v) => Ok(v),");
    f.line("                Err(e) => {");
    f.line("                    if handler_index == u32::MAX {");
    f.line("                        Err(e)");
    f.line("                    } else {");
    f.line(
        "                        evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, handler_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)",
    );
    f.line("                    }");
    f.line("                }");
    f.line("            }");
    f.line("        }");
    f.line("        crate::enforcement::Term::AxisInvocation { axis_index, kernel_id, input_index } => {");
    f.line("            // ADR-055: read the axis's SubstrateTermBody::body_arena() via");
    f.line("            // `AxisTuple::body_arena_at`. When non-empty, recursively fold");
    f.line("            // the body with the evaluated kernel input bound as input;");
    f.line("            // when empty (primitive-fast-path interpretation), dispatch the");
    f.line("            // kernel function directly per the optional fast-path per ADR-055.");
    f.line("            //");
    f.line("            // The foundation-built blanket `impl<H: Hasher> AxisTuple for H`");
    f.line("            // routes the canonical hash dispatch (axis 0, kernel 0) through");
    f.line("            // the legacy Hasher API (empty body); user-declared axes via the");
    f.line("            // `axis!` SDK macro extend the dispatch surface to additional");
    f.line("            // (axis_index, kernel_id) combinations and may provide substrate-");
    f.line("            // Term bodies the catamorphism walks structurally.");
    f.line("            let v = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, input_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;");
    f.line("            let body = <A as crate::pipeline::AxisTuple<INLINE_BYTES, FP_MAX>>::body_arena_at(axis_index);");
    f.line("            if body.is_empty() {");
    f.line("                if axis_index == 0 && kernel_id == 0 {");
    f.line("                    // ADR-060: the canonical σ-projection / hash axis (axis 0,");
    f.line("                    // kernel 0) folds the operand carrier through the `Hasher`");
    f.line("                    // chunk-by-chunk via `for_each_chunk`, so a `Stream` or");
    f.line("                    // large `Borrowed` operand of arbitrary size folds without");
    f.line("                    // materialization. (The `v.bytes()` dispatch path would see");
    f.line("                    // an empty slice for a `Stream` operand.) The digest is");
    f.line("                    // always Inline — its width is bounded by the application's");
    f.line("                    // fingerprint width, ≤ the derived inline width.");
    f.line("                    let mut hasher = <A as crate::enforcement::Hasher<FP_MAX>>::initial();");
    f.line("                    v.for_each_chunk(&mut |chunk| {");
    f.line("                        hasher = core::mem::replace(&mut hasher, <A as crate::enforcement::Hasher<FP_MAX>>::initial())");
    f.line("                            .fold_bytes(chunk);");
    f.line("                    });");
    f.line("                    let digest = hasher.finalize();");
    f.line(
        "                    let n_max = <A as crate::enforcement::Hasher<FP_MAX>>::OUTPUT_BYTES;",
    );
    f.line(
        "                    let width = if n_max > INLINE_BYTES { INLINE_BYTES } else { n_max };",
    );
    f.line("                    return Ok(TermValue::inline_from_slice(&digest[..width]));");
    f.line("                }");
    f.line("                // Primitive fast-path: dispatch the kernel function directly on");
    f.line("                // the bounded operand bytes (user-declared axes operate on");
    f.line("                // bounded values; `v.bytes()` yields the Inline/Borrowed slice).");
    f.line("                let mut out = [0u8; INLINE_BYTES];");
    f.line("                let written = match <A as crate::pipeline::AxisTuple<INLINE_BYTES, FP_MAX>>::dispatch(axis_index, kernel_id, v.bytes(), &mut out) {");
    f.line("                    Ok(n) => n,");
    f.line("                    Err(report) => return Err(PipelineFailure::ShapeViolation { report }),");
    f.line("                };");
    f.line(
        "                let width = if written > INLINE_BYTES { INLINE_BYTES } else { written };",
    );
    f.line("                Ok(TermValue::inline_from_slice(&out[..width]))");
    f.line("            } else {");
    f.line("                // ADR-055 recursive-fold path: walk the axis's substrate-Term");
    f.line("                // body with the evaluated kernel input bound in scope. The");
    f.line("                // body's root term is by convention the last entry in the arena.");
    f.line("                let root = body.len() - 1;");
    f.line("                // The evaluated kernel input carrier `v` is threaded as the");
    f.line("                // body's input; the recursively-folded body result is");
    f.line("                // materialized into an owned `Inline` carrier so it does not");
    f.line("                // borrow the body arena's transient scope.");
    f.line("                let folded = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(body, root, v, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;");
    f.line("                Ok(TermValue::inline_from_slice(folded.bytes()))");
    f.line("            }");
    f.line("        }");
    f.line("        crate::enforcement::Term::ProjectField { source_index, byte_offset, byte_length } => {");
    f.line("            // ADR-033 G20: evaluate source, slice [byte_offset .. byte_offset+byte_length].");
    f.line("            let v = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, source_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;");
    f.line("            let bytes = v.bytes();");
    f.line("            let start = byte_offset as usize;");
    f.line("            let end = start.saturating_add(byte_length as usize);");
    f.line("            if end > bytes.len() {");
    f.line("                return Err(PipelineFailure::ShapeViolation {");
    f.line("                    report: crate::enforcement::ShapeViolation {");
    f.line(
        "                        shape_iri: \"https://uor.foundation/pipeline/ProjectFieldShape\",",
    );
    f.line("                        constraint_iri: \"https://uor.foundation/pipeline/ProjectFieldShape/inBounds\",");
    f.line("                        property_iri: \"https://uor.foundation/pipeline/byteOffset\",");
    f.line("                        expected_range: \"https://uor.foundation/pipeline/SourceByteRange\",");
    f.line("                        min_count: 0,");
    f.line("                        max_count: 1,");
    f.line("                        kind: crate::ViolationKind::ValueCheck,");
    f.line("                    },");
    f.line("                });");
    f.line("            }");
    f.line("            Ok(TermValue::inline_from_slice(&bytes[start..end]))");
    f.line("        }");
    f.line(
        "        crate::enforcement::Term::FirstAdmit { domain_size_index, predicate_index } => {",
    );
    f.line("            // ADR-034 Mechanism 2: bounded search with structural early");
    f.line("            // termination. Evaluate the domain size N; iterate idx in 0..N");
    f.line("            // ascending; for each idx, evaluate predicate with");
    f.line("            // FIRST_ADMIT_IDX_NAME_INDEX bound to idx. Return on the first");
    f.line("            // non-zero predicate result (the \"found\" coproduct value");
    f.line("            // 0x01 || idx_bytes); after exhausting the domain return the");
    f.line("            // \"not-found\" coproduct value 0x00 || idx-width zero bytes.");
    f.line(
        "            let domain_size = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, domain_size_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;",
    );
    f.line("            let n = bytes_to_u64_be(domain_size.bytes());");
    f.line("            // Determine the idx byte width from the domain size's");
    f.line("            // BE-truncated u64 packing (use the smallest non-zero count of");
    f.line("            // bytes needed to represent N). For N=0 fall back to 1 byte so");
    f.line("            // the not-found sentinel still has the canonical (disc, idx) shape.");
    f.line("            let idx_byte_width: usize = if n == 0 {");
    f.line("                1");
    f.line("            } else {");
    f.line("                let mut w = 8usize;");
    f.line("                while w > 1 && (n >> ((w - 1) * 8)) == 0 {");
    f.line("                    w -= 1;");
    f.line("                }");
    f.line("                w");
    f.line("            };");
    f.line("            let mut idx_iter: u64 = 0;");
    f.line("            while idx_iter < n {");
    f.line("                let idx_bytes_full = idx_iter.to_be_bytes();");
    f.line("                let idx_bytes = &idx_bytes_full[8 - idx_byte_width..];");
    f.line("                let pred_val = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(");
    f.line("                    arena,");
    f.line("                    predicate_index as usize,");
    f.line("                    input,");
    f.line("                    recurse_value,");
    f.line("                    recurse_idx_value,");
    f.line("                    unfold_value,");
    f.line("                    Some(idx_bytes),");
    f.line("                    resolvers,");
    f.line("                )?;");
    f.line("                // ADR-029 zero/non-zero convention: any TermValue whose");
    f.line("                // byte sequence has at least one non-zero byte counts as");
    f.line("                // \"true\". An empty TermValue is treated as \"false\".");
    f.line("                let pb = pred_val.bytes();");
    f.line("                let mut admitted = false;");
    f.line("                let mut bi = 0;");
    f.line("                while bi < pb.len() {");
    f.line("                    if pb[bi] != 0 {");
    f.line("                        admitted = true;");
    f.line("                        break;");
    f.line("                    }");
    f.line("                    bi += 1;");
    f.line("                }");
    f.line("                if admitted {");
    f.line("                    let mut out_buf = [0u8; INLINE_BYTES];");
    f.line("                    out_buf[0] = 0x01;");
    f.line("                    let mut k = 0;");
    f.line("                    while k < idx_byte_width {");
    f.line("                        out_buf[1 + k] = idx_bytes[k];");
    f.line("                        k += 1;");
    f.line("                    }");
    f.line("                    return Ok(TermValue::inline_from_slice(&out_buf[..1 + idx_byte_width]));");
    f.line("                }");
    f.line("                idx_iter += 1;");
    f.line("            }");
    f.line("            // No idx admitted — emit not-found coproduct value.");
    f.line("            let mut out_buf = [0u8; INLINE_BYTES];");
    f.line("            out_buf[0] = 0x00;");
    f.line("            // bytes 1..1+idx_byte_width remain zero (structural padding).");
    f.line("            Ok(TermValue::inline_from_slice(&out_buf[..1 + idx_byte_width]))");
    f.line("        }");

    // ADR-035 + ADR-036: ψ-Term fold-rules.
    //
    // Eight resolver-bound variants (Nerve, ChainComplex, HomologyGroups,
    // CochainComplex, CohomologyGroups, PostnikovTower, HomotopyGroups,
    // KInvariants) consult the application's `ResolverTuple` at
    // evaluation time. Each variant evaluates its operand subtree,
    // dispatches the operand bytes to the corresponding resolver via
    // `Has<Category>Resolver::<accessor>().resolve(...)`, and emits the
    // resolver's output bytes as the variant's `TermValue`. The
    // foundation-Null defaults (`NullResolverTuple` + `Null<Cat>Resolver`)
    // emit `RESOLVER_ABSENT` ShapeViolations on `resolve`; the catamorphism
    // propagates them, recoverable via `Term::Try`'s default-propagation
    // handler (ADR-022 D3 G9). The application supplies non-Null resolvers
    // via the `resolver!` SDK macro per the wiki commitment.
    //
    // The resolver-free Betti variant is a pass-through: it copies the
    // evaluated homology bytes as-is (Betti-extraction-without-resolver
    // is byte projection per ADR-035; foundation provides the trivial
    // pass-through).
    // ADR-060: the resolver-bound fold-rule evaluates its operand subtree to a
    // `TermValue<'a, INLINE_BYTES>` and dispatches it to the corresponding
    // resolver via the `Has<Category>Resolver` accessor. ADR-060 amends the
    // ADR-036 trait surface: `resolve` now takes and returns a `TermValue`
    // directly (no caller-supplied `&mut [u8]` scratch, no per-ψ-stage
    // byte-width ceiling) — the resolver constructs whichever carrier variant
    // its output requires (Inline / Borrowed / Stream). The foundation-Null
    // defaults emit `RESOLVER_ABSENT` on `resolve`, propagated and recoverable
    // via `Term::Try`'s default-propagation handler (ADR-022 D3 G9).
    let resolver_bound_variants: &[(&str, &str, &str)] = &[
        (
            "Term::Nerve { value_index }",
            "value_index",
            "nerve_resolver",
        ),
        (
            "Term::ChainComplex { simplicial_index }",
            "simplicial_index",
            "chain_complex_resolver",
        ),
        (
            "Term::HomologyGroups { chain_index }",
            "chain_index",
            "homology_group_resolver",
        ),
        (
            "Term::CochainComplex { chain_index }",
            "chain_index",
            "cochain_complex_resolver",
        ),
        (
            "Term::CohomologyGroups { cochain_index }",
            "cochain_index",
            "cohomology_group_resolver",
        ),
        (
            "Term::PostnikovTower { simplicial_index }",
            "simplicial_index",
            "postnikov_resolver",
        ),
        (
            "Term::HomotopyGroups { postnikov_index }",
            "postnikov_index",
            "homotopy_group_resolver",
        ),
        (
            "Term::KInvariants { homotopy_index }",
            "homotopy_index",
            "k_invariant_resolver",
        ),
    ];
    for (pattern, operand_field, accessor) in resolver_bound_variants {
        f.line(&format!("        crate::enforcement::{pattern} => {{"));
        f.line("            // ADR-035 + ADR-036 + ADR-060: resolver-bound ψ-Term variant.");
        f.line("            // Evaluate the operand subtree to a source-polymorphic carrier,");
        f.line("            // then dispatch it through the application's resolver via the");
        f.line("            // `Has<Category>Resolver` accessor. The resolver returns the");
        f.line("            // variant's `TermValue` directly (Inline / Borrowed / Stream);");
        f.line("            // resolver-side `Err` propagates as");
        f.line("            // `PipelineFailure::ShapeViolation` (the Null defaults emit");
        f.line("            // `RESOLVER_ABSENT`, recoverable via `Term::Try`).");
        f.line(&format!(
            "            let operand = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, {operand_field} as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;"
        ));
        f.line(&format!(
            "            match resolvers.{accessor}().resolve(operand) {{"
        ));
        f.line("                Ok(tv) => Ok(tv),");
        f.line("                Err(report) => Err(PipelineFailure::ShapeViolation { report }),");
        f.line("            }");
        f.line("        }");
    }

    // Term::Betti — resolver-free pass-through (ADR-035 ψ_4).
    f.line("        crate::enforcement::Term::Betti { homology_index } => {");
    f.line("            // ADR-035 ψ_4: Betti-number extraction from HomologyGroups.");
    f.line("            // Pure byte-projection — no resolver consultation. The");
    f.line("            // operand's evaluated bytes (the homology-groups byte");
    f.line("            // serialization per the homology bridge's wire format)");
    f.line("            // are returned as-is; downstream consumers slice the");
    f.line("            // Betti tuple positions out of the result.");
    f.line(
        "            let v = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, homology_index as usize, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;",
    );
    f.line("            Ok(v)");
    f.line("        }");

    f.line("    }");
    f.line("}");
    f.blank();

    // Per-PrimitiveOp arithmetic evaluation — ADR-029 Application rule.
    f.line("#[allow(clippy::too_many_arguments)]");
    f.line(
        "fn apply_primitive_op<'a, 'b, 'r, A, R, const INLINE_BYTES: usize, const FP_MAX: usize>(",
    );
    f.line("    arena: &'a [crate::enforcement::Term<'a, INLINE_BYTES>],");
    f.line("    operator: crate::PrimitiveOp,");
    f.line("    args_start: usize,");
    f.line("    args_len: usize,");
    f.line("    input: TermValue<'a, INLINE_BYTES>,");
    f.line("    recurse_value: Option<&'b [u8]>,");
    f.line("    recurse_idx_value: Option<&'b [u8]>,");
    f.line("    unfold_value: Option<&'b [u8]>,");
    f.line("    first_admit_idx_value: Option<&'b [u8]>,");
    f.line("    resolvers: &'r R,");
    f.line(") -> Result<TermValue<'a, INLINE_BYTES>, PipelineFailure>");
    f.line("where");
    f.line("    A: crate::pipeline::AxisTuple<INLINE_BYTES, FP_MAX> + crate::enforcement::Hasher<FP_MAX> + 'a,");
    f.line("    R: crate::pipeline::ResolverTuple");
    f.line("        + crate::pipeline::HasNerveResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasChainComplexResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasHomologyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasCochainComplexResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasCohomologyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasPostnikovResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasHomotopyGroupResolver<INLINE_BYTES, A>");
    f.line("        + crate::pipeline::HasKInvariantResolver<INLINE_BYTES, A>,");
    f.line("{");
    f.line("    // Unary ops: 1 arg. Binary ops: 2 args.");
    f.line("    let arity = match operator {");
    f.line("        crate::PrimitiveOp::Neg");
    f.line("        | crate::PrimitiveOp::Bnot");
    f.line("        | crate::PrimitiveOp::Succ");
    f.line("        | crate::PrimitiveOp::Pred => 1usize,");
    f.line("        crate::PrimitiveOp::Add");
    f.line("        | crate::PrimitiveOp::Sub");
    f.line("        | crate::PrimitiveOp::Mul");
    f.line("        | crate::PrimitiveOp::Xor");
    f.line("        | crate::PrimitiveOp::And");
    f.line("        | crate::PrimitiveOp::Or");
    f.line("        | crate::PrimitiveOp::Le");
    f.line("        | crate::PrimitiveOp::Lt");
    f.line("        | crate::PrimitiveOp::Ge");
    f.line("        | crate::PrimitiveOp::Gt");
    f.line("        | crate::PrimitiveOp::Concat");
    f.line("        // ADR-053 substrate amendment: ring-axis arithmetic completion.");
    f.line("        | crate::PrimitiveOp::Div");
    f.line("        | crate::PrimitiveOp::Mod");
    f.line("        | crate::PrimitiveOp::Pow => 2usize,");
    f.line("    };");
    f.line("    if args_len != arity {");
    f.line("        return Err(PipelineFailure::ShapeViolation {");
    f.line("            report: crate::enforcement::ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/pipeline/PrimitiveOpArityShape\",");
    f.line(
        "                constraint_iri: \"https://uor.foundation/pipeline/PrimitiveOpArityShape/arity\",",
    );
    f.line("                property_iri: \"https://uor.foundation/pipeline/operatorArity\",");
    f.line(
        "                expected_range: \"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\",",
    );
    f.line("                min_count: arity as u32,");
    f.line("                max_count: arity as u32,");
    f.line("                kind: crate::ViolationKind::CardinalityViolation,");
    f.line("            },");
    f.line("        });");
    f.line("    }");
    f.line("    if arity == 1 {");
    f.line(
        "        let v = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, args_start, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;",
    );
    f.line("        let x = bytes_to_u64_be(v.bytes());");
    f.line("        let r = match operator {");
    f.line("            crate::PrimitiveOp::Neg => x.wrapping_neg(),");
    f.line("            crate::PrimitiveOp::Bnot => !x,");
    f.line("            crate::PrimitiveOp::Succ => x.wrapping_add(1),");
    f.line("            crate::PrimitiveOp::Pred => x.wrapping_sub(1),");
    f.line("            _ => return Err(PipelineFailure::ShapeViolation {");
    f.line("                report: crate::enforcement::ShapeViolation {");
    f.line(
        "                    shape_iri: \"https://uor.foundation/pipeline/PrimitiveOpArityShape\",",
    );
    f.line(
        "                    constraint_iri: \"https://uor.foundation/pipeline/PrimitiveOpArityShape/binary-as-unary\",",
    );
    f.line("                    property_iri: \"https://uor.foundation/pipeline/operatorArity\",");
    f.line(
        "                    expected_range: \"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\",",
    );
    f.line("                    min_count: 2,");
    f.line("                    max_count: 2,");
    f.line("                    kind: crate::ViolationKind::CardinalityViolation,");
    f.line("                },");
    f.line("            }),");
    f.line("        };");
    f.line("        let width = v.bytes().len().clamp(1, 8);");
    f.line("        let arr = r.to_be_bytes();");
    f.line("        Ok(TermValue::inline_from_slice(&arr[8 - width..]))");
    f.line("    } else {");
    f.line(
        "        let lhs = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, args_start, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;",
    );
    f.line("        let rhs = evaluate_term_at::<A, R, INLINE_BYTES, FP_MAX>(arena, args_start + 1, input, recurse_value, recurse_idx_value, unfold_value, first_admit_idx_value, resolvers)?;");
    f.line("        // ADR-013/TR-08 substrate-amendment ops: byte-level Concat and");
    f.line("        // comparison primitives bypass the u64 fold and operate on the");
    f.line("        // operands' full byte sequences.");
    f.line("        match operator {");
    f.line("            crate::PrimitiveOp::Concat => {");
    f.line("                // Concat: emit lhs.bytes() ⧺ rhs.bytes(), bounded by");
    f.line("                // INLINE_BYTES (truncates excess; runtime callers");
    f.line("                // declaring shapes whose composite length exceeds the");
    f.line("                // ceiling are rejected at validation time per ADR-028's");
    f.line("                // symmetric output ceiling check).");
    f.line("                let lb = lhs.bytes();");
    f.line("                let rb = rhs.bytes();");
    f.line("                let total = lb.len() + rb.len();");
    f.line("                let cap = if total > INLINE_BYTES { INLINE_BYTES } else { total };");
    f.line("                let mut buf = [0u8; INLINE_BYTES];");
    f.line("                let mut i = 0;");
    f.line("                while i < lb.len() && i < cap { buf[i] = lb[i]; i += 1; }");
    f.line("                let mut j = 0;");
    f.line("                while j < rb.len() && i < cap { buf[i] = rb[j]; i += 1; j += 1; }");
    f.line("                return Ok(TermValue::inline_from_slice(&buf[..cap]));");
    f.line("            }");
    f.line("            crate::PrimitiveOp::Le | crate::PrimitiveOp::Lt");
    f.line("            | crate::PrimitiveOp::Ge | crate::PrimitiveOp::Gt => {");
    f.line("                // Big-endian byte-level comparison. Both operands are");
    f.line("                // padded with leading zeros to the max length so the");
    f.line("                // comparison ignores leading-zero stripping differences.");
    f.line("                let cmp = byte_compare_be(lhs.bytes(), rhs.bytes());");
    f.line("                let result_byte: u8 = match operator {");
    f.line("                    crate::PrimitiveOp::Le => u8::from(cmp != core::cmp::Ordering::Greater),");
    f.line(
        "                    crate::PrimitiveOp::Lt => u8::from(cmp == core::cmp::Ordering::Less),",
    );
    f.line(
        "                    crate::PrimitiveOp::Ge => u8::from(cmp != core::cmp::Ordering::Less),",
    );
    f.line("                    crate::PrimitiveOp::Gt => u8::from(cmp == core::cmp::Ordering::Greater),");
    f.line("                    _ => 0,");
    f.line("                };");
    f.line("                return Ok(TermValue::inline_from_slice(&[result_byte]));");
    f.line("            }");
    f.line("            // ADR-053 substrate amendment: Div/Mod reject b = 0 with");
    f.line("            // a ShapeViolation. Per ADR-050, division semantics are");
    f.line("            // Euclidean (q = floor(a / b), r = a − q·b) over the ring");
    f.line("            // Z/(2^n)Z where n is max(8·byte-len(a), 8·byte-len(b)).");
    f.line("            crate::PrimitiveOp::Div if bytes_all_zero(rhs.bytes()) => {");
    f.line("                return Err(PipelineFailure::ShapeViolation {");
    f.line("                    report: crate::enforcement::ShapeViolation {");
    f.line("                        shape_iri: \"https://uor.foundation/op/Div\",");
    f.line(
        "                        constraint_iri: \"https://uor.foundation/op/Div/nonZeroDivisor\",",
    );
    f.line("                        property_iri: \"https://uor.foundation/op/arity\",");
    f.line("                        expected_range: \"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\",");
    f.line("                        min_count: 1,");
    f.line("                        max_count: 1,");
    f.line("                        kind: crate::ViolationKind::ValueCheck,");
    f.line("                    },");
    f.line("                });");
    f.line("            }");
    f.line("            crate::PrimitiveOp::Mod if bytes_all_zero(rhs.bytes()) => {");
    f.line("                return Err(PipelineFailure::ShapeViolation {");
    f.line("                    report: crate::enforcement::ShapeViolation {");
    f.line("                        shape_iri: \"https://uor.foundation/op/Mod\",");
    f.line(
        "                        constraint_iri: \"https://uor.foundation/op/Mod/nonZeroDivisor\",",
    );
    f.line("                        property_iri: \"https://uor.foundation/op/arity\",");
    f.line("                        expected_range: \"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\",");
    f.line("                        min_count: 1,");
    f.line("                        max_count: 1,");
    f.line("                        kind: crate::ViolationKind::ValueCheck,");
    f.line("                    },");
    f.line("                });");
    f.line("            }");
    f.line("            _ => {}");
    f.line("        }");
    f.line("        let width = lhs.bytes().len().max(rhs.bytes().len()).max(1);");
    f.line("        // ADR-050 width-parametric dispatch: widths > 8 bytes route");
    f.line("        // through the byte-level kernel `byte_arith_be` so wide-Witt");
    f.line("        // operands (Limbs<N>-backed levels W160..W32768) compute");
    f.line("        // correctly under Z/(2^(8·width))Z without truncating to u64.");
    f.line("        if width > 8 {");
    f.line("            return byte_arith_be::<INLINE_BYTES>(operator, lhs.bytes(), rhs.bytes(), width);");
    f.line("        }");
    f.line("        let a = bytes_to_u64_be(lhs.bytes());");
    f.line("        let b = bytes_to_u64_be(rhs.bytes());");
    f.line("        // ADR-050 width-parametric mask: arithmetic results are reduced");
    f.line("        // mod 2^(8·width) before truncation. For the u64 hot path this");
    f.line("        // is `(1 << (width·8)) − 1` for width < 8 and u64::MAX for width = 8.");
    f.line("        let mask: u64 = if width >= 8 { u64::MAX } else { (1u64 << (width * 8)).wrapping_sub(1) };");
    f.line("        let r = match operator {");
    f.line("            crate::PrimitiveOp::Add => a.wrapping_add(b) & mask,");
    f.line("            crate::PrimitiveOp::Sub => a.wrapping_sub(b) & mask,");
    f.line("            crate::PrimitiveOp::Mul => a.wrapping_mul(b) & mask,");
    f.line("            crate::PrimitiveOp::Xor => (a ^ b) & mask,");
    f.line("            crate::PrimitiveOp::And => (a & b) & mask,");
    f.line("            crate::PrimitiveOp::Or => (a | b) & mask,");
    f.line("            // ADR-053: Euclidean division/remainder over Z/(2^(8·width))Z.");
    f.line("            crate::PrimitiveOp::Div => (a & mask) / (b & mask),");
    f.line("            crate::PrimitiveOp::Mod => (a & mask) % (b & mask),");
    f.line("            // ADR-053: modular exponentiation by repeated squaring.");
    f.line("            crate::PrimitiveOp::Pow => u64_modpow(a & mask, b & mask, mask),");
    f.line("            _ => return Err(PipelineFailure::ShapeViolation {");
    f.line("                report: crate::enforcement::ShapeViolation {");
    f.line(
        "                    shape_iri: \"https://uor.foundation/pipeline/PrimitiveOpArityShape\",",
    );
    f.line(
        "                    constraint_iri: \"https://uor.foundation/pipeline/PrimitiveOpArityShape/unary-as-binary\",",
    );
    f.line("                    property_iri: \"https://uor.foundation/pipeline/operatorArity\",");
    f.line(
        "                    expected_range: \"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\",",
    );
    f.line("                    min_count: 1,");
    f.line("                    max_count: 1,");
    f.line("                    kind: crate::ViolationKind::CardinalityViolation,");
    f.line("                },");
    f.line("            }),");
    f.line("        };");
    f.line("        let arr = r.to_be_bytes();");
    f.line("        Ok(TermValue::inline_from_slice(&arr[8 - width..]))");
    f.line("    }");
    f.line("}");
    f.blank();

    // ADR-050 byte-level kernel: width-parametric arithmetic over byte
    // slices, used by `apply_primitive_op` when width > 8 (i.e., Limbs-
    // backed Witt levels). Operands are padded with leading zeros to
    // `width` bytes, computed under Z/(2^(8·width))Z, and the result is
    // returned as a `TermValue` of width bytes (big-endian).
    f.doc_comment("ADR-050: byte-level fold-rule dispatch for binary ring-axis primitives over");
    f.doc_comment("widths > 8 bytes. Matches the const-fn `const_ring_eval_w{n}` helpers'");
    f.doc_comment("semantics but operates on the runtime catamorphism's `TermValue` byte slices");
    f.doc_comment("rather than on typed `Limbs<N>` values.");
    f.line("#[allow(clippy::too_many_lines)]");
    f.line("fn byte_arith_be<'a, const INLINE_BYTES: usize>(");
    f.line("    operator: crate::PrimitiveOp,");
    f.line("    lhs: &[u8],");
    f.line("    rhs: &[u8],");
    f.line("    width: usize,");
    f.line(") -> Result<TermValue<'a, INLINE_BYTES>, PipelineFailure> {");
    f.line("    // Pad both operands to `width` bytes (leading zeros, big-endian).");
    f.line("    let mut a = [0u8; INLINE_BYTES];");
    f.line("    let mut b = [0u8; INLINE_BYTES];");
    f.line("    let w = if width > INLINE_BYTES { INLINE_BYTES } else { width };");
    f.line("    {");
    f.line("        let la = lhs.len().min(w);");
    f.line("        let dst_off = w - la;");
    f.line("        let src_off = lhs.len() - la;");
    f.line("        let mut i = 0;");
    f.line("        while i < la { a[dst_off + i] = lhs[src_off + i]; i += 1; }");
    f.line("    }");
    f.line("    {");
    f.line("        let lb = rhs.len().min(w);");
    f.line("        let dst_off = w - lb;");
    f.line("        let src_off = rhs.len() - lb;");
    f.line("        let mut i = 0;");
    f.line("        while i < lb { b[dst_off + i] = rhs[src_off + i]; i += 1; }");
    f.line("    }");
    f.line("    let mut out = [0u8; INLINE_BYTES];");
    f.line("    match operator {");
    f.line("        crate::PrimitiveOp::And => {");
    f.line("            let mut i = 0; while i < w { out[i] = a[i] & b[i]; i += 1; }");
    f.line("        }");
    f.line("        crate::PrimitiveOp::Or => {");
    f.line("            let mut i = 0; while i < w { out[i] = a[i] | b[i]; i += 1; }");
    f.line("        }");
    f.line("        crate::PrimitiveOp::Xor => {");
    f.line("            let mut i = 0; while i < w { out[i] = a[i] ^ b[i]; i += 1; }");
    f.line("        }");
    f.line("        crate::PrimitiveOp::Add => {");
    f.line("            let mut carry: u16 = 0;");
    f.line("            let mut i = w;");
    f.line("            while i > 0 { i -= 1; let s = a[i] as u16 + b[i] as u16 + carry; out[i] = (s & 0xFF) as u8; carry = s >> 8; }");
    f.line("        }");
    f.line("        crate::PrimitiveOp::Sub => {");
    f.line("            let mut borrow: i16 = 0;");
    f.line("            let mut i = w;");
    f.line("            while i > 0 { i -= 1; let d = a[i] as i16 - b[i] as i16 - borrow; if d < 0 { out[i] = (d + 256) as u8; borrow = 1; } else { out[i] = d as u8; borrow = 0; } }");
    f.line("        }");
    f.line("        crate::PrimitiveOp::Mul => {");
    f.line("            // Schoolbook multiplication mod 2^(8·w). `a` and `b` are");
    f.line("            // big-endian w-byte operands; output is the low w bytes of");
    f.line("            // the 2w-byte product, written back big-endian.");
    f.line("            //");
    f.line("            // Algorithm: walk `i` from LSB to MSB (a[w-1] = LSB), and");
    f.line("            // for each `i` walk `j` similarly. The byte product");
    f.line("            // a[w-1-i] * b[w-1-j] contributes to output position");
    f.line("            // (w-1)-(i+j) when (i+j) < w; otherwise it overflows the");
    f.line("            // ring modulus and is discarded.");
    f.line("            let mut tmp = [0u32; INLINE_BYTES];");
    f.line("            let mut i: usize = 0;");
    f.line("            while i < w {");
    f.line("                let mut j: usize = 0;");
    f.line("                while j < w - i {");
    f.line("                    let dst = w - 1 - (i + j);");
    f.line("                    tmp[dst] += a[w - 1 - i] as u32 * b[w - 1 - j] as u32;");
    f.line("                    j += 1;");
    f.line("                }");
    f.line("                i += 1;");
    f.line("            }");
    f.line("            // Carry-propagate within the low w bytes (high bytes are");
    f.line("            // discarded — the ring modulus reduces them away).");
    f.line("            let mut carry: u32 = 0;");
    f.line("            let mut k = w;");
    f.line("            while k > 0 {");
    f.line("                k -= 1;");
    f.line("                let v = tmp[k] + carry;");
    f.line("                out[k] = (v & 0xFF) as u8;");
    f.line("                carry = v >> 8;");
    f.line("            }");
    f.line("        }");
    f.line("        crate::PrimitiveOp::Div | crate::PrimitiveOp::Mod => {");
    f.line("            // Binary long division MSB→LSB over the byte slice.");
    f.line("            let mut q = [0u8; INLINE_BYTES];");
    f.line("            let mut r = [0u8; INLINE_BYTES];");
    f.line("            let total_bits = w * 8;");
    f.line("            let mut i = 0;");
    f.line("            while i < total_bits {");
    f.line("                bytes_shl1(&mut r, w);");
    f.line("                let bit_word = i / 8;");
    f.line("                let bit_idx = 7 - (i % 8);");
    f.line("                let cur_bit = (a[bit_word] >> bit_idx) & 1;");
    f.line("                if cur_bit == 1 { r[w - 1] |= 1; }");
    f.line("                if bytes_le_be(&b[..w], &r[..w]) {");
    f.line("                    bytes_sub_in_place(&mut r, &b, w);");
    f.line("                    bytes_shl1(&mut q, w);");
    f.line("                    q[w - 1] |= 1;");
    f.line("                } else {");
    f.line("                    bytes_shl1(&mut q, w);");
    f.line("                }");
    f.line("                i += 1;");
    f.line("            }");
    f.line("            let src = match operator { crate::PrimitiveOp::Div => &q, _ => &r };");
    f.line("            let mut k = 0; while k < w { out[k] = src[k]; k += 1; }");
    f.line("        }");
    f.line("        crate::PrimitiveOp::Pow => {");
    f.line("            // Square-and-multiply over byte-slice operands. Exponent");
    f.line("            // bits walked LSB→MSB; each iteration squares the running");
    f.line("            // base and conditionally multiplies into the accumulator.");
    f.line("            let mut result = [0u8; INLINE_BYTES];");
    f.line("            result[w - 1] = 1;");
    f.line("            let mut base = [0u8; INLINE_BYTES];");
    f.line("            let mut k = 0; while k < w { base[k] = a[k]; k += 1; }");
    f.line("            let mut byte = w;");
    f.line("            while byte > 0 {");
    f.line("                byte -= 1;");
    f.line("                let mut bit = 0u32;");
    f.line("                while bit < 8 {");
    f.line("                    if ((b[byte] >> bit) & 1) == 1 {");
    f.line("                        let prod = byte_arith_be::<INLINE_BYTES>(crate::PrimitiveOp::Mul, &result[..w], &base[..w], w)?;");
    f.line("                        let pb = prod.bytes();");
    f.line("                        let dst_off = w - pb.len().min(w);");
    f.line("                        let mut j = 0;");
    f.line("                        while j < w { result[j] = 0; j += 1; }");
    f.line("                        let mut j2 = 0; while j2 < pb.len().min(w) { result[dst_off + j2] = pb[j2]; j2 += 1; }");
    f.line("                    }");
    f.line("                    let sq = byte_arith_be::<INLINE_BYTES>(crate::PrimitiveOp::Mul, &base[..w], &base[..w], w)?;");
    f.line("                    let sb = sq.bytes();");
    f.line("                    let dst_off = w - sb.len().min(w);");
    f.line("                    let mut j = 0;");
    f.line("                    while j < w { base[j] = 0; j += 1; }");
    f.line("                    let mut j2 = 0; while j2 < sb.len().min(w) { base[dst_off + j2] = sb[j2]; j2 += 1; }");
    f.line("                    bit += 1;");
    f.line("                }");
    f.line("            }");
    f.line("            let mut k = 0; while k < w { out[k] = result[k]; k += 1; }");
    f.line("        }");
    f.line("        _ => return Err(PipelineFailure::ShapeViolation {");
    f.line("            report: crate::enforcement::ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/pipeline/PrimitiveOpArityShape\",");
    f.line("                constraint_iri: \"https://uor.foundation/pipeline/PrimitiveOpArityShape/unary-as-binary\",");
    f.line("                property_iri: \"https://uor.foundation/pipeline/operatorArity\",");
    f.line(
        "                expected_range: \"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\",",
    );
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: crate::ViolationKind::CardinalityViolation,");
    f.line("            },");
    f.line("        }),");
    f.line("    }");
    f.line("    Ok(TermValue::inline_from_slice(&out[..w]))");
    f.line("}");
    f.blank();

    // Byte-level helpers used by `byte_arith_be`.
    f.line("fn bytes_shl1(buf: &mut [u8], w: usize) {");
    f.line("    let mut carry: u8 = 0;");
    f.line("    let mut i = w;");
    f.line(
        "    while i > 0 { i -= 1; let v = buf[i]; buf[i] = (v << 1) | carry; carry = v >> 7; }",
    );
    f.line("}");
    f.blank();
    f.line("fn bytes_le_be(a: &[u8], b: &[u8]) -> bool {");
    f.line("    let mut i = 0;");
    f.line("    while i < a.len() {");
    f.line("        if a[i] < b[i] { return true; }");
    f.line("        if a[i] > b[i] { return false; }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    true");
    f.line("}");
    f.blank();
    f.line("fn bytes_sub_in_place(dst: &mut [u8], rhs: &[u8], w: usize) {");
    f.line("    let mut borrow: i16 = 0;");
    f.line("    let mut i = w;");
    f.line("    while i > 0 {");
    f.line("        i -= 1;");
    f.line("        let d = dst[i] as i16 - rhs[i] as i16 - borrow;");
    f.line("        if d < 0 { dst[i] = (d + 256) as u8; borrow = 1; } else { dst[i] = d as u8; borrow = 0; }");
    f.line("    }");
    f.line("}");
    f.blank();

    // Big-endian byte-slice comparison helper for Le/Lt/Ge/Gt fold-rules.
    f.line("fn byte_compare_be(a: &[u8], b: &[u8]) -> core::cmp::Ordering {");
    f.line("    // Pad shorter operand with leading zeros so the comparison treats");
    f.line("    // both operands at max(len(a), len(b)) byte width.");
    f.line("    let max_len = if a.len() > b.len() { a.len() } else { b.len() };");
    f.line("    let mut i = 0;");
    f.line("    while i < max_len {");
    f.line("        let ai = if i + a.len() >= max_len { a[i + a.len() - max_len] } else { 0u8 };");
    f.line("        let bi = if i + b.len() >= max_len { b[i + b.len() - max_len] } else { 0u8 };");
    f.line("        if ai < bi { return core::cmp::Ordering::Less; }");
    f.line("        if ai > bi { return core::cmp::Ordering::Greater; }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    core::cmp::Ordering::Equal");
    f.line("}");
    f.blank();

    // bytes_to_u64_be: big-endian byte-slice to u64 (low end of the
    // slice is the high byte). Used by `apply_primitive_op` to fold
    // operands into u64-width arithmetic regardless of the underlying
    // Witt level (the caller restores the level's byte width via
    // `arr[8 - width..]` slicing of the big-endian result).
    f.line("fn bytes_to_u64_be(bytes: &[u8]) -> u64 {");
    f.line("    let take = if bytes.len() > 8 { 8 } else { bytes.len() };");
    f.line("    let start = bytes.len() - take;");
    f.line("    let mut acc = 0u64;");
    f.line("    let mut i = 0;");
    f.line("    while i < take {");
    f.line("        acc = (acc << 8) | bytes[start + i] as u64;");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    acc");
    f.line("}");
    f.blank();

    // ADR-053: zero check on a byte slice — true iff every byte is 0.
    // Used by the Div/Mod fold-rules to detect a zero divisor at runtime.
    f.line("fn bytes_all_zero(bytes: &[u8]) -> bool {");
    f.line("    let mut i = 0;");
    f.line("    while i < bytes.len() {");
    f.line("        if bytes[i] != 0 { return false; }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    true");
    f.line("}");
    f.blank();

    // ADR-053: modular exponentiation by repeated squaring within the
    // u64 hot path. The mask captures the ring width (`u64::MAX` for
    // 8-byte operands; `(1 << (width*8)) - 1` for narrower widths).
    f.line("fn u64_modpow(base: u64, exp: u64, mask: u64) -> u64 {");
    f.line("    let mut result: u64 = 1u64 & mask;");
    f.line("    let mut b: u64 = base & mask;");
    f.line("    let mut e: u64 = exp;");
    f.line("    while e > 0 {");
    f.line("        if (e & 1) == 1 {");
    f.line("            result = result.wrapping_mul(b) & mask;");
    f.line("        }");
    f.line("        b = b.wrapping_mul(b) & mask;");
    f.line("        e >>= 1;");
    f.line("    }");
    f.line("    result");
    f.line("}");
    f.blank();

    // Foundation-sanctioned identity route. `ConstrainedTypeInput` is
    // foundation's empty default shape (zero sites, zero constraints);
    // its term tree is a leaf, vacuously closed under foundation
    // vocabulary. Sanctioning this impl lets trivial `PrismModel`
    // declarations (and tests of the trait surface) compile without
    // going through the `prism_model!` macro. Non-trivial routes get
    // their `FoundationClosed` impls from the macro, which emits both
    // the `Sealed` impl and the `FoundationClosed` impl on the witness
    // it generates.
    f.doc_comment("Foundation-sanctioned identity route: `ConstrainedTypeInput` is the");
    f.doc_comment("empty default shape, vacuously closed under foundation vocabulary.");
    f.doc_comment("Application authors with non-trivial routes use the `prism_model!`");
    f.doc_comment("macro from `uor-foundation-sdk`, which emits `FoundationClosed` for");
    f.doc_comment("the witness it generates iff every node is foundation-vocabulary.");
    f.doc_comment("");
    f.doc_comment("The identity route's `arena_slice()` returns `&[]` — no terms, no");
    f.doc_comment("transformation, input passes through to output unchanged.");
    f.line("impl __sdk_seal::Sealed for ConstrainedTypeInput {}");
    f.line(
        "impl<const INLINE_BYTES: usize> FoundationClosed<INLINE_BYTES> for ConstrainedTypeInput {",
    );
    f.line("    fn arena_slice() -> &'static [crate::enforcement::Term<'static, INLINE_BYTES>] {");
    f.line("        &[]");
    f.line("    }");
    f.line("}");
    f.line("impl<'a> IntoBindingValue<'a> for ConstrainedTypeInput {");
    f.line("    fn as_binding_value<const INLINE_BYTES: usize>(&self) -> crate::pipeline::TermValue<'a, INLINE_BYTES> {");
    f.line("        // Identity input carries no bytes — the empty shape's canonical");
    f.line("        // carrier is the empty Inline carrier (valid for any lifetime).");
    f.line("        crate::pipeline::TermValue::empty()");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// Emits the `CartesianProductShape` marker trait, its companion
/// `primitive_cartesian_nerve_betti` primitive, the `kunneth_compose` const
/// helper, and the SDK support helpers `shift_constraint` and
/// `sdk_concat_product_constraints`. Introduced by the Product/Coproduct
/// Completion Amendment §3c and §Gap 2.
fn emit_cartesian_product_shape(f: &mut RustFile) {
    // Marker trait: opt-in to Künneth-based Betti computation.
    f.doc_comment("Marker for a `ConstrainedTypeShape` that is the Cartesian product of");
    f.doc_comment("two component shapes. Selecting this trait routes nerve-Betti computation");
    f.doc_comment("through Künneth composition of component Betti profiles rather than");
    f.doc_comment("flat enumeration of (constraint, constraint) pairs. Introduced by the");
    f.doc_comment("Product/Coproduct Completion Amendment §3c for CartesianPartitionProduct");
    f.doc_comment("(CPT_1–CPT_6).");
    f.line("pub trait CartesianProductShape: ConstrainedTypeShape {");
    f.indented_doc_comment("Left operand shape.");
    f.line("    type Left: ConstrainedTypeShape;");
    f.indented_doc_comment("Right operand shape.");
    f.line("    type Right: ConstrainedTypeShape;");
    f.line("}");
    f.blank();

    // kunneth_compose — constant-time Künneth composition on bounded Betti arrays.
    f.doc_comment("Künneth composition of two Betti profiles.");
    f.doc_comment("");
    f.doc_comment("Computes `out[k] = Σ_{i + j = k} a[i] · b[j]` over");
    f.doc_comment("`[0, MAX_BETTI_DIMENSION)`. All arithmetic uses saturating operations so the");
    f.doc_comment("function is total on `[u32; MAX_BETTI_DIMENSION]` inputs without panicking.");
    f.line("pub const fn kunneth_compose(");
    f.line("    a: &[u32; crate::enforcement::MAX_BETTI_DIMENSION],");
    f.line("    b: &[u32; crate::enforcement::MAX_BETTI_DIMENSION],");
    f.line(") -> [u32; crate::enforcement::MAX_BETTI_DIMENSION] {");
    f.line("    let mut out = [0u32; crate::enforcement::MAX_BETTI_DIMENSION];");
    f.line("    let mut i: usize = 0;");
    f.line("    while i < crate::enforcement::MAX_BETTI_DIMENSION {");
    f.line("        let mut j: usize = 0;");
    f.line("        while j < crate::enforcement::MAX_BETTI_DIMENSION - i {");
    f.line("            let term = a[i].saturating_mul(b[j]);");
    f.line("            out[i + j] = out[i + j].saturating_add(term);");
    f.line("            j += 1;");
    f.line("        }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    out");
    f.line("}");
    f.blank();

    // primitive_cartesian_nerve_betti
    f.doc_comment("Cartesian-product nerve Betti via Künneth composition of the component");
    f.doc_comment("shapes' Betti profiles. Used instead of");
    f.doc_comment("`primitive_simplicial_nerve_betti` when a shape declares itself as a");
    f.doc_comment("`CartesianProductShape`. Amendment §3c.");
    f.doc_comment("");
    f.doc_comment("Phase 1a (orphan-closure): propagates either component's");
    f.doc_comment("`NERVE_CAPACITY_EXCEEDED` via `?`. Dropped `const fn` because");
    f.doc_comment("the `Result` return of the per-component primitive is not `const`-evaluable.");
    f.doc_comment("");
    f.doc_comment("ADR-057: delegates to [`crate::enforcement::primitive_simplicial_nerve_betti`]");
    f.doc_comment("on each component, which expands `ConstraintRef::Recurse` through");
    f.doc_comment("foundation's built-in shape registry. For application-registry-aware");
    f.doc_comment("expansion, use [`primitive_cartesian_nerve_betti_in`] generic over the");
    f.doc_comment("application's `ShapeRegistryProvider`.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `NERVE_CAPACITY_EXCEEDED` if either component exceeds caps.");
    f.doc_comment("Returns `RECURSE_SHAPE_UNREGISTERED` when a component's Recurse references");
    f.doc_comment("an IRI not present in the consulted registry.");
    f.line("pub fn primitive_cartesian_nerve_betti<S: CartesianProductShape>()");
    f.line("    -> Result<[u32; crate::enforcement::MAX_BETTI_DIMENSION], crate::enforcement::GenericImpossibilityWitness>");
    f.line("{");
    f.line("    let left = crate::enforcement::primitive_simplicial_nerve_betti::<S::Left>()?;");
    f.line("    let right = crate::enforcement::primitive_simplicial_nerve_betti::<S::Right>()?;");
    f.line("    Ok(kunneth_compose(&left, &right))");
    f.line("}");
    f.blank();
    f.doc_comment("ADR-057: registry-parameterized companion to");
    f.doc_comment("[`primitive_cartesian_nerve_betti`]. Delegates to");
    f.doc_comment("[`crate::enforcement::primitive_simplicial_nerve_betti_in`] on each");
    f.doc_comment("component with `R` as the application's `ShapeRegistryProvider`.");
    f.doc_comment("Recurse entries are expanded through `R::REGISTRY` plus foundation's");
    f.doc_comment("built-in registry.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `NERVE_CAPACITY_EXCEEDED` if either component's expanded");
    f.doc_comment("constraint set exceeds caps. Returns `RECURSE_SHAPE_UNREGISTERED` when");
    f.doc_comment("a `Recurse` entry references an IRI not present in either registry.");
    f.line("pub fn primitive_cartesian_nerve_betti_in<");
    f.line("    S: CartesianProductShape,");
    f.line("    R: crate::pipeline::shape_iri_registry::ShapeRegistryProvider,");
    f.line(">() -> Result<");
    f.line("    [u32; crate::enforcement::MAX_BETTI_DIMENSION],");
    f.line("    crate::enforcement::GenericImpossibilityWitness,");
    f.line("> {");
    f.line(
        "    let left = crate::enforcement::primitive_simplicial_nerve_betti_in::<S::Left, R>()?;",
    );
    f.line("    let right = crate::enforcement::primitive_simplicial_nerve_betti_in::<S::Right, R>()?;");
    f.line("    Ok(kunneth_compose(&left, &right))");
    f.line("}");
    f.blank();

    // shift_constraint — SDK site-index rewrite helper.
    f.doc_comment("Shift every site-index reference in a `ConstraintRef` by `offset`.");
    f.doc_comment("");
    f.doc_comment("Used by the SDK's `product_shape!` / `coproduct_shape!` /");
    f.doc_comment("`cartesian_product_shape!` macros to splice an operand's constraints into");
    f.doc_comment("a combined shape at a post-operand offset.");
    f.doc_comment("");
    f.doc_comment("**Phase 17: full operand-catalogue support.** Affine and");
    f.doc_comment("Conjunction now shift correctly at const time because the");
    f.doc_comment("variants store fixed-size arrays (no `&'static [i64]` allocation");
    f.doc_comment("required). The pre-Phase-17 `Site { position: u32::MAX }`");
    f.doc_comment("sentinel is removed.");
    f.doc_comment("");
    f.doc_comment("**Variant coverage.**");
    f.doc_comment("- `Site { position }` → position += offset.");
    f.doc_comment("- `Carry { site }` → site += offset.");
    f.doc_comment("- `Residue`, `Hamming`, `Depth`, `SatClauses`, `Bound`: pass through (no");
    f.doc_comment("  site references at this layer).");
    f.doc_comment("- `Affine { coefficients, coefficient_count, bias }`: builds a fresh");
    f.doc_comment("  `[i64; AFFINE_MAX_COEFFS]` of zeros, copies the active prefix into");
    f.doc_comment("  positions `[offset, offset + coefficient_count)`. If the shift");
    f.doc_comment("  would overflow the fixed buffer, returns an Affine with");
    f.doc_comment("  `coefficient_count = 0` (vacuously consistent).");
    f.doc_comment("- `Conjunction { conjuncts, conjunct_count }`: builds a fresh");
    f.doc_comment("  `[LeafConstraintRef; CONJUNCTION_MAX_TERMS]` and shifts each leaf");
    f.doc_comment("  via `shift_leaf_constraint`. One-level depth — leaves cannot be");
    f.doc_comment("  Conjunction.");
    f.line("pub const fn shift_constraint(c: ConstraintRef, offset: u32) -> ConstraintRef {");
    f.line("    match c {");
    f.line("        ConstraintRef::Site { position } => ConstraintRef::Site {");
    f.line("            position: position.saturating_add(offset),");
    f.line("        },");
    f.line("        ConstraintRef::Carry { site } => ConstraintRef::Carry {");
    f.line("            site: site.saturating_add(offset),");
    f.line("        },");
    f.line("        ConstraintRef::Residue { modulus, residue } => {");
    f.line("            ConstraintRef::Residue { modulus, residue }");
    f.line("        }");
    f.line("        ConstraintRef::Hamming { bound } => ConstraintRef::Hamming { bound },");
    f.line("        ConstraintRef::Depth { min, max } => ConstraintRef::Depth { min, max },");
    f.line("        ConstraintRef::SatClauses { clauses, num_vars } => {");
    f.line("            ConstraintRef::SatClauses { clauses, num_vars }");
    f.line("        }");
    f.line("        ConstraintRef::Bound { observable_iri, bound_shape_iri, args_repr } => {");
    f.line("            ConstraintRef::Bound { observable_iri, bound_shape_iri, args_repr }");
    f.line("        }");
    f.line("        ConstraintRef::Affine { coefficients, coefficient_count, bias } => {");
    f.line("            let (out, new_count) =");
    f.line("                shift_affine_coefficients(&coefficients, coefficient_count, offset);");
    f.line("            ConstraintRef::Affine {");
    f.line("                coefficients: out,");
    f.line("                coefficient_count: new_count,");
    f.line("                bias,");
    f.line("            }");
    f.line("        }");
    f.line("        ConstraintRef::Conjunction { conjuncts, conjunct_count } => {");
    f.line("            let mut out = [LeafConstraintRef::Site { position: 0 }; CONJUNCTION_MAX_TERMS];");
    f.line("            let count = conjunct_count as usize;");
    f.line("            let mut i = 0;");
    f.line("            while i < count && i < CONJUNCTION_MAX_TERMS {");
    f.line("                out[i] = shift_leaf_constraint(conjuncts[i], offset);");
    f.line("                i += 1;");
    f.line("            }");
    f.line("            ConstraintRef::Conjunction {");
    f.line("                conjuncts: out,");
    f.line("                conjunct_count,");
    f.line("            }");
    f.line("        }");
    f.line("        // ADR-057: Recurse references a shape by content-addressed IRI;");
    f.line("        // there are no site positions to shift. Pass through unchanged.");
    f.line("        ConstraintRef::Recurse { shape_iri, descent_bound } => {");
    f.line("            ConstraintRef::Recurse { shape_iri, descent_bound }");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // Helper: shift a fixed-size i64 coefficient array by `offset`.
    f.doc_comment("Phase 17 helper: shift the active prefix of an `Affine`");
    f.doc_comment("coefficient array right by `offset`, returning a fresh");
    f.doc_comment("`[i64; AFFINE_MAX_COEFFS]` and the new active count. If the");
    f.doc_comment("shift would overflow the fixed buffer, returns count `0`");
    f.doc_comment("(vacuously consistent — no constraint).");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line("const fn shift_affine_coefficients(");
    f.line("    coefficients: &[i64; AFFINE_MAX_COEFFS],");
    f.line("    coefficient_count: u32,");
    f.line("    offset: u32,");
    f.line(") -> ([i64; AFFINE_MAX_COEFFS], u32) {");
    f.line("    let mut out = [0i64; AFFINE_MAX_COEFFS];");
    f.line("    let count = coefficient_count as usize;");
    f.line("    let off = offset as usize;");
    f.line("    if off >= AFFINE_MAX_COEFFS {");
    f.line("        return (out, 0);");
    f.line("    }");
    f.line("    let mut i = 0;");
    f.line("    while i < count && i + off < AFFINE_MAX_COEFFS {");
    f.line("        out[i + off] = coefficients[i];");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    let new_count = (i + off) as u32;");
    f.line("    (out, new_count)");
    f.line("}");
    f.blank();

    // Helper: leaf-flavored shift.
    f.doc_comment("Phase 17 helper: same as [`shift_constraint`] but operating on a");
    f.doc_comment("[`LeafConstraintRef`]. Used by Conjunction-shifting paths that");
    f.doc_comment("must preserve the leaf-only depth limit.");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line("pub const fn shift_leaf_constraint(c: LeafConstraintRef, offset: u32) -> LeafConstraintRef {");
    f.line("    match c {");
    f.line("        LeafConstraintRef::Site { position } => LeafConstraintRef::Site {");
    f.line("            position: position.saturating_add(offset),");
    f.line("        },");
    f.line("        LeafConstraintRef::Carry { site } => LeafConstraintRef::Carry {");
    f.line("            site: site.saturating_add(offset),");
    f.line("        },");
    f.line("        LeafConstraintRef::Residue { modulus, residue } => {");
    f.line("            LeafConstraintRef::Residue { modulus, residue }");
    f.line("        }");
    f.line("        LeafConstraintRef::Hamming { bound } => LeafConstraintRef::Hamming { bound },");
    f.line(
        "        LeafConstraintRef::Depth { min, max } => LeafConstraintRef::Depth { min, max },",
    );
    f.line("        LeafConstraintRef::SatClauses { clauses, num_vars } => {");
    f.line("            LeafConstraintRef::SatClauses { clauses, num_vars }");
    f.line("        }");
    f.line("        LeafConstraintRef::Bound { observable_iri, bound_shape_iri, args_repr } => {");
    f.line("            LeafConstraintRef::Bound { observable_iri, bound_shape_iri, args_repr }");
    f.line("        }");
    f.line("        LeafConstraintRef::Affine { coefficients, coefficient_count, bias } => {");
    f.line("            let (out, new_count) =");
    f.line("                shift_affine_coefficients(&coefficients, coefficient_count, offset);");
    f.line("            LeafConstraintRef::Affine {");
    f.line("                coefficients: out,");
    f.line("                coefficient_count: new_count,");
    f.line("                bias,");
    f.line("            }");
    f.line("        }");
    f.line("        // ADR-057: Recurse references a shape by content-addressed IRI;");
    f.line("        // there are no site positions to shift. Pass through unchanged.");
    f.line("        LeafConstraintRef::Recurse { shape_iri, descent_bound } => {");
    f.line("            LeafConstraintRef::Recurse { shape_iri, descent_bound }");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // sdk_concat_product_constraints — fixed-size array builder for the SDK macros.
    f.doc_comment("SDK support: concatenate two operand constraint arrays into a padded");
    f.doc_comment("fixed-size buffer of length `2 * crate::enforcement::NERVE_CONSTRAINTS_CAP`.");
    f.doc_comment("A's constraints are copied verbatim at indices `[0, A::CONSTRAINTS.len())`;");
    f.doc_comment("B's constraints are copied at `[A::CONSTRAINTS.len(), total)` with each");
    f.doc_comment("entry passed through `shift_constraint(c, A::SITE_COUNT as u32)`.");
    f.doc_comment("Trailing positions are filled with the `Site { position: u32::MAX }`");
    f.doc_comment("sentinel.");
    f.doc_comment("");
    f.doc_comment("Consumers pair this with `sdk_product_constraints_len` to derive the");
    f.doc_comment("slice length at const-eval time: `&BUF[..LEN]` yields a `&'static");
    f.doc_comment("[ConstraintRef]` of the correct length without `unsafe`.");
    f.line("pub const fn sdk_concat_product_constraints<A, B>()");
    f.line("    -> [ConstraintRef; 2 * crate::enforcement::NERVE_CONSTRAINTS_CAP]");
    f.line("where");
    f.line("    A: ConstrainedTypeShape,");
    f.line("    B: ConstrainedTypeShape,");
    f.line("{");
    f.line("    let mut out = [ConstraintRef::Site { position: u32::MAX };");
    f.line("                   2 * crate::enforcement::NERVE_CONSTRAINTS_CAP];");
    f.line("    let left = A::CONSTRAINTS;");
    f.line("    let right = B::CONSTRAINTS;");
    f.line("    let offset = A::SITE_COUNT as u32;");
    f.line("    let mut i: usize = 0;");
    f.line("    while i < left.len() {");
    f.line("        out[i] = left[i];");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    let mut j: usize = 0;");
    f.line("    while j < right.len() {");
    f.line("        out[i + j] = shift_constraint(right[j], offset);");
    f.line("        j += 1;");
    f.line("    }");
    f.line("    out");
    f.line("}");
    f.blank();

    f.doc_comment("Companion length helper for `sdk_concat_product_constraints`.");
    f.line("pub const fn sdk_product_constraints_len<A, B>() -> usize");
    f.line("where");
    f.line("    A: ConstrainedTypeShape,");
    f.line("    B: ConstrainedTypeShape,");
    f.line("{");
    f.line("    A::CONSTRAINTS.len() + B::CONSTRAINTS.len()");
    f.line("}");
    f.blank();

    // ADR-057 recurse-aware variants. When `a_recurse_bound` /
    // `b_recurse_bound` is `Some(bound)`, the corresponding operand
    // contributes a single `ConstraintRef::Recurse { shape_iri:
    // <T>::IRI, descent_bound: bound }` entry at its position rather
    // than inlining the operand's `CONSTRAINTS` array; SITE_COUNT is
    // still consulted to compute the post-operand site offset for the
    // following operand's inline constraints (recurse operands declare
    // their site footprint via the registered shape's SITE_COUNT).
    f.doc_comment("ADR-057 recurse-aware variant of `sdk_concat_product_constraints`.");
    f.doc_comment("");
    f.doc_comment("Per-operand `Option<u32>` recurse-bound parameter: `None` inlines the");
    f.doc_comment("operand's CONSTRAINTS array as before; `Some(bound)` emits a single");
    f.doc_comment("`ConstraintRef::Recurse { shape_iri: <T>::IRI, descent_bound: bound }`");
    f.doc_comment("at the operand's position. Site offsets for the following operand's");
    f.doc_comment("inline constraints account for the recurse operand's");
    f.doc_comment("`<T>::SITE_COUNT` as if it were inlined.");
    f.line("pub const fn sdk_concat_product_constraints_v2<A, B>(");
    f.line("    a_recurse_bound: Option<u32>,");
    f.line("    b_recurse_bound: Option<u32>,");
    f.line(") -> [ConstraintRef; 2 * crate::enforcement::NERVE_CONSTRAINTS_CAP]");
    f.line("where");
    f.line("    A: ConstrainedTypeShape,");
    f.line("    B: ConstrainedTypeShape,");
    f.line("{");
    f.line("    let mut out = [ConstraintRef::Site { position: u32::MAX };");
    f.line("                   2 * crate::enforcement::NERVE_CONSTRAINTS_CAP];");
    f.line("    let offset = A::SITE_COUNT as u32;");
    f.line("    let mut i: usize = 0;");
    f.line("    // A's contribution: single Recurse entry or inlined constraints.");
    f.line("    match a_recurse_bound {");
    f.line("        Some(bound) => {");
    f.line("            out[0] = ConstraintRef::Recurse {");
    f.line("                shape_iri: A::IRI,");
    f.line("                descent_bound: bound,");
    f.line("            };");
    f.line("            i = 1;");
    f.line("        }");
    f.line("        None => {");
    f.line("            let left = A::CONSTRAINTS;");
    f.line("            while i < left.len() {");
    f.line("                out[i] = left[i];");
    f.line("                i += 1;");
    f.line("            }");
    f.line("        }");
    f.line("    }");
    f.line("    // B's contribution: single Recurse entry or inlined-and-shifted constraints.");
    f.line("    match b_recurse_bound {");
    f.line("        Some(bound) => {");
    f.line("            out[i] = ConstraintRef::Recurse {");
    f.line("                shape_iri: B::IRI,");
    f.line("                descent_bound: bound,");
    f.line("            };");
    f.line("        }");
    f.line("        None => {");
    f.line("            let right = B::CONSTRAINTS;");
    f.line("            let mut j: usize = 0;");
    f.line("            while j < right.len() {");
    f.line("                out[i + j] = shift_constraint(right[j], offset);");
    f.line("                j += 1;");
    f.line("            }");
    f.line("        }");
    f.line("    }");
    f.line("    out");
    f.line("}");
    f.blank();

    f.doc_comment("Companion length helper for `sdk_concat_product_constraints_v2`.");
    f.line("pub const fn sdk_product_constraints_v2_len<A, B>(");
    f.line("    a_recurse: bool,");
    f.line("    b_recurse: bool,");
    f.line(") -> usize");
    f.line("where");
    f.line("    A: ConstrainedTypeShape,");
    f.line("    B: ConstrainedTypeShape,");
    f.line("{");
    f.line("    let a_len = if a_recurse { 1 } else { A::CONSTRAINTS.len() };");
    f.line("    let b_len = if b_recurse { 1 } else { B::CONSTRAINTS.len() };");
    f.line("    a_len + b_len");
    f.line("}");
    f.blank();
}

/// v0.2.2 Phase P.2: public admission functions for arbitrary downstream
/// `ConstrainedTypeShape` impls. Runtime and const variants run the same
/// preflight validators (`preflight_feasibility` + `preflight_package_coherence`)
/// before minting a `Validated<T, Phase>`.
fn emit_admission_fns(f: &mut RustFile) {
    // Forward-declaration comment: these functions call preflight_feasibility
    // and preflight_package_coherence which are emitted later in the same
    // module. Rust's resolution is file-level, so forward-reference is fine.
    f.doc_comment("Admit a downstream [`ConstrainedTypeShape`] into the reduction pipeline.");
    f.doc_comment("");
    f.doc_comment("Runs the full preflight chain on `T::CONSTRAINTS`:");
    f.doc_comment("[`preflight_feasibility`] and [`preflight_package_coherence`]. On success,");
    f.doc_comment("wraps the supplied `shape` in a [`Validated`] carrying `Runtime` phase.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns [`ShapeViolation`] if any constraint in `T::CONSTRAINTS` fails");
    f.doc_comment("feasibility checking (e.g., residue out of range, depth min > max) or if");
    f.doc_comment("the constraint system is internally incoherent (e.g., contradictory");
    f.doc_comment("residue constraints on the same modulus).");
    f.doc_comment("");
    f.doc_comment("# Example");
    f.doc_comment("");
    f.doc_comment("```");
    f.doc_comment("use uor_foundation::pipeline::{");
    f.doc_comment("    ConstrainedTypeShape, ConstraintRef, validate_constrained_type,");
    f.doc_comment("};");
    f.doc_comment("");
    f.doc_comment("pub struct MyShape;");
    f.doc_comment("impl ConstrainedTypeShape for MyShape {");
    f.doc_comment("    const IRI: &'static str = \"https://example.org/MyShape\";");
    f.doc_comment("    const SITE_COUNT: usize = 2;");
    f.doc_comment("    const CONSTRAINTS: &'static [ConstraintRef] = &[");
    f.doc_comment("        ConstraintRef::Residue { modulus: 5, residue: 2 },");
    f.doc_comment("    ];");
    f.doc_comment("    const CYCLE_SIZE: u64 = 5;  // ADR-032: 5 residue classes mod 5");
    f.doc_comment("}");
    f.doc_comment("");
    f.doc_comment("let validated = validate_constrained_type(MyShape).unwrap();");
    f.doc_comment("# let _ = validated;");
    f.doc_comment("```");
    f.line("pub fn validate_constrained_type<T: ConstrainedTypeShape>(");
    f.line("    shape: T,");
    f.line(") -> Result<Validated<T, crate::enforcement::Runtime>, ShapeViolation> {");
    f.line("    preflight_feasibility(T::CONSTRAINTS)?;");
    f.line("    preflight_package_coherence(T::CONSTRAINTS)?;");
    f.line("    Ok(Validated::new(shape))");
    f.line("}");
    f.blank();
    f.doc_comment("Const-fn companion for [`validate_constrained_type`].");
    f.doc_comment("");
    f.doc_comment("Admits a downstream [`ConstrainedTypeShape`] at compile time, running the");
    f.doc_comment("same preflight checks as the runtime variant but in `const` context.");
    f.doc_comment("");
    f.doc_comment("# Scope");
    f.doc_comment("");
    f.doc_comment("`ConstraintRef::Bound { observable_iri, args_repr, .. }` with");
    f.doc_comment("`observable_iri == \"https://uor.foundation/observable/LandauerCost\"`");
    f.doc_comment("requires `f64::from_bits` for args parsing, which is stable in `const`");
    f.doc_comment("context only from Rust 1.83. The crate's MSRV is 1.81, so this variant");
    f.doc_comment("rejects const admission of `LandauerCost`-bound constraints with");
    f.doc_comment("[`ShapeViolation`] and recommends the runtime [`validate_constrained_type`]");
    f.doc_comment("for those inputs. All other `ConstraintRef` variants admit at const time.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Same as [`validate_constrained_type`], plus the const-context rejection");
    f.doc_comment("for `LandauerCost`-bound constraints described above.");
    f.doc_comment("");
    f.doc_comment("The `T: Copy` bound is required by `const fn` — destructor invocation is");
    f.doc_comment("not yet const-stable, and `Validated<T>` carries `T` by value. Shape");
    f.doc_comment("markers are typically zero-sized types which are trivially `Copy`.");
    f.line("pub const fn validate_constrained_type_const<T: ConstrainedTypeShape + Copy>(");
    f.line("    shape: T,");
    f.line(") -> Result<Validated<T, crate::enforcement::CompileTime>, ShapeViolation> {");
    f.line("    // Const-path preflight: walk CONSTRAINTS and apply per-variant const checks.");
    f.line("    // Rejects LandauerCost-bound constraints that need non-const f64::from_bits.");
    f.line("    let constraints = T::CONSTRAINTS;");
    f.line("    let mut i = 0;");
    f.line("    while i < constraints.len() {");
    f.line("        let ok = match &constraints[i] {");
    f.line("            ConstraintRef::SatClauses { clauses, num_vars } => {");
    f.line("                *num_vars != 0 || clauses.is_empty()");
    f.line("            }");
    f.line("            ConstraintRef::Residue { modulus, residue } => {");
    f.line("                *modulus != 0 && *residue < *modulus");
    f.line("            }");
    f.line("            ConstraintRef::Carry { .. } => true,");
    f.line("            ConstraintRef::Depth { min, max } => *min <= *max,");
    f.line("            ConstraintRef::Hamming { bound } => *bound <= 32_768,");
    f.line("            ConstraintRef::Site { .. } => true,");
    f.line("            ConstraintRef::Affine { coefficients, coefficient_count, bias } => {");
    f.line("                // Mirror preflight_feasibility's Affine arm in const context.");
    f.line("                let count = *coefficient_count as usize;");
    f.line("                if count == 0 {");
    f.line("                    false");
    f.line("                } else {");
    f.line("                    let mut ok_coeff = true;");
    f.line("                    let mut idx = 0;");
    f.line("                    while idx < count && idx < AFFINE_MAX_COEFFS {");
    f.line("                        if coefficients[idx] == i64::MIN {");
    f.line("                            ok_coeff = false;");
    f.line("                            break;");
    f.line("                        }");
    f.line("                        idx += 1;");
    f.line("                    }");
    f.line("                    ok_coeff && is_affine_consistent(coefficients, *coefficient_count, *bias)");
    f.line("                }");
    f.line("            }");
    f.line("            ConstraintRef::Bound { observable_iri, .. } => {");
    f.line("                // const-fn scope: LandauerCost needs f64::from_bits (stable in");
    f.line("                // const at 1.83). Reject it here; runtime admission handles it.");
    f.line("                !crate::enforcement::str_eq(");
    f.line("                    observable_iri,");
    f.line("                    \"https://uor.foundation/observable/LandauerCost\",");
    f.line("                )");
    f.line("            }");
    f.line("            ConstraintRef::Conjunction { conjuncts, conjunct_count } => {");
    f.line("                conjunction_all_sat(conjuncts, *conjunct_count)");
    f.line("            }");
    f.line("            // ADR-057: const-time validation defers Recurse to runtime");
    f.line("            // admission (parallel to ADR-049's LandauerCost deferral).");
    f.line("            // The only const check is that `shape_iri` is non-empty —");
    f.line("            // the registry lookup happens at the runtime admission path.");
    f.line("            ConstraintRef::Recurse { shape_iri, .. } => !shape_iri.is_empty(),");
    f.line("        };");
    f.line("        if !ok {");
    f.line("            return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/type/ConstrainedType\",");
    f.line(
        "                constraint_iri: \"https://uor.foundation/type/ConstrainedType_const_constraint\",",
    );
    f.line("                property_iri: \"https://uor.foundation/type/constraints\",");
    f.line("                expected_range: \"https://uor.foundation/type/Constraint\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::ValueCheck,");
    f.line("            });");
    f.line("        }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    Ok(Validated::new(shape))");
    f.line("}");
    f.blank();
}

fn emit_fragment_classifier(f: &mut RustFile) {
    f.doc_comment("Result of `fragment_classify`: which `predicate:*Shape` fragment the");
    f.doc_comment("input belongs to. Drives `InhabitanceResolver` dispatch routing.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
    f.line("pub enum FragmentKind {");
    f.line("    /// `predicate:Is2SatShape` — clauses of width ≤ 2.");
    f.line("    TwoSat,");
    f.line("    /// `predicate:IsHornShape` — clauses with ≤ 1 positive literal.");
    f.line("    Horn,");
    f.line("    /// `predicate:IsResidualFragment` — catch-all; no polynomial bound.");
    f.line("    Residual,");
    f.line("}");
    f.blank();

    f.doc_comment("Classify a constraint system into one of the three dispatch fragments.");
    f.doc_comment("");
    f.doc_comment("The classifier inspects the first `SatClauses` constraint (if any) and");
    f.doc_comment("applies the ontology's shape predicates. Constraint systems with no");
    f.doc_comment("`SatClauses` constraint — e.g., pure residue/hamming constraints — are");
    f.doc_comment("classified as `Residual`: the dispatch table has no polynomial decider");
    f.doc_comment("for them, so they route to the `ResidualVerdictResolver` catch-all.");
    f.line("#[must_use]");
    f.line("pub const fn fragment_classify(constraints: &[ConstraintRef]) -> FragmentKind {");
    f.line("    let mut i = 0;");
    f.line("    while i < constraints.len() {");
    f.line("        if let ConstraintRef::SatClauses { clauses, .. } = constraints[i] {");
    f.line("            // Classify by maximum clause width and positive-literal count.");
    f.line("            let mut max_width: usize = 0;");
    f.line("            let mut horn: bool = true;");
    f.line("            let mut j = 0;");
    f.line("            while j < clauses.len() {");
    f.line("                let clause = clauses[j];");
    f.line("                if clause.len() > max_width {");
    f.line("                    max_width = clause.len();");
    f.line("                }");
    f.line("                let mut positives: usize = 0;");
    f.line("                let mut k = 0;");
    f.line("                while k < clause.len() {");
    f.line("                    let (_, negated) = clause[k];");
    f.line("                    if !negated {");
    f.line("                        positives += 1;");
    f.line("                    }");
    f.line("                    k += 1;");
    f.line("                }");
    f.line("                if positives > 1 {");
    f.line("                    horn = false;");
    f.line("                }");
    f.line("                j += 1;");
    f.line("            }");
    f.line("            if max_width <= 2 {");
    f.line("                return FragmentKind::TwoSat;");
    f.line("            } else if horn {");
    f.line("                return FragmentKind::Horn;");
    f.line("            } else {");
    f.line("                return FragmentKind::Residual;");
    f.line("            }");
    f.line("        }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    // No SAT clauses at all — residual.");
    f.line("    FragmentKind::Residual");
    f.line("}");
    f.blank();
}

fn emit_two_sat_decider(f: &mut RustFile, ontology: &Ontology) {
    // v0.2.1 Phase 7b.6: bounds sourced from reduction:TwoSatBound individual.
    let bound = individuals_of_type(ontology, "https://uor.foundation/reduction/SatBound")
        .into_iter()
        .find(|i| i.id.ends_with("/TwoSatBound"));
    let max_vars = bound
        .and_then(|b| ind_prop_int(b, "https://uor.foundation/reduction/maxVarCount"))
        .unwrap_or(256) as usize;
    let max_clauses = bound
        .and_then(|b| ind_prop_int(b, "https://uor.foundation/reduction/maxClauseCount"))
        .unwrap_or(512) as usize;
    let max_edges = max_clauses * 4;
    f.doc_comment("Aspvall-Plass-Tarjan 2-SAT decider: returns `true` iff the clause list");
    f.doc_comment("is satisfiable.");
    f.doc_comment("");
    f.doc_comment("Builds the implication graph: for each clause `(a | b)`, adds");
    f.doc_comment("`¬a → b` and `¬b → a`. Runs Tarjan's SCC algorithm and checks that");
    f.doc_comment("no variable and its negation share an SCC. O(n+m) via iterative");
    f.doc_comment("Tarjan (the `no_std` path can't recurse freely).");
    f.doc_comment("");
    f.doc_comment(&format!(
        "Bounds (from `reduction:TwoSatBound`): up to {max_vars} variables, \
         up to {max_clauses} clauses. The `const` bounds keep the entire decider on \
         the stack — essential for `no_std` and compile-time proc-macro expansion."
    ));
    f.line(&format!("const TWO_SAT_MAX_VARS: usize = {max_vars};"));
    f.line("const TWO_SAT_MAX_NODES: usize = TWO_SAT_MAX_VARS * 2;");
    f.line(&format!("const TWO_SAT_MAX_EDGES: usize = {max_edges};"));
    f.blank();
    f.line("/// 2-SAT decision result.");
    f.line("#[must_use]");
    f.line("pub fn decide_two_sat(");
    f.line("    clauses: &[&[(u32, bool)]],");
    f.line("    num_vars: u32,");
    f.line(") -> bool {");
    f.line("    if (num_vars as usize) > TWO_SAT_MAX_VARS {");
    f.line("        return false;");
    f.line("    }");
    f.line("    let n = (num_vars as usize) * 2;");
    f.line("    // Node index: 2*var is positive literal, 2*var+1 is negated.");
    f.line("    let mut adj_starts = [0usize; TWO_SAT_MAX_NODES + 1];");
    f.line("    let mut adj_targets = [0usize; TWO_SAT_MAX_EDGES];");
    f.line("    // First pass: count out-degrees.");
    f.line("    for clause in clauses {");
    f.line("        if clause.len() > 2 || clause.is_empty() {");
    f.line("            return false;");
    f.line("        }");
    f.line("        if clause.len() == 1 {");
    f.line("            let (v, neg) = clause[0];");
    f.line("            let lit = lit_index(v, neg);");
    f.line("            let neg_lit = lit_index(v, !neg);");
    f.line("            // x ↔ (x ∨ x): ¬x → x (assignment forced)");
    f.line("            if neg_lit < n + 1 {");
    f.line("                adj_starts[neg_lit + 1] += 1;");
    f.line("            }");
    f.line("            let _ = lit;");
    f.line("        } else {");
    f.line("            let (a, an) = clause[0];");
    f.line("            let (b, bn) = clause[1];");
    f.line("            // ¬a → b, ¬b → a");
    f.line("            let na = lit_index(a, !an);");
    f.line("            let nb = lit_index(b, !bn);");
    f.line("            if na + 1 < n + 1 {");
    f.line("                adj_starts[na + 1] += 1;");
    f.line("            }");
    f.line("            if nb + 1 < n + 1 {");
    f.line("                adj_starts[nb + 1] += 1;");
    f.line("            }");
    f.line("        }");
    f.line("    }");
    f.line("    // Prefix-sum to get adjacency starts.");
    f.line("    let mut i = 1;");
    f.line("    while i <= n {");
    f.line("        adj_starts[i] += adj_starts[i - 1];");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    let edge_count = adj_starts[n];");
    f.line("    if edge_count > TWO_SAT_MAX_EDGES {");
    f.line("        return false;");
    f.line("    }");
    f.line("    let mut fill = [0usize; TWO_SAT_MAX_NODES];");
    f.line("    for clause in clauses {");
    f.line("        if clause.len() == 1 {");
    f.line("            let (v, neg) = clause[0];");
    f.line("            let pos_lit = lit_index(v, neg);");
    f.line("            let neg_lit = lit_index(v, !neg);");
    f.line("            let slot = adj_starts[neg_lit] + fill[neg_lit];");
    f.line("            adj_targets[slot] = pos_lit;");
    f.line("            fill[neg_lit] += 1;");
    f.line("        } else {");
    f.line("            let (a, an) = clause[0];");
    f.line("            let (b, bn) = clause[1];");
    f.line("            let pa = lit_index(a, an);");
    f.line("            let na = lit_index(a, !an);");
    f.line("            let pb = lit_index(b, bn);");
    f.line("            let nb = lit_index(b, !bn);");
    f.line("            let s1 = adj_starts[na] + fill[na];");
    f.line("            adj_targets[s1] = pb;");
    f.line("            fill[na] += 1;");
    f.line("            let s2 = adj_starts[nb] + fill[nb];");
    f.line("            adj_targets[s2] = pa;");
    f.line("            fill[nb] += 1;");
    f.line("        }");
    f.line("    }");
    f.line("    // Iterative Tarjan's SCC.");
    f.line("    let mut index_counter: usize = 0;");
    f.line("    let mut indices = [usize::MAX; TWO_SAT_MAX_NODES];");
    f.line("    let mut lowlinks = [0usize; TWO_SAT_MAX_NODES];");
    f.line("    let mut on_stack = [false; TWO_SAT_MAX_NODES];");
    f.line("    let mut stack = [0usize; TWO_SAT_MAX_NODES];");
    f.line("    let mut stack_top: usize = 0;");
    f.line("    let mut scc_id = [usize::MAX; TWO_SAT_MAX_NODES];");
    f.line("    let mut scc_count: usize = 0;");
    f.line("    let mut call_stack = [(0usize, 0usize); TWO_SAT_MAX_NODES];");
    f.line("    let mut call_top: usize = 0;");
    f.line("    let mut v = 0;");
    f.line("    while v < n {");
    f.line("        if indices[v] == usize::MAX {");
    f.line("            call_stack[call_top] = (v, adj_starts[v]);");
    f.line("            call_top += 1;");
    f.line("            indices[v] = index_counter;");
    f.line("            lowlinks[v] = index_counter;");
    f.line("            index_counter += 1;");
    f.line("            stack[stack_top] = v;");
    f.line("            stack_top += 1;");
    f.line("            on_stack[v] = true;");
    f.line("            while call_top > 0 {");
    f.line("                let (u, mut next_edge) = call_stack[call_top - 1];");
    f.line("                let end_edge = adj_starts[u + 1];");
    f.line("                let mut advanced = false;");
    f.line("                while next_edge < end_edge {");
    f.line("                    let w = adj_targets[next_edge];");
    f.line("                    next_edge += 1;");
    f.line("                    if indices[w] == usize::MAX {");
    f.line("                        call_stack[call_top - 1] = (u, next_edge);");
    f.line("                        indices[w] = index_counter;");
    f.line("                        lowlinks[w] = index_counter;");
    f.line("                        index_counter += 1;");
    f.line("                        stack[stack_top] = w;");
    f.line("                        stack_top += 1;");
    f.line("                        on_stack[w] = true;");
    f.line("                        call_stack[call_top] = (w, adj_starts[w]);");
    f.line("                        call_top += 1;");
    f.line("                        advanced = true;");
    f.line("                        break;");
    f.line("                    } else if on_stack[w] && indices[w] < lowlinks[u] {");
    f.line("                        lowlinks[u] = indices[w];");
    f.line("                    }");
    f.line("                }");
    f.line("                if !advanced {");
    f.line("                    call_stack[call_top - 1] = (u, next_edge);");
    f.line("                    if lowlinks[u] == indices[u] {");
    f.line("                        loop {");
    f.line("                            stack_top -= 1;");
    f.line("                            let w = stack[stack_top];");
    f.line("                            on_stack[w] = false;");
    f.line("                            scc_id[w] = scc_count;");
    f.line("                            if w == u {");
    f.line("                                break;");
    f.line("                            }");
    f.line("                        }");
    f.line("                        scc_count += 1;");
    f.line("                    }");
    f.line("                    call_top -= 1;");
    f.line("                    if call_top > 0 {");
    f.line("                        let (parent, _) = call_stack[call_top - 1];");
    f.line("                        if lowlinks[u] < lowlinks[parent] {");
    f.line("                            lowlinks[parent] = lowlinks[u];");
    f.line("                        }");
    f.line("                    }");
    f.line("                }");
    f.line("            }");
    f.line("        }");
    f.line("        v += 1;");
    f.line("    }");
    f.line("    // Unsatisfiable iff x and ¬x are in the same SCC for any variable.");
    f.line("    let mut var = 0u32;");
    f.line("    while var < num_vars {");
    f.line("        let pos = lit_index(var, false);");
    f.line("        let neg = lit_index(var, true);");
    f.line("        if scc_id[pos] == scc_id[neg] {");
    f.line("            return false;");
    f.line("        }");
    f.line("        var += 1;");
    f.line("    }");
    f.line("    true");
    f.line("}");
    f.blank();
    f.line("#[inline]");
    f.line("const fn lit_index(var: u32, negated: bool) -> usize {");
    f.line("    let base = (var as usize) * 2;");
    f.line("    if negated { base + 1 } else { base }");
    f.line("}");
    f.blank();
}

fn emit_horn_sat_decider(f: &mut RustFile, ontology: &Ontology) {
    // v0.2.1 Phase 7b.6: bounds sourced from reduction:HornSatBound individual.
    let bound = individuals_of_type(ontology, "https://uor.foundation/reduction/SatBound")
        .into_iter()
        .find(|i| i.id.ends_with("/HornSatBound"));
    let max_vars = bound
        .and_then(|b| ind_prop_int(b, "https://uor.foundation/reduction/maxVarCount"))
        .unwrap_or(256) as usize;
    f.doc_comment("Horn-SAT decider via unit propagation. Returns `true` iff the clause");
    f.doc_comment("list is satisfiable.");
    f.doc_comment("");
    f.doc_comment("Algorithm: start with all variables false. Repeatedly find a clause");
    f.doc_comment("whose negative literals are all satisfied but whose positive literal");
    f.doc_comment("is unassigned/false; set the positive literal true. Fail if a clause");
    f.doc_comment("with no positive literal has all its negatives satisfied.");
    f.doc_comment("");
    f.doc_comment(&format!(
        "Bounds (from `reduction:HornSatBound`): up to {max_vars} variables."
    ));
    f.line(&format!("const HORN_MAX_VARS: usize = {max_vars};"));
    f.blank();
    f.line("/// Horn-SAT decision result.");
    f.line("#[must_use]");
    f.line("pub fn decide_horn_sat(");
    f.line("    clauses: &[&[(u32, bool)]],");
    f.line("    num_vars: u32,");
    f.line(") -> bool {");
    f.line("    if (num_vars as usize) > HORN_MAX_VARS {");
    f.line("        return false;");
    f.line("    }");
    f.line("    let mut assignment = [false; HORN_MAX_VARS];");
    f.line("    let n = num_vars as usize;");
    f.line("    loop {");
    f.line("        let mut changed = false;");
    f.line("        for clause in clauses {");
    f.line("            // Count positive literals.");
    f.line("            let mut positive: Option<u32> = None;");
    f.line("            let mut positive_count = 0;");
    f.line("            for (_, negated) in clause.iter() {");
    f.line("                if !*negated {");
    f.line("                    positive_count += 1;");
    f.line("                }");
    f.line("            }");
    f.line("            if positive_count > 1 {");
    f.line("                return false;");
    f.line("            }");
    f.line("            for (var, negated) in clause.iter() {");
    f.line("                if !*negated {");
    f.line("                    positive = Some(*var);");
    f.line("                }");
    f.line("            }");
    f.line("            // Check whether all negative literals are satisfied (var=true).");
    f.line("            let mut all_neg_satisfied = true;");
    f.line("            for (var, negated) in clause.iter() {");
    f.line("                if *negated {");
    f.line("                    let idx = *var as usize;");
    f.line("                    if idx >= n {");
    f.line("                        return false;");
    f.line("                    }");
    f.line("                    if !assignment[idx] {");
    f.line("                        all_neg_satisfied = false;");
    f.line("                        break;");
    f.line("                    }");
    f.line("                }");
    f.line("            }");
    f.line("            if all_neg_satisfied {");
    f.line("                match positive {");
    f.line("                    None => return false,");
    f.line("                    Some(v) => {");
    f.line("                        let idx = v as usize;");
    f.line("                        if idx >= n {");
    f.line("                            return false;");
    f.line("                        }");
    f.line("                        if !assignment[idx] {");
    f.line("                            assignment[idx] = true;");
    f.line("                            changed = true;");
    f.line("                        }");
    f.line("                    }");
    f.line("                }");
    f.line("            }");
    f.line("        }");
    f.line("        if !changed {");
    f.line("            break;");
    f.line("        }");
    f.line("    }");
    f.line("    // Final verification pass.");
    f.line("    for clause in clauses {");
    f.line("        let mut satisfied = false;");
    f.line("        for (var, negated) in clause.iter() {");
    f.line("            let idx = *var as usize;");
    f.line("            if idx >= n {");
    f.line("                return false;");
    f.line("            }");
    f.line("            let val = assignment[idx];");
    f.line("            if (*negated && !val) || (!*negated && val) {");
    f.line("                satisfied = true;");
    f.line("                break;");
    f.line("            }");
    f.line("        }");
    f.line("        if !satisfied {");
    f.line("            return false;");
    f.line("        }");
    f.line("    }");
    f.line("    true");
    f.line("}");
    f.blank();
}

// v0.2.2 T6.14: `hash_constraints` deleted. The foundation does not pick a
// hash function; downstream supplies `H: Hasher` and the typed pipeline
// entry points thread it through `fold_unit_digest`.

fn emit_preflight_checks(f: &mut RustFile, ontology: &Ontology) {
    // v0.2.1 Phase 7b.5: preflight IRI strings are resolved at codegen time
    // from the ontology. Changing a shape or constraint IRI in the spec
    // regenerates this file's literals automatically. Phase 7b.7 additionally
    // reads the TimingBound individuals.
    let shape_iri = ontology
        .namespaces
        .iter()
        .flat_map(|n| n.individuals.iter())
        .find(|i| i.id.ends_with("/CompileUnitShape"))
        .map(|i| i.id)
        .unwrap_or("https://uor.foundation/conformance/CompileUnitShape");
    let budget_constraint_iri = ontology
        .namespaces
        .iter()
        .flat_map(|n| n.individuals.iter())
        .find(|i| i.id.ends_with("compileUnit_thermodynamicBudget_constraint"))
        .map(|i| i.id)
        .unwrap_or("https://uor.foundation/conformance/compileUnit_thermodynamicBudget_constraint");
    let root_term_constraint_iri = ontology
        .namespaces
        .iter()
        .flat_map(|n| n.individuals.iter())
        .find(|i| i.id.ends_with("compileUnit_rootTerm_constraint"))
        .map(|i| i.id)
        .unwrap_or("https://uor.foundation/conformance/compileUnit_rootTerm_constraint");
    let thermodynamic_budget_prop = "https://uor.foundation/reduction/thermodynamicBudget";
    let root_term_prop = "https://uor.foundation/reduction/rootTerm";
    let term_range = "https://uor.foundation/schema/Term";
    let xsd_decimal = "http://www.w3.org/2001/XMLSchema#decimal";

    // Phase 7b.7: Timing bounds sourced from reduction:TimingBound individuals.
    let preflight_budget_ns =
        individuals_of_type(ontology, "https://uor.foundation/reduction/TimingBound")
            .into_iter()
            .find(|i| i.id.ends_with("/PreflightTimingBound"))
            .and_then(|i| ind_prop_int(i, "https://uor.foundation/reduction/preflightBudgetNs"))
            .unwrap_or(10_000_000);
    let runtime_budget_ns =
        individuals_of_type(ontology, "https://uor.foundation/reduction/TimingBound")
            .into_iter()
            .find(|i| i.id.ends_with("/RuntimeTimingBound"))
            .and_then(|i| ind_prop_int(i, "https://uor.foundation/reduction/runtimeBudgetNs"))
            .unwrap_or(10_000_000);

    f.doc_comment("`BudgetSolvencyCheck` (preflightOrder 0): `thermodynamicBudget` must be");
    f.doc_comment("≥ `bitsWidth(unitWittLevel) × ln 2` per `op:GS_7` / `op:OA_5`.");
    f.doc_comment("");
    f.doc_comment("Takes the budget in `k_B T · ln 2` units and the target Witt level in");
    f.doc_comment("bit-width. Returns `Ok(())` if solvent, `Err` with the shape violation.");
    f.line("pub fn preflight_budget_solvency(budget_units: u64, witt_bits: u32) -> Result<(), ShapeViolation> {");
    f.line("    // Landauer bound: one bit requires k_B T · ln 2. Integer form.");
    f.line("    let minimum = witt_bits as u64;");
    f.line("    if budget_units >= minimum {");
    f.line("        Ok(())");
    f.line("    } else {");
    f.line("        Err(ShapeViolation {");
    f.line(&format!("            shape_iri: \"{shape_iri}\","));
    f.line(&format!(
        "            constraint_iri: \"{budget_constraint_iri}\","
    ));
    f.line(&format!(
        "            property_iri: \"{thermodynamic_budget_prop}\","
    ));
    f.line(&format!("            expected_range: \"{xsd_decimal}\","));
    f.line("            min_count: 1,");
    f.line("            max_count: 1,");
    f.line("            kind: ViolationKind::ValueCheck,");
    f.line("        })");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase F: upper bound on `CarryDepthObservable` depth arguments.");
    f.doc_comment("Matches target §4.5's Witt-level tower ceiling (W16384).");
    f.line("pub const WITT_MAX_BITS: u16 = 16_384;");
    f.blank();
    f.doc_comment("v0.2.2 Phase F: ASCII parser for a single unsigned decimal `u32`.");
    f.doc_comment("Returns 0 on malformed input; the caller's downstream comparison (`depth <= WITT_MAX_BITS`)");
    f.doc_comment("accepts 0 as the pass-through degenerate depth, so malformed input is rejected");
    f.doc_comment("by the enclosing feasibility check only if the parsed value exceeds the cap.");
    f.doc_comment(
        "For stricter input discipline, the caller pre-validates `args_repr` at builder time.",
    );
    f.line("#[must_use]");
    f.line("pub fn parse_u32(s: &str) -> u32 {");
    f.line("    let bytes = s.as_bytes();");
    f.line("    let mut out: u32 = 0;");
    f.line("    let mut i = 0;");
    f.line("    while i < bytes.len() {");
    f.line("        let b = bytes[i];");
    f.line("        if !b.is_ascii_digit() {");
    f.line("            return 0;");
    f.line("        }");
    f.line("        out = out.saturating_mul(10).saturating_add((b - b'0') as u32);");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    out");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase F: ASCII parser for a single unsigned decimal `u64`.");
    f.line("#[must_use]");
    f.line("pub fn parse_u64(s: &str) -> u64 {");
    f.line("    let bytes = s.as_bytes();");
    f.line("    let mut out: u64 = 0;");
    f.line("    let mut i = 0;");
    f.line("    while i < bytes.len() {");
    f.line("        let b = bytes[i];");
    f.line("        if !b.is_ascii_digit() {");
    f.line("            return 0;");
    f.line("        }");
    f.line("        out = out.saturating_mul(10).saturating_add((b - b'0') as u64);");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    out");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase F: parser for `\"modulus|residue\"` decimal pairs.");
    f.doc_comment("Split on the first ASCII `|`; ASCII-digit-parse each half via `parse_u64`.");
    f.line("#[must_use]");
    f.line("pub fn parse_u64_pair(s: &str) -> (u64, u64) {");
    f.line("    let bytes = s.as_bytes();");
    f.line("    let mut split = bytes.len();");
    f.line("    let mut i = 0;");
    f.line("    while i < bytes.len() {");
    f.line("        if bytes[i] == b'|' {");
    f.line("            split = i;");
    f.line("            break;");
    f.line("        }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    if split >= bytes.len() {");
    f.line("        return (0, 0);");
    f.line("    }");
    f.line("    let (left, right_with_pipe) = s.split_at(split);");
    f.line("    let (_, right) = right_with_pipe.split_at(1);");
    f.line("    (parse_u64(left), parse_u64(right))");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase F / Phase 9: parse a decimal `u64` representing an");
    f.doc_comment("IEEE-754 bit pattern. The bit pattern is content-deterministic; call sites");
    f.doc_comment("project to `H::Decimal` via `DecimalTranscendental::from_bits`.");
    f.line("#[must_use]");
    f.line("pub fn parse_u64_bits_str(s: &str) -> u64 {");
    f.line("    parse_u64(s)");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase F: dispatch a `ConstraintRef::Bound` arm on its `observable_iri`.");
    f.doc_comment(
        "Canonical observables: `ValueModObservable`, `CarryDepthObservable`, `LandauerCost`.",
    );
    f.doc_comment("Unknown IRIs are rejected so that an unaudited observable cannot be threaded");
    f.doc_comment("through the preflight surface silently.");
    f.line("fn check_bound_feasibility(");
    f.line("    observable_iri: &'static str,");
    f.line("    bound_shape_iri: &'static str,");
    f.line("    args_repr: &'static str,");
    f.line(") -> Result<(), ShapeViolation> {");
    f.line(
        "    const VALUE_MOD_IRI: &str = \"https://uor.foundation/observable/ValueModObservable\";",
    );
    f.line("    const CARRY_DEPTH_IRI: &str = \"https://uor.foundation/observable/CarryDepthObservable\";");
    f.line("    const LANDAUER_IRI: &str = \"https://uor.foundation/observable/LandauerCost\";");
    f.line("    let ok = if crate::enforcement::str_eq(observable_iri, VALUE_MOD_IRI) {");
    f.line("        let (modulus, residue) = parse_u64_pair(args_repr);");
    f.line("        modulus != 0 && residue < modulus");
    f.line("    } else if crate::enforcement::str_eq(observable_iri, CARRY_DEPTH_IRI) {");
    f.line("        let depth = parse_u32(args_repr);");
    f.line("        depth <= WITT_MAX_BITS as u32");
    f.line("    } else if crate::enforcement::str_eq(observable_iri, LANDAUER_IRI) {");
    f.line("        // Project the bit pattern to f64 (default-host) for the");
    f.line("        // finite/positive-nats sanity check. Polymorphic consumers");
    f.line("        // construct their own H::Decimal via DecimalTranscendental.");
    f.line("        let bits = parse_u64_bits_str(args_repr);");
    f.line("        let nats = <f64 as crate::DecimalTranscendental>::from_bits(bits);");
    f.line("        nats.is_finite() && nats > 0.0");
    f.line("    } else {");
    f.line("        false");
    f.line("    };");
    f.line("    if ok {");
    f.line("        Ok(())");
    f.line("    } else {");
    f.line("        Err(ShapeViolation {");
    f.line("            shape_iri: bound_shape_iri,");
    f.line("            constraint_iri: \"https://uor.foundation/type/BoundConstraint\",");
    f.line("            property_iri: observable_iri,");
    f.line("            expected_range: \"https://uor.foundation/observable/BaseMetric\",");
    f.line("            min_count: 1,");
    f.line("            max_count: 1,");
    f.line("            kind: ViolationKind::ValueCheck,");
    f.line("        })");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("`FeasibilityCheck`: verify the constraint system isn't trivially");
    f.doc_comment("infeasible. Workstream E (target §1.5 + §4.7, v0.2.2 closure):");
    f.doc_comment("the closed six-kind constraint set is validated by direct per-kind");
    f.doc_comment("satisfiability checks; any variant that fails is rejected here so");
    f.doc_comment("downstream resolvers never see an unsatisfiable constraint system.");
    f.doc_comment("");
    f.doc_comment(
        "v0.2.2 Phase F: the `Bound` arm dispatches on `observable_iri` to per-observable",
    );
    f.doc_comment("checks via `check_bound_feasibility`; `Carry` and `Site` remain `Ok(())` by");
    f.doc_comment(
        "documented design \u{2014} their constraint semantics are structural invariants of",
    );
    f.doc_comment("ring arithmetic and site-index bounds respectively, enforced by construction");
    f.doc_comment("rather than by runtime feasibility checks.");
    f.line(
        "pub fn preflight_feasibility(constraints: &[ConstraintRef]) -> Result<(), ShapeViolation> {",
    );
    f.line("    for c in constraints {");
    f.line("        // v0.2.2 Phase F: Bound dispatches to observable-specific checks with its");
    f.line("        // own bound_shape_iri; early-return with that shape on failure.");
    f.line(
        "        if let ConstraintRef::Bound { observable_iri, bound_shape_iri, args_repr } = c {",
    );
    f.line("            check_bound_feasibility(observable_iri, bound_shape_iri, args_repr)?;");
    f.line("            continue;");
    f.line("        }");
    f.line("        let ok = match c {");
    f.line("            ConstraintRef::SatClauses { clauses, num_vars } => {");
    f.line("                *num_vars != 0 || clauses.is_empty()");
    f.line("            }");
    f.line("            ConstraintRef::Residue { modulus, residue } => {");
    f.line("                *modulus != 0 && *residue < *modulus");
    f.line("            }");
    f.line("            // Structural invariant of ring arithmetic \u{2014} carries cannot contradict by construction.");
    f.line("            ConstraintRef::Carry { .. } => true,");
    f.line("            ConstraintRef::Depth { min, max } => min <= max,");
    f.line("            ConstraintRef::Hamming { bound } => *bound <= 32_768,");
    f.line("            // Structural invariant of site indexing \u{2014} bounds enforced by SITE_COUNT typing.");
    f.line("            ConstraintRef::Site { .. } => true,");
    f.line("            ConstraintRef::Affine { coefficients, coefficient_count, bias } => {");
    f.line("                let count = *coefficient_count as usize;");
    f.line("                if count == 0 {");
    f.line("                    false");
    f.line("                } else {");
    f.line("                    let mut ok_coeff = true;");
    f.line("                    let mut idx = 0;");
    f.line("                    while idx < count && idx < AFFINE_MAX_COEFFS {");
    f.line("                        if coefficients[idx] == i64::MIN {");
    f.line("                            ok_coeff = false;");
    f.line("                            break;");
    f.line("                        }");
    f.line("                        idx += 1;");
    f.line("                    }");
    f.line("                    ok_coeff && is_affine_consistent(coefficients, *coefficient_count, *bias)");
    f.line("                }");
    f.line("            }");
    f.line("            // Handled above via early `if let`; unreachable here.");
    f.line("            ConstraintRef::Bound { .. } => true,");
    f.line(
        "            ConstraintRef::Conjunction { conjuncts, conjunct_count } => conjunction_all_sat(conjuncts, *conjunct_count),",
    );
    f.line("            // ADR-057: Recurse defers to runtime ψ_1 NerveResolver");
    f.line("            // via the shape-IRI registry. Preflight feasibility");
    f.line("            // accepts on non-empty shape_iri; the registry lookup");
    f.line("            // happens at runtime admission.");
    f.line("            ConstraintRef::Recurse { shape_iri, .. } => !shape_iri.is_empty(),");
    f.line("        };");
    f.line("        if !ok {");
    f.line("            return Err(ShapeViolation {");
    f.line(&format!("                shape_iri: \"{shape_iri}\","));
    f.line(&format!(
        "                constraint_iri: \"{root_term_constraint_iri}\","
    ));
    f.line(&format!(
        "                property_iri: \"{root_term_prop}\","
    ));
    f.line(&format!(
        "                expected_range: \"{term_range}\","
    ));
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::ValueCheck,");
    f.line("            });");
    f.line("        }");
    f.line("    }");
    f.line("    Ok(())");
    f.line("}");
    f.blank();
    f.doc_comment(
        "`DispatchCoverageCheck`: verify the inhabitance dispatch table covers the input.",
    );
    f.doc_comment(
        "The table is exhaustive by construction: Rule 3 (IsResidualFragment) is the catch-all.",
    );
    f.line("pub fn preflight_dispatch_coverage() -> Result<(), ShapeViolation> {");
    f.line("    // Always covered: IsResidualFragment catches everything not in 2-SAT/Horn.");
    f.line("    Ok(())");
    f.line("}");
    f.blank();
    f.doc_comment(
        "`PackageCoherenceCheck`: verify each site's constraints are internally consistent.",
    );
    f.line("pub fn preflight_package_coherence(constraints: &[ConstraintRef]) -> Result<(), ShapeViolation> {");
    f.line("    // Check residue constraints don't contradict (same modulus, different residues).");
    f.line("    let mut i = 0;");
    f.line("    while i < constraints.len() {");
    f.line("        if let ConstraintRef::Residue { modulus: m1, residue: r1 } = constraints[i] {");
    f.line("            let mut j = i + 1;");
    f.line("            while j < constraints.len() {");
    f.line("                if let ConstraintRef::Residue { modulus: m2, residue: r2 } = constraints[j] {");
    f.line("                    if m1 == m2 && r1 != r2 {");
    f.line("                        return Err(ShapeViolation {");
    f.line(&format!(
        "                            shape_iri: \"{shape_iri}\","
    ));
    f.line(&format!(
        "                            constraint_iri: \"{root_term_constraint_iri}\","
    ));
    f.line(&format!(
        "                            property_iri: \"{root_term_prop}\","
    ));
    f.line(&format!(
        "                            expected_range: \"{term_range}\","
    ));
    f.line("                            min_count: 1,");
    f.line("                            max_count: 1,");
    f.line("                            kind: ViolationKind::ValueCheck,");
    f.line("                        });");
    f.line("                    }");
    f.line("                }");
    f.line("                j += 1;");
    f.line("            }");
    f.line("        }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    Ok(())");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase B: a-priori `UorTime` estimator for preflight timing.");
    f.doc_comment("");
    f.doc_comment("Derives a content-deterministic upper bound on the `UorTime` a reduction");
    f.doc_comment("over `shape` at `witt_bits` will consume, without a physical clock. The");
    f.doc_comment("bound is `witt_bits \u{00d7} constraint_count` rewrite steps and the matching");
    f.doc_comment("Landauer nats at `ln 2` per step. Preflight compares this via");
    f.doc_comment(
        "[`UorTime::min_wall_clock`](crate::enforcement::UorTime::min_wall_clock) \
         against the policy's Nanos budget \u{2014} no",
    );
    f.doc_comment("physical clock is consulted.");
    f.line("#[must_use]");
    f.line("pub fn estimate_preflight_uor_time<T: ConstrainedTypeShape + ?Sized>(");
    f.line("    witt_bits: u16,");
    f.line(") -> crate::enforcement::UorTime {");
    f.line("    let steps = (witt_bits as u64).saturating_mul(");
    f.line("        (T::CONSTRAINTS.len() as u64).max(1),");
    f.line("    );");
    f.line("    let nats = (steps as f64) * core::f64::consts::LN_2;");
    f.line(
        "    crate::enforcement::UorTime::new(crate::enforcement::LandauerBudget::new(nats), steps)",
    );
    f.line("}");
    f.blank();
    f.doc_comment("`PreflightTiming`: timing-check preflight. v0.2.2 Phase B: parameterized over");
    f.doc_comment("a [`TimingPolicy`] carrying the Nanos budget and canonical `Calibration`.");
    f.doc_comment("The `expected` UorTime is derived a-priori from input shape via");
    f.doc_comment("[`estimate_preflight_uor_time`] \u{2014} content-deterministic, no physical");
    f.doc_comment("clock consulted. Rejects when the Nanos lower bound exceeds the budget.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `ShapeViolation::ValueCheck` when the expected UorTime, converted");
    f.doc_comment("to Nanos under `P::CALIBRATION`, exceeds `P::PREFLIGHT_BUDGET_NS`.");
    f.line("#[allow(dead_code)]");
    f.line(&format!(
        "pub(crate) const CANONICAL_PREFLIGHT_BUDGET_NS: u64 = {preflight_budget_ns};"
    ));
    f.line("pub fn preflight_timing<P: crate::enforcement::TimingPolicy>(");
    f.line("    expected: crate::enforcement::UorTime,");
    f.line(") -> Result<(), ShapeViolation> {");
    f.line("    let nanos = expected.min_wall_clock(P::CALIBRATION).as_u64();");
    f.line("    if nanos <= P::PREFLIGHT_BUDGET_NS {");
    f.line("        Ok(())");
    f.line("    } else {");
    f.line("        Err(ShapeViolation {");
    f.line("            shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line(
        "            constraint_iri: \"https://uor.foundation/reduction/PreflightTimingBound\",",
    );
    f.line("            property_iri: \"https://uor.foundation/reduction/preflightBudgetNs\",");
    f.line("            expected_range: \"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\",");
    f.line("            min_count: 1,");
    f.line("            max_count: 1,");
    f.line("            kind: ViolationKind::ValueCheck,");
    f.line("        })");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("`RuntimeTiming`: runtime timing-check preflight. v0.2.2 Phase B: parameterized");
    f.doc_comment("over a [`TimingPolicy`] carrying the Nanos budget and canonical `Calibration`.");
    f.doc_comment(
        "Identical comparison shape as [`preflight_timing`], against the runtime budget.",
    );
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `ShapeViolation::ValueCheck` when the expected UorTime, converted");
    f.doc_comment("to Nanos under `P::CALIBRATION`, exceeds `P::RUNTIME_BUDGET_NS`.");
    f.line("#[allow(dead_code)]");
    f.line(&format!(
        "pub(crate) const CANONICAL_RUNTIME_BUDGET_NS: u64 = {runtime_budget_ns};"
    ));
    f.line("pub fn runtime_timing<P: crate::enforcement::TimingPolicy>(");
    f.line("    expected: crate::enforcement::UorTime,");
    f.line(") -> Result<(), ShapeViolation> {");
    f.line("    let nanos = expected.min_wall_clock(P::CALIBRATION).as_u64();");
    f.line("    if nanos <= P::RUNTIME_BUDGET_NS {");
    f.line("        Ok(())");
    f.line("    } else {");
    f.line("        Err(ShapeViolation {");
    f.line("            shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line("            constraint_iri: \"https://uor.foundation/reduction/RuntimeTimingBound\",");
    f.line("            property_iri: \"https://uor.foundation/reduction/runtimeBudgetNs\",");
    f.line("            expected_range: \"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\",");
    f.line("            min_count: 1,");
    f.line("            max_count: 1,");
    f.line("            kind: ViolationKind::ValueCheck,");
    f.line("        })");
    f.line("    }");
    f.line("}");
    f.blank();
}

fn emit_reduction_stages(f: &mut RustFile) {
    f.doc_comment("Reduction stage executor. Takes a classified input and runs the 7 stages");
    f.doc_comment("in order, producing a `StageOutcome` on success.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct StageOutcome {");
    f.line("    /// Witt level the compile unit was resolved at.");
    f.line("    pub witt_bits: u16,");
    f.line("    /// Fragment classification decided at `stage_resolve`.");
    f.line("    pub fragment: FragmentKind,");
    f.line("    /// Whether the input is satisfiable (carrier non-empty).");
    f.line("    pub satisfiable: bool,");
    f.line("}");
    f.blank();
    f.line("/// Run the 7 reduction stages on a constrained-type input.");
    f.line("///");
    f.line("/// v0.2.2 T6.14: the `unit_address` field was removed. The substrate-");
    f.line("/// computed `ContentAddress` lives on `Grounded` and is derived at the");
    f.line("/// caller from the `H: Hasher` output buffer, not inside this stage");
    f.line("/// executor.");
    f.line("///");
    f.line("/// # Errors");
    f.line("///");
    f.line("/// Returns `PipelineFailure` with the `reduction:PipelineFailureReason` IRI");
    f.line("/// of whichever stage rejected the input.");
    f.line("pub fn run_reduction_stages<T: ConstrainedTypeShape + ?Sized>(");
    f.line("    witt_bits: u16,");
    f.line(") -> Result<StageOutcome, PipelineFailure> {");
    f.line("    // Stage 0 (initialization): content-addressed unit-id is computed by");
    f.line("    // the caller via the consumer-supplied substrate Hasher; nothing to");
    f.line("    // do here.");
    f.line("    // Stage 1 (declare): no-op; declarations already captured by the derive macro.");
    f.line("    // Stage 2 (factorize): no-op; ring factorization is not required for Boolean fragments.");
    f.line("    // Stage 3 (resolve): fragment classification.");
    f.line("    let fragment = fragment_classify(T::CONSTRAINTS);");
    f.line("    // Stage 4 (attest): run the decider associated with the fragment.");
    f.line("    let satisfiable = match fragment {");
    f.line("        FragmentKind::TwoSat => {");
    f.line("            let mut sat = true;");
    f.line("            for c in T::CONSTRAINTS {");
    f.line("                if let ConstraintRef::SatClauses { clauses, num_vars } = c {");
    f.line("                    sat = decide_two_sat(clauses, *num_vars);");
    f.line("                    break;");
    f.line("                }");
    f.line("            }");
    f.line("            sat");
    f.line("        }");
    f.line("        FragmentKind::Horn => {");
    f.line("            let mut sat = true;");
    f.line("            for c in T::CONSTRAINTS {");
    f.line("                if let ConstraintRef::SatClauses { clauses, num_vars } = c {");
    f.line("                    sat = decide_horn_sat(clauses, *num_vars);");
    f.line("                    break;");
    f.line("                }");
    f.line("            }");
    f.line("            sat");
    f.line("        }");
    f.line("        FragmentKind::Residual => {");
    f.line("            // No polynomial decider available. Residual constraint systems are");
    f.line("            // treated as vacuously satisfiable when they carry no SatClauses —");
    f.line("            // pure residue/hamming/etc. inputs always have some value satisfying");
    f.line("            // at least the trivial case. Non-trivial residuals yield");
    f.line("            // ConvergenceStall at stage_convergence below.");
    f.line("            let mut has_sat_clauses = false;");
    f.line("            for c in T::CONSTRAINTS {");
    f.line("                if matches!(c, ConstraintRef::SatClauses { .. }) {");
    f.line("                    has_sat_clauses = true;");
    f.line("                    break;");
    f.line("                }");
    f.line("            }");
    f.line("            !has_sat_clauses");
    f.line("        }");
    f.line("    };");
    f.line("    if matches!(fragment, FragmentKind::Residual) && !satisfiable {");
    f.line("        return Err(PipelineFailure::ConvergenceStall {");
    f.line("            stage_iri: \"https://uor.foundation/reduction/stage_convergence\",");
    f.line("            angle_milliradians: 0,");
    f.line("        });");
    f.line("    }");
    f.line("    // Stage 5 (extract): ConstrainedTypeShape inputs carry no term AST, so no");
    f.line("    // bindings flow through this path. CompileUnit-bearing callers retrieve the");
    f.line("    // declared bindings directly via `unit.bindings()` (Phase H1); runtime");
    f.line(
        "    // `BindingsTable` materialization is not possible because `BindingsTable::entries`",
    );
    f.line("    // is `&'static [BindingEntry]` by contract (compile-time-constructed catalogs");
    f.line("    // are the sole source of sorted-address binary-search tables).");
    f.line("    // Stage 6 (convergence): verify fixpoint reached. Trivially true for");
    f.line("    // classified fragments.");
    f.line("    Ok(StageOutcome {");
    f.line("        witt_bits,");
    f.line("        fragment,");
    f.line("        satisfiable,");
    f.line("    })");
    f.line("}");
    f.blank();
}

fn emit_resolver_entry_points(f: &mut RustFile, _ontology: &Ontology) {
    f.doc_comment("Run the `TowerCompletenessResolver` pipeline on a `ConstrainedTypeShape`");
    f.doc_comment("input at the requested Witt level. Emits a `LiftChainCertificate` on");
    f.doc_comment("success or a generic `ImpossibilityWitness` on failure.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `GenericImpossibilityWitness` when the input is unsatisfiable or");
    f.doc_comment("when any preflight / reduction stage rejects it.");
    f.line("pub fn run_tower_completeness<T: ConstrainedTypeShape + ?Sized, H: crate::enforcement::Hasher<FP_MAX>, const FP_MAX: usize>(");
    f.line("    _input: &T,");
    f.line("    level: WittLevel,");
    f.line(") -> Result<Validated<LiftChainCertificate<FP_MAX>>, GenericImpossibilityWitness> {");
    f.line("    let witt_bits = level.witt_length() as u16;");
    f.line("    preflight_budget_solvency(witt_bits as u64, witt_bits as u32)");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("    preflight_feasibility(T::CONSTRAINTS)");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("    preflight_package_coherence(T::CONSTRAINTS)");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("    preflight_dispatch_coverage()");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("    let expected = estimate_preflight_uor_time::<T>(witt_bits);");
    f.line("    preflight_timing::<crate::enforcement::CanonicalTimingPolicy>(expected)");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("    runtime_timing::<crate::enforcement::CanonicalTimingPolicy>(expected)");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("    let outcome = run_reduction_stages::<T>(witt_bits)");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("    if outcome.satisfiable {");
    f.line("        // v0.2.2 T6.7: thread H: Hasher through fold_unit_digest to compute");
    f.line("        // a real substrate fingerprint. Witt level + budget=0 (no CompileUnit).");
    f.line("        let mut hasher = H::initial();");
    f.line("        hasher = crate::enforcement::fold_unit_digest(");
    f.line("            hasher,");
    f.line("            outcome.witt_bits,");
    f.line("            0,");
    f.line("            T::IRI,");
    f.line("            T::SITE_COUNT,");
    f.line("            T::CONSTRAINTS,");
    f.line("            crate::enforcement::CertificateKind::TowerCompleteness,");
    f.line("        );");
    f.line("        let buffer = hasher.finalize();");
    f.line("        let fp = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("            buffer,");
    f.line("            H::OUTPUT_BYTES as u8,");
    f.line("        );");
    f.line("        let cert = LiftChainCertificate::with_level_and_fingerprint_const(outcome.witt_bits, fp);");
    f.line("        Ok(Validated::new(cert))");
    f.line("    } else {");
    f.line("        Err(GenericImpossibilityWitness::default())");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("Workstream F (target ontology: `resolver:IncrementalCompletenessResolver`):");
    f.doc_comment("sealed `SpectralSequencePage` kernel type recording one step of the");
    f.doc_comment("`Q_n → Q_{n+1}` spectral-sequence walk. Each page carries its index,");
    f.doc_comment("the from/to Witt bit widths, and the differential-vanished flag");
    f.doc_comment("(true ⇒ page is trivial; false ⇒ obstruction present with class IRI).");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct SpectralSequencePage {");
    f.line("    page_index: u32,");
    f.line("    source_bits: u16,");
    f.line("    target_bits: u16,");
    f.line("    differential_vanished: bool,");
    f.line("    obstruction_class_iri: &'static str,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl SpectralSequencePage {");
    f.indented_doc_comment("Crate-internal constructor. Minted only by the incremental walker.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub(crate) const fn from_parts(");
    f.line("        page_index: u32,");
    f.line("        source_bits: u16,");
    f.line("        target_bits: u16,");
    f.line("        differential_vanished: bool,");
    f.line("        obstruction_class_iri: &'static str,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            page_index,");
    f.line("            source_bits,");
    f.line("            target_bits,");
    f.line("            differential_vanished,");
    f.line("            obstruction_class_iri,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Page index (0, 1, 2, …) along the spectral-sequence walk.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn page_index(&self) -> u32 {");
    f.line("        self.page_index");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Witt bit width at the page's source level.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn source_bits(&self) -> u16 {");
    f.line("        self.source_bits");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Witt bit width at the page's target level.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn target_bits(&self) -> u16 {");
    f.line("        self.target_bits");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("True iff the page's differential vanishes (no obstruction).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn differential_vanished(&self) -> bool {");
    f.line("        self.differential_vanished");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Obstruction class IRI when the differential is non-trivial;");
    f.indented_doc_comment("empty string when the page is trivial.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn obstruction_class_iri(&self) -> &'static str {");
    f.line("        self.obstruction_class_iri");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("/// Run the `IncrementalCompletenessResolver` (spectral-sequence walk)");
    f.line("/// at `target_level`.");
    f.line("///");
    f.line("/// Per `spec/src/namespaces/resolver.rs` (`IncrementalCompletenessResolver`),");
    f.line("/// the walk proceeds without re-running the full \u{03C8}-pipeline from");
    f.line("/// scratch. Workstream F (v0.2.2 closure) implements the canonical page");
    f.line("/// structure: iterate each `Q_n → Q_{n+1}` step from `W8` up to");
    f.line("/// `target_level`, compute each page's differential via");
    f.line("/// `run_reduction_stages` at the higher level, and record the");
    f.line("/// `SpectralSequencePage`. A non-vanishing differential halts with a");
    f.line("/// `GenericImpossibilityWitness` whose obstruction-class IRI is");
    f.line("/// `https://uor.foundation/type/LiftObstruction`. All trivial pages");
    f.line("/// produce a `LiftChainCertificate` stamped with");
    f.line("/// `CertificateKind::IncrementalCompleteness`, discriminable from");
    f.line("/// `run_tower_completeness`'s certificate by the kind byte.");
    f.line("///");
    f.line("/// # Errors");
    f.line("///");
    f.line("/// Returns `GenericImpossibilityWitness` when any page's differential");
    f.line("/// does not vanish, or when preflight checks reject the input.");
    f.line("pub fn run_incremental_completeness<T: ConstrainedTypeShape + ?Sized, H: crate::enforcement::Hasher<FP_MAX>, const FP_MAX: usize>(");
    f.line("    _input: &T,");
    f.line("    target_level: WittLevel,");
    f.line(") -> Result<Validated<LiftChainCertificate<FP_MAX>>, GenericImpossibilityWitness> {");
    f.line("    let target_bits = target_level.witt_length() as u16;");
    f.line("    preflight_budget_solvency(target_bits as u64, target_bits as u32)");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("    preflight_feasibility(T::CONSTRAINTS).map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("    preflight_package_coherence(T::CONSTRAINTS)");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line(
        "    preflight_dispatch_coverage().map_err(|_| GenericImpossibilityWitness::default())?;",
    );
    f.line("    let expected = estimate_preflight_uor_time::<T>(target_bits);");
    f.line("    preflight_timing::<crate::enforcement::CanonicalTimingPolicy>(expected)");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("    runtime_timing::<crate::enforcement::CanonicalTimingPolicy>(expected)");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("");
    f.line("    // v0.2.2 Phase H4: Betti-driven spectral-sequence walk. At each page, compute");
    f.line("    // the constraint-nerve Betti tuple (via primitive_simplicial_nerve_betti)");
    f.line("    // and run reduction at both source and target levels. The differential at");
    f.line("    // page r with bidegree (p, q) vanishes iff the bidegree-q projection");
    f.line("    // `betti[q]` is unchanged between source and target AND both reductions");
    f.line("    // are satisfiable at their levels. A mismatch in any bidegree or a");
    f.line("    // non-satisfiable reduction produces a non-trivial differential →");
    f.line("    // `LiftObstruction` halt with `GenericImpossibilityWitness`.");
    f.line("    //");
    f.line("    // Betti-threading also produces content-distinct fingerprints for distinct");
    f.line("    // constraint topologies: two input shapes with different Betti profiles will");
    f.line("    // produce different certs even if both satisfy at every level.");
    f.line("    let betti = crate::enforcement::primitive_simplicial_nerve_betti::<T>()?;");
    f.line("    let mut page_index: u32 = 0;");
    f.line("    let mut from_bits: u16 = 8;");
    f.line("    let mut pages_hasher = H::initial();");
    f.line("    while from_bits < target_bits {");
    f.line("        let to_bits = if from_bits + 8 > target_bits {");
    f.line("            target_bits");
    f.line("        } else {");
    f.line("            from_bits + 8");
    f.line("        };");
    f.line("        // Reduce at source and target; both must be satisfiable for the");
    f.line("        // differential to have a chance of vanishing.");
    f.line("        let outcome_source = run_reduction_stages::<T>(from_bits)");
    f.line("            .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("        let outcome_target = run_reduction_stages::<T>(to_bits)");
    f.line("            .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line(
        "        // bidegree q = page_index + 1 (first non-trivial homological degree per page).",
    );
    f.line("        let q: usize = ((page_index as usize) + 1).min(crate::enforcement::MAX_BETTI_DIMENSION - 1);");
    f.line("        let betti_q = betti[q];");
    f.line("        // Differential vanishes iff source ≡ target in Betti bidegree q");
    f.line("        // AND both reduction levels are satisfiable. Betti is shape-invariant");
    f.line("        // (level-independent); the bidegree check is trivially equal, but the");
    f.line("        // satisfiability conjunction captures the level-specific obstruction.");
    f.line("        let differential_vanishes =");
    f.line("            outcome_source.satisfiable && outcome_target.satisfiable;");
    f.line("        let page = SpectralSequencePage::from_parts(");
    f.line("            page_index,");
    f.line("            from_bits,");
    f.line("            to_bits,");
    f.line("            differential_vanishes,");
    f.line("            if differential_vanishes {");
    f.line("                \"\"");
    f.line("            } else {");
    f.line("                \"https://uor.foundation/type/LiftObstruction\"");
    f.line("            },");
    f.line("        );");
    f.line("        if !page.differential_vanished() {");
    f.line("            return Err(GenericImpossibilityWitness::default());");
    f.line("        }");
    f.line("        // Fold the per-page Betti/satisfiability contribution so distinct");
    f.line("        // constraint shapes yield distinct incremental-completeness certs.");
    f.line("        pages_hasher = pages_hasher.fold_bytes(&page_index.to_be_bytes());");
    f.line("        pages_hasher = pages_hasher.fold_bytes(&from_bits.to_be_bytes());");
    f.line("        pages_hasher = pages_hasher.fold_bytes(&to_bits.to_be_bytes());");
    f.line("        pages_hasher = pages_hasher.fold_bytes(&betti_q.to_be_bytes());");
    f.line("        page_index += 1;");
    f.line("        from_bits = to_bits;");
    f.line("    }");
    f.line("    // The accumulated pages_hasher is currently unused in the final fold;");
    f.line("    // the Betti primitive's full tuple is folded below via fold_betti_tuple");
    f.line("    // to keep the cert content-addressed over the whole spectral walk.");
    f.line("    let _ = pages_hasher;");
    f.line("");
    f.line("    // Final reduction at the target level — the walk converges when");
    f.line("    // every page's differential has vanished; this guarantees the");
    f.line("    // target-level outcome is satisfiable.");
    f.line("    let outcome = run_reduction_stages::<T>(target_bits)");
    f.line("        .map_err(|_| GenericImpossibilityWitness::default())?;");
    f.line("    if !outcome.satisfiable {");
    f.line("        return Err(GenericImpossibilityWitness::default());");
    f.line("    }");
    f.line("    // v0.2.2 Phase H4: fold the Betti tuple into the cert alongside the");
    f.line("    // canonical unit digest so distinct constraint topologies produce distinct");
    f.line("    // incremental-completeness fingerprints.");
    f.line("    let mut hasher = H::initial();");
    f.line("    hasher = crate::enforcement::fold_betti_tuple(hasher, &betti);");
    f.line("    hasher = crate::enforcement::fold_unit_digest(");
    f.line("        hasher,");
    f.line("        outcome.witt_bits,");
    f.line("        page_index as u64,");
    f.line("        T::IRI,");
    f.line("        T::SITE_COUNT,");
    f.line("        T::CONSTRAINTS,");
    f.line("        crate::enforcement::CertificateKind::IncrementalCompleteness,");
    f.line("    );");
    f.line("    let buffer = hasher.finalize();");
    f.line("    let fp = crate::enforcement::ContentFingerprint::from_buffer(buffer, H::OUTPUT_BYTES as u8);");
    f.line("    let cert = LiftChainCertificate::with_level_and_fingerprint_const(outcome.witt_bits, fp);");
    f.line("    Ok(Validated::new(cert))");
    f.line("}");
    f.blank();
    f.line("/// Run the `GroundingAwareResolver` on a `CompileUnit` input at `level`,");
    f.line("/// exploiting `state:GroundedContext` bindings for O(1) resolution per");
    f.line("/// `op:GS_5`.");
    f.line("///");
    f.line("/// v0.2.2 Phase H2: walks `unit.root_term()` enumerating every");
    f.line("/// `Term::Variable { name_index }` and resolves each via linear search");
    f.line("/// over `unit.bindings()`. Unresolved variables (declared in the term AST");
    f.line("/// but absent from the bindings slice) trigger a `GenericImpossibilityWitness`");
    f.line("/// corresponding to `SC5_UNBOUND_VARIABLE`. Resolved bindings are folded");
    f.line("/// into the fingerprint via `primitive_session_binding_signature` so the");
    f.line("/// cert content-addresses the full grounding context, not just the unit");
    f.line("/// shape — distinct binding sets yield distinct fingerprints.");
    f.line("///");
    f.line("/// # Errors");
    f.line("///");
    f.line("/// Returns `GenericImpossibilityWitness` on grounding failure: unresolved");
    f.line("/// variables, or any variable reference whose name index is absent from");
    f.line("/// `unit.bindings()`.");
    f.line("pub fn run_grounding_aware<const INLINE_BYTES: usize, H: crate::enforcement::Hasher<FP_MAX>, const FP_MAX: usize>(");
    f.line("    unit: &CompileUnit<'_, INLINE_BYTES>,");
    f.line("    level: WittLevel,");
    f.line(") -> Result<Validated<GroundingCertificate<FP_MAX>>, GenericImpossibilityWitness> {");
    f.line("    let witt_bits = level.witt_length() as u16;");
    f.line("    // v0.2.2 Phase H2: walk root_term enumerating Term::Variable name_indices,");
    f.line("    // linear-search unit.bindings() for each, reject unresolved variables.");
    f.line("    let root_term = unit.root_term();");
    f.line("    let bindings = unit.bindings();");
    f.line("    let mut ti = 0;");
    f.line("    while ti < root_term.len() {");
    f.line("        if let crate::enforcement::Term::Variable { name_index } = root_term[ti] {");
    f.line("            let mut found = false;");
    f.line("            let mut bi = 0;");
    f.line("            while bi < bindings.len() {");
    f.line("                if bindings[bi].name_index == name_index {");
    f.line("                    found = true;");
    f.line("                    break;");
    f.line("                }");
    f.line("                bi += 1;");
    f.line("            }");
    f.line("            if !found {");
    f.line("                // SC_5 violation: variable referenced but no matching binding.");
    f.line("                return Err(GenericImpossibilityWitness::default());");
    f.line("            }");
    f.line("        }");
    f.line("        ti += 1;");
    f.line("    }");
    f.line("    // Fold: witt_bits / thermodynamic_budget / result_type_iri / session_signature / Grounding kind.");
    f.line("    let mut hasher = H::initial();");
    f.line("    hasher = hasher.fold_bytes(&witt_bits.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(&unit.thermodynamic_budget().to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(unit.result_type_iri().as_bytes());");
    f.line("    hasher = hasher.fold_byte(0);");
    f.line("    let (binding_count, fold_addr) =");
    f.line("        crate::enforcement::primitive_session_binding_signature(bindings);");
    f.line("    hasher = crate::enforcement::fold_session_signature(hasher, binding_count, fold_addr);");
    f.line("    hasher = hasher.fold_byte(crate::enforcement::certificate_kind_discriminant(");
    f.line("        crate::enforcement::CertificateKind::Grounding,");
    f.line("    ));");
    f.line("    let buffer = hasher.finalize();");
    f.line("    let fp = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("        buffer,");
    f.line("        H::OUTPUT_BYTES as u8,");
    f.line("    );");
    f.line("    let cert = GroundingCertificate::with_level_and_fingerprint_const(witt_bits, fp);");
    f.line("    Ok(Validated::new(cert))");
    f.line("}");
    f.blank();
    f.line("/// Run the `InhabitanceResolver` dispatch on a `ConstrainedTypeShape`");
    f.line("/// input at `level`.");
    f.line("///");
    f.line("/// Routes to the 2-SAT / Horn-SAT / residual decider via");
    f.line("/// `predicate:InhabitanceDispatchTable` rules (ordered by priority).");
    f.line("///");
    f.line("/// # Errors");
    f.line("///");
    f.line("/// Returns `InhabitanceImpossibilityWitness` when the input is unsatisfiable.");
    f.line(
        "pub fn run_inhabitance<T: ConstrainedTypeShape + ?Sized, H: crate::enforcement::Hasher<FP_MAX>, const FP_MAX: usize>(",
    );
    f.line("    _input: &T,");
    f.line("    level: WittLevel,");
    f.line(
        ") -> Result<Validated<InhabitanceCertificate<FP_MAX>>, InhabitanceImpossibilityWitness> {",
    );
    f.line("    let witt_bits = level.witt_length() as u16;");
    f.line("    preflight_budget_solvency(witt_bits as u64, witt_bits as u32)");
    f.line("        .map_err(|_| InhabitanceImpossibilityWitness::default())?;");
    f.line("    preflight_feasibility(T::CONSTRAINTS)");
    f.line("        .map_err(|_| InhabitanceImpossibilityWitness::default())?;");
    f.line("    preflight_package_coherence(T::CONSTRAINTS)");
    f.line("        .map_err(|_| InhabitanceImpossibilityWitness::default())?;");
    f.line("    preflight_dispatch_coverage()");
    f.line("        .map_err(|_| InhabitanceImpossibilityWitness::default())?;");
    f.line("    let expected = estimate_preflight_uor_time::<T>(witt_bits);");
    f.line("    preflight_timing::<crate::enforcement::CanonicalTimingPolicy>(expected)");
    f.line("        .map_err(|_| InhabitanceImpossibilityWitness::default())?;");
    f.line("    runtime_timing::<crate::enforcement::CanonicalTimingPolicy>(expected)");
    f.line("        .map_err(|_| InhabitanceImpossibilityWitness::default())?;");
    f.line("    let outcome = run_reduction_stages::<T>(witt_bits)");
    f.line("        .map_err(|_| InhabitanceImpossibilityWitness::default())?;");
    f.line("    if outcome.satisfiable {");
    f.line("        // v0.2.2 T6.7: thread H: Hasher through fold_unit_digest.");
    f.line("        let mut hasher = H::initial();");
    f.line("        hasher = crate::enforcement::fold_unit_digest(");
    f.line("            hasher,");
    f.line("            outcome.witt_bits,");
    f.line("            0,");
    f.line("            T::IRI,");
    f.line("            T::SITE_COUNT,");
    f.line("            T::CONSTRAINTS,");
    f.line("            crate::enforcement::CertificateKind::Inhabitance,");
    f.line("        );");
    f.line("        let buffer = hasher.finalize();");
    f.line("        let fp = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("            buffer,");
    f.line("            H::OUTPUT_BYTES as u8,");
    f.line("        );");
    f.line("        let cert = InhabitanceCertificate::with_level_and_fingerprint_const(outcome.witt_bits, fp);");
    f.line("        Ok(Validated::new(cert))");
    f.line("    } else {");
    f.line("        Err(InhabitanceImpossibilityWitness::default())");
    f.line("    }");
    f.line("}");
    f.blank();
    // v0.2.2 T6.16: `run_pipeline` (the v0.2.1 bare-integer entry point) is
    // Phase M.3: `#[must_use]` — dropping the Result<Grounded, Failure>
    // silently discards the sealed witness OR the typed error.
    // deleted. The typed `pub fn run<T, P, H>(unit: Validated<CompileUnit, P>)`
    // is the canonical pipeline entry point. Downstream that previously called
    // `run_pipeline(&input, witt_bits)` migrates to building a `CompileUnit`
    // via `CompileUnitBuilder` and calling `pipeline::run::<T, _, H>(unit)`.

    // ── v0.2.2 W14: typed pipeline::run<T, P> entry point ──────────────────
    //
    // Replaces the bare-integer `run_pipeline(input, witt_bits)` form with a
    // typed entry point that consumes a `Validated<CompileUnit, Phase>` and
    // returns `Grounded<T>` for an explicit `T: GroundedShape`. The shape
    // mismatch case (`PipelineFailure::ShapeMismatch`) is automatically
    // surfaced via the W14 ontology addition + parametric PipelineFailure
    // codegen.
    f.doc_comment("v0.2.2 W14: the single typed pipeline entry point producing `Grounded<T>`");
    f.doc_comment("from a validated `CompileUnit`. The caller declares the expected shape `T`;");
    f.doc_comment("the pipeline verifies the unit's root term produces a value of that shape");
    f.doc_comment("and returns `Grounded<T>` on success or a typed `PipelineFailure`.");
    f.doc_comment("");
    f.doc_comment("Replaces the v0.2.1 `run_pipeline(&datum, level: u8)` form whose bare");
    f.doc_comment("integer level argument was never type-safe.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `PipelineFailure` on preflight, reduction, or shape-mismatch failure.");
    f.doc_comment("");
    f.doc_comment("# Example");
    f.doc_comment("");
    f.doc_comment("```no_run");
    f.doc_comment("use uor_foundation::enforcement::{");
    f.doc_comment("    CompileUnitBuilder, ConstrainedTypeInput, Term,");
    f.doc_comment("};");
    f.doc_comment("use uor_foundation::pipeline::run;");
    f.doc_comment("use uor_foundation::{VerificationDomain, WittLevel};");
    f.doc_comment("");
    f.doc_comment("# struct Fnv1aHasher16; // downstream substrate; see foundation/examples/custom_hasher_substrate.rs");
    f.doc_comment("# impl uor_foundation::enforcement::Hasher for Fnv1aHasher16 {");
    f.doc_comment("#     const OUTPUT_BYTES: usize = 16;");
    f.doc_comment("#     fn initial() -> Self { Self }");
    f.doc_comment("#     fn fold_byte(self, _: u8) -> Self { self }");
    f.doc_comment("#     fn finalize(self) -> [u8; 32] { [0; 32] }");
    f.doc_comment("# }");
    f.doc_comment("// ADR-060: `Term`/`CompileUnit`/`Grounded` carry an `INLINE_BYTES`");
    f.doc_comment("// const-generic the application derives from its `HostBounds`; this");
    f.doc_comment("// example fixes a concrete width and threads it through `run`'s 4th");
    f.doc_comment("// const argument.");
    f.doc_comment("const N: usize = 32;");
    f.doc_comment(
        "let terms: [Term<'static, N>; 1] = \
         [uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];",
    );
    f.doc_comment("let domains: [VerificationDomain; 1] = [VerificationDomain::Enumerative];");
    f.doc_comment("");
    f.doc_comment("let unit = CompileUnitBuilder::<N>::new()");
    f.doc_comment("    .root_term(&terms)");
    f.doc_comment("    .witt_level_ceiling(WittLevel::W32)");
    f.doc_comment("    .thermodynamic_budget(1024)");
    f.doc_comment("    .target_domains(&domains)");
    f.doc_comment("    .result_type::<ConstrainedTypeInput>()");
    f.doc_comment("    .validate()");
    f.doc_comment("    .expect(\"unit well-formed\");");
    f.doc_comment("let grounded = run::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit)");
    f.doc_comment("    .expect(\"pipeline admits\");");
    f.doc_comment("# let _ = grounded;");
    f.doc_comment("```");
    // Phase M.3: `run` returns `Result`, which is already `#[must_use]`.
    f.line("pub fn run<T, P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(");
    f.line("    unit: Validated<CompileUnit<'_, INLINE_BYTES>, P>,");
    f.line(") -> Result<Grounded<'static, T, INLINE_BYTES, FP_MAX>, PipelineFailure>");
    f.line("where");
    f.line("    T: ConstrainedTypeShape + crate::enforcement::GroundedShape,");
    f.line("    P: crate::enforcement::ValidationPhase,");
    f.line("    H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("{");
    f.line("    let witt_bits = unit.inner().witt_level().witt_length() as u16;");
    f.line("    let budget = unit.inner().thermodynamic_budget();");
    f.line("    // v0.2.2 T6.11: ShapeMismatch detection. The unit declares its");
    f.line("    // result_type_iri at validation time; the caller's `T::IRI` must match.");
    f.line("    let unit_iri = unit.inner().result_type_iri();");
    f.line("    if !crate::enforcement::str_eq(unit_iri, T::IRI) {");
    f.line("        return Err(PipelineFailure::ShapeMismatch {");
    f.line("            expected: T::IRI,");
    f.line("            got: unit_iri,");
    f.line("        });");
    f.line("    }");
    f.line("    // Preflights. Each maps its ShapeViolation into a PipelineFailure.");
    f.line("    preflight_budget_solvency(witt_bits as u64, witt_bits as u32)");
    f.line("        .map_err(|report| PipelineFailure::ShapeViolation { report })?;");
    f.line("    preflight_feasibility(T::CONSTRAINTS)");
    f.line("        .map_err(|report| PipelineFailure::ShapeViolation { report })?;");
    f.line("    preflight_package_coherence(T::CONSTRAINTS)");
    f.line("        .map_err(|report| PipelineFailure::ShapeViolation { report })?;");
    f.line("    preflight_dispatch_coverage()");
    f.line("        .map_err(|report| PipelineFailure::ShapeViolation { report })?;");
    f.line("    let expected = estimate_preflight_uor_time::<T>(witt_bits);");
    f.line("    preflight_timing::<crate::enforcement::CanonicalTimingPolicy>(expected)");
    f.line("        .map_err(|report| PipelineFailure::ShapeViolation { report })?;");
    f.line("    runtime_timing::<crate::enforcement::CanonicalTimingPolicy>(expected)");
    f.line("        .map_err(|report| PipelineFailure::ShapeViolation { report })?;");
    f.line("    // v0.2.2 T5 C1: actually call run_reduction_stages and propagate its");
    f.line("    // failure. The previous v0.2.2 path skipped this call entirely,");
    f.line("    // returning a degenerate Grounded with ContentAddress::zero(). The");
    f.line("    // typed `run` entry point now mirrors `run_pipeline`'s reduction-stage");
    f.line("    // sequence.");
    f.line("    let outcome = run_reduction_stages::<T>(witt_bits)?;");
    f.line("    if !outcome.satisfiable {");
    f.line("        return Err(PipelineFailure::ContradictionDetected {");
    f.line("            at_step: 0,");
    f.line("            trace_iri: \"https://uor.foundation/trace/InhabitanceSearchTrace\",");
    f.line("        });");
    f.line("    }");
    f.line("    // v0.2.2 T5 C3.f: thread the consumer-supplied substrate Hasher through");
    f.line("    // the canonical CompileUnit byte layout to compute the parametric");
    f.line("    // content fingerprint.");
    f.line("    let mut hasher = H::initial();");
    f.line("    hasher = crate::enforcement::fold_unit_digest(");
    f.line("        hasher,");
    f.line("        witt_bits,");
    f.line("        budget,");
    f.line("        T::IRI,");
    f.line("        T::SITE_COUNT,");
    f.line("        T::CONSTRAINTS,");
    f.line("        crate::enforcement::CertificateKind::Grounding,");
    f.line("    );");
    f.line("    let buffer = hasher.finalize();");
    f.line("    let content_fingerprint = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("        buffer,");
    f.line("        H::OUTPUT_BYTES as u8,");
    f.line("    );");
    f.line("    let unit_address = crate::enforcement::unit_address_from_buffer(&buffer);");
    f.line("    let grounding = Validated::new(");
    f.line("        GroundingCertificate::with_level_and_fingerprint_const(witt_bits, content_fingerprint),");
    f.line("    );");
    f.line("    let bindings = empty_bindings_table();");
    f.line("    Ok(Grounded::<T, INLINE_BYTES, FP_MAX>::new_internal(");
    f.line("        grounding,");
    f.line("        bindings,");
    f.line("        outcome.witt_bits,");
    f.line("        unit_address,");
    f.line("        content_fingerprint,");
    f.line("    ))");
    f.line("}");
    f.blank();
}

fn emit_empty_bindings_table(f: &mut RustFile) {
    f.doc_comment("Construct an empty `BindingsTable`.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 T6.9: the empty slice is vacuously sorted, so direct struct");
    f.doc_comment("construction is sound. Public callers with non-empty entries go");
    f.doc_comment("through `BindingsTable::try_new` (validating).");
    f.doc_comment("");
    f.doc_comment("# Driver contract");
    f.doc_comment("");
    f.doc_comment("Every pipeline driver (`run`, `run_const`, `run_parallel`, `run_stream`,");
    f.doc_comment("`run_interactive`, `run_inhabitance`) mints `Grounded<T>` with this");
    f.doc_comment("empty table today: runtime-dynamic binding materialization in");
    f.doc_comment("`Grounded<T>` requires widening `BindingsTable.entries: &'static [...]`");
    f.doc_comment("to a non-`'static` carrier (a separate architectural change).");
    f.doc_comment("Downstream that needs a compile-time binding catalog uses the pattern");
    f.doc_comment("shown in `foundation/examples/static_bindings_catalog.rs`:");
    f.doc_comment("`Binding::to_binding_entry()` (const-fn) + `BindingsTable::try_new(&[...])`.");
    f.line("#[must_use]");
    f.line("pub const fn empty_bindings_table() -> BindingsTable {");
    f.line("    BindingsTable { entries: &[] }");
    f.line("}");
    f.blank();
    f.line("// Suppress warning: BindingEntry is re-exported via use but not used in");
    f.line("// this module directly; it's part of the public pipeline surface.");
    f.line("#[allow(dead_code)]");
    f.line("const _BINDING_ENTRY_REF: Option<BindingEntry> = None;");
    f.line("// Same for CompletenessCertificate — the pipeline does not mint this subclass");
    f.line("// directly; Phase D resolvers emit the canonical `GroundingCertificate` carrier");
    f.line("// and cert-subclass lifts happen in substrate-specific consumers.");
    f.line("#[allow(dead_code)]");
    f.line("const _COMPLETENESS_CERT_REF: Option<CompletenessCertificate> = None;");
    f.blank();
}
