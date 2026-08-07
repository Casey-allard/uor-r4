//! Codebook training-set size vs graded-code quality (issue #460 lever 2).
//!
//! ## Front
//!
//! `RVQ_SAMPLE_CAP = 10_000` (`compiler.rs`) hard-caps the context codebook's
//! k-means training set at 10,000 vectors **independent of corpus size**.
//! `sampled_kmeans_rvq` subsamples its input to that cap and the caller
//! discards the full-set codes, so at the 500k fixture corpus the codebook
//! that produces every graded class code is fit on 2.5% of the construction
//! split — 39 vectors per centroid, per stage.
//!
//! Two facts make this worth a measurement rather than an assumption.
//!
//! First, `docs/capacity_scaling_432.md` §3.1 argues that issue #407's
//! `CTX_SAMPLE` 6,000 → 50,000 raise "did not change the codebook's
//! training-set size at all", so the +0.5–0.7pp #407 attributed to sample
//! starvation was never delivered. **That claim is one step too strong, and
//! the arithmetic matters here.** The training set is
//! `min(CTX_SAMPLE, RVQ_SAMPLE_CAP)`. At `CTX_SAMPLE = 6_000` the 10,000 cap
//! does not bind, so #407's raise moved the training set 6,000 → 10,000 — a
//! real increase, just a small one. What cannot help is raising the pool
//! *above* the cap, which is every configuration since. Sweep A below tests
//! exactly that regime.
//!
//! Second, §6 records `code.codebook_sample_fraction` as already SATURATED at
//! 500k, meaning the measurements the rest of the programme is calibrated
//! against were themselves taken on an under-fit codebook. How much that
//! costs has never been measured, and it bounds how much any recalibration
//! could recover.
//!
//! ## Arms
//!
//! The k-means training set is `min(CTX_SAMPLE, RVQ_SAMPLE_CAP)`, so the two
//! knobs have to be swept together to say anything. Two sweeps:
//!
//! **Sweep A — does `CTX_SAMPLE` alone do anything?** `RVQ_SAMPLE_CAP` pinned
//! at the shipped 10,000 while `CTX_SAMPLE` ∈ {10k, 50k, 200k}. This is the
//! direct adjudication of §3.1's claim, and of #407's attribution: #407 raised
//! `CTX_SAMPLE` 6k → 50k and credited +0.5–0.7pp to sample starvation, while
//! §3.1 argues that raise could not have reached the codebook at all. One of
//! those is wrong and this sweep says which. Prediction under §3.1: flat.
//!
//! **Sweep B — does the training set itself matter?** `CTX_SAMPLE` and
//! `RVQ_SAMPLE_CAP` moved together ∈ {10k, 50k, 200k}, so the training set is
//! genuinely 10k / 50k / 200k vectors.
//!
//! Every arm is a full teacher-free `compile_recorded` of the pinned fixture
//! corpus followed by a construction-only store build and a held-out top-1
//! read. Same corpus, same split, same `K`, same `STAGES`, same nominal key
//! space, same cover throughout — only the codebook moves.
//!
//! The shipped configuration (`CTX_SAMPLE = 50_000`, `RVQ_SAMPLE_CAP =
//! 10_000`) is one of the sweep-A arms, so the baseline every delta is quoted
//! against is the deployed one rather than a nearby stand-in.
//!
//! ## Metric
//!
//! Held-out store top-1: the fraction of held-out records whose
//! `predict_witness_plain` token equals the recorded next token. This is the
//! "store baseline" row #460 reported moving 26.4 → 25.4 under STAGES=5, so
//! the two experiments are directly comparable.
//!
//! ## Validity gate (hard)
//!
//! Every arm must produce its own codebook κ. Two arms sharing a codebook
//! would mean a knob is not reaching `sampled_kmeans_rvq`, and the comparison
//! between them would be vacuous rather than null — the failure mode that
//! made #407's attribution wrong in the first place.
//!
//! ## Exit rule (pre-declared)
//!
//! Positive if held-out top-1 at the largest training set exceeds the SHIPPED
//! configuration by ≥ 1.0pp.
//!
//! Deltas are compared against the standard error of the *difference*, not of
//! a single arm. The arms share the held-out set, so the independent-sample
//! form used here overstates the error; it is the conservative choice.
//!
//! ## Why this is not a re-run of the STAGES=5 negative
//!
//! Both mechanisms are expected to lower records-per-key, for opposite
//! reasons. STAGES=5 lowered it by adding key *resolution*, and top-1 fell —
//! read as thinner evidence per key. A better-fit codebook lowers it by
//! putting evidence on the *right* key. So occupancy-up-and-top-1-up
//! separates fit from resolution and confirms the evidence-quality thesis on
//! an independent lever; occupancy-up-and-top-1-down means records-per-key is
//! the binding quantity whatever the cause, which is a stronger and more
//! general negative than #460's and would mean the STAGES=5 result was never
//! about subdivision specifically.
//!
//! ## Cost and κ
//!
//! Five recorded compiles plus five store builds on 500k records, about 30
//! minutes on two cores; no teacher, no Gate C, no checkpoint. Adopting a
//! different cap is κ-affecting and would need the #407 re-pin ceremony —
//! this harness only measures, and writes nothing.

use std::collections::BTreeSet;

use uor_r4_core::transformerless::{compiler, runtime};

/// `(ctx_sample, rvq_cap, label)`. The k-means training set is the min of the
/// two. Sweep A pins the cap and moves the pool; sweep B moves both.
const ARMS: [(usize, usize, &str); 5] = [
    (10_000, 10_000, "A: pool 10k, cap 10k"),
    (50_000, 10_000, "A: pool 50k, cap 10k  <- SHIPPED"),
    (200_000, 10_000, "A: pool 200k, cap 10k"),
    (50_000, 50_000, "B: pool 50k, cap 50k"),
    (200_000, 200_000, "B: pool 200k, cap 200k"),
];
/// Index of the shipped configuration in `ARMS` — the baseline for deltas.
const SHIPPED: usize = 1;
/// Pre-declared: positive if the largest training set beats shipped by this.
const EXIT_RULE_PP: f64 = 1.0;

fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

struct Arm {
    label: &'static str,
    ctx_sample: usize,
    cap: usize,
    training_set: usize,
    occupied_keys: usize,
    records_per_key: f64,
    top1: f64,
    standard_error: f64,
    held_out: usize,
    codebook_kappa: String,
}

/// One arm: compile at `(ctx_sample, cap)`, build the construction-only
/// store, read held-out top-1.
fn run_arm(
    corpus: &compiler::Corpus,
    vocab: usize,
    (ctx_sample, cap, label): (usize, usize, &'static str),
) -> Arm {
    // Both knobs are read through `capacity_override_usize`, so these env
    // vars are the only thing that differs between arms.
    std::env::set_var("R4_CTX_SAMPLE", ctx_sample.to_string());
    std::env::set_var("R4_RVQ_SAMPLE_CAP", cap.to_string());

    let art = compiler::compile_recorded(corpus, vocab).expect("recorded compile");
    let (store, codes) = runtime::build_store(&art, corpus);

    // Occupancy over the CONSTRUCTION split only: held-out records do not
    // contribute evidence, so counting their codes would inflate occupancy
    // without any store consequence.
    let cut = compiler::train_cut(corpus);
    let mut distinct = BTreeSet::new();
    let mut construction = 0usize;
    for (i, code) in codes.iter().enumerate() {
        if corpus.story[i] >= cut {
            continue;
        }
        construction += 1;
        distinct.insert(*code);
    }

    let mut correct = 0usize;
    let mut held_out = 0usize;
    for (i, code) in codes.iter().enumerate() {
        if corpus.story[i] < cut {
            continue;
        }
        held_out += 1;
        if runtime::predict_witness_plain(&store, code).token == corpus.next[i] {
            correct += 1;
        }
    }

    let top1 = correct as f64 / held_out as f64 * 100.0;
    Arm {
        label,
        ctx_sample,
        cap,
        training_set: ctx_sample.min(cap),
        occupied_keys: distinct.len(),
        records_per_key: construction as f64 / distinct.len() as f64,
        top1,
        standard_error: (top1 / 100.0 * (1.0 - top1 / 100.0) / held_out as f64).sqrt() * 100.0,
        held_out,
        // The codebook κ distinguishes arms: if two caps produce the same
        // codebook the comparison between them is vacuous.
        codebook_kappa: codebook_kappa(&art),
    }
}

/// Content digest of the artifact's context codebooks — the quantity the arms
/// are supposed to be changing.
fn codebook_kappa(art: &compiler::Compiled) -> String {
    let mut hasher = blake3::Hasher::new();
    for stage in &art.dot_cb {
        for entry in stage {
            hasher.update(&entry.to_le_bytes());
        }
    }
    hasher.finalize().to_hex()[..16].to_owned()
}

#[test]
#[ignore = "measurement harness (issue #460 lever 2); run explicitly with --ignored"]
fn codebook_fit_sweep() {
    let meta = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let Some(corpus) = compiler::load_corpus_from(&meta, &recs) else {
        println!("SKIP: corpus fixtures absent ({meta} + {recs}); vacuous green");
        return;
    };
    let vocab = corpus
        .next
        .iter()
        .chain(corpus.top_tokens.iter().flatten())
        .copied()
        .max()
        .unwrap_or(0) as usize
        + 1;
    let cut = compiler::train_cut(&corpus);
    let construction = corpus.story.iter().filter(|&&s| s < cut).count();
    println!(
        "codebook fit sweep (#460 lever 2): {} records, {} stories, \
         cut at story {cut} => {construction} construction / {} held out; vocab {vocab}",
        corpus.n,
        corpus.stories,
        corpus.n - construction
    );
    println!("K = {}, STAGES = {}", compiler::K, compiler::STAGES);

    let mut arms = Vec::new();
    for spec in ARMS {
        let training = spec.0.min(spec.1);
        println!(
            "\n-- {} : training set {training} ({:.2}% of the construction split, \
             {:.0} vectors per centroid per stage) --",
            spec.2,
            training as f64 / construction as f64 * 100.0,
            training as f64 / compiler::K as f64
        );
        let arm = run_arm(&corpus, vocab, spec);
        println!(
            "   codebook κ {}  occupied keys {}  records/key {:.2}  \
             held-out top-1 {:.2}% ± {:.2}pp (n={})",
            arm.codebook_kappa,
            arm.occupied_keys,
            arm.records_per_key,
            arm.top1,
            arm.standard_error,
            arm.held_out
        );
        arms.push(arm);
    }

    let shipped = &arms[SHIPPED];
    println!("\n==== summary (deltas vs the SHIPPED configuration) ====");
    println!("arm\t\t\t\tpool\tcap\ttrain\tocc.keys\trec/key\ttop-1\t±\tvs shipped");
    for arm in &arms {
        println!(
            "{:<30}\t{}\t{}\t{}\t{}\t\t{:.2}\t{:.2}%\t{:.2}\t{:+.2}pp",
            arm.label,
            arm.ctx_sample,
            arm.cap,
            arm.training_set,
            arm.occupied_keys,
            arm.records_per_key,
            arm.top1,
            arm.standard_error,
            arm.top1 - shipped.top1
        );
    }

    // --- validity: the arms must actually be different compiles ---
    let distinct: BTreeSet<&str> = arms.iter().map(|a| a.codebook_kappa.as_str()).collect();
    println!("\n==== validity ====");
    println!(
        "distinct codebooks across {} arms: {} {}",
        arms.len(),
        distinct.len(),
        if distinct.len() == arms.len() {
            "PASS"
        } else {
            "VOID — arms share a codebook, so some comparison measures nothing"
        }
    );
    assert_eq!(
        distinct.len(),
        arms.len(),
        "each arm must produce its own codebook; if two coincide the knobs are \
         not reaching `sampled_kmeans_rvq` and those numbers are void"
    );

    // --- sweep A: does CTX_SAMPLE alone do anything? (§3.1 / #407) ---
    let sweep_a: Vec<&Arm> = arms.iter().filter(|a| a.cap == 10_000).collect();
    let a_spread = sweep_a
        .iter()
        .map(|a| a.top1)
        .fold(f64::NEG_INFINITY, f64::max)
        - sweep_a.iter().map(|a| a.top1).fold(f64::INFINITY, f64::min);
    println!("\n==== sweep A: CTX_SAMPLE alone, cap pinned at the shipped 10k ====");
    println!(
        "top-1 spread across a 20x pool range: {a_spread:.2}pp (SE {:.2}pp)",
        shipped.standard_error
    );
    println!(
        "{}",
        if a_spread < 2.0 * shipped.standard_error {
            "CONFIRMS the operative half of docs/capacity_scaling_432.md §3.1: \
             widening the draw pool ABOVE the cap does not move the codebook's \
             quality. Note what this does NOT say — #407 raised CTX_SAMPLE from \
             6,000, below the 10,000 cap, so its raise did move the training set \
             (6k -> 10k) and §3.1's 'did not change the training-set size at all' \
             is too strong. What is dead is raising CTX_SAMPLE any further while \
             the cap stands, which is every configuration since #407."
        } else {
            "REFUTES §3.1: the draw pool alone moves top-1 even above the cap, so \
             something other than the training-set size is carrying the effect."
        }
    );

    // --- sweep B: the training set itself ---
    println!("\n==== sweep B: the k-means training set ====");
    let best = arms
        .iter()
        .max_by(|a, b| a.top1.partial_cmp(&b.top1).expect("finite"))
        .expect("arms");
    let largest = arms.last().expect("arms");
    let delta = largest.top1 - shipped.top1;
    let best_delta = best.top1 - shipped.top1;
    println!(
        "shipped {:.2}%  ->  best {:.2}% at training set {} ({best_delta:+.2}pp)",
        shipped.top1, best.top1, best.training_set
    );
    println!(
        "largest training set ({}) gives {:.2}% ({delta:+.2}pp vs shipped) \
         [exit rule: >= +{EXIT_RULE_PP:.1}pp]",
        largest.training_set, largest.top1
    );

    // The delta is a difference between two arms, so the single-arm standard
    // error is the wrong yardstick. Arms share the held-out set, which makes
    // this paired and means the independent-sample form below OVERSTATES the
    // error — it is used deliberately as the conservative choice, since the
    // per-record discordance needed for the paired form is not retained here.
    let se_diff = (shipped.standard_error.powi(2) + best.standard_error.powi(2)).sqrt();
    println!(
        "\nstandard error of the DIFFERENCE: {se_diff:.2}pp (conservative; arms are \
         paired on the held-out set). Best delta is {:.1} SE.",
        best_delta / se_diff
    );

    println!("\n==== verdict against the pre-declared exit rule ====");
    if delta >= EXIT_RULE_PP {
        println!("POSITIVE: codebook fit clears the exit rule. Propose §5.3 scaling.");
    } else if best_delta > 2.0 * se_diff {
        println!(
            "NEGATIVE against the exit rule, but NOT null. The training set is a \
             real lever worth {best_delta:+.2}pp ({:.1} SE) — well below the \
             +{EXIT_RULE_PP:.1}pp bar and it SATURATES: the largest training set ({}) \
             does not beat the best ({}). So §5.3's uncapped `min(N/10, 500k)` \
             prescription buys nothing above the saturation point and should be \
             narrowed to it.",
            best_delta / se_diff,
            largest.training_set,
            best.training_set
        );
    } else {
        println!(
            "NEGATIVE: the training set moves top-1 by {best_delta:+.2}pp at best, \
             which is {:.1} SE on the difference — not separable from noise at this \
             held-out size. Withdraw the §5.3 codebook-scaling recommendation rather \
             than leaving it standing as untested advice.",
            best_delta / se_diff
        );
    }

    // --- the discriminating comparison against the STAGES=5 negative ---
    println!("\n==== records-per-key: is it the binding quantity? ====");
    println!(
        "#460 read the STAGES=5 negative as 'thinner per-key evidence' — records/key \
         fell 36.02 -> 18.80 and top-1 fell with it."
    );
    if best.records_per_key < shipped.records_per_key && best.top1 > shipped.top1 {
        println!(
            "HERE, records/key FELL {:.2} -> {:.2} while top-1 ROSE {:+.2}pp. \
             Lower records-per-key is therefore NOT intrinsically bad: it is bad when \
             it comes from added key RESOLUTION and good when it comes from better \
             codebook FIT. Records-per-key is a symptom, not the binding quantity, \
             and #460's causal reading should be narrowed to resolution specifically.",
            shipped.records_per_key, best.records_per_key, best_delta
        );
    } else {
        println!(
            "HERE, records/key {:.2} -> {:.2} and top-1 {:+.2}pp — this arm does not \
             separate fit from resolution, so #460's reading stands unrefined.",
            shipped.records_per_key, best.records_per_key, best_delta
        );
    }
}
