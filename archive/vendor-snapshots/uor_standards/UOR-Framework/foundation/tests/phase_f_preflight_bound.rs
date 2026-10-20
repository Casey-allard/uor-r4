//! Phase F: `preflight_feasibility` `Bound`-arm dispatch on `observable_iri`.
//!
//! Canonical observables checked:
//! - `observable:ValueModObservable` — args `"modulus|residue"`; pass iff `modulus != 0 && residue < modulus`.
//! - `observable:CarryDepthObservable` — args `"depth"`; pass iff `depth <= WITT_MAX_BITS`.
//! - `observable:LandauerCost` — args `"u64-bits"` interpreted as `f64::from_bits`; pass iff finite and `> 0`.
//!
//! An unknown `observable_iri` is rejected so unaudited observables cannot thread through preflight.

use uor_foundation::pipeline::{preflight_feasibility, ConstraintRef};

const VALUE_MOD: &str = "https://uor.foundation/observable/ValueModObservable";
const CARRY_DEPTH: &str = "https://uor.foundation/observable/CarryDepthObservable";
const LANDAUER: &str = "https://uor.foundation/observable/LandauerCost";
const BOUND_SHAPE: &str = "https://uor.foundation/conformance/BoundShape";

#[test]
fn value_mod_valid_residue_passes() {
    let cs = &[ConstraintRef::Bound {
        observable_iri: VALUE_MOD,
        bound_shape_iri: BOUND_SHAPE,
        args_repr: "7|3",
    }];
    assert!(
        preflight_feasibility(cs).is_ok(),
        "7|3 means residue 3 mod 7 — valid"
    );
}

#[test]
fn value_mod_residue_out_of_range_rejected() {
    let cs = &[ConstraintRef::Bound {
        observable_iri: VALUE_MOD,
        bound_shape_iri: BOUND_SHAPE,
        args_repr: "5|7",
    }];
    let err = preflight_feasibility(cs).unwrap_err();
    assert_eq!(err.shape_iri, BOUND_SHAPE);
    assert_eq!(err.property_iri, VALUE_MOD);
}

#[test]
fn carry_depth_in_range_passes() {
    let cs = &[ConstraintRef::Bound {
        observable_iri: CARRY_DEPTH,
        bound_shape_iri: BOUND_SHAPE,
        args_repr: "128",
    }];
    assert!(
        preflight_feasibility(cs).is_ok(),
        "depth 128 is well within WITT_MAX_BITS"
    );
}

#[test]
fn carry_depth_over_max_rejected() {
    let cs = &[ConstraintRef::Bound {
        observable_iri: CARRY_DEPTH,
        bound_shape_iri: BOUND_SHAPE,
        args_repr: "20000",
    }];
    let err = preflight_feasibility(cs).unwrap_err();
    assert_eq!(err.shape_iri, BOUND_SHAPE);
    assert_eq!(err.property_iri, CARRY_DEPTH);
}

#[test]
fn landauer_positive_finite_passes() {
    // 1.0_f64.to_bits() = 0x3FF0000000000000 = 4_607_182_418_800_017_408
    let cs = &[ConstraintRef::Bound {
        observable_iri: LANDAUER,
        bound_shape_iri: BOUND_SHAPE,
        args_repr: "4607182418800017408",
    }];
    assert!(
        preflight_feasibility(cs).is_ok(),
        "1.0 nats is finite and positive"
    );
}

#[test]
fn landauer_zero_rejected() {
    // 0.0_f64.to_bits() = 0 — zero nats is not > 0.0
    let cs = &[ConstraintRef::Bound {
        observable_iri: LANDAUER,
        bound_shape_iri: BOUND_SHAPE,
        args_repr: "0",
    }];
    let err = preflight_feasibility(cs).unwrap_err();
    assert_eq!(err.shape_iri, BOUND_SHAPE);
    assert_eq!(err.property_iri, LANDAUER);
}

#[test]
fn unknown_observable_rejected() {
    let cs = &[ConstraintRef::Bound {
        observable_iri: "https://uor.foundation/observable/FabricatedObservable",
        bound_shape_iri: BOUND_SHAPE,
        args_repr: "whatever",
    }];
    let err = preflight_feasibility(cs).unwrap_err();
    assert_eq!(err.shape_iri, BOUND_SHAPE);
    assert_eq!(
        err.property_iri,
        "https://uor.foundation/observable/FabricatedObservable"
    );
    assert_eq!(
        err.constraint_iri,
        "https://uor.foundation/type/BoundConstraint"
    );
}
