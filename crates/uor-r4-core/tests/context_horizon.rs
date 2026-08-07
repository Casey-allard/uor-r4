//! Influence-horizon instrument for the Bott-Fock context fold (issue #424).
//!
//! `tests/context_scaling_benchmarks.rs` measures the fold's *cost* — fixed
//! 256-entry state, bounded per-token latency. This file measures the
//! property that decides whether the fold can carry long context at all:
//! how far back a token still influences the folded state.
//!
//! The fold's update is `cell <- cell - (cell >> 2)` plus a saturated
//! injection, i.e. a geometric decay with ratio 3/4 per token. The decay
//! ratio is a free constant, and it — not the O(1) state size — sets the
//! horizon. This instrument pins the horizon the *shipped* constant
//! produces, so a future change to that constant cannot silently alter the
//! mechanism's reach.
//!
//! Method: fold two token streams that are identical except for the single
//! token `k` positions from the end, then compare the resulting states.
//! The surviving L1 difference is the influence of a token `k` steps back.
//!
//! Recorded verdict (2026-08-07, issue #424): influence falls to ~10% of
//! its immediate value by k=8 — the width of the window the runtime
//! already has — and reaches *exactly zero* by k=64. The fold is therefore
//! structurally incapable of carrying context beyond 63 tokens at the
//! shipped decay constant, independent of how many tokens are folded in.
//! See `docs/context_horizon_424.md` for the corpus-side ceiling this was
//! measured against.

use uor_r4_core::transformerless::bott_fock::{BottFockContextStore, CONTEXT_DIM, STATE_ENTRIES};

/// Deterministic xorshift embedding fill — same shape as the module's own
/// test helper, so streams here are reproducible without an RNG dependency.
fn sample_embedding(seed: u64, out: &mut [i16; CONTEXT_DIM]) {
    let mut x = seed | 1;
    let mut i = 0usize;
    while i < CONTEXT_DIM {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        out[i] = x as i16;
        i += 1;
    }
}

/// Fold `total` tokens, replacing the one at `total - 1 - k` when `perturb`
/// is set. Returns the final state.
fn fold_stream(total: usize, k: usize, perturb: bool) -> [i16; STATE_ENTRIES] {
    let mut store = BottFockContextStore::new();
    let mut token = [0i16; CONTEXT_DIM];
    for i in 0..total {
        if perturb && i == total - 1 - k {
            sample_embedding(i as u64 + 999_999, &mut token);
        } else {
            sample_embedding(i as u64 + 7, &mut token);
        }
        store.append_token(&token);
    }
    *store.state()
}

/// L1 distance between the folded states of the reference stream and the
/// stream perturbed `k` tokens from the end.
fn influence_at_lag(total: usize, k: usize) -> i64 {
    let reference = fold_stream(total, k, false);
    let perturbed = fold_stream(total, k, true);
    (0..STATE_ENTRIES)
        .map(|i| (reference[i] as i64 - perturbed[i] as i64).abs())
        .sum()
}

const STREAM: usize = 4_096;

/// The shipped decay constant confines the fold to a bounded past. A token
/// 64 positions back leaves *no* trace in the state — not a small trace, a
/// bit-identical one. This is the ceiling that makes the fold unable to
/// carry long context, and it is a property of the decay ratio alone.
#[test]
fn influence_horizon_is_bounded_and_reaches_zero() {
    let immediate = influence_at_lag(STREAM, 0);
    assert!(
        immediate > 0,
        "the most recent token must influence the state"
    );

    // Monotone decay: influence never grows with distance.
    let mut previous = immediate;
    for k in [1usize, 2, 4, 8, 16, 24, 32, 48] {
        let current = influence_at_lag(STREAM, k);
        assert!(
            current <= previous,
            "influence must not grow with distance: lag {k} gave {current}, previous {previous}"
        );
        previous = current;
    }

    let vanished = influence_at_lag(STREAM, 64);
    assert_eq!(
        vanished, 0,
        "at the shipped decay constant a token 64 positions back must leave \
         no trace; got L1 {vanished}. If this constant is retuned, update \
         docs/context_horizon_424.md — the horizon is the mechanism's reach."
    );
}

/// Nine tenths of the fold's representational mass sits inside the eight
/// most recent tokens — the window the runtime already carries. Whatever
/// the fold adds beyond that window is the remaining tenth, and this is the
/// arithmetic that made the #424 A/B unreachable at the shipped constant.
#[test]
fn most_influence_mass_lies_inside_the_existing_window() {
    let immediate = influence_at_lag(STREAM, 0) as f64;
    let at_window_edge = influence_at_lag(STREAM, 8) as f64;
    let residue = at_window_edge / immediate;
    assert!(
        residue < 0.15,
        "influence surviving past the 8-token window must stay under 15% of \
         the immediate token's; got {:.1}%",
        residue * 100.0
    );
    // And it is genuinely nonzero — the fold does reach past the window,
    // just not far. Recording both bounds keeps the claim honest.
    assert!(
        residue > 0.01,
        "the fold does reach past the window; got {:.3}%",
        residue * 100.0
    );
}

/// Distinct long streams must still fold to distinct states: the horizon
/// bound above is a decay property, not state collapse. Without this, a
/// zero at lag 64 could be read as "the state saturated and stopped
/// responding to anything", which is a different (and worse) defect.
#[test]
fn bounded_horizon_is_decay_not_state_collapse() {
    let mut states = Vec::new();
    for stream in 0..32u64 {
        let mut store = BottFockContextStore::new();
        let mut token = [0i16; CONTEXT_DIM];
        for i in 0..1_024u64 {
            sample_embedding(
                i.wrapping_mul(2_654_435_761)
                    .wrapping_add(stream.wrapping_mul(1_000_003)),
                &mut token,
            );
            store.append_token(&token);
        }
        states.push(*store.state());
    }
    for i in 0..states.len() {
        for j in (i + 1)..states.len() {
            assert_ne!(
                states[i], states[j],
                "distinct streams {i} and {j} folded to identical states"
            );
        }
        assert!(
            states[i]
                .iter()
                .any(|&cell| cell != i16::MAX && cell != i16::MIN),
            "stream {i} saturated every cell; the state is not carrying content"
        );
    }
}
