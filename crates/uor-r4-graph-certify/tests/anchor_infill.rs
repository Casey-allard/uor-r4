//! Anchor-infill criterion, Day-0 harness (issue #394).
//!
//! Grades constrained syntax completion between routed content anchors on
//! the checked-in fixture corpus: every 4th token of each story's reference
//! stream is a pinned anchor (mirroring the original hybrid's injection
//! cadence); the remaining positions are free and are what gets scored.
//!
//! Day-0 scope: the shipped causal store is graded on the free-position
//! slice exactly as-is (it sees past tokens only), against a null ladder
//! that includes a *forward-anchor-conditioned* table — the cheapest
//! possible proxy for how much syntax information the next anchor carries.
//! No serving-path changes; certifier-side instrumentation only (f32 and
//! allocation permitted here, never in the kernel).
//!
//! Run:
//!   cargo test -p uor-r4-graph-certify --test anchor_infill -- --ignored --nocapture

use std::collections::BTreeMap;

use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::runtime;

const ANCHOR_STRIDE: usize = 4;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// Stream position of every record inside its story (0-based token index of
/// the record's *input* token). Records are sequential per story.
fn story_positions(c: &compiler::Corpus) -> Vec<usize> {
    let mut positions = Vec::with_capacity(c.n);
    let mut current_story = u32::MAX;
    let mut pos = 0usize;
    for i in 0..c.n {
        if c.story[i] != current_story {
            current_story = c.story[i];
            pos = 0;
        } else {
            pos += 1;
        }
        positions.push(pos);
    }
    positions
}

struct Arm {
    name: &'static str,
    hits: [u64; ANCHOR_STRIDE],
    totals: [u64; ANCHOR_STRIDE],
}

impl Arm {
    fn new(name: &'static str) -> Self {
        Arm {
            name,
            hits: [0; ANCHOR_STRIDE],
            totals: [0; ANCHOR_STRIDE],
        }
    }
    fn score(&mut self, offset: usize, pred: Option<u32>, truth: u32) {
        self.totals[offset] += 1;
        if pred == Some(truth) {
            self.hits[offset] += 1;
        }
    }
    fn report(&self) {
        let (hits, totals): (u64, u64) = (self.hits.iter().sum(), self.totals.iter().sum());
        let pct = |h: u64, t: u64| {
            if t == 0 {
                0.0
            } else {
                100.0 * h as f64 / t as f64
            }
        };
        print!(
            "{:<24} top1 {:>5.1}% on {} free targets |",
            self.name,
            pct(hits, totals),
            totals
        );
        for offset in 1..ANCHOR_STRIDE {
            print!(
                " off{offset} {:>5.1}%",
                pct(self.hits[offset], self.totals[offset])
            );
        }
        println!();
    }
}

fn argmax(dist: &BTreeMap<u32, u64>) -> Option<u32> {
    dist.iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&tok, _)| tok)
}

#[test]
#[ignore = "Day-0 measurement harness; run explicitly with --ignored"]
fn anchor_infill_day0() {
    let c = compiler::load_corpus_from(&fixture("c_meta.bin"), &fixture("c_recs.bin"))
        .expect("checked-in fixture corpus");
    let art = compiler::load_artifacts_from(&fixture("tless_artifacts.bin"))
        .expect("checked-in fixture artifacts");
    let cut = (c.stories as f64 * 0.8) as u32;
    let positions = story_positions(&c);

    println!(
        "anchor-infill Day-0 (#394): {} records, {} stories, anchor stride {}",
        c.n, c.stories, ANCHOR_STRIDE
    );

    // ---- construction-partition tables (story < cut ONLY) ----
    let mut unigram: BTreeMap<u32, u64> = BTreeMap::new();
    let mut bigram: BTreeMap<u32, BTreeMap<u32, u64>> = BTreeMap::new();
    // forward-anchor table: (distance to next anchor, anchor token) -> dist.
    // distance d in 1..ANCHOR_STRIDE: the target sits d positions before the
    // next pinned token in the reference stream.
    let mut fwd_anchor: BTreeMap<(usize, u32), BTreeMap<u32, u64>> = BTreeMap::new();
    #[allow(clippy::needless_range_loop)] // index i addresses four parallel corpus arrays
    for i in 0..c.n {
        if c.story[i] >= cut {
            continue;
        }
        *unigram.entry(c.next[i]).or_default() += 1;
        *bigram
            .entry(c.input[i])
            .or_default()
            .entry(c.next[i])
            .or_default() += 1;
        // target stream index of this record's prediction:
        let target_pos = positions[i] + 1;
        if !target_pos.is_multiple_of(ANCHOR_STRIDE) {
            // next anchor stream index and its token value, if inside story
            let next_anchor_pos = target_pos.next_multiple_of(ANCHOR_STRIDE);
            let lookahead = next_anchor_pos - target_pos; // 1..=3
            let j = i + lookahead; // record whose *next* is the anchor token
            if j < c.n && c.story[j] == c.story[i] {
                *fwd_anchor
                    .entry((lookahead, c.next[j]))
                    .or_default()
                    .entry(c.next[i])
                    .or_default() += 1;
            }
        }
    }
    let unigram_pred = argmax(&unigram);

    // ---- shipped store on the construction partition ----
    let (store, codes) = runtime::build_store(&art, &c);

    // ---- grade all arms on held-out free targets ----
    let mut store_arm = Arm::new("shipped-store");
    let mut unigram_arm = Arm::new("null:unigram");
    let mut bigram_arm = Arm::new("null:bigram");
    let mut prev_anchor_arm = Arm::new("null:prev-anchor-copy");
    let mut fwd_anchor_arm = Arm::new("null:fwd-anchor-table");

    for i in 0..c.n {
        if c.story[i] < cut {
            continue;
        }
        let target_pos = positions[i] + 1;
        let offset = target_pos % ANCHOR_STRIDE;
        if offset == 0 {
            continue; // target is a pinned anchor: given, not graded
        }
        let truth = c.next[i];

        store_arm.score(
            offset,
            Some(runtime::predict_plain(&store, &codes[i])),
            truth,
        );
        unigram_arm.score(offset, unigram_pred, truth);
        bigram_arm.score(
            offset,
            bigram.get(&c.input[i]).and_then(argmax).or(unigram_pred),
            truth,
        );
        // previous anchor token: the reference token at the last pinned
        // stream index at or before target_pos.
        let prev_anchor_pos = (target_pos / ANCHOR_STRIDE) * ANCHOR_STRIDE;
        let back = target_pos - prev_anchor_pos; // 1..=3
        let prev_anchor_tok = if back <= positions[i] + 1 {
            // token at stream index prev_anchor_pos is this record's input
            // stream walked back (input of record i is stream index positions[i])
            runtime::history_token(&c, i, back)
        } else {
            None
        };
        prev_anchor_arm.score(offset, prev_anchor_tok, truth);

        let next_anchor_pos = target_pos.next_multiple_of(ANCHOR_STRIDE);
        let lookahead = next_anchor_pos - target_pos;
        let j = i + lookahead;
        let fwd_pred = if j < c.n && c.story[j] == c.story[i] {
            fwd_anchor
                .get(&(lookahead, c.next[j]))
                .and_then(argmax)
                .or(unigram_pred)
        } else {
            unigram_pred
        };
        fwd_anchor_arm.score(offset, fwd_pred, truth);
    }

    for arm in [
        &store_arm,
        &unigram_arm,
        &bigram_arm,
        &prev_anchor_arm,
        &fwd_anchor_arm,
    ] {
        arm.report();
    }
}
