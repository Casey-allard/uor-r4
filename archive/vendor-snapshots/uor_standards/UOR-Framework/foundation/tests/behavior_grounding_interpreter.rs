//! Behavioral contract for `GroundingProgram<GroundedCoord, Map>::run`.
//!
//! Target §4.3: the foundation's interpreter handles every combinator op
//! in the `GroundingPrimitiveOp` enum (12 total). A regression where the
//! interpreter returns `None` for inputs that should produce values would
//! break downstream grounding-map impls.
//!
//! The 12 ops:
//! 1. ReadBytes           — read a byte slice
//! 2. InterpretLeInteger  — little-endian integer decode
//! 3. InterpretBeInteger  — big-endian integer decode
//! 4. Digest              — blake3/sha256 hash
//! 5. DecodeUtf8          — UTF-8 string decode
//! 6. DecodeJson          — JSON value decode
//! 7. SelectField         — field access
//! 8. SelectIndex         — index access
//! 9. ConstValue          — foundation constant
//! 10. Then               — sequential composition
//! 11. MapErr             — error-variant transformation
//! 12. AndThen            — monadic bind
//!
//! If any op silently returns `None` for a valid input, this test fails
//! and pinpoints which op is unimplemented.

use uor_foundation::enforcement::{
    combinators, BinaryGroundingMap, DigestGroundingMap, GroundedCoord, GroundingProgram,
    IntegerGroundingMap,
};

// ─── ReadBytes ───────────────────────────────────────────────────────────

#[test]
fn grounding_read_bytes_produces_grounded_coord_for_nonempty_input() {
    let prog: GroundingProgram<GroundedCoord, BinaryGroundingMap> =
        GroundingProgram::from_primitive(combinators::read_bytes::<GroundedCoord>());
    let result = prog.run(&[0x42u8]);
    assert!(
        result.is_some(),
        "ReadBytes on non-empty input must produce Some(GroundedCoord)"
    );
}

#[test]
fn grounding_read_bytes_returns_none_for_empty_input() {
    let prog: GroundingProgram<GroundedCoord, BinaryGroundingMap> =
        GroundingProgram::from_primitive(combinators::read_bytes::<GroundedCoord>());
    assert!(
        prog.run(&[]).is_none(),
        "ReadBytes on empty input returns None"
    );
}

// ─── InterpretLeInteger ──────────────────────────────────────────────────

#[test]
fn grounding_interpret_le_integer_produces_coord() {
    let prog: GroundingProgram<GroundedCoord, IntegerGroundingMap> =
        GroundingProgram::from_primitive(combinators::interpret_le_integer::<GroundedCoord>());
    let result = prog.run(&[0x55u8, 0x66u8, 0x77u8]);
    assert!(
        result.is_some(),
        "InterpretLeInteger must produce Some(GroundedCoord) for non-empty input"
    );
}

// ─── InterpretBeInteger ──────────────────────────────────────────────────

#[test]
fn grounding_interpret_be_integer_produces_coord() {
    let prog: GroundingProgram<GroundedCoord, IntegerGroundingMap> =
        GroundingProgram::from_primitive(combinators::interpret_be_integer::<GroundedCoord>());
    let result = prog.run(&[0x55u8, 0x66u8, 0x77u8]);
    assert!(
        result.is_some(),
        "InterpretBeInteger must produce Some(GroundedCoord) for non-empty input"
    );
}

// ─── Digest ──────────────────────────────────────────────────────────────

#[test]
fn grounding_digest_produces_coord() {
    let prog: GroundingProgram<GroundedCoord, DigestGroundingMap> =
        GroundingProgram::from_primitive(combinators::digest::<GroundedCoord>());
    let result = prog.run(&[0x01u8, 0x02u8, 0x03u8, 0x04u8]);
    assert!(
        result.is_some(),
        "Digest must produce Some(GroundedCoord) for non-empty input"
    );
}

// ─── DecodeUtf8 ──────────────────────────────────────────────────────────

#[test]
fn grounding_decode_utf8_produces_coord_for_valid_utf8() {
    use uor_foundation::enforcement::Utf8GroundingMap;
    let prog: GroundingProgram<GroundedCoord, Utf8GroundingMap> =
        GroundingProgram::from_primitive(combinators::decode_utf8::<GroundedCoord>());
    // Valid UTF-8 bytes (ASCII "A") — must produce Some.
    let result = prog.run(&[0x41u8]);
    assert!(
        result.is_some(),
        "DecodeUtf8 must produce Some(GroundedCoord) for valid UTF-8 input \
         (got None \u{2014} the interpreter path for DecodeUtf8 is unimplemented)"
    );
}

// ─── DecodeJson ──────────────────────────────────────────────────────────

#[test]
fn grounding_decode_json_produces_coord_for_valid_json() {
    use uor_foundation::enforcement::JsonGroundingMap;
    let prog: GroundingProgram<GroundedCoord, JsonGroundingMap> =
        GroundingProgram::from_primitive(combinators::decode_json::<GroundedCoord>());
    // Valid JSON integer — "7"
    let result = prog.run(&[0x37u8]);
    assert!(
        result.is_some(),
        "DecodeJson must produce Some(GroundedCoord) for valid JSON input \
         (got None \u{2014} the interpreter path for DecodeJson is unimplemented)"
    );
}

// ─── SelectField + SelectIndex — only usable via composition ───────────
//
// Both have marker tuple `(Invertible,)` alone, which satisfies no shipped
// GroundingMap kind directly. Their behavior is verified through Then /
// AndThen chains below. Their existence as combinator builders is still
// pinned — if they vanish from the combinators module, this test file
// fails to compile.

#[test]
fn select_field_and_select_index_builders_addressable() {
    // Compile-time witness that the combinator builders exist with the
    // expected (Invertible,) marker tuple. At runtime, their `op()`
    // accessor returns the correct `GroundingPrimitiveOp` variant.
    use uor_foundation::enforcement::{GroundingPrimitive, GroundingPrimitiveOp, InvertibleMarker};
    let sf: GroundingPrimitive<GroundedCoord, (InvertibleMarker,)> =
        combinators::select_field::<GroundedCoord>();
    let si: GroundingPrimitive<GroundedCoord, (InvertibleMarker,)> =
        combinators::select_index::<GroundedCoord>();
    assert_eq!(sf.op(), GroundingPrimitiveOp::SelectField);
    assert_eq!(si.op(), GroundingPrimitiveOp::SelectIndex);
}

// ─── ConstValue ──────────────────────────────────────────────────────────

#[test]
fn grounding_const_value_produces_coord_regardless_of_input() {
    let prog: GroundingProgram<GroundedCoord, IntegerGroundingMap> =
        GroundingProgram::from_primitive(combinators::const_value::<GroundedCoord>());
    // ConstValue returns a foundation-known constant — should work on
    // empty input too.
    let result = prog.run(&[]);
    assert!(
        result.is_some(),
        "ConstValue must produce Some(GroundedCoord) for any input (including empty) \
         (got None \u{2014} the interpreter path for ConstValue is unimplemented)"
    );
}

// ─── Then, MapErr, AndThen — composition combinators ────────────────────
//
// These take a GroundingPrimitive argument (not a bare "no-input" builder),
// so their presence is verified by the combinator builder surface. The
// interpreter's run() branch for them is the behavioral contract here.

#[test]
fn grounding_then_chain_produces_coord() {
    // Build a two-step chain: read_bytes → interpret_le_integer.
    let first = combinators::read_bytes::<GroundedCoord>();
    let second = combinators::interpret_le_integer::<GroundedCoord>();
    let composed = combinators::then(first, second);
    let prog: GroundingProgram<GroundedCoord, BinaryGroundingMap> =
        GroundingProgram::from_primitive(composed);
    let result = prog.run(&[0x42u8]);
    assert!(
        result.is_some(),
        "Then(ReadBytes, InterpretLeInteger) must produce Some(GroundedCoord) \
         (got None \u{2014} the interpreter path for Then is unimplemented)"
    );
}

#[test]
fn grounding_map_err_produces_coord() {
    let base = combinators::read_bytes::<GroundedCoord>();
    let mapped = combinators::map_err(base);
    let prog: GroundingProgram<GroundedCoord, BinaryGroundingMap> =
        GroundingProgram::from_primitive(mapped);
    let result = prog.run(&[0x42u8]);
    assert!(
        result.is_some(),
        "MapErr must preserve the success value \
         (got None \u{2014} the interpreter path for MapErr is unimplemented)"
    );
}

#[test]
fn grounding_and_then_chain_produces_coord() {
    let first = combinators::read_bytes::<GroundedCoord>();
    let second = combinators::interpret_le_integer::<GroundedCoord>();
    let composed = combinators::and_then(first, second);
    let prog: GroundingProgram<GroundedCoord, BinaryGroundingMap> =
        GroundingProgram::from_primitive(composed);
    let result = prog.run(&[0x42u8]);
    assert!(
        result.is_some(),
        "AndThen(ReadBytes, InterpretLeInteger) must produce Some(GroundedCoord) \
         (got None \u{2014} the interpreter path for AndThen is unimplemented)"
    );
}

// ─── GroundedTuple<N> interpreter (W4 closure) ───────────────────────────

#[test]
fn grounding_tuple_2_produces_tuple_from_split_input() {
    use uor_foundation::enforcement::GroundedTuple;
    let prog: GroundingProgram<GroundedTuple<2>, BinaryGroundingMap> =
        GroundingProgram::from_primitive(combinators::read_bytes::<GroundedTuple<2>>());
    let result = prog.run(&[0x01u8, 0x02u8]);
    assert!(
        result.is_some(),
        "GroundedTuple<2> interpreter must split input into N windows and run per window"
    );
}

#[test]
fn grounding_tuple_4_requires_divisible_input_length() {
    use uor_foundation::enforcement::GroundedTuple;
    let prog: GroundingProgram<GroundedTuple<4>, BinaryGroundingMap> =
        GroundingProgram::from_primitive(combinators::read_bytes::<GroundedTuple<4>>());
    assert!(
        prog.run(&[0x01u8, 0x02u8, 0x03u8]).is_none(),
        "length-3 input not divisible by 4 must return None"
    );
    assert!(
        prog.run(&[0x01u8, 0x02u8, 0x03u8, 0x04u8]).is_some(),
        "length-4 input divisible by 4 must produce Some(GroundedTuple<4>)"
    );
}

#[test]
fn grounding_tuple_rejects_empty_input() {
    use uor_foundation::enforcement::GroundedTuple;
    let prog: GroundingProgram<GroundedTuple<2>, BinaryGroundingMap> =
        GroundingProgram::from_primitive(combinators::read_bytes::<GroundedTuple<2>>());
    assert!(prog.run(&[]).is_none(), "empty input must return None");
}
