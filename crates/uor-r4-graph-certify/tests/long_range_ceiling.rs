//! Long-range context ceiling (issue #424).
//!
//! Front: the shipped runtime's context is a hard 8-token window, and
//! `bott_fock.rs` is named in `docs/r4_furey_quantum_geometric_plan.md` as
//! the priority candidate for closing "the long-context gap". Before
//! activating any carrier behind a feature flag and paying a Gate C A/B,
//! this harness answers the question that gates that spend: **how much
//! top-1 is available to ANY long-range carrier on this corpus?**
//!
//! The reachability arithmetic in `AGENTS.md` requires that ceiling be
//! computed before a run measured in hours. This is that computation, and
//! it is cheap — pure counting over the record stream, no compile, no
//! store, no Gate C.
//!
//! ## Why this bounds `bott_fock` without building it
//!
//! `BottFockContextStore` folds the context into a fixed 256-entry i16
//! state with a geometric decay of 3/4 per token. Its inductive bias is
//! therefore: *a distance-discounted, order-sensitive summary of the
//! tokens before the window*. The `BF-DECAY` arm below implements that
//! same bias **losslessly** — exact counts, exact distances, unbounded
//! precision, no 256-cell bottleneck. Anything the fold can express, this
//! arm can express; the converse is false. So the arm's top-1 is an upper
//! bound on the fold's, and a negative here is a negative for the fold at
//! that decay constant — without the flag, the wiring, or the A/B.
//!
//! Sweeping the decay constant separates two very different findings:
//! "the mechanism is wrong" (flat across all decays) from "the constant
//! is wrong" (rises with the horizon). Those imply different next actions,
//! which is what makes the sweep worth running.
//!
//! ## Variables
//!
//! Story-bounded token streams reconstructed by `load_corpus_from`
//! (`input[i] = next[i-1]` within a story). For an evaluation position
//! `t` inside a story:
//!   - WINDOW: the `WINDOW` tokens immediately before `t` — what the
//!     runtime already has;
//!   - DISTANT: every earlier token in the same story, strictly before the
//!     window — what a long-range carrier would add.
//!
//! ## Split
//!
//! Document-level by story ordinal (`story % 5 == 0` held out), mirroring
//! the bundle's own 1-in-5 document partition rule. Document-level means
//! no position from an evaluated story contributes to the tables, so
//! DISTANT context cannot leak the answer through the n-gram counts.
//!
//! ## Baseline
//!
//! Backoff trigram -> bigram -> unigram argmax over construction stories —
//! the shape of the shipped serving stack's packed NGRAM rows. It is a
//! weaker absolute predictor than the full stack, which makes every ceiling
//! reported here *generous*: a stronger baseline has already captured more
//! of the same signal, so measured headroom can only shrink against it.
//!
//! ## Arms
//!
//!   BASE       backoff n-gram, 8-token window only.
//!   CACHE      + lambda * (count of the candidate in DISTANT).
//!              Order-free: a bag of the distant tokens. This is the
//!              classic unigram-cache LM and it isolates *topical* value.
//!   INDUCTION  + lambda * (count of adjacent (last token, candidate)
//!              pairs in DISTANT). Order-sensitive, undecayed, unbounded
//!              horizon — the ceiling for the order-sensitive family.
//!   BF-DECAY   INDUCTION with each occurrence weighted by
//!              `decay^(distance)`, swept over decay. `decay = 0.75` is
//!              the constant `bott_fock.rs` ships
//!              (`cell <- cell - (cell >> 2)`).
//!
//! CACHE vs INDUCTION separates topical value from order-carried value.
//! That distinction decides the carrier's *shape*: if CACHE captures the
//! gain, an order-sensitive matrix fold is the wrong machine and a
//! document-level token cache is the right one, at a fraction of the cost.
//!
//! ## Null arm (falsifier)
//!
//! Every arm is re-run with DISTANT taken from a **different story** — a
//! fixed derangement of story indices, same position, same length. The
//! null keeps the arms' generic statistics (a distant prefix of natural
//! text still contains common bigrams) and destroys only the link to *this*
//! document's history. A gain that survives in the null is not long-range
//! context; it is a prior on frequent continuations.
//!
//! The null is a **validity gate, not a subtraction.** `observed - null`
//! measures how much better this document's history is than a stranger's —
//! a statement about information content. It is NOT what a carrier buys,
//! because a deployed system's alternative is *no* distant evidence, not
//! *wrong* distant evidence. At high lambda a wrong donor actively damages
//! the baseline, so `observed - null` grows while the achievable gain does
//! not; subtracting the null would report a carrier as several points more
//! valuable than any A/B could ever confirm. The primary metric here is
//! therefore `observed - base`, and the null must show no comparable gain
//! for that number to count as long-range context.
//!
//! ## Pre-declared exit rule
//!
//! Let CEILING = best `observed - base` over the INDUCTION lambda grid,
//! admitted only if the null arm at the same lambda shows no comparable
//! gain (null gain < half the observed gain).
//!   - CEILING < 0.5pp  -> the long-context gap is not worth a carrier on
//!     this corpus at all; #424 closes negative and the gap question
//!     closes with it.
//!   - CEILING >= 0.5pp and BF-DECAY at 0.75 retains < half of it -> the
//!     mechanism is sound and the shipped *constant* is the defect; the
//!     next action is to re-parameterize the decay, not to A/B the fold as
//!     shipped.
//!   - CEILING >= 0.5pp and BF-DECAY at 0.75 retains >= half -> the fold as
//!     shipped is worth activating; proceed to the flagged Gate C A/B.
//!
//! Standard error is reported beside every rate; at these position counts
//! it is about 0.16pp, so a 0.5pp threshold is roughly 3 SE.
//!
//! Recorded outcome (2026-08-07, corpus `blake3:194db0ee…`, 3,000 stories
//! / 361,693 positions) is in `docs/context_horizon_424.md`.

use std::collections::HashMap;

use uor_r4_core::transformerless::compiler::{self, Corpus};

/// The runtime's shipped context width.
const WINDOW: usize = 8;
/// Held-out selector over story ordinals — the bundle's own 1-in-5 rule.
const HELD_OUT_MODULUS: u32 = 5;
const LAMBDA_GRID: [f64; 6] = [0.05, 0.1, 0.5, 1.0, 2.0, 4.0];
const DECAY_GRID: [f64; 8] = [0.75, 0.80, 0.85, 0.90, 0.95, 0.97, 0.99, 0.999];
/// The constant `bott_fock.rs` ships: `cell <- cell - (cell >> 2)`.
const SHIPPED_DECAY: f64 = 0.75;
const CEILING_THRESHOLD_PP: f64 = 0.5;

fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

/// Story-bounded token streams: one `Vec<u32>` per story, in corpus order.
fn stories(corpus: &Corpus) -> Vec<(u32, Vec<u32>)> {
    let mut out: Vec<(u32, Vec<u32>)> = Vec::new();
    for i in 0..corpus.n {
        let id = corpus.story[i];
        match out.last_mut() {
            Some((last, tokens)) if *last == id => tokens.push(corpus.next[i]),
            _ => out.push((id, vec![corpus.next[i]])),
        }
    }
    out
}

/// Backoff n-gram tables built from the construction stories only.
struct Tables {
    tri: HashMap<(u32, u32), HashMap<u32, u32>>,
    bi: HashMap<u32, HashMap<u32, u32>>,
    uni: HashMap<u32, u32>,
}

impl Tables {
    fn build(construction: &[&Vec<u32>]) -> Self {
        let mut tri: HashMap<(u32, u32), HashMap<u32, u32>> = HashMap::new();
        let mut bi: HashMap<u32, HashMap<u32, u32>> = HashMap::new();
        let mut uni: HashMap<u32, u32> = HashMap::new();
        for tokens in construction {
            for (i, &token) in tokens.iter().enumerate() {
                *uni.entry(token).or_default() += 1;
                if i + 1 < tokens.len() {
                    *bi.entry(token)
                        .or_default()
                        .entry(tokens[i + 1])
                        .or_default() += 1;
                }
                if i + 2 < tokens.len() {
                    *tri.entry((token, tokens[i + 1]))
                        .or_default()
                        .entry(tokens[i + 2])
                        .or_default() += 1;
                }
            }
        }
        Self { tri, bi, uni }
    }

    /// Candidate continuation counts for position `t`, longest context first.
    fn candidates(&self, tokens: &[u32], t: usize) -> &HashMap<u32, u32> {
        if t >= 2 {
            if let Some(row) = self.tri.get(&(tokens[t - 2], tokens[t - 1])) {
                return row;
            }
        }
        if t >= 1 {
            if let Some(row) = self.bi.get(&tokens[t - 1]) {
                return row;
            }
        }
        &self.uni
    }
}

/// Which distance-weighted evidence, if any, the arm adds to the baseline.
#[derive(Clone, Copy, PartialEq)]
enum Arm {
    Base,
    /// Order-free bag of the distant tokens.
    Cache,
    /// Order-sensitive; `Some(decay)` discounts by distance, `None` does not.
    Induction(Option<f64>),
}

/// Deterministic argmax: highest score, ties broken by lowest token id.
/// `HashMap` iteration order is not stable across runs, so the tie-break is
/// load-bearing — the same class of defect as issue #451.
fn argmax(scores: &HashMap<u32, f64>) -> u32 {
    let mut best_token = u32::MAX;
    let mut best_score = f64::NEG_INFINITY;
    for (&token, &score) in scores {
        if score > best_score || (score == best_score && token < best_token) {
            best_score = score;
            best_token = token;
        }
    }
    best_token
}

/// Top-1 rate over the held-out positions.
///
/// `distant_from` selects the story supplying DISTANT: the evaluated story
/// itself for the observed arm, a different story for the null.
fn top1(
    tables: &Tables,
    held: &[&Vec<u32>],
    distant_from: &[&Vec<u32>],
    arm: Arm,
    lambda: f64,
) -> (f64, f64, usize) {
    let mut correct = 0usize;
    let mut total = 0usize;
    for (index, tokens) in held.iter().enumerate() {
        let donor = distant_from[index];
        for t in (WINDOW + 1)..tokens.len() {
            let candidates = tables.candidates(tokens, t);
            if candidates.is_empty() {
                continue;
            }
            total += 1;
            let predicted = if arm == Arm::Base {
                let mut scores = HashMap::new();
                for (&token, &count) in candidates {
                    scores.insert(token, count as f64);
                }
                argmax(&scores)
            } else {
                // DISTANT: strictly before the window, capped at the donor's
                // own length so the null sees a prefix of the same size.
                let cut = (t - WINDOW).min(donor.len());
                let distant = &donor[..cut];
                let mut bonus: HashMap<u32, f64> = HashMap::new();
                match arm {
                    Arm::Cache => {
                        for &token in distant {
                            *bonus.entry(token).or_default() += 1.0;
                        }
                    }
                    Arm::Induction(decay) => {
                        let current = tokens[t - 1];
                        for i in 0..distant.len().saturating_sub(1) {
                            if distant[i] != current {
                                continue;
                            }
                            let weight = match decay {
                                None => 1.0,
                                Some(d) => d.powi((distant.len() - 1 - i) as i32),
                            };
                            *bonus.entry(distant[i + 1]).or_default() += weight;
                        }
                    }
                    Arm::Base => unreachable!(),
                }
                let max = candidates.values().copied().max().unwrap_or(1) as f64;
                let mut scores: HashMap<u32, f64> = candidates
                    .iter()
                    .map(|(&token, &count)| (token, count as f64 / max))
                    .collect();
                for (token, weight) in bonus {
                    *scores.entry(token).or_default() += lambda * weight;
                }
                argmax(&scores)
            };
            if predicted == tokens[t] {
                correct += 1;
            }
        }
    }
    let rate = correct as f64 / total as f64 * 100.0;
    let se = (rate / 100.0 * (1.0 - rate / 100.0) / total as f64).sqrt() * 100.0;
    (rate, se, total)
}

/// Fixed derangement of the held-out stories: index `i` receives the
/// distant prefix of story `i + 1 (mod len)`. Deterministic, no RNG, and a
/// true derangement for any length above one, so no story is ever its own
/// donor.
fn donors<'a>(held: &[&'a Vec<u32>]) -> Vec<&'a Vec<u32>> {
    (0..held.len())
        .map(|i| held[(i + 1) % held.len()])
        .collect()
}

#[test]
#[ignore = "measurement harness (issue #424); run explicitly with --ignored"]
fn long_range_ceiling() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let Some(corpus) = compiler::load_corpus_from(&meta_path, &recs_path) else {
        println!("SKIP: corpus fixtures absent ({meta_path} + {recs_path}); vacuous green");
        return;
    };
    println!("corpus: {meta_path} + {recs_path}");

    let all = stories(&corpus);
    let construction: Vec<&Vec<u32>> = all
        .iter()
        .filter(|(id, _)| id % HELD_OUT_MODULUS != 0)
        .map(|(_, tokens)| tokens)
        .collect();
    let held: Vec<&Vec<u32>> = all
        .iter()
        .filter(|(id, _)| id % HELD_OUT_MODULUS == 0)
        .map(|(_, tokens)| tokens)
        .collect();
    let longest = all.iter().map(|(_, t)| t.len()).max().unwrap_or(0);
    println!(
        "long-range ceiling (#424): {} records, {} stories ({} construction / {} held out); \
         longest story {longest} tokens, WINDOW = {WINDOW}",
        corpus.n,
        all.len(),
        construction.len(),
        held.len()
    );
    if longest <= WINDOW + 2 {
        println!("SKIP: no story is longer than the window; nothing to measure");
        return;
    }
    // The corpus caps how far "long range" can even mean. Print it: a
    // ceiling measured on 128-token documents does not license a claim
    // about book-length context.
    println!(
        "NOTE: the longest story is {longest} tokens, so DISTANT never exceeds \
         {} tokens. This bounds the scope of every number below.",
        longest.saturating_sub(WINDOW + 1)
    );

    let tables = Tables::build(&construction);
    println!(
        "tables: {} trigram contexts, {} bigram contexts, {} unigram types",
        tables.tri.len(),
        tables.bi.len(),
        tables.uni.len()
    );

    let donor = donors(&held);
    let self_donor: Vec<&Vec<u32>> = held.clone();

    let (base, base_se, positions) = top1(&tables, &held, &self_donor, Arm::Base, 0.0);
    println!(
        "\nBASE  backoff tri->bi->uni, window only : {base:.2}% +/- {base_se:.2}pp (n={positions})"
    );

    // GAIN is the primary metric (what a carrier buys); MARGIN is reported
    // beside it because the two are routinely confused and only GAIN is
    // what a Gate C A/B could confirm.
    println!("\narm\t\tlambda\tobserved\tnull\tGAIN(obs-base)\tnull gain\tmargin(obs-null)");
    let mut ceiling = f64::NEG_INFINITY;
    let mut ceiling_lambda = 0.0;
    let mut cache_best = f64::NEG_INFINITY;
    for (label, arm) in [("CACHE", Arm::Cache), ("INDUCTION", Arm::Induction(None))] {
        for lambda in LAMBDA_GRID {
            let (observed, _, _) = top1(&tables, &held, &self_donor, arm, lambda);
            let (null, _, _) = top1(&tables, &held, &donor, arm, lambda);
            let gain = observed - base;
            let null_gain = null - base;
            // A gain only counts as long-range context if a stranger's
            // history does not reproduce it.
            let admitted = gain > 0.0 && null_gain < gain / 2.0;
            println!(
                "{label}\t{lambda}\t{observed:.2}%\t\t{null:.2}%\t{gain:+.2}pp\t\t{null_gain:+.2}pp\t\t{:+.2}pp{}",
                observed - null,
                if admitted { "" } else { "  (not admitted)" }
            );
            if admitted && label == "INDUCTION" && gain > ceiling {
                ceiling = gain;
                ceiling_lambda = lambda;
            }
            if admitted && label == "CACHE" && gain > cache_best {
                cache_best = gain;
            }
        }
    }
    println!("\nCEILING (order-sensitive, lossless, unbounded horizon) = {ceiling:+.2}pp at lambda {ceiling_lambda}");
    println!(
        "  of which order-free (CACHE, topical) accounts for {cache_best:+.2}pp; \
         the remaining {:+.2}pp is order-carried.",
        ceiling - cache_best
    );

    println!("\n--- decay sweep at lambda {ceiling_lambda}: what the horizon constant costs ---");
    println!("decay\thorizon~\tobserved\tnull\tGAIN(obs-base)\tretained");
    let mut shipped_retained = f64::NAN;
    for decay in DECAY_GRID {
        let arm = Arm::Induction(Some(decay));
        let (observed, _, _) = top1(&tables, &held, &self_donor, arm, ceiling_lambda);
        let (null, _, _) = top1(&tables, &held, &donor, arm, ceiling_lambda);
        let adjusted = observed - base;
        let retained = adjusted / ceiling * 100.0;
        // Horizon: distance at which a contribution falls below 1e-3.
        let horizon = (1e-3f64).ln() / decay.ln();
        let tag = if decay == SHIPPED_DECAY {
            "  <-- SHIPPED bott_fock.rs"
        } else {
            ""
        };
        println!(
            "{decay}\t{horizon:.0}\t\t{observed:.2}%\t\t{null:.2}%\t{adjusted:+.2}pp\t\t{retained:.0}%{tag}"
        );
        if decay == SHIPPED_DECAY {
            shipped_retained = retained;
        }
    }

    println!("\n==== verdict against the pre-declared exit rule ====");
    if ceiling < CEILING_THRESHOLD_PP {
        println!(
            "NEGATIVE: ceiling {ceiling:+.2}pp < {CEILING_THRESHOLD_PP}pp. No long-range carrier \
             is worth building on this corpus; the gap question closes with #424."
        );
    } else if shipped_retained < 50.0 {
        println!(
            "MECHANISM SOUND, CONSTANT WRONG: ceiling {ceiling:+.2}pp is reachable, but the \
             shipped decay {SHIPPED_DECAY} retains only {shipped_retained:.0}% of it. \
             Re-parameterize the decay in bott_fock.rs before any Gate C A/B; activating the \
             fold as shipped forfeits {:.0}% of the available signal.",
            100.0 - shipped_retained
        );
    } else {
        println!(
            "PROCEED: ceiling {ceiling:+.2}pp and the shipped decay retains \
             {shipped_retained:.0}% of it. The flagged Gate C A/B is reachable."
        );
    }

    // The harness must not silently report a ceiling it cannot resolve.
    assert!(
        positions > 10_000,
        "too few held-out positions ({positions}) for a ceiling at this resolution"
    );
}
