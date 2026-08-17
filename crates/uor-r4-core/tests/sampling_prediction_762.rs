//! Regression tests for issue #762 lever 2 (`predict_witness_sampled`,
//! `generate_sampled_into`): proves the new opt-in sampling path is
//! genuinely probabilistic (varies with seed) yet reproducible (fixed seed
//! -> fixed output), stays within the evidence set, and does not change
//! `predict_witness`'s or `generate_greedy_into`'s existing deterministic
//! behavior at all.

use std::collections::BTreeSet;

use uor_r4_core::transformerless::compiler::{self, Compiled, STAGES};
use uor_r4_core::transformerless::runtime::{self, Prediction, Runtime, SampleRng, Store};

fn fixture_artifacts() -> Compiled {
    let dir = env!("CARGO_MANIFEST_DIR");
    let bytes = std::fs::read(format!("{dir}/tests/fixtures/tless_artifacts.bin"))
        .expect("fixture artifacts.bin present");
    compiler::parse_artifacts(&bytes).expect("fixture artifacts parse")
}

/// A store with a rich, contested distribution: several tokens with
/// meaningfully different counts under one code, so argmax always picks
/// the same winner (10, the highest count) but weighted sampling has real
/// spread to work with. `add_evidence` always also writes the depth-0
/// (empty-key, universal) row, so this evidence is reachable via
/// `assign_window` fallback regardless of what code an arbitrary window
/// resolves to — see `add_evidence_multi` in runtime.rs.
fn rich_store() -> Store {
    let mut store: Store = (0..=STAGES).map(|_| Default::default()).collect();
    let code = [0u8; STAGES];
    runtime::add_evidence(&mut store, &code, 10, 50);
    runtime::add_evidence(&mut store, &code, 20, 30);
    runtime::add_evidence(&mut store, &code, 30, 15);
    runtime::add_evidence(&mut store, &code, 40, 5);
    store
}

#[test]
fn predict_witness_sampled_is_deterministic_for_a_fixed_seed() {
    let art = fixture_artifacts();
    let store = rich_store();
    let code = [0u8; STAGES];

    let mut rt_a = Runtime::new(&art);
    let mut rng_a = SampleRng::new(42);
    let a = rt_a.predict_witness_sampled(&store, &code, &mut rng_a);

    let mut rt_b = Runtime::new(&art);
    let mut rng_b = SampleRng::new(42);
    let b = rt_b.predict_witness_sampled(&store, &code, &mut rng_b);

    assert_eq!(a, b, "same seed must reproduce the same sampled prediction");
}

#[test]
fn predict_witness_sampled_varies_across_seeds_on_a_contested_distribution() {
    let art = fixture_artifacts();
    let store = rich_store();
    let code = [0u8; STAGES];

    let mut seen = BTreeSet::new();
    for seed in 0u32..64 {
        let mut rt = Runtime::new(&art);
        let mut rng = SampleRng::new(seed);
        let p = rt.predict_witness_sampled(&store, &code, &mut rng);
        seen.insert(p.token);
    }
    assert!(
        seen.len() > 1,
        "sampling across 64 seeds on a 4-candidate contested distribution \
         picked only one distinct token — sampling is not actually varying \
         the outcome"
    );
}

#[test]
fn predict_witness_sampled_never_returns_a_token_outside_the_evidence_set() {
    let art = fixture_artifacts();
    let store = rich_store();
    let code = [0u8; STAGES];
    let valid: BTreeSet<u32> = [10, 20, 30, 40].into_iter().collect();

    for seed in 0u32..200 {
        let mut rt = Runtime::new(&art);
        let mut rng = SampleRng::new(seed);
        let p = rt.predict_witness_sampled(&store, &code, &mut rng);
        assert!(
            valid.contains(&p.token),
            "seed {seed} produced token {} outside the evidence set",
            p.token
        );
    }
}

#[test]
fn predict_witness_default_path_is_unchanged_by_the_sampling_addition() {
    let art = fixture_artifacts();
    let store = rich_store();
    let code = [0u8; STAGES];

    let mut rt = Runtime::new(&art);
    let p = rt.predict_witness(&store, &code);
    // The highest-count candidate (10, count 50) must still win under
    // plain argmax, unaffected by predict_witness_sampled existing in the
    // same impl block.
    assert_eq!(p.token, 10);
    assert_eq!(p.count, 50);
}

#[test]
fn generate_sampled_into_varies_across_seeds_and_generate_greedy_into_is_unaffected() {
    let art = fixture_artifacts();
    let store = rich_store();
    let seed_tokens = [1u32, 2, 3];

    // generate_greedy_into: deterministic, resolved every step via the
    // depth-0 fallback row (this store has no evidence at any code an
    // arbitrary [1, 2, 3] window resolves to, so `predict_witness` always
    // falls through to the universal empty-key row -- same distribution,
    // {10: 50, 20: 30, 30: 15, 40: 5}, at each of the 4 steps).
    //
    // The naive expectation is "always token 10" (the argmax winner), but
    // `predict_witness`'s existing, pre-#762 repetition penalty
    // (`self.state.token_occurrences(t)`, scored via
    // `score -= (val << 10) - (val << 4) - (val << 3)`) already fires as
    // soon as a token has been emitted once in this `Runtime`'s state, not
    // just on an immediate repeat. So each step's winner is knocked out of
    // contention for every later step, and the 4 steps walk the
    // distribution in count order: 10 (score 50, first pick) -> 20 (10 is
    // now penalized to -950, 20's 30 wins) -> 30 (10 and 20 penalized, 30's
    // 15 wins) -> 40 (10/20/30 penalized, 40's 5 wins by default). This is
    // existing `predict_witness` behavior, unrelated to and unaffected by
    // the #762 sampling addition -- asserting the exact sequence here (not
    // "always 10") is what actually proves the default path is unchanged.
    let mut rt = Runtime::new(&art);
    let mut out = [Prediction::default(); 4];
    rt.generate_greedy_into(&store, &seed_tokens, &mut out);
    let tokens: Vec<u32> = out.iter().map(|p| p.token).collect();
    assert_eq!(
        tokens,
        vec![10, 20, 30, 40],
        "generate_greedy_into's default (unsampled) path must still walk \
         the repetition-penalized distribution in this exact, established \
         order -- any change here means the sampling addition perturbed \
         the existing deterministic path"
    );

    // generate_sampled_into: collect the first generated token across many
    // seeds and confirm it isn't always 10.
    let mut seen = BTreeSet::new();
    for seed in 0u32..64 {
        let mut rt = Runtime::new(&art);
        let mut rng = SampleRng::new(seed);
        let mut out = [Prediction::default(); 1];
        rt.generate_sampled_into(&store, &seed_tokens, &mut rng, &mut out);
        seen.insert(out[0].token);
    }
    assert!(
        seen.len() > 1,
        "generate_sampled_into across 64 seeds picked only one distinct \
         first token"
    );
}

#[test]
fn sample_rng_zero_seed_does_not_produce_a_stuck_zero_sequence() {
    let mut rng = SampleRng::new(0);
    let mut seen = BTreeSet::new();
    for _ in 0..8 {
        // next_u32 is private; exercise it indirectly through a sampling
        // call instead of reaching in directly.
        seen.insert(format!("{rng:?}"));
        let art = fixture_artifacts();
        let store = rich_store();
        let code = [0u8; STAGES];
        let mut rt = Runtime::new(&art);
        let _ = rt.predict_witness_sampled(&store, &code, &mut rng);
    }
    assert!(
        seen.len() > 1,
        "SampleRng::new(0) must not produce a degenerate stuck sequence"
    );
}
