//! Equality obligation for the prepared assignment path (issue #469 lever B).
//!
//! `assign_code_for_bundle_with` routes the per-stage dot scan through the
//! vectorized kernel using tables decoded once. The assign path is κ-pinned,
//! so the only thing that makes that routing shippable is a proof — not an
//! argument — that it produces the same code as the scalar scan for every
//! artifact shape and every bundle.
//!
//! This file is that proof at the unit level: a differential test against the
//! scalar path over both dot-metric artifact shapes (TLA6 plain, TLA7
//! residual), the sign-metric fallback, and randomized bundles chosen to
//! exercise sign changes, zero work, and magnitudes large enough to make the
//! accumulation order observable if it were fragile.
//!
//! `tests/kappa_reproduction.rs` is the same obligation at corpus scale
//! against the pinned κ, and it is the gate that actually blocks a ship.
//!
//! Note on what is NOT tested here: the vectorized kernel dispatches on
//! runtime CPU features (AVX2, NEON, scalar fallback). A run of this test
//! witnesses the dispatch available on the machine it ran on. CI covering
//! x86_64 and the aarch64 development machine is what makes the set complete;
//! `prepared_reports_vectorization` records which path was taken so a green
//! run cannot be mistaken for coverage it did not have.

use uor_r4_core::transformerless::{compiler, runtime};

fn synthetic_dot_art() -> compiler::Compiled {
    let mut art = compiler::Compiled {
        token_codes: vec![0u8; compiler::STAGES],
        stage_books: (0..compiler::STAGES)
            .map(|_| vec![0i8; compiler::K * compiler::D])
            .collect(),
        stage_shifts: vec![0u8; compiler::STAGES],
        thresholds: (0..compiler::D as i64).map(|d| (d % 7) - 3).collect(),
        class_sigs: (0..compiler::STAGES)
            .map(|_| vec![0u8; compiler::K * compiler::D / 8])
            .collect(),
        ctx_cb: Vec::new(),
        token_stage_kappas: Vec::new(),
        dot_cb: Vec::new(),
        resid_cb: Vec::new(),
        resid_scale_shifts: Vec::new(),
        norm_fold_const: 0,
    };
    art.dot_cb = (0..compiler::STAGES)
        .map(|st| {
            (0..compiler::K * compiler::D)
                .map(|i| {
                    let v = ((i + st) % 13) as f32 - 6.0;
                    compiler::pack_dot_entry(v / 8.0)
                })
                .collect()
        })
        .collect();
    art
}

fn synthetic_resid_art() -> compiler::Compiled {
    let mut art = synthetic_dot_art();
    art.resid_cb = (0..compiler::STAGES)
        .map(|st| {
            (0..compiler::K * compiler::D)
                .map(|i| (((i + st) % 13) as i8) - 6)
                .collect()
        })
        .collect();
    art.resid_scale_shifts = vec![4u8; compiler::STAGES];
    art.norm_fold_const = 10;
    art
}

/// Deterministic xorshift64* bundle stream — no RNG dependency, and the same
/// stream on every machine, so a divergence is reproducible from the seed
/// alone.
///
/// Magnitudes are bounded to ±1e9. That is not timidity: the scalar reference
/// `dot_score_plain` accumulates `D` terms into an `i64` without saturating,
/// so a debug build traps on overflow long before any prepared/scalar
/// disagreement could be observed. Real bundles are window token counts and
/// sit orders of magnitude below this bound, so the tested regime strictly
/// contains the deployed one while staying inside the reference's own domain.
fn bundles(count: usize) -> Vec<[i64; compiler::D]> {
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    (0..count)
        .map(|index| {
            std::array::from_fn(|_| match index % 4 {
                // all-zero work: every class scores 0, so the tie rule alone
                // decides the class. The strongest single test of tie parity.
                0 => 0,
                // small signed values around zero
                1 => (next() % 17) as i64 - 8,
                // large magnitudes: shifts move real bits, and any accumulator
                // width difference between the paths would show here
                2 => (next() % 2_000_001) as i64 - 1_000_000,
                _ => (next() % 2_000_000_001) as i64 - 1_000_000_000,
            })
        })
        .collect()
}

/// The prepared path must agree with the scalar path on every bundle, for
/// both dot-metric artifact shapes.
#[test]
fn prepared_matches_scalar_on_dot_and_residual_artifacts() {
    for (label, art) in [
        ("TLA6 plain dot", synthetic_dot_art()),
        ("TLA7 residual", synthetic_resid_art()),
    ] {
        let tables = runtime::AssignTables::new(&art);
        assert!(
            tables.is_vectorized(),
            "[{label}] dot artifacts must decode to prepared tables, or this test is vacuous"
        );
        let mut distinct = std::collections::BTreeSet::new();
        for (index, bundle) in bundles(512).iter().enumerate() {
            let scalar = runtime::assign_code_for_bundle(&art, bundle);
            let prepared = runtime::assign_code_for_bundle_with(&tables, &art, bundle);
            assert_eq!(
                scalar, prepared,
                "[{label}] prepared/scalar divergence at bundle {index}"
            );
            distinct.insert(scalar);
        }
        // A path that returned a constant code would pass the comparison
        // above while measuring nothing.
        assert!(
            distinct.len() > 1,
            "[{label}] every bundle produced the same code ({distinct:?}); \
             the fixture does not exercise the scan"
        );
    }
}

/// The membership-beam form's primary code is what the corpus pass consumes.
/// `code_plain_with` takes the argmax form instead, so their equality is part
/// of this obligation rather than an assumption inherited from a doc comment.
#[test]
fn beam_primary_equals_prepared_code() {
    for (label, art) in [
        ("TLA6 plain dot", synthetic_dot_art()),
        ("TLA7 residual", synthetic_resid_art()),
    ] {
        let tables = runtime::AssignTables::new(&art);
        for (index, bundle) in bundles(256).iter().enumerate() {
            assert_eq!(
                runtime::assign_for_bundle(&art, bundle),
                runtime::assign_code_for_bundle_with(&tables, &art, bundle),
                "[{label}] beam primary vs prepared argmax at bundle {index}"
            );
        }
    }
}

/// Sign-metric artifacts have no dot tables. They must decode to `None` and
/// fall back to the scalar path with identical results — the fallback-or-
/// nothing rule, with no third outcome.
#[test]
fn sign_metric_artifacts_fall_back_to_scalar() {
    let mut art = synthetic_dot_art();
    art.dot_cb = Vec::new();
    let tables = runtime::AssignTables::new(&art);
    assert!(
        !tables.is_vectorized(),
        "an artifact with no dot tables must not report a vectorized path"
    );
    for (index, bundle) in bundles(128).iter().enumerate() {
        assert_eq!(
            runtime::assign_code_for_bundle(&art, bundle),
            runtime::assign_code_for_bundle_with(&tables, &art, bundle),
            "sign-metric fallback divergence at bundle {index}"
        );
    }
}

/// A short `dot_cb` must leave the same trailing zero stages as the scalar
/// zip. This is the stage-arity half of the equality argument, and it is the
/// one most likely to rot if `STAGES` or the table layout ever changes.
#[test]
fn short_dot_table_list_leaves_the_same_trailing_stages() {
    let mut art = synthetic_dot_art();
    art.dot_cb.truncate(compiler::STAGES - 1);
    let tables = runtime::AssignTables::new(&art);
    for (index, bundle) in bundles(128).iter().enumerate() {
        let scalar = runtime::assign_code_for_bundle(&art, bundle);
        assert_eq!(
            scalar[compiler::STAGES - 1],
            0,
            "the scalar path must leave the unbacked stage at zero"
        );
        assert_eq!(
            scalar,
            runtime::assign_code_for_bundle_with(&tables, &art, bundle),
            "short-table divergence at bundle {index}"
        );
    }
}

/// The obligation at corpus scale, on a REAL artifact, with no teacher
/// checkpoint required.
///
/// `tests/kappa_reproduction.rs` carries the authoritative κ gate, but it
/// compiles a fresh artifact and therefore *skips* wherever the pinned
/// checkpoint is absent — which is most machines, and which is precisely the
/// vacuous-green failure mode issue #354 recorded for `allocation_census`. A
/// change to the assign path cannot rest on a gate that silently does not
/// run.
///
/// So this test uses the checked-in TLA7 container and corpus fixtures the
/// repository already pins, and asserts prepared == scalar over real corpus
/// positions. It fails rather than skips when the fixtures are missing: they
/// are committed, so their absence is a broken checkout, not an environment
/// without a checkpoint.
#[test]
fn prepared_matches_scalar_on_the_pinned_artifact_fixture() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let container = std::fs::read(format!("{dir}/tests/fixtures/tless_artifacts.bin"))
        .expect("pinned TLA7 container fixture is committed; a missing file is a broken checkout");
    let art = compiler::parse_artifacts(&container).expect("pinned container parses");
    let corpus = compiler::load_corpus_from(
        &format!("{dir}/tests/fixtures/c_meta.bin"),
        &format!("{dir}/tests/fixtures/c_recs.bin"),
    )
    .expect("pinned corpus fixtures load");

    let tables = runtime::AssignTables::new(&art);
    assert!(
        tables.is_vectorized(),
        "the pinned artifact carries dot tables; without them this test would \
         only exercise the scalar fallback against itself"
    );
    let residual = !art.resid_cb.is_empty();

    let rot = compiler::derive_rotations();
    // 512 positions, matching the κ witness's sample. Enough to produce
    // hundreds of distinct codes; small enough that a debug-profile CI run
    // does not pay a minute for it.
    let sample = corpus.n.min(512);
    let stride = (corpus.n / sample).max(1);
    let mut distinct = std::collections::BTreeSet::new();
    for step in 0..sample {
        let position = step * stride;
        if position >= corpus.n {
            break;
        }
        let bundle = runtime::bundle_plain(&art, &rot, &corpus, position);
        let scalar = runtime::assign_code_for_bundle(&art, &bundle);
        assert_eq!(
            scalar,
            runtime::assign_code_for_bundle_with(&tables, &art, &bundle),
            "prepared/scalar divergence at corpus position {position}"
        );
        // The batch entry point the corpus pass actually calls, against the
        // beam form it replaced.
        assert_eq!(
            runtime::code_plain(&art, &rot, &corpus, position),
            runtime::code_plain_with(&tables, &art, &rot, &corpus, position),
            "code_plain divergence at corpus position {position}"
        );
        distinct.insert(scalar);
    }
    assert!(
        distinct.len() > 1,
        "every sampled position produced the same code; the fixture is not \
         exercising the scan"
    );
    println!(
        "pinned artifact ({}): {sample} corpus positions, {} distinct codes, all forms agree",
        if residual {
            "TLA7 residual"
        } else {
            "TLA6 dot"
        },
        distinct.len()
    );
}

/// Records which dispatch this run witnessed, so a green result is not read
/// as coverage of a kernel the machine never executed.
#[test]
fn prepared_reports_vectorization() {
    let art = synthetic_dot_art();
    assert!(runtime::AssignTables::new(&art).is_vectorized());
    println!(
        "arch = {}, prepared dot tables decoded; kernel dispatch is chosen at runtime",
        std::env::consts::ARCH
    );
}
