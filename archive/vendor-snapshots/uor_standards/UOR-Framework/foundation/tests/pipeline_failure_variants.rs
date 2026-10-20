//! v0.2.1 integration test: PipelineFailure enum variants.
//!
//! Verifies the parametric variant emission from `reduction:FailureField`
//! individuals. The 5 PipelineFailureReason individuals plus
//! LiftObstructionFailure plus ShapeViolation form the 7-variant enum
//! the ergonomics spec §3.5 mandates.

use uor_foundation::enforcement::{PipelineFailure, ShapeViolation};

#[test]
fn dispatch_miss_variant_constructible() {
    let f = PipelineFailure::DispatchMiss {
        query_iri: "https://example.org/q",
        table_iri: "https://example.org/t",
    };
    let _ = format!("{f:?}");
}

#[test]
fn grounding_failure_variant_constructible() {
    let f = PipelineFailure::GroundingFailure {
        reason_iri: "https://example.org/reason",
    };
    let _ = format!("{f:?}");
}

#[test]
fn convergence_stall_variant_constructible() {
    let f = PipelineFailure::ConvergenceStall {
        stage_iri: "https://example.org/stage",
        angle_milliradians: 314,
    };
    let _ = format!("{f:?}");
}

#[test]
fn contradiction_detected_variant_constructible() {
    let f = PipelineFailure::ContradictionDetected {
        at_step: 42,
        trace_iri: "https://example.org/trace",
    };
    let _ = format!("{f:?}");
}

#[test]
fn coherence_violation_variant_constructible() {
    let f = PipelineFailure::CoherenceViolation {
        site_position: 7,
        constraint_iri: "https://example.org/c",
    };
    let _ = format!("{f:?}");
}

#[test]
fn lift_obstruction_failure_variant_constructible() {
    let f = PipelineFailure::LiftObstructionFailure {
        site_position: 3,
        obstruction_class_iri: "https://example.org/o",
    };
    let _ = format!("{f:?}");
}

#[test]
fn shape_violation_variant_wraps_existing_struct() {
    let report = ShapeViolation {
        shape_iri: "https://example.org/s",
        constraint_iri: "https://example.org/c",
        property_iri: "https://example.org/p",
        expected_range: "xsd:string",
        min_count: 1,
        max_count: 1,
        kind: uor_foundation::ViolationKind::Missing,
    };
    let f = PipelineFailure::ShapeViolation { report };
    let _ = format!("{f:?}");
}
