//! Stratified-construction harness (issue #435, first measurement):
//! construction-time structural stratification.
//!
//! CLAIM under test: the substrate's redundancy-regime performance (the
//! ~36.5% deep-observation corpus) versus the flat broad-corpus number
//! (~26% on wiki10k, composition not scale — the 500k subsample scores the
//! same) is a property of construction COMPOSITION. Per-stratum
//! construction plus stratum routing should recover part of the gap.
//!
//! Design (cheapest faithful cut, harness-level store, no compile):
//!
//! * Evidence key: last-two-token context `(prev2, prev1)` — the dominant
//!   NGRAM-like key. Representation-free by construction, so this measures
//!   stratification of EVIDENCE, decoupled from RVQ codes.
//! * Evidence source: observed next-token counts (`c.next`) from the
//!   construction partition — the substrate's construction evidence on
//!   natural text (teacher top-k is off-distribution there, see the
//!   observed-evidence arm rationale in `anchor_infill.rs`).
//! * FLAT arm: one store over all construction positions.
//! * Content strata: each story gets a sparse unigram signature over the
//!   top `TOP_N` most frequent construction tokens (frequency and vocab
//!   from the construction partition only), L2-normalized; k-means with
//!   K in `STRATA_KS`, deterministic strided seeding, fixed iteration
//!   count, deterministic empty-cluster reseed. Construction stories are
//!   partitioned by nearest centroid and one store is built per stratum.
//! * ROUTED arm: a held-out story is assigned to its nearest centroid
//!   (routing consumes only the held-out story's own tokens plus the
//!   construction-derived centroids). Reported both strict (stratum store
//!   only, unigram on key miss) and with backoff to the FLAT store on key
//!   miss — backoff is the primary arm (per-stratum store plus global
//!   backoff is the realistic serving shape).
//! * ORACLE arm: each held-out story is scored under EVERY stratum store
//!   (same backoff rule) and credited with its best stratum — the routing
//!   ceiling. Selection bias is quantified by the modulo control below.
//! * MODULO control: stratum = `story_id % K` — a topic-agnostic
//!   partition. Its routed arm shows the cost of content-blind routing;
//!   its oracle arm is the selection-bias floor a content oracle must
//!   clear. Story-identity strata (the per-story extreme) are excluded on
//!   purpose: under the hash split, held-out stories are disjoint from
//!   construction stories, so identity strata carry no evidence for any
//!   held-out story.
//!
//! Reported per arm: top-1 on held-out positions, and evidence-hit rate
//! (fraction of held-out positions whose key is present in the consulted
//! store before any backoff).
//!
//! PRE-DECLARED RULE: stratification wins if the ROUTED-backoff arm beats
//! FLAT top-1 by at least two percentage points on the broad corpus
//! (wiki10k / mc1). The ORACLE arm reports the ceiling only and never
//! decides the claim.
//!
//! Run (fixture smoke):
//!   cargo test --release -p uor-r4-graph-certify --test strata_construction -- --ignored --nocapture
//! Broad corpus:
//!   R4_CORPUS_META=... R4_CORPUS_RECS=... [R4_STORIES=stories.jsonl] \
//!     cargo test --release -p uor-r4-graph-certify --test strata_construction -- --ignored --nocapture

use std::collections::BTreeMap;
use std::collections::HashMap;

use uor_r4_core::transformerless::compiler;
use uor_r4_core::transformerless::runtime;

/// Last-two-token context key: (token two back, previous token).
type TriKey = (u32, u32);
/// Next-token count distribution.
type Dist = BTreeMap<u32, u32>;
/// One evidence store: last-two-token context -> next-token counts.
type TriStore = HashMap<TriKey, Dist>;
/// Sparse story signature over the top-N vocabulary: (vocab slot, weight).
type Signature = Vec<(u16, f32)>;

/// Signature vocabulary size: story unigram vectors are restricted to the
/// top `TOP_N` most frequent construction tokens. Keeps the signature pass
/// memory-bounded on the 2.1M-record corpus (sparse per-story vectors,
/// dense centroids of `K * TOP_N` f32 only).
const TOP_N: usize = 1024;
/// Fixed k-means iteration count (deterministic, no convergence test).
const KMEANS_ITERS: usize = 10;
/// Stratum counts measured for both the content and the modulo partition.
const STRATA_KS: [usize; 2] = [8, 32];
/// Pre-declared win margin for ROUTED over FLAT, in percentage points.
const WIN_MARGIN_PP: f64 = 2.0;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn argmax(dist: &Dist) -> Option<u32> {
    dist.iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
}

fn pct(h: u64, t: u64) -> f64 {
    100.0 * h as f64 / t.max(1) as f64
}

struct Arm {
    name: String,
    top1: u64,
    ev_hits: u64,
    total: u64,
}

impl Arm {
    fn new(name: String) -> Self {
        Arm {
            name,
            top1: 0,
            ev_hits: 0,
            total: 0,
        }
    }
    fn score(&mut self, pred: Option<u32>, evidence_hit: bool, truth: u32) {
        self.total += 1;
        if evidence_hit {
            self.ev_hits += 1;
        }
        if pred == Some(truth) {
            self.top1 += 1;
        }
    }
    fn top1_pct(&self) -> f64 {
        pct(self.top1, self.total)
    }
    fn report(&self) {
        println!(
            "{:<26} top1 {:>5.1}% | evidence-hit {:>5.1}% (n={})",
            self.name,
            self.top1_pct(),
            pct(self.ev_hits, self.total),
            self.total
        );
    }
}

/// One held-out graded position, precomputed once.
struct EvalPos {
    story: u32,
    key: Option<TriKey>,
    truth: u32,
}

/// Squared Euclidean distance from a unit-norm sparse signature to each
/// dense centroid, up to the constant `||x||^2` term: argmin over
/// `||c_j||^2 - 2 * <x, c_j>`. Ties break to the lowest centroid index.
fn nearest_centroid(sig: &Signature, centroids: &[f32], k: usize) -> usize {
    let mut best = (0usize, f32::MAX);
    for j in 0..k {
        let row = &centroids[j * TOP_N..(j + 1) * TOP_N];
        let mut norm2 = 0f32;
        for &v in row {
            norm2 += v * v;
        }
        let mut dot = 0f32;
        for &(slot, w) in sig {
            dot += w * row[slot as usize];
        }
        let d2 = norm2 - 2.0 * dot;
        if d2 < best.1 {
            best = (j, d2);
        }
    }
    best.0
}

/// Deterministic k-means over the construction stories' signatures:
/// strided initialization, fixed iteration count, empty clusters reseeded
/// from a deterministic index. Returns dense centroids (`k * TOP_N`).
fn kmeans_signatures(sigs: &[&Signature], k: usize) -> Vec<f32> {
    let n = sigs.len();
    let mut centroids = vec![0f32; k * TOP_N];
    let seed_row = |centroids: &mut [f32], j: usize, src: usize| {
        let row = &mut centroids[j * TOP_N..(j + 1) * TOP_N];
        row.fill(0.0);
        for &(slot, w) in sigs[src] {
            row[slot as usize] = w;
        }
    };
    for j in 0..k {
        seed_row(&mut centroids, j, j * n / k);
    }
    for _ in 0..KMEANS_ITERS {
        let mut sums = vec![0f64; k * TOP_N];
        let mut counts = vec![0u64; k];
        for sig in sigs {
            let j = nearest_centroid(sig, &centroids, k);
            counts[j] += 1;
            for &(slot, w) in sig.iter() {
                sums[j * TOP_N + slot as usize] += f64::from(w);
            }
        }
        for j in 0..k {
            if counts[j] == 0 {
                seed_row(&mut centroids, j, (j * 2_654_435_761) % n);
                continue;
            }
            let row = &mut centroids[j * TOP_N..(j + 1) * TOP_N];
            for (t, slot) in row.iter_mut().enumerate() {
                *slot = (sums[j * TOP_N + t] / counts[j] as f64) as f32;
            }
        }
    }
    centroids
}

/// Build one evidence store per stratum from the construction positions.
fn build_stratum_stores(
    c: &compiler::Corpus,
    is_constr: &[bool],
    assign: &[usize],
    k: usize,
) -> Vec<TriStore> {
    let mut stores: Vec<TriStore> = (0..k).map(|_| TriStore::new()).collect();
    #[allow(clippy::needless_range_loop)] // parallel corpus arrays
    for i in 0..c.n {
        let sid = c.story[i] as usize;
        if !is_constr[sid] {
            continue;
        }
        let Some(p2) = runtime::history_token(c, i, 2) else {
            continue;
        };
        *stores[assign[sid]]
            .entry((p2, c.input[i]))
            .or_default()
            .entry(c.next[i])
            .or_default() += 1;
    }
    stores
}

/// Grade one stratum configuration: routed strict, routed with FLAT
/// backoff, and the per-story oracle ceiling (all with the same backoff
/// rule). `assign` maps EVERY story (construction and held-out) to a
/// stratum; construction assignments were used to build `stores`.
fn eval_strata(
    label: &str,
    eval: &[EvalPos],
    stores: &[TriStore],
    assign: &[usize],
    flat: &TriStore,
    unigram_pred: Option<u32>,
) -> f64 {
    let k = stores.len();
    let mut routed_strict = Arm::new(format!("{label}-routed-strict"));
    let mut routed_backoff = Arm::new(format!("{label}-routed-backoff"));
    // per held-out story: hits and evidence hits under every stratum
    let mut per_story: HashMap<u32, (Vec<u32>, Vec<u32>, u32)> = HashMap::new();
    for p in eval {
        let flat_dist = p.key.and_then(|key| flat.get(&key));
        let flat_pred = flat_dist.and_then(argmax);
        let routed = assign[p.story as usize];
        let entry = per_story
            .entry(p.story)
            .or_insert_with(|| (vec![0; k], vec![0; k], 0));
        entry.2 += 1;
        for (j, store) in stores.iter().enumerate() {
            let dist = p.key.and_then(|key| store.get(&key));
            let pred = dist.and_then(argmax).or(flat_pred).or(unigram_pred);
            if dist.is_some() {
                entry.1[j] += 1;
            }
            if pred == Some(p.truth) {
                entry.0[j] += 1;
            }
            if j == routed {
                routed_backoff.score(pred, dist.is_some(), p.truth);
                routed_strict.score(
                    dist.and_then(argmax).or(unigram_pred),
                    dist.is_some(),
                    p.truth,
                );
            }
        }
    }
    let mut oracle = Arm::new(format!("{label}-oracle"));
    for (hits, evs, n) in per_story.values() {
        // best stratum by top-1 hits, ties to the lowest stratum id
        let best = (0..k)
            .max_by_key(|&j| (hits[j], std::cmp::Reverse(j)))
            .unwrap_or(0);
        oracle.top1 += u64::from(hits[best]);
        oracle.ev_hits += u64::from(evs[best]);
        oracle.total += u64::from(*n);
    }
    routed_strict.report();
    routed_backoff.report();
    oracle.report();
    routed_backoff.top1_pct()
}

#[test]
#[ignore = "measurement harness (#435); run explicitly with --ignored"]
fn strata_construction() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let c = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    println!("corpus: {meta_path} + {recs_path}");
    let cut = (c.stories as f64 * 0.8) as u32;
    // Partition override, same protocol as anchor_infill.rs: R4_STORIES
    // points at the obs pass's stories.jsonl and supplies the D3
    // article-hash partition; unset falls back to the sequential story cut.
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
    println!(
        "strata-construction (#435): {} records, {} stories, top-N {TOP_N}, K {STRATA_KS:?}",
        c.n, c.stories
    );

    // ---- construction-partition vocabulary and unigram null ----
    let mut unigram: Dist = Dist::new();
    #[allow(clippy::needless_range_loop)] // parallel corpus arrays
    for i in 0..c.n {
        if constr[c.story[i] as usize] {
            *unigram.entry(c.next[i]).or_default() += 1;
        }
    }
    let unigram_pred = argmax(&unigram);
    let mut ranked: Vec<(u32, u32)> = unigram.iter().map(|(&t, &n)| (t, n)).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked.truncate(TOP_N);
    let slot_of: HashMap<u32, u16> = ranked
        .iter()
        .enumerate()
        .map(|(s, &(t, _))| (t, s as u16))
        .collect();

    // ---- story signatures (sparse, L2-normalized) for ALL stories ----
    // Construction stories feed k-means and the stratum partition;
    // held-out stories are routed by their own tokens at eval time.
    let mut raw: Vec<HashMap<u16, u32>> = vec![HashMap::new(); c.stories as usize];
    #[allow(clippy::needless_range_loop)] // parallel corpus arrays
    for i in 0..c.n {
        if let Some(&slot) = slot_of.get(&c.next[i]) {
            *raw[c.story[i] as usize].entry(slot).or_default() += 1;
        }
    }
    let sigs: Vec<Signature> = raw
        .into_iter()
        .map(|m| {
            let mut v: Signature = m.into_iter().map(|(s, n)| (s, n as f32)).collect();
            v.sort_unstable_by_key(|&(s, _)| s);
            let norm = v.iter().map(|&(_, w)| f64::from(w * w)).sum::<f64>().sqrt();
            if norm > 0.0 {
                for (_, w) in v.iter_mut() {
                    *w = (f64::from(*w) / norm) as f32;
                }
            }
            v
        })
        .collect();
    let constr_sigs: Vec<&Signature> = (0..c.stories as usize)
        .filter(|&sid| constr[sid])
        .map(|sid| &sigs[sid])
        .collect();

    // ---- FLAT arm ----
    let single = vec![0usize; c.stories as usize];
    let flat = build_stratum_stores(&c, &constr, &single, 1)
        .pop()
        .expect("flat store");
    println!("flat store: {} keys", flat.len());

    // held-out graded positions, precomputed once
    let eval: Vec<EvalPos> = (0..c.n)
        .filter(|&i| !constr[c.story[i] as usize])
        .map(|i| EvalPos {
            story: c.story[i],
            key: runtime::history_token(&c, i, 2).map(|p2| (p2, c.input[i])),
            truth: c.next[i],
        })
        .collect();

    let mut flat_arm = Arm::new("flat".to_string());
    for p in &eval {
        let dist = p.key.and_then(|key| flat.get(&key));
        flat_arm.score(
            dist.and_then(argmax).or(unigram_pred),
            dist.is_some(),
            p.truth,
        );
    }
    flat_arm.report();
    let flat_top1 = flat_arm.top1_pct();

    // ---- stratified arms ----
    for k in STRATA_KS {
        println!("---- K = {k} ----");
        // content strata: k-means over construction signatures, every
        // story (construction and held-out) routed to nearest centroid
        let centroids = kmeans_signatures(&constr_sigs, k);
        let assign: Vec<usize> = sigs
            .iter()
            .map(|sig| nearest_centroid(sig, &centroids, k))
            .collect();
        let mut sizes = vec![0u64; k];
        for (sid, &j) in assign.iter().enumerate() {
            if constr[sid] {
                sizes[j] += 1;
            }
        }
        println!("content stratum sizes (construction stories): {sizes:?}");
        let stores = build_stratum_stores(&c, &constr, &assign, k);
        let routed_top1 = eval_strata(
            &format!("content-K{k}"),
            &eval,
            &stores,
            &assign,
            &flat,
            unigram_pred,
        );
        drop(stores);
        // modulo control: topic-agnostic partition of the same size
        let mod_assign: Vec<usize> = (0..c.stories as usize).map(|sid| sid % k).collect();
        let mod_stores = build_stratum_stores(&c, &constr, &mod_assign, k);
        eval_strata(
            &format!("modulo-K{k}"),
            &eval,
            &mod_stores,
            &mod_assign,
            &flat,
            unigram_pred,
        );
        let delta = routed_top1 - flat_top1;
        println!(
            "K={k}: routed-backoff {:.1}% vs flat {:.1}% (delta {:+.1}pp) -> {}",
            routed_top1,
            flat_top1,
            delta,
            if delta >= WIN_MARGIN_PP {
                "STRATIFICATION WINS (pre-declared rule: routed >= flat + 2pp)"
            } else {
                "no win under the pre-declared rule (routed >= flat + 2pp)"
            }
        );
    }
}
