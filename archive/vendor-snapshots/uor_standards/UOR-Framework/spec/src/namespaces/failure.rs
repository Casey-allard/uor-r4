//! `failure/` namespace — Failure algebra.
//!
//! The `failure/` namespace formalizes partial computations, typed failure
//! propagation, and recovery. Computations that cannot reach convergence are
//! represented as typed failures that compose under monoidal and parallel
//! products.
//!
//! - **Amendment 76**: 12 classes, 11 properties, 0 individuals (identities in op/)
//!
//! **Space classification:** `kernel` — immutable algebra.

use crate::model::iris::*;
use crate::model::{Class, Namespace, NamespaceModule, Property, PropertyKind, Space};

/// Returns the `failure/` namespace module.
#[must_use]
pub fn module() -> NamespaceModule {
    NamespaceModule {
        namespace: Namespace {
            prefix: "failure",
            iri: NS_FAILURE,
            label: "UOR Failure Algebra",
            comment: "Partial computations, typed failure propagation, and \
                      recovery. Formalizes how computations that cannot reach \
                      convergence are represented, composed, and recovered \
                      from within the ring substrate.",
            space: Space::Kernel,
            imports: &[
                NS_OP,
                NS_SCHEMA,
                NS_EFFECT,
                NS_STATE,
                NS_REDUCTION,
                NS_CERT,
                NS_TRACE,
                NS_PROOF,
            ],
        },
        classes: classes(),
        properties: properties(),
        individuals: vec![],
    }
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            id: "https://uor.foundation/failure/ComputationResult",
            label: "ComputationResult",
            comment: "The outcome of a computation that may fail: either a \
                      Success carrying a datum, or a Failure carrying a typed \
                      reason. The coproduct of the success and failure cases.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/failure/Success",
            label: "Success",
            comment: "A computation that reached convergence and produced a \
                      valid datum. Carries the output datum and the \
                      computation certificate.",
            subclass_of: &["https://uor.foundation/failure/ComputationResult"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/failure/Failure",
            label: "Failure",
            comment: "A computation that could not reach convergence. \
                      Carries a typed FailureReason and the reduction state at \
                      the point of failure.",
            subclass_of: &["https://uor.foundation/failure/ComputationResult"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/failure/FailureReason",
            label: "FailureReason",
            comment: "A typed classification of why a computation failed.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/failure/GuardFailure",
            label: "GuardFailure",
            comment: "A reduction step guard evaluated to false and no \
                      alternative transition exists.",
            subclass_of: &["https://uor.foundation/failure/FailureReason"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/failure/ConstraintContradiction",
            label: "ConstraintContradiction",
            comment: "Two constraints in the type\u{2019}s constraint set are \
                      jointly unsatisfiable.",
            subclass_of: &["https://uor.foundation/failure/FailureReason"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/failure/SiteExhaustion",
            label: "SiteExhaustion",
            comment: "The linear budget was exhausted before resolution \
                      completed.",
            subclass_of: &["https://uor.foundation/failure/FailureReason"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/failure/LiftObstructionFailure",
            label: "LiftObstructionFailure",
            comment: "A WittLift encountered a non-trivial obstruction \
                      that could not be resolved.",
            subclass_of: &["https://uor.foundation/failure/FailureReason"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/failure/PartialComputation",
            label: "PartialComputation",
            comment: "A computation that produces a ComputationResult rather \
                      than a guaranteed datum. The general case of all \
                      computations.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/failure/TotalComputation",
            label: "TotalComputation",
            comment: "A computation where the FailureReason type is empty — \
                      failure is structurally impossible.",
            subclass_of: &["https://uor.foundation/failure/PartialComputation"],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/failure/Recovery",
            label: "Recovery",
            comment: "A strategy for converting a Failure into a Success by \
                      modifying the computation path.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
        Class {
            id: "https://uor.foundation/failure/FailurePropagation",
            label: "FailurePropagation",
            comment: "The rule for how failures compose under monoidal and \
                      parallel products.",
            subclass_of: &[OWL_THING],
            disjoint_with: &[],
        },
    ]
}

fn properties() -> Vec<Property> {
    vec![
        // Object properties
        Property {
            id: "https://uor.foundation/failure/resultDatum",
            label: "resultDatum",
            comment: "The output datum of a successful computation.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/failure/Success"),
            range: "https://uor.foundation/schema/Datum",
        },
        Property {
            id: "https://uor.foundation/failure/resultCertificate",
            label: "resultCertificate",
            comment: "The certificate attesting the computation\u{2019}s \
                      correctness.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/failure/Success"),
            range: "https://uor.foundation/proof/ComputationCertificate",
        },
        Property {
            id: "https://uor.foundation/failure/failureReason",
            label: "failureReason",
            comment: "The typed reason for failure.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/failure/Failure"),
            range: "https://uor.foundation/failure/FailureReason",
        },
        Property {
            id: "https://uor.foundation/failure/failureState",
            label: "failureState",
            comment: "The reduction state at the point of failure.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/failure/Failure"),
            range: "https://uor.foundation/reduction/ReductionState",
        },
        Property {
            id: "https://uor.foundation/failure/failureStage",
            label: "failureStage",
            comment: "The reduction step where failure occurred.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/failure/Failure"),
            range: "https://uor.foundation/reduction/ReductionStep",
        },
        Property {
            id: "https://uor.foundation/failure/recoveryStrategy",
            label: "recoveryStrategy",
            comment: "Available recovery strategies for this failure.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/failure/Failure"),
            range: "https://uor.foundation/failure/Recovery",
        },
        Property {
            id: "https://uor.foundation/failure/recoveryEffect",
            label: "recoveryEffect",
            comment: "The effect applied to recover from failure.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/failure/Recovery"),
            range: "https://uor.foundation/effect/Effect",
        },
        Property {
            id: "https://uor.foundation/failure/recoveryTarget",
            label: "recoveryTarget",
            comment: "The reduction step to retry after recovery.",
            kind: PropertyKind::Object,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/failure/Recovery"),
            range: "https://uor.foundation/reduction/ReductionStep",
        },
        Property {
            id: "https://uor.foundation/failure/propagationRule",
            label: "propagationRule",
            comment: "Which failure reasons propagate through this \
                      composition.",
            kind: PropertyKind::Object,
            functional: false,
            required: false,
            domain: Some("https://uor.foundation/failure/FailurePropagation"),
            range: "https://uor.foundation/failure/FailureReason",
        },
        // Datatype properties
        Property {
            id: "https://uor.foundation/failure/isTotal",
            label: "isTotal",
            comment: "True iff the computation is a TotalComputation (no \
                      possible failure path).",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/failure/PartialComputation"),
            range: XSD_BOOLEAN,
        },
        Property {
            id: "https://uor.foundation/failure/failureDepth",
            label: "failureDepth",
            comment: "The reduction step index at which failure occurred.",
            kind: PropertyKind::Datatype,
            functional: true,
            required: false,
            domain: Some("https://uor.foundation/failure/Failure"),
            range: XSD_NON_NEGATIVE_INTEGER,
        },
    ]
}
