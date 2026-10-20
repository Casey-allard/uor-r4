//! Phase K (target §4.3 + §9 criterion 1): mechanical closure of W4 —
//! `Grounding::ground`'s kind discriminator is mechanically verifiable from
//! the combinator decomposition rather than being a promise.
//!
//! The verification works at the type level: `program()` returns a
//! `GroundingProgram<Self::Output, Self::Map>`, whose construction via
//! `from_primitive<Markers>` requires `Markers: MarkersImpliedBy<Map>`. If
//! a downstream impl claims `Map = IntegerGroundingMap` (which requires
//! `Total + Invertible + PreservesStructure`) but provides a program built
//! from a primitive whose marker tuple lacks a required property, the
//! code fails to compile. This test exercises the valid path and relies on
//! the existing `compile_fail` doctest in enforcement.rs for the rejection
//! path.

use uor_foundation::enforcement::{
    combinators, BinaryGroundingMap, GroundedCoord, Grounding, GroundingExt, GroundingProgram,
    IntegerGroundingMap,
};

/// A conformant grounding impl: the combinator program's marker tuple
/// satisfies `MarkersImpliedBy<BinaryGroundingMap>` (which requires
/// `Total + Invertible`). `read_bytes` returns `(Total, Invertible)` — match.
struct ReadByteGrounding;

impl Grounding for ReadByteGrounding {
    type Output = GroundedCoord;
    type Map = BinaryGroundingMap;

    fn program(&self) -> GroundingProgram<GroundedCoord, BinaryGroundingMap> {
        // Phase K: the type system verifies at THIS POINT that read_bytes'
        // (Total, Invertible) tuple satisfies MarkersImpliedBy<BinaryGroundingMap>.
        // Replace with `digest::<GroundedCoord>()` and the code fails to compile
        // because (Total,) does not imply Invertible.
        GroundingProgram::from_primitive(combinators::read_bytes::<GroundedCoord>())
    }
    // W4 closure: `ground()` is supplied by `GroundingExt`'s blanket impl;
    // downstream impls provide only `program()`.
}

/// A conformant impl with a stronger map kind.
struct InterpretIntegerGrounding;

impl Grounding for InterpretIntegerGrounding {
    type Output = GroundedCoord;
    type Map = IntegerGroundingMap;

    fn program(&self) -> GroundingProgram<GroundedCoord, IntegerGroundingMap> {
        // interpret_le_integer returns (Total, Invertible, PreservesStructure)
        // — exactly what IntegerGroundingMap requires.
        GroundingProgram::from_primitive(combinators::interpret_le_integer::<GroundedCoord>())
    }
    // W4 closure: `ground()` is supplied by `GroundingExt`.
}

#[test]
fn phase_k_program_returns_typed_grounding_program() {
    let g = ReadByteGrounding;
    let prog: GroundingProgram<GroundedCoord, BinaryGroundingMap> = g.program();
    let _ = prog;
}

#[test]
fn phase_k_ground_runs_combinator_program_for_first_byte() {
    let g = ReadByteGrounding;
    let result = g.ground(&[0x42u8]);
    assert!(result.is_some(), "read_bytes must produce a GroundedCoord");
}

#[test]
fn phase_k_ground_returns_none_on_empty_input() {
    let g = ReadByteGrounding;
    assert!(g.ground(&[]).is_none(), "empty input must return None");
}

#[test]
fn phase_k_integer_grounding_runs() {
    let g = InterpretIntegerGrounding;
    let result = g.ground(&[0x55u8, 0x66u8]);
    assert!(result.is_some());
}

#[test]
fn phase_k_ground_is_deterministic() {
    // Same input → same output. Foundation interpreter is pure.
    let g = ReadByteGrounding;
    let a = g.ground(&[0xabu8]);
    let b = g.ground(&[0xabu8]);
    assert_eq!(a, b);
}
