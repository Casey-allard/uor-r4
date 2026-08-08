//! #456 null arm — the reconstructability certificate's K-2 mutation guard.
//!
//! PR #462 landed the EXCT-disabled reconstruction metric per cover-sweep point
//! and its harvest confirmed the metric *discriminates* between covers (2.36
//! bits/token between configurations the with-EXCT agreement metric calls
//! identical). The one exit criterion left open was the pre-registered null arm:
//! **shuffled emission tables must score at the unigram floor**. A certificate
//! that still beats the floor on corrupted structure is measuring an artifact,
//! not reconstructability.
//!
//! This harness runs [`cover_sweep::reconstruction_null`] on the default cover
//! point: it scores the EXCT-disabled reconstruction twice on the same held-out
//! slice — real emission tables, then the same tables with every region's ΔE
//! list deranged (root prior untouched) — and checks that the shape-matched
//! shuffle DEGRADES the reconstruction (the mutation guard), so the metric is
//! reflecting region structure rather than byte-shape.
//!
//! It also records — and the measurement was a surprise worth keeping — that the
//! EXCT-disabled graph reconstruction is itself BELOW the unigram floor (16.3
//! bits / 1.5% top-1 vs 8.7 bits / 6.4% top-1): the graph residuals applied
//! without exact-context gating are net-worse than a trivial unigram prior. The
//! pre-registered "shuffled must score at the unigram floor" wording assumed the
//! reconstruction beats the floor; it does not, so that comparison is recorded
//! rather than asserted. The consequence for #456 item 3 (recon bits as a
//! compiler split criterion) is a NEGATIVE: the metric discriminates among covers
//! that are all sub-unigram.
//!
//! `#[ignore]`d (needs the pinned fixtures + two Gate C passes). Run:
//!   R4_RECON_NULL_HELD=20000 \
//!   cargo test --release -p uor-r4-graph-cli --test reconstruction_null -- --ignored --nocapture
//! `R4_RECON_NULL_HELD` caps the held-out slice (default 20000) to keep the two
//! passes cheap; the verdict is scale-robust because both arms and the floor use
//! the same slice.

use std::path::PathBuf;

use uor_r4_graph_certify::ScoreConfig;
use uor_r4_graph_cli::cover_sweep::{load_inputs, reconstruction_null, sweep_grid};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../uor-r4-core/tests/fixtures")
        .join(name)
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[test]
#[ignore = "#456 null arm; needs fixtures + two Gate C passes — run with --ignored"]
fn reconstruction_certificate_collapses_on_shuffled_emissions() {
    let meta = fixture("c_meta.bin");
    let recs = fixture("c_recs.bin");
    let art = fixture("tless_artifacts.bin");
    if !meta.exists() || !recs.exists() || !art.exists() {
        eprintln!("reconstruction_null: fixtures absent, skipping (κ-test convention)");
        return;
    }

    let inputs = load_inputs(&meta, &recs, &art).expect("load sweep inputs");
    let point = sweep_grid()
        .into_iter()
        .find(|p| p.baseline)
        .expect("the grid carries the default (baseline) point");
    let config = ScoreConfig::default();
    let held_cap = env_usize("R4_RECON_NULL_HELD", 20_000);

    let rn = reconstruction_null(&inputs, &point, &config, held_cap, 0x00C0_FFEE_1234)
        .expect("reconstruction null arm");

    println!(
        "#456 null arm — point {} on {} held-out positions",
        rn.label, rn.held_out_scored
    );
    println!(
        "  real emissions  : top1 {:.4}  bits {:.4}",
        rn.real.top1_agreement, rn.real.bits_per_token
    );
    println!(
        "  deranged (null) : top1 {:.4}  bits {:.4}",
        rn.null.top1_agreement, rn.null.bits_per_token
    );
    println!(
        "  unigram floor   : top1 {:.4}  bits {:.4}",
        rn.unigram_top1, rn.unigram_bits
    );
    println!(
        "  real vs floor   : {:+.4} bits (POSITIVE = graph reconstruction is WORSE than a unigram prior — the sub-unigram finding)",
        rn.real.bits_per_token - rn.unigram_bits
    );
    println!(
        "  null vs floor   : {:+.4} bits (deranging pushes it further below the floor still)",
        rn.null.bits_per_token - rn.unigram_bits
    );

    // Anti-vacuity: the arms actually ran on a non-empty slice.
    assert!(rn.held_out_scored > 0, "held-out slice must be non-empty");

    // THE NULL ARM (K-2 mutation guard). The pre-registered wording was
    // "shuffled emission tables must score at the unigram floor", which assumed
    // the reconstruction beats the floor and that deranging would drop it back
    // down to it. The measurement refutes that assumption — see the recorded
    // sub-unigram finding below — so the operative guard is the one that
    // survives it: a shape-matched shuffle of the emission tables must DEGRADE
    // the reconstruction, never preserve or improve it. If deranging left the
    // score unchanged (or better), the certificate would be measuring the
    // cover's byte-shape, not its residual structure.
    let margin_bits = 0.5;
    assert!(
        rn.real.bits_per_token + margin_bits < rn.null.bits_per_token,
        "deranging the emission tables did not degrade the reconstruction by the \
         required margin: real {:.4} bits vs deranged {:.4} bits (need real + {margin_bits} \
         < deranged) — the metric is not reflecting region structure",
        rn.real.bits_per_token,
        rn.null.bits_per_token
    );
    assert!(
        rn.real.top1_agreement >= rn.null.top1_agreement,
        "deranging IMPROVED top-1 ({:.4} deranged vs {:.4} real) — a shuffle must not \
         help; the certificate is measuring an artifact",
        rn.null.top1_agreement,
        rn.real.top1_agreement
    );

    // RECORDED FINDING (not a pass/fail — a measured property). The EXCT-disabled
    // graph reconstruction is BELOW the unigram floor: the graph residuals,
    // applied without exact-context gating, are net-worse than a trivial unigram
    // prior (matches the repo's "graph-resolved ~1% top-1 / 16.3 bits" record).
    // The recon-bits metric therefore discriminates among covers that are ALL
    // sub-unigram — a negative for promoting it to a compiler split/stop
    // criterion (issue #456 item 3). Asserted only as a tripwire: if a future
    // change makes the graph reconstruction beat the unigram floor, this fires so
    // the finding gets revisited (a very welcome failure).
    assert!(
        rn.real.bits_per_token > rn.unigram_bits,
        "the graph reconstruction now BEATS the unigram floor (real {:.4} < floor {:.4}) — \
         the #456 sub-unigram finding no longer holds; revisit item 3",
        rn.real.bits_per_token,
        rn.unigram_bits
    );
}
