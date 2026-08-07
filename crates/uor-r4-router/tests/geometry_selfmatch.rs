//! Does the cosine identify a stored sentence from its own text? (issue #486)
//!
//! # What #484 left open
//!
//! Ranking by bare cosine through `retrieve_geometric_resonance`, the target
//! sits at median rank 21,082 of 46,342 **even when the probe is the exact
//! stored sentence** — against a random median of 23,171. Not the probe form,
//! not subsampling, not word order; `docs/lexical_weight_484.md` ruled those
//! out. What it did not do is say WHY.
//!
//! That matters because the same stack has a measurement showing cosine
//! retrieval works: #442's de-banding result, retrieval MRR 0.2348 -> 0.8948,
//! from a harness ranking by cosine ALONE. The same stored vectors support
//! 0.8948 in one harness and sit at chance in this one, so the fault lives
//! somewhere specific and this is a bisection rather than an exploration.
//!
//! # Why this needs no new router API
//!
//! With `set_lexical_weight(0.0)` and `set_unscaled_geometric_term(true)`
//! (both #484), relevance reduces to `sim + scope_boost` with `scope_boost`
//! constant at 15.0 for identity-scoped items. So the ranked list IS the
//! cosine of every candidate against the query, straight out of the deployed
//! path — no reimplementation to get subtly wrong, which is the usual way a
//! diagnostic ends up measuring itself instead of the system.
//!
//! The 15.0 offset is asserted, not assumed: if the observed relevance range
//! leaves `[-1, 1]` after subtracting it, the boost is not what this harness
//! thinks and every number below is void.
//!
//! # What each arm decides
//!
//! - **self vs cross separation, identity probe, band query (deployed).** The
//!   headline. If `sim(q(S), v(S))` is not separated from `sim(q(S), v(T))`
//!   for random `T`, the query projection and the stored vector do not
//!   correspond and everything downstream is explained.
//! - **the same with `set_full_width_query(true)`.** THE LEADING HYPOTHESIS:
//!   #465 made storage full-width and left the query band-only, so the cosine
//!   sees one sixteenth of the query's coordinates. #480 measured that the
//!   symmetric shape does not change ORDERING; nobody checked whether it
//!   changes SELF-IDENTIFICATION, which is a different question.
//! - **the spread of the cross distribution.** If every candidate scores
//!   nearly the same cosine, the stored vectors are saturated and there is
//!   nothing to discriminate — a bug with a fix, not an architectural fact.
//! - **determinism.** The same sentence indexed twice must produce the same
//!   retrieval, or nothing above means anything.
//!
//! # Reading the output
//!
//! `self percentile` is where the target's own cosine falls in the
//! distribution of all candidates' cosines, 1.0 being best. **0.5 is chance.**
//! A working content geometry puts it at or near 1.0.
//!
//! # Running it
//!
//! ```text
//! cargo test --release -p uor-r4-router --test geometry_selfmatch -- --ignored --nocapture
//! ```
//!
//! A few minutes; no teacher, no checkpoint. Knobs: `R4_SELFMATCH_PROBES`
//! (100), `R4_HOPF_CONSTR_STORIES` (2,000), `R4_CORPUS_META`/`R4_CORPUS_RECS`.

use std::collections::HashSet;

use uor_r4_core::transformerless::compiler;
use uor_r4_router::UorR4Router;

const ID: &str = "user:selfmatch";
/// The `scope_boost` every identity-scoped candidate carries, asserted below.
const SCOPE_BOOST: f64 = 15.0;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn token_word(token: u32) -> String {
    format!("t{token:05}")
}

fn render_window(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens.iter().map(|&token| token_word(token)).collect();
    format!("{}.", words.join(" "))
}

fn render_query(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens
        .iter()
        .step_by(2)
        .rev()
        .map(|&token| token_word(token))
        .collect();
    words.join(" ")
}

/// Mean and standard deviation.
fn moments(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    (mean, variance.sqrt())
}

struct ArmResult {
    self_percentile_mean: f64,
    self_sim_mean: f64,
    cross_sim_mean: f64,
    cross_sim_sd: f64,
    separation_sd: f64,
    top1_rate: f64,
}

#[test]
#[ignore = "diagnostic harness (issue #486); run explicitly with --ignored"]
fn does_the_cosine_identify_a_sentence_from_itself() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let Some(c) = compiler::load_corpus_from(&meta_path, &recs_path) else {
        println!("SKIP: corpus fixtures absent ({meta_path} + {recs_path})");
        return;
    };
    println!("#486: does the cosine identify a sentence from itself?");
    println!("corpus: {meta_path} + {recs_path} ({} records)", c.n);

    let cut = (c.stories as f64 * 0.8) as u32;
    let constr: Vec<bool> = (0..c.stories).map(|sid| sid < u64::from(cut)).collect();

    let mut streams: Vec<(u32, Vec<u32>)> = Vec::new();
    for ((&sid, &input), &next) in c.story.iter().zip(&c.input).zip(&c.next).take(c.n) {
        if streams.last().map(|(last, _)| *last) != Some(sid) {
            streams.push((sid, vec![input]));
        }
        let (_, stream) = streams.last_mut().expect("just pushed");
        stream.push(next);
    }

    let story_cap = env_usize("R4_HOPF_CONSTR_STORIES", 2_000);
    let probe_cap = env_usize("R4_SELFMATCH_PROBES", 100).max(1);
    let capped: Vec<&(u32, Vec<u32>)> = streams
        .iter()
        .filter(|(sid, _)| constr[*sid as usize])
        .take(story_cap)
        .collect();

    let mut windows: Vec<String> = Vec::new();
    let mut window_tokens: Vec<Vec<u32>> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (_, stream) in &capped {
        for chunk in stream.chunks_exact(compiler::WINDOW) {
            let sentence = render_window(chunk);
            if seen.insert(sentence.clone()) {
                windows.push(sentence);
                window_tokens.push(chunk.to_vec());
            }
        }
    }
    let stride = (windows.len() / probe_cap).max(1);
    let targets: Vec<usize> = (0..windows.len()).step_by(stride).take(probe_cap).collect();
    println!(
        "CAPS: {} stories, {} distinct windows, {} probes",
        capped.len(),
        windows.len(),
        targets.len()
    );

    let corpus_text: String = windows.join(" ");

    let arm = |label: &str, full_width: bool, identity_probe: bool, content: bool| -> ArmResult {
        let mut router = UorR4Router::new(0.5);
        router.set_lexical_weight(0.0);
        router.set_unscaled_geometric_term(true);
        router.set_full_width_query(full_width);
        router.set_content_query_vector(content);
        let indexed = router.index_corpus(&corpus_text, ID);
        assert_eq!(indexed, windows.len(), "[{label}] indexed every window");

        let mut self_percentiles = Vec::new();
        let mut self_sims = Vec::new();
        let mut cross_means = Vec::new();
        let mut cross_sds = Vec::new();
        let mut separations = Vec::new();
        let mut top1 = 0usize;

        for &t in &targets {
            let probe = if identity_probe {
                windows[t].clone()
            } else {
                render_query(&window_tokens[t])
            };
            let results = router.get_top_resonances_native(&probe, ID, windows.len());
            assert!(!results.is_empty(), "[{label}] retrieval returned nothing");

            // The boost is asserted, not assumed: strip it and the remainder
            // must be a cosine. If it is not, this harness is reading the
            // wrong quantity and every number it prints is void.
            let sims: Vec<f64> = results.iter().map(|r| r.relevance - SCOPE_BOOST).collect();
            for &s in &sims {
                assert!(
                    (-1.001..=1.001).contains(&s),
                    "[{label}] relevance minus scope_boost is {s}, not a cosine — the \
                     boost assumption is wrong and this diagnostic is void"
                );
            }

            let Some(pos) = results.iter().position(|r| r.sentence == windows[t]) else {
                panic!("[{label}] target absent from the full candidate list");
            };
            let self_sim = sims[pos];
            let cross: Vec<f64> = sims
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != pos)
                .map(|(_, &s)| s)
                .collect();
            let below = cross.iter().filter(|&&s| s < self_sim).count() as f64;
            self_percentiles.push(below / cross.len() as f64);
            self_sims.push(self_sim);
            let (mean, sd) = moments(&cross);
            cross_means.push(mean);
            cross_sds.push(sd);
            separations.push(if sd > 0.0 {
                (self_sim - mean) / sd
            } else {
                0.0
            });
            if pos == 0 {
                top1 += 1;
            }
        }

        ArmResult {
            self_percentile_mean: moments(&self_percentiles).0,
            self_sim_mean: moments(&self_sims).0,
            cross_sim_mean: moments(&cross_means).0,
            cross_sim_sd: moments(&cross_sds).0,
            separation_sd: moments(&separations).0,
            top1_rate: top1 as f64 / targets.len() as f64,
        }
    };

    println!("\narm\t\t\t\tself pct\tself sim\tcross sim\tcross sd\tsep (sd)\ttop-1");
    let mut rows = Vec::new();
    for (label, full_width, identity_probe, content) in [
        ("identity probe, band query", false, true, false),
        ("identity probe, FULL-WIDTH query", true, true, false),
        ("shipped probe, band query", false, false, false),
        ("shipped probe, FULL-WIDTH query", true, false, false),
        // #486 THE FIX ARM: query encoded through the same content
        // construction that produced every stored vector.
        ("identity probe, CONTENT query", false, true, true),
        ("shipped probe, CONTENT query", false, false, true),
    ] {
        let r = arm(label, full_width, identity_probe, content);
        println!(
            "{label:<32}{:.4}\t\t{:+.4}\t\t{:+.4}\t\t{:.4}\t\t{:+.2}\t\t{:.4}",
            r.self_percentile_mean,
            r.self_sim_mean,
            r.cross_sim_mean,
            r.cross_sim_sd,
            r.separation_sd,
            r.top1_rate
        );
        rows.push((label, r));
    }

    println!(
        "\n`self pct` is where the target's own cosine falls among all {} candidates' \
         cosines; 1.0 is perfect, **0.5 is chance**. `sep` is the same thing in standard \
         deviations of the cross distribution.",
        windows.len()
    );

    // ---- determinism: the same text indexed twice must retrieve the same ----
    {
        let mut a = UorR4Router::new(0.5);
        a.set_lexical_weight(0.0);
        a.set_unscaled_geometric_term(true);
        a.index_corpus(&corpus_text, ID);
        let mut b = UorR4Router::new(0.5);
        b.set_lexical_weight(0.0);
        b.set_unscaled_geometric_term(true);
        b.index_corpus(&corpus_text, ID);
        let probe = windows[targets[0]].clone();
        let ra = a.get_top_resonances_native(&probe, ID, 50);
        let rb = b.get_top_resonances_native(&probe, ID, 50);
        let same = ra.len() == rb.len()
            && ra.iter().zip(&rb).all(|(x, y)| {
                x.sentence == y.sentence && (x.relevance - y.relevance).abs() < 1e-12
            });
        println!(
            "\ndeterminism: two independent indexings of the same corpus retrieve {} — {}",
            if same { "identically" } else { "DIFFERENTLY" },
            if same {
                "PASS"
            } else {
                "VOID (nothing above means anything)"
            }
        );
        assert!(
            same,
            "indexing is not deterministic; the diagnostic is void"
        );
    }

    // ---- classification ----
    println!("\n==== classification ====");
    let (_, identity_band) = &rows[0];
    let (_, identity_full) = &rows[1];
    println!(
        "identity probe, deployed band query: self percentile {:.4}, separation {:+.2} sd",
        identity_band.self_percentile_mean, identity_band.separation_sd
    );
    println!(
        "identity probe, full-width query:    self percentile {:.4}, separation {:+.2} sd",
        identity_full.self_percentile_mean, identity_full.separation_sd
    );
    let band_works = identity_band.self_percentile_mean > 0.95;
    let full_works = identity_full.self_percentile_mean > 0.95;
    if band_works {
        println!(
            "The deployed query projection DOES identify a sentence from itself. The #484 \
             ranking failure is then downstream of the vectors, not in them — look at the \
             ranking, not the geometry."
        );
    } else if full_works {
        println!(
            "(c) BAND-PROJECTION LOSS. The deployed band-only query cannot identify a \
             sentence from itself; the full-width query can. #465 made storage full-width \
             and left the query banded, and #480 measured only that the symmetric shape \
             does not change ORDERING under a lexical ranking — which it would not, since \
             the lexical term decides the order. This is a BUG WITH A FIX: flipping the \
             query shape makes geometric retrieval reachable, and the #480/#484 negatives \
             should be re-run against it."
        );
    } else if identity_band.cross_sim_sd < 0.01 {
        println!(
            "(a) SATURATED STORED VECTORS. Cross-candidate cosine spread is {:.5}, so every \
             stored vector looks alike and there is nothing for a cosine to discriminate. \
             A bug with a fix, upstream in what `index_corpus` writes.",
            identity_band.cross_sim_sd
        );
    } else {
        println!(
            "(b) NON-CORRESPONDING OBJECTS. The cross distribution has real spread \
             ({:.5} sd), so the vectors differ from each other — but the query projection \
             of a sentence does not match that sentence's stored vector under either \
             shape. The query and stored vectors are not the same kind of object. This is \
             ARCHITECTURAL, not a wiring bug, and it is the decision flagged on #486.",
            identity_band.cross_sim_sd
        );
    }
}
