//! Memory-lift three-arm harness at D3 corpus scale (issue #423): the
//! full-corpus reading promised on issue #255. Same methodology as the
//! fixture harness (`memory_lift_ab.rs`) — content-free (pre-#245 stub,
//! arm one) vs content-derived (production, post-#245, arm two) vs
//! shuffled-vector control (arm three), retrieval quality measured as
//! mean reciprocal rank (MRR) of the target sentence by cosine over
//! stored state vectors — pointed at the D3 natural corpus instead of
//! the ten-sentence fixture.
//!
//! # Corpus and token-to-word mapping
//!
//! The D3 natural corpus is token ids only (span/byte offsets but no
//! byte store), so each token id is rendered as a synthetic word
//! (`t00042`) — the issue-421 mapping: the router's word-to-prime
//! assignment is arrival-ordered and its seeded word vector depends
//! only on the assigned prime, so the router never consumes natural
//! language, only word identity and co-occurrence; the bijection
//! preserves exactly the structure the router consumes. Sentences are
//! consecutive non-overlapping eight-token windows (`compiler::WINDOW`)
//! of each construction-split story's token stream, rendered with a
//! terminating period (the form the router's sentence splitter stores)
//! and deduplicated to first occurrence so that sentence text is a
//! unique key into the store. The split is the anchor_infill.rs law:
//! the D3 hash partition when `R4_STORIES` is set, otherwise the
//! sequential eighty/twenty story cut.
//!
//! # Retrieval task (the fixture design at scale)
//!
//! In the fixture, each probe query shares content words with exactly
//! one corpus sentence. The corpus-scale analog: each probe's query is
//! its target window's even-offset tokens in reverse order — half the
//! words, different order, no novel words. The query becomes a
//! content-derived vector through the router's own indexing surface
//! (scratch identity — the fixture's `retrieval_metrics` pattern), and
//! the target's rank is its position among all stored windows by
//! cosine, descending, ties broken by ascending window index (the
//! fixture's stable-sort semantics, made deterministic here by fixing
//! the candidate order; the fixture's own order was HashMap-iteration
//! dependent, its documented flake source). Targets are defined by
//! construction; other windows may share query tokens, which depresses
//! all arms equally.
//!
//! # Arms
//!
//! Arm one ingests every window via `index_sentence_content_free` (the
//! pre-#245 stub reconstruction, per-sentence, as in the fixture). Arm
//! two ingests the same windows through the production bulk surface
//! (`index_corpus`, the issue-421 ingestion path) — per-sentence
//! `index_sentence` is equivalent but its duplicate scan is quadratic
//! at corpus scale. Arm three is the fixture control: arm two's stored
//! vectors under the fixed half-length rotation across window
//! positions. Because the rotation permutes stored positions, not
//! values, arm three is scored from arm two's similarity table with a
//! rotated index — exactly the fixture construction (whose control
//! router is built identically to its production router and therefore
//! yields identical query vectors) without a third full-scale
//! ingestion. Queries for arm one route through arm one's router,
//! mirroring the fixture's per-arm query construction.
//!
//! # Caps (documented, printed, no silent truncation)
//!
//! The router store keeps two 512-dim f64 copies per sentence, so
//! ingestion is capped at the first `R4_MEMLIFT_CONSTR_STORIES`
//! construction stories (default two thousand, ascending story id) and
//! probes are capped at `R4_MEMLIFT_PROBES` target windows (default
//! five hundred, evenly strided across the deduplicated window list).
//! Both caps print in the report.
//!
//! # Pre-declared exit rule
//!
//! Content-derived (arm two) is declared VALUE-CARRYING for issue
//! \#245's restoration if and only if its MRR exceeds content-free
//! (arm one) by at least two percentage points of reciprocal rank (an
//! MRR margin of at least 0.020) AND exceeds the shuffled control (arm
//! three). Anything less is a recorded negative. All three arms are
//! reported regardless; direction is a result, not an assertion (the
//! fixture harness's own convention after its 2026-07-30 merge-queue
//! flake — only structural invariants gate).
//!
//! Run (natural stack):
//!   R4_CORPUS_META=/tmp/c_meta.bin R4_CORPUS_RECS=/tmp/c_recs.bin \
//!   R4_STORIES=/tmp/wiki-obs/stories.jsonl \
//!   cargo test --release -p uor-r4-router --test memory_lift_corpus -- \
//!   --ignored --nocapture

use std::collections::{HashMap, HashSet};

use uor_r4_core::transformerless::compiler;
use uor_r4_router::UorR4Router;

/// Identity scope of each arm's corpus store.
const ID: &str = "user:memlift";

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

/// Token id rendered as the synthetic router word (mapping, module docs).
fn token_word(token: u32) -> String {
    format!("t{token:05}")
}

/// An eight-token window rendered as the stored sentence form (with the
/// terminator the router's sentence splitter keeps).
fn render_window(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens.iter().map(|&token| token_word(token)).collect();
    format!("{}.", words.join(" "))
}

/// The probe query for a target window: even-offset tokens in reverse
/// order (module docs) — half the words, different order, no novel words.
fn render_query(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens
        .iter()
        .step_by(2)
        .rev()
        .map(|&token| token_word(token))
        .collect();
    words.join(" ")
}

fn cosine(a: &[f64], b: &[f64], norm_a: f64, norm_b: f64) -> f64 {
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    dot / (norm_a * norm_b)
}

fn norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

/// Rank of `target` in a stable descending sort of `sims` (equal
/// similarity ranks by ascending index — module docs). `rotation`
/// re-maps stored positions for the shuffled arm: position `j` carries
/// the vector (hence similarity) of position `(j + rotation) % n`.
fn rank_of(sims: &[f64], target: usize, rotation: usize) -> usize {
    let n = sims.len();
    let target_sim = sims[(target + rotation) % n];
    let mut rank = 1usize;
    for (position, _) in sims.iter().enumerate() {
        let sim = sims[(position + rotation) % n];
        if sim > target_sim || (sim == target_sim && position < target) {
            rank += 1;
        }
    }
    rank
}

/// Content-derived query vectors through the router's own indexing
/// surface (scratch identities — the fixture's `retrieval_metrics`
/// pattern, one distinct scratch id per probe).
fn query_vectors(router: &mut UorR4Router, queries: &[String]) -> Vec<Vec<f64>> {
    queries
        .iter()
        .enumerate()
        .map(|(qi, q)| {
            let scratch = format!("user:q{qi}");
            router.index_sentence(q, &scratch);
            let items = router.corpus_items_for(&scratch);
            assert_eq!(items.len(), 1, "one stored item per probe query");
            items[0].state_vector.clone()
        })
        .collect()
}

/// Stored state vectors re-ordered into window order (the store's own
/// iteration order is HashMap-dependent; sentence text is the key).
fn aligned_vectors<'a>(router: &'a UorR4Router, windows: &[String]) -> Vec<&'a [f64]> {
    let items = router.corpus_items_for(ID);
    assert_eq!(
        items.len(),
        windows.len(),
        "store holds every deduplicated window (no silent truncation)"
    );
    let by_sentence: HashMap<&str, &[f64]> = items
        .iter()
        .map(|item| (item.sentence.as_str(), item.state_vector.as_slice()))
        .collect();
    windows
        .iter()
        .map(|w| {
            *by_sentence
                .get(w.as_str())
                .expect("every window is stored under its own text")
        })
        .collect()
}

/// (top-1 hit rate, MRR) for one arm given per-probe ranks.
fn metrics(ranks: &[usize]) -> (f64, f64) {
    let hits = ranks.iter().filter(|&&r| r == 1).count() as f64;
    let mrr: f64 = ranks.iter().map(|&r| 1.0 / r as f64).sum();
    (hits / ranks.len() as f64, mrr / ranks.len() as f64)
}

#[test]
#[ignore = "issue #423 measurement harness; run explicitly with --ignored"]
fn three_arm_memory_lift_corpus_scale() {
    // ---- natural stack ----
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let c = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    println!("corpus: {meta_path} + {recs_path} ({} records)", c.n);

    // ---- split: D3 hash partition when R4_STORIES is set, else the
    // sequential eighty/twenty story cut (anchor_infill.rs law) ----
    let cut = (c.stories as f64 * 0.8) as u32;
    let constr: Vec<bool> = match std::env::var("R4_STORIES") {
        Ok(path) => {
            let text = std::fs::read_to_string(&path).expect("stories.jsonl");
            let mut v = vec![true; c.stories as usize];
            for line in text.lines() {
                let Some(story_pos) = line.find("\"story\":") else {
                    continue;
                };
                let story: usize = line[story_pos + 8..]
                    .split(',')
                    .next()
                    .and_then(|x| x.trim().parse().ok())
                    .expect("story id");
                if story < v.len() {
                    v[story] = !line.contains("\"partition\":\"HeldOut\"");
                }
            }
            println!(
                "partition: D3 hash split from {path} ({} construction / {} held-out stories)",
                v.iter().filter(|&&b| b).count(),
                v.iter().filter(|&&b| !b).count()
            );
            v
        }
        Err(_) => (0..c.stories).map(|sid| sid < u64::from(cut)).collect(),
    };

    // ---- per-story token streams (stream index zero = first input) ----
    let mut streams: Vec<(u32, Vec<u32>)> = Vec::new();
    for ((&sid, &input), &next) in c.story.iter().zip(&c.input).zip(&c.next).take(c.n) {
        if streams.last().map(|(last, _)| *last) != Some(sid) {
            streams.push((sid, vec![input]));
        }
        let (_, stream) = streams.last_mut().expect("just pushed");
        stream.push(next);
    }

    // ---- CAPS (module docs; printed, never silent) ----
    let story_cap = env_usize("R4_MEMLIFT_CONSTR_STORIES", 2_000);
    let probe_cap = env_usize("R4_MEMLIFT_PROBES", 500).max(1);
    let constr_total = streams.iter().filter(|(s, _)| constr[*s as usize]).count();
    let capped: Vec<&(u32, Vec<u32>)> = streams
        .iter()
        .filter(|(sid, _)| constr[*sid as usize])
        .take(story_cap)
        .collect();
    println!(
        "CAPS: ingesting {} of {} construction stories; up to {} probes \
         (R4_MEMLIFT_CONSTR_STORIES / R4_MEMLIFT_PROBES override)",
        capped.len(),
        constr_total,
        probe_cap
    );

    // ---- eight-token windows, deduplicated to first occurrence ----
    let mut windows: Vec<String> = Vec::new();
    let mut window_tokens: Vec<Vec<u32>> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut raw_windows = 0usize;
    for (_, stream) in &capped {
        for chunk in stream.chunks_exact(compiler::WINDOW) {
            raw_windows += 1;
            let sentence = render_window(chunk);
            if seen.insert(sentence.clone()) {
                windows.push(sentence);
                window_tokens.push(chunk.to_vec());
            }
        }
    }
    println!(
        "windows: {} eight-token windows, {} distinct after dedup",
        raw_windows,
        windows.len()
    );
    assert!(
        windows.len() > probe_cap,
        "corpus-scale premise: more windows than probes"
    );

    // ---- probes: evenly strided targets across the window list ----
    let stride = (windows.len() / probe_cap).max(1);
    let targets: Vec<usize> = (0..windows.len()).step_by(stride).take(probe_cap).collect();
    let queries: Vec<String> = targets
        .iter()
        .map(|&t| render_query(&window_tokens[t]))
        .collect();
    println!(
        "probes: {} targets, window stride {stride}, query = even-offset \
         tokens reversed ({} of {} words)",
        targets.len(),
        compiler::WINDOW.div_ceil(2),
        compiler::WINDOW
    );

    let rotation = windows.len() / 2;

    // ---- arm one: content-free (pre-#245 stub reconstruction) ----
    let (h1, m1) = {
        let mut r1 = UorR4Router::new(0.5);
        // Issue #434: arm one reconstructs the PRE-#245 stub, whose stored
        // vectors are banded slices of the session state. Its probe
        // queries come from the same router through `index_sentence`, so
        // the arm is only self-consistent when query and store share a
        // shape — hold this router in the banded mode. Arm two (the
        // production arm) uses the post-#434 full-width default.
        r1.set_banded_storage(true);
        for w in &windows {
            r1.index_sentence_content_free(w, ID);
        }
        let qv = query_vectors(&mut r1, &queries);
        let vectors = aligned_vectors(&r1, &windows);
        let norms: Vec<f64> = vectors.iter().map(|v| norm(v)).collect();
        let ranks: Vec<usize> = targets
            .iter()
            .zip(&qv)
            .map(|(&t, q)| {
                let qn = norm(q);
                let sims: Vec<f64> = vectors
                    .iter()
                    .zip(&norms)
                    .map(|(v, &vn)| cosine(q, v, qn, vn))
                    .collect();
                rank_of(&sims, t, 0)
            })
            .collect();
        metrics(&ranks)
    };

    // ---- arms two and three: content-derived and its shuffled control
    // (one similarity table, rotated index for the control; module docs) ----
    let (h2, m2, h3, m3) = {
        let mut r2 = UorR4Router::new(0.5);
        let corpus_text: String = windows.join(" ");
        let indexed = r2.index_corpus(&corpus_text, ID);
        assert_eq!(
            indexed,
            windows.len(),
            "production bulk surface indexed every distinct window"
        );
        let qv = query_vectors(&mut r2, &queries);
        let vectors = aligned_vectors(&r2, &windows);
        let norms: Vec<f64> = vectors.iter().map(|v| norm(v)).collect();
        let mut ranks2 = Vec::with_capacity(targets.len());
        let mut ranks3 = Vec::with_capacity(targets.len());
        for (&t, q) in targets.iter().zip(&qv) {
            let qn = norm(q);
            let sims: Vec<f64> = vectors
                .iter()
                .zip(&norms)
                .map(|(v, &vn)| cosine(q, v, qn, vn))
                .collect();
            ranks2.push(rank_of(&sims, t, 0));
            ranks3.push(rank_of(&sims, t, rotation));
        }
        let (h2, m2) = metrics(&ranks2);
        let (h3, m3) = metrics(&ranks3);
        (h2, m2, h3, m3)
    };

    println!(
        "memory-lift at corpus scale (issue #423): {} windows, {} probes",
        windows.len(),
        targets.len()
    );
    println!("  arm 1 content-free (pre-#245 stub): top1 {h1:.3} | MRR {m1:.4}");
    println!("  arm 2 content-derived (production):  top1 {h2:.3} | MRR {m2:.4}");
    println!("  arm 3 shuffled-vector control:       top1 {h3:.3} | MRR {m3:.4}");

    // Structural invariants gate; direction prints (module docs).
    for (name, m) in [("arm 1", m1), ("arm 2", m2), ("arm 3", m3)] {
        assert!((0.0..=1.0).contains(&m), "{name} MRR out of range: {m:.4}");
    }

    // ---- pre-declared exit rule (module docs) ----
    let value_carrying = m2 >= m1 + 0.020 && m2 > m3;
    println!(
        "exit rule (#423): arm 2 MRR {m2:.4} vs arm 1 {m1:.4} ({:+.4}, need at least \
         plus 0.020) and vs arm 3 {m3:.4} ({:+.4}, need positive) -> {}",
        m2 - m1,
        m2 - m3,
        if value_carrying {
            "VALUE-CARRYING (#245 restoration)"
        } else {
            "recorded NEGATIVE"
        }
    );
}
