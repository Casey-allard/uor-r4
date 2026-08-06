//! Two-sided (left, right) context keying vs one-token-future product
//! fusion, plus the distribution-class growth law (issue #436).
//!
//! MOTIVATION (Kracht, *The Mathematics of Language*). Page twenty-four
//! develops the Galois connection between strings and CONTEXTS, where a
//! context is a PAIR of a left part and a right part; the closed sets of
//! that connection (Sestier closure) are the distribution classes of the
//! language. Page four hundred thirty-eight defines `Cont(x)` two-sidedly
//! for the same reason: substitutability is decided by what may stand on
//! BOTH sides of a string, not by its left context alone. Our scorer
//! conditions on left context only. The shipped forward-anchor channel
//! (FWDA: keyed by lookahead distance and the anchor token, fused as a
//! product of experts with the causal store) is, in retrospect, a crude
//! ONE-TOKEN right-context probe; it measured plus four point two points
//! at five hundred thousand records. The hypothesis under test here is
//! stronger: right context carries real structure, and two-sided KEYING
//! dominates one-token-future product fusion.
//!
//! HONESTY NOTE (repeated in the printed output, and binding on anyone
//! quoting these numbers). Two-sided conditioning is NOT causally
//! available to left-to-right generation: at generation time the tokens
//! after the target do not exist. This file is an INFILL / ANALYSIS
//! measurement, the same regime as our validated A-mode infill path. It
//! answers "does right context carry structure the left key is missing",
//! and it must never be quoted as a generation number.
//!
//! ARMS (held-out positions, next-token prediction, natural corpus).
//!
//! ARM L (left-only baseline, the reference the others must beat): the
//! store/EXCT idiom — evidence keyed by the graded code prefix of the
//! left context, read at the deepest populated prefix `code[..d]` for
//! `d` from `STAGES` down to zero. Evidence is the observed corpus
//! continuation (`c.next`) over the construction split, which is the
//! evidence source the natural-corpus arms use throughout.
//!
//! ARM LR (two-sided): key is the PAIR of the left graded code prefix and
//! a RIGHT graded code prefix at MATCHED granularity. Right-key
//! construction (chosen deliberately; see the justification below): the
//! next `R` tokens after the target position (`R` default four,
//! env-overridable), laid out REVERSE-ORDERED — farthest first, so the
//! token immediately after the target lands in the most-recent slot —
//! and pushed through the SAME `bundle_window_plain` and
//! `assign_for_bundle` machinery that produces the left code. The right
//! key is therefore a `[u8; STAGES]` graded code drawn from the same
//! stage codebooks, with the same class alphabet and the same prefix
//! semantics as the left key, so "the pair at depth d" is a genuinely
//! matched-granularity object. A truncated hash of the next `R` token
//! ids was the alternative; it was rejected because a hash has no graded
//! prefix structure, so a hashed right key cannot back off at matched
//! depth and any measured difference would confound keying with
//! granularity. Backoff: deepest populated pair `(left[..d], right[..d])`
//! for `d` from `STAGES` down to one, then the left-only chain, then the
//! construction unigram.
//!
//! ARM FWD (comparator, the shipped mechanism): forward-anchor rows built
//! by the shipped `score::compile_forward_anchor_rows` over the
//! construction split, product-fused with ARM L's distribution under
//! `fuse_forward_arm`'s law (sum of smoothed log-probabilities over the
//! union support, absentees at each channel's smoothing floor). The
//! shipped `fuse_forward_arm` is private and takes a `GraphScorer`
//! `ScoreOutcome`, so its law is reimplemented certifier-side here; the
//! CHANNEL is the shipped compiler's output, cap and minimum-total gate
//! included. Forward rows exist only for lookahead distances one through
//! three (stride four, mirroring the private `score::M2_STRIDE`), so at
//! anchor positions ARM FWD falls back to ARM L; the free-position slice
//! is reported separately so ARM FWD is also read on exactly the slice
//! its shipped measurement covers.
//!
//! ARM SHUF (falsifier): identical to ARM LR, except the right key comes
//! from a DIFFERENT held-out position — a fixed half-length rotation over
//! the evaluated position list. Key cardinality, backoff shape and
//! smoothing are unchanged, so any ARM LR gain that survives ARM SHUF is
//! right-context INFORMATION rather than an artifact of a larger key
//! space.
//!
//! NULL: construction-unigram argmax.
//!
//! ARM LR-FUSE is a diagnostic only (product of the two-sided pair
//! channel with the left channel); it is NOT part of the exit rule.
//!
//! BITS. Every keying arm's bits per token come from the Witten-Bell
//! backoff mixture over exactly the chain that arm predicts from (the
//! `eval` WB rule, leftover mass uniform over the vocabulary); the fusion
//! arms use that mixture as the base channel times the row's add-half
//! smoothed probability, normalized exactly over the vocabulary. Every
//! arm's bits are therefore a proper code length on one scale.
//!
//! PRE-DECLARED EXIT RULE. Two-sided keying is CONFIRMED iff
//! ARM LR top-one is at least ARM L top-one plus two point zero points
//! AND ARM LR beats ARM FWD AND ARM LR beats ARM SHUF. All arms and
//! their bits per token are reported regardless of the verdict.
//!
//! SECOND MEASUREMENT (`distribution_class_growth`): how the NUMBER OF
//! DISTINCT EQUIVALENCE CLASSES grows with corpus size. Nested prefixes
//! of the construction split are taken by story id (so the splits nest
//! exactly), and each size reports distinct left keys, distinct
//! two-sided keys, distinct next-token-distribution SIGNATURES under two
//! definitions, and mean support per class. The signature definitions are
//! the cheap proxy for Sestier-closed distribution classes: TOP-ONE
//! signature groups keys whose argmax next token agrees; DIST signature
//! groups keys whose top `SIG_TOP` next tokens agree with each
//! probability quantized to `SIG_BINS` bins (default sixteen, i.e. a
//! tolerance of six point two five points of probability mass). The
//! question: does the natural class count keep growing with data — in
//! which case any fixed region budget or absolute split threshold is
//! wrong — and at what rate. A log-log slope against the position count
//! is printed per step and overall.
//!
//! Run:
//!   R4_CORPUS_META=/tmp/c_meta.bin R4_CORPUS_RECS=/tmp/c_recs.bin \
//!   R4_STORIES=/tmp/wiki-obs/stories.jsonl R4_ARTIFACTS=/tmp/tless_artifacts.bin \
//!   cargo test -p uor-r4-graph-certify --test two_sided_context -- \
//!     --ignored --nocapture

use std::collections::{BTreeMap, HashMap, HashSet};

use uor_r4_core::transformerless::compiler::{self, Corpus, SIG_BYTES, STAGES};
use uor_r4_core::transformerless::runtime;
use uor_r4_graph_certify::score::compile_forward_anchor_rows;
use uor_r4_graph_compiler::induction::Observation;

/// Token to count table (certifier-side f64/alloc is permitted here).
type Dist = BTreeMap<u32, u64>;
/// One store level: packed graded-code prefix to evidence.
type Level = HashMap<u32, Cell>;
/// One two-sided level: packed (left, right) prefix pair to evidence.
type PairLevel = HashMap<u64, Cell>;
/// Quantized next-token-distribution signature (growth law).
type Signature = Vec<(u32, u16)>;

/// Anchor stride of the shipped forward-anchor channel; mirrors the
/// private `score::M2_STRIDE`, which `compile_forward_anchor_rows` uses.
const FWD_STRIDE: usize = 4;

fn fixture(name: &str) -> String {
    format!(
        "{}/../uor-r4-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Stream position of every record inside its story (0-based index of the
/// record's INPUT token); records are sequential per story.
fn story_positions(c: &Corpus) -> Vec<usize> {
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

/// Construction membership per story: the D3 article-hash partition from
/// the observation pass when `R4_STORIES` is set, else the sequential
/// eighty-percent story cut.
fn construction_split(c: &Corpus) -> Vec<bool> {
    let cut = (c.stories as f64 * 0.8) as u32;
    match std::env::var("R4_STORIES") {
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
    }
}

/// Pack a graded-code prefix into one key word; the depth is carried by
/// the level index, so no depth tag is needed inside the word.
fn pack(code: &[u8; STAGES], depth: usize) -> u32 {
    let mut key = 0u32;
    for &b in &code[..depth] {
        key = (key << 8) | u32::from(b);
    }
    key
}

fn pack_pair(left: u32, right: u32) -> u64 {
    (u64::from(left) << 32) | u64::from(right)
}

/// One stored table cell: token counts plus the running evidence total,
/// so no scoring path ever re-sums a level.
#[derive(Default)]
struct Cell {
    dist: Dist,
    total: u64,
}

impl Cell {
    fn add(&mut self, token: u32) {
        *self.dist.entry(token).or_default() += 1;
        self.total += 1;
    }
    fn channel(&self) -> Channel<'_> {
        Channel::new(&self.dist, self.total)
    }
}

/// Canonical argmax: highest count, ties to the lowest token id.
fn argmax(dist: &Dist) -> Option<u32> {
    dist.iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, _)| t)
}

/// A distribution with its evidence total precomputed once: the totals
/// are summed when a table is BUILT, never inside the per-position
/// scoring loop (the root level holds tens of thousands of entries; a
/// re-sum per probability call is what makes the honest bits ladder
/// unaffordable).
#[derive(Clone, Copy)]
struct Channel<'a> {
    dist: &'a Dist,
    total: u64,
}

impl<'a> Channel<'a> {
    fn new(dist: &'a Dist, total: u64) -> Self {
        Channel { dist, total }
    }
    /// Add-half smoothed probability of `truth`.
    fn smoothed(&self, truth: u32, vocab: f64) -> f64 {
        let c = self.dist.get(&truth).copied().unwrap_or(0) as f64;
        ((c + 0.5) / (self.total as f64 + 0.5 * vocab)).max(1e-30)
    }
    /// The probability an absent token receives (the fusion floor).
    fn floor(&self, vocab: f64) -> f64 {
        (0.5 / (self.total as f64 + 0.5 * vocab)).max(1e-30)
    }
}

/// Witten-Bell backoff-mixture probability of `truth` over an ORDERED
/// backoff chain (deepest key first — exactly the chain the matching arm
/// predicts from), with the leftover mass spread uniformly. Same rule the
/// certify `eval` WB metric and the anchor-infill bits ladder use, so the
/// bits column of a deep sparse key is not crushed by a flat add-half
/// floor and every arm's bits are a proper code length over the vocab.
fn wb_prob(chain: &[Channel<'_>], truth: u32, vocab: f64) -> f64 {
    let mut rem = 1.0f64;
    let mut acc = 0.0f64;
    for ch in chain {
        if ch.total == 0 {
            continue;
        }
        let total = ch.total as f64;
        let lam = total / (total + ch.dist.len() as f64);
        let p = ch.dist.get(&truth).copied().unwrap_or(0) as f64 / total;
        acc += rem * lam * p;
        rem *= 1.0 - lam;
    }
    (acc + rem / vocab).max(1e-30)
}

/// Product-of-experts argmax (the `fuse_forward_arm` selection law: sum of
/// log-probabilities, absentees at the channel's smoothing floor, ties to
/// the lowest token id). The base channel is the arm's backoff chain, the
/// second channel a single count row; candidates are the row support
/// union the deepest chain level's support (the base is nonzero on the
/// whole vocab, so the candidate set is bounded deliberately).
fn fuse_argmax(chain: &[Channel<'_>], row: Option<Channel<'_>>, vocab: f64) -> Option<u32> {
    let Some(row) = row else {
        return chain.first().and_then(|ch| argmax(ch.dist));
    };
    let mut support: Vec<u32> = row.dist.keys().copied().collect();
    if let Some(top) = chain.first() {
        support.extend(top.dist.keys().copied());
    }
    support.sort_unstable();
    support.dedup();
    support
        .into_iter()
        .map(|t| {
            let s = wb_prob(chain, t, vocab).ln() + row.smoothed(t, vocab).ln();
            (t, s)
        })
        .max_by(|p, q| p.1.partial_cmp(&q.1).unwrap().then(q.0.cmp(&p.0)))
        .map(|(t, _)| t)
}

/// Exactly normalized product-of-experts probability of `truth`: the base
/// chain's backoff mixture times the row's smoothed probability, divided
/// by the exact vocabulary-wide normalizer (out-of-row tokens all take
/// the row's floor, so the normalizer closes in row-support time).
fn fuse_prob(chain: &[Channel<'_>], row: Option<Channel<'_>>, truth: u32, vocab: f64) -> f64 {
    let Some(row) = row else {
        return wb_prob(chain, truth, vocab);
    };
    let floor = row.floor(vocab);
    let mut z = floor;
    for &t in row.dist.keys() {
        z += wb_prob(chain, t, vocab) * (row.smoothed(t, vocab) - floor);
    }
    let p = wb_prob(chain, truth, vocab) * row.smoothed(truth, vocab);
    (p / z.max(1e-30)).max(1e-30)
}

/// One measured arm: top-one on all graded positions, top-one on the
/// free (non-anchor) slice, and bits per token.
struct Arm {
    name: &'static str,
    hits: u64,
    total: u64,
    free_hits: u64,
    free_total: u64,
    bits: f64,
}

impl Arm {
    fn new(name: &'static str) -> Self {
        Arm {
            name,
            hits: 0,
            total: 0,
            free_hits: 0,
            free_total: 0,
            bits: 0.0,
        }
    }
    fn score(&mut self, pred: Option<u32>, truth: u32, prob: f64, free: bool) {
        self.total += 1;
        self.bits += -prob.log2();
        let hit = u64::from(pred == Some(truth));
        self.hits += hit;
        if free {
            self.free_total += 1;
            self.free_hits += hit;
        }
    }
    fn top1(&self) -> f64 {
        100.0 * self.hits as f64 / self.total.max(1) as f64
    }
    fn free_top1(&self) -> f64 {
        100.0 * self.free_hits as f64 / self.free_total.max(1) as f64
    }
    fn bits_per_token(&self) -> f64 {
        self.bits / self.total.max(1) as f64
    }
    fn report(&self) {
        println!(
            "{:<22} top1 {:>6.2}% | free-slice top1 {:>6.2}% | bits/token {:>7.3} (n={})",
            self.name,
            self.top1(),
            self.free_top1(),
            self.bits_per_token(),
            self.total
        );
    }
}

/// Left graded code of every corpus position, derived through the shipped
/// bundle and assignment path, chunked across worker threads exactly as
/// `runtime::build_store_with_threads` does.
fn derive_left_codes(art: &compiler::Compiled, c: &Corpus, threads: usize) -> Vec<[u8; STAGES]> {
    derive_codes(c.n, threads, |lo, hi| {
        let rot = compiler::derive_rotations();
        (lo..hi)
            .map(|i| {
                let bundle = runtime::bundle_plain(art, &rot, c, i);
                (runtime::assign_for_bundle(art, &bundle), true)
            })
            .collect()
    })
    .0
}

/// Right graded code of every corpus position at matched granularity: the
/// next `r_len` tokens after the target, REVERSE-ORDERED (farthest first,
/// so the token immediately after the target occupies the most-recent
/// dyadic slot), through the same window-bundle and assignment path as
/// the left code. The flag is false when no in-story right token exists
/// (story end), in which case the arms back off to left-only.
fn derive_right_codes(
    art: &compiler::Compiled,
    c: &Corpus,
    r_len: usize,
    threads: usize,
) -> (Vec<[u8; STAGES]>, Vec<bool>) {
    derive_codes(c.n, threads, |lo, hi| {
        let rot = compiler::derive_rotations();
        let mut window: Vec<u32> = Vec::with_capacity(r_len);
        (lo..hi)
            .map(|i| {
                window.clear();
                for r in (1..=r_len).rev() {
                    if i + r < c.n && c.story[i + r] == c.story[i] {
                        window.push(c.next[i + r]);
                    }
                }
                if window.is_empty() {
                    return ([0u8; STAGES], false);
                }
                let bundle = runtime::bundle_window_plain(art, &rot, &window);
                (runtime::assign_for_bundle(art, &bundle), true)
            })
            .collect()
    })
}

/// One worker chunk: `(code, key-available)` per position, in position
/// order.
type CodeChunk = Vec<([u8; STAGES], bool)>;

/// Chunked parallel driver shared by both code derivations; the worker
/// returns codes in position order for its chunk.
fn derive_codes<F>(n: usize, threads: usize, worker: F) -> (Vec<[u8; STAGES]>, Vec<bool>)
where
    F: Fn(usize, usize) -> CodeChunk + Sync,
{
    let workers = threads.max(1).min(n.max(1));
    let chunk = n.div_ceil(workers);
    let mut chunks: Vec<(usize, CodeChunk)> = Vec::with_capacity(workers);
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        let worker = &worker;
        for (id, lo) in (0..n).step_by(chunk.max(1)).enumerate() {
            let hi = (lo + chunk).min(n);
            handles.push((id, scope.spawn(move || worker(lo, hi))));
        }
        for (id, handle) in handles {
            chunks.push((id, handle.join().expect("code worker panicked")));
        }
    });
    chunks.sort_by_key(|(id, _)| *id);
    let mut codes = Vec::with_capacity(n);
    let mut flags = Vec::with_capacity(n);
    for (_, part) in chunks {
        for (code, ok) in part {
            codes.push(code);
            flags.push(ok);
        }
    }
    (codes, flags)
}

/// Left-only backoff chain, deepest populated prefix first (the store /
/// EXCT read rule): the head is the arm's prediction table, the whole
/// chain is its Witten-Bell mixture.
fn left_chain<'a>(levels: &'a [Level], code: &[u8; STAGES]) -> Vec<Channel<'a>> {
    (0..=STAGES)
        .rev()
        .filter_map(|d| levels[d].get(&pack(code, d)).map(Cell::channel))
        .collect()
}

/// Matched-granularity two-sided chain, deepest populated pair first,
/// with the depth that answered (zero when no pair level did).
fn pair_chain<'a>(
    levels: &'a [PairLevel],
    left: &[u8; STAGES],
    right: &[u8; STAGES],
) -> (Vec<Channel<'a>>, usize) {
    let mut chain = Vec::new();
    let mut depth = 0usize;
    for d in (1..=STAGES).rev() {
        if let Some(cell) = levels[d].get(&pack_pair(pack(left, d), pack(right, d))) {
            if depth == 0 {
                depth = d;
            }
            chain.push(cell.channel());
        }
    }
    (chain, depth)
}

#[test]
#[ignore = "measurement harness; run explicitly with --ignored"]
fn two_sided_context_keying() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let art_path = std::env::var("R4_ARTIFACTS").unwrap_or_else(|_| fixture("tless_artifacts.bin"));
    let c = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    let art = compiler::load_artifacts_from(&art_path).expect("artifacts");
    let r_len = env_usize("R4_TS_RIGHT_R", 4);
    let max_eval = env_usize("R4_TS_MAX_EVAL", 250_000);
    let threads = env_usize(
        "R4_TS_THREADS",
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1),
    );
    println!("corpus: {meta_path} + {recs_path} ({} records)", c.n);
    println!("artifacts: {art_path}");
    println!(
        "caps: R4_TS_RIGHT_R={r_len} R4_TS_MAX_EVAL={max_eval} R4_TS_THREADS={threads} \
         (stride {FWD_STRIDE}, STAGES {STAGES})"
    );
    println!(
        "HONESTY NOTE: two-sided conditioning is NOT causally available to \
         left-to-right generation. This is an infill/analysis measurement \
         (A-mode regime); it must never be quoted as a generation number."
    );

    let constr = construction_split(&c);
    let is_constr = |sid: u32| constr[sid as usize];
    let positions = story_positions(&c);
    let vocab = (art.token_codes.len() / STAGES).max(1) as f64;

    let left_codes = derive_left_codes(&art, &c, threads);
    let (right_codes, right_ok) = derive_right_codes(&art, &c, r_len, threads);
    println!("codes derived: left {} | right {}", c.n, right_ok.len());

    // ---- construction tables ----
    let mut unigram = Cell::default();
    let mut left_levels: Vec<Level> = (0..=STAGES).map(|_| HashMap::new()).collect();
    let mut pair_levels: Vec<PairLevel> = (0..=STAGES).map(|_| HashMap::new()).collect();
    let mut train: Vec<Observation> = Vec::new();
    #[allow(clippy::needless_range_loop)] // index i addresses parallel corpus arrays
    for i in 0..c.n {
        if !is_constr(c.story[i]) {
            continue;
        }
        let truth = c.next[i];
        unigram.add(truth);
        for (d, level) in left_levels.iter_mut().enumerate() {
            level.entry(pack(&left_codes[i], d)).or_default().add(truth);
        }
        if right_ok[i] {
            for d in 1..=STAGES {
                let key = pack_pair(pack(&left_codes[i], d), pack(&right_codes[i], d));
                pair_levels[d].entry(key).or_default().add(truth);
            }
        }
        train.push(Observation {
            position: i as u32,
            sample: [0; 32],
            vector: Vec::new(),
            sig: [0; SIG_BYTES],
            prev: c.input[i],
            next: truth,
        });
    }
    let unigram_pred = argmax(&unigram.dist);
    println!(
        "construction: {} positions | left keys(d{STAGES}) {} | pair keys(d{STAGES}) {}",
        train.len(),
        left_levels[STAGES].len(),
        pair_levels[STAGES].len()
    );

    // ---- shipped forward-anchor channel (ARM FWD) ----
    let rows = compile_forward_anchor_rows(&c, &train);
    drop(train);
    // Rows carry the FULL pre-cap total on the wire; the held channel is
    // the capped entry set, so its smoothing total is the sum of the
    // entries actually present.
    let mut fwd: HashMap<(u8, u32), Cell> = HashMap::new();
    for row in &rows {
        let dist: Dist = row
            .entries
            .iter()
            .map(|&(t, count)| (t, u64::from(count)))
            .collect();
        let total = dist.values().sum();
        fwd.insert((row.distance, row.anchor), Cell { dist, total });
    }
    println!("forward-anchor rows compiled: {}", rows.len());
    drop(rows);

    // ---- held-out evaluation positions ----
    let all_eval: Vec<usize> = (0..c.n).filter(|&i| !is_constr(c.story[i])).collect();
    let step = all_eval.len().div_ceil(max_eval.max(1)).max(1);
    let eval: Vec<usize> = all_eval.iter().copied().step_by(step).collect();
    drop(all_eval);
    let n_eval = eval.len();
    println!("held-out eval positions: {n_eval} (subsample step {step})");

    let mut arm_l = Arm::new("ARM L (left-only)");
    let mut arm_lr = Arm::new("ARM LR (two-sided)");
    let mut arm_fwd = Arm::new("ARM FWD (fusion)");
    let mut arm_shuf = Arm::new("ARM SHUF (falsifier)");
    let mut arm_null = Arm::new("NULL (unigram)");
    let mut arm_lrfuse = Arm::new("diag LR x L (fuse)");
    // resolution census for ARM LR: how deep the pair chain answered
    // (index STAGES + 1 counts left-only fallbacks).
    let mut lr_depth = [0u64; STAGES + 2];

    for (k, &i) in eval.iter().enumerate() {
        let truth = c.next[i];
        let target_pos = positions[i] + 1;
        let free = !target_pos.is_multiple_of(FWD_STRIDE);

        // ARM L
        let l_chain = left_chain(&left_levels, &left_codes[i]);
        let ld = l_chain.first().copied();
        arm_l.score(
            ld.and_then(|ch| argmax(ch.dist)).or(unigram_pred),
            truth,
            wb_prob(&l_chain, truth, vocab),
            free,
        );

        // ARM LR
        let (p_chain, depth) = if right_ok[i] {
            pair_chain(&pair_levels, &left_codes[i], &right_codes[i])
        } else {
            (Vec::new(), 0)
        };
        let pd = p_chain.first().copied();
        if pd.is_some() {
            lr_depth[depth] += 1;
        } else {
            lr_depth[STAGES + 1] += 1;
        }
        let lr_chain: Vec<Channel<'_>> = p_chain
            .iter()
            .copied()
            .chain(l_chain.iter().copied())
            .collect();
        arm_lr.score(
            pd.or(ld).and_then(|ch| argmax(ch.dist)).or(unigram_pred),
            truth,
            wb_prob(&lr_chain, truth, vocab),
            free,
        );

        // diagnostic: two-sided channel product-fused with the left channel
        arm_lrfuse.score(
            fuse_argmax(&l_chain, pd, vocab).or(unigram_pred),
            truth,
            fuse_prob(&l_chain, pd, truth, vocab),
            free,
        );

        // ARM FWD: shipped forward-anchor row fused with ARM L
        let fwd_row = if free {
            let lookahead = target_pos.next_multiple_of(FWD_STRIDE) - target_pos;
            let j = i + lookahead;
            if j < c.n && c.story[j] == c.story[i] {
                fwd.get(&(lookahead as u8, c.next[j])).map(Cell::channel)
            } else {
                None
            }
        } else {
            None
        };
        arm_fwd.score(
            fuse_argmax(&l_chain, fwd_row, vocab).or(unigram_pred),
            truth,
            fuse_prob(&l_chain, fwd_row, truth, vocab),
            free,
        );

        // ARM SHUF: same construction as ARM LR with a foreign right key
        let src = eval[(k + n_eval / 2) % n_eval];
        let (s_chain, _) = if right_ok[src] {
            pair_chain(&pair_levels, &left_codes[i], &right_codes[src])
        } else {
            (Vec::new(), 0)
        };
        let sd = s_chain.first().copied();
        let shuf_chain: Vec<Channel<'_>> = s_chain
            .iter()
            .copied()
            .chain(l_chain.iter().copied())
            .collect();
        arm_shuf.score(
            sd.or(ld).and_then(|ch| argmax(ch.dist)).or(unigram_pred),
            truth,
            wb_prob(&shuf_chain, truth, vocab),
            free,
        );

        // NULL
        arm_null.score(
            unigram_pred,
            truth,
            wb_prob(&[unigram.channel()], truth, vocab),
            free,
        );
    }

    println!("---- arms (held-out next-token prediction) ----");
    for arm in [&arm_null, &arm_l, &arm_fwd, &arm_lr, &arm_shuf, &arm_lrfuse] {
        arm.report();
    }
    print!("ARM LR pair-resolution census:");
    for (d, &count) in lr_depth.iter().enumerate().skip(1) {
        if d <= STAGES {
            print!(" d{d} {count}");
        } else {
            print!(" left-only {count}");
        }
    }
    println!();

    let delta = arm_lr.top1() - arm_l.top1();
    let gate_margin = delta >= 2.0;
    let gate_fwd = arm_lr.top1() > arm_fwd.top1();
    let gate_shuf = arm_lr.top1() > arm_shuf.top1();
    println!(
        "exit rule: LR - L = {delta:+.2}pp (need >= +2.00) | LR > FWD {gate_fwd} \
         ({:+.2}pp) | LR > SHUF {gate_shuf} ({:+.2}pp)",
        arm_lr.top1() - arm_fwd.top1(),
        arm_lr.top1() - arm_shuf.top1()
    );
    println!(
        "VERDICT: two-sided keying {}",
        if gate_margin && gate_fwd && gate_shuf {
            "CONFIRMED"
        } else {
            "NOT CONFIRMED"
        }
    );
    println!(
        "HONESTY NOTE (repeat): infill/analysis only — right context is not \
         available to causal generation; do not quote as a generation number."
    );
}

/// One row of the distribution-class growth table.
struct GrowthRow {
    frac: usize,
    stories: usize,
    positions: u64,
    left_keys: usize,
    pair_keys: usize,
    left_top1: usize,
    left_sig: usize,
    pair_top1: usize,
    pair_sig: usize,
}

/// Quantized distribution signature: the top `top` tokens by count (ties
/// to the lowest token id) with each probability floored into `bins`
/// bins. Two keys share a signature when their leading next-token mass
/// agrees within one bin, the documented tolerance proxy for Sestier
/// distributional equivalence.
fn signature(cell: &Cell, top: usize, bins: u64) -> Signature {
    let total = cell.total.max(1);
    let mut ranked: Vec<(u32, u64)> = cell.dist.iter().map(|(&t, &n)| (t, n)).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked.truncate(top);
    ranked
        .into_iter()
        .map(|(t, n)| (t, ((n * bins) / total) as u16))
        .collect()
}

#[test]
#[ignore = "measurement harness; run explicitly with --ignored"]
fn distribution_class_growth() {
    let meta_path = std::env::var("R4_CORPUS_META").unwrap_or_else(|_| fixture("c_meta.bin"));
    let recs_path = std::env::var("R4_CORPUS_RECS").unwrap_or_else(|_| fixture("c_recs.bin"));
    let art_path = std::env::var("R4_ARTIFACTS").unwrap_or_else(|_| fixture("tless_artifacts.bin"));
    let c = compiler::load_corpus_from(&meta_path, &recs_path).expect("corpus");
    let art = compiler::load_artifacts_from(&art_path).expect("artifacts");
    let r_len = env_usize("R4_TS_RIGHT_R", 4);
    let sig_top = env_usize("R4_TS_SIG_TOP", 4);
    let sig_bins = env_usize("R4_TS_SIG_BINS", 16) as u64;
    let min_sup = env_usize("R4_TS_MIN_SUP", 1) as u64;
    let threads = env_usize(
        "R4_TS_THREADS",
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1),
    );
    let fracs: Vec<usize> = std::env::var("R4_TS_GROWTH_FRACS")
        .unwrap_or_else(|_| "6,12,25,50,100".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    println!("corpus: {meta_path} + {recs_path} ({} records)", c.n);
    println!("artifacts: {art_path}");
    println!(
        "caps: R4_TS_RIGHT_R={r_len} R4_TS_SIG_TOP={sig_top} R4_TS_SIG_BINS={sig_bins} \
         R4_TS_MIN_SUP={min_sup} R4_TS_THREADS={threads} R4_TS_GROWTH_FRACS={fracs:?}"
    );
    println!(
        "HONESTY NOTE: the two-sided class counts below describe an \
         infill/analysis keying, not a causal generation mechanism."
    );

    let constr = construction_split(&c);
    let is_constr = |sid: u32| constr[sid as usize];
    let left_codes = derive_left_codes(&art, &c, threads);
    let (right_codes, right_ok) = derive_right_codes(&art, &c, r_len, threads);

    // nested prefixes by story id: rank construction stories in ascending
    // id order, so a smaller fraction's story set is a subset of a larger
    // one's by construction.
    let constr_ids: Vec<u32> = (0..c.stories as u32).filter(|&s| is_constr(s)).collect();
    let mut rank = vec![usize::MAX; c.stories as usize];
    for (r, &sid) in constr_ids.iter().enumerate() {
        rank[sid as usize] = r;
    }
    let mut cuts: Vec<(usize, usize)> = fracs
        .iter()
        .map(|&f| (f, (constr_ids.len() * f).div_ceil(100).max(1)))
        .collect();
    cuts.sort_by_key(|&(_, n)| n);
    cuts.dedup_by_key(|&mut (_, n)| n);

    let mut left4: Level = HashMap::new();
    let mut pair4: PairLevel = HashMap::new();
    let mut positions_seen = 0u64;
    let mut rows: Vec<GrowthRow> = Vec::new();
    let mut cut_idx = 0usize;

    let snapshot = |left4: &Level,
                    pair4: &PairLevel,
                    positions: u64,
                    frac: usize,
                    stories: usize|
     -> GrowthRow {
        let mut left_top1: HashSet<u32> = HashSet::new();
        let mut left_sig: HashSet<Signature> = HashSet::new();
        for cell in left4.values() {
            if cell.total < min_sup {
                continue;
            }
            if let Some(t) = argmax(&cell.dist) {
                left_top1.insert(t);
            }
            left_sig.insert(signature(cell, sig_top, sig_bins));
        }
        let mut pair_top1: HashSet<u32> = HashSet::new();
        let mut pair_sig: HashSet<Signature> = HashSet::new();
        for cell in pair4.values() {
            if cell.total < min_sup {
                continue;
            }
            if let Some(t) = argmax(&cell.dist) {
                pair_top1.insert(t);
            }
            pair_sig.insert(signature(cell, sig_top, sig_bins));
        }
        GrowthRow {
            frac,
            stories,
            positions,
            left_keys: left4.len(),
            pair_keys: pair4.len(),
            left_top1: left_top1.len(),
            left_sig: left_sig.len(),
            pair_top1: pair_top1.len(),
            pair_sig: pair_sig.len(),
        }
    };

    #[allow(clippy::needless_range_loop)] // index i addresses parallel corpus arrays
    for i in 0..c.n {
        let sid = c.story[i];
        if !is_constr(sid) {
            continue;
        }
        let r = rank[sid as usize];
        while cut_idx < cuts.len() && r >= cuts[cut_idx].1 {
            let (frac, stories) = cuts[cut_idx];
            rows.push(snapshot(&left4, &pair4, positions_seen, frac, stories));
            cut_idx += 1;
        }
        let truth = c.next[i];
        positions_seen += 1;
        left4
            .entry(pack(&left_codes[i], STAGES))
            .or_default()
            .add(truth);
        if right_ok[i] {
            let key = pack_pair(pack(&left_codes[i], STAGES), pack(&right_codes[i], STAGES));
            pair4.entry(key).or_default().add(truth);
        }
    }
    while cut_idx < cuts.len() {
        let (frac, stories) = cuts[cut_idx];
        rows.push(snapshot(&left4, &pair4, positions_seen, frac, stories));
        cut_idx += 1;
    }

    println!("---- distribution-class growth law (construction prefixes) ----");
    println!(
        "{:>5} {:>7} {:>9} {:>10} {:>10} {:>9} {:>9} {:>9} {:>9} {:>8} {:>8}",
        "frac%",
        "stories",
        "positions",
        "L-keys",
        "LR-keys",
        "L-top1",
        "L-sig",
        "LR-top1",
        "LR-sig",
        "sup/L",
        "sup/LR"
    );
    for row in &rows {
        println!(
            "{:>5} {:>7} {:>9} {:>10} {:>10} {:>9} {:>9} {:>9} {:>9} {:>8.2} {:>8.2}",
            row.frac,
            row.stories,
            row.positions,
            row.left_keys,
            row.pair_keys,
            row.left_top1,
            row.left_sig,
            row.pair_top1,
            row.pair_sig,
            row.positions as f64 / row.left_keys.max(1) as f64,
            row.positions as f64 / row.pair_keys.max(1) as f64
        );
    }
    println!("---- log-log growth slopes (d log classes / d log positions) ----");
    for w in rows.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let dlp = (b.positions.max(1) as f64 / a.positions.max(1) as f64).ln();
        let slope = |x: usize, y: usize| {
            if dlp <= 0.0 || x == 0 {
                0.0
            } else {
                (y.max(1) as f64 / x.max(1) as f64).ln() / dlp
            }
        };
        println!(
            "{:>3}% -> {:>3}%: L-keys {:.3} | LR-keys {:.3} | L-sig {:.3} | LR-sig {:.3}",
            a.frac,
            b.frac,
            slope(a.left_keys, b.left_keys),
            slope(a.pair_keys, b.pair_keys),
            slope(a.left_sig, b.left_sig),
            slope(a.pair_sig, b.pair_sig)
        );
    }
    if let (Some(first), Some(last)) = (rows.first(), rows.last()) {
        let dlp = (last.positions.max(1) as f64 / first.positions.max(1) as f64).ln();
        let slope = |x: usize, y: usize| {
            if dlp <= 0.0 {
                0.0
            } else {
                (y.max(1) as f64 / x.max(1) as f64).ln() / dlp
            }
        };
        println!(
            "overall: L-keys {:.3} | LR-keys {:.3} | L-sig {:.3} | LR-sig {:.3}",
            slope(first.left_keys, last.left_keys),
            slope(first.pair_keys, last.pair_keys),
            slope(first.left_sig, last.left_sig),
            slope(first.pair_sig, last.pair_sig)
        );
        println!(
            "reading: a slope near zero means the class count saturates and a \
             fixed region budget is defensible; a slope well above zero means \
             the natural class count keeps growing with data, so any fixed \
             region budget or absolute split threshold is wrong at scale."
        );
    }
}
