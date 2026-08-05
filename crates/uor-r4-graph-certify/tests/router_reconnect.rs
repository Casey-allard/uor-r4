//! Router-anchor reconnection harness (issue #421 M-R1): the first
//! measurement of the ORIGINAL hybrid architecture — the f64 geometric
//! router (`uor-r4-router`, content-bearing angular storage) supplying
//! every-fourth-token content anchors, the graph infill engine
//! (`infill_fill`, the issue-399 A-mode serving surface) filling the
//! free positions between them.
//!
//! # Protocol
//!
//! On the natural stack, under the SAME construction/held-out split as
//! `anchor_infill.rs` (the D3 hash split when `R4_STORIES` is set,
//! otherwise the sequential eighty/twenty story cut), each held-out
//! story becomes a stride-four skeleton: stream index zero is the seed
//! (the true first token, identical in every arm), stream indices that
//! are multiples of four are anchors, everything else is free. Anchor
//! positions carry, per arm:
//!
//! - ARM A (ceiling): the true corpus token.
//! - ARM B (router): the ROUTER-selected token (mapping below).
//! - ARM C (null): the construction-split unigram-argmax token.
//! - ARM D (null): ARM B's tokens rotated to wrong positions by a
//!   fixed half-length rotation within each story — the issue-273
//!   falsifier control (same router outputs, wrong positions).
//!
//! Each skeleton is filled with `score_runtime::infill_fill` against a
//! scored R4G1 artifact produced by the score pipeline on this corpus
//! and split (`R4_SCORED_R4G1`; its FWDA section IS the forward
//! channel). Grading: per-arm anchor accuracy (arm token equals the
//! true anchor token) and free-position top-one versus corpus truth.
//!
//! # PRE-DECLARED EXIT RULE
//!
//! ARM B is a POSITIVE signal if and only if its free-position top-one
//! exceeds ARM C by at least two percentage points AND exceeds ARM D
//! (routed beats shuffled — the issue-273 claim). Anything less is a
//! recorded negative.
//!
//! # Router-to-token mapping (the design under measurement)
//!
//! The router's content-bearing store is text-in (`index_corpus` /
//! `index_sentence`), but its internal representation is word-identity
//! based: word-to-prime assignment is arrival-ordered and the seeded
//! word vector depends only on the assigned prime, so the router never
//! consumes natural-language meaning — only word identity and
//! co-occurrence. No text source exists for this corpus (the corpus
//! records carry span/byte OFFSETS but no byte store, and the obs-pass
//! `stories.jsonl` carries story metadata only), so each token id is
//! rendered as a synthetic word (`t00042`), a bijection that preserves
//! exactly the structure the router consumes. This is the honest
//! pairing; it is documented here because it is a mapping choice, not
//! a property of the router.
//!
//! Ingestion: for every construction-split anchor with a full
//! CTX-token predecessor context, that context (rendered as words) is
//! indexed as one sentence via the router's production bulk surface
//! (`index_corpus`), and a side table records the anchor tokens that
//! followed each distinct context in construction. Query: at each
//! held-out anchor position, the TRUE preceding CTX tokens are rendered
//! the same way and turned into a content-derived query vector through
//! the router's own indexing surface (scratch identity — the
//! issue-255 memory-lift methodology), the stored construction context
//! with the highest cosine is retrieved, and the router-selected token
//! is that context's majority anchor continuation (ties to the lowest
//! token id). Limitations, stated up front: (a) the query context is
//! oracle (true held-out tokens, including free positions the engine
//! must later fill) — ARM B is therefore an upper bound on pure-
//! generation router anchor supply; (b) retrieval is cosine over the
//! stored content-derived state vectors (the #255 A/B methodology)
//! rather than `get_top_resonances_native`, whose per-item allocation
//! makes it infeasible at this scale — cosine over the same stored
//! vectors is the content-bearing core of that path.
//!
//! # CAPS (documented, no silent truncation)
//!
//! Full-split router ingestion is infeasible (the store keeps two
//! 512-dim f64 copies per sentence), so the router ingests the first
//! `R4_RECONNECT_CONSTR_STORIES` construction stories (default two
//! thousand) and the harness evaluates the first
//! `R4_RECONNECT_HELD_STORIES` held-out stories (default two hundred),
//! both by ascending story id. The unigram null and the scored
//! artifact use the FULL construction split (the pipeline's own
//! basis), so the router arm is, if anything, information-starved
//! relative to the nulls — a positive under the cap is real; a
//! negative is recorded as negative-at-cap. The caps are printed in
//! the report.
//!
//! Run (natural stack):
//!   R4_CORPUS_META=/tmp/c_meta.bin R4_CORPUS_RECS=/tmp/c_recs.bin \
//!   R4_STORIES=/tmp/wiki-obs/stories.jsonl \
//!   R4_ARTIFACTS=/tmp/tless_artifacts.bin \
//!   R4_SCORED_R4G1=/tmp/strict_score/score.r4g1 \
//!   cargo test --release -p uor-r4-graph-certify --test router_reconnect -- --ignored --nocapture

use std::collections::BTreeMap;

use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_certify::score;
use uor_r4_graph_certify::score_runtime::{infill_fill, GraphScorer};
use uor_r4_router::UorR4Router;

/// Anchor stride of the infill protocol (issues 394/399/421).
const STRIDE: usize = 4;
/// Router context length in tokens (matches the engine window).
const CTX: usize = compiler::WINDOW;
/// Identity scope of the construction-split router store.
const IDENT: &str = "user:reconnect";

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

/// A token context rendered as router sentence text (no terminator).
fn render_context(tokens: &[u32]) -> String {
    let words: Vec<String> = tokens.iter().map(|&token| token_word(token)).collect();
    words.join(" ")
}

/// Highest count, ties to the lowest token — the harness-standard argmax.
fn majority(dist: &BTreeMap<u32, u32>) -> Option<u32> {
    dist.iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&token, _)| token)
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

/// Per-arm grading: anchor accuracy plus free-position top-one by
/// stride offset (offset = stream index modulo the stride, one..three).
struct Arm {
    name: &'static str,
    anchor_hits: u64,
    anchor_total: u64,
    free_hits: [u64; STRIDE],
    free_total: [u64; STRIDE],
}

impl Arm {
    fn new(name: &'static str) -> Self {
        Arm {
            name,
            anchor_hits: 0,
            anchor_total: 0,
            free_hits: [0; STRIDE],
            free_total: [0; STRIDE],
        }
    }
    fn grade(&mut self, skeleton: &[Option<u32>], filled: &[u32], truth: &[u32]) {
        for (index, (&truth_token, filled_token)) in truth.iter().zip(filled).enumerate() {
            if index == 0 {
                continue; // shared seed, never graded
            }
            if index.is_multiple_of(STRIDE) {
                self.anchor_total += 1;
                if skeleton[index] == Some(truth_token) {
                    self.anchor_hits += 1;
                }
            } else {
                let offset = index % STRIDE;
                self.free_total[offset] += 1;
                if *filled_token == truth_token {
                    self.free_hits[offset] += 1;
                }
            }
        }
    }
    fn free_top1(&self) -> f64 {
        let hits: u64 = self.free_hits.iter().sum();
        let total: u64 = self.free_total.iter().sum();
        100.0 * hits as f64 / total.max(1) as f64
    }
    fn report(&self) {
        let pct = |h: u64, t: u64| 100.0 * h as f64 / t.max(1) as f64;
        print!(
            "{:<22} anchor-acc {:>5.1}% ({}/{}) | free top1 {:>5.1}% on {} |",
            self.name,
            pct(self.anchor_hits, self.anchor_total),
            self.anchor_hits,
            self.anchor_total,
            self.free_top1(),
            self.free_total.iter().sum::<u64>()
        );
        for offset in 1..STRIDE {
            print!(
                " off{offset} {:>5.1}%",
                pct(self.free_hits[offset], self.free_total[offset])
            );
        }
        println!();
    }
}

#[test]
#[ignore = "M-R1 measurement harness; run explicitly with --ignored"]
fn router_reconnect_m_r1() {
    // ---- natural stack ----
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let c = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    println!("corpus: {meta_path} + {recs_path} ({} records)", c.n);
    let art_path = std::env::var("R4_ARTIFACTS").unwrap_or_else(|_| fixture("tless_artifacts.bin"));
    let teacher_bytes = std::fs::read(&art_path).expect("teacher artifact container");
    let artifacts = compiler::parse_artifacts(&teacher_bytes).expect("TLA artifact container");
    let r4g1_path = std::env::var("R4_SCORED_R4G1")
        .expect("set R4_SCORED_R4G1 to a score-pipeline R4G1 built on this corpus and split");
    let r4g1 = std::fs::read(&r4g1_path).expect("scored R4G1 bytes");
    let scorer = GraphScorer::from_artifact(
        &r4g1,
        Some(&teacher_bytes),
        score::DEFAULT_ROOT_TOP_B,
        score::DEFAULT_EXCT_TOP_X,
    )
    .expect("scorer from scored artifact + matching teacher");
    let rotations = runtime::derive_rotations();
    println!(
        "artifact: {r4g1_path} ({} fwd-anchor rows) | teacher: {art_path}",
        scorer.forward_anchor_row_count()
    );
    assert!(
        scorer.forward_anchor_row_count() > 0,
        "scored artifact carries no FWDA section — infill would degrade to base scoring; \
         rebuild the artifact with the current score pipeline"
    );

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

    // ---- full-construction-split unigram null (the pipeline's basis) ----
    let mut unigram: BTreeMap<u32, u32> = BTreeMap::new();
    for (&sid, &next) in c.story.iter().zip(&c.next).take(c.n) {
        if constr[sid as usize] {
            *unigram.entry(next).or_default() += 1;
        }
    }
    let unigram_pred = majority(&unigram).expect("non-empty construction split");

    // ---- CAPS (module docs; printed, never silent) ----
    let constr_cap = env_usize("R4_RECONNECT_CONSTR_STORIES", 2_000);
    let held_cap = env_usize("R4_RECONNECT_HELD_STORIES", 200);
    let constr_streams: Vec<&(u32, Vec<u32>)> = streams
        .iter()
        .filter(|(sid, _)| constr[*sid as usize])
        .take(constr_cap)
        .collect();
    let held_streams: Vec<&(u32, Vec<u32>)> = streams
        .iter()
        .filter(|(sid, _)| !constr[*sid as usize])
        .take(held_cap)
        .collect();
    let constr_total = streams.iter().filter(|(s, _)| constr[*s as usize]).count();
    let held_total = streams.len() - constr_total;
    println!(
        "CAPS: router ingests {} of {} construction stories; eval on {} of {} held-out stories \
         (R4_RECONNECT_CONSTR_STORIES / R4_RECONNECT_HELD_STORIES override)",
        constr_streams.len(),
        constr_total,
        held_streams.len(),
        held_total
    );

    // ---- router ingestion: one sentence per distinct anchor context ----
    // Side table: rendered context sentence (with the terminator the
    // router's sentence splitter stores) -> anchor-continuation counts.
    let mut side_table: BTreeMap<String, BTreeMap<u32, u32>> = BTreeMap::new();
    for (_, stream) in &constr_streams {
        for t in (STRIDE..stream.len()).step_by(STRIDE) {
            if t >= CTX {
                let sentence = format!("{}.", render_context(&stream[t - CTX..t]));
                *side_table
                    .entry(sentence)
                    .or_default()
                    .entry(stream[t])
                    .or_default() += 1;
            }
        }
    }
    let corpus_text: String = side_table
        .keys()
        .cloned()
        .collect::<Vec<String>>()
        .join(" ");
    let mut router = UorR4Router::new(0.5);
    let indexed = router.index_corpus(&corpus_text, IDENT);
    println!(
        "router store: {} distinct anchor contexts, {} indexed sentences",
        side_table.len(),
        indexed
    );

    // ---- held-out anchor queries (oracle context, module docs) ----
    // Phase one: content-derived query vectors through the router's own
    // indexing surface (scratch identities, the issue-255 pattern).
    // query_slots[story][anchor] = Ok(token) fallback or Err(query id).
    let mut query_vectors: Vec<Vec<f64>> = Vec::new();
    let mut query_slots: Vec<Vec<Result<u32, usize>>> = Vec::new();
    let mut fallback_short_context = 0u64;
    for (_, stream) in &held_streams {
        let mut slots = Vec::new();
        for t in (STRIDE..stream.len()).step_by(STRIDE) {
            if t >= CTX {
                let scratch = format!("user:q{}", query_vectors.len());
                router.index_sentence(&render_context(&stream[t - CTX..t]), &scratch);
                let items = router.corpus_items_for(&scratch);
                match items.first() {
                    Some(item) => {
                        slots.push(Err(query_vectors.len()));
                        query_vectors.push(item.state_vector.clone());
                    }
                    None => {
                        fallback_short_context += 1;
                        slots.push(Ok(unigram_pred));
                    }
                }
            } else {
                fallback_short_context += 1;
                slots.push(Ok(unigram_pred));
            }
        }
        query_slots.push(slots);
    }

    // Phase two: cosine retrieval over the stored construction vectors,
    // deterministic (items sorted by sentence; strict-greater keeps the
    // first, so ties break to the lexicographically first sentence).
    let mut items: Vec<(&str, &[f64])> = router
        .corpus_items_for(IDENT)
        .into_iter()
        .map(|item| (item.sentence.as_str(), item.state_vector.as_slice()))
        .collect();
    items.sort_by_key(|(sentence, _)| *sentence);
    let mut side_table_miss = 0u64;
    let entries: Vec<(f64, &[f64], u32)> = items
        .iter()
        .filter_map(
            |(sentence, vector)| match side_table.get(*sentence).and_then(majority) {
                Some(token) => Some((norm(vector), *vector, token)),
                None => {
                    side_table_miss += 1;
                    None
                }
            },
        )
        .collect();
    let mut weak_retrievals = 0u64;
    let routed_tokens: Vec<u32> = query_vectors
        .iter()
        .map(|q| {
            let qn = norm(q);
            let mut best_sim = f64::NEG_INFINITY;
            let mut best_token = unigram_pred;
            for &(vn, vector, token) in &entries {
                let sim = cosine(q, vector, qn, vn);
                if sim > best_sim {
                    best_sim = sim;
                    best_token = token;
                }
            }
            if best_sim <= 0.0 {
                weak_retrievals += 1;
            }
            best_token
        })
        .collect();
    println!(
        "queries: {} routed, {fallback_short_context} short-context fallbacks, \
         {weak_retrievals} zero-cosine retrievals, {side_table_miss} store items \
         without side-table row",
        routed_tokens.len()
    );

    // ---- fill and grade the four arms ----
    let mut arm_true = Arm::new("ARM A true-anchor");
    let mut arm_router = Arm::new("ARM B router-anchor");
    let mut arm_unigram = Arm::new("ARM C unigram-anchor");
    let mut arm_shuffled = Arm::new("ARM D shuffled-router");
    let mut single_anchor_stories = 0u64;
    for ((_, stream), slots) in held_streams.iter().zip(&query_slots) {
        let truth = stream.as_slice();
        let anchors_b: Vec<u32> = slots
            .iter()
            .map(|slot| match slot {
                Ok(token) => *token,
                Err(query) => routed_tokens[*query],
            })
            .collect();
        // Fixed half-length rotation (the issue-273 falsifier control);
        // identity on stories with fewer than two anchors (counted).
        let m = anchors_b.len();
        if m < 2 {
            single_anchor_stories += 1;
        }
        let anchors_d: Vec<u32> = (0..m).map(|k| anchors_b[(k + m / 2) % m]).collect();

        let skeleton_for = |anchor_of: &dyn Fn(usize, usize) -> u32| -> Vec<Option<u32>> {
            truth
                .iter()
                .enumerate()
                .map(|(index, &token)| {
                    if index == 0 {
                        Some(token) // shared seed
                    } else if index.is_multiple_of(STRIDE) {
                        Some(anchor_of(index / STRIDE - 1, index))
                    } else {
                        None
                    }
                })
                .collect()
        };
        let sk_a = skeleton_for(&|_, index| truth[index]);
        let sk_b = skeleton_for(&|slot, _| anchors_b[slot]);
        let sk_c = skeleton_for(&|_, _| unigram_pred);
        let sk_d = skeleton_for(&|slot, _| anchors_d[slot]);
        for (arm, skeleton) in [
            (&mut arm_true, &sk_a),
            (&mut arm_router, &sk_b),
            (&mut arm_unigram, &sk_c),
            (&mut arm_shuffled, &sk_d),
        ] {
            let filled =
                infill_fill(&scorer, &artifacts, &rotations, skeleton).expect("infill fill");
            arm.grade(skeleton, &filled, truth);
        }
    }

    println!(
        "router-reconnect M-R1 (#421): stride {STRIDE}, context {CTX} tokens, \
         {single_anchor_stories} stories with fewer than two anchors (shuffle = identity there)"
    );
    for arm in [&arm_true, &arm_router, &arm_unigram, &arm_shuffled] {
        arm.report();
    }

    // ---- pre-declared exit rule (module docs) ----
    let b = arm_router.free_top1();
    let c_null = arm_unigram.free_top1();
    let d = arm_shuffled.free_top1();
    let positive = b >= c_null + 2.0 && b > d;
    println!(
        "exit rule (#421 M-R1): ARM B {b:.2}% vs ARM C {c_null:.2}% ({:+.2}pp, need at least \
         two) and vs ARM D {d:.2}% ({:+.2}pp, need positive) -> {}",
        b - c_null,
        b - d,
        if positive {
            "POSITIVE signal"
        } else {
            "recorded NEGATIVE"
        }
    );
}
