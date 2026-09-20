//! Corrected small learning gate for the prior-only residual.
//!
//! Claims an exclusive report root before touching a model, runs the authored capacity witness and one
//! learning arm on the balanced fixture, records the curve, applies the frozen gate, and seals the
//! attempt. A separated research gate result, not a unit test.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use uor_r4_core::native_geometric::learner::prior_learning::{
    authored_witness, bias_codes_from_counts, fixture_mask, Config, PriorCore, PriorTrainer,
};
use uor_r4_core::report_output::{claim, seal, verify};

const VOCAB: usize = 4;
const DV: usize = 32;
const SEED: u64 = 13;
const STEPS: [usize; 4] = [0, 128, 512, 2000];
/// Frozen before any result was seen.
const ACCURACY_BAR: f64 = 0.9;
const CE_MARGIN_BITS: f64 = 0.10;
const CONSTANT_CE_BITS: f64 = 2.0; // log2(4): the uniform predictor on four classes

struct Fixture {
    /// `(x, y, (x+y) % 4)` for all 16 ordered pairs.
    seqs: Vec<Vec<u32>>,
}

impl Fixture {
    fn build() -> Self {
        let mut seqs = Vec::new();
        for x in 0..VOCAB as u32 {
            for y in 0..VOCAB as u32 {
                seqs.push(vec![x, y, (x + y) % 4]);
            }
        }
        Self { seqs }
    }

    fn mask(&self) -> Vec<bool> {
        fixture_mask(3)
    }

    /// Accuracy and mean CE on the scored position only.
    fn score(&self, core: &PriorCore, pairs: &[(u32, u32, u32)]) -> (f64, f64) {
        self.score_with(core, pairs, true)
    }

    /// The same, with the contextual residual optionally disabled (the constant control).
    fn score_with(
        &self,
        core: &PriorCore,
        pairs: &[(u32, u32, u32)],
        use_context: bool,
    ) -> (f64, f64) {
        let mut hit = 0usize;
        let mut bits = 0f64;
        for &(x, y, s) in pairs {
            let z = core.int_logits(x as usize, y as usize, use_context);
            let mut best = 0usize;
            for (r, &v) in z.iter().enumerate() {
                if v > z[best] {
                    best = r;
                }
            }
            if best as u32 == s {
                hit += 1;
            }
            bits += core.bits_one(&z, s);
        }
        (hit as f64 / pairs.len() as f64, bits / pairs.len() as f64)
    }
}

/// Zero one position table before the shared nonlinearity, keeping every other parameter fixed.
fn knockout(core: &PriorCore, which: &str) -> PriorCore {
    let mut c = core.clone();
    let zeros = |rows: usize, cols: usize| vec![0f32; rows * cols];
    match which {
        "prev" => {
            c.e_old = uor_r4_core::native_geometric::learner::TernaryLinear::quantize(
                &zeros(VOCAB + 1, DV),
                VOCAB + 1,
                DV,
            )
        }
        "cur" => {
            c.e_new = uor_r4_core::native_geometric::learner::TernaryLinear::quantize(
                &zeros(VOCAB, DV),
                VOCAB,
                DV,
            )
        }
        _ => unreachable!(),
    };
    c
}

/// Permute the whole context -> target association, preserving the context marginals and the exact
/// multiset of targets. Returns the permuted pairs and whether the association actually changed.
fn permuted_pairs(seed: u64) -> (Vec<(u32, u32, u32)>, bool) {
    let mut targets: Vec<u32> = (0..16).map(|i| ((i / 4 + i % 4) % 4) as u32).collect();
    let mut st = seed | 1;
    for i in (1..targets.len()).rev() {
        st ^= st << 13;
        st ^= st >> 7;
        st ^= st << 17;
        let j = (st as usize) % (i + 1);
        targets.swap(i, j);
    }
    let mut pairs = Vec::new();
    let mut changed = false;
    for (i, t) in targets.iter().enumerate() {
        let x = (i / 4) as u32;
        let y = (i % 4) as u32;
        let original = (x + y) % 4;
        if *t != original {
            changed = true;
        }
        pairs.push((x, y, *t));
    }
    (pairs, changed)
}

fn main() -> ExitCode {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| ".uor-models/prior-learning-gate-2026-09-20/attempt-1".to_string()),
    );
    // Exclusively claim the report root before any model work.
    if let Err(e) = claim(&root) {
        eprintln!("error: claim {}: {e}", root.display());
        return ExitCode::from(1);
    }

    let fx = Fixture::build();
    let mask = fx.mask();
    println!("=== corrected small learning gate (prior-only) ===");
    println!("config V={VOCAB} dv={DV} seed={SEED} scored targets = final position only");
    println!(
        "gate frozen before results: accuracy >= {ACCURACY_BAR}, CE >= {CE_MARGIN_BITS} bit below {CONSTANT_CE_BITS}, positive permutation penalty"
    );

    // --- authored capacity witness (separate from every learned arm) --------
    let witness = match authored_witness(VOCAB, DV) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("error: witness: {e}");
            return ExitCode::from(1);
        }
    };
    let all: Vec<(u32, u32, u32)> = (0..16)
        .map(|i| {
            let x = (i / 4) as u32;
            let y = (i % 4) as u32;
            (x, y, (x + y) % 4)
        })
        .collect();
    let (w_acc, w_ce) = fx.score(&witness, &all);
    let expected_ce = (1.0 + 3.0 * (-4.0f64).exp()).log2();
    println!(
        "authored witness: accuracy {w_acc:.4} (16/16 expected) mean CE {w_ce:.4} bits (analytic {expected_ce:.4})"
    );
    let bytes = witness.to_bytes(&[0u8; 32], 0);
    let round = PriorCore::from_bytes(&bytes).expect("witness reload");
    let (r_acc, _) = fx.score(&round, &all);
    println!("witness export/reload accuracy {r_acc:.4}");
    if w_acc < 1.0 || r_acc < 1.0 || (w_ce - expected_ce).abs() > 1e-9 {
        eprintln!("error: the capacity witness does not represent the task");
        return ExitCode::from(1);
    }

    // --- one learning arm ---------------------------------------------------
    let mut counts = vec![1u64; VOCAB];
    let mut total = 0u64;
    for s in &fx.seqs {
        for t in s.iter().skip(1) {
            counts[*t as usize] += 1;
            total += 1;
        }
    }
    let bias = bias_codes_from_counts(&counts, total, VOCAB);
    let cfg = Config::new(VOCAB, DV);
    let (f_bits, norm_bits, bias_scale_bits) = (cfg.f_bits, cfg.norm_bits, cfg.bias_scale_bits);
    let mut t = match PriorTrainer::new(cfg, SEED, bias, 0.45) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: trainer: {e}");
            return ExitCode::from(1);
        }
    };

    let mut curve = Vec::new();
    let mut snapshots = Vec::new();
    for target in STEPS {
        while t.step < target as u64 {
            let batch: Vec<Vec<u32>> = fx.seqs.clone();
            t.train_batch_masked(&batch, Some(&mask));
        }
        let core = t.to_core().expect("core");
        let (acc, ce) = fx.score(&core, &all);
        let (c_acc, c_ce) = fx.score_with(&core, &all, false);
        let knockout_prev = fx.score(&knockout(&core, "prev"), &all);
        let knockout_cur = fx.score(&knockout(&core, "cur"), &all);
        let nonzero_codes = core.w_o.packed().iter().filter(|b| **b != 0).count();
        println!(
            "step {target:>4}: acc {acc:.4} CE {ce:.4} | context-disabled acc {c_acc:.4} CE {c_ce:.4} | knockouts prev {:.4} cur {:.4} | w_o nonzero bytes {nonzero_codes}",
            knockout_prev.0, knockout_cur.0
        );
        curve.push(serde_json::json!({
            "step": target,
            "accuracy": acc,
            "ce_bits": ce,
            "context_disabled_accuracy": c_acc,
            "context_disabled_ce_bits": c_ce,
            "knockout_prev_accuracy": knockout_prev.0,
            "knockout_prev_ce_bits": knockout_prev.1,
            "knockout_cur_accuracy": knockout_cur.0,
            "knockout_cur_ce_bits": knockout_cur.1,
            "w_o_nonzero_code_bytes": nonzero_codes,
        }));
        snapshots.push((target, core));
    }

    let final_core = &snapshots.last().unwrap().1;
    let (final_acc, final_ce) = fx.score(final_core, &all);

    // --- permutation control on the frozen final artifact -------------------
    let (perm_pairs, changed) = permuted_pairs(0xA5A5_1234);
    let (p_acc, p_ce) = fx.score(final_core, &perm_pairs);
    let penalty = p_ce - final_ce;
    println!(
        "permuted context->target: accuracy {p_acc:.4} CE {p_ce:.4} | penalty {penalty:+.4} bits | association actually changed: {changed}"
    );

    let gate_accuracy = final_acc >= ACCURACY_BAR;
    let gate_ce = final_ce <= CONSTANT_CE_BITS - CE_MARGIN_BITS;
    let gate_perm = changed && penalty > 0.0;
    println!(
        "GATE: accuracy {} | CE margin {} | permutation penalty {}",
        if gate_accuracy { "PASS" } else { "FAIL" },
        if gate_ce { "PASS" } else { "FAIL" },
        if gate_perm { "PASS" } else { "FAIL" },
    );
    let passed = gate_accuracy && gate_ce && gate_perm;
    println!(
        "OVERALL: {}",
        if passed {
            "PASS — the exported low-bit prior learns the balanced fixture"
        } else {
            "FAIL — the corrected gate does not pass; the floating comparator is the next step"
        }
    );

    // --- persist, seal, verify ---------------------------------------------
    let result = serde_json::json!({
        "schema": "uor-r4.prior-learning-gate/1",
        "config": {"vocab": VOCAB, "dv": DV, "seed": SEED, "nf": f_bits, "norm_bits": norm_bits, "bias_scale_bits": bias_scale_bits},
        "gate_frozen": {"accuracy_bar": ACCURACY_BAR, "ce_margin_bits": CE_MARGIN_BITS, "constant_ce_bits": CONSTANT_CE_BITS},
        "witness": {"accuracy": w_acc, "ce_bits": w_ce, "analytic_ce_bits": expected_ce, "reload_accuracy": r_acc},
        "final": {"accuracy": final_acc, "ce_bits": final_ce, "permuted_accuracy": p_acc, "permuted_ce_bits": p_ce, "permutation_penalty_bits": penalty, "permutation_changed_association": changed},
        "gates": {"accuracy": gate_accuracy, "ce_margin": gate_ce, "permutation_penalty": gate_perm},
        "passed": passed,
    });
    let write = |name: &str, body: String| -> Result<(), String> {
        std::fs::write(root.join(name), body).map_err(|e| format!("{name}: {e}"))
    };
    if let Err(e) = write(
        "result.json",
        serde_json::to_string_pretty(&result).unwrap(),
    ) {
        eprintln!("error: {e}");
        return ExitCode::from(1);
    }
    if let Err(e) = write("curve.json", serde_json::to_string_pretty(&curve).unwrap()) {
        eprintln!("error: {e}");
        return ExitCode::from(1);
    }
    let preds: Vec<serde_json::Value> = snapshots
        .iter()
        .map(|(step, core)| {
            let all16: Vec<serde_json::Value> = all
                .iter()
                .map(|&(x, y, s)| {
                    let z = core.int_logits(x as usize, y as usize, true);
                    let mut best = 0usize;
                    for (r, &v) in z.iter().enumerate() {
                        if v > z[best] {
                            best = r;
                        }
                    }
                    serde_json::json!({"x": x, "y": y, "target": s, "predicted": best, "correct": best as u32 == s, "z": z})
                })
                .collect();
            serde_json::json!({"step": step, "predictions": all16})
        })
        .collect();
    if let Err(e) = write(
        "predictions.json",
        serde_json::to_string_pretty(&preds).unwrap(),
    ) {
        eprintln!("error: {e}");
        return ExitCode::from(1);
    }
    // The final serving artifact, saved under its own report root.
    let artifact = final_core.to_bytes(&[0u8; 32], SEED);
    let artifact_path = root.join("prior_only.cpl2");
    if let Err(e) = std::fs::write(&artifact_path, &artifact) {
        eprintln!("error: artifact: {e}");
        return ExitCode::from(1);
    }
    let ckpt_path = root.join("prior_only.ckpt");
    let ckpt = t.checkpoint_bytes();
    if let Err(e) = std::fs::write(&ckpt_path, &ckpt) {
        eprintln!("error: checkpoint: {e}");
        return ExitCode::from(1);
    }
    println!(
        "artifact {} bytes | checkpoint {} bytes",
        artifact.len(),
        ckpt.len()
    );

    match seal(&root) {
        Ok(m) => println!("sealed {}", m.display()),
        Err(e) => {
            eprintln!("error: seal: {e}");
            return ExitCode::from(1);
        }
    }
    match verify(&root) {
        Ok(unlisted) => println!("verified sealed attempt ({} unlisted)", unlisted.len()),
        Err(e) => {
            eprintln!("error: verify: {e}");
            return ExitCode::from(1);
        }
    }
    if passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}
