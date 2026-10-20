//! `observable/` namespace — Observable quantities and metrics.
//!
//! Observables are kernel-computed measurements of UOR objects. They form a
//! rich taxonomy covering ring geometry, Hamming geometry, path-dependent
//! quantities, and catastrophe-theoretic measurements.
//!
//! **Space classification:** `bridge` — kernel-computed, user-requested.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `observable/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "observable",
            iri: NS_OBSERVABLE,
            label: "UOR Observables",
            comment: "Observable quantities and metrics computed by the UOR kernel. \
                      Includes ring-metric, Hamming-metric, curvature, holonomy, \
                      and catastrophe-theoretic observables.",
            space: Space::Bridge,
            imports: &[NS_OP, NS_SCHEMA, NS_PARTITION, NS_TYPE],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        // Root
        Class {
            id: "https://uor.foundation/observable/Observable",
            label: "Observable",
            comment: "A measurable quantity in the UOR Framework. All observables \
                      are kernel-computed and user-consumed.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Observable categories
        Class {
            id: "https://uor.foundation/observable/StratumObservable",
            label: "StratumObservable",
            comment: "An observable measuring stratum-level properties: position \
                      within the ring's layer structure.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/MetricObservable",
            label: "MetricObservable",
            comment: "An observable measuring geometric distance between ring elements \
                      under a specific metric.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/PathObservable",
            label: "PathObservable",
            comment: "An observable measuring properties of paths through the ring: \
                      path length, total variation, winding number.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/ReductionObservable",
            label: "ReductionObservable",
            comment: "An observable measuring reduction properties: the length and \
                      count of operation sequences.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/CatastropheObservable",
            label: "CatastropheObservable",
            comment: "An observable measuring catastrophe-theoretic properties: \
                      thresholds at which qualitative changes occur in the partition.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/CurvatureObservable",
            label: "CurvatureObservable",
            comment: "An observable measuring the curvature of the UOR geometry: \
                      the gap between ring-isometry and Hamming-isometry for a \
                      given transform.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/HolonomyObservable",
            label: "HolonomyObservable",
            comment: "An observable measuring holonomy: the accumulated transformation \
                      when traversing a closed path in the ring.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        // Wiki ADR-049 closed-catalog extension: SpectralObservable as a
        // top-level Observable subclass parallel to the seven
        // internally-derived categories. Distinct because its values are
        // structural readings of the σ-projection's frequency-domain
        // spectrum (Walsh–Hadamard parities at specific frequencies),
        // not from algebraic / topological structure. Foundation's
        // `WalshHadamardParity` observable lives under this subclass.
        Class {
            id: "https://uor.foundation/observable/SpectralObservable",
            label: "SpectralObservable",
            comment: "ADR-049: an observable whose value is a structural \
                      reading of a digest's frequency-domain spectrum. \
                      Distinct from the seven internally-derived Observable \
                      categories (Stratum / Metric / Path / Reduction / \
                      Catastrophe / Curvature / Holonomy) — its values are \
                      Walsh–Hadamard parities at specific frequencies, \
                      not derivable from the framework's internal \
                      algebraic/topological structure. Foundation's typed \
                      observable `WalshHadamardParity` per ADR-049 falls \
                      under this subclass; predicates over its values \
                      enter the typed-commitment surface per ADR-048 as \
                      `SingletonCommitment<WalshHadamardParity>` operands.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        // Wiki ADR-040 + ADR-049 closed-catalog extension:
        // ValueThresholdObservable as a top-level Observable subclass
        // parallel to SpectralObservable / AxisProjectionObservable.
        // Distinct because its values carry from a byte-sequence
        // threshold comparison (`digest <= target` under big-endian
        // unsigned ordering), not from the framework's internal
        // algebraic/topological structure. Realizes the
        // `type:LexicographicLessEqBound` bound-shape primitive's
        // dispatch path per ADR-040; foundation's typed observable
        // `LexicographicLessEqThreshold` falls under this subclass.
        // Consumed by `TargetCommitment = SingletonCommitment<…>` as
        // the canonical search-cost commitment per ADR-048.
        Class {
            id: "https://uor.foundation/observable/ValueThresholdObservable",
            label: "ValueThresholdObservable",
            comment: "ADR-040 + ADR-049: an observable whose value is a byte-sequence \
                      threshold comparison reading of a digest. Distinct from the seven \
                      internally-derived Observable categories (Stratum / Metric / Path / \
                      Reduction / Catastrophe / Curvature / Holonomy) and from \
                      SpectralObservable / AxisProjectionObservable — its values carry \
                      from `(digest as big-endian unsigned integer) <= (target as \
                      big-endian unsigned integer)`, the predicate form ADR-040 named \
                      when it committed `type:LexicographicLessEqBound`. Foundation's \
                      typed observable `LexicographicLessEqThreshold` per ADR-049 falls \
                      under this subclass; the canonical search-cost commitment alias \
                      `TargetCommitment = SingletonCommitment<LexicographicLessEqThreshold>` \
                      per ADR-048 consumes it. The ConstraintRef::Bound.args_repr \
                      canonical-string-form encoding for ValueThresholdObservable \
                      arguments carries the target byte sequence as the bound's \
                      argument directly.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        // ADR-038 closed-catalog extension: AxisProjectionObservable as
        // a top-level Observable subclass parallel to the seven
        // internally-derived categories (Stratum / Metric / Path /
        // Reduction / Catastrophe / Curvature / Holonomy). Distinct
        // because its values carry from the substrate-extension surface
        // per ADR-030 (axis-realized projections of typed sites through
        // application-declared AxisTuple kernels), not from the
        // framework's internal algebraic/topological structure. The
        // ConstraintRef::Bound.args_repr canonical-string-form encoding
        // for AxisProjectionObservable arguments is
        //   `axis_address=<hex>;kernel=<symbolic>;sites=<site-list>[;target=<target-spec>]`
        // — axis identification by content-address (AXIS_ADDRESS per
        // ADR-030), not by tuple position, so the encoding is
        // application-invariant across AxisTuple reorderings.
        Class {
            id: "https://uor.foundation/observable/AxisProjectionObservable",
            label: "AxisProjectionObservable",
            comment: "ADR-038: an observable whose value is the axis-realized \
                      projection of typed sites through an application-declared \
                      AxisTuple kernel per ADR-030. Distinct from the seven \
                      internally-derived Observable categories (Stratum, Metric, \
                      Path, Reduction, Catastrophe, Curvature, Holonomy) — its \
                      values carry from the substrate-extension surface (axis \
                      kernels), not from the framework's internal algebraic / \
                      topological structure. The closed-catalog discipline holds: \
                      foundation owns the subclass; applications consume catalog \
                      variants through canonical-string-form `args_repr` on \
                      `ConstraintRef::Bound`. The args_repr encoding (per ADR-038) \
                      is `axis_address=<hex>;kernel=<symbolic>;sites=<site-list>\
                      [;target=<target-spec>]` — axis identification by \
                      content-address (AXIS_ADDRESS per ADR-030), not by tuple \
                      position, so the encoding is application-invariant.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        // Metric subclasses
        Class {
            id: "https://uor.foundation/observable/RingMetric",
            label: "RingMetric",
            comment: "Distance between two ring elements under the ring metric: \
                      d_R(x, y) = |x - y| mod 2^n.",
            subclass_of: &["https://uor.foundation/observable/MetricObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/HammingMetric",
            label: "HammingMetric",
            comment: "Distance between two ring elements under the Hamming metric: \
                      the number of bit positions where they differ.",
            subclass_of: &["https://uor.foundation/observable/MetricObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/IncompatibilityMetric",
            label: "IncompatibilityMetric",
            comment: "The metric incompatibility between two ring elements: the \
                      divergence between their ring-metric and Hamming-metric \
                      distances, measuring geometric curvature.",
            subclass_of: &["https://uor.foundation/observable/MetricObservable"],
            disjoint_with: &[],
        },
        // v0.2.2 Phase D (Q4) — bound observable for residue and affine
        // BoundConstraint kinds.
        Class {
            id: "https://uor.foundation/observable/ValueModObservable",
            label: "ValueModObservable",
            comment: "Observes a Datum's value modulo a configurable modulus. \
                      Used as the bound observable for BoundConstraint \
                      instances representing residue and affine constraint \
                      kinds (residueConstraintKind, affineConstraintKind).",
            subclass_of: &["https://uor.foundation/observable/MetricObservable"],
            disjoint_with: &[],
        },
        // v0.2.2 Phase E — the grounding completion ratio observable.
        Class {
            id: "https://uor.foundation/observable/GroundingSigma",
            label: "GroundingSigma",
            comment: "Observes the grounding completion ratio \u{03C3} \u{2208} \
                      [0, 1] of a context, where \u{03C3} = 1 indicates the \
                      ground state (state:GroundedContext). Backs the \
                      sigma_metric BaseMetric accessor on Grounded<T>.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        // v0.2.2 Phase E — JacobianObservable: observes the per-site Jacobian
        // row of a Datum at a particular WittLevel.
        Class {
            id: "https://uor.foundation/observable/JacobianObservable",
            label: "JacobianObservable",
            comment: "Observes the per-site Jacobian row of a Datum at a \
                      particular WittLevel, computed as the sequence of \
                      partial derivatives of the ring operation with respect \
                      to each site coordinate. Backs the jacobian_metric \
                      BaseMetric accessor on Grounded<T>; the Rust-side \
                      JacobianMetric<L> is parametric over the level marker.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        // Measurement result types
        Class {
            id: "https://uor.foundation/observable/StratumValue",
            label: "StratumValue",
            comment: "The stratum index of a ring element.",
            subclass_of: &["https://uor.foundation/observable/StratumObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/StratumDelta",
            label: "StratumDelta",
            comment: "The difference in stratum between two ring elements.",
            subclass_of: &["https://uor.foundation/observable/StratumObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/StratumTrajectory",
            label: "StratumTrajectory",
            comment: "The sequence of strata traversed by a path through the ring.",
            subclass_of: &["https://uor.foundation/observable/StratumObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/PathLength",
            label: "PathLength",
            comment: "The length of a path through the ring, measured in operation steps.",
            subclass_of: &["https://uor.foundation/observable/PathObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/TotalVariation",
            label: "TotalVariation",
            comment: "The total variation of a path: the sum of metric distances \
                      between consecutive elements.",
            subclass_of: &["https://uor.foundation/observable/PathObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/WindingNumber",
            label: "WindingNumber",
            comment: "The winding number of a closed path: the number of times \
                      the path wraps around the ring.",
            subclass_of: &["https://uor.foundation/observable/PathObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/ReductionLength",
            label: "ReductionLength",
            comment: "The number of operation applications in a reduction sequence.",
            subclass_of: &["https://uor.foundation/observable/ReductionObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/ReductionCount",
            label: "ReductionCount",
            comment: "The number of distinct reduction sequences in a computation.",
            subclass_of: &["https://uor.foundation/observable/ReductionObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/CatastropheThreshold",
            label: "CatastropheThreshold",
            comment: "A critical value at which a qualitative change occurs in \
                      the partition structure.",
            subclass_of: &["https://uor.foundation/observable/CatastropheObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/CatastropheCount",
            label: "CatastropheCount",
            comment: "The number of catastrophe events (qualitative partition changes) \
                      in a computation.",
            subclass_of: &["https://uor.foundation/observable/CatastropheObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/Commutator",
            label: "Commutator",
            comment: "The commutator [f, g](x) = f(g(x)) - g(f(x)) of two operations, \
                      measuring their non-commutativity.",
            subclass_of: &["https://uor.foundation/observable/CurvatureObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/CurvatureFlux",
            label: "CurvatureFlux",
            comment: "The integrated curvature over a region of type space: the \
                      total metric incompatibility accumulated.",
            subclass_of: &["https://uor.foundation/observable/CurvatureObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/Monodromy",
            label: "Monodromy",
            comment: "The monodromy of a closed path: the net transformation \
                      accumulated when traversing a loop in the type space.",
            subclass_of: &["https://uor.foundation/observable/HolonomyObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/ParallelTransport",
            label: "ParallelTransport",
            comment: "The parallel transport of a vector along a path: the canonical \
                      lift of the path to the tangent bundle of the ring.",
            subclass_of: &["https://uor.foundation/observable/HolonomyObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/DihedralElement",
            label: "DihedralElement",
            comment: "An element of the dihedral group D_{2^n} acting on the type \
                      space. Each dihedral element induces an isometry of 𝒯_n.",
            subclass_of: &["https://uor.foundation/observable/HolonomyObservable"],
            disjoint_with: &[],
        },
        // Amendment 23: Typed controlled vocabulary class
        Class {
            id: "https://uor.foundation/observable/MeasurementUnit",
            label: "MeasurementUnit",
            comment: "A unit of measurement for observable quantities. Each \
                      MeasurementUnit individual names a specific unit (bits, \
                      ring steps, dimensionless) replacing the string-valued \
                      observable:unit property.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 18: Analytical Observables
        Class {
            id: "https://uor.foundation/observable/Jacobian",
            label: "Jacobian",
            comment: "Site-by-site curvature decomposition. J_k measures the \
                      discrete derivative of the incompatibility metric at site \
                      position k: J_k = |d_R(x, succ(x)) - d_H(x, succ(x))| \
                      restricted to position k.",
            subclass_of: &["https://uor.foundation/observable/CurvatureObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/TopologicalObservable",
            label: "TopologicalObservable",
            comment: "An observable measuring a topological invariant of the \
                      resolution space. Topological observables are invariant \
                      under continuous deformations of the constraint configuration.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/BettiNumber",
            label: "BettiNumber",
            comment: "The rank of a homology group of the constraint nerve. \
                      β_k = rank(H_k(N(C))) counts the k-dimensional holes \
                      in the constraint configuration.",
            subclass_of: &["https://uor.foundation/observable/TopologicalObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/SpectralGap",
            label: "SpectralGap",
            comment: "The smallest positive eigenvalue of the constraint nerve \
                      Laplacian. Controls the convergence rate of iterative \
                      resolution: larger gap = faster convergence.",
            subclass_of: &["https://uor.foundation/observable/TopologicalObservable"],
            disjoint_with: &[],
        },
        // Gap E: Thermodynamic Observable branch
        Class {
            id: "https://uor.foundation/observable/ThermoObservable",
            label: "ThermoObservable",
            comment: "An observable measuring thermodynamic properties of the resolution process: \
                      residual entropy, Landauer cost, and reduction distribution statistics.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/ResidualEntropy",
            label: "ResidualEntropy",
            comment: "S_residual: the residual Shannon entropy of the site distribution after \
                      partial resolution. Computed as S = (Σ κ_k − χ(N(C))) × ln 2 (IT_7b). \
                      Unit: Nats.",
            subclass_of: &["https://uor.foundation/observable/ThermoObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/LandauerCost",
            label: "LandauerCost",
            comment: "The minimum thermodynamic cost (in units of k_B T ln 2) of erasing one \
                      bit of site uncertainty. The UOR ring operates at β* = ln 2 — the \
                      Landauer temperature.",
            subclass_of: &["https://uor.foundation/observable/ThermoObservable"],
            disjoint_with: &[],
        },
        // v0.2.2 Phase A: LandauerBudget — sealed observable carrier for accumulated
        // Landauer cost. Backs the Rust-side enforcement::LandauerBudget newtype, which
        // is one of the two clocks of UorTime.
        Class {
            id: "https://uor.foundation/observable/LandauerBudget",
            label: "LandauerBudget",
            comment: "A sealed observable carrier for accumulated Landauer cost in nats. \
                      Monotonic within a single pipeline invocation. The UOR ring operates \
                      at the Landauer temperature (β* = ln 2), so this observable is a \
                      direct measure of irreversible bit-erasure performed by the \
                      computation up to the witness it accompanies.",
            subclass_of: &["https://uor.foundation/observable/ThermoObservable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/ReductionEntropy",
            label: "ReductionEntropy",
            comment: "The Shannon entropy of the reduction distribution P(j) = 2^{−j}. At the \
                      Landauer temperature, this equals ln 2 per reduction step — each step erases \
                      exactly one bit of site uncertainty.",
            subclass_of: &["https://uor.foundation/observable/ThermoObservable"],
            disjoint_with: &[],
        },
        // Amendment 28: Type synthesis signature
        Class {
            id: "https://uor.foundation/observable/SynthesisSignature",
            label: "SynthesisSignature",
            comment: "A named topological signature: a pair (realised Euler characteristic, \
                      realised Betti profile). Linked from TypeSynthesisResult. Allows \
                      comparison between the goal signature and the actually achieved signature.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 29: Quantum level spectral sequence
        Class {
            id: "https://uor.foundation/observable/SpectralSequencePage",
            label: "SpectralSequencePage",
            comment:
                "A single page E_r of the quantum level spectral sequence. Carries the \
                      page index r and the differential d_r. The sequence converges when all \
                      differentials vanish — typically by E_3 for simple constraint configurations.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/LiftObstructionClass",
            label: "LiftObstructionClass",
            comment: "The cohomology class in H^2(N(C(T))) representing the LiftObstruction \
                      for a specific WittLift. The class is zero iff the obstruction is \
                      trivial. When non-zero, it indexes the specific site pair at Q_{n+1} \
                      that cannot be closed by the lifted constraint set alone.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 30: Monodromy observable machinery
        Class {
            id: "https://uor.foundation/observable/MonodromyClass",
            label: "MonodromyClass",
            comment: "A classification of a type's holonomy: the subgroup of D_{2^n} generated \
                      by all Monodromy observables computed over closed paths in the type's \
                      constraint nerve. Trivial iff every closed constraint path returns to its \
                      starting site assignment without net dihedral transformation.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/HolonomyGroup",
            label: "HolonomyGroup",
            comment:
                "The holonomy group of a ConstrainedType: the group of all Monodromy \
                      elements achievable by closed paths in the constraint nerve. Always a \
                      subgroup of D_{2^n}. Trivial iff the type has trivial monodromy everywhere; \
                      equals D_{2^n} iff paths involving both neg and bnot involutions are present.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/ClosedConstraintPath",
            label: "ClosedConstraintPath",
            comment: "A sequence of constraint applications forming a closed loop in the \
                      constraint nerve — beginning and ending at the same site assignment. \
                      The Monodromy of the loop is the net DihedralElement accumulated \
                      when traversing it.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 31: PhaseBoundaryType enum class
        Class {
            id: "https://uor.foundation/observable/PhaseBoundaryType",
            label: "PhaseBoundaryType",
            comment: "A classification of phase boundary in the catastrophe \
                      diagram: period boundary (g divides 2^n − 1) or \
                      power-of-two boundary (g = 2^k).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 34: Morphospace Achievability
        Class {
            id: "https://uor.foundation/observable/AchievabilityStatus",
            label: "AchievabilityStatus",
            comment: "The achievability classification of a topological signature \
                      in the morphospace. Either Achievable or Forbidden \
                      (witnessed by ImpossibilityWitness).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 54: Higher homotopy observables
        Class {
            id: "https://uor.foundation/observable/HomotopyGroup",
            label: "HomotopyGroup",
            comment: "The k-th homotopy group \u{03c0}k(N(C), v) of the constraint nerve \
                      based at vertex v.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/HigherMonodromy",
            label: "HigherMonodromy",
            comment: "The image of \u{03c0}k(N(C)) \u{2192} Aut(sitek) for k > 1. \
                      Generalises the MN_6 monodromy homomorphism.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/WhiteheadProduct",
            label: "WhiteheadProduct",
            comment: "The Whitehead product [\u{03b1}, \u{03b2}] \u{2208} \u{03c0}p+q\u{2212}1 \
                      for \u{03b1} \u{2208} \u{03c0}p, \u{03b2} \u{2208} \u{03c0}q.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 57: Stratification Record
        Class {
            id: "https://uor.foundation/observable/StratificationRecord",
            label: "StratificationRecord",
            comment: "A record of the holonomy stratification of the moduli space \
                      at a given quantum level: the list of HolonomyStrata, their \
                      codimensions, and their relationship to the MorphospaceBoundary.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 59: Named Base Metrics
        Class {
            id: "https://uor.foundation/observable/BaseMetric",
            label: "BaseMetric",
            comment: "Superclass for the six universal measurements. Every computation \
                      on the ring produces these six quantities: d_\u{0394}, \u{03c3}, \
                      J_k, \u{03b2}_k, \u{03c7}, r.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/GroundingObservable",
            label: "GroundingObservable",
            comment: "The grounding metric \u{03c3} = pinned sites / total sites. \
                      Ranges from 0 (no sites pinned) to 1 (fully grounded).",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/observable/EulerCharacteristicObservable",
            label: "EulerCharacteristicObservable",
            comment: "The Euler characteristic \u{03c7} = \u{03a3}(\u{2212}1)^k \u{03b2}_k \
                      of the constraint nerve. An integer-valued topological invariant.",
            subclass_of: &["https://uor.foundation/observable/Observable"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/observable/value",
            label: "value",
            comment: "The numeric value of an observable measurement.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/Observable"),
            range: XSD_DECIMAL,
        },
        // observable:unit property removed by Amendment 23 (replaced by hasUnit)
        Property {
            id: "https://uor.foundation/observable/source",
            label: "source",
            comment: "The source object of this measurement (datum, partition, \
                      or path start point).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/Observable"),
            range: OWL_THING,
        },
        Property {
            id: "https://uor.foundation/observable/target",
            label: "target",
            comment: "The target object of this measurement (for metrics and \
                      path-end measurements).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/Observable"),
            range: OWL_THING,
        },
        // Amendment 23: Typed controlled vocabulary property
        Property {
            id: "https://uor.foundation/observable/hasUnit",
            label: "hasUnit",
            comment: "The measurement unit of this observable. Replaces the \
                      string-valued observable:unit property with a typed \
                      reference to a MeasurementUnit individual.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/Observable"),
            range: "https://uor.foundation/observable/MeasurementUnit",
        },
        // Amendment 18: Analytical Observable properties
        Property {
            id: "https://uor.foundation/observable/sitePosition",
            label: "sitePosition",
            comment: "The site position k at which this Jacobian entry is measured.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/Jacobian"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/derivativeValue",
            label: "derivativeValue",
            comment: "The discrete derivative value at this site position.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/Jacobian"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/observable/dimension",
            label: "dimension",
            comment: "The dimension k of the topological observable (e.g., the \
                      degree of the Betti number or the dimension of the spectral gap).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/TopologicalObservable"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 28: SynthesisSignature properties
        Property {
            id: "https://uor.foundation/observable/realisedEuler",
            label: "realisedEuler",
            comment: "The Euler characteristic actually achieved by this synthesis signature.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/SynthesisSignature"),
            range: XSD_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/realisedBetti",
            label: "realisedBetti",
            comment: "Non-functional. Realised Betti number values, one assertion per \
                      homological degree.",
            kind: PropertyKind::Datatype,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/observable/SynthesisSignature"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 29: SpectralSequencePage properties
        Property {
            id: "https://uor.foundation/observable/pageIndex",
            label: "pageIndex",
            comment: "The page r of this spectral sequence page. r=1 is the initial page; \
                      convergence is declared when all d_r are zero.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/SpectralSequencePage"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/differentialIsZero",
            label: "differentialIsZero",
            comment: "True iff d_r = 0 on this page — no further corrections to the \
                      lifted homology.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/SpectralSequencePage"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/observable/convergedAt",
            label: "convergedAt",
            comment: "The page index r at which the spectral sequence converged \
                      (all subsequent differentials zero).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/SpectralSequencePage"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 29: LiftObstructionClass property
        Property {
            id: "https://uor.foundation/observable/obstructionClass",
            label: "obstructionClass",
            comment: "The cohomology class in H^2(N(C(T))) representing this obstruction.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/LiftObstructionClass"),
            range: "https://uor.foundation/cohomology/CohomologyGroup",
        },
        // Amendment 30: Monodromy extensions
        Property {
            id: "https://uor.foundation/observable/monodromyLoop",
            label: "monodromyLoop",
            comment: "The closed path that generates this monodromy value.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/Monodromy"),
            range: "https://uor.foundation/observable/ClosedConstraintPath",
        },
        Property {
            id: "https://uor.foundation/observable/monodromyElement",
            label: "monodromyElement",
            comment: "The dihedral element g in D_{2^n} accumulated when traversing the loop. \
                      The monodromy is trivial iff this element is the group identity.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/Monodromy"),
            range: "https://uor.foundation/observable/DihedralElement",
        },
        Property {
            id: "https://uor.foundation/observable/isTrivialMonodromy",
            label: "isTrivialMonodromy",
            comment: "True iff the monodromyElement is the identity in D_{2^n}.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/Monodromy"),
            range: XSD_BOOLEAN,
        },
        // Amendment 30: HolonomyGroup properties
        Property {
            id: "https://uor.foundation/observable/holonomyGroup",
            label: "holonomyGroup",
            comment: "Non-functional. The generators of the holonomy group: one DihedralElement \
                      per generating monodromy.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/observable/HolonomyGroup"),
            range: "https://uor.foundation/observable/DihedralElement",
        },
        Property {
            id: "https://uor.foundation/observable/holonomyGroupOrder",
            label: "holonomyGroupOrder",
            comment: "The order of the holonomy group as a subgroup of D_{2^n}. \
                      For a FlatType: 1. For full dihedral holonomy: 2^{n+1}.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/HolonomyGroup"),
            range: XSD_POSITIVE_INTEGER,
        },
        // Amendment 30: ClosedConstraintPath properties
        Property {
            id: "https://uor.foundation/observable/pathLength",
            label: "pathLength",
            comment: "The number of constraint application steps in this closed path.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/ClosedConstraintPath"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/pathConstraints",
            label: "pathConstraints",
            comment: "Non-functional. The ordered sequence of constraints traversed. \
                      One assertion per step.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/observable/ClosedConstraintPath"),
            range: "https://uor.foundation/type/Constraint",
        },
        // Amendment 30: DihedralElement extensions
        Property {
            id: "https://uor.foundation/observable/dihedralElementValue",
            label: "dihedralElementValue",
            comment: "Non-functional. One assertion per generator in the normal form of the \
                      element — the sequence of neg and/or bnot operations that realises this \
                      dihedral element when composed.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/observable/DihedralElement"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/observable/isIdentityElement",
            label: "isIdentityElement",
            comment: "True iff this element is the group identity (the trivial monodromy value).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/DihedralElement"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/observable/elementOrder",
            label: "elementOrder",
            comment: "The order of this element in D_{2^n}: the smallest k >= 1 such that \
                      g^k = id. For neg and bnot: order 2.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/DihedralElement"),
            range: XSD_POSITIVE_INTEGER,
        },
        // Amendment 31: Thermodynamic + Phase Diagram properties
        Property {
            id: "https://uor.foundation/observable/hardnessEstimate",
            label: "hardnessEstimate",
            comment: "An estimated computational hardness for a ThermoObservable, \
                      connecting thermodynamic cost to complexity (TH_9 realisation).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/ThermoObservable"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/observable/phaseN",
            label: "phaseN",
            comment: "The ring dimension coordinate n in the (n, g) catastrophe \
                      phase diagram (PD_1 n-coordinate).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/CatastropheObservable"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/phaseG",
            label: "phaseG",
            comment: "The group-order coordinate g in the (n, g) catastrophe \
                      phase diagram (PD_1 g-coordinate).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/CatastropheObservable"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/onResonanceLine",
            label: "onResonanceLine",
            comment: "True when g divides 2^n − 1, placing this observable on \
                      a resonance line in the phase diagram (PD_4).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/CatastropheObservable"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/observable/phaseBoundaryType",
            label: "phaseBoundaryType",
            comment: "The type of phase boundary at this point in the catastrophe \
                      diagram: PeriodBoundary or PowerOfTwoBoundary (PD_2).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/CatastropheObservable"),
            range: "https://uor.foundation/observable/PhaseBoundaryType",
        },
        // Amendment 34: Morphospace Achievability
        Property {
            id: "https://uor.foundation/observable/achievabilityStatus",
            label: "achievabilityStatus",
            comment: "The achievability classification of this observable's \
                      topological signature in the morphospace.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/SynthesisSignature"),
            range: "https://uor.foundation/observable/AchievabilityStatus",
        },
        Property {
            id: "https://uor.foundation/observable/isAchievable",
            label: "isAchievable",
            comment: "Whether this signature has been empirically verified as \
                      achievable at some quantum level.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/SynthesisSignature"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/observable/isForbidden",
            label: "isForbidden",
            comment: "Whether this signature has been formally proven impossible \
                      by an ImpossibilityWitness.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/SynthesisSignature"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/observable/achievabilityWitness",
            label: "achievabilityWitness",
            comment: "The proof individual (ImpossibilityWitness or \
                      AxiomaticDerivation) that grounds this signature's \
                      achievability classification.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/SynthesisSignature"),
            range: "https://uor.foundation/proof/Proof",
        },
        // Amendment 37: DihedralElement internal algebra (Gap 6)
        Property {
            id: "https://uor.foundation/observable/rotationExponent",
            label: "rotationExponent",
            comment: "The rotation exponent k ∈ \\[0, 2^n) of this dihedral element \
                      in the standard representation r^k s^s. Together with \
                      reflectionBit, uniquely identifies the element within \
                      D_\\{2^n\\}.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/DihedralElement"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/reflectionBit",
            label: "reflectionBit",
            comment: "The reflection flag s ∈ \\{0,1\\} of this dihedral element. \
                      False = pure rotation (r^k), true = reflection (r^k·s). \
                      D_7 defines composition: (r^a s^p)(r^b s^q) = \
                      r^(a + (-1)^p b) s^(p XOR q).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/DihedralElement"),
            range: XSD_BOOLEAN,
        },
        // Amendment 54: Higher homotopy observable properties
        Property {
            id: "https://uor.foundation/observable/homotopyDimension",
            label: "homotopyDimension",
            comment: "The dimension k of this homotopy group \u{03c0}k.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/HomotopyGroup"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/homotopyRank",
            label: "homotopyRank",
            comment: "The rank of this homotopy group (number of free generators).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/HomotopyGroup"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/homotopyBasepoint",
            label: "homotopyBasepoint",
            comment: "The basepoint vertex v at which this homotopy group is computed.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/HomotopyGroup"),
            range: "https://uor.foundation/type/Constraint",
        },
        Property {
            id: "https://uor.foundation/observable/higherMonodromyDimension",
            label: "higherMonodromyDimension",
            comment: "The dimension k > 1 at which this higher monodromy acts.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/HigherMonodromy"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/whiteheadTrivial",
            label: "whiteheadTrivial",
            comment: "True iff this Whitehead product is trivial (zero in \u{03c0}p+q\u{2212}1).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/WhiteheadProduct"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/observable/postnikovTruncation",
            label: "postnikovTruncation",
            comment: "The Postnikov truncation associated with this spectral sequence page.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/SpectralSequencePage"),
            range: "https://uor.foundation/homology/PostnikovTruncation",
        },
        // Amendment 57: StratificationRecord properties
        Property {
            id: "https://uor.foundation/observable/stratificationLevel",
            label: "stratificationLevel",
            comment: "The quantum level at which this stratification is computed.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/StratificationRecord"),
            range: "https://uor.foundation/schema/WittLevel",
        },
        Property {
            id: "https://uor.foundation/observable/stratificationStratum",
            label: "stratificationStratum",
            comment: "A HolonomyStratum in this stratification record.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/observable/StratificationRecord"),
            range: "https://uor.foundation/type/HolonomyStratum",
        },
        // Amendment 59: Named Base Metrics properties
        Property {
            id: "https://uor.foundation/observable/metricDomain",
            label: "metricDomain",
            comment: "The mathematical domain of this base metric.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/observable/metricRange",
            label: "metricRange",
            comment: "The mathematical range (codomain) of this base metric.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/observable/metricComposition",
            label: "metricComposition",
            comment: "How this metric composes with others in the measurement tower.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/observable/referencesClass",
            label: "referencesClass",
            comment: "IRI of the existing observable class that this base \
                      metric references. Annotation-valued (not ObjectProperty) \
                      because the assertion is a class-level reference, not an \
                      instance-level one: the metric describes a class of \
                      Observable phenomena, not a specific individual.",
            kind: PropertyKind::Annotation,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: XSD_STRING,
        },
        Property {
            id: "https://uor.foundation/observable/referencesIdentity",
            label: "referencesIdentity",
            comment: "The existing identity that defines this base metric.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: "https://uor.foundation/op/Identity",
        },
        Property {
            id: "https://uor.foundation/observable/saturationNumerator",
            label: "saturationNumerator",
            comment: "The count of pinned sites (numerator of \u{03c3}).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/GroundingObservable"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/saturationDenominator",
            label: "saturationDenominator",
            comment: "The total site count (denominator of \u{03c3}).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/GroundingObservable"),
            range: XSD_POSITIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/alternatingSum",
            label: "alternatingSum",
            comment: "The alternating sum formula for Euler characteristic.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/EulerCharacteristicObservable"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // Amendment 71: Missing BaseMetric properties (7)
        Property {
            id: "https://uor.foundation/observable/metricUnit",
            label: "metricUnit",
            comment: "The unit of measurement for this base metric.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: "https://uor.foundation/observable/MeasurementUnit",
        },
        Property {
            id: "https://uor.foundation/observable/metricPrecision",
            label: "metricPrecision",
            comment: "The precision or resolution of this base metric.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/metricMonotonicity",
            label: "metricMonotonicity",
            comment: "Monotonicity property of this metric (e.g., non-decreasing).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/observable/metricDecomposition",
            label: "metricDecomposition",
            comment: "The decomposition rule for this metric into sub-metrics.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/observable/metricTowerPosition",
            label: "metricTowerPosition",
            comment: "The position of this metric in the metric tower.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/observable/metricComputationCost",
            label: "metricComputationCost",
            comment: "The computational cost of evaluating this metric.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        Property {
            id: "https://uor.foundation/observable/metricBound",
            label: "metricBound",
            comment: "Upper or lower bound on the metric value.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/BaseMetric"),
            range: "https://uor.foundation/schema/TermExpression",
        },
        // v0.2.2 Phase A: landauerNats — the single functional payload property of
        // the LandauerBudget carrier. Records accumulated Landauer cost in observable:Nats.
        Property {
            id: "https://uor.foundation/observable/landauerNats",
            label: "landauerNats",
            comment: "The accumulated Landauer cost carried by a LandauerBudget instance, \
                      measured in nats. Monotonic within a pipeline invocation. The unit is \
                      observable:Nats — every increment corresponds to a number of \
                      irreversible bit-erasures times ln 2 (op:OA_5).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/observable/LandauerBudget"),
            range: XSD_DECIMAL,
        },
    ]
}

// Amendment 23: Typed controlled vocabulary individuals
fn individuals() -> Vec<Individual> {
    vec![
        Individual {
            id: "https://uor.foundation/observable/Bits",
            type_: "https://uor.foundation/observable/MeasurementUnit",
            label: "Bits",
            comment: "Information-theoretic unit: the measurement is in bits \
                      (e.g., Hamming weight, entropy).",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/observable/RingSteps",
            type_: "https://uor.foundation/observable/MeasurementUnit",
            label: "RingSteps",
            comment: "Ring-arithmetic unit: the measurement is in ring distance \
                      steps (|x - y| mod 2^n).",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/observable/Dimensionless",
            type_: "https://uor.foundation/observable/MeasurementUnit",
            label: "Dimensionless",
            comment: "Dimensionless unit: the measurement is a pure number \
                      (e.g., winding number, Betti number, spectral gap).",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/observable/Nats",
            type_: "https://uor.foundation/observable/MeasurementUnit",
            label: "Nats",
            comment: "Natural information unit: entropy measured in nats (using natural \
                      logarithm). S_residual is expressed in nats when computed as \
                      (Σ κ_k − χ) × ln 2.",
            properties: &[],
        },
        // Amendment 31: PhaseBoundaryType vocabulary individuals
        Individual {
            id: "https://uor.foundation/observable/PeriodBoundary",
            type_: "https://uor.foundation/observable/PhaseBoundaryType",
            label: "PeriodBoundary",
            comment: "A phase boundary where g divides 2^n − 1, meaning g is \
                      a period of the multiplicative structure of R_n.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/observable/PowerOfTwoBoundary",
            type_: "https://uor.foundation/observable/PhaseBoundaryType",
            label: "PowerOfTwoBoundary",
            comment: "A phase boundary where g = 2^k, meaning g aligns with \
                      the binary stratification of R_n.",
            properties: &[],
        },
        // Amendment 34: AchievabilityStatus vocabulary individuals
        Individual {
            id: "https://uor.foundation/observable/Achievable",
            type_: "https://uor.foundation/observable/AchievabilityStatus",
            label: "Achievable",
            comment: "The signature has been verified as achievable \
                      at some quantum level by an AxiomaticDerivation proof.",
            properties: &[],
        },
        Individual {
            id: "https://uor.foundation/observable/Forbidden",
            type_: "https://uor.foundation/observable/AchievabilityStatus",
            label: "Forbidden",
            comment: "The signature has been formally proven impossible by an \
                      ImpossibilityWitness deriving from MS_1, MS_2, or other \
                      impossibility theorems.",
            properties: &[],
        },
        // Amendment 59: Named Base Metric individuals
        Individual {
            id: "https://uor.foundation/observable/d_delta_metric",
            type_: "https://uor.foundation/observable/BaseMetric",
            label: "d_delta_metric",
            comment: "d_\u{0394}: the incompatibility metric |d_R \u{2212} d_H| per site pair.",
            properties: &[
                (
                    "https://uor.foundation/observable/metricDomain",
                    IndividualValue::Str("pair of ring elements"),
                ),
                (
                    "https://uor.foundation/observable/metricRange",
                    IndividualValue::Str("non-negative integer"),
                ),
                (
                    "https://uor.foundation/observable/referencesClass",
                    IndividualValue::IriRef(
                        "https://uor.foundation/observable/IncompatibilityMetric",
                    ),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/observable/sigma_metric",
            type_: "https://uor.foundation/observable/BaseMetric",
            label: "sigma_metric",
            comment: "\u{03c3}: the grounding metric, pinned sites / total sites.",
            properties: &[
                (
                    "https://uor.foundation/observable/metricDomain",
                    IndividualValue::Str("computation state"),
                ),
                (
                    "https://uor.foundation/observable/metricRange",
                    IndividualValue::Str("decimal in \u{005b}0, 1\u{005d}"),
                ),
                (
                    "https://uor.foundation/observable/referencesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/GS_2"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/observable/jacobian_metric",
            type_: "https://uor.foundation/observable/BaseMetric",
            label: "jacobian_metric",
            comment: "J_k: per-site curvature, \u{2202}_R f_k.",
            properties: &[
                (
                    "https://uor.foundation/observable/metricDomain",
                    IndividualValue::Str("computation state \u{00d7} site index"),
                ),
                (
                    "https://uor.foundation/observable/metricRange",
                    IndividualValue::Str("decimal"),
                ),
                (
                    "https://uor.foundation/observable/referencesClass",
                    IndividualValue::IriRef("https://uor.foundation/observable/Jacobian"),
                ),
                (
                    "https://uor.foundation/observable/referencesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/DC_6"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/observable/betti_metric",
            type_: "https://uor.foundation/observable/BaseMetric",
            label: "betti_metric",
            comment: "\u{03b2}_k: per-dimension Betti number of the constraint nerve.",
            properties: &[
                (
                    "https://uor.foundation/observable/metricDomain",
                    IndividualValue::Str("simplicial complex \u{00d7} dimension"),
                ),
                (
                    "https://uor.foundation/observable/metricRange",
                    IndividualValue::Str("non-negative integer"),
                ),
                (
                    "https://uor.foundation/observable/referencesClass",
                    IndividualValue::IriRef("https://uor.foundation/observable/BettiNumber"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/observable/euler_metric",
            type_: "https://uor.foundation/observable/BaseMetric",
            label: "euler_metric",
            comment: "\u{03c7}: Euler characteristic, \u{03a3}(\u{2212}1)^k \u{03b2}_k.",
            properties: &[
                (
                    "https://uor.foundation/observable/metricDomain",
                    IndividualValue::Str("simplicial complex"),
                ),
                (
                    "https://uor.foundation/observable/metricRange",
                    IndividualValue::Str("integer"),
                ),
                (
                    "https://uor.foundation/observable/referencesIdentity",
                    IndividualValue::IriRef("https://uor.foundation/op/IT_2"),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/observable/residual_metric",
            type_: "https://uor.foundation/observable/BaseMetric",
            label: "residual_metric",
            comment: "r: count of free (unpinned) sites, the residual entropy.",
            properties: &[
                (
                    "https://uor.foundation/observable/metricDomain",
                    IndividualValue::Str("computation state"),
                ),
                (
                    "https://uor.foundation/observable/metricRange",
                    IndividualValue::Str("non-negative integer"),
                ),
                (
                    "https://uor.foundation/observable/referencesClass",
                    IndividualValue::IriRef("https://uor.foundation/observable/ResidualEntropy"),
                ),
            ],
        },
    ]
}
