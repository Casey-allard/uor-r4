//! `trace/` namespace — Computation execution traces.
//!
//! Traces record the actual execution path of a kernel computation: which
//! operations were applied, in what order, and what the intermediate results
//! were. They are the runtime log of kernel activity.
//!
//! **Space classification:** `bridge` — kernel-produced, user-consumed.

use crate::model::iris::*;
use crate::model::{
    Class, Individual, IndividualValue, Namespace, NamespaceModule, Property, PropertyKind, Space,
};

/// Returns the `trace/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "trace",
            iri: NS_TRACE,
            label: "UOR Computation Traces",
            comment: "Execution traces recording the sequence of kernel operations, \
                      intermediate results, and accumulated metrics for a computation.",
            space: Space::Bridge,
            imports: &[NS_SCHEMA, NS_OP, NS_CERT, NS_OBSERVABLE],
        },
        classes: classes(),
        properties: properties(),
        individuals: individuals(),
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/trace/ComputationTrace",
            label: "ComputationTrace",
            comment: "A complete record of a kernel computation: the input, output, \
                      every operation step, and accumulated metrics.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/trace/ComputationStep",
            label: "ComputationStep",
            comment: "A single step in a computation trace: one operation applied \
                      to produce one output from one or more inputs.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/trace/TraceMetrics",
            label: "TraceMetrics",
            comment: "Summary metrics for a computation trace: total steps, \
                      accumulated ring distance, and accumulated Hamming distance.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 35: Computational Geodesic
        Class {
            id: "https://uor.foundation/trace/GeodesicTrace",
            label: "GeodesicTrace",
            comment: "A computation trace that satisfies the dual geodesic condition \
                      (GD_1): AR_1-ordered and DC_10-selected. The path of least \
                      dissipation through the resolution landscape.",
            subclass_of: &["https://uor.foundation/trace/ComputationTrace"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/trace/GeodesicViolation",
            label: "GeodesicViolation",
            comment: "A record of a geodesic condition violation at a specific step \
                      of a computation trace. Produced by GeodesicValidator when \
                      J_k(step_i) < max_\\{free\\} J_k(state_i).",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // Amendment 36: Measurement Boundary
        Class {
            id: "https://uor.foundation/trace/MeasurementEvent",
            label: "MeasurementEvent",
            comment: "A specialized computation step recording a single projective \
                      collapse of a SuperposedSiteState. Carries pre-collapse \
                      entropy and post-collapse Landauer cost (QM_1).",
            subclass_of: &["https://uor.foundation/trace/ComputationStep"],
            disjoint_with: &[],
        },
        // Amendment 37: Measurement Outcome (Gap 10)
        Class {
            id: "https://uor.foundation/trace/MeasurementOutcome",
            label: "MeasurementOutcome",
            comment: "A single outcome of a projective measurement on a \
                      SuperposedSiteState, recording the classical site \
                      index (outcomeValue) and its Born-rule probability \
                      |α_k|² (outcomeProbability). Multiple outcomes form \
                      the probability distribution of a measurement.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        // v0.2.1: Inhabitance Verdict Instantiation
        Class {
            id: "https://uor.foundation/trace/InhabitanceSearchTrace",
            label: "InhabitanceSearchTrace",
            comment: "A subclass of trace:ComputationTrace specialised to \
                      inhabitance-search execution. Records the sequence of \
                      derivation:InhabitanceStep entries the resolver \
                      traversed and any derivation:InhabitanceCheckpoint \
                      entries it crossed.",
            subclass_of: &["https://uor.foundation/trace/ComputationTrace"],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        Property {
            id: "https://uor.foundation/trace/input",
            label: "input",
            comment: "The input datum of this computation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/trace/output",
            label: "output",
            comment: "The output datum of this computation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/trace/step",
            label: "step",
            comment: "A computation step in this trace.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: "https://uor.foundation/trace/ComputationStep",
        },
        Property {
            id: "https://uor.foundation/trace/monodromy",
            label: "monodromy",
            comment: "The monodromy accumulated by this computation: the net \
                      dihedral group element produced by the full operation sequence.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: "https://uor.foundation/observable/DihedralElement",
        },
        Property {
            id: "https://uor.foundation/trace/from",
            label: "from",
            comment: "The input datum of this computation step.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationStep"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/trace/to",
            label: "to",
            comment: "The output datum of this computation step.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationStep"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/trace/operation",
            label: "operation",
            comment: "The operation applied in this computation step.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationStep"),
            range: "https://uor.foundation/op/Operation",
        },
        Property {
            id: "https://uor.foundation/trace/index",
            label: "index",
            comment: "The zero-based sequential index of this step within its trace.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationStep"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/trace/stepCount",
            label: "stepCount",
            comment: "Total number of computation steps in this trace.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/TraceMetrics"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/trace/totalRingDistance",
            label: "totalRingDistance",
            comment: "Total ring-metric distance accumulated across all steps.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/TraceMetrics"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/trace/totalHammingDistance",
            label: "totalHammingDistance",
            comment: "Total Hamming-metric distance accumulated across all steps.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/TraceMetrics"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/trace/certifiedBy",
            label: "certifiedBy",
            comment: "The certificate that attests to the correctness of this \
                      computation trace.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: "https://uor.foundation/cert/Certificate",
        },
        // Amendment 31: Residual entropy link (TH_9 connection)
        Property {
            id: "https://uor.foundation/trace/residualEntropy",
            label: "residualEntropy",
            comment: "The residual entropy observable remaining after this computation \
                      trace, linking to the ThermoObservable taxonomy (TH_9 connection).",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: "https://uor.foundation/observable/ResidualEntropy",
        },
        // Amendment 35: Computational Geodesic properties
        Property {
            id: "https://uor.foundation/trace/isGeodesic",
            label: "isGeodesic",
            comment: "Whether this computation trace satisfies the dual geodesic \
                      condition (GD_1): AR_1-ordered and DC_10-selected.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: XSD_BOOLEAN,
        },
        // Amendment 96 (2026-04-12): `trace:geodesicCertificate`
        // removed. `cert:certifiedGeodesic` in cert.rs is the
        // authoritative direction of the trace/certificate
        // attestation; storing both sides created a mutual dependency
        // cycle with no finite inhabitant in Lean. Reverse lookup
        // ("given a trace, find its certificate") is available via
        // SPARQL query against `cert:certifiedGeodesic`.
        Property {
            id: "https://uor.foundation/trace/geodesicViolation",
            label: "geodesicViolation",
            comment: "A GeodesicViolation record indicating where the trace \
                      deviated from the geodesic condition.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: "https://uor.foundation/trace/GeodesicViolation",
        },
        Property {
            id: "https://uor.foundation/trace/stepEntropyCost",
            label: "stepEntropyCost",
            comment: "The entropy cost of a single computation step. On a geodesic, \
                      this equals ln 2 for every step (GD_2).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationStep"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/trace/cumulativeEntropyCost",
            label: "cumulativeEntropyCost",
            comment: "The total entropy cost accumulated across all steps of a trace. \
                      On a geodesic, equals freeRank_initial × ln 2 (GD_3).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/trace/jacobianAtStep",
            label: "jacobianAtStep",
            comment: "The Jacobian value J_k at this step, used by the \
                      GeodesicValidator to check DC_10 maximality.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationStep"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/trace/adiabaticallyOrdered",
            label: "adiabaticallyOrdered",
            comment: "Whether the step sequence of this trace follows the AR_1 \
                      adiabatic ordering (decreasing freeRank × cost-per-site).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/trace/violationReason",
            label: "violationReason",
            comment: "Human-readable description of why a geodesic violation \
                      occurred, citing the step index and the unused higher-J_k option.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/GeodesicViolation"),
            range: XSD_STRING,
        },
        // Amendment 36: Measurement Boundary properties
        Property {
            id: "https://uor.foundation/trace/measurementEvent",
            label: "measurementEvent",
            comment: "A MeasurementEvent step within this computation trace.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: "https://uor.foundation/trace/MeasurementEvent",
        },
        Property {
            id: "https://uor.foundation/trace/preCollapseEntropy",
            label: "preCollapseEntropy",
            comment: "The von Neumann entropy S_vN of the SuperposedSiteState \
                      before projective collapse.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/MeasurementEvent"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/trace/postCollapseLandauerCost",
            label: "postCollapseLandauerCost",
            comment: "The Landauer cost incurred by the projective collapse. \
                      Equals preCollapseEntropy at β* = ln 2 (QM_1).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/MeasurementEvent"),
            range: XSD_DECIMAL,
        },
        Property {
            id: "https://uor.foundation/trace/collapseStep",
            label: "collapseStep",
            comment: "The step index within the enclosing ComputationTrace at \
                      which this projective collapse occurred.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/MeasurementEvent"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        // Amendment 37: Amplitude vector on MeasurementEvent (Gap 1)
        Property {
            id: "https://uor.foundation/trace/amplitudeVector",
            label: "amplitudeVector",
            comment: "The full pre-collapse amplitude vector of all branches \
                      at the time of measurement. Enables Born rule verification \
                      (QM_5): P(outcome k) = |α_k|² / Σ|αᵢ|².",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/MeasurementEvent"),
            range: XSD_DECIMAL,
        },
        // Amendment 37: Geodesic predicate decomposition (Gap 9)
        Property {
            id: "https://uor.foundation/trace/isAR1Ordered",
            label: "isAR1Ordered",
            comment: "Whether this computation trace has steps ordered by the \
                      AR_1 adiabatic metric (decreasing freeRank × cost-per-site). \
                      One of the two sub-predicates of isGeodesic (GD_6).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/trace/isDC10Selected",
            label: "isDC10Selected",
            comment: "Whether each step of this computation trace was selected by \
                      the DC_10 Jacobian criterion (maximal J_k among free sites). \
                      One of the two sub-predicates of isGeodesic (GD_6).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/ComputationTrace"),
            range: XSD_BOOLEAN,
        },
        // Amendment 37: MeasurementOutcome properties (Gap 10)
        Property {
            id: "https://uor.foundation/trace/outcomeValue",
            label: "outcomeValue",
            comment: "The classical site index selected by projective collapse \
                      in this measurement outcome.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/MeasurementOutcome"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
        Property {
            id: "https://uor.foundation/trace/outcomeProbability",
            label: "outcomeProbability",
            comment: "The Born-rule probability of this measurement outcome: \
                      |α_k|² where α_k is the amplitude of the collapsed site.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/trace/MeasurementOutcome"),
            range: XSD_DECIMAL,
        },
        // v0.2.1: InhabitanceSearchTrace property
        Property {
            id: "https://uor.foundation/trace/checkpoint",
            label: "checkpoint",
            comment: "Checkpoints crossed by the inhabitance search. Each \
                      checkpoint marks an audit point where the resolver state \
                      can be restored if the search backtracks.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/trace/InhabitanceSearchTrace"),
            range: "https://uor.foundation/derivation/InhabitanceCheckpoint",
        },
    ]
}

fn individuals() -> Vec<Individual> {
    vec![
        // Amendment 35: Formal geodesic trace instances at each quantum level
        Individual {
            id: "https://uor.foundation/trace/geodesic_Q0",
            type_: "https://uor.foundation/trace/GeodesicTrace",
            label: "geodesic_Q0",
            comment: "Canonical geodesic trace at quantum level Q0 (n=8). \
                      Demonstrates GD_1 through GD_3 at the base level.",
            properties: &[
                (
                    "https://uor.foundation/trace/isGeodesic",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/trace/adiabaticallyOrdered",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/trace/geodesic_Q1",
            type_: "https://uor.foundation/trace/GeodesicTrace",
            label: "geodesic_Q1",
            comment: "Canonical geodesic trace at quantum level Q1 (n=16). \
                      Demonstrates geodesic scaling from Q0 to Q1.",
            properties: &[
                (
                    "https://uor.foundation/trace/isGeodesic",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/trace/adiabaticallyOrdered",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/trace/geodesic_Q2",
            type_: "https://uor.foundation/trace/GeodesicTrace",
            label: "geodesic_Q2",
            comment: "Canonical geodesic trace at quantum level Q2 (n=32). \
                      Demonstrates geodesic scaling from Q1 to Q2.",
            properties: &[
                (
                    "https://uor.foundation/trace/isGeodesic",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/trace/adiabaticallyOrdered",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/trace/geodesic_Q3",
            type_: "https://uor.foundation/trace/GeodesicTrace",
            label: "geodesic_Q3",
            comment: "Canonical geodesic trace at quantum level Q3 (n=64). \
                      Demonstrates geodesic scaling from Q2 to Q3.",
            properties: &[
                (
                    "https://uor.foundation/trace/isGeodesic",
                    IndividualValue::Bool(true),
                ),
                (
                    "https://uor.foundation/trace/adiabaticallyOrdered",
                    IndividualValue::Bool(true),
                ),
            ],
        },
        // Amendment 36: Formal measurement event instances
        Individual {
            id: "https://uor.foundation/trace/collapse_equal_superposition",
            type_: "https://uor.foundation/trace/MeasurementEvent",
            label: "collapse_equal_superposition",
            comment: "Canonical measurement event: collapse of an equal superposition \
                      (|α|² = 0.5). Maximum von Neumann entropy S_vN = ln 2. \
                      Maximum Landauer cost per QM_1.",
            properties: &[
                (
                    "https://uor.foundation/trace/preCollapseEntropy",
                    IndividualValue::Float(std::f64::consts::LN_2),
                ),
                (
                    "https://uor.foundation/trace/postCollapseLandauerCost",
                    IndividualValue::Float(std::f64::consts::LN_2),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/trace/collapse_biased",
            type_: "https://uor.foundation/trace/MeasurementEvent",
            label: "collapse_biased",
            comment: "Canonical measurement event: collapse of a biased superposition \
                      (|α|² = 0.9). Lower entropy than equal superposition. \
                      Demonstrates QM_3 bound.",
            properties: &[
                (
                    "https://uor.foundation/trace/preCollapseEntropy",
                    IndividualValue::Float(0.325083),
                ),
                (
                    "https://uor.foundation/trace/postCollapseLandauerCost",
                    IndividualValue::Float(0.325083),
                ),
            ],
        },
        Individual {
            id: "https://uor.foundation/trace/collapse_classical",
            type_: "https://uor.foundation/trace/MeasurementEvent",
            label: "collapse_classical",
            comment: "Canonical measurement event: collapse of a classical state \
                      (|α|² = 1). Zero entropy, zero Landauer cost. Demonstrates \
                      QM_4 idempotence.",
            properties: &[
                (
                    "https://uor.foundation/trace/preCollapseEntropy",
                    IndividualValue::Float(0.0),
                ),
                (
                    "https://uor.foundation/trace/postCollapseLandauerCost",
                    IndividualValue::Float(0.0),
                ),
            ],
        },
    ]
}
