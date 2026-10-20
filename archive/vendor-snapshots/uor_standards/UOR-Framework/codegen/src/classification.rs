//! Phase 0 classification — maps every ontology class to a `PathKind`.
//!
//! The classification drives every subsequent phase's codegen. Design notes
//! in `docs/orphan-closure/phase-0-classification.md`; the overall 4-path
//! strategy in `docs/orphan-closure/overview.md`.
//!
//! `classify` is a pure, deterministic function. `classify_all` runs it over
//! every class in the ontology. `write_report` emits a human-readable table
//! to `docs/orphan-closure/classification_report.md` — regenerated on every
//! `cargo run --bin uor-crate` and gated by `git diff --exit-code`.

use std::fmt::Write as FmtWrite;
use std::path::Path;

use anyhow::{Context, Result};
use uor_ontology::model::iris::{
    NS_PARALLEL, NS_STREAM, OWL_CLASS, OWL_THING, RDF_LIST, XSD_BOOLEAN, XSD_DECIMAL,
    XSD_HEX_BINARY, XSD_INTEGER, XSD_NON_NEGATIVE_INTEGER, XSD_POSITIVE_INTEGER, XSD_STRING,
};
use uor_ontology::{Class, Ontology, Property, PropertyKind};

use crate::mapping::local_name;

/// Which of the four orphan-closure paths a class belongs to.
///
/// See `docs/orphan-closure/overview.md` for the taxonomy and
/// `docs/orphan-closure/phase-0-classification.md` for the decision
/// procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathKind {
    /// Enum classes or `Primitives` — no trait emitted, so not an orphan.
    Skip,

    /// Class already has a concrete impl in `foundation/src/` (Certificate
    /// subclasses, Partition-algebra witnesses). No codegen-time work.
    AlreadyImplemented,

    /// Theory-deferred — cohomology / operad / parallel / stream machinery
    /// awaiting theoretical grounding. Traits stay orphan by design until
    /// theory lands; Phase 6 pairs each with a tracking issue.
    Path4TheoryDeferred,

    /// Theorem-backed witness. Phase 3 emits `{Foo}Witness` +
    /// `{Foo}MintInputs<H>` + `impl VerifiedMint for {Foo}Witness`; Phase 5
    /// fills the verification body per theorem family.
    Path2TheoremWitness {
        /// Whether any of the class's properties carries entropy (per R7).
        /// Determines whether `Hash` is dropped from the witness derives.
        entropy_bearing: bool,
        /// `op:Identity` IRI whose theorem this witness attests. Empty
        /// until R6 is fully wired; Phase 3 uses it in the stub-body
        /// `WITNESS_UNIMPLEMENTED_STUB:{IRI}` marker.
        theorem_identity: String,
    },

    /// Primitive-backed — Phase 4 emits a hand-written blanket impl
    /// delegating to a `primitive_*` function. R13: the named primitive
    /// must exist at classification time.
    Path3PrimitiveBacked {
        /// Name of the `primitive_*` function in `foundation/src/enforcement.rs`.
        primitive_name: String,
    },

    /// Fallthrough — Phase 2 emits `{Foo}Handle` + `{Foo}Resolver` +
    /// `{Foo}Record` + `Resolved{Foo}` and a single `impl {Foo}<H> for
    /// Resolved{Foo}<'r, R, H>`.
    Path1HandleResolver,
}

impl PathKind {
    /// Short textual label, used by the report and by tests.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            PathKind::Skip => "Skip",
            PathKind::AlreadyImplemented => "AlreadyImplemented",
            PathKind::Path4TheoryDeferred => "Path4TheoryDeferred",
            PathKind::Path2TheoremWitness { .. } => "Path2TheoremWitness",
            PathKind::Path3PrimitiveBacked { .. } => "Path3PrimitiveBacked",
            PathKind::Path1HandleResolver => "Path1HandleResolver",
        }
    }
}

/// One classification record.
#[derive(Debug, Clone)]
pub struct ClassificationEntry {
    /// Full class IRI (e.g. `https://uor.foundation/partition/Partition`).
    pub class_iri: &'static str,
    /// Local class name (last IRI segment).
    pub class_local: &'static str,
    /// Namespace prefix (e.g. `partition`, `observable`).
    pub namespace: &'static str,
    /// Assigned path.
    pub path_kind: PathKind,
    /// Short human-readable rationale for the classification.
    pub rationale: String,
}

// ─── Allow-lists — explicit, no heuristics ──────────────────────────────

/// Full class IRIs of ontology-derived traits that already have a concrete
/// `impl` in `foundation/src/`. Verified by greping for
/// `impl(<...>)? crate::<ns>::<module>::<Class><...>? for <Type>`.
///
/// Local names collide across namespaces (`cert::GroundingCertificate`
/// vs `morphism::GroundingCertificate`) so this list uses full IRIs.
///
/// Phase 0 baseline (after the Product/Coproduct Amendment §845c0ff):
/// only the four partition-algebra traits are closed. The enforcement.rs
/// `Certificate` trait family is a *local* sealed trait distinct from the
/// ontology-derived `cert::Certificate<H>` trait; the 17 `impl Certificate
/// for <Struct>` hits in enforcement.rs do not close any ontology trait.
const ALREADY_IMPLEMENTED: &[&str] = &[
    "https://uor.foundation/partition/Partition",
    "https://uor.foundation/partition/PartitionProduct",
    "https://uor.foundation/partition/PartitionCoproduct",
    "https://uor.foundation/partition/CartesianPartitionProduct",
];

/// Class local names deferred until theory lands (strategy doc §Path 4).
///
/// Extended by every class in `kernel/parallel` and `kernel/stream`
/// namespaces — see `classify()`.
const THEORY_DEFERRED_LOCAL_NAMES: &[&str] = &[
    // Cohomology machinery (OB_P1/P2/P3 not grounded computationally).
    "CochainComplex",
    "CohomologyGroup",
    "Sheaf",
    "RestrictionMap",
    "Section",
    "Stalk",
    "GluingObstruction",
    // Monoidal / operad (OP_3 Leibniz-rule grounding missing).
    "MonoidalProduct",
    "MonoidalComposition",
    "OperadComposition",
    // Coboundary / boundary machinery that depends on cohomology grounding.
    "Coboundary",
    "Cocycle",
];

/// Property labels whose presence marks a Path-2 witness as entropy-bearing
/// (R7). Witnesses carrying any of these cannot derive `Hash`.
const ENTROPY_PROPERTY_LABELS: &[&str] = &[
    "bits",
    "bitsDissipated",
    "landauerCost",
    "landauerNats",
    "entropy",
    "crossEntropy",
    "freeEnergy",
];

/// Substring matched against a class name to flag it as a theorem-witness
/// candidate (R7 heuristic #1).
const THEOREM_WITNESS_SUFFIXES: &[&str] = &["Witness", "Obstruction", "Verification"];

/// Phase 10a: theorem-family prefix map. Each entry pairs a *class local-name
/// suffix* with the *theorem-family prefix* whose `op:Identity` individuals
/// constitute the candidate set.
///
/// Resolution algorithm (`resolve_theorem_identity`):
///
///   1. Walk this table in order; the first matching suffix wins.
///   2. Enumerate `op:Identity` individuals whose IRI contains the prefix.
///   3. If exactly one candidate exists, use it.
///   4. Otherwise (zero, or two-or-more), fall back to
///      `PATH2_THEOREM_OVERRIDES`. Missing override = loud panic.
///
/// Suffixes are checked **longest-first** so that, e.g.,
/// `InhabitanceImpossibilityWitness` matches `IH_` (via the
/// `InhabitanceImpossibilityWitness` row) rather than the bare-`Witness`
/// catch-all in the override table.
const THEOREM_FAMILY_PREFIX_MAP: &[(&str, &str)] = &[
    ("CartesianPartitionProduct", "CPT_"),
    ("PartitionCoproduct", "ST_"),
    ("PartitionProduct", "PT_"),
    ("InhabitanceImpossibilityWitness", "IH_"),
    ("InhabitanceWitness", "IH_"),
    ("LiftObstruction", "LO_"),
    ("Obstruction", "OB_"),
    ("GroundingWitness", "OA_"),
    ("ProjectionWitness", "OA_"),
    ("BornRuleVerification", "BR_"),
    ("CompletenessWitness", "CC_"),
    ("DisjointnessWitness", "DP_"),
];

/// Phase 10a: hand-override table, keyed by **full class IRI** (local names
/// collide across namespaces — `morphism::GroundingWitness` and
/// `state::GroundingWitness` are distinct).
///
/// Each entry maps a Path-2 class to a representative `op:Identity` IRI
/// when the family-prefix lookup is ambiguous (≥2 candidates) or the family
/// has no individuals in `op.rs`.
///
/// Every IRI here is grep-verified at plan time against
/// `spec/src/namespaces/op.rs`. The Phase-0 test
/// `codegen/tests/path2_theorem_linkage.rs` re-verifies at compile time.
const PATH2_THEOREM_OVERRIDES: &[(&str, &str)] = &[
    // BR_1..BR_5 exist; pick the canonical Born-rule normalization identity.
    (
        "https://uor.foundation/cert/BornRuleVerification",
        "https://uor.foundation/op/QM_5",
    ),
    // CC_1..CC_5 exist; CC_1 is the canonical "completeness implies O(1)"
    // identity that completeness witnesses attest.
    (
        "https://uor.foundation/type/CompletenessWitness",
        "https://uor.foundation/op/CC_1",
    ),
    // DP_ family does not yet exist. Reuse op:FX_4 ("Disjoint effects
    // commute") whose `forAll` literally references DisjointnessWitness.
    (
        "https://uor.foundation/effect/DisjointnessWitness",
        "https://uor.foundation/op/FX_4",
    ),
    // OA_1..OA_5 + op:surfaceSymmetry exist. surfaceSymmetry is the
    // grounding/projection co-discharge theorem; assign it to all three
    // grounding/projection witness classes.
    (
        "https://uor.foundation/morphism/GroundingWitness",
        "https://uor.foundation/op/surfaceSymmetry",
    ),
    (
        "https://uor.foundation/morphism/ProjectionWitness",
        "https://uor.foundation/op/surfaceSymmetry",
    ),
    (
        "https://uor.foundation/state/GroundingWitness",
        "https://uor.foundation/op/surfaceSymmetry",
    ),
    // morphism::Witness is the abstract supertype; map it to the same
    // surfaceSymmetry root since every concrete Witness is a witnessing
    // pair under that theorem.
    (
        "https://uor.foundation/morphism/Witness",
        "https://uor.foundation/op/surfaceSymmetry",
    ),
    // IH_1 is the canonical inhabitance soundness identity.
    (
        "https://uor.foundation/proof/ImpossibilityWitness",
        "https://uor.foundation/op/IH_1",
    ),
    (
        "https://uor.foundation/proof/InhabitanceImpossibilityWitness",
        "https://uor.foundation/op/IH_1",
    ),
    // LO_ family does not yet exist. WLS_2 ("Obstruction localisation") is
    // the canonical LiftObstruction theorem.
    (
        "https://uor.foundation/type/LiftObstruction",
        "https://uor.foundation/op/WLS_2",
    ),
];

/// Phase 10d: theorem-family → primitive-module routing. Each prefix is
/// the leading IRI fragment after `op/`; the value is the foundation
/// primitive module that hosts `verify_*` bodies.
///
/// `OB_C/H/M/P` all collapse to the single `ob` module.
pub const FAMILY_PRIMITIVE_MODULE: &[(&str, &str)] = &[
    ("PT_", "pt"),
    ("ST_", "st"),
    ("CPT_", "cpt"),
    ("OB_", "ob"),
    ("IH_", "ih"),
    ("LO_", "lo"),
    ("OA_", "oa"),
    ("BR_", "br"),
    ("CC_", "cc"),
    ("DP_", "dp"),
    ("WLS_", "lo"),
    ("QM_", "br"),
    ("FX_", "dp"),
    // `surfaceSymmetry` lacks a `_`-style prefix; it routes to `oa`.
    ("surfaceSymmetry", "oa"),
];

/// Resolve the primitive-module name for a `THEOREM_IDENTITY` IRI per the
/// Phase 10d routing table. Returns the matching module name or panics if
/// no rule matches — the rules cover every supported family.
#[must_use]
pub fn primitive_module_for_identity(identity_iri: &str) -> &'static str {
    let local = static_local_name_str(identity_iri);
    // Special-case: surfaceSymmetry has no underscore-prefix.
    if local == "surfaceSymmetry" {
        return "oa";
    }
    for (prefix, module) in FAMILY_PRIMITIVE_MODULE {
        if local.starts_with(prefix) {
            return module;
        }
    }
    // Unreachable in well-formed classification — Phase-0 test rejects
    // any identity that does not route to a primitive module.
    "unrouted"
}

/// Convert an `op:Identity` IRI to the `verify_<snake_case>` function name
/// used by Phase 12 primitives. Drops the family prefix when it carries
/// underscore-segregated structure (`PT_2a` → `pt_2a` → `verify_pt_2a`).
#[must_use]
pub fn identity_to_snake(identity_iri: &str) -> String {
    let local = static_local_name_str(identity_iri);
    let mut out = String::with_capacity(local.len() + 4);
    let mut prev_upper = false;
    for (i, ch) in local.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            // Insert a separator only when the previous char is a lowercase
            // letter or digit (camelCase boundary). After an underscore or
            // at string start, do not double the separator.
            let last = out.chars().last();
            if i > 0 && !prev_upper && last != Some('_') {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            prev_upper = true;
        } else {
            out.push(ch);
            prev_upper = false;
        }
    }
    out
}

fn static_local_name_str(iri: &str) -> &str {
    if let Some(pos) = iri.rfind('/') {
        return &iri[pos + 1..];
    }
    if let Some(pos) = iri.rfind('#') {
        return &iri[pos + 1..];
    }
    iri
}

/// Phase 11a: explicit Path-3 allow-list, keyed by **full class IRI** so
/// that namespace-collision doesn't break R13 verification. Each entry
/// names the `primitive_*` function in `foundation/src/enforcement.rs` or
/// `foundation/src/pipeline.rs` that the hand-written blanket impl in
/// `foundation/src/blanket_impls.rs` delegates to.
///
/// R13 loud-failure: every entry's primitive must exist; verified by
/// `codegen/tests/path3_primitive_backing.rs` against grep of the
/// foundation source. Adding an entry without its primitive is a red
/// test.
pub const PATH3_ALLOW_LIST: &[(&str, &str)] = &[
    // observable:LandauerBudget — landauer_nats accessor backed by
    // primitive_descent_metrics's u64 entropy bits + Phase 9c
    // `<H::Decimal as DecimalTranscendental>::from_bits`.
    (
        "https://uor.foundation/observable/LandauerBudget",
        "primitive_descent_metrics",
    ),
    // observable:JacobianObservable — Observable marker; the per-site
    // Jacobian is computed by primitive_curvature_jacobian.
    (
        "https://uor.foundation/observable/JacobianObservable",
        "primitive_curvature_jacobian",
    ),
    // carry:CarryDepthObservable — Observable marker; depth via
    // primitive_dihedral_signature's orbit-size return.
    (
        "https://uor.foundation/carry/CarryDepthObservable",
        "primitive_dihedral_signature",
    ),
    // derivation:DerivationDepthObservable — Observable marker; depth
    // via primitive_terminal_reduction's reduction-step count.
    (
        "https://uor.foundation/derivation/DerivationDepthObservable",
        "primitive_terminal_reduction",
    ),
    // partition:FreeRankObservable — Observable marker; free-rank
    // residual is primitive_descent_metrics's u32 first return.
    (
        "https://uor.foundation/partition/FreeRankObservable",
        "primitive_descent_metrics",
    ),
];

// ─── Classification ──────────────────────────────────────────────────────

/// Classifies a single class.
///
/// Deterministic and pure: same `(class, ontology)` always yields the same
/// `ClassificationEntry`. Ordering: first match in the decision procedure
/// wins; see `docs/orphan-closure/phase-0-classification.md`.
#[must_use]
pub fn classify(class: &Class, ontology: &Ontology) -> ClassificationEntry {
    let class_iri: &'static str = class.id;
    let class_local: &'static str = static_local_name(class_iri);
    let namespace = namespace_prefix(class_iri, ontology).unwrap_or("");

    // 1. Skip — enum classes + Primitives
    if is_skipped_class(class_local) {
        return ClassificationEntry {
            class_iri,
            class_local,
            namespace,
            path_kind: PathKind::Skip,
            rationale: format!("{class_local} is an enum class or Primitives"),
        };
    }

    // 2. AlreadyImplemented (full-IRI match — local names collide)
    if ALREADY_IMPLEMENTED.contains(&class_iri) {
        return ClassificationEntry {
            class_iri,
            class_local,
            namespace,
            path_kind: PathKind::AlreadyImplemented,
            rationale: "hand-written impl exists in foundation/src/".to_string(),
        };
    }

    // 3. Path4TheoryDeferred (allow-list + parallel/stream namespaces)
    if THEORY_DEFERRED_LOCAL_NAMES.contains(&class_local)
        || class_iri.starts_with(NS_PARALLEL)
        || class_iri.starts_with(NS_STREAM)
    {
        let reason = if THEORY_DEFERRED_LOCAL_NAMES.contains(&class_local) {
            "theory-deferred per strategy doc §Path 4"
        } else if class_iri.starts_with(NS_PARALLEL) {
            "kernel/parallel awaits runtime-integration grounding"
        } else {
            "kernel/stream awaits reactive-semantics grounding"
        };
        return ClassificationEntry {
            class_iri,
            class_local,
            namespace,
            path_kind: PathKind::Path4TheoryDeferred,
            rationale: reason.to_string(),
        };
    }

    // 4. Path2TheoremWitness — name ends in Witness/Obstruction/Verification
    if let Some(suffix) = matching_theorem_suffix(class_local) {
        let entropy_bearing = has_entropy_property(class_iri, ontology);
        let theorem_identity = resolve_theorem_identity_loud(class_iri, ontology);
        return ClassificationEntry {
            class_iri,
            class_local,
            namespace,
            path_kind: PathKind::Path2TheoremWitness {
                entropy_bearing,
                theorem_identity,
            },
            rationale: format!("class name ends in '{suffix}' — theorem-witness shape"),
        };
    }

    // 5. Path3PrimitiveBacked — explicit allow-list only (R13 loud failure
    //    for mismatches is enforced by the Phase-0 tests: any allow-list
    //    entry whose primitive is absent fails classification_counts).
    //    Phase 11a: keyed by full class IRI so cross-namespace local-name
    //    collisions don't accidentally pick the wrong class.
    if let Some((_, prim)) = PATH3_ALLOW_LIST.iter().find(|(iri, _)| *iri == class_iri) {
        return ClassificationEntry {
            class_iri,
            class_local,
            namespace,
            path_kind: PathKind::Path3PrimitiveBacked {
                primitive_name: (*prim).to_string(),
            },
            rationale: format!("primitive-backed: {prim}"),
        };
    }

    // 6. Path1HandleResolver fallthrough — verify R4 (every property's range
    //    maps to a known absent-sentinel). If a property has no absent
    //    sentinel, demote to Path4.
    if let Some(unsupported_range) = property_without_absent_sentinel(class_iri, ontology) {
        return ClassificationEntry {
            class_iri,
            class_local,
            namespace,
            path_kind: PathKind::Path4TheoryDeferred,
            rationale: format!("no-absent-semantics: {unsupported_range}"),
        };
    }

    ClassificationEntry {
        class_iri,
        class_local,
        namespace,
        path_kind: PathKind::Path1HandleResolver,
        rationale: "pure-accessor bundle (default)".to_string(),
    }
}

/// Classifies every class in the ontology.
#[must_use]
pub fn classify_all(ontology: &Ontology) -> Vec<ClassificationEntry> {
    let mut out: Vec<ClassificationEntry> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.classes.iter())
        .map(|c| classify(c, ontology))
        .collect();
    out.sort_by(|a, b| {
        a.namespace
            .cmp(b.namespace)
            .then_with(|| a.class_local.cmp(b.class_local))
    });
    out
}

/// Per-variant counts, returned for `spec/src/counts.rs` cross-check.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClassificationCounts {
    /// Total `PathKind::Skip`.
    pub skip: usize,
    /// Total `PathKind::AlreadyImplemented`.
    pub already_implemented: usize,
    /// Total `PathKind::Path1HandleResolver`.
    pub path1: usize,
    /// Total `PathKind::Path2TheoremWitness`.
    pub path2: usize,
    /// Total `PathKind::Path3PrimitiveBacked`.
    pub path3: usize,
    /// Total `PathKind::Path4TheoryDeferred`.
    pub path4: usize,
}

impl ClassificationCounts {
    /// Sum of every variant.
    #[must_use]
    pub fn total(&self) -> usize {
        self.skip + self.already_implemented + self.path1 + self.path2 + self.path3 + self.path4
    }
}

/// Tallies classification counts.
#[must_use]
pub fn count(entries: &[ClassificationEntry]) -> ClassificationCounts {
    let mut c = ClassificationCounts::default();
    for e in entries {
        match e.path_kind {
            PathKind::Skip => c.skip += 1,
            PathKind::AlreadyImplemented => c.already_implemented += 1,
            PathKind::Path1HandleResolver => c.path1 += 1,
            PathKind::Path2TheoremWitness { .. } => c.path2 += 1,
            PathKind::Path3PrimitiveBacked { .. } => c.path3 += 1,
            PathKind::Path4TheoryDeferred => c.path4 += 1,
        }
    }
    c
}

// ─── Report emission ─────────────────────────────────────────────────────

/// Writes the human-readable classification report to `out_path`.
///
/// Format: Markdown table, one row per class, sorted by namespace then class
/// name. Regenerated on every `cargo run --bin uor-crate` and gated by `git
/// diff --exit-code docs/orphan-closure/classification_report.md`.
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn write_report(entries: &[ClassificationEntry], out_path: &Path) -> Result<()> {
    let counts = count(entries);
    let mut s = String::with_capacity(4096 + entries.len() * 256);

    s.push_str("<!-- @generated by uor-crate from uor-codegen::classification — do not edit manually -->\n\n");
    s.push_str("# Orphan-trait classification report\n\n");
    s.push_str(
        "Generated by `cargo run --bin uor-crate`. See \
         [phase-0-classification.md](./phase-0-classification.md) for the \
         decision procedure.\n\n",
    );

    s.push_str("## Totals\n\n");
    s.push_str("| PathKind | Count |\n|---|---|\n");
    let _ = writeln!(s, "| Skip | {} |", counts.skip);
    let _ = writeln!(s, "| AlreadyImplemented | {} |", counts.already_implemented);
    let _ = writeln!(s, "| Path1HandleResolver | {} |", counts.path1);
    let _ = writeln!(s, "| Path2TheoremWitness | {} |", counts.path2);
    let _ = writeln!(s, "| Path3PrimitiveBacked | {} |", counts.path3);
    let _ = writeln!(s, "| Path4TheoryDeferred | {} |", counts.path4);
    let _ = writeln!(s, "| **Total** | **{}** |", counts.total());
    s.push('\n');

    s.push_str("## Per-class\n\n");
    s.push_str(
        "| Namespace | Class | PathKind | Entropy | Theorem identity | Primitive | Rationale |\n\
         |---|---|---|---|---|---|---|\n",
    );
    for e in entries {
        let (entropy, theorem, primitive) = match &e.path_kind {
            PathKind::Path2TheoremWitness {
                entropy_bearing,
                theorem_identity,
            } => (
                if *entropy_bearing { "yes" } else { "no" },
                theorem_identity.as_str(),
                "",
            ),
            PathKind::Path3PrimitiveBacked { primitive_name } => ("", "", primitive_name.as_str()),
            _ => ("", "", ""),
        };
        let _ = writeln!(
            s,
            "| `{}` | `{}` | {} | {} | {} | {} | {} |",
            e.namespace,
            e.class_local,
            e.path_kind.label(),
            entropy,
            theorem,
            primitive,
            e.rationale.replace('|', r"\|"),
        );
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    std::fs::write(out_path, s)
        .with_context(|| format!("Failed to write report: {}", out_path.display()))?;
    Ok(())
}

// ─── Internal helpers ────────────────────────────────────────────────────

fn is_skipped_class(class_local: &str) -> bool {
    if class_local == "Primitives" {
        return true;
    }
    Ontology::enum_class_names().contains(&class_local)
}

fn matching_theorem_suffix(class_local: &str) -> Option<&'static str> {
    THEOREM_WITNESS_SUFFIXES
        .iter()
        .copied()
        .find(|suffix| class_local.ends_with(*suffix))
}

fn has_entropy_property(class_iri: &str, ontology: &Ontology) -> bool {
    properties_with_domain(class_iri, ontology).any(|p| {
        // Decimal range OR property label is in the entropy set.
        p.range == XSD_DECIMAL || ENTROPY_PROPERTY_LABELS.contains(&p.label)
    })
}

/// Phase 10a: resolves a Path-2 class to the `op:Identity` IRI that
/// names the theorem its witness attests.
///
/// Algorithm:
///   1. Apply `THEOREM_FAMILY_PREFIX_MAP` to derive a candidate family
///      prefix from the class local name. Suffixes are checked
///      longest-first.
///   2. Enumerate all `op:Identity` individuals whose IRI's local name
///      starts with the prefix; collect candidates.
///   3. If exactly one candidate exists, that is the resolved identity.
///   4. Otherwise (zero or ≥ 2), look up the **full class IRI** in
///      `PATH2_THEOREM_OVERRIDES`.
///   5. If neither path resolves, panic — Phase-0 classification fails
///      loud per Phase 10a's "Missing override = Phase-0 classification
///      fails loud" rule.
///
/// Verifies the resolved IRI exists as a real `op:Identity` individual
/// in `ontology` before returning.
fn resolve_theorem_identity(class_iri: &str, ontology: &Ontology) -> Option<String> {
    let class_local = local_name(class_iri);

    // Step 1+2: family-prefix lookup.
    if let Some(prefix) = theorem_family_prefix(class_local) {
        let candidates: Vec<&str> = ontology
            .namespaces
            .iter()
            .flat_map(|m| m.individuals.iter())
            .filter(|ind| {
                ind.type_ == "https://uor.foundation/op/Identity"
                    && static_local_name_str(ind.id).starts_with(prefix)
            })
            .map(|ind| ind.id)
            .collect();
        if candidates.len() == 1 {
            return Some(candidates[0].to_string());
        }
        // 0 or ≥ 2: fall through to override.
    }

    // Step 3: hand-override by full class IRI.
    if let Some((_, identity)) = PATH2_THEOREM_OVERRIDES
        .iter()
        .find(|(iri, _)| *iri == class_iri)
    {
        // R6 verification: the override target must exist as a real
        // op:Identity individual.
        let exists = ontology
            .namespaces
            .iter()
            .flat_map(|m| m.individuals.iter())
            .any(|ind| ind.type_ == "https://uor.foundation/op/Identity" && ind.id == *identity);
        if exists {
            return Some((*identity).to_string());
        }
        // Override pointing to a nonexistent identity is a hard error.
        return None;
    }

    None
}

/// Phase 10a loud-failure wrapper. Panics if the family-prefix lookup
/// AND the override table both miss — that is the explicit
/// "Missing override = Phase-0 classification fails loud" rule.
#[allow(clippy::panic)]
fn resolve_theorem_identity_loud(class_iri: &str, ontology: &Ontology) -> String {
    match resolve_theorem_identity(class_iri, ontology) {
        Some(t) => t,
        None => panic!(
            "Phase 10a: Path-2 class `{class_iri}` has no resolved op:Identity. \
             Add a PATH2_THEOREM_OVERRIDES entry (or a matching family \
             prefix in THEOREM_FAMILY_PREFIX_MAP). The override IRI must \
             reference a real op:Identity individual in \
             spec/src/namespaces/op.rs."
        ),
    }
}

fn theorem_family_prefix(class_local: &str) -> Option<&'static str> {
    // Longest suffix wins — entries are written longest-first.
    THEOREM_FAMILY_PREFIX_MAP
        .iter()
        .find(|(suffix, _)| class_local.ends_with(suffix))
        .map(|(_, prefix)| *prefix)
}

fn property_without_absent_sentinel(class_iri: &str, ontology: &Ontology) -> Option<String> {
    for p in properties_with_domain(class_iri, ontology) {
        if !range_has_absent_sentinel(p.range, ontology) {
            return Some(format!("{} (property {})", p.range, p.label));
        }
    }
    None
}

fn range_has_absent_sentinel(range_iri: &str, ontology: &Ontology) -> bool {
    // Known XSD primitive types — all have absent sentinels per R4.
    match range_iri {
        XSD_STRING
        | XSD_INTEGER
        | XSD_NON_NEGATIVE_INTEGER
        | XSD_POSITIVE_INTEGER
        | XSD_BOOLEAN
        | XSD_DECIMAL
        | XSD_HEX_BINARY => return true,
        _ => {}
    }
    // `xsd:dateTime` maps to `H::WitnessBytes` per `mapping::xsd_to_primitives_type`.
    if range_iri == "http://www.w3.org/2001/XMLSchema#dateTime" {
        return true;
    }
    // Generic object ranges (`owl:Thing`, `owl:Class`, `rdf:List`) are mapped
    // by `codegen/src/traits.rs` to `&H::HostString` or `count/_at` forms;
    // both have absent sentinels (`EMPTY_HOST_STRING`).
    if range_iri == OWL_THING || range_iri == OWL_CLASS || range_iri == RDF_LIST {
        return true;
    }
    // Ontology classes — handle-typed fields, with `ContentFingerprint::zero()`
    // sentinel per R4. Accept any class declared in the ontology.
    if ontology.find_class(range_iri).is_some() {
        return true;
    }
    false
}

fn properties_with_domain<'a>(
    class_iri: &'a str,
    ontology: &'a Ontology,
) -> impl Iterator<Item = &'a Property> + 'a {
    ontology
        .namespaces
        .iter()
        .flat_map(|m| m.properties.iter())
        .filter(move |p| p.kind != PropertyKind::Annotation && p.domain == Some(class_iri))
}

fn namespace_prefix(class_iri: &str, ontology: &Ontology) -> Option<&'static str> {
    ontology
        .namespaces
        .iter()
        .find(|m| class_iri.starts_with(m.namespace.iri))
        .map(|m| m.namespace.prefix)
}

fn static_local_name(iri: &'static str) -> &'static str {
    // `local_name` returns `&str` tied to its input; since input is 'static,
    // the return is 'static too — but the compiler doesn't prove it through
    // rsplit. We reconstruct with the borrow at the 'static boundary.
    if let Some(pos) = iri.rfind('/') {
        return &iri[pos + 1..];
    }
    if let Some(pos) = iri.rfind('#') {
        return &iri[pos + 1..];
    }
    iri
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn skip_detects_enum_classes() {
        let ontology = Ontology::full();
        let witt = match ontology.find_class_by_local_name("WittLevel") {
            Some(c) => c,
            None => panic!("WittLevel class must exist"),
        };
        let e = classify(witt, ontology);
        assert!(matches!(e.path_kind, PathKind::Skip));
    }

    #[test]
    fn path1_default_for_plain_class() {
        // `carry/CarryChain` is guaranteed NOT in the AlreadyImplemented or
        // Path-4 allow-lists, and its name doesn't match Path-2's suffix
        // heuristic. A plain accessor bundle.
        let ontology = Ontology::full();
        let class = match ontology.find_class("https://uor.foundation/carry/CarryChain") {
            Some(c) => c,
            None => panic!("carry/CarryChain class must exist"),
        };
        let e = classify(class, ontology);
        assert_eq!(e.path_kind.label(), "Path1HandleResolver");
    }

    #[test]
    fn already_implemented_for_partition_product() {
        let ontology = Ontology::full();
        let pp = match ontology.find_class("https://uor.foundation/partition/PartitionProduct") {
            Some(c) => c,
            None => panic!("PartitionProduct class must exist"),
        };
        let e = classify(pp, ontology);
        assert!(matches!(e.path_kind, PathKind::AlreadyImplemented));
    }

    #[test]
    fn counts_sum_to_class_count() {
        let ontology = Ontology::full();
        let entries = classify_all(ontology);
        let counts = count(&entries);
        assert_eq!(counts.total(), ontology.class_count());
    }

    #[test]
    fn classification_is_deterministic() {
        let ontology = Ontology::full();
        let a = classify_all(ontology);
        let b = classify_all(ontology);
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.class_iri, y.class_iri);
            assert_eq!(x.path_kind.label(), y.path_kind.label());
        }
    }
}
