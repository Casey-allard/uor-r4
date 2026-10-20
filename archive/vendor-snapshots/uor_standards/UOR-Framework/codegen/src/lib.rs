//! UOR Foundation code generator.
//!
//! Reads the ontology from `uor_ontology::Ontology::full()` and generates the
//! `uor-foundation` Rust trait crate. The generated crate exports every ontology
//! class as a trait, every property as a method, and every named individual as a
//! constant — giving PRISM and other implementations a well-defined Rust interface.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

pub mod classification;
pub mod emit;
pub mod enforcement;
pub mod enums;
pub mod individuals;
pub mod mapping;
pub mod pipeline;
pub mod resolved_wrapper;
pub mod sdk_macros;
pub mod traits;
pub mod witness_scaffolds;

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::path::Path;

use anyhow::Result;
use uor_ontology::model::Space;
use uor_ontology::{Ontology, Property};

use emit::RustFile;
use mapping::namespace_mappings;

/// Report of what was generated.
#[derive(Debug, Default)]
pub struct GenerationReport {
    /// Number of traits generated.
    pub trait_count: usize,
    /// Number of methods generated.
    pub method_count: usize,
    /// Number of enums generated.
    pub enum_count: usize,
    /// Number of individual constants generated.
    pub const_count: usize,
    /// Files written.
    pub files: Vec<String>,
}

/// Generates the complete `uor-foundation` crate source into `out_dir`,
/// and the companion `uor-foundation-sdk` proc-macro crate source into
/// `sdk_out`.
///
/// # Errors
///
/// Returns an error if any file cannot be written.
pub fn generate(ontology: &Ontology, out_dir: &Path, sdk_out: &Path) -> Result<GenerationReport> {
    let mut report = GenerationReport::default();
    let ns_map = namespace_mappings();

    // 1. Generate enums.rs
    let enums_content = enums::generate_enums_file(ontology);
    report.enum_count = enums::detect_enums(ontology).len();
    emit::write_file(&out_dir.join("enums.rs"), &enums_content)?;
    report.files.push("enums.rs".to_string());

    // 2. Generate per-namespace module files
    let mut kernel_modules = Vec::new();
    let mut bridge_modules = Vec::new();
    let mut user_modules = Vec::new();

    // Build cross-namespace property-by-domain lookup for inherited associated type detection.
    let all_props_by_domain: HashMap<&str, Vec<&Property>> = {
        let mut map: HashMap<&str, Vec<&Property>> = HashMap::new();
        for module in &ontology.namespaces {
            for prop in &module.properties {
                if let Some(domain) = prop.domain {
                    map.entry(domain).or_default().push(prop);
                }
            }
        }
        map
    };

    for module in &ontology.namespaces {
        let ns_iri = module.namespace.iri;
        let mapping = match ns_map.get(ns_iri) {
            Some(m) => m,
            None => continue,
        };

        let content = traits::generate_namespace_module(module, &ns_map, &all_props_by_domain);

        // Append PrimitiveOp impls to op.rs
        let content = if mapping.file_module == "op" {
            let op_impls = individuals::generate_primitive_op_impls(ontology);
            format!("{content}\n{op_impls}")
        } else {
            content
        };

        let file_path = out_dir
            .join(mapping.space_module)
            .join(format!("{}.rs", mapping.file_module));
        emit::write_file(&file_path, &content)?;

        let file_rel = format!("{}/{}.rs", mapping.space_module, mapping.file_module);
        report.files.push(file_rel);

        // Count traits and methods
        report.trait_count += module.classes.len();
        for prop in &module.properties {
            if prop.domain.is_some() && prop.kind != uor_ontology::PropertyKind::Annotation {
                report.method_count += 1;
            }
        }
        report.const_count += module
            .individuals
            .iter()
            .filter(|ind| {
                let t = mapping::local_name(ind.type_);
                !mapping::ENUM_VARIANT_CLASS_NAMES.contains(&t)
            })
            .count();

        match module.namespace.space {
            Space::Kernel => kernel_modules.push(mapping.file_module),
            Space::Bridge => bridge_modules.push(mapping.file_module),
            Space::User => user_modules.push(mapping.file_module),
        }
    }

    // 3. Generate space mod.rs files
    generate_mod_file(out_dir, "kernel", &kernel_modules, &mut report)?;
    generate_mod_file(out_dir, "bridge", &bridge_modules, &mut report)?;
    generate_mod_file(out_dir, "user", &user_modules, &mut report)?;

    // 4. Generate lib.rs
    let lib_content = generate_lib_rs(ontology);
    emit::write_file(&out_dir.join("lib.rs"), &lib_content)?;
    report.files.push("lib.rs".to_string());

    // 5. Generate README.md (written to parent of out_dir, i.e. foundation/)
    if let Some(crate_dir) = out_dir.parent() {
        let readme = generate_readme(ontology);
        emit::write_file(&crate_dir.join("README.md"), &readme)?;
    }

    // 6. Generate enforcement.rs (declarative enforcement types)
    let enforcement_content = enforcement::generate_enforcement_module(ontology);
    let enforcement_path = out_dir.join("enforcement.rs");
    emit::write_file(&enforcement_path, &enforcement_content)?;
    // Run rustfmt on the generated file to ensure it matches cargo fmt output.
    let _ = std::process::Command::new("rustfmt")
        .arg(&enforcement_path)
        .status();
    report.files.push("enforcement.rs".to_string());

    // 7. Generate pipeline.rs (v0.2.1 reduction pipeline driver)
    let pipeline_content = pipeline::generate_pipeline_module(ontology);
    let pipeline_path = out_dir.join("pipeline.rs");
    emit::write_file(&pipeline_path, &pipeline_content)?;
    let _ = std::process::Command::new("rustfmt")
        .arg(&pipeline_path)
        .status();
    report.files.push("pipeline.rs".to_string());

    // 7a-1. Phase 10 — VerifiedMint witness scaffolds for Path-2 classes.
    // Generated alongside enforcement.rs / pipeline.rs; emits the
    // `OntologyVerifiedMint` trait, one `Mint{Foo}` + `Mint{Foo}Inputs<H>` +
    // `Certificate` + `OntologyVerifiedMint` per Path-2 class, plus per-
    // family primitive stub modules under `foundation/src/primitives/`.
    let scaffolds_content = witness_scaffolds::generate_witness_scaffolds_module(ontology);
    let scaffolds_path = out_dir.join("witness_scaffolds.rs");
    emit::write_file(&scaffolds_path, &scaffolds_content)?;
    let _ = std::process::Command::new("rustfmt")
        .arg(&scaffolds_path)
        .status();
    report.files.push("witness_scaffolds.rs".to_string());

    for (relative_path, content) in witness_scaffolds::generate_primitives_modules(ontology) {
        let path = out_dir.join(&relative_path);
        emit::write_file(&path, &content)?;
        let _ = std::process::Command::new("rustfmt").arg(&path).status();
        report.files.push(relative_path);
    }

    // 7b. Classification report. Phase 0 of the orphan-closure plan: every
    // class in the ontology is classified into a `PathKind`, and the human-
    // readable report is written alongside the design notes under
    // docs/orphan-closure/. Regenerated on every `cargo run --bin uor-crate`;
    // `git diff --exit-code docs/orphan-closure/classification_report.md`
    // gates drift between the ontology and the classification.
    //
    // The report lives in the repo, not under `out_dir` (which points at
    // `foundation/src/`). We walk up from `out_dir` to the workspace root and
    // write the report relative to it. If the workspace root can't be found
    // (out_dir has no parent chain leading to a `docs/` dir), the step is a
    // no-op — the classification logic still runs and the test suite still
    // exercises it, but no file is written.
    if let Some(workspace_root) = find_workspace_root(out_dir) {
        let entries = classification::classify_all(ontology);
        let report_path = workspace_root
            .join("docs")
            .join("orphan-closure")
            .join("classification_report.md");
        classification::write_report(&entries, &report_path)?;
        report.files.push(format!("{}", report_path.display()));
    }

    // 8. Generate uor-foundation-sdk/src/lib.rs (Product/Coproduct Completion
    // Amendment Part B). The SDK is a proc-macro crate emitting
    // `product_shape!` / `coproduct_shape!` / `cartesian_product_shape!`
    // macros — its sources are derived from the same ontology snapshot
    // that produces foundation/, guaranteeing the two crates cannot drift
    // apart. Consumers who want pure-traits surface take only uor-foundation;
    // consumers who want macro ergonomics also depend on uor-foundation-sdk.
    let sdk_lib_content = sdk_macros::generate_sdk_lib();
    let sdk_lib_path = sdk_out.join("lib.rs");
    emit::write_file(&sdk_lib_path, &sdk_lib_content)?;
    let _ = std::process::Command::new("rustfmt")
        .arg(&sdk_lib_path)
        .status();
    report.files.push(format!("{}", sdk_lib_path.display()));

    // v0.2.2 W15: the `uor-foundation-macros` crate is deleted in v0.2.2.
    // No macro-crate ontology assets to generate. The proc macros it housed
    // (`uor!`, `uor_ground!`, `#[derive(ConstrainedType)]`, `#[derive(CompileUnit)]`,
    // `#[uor_grounded]`) are removed: the contract is enforced at the type and
    // visibility level (sealed traits, `pub(crate)` constructors, the W6
    // dylint group, the public-API snapshot), and the canonical builder
    // surface is the public `Term::*` enum variants directly.

    Ok(report)
}

/// Walks up from `out_dir` (expected to be `.../foundation/src/`) looking
/// for the workspace root — the first ancestor directory containing both a
/// `Cargo.toml` and a `docs/` dir. Returns `None` if no such ancestor
/// exists (e.g., during `cargo test` against a `std::env::temp_dir()`
/// target). Used by the Phase 0 classification-report write step.
fn find_workspace_root(out_dir: &Path) -> Option<std::path::PathBuf> {
    let mut cur = out_dir;
    loop {
        if cur.join("Cargo.toml").exists() && cur.join("docs").is_dir() {
            return Some(cur.to_path_buf());
        }
        cur = cur.parent()?;
    }
}

/// Generates a `mod.rs` file for a space module.
fn generate_mod_file(
    out_dir: &Path,
    space: &str,
    modules: &[&str],
    report: &mut GenerationReport,
) -> Result<()> {
    let mut f = RustFile::new(&format!("`{space}` space modules."));

    let mut sorted_modules: Vec<&str> = modules.to_vec();
    sorted_modules.sort_unstable();
    for module in &sorted_modules {
        let _ = writeln!(f.buf, "pub mod {module};");
    }

    let path = out_dir.join(space).join("mod.rs");
    emit::write_file(&path, &f.finish())?;
    report.files.push(format!("{space}/mod.rs"));
    Ok(())
}

/// Generates the crate root `lib.rs`.
fn generate_lib_rs(ontology: &Ontology) -> String {
    let mut f = RustFile::new(&format!(
        "UOR Foundation — typed Rust traits for the complete ontology.\n\
         //!\n\
         //! Version: {}\n\
         //!\n\
         //! This crate exports every ontology class as a trait, every property as a\n\
         //! method, and every named individual as a constant. Implementations import\n\
         //! these traits and provide concrete types.\n\
         //!\n\
         //! # Principal data path\n\
         //!\n\
         //! v0.2.2 establishes a single sanctioned API path. Everything else has been\n\
         //! deleted (no proc-macro back-doors, no second constructor for sealed types):\n\
         //!\n\
         //! ```text\n\
         //!  host bytes  ──▶  impl Grounding<Map = …>  ──▶  Datum<L>   [W4: kind-typed]\n\
         //!                                                  │\n\
         //!                                                  ▼\n\
         //!  builder.validate_const() │ .validate()  ──▶  Validated<T, Phase>\n\
         //!                                                  │            [W2 + W13]\n\
         //!                                                  ▼\n\
         //!  pipeline::run::<T, P>(unit)  ──▶  Grounded<T>\n\
         //!                                       │            [W14]\n\
         //!                                       ▼\n\
         //!                            .triad() → Triad<L>     [W8]\n\
         //!                            .certificate()          [W11: Certified<C>]\n\
         //! ```\n\
         //!\n\
         //! Every contract is enforced at the type and visibility level. Sealed traits,\n\
         //! `pub(crate)` constructors, and the v0.2.2 conformance suite (W5 ψ-leakage gate,\n\
         //! W6 public-API snapshot) catch any deviation.\n\
         //!\n\
         //! # Substitution axes (per the UOR-Framework wiki)\n\
         //!\n\
         //! The wiki names three substitution axes the application author selects\n\
         //! against — `HostTypes`, `HostBounds`, `Hasher`. The foundation defines\n\
         //! the trait surface; downstream substrates declare the bound the trait\n\
         //! must satisfy; the application's crate selects the implementation.\n\
         //!\n\
         //! ## `HostTypes` (target §4.1 W10)\n\
         //!\n\
         //! Downstream chooses representations only for the three slots that genuinely\n\
         //! vary across host environments. Witt-level integers, booleans, IRIs, canonical\n\
         //! bytes, and `UorTime` are foundation-owned and derived from `WittLevel`.\n\
         //!\n\
         //! ```no_run\n\
         //! use uor_foundation::{{HostTypes, DefaultHostTypes}};\n\
         //!\n\
         //! // Use the canonical defaults: f64 / str / [u8].\n\
         //! type H = DefaultHostTypes;\n\
         //!\n\
         //! // Or override one slot:\n\
         //! struct EmbeddedHost;\n\
         //! impl HostTypes for EmbeddedHost {{\n\
         //!     type Decimal = f32;          // override: tighter precision budget\n\
         //!     type HostString = str;       // default\n\
         //!     type WitnessBytes = [u8];    // default\n\
         //!     const EMPTY_DECIMAL: f32 = 0.0;\n\
         //!     const EMPTY_HOST_STRING: &'static str = \"\";\n\
         //!     const EMPTY_WITNESS_BYTES: &'static [u8] = &[];\n\
         //! }}\n\
         //! ```\n\
         //!\n\
         //! ## `HostBounds` (wiki ADR-018)\n\
         //!\n\
         //! Carries every capacity bound that varies along the principal data\n\
         //! path: the fingerprint output width range, the trace event-count\n\
         //! ceiling, the algebraic-level bit-width ceiling. Per ADR-018 the\n\
         //! architecture admits no capacity bound outside `HostBounds`.\n\
         //!\n\
         //! ```no_run\n\
         //! use uor_foundation::HostBounds;\n\
         //!\n\
         //! // ADR-060: every application declares its own `impl HostBounds`;\n\
         //! // there is no `DefaultHostBounds`.\n\
         //! struct MyBounds;\n\
         //! impl HostBounds for MyBounds {{\n\
         //!     const FINGERPRINT_MIN_BYTES: usize = 16;\n\
         //!     const FINGERPRINT_MAX_BYTES: usize = 32;\n\
         //!     const TRACE_MAX_EVENTS: usize = 256;\n\
         //!     const WITT_LEVEL_MAX_BITS: u32 = 64;\n\
         //!     const FOLD_UNROLL_THRESHOLD: usize = 8;\n\
         //!     const BETTI_DIMENSION_MAX: usize = 8;\n\
         //!     const NERVE_CONSTRAINTS_MAX: usize = 8;\n\
         //!     const NERVE_SITES_MAX: usize = 8;\n\
         //!     const JACOBIAN_SITES_MAX: usize = 8;\n\
         //!     const RECURSION_TRACE_DEPTH_MAX: usize = 16;\n\
         //!     const OP_CHAIN_DEPTH_MAX: usize = 8;\n\
         //!     const AFFINE_COEFFS_MAX: usize = 8;\n\
         //!     const CONJUNCTION_TERMS_MAX: usize = 8;\n\
         //!     const UNFOLD_ITERATIONS_MAX: usize = 256;\n\
         //! }}\n\
         //! assert_eq!(<MyBounds as HostBounds>::FINGERPRINT_MAX_BYTES, 32);\n\
         //! ```\n\
         //!\n\
         //!\n\
         //! # Module structure\n\
         //!\n\
         //! - [`kernel`] — Immutable foundation: addressing, schema, operations\n\
         //! - [`bridge`] — Kernel-computed, user-consumed: queries, resolution, partitions, proofs\n\
         //! - [`user`] — Runtime declarations: types, morphisms, state\n\
         //! - [`enforcement`] — Sealed types and the principal-path entry surface\n\
         //! - [`pipeline`] — `pipeline::run::<T, P>` and the resolver dispatch\n\
         //!\n\
         //! # Enforcement layer\n\
         //!\n\
         //! [`enforcement`] provides the sealed types that v0.2.2 forbids downstream\n\
         //! from constructing directly:\n\
         //!\n\
         //! **Layer 1 — Opaque witnesses.** [`enforcement::Datum`],\n\
         //! [`enforcement::Validated`], [`enforcement::Derivation`],\n\
         //! [`enforcement::FreeRank`], [`enforcement::Grounded`],\n\
         //! [`enforcement::Certified`], [`enforcement::Triad`]: sealed types with\n\
         //! private fields. Only the foundation's pipeline / resolver paths produce them.\n\
         //!\n\
         //! **Layer 2 — Declarative builders.** [`enforcement::CompileUnitBuilder`]\n\
         //! and 8 others collect declarations and emit `Validated<T, Phase>` on\n\
         //! success or [`enforcement::ShapeViolation`] with a machine-readable IRI.\n\
         //!\n\
         //! **Layer 3 — Term AST.** [`enforcement::Term`] and\n\
         //! [`enforcement::TermArena`]: stack-resident, `#![no_std]`-safe expression\n\
         //! trees. The `Term` enum's struct-variant constructors are the canonical\n\
         //! builder API — there is no DSL macro in v0.2.2.\n\
         //!\n\
         //! # Foundation as a signature category (wiki ADR-019)\n\
         //!\n\
         //! `uor-foundation`'s vocabulary is the **signature category** of Prism's\n\
         //! typed routes. Objects are [`pipeline::ConstrainedTypeShape`] impls under\n\
         //! the substitution-axis selections; morphisms are typed compositions of\n\
         //! [`PrimitiveOp`] discriminants and [`enforcement::Term::Application`]\n\
         //! constructions.\n\
         //!\n\
         //! The category supports a **signature endofunctor** F whose enriched\n\
         //! signature has one branch per [`enforcement::Term`] variant — the four\n\
         //! first-order constructors (`Literal`, `Application`, `Lift`, `Project`),\n\
         //! plus binding (`Variable`), case analysis (`Match`), explicit fixed-point\n\
         //! (`Recurse`), explicit unfold (`Unfold`), and failure-promote (`Try`).\n\
         //!\n\
         //! [`enforcement::Term`] is F's **initial algebra**: any well-typed term\n\
         //! tree is an element of the free term language F generates from its\n\
         //! enriched signature, and any carrier supporting the same enriched\n\
         //! structure admits a *unique* structure-preserving map from `Term`.\n\
         //!\n\
         //! [`pipeline::run`] is the **catamorphism** into the runtime carrier\n\
         //! parameterized by the substitution axes (`HostTypes`, `HostBounds`,\n\
         //! `Hasher`). Its dual, the **anamorphism**, is the trace-replay surface\n\
         //! ([`enforcement::Derivation::replay`] +\n\
         //! [`enforcement::replay::certify_from_trace`]); the trace is the\n\
         //! anamorphism's witness object. The four UOR-domain sealed types\n\
         //! ([`enforcement::Datum`], [`enforcement::Triad`],\n\
         //! [`enforcement::Derivation`], [`enforcement::FreeRank`]) and the three\n\
         //! Prism-mechanism sealed types ([`enforcement::Validated`],\n\
         //! [`enforcement::Grounded`], [`enforcement::Certified`]) are **fixed\n\
         //! points** of the typed pipeline endofunctor (distinct from F; see wiki\n\
         //! section 8, Fixed Points).\n\
         //!\n\
         //! Initiality and uniqueness of the catamorphism hold *within each fixed\n\
         //! choice of the three substitution axes*: for a given\n\
         //! `(HostTypes, HostBounds, Hasher)` selection, there is exactly one\n\
         //! F-algebra homomorphism from `Term` to the corresponding carrier. The\n\
         //! categorical statement of ADR-018's capacity completeness is that the\n\
         //! indexing of carriers is *total* over `HostBounds` — every\n\
         //! capacity-bounded width that varies along the principal data path is\n\
         //! part of the index. The substrate-vs-implementor split (ADR-007's\n\
         //! three-position pattern) extends naturally: foundation defines F and\n\
         //! the initial algebra; the runtime declares the bound any application's\n\
         //! `Route` MUST satisfy and the catamorphism that compiles it; the\n\
         //! application author selects the F-algebra by writing the model.\n\
         //!\n\
         //! # `PrismModel` — the application author's typed iso (wiki ADR-020)\n\
         //!\n\
         //! [`pipeline::PrismModel`] codifies the iso the application author\n\
         //! declares: an `Input` feature type, an `Output` label type, and a\n\
         //! type-level `Route` witness of the term tree mapping one to the other.\n\
         //! The `Route` associated type is bound by [`pipeline::FoundationClosed`]\n\
         //! — closure under foundation vocabulary, enforced at the application's\n\
         //! compile time per UORassembly (TC-04, ADR-006). `forward()` is the\n\
         //! catamorphism into [`pipeline::run`]'s runtime carrier; together with\n\
         //! the `Trace`-witnessed anamorphism through\n\
         //! [`enforcement::replay::certify_from_trace`] it forms the verifiable\n\
         //! round-trip described in the wiki.\n\
         //!\n\
         //! # Witness minting (Phases 10 + 12 + 14 + 15)\n\
         //!\n\
         //! Path-2 ontology classes (theorem-attesting witnesses such as\n\
         //! [`witness_scaffolds::MintBornRuleVerification`],\n\
         //! [`witness_scaffolds::MintCompletenessWitness`], etc.) implement the\n\
         //! sealed [`OntologyVerifiedMint`] trait. Mint a witness by populating\n\
         //! its `Mint{{Foo}}Inputs<H>` bundle with non-zero handles + true\n\
         //! attestation flags + non-empty strings, then calling\n\
         //! `Mint{{Foo}}::ontology_mint::<H>(inputs)`. On structural-invariant\n\
         //! failure each `verify_*` primitive in `crate::primitives::{{family}}`\n\
         //! returns a typed [`enforcement::GenericImpossibilityWitness`] whose\n\
         //! IRI cites the specific failing op-namespace identity (BR_*,\n\
         //! CC_*, IH_*, FX_4, WLS_*, surfaceSymmetry).\n\
         //!\n\
         //! # Per-class observable views (Phase 16)\n\
         //!\n\
         //! [`enforcement::Validated<T, Phase>`] does not directly implement\n\
         //! the kind-specific `Observable<H>` trait — Rust forbids multiple\n\
         //! `Observable<H>` impls per type, and the bare blanket would return\n\
         //! a sentinel for every kind. Consumers select an explicit kind via\n\
         //! the inherent accessors\n\
         //! [`Validated::as_landauer`](enforcement::Validated::as_landauer),\n\
         //! [`Validated::as_jacobian`](enforcement::Validated::as_jacobian),\n\
         //! [`Validated::as_carry_depth`](enforcement::Validated::as_carry_depth),\n\
         //! [`Validated::as_derivation_depth`](enforcement::Validated::as_derivation_depth),\n\
         //! and [`Validated::as_free_rank`](enforcement::Validated::as_free_rank).\n\
         //! Each returns a zero-cost newtype view in [`blanket_impls`] whose\n\
         //! `Observable<H>::value()` body delegates to the relevant primitive\n\
         //! (`primitive_descent_metrics`, `primitive_curvature_jacobian`,\n\
         //! `primitive_dihedral_signature`, etc.).\n\
         //!\n\
         //! Calling `.landauer_nats()` on the view requires the\n\
         //! `bridge::observable::LandauerBudget` *trait* to be in scope —\n\
         //! distinct from the `enforcement::LandauerBudget` *struct*\n\
         //! re-exported at the crate root. Likewise for `as_jacobian` →\n\
         //! `bridge::observable::JacobianObservable`,\n\
         //! `as_carry_depth` → `kernel::carry::CarryDepthObservable`,\n\
         //! `as_derivation_depth` →\n\
         //! `bridge::derivation::DerivationDepthObservable`, and\n\
         //! `as_free_rank` → `bridge::partition::FreeRankObservable`.\n\
         //! `<_ as Observable<H>>::value(&view)` works for every view\n\
         //! once `Observable` itself is in scope.\n\
         //!\n\
         //! # Resolvers (v0.2.2 W12)\n\
         //!\n\
         //! Verdict-producing resolvers are free functions in module-per-resolver\n\
         //! organization. Each function returns a `Result<Certified<Cert>, Witness>`:\n\
         //!\n\
         //! - `enforcement::resolver::inhabitance::certify(input)` — inhabitance verdict\n\
         //! - `enforcement::resolver::tower_completeness::certify(input)` — tower completeness\n\
         //! - `enforcement::resolver::incremental_completeness::certify(input)` — incremental\n\
         //! - `enforcement::resolver::grounding_aware::certify(unit)` — grounding-aware\n\
         //!\n\
         //! # Features\n\
         //!\n\
         //! The crate ships this feature-flag layout. Every capability the `default`\n\
         //! build omits is opt-in; the default is `#![no_std]`-pure and alloc-free.\n\
         //!\n\
         //! | Feature         | Default | Adds | When to enable |\n\
         //! |-----------------|---------|------|----------------|\n\
         //! | `alloc`         | off     | `extern crate alloc`; alloc-backed diagnostic helpers | Heap available but no OS |\n\
         //! | `std`           | off     | `alloc` + std-specific paths | Hosted platforms |\n\
         //! | `libm`          | **on** (unconditional dep) | `libm`-backed `ln`, `exp`, `sqrt` for transcendental observables | Always on — required by `xsd:decimal` observables (see target §1.6) |\n\
         //! | `serde`         | off     | `serde::{{Serialize, Deserialize}}` on `Trace`, `TraceEvent`, and other carriers | Exporting traces to external verifiers |\n\
         //! | `observability` | off     | `alloc` + a `subscribe(handler: FnMut(&TraceEvent))` surface | Runtime observation of the reduction pipeline |\n\
         //!\n\
         //! The `default = []` posture means bare-metal targets (`thumbv7em-none-eabihf`)\n\
         //! build without any feature flag. CI validates three configurations: the\n\
         //! bare-metal `no_std` cross-build, the `alloc`-additive hosted build, and\n\
         //! the `--all-features` composite. See target §1.6 and §7.5.\n\
         //!\n\
         //! # Scope note\n\
         //!\n\
         //! This crate is conformance-first: every surface the ontology specifies\n\
         //! is present, and every surface it rejects (e.g., the deleted v0.2.1\n\
         //! `Primitives` trait and unit-struct resolver façades) is absent. There\n\
         //! is no migration layer, no deprecation aliases, and no compatibility\n\
         //! shims — the crate is either in conformance with the ontology or it isn't.",
        ontology.version,
    ));

    f.line("#![no_std]");
    f.blank();
    // ADR-060: the optional `alloc` feature pulls in `extern crate alloc`
    // ONLY for the opt-in, per-caller `TermValue::to_vec()` materialization
    // helper. The default build (`default = []`) is strictly no_std + no-alloc;
    // the principal data path (the source-polymorphic carrier and the
    // catamorphism's chunk-folding σ-projection) allocates nothing — unbounded
    // `TermValue::Stream` payloads fold chunk-by-chunk via `for_each_chunk`
    // with zero heap use. Downstream no-alloc libraries depend with
    // `default-features = false` and never see this declaration.
    f.line("#[cfg(feature = \"alloc\")]");
    f.line("extern crate alloc;");
    f.blank();
    // Phase 11d — `pub mod blanket_impls;` lands in the generated lib.rs
    // so the hand-written `@codegen-exempt`-banner-protected file at
    // `foundation/src/blanket_impls.rs` is exposed to crate consumers.
    // Phase 11c's emit::write_file banner check preserves the file
    // across `uor-crate` regeneration runs.
    f.line("pub mod blanket_impls;");
    f.line("pub mod bridge;");
    f.line("pub mod enforcement;");
    f.line("pub mod enums;");
    f.line("pub mod kernel;");
    f.line("pub mod pipeline;");
    // Phase 10 — Path-2 VerifiedMint scaffolds + per-family primitive
    // stubs. Generated alongside enforcement.rs / pipeline.rs.
    f.line("pub mod primitives;");
    f.line("pub mod user;");
    f.line("pub mod witness_scaffolds;");
    f.blank();
    f.line("pub use enums::*;");
    f.blank();
    // Phase 10 — re-export the new mint trait + every Mint{Foo} witness so
    // downstream consumers can `use uor_foundation::{OntologyVerifiedMint,
    // MintBornRuleVerification, ...}` without crawling submodules.
    f.line("pub use witness_scaffolds::OntologyVerifiedMint;");
    f.blank();
    // v0.2.2 T4.5.c + T5.11: convenience re-exports. The enforcement module
    // remains the source of truth; these re-exports shorten common import
    // paths for downstream consumers. T5.11 added the Hasher /
    // ContentFingerprint family, error types, AST types, and constants so
    // every public type a downstream consumer reaches for resolves under
    // `uor_foundation::*`.
    f.line("pub use enforcement::{");
    f.line("    BindingEntry, BindingsTable, BindingsTableError, BoundConstraint, Calibration,");
    f.line("    CalibrationError, Certificate, CertificateKind, Certified, CompileUnit, CompileUnitBuilder,");
    f.line("    ContentAddress, ContentFingerprint, Derivation, Grounded, GroundingCertificate, Hasher,");
    f.line("    LandauerBudget, MultiplicationCertificate, Nanos, PipelineFailure, ReplayError, ShapeViolation,");
    f.line("    Term, TermArena, TermList, Trace, TraceEvent, UorTime, Validated,");
    f.line("    TRACE_REPLAY_FORMAT_VERSION,");
    f.line("};");
    f.blank();
    // Product/Coproduct Completion Amendment §2.5: public re-exports of the");
    // three sealed witness types, their paired Evidence / MintInputs structs,
    // the VerifiedMint sealed mint trait, and the resolver protocol. Emitted
    // as a separate `pub use` block so the Amendment-added surface is visually
    // distinct from the core enforcement surface above.
    f.line("pub use enforcement::{");
    f.line("    CartesianProductEvidence, CartesianProductMintInputs, CartesianProductWitness,");
    f.line("    GenericImpossibilityWitness, NullPartition, PartitionCoproductEvidence,");
    f.line("    PartitionCoproductMintInputs, PartitionCoproductWitness, PartitionHandle,");
    f.line("    PartitionProductEvidence, PartitionProductMintInputs, PartitionProductWitness, PartitionRecord,");
    f.line("    PartitionResolver, VerifiedMint,");
    f.line("};");
    f.blank();

    // Phase 9 (orphan-closure): `DecimalTranscendental` supertrait. Defined
    // BEFORE the `HostTypes` doc block so the doc comments stay adjacent to
    // their respective `pub trait` lines (the `rust/api` validator looks
    // back at most 3 lines for a `///` comment). Defines the closed
    // arithmetic + transcendentals every `HostTypes::Decimal` must support;
    // f64 / f32 delegate to `libm`. Custom host types (interval arithmetic,
    // fixed-point, arbitrary precision) bring their own.
    f.doc_comment("Closed arithmetic and transcendental math for the `HostTypes::Decimal`");
    f.doc_comment("slot. `f64` and `f32` implement this via `libm`. Downstream `HostTypes`");
    f.doc_comment("impls are free to bring their own implementation (interval arithmetic,");
    f.doc_comment("arbitrary precision, fixed-point, etc.).");
    f.line("pub trait DecimalTranscendental:");
    f.line("    Copy");
    f.line("    + Default");
    f.line("    + core::fmt::Debug");
    f.line("    + PartialEq");
    f.line("    + PartialOrd");
    f.line("    + core::ops::Add<Output = Self>");
    f.line("    + core::ops::Sub<Output = Self>");
    f.line("    + core::ops::Mul<Output = Self>");
    f.line("    + core::ops::Div<Output = Self>");
    f.line("{");
    f.indented_doc_comment(
        "Construct from an unsigned 32-bit integer. \
         f32 / f64 use `as` widening; downstream impls bring their own promotion.",
    );
    f.line("    fn from_u32(value: u32) -> Self;");
    f.indented_doc_comment(
        "Construct from an unsigned 64-bit integer (rewrite-step counts, etc.).",
    );
    f.line("    fn from_u64(value: u64) -> Self;");
    f.indented_doc_comment(
        "Saturating projection to `u64`. Used by `UorTime::min_wall_clock` to \
         convert a wall-clock seconds-Decimal into integer nanoseconds.",
    );
    f.line("    fn as_u64_saturating(self) -> u64;");
    f.indented_doc_comment("Natural logarithm.");
    f.line("    fn ln(self) -> Self;");
    f.indented_doc_comment("Exponential `e^x`.");
    f.line("    fn exp(self) -> Self;");
    f.indented_doc_comment("Square root.");
    f.line("    fn sqrt(self) -> Self;");
    f.indented_doc_comment("Construct from an IEEE-754 bit pattern (default-host f64 round-trip).");
    f.line("    fn from_bits(bits: u64) -> Self;");
    f.indented_doc_comment("Project to an IEEE-754 bit pattern (default-host f64 round-trip).");
    f.line("    fn to_bits(self) -> u64;");
    f.indented_doc_comment(
        "Entropy contribution `x * ln(x)`, with the convention `0 * ln(0) = 0`.",
    );
    f.line("    #[inline]");
    f.line("    fn entropy_term_nats(self) -> Self {");
    f.line("        if self == Self::default() {");
    f.line("            return Self::default();");
    f.line("        }");
    f.line("        self * self.ln()");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl DecimalTranscendental for f64 {");
    f.line("    #[inline]");
    f.line("    fn from_u32(value: u32) -> Self {");
    f.line("        f64::from(value)");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn from_u64(value: u64) -> Self {");
    f.line("        // u64 -> f64 is lossy above 2^53; documented at use sites.");
    f.line("        value as f64");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn as_u64_saturating(self) -> u64 {");
    f.line("        if self.partial_cmp(&0.0).is_none_or(|o| o.is_lt()) {");
    f.line("            return 0;");
    f.line("        }");
    f.line("        if self >= u64::MAX as f64 {");
    f.line("            return u64::MAX;");
    f.line("        }");
    f.line("        self as u64");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn ln(self) -> Self {");
    f.line("        libm::log(self)");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn exp(self) -> Self {");
    f.line("        libm::exp(self)");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn sqrt(self) -> Self {");
    f.line("        libm::sqrt(self)");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn from_bits(bits: u64) -> Self {");
    f.line("        f64::from_bits(bits)");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn to_bits(self) -> u64 {");
    f.line("        f64::to_bits(self)");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl DecimalTranscendental for f32 {");
    f.line("    #[inline]");
    f.line("    fn from_u32(value: u32) -> Self {");
    f.line("        // u32 -> f32 is lossy at high values; this is the documented");
    f.line("        // host-default behavior for arithmetic constants.");
    f.line("        value as f32");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn from_u64(value: u64) -> Self {");
    f.line("        // u64 -> f32 is lossy above 2^24; documented at use sites.");
    f.line("        value as f32");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn as_u64_saturating(self) -> u64 {");
    f.line("        if self.partial_cmp(&0.0).is_none_or(|o| o.is_lt()) {");
    f.line("            return 0;");
    f.line("        }");
    f.line("        if self >= u64::MAX as f32 {");
    f.line("            return u64::MAX;");
    f.line("        }");
    f.line("        self as u64");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn ln(self) -> Self {");
    f.line("        libm::logf(self)");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn exp(self) -> Self {");
    f.line("        libm::expf(self)");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn sqrt(self) -> Self {");
    f.line("        libm::sqrtf(self)");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn from_bits(bits: u64) -> Self {");
    f.line("        // f32 has no native u64-bit constructor; widen via f64 then narrow.");
    f.line("        f64::from_bits(bits) as f32");
    f.line("    }");
    f.line("    #[inline]");
    f.line("    fn to_bits(self) -> u64 {");
    f.line("        (self as f64).to_bits()");
    f.line("    }");
    f.line("}");
    f.blank();

    // Phase B: the v0.2.1 `Primitives` trait is deleted unconditionally.
    // The narrower `HostTypes` trait (three slots: `Decimal`, `HostString`,
    // `WitnessBytes`) is the only host-environment carrier. Target §4.1 W10
    // closes the deprecation; target §4.1 also removes the `DateTime` slot
    // because the foundation maintains no wall-clock source.
    f.doc_comment("Phase B (target §4.1 W10): narrow host-types trait — the only carrier for");
    f.doc_comment("the slots that genuinely vary across host environments. Foundation-owned");
    f.doc_comment("types (Witt-level integers, booleans, IRIs, canonicalBytes, `UorTime`) are");
    f.doc_comment("derived from the `WittLevel` family and not exposed here.");
    f.doc_comment("");
    f.doc_comment("Three slots: `Decimal` (real-number representation), `HostString` (opaque");
    f.doc_comment("host string, NOT a foundation IRI), and `WitnessBytes` (opaque host byte");
    f.doc_comment("sequence, NOT a foundation `canonicalBytes` constant). The v0.2.1 `DateTime`");
    f.doc_comment("slot is removed; downstream associates timestamps out-of-band.");
    f.doc_comment("");
    f.doc_comment("# Example");
    f.doc_comment("");
    f.doc_comment("```");
    f.doc_comment("use uor_foundation::{HostTypes, DefaultHostTypes};");
    f.doc_comment("");
    f.doc_comment("// Canonical defaults: f64 / str / [u8].");
    f.doc_comment("type DefaultH = DefaultHostTypes;");
    f.doc_comment("");
    f.doc_comment("// Override the Decimal slot for embedded targets with tighter precision:");
    f.doc_comment("struct EmbeddedHost;");
    f.doc_comment("impl HostTypes for EmbeddedHost {");
    f.doc_comment("    type Decimal = f32;          // override");
    f.doc_comment("    type HostString = str;       // default");
    f.doc_comment("    type WitnessBytes = [u8];    // default");
    f.doc_comment("    const EMPTY_DECIMAL: f32 = 0.0;");
    f.doc_comment("    const EMPTY_HOST_STRING: &'static str = \"\";");
    f.doc_comment("    const EMPTY_WITNESS_BYTES: &'static [u8] = &[];");
    f.doc_comment("}");
    f.doc_comment("");
    f.doc_comment("# let _ = (core::marker::PhantomData::<DefaultH>, core::marker::PhantomData::<EmbeddedHost>);");
    f.doc_comment("```");
    f.line("pub trait HostTypes {");
    f.indented_doc_comment(
        "Real-number representation for kernel observables (entropies, amplitudes, rates).\n\
         `DefaultHostTypes` selects `f64`. Override with higher-precision or interval\n\
         arithmetic as needed.\n\
         \n\
         Phase 9 bound: every `Decimal` must implement [`DecimalTranscendental`] —\n\
         closed arithmetic + ln/exp/sqrt + IEEE-754 bit-pattern round-trip. The\n\
         in-tree `f64` and `f32` impls satisfy this via `libm`.",
    );
    f.line("    type Decimal: DecimalTranscendental;");
    f.blank();
    f.indented_doc_comment(
        "Host-supplied opaque string (NOT a foundation IRI).\n\
         `DefaultHostTypes` selects `str`. Override with owned `String`, `Cow<'_, str>`,",
    );
    f.indented_doc_comment("etc. for embedded / host-heap environments.");
    f.indented_doc_comment("");
    f.indented_doc_comment("The `'static` bound is required by the Product/Coproduct Completion");
    f.indented_doc_comment("Amendment §B1 `EMPTY_HOST_STRING` constant — every conforming `H`");
    f.indented_doc_comment("must be able to expose a `&'static HostString`. All in-tree impls");
    f.indented_doc_comment("(`DefaultHostTypes::HostString = str`) already satisfy this.");
    f.line("    type HostString: ?Sized + 'static;");
    f.blank();
    f.indented_doc_comment(
        "Host-supplied opaque byte sequence (NOT a foundation `canonicalBytes` constant).\n\
         `DefaultHostTypes` selects `[u8]`. Override with owned `Vec<u8>`, `Bytes`,",
    );
    f.indented_doc_comment("etc. for host-heap environments.");
    f.indented_doc_comment("");
    f.indented_doc_comment("The `'static` bound mirrors `HostString` for the same reason — see");
    f.indented_doc_comment("the `EMPTY_WITNESS_BYTES` constant below.");
    f.line("    type WitnessBytes: ?Sized + 'static;");
    f.blank();
    // Product/Coproduct Completion Amendment §B1: empty-value defaults
    // for all three host-supplied slots. These are read by `NullPartition`
    // and the partition-algebra stub sub-trait types (`NullElement<H>`,
    // etc.) when satisfying `Partition<H>` accessors that return
    // references to host-supplied data. Every conforming `H` must
    // provide them; consumer impls choose what "empty" means in their
    // host environment.
    f.indented_doc_comment("Empty / zero `Decimal` value for resolver-absent partition accessors.");
    f.indented_doc_comment("`DefaultHostTypes` selects `0.0`. Used by `NullPartition::density()`");
    f.indented_doc_comment("and analogous H-typed defaults.");
    f.line("    const EMPTY_DECIMAL: Self::Decimal;");
    f.blank();
    f.indented_doc_comment("Empty `&'static HostString` reference for resolver-absent accessors.");
    f.indented_doc_comment("`DefaultHostTypes` selects `&\"\"` coerced to `&str`. Used by");
    f.indented_doc_comment("`NullPartition::product_category_level()` and the address-typed");
    f.indented_doc_comment("string accessors on `NullElement<H>`.");
    f.line("    const EMPTY_HOST_STRING: &'static Self::HostString;");
    f.blank();
    f.indented_doc_comment(
        "Empty `&'static WitnessBytes` reference for resolver-absent accessors.",
    );
    f.indented_doc_comment("`DefaultHostTypes` selects `&[]` coerced to `&[u8]`. Used by");
    f.indented_doc_comment("`NullElement<H>::canonical_bytes()`.");
    f.line("    const EMPTY_WITNESS_BYTES: &'static Self::WitnessBytes;");
    f.line("}");
    f.blank();

    f.doc_comment("Phase B: canonical default impl of [`HostTypes`]. Selects `f64`/`str`/`[u8]`.");
    f.doc_comment("Use as `type H = uor_foundation::DefaultHostTypes;` to inherit the defaults;");
    f.doc_comment("replace with a downstream marker struct if any slot needs an override.");
    f.line("#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct DefaultHostTypes;");
    f.blank();
    // Phase 9 (orphan-closure): IEEE-754 bit-pattern constants for the
    // physical / numerical literals that previously appeared as `: f64`
    // hardcodes throughout the foundation. Use sites convert to
    // `H::Decimal` via `DecimalTranscendental::from_bits`.
    f.doc_comment("π · ℏ = `core::f64::consts::PI * 1.054_571_817e-34` J·s, in IEEE-754 bits.");
    f.doc_comment("Half of one orthogonal-state-transition Margolus-Levitin bound.");
    f.line("pub const PI_TIMES_H_BAR_BITS: u64 =");
    f.line("    f64::to_bits(core::f64::consts::PI * 1.054_571_817e-34_f64);");
    f.blank();
    f.doc_comment(
        "Nanoseconds per second (`1.0e9`) in IEEE-754 bits. Used by `UorTime::min_wall_clock`.",
    );
    f.line("pub const NANOS_PER_SECOND_BITS: u64 = f64::to_bits(1.0e9_f64);");
    f.blank();
    f.doc_comment(
        "Natural logarithm of 2, in IEEE-754 bits. Drives the Landauer bit-erasure unit.",
    );
    f.line("pub const LN_2_BITS: u64 = f64::to_bits(core::f64::consts::LN_2);");
    f.blank();

    // Calibration plausibility envelope, encoded as bit patterns. The
    // physical interpretation is documented at the consumer site
    // (`Calibration::new`).
    f.doc_comment("Lower bound for `Calibration::k_b_t` (1e-30 J), in IEEE-754 bits.");
    f.line("pub const CALIBRATION_KBT_LO_BITS: u64 = f64::to_bits(1.0e-30_f64);");
    f.blank();
    f.doc_comment("Upper bound for `Calibration::k_b_t` (1e-15 J), in IEEE-754 bits.");
    f.line("pub const CALIBRATION_KBT_HI_BITS: u64 = f64::to_bits(1.0e-15_f64);");
    f.blank();
    f.doc_comment("Upper bound for `Calibration::thermal_power` (1e9 W), in IEEE-754 bits.");
    f.line("pub const CALIBRATION_THERMAL_POWER_HI_BITS: u64 = f64::to_bits(1.0e9_f64);");
    f.blank();
    f.doc_comment(
        "Upper bound for `Calibration::characteristic_energy` (1e3 J), in IEEE-754 bits.",
    );
    f.line("pub const CALIBRATION_CHAR_ENERGY_HI_BITS: u64 = f64::to_bits(1.0e3_f64);");
    f.blank();

    f.line("impl HostTypes for DefaultHostTypes {");
    f.line("    type Decimal = f64;");
    f.line("    type HostString = str;");
    f.line("    type WitnessBytes = [u8];");
    // `Self::Decimal` resolves to f64 inside this impl; the literal `0.0`");
    // type-infers against it. No `: f64` syntax appears in source.");
    f.line("    const EMPTY_DECIMAL: Self::Decimal = 0.0;");
    f.line("    const EMPTY_HOST_STRING: &'static str = \"\";");
    f.line("    const EMPTY_WITNESS_BYTES: &'static [u8] = &[];");
    f.line("}");
    f.blank();

    // ADR-018 (wiki) — `HostBounds` substitution axis.
    //
    // The wiki (https://github.com/UOR-Foundation/UOR-Framework/wiki) names
    // three substitution axes — `HostTypes`, `HostBounds`, `Hasher`. ADR-018
    // mandates that every capacity bound that varies along the principal
    // data path is carried by `HostBounds` as an associated constant, and
    // that signatures parameterize through `HostBounds` rather than through
    // free-standing constants.
    //
    // On stable Rust 1.83 (the workspace MSRV) the parameterization uses
    // min-const-generics: capacity-bearing types — `Hasher`,
    // `ContentFingerprint`, `Trace` — and capacity-bearing functions
    // (`Derivation::replay`, `replay::certify_from_trace`,
    // `unit_address_from_buffer`, the `__test_helpers::trace_*` ctors) all
    // carry `<const FP_MAX: usize>` / `<const TR_MAX: usize>` parameters.
    // Applications selecting a different `HostBounds` impl populate each
    // parameter with `<MyBounds as HostBounds>::CONST` at instantiation sites.
    // Type-level defaults (`Hasher<const FP_MAX: usize = 32>`, etc.) carry the
    // conventional 32-byte width so callers using those defaults never write
    // turbofish. The wiki's example syntax `[u8; H::FINGERPRINT_MAX_BYTES]`
    // requires nightly `generic_const_exprs`, which the stable workspace does
    // not enable.
    //
    // `TRACE_REPLAY_FORMAT_VERSION` stays foundation-fixed: the wiki
    // explicitly carves out wire-format identifiers from the `HostBounds`
    // rule, since cross-implementation interop requires a single shared
    // format identifier.
    f.doc_comment("Substitution axis 2 of 3 (per the UOR-Framework wiki). Carries every");
    f.doc_comment("capacity bound that varies along the principal data path: the fingerprint");
    f.doc_comment("output width range, the trace event-count ceiling, and the algebraic-level");
    f.doc_comment("bit-width ceiling. The application author selects an impl; the foundation");
    f.doc_comment("(this trait) declares the contract.");
    f.doc_comment("");
    f.doc_comment("Per the wiki's ADR-018, the architecture admits no capacity bound outside");
    f.doc_comment("`HostBounds`. Foundation's `Hasher`, `ContentFingerprint`, and `Trace` are");
    f.doc_comment("const-generic over their capacity bounds; applications populate each");
    f.doc_comment("type's const-generic with `<MyBounds as HostBounds>::CONST`. There are no");
    f.doc_comment("free-standing capacity constants on the public surface — collapsing the");
    f.doc_comment("substitution axis is exactly what ADR-018 rejects.");
    f.doc_comment("");
    f.doc_comment("# Example");
    f.doc_comment("");
    f.doc_comment("```");
    f.doc_comment("use uor_foundation::HostBounds;");
    f.doc_comment("");
    f.doc_comment("// ADR-060: there is no `DefaultHostBounds`. Every application declares");
    f.doc_comment("// its own capacity profile; the foundation supplies no default policy.");
    f.doc_comment("struct BitcoinPow;");
    f.doc_comment("impl HostBounds for BitcoinPow {");
    f.doc_comment("    const FINGERPRINT_MIN_BYTES: usize = 32;");
    f.doc_comment("    const FINGERPRINT_MAX_BYTES: usize = 32;");
    f.doc_comment("    const TRACE_MAX_EVENTS: usize = 1024;");
    f.doc_comment("    const WITT_LEVEL_MAX_BITS: u32 = 256;");
    f.doc_comment("    // ADR-060: 10 retained structural-count / catamorphism-trace caps.");
    f.doc_comment("    const FOLD_UNROLL_THRESHOLD: usize = 8;");
    f.doc_comment("    const BETTI_DIMENSION_MAX: usize = 8;");
    f.doc_comment("    const NERVE_CONSTRAINTS_MAX: usize = 8;");
    f.doc_comment("    const NERVE_SITES_MAX: usize = 8;");
    f.doc_comment("    const JACOBIAN_SITES_MAX: usize = 8;");
    f.doc_comment("    const RECURSION_TRACE_DEPTH_MAX: usize = 16;");
    f.doc_comment("    const OP_CHAIN_DEPTH_MAX: usize = 8;");
    f.doc_comment("    const AFFINE_COEFFS_MAX: usize = 8;");
    f.doc_comment("    const CONJUNCTION_TERMS_MAX: usize = 8;");
    f.doc_comment("    const UNFOLD_ITERATIONS_MAX: usize = 256;");
    f.doc_comment("}");
    f.doc_comment("assert_eq!(<BitcoinPow as HostBounds>::FINGERPRINT_MAX_BYTES, 32);");
    f.doc_comment("```");
    f.line("pub trait HostBounds {");
    f.indented_doc_comment("Minimum content-fingerprint width in bytes that the application's");
    f.indented_doc_comment("selected `Hasher` impl MUST produce. Derived from the application's");
    f.indented_doc_comment("collision-probability target (16 bytes ≈ 2^-64 under the birthday");
    f.indented_doc_comment("bound; raise to 32 to reach 2^-128).");
    f.line("    const FINGERPRINT_MIN_BYTES: usize;");
    f.blank();
    f.indented_doc_comment("Maximum content-fingerprint width in bytes — the inline buffer");
    f.indented_doc_comment("capacity carried by every `ContentFingerprint`. The application's");
    f.indented_doc_comment("selected `Hasher` MUST produce output no wider than this.");
    f.line("    const FINGERPRINT_MAX_BYTES: usize;");
    f.blank();
    f.indented_doc_comment("Maximum number of `TraceEvent` values a `Trace` may carry.");
    f.indented_doc_comment("Bounds the inline event buffer and caps verification time.");
    f.line("    const TRACE_MAX_EVENTS: usize;");
    f.blank();
    f.indented_doc_comment("Algebraic-level bit-width ceiling. Caps the Witt-level any value");
    f.indented_doc_comment("along the principal data path may compute against. A value of 64");
    f.indented_doc_comment("corresponds to `WittLevel::W64`.");
    f.line("    const WITT_LEVEL_MAX_BITS: u32;");
    f.blank();
    // ADR-060: 10 retained data-shape capacity caps (structural-count and
    // catamorphism-trace bounds) per ADR-037. ADR-060 SUPERSEDES ADR-037's
    // 12-member byte-width-cap family — the 4 carrier ceilings
    // (TERM_VALUE_MAX_BYTES, AXIS_OUTPUT_BYTES_MAX, ROUTE_INPUT_BUFFER_BYTES,
    // ROUTE_OUTPUT_BUFFER_BYTES) and the 8 ψ-stage *_OUTPUT_BYTES_MAX ceilings
    // are REMOVED. Those byte widths are no longer application-declared caps
    // but foundation-derived `const fn` values (`carrier_inline_bytes::<B>()`,
    // `nerve_carrier_bytes::<B>()`, …) computed from the structural counts
    // below × foundation-fixed per-element wire widths. The source-polymorphic
    // `TermValue` (Inline / Borrowed / Stream) carries unbounded payloads with
    // no carrier-side ceiling, so no byte-width cap belongs on `HostBounds`.
    f.indented_doc_comment("ADR-037: threshold below which `fold_n(n, init, step)` lowers to");
    f.indented_doc_comment("an unrolled `Term::Application` chain (one application per");
    f.indented_doc_comment("iteration). Counts `n >= FOLD_UNROLL_THRESHOLD` lower to");
    f.indented_doc_comment("`Term::Recurse` instead.");
    f.line("    const FOLD_UNROLL_THRESHOLD: usize;");
    f.blank();
    f.indented_doc_comment("ADR-037: maximum dimension index for `BettiNumbers` arrays —");
    f.indented_doc_comment("`β_k(K)` is stored for `k` in `0..BETTI_DIMENSION_MAX`.");
    f.line("    const BETTI_DIMENSION_MAX: usize;");
    f.blank();
    f.indented_doc_comment("ADR-037: maximum number of constraints in a single constraint");
    f.indented_doc_comment("nerve's array representation.");
    f.line("    const NERVE_CONSTRAINTS_MAX: usize;");
    f.blank();
    f.indented_doc_comment("ADR-037: maximum number of sites in a single constraint nerve's");
    f.indented_doc_comment("array representation.");
    f.line("    const NERVE_SITES_MAX: usize;");
    f.blank();
    f.indented_doc_comment("ADR-037: maximum number of sites for which a Jacobian matrix");
    f.indented_doc_comment("can be stored inline.");
    f.line("    const JACOBIAN_SITES_MAX: usize;");
    f.blank();
    f.indented_doc_comment("ADR-037: maximum recursion depth for the trace-replay stack.");
    f.line("    const RECURSION_TRACE_DEPTH_MAX: usize;");
    f.blank();
    f.indented_doc_comment("ADR-037: maximum operator-chain depth in a single operation.");
    f.line("    const OP_CHAIN_DEPTH_MAX: usize;");
    f.blank();
    f.indented_doc_comment("ADR-037: maximum number of affine coefficients per");
    f.indented_doc_comment("`ConstraintRef::Affine`.");
    f.line("    const AFFINE_COEFFS_MAX: usize;");
    f.blank();
    f.indented_doc_comment("ADR-037: maximum number of conjuncts per");
    f.indented_doc_comment("`ConstraintRef::Conjunction`.");
    f.line("    const CONJUNCTION_TERMS_MAX: usize;");
    f.blank();
    f.indented_doc_comment("ADR-037: maximum iteration count for `Term::Unfold` evaluation.");
    f.line("    const UNFOLD_ITERATIONS_MAX: usize;");
    f.line("}");
    f.blank();

    // ADR-060: `DefaultHostBounds` is removed. There is no "default"
    // application, so the foundation supplies no default capacity policy.
    // Every application declares its own `impl HostBounds`, and every numeric
    // constant the foundation honors at runtime traces to an application's
    // explicit declaration. The foundation supplies the `const fn` derivations
    // (`carrier_inline_bytes::<B>()`, the per-ψ-stage `*_carrier_bytes::<B>()`)
    // that compute per-carrier byte widths from the declared primitives; it
    // does not supply default values for the primitives. This makes ADR-018's
    // parametric-in-capacity commitment load-bearing in fact rather than in
    // name (Rejected alternative 4).

    f.finish()
}

/// Generates `README.md` for the published crate.
fn generate_readme(ontology: &Ontology) -> String {
    let ns_map = namespace_mappings();

    // Build module table rows dynamically from the ontology
    let mut rows = String::new();
    for module in &ontology.namespaces {
        if let Some(mapping) = ns_map.get(module.namespace.iri) {
            let space_label = match module.namespace.space {
                Space::Kernel => "Kernel",
                Space::Bridge => "Bridge",
                Space::User => "User",
            };
            // Use first sentence of namespace comment as description
            let desc = module
                .namespace
                .comment
                .split('.')
                .next()
                .unwrap_or(module.namespace.label);
            let _ = writeln!(
                rows,
                "| `{}::{}` | {} | {} |",
                mapping.space_module, mapping.file_module, space_label, desc
            );
        }
    }

    format!(
        r#"# uor-foundation

The complete [UOR Foundation](https://uor.foundation/) ontology encoded as
typed Rust traits. Import and implement.

## Contents

- {ns} namespaces
- {classes} OWL classes (one trait each)
- {props} OWL properties (one method each)
- {inds} named individuals (constants and enums)
- `enforcement` module with declarative builders and opaque witnesses
- `uor!` proc macro for compile-time term-language DSL

## Quick start

```toml
[dependencies]
uor-foundation = "{version}"
```

### HostTypes (target §4.1 W10)

Every foundation trait is parametric over `HostTypes` — a sealed bundle
declaring the host-environment types the ontology references.
`DefaultHostTypes` ships for the common case; downstream implementers
supply their own `HostTypes` impl when they need non-default carriers.

```rust
use uor_foundation::{{DefaultHostTypes, HostTypes}};

// Default bundle — satisfies every ring-adjacent surface. Ring
// arithmetic is mono-sorted by construction of the term grammar: host
// slots never participate.
type H = DefaultHostTypes;
```

### Grounding maps (target §4.3)

Downstream sources of external data bind to `GroundingMapKind` by
implementing `Grounding` with a combinator `program()`. The foundation
supplies `ground()` via the sealed `GroundingExt` extension trait — the
program's marker tuple mechanically verifies that the declared `Map`
matches the combinator decomposition.

```rust
use uor_foundation::enforcement::{{
    combinators, BinaryGroundingMap, GroundedCoord, Grounding, GroundingExt,
    GroundingProgram,
}};

struct ReadFirstByte;

impl Grounding for ReadFirstByte {{
    type Output = GroundedCoord;
    type Map = BinaryGroundingMap;

    // Downstream provides ONLY program(). `ground()` is foundation-owned
    // via the sealed `GroundingExt` blanket impl.
    fn program(&self) -> GroundingProgram<GroundedCoord, BinaryGroundingMap> {{
        GroundingProgram::from_primitive(combinators::read_bytes::<GroundedCoord>())
    }}
}}

// Callers reach ground() through the sealed extension trait.
let g = ReadFirstByte;
let coord: Option<GroundedCoord> = <ReadFirstByte as GroundingExt>::ground(&g, &[0x42]);
```

### Resolvers (target §4.2)

Every resolver is a module with a `certify` free function that consumes
a `&Validated<Input, P>` carrier and returns
`Result<Certified<SuccessCert>, Certified<ImpossibilityWitness>>`.

```rust,ignore
use uor_foundation::enforcement::{{resolver, ConstrainedTypeInput}};
use uor_foundation_test_helpers::{{validated_runtime, Fnv1aHasher16}};

let input = validated_runtime(ConstrainedTypeInput::default());
let cert = resolver::tower_completeness::certify::<_, _, Fnv1aHasher16>(&input)?;
// cert: Certified<LiftChainCertificate>
```

The 22 resolver modules share this shape; the only exception is
`multiplication::certify(&MulContext)` whose input is a self-validated
shape (target §4.2 MulContext exemption).

### Wall-clock (target §1.7)

`UorTime` records three foundation-internal clocks; the wall-clock
lower bound emerges from `min_wall_clock(&Calibration)`:

```rust,ignore
use uor_foundation::enforcement::calibrations::X86_SERVER;
let min_nanos = grounded.uor_time().min_wall_clock(&X86_SERVER).as_u64();
```

## Module structure

| Module | Space | Description |
|--------|-------|-------------|
{module_rows}| `enums` | — | Controlled vocabulary enums (WittLevel, PrimitiveOp, etc.) |
| `enforcement` | — | Opaque witnesses, declarative builders, Term AST |

## Features

This crate is `#![no_std]` with a single mandatory dependency on `libm`
(always-on transcendental math per target §1.6). The `uor!` proc macro
is re-exported from `uor-foundation` and parses term-language expressions
at compile time.

## Substrate-pluggable hashing

`uor-foundation` never picks a hash function. Every public path that
produces a `Grounded`, `Trace`, or `GroundingCertificate` takes a generic
`H: Hasher` parameter and threads the caller's substrate through
`fold_unit_digest` (or one of the sibling `fold_*_digest` helpers). The
foundation defines only the byte-layout contract and the `ContentFingerprint`
parametric carrier; downstream code supplies the cryptographic primitive.

```rust,ignore
use uor_foundation::enforcement::{{Hasher, ContentFingerprint}};
use uor_foundation::pipeline::run;

struct Blake3Hasher {{ /* ... */ }}
impl Hasher for Blake3Hasher {{
    const OUTPUT_BYTES: usize = 32;
    fn initial() -> Self {{ /* ... */ }}
    fn fold_byte(self, b: u8) -> Self {{ /* ... */ }}
    fn fold_bytes(self, bytes: &[u8]) -> Self {{ /* ... */ }}
    fn finalize(self) -> [u8; 32] {{ /* width = your `HostBounds::FINGERPRINT_MAX_BYTES` */ }}
}}

// ADR-060: `run` carries the `const INLINE_BYTES` carrier width as its final
// generic; instantiate it from your `HostBounds` via
// `pipeline::carrier_inline_bytes::<MyBounds>()`.
let grounded = run::<MyShape, _, Blake3Hasher, INLINE_BYTES>(validated_unit)?;
```

The recommended production substrate is BLAKE3: fast, cryptographically
sound, and 32-byte output. See PRISM's `Hasher` impl for a worked reference.
FNV-1a test substrates live in `uor-foundation-test-helpers` and are used
only by the round-trip conformance tests; they are not fit for production.

The typed pipeline entry points (`pipeline::run`, `run_const`, `run_parallel`,
`run_stream`, `run_interactive`) and every resolver facade
(`TowerCompletenessResolver`, `IncrementalCompletenessResolver`,
`GroundingAwareResolver`, `InhabitanceResolver`, `MultiplicationResolver`)
are generic over `H: Hasher`. There are no fallback paths, no
zero-fingerprint sentinels, and no `Default` impls on cert shims — a
substrate is mandatory at every grounding site.

## Product / coproduct witnesses (Product/Coproduct Completion Amendment)

Three sealed witness types attest that a shape decomposes as one of
the partition-algebra operations:

- `PartitionProductWitness` — gated on PT_1 / PT_3 / PT_4 and the
  `foundation/ProductLayoutWidth` invariant (UOR `A × B`, χ additive).
- `PartitionCoproductWitness` — gated on ST_1 / ST_2 / ST_6 / ST_7 /
  ST_8 / ST_9 / ST_10, the `foundation/CoproductLayoutWidth` invariant,
  and `foundation/CoproductTagEncoding` (UOR `A + B`, `ln 2` tag
  entropy). `validate_coproduct_structure` walks the supplied
  `ConstraintRef` array at mint time to verify the canonical
  tag-pinner encoding structurally.
- `CartesianProductWitness` — gated on CPT_1 / CPT_3 / CPT_4 / CPT_5
  and the `foundation/CartesianLayoutWidth` invariant (UOR `A ⊠ B`,
  χ multiplicative, Betti via Künneth).

Every witness implements `Certificate` with a partition-namespace IRI
and a paired `*Evidence` associated type. The sealed `VerifiedMint`
trait routes each `*MintInputs` struct through the corresponding mint
primitive; failures return `GenericImpossibilityWitness::for_identity`
citing the specific `op:*` theorem or `foundation:*` layout invariant
that was violated.

```rust,ignore
use uor_foundation::{{
    PartitionProductMintInputs, PartitionProductWitness, VerifiedMint,
}};

let witness = PartitionProductWitness::mint_verified(inputs)?;
assert_eq!(witness.combined_site_budget(), /* A.sb + B.sb */);
```

`PartitionHandle<H>` is the content-addressed identity token for a
partition; pair it with a `PartitionResolver<H>` via `resolve_with`
to recover full `PartitionRecord<H>` data (site budget, Euler, Betti,
entropy). Ergonomic ergonomic macros (`product_shape!`,
`coproduct_shape!`, `cartesian_product_shape!`) live in the opt-in
companion `uor-foundation-sdk` crate.

## License

MIT — see [LICENSE](https://github.com/UOR-Foundation/UOR-Framework/blob/main/LICENSE).
"#,
        version = ontology.version,
        ns = ontology.namespaces.len(),
        classes = ontology.class_count(),
        props = ontology.property_count(),
        inds = ontology.individual_count(),
        module_rows = rows,
    )
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn generation_produces_correct_trait_count() {
        let ontology = uor_ontology::Ontology::full();
        let tmp = std::env::temp_dir().join("uor_codegen_test");
        let sdk_tmp = std::env::temp_dir().join("uor_codegen_test_sdk");
        let _ = std::fs::create_dir_all(&tmp);
        let _ = std::fs::create_dir_all(&sdk_tmp);
        match generate(ontology, &tmp, &sdk_tmp) {
            Ok(report) => {
                assert_eq!(
                    report.trait_count,
                    uor_ontology::counts::CLASSES,
                    "Trait count should match CLASSES"
                );
                assert!(
                    report.method_count >= uor_ontology::counts::METHODS,
                    "Method count ({}) should be >= METHODS ({})",
                    report.method_count,
                    uor_ontology::counts::METHODS
                );
            }
            Err(e) => panic!("Code generation failed: {e}"),
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
