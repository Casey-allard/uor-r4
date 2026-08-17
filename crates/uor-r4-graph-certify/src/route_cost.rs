//! Typed cost/scope/objective/witness schema for the #602–#606 seam
//! (#653 phase-3: "Apply the typed plan/scope/resource/cost/witness
//! pattern to at least one active pipeline seam").
//!
//! This is an r4-native DESIGN, not a Lean-to-Rust port. It adopts the
//! *shape* of `proofs/wasm-gemm-gnaf/WasmGemmGnaf/Cost/`'s pattern
//! (static/dynamic-sum/dynamic-max cost coordinates, a declared
//! first-order weight objective, a declared resource bound, and a
//! composed witness carrying a claim) to the one quantity this seam
//! already measures in closed form: [`RouteOpCensus`], #604's
//! data-independent per-step operation count.
//!
//! ## What is deliberately NOT adopted
//!
//! - GNAF's `ProperObjective`/sublevel-finiteness machinery exists to
//!   discharge a Lean global-optimality theorem. This seam has no such
//!   theorem; a declared, checked resource ceiling is enough. Porting
//!   the finiteness proof apparatus here would be substantial unused
//!   weight — skipped for v1, revisit only if a future r4 proof
//!   obligation actually needs it.
//! - A "static" (one-time construction) cost coordinate in GNAF's sense
//!   does not have a clean per-run analogue here: a route-attention
//!   instance is rebuilt fresh each step in the fit ladder (the
//!   candidate window grows causally with position), so there is no
//!   single "build the artifact once" charge to measure. [`RouteStaticCost`]
//!   is therefore a DECLARED bound (the worst-case instance shape at
//!   [`ROUTE_MAX_CANDIDATES`]/[`ROUTE_MAX_TOP_M`]), not a measured
//!   per-run quantity — [`RouteCostVector::static_cost`] on a measured
//!   witness is always [`RouteStaticCost::worst_case`].
//!
//! ## Composition, never duplication
//!
//! The dynamic coordinates are [`RouteOpCensus`]'s own seven fields,
//! reused verbatim (not reinterpreted) — a census IS a dynamic cost
//! vector under this schema, so folding many steps' censuses is folding
//! many steps' costs. Nothing here re-measures or re-derives what
//! `uor-r4-graph-format::route_attention`'s closed forms and
//! `uor-r4-graph-certify::route_fit_report`'s ladder already compute.
//!
//! ## Status/claim vocabulary: reused in spirit, mirrored not imported
//!
//! [`RouteCostWitness`] reports [`ExecutionStatus`]/[`OptimizationStatus`]
//! matching `uor_r4_naf::claims`'s GNAF §12.4/§12.5 vocabulary (already
//! adopted by #623/PR#631) variant-for-variant, rather than inventing a
//! parallel pass/fail type -- but this module does NOT depend on
//! `uor-r4-naf` and instead mirrors just these two enums locally.
//! Reason: `uor-r4-naf`'s own `Cargo.toml` declares it "never a
//! dependency of a shipped r4 crate," and `uor-r4-graph-certify` IS
//! (transitively, via `uor-r4-proof-model`, which the root `r4` binary
//! depends on directly) part of that shipped graph -- adding the edge
//! here would violate an existing, deliberate architectural boundary.
//! Mirroring the vocabulary (same variants, same meaning, additionally
//! serde-derived here since this witness is serialized) keeps the
//! GNAF-vocabulary discipline #653 asks for without crossing it.
//!
//! A cost witness records the measurement and whether it fits the
//! declared bound; it does NOT gate [`crate::target_operator_certificate::derive_overall_quality`] --
//! folding cost compliance into that derivation is a quality-rule
//! change, which the certificate schema's own #600 discipline requires
//! to be a new registry version, not an in-place edit of v1. This
//! module only adds the additive, informational
//! [`crate::route_fit_report::RuntimeChecks::cost`] field; promoting it
//! to a hard gate is a deliberate, separately-reviewed follow-up.
//!
//! ## Identity, following the #600 pattern
//!
//! [`RouteCostObjective`] and [`RouteResourceBounds`] each carry a
//! pinned canonical line format, a `blake3:<hex>` declared-identity
//! digest, and a versioned registry ([`route_cost_objective`],
//! [`route_resource_bounds`]) that refuses an unknown `(id, version)`
//! by name on the sanctioned [`SourceUnavailable`] surface — the same
//! discipline `AttentionOperatorSpec`/`TargetOperatorCertificateSpec`
//! already use.

use serde::{Deserialize, Serialize};
use uor_r4_graph_format::route_attention::{RouteOpCensus, ROUTE_MAX_CANDIDATES, ROUTE_MAX_TOP_M};
use uor_r4_model_source::SourceUnavailable;

/// Mirrors `uor_r4_naf::claims::ExecutionStatus` (GNAF §12.5)
/// variant-for-variant. See the module doc's "Status/claim vocabulary"
/// section for why this is a local mirror rather than a dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    NotRun,
    Accepted,
    Invalid,
    Unresolved,
    Unsupported,
    Unadmitted,
    Incoherent,
    Unsealed,
}

/// Mirrors `uor_r4_naf::claims::OptimizationStatus` (GNAF §12.5)
/// variant-for-variant. See the module doc's "Status/claim vocabulary"
/// section for why this is a local mirror rather than a dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationStatus {
    NotRequested,
    Certified,
    Infeasible,
    Unattained,
    OptimizationIncomplete,
}

/// Declared (never measured) static-shape bound: the worst-case
/// instance size at the format's own declared caps. See the module
/// doc for why this is declared rather than measured.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteStaticCost {
    /// Worst-case instance bytes: header + mask + N candidate code
    /// windows + N contributions, at `N = ROUTE_MAX_CANDIDATES`.
    pub instance_bytes: u64,
}

impl RouteStaticCost {
    /// The declared worst-case bound at the format's own caps.
    pub fn worst_case() -> Self {
        const HEADER_LEN: u64 = 16;
        const CODE_BYTES: u64 = 36;
        let n = ROUTE_MAX_CANDIDATES as u64;
        Self {
            instance_bytes: HEADER_LEN + CODE_BYTES + n * CODE_BYTES + n * 4,
        }
    }
}

/// Dynamic (per-step) coordinates: [`RouteOpCensus`]'s own seven
/// fields, reused verbatim.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteDynamicCost {
    pub adds: u64,
    pub xors: u64,
    pub popcounts: u64,
    pub compares: u64,
    pub table_reads: u64,
    pub bytes_read: u64,
    pub candidates_examined: u64,
}

impl From<RouteOpCensus> for RouteDynamicCost {
    fn from(census: RouteOpCensus) -> Self {
        Self {
            adds: census.adds,
            xors: census.xors,
            popcounts: census.popcounts,
            compares: census.compares,
            table_reads: census.table_reads,
            bytes_read: census.bytes_read,
            candidates_examined: census.candidates_examined,
        }
    }
}

impl RouteDynamicCost {
    /// Componentwise sum (GNAF `sequentialCompose`'s additive half —
    /// every one of this seam's seven coordinates is cumulative, none
    /// is a peak quantity, so there is no max half to split out here).
    pub fn add(&self, other: &Self) -> Self {
        Self {
            adds: self.adds + other.adds,
            xors: self.xors + other.xors,
            popcounts: self.popcounts + other.popcounts,
            compares: self.compares + other.compares,
            table_reads: self.table_reads + other.table_reads,
            bytes_read: self.bytes_read + other.bytes_read,
            candidates_examined: self.candidates_examined + other.candidates_examined,
        }
    }

    /// Componentwise max (GNAF `dynamicMax`'s aggregation across the
    /// full raw-invocation domain — here, across steps).
    pub fn componentwise_max(&self, other: &Self) -> Self {
        Self {
            adds: self.adds.max(other.adds),
            xors: self.xors.max(other.xors),
            popcounts: self.popcounts.max(other.popcounts),
            compares: self.compares.max(other.compares),
            table_reads: self.table_reads.max(other.table_reads),
            bytes_read: self.bytes_read.max(other.bytes_read),
            candidates_examined: self.candidates_examined.max(other.candidates_examined),
        }
    }
}

/// The composed cost vector: SPEC-9.1-style split, `dynamic_sum`
/// accumulated by addition across steps, `dynamic_max` accumulated by
/// componentwise max across steps — GNAF's `ArtifactVector` shape,
/// scaled to this seam's seven measured coordinates.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCostVector {
    pub static_cost: RouteStaticCost,
    pub dynamic_sum: RouteDynamicCost,
    pub dynamic_max: RouteDynamicCost,
}

impl RouteCostVector {
    /// The additive identity: zero dynamic cost, the declared
    /// worst-case static bound (static cost is declared, not
    /// accumulated — see the module doc).
    pub fn zero() -> Self {
        Self {
            static_cost: RouteStaticCost::worst_case(),
            dynamic_sum: RouteDynamicCost::default(),
            dynamic_max: RouteDynamicCost::default(),
        }
    }

    /// Fold one more step's census into this vector — GNAF
    /// `sequentialCompose`, restricted to this seam's all-cumulative
    /// coordinate set.
    pub fn accumulate_step(&mut self, census: RouteOpCensus) {
        let step: RouteDynamicCost = census.into();
        self.dynamic_sum = self.dynamic_sum.add(&step);
        self.dynamic_max = self.dynamic_max.componentwise_max(&step);
    }

    /// Combine two already-aggregated vectors (e.g. across heads or
    /// across scopes) — sum the sums, max the maxes, keep the shared
    /// declared static bound. Associative and commutative, with
    /// [`RouteCostVector::zero`] as identity.
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            static_cost: self.static_cost,
            dynamic_sum: self.dynamic_sum.add(&other.dynamic_sum),
            dynamic_max: self.dynamic_max.componentwise_max(&other.dynamic_max),
        }
    }
}

/// A componentwise weight body — first-order, serializable, no
/// closures (GNAF `ObjectiveBody`). `static_weights`/`dynamic_weights`
/// are interpreted as per-coordinate multipliers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCostObjective {
    pub id: String,
    pub version: u32,
    pub static_weights: RouteStaticCost,
    pub dynamic_sum_weights: RouteDynamicCost,
}

fn dynamic_dot(weights: &RouteDynamicCost, values: &RouteDynamicCost) -> u64 {
    weights.adds.saturating_mul(values.adds)
        + weights.xors.saturating_mul(values.xors)
        + weights.popcounts.saturating_mul(values.popcounts)
        + weights.compares.saturating_mul(values.compares)
        + weights.table_reads.saturating_mul(values.table_reads)
        + weights.bytes_read.saturating_mul(values.bytes_read)
        + weights
            .candidates_examined
            .saturating_mul(values.candidates_examined)
}

/// The weighted-sum score of a cost vector under an objective (GNAF
/// `evaluate`). The canonical (v1, "first release") objective weighs
/// every coordinate 1, so this collapses to a plain sum, exactly as
/// GNAF's `CanonicalObjective.score` does.
pub fn score(objective: &RouteCostObjective, cost: &RouteCostVector) -> u64 {
    objective
        .static_weights
        .instance_bytes
        .saturating_mul(cost.static_cost.instance_bytes)
        + dynamic_dot(&objective.dynamic_sum_weights, &cost.dynamic_sum)
}

impl RouteCostObjective {
    fn canonical_v1() -> Self {
        Self {
            id: "route-cost-canonical".to_owned(),
            version: 1,
            static_weights: RouteStaticCost { instance_bytes: 1 },
            dynamic_sum_weights: RouteDynamicCost {
                adds: 1,
                xors: 1,
                popcounts: 1,
                compares: 1,
                table_reads: 1,
                bytes_read: 1,
                candidates_examined: 1,
            },
        }
    }

    /// Pinned canonical line format the declared-identity digest is
    /// computed over — the #600 pattern.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!(
            "route-cost-objective/{}/{}/static={}/dyn_sum=adds:{},xors:{},popcounts:{},\
             compares:{},table_reads:{},bytes_read:{},candidates_examined:{}",
            self.id,
            self.version,
            self.static_weights.instance_bytes,
            self.dynamic_sum_weights.adds,
            self.dynamic_sum_weights.xors,
            self.dynamic_sum_weights.popcounts,
            self.dynamic_sum_weights.compares,
            self.dynamic_sum_weights.table_reads,
            self.dynamic_sum_weights.bytes_read,
            self.dynamic_sum_weights.candidates_examined,
        )
        .into_bytes()
    }

    /// `blake3:<hex>` declared-identity digest over [`Self::canonical_bytes`].
    pub fn declared_digest(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.canonical_bytes()).to_hex())
    }
}

/// Registry id of the canonical (all-weights-1) objective.
pub const ROUTE_COST_OBJECTIVE_ID: &str = "route-cost-canonical";
/// Registry version of the canonical objective.
pub const ROUTE_COST_OBJECTIVE_VERSION: u32 = 1;

/// Versioned registry lookup — refuses an unknown `(id, version)` by
/// name, never guesses (the #600 discipline).
pub fn route_cost_objective(
    id: &str,
    version: u32,
) -> Result<RouteCostObjective, SourceUnavailable> {
    match (id, version) {
        (ROUTE_COST_OBJECTIVE_ID, ROUTE_COST_OBJECTIVE_VERSION) => {
            Ok(RouteCostObjective::canonical_v1())
        }
        _ => Err(SourceUnavailable::new(format!(
            "unknown route-cost objective ({id}, {version}); registered: \
             {ROUTE_COST_OBJECTIVE_ID}/{ROUTE_COST_OBJECTIVE_VERSION}"
        ))),
    }
}

/// A declared per-coordinate ceiling a certificate's measured cost
/// vector must stay under (GNAF `ResourceBounds`). v1 is populated
/// from constants and closed forms this seam already declares: the
/// format's own [`ROUTE_MAX_CANDIDATES`]/[`ROUTE_MAX_TOP_M`] caps and
/// `expected_route_census`'s closed form evaluated at those caps for a
/// single step, then scaled by a declared maximum step count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteResourceBounds {
    pub id: String,
    pub version: u32,
    pub max_steps: u64,
    pub max_static: RouteStaticCost,
    pub max_dynamic_sum: RouteDynamicCost,
    pub max_dynamic_max: RouteDynamicCost,
}

/// Whether a measured cost vector fits within a declared bound —
/// every coordinate, both static and dynamic (sum and max), must be
/// at or under its ceiling.
pub fn within(bounds: &RouteResourceBounds, cost: &RouteCostVector) -> bool {
    cost.static_cost.instance_bytes <= bounds.max_static.instance_bytes
        && dynamic_le(&cost.dynamic_sum, &bounds.max_dynamic_sum)
        && dynamic_le(&cost.dynamic_max, &bounds.max_dynamic_max)
}

fn dynamic_le(cost: &RouteDynamicCost, bound: &RouteDynamicCost) -> bool {
    cost.adds <= bound.adds
        && cost.xors <= bound.xors
        && cost.popcounts <= bound.popcounts
        && cost.compares <= bound.compares
        && cost.table_reads <= bound.table_reads
        && cost.bytes_read <= bound.bytes_read
        && cost.candidates_examined <= bound.candidates_examined
}

impl RouteResourceBounds {
    /// v1 bound: [`DEFAULT_MAX_STEPS`] steps at the format's own
    /// declared caps (`N = ROUTE_MAX_CANDIDATES`, `M = ROUTE_MAX_TOP_M`),
    /// using `expected_route_census`'s closed form for one step scaled
    /// by the step ceiling.
    fn declared_v1() -> Self {
        let per_step = uor_r4_graph_format::route_attention::RouteOpCensus {
            adds: 36 * ROUTE_MAX_CANDIDATES as u64 + ROUTE_MAX_TOP_M as u64,
            xors: 36 * ROUTE_MAX_CANDIDATES as u64,
            popcounts: 36 * ROUTE_MAX_CANDIDATES as u64,
            compares: (ROUTE_MAX_TOP_M * ROUTE_MAX_CANDIDATES) as u64,
            table_reads: 2 * ROUTE_MAX_CANDIDATES as u64 + ROUTE_MAX_TOP_M as u64,
            bytes_read: 72 * ROUTE_MAX_CANDIDATES as u64 + 4 * ROUTE_MAX_TOP_M as u64,
            candidates_examined: ROUTE_MAX_CANDIDATES as u64,
        };
        let per_step: RouteDynamicCost = per_step.into();
        let scaled = |value: u64| value.saturating_mul(DEFAULT_MAX_STEPS);
        Self {
            id: ROUTE_RESOURCE_BOUNDS_ID.to_owned(),
            version: ROUTE_RESOURCE_BOUNDS_VERSION,
            max_steps: DEFAULT_MAX_STEPS,
            max_static: RouteStaticCost::worst_case(),
            max_dynamic_sum: RouteDynamicCost {
                adds: scaled(per_step.adds),
                xors: scaled(per_step.xors),
                popcounts: scaled(per_step.popcounts),
                compares: scaled(per_step.compares),
                table_reads: scaled(per_step.table_reads),
                bytes_read: scaled(per_step.bytes_read),
                candidates_examined: scaled(per_step.candidates_examined),
            },
            // The worst single step is bounded by the same per-step
            // closed form at the format's own caps -- a step can never
            // cost more than the declared-cap closed form regardless
            // of how many steps ran.
            max_dynamic_max: per_step,
        }
    }

    /// Pinned canonical line format.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!(
            "route-resource-bounds/{}/{}/max_steps={}/max_static={}/\
             max_dyn_sum=adds:{},xors:{},popcounts:{},compares:{},table_reads:{},\
             bytes_read:{},candidates_examined:{}/\
             max_dyn_max=adds:{},xors:{},popcounts:{},compares:{},table_reads:{},\
             bytes_read:{},candidates_examined:{}",
            self.id,
            self.version,
            self.max_steps,
            self.max_static.instance_bytes,
            self.max_dynamic_sum.adds,
            self.max_dynamic_sum.xors,
            self.max_dynamic_sum.popcounts,
            self.max_dynamic_sum.compares,
            self.max_dynamic_sum.table_reads,
            self.max_dynamic_sum.bytes_read,
            self.max_dynamic_sum.candidates_examined,
            self.max_dynamic_max.adds,
            self.max_dynamic_max.xors,
            self.max_dynamic_max.popcounts,
            self.max_dynamic_max.compares,
            self.max_dynamic_max.table_reads,
            self.max_dynamic_max.bytes_read,
            self.max_dynamic_max.candidates_examined,
        )
        .into_bytes()
    }

    /// `blake3:<hex>` declared-identity digest over [`Self::canonical_bytes`].
    pub fn declared_digest(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.canonical_bytes()).to_hex())
    }
}

/// A declared step ceiling for the v1 bound: generous enough for the
/// fit ladder's largest declared scope (`model`, whole-model) while
/// still being an explicit, checked number rather than "unbounded."
/// Revisit if a scope's real step count ever approaches this.
pub const DEFAULT_MAX_STEPS: u64 = 1_000_000;

/// Registry id of the v1 declared resource bound.
pub const ROUTE_RESOURCE_BOUNDS_ID: &str = "route-cost-bounds-v1";
/// Registry version of the v1 declared resource bound.
pub const ROUTE_RESOURCE_BOUNDS_VERSION: u32 = 1;

/// Versioned registry lookup — refuses an unknown `(id, version)` by
/// name, never guesses.
pub fn route_resource_bounds(
    id: &str,
    version: u32,
) -> Result<RouteResourceBounds, SourceUnavailable> {
    match (id, version) {
        (ROUTE_RESOURCE_BOUNDS_ID, ROUTE_RESOURCE_BOUNDS_VERSION) => {
            Ok(RouteResourceBounds::declared_v1())
        }
        _ => Err(SourceUnavailable::new(format!(
            "unknown route-resource bounds ({id}, {version}); registered: \
             {ROUTE_RESOURCE_BOUNDS_ID}/{ROUTE_RESOURCE_BOUNDS_VERSION}"
        ))),
    }
}

/// A composed cost witness: the measured vector, which registered
/// objective scored it and bound checked it, the score and bound
/// verdict, and the GNAF §12.4/§12.5 status/claim reported
/// independently of each other (`uor_r4_naf::claims` — reused, not
/// reinvented; see the module doc for why this never gates the #606
/// quality derivation).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCostWitness {
    pub cost: RouteCostVector,
    pub objective_id: String,
    pub objective_version: u32,
    pub score: u64,
    pub bounds_id: String,
    pub bounds_version: u32,
    pub within_bounds: bool,
    pub execution: ExecutionStatus,
    pub optimization: OptimizationStatus,
}

/// Compose a witness from a measured cost vector against the
/// registered v1 objective and bound. Pure and deterministic: the same
/// cost vector always produces the same witness.
pub fn compute_route_cost_witness(cost: RouteCostVector) -> RouteCostWitness {
    // Registered lookups of the canonical (id, version) pair: cannot
    // fail (this module registers exactly what it looks up), so a
    // mismatch here would be this module's own bug, not a caller
    // input problem.
    let objective = route_cost_objective(ROUTE_COST_OBJECTIVE_ID, ROUTE_COST_OBJECTIVE_VERSION)
        .expect("the canonical route-cost objective is always registered");
    let bounds = route_resource_bounds(ROUTE_RESOURCE_BOUNDS_ID, ROUTE_RESOURCE_BOUNDS_VERSION)
        .expect("the v1 route-resource bounds are always registered");
    let measured_score = score(&objective, &cost);
    let fits = within(&bounds, &cost);
    RouteCostWitness {
        cost,
        objective_id: objective.id.clone(),
        objective_version: objective.version,
        score: measured_score,
        bounds_id: bounds.id.clone(),
        bounds_version: bounds.version,
        within_bounds: fits,
        // The measurement itself always completes (accumulation cannot
        // fail); ACCEPTED reports that. Whether the resource claim
        // itself holds is `within_bounds` plus the optimization status
        // below -- an accepted execution with `within_bounds = false`
        // is a truthful "measured and over budget," never a refused
        // measurement.
        execution: ExecutionStatus::Accepted,
        // This seam has no "solve for the cheapest possible cost"
        // search -- it measures what the deployed kernel actually
        // does against a declared ceiling, so no optimization claim is
        // being requested. `NotRequested` reports that honestly (the
        // GNAF-vocabulary analogue of GlobalOptimal/BestKnown is
        // reserved for a genuine search process, which does not exist
        // at this seam).
        optimization: OptimizationStatus::NotRequested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn census(adds: u64) -> RouteOpCensus {
        RouteOpCensus {
            adds,
            xors: adds + 1,
            popcounts: adds + 2,
            compares: adds + 3,
            table_reads: adds + 4,
            bytes_read: adds + 5,
            candidates_examined: adds + 6,
        }
    }

    #[test]
    fn accumulate_step_sums_and_maxes_independently() {
        let mut vector = RouteCostVector::zero();
        vector.accumulate_step(census(10));
        vector.accumulate_step(census(20));
        vector.accumulate_step(census(5));
        // Sum: 10+20+5 = 35 (and each field's own +offset per step).
        assert_eq!(vector.dynamic_sum.adds, 35);
        assert_eq!(vector.dynamic_sum.xors, 38); // (11+21+6)
                                                 // Max: the 20-census step dominates every field (all fields
                                                 // are `adds + constant`, so the largest `adds` step maxes
                                                 // every field).
        assert_eq!(vector.dynamic_max.adds, 20);
        assert_eq!(vector.dynamic_max.xors, 21);
        assert_eq!(vector.dynamic_max.candidates_examined, 26);
        // Static cost is the declared bound, never accumulated.
        assert_eq!(vector.static_cost, RouteStaticCost::worst_case());
    }

    #[test]
    fn combine_is_associative_with_zero_identity() {
        let mut a = RouteCostVector::zero();
        a.accumulate_step(census(1));
        let mut b = RouteCostVector::zero();
        b.accumulate_step(census(2));
        let mut c = RouteCostVector::zero();
        c.accumulate_step(census(3));

        let left = a.combine(&b).combine(&c);
        let right = a.combine(&b.combine(&c));
        assert_eq!(left, right);

        let with_zero = a.combine(&RouteCostVector::zero());
        // Combining with zero adds zero dynamic cost, changing nothing
        // but leaving the shared declared static bound in place.
        assert_eq!(with_zero.dynamic_sum, a.dynamic_sum);
        assert_eq!(with_zero.dynamic_max, a.dynamic_max);
    }

    #[test]
    fn canonical_objective_is_a_plain_sum_of_all_coordinates() {
        let objective = route_cost_objective(ROUTE_COST_OBJECTIVE_ID, ROUTE_COST_OBJECTIVE_VERSION)
            .expect("registered");
        let mut cost = RouteCostVector::zero();
        cost.accumulate_step(census(1));
        let expected_dynamic: u64 = [
            cost.dynamic_sum.adds,
            cost.dynamic_sum.xors,
            cost.dynamic_sum.popcounts,
            cost.dynamic_sum.compares,
            cost.dynamic_sum.table_reads,
            cost.dynamic_sum.bytes_read,
            cost.dynamic_sum.candidates_examined,
        ]
        .iter()
        .sum();
        let expected = cost.static_cost.instance_bytes + expected_dynamic;
        assert_eq!(score(&objective, &cost), expected);
    }

    #[test]
    fn unknown_objective_and_bounds_are_refused_by_name() {
        assert!(route_cost_objective("nope", 1).is_err());
        assert!(route_cost_objective(ROUTE_COST_OBJECTIVE_ID, 99).is_err());
        assert!(route_resource_bounds("nope", 1).is_err());
        assert!(route_resource_bounds(ROUTE_RESOURCE_BOUNDS_ID, 99).is_err());
    }

    #[test]
    fn identity_digest_is_stable_and_changes_with_the_body() {
        let a = route_cost_objective(ROUTE_COST_OBJECTIVE_ID, ROUTE_COST_OBJECTIVE_VERSION)
            .expect("registered");
        let b = route_cost_objective(ROUTE_COST_OBJECTIVE_ID, ROUTE_COST_OBJECTIVE_VERSION)
            .expect("registered");
        assert_eq!(a.declared_digest(), b.declared_digest());
        let mut c = a.clone();
        c.dynamic_sum_weights.adds = 2;
        assert_ne!(a.declared_digest(), c.declared_digest());
        assert!(a.declared_digest().starts_with("blake3:"));
    }

    #[test]
    fn a_tiny_measured_vector_fits_the_v1_bound() {
        let mut cost = RouteCostVector::zero();
        cost.accumulate_step(census(1));
        cost.accumulate_step(census(2));
        let witness = compute_route_cost_witness(cost);
        assert!(witness.within_bounds);
        assert_eq!(witness.execution, ExecutionStatus::Accepted);
        assert_eq!(witness.optimization, OptimizationStatus::NotRequested);
        assert_eq!(witness.objective_id, ROUTE_COST_OBJECTIVE_ID);
        assert_eq!(witness.bounds_id, ROUTE_RESOURCE_BOUNDS_ID);
    }

    #[test]
    fn a_vector_over_the_declared_bound_is_reported_not_within_bounds() {
        // Fabricate an absurdly large dynamic_sum directly (bypassing
        // accumulate_step, which could never reach this in practice
        // within the declared step/candidate caps) to exercise the
        // over-budget arm honestly: measured and out of bounds, never
        // silently clamped or refused.
        let mut cost = RouteCostVector::zero();
        cost.dynamic_sum.adds = u64::MAX / 2;
        let witness = compute_route_cost_witness(cost);
        assert!(!witness.within_bounds);
        // Still ACCEPTED: the measurement completed; the resource
        // claim just did not hold.
        assert_eq!(witness.execution, ExecutionStatus::Accepted);
    }
}
