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
//! #399 extension: fusion headroom probes (offset-route, count-confidence,
//! product-of-experts) — instrumentation upper bounds on what a mechanism
//! that consumes forward anchors can add over the causal store.
//!
//! Run:
//!   cargo test -p uor-r4-graph-certify --test anchor_infill -- --ignored --nocapture

use std::collections::BTreeMap;

use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::runtime;

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
    hits: Vec<u64>,
    totals: Vec<u64>,
}

impl Arm {
    fn new(name: &'static str, stride: usize) -> Self {
        Arm {
            name,
            hits: vec![0; stride],
            totals: vec![0; stride],
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
        for offset in 1..self.hits.len() {
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

/// Deepest populated store distribution for a code (the distribution behind
/// `predict_witness_plain`'s argmax).
fn store_dist<'a>(
    store: &'a runtime::Store,
    code: &[u8; compiler::STAGES],
) -> Option<&'a BTreeMap<u32, u32>> {
    for d in (0..=compiler::STAGES).rev() {
        if let Some(dist) = store[d].get(&code[..d]) {
            return Some(dist);
        }
    }
    None
}

/// Smoothed log-probability of `t` under a count distribution.
fn smoothed_ln(dist: &BTreeMap<u32, u64>, total: u64, t: u32) -> f64 {
    let c = dist.get(&t).copied().unwrap_or(0) as f64;
    ((c + 0.5) / (total as f64 + 16_000.0)).ln()
}

/// #399 headroom probe, product-of-experts fusion: argmax over the union of
/// the two distributions' supports of the sum of smoothed log-probabilities.
fn fuse_product(
    store: Option<&BTreeMap<u32, u32>>,
    fwd: Option<&BTreeMap<u32, u64>>,
) -> Option<u32> {
    let s64: Option<BTreeMap<u32, u64>> =
        store.map(|d| d.iter().map(|(&t, &c)| (t, c as u64)).collect());
    match (&s64, fwd) {
        (None, None) => None,
        (Some(s), None) => argmax(s),
        (None, Some(f)) => argmax(f),
        (Some(s), Some(f)) => {
            let st: u64 = s.values().sum();
            let ft: u64 = f.values().sum();
            s.keys()
                .chain(f.keys())
                .map(|&t| {
                    let score = smoothed_ln(s, st, t) + smoothed_ln(f, ft, t);
                    (t, score)
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(b.0.cmp(&a.0)))
                .map(|(t, _)| t)
        }
    }
}

/// The compiled graded code of a token — its geometric address in the
/// artifact's token codebook ([V × STAGES] bytes).
fn token_code(art: &compiler::Compiled, t: u32) -> Option<&[u8]> {
    art.token_codes
        .chunks_exact(compiler::STAGES)
        .nth(t as usize)
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
    let stride: usize = std::env::var("R4_INFILL_STRIDE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);

    println!(
        "anchor-infill Day-0 (#394): {} records, {} stories, anchor stride {}",
        c.n, c.stories, stride
    );

    // ---- construction-partition tables (story < cut ONLY) ----
    let mut unigram: BTreeMap<u32, u64> = BTreeMap::new();
    let mut bigram: BTreeMap<u32, BTreeMap<u32, u64>> = BTreeMap::new();
    // forward-anchor table: (distance to next anchor, anchor token) -> dist.
    // distance d in 1..stride: the target sits d positions before the
    // next pinned token in the reference stream.
    let mut fwd_anchor: BTreeMap<(usize, u32), BTreeMap<u32, u64>> = BTreeMap::new();
    // #399 step 2: E_b-style REGION conditioning — key the forward table by
    // the anchor token's graded-code prefix (its compiled geometric address)
    // instead of its raw identity, at every depth. Statistics are shared
    // across geometrically similar anchors; the held-out comparison against
    // the token-identity table is the "does geometry carry syntax" test.
    let mut fwd_region: BTreeMap<(usize, usize, Vec<u8>), BTreeMap<u32, u64>> = BTreeMap::new();
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
        if !target_pos.is_multiple_of(stride) {
            // next anchor stream index and its token value, if inside story
            let next_anchor_pos = target_pos.next_multiple_of(stride);
            let lookahead = next_anchor_pos - target_pos; // 1..=3
            let j = i + lookahead; // record whose *next* is the anchor token
            if j < c.n && c.story[j] == c.story[i] {
                *fwd_anchor
                    .entry((lookahead, c.next[j]))
                    .or_default()
                    .entry(c.next[i])
                    .or_default() += 1;
                if let Some(code) = token_code(&art, c.next[j]) {
                    for depth in 1..=compiler::STAGES {
                        *fwd_region
                            .entry((lookahead, depth, code[..depth].to_vec()))
                            .or_default()
                            .entry(c.next[i])
                            .or_default() += 1;
                    }
                }
            }
        }
    }
    let unigram_pred = argmax(&unigram);

    // ---- shipped store on the construction partition ----
    let (store, codes) = runtime::build_store(&art, &c);

    // ---- grade all arms on held-out free targets ----
    let mut store_arm = Arm::new("shipped-store", stride);
    let mut unigram_arm = Arm::new("null:unigram", stride);
    let mut bigram_arm = Arm::new("null:bigram", stride);
    let mut prev_anchor_arm = Arm::new("null:prev-anchor-copy", stride);
    let mut fwd_anchor_arm = Arm::new("null:fwd-anchor-table", stride);
    // #399 headroom probes: cheapest possible consumers of forward context,
    // fused with the causal store. Instrumentation upper bounds only.
    let mut route_arm = Arm::new("fuse:offset-route", stride);
    let mut conf_arm = Arm::new("fuse:count-confidence", stride);
    let mut product_arm = Arm::new("fuse:product", stride);
    // #399 step 2 arms: region-conditioned forward tables.
    let mut region_arms: Vec<Arm> = vec![
        Arm::new("null:fwd-region-d1", stride),
        Arm::new("null:fwd-region-d2", stride),
        Arm::new("null:fwd-region-d3", stride),
        Arm::new("null:fwd-region-d4", stride),
    ];
    let mut product_region_arm = Arm::new("fuse:product-region", stride);
    let mut token_key_missing = 0u64;
    let mut region3_key_missing = 0u64;

    for i in 0..c.n {
        if c.story[i] < cut {
            continue;
        }
        let target_pos = positions[i] + 1;
        let offset = target_pos % stride;
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
        let prev_anchor_pos = (target_pos / stride) * stride;
        let back = target_pos - prev_anchor_pos; // 1..=3
        let prev_anchor_tok = if back <= positions[i] + 1 {
            // token at stream index prev_anchor_pos is this record's input
            // stream walked back (input of record i is stream index positions[i])
            runtime::history_token(&c, i, back)
        } else {
            None
        };
        prev_anchor_arm.score(offset, prev_anchor_tok, truth);

        let next_anchor_pos = target_pos.next_multiple_of(stride);
        let lookahead = next_anchor_pos - target_pos;
        let j = i + lookahead;
        let fwd_dist = if j < c.n && c.story[j] == c.story[i] {
            fwd_anchor.get(&(lookahead, c.next[j]))
        } else {
            None
        };
        let fwd_pred_raw = fwd_dist.and_then(argmax);
        fwd_anchor_arm.score(offset, fwd_pred_raw.or(unigram_pred), truth);

        // ---- #399 fusion probes ----
        let store_witness = runtime::predict_witness_plain(&store, &codes[i]);
        let store_pred = Some(store_witness.token);

        // (a) route by offset: forward table owns the pre-anchor position.
        let route_pred = if offset == stride - 1 {
            fwd_pred_raw.or(store_pred)
        } else {
            store_pred
        };
        route_arm.score(offset, route_pred, truth);

        // (b) normalized-confidence pick between the two argmaxes.
        let conf_pred = match fwd_dist {
            Some(f) => {
                let f_total: u64 = f.values().sum();
                let f_max = f.values().copied().max().unwrap_or(0);
                let f_conf = f_max as f64 / (f_total as f64 + 8.0);
                let s_dist = store_dist(&store, &codes[i]);
                let s_total: u64 = s_dist
                    .map(|d| d.values().map(|&c| c as u64).sum())
                    .unwrap_or(0);
                let s_conf = store_witness.count as f64 / (s_total as f64 + 8.0);
                if f_conf > s_conf {
                    fwd_pred_raw.or(store_pred)
                } else {
                    store_pred
                }
            }
            None => store_pred,
        };
        conf_arm.score(offset, conf_pred, truth);

        // (c) product-of-experts over the union support.
        let product_pred = fuse_product(store_dist(&store, &codes[i]), fwd_dist).or(store_pred);
        product_arm.score(offset, product_pred, truth);

        // ---- #399 step 2: region-conditioned forward tables ----
        let anchor_code = if j < c.n && c.story[j] == c.story[i] {
            token_code(&art, c.next[j])
        } else {
            None
        };
        let mut region_dists: [Option<&BTreeMap<u32, u64>>; 4] = [None; 4];
        if let Some(code) = anchor_code {
            for depth in 1..=compiler::STAGES {
                region_dists[depth - 1] =
                    fwd_region.get(&(lookahead, depth, code[..depth].to_vec()));
            }
        }
        for depth in 1..=compiler::STAGES {
            region_arms[depth - 1].score(
                offset,
                region_dists[depth - 1].and_then(argmax).or(unigram_pred),
                truth,
            );
        }
        if fwd_dist.is_none() {
            token_key_missing += 1;
        }
        if region_dists[2].is_none() {
            region3_key_missing += 1;
        }
        // token table when populated, else deepest populated region prefix:
        // the E_b-style backoff chain, product-fused with the causal store.
        let eb_dist = fwd_dist
            .or(region_dists[3])
            .or(region_dists[2])
            .or(region_dists[1])
            .or(region_dists[0]);
        let product_region_pred =
            fuse_product(store_dist(&store, &codes[i]), eb_dist).or(store_pred);
        product_region_arm.score(offset, product_region_pred, truth);
    }

    for arm in [
        &store_arm,
        &unigram_arm,
        &bigram_arm,
        &prev_anchor_arm,
        &fwd_anchor_arm,
        &route_arm,
        &conf_arm,
        &product_arm,
    ] {
        arm.report();
    }
    for arm in &region_arms {
        arm.report();
    }
    product_region_arm.report();
    println!(
        "backoff diagnostics: token-table key missing {token_key_missing} | region-d3 key missing {region3_key_missing}"
    );
}
