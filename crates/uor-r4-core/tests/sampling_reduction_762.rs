//! Exhaustive/randomized verification of `reduce_into_range` (issue #762
//! lever 2 prototype) against the real `%` operator.
//!
//! This lives in an external `tests/` file rather than inside
//! `runtime.rs` itself (or even inside `mod.rs`'s inline test modules)
//! because `p4_runtime_source_scan` scans all of `runtime.rs` including
//! its own test code for forbidden `/`, `%`, `*` tokens — the same reason
//! `dot_assignment_tests` lives in `mod.rs` rather than in `runtime.rs`.
//! An external integration test crate only sees `uor_r4_core`'s public
//! API, which is why `reduce_into_range` is `pub` rather than
//! `pub(crate)`.

use uor_r4_core::transformerless::runtime::reduce_into_range;

#[test]
fn reduce_into_range_matches_modulo_exhaustively_for_small_values() {
    for bound in 1u32..300 {
        for value in 0u32..600 {
            let want = value % bound;
            let got = reduce_into_range(value, bound);
            assert_eq!(
                got, want,
                "reduce_into_range({value}, {bound}) = {got}, want {want}"
            );
        }
    }
}

#[test]
fn reduce_into_range_zero_bound_returns_zero() {
    assert_eq!(reduce_into_range(0, 0), 0);
    assert_eq!(reduce_into_range(12345, 0), 0);
    assert_eq!(reduce_into_range(u32::MAX, 0), 0);
}

#[test]
fn reduce_into_range_value_less_than_bound_is_identity() {
    assert_eq!(reduce_into_range(0, 1), 0);
    assert_eq!(reduce_into_range(5, 10), 5);
    assert_eq!(reduce_into_range(u32::MAX - 1, u32::MAX), u32::MAX - 1);
}

#[test]
fn reduce_into_range_edge_cases_against_modulo() {
    let cases: &[(u32, u32)] = &[
        (0, 1),
        (1, 1),
        (u32::MAX, 1),
        (u32::MAX, u32::MAX),
        (u32::MAX, u32::MAX - 1),
        (u32::MAX / 2, 3),
        (u32::MAX, 2),
        (u32::MAX, 3),
        (u32::MAX, 7),
        (0x8000_0000, 0x8000_0000),
        (0x8000_0000, 3),
        (0xFFFF_FFFE, 0x7FFF_FFFF),
        (1 << 31, 1 << 30),
        (1 << 31, (1 << 30) + 1),
        (7, 7),
        (14, 7),
        (13, 7),
        (100, 100),
        (100, 99),
        (100, 3),
    ];
    for &(value, bound) in cases {
        let want = value % bound;
        let got = reduce_into_range(value, bound);
        assert_eq!(
            got, want,
            "reduce_into_range({value}, {bound}) = {got}, want {want}"
        );
    }
}

#[test]
fn reduce_into_range_matches_modulo_on_a_large_random_sweep() {
    // Deterministic xorshift32, fine to use `%` here since this file is
    // outside runtime.rs's P-4 scan.
    let mut state = 0x00C0_FFEEu32;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state
    };
    for _ in 0..200_000 {
        let value = next();
        let mut bound = next();
        if bound == 0 {
            bound = 1;
        }
        let want = value % bound;
        let got = reduce_into_range(value, bound);
        assert_eq!(
            got, want,
            "reduce_into_range({value}, {bound}) = {got}, want {want}"
        );
    }
}

#[test]
fn reduce_into_range_powers_of_two_bounds() {
    for shift in 0u32..32 {
        let bound = 1u32 << shift;
        for value in [0u32, 1, bound - 1, bound, bound + 1, u32::MAX, u32::MAX - 1] {
            let want = value % bound;
            let got = reduce_into_range(value, bound);
            assert_eq!(
                got, want,
                "reduce_into_range({value}, {bound}) = {got}, want {want}"
            );
        }
    }
}
