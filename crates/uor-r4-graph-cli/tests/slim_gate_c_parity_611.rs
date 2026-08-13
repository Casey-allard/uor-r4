//! #611 parity arm — the slim sweep Gate C evaluator must match the full
//! evaluator on every metric the sweep and its null-arm harness consume.
//!
//! #611 adds `evaluate_gate_c_sweep`, which computes only Rule 1, Rule 1+2,
//! the TLA3 baseline, and the analytic unigram nulls — two scorers and one
//! lean pass instead of the full evaluator's ~thirty arms. Because it calls
//! the SAME per-position primitives as the full evaluator (same code
//! derivation, same scorers, same `outcome_bits` / `predict_witness_plain` /
//! `witten_bell_probability` / unigram-null derivation / `gate_c_sample`), its
//! numbers must equal the full `evaluate_gate_c`'s within the deterministic
//! (same-machine, byte-exact) floating-point contract.
//!
//! This harness builds the real scored artifact for the default cover point
//! via `run_point` (whose returned bytes it reuses, so both evaluators score
//! the identical artifact), then:
//!   1. asserts the sweep row's metrics (produced through the slim evaluator)
//!      equal the full evaluator's rule1/rule12/baseline exactly, and
//!   2. asserts `evaluate_gate_c_sweep` matches the full evaluator on those
//!      three metrics AND on the analytic nulls the null-arm harness reads.
//!
//! `#[ignore]`d and presence-gated: it needs the pinned fixtures and runs BOTH
//! Gate C paths. Run:
//!   R4_GATE_C_SAMPLE=200 \
//!   cargo test --release -p uor-r4-graph-cli --test slim_gate_c_parity_611 \
//!     -- --ignored --nocapture

use std::path::PathBuf;
use std::time::Instant;

use uor_r4_graph_certify::{ScoreConfig, evaluate_gate_c, evaluate_gate_c_sweep};
use uor_r4_graph_cli::cover_sweep::{PreparedScoring, load_inputs, run_point, sweep_grid};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uor-r4-core/tests/fixtures")
        .join(name)
}

#[test]
#[ignore = "#611 parity arm; needs fixtures + full+slim Gate C — run with --ignored"]
fn slim_gate_c_matches_full_on_consumed_metrics() {
    let meta = fixture("c_meta.bin");
    let recs = fixture("c_recs.bin");
    let art = fixture("tless_artifacts.bin");
    if !meta.exists() || !recs.exists() || !art.exists() {
        eprintln!("slim_gate_c_parity_611: fixtures absent, skipping (κ-test convention)");
        return;
    }

    let inputs = load_inputs(&meta, &recs, &art).expect("load sweep inputs");
    let config = ScoreConfig::default();
    let prepared = PreparedScoring::prepare(&inputs, &config);

    let point = sweep_grid()
        .into_iter()
        .find(|p| p.baseline)
        .expect("the grid carries the default (baseline) point");

    // run_point now scores via the slim evaluator; reuse the exact artifact
    // bytes it produced so the full evaluator scores the identical artifact.
    let (row, baseline_metrics, artifact_bytes, _timing) =
        run_point(&inputs, &point, &config, &prepared).expect("run_point");

    let full_start = Instant::now();
    let full = evaluate_gate_c(
        &artifact_bytes,
        &inputs.artifact_container,
        &inputs.artifacts,
        &inputs.store,
        &inputs.corpus,
        &inputs.held_out,
        &config,
    )
    .expect("full Gate C");
    let full_ms = full_start.elapsed().as_millis();

    // (1) The sweep row's slim-derived metrics equal the full evaluator's.
    assert_eq!(
        row.gate_c_rule12, full.rule12_precedence,
        "Rule 1+2 agreement diverged between the slim and full evaluators"
    );
    assert_eq!(
        row.reconstruction, full.rule1_chain,
        "Rule 1 reconstruction diverged between the slim and full evaluators"
    );
    assert_eq!(
        baseline_metrics, full.tla3_baseline,
        "TLA3 baseline diverged between the slim and full evaluators"
    );

    // (2) The slim evaluator called directly matches the full evaluator on the
    // three metrics AND the analytic nulls the #456 null-arm harness reads.
    let slim_start = Instant::now();
    let slim = evaluate_gate_c_sweep(
        &artifact_bytes,
        &inputs.artifact_container,
        &inputs.artifacts,
        &inputs.store,
        &inputs.corpus,
        &inputs.held_out,
        &config,
    )
    .expect("slim Gate C");
    let slim_ms = slim_start.elapsed().as_millis();

    assert_eq!(slim.rule1_chain, full.rule1_chain);
    assert_eq!(slim.rule12_precedence, full.rule12_precedence);
    assert_eq!(slim.tla3_baseline, full.tla3_baseline);
    assert_eq!(slim.positions_sampled, full.positions_sampled);
    assert_eq!(slim.held_out_population, full.held_out_population);

    // Analytic nulls (GateCNulls is not PartialEq — compare fields).
    assert_eq!(
        slim.nulls.unigram_train_argmax,
        full.nulls.unigram_train_argmax
    );
    assert_eq!(
        slim.nulls.unigram_null_top1_all,
        full.nulls.unigram_null_top1_all
    );
    assert_eq!(
        slim.nulls.unigram_null_bits_all,
        full.nulls.unigram_null_bits_all
    );
    assert_eq!(
        slim.nulls.unigram_null_top1_generalization,
        full.nulls.unigram_null_top1_generalization
    );
    assert_eq!(
        slim.nulls.unigram_null_bits_generalization,
        full.nulls.unigram_null_bits_generalization
    );
    assert_eq!(slim.nulls.train_positions, full.nulls.train_positions);
    assert_eq!(slim.nulls.held_out_positions, full.nulls.held_out_positions);

    eprintln!(
        "#611 parity — point {} on {} held-out ({} scored): \
         rule1 {:.6}/{:.4}bpt  rule12 {:.6}/{:.4}bpt  baseline {:.6}/{:.4}bpt — slim == full",
        row.label,
        full.rule12_precedence.positions,
        if full.positions_sampled == 0 {
            full.rule12_precedence.positions
        } else {
            full.positions_sampled
        },
        slim.rule1_chain.top1_agreement,
        slim.rule1_chain.bits_per_token,
        slim.rule12_precedence.top1_agreement,
        slim.rule12_precedence.bits_per_token,
        slim.tla3_baseline.top1_agreement,
        slim.tla3_baseline.bits_per_token,
    );
    eprintln!(
        "#611 benchmark — full Gate C {full_ms} ms vs slim {slim_ms} ms on the same artifact \
         ({:.1}x faster; the slim path builds 2 scorers not 7 and skips the whole-corpus \
         forward-anchor/right-context/two-sided/latent passes)",
        if slim_ms == 0 {
            f64::from(u32::try_from(full_ms).unwrap_or(u32::MAX))
        } else {
            full_ms as f64 / slim_ms as f64
        }
    );
}
