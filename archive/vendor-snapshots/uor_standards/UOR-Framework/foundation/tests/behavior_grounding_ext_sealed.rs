//! Behavioral contract for W4 closure.
//!
//! Authority: target §4.3 + §9 criterion 1. `Grounding::ground` is
//! foundation-supplied via `GroundingExt`'s blanket impl. Downstream
//! cannot override `ground()` (trait has no such method) and cannot
//! implement `GroundingExt` directly (sealed supertrait).
//!
//! These tests witness the contract structurally AND behaviorally.

use uor_foundation::enforcement::{
    combinators, BinaryGroundingMap, GroundedCoord, GroundingExt, GroundingProgram,
};

/// Downstream impl providing only `program()`. The foundation supplies
/// `ground()` via the `GroundingExt` blanket.
struct ReadByteGrounding;

impl uor_foundation::enforcement::Grounding for ReadByteGrounding {
    type Output = GroundedCoord;
    type Map = BinaryGroundingMap;
    fn program(&self) -> GroundingProgram<GroundedCoord, BinaryGroundingMap> {
        GroundingProgram::from_primitive(combinators::read_bytes::<GroundedCoord>())
    }
}

/// Second downstream impl with a richer program: `Then(ReadBytes,
/// InterpretLeInteger)`. Exercises the composition path.
struct ThenChainGrounding;

impl uor_foundation::enforcement::Grounding for ThenChainGrounding {
    type Output = GroundedCoord;
    type Map = BinaryGroundingMap;
    fn program(&self) -> GroundingProgram<GroundedCoord, BinaryGroundingMap> {
        // `then(read_bytes, interpret_le_integer)` has marker intersection
        // `(TotalMarker, InvertibleMarker)` — exactly what BinaryGroundingMap
        // requires. This is the composition path exercised by the interpreter.
        let first = combinators::read_bytes::<GroundedCoord>();
        let second = combinators::interpret_le_integer::<GroundedCoord>();
        GroundingProgram::from_primitive(combinators::then(first, second))
    }
}

#[test]
fn grounding_ext_ground_delegates_to_program_run() {
    let g = ReadByteGrounding;
    // Calling `.ground()` works because `GroundingExt` is blanket-impl'd
    // over `Grounding` — no manual override in `ReadByteGrounding`.
    let result = <ReadByteGrounding as GroundingExt>::ground(&g, &[0x42u8]);
    assert!(
        result.is_some(),
        "GroundingExt::ground must delegate to program().run_program()"
    );
}

#[test]
fn grounding_ext_ground_handles_composition_program() {
    let g = ThenChainGrounding;
    let result = <ThenChainGrounding as GroundingExt>::ground(&g, &[0x42u8]);
    assert!(
        result.is_some(),
        "GroundingExt::ground must walk composition chains via run_program"
    );
}

#[test]
fn grounding_ext_ground_returns_none_for_empty_input() {
    // ReadBytes on empty input → None, propagated through GroundingExt.
    let g = ReadByteGrounding;
    assert!(
        <ReadByteGrounding as GroundingExt>::ground(&g, &[]).is_none(),
        "empty-input rejection must propagate through GroundingExt"
    );
}

// Compile-time witness: `Grounding` does not expose `fn ground`. If it
// did, this `impl Grounding` body would be required to provide it — but
// omitting `fn ground` compiles cleanly today (trait requires only
// `type Output`, `type Map`, and `fn program`). That silent compilation
// IS the proof that `ground` is no longer in `Grounding`. No `compile_fail`
// doctest needed — the positive tests above exercise the closure.
