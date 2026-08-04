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

/// Granularity direction (#393): grade predictions by REGION of token —
/// prefix equality of the compiled token codes at depth 2 and 3 — instead
/// of exact identity. Measures semantic-neighborhood routing that
/// token-exact scoring cannot see.
struct RegionScore {
    name: &'static str,
    hit1: u64,
    hit2: u64,
    hit3: u64,
    total: u64,
}

impl RegionScore {
    fn new(name: &'static str) -> Self {
        RegionScore {
            name,
            hit1: 0,
            hit2: 0,
            hit3: 0,
            total: 0,
        }
    }
    fn score(&mut self, art: &compiler::Compiled, pred: Option<u32>, truth: u32) {
        self.total += 1;
        let (Some(p), Some(pt), Some(tt)) = (
            pred,
            pred.and_then(|p| token_code(art, p)),
            token_code(art, truth),
        ) else {
            return;
        };
        let _ = p;
        if pt[0] == tt[0] {
            self.hit1 += 1;
        }
        if pt[..2] == tt[..2] {
            self.hit2 += 1;
        }
        if pt[..3] == tt[..3] {
            self.hit3 += 1;
        }
    }
    fn report(&self) {
        println!(
            "region-match {:<20} d1 {:>5.1}% | d2 {:>5.1}% | d3 {:>5.1}% (n={})",
            self.name,
            100.0 * self.hit1 as f64 / self.total.max(1) as f64,
            100.0 * self.hit2 as f64 / self.total.max(1) as f64,
            100.0 * self.hit3 as f64 / self.total.max(1) as f64,
            self.total
        );
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

/// λ-smoothed backoff-mixture probability of `truth` under the store —
/// the same rule the certify `eval` WB metric uses.
fn store_prob(store: &runtime::Store, code: &[u8; compiler::STAGES], truth: u32) -> f64 {
    let mut lams: Vec<(f64, f64)> = Vec::new(); // (lambda, p(truth) at level)
    for d in 0..=compiler::STAGES {
        if let Some(dist) = store[d].get(&code[..d]) {
            let total: u32 = dist.values().sum();
            let lam = total as f64 / (total as f64 + dist.len() as f64);
            let pt = dist.get(&truth).copied().unwrap_or(0) as f64 / total as f64;
            lams.push((lam, pt));
        }
    }
    let mut rem = 1.0f64;
    let mut acc = 0.0f64;
    for li in (0..lams.len()).rev() {
        let w = rem * lams[li].0;
        rem *= 1.0 - lams[li].0;
        acc += w * lams[li].1;
    }
    (acc + rem / 32_000.0).max(1e-30)
}

/// Smoothed probability of `truth` under a count table (add-half, 16k floor).
fn table_prob(dist: Option<&BTreeMap<u32, u64>>, truth: u32) -> f64 {
    match dist {
        None => 1.0 / 32_000.0,
        Some(d) => {
            let total: u64 = d.values().sum();
            let c = d.get(&truth).copied().unwrap_or(0) as f64;
            ((c + 0.5) / (total as f64 + 16_000.0)).max(1e-30)
        }
    }
}

/// Normalized product-of-experts probability of `truth`: renormalize the
/// product of the two smoothed channels over the union support plus a
/// uniform out-of-support remainder.
fn product_prob(
    store: &runtime::Store,
    code: &[u8; compiler::STAGES],
    fwd: Option<&BTreeMap<u32, u64>>,
    truth: u32,
) -> f64 {
    let sd = store_dist(store, code);
    let (Some(s), Some(f)) = (sd, fwd) else {
        // single-channel fallback
        return match (sd, fwd) {
            (Some(_), None) => store_prob(store, code, truth),
            (None, Some(f)) => table_prob(Some(f), truth),
            _ => 1.0 / 32_000.0,
        };
    };
    let st: u64 = s.values().map(|&c| c as u64).sum();
    let ft: u64 = f.values().sum();
    let sp = |t: u32| {
        let c = s.get(&t).copied().unwrap_or(0) as f64;
        (c + 0.5) / (st as f64 + 16_000.0)
    };
    let fp = |t: u32| {
        let c = f.get(&t).copied().unwrap_or(0) as f64;
        (c + 0.5) / (ft as f64 + 16_000.0)
    };
    let mut z = 0.0f64;
    let mut in_support = false;
    let mut pt = 0.0f64;
    for &t in s.keys().chain(f.keys()) {
        let p = sp(t) * fp(t);
        z += p;
        if t == truth {
            in_support = true;
            pt = p;
        }
    }
    // out-of-support floor: both channels at their smoothing floor
    let floor = (0.5 / (st as f64 + 16_000.0)) * (0.5 / (ft as f64 + 16_000.0));
    let z_full = z + floor * 32_000.0;
    if in_support {
        (pt / z_full).max(1e-30)
    } else {
        (floor / z_full).max(1e-30)
    }
}

/// Product-of-experts over any number of smoothed count channels: argmax
/// over the union support of the sum of smoothed log-probabilities.
fn fuse_product_multi(channels: &[(&BTreeMap<u32, u64>, u64)]) -> Option<u32> {
    if channels.is_empty() {
        return None;
    }
    let mut support: Vec<u32> = Vec::new();
    for (d, _) in channels {
        support.extend(d.keys().copied());
    }
    support.sort_unstable();
    support.dedup();
    support
        .into_iter()
        .map(|t| {
            let score: f64 = channels
                .iter()
                .map(|(d, total)| smoothed_ln(d, *total, t))
                .sum();
            (t, score)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(b.0.cmp(&a.0)))
        .map(|(t, _)| t)
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
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let c = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    println!("corpus: {meta_path} + {recs_path}");
    let art_path = std::env::var("R4_ARTIFACTS").unwrap_or_else(|_| fixture("tless_artifacts.bin"));
    let art = compiler::load_artifacts_from(&art_path).expect("artifacts");
    println!("artifacts: {art_path}");
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
    // Ari's question (#399/#394 thread): the rows-inclusive causal baseline.
    // Certifier-side proxy for the NGRAM trigram rows: exact-context
    // (prev2, prev1) -> next counts from the construction partition.
    let mut trigram: BTreeMap<(u32, u32), BTreeMap<u32, u64>> = BTreeMap::new();
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
        if let Some(p2) = runtime::history_token(&c, i, 2) {
            *trigram
                .entry((p2, c.input[i]))
                .or_default()
                .entry(c.next[i])
                .or_default() += 1;
        }
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
    // Observed-evidence variant: SAME codes, but evidence = actual corpus
    // continuations (c.next counts) instead of teacher top-k. On natural
    // text the teacher is off-distribution and teacher-k evidence collapses
    // the store; this arm isolates evidence source from addressing geometry.
    let mut store_obs: runtime::Store = (0..=compiler::STAGES).map(|_| BTreeMap::new()).collect();
    #[allow(clippy::needless_range_loop)] // parallel corpus arrays
    for i2 in 0..c.n {
        if c.story[i2] >= cut {
            continue;
        }
        for (d, level) in store_obs.iter_mut().enumerate() {
            *level
                .entry(codes[i2][..d].to_vec())
                .or_default()
                .entry(c.next[i2])
                .or_default() += 1;
        }
    }

    // ---- #399 M0: region-mediated forward channel (E_b-composed, distance-tagged) ----
    // emit_index: token -> (context-region prefix P) -> count, construction only.
    // walk table: (distance, emitting-region prefix) -> target-token counts.
    const M0_P: usize = 3;
    const M0_TOPR: usize = 8;
    let mut emit_index: BTreeMap<u32, BTreeMap<Vec<u8>, u32>> = BTreeMap::new();
    let mut m0_walk: BTreeMap<(usize, Vec<u8>), BTreeMap<u32, u64>> = BTreeMap::new();
    #[allow(clippy::needless_range_loop)] // parallel corpus arrays
    for j in 0..c.n {
        if c.story[j] >= cut {
            continue;
        }
        let rj = codes[j][..M0_P].to_vec();
        *emit_index
            .entry(c.next[j])
            .or_default()
            .entry(rj.clone())
            .or_default() += 1;
        // j emits token next[j]; if that token sits at an anchor stream position,
        // record the targets at distances 1..stride before it.
        let emit_pos = positions[j] + 1;
        if emit_pos.is_multiple_of(stride) {
            for d in 1..stride {
                if j >= d && c.story[j - d] == c.story[j] {
                    *m0_walk
                        .entry((d, rj.clone()))
                        .or_default()
                        .entry(c.next[j - d])
                        .or_default() += 1;
                }
            }
        }
    }

    // ---- grade all arms on held-out free targets ----
    let mut store_arm = Arm::new("shipped-store", stride);
    let mut store_obs_arm = Arm::new("store-observed-ev", stride);
    let mut obs_fuse_arm = Arm::new("fuse:obs*tri*m0", stride);
    let mut rg_unigram = RegionScore::new("null:unigram");
    let mut rg_trigram = RegionScore::new("null:trigram");
    let mut rg_obs = RegionScore::new("store-observed-ev");
    let mut rg_m0 = RegionScore::new("m0:region-walk");
    let mut rg_obs_fuse = RegionScore::new("fuse:obs*tri*m0");
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
    // #394 bits ladder on free targets
    let (mut bits_store, mut bits_fwd, mut bits_prod, mut bits_uni, mut bits_n) =
        (0f64, 0f64, 0f64, 0f64, 0u64);
    let mut tri_arm = Arm::new("null:trigram", stride);
    let mut rows_proxy_arm = Arm::new("rows-proxy:tri>store", stride);
    let mut prod_tri_arm = Arm::new("fuse:store*tri", stride);
    let mut prod_tri_fwd_arm = Arm::new("fuse:store*tri*fwd", stride);
    let mut m0_arm = Arm::new("m0:region-walk", stride);
    let mut m0_fuse_arm = Arm::new("fuse:store*tri*m0", stride);
    let uni_total: u64 = unigram.values().sum();
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

        // ---- rows-inclusive causal baseline + decisive fusion ----
        let tri_dist =
            runtime::history_token(&c, i, 2).and_then(|p2| trigram.get(&(p2, c.input[i])));
        tri_arm.score(offset, tri_dist.and_then(argmax).or(unigram_pred), truth);
        rows_proxy_arm.score(offset, tri_dist.and_then(argmax).or(store_pred), truth);
        let store_u64: Option<BTreeMap<u32, u64>> = store_dist(&store, &codes[i])
            .map(|d| d.iter().map(|(&t, &cn)| (t, cn as u64)).collect());
        let mut channels: Vec<(&BTreeMap<u32, u64>, u64)> = Vec::new();
        if let Some(sd) = &store_u64 {
            channels.push((sd, sd.values().sum()));
        }
        if let Some(td) = tri_dist {
            channels.push((td, td.values().sum()));
        }
        prod_tri_arm.score(offset, fuse_product_multi(&channels).or(store_pred), truth);
        if let Some(fd) = fwd_dist {
            channels.push((fd, fd.values().sum()));
        }
        prod_tri_fwd_arm.score(offset, fuse_product_multi(&channels).or(store_pred), truth);

        // ---- observed-evidence store arms ----
        let obs_dist = store_dist(&store_obs, &codes[i]);
        let obs_u64: Option<BTreeMap<u32, u64>> =
            obs_dist.map(|d| d.iter().map(|(&t, &cn)| (t, cn as u64)).collect());
        let obs_pred_cache = obs_dist
            .and_then(|d| {
                d.iter()
                    .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
                    .map(|(&t, _)| t)
            })
            .or(unigram_pred);
        store_obs_arm.score(offset, obs_pred_cache, truth);

        // ---- #399 M0 eval: anchor token -> top emitting regions -> walk mixture ----
        let m0_dist: Option<BTreeMap<u32, u64>> = if j < c.n && c.story[j] == c.story[i] {
            emit_index.get(&c.next[j]).map(|regions| {
                let mut top: Vec<(&Vec<u8>, &u32)> = regions.iter().collect();
                top.sort_by_key(|(_, &cnt)| std::cmp::Reverse(cnt));
                let mut mix: BTreeMap<u32, u64> = BTreeMap::new();
                for (region, &rcnt) in top.into_iter().take(M0_TOPR) {
                    if let Some(dist) = m0_walk.get(&(lookahead, region.clone())) {
                        let total: u64 = dist.values().sum();
                        for (&t, &cnt) in dist {
                            // weight each region's contribution by its emit count,
                            // normalized per-region (integer-scaled mixture)
                            *mix.entry(t).or_default() += (cnt * rcnt as u64 * 1000) / total.max(1);
                        }
                    }
                }
                mix
            })
        } else {
            None
        };
        let m0_ref = m0_dist.as_ref().filter(|m| !m.is_empty());
        m0_arm.score(offset, m0_ref.and_then(argmax).or(unigram_pred), truth);
        let mut m0_channels: Vec<(&BTreeMap<u32, u64>, u64)> = Vec::new();
        if let Some(sd) = &store_u64 {
            m0_channels.push((sd, sd.values().sum()));
        }
        if let Some(td) = tri_dist {
            m0_channels.push((td, td.values().sum()));
        }
        if let Some(md) = m0_ref {
            m0_channels.push((md, md.values().sum()));
        }
        m0_fuse_arm.score(
            offset,
            fuse_product_multi(&m0_channels).or(store_pred),
            truth,
        );
        let mut obs_channels: Vec<(&BTreeMap<u32, u64>, u64)> = Vec::new();
        if let Some(od) = &obs_u64 {
            obs_channels.push((od, od.values().sum()));
        }
        if let Some(td) = tri_dist {
            obs_channels.push((td, td.values().sum()));
        }
        if let Some(md) = m0_ref {
            obs_channels.push((md, md.values().sum()));
        }
        let obs_fuse_pred = fuse_product_multi(&obs_channels).or(unigram_pred);
        obs_fuse_arm.score(offset, obs_fuse_pred, truth);
        // granularity direction: region-match grading of the key arms
        rg_unigram.score(&art, unigram_pred, truth);
        rg_trigram.score(&art, tri_dist.and_then(argmax).or(unigram_pred), truth);
        rg_obs.score(&art, obs_pred_cache, truth);
        rg_m0.score(&art, m0_ref.and_then(argmax).or(unigram_pred), truth);
        rg_obs_fuse.score(&art, obs_fuse_pred, truth);

        // ---- #394 bits ladder ----
        bits_n += 1;
        bits_store += -store_prob(&store, &codes[i], truth).log2();
        bits_fwd += -table_prob(fwd_dist, truth).log2();
        bits_prod += -product_prob(&store, &codes[i], fwd_dist, truth).log2();
        let uc = unigram.get(&truth).copied().unwrap_or(0) as f64;
        bits_uni += -(((uc + 0.5) / (uni_total as f64 + 16_000.0)).max(1e-30)).log2();

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
    tri_arm.report();
    rows_proxy_arm.report();
    prod_tri_arm.report();
    prod_tri_fwd_arm.report();
    m0_arm.report();
    m0_fuse_arm.report();
    store_obs_arm.report();
    obs_fuse_arm.report();
    rg_unigram.report();
    rg_trigram.report();
    rg_obs.report();
    rg_m0.report();
    rg_obs_fuse.report();
    println!(
        "bits ladder (free targets, n={bits_n}): unigram {:.3} | store {:.3} | fwd-table {:.3} | product {:.3}",
        bits_uni / bits_n as f64,
        bits_store / bits_n as f64,
        bits_fwd / bits_n as f64,
        bits_prod / bits_n as f64
    );
    println!(
        "backoff diagnostics: token-table key missing {token_key_missing} | region-d3 key missing {region3_key_missing}"
    );
}
