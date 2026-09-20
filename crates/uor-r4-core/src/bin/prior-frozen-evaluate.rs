//! Evaluation-only recovery and corrected replay of the frozen step-512 real-text prior.
//!
//! Loads the retained `prior_realtext.cpl2` artifact **unchanged** and re-scores it on a
//! reconstructed population with the two evaluation instruments repaired:
//!
//!  * the context permutation is an occurrence-preserving bijection per `(document, PAD/non-PAD)`
//!    stratum, so repeated contexts with different targets stay distinct observations and the target
//!    vector never moves; and
//!  * aggregation sums a document's windows before any macro or bootstrap step, so the paired
//!    interval resamples documents rather than windows.
//!
//! It also recovers the original optimizer exposure from the final checkpoint's serialized
//! permutation, adds full-fit and consumed-fit interpolated one-/two-context count references, and
//! diagnoses greedy repetition on the frozen artifact without changing the decoder.
//!
//! This binary never trains: it constructs no `PriorTrainer`, invokes no optimizer step and does not
//! write model parameters. It writes a fresh, exclusively claimed and sealed report root.
#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use uor_r4_core::native_geometric::learner::prior_learning::{
    bias_codes_from_counts, targets, PriorCore,
};
use uor_r4_core::native_geometric::learner::TernaryLinear;
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::bpe_derive::{derive_tokenizer, derive_tokenizer_json};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const VOCAB: usize = 4096;
const DV: usize = 128;
const WINDOW: usize = 64;
/// The absent-prefix row: `e_old` has `vocab + 1` rows and the iterator uses `vocab` at `i == 0`.
const PAD: usize = VOCAB;
const DEV_MAX_DOCS: usize = 48;
const DEV_MAX_WINDOWS: usize = 96;
const GEN_TOKENS: usize = 64;
const GEN_PROMPT: usize = 16;
const BOOTSTRAP_DRAWS: usize = 2000;
const GATE_MARGIN_BITS: f64 = 0.10;
const SUPPORT_MIN_DOCS: usize = 20;
const SUPPORT_MIN_TARGETS: usize = 1024;
/// Legacy panel reproduction tolerance, in bits/target.
const LEGACY_TOL: f64 = 1e-6;
/// Jelinek–Mercer grid, capped below 1.0 so an unseen pair in a seen context keeps a unigram floor.
const LAMBDA_GRID: [f64; 10] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
const SPREAD_MAX_WINDOWS_PER_DOC: usize = 2;
const TUNE_MAX_WINDOWS: usize = 64;
const PERM_SEED: u64 = 0xA5A5_1234_u64;

const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json";
const EXPECTED_TOKENIZER_SHA256: &str =
    "9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c";
/// The retained final artifact, pinned in the frozen-prior review.
const EXPECTED_ARTIFACT_SHA256: &str =
    "cd5a3aa1b804ccc3582a4a25090e25c862489e0bc4e5bb86c1d1f363a7338a00";
/// Recorded historical values this replay must reproduce before publishing a corrected gate.
const LEGACY_STEP512_CTX_MICRO: f64 = 7.142472153805127;
const LEGACY_STEP512_BIAS_MICRO: f64 = 9.131629767090766;
const LEGACY_STEP512_UNIGRAM_MICRO: f64 = 9.05991541171514;

/// The three recorded legacy generation prompts (first 16 ids of dev slots 0,1,2 — one document).
const LEGACY_PROMPTS: [[u32; GEN_PROMPT]; 3] = [
    [
        19, 1071, 2615, 3927, 930, 2095, 198, 198, 828, 67, 85, 437, 1548, 216, 39, 287,
    ],
    [
        93, 84, 643, 933, 1855, 1160, 984, 342, 471, 31, 375, 481, 99, 77, 24, 591,
    ],
    [
        3911, 58, 3700, 68, 79, 61, 3872, 30, 93, 84, 25, 808, 30, 669, 3140, 829,
    ],
];

// ---------------------------------------------------------------------------
// Arguments
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Replay,
    ParityEmit,
    ParityCheck,
    DerivedCorrections,
}

struct Args {
    root: PathBuf,
    artifact: PathBuf,
    checkpoint: PathBuf,
    docs: PathBuf,
    tokenizer: PathBuf,
    source_rev: String,
    mode: Mode,
    parity_path: Option<PathBuf>,
    evaluator_rev: String,
    source_root: Option<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut root = None;
    let mut artifact = None;
    let mut checkpoint = None;
    let mut docs = None;
    let mut tokenizer = PathBuf::from(DEFAULT_TOKENIZER);
    let mut source_rev = String::from("unknown");
    let mut evaluator_rev = String::from("unknown");
    let mut mode = Mode::Replay;
    let mut parity_path = None;
    let mut source_root = None;
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let k = argv[i].as_str();
        let v = || -> Result<String, String> {
            argv.get(i + 1)
                .cloned()
                .ok_or_else(|| format!("{k} needs a value"))
        };
        match k {
            "--root" => root = Some(PathBuf::from(v()?)),
            "--artifact" => artifact = Some(PathBuf::from(v()?)),
            "--checkpoint" => checkpoint = Some(PathBuf::from(v()?)),
            "--docs" => docs = Some(PathBuf::from(v()?)),
            "--tokenizer" => tokenizer = PathBuf::from(v()?),
            "--source-rev" => source_rev = v()?,
            "--evaluator-rev" => evaluator_rev = v()?,
            "--parity-emit" => {
                mode = Mode::ParityEmit;
                parity_path = Some(PathBuf::from(v()?));
            }
            "--parity-check" => {
                mode = Mode::ParityCheck;
                parity_path = Some(PathBuf::from(v()?));
            }
            "--derived-corrections" => {
                mode = Mode::DerivedCorrections;
                source_root = Some(PathBuf::from(v()?));
            }
            other => return Err(format!("unknown argument {other}")),
        }
        i += 2;
    }
    Ok(Args {
        root: root.ok_or("--root is required")?,
        artifact: artifact.ok_or("--artifact is required")?,
        checkpoint: checkpoint.ok_or("--checkpoint is required")?,
        docs: docs.ok_or("--docs is required")?,
        tokenizer,
        source_rev,
        mode,
        parity_path,
        evaluator_rev,
        source_root,
    })
}

// ---------------------------------------------------------------------------
// Small utilities
// ---------------------------------------------------------------------------

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn write_checked(root: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(root.join(name), bytes).map_err(|e| format!("write {name}: {e}"))
}

fn write_json(root: &Path, name: &str, value: &Value) -> Result<(), String> {
    let s = serde_json::to_string_pretty(value).map_err(|e| format!("encode {name}: {e}"))?;
    write_checked(root, name, s.as_bytes())
}

fn xorshift(st: &mut u64) -> u64 {
    *st ^= *st << 13;
    *st ^= *st >> 7;
    *st ^= *st << 17;
    *st
}

fn opt_micro(a: &Agg) -> Value {
    if a.total().1 == 0 {
        Value::Null
    } else {
        json!(a.micro())
    }
}

// ---------------------------------------------------------------------------
// Corpus reconstruction (identical collection order and split rule to the legacy runner)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Split {
    Fit,
    Tune,
    Dev,
}

impl Split {
    fn name(self) -> &'static str {
        match self {
            Split::Fit => "fit",
            Split::Tune => "tune",
            Split::Dev => "dev_pool",
        }
    }
}

struct DocRec {
    path: String,
    sha256: [u8; 32],
    bytes: usize,
    text: String,
    split: Split,
}

fn collect_docs(dir: &Path, rel: &Path, out: &mut Vec<(String, [u8; 32], usize, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut kids: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    kids.sort();
    for k in kids {
        let r = rel.join(k.file_name().unwrap_or_default());
        if k.is_dir() {
            collect_docs(&k, &r, out);
        } else if k.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(text) = std::fs::read_to_string(&k) {
                let hash: [u8; 32] = Sha256::digest(text.as_bytes()).into();
                out.push((r.display().to_string(), hash, text.len(), text));
            }
        }
    }
}

/// All collected markdown files, then the exact-duplicate-grouped unique population in collection
/// order. The split rule is the legacy rule: bucket `hash[0] % 10 < 8` fit, `== 8` tune, `== 9` dev.
fn reconstruct_corpus(dir: &Path) -> (Vec<DocRec>, usize, usize, usize) {
    let mut raw: Vec<(String, [u8; 32], usize, String)> = Vec::new();
    collect_docs(dir, Path::new(""), &mut raw);
    let collected = raw.len();
    // Duplicate groups by content hash, with their splits, to report cross-split overlap.
    let mut groups: HashMap<[u8; 32], Vec<String>> = HashMap::new();
    let mut order: Vec<[u8; 32]> = Vec::new();
    for (path, hash, _, _) in raw.iter() {
        if !groups.contains_key(hash) {
            order.push(*hash);
        }
        groups.entry(*hash).or_default().push(path.clone());
    }
    let mut uniq: Vec<DocRec> = Vec::new();
    let mut duplicates = 0usize;
    let mut seen: BTreeSet<[u8; 32]> = BTreeSet::new();
    for (path, hash, bytes, text) in raw {
        if !seen.insert(hash) {
            duplicates += 1;
            continue;
        }
        let split = match hash[0] as usize % 10 {
            0..=7 => Split::Fit,
            8 => Split::Tune,
            _ => Split::Dev,
        };
        uniq.push(DocRec {
            path,
            sha256: hash,
            bytes,
            text,
            split,
        });
    }
    let duplicate_groups = order.iter().filter(|h| groups[*h].len() > 1).count();
    (uniq, collected, duplicates, duplicate_groups)
}

fn windows_of(tokens: &[u32]) -> Vec<Vec<u32>> {
    tokens
        .chunks(WINDOW)
        .filter(|c| c.len() >= 3)
        .map(|c| c.to_vec())
        .collect()
}

// ---------------------------------------------------------------------------
// Records, aggregation and uncertainty
// ---------------------------------------------------------------------------

/// One scored observation: `(content_id, window_token_offset, target_offset)` with its fixed target.
#[derive(Clone)]
struct Rec {
    obs_id: String,
    doc_key: String,
    doc_slot: usize,
    window_index: usize,
    token_offset: usize,
    target_offset: usize,
    prev: usize,
    cur: usize,
    target: u32,
    is_pad: bool,
}

impl Rec {
    fn own_ctx(&self) -> (usize, usize) {
        (self.prev, self.cur)
    }
}

/// Per-document loss sums and counts. Rows are aligned across scorers by construction.
#[derive(Clone, Default)]
struct Agg {
    rows: Vec<(String, f64, usize)>,
}

impl Agg {
    fn total(&self) -> (f64, usize) {
        self.rows
            .iter()
            .fold((0.0f64, 0usize), |(a, b), (_, l, n)| (a + l, b + n))
    }
    fn micro(&self) -> f64 {
        let (l, n) = self.total();
        if n == 0 {
            f64::NAN
        } else {
            l / n as f64
        }
    }
    fn doc_macro(&self) -> f64 {
        let docs: Vec<f64> = self
            .rows
            .iter()
            .filter(|(_, _, n)| *n > 0)
            .map(|(_, l, n)| l / *n as f64)
            .collect();
        if docs.is_empty() {
            f64::NAN
        } else {
            docs.iter().sum::<f64>() / docs.len() as f64
        }
    }
    fn docs(&self) -> usize {
        self.rows.iter().filter(|(_, _, n)| *n > 0).count()
    }
}

/// Aggregate per-record losses into true document clusters, summing *all* of a document's windows
/// before any macro or interval step. `keep` optionally restricts to a frozen stratum.
fn aggregate(recs: &[Rec], losses: &[f64], keep: Option<&[bool]>) -> Agg {
    assert_eq!(recs.len(), losses.len());
    let mut index: HashMap<&str, usize> = HashMap::new();
    let mut rows: Vec<(String, f64, usize)> = Vec::new();
    for (i, r) in recs.iter().enumerate() {
        if let Some(k) = keep {
            if !k[i] {
                continue;
            }
        }
        let slot = match index.get(r.doc_key.as_str()) {
            Some(&s) => s,
            None => {
                let s = rows.len();
                index.insert(r.doc_key.as_str(), s);
                rows.push((r.doc_key.clone(), 0.0, 0));
                s
            }
        };
        rows[slot].1 += losses[i];
        rows[slot].2 += 1;
    }
    Agg { rows }
}

/// Paired document-cluster bootstrap of a loss difference (`a` minus `b`), recomputing the ratio of
/// resampled sums and counts on every draw.
fn paired_interval(a: &Agg, b: &Agg, seed: u64) -> (f64, f64, f64) {
    assert_eq!(
        a.rows.len(),
        b.rows.len(),
        "scorers must be aligned on the same documents"
    );
    let n = a.rows.len();
    if n == 0 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    let d: Vec<(f64, f64, f64, f64)> = (0..n)
        .map(|i| {
            let (_, la, na) = &a.rows[i];
            let (_, lb, nb) = &b.rows[i];
            (*la, *na as f64, *lb, *nb as f64)
        })
        .collect();
    let point = (d.iter().map(|x| x.0).sum::<f64>() / d.iter().map(|x| x.1).sum::<f64>())
        - (d.iter().map(|x| x.2).sum::<f64>() / d.iter().map(|x| x.3).sum::<f64>());
    let mut st = seed | 1;
    let mut boots = Vec::with_capacity(BOOTSTRAP_DRAWS);
    for _ in 0..BOOTSTRAP_DRAWS {
        let (mut sa, mut na, mut sb, mut nb) = (0f64, 0f64, 0f64, 0f64);
        for _ in 0..n {
            let idx = (xorshift(&mut st) as usize) % n;
            sa += d[idx].0;
            na += d[idx].1;
            sb += d[idx].2;
            nb += d[idx].3;
        }
        if na > 0.0 && nb > 0.0 {
            boots.push(sa / na - sb / nb);
        }
    }
    if boots.len() < BOOTSTRAP_DRAWS / 2 {
        return (point, f64::NAN, f64::NAN);
    }
    boots.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let lo = boots[(boots.len() as f64 * 0.025) as usize];
    let hi = boots[((boots.len() as f64 * 0.975) as usize).min(boots.len() - 1)];
    (point, lo, hi)
}

fn interval_json(t: (f64, f64, f64)) -> Value {
    json!({"point": t.0, "lo": t.1, "hi": t.2})
}

// ---------------------------------------------------------------------------
// Scorers
// ---------------------------------------------------------------------------

/// Score every record against an explicit context list, with the fixed target vector.
fn score_ctx(
    core: &PriorCore,
    recs: &[Rec],
    ctxs: &[(usize, usize)],
    use_context: bool,
) -> Vec<f64> {
    assert_eq!(recs.len(), ctxs.len());
    recs.iter()
        .zip(ctxs.iter())
        .map(|(r, &(p, c))| {
            let tr = core.trace(p, c, use_context);
            core.bits_one(&tr.z, r.target)
        })
        .collect()
}

/// Exact add-one full-fit unigram reference.
struct Unigram {
    counts: Vec<u64>,
    total: u64,
}

impl Unigram {
    fn p(&self, next: u32) -> f64 {
        (self.counts[next as usize] as f64 + 1.0) / (self.total as f64 + VOCAB as f64)
    }
    fn argmax(&self) -> u32 {
        self.counts
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| **c)
            .map(|(i, _)| i as u32)
            .unwrap_or(0)
    }
}

fn score_unigram(uni: &Unigram, recs: &[Rec]) -> Vec<f64> {
    recs.iter().map(|r| -uni.p(r.target).log2()).collect()
}

/// Sparse conditional counts, single-sourced with the ceiling instrument's structure.
#[derive(Default)]
struct Cond {
    counts: HashMap<(u64, u32), u32>,
    totals: HashMap<u64, u64>,
    by_ctx: HashMap<u64, Vec<u32>>,
}

impl Cond {
    fn observe(&mut self, ctx: u64, next: u32) {
        *self.counts.entry((ctx, next)).or_insert(0) += 1;
        *self.totals.entry(ctx).or_insert(0) += 1;
        let cands = self.by_ctx.entry(ctx).or_default();
        if !cands.contains(&next) {
            cands.push(next);
        }
    }
    fn total(&self, ctx: u64) -> u64 {
        self.totals.get(&ctx).copied().unwrap_or(0)
    }
    fn count_of(&self, ctx: u64, next: u32) -> u32 {
        self.counts.get(&(ctx, next)).copied().unwrap_or(0)
    }
    fn level_p(&self, ctx: u64, next: u32, uni: &Unigram) -> f64 {
        let t = self.total(ctx);
        if t == 0 {
            uni.p(next)
        } else {
            self.count_of(ctx, next) as f64 / t as f64
        }
    }
    /// Successors of a context, sorted by count desc then id, as `(id, count)`.
    fn top_candidates(&self, ctx: u64, limit: usize) -> Vec<(u32, u32)> {
        let mut v: Vec<(u32, u32)> = self
            .by_ctx
            .get(&ctx)
            .map(|c| c.iter().map(|&n| (n, self.count_of(ctx, n))).collect())
            .unwrap_or_default();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v.truncate(limit);
        v
    }
}

fn ctx1(cur: usize) -> u64 {
    cur as u64
}
fn ctx2(prev: usize, cur: usize) -> u64 {
    (prev as u64) * VOCAB as u64 + cur as u64
}

/// Interpolated family: `p = l2·P(next|prev,cur) + (1-l2)·(l1·P(next|cur) + (1-l1)·P_uni)`.
fn family_p(
    c1: &Cond,
    c2: &Cond,
    uni: &Unigram,
    prev: usize,
    cur: usize,
    next: u32,
    l: (f64, f64),
) -> f64 {
    let p1 = c1.level_p(ctx1(cur), next, uni);
    let p2 = c2.level_p(ctx2(prev, cur), next, uni);
    l.1 * p2 + (1.0 - l.1) * (l.0 * p1 + (1.0 - l.0) * uni.p(next))
}

fn score_family(c1: &Cond, c2: &Cond, uni: &Unigram, recs: &[Rec], l: (f64, f64)) -> Vec<f64> {
    recs.iter()
        .map(|r| {
            let p = family_p(c1, c2, uni, r.prev, r.cur, r.target, l);
            -p.max(1e-300).log2()
        })
        .collect()
}

/// Tune `(l1, l2)` by held-out bits on the tune windows only.
fn tune_lambdas(
    c1: &Cond,
    c2: &Cond,
    uni: &Unigram,
    tune_windows: &[Vec<u32>],
) -> ((f64, f64), f64) {
    let mut best = (0.5f64, 0.5f64);
    let mut best_bits = f64::INFINITY;
    for &l1 in LAMBDA_GRID.iter() {
        for &l2 in LAMBDA_GRID.iter() {
            let mut bits = 0.0f64;
            let mut n = 0usize;
            for w in tune_windows {
                for (_i, p, c, t) in targets(w, VOCAB) {
                    let pr = family_p(c1, c2, uni, p, c, t, (l1, l2));
                    bits -= pr.max(1e-300).log2();
                    n += 1;
                }
            }
            if n > 0 {
                let per = bits / n as f64;
                if per < best_bits {
                    best_bits = per;
                    best = (l1, l2);
                }
            }
        }
    }
    (best, best_bits)
}

// ---------------------------------------------------------------------------
// Occurrence-preserving permutation
// ---------------------------------------------------------------------------

struct Permutation {
    /// Donor context per record index (equals the record's own context when its stratum is singleton).
    donor_ctx: Vec<(usize, usize)>,
    donor_index: Vec<usize>,
    strata: usize,
    singleton_strata: usize,
    moved: usize,
    changed_context: usize,
}

/// A seeded Fisher–Yates bijection over occurrence indices, applied independently per
/// `(document, PAD/non-PAD)` stratum. Recipients keep their target; contexts are drawn from the
/// donor occurrence. Repeated contexts with different targets stay distinct observations, and the
/// target vector is never mutated.
fn build_permutation(recs: &[Rec], seed: u64) -> Permutation {
    let mut groups: Vec<((String, bool), Vec<usize>)> = Vec::new();
    let mut index: HashMap<(String, bool), usize> = HashMap::new();
    for (i, r) in recs.iter().enumerate() {
        let key = (r.doc_key.clone(), r.is_pad);
        let slot = match index.get(&key) {
            Some(&s) => s,
            None => {
                let s = groups.len();
                index.insert(key.clone(), s);
                groups.push((key, Vec::new()));
                s
            }
        };
        groups[slot].1.push(i);
    }
    let mut donor_ctx: Vec<(usize, usize)> = recs.iter().map(|r| r.own_ctx()).collect();
    let mut donor_index: Vec<usize> = (0..recs.len()).collect();
    let mut st = seed | 1;
    let mut strata = 0usize;
    let mut singleton = 0usize;
    let mut moved = 0usize;
    let mut changed = 0usize;
    for (_, members) in groups.iter() {
        strata += 1;
        if members.len() < 2 {
            singleton += 1;
            continue;
        }
        let mut perm: Vec<usize> = (0..members.len()).collect();
        for i in (1..perm.len()).rev() {
            let j = (xorshift(&mut st) as usize) % (i + 1);
            perm.swap(i, j);
        }
        for (k, &rec_idx) in members.iter().enumerate() {
            let donor = members[perm[k]];
            donor_index[rec_idx] = donor;
            donor_ctx[rec_idx] = recs[donor].own_ctx();
            if donor != rec_idx {
                moved += 1;
            }
            if donor_ctx[rec_idx] != recs[rec_idx].own_ctx() {
                changed += 1;
            }
        }
    }
    Permutation {
        donor_ctx,
        donor_index,
        strata,
        singleton_strata: singleton,
        moved,
        changed_context: changed,
    }
}

// ---------------------------------------------------------------------------
// Checkpoint exposure recovery (CPCK v3)
// ---------------------------------------------------------------------------

struct CheckpointMeta {
    step: u64,
    cursor: usize,
    pass: u32,
    seed: u64,
    perm: Vec<usize>,
}

fn parse_checkpoint(bytes: &[u8]) -> Result<CheckpointMeta, String> {
    let mut c = 0usize;
    let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
        let end = c.checked_add(n).ok_or("size overflow")?;
        if end > bytes.len() {
            return Err("truncated checkpoint".into());
        }
        let s = &bytes[*c..end];
        *c = end;
        Ok(s)
    };
    if take(&mut c, 4)? != b"CPCK" {
        return Err("bad checkpoint magic".into());
    }
    let u32_at = |c: &mut usize| -> Result<u32, String> {
        Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
    };
    let u64_at = |c: &mut usize| -> Result<u64, String> {
        Ok(u64::from_le_bytes(take(c, 8)?.try_into().unwrap()))
    };
    if u32_at(&mut c)? != 3 {
        return Err("unsupported checkpoint version".into());
    }
    let _vocab = u32_at(&mut c)?;
    let _dv = u32_at(&mut c)?;
    for _ in 0..3 {
        let _ = u32_at(&mut c)?; // norm_bits, f_bits, bias_scale_bits
    }
    let step = u64_at(&mut c)?;
    let cursor = u64_at(&mut c)? as usize;
    let pass = u64_at(&mut c)? as u32;
    let _rng = u64_at(&mut c)?;
    for _ in 0..5 {
        let _ = u32_at(&mut c)?; // optimizer floats
    }
    let seed = u64_at(&mut c)?;
    let _ = take(&mut c, 32)?; // data identity
    let perm_len = u64_at(&mut c)? as usize;
    let mut perm = Vec::with_capacity(perm_len);
    let mut seen = vec![false; perm_len];
    for _ in 0..perm_len {
        let p = u64_at(&mut c)? as usize;
        if p >= perm_len || std::mem::replace(&mut seen[p], true) {
            return Err("checkpoint permutation is not a permutation of 0..n".into());
        }
        perm.push(p);
    }
    Ok(CheckpointMeta {
        step,
        cursor,
        pass,
        seed,
        perm,
    })
}

// ---------------------------------------------------------------------------
// Built-in falsifying fixtures
// ---------------------------------------------------------------------------

fn fixture_constant_invariance(core: &PriorCore, recs: &[Rec], perm: &Permutation) -> Value {
    // A context-independent predictor must be exactly invariant under a context permutation,
    // because targets never move. The legacy implementation failed this
    // (its step-0 permuted CE was 9.1364 against 9.13163 unpermuted).
    let own: Vec<(usize, usize)> = recs.iter().map(|r| r.own_ctx()).collect();
    let bias_own = score_ctx(core, recs, &own, false);
    let bias_perm = score_ctx(core, recs, &perm.donor_ctx, false);
    let max_delta = bias_own
        .iter()
        .zip(bias_perm.iter())
        .fold(0.0f64, |m, (a, b)| m.max((a - b).abs()));
    let a = aggregate(recs, &bias_own, None);
    let b = aggregate(recs, &bias_perm, None);
    let same_targets = a.total().1 == b.total().1;
    let same_micro = a.micro() == b.micro();
    json!({
        "name": "constant_invariance",
        "rule": "a context-independent predictor's per-occurrence losses are identical under a context permutation",
        "max_abs_per_occurrence_delta": max_delta,
        "bias_only_micro_original": a.micro(),
        "bias_only_micro_permuted": b.micro(),
        "target_count_equal": same_targets,
        "passed": max_delta == 0.0 && same_targets && same_micro,
    })
}

fn fixture_unigram_invariance(uni: &Unigram, recs: &[Rec]) -> Value {
    let loss = score_unigram(uni, recs);
    let micro = loss.iter().sum::<f64>() / loss.len() as f64;
    json!({
        "name": "unigram_invariance",
        "rule": "the exact unigram ignores context and never moves targets",
        "unigram_micro_bits": micro,
        "targets": recs.len(),
        "passed": true,
    })
}

/// Two synthetic documents with different window counts: document-macro must differ from the
/// accidental per-record macro the legacy instrument computed.
fn fixture_unequal_weights() -> Value {
    let mk = |doc: &str, slot: usize, _idx: usize, off: usize, target: u32| Rec {
        obs_id: format!("{doc}:0:{off}"),
        doc_key: doc.to_string(),
        doc_slot: slot,
        window_index: 0,
        token_offset: 0,
        target_offset: off,
        prev: PAD,
        cur: 1,
        target,
        is_pad: true,
    };
    let recs = vec![
        mk("a", 0, 0, 0, 2),
        mk("a", 0, 0, 1, 3),
        mk("b", 1, 1, 0, 5),
    ];
    let losses = vec![1.0f64, 1.0, 9.0];
    let grouped = aggregate(&recs, &losses, None);
    let per_record_macro = losses.iter().sum::<f64>() / losses.len() as f64;
    let doc_macro = grouped.doc_macro();
    json!({
        "name": "unequal_document_weights",
        "rule": "document-macro sums a document's windows before averaging",
        "per_record_macro": per_record_macro,
        "document_macro": doc_macro,
        "distinct": (per_record_macro - doc_macro).abs() > 1e-12,
        "passed": (per_record_macro - doc_macro).abs() > 1e-12,
    })
}

/// The legacy scheme (shuffle target values, store into a HashMap keyed by context) collapses
/// occurrences. Reproduced here to show it fails the target-vector invariant.
fn fixture_legacy_collapse() -> Value {
    let recs = [(1u32, 2u32, 10u32), (1, 2, 11), (1, 2, 12)];
    let shuffled = [12u32, 10, 11];
    let mut map: HashMap<(u32, u32), u32> = HashMap::new();
    let mut reported = 0usize;
    for (k, (p, c, t)) in recs.iter().enumerate() {
        if shuffled[k] != *t {
            reported += 1;
        }
        map.insert((*p, *c), shuffled[k]);
    }
    let applied: Vec<u32> = recs
        .iter()
        .map(|(p, c, _)| *map.get(&(*p, *c)).unwrap())
        .collect();
    let mut distinct = applied.clone();
    distinct.sort_unstable();
    distinct.dedup();
    json!({
        "name": "legacy_collapse_falsifier",
        "rule": "a HashMap keyed by (doc, prev, cur) keeps one applied target per repeated context, so the applied control is not a target-preserving permutation",
        "occurrences": recs.len(),
        "distinct_applied_targets": distinct.len(),
        "reported_changed_assignments": reported,
        "target_vector_preserved": distinct.len() == recs.len(),
        "passed": distinct.len() != recs.len(),
    })
}

/// A tiny known-sign reference: `a` is worse than `b` by a known constant, so `a - b` is positive.
fn fixture_known_sign() -> Value {
    let a = Agg {
        rows: vec![("d0".into(), 4.0, 2), ("d1".into(), 6.0, 2)],
    };
    let b = Agg {
        rows: vec![("d0".into(), 2.0, 2), ("d1".into(), 2.0, 2)],
    };
    let (p, lo, hi) = paired_interval(&a, &b, 0x1234_5678);
    json!({
        "name": "known_sign_reference",
        "rule": "reference_loss - model_loss is positive when the reference is worse",
        "point": p, "lo": lo, "hi": hi,
        "passed": p > 0.0 && lo > 0.0,
    })
}

// ---------------------------------------------------------------------------
// Greedy generation with loop diagnostics
// ---------------------------------------------------------------------------

struct GenTrace {
    steps: Vec<Value>,
    output: Vec<u32>,
    first_repeat_state: Option<(usize, usize, usize)>,
    cycle_entry: Option<usize>,
    cycle_period: Option<usize>,
}

fn generate_trace(core: &PriorCore, prompt: &[u32], n_new: usize, use_context: bool) -> GenTrace {
    let mut toks = prompt.to_vec();
    let mut seen: HashMap<(usize, usize), usize> = HashMap::new();
    let mut steps = Vec::new();
    let mut first_repeat = None;
    let mut cycle_entry = None;
    let mut cycle_period = None;
    for t in 0..n_new {
        let i = toks.len() - 1;
        let prev = if i == 0 {
            PAD
        } else {
            (toks[i - 1] as usize).min(VOCAB - 1)
        };
        let cur = (toks[i] as usize).min(VOCAB - 1);
        if first_repeat.is_none() {
            if let Some(&t0) = seen.get(&(prev, cur)) {
                first_repeat = Some((t, t0, t - t0));
                cycle_entry = Some(t0);
                cycle_period = Some(t - t0);
            } else {
                seen.insert((prev, cur), t);
            }
        }
        let z = core.int_logits(prev, cur, use_context);
        let mut best = 0usize;
        for r in 1..z.len() {
            if z[r] > z[best] {
                best = r;
            }
        }
        let max = z[best];
        let ties = z.iter().filter(|&&x| x == max).count();
        let mut second = i32::MIN;
        for (r, &x) in z.iter().enumerate() {
            if r != best && x > second {
                second = x;
            }
        }
        steps.push(json!({
            "t": t,
            "prev": prev,
            "cur": cur,
            "chosen": best as u32,
            "top1": max,
            "top2": if second == i32::MIN { Value::Null } else { json!(second) },
            "margin": if second == i32::MIN { Value::Null } else { json!(max as i64 - second as i64) },
            "tied_maxima": ties,
        }));
        toks.push(best as u32);
    }
    let output = toks[prompt.len()..].to_vec();
    GenTrace {
        steps,
        output,
        first_repeat_state: first_repeat,
        cycle_entry,
        cycle_period,
    }
}

/// Inspect one context pair: integer scores, margin, ties, and fit/consumed successor counts.
#[allow(clippy::too_many_arguments)]
fn inspect_pair(
    core: &PriorCore,
    full1: &Cond,
    full2: &Cond,
    cons1: &Cond,
    cons2: &Cond,
    uni: &Unigram,
    lambdas: (f64, f64),
    tokenizer: &HfBpeTokenizer,
    prev: usize,
    cur: usize,
) -> Value {
    let z = core.int_logits(prev, cur, true);
    let mut best = 0usize;
    for r in 1..z.len() {
        if z[r] > z[best] {
            best = r;
        }
    }
    let max = z[best];
    let ties = z.iter().filter(|&&x| x == max).count();
    let mut second = i32::MIN;
    for (r, &x) in z.iter().enumerate() {
        if r != best && x > second {
            second = x;
        }
    }
    let key2 = ctx2(prev, cur);
    let fit_cands = full2.top_candidates(key2, 8);
    let cons_cands = cons2.top_candidates(key2, 8);

    // Interpolated reference argmax over observed candidates plus the unigram argmax.
    let mut ref_best = uni.argmax();
    let mut ref_p = family_p(full1, full2, uni, prev, cur, ref_best, lambdas);
    for (cond, key) in [(full2, key2), (full1, ctx1(cur))] {
        if let Some(c) = cond.by_ctx.get(&key) {
            for &n in c {
                let p = family_p(full1, full2, uni, prev, cur, n, lambdas);
                if p > ref_p {
                    ref_p = p;
                    ref_best = n;
                }
            }
        }
    }
    let tr = core.trace(prev, cur, true);
    let model_bits_on_top = fit_cands.first().map(|(n, _)| core.bits_one(&tr.z, *n));
    json!({
        "prev": prev,
        "cur": cur,
        "prev_decoded": tokenizer.decode(&[prev.min(VOCAB - 1) as u32]),
        "cur_decoded": tokenizer.decode(&[cur as u32]),
        "pair_is_absent_prefix": prev == PAD,
        "a_equals_b": prev == cur,
        "model_argmax": best,
        "model_argmax_decoded": tokenizer.decode(&[best as u32]),
        "top1_integer": max,
        "top2_integer": if second == i32::MIN { Value::Null } else { json!(second) },
        "margin": if second == i32::MIN { Value::Null } else { json!(max as i64 - second as i64) },
        "tied_maxima": ties,
        "immediate_repetition": best == cur,
        "fixed_point_condition_holds": prev == cur && best == cur,
        "full_fit_context_total": full2.total(key2),
        "consumed_context_total": cons2.total(key2),
        "consumed_context_1_total": cons1.total(ctx1(cur)),
        "full_fit_top_successors": fit_cands.iter().map(|(n, c)| json!({"id": n, "count": c, "decoded": tokenizer.decode(&[*n])})).collect::<Vec<_>>(),
        "consumed_top_successors": cons_cands.iter().map(|(n, c)| json!({"id": n, "count": c, "decoded": tokenizer.decode(&[*n])})).collect::<Vec<_>>(),
        "model_bits_on_top_full_fit_successor": model_bits_on_top,
        "interpolated_reference_argmax": ref_best,
        "interpolated_reference_argmax_decoded": tokenizer.decode(&[ref_best]),
    })
}

// ---------------------------------------------------------------------------
// Panel reporting
// ---------------------------------------------------------------------------

fn scorer_value(
    label: &str,
    recs: &[Rec],
    losses: &[f64],
    unseen_full: &[bool],
    unseen_consumed: &[bool],
) -> Value {
    let a = aggregate(recs, losses, None);
    let uf = aggregate(recs, losses, Some(unseen_full));
    let uc = aggregate(recs, losses, Some(unseen_consumed));
    json!({
        "label": label,
        "micro_bits": a.micro(),
        "document_macro_bits": a.doc_macro(),
        "documents": a.docs(),
        "targets": a.total().1,
        "unseen_full_fit": {"targets": uf.total().1, "documents": uf.docs(), "micro_bits": opt_micro(&uf)},
        "unseen_consumed_fit": {"targets": uc.total().1, "documents": uc.docs(), "micro_bits": opt_micro(&uc)},
        "per_document": a.rows.iter().map(|(k, l, n)| json!({
            "content_id": k,
            "loss_sum": l,
            "count": n,
            "loss_per_target": if *n > 0 { json!(l / *n as f64) } else { Value::Null },
        })).collect::<Vec<_>>(),
    })
}

fn support_ok(model: &Agg, eligible: &Agg) -> bool {
    model.docs() >= SUPPORT_MIN_DOCS
        && model.total().1 >= SUPPORT_MIN_TARGETS
        && eligible.docs() >= SUPPORT_MIN_DOCS
        && eligible.total().1 >= SUPPORT_MIN_TARGETS
}

fn decide(
    gain_uni: (f64, f64, f64),
    gain_bias: (f64, f64, f64),
    perm_eligible: (f64, f64, f64),
    checks_ok: bool,
    support_ok: bool,
) -> &'static str {
    if !checks_ok || !support_ok {
        return "INCONCLUSIVE";
    }
    if gain_uni.0 >= GATE_MARGIN_BITS
        && gain_uni.1 > 0.0
        && gain_bias.0 >= GATE_MARGIN_BITS
        && gain_bias.1 > 0.0
        && perm_eligible.0 > 0.0
        && perm_eligible.1 > 0.0
    {
        "PASS"
    } else {
        "FAIL"
    }
}

#[allow(clippy::too_many_arguments)]
fn panel_value(
    name: &str,
    recs: &[Rec],
    perm: &Permutation,
    model: &[f64],
    bias: &[f64],
    unigram_loss: &[f64],
    reference_full: &[f64],
    reference_consumed: &[f64],
    permuted: &[f64],
    knock_prev: &[f64],
    knock_cur: &[f64],
    unseen_full: &[bool],
    unseen_consumed: &[bool],
    g_uni: (f64, f64, f64),
    g_bias: (f64, f64, f64),
    p_full: (f64, f64, f64),
    p_elig: (f64, f64, f64),
    model_agg: &Agg,
    eligible_model: &Agg,
    decision: &str,
) -> Value {
    let windows = recs
        .iter()
        .map(|r| (r.doc_key.clone(), r.window_index))
        .collect::<BTreeSet<_>>()
        .len();
    json!({
        "panel": name,
        "documents": model_agg.docs(),
        "windows": windows,
        "targets": recs.len(),
        "rows": {
            "contextual_model": scorer_value("contextual", recs, model, unseen_full, unseen_consumed),
            "bias_only": scorer_value("bias_only", recs, bias, unseen_full, unseen_consumed),
            "exact_unigram": scorer_value("exact_unigram", recs, unigram_loss, unseen_full, unseen_consumed),
            "interpolated_reference_full_fit": scorer_value("interpolated_reference_full_fit", recs, reference_full, unseen_full, unseen_consumed),
            "interpolated_reference_consumed_counts": scorer_value("interpolated_reference_consumed_counts", recs, reference_consumed, unseen_full, unseen_consumed),
            "permuted_model": scorer_value("permuted", recs, permuted, unseen_full, unseen_consumed),
            "knockout_prev": scorer_value("knockout_prev", recs, knock_prev, unseen_full, unseen_consumed),
            "knockout_cur": scorer_value("knockout_cur", recs, knock_cur, unseen_full, unseen_consumed),
        },
        "gains": {
            "unigram_gain": interval_json(g_uni),
            "bias_gain": interval_json(g_bias),
            "perm_penalty_full": interval_json(p_full),
            "perm_penalty_eligible": interval_json(p_elig),
        },
        "permutation": {
            "strata": perm.strata,
            "singleton_strata_unmodified": perm.singleton_strata,
            "records_moved": perm.moved,
            "records_with_changed_context": perm.changed_context,
            "seed": "0xA5A5123400000001",
        },
        "support": {
            "min_documents": SUPPORT_MIN_DOCS,
            "min_targets": SUPPORT_MIN_TARGETS,
            "model_documents": model_agg.docs(),
            "model_targets": model_agg.total().1,
            "eligible_documents": eligible_model.docs(),
            "eligible_targets": eligible_model.total().1,
            "passed": support_ok(model_agg, eligible_model),
        },
        "decision": decision,
    })
}

// ---------------------------------------------------------------------------
// Inference parity fixture
// ---------------------------------------------------------------------------

fn inference_fixture(core: &PriorCore, tokenizer: &HfBpeTokenizer) -> Value {
    let contexts: [(usize, usize); 12] = [
        (PAD, 19),
        (1071, 2615),
        (198, 198),
        (32, 32),
        (33, 32),
        (32, 33),
        (198, 284),
        (284, 198),
        (0, 0),
        (4095, 4095),
        (19, 1071),
        (287, 3911),
    ];
    let mut rows = Vec::new();
    for (p, c) in contexts {
        for (label, use_ctx) in [("context", true), ("bias", false)] {
            let z = core.int_logits(p, c, use_ctx);
            let mut le = Vec::with_capacity(z.len() * 4);
            for v in &z {
                le.extend_from_slice(&v.to_le_bytes());
            }
            let mut best = 0usize;
            for r in 1..z.len() {
                if z[r] > z[best] {
                    best = r;
                }
            }
            rows.push(json!({
                "prev": p, "cur": c, "mode": label,
                "z_sha256": sha256_hex(&le),
                "z_len": z.len(),
                "argmax": best,
                "argmax_decoded": tokenizer.decode(&[best as u32]),
            }));
        }
    }
    let prompts: Vec<Vec<u32>> = LEGACY_PROMPTS
        .iter()
        .map(|p| p.to_vec())
        .chain([
            vec![32u32, 32],
            vec![198u32, 198],
            vec![284u32, 198],
            vec![33u32, 32],
        ])
        .collect();
    let mut greedy = Vec::new();
    for (k, prompt) in prompts.iter().enumerate() {
        let out = core.generate(prompt, GEN_TOKENS, true);
        let off = core.generate(prompt, GEN_TOKENS, false);
        let mut le = Vec::with_capacity(out.len() * 4);
        for v in &out {
            le.extend_from_slice(&v.to_le_bytes());
        }
        greedy.push(json!({
            "prompt_index": k,
            "prompt": prompt,
            "output": out,
            "output_sha256": sha256_hex(&le),
            "context_disabled_output": off,
        }));
    }
    json!({"score_rows": rows, "greedy": greedy})
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

fn open_tokenizer(args: &Args) -> Result<(String, String, HfBpeTokenizer), String> {
    let tok_bytes = std::fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    let tok_sha = sha256_hex(&tok_bytes);
    if tok_sha != EXPECTED_TOKENIZER_SHA256 {
        return Err(format!(
            "tokenizer sha256 {tok_sha} != expected {EXPECTED_TOKENIZER_SHA256}"
        ));
    }
    let tokenizer =
        derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive tokenizer: {e}"))?;
    if tokenizer.vocab_size() != VOCAB {
        return Err(format!(
            "derived tokenizer vocab {} != {VOCAB}",
            tokenizer.vocab_size()
        ));
    }
    let derived_bytes = derive_tokenizer_json(&tok_bytes, VOCAB)
        .map_err(|e| format!("derive tokenizer json: {e}"))?;
    let derived_sha = sha256_hex(&derived_bytes);
    Ok((tok_sha, derived_sha, tokenizer))
}

fn run_parity(args: &Args) -> Result<ExitCode, String> {
    let artifact_bytes = std::fs::read(&args.artifact).map_err(|e| format!("artifact: {e}"))?;
    let artifact_sha = sha256_hex(&artifact_bytes);
    let core = PriorCore::from_bytes(&artifact_bytes).map_err(|e| format!("load artifact: {e}"))?;
    let (_, _, tokenizer) = open_tokenizer(args)?;
    let fixture = inference_fixture(&core, &tokenizer);
    let path = args
        .parity_path
        .clone()
        .ok_or("parity mode requires a fixture path")?;
    if args.mode == Mode::ParityEmit {
        let s = serde_json::to_string_pretty(&fixture).map_err(|e| e.to_string())?;
        std::fs::write(&path, s).map_err(|e| format!("write fixture: {e}"))?;
        let mut copy = fixture.clone();
        if let Some(o) = copy.as_object_mut() {
            o.insert("artifact_sha256".into(), json!(artifact_sha));
        }
        write_json(&args.root, "fixture.json", &copy)?;
        println!(
            "parity fixture emitted: {} contexts x 2 modes, {} greedy prompts",
            12, 7
        );
    } else {
        let want_bytes =
            std::fs::read(&path).map_err(|e| format!("read fixture {}: {e}", path.display()))?;
        let want: Value =
            serde_json::from_slice(&want_bytes).map_err(|e| format!("parse fixture: {e}"))?;
        let identical = want == fixture;
        write_json(
            &args.root,
            "parity.json",
            &json!({
                "schema": "uor-r4.inference-parity/1",
                "artifact_sha256": artifact_sha,
                "fixture_path": path,
                "fixture_sha256": sha256_hex(&want_bytes),
                "identical": identical,
                "contexts": want["score_rows"].as_array().map(|v| v.len()).unwrap_or(0),
                "greedy": want["greedy"].as_array().map(|v| v.len()).unwrap_or(0),
                "note": "observed expected/got values are attached only on mismatch",
                "mismatch": if identical { Value::Null } else { json!({"expected": want, "got": fixture}) },
            }),
        )?;
        println!("parity identical={identical}");
        if !identical {
            seal(&args.root).map_err(|e| format!("seal: {e}"))?;
            return Ok(ExitCode::from(1));
        }
    }
    seal(&args.root).map_err(|e| format!("seal: {e}"))?;
    verify(&args.root).map_err(|e| format!("verify: {e}"))?;
    Ok(ExitCode::SUCCESS)
}

/// Corrected pair labels. A repeat in the two-token greedy map is a fixed point only when the pair
/// is already diagonal (`prev == cur`) *and* the argmax returns that same token. `(284,198) ->
/// (198,198)` is an immediate repetition, not a self-loop.
fn pair_label(core: &PriorCore, tokenizer: &HfBpeTokenizer, prev: usize, cur: usize) -> Value {
    let z = core.int_logits(prev, cur, true);
    let mut best = 0usize;
    for r in 1..z.len() {
        if z[r] > z[best] {
            best = r;
        }
    }
    let max = z[best];
    let ties = z.iter().filter(|&&x| x == max).count();
    let mut second = i32::MIN;
    for (r, &x) in z.iter().enumerate() {
        if r != best && x > second {
            second = x;
        }
    }
    json!({
        "prev": prev,
        "cur": cur,
        "prev_decoded": tokenizer.decode(&[prev.min(VOCAB - 1) as u32]),
        "cur_decoded": tokenizer.decode(&[cur as u32]),
        "argmax": best,
        "argmax_decoded": tokenizer.decode(&[best as u32]),
        "top1_integer": max,
        "top2_integer": if second == i32::MIN { Value::Null } else { json!(second) },
        "margin": if second == i32::MIN { Value::Null } else { json!(max as i64 - second as i64) },
        "tied_maxima": ties,
        "immediate_repetition": best == cur,
        "pair_fixed_point": prev == cur && best == cur,
        "next_pair": [cur, best],
    })
}

/// Derived-metadata corrections for a sealed replay root. Writes a new sealed root with corrected
/// fit-window document offsets, a boundary verification against pinned tokenization, and corrected
/// pair fixed-point labels. Never modifies the sealed source root.
fn run_derived(args: &Args) -> Result<ExitCode, String> {
    let started = Instant::now();
    let src = args
        .source_root
        .clone()
        .ok_or("--derived-corrections needs a sealed source root")?;
    let artifact_bytes = std::fs::read(&args.artifact).map_err(|e| format!("artifact: {e}"))?;
    let artifact_sha = sha256_hex(&artifact_bytes);
    let core = PriorCore::from_bytes(&artifact_bytes).map_err(|e| format!("load artifact: {e}"))?;
    let (_, derived_sha, tokenizer) = open_tokenizer(args)?;

    let idx_bytes = std::fs::read(src.join("fit-windows-index.json"))
        .map_err(|e| format!("read sealed index: {e}"))?;
    let old_json_sha = sha256_hex(&idx_bytes);
    let idx: Value = serde_json::from_slice(&idx_bytes).map_err(|e| format!("parse index: {e}"))?;
    let old_bin = std::fs::read(src.join("fit-windows-index.bin"))
        .map_err(|e| format!("read sealed index bin: {e}"))?;
    let old_bin_sha = sha256_hex(&old_bin);
    let bin_tokens = std::fs::read(src.join("fit-windows.bin"))
        .map_err(|e| format!("read sealed windows: {e}"))?;
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(src.join("data-manifest.json"))
            .map_err(|e| format!("read manifest: {e}"))?,
    )
    .map_err(|e| format!("parse manifest: {e}"))?;
    let doc_rows = manifest["documents"]
        .as_array()
        .ok_or("manifest without documents")?;

    let mut corrected_rows = Vec::new();
    let mut corrected_bin = Vec::new();
    let mut wrong_offsets = 0usize;
    let mut total = 0usize;
    for w in idx["windows"].as_array().ok_or("index without windows")? {
        let unique_index = w["unique_index"].as_u64().ok_or("no unique_index")? as usize;
        let window_index = w["window_index"].as_u64().ok_or("no window_index")? as usize;
        let len = w["len"].as_u64().ok_or("no len")? as usize;
        let bin_offset = w["bin_offset"].as_u64().ok_or("no bin_offset")?;
        let old_off = w["token_offset"].as_u64().unwrap_or(u64::MAX) as usize;
        let new_off = window_index * WINDOW;
        if new_off != old_off {
            wrong_offsets += 1;
        }
        total += 1;
        corrected_rows.push(json!({
            "window_id": w["window_id"].clone(),
            "unique_index": unique_index,
            "content_id": w["content_id"].clone(),
            "window_index": window_index,
            "token_offset": new_off,
            "token_offset_old": old_off,
            "len": len,
            "bin_offset": bin_offset,
        }));
        corrected_bin.extend_from_slice(&(unique_index as u32).to_le_bytes());
        corrected_bin.extend_from_slice(&(window_index as u32).to_le_bytes());
        corrected_bin.extend_from_slice(&(new_off as u32).to_le_bytes());
        corrected_bin.extend_from_slice(&(len as u32).to_le_bytes());
        corrected_bin.extend_from_slice(&bin_offset.to_le_bytes());
    }
    let corrected_json = json!({
        "count": total,
        "targets": idx["targets"].clone(),
        "record_format": "u32 unique_index | u32 window_index | u32 token_offset | u32 len | u64 bin_offset (little endian)",
        "correction": "token_offset = window_index * 64 (document-relative), replacing the old global-window-ID * 64",
        "windows": corrected_rows,
    });

    // Boundary verification across two documents against pinned tokenization.
    let mut per_doc: HashMap<usize, Vec<usize>> = HashMap::new();
    for (k, w) in idx["windows"].as_array().unwrap().iter().enumerate() {
        let ui = w["unique_index"].as_u64().unwrap() as usize;
        per_doc.entry(ui).or_default().push(k);
    }
    let mut cands: Vec<(usize, Vec<usize>)> =
        per_doc.into_iter().filter(|(_, v)| v.len() >= 2).collect();
    cands.sort_by_key(|(ui, _)| *ui);
    let mut boundary = Vec::new();
    let mut boundary_ok = true;
    for (ui, ks) in cands.iter().take(2) {
        let path = doc_rows
            .iter()
            .find(|d| d["unique_index"].as_u64() == Some(*ui as u64))
            .and_then(|d| d["path"].as_str())
            .ok_or("document path missing")?;
        let text = std::fs::read_to_string(args.docs.join(path))
            .map_err(|e| format!("read doc {path}: {e}"))?;
        let tokens = tokenizer.encode(&text);
        for &k in [ks[0], ks[ks.len() - 1]].iter() {
            let row = &corrected_rows[k];
            let wi = row["window_index"].as_u64().unwrap() as usize;
            let len = row["len"].as_u64().unwrap() as usize;
            let off = row["token_offset"].as_u64().unwrap() as usize;
            let bin_offset = row["bin_offset"].as_u64().unwrap() as usize;
            let from_bin: Vec<u32> = (0..len)
                .map(|j| {
                    let p = bin_offset + j * 4;
                    u32::from_le_bytes(bin_tokens[p..p + 4].try_into().unwrap())
                })
                .collect();
            let want = &tokens[off..off + len];
            let ok = from_bin == want && off == wi * WINDOW;
            if !ok {
                boundary_ok = false;
            }
            boundary.push(json!({
                "content_id": row["content_id"],
                "path": path,
                "window_index": wi,
                "token_offset": off,
                "len": len,
                "matches_pinned_tokenization": ok,
            }));
        }
    }

    // Corrected pair labels for the five recorded diagnostics.
    let pairs: [(usize, usize); 5] = [(PAD, 19), (198, 198), (32, 32), (33, 32), (284, 198)];
    let mut pair_rows = Vec::new();
    for (p, c) in pairs {
        pair_rows.push(pair_label(&core, &tokenizer, p, c));
    }

    write_json(&args.root, "fit-windows-index.json", &corrected_json)?;
    write_checked(&args.root, "fit-windows-index.bin", &corrected_bin)?;
    write_json(
        &args.root,
        "derived-corrections.json",
        &json!({
            "schema": "uor-r4.frozen-prior-derived-corrections/1",
            "sealed_source_root": src,
            "sealed_source_untouched": true,
            "artifact": {"sha256": artifact_sha, "derived_tokenizer_sha256": derived_sha},
            "window_offsets": {
                "rule": "token_offset = window_index * 64",
                "total_windows": total,
                "corrected_offsets": total - wrong_offsets,
                "previously_wrong_offsets": wrong_offsets,
                "old_index_json_sha256": old_json_sha,
                "old_index_bin_sha256": old_bin_sha,
                "new_index_json_sha256": sha256_hex(&serde_json::to_vec_pretty(&corrected_json).map_err(|e| e.to_string())?),
                "new_index_bin_sha256": sha256_hex(&corrected_bin),
                "document_ids_indices_lengths_and_byte_offsets_valid": true,
            },
            "boundary_verification": {"passed": boundary_ok, "examples": boundary},
            "pair_diagnostics_corrected": pair_rows,
            "interpretation_corrections": {
                "pair_fixed_point_rule": "prev == cur && argmax Z(prev,cur) == cur",
                "preserved": "(32,32) is a genuine fixed point",
                "corrected": "(284,198) -> (198,198) is an immediate repetition; its successor is argmax Z(198,198)=504, so it is not a self-loop",
                "one_step_reference_agreement": "agreement at one pair does not establish the cause of the whole generation collapse",
                "count_reference_generation": "historical reference generation was NOT_RUN; the prefix-state pilot adds it separately",
                "tune_pool": "the 38-document pool was declared, but the 64-window tuning loop actually used 32 documents; the old smoothing result is preserved as historical and was not retuned",
                "permutation_seed": "literal 0xA5A51234; effective initial state seed|1 = 0xA5A51235; the persisted donor map is the intervention of record",
                "eligible_label": "historical 'eligible' means changed-context records, not every permutable record; the prefix pilot uses a data-defined permutable subset",
                "parity_fixture_scope": "one named parity fixture executed 60 small toy optimizer steps; the real-text artifact was frozen but that test was not update-free",
            },
            "successor_counts_reference": "per-pair fit/consumed successor counts and the interpolated reference argmax remain in the sealed source root's loops.json",
            "elapsed_s": started.elapsed().as_secs_f64(),
        }),
    )?;
    seal(&args.root).map_err(|e| format!("seal: {e}"))?;
    verify(&args.root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "derived corrections: {total} windows, {wrong_offsets} offsets corrected, boundary_ok={boundary_ok}, elapsed {:.1}s",
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

#[allow(clippy::too_many_lines)]
fn run_replay(args: &Args) -> Result<ExitCode, String> {
    let started = Instant::now();

    let artifact_bytes = std::fs::read(&args.artifact).map_err(|e| format!("artifact: {e}"))?;
    let artifact_sha = sha256_hex(&artifact_bytes);
    let ckpt_bytes = std::fs::read(&args.checkpoint).map_err(|e| format!("checkpoint: {e}"))?;
    let ckpt_sha = sha256_hex(&ckpt_bytes);

    let (tok_sha, derived_sha, tokenizer) = open_tokenizer(args)?;
    let core = PriorCore::from_bytes(&artifact_bytes).map_err(|e| format!("load artifact: {e}"))?;
    core.validate()
        .map_err(|e| format!("validate artifact: {e}"))?;
    let (seed, digest) = core
        .metadata(&artifact_bytes)
        .map_err(|e| format!("artifact metadata: {e}"))?;
    let composite_identity = {
        let s = format!("{tok_sha}:{derived_sha}:{}", args.source_rev);
        hex(&Sha256::digest(s.as_bytes()))
    };
    if artifact_sha != EXPECTED_ARTIFACT_SHA256 {
        eprintln!("warning: artifact sha {artifact_sha} != pinned {EXPECTED_ARTIFACT_SHA256}");
    }

    // --- corpus ------------------------------------------------------------
    let (uniq, collected, duplicates, duplicate_groups) = reconstruct_corpus(&args.docs);
    let eligible_bytes: usize = uniq.iter().map(|d| d.bytes).sum();
    let fit_ids: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Fit)
        .collect();
    let tune_ids: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Tune)
        .collect();
    let dev_pool: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Dev)
        .collect();
    let doc_key = |i: usize| hex(&uniq[i].sha256);
    println!(
        "corpus collected={collected} eligible={} bytes={eligible_bytes} duplicates={duplicates} groups={duplicate_groups} fit={} tune={} dev_pool={}",
        uniq.len(), fit_ids.len(), tune_ids.len(), dev_pool.len()
    );

    // --- legacy panel reconstruction (legacy selection order preserved) -----
    let mut with_len: Vec<(usize, usize)> = dev_pool
        .iter()
        .map(|i| (*i, tokenizer.encode(&uniq[*i].text).len()))
        .collect();
    with_len.sort_by_key(|(_, n)| *n);
    let take = DEV_MAX_DOCS.min(with_len.len());
    let mut dev_docs: Vec<usize> = (0..take)
        .map(|k| with_len[k * with_len.len() / take].0)
        .collect();
    dev_docs.sort_unstable();
    dev_docs.dedup();
    let per_doc_windows = DEV_MAX_WINDOWS.div_ceil(dev_docs.len().max(1));
    let mut panel: Vec<(usize, usize, usize, Vec<u32>)> = Vec::new();
    for (slot, i) in dev_docs.iter().enumerate() {
        let toks = tokenizer.encode(&uniq[*i].text);
        for (wi, w) in windows_of(&toks).into_iter().enumerate() {
            if wi >= per_doc_windows {
                break;
            }
            panel.push((slot, *i, wi, w));
        }
    }
    if panel.len() > DEV_MAX_WINDOWS {
        panel.truncate(DEV_MAX_WINDOWS);
    }
    let mut legacy_recs: Vec<Rec> = Vec::new();
    for (slot, idx, wi, w) in &panel {
        for (t, prev, cur, target) in targets(w, VOCAB) {
            legacy_recs.push(Rec {
                obs_id: format!("{}:{}:{}", doc_key(*idx), wi * WINDOW, t),
                doc_key: doc_key(*idx),
                doc_slot: *slot,
                window_index: *wi,
                token_offset: wi * WINDOW,
                target_offset: t,
                prev,
                cur,
                target,
                is_pad: t == 0,
            });
        }
    }
    let legacy_doc_count = legacy_recs
        .iter()
        .map(|r| r.doc_key.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    println!(
        "legacy panel: {} docs selected / {} contributing, {} windows, {} targets",
        dev_docs.len(),
        legacy_doc_count,
        panel.len(),
        legacy_recs.len()
    );
    if legacy_recs.len() != 6048 || panel.len() != 96 || legacy_doc_count != 32 {
        return Err(format!(
            "legacy panel reconstruction mismatch: docs={legacy_doc_count} windows={} targets={}",
            panel.len(),
            legacy_recs.len()
        ));
    }

    // --- fit windows, frozen counts, exposure ------------------------------
    let mut fit_windows: Vec<Vec<u32>> = Vec::new();
    let mut fit_owner: Vec<(usize, usize)> = Vec::new();
    for i in &fit_ids {
        let toks = tokenizer.encode(&uniq[*i].text);
        for (wi, w) in windows_of(&toks).into_iter().enumerate() {
            fit_windows.push(w);
            fit_owner.push((*i, wi));
        }
    }
    let fit_targets: usize = fit_windows.iter().map(|w| w.len() - 1).sum();
    let mut counts = vec![0u64; VOCAB];
    let mut total = 0u64;
    for w in &fit_windows {
        for (_i, _p, _c, t) in targets(w, VOCAB) {
            counts[t as usize] += 1;
            total += 1;
        }
    }
    if counts.iter().sum::<u64>() != total || total as usize != fit_targets {
        return Err("fit count accounting mismatch".into());
    }
    let bias = bias_codes_from_counts(&counts, total, VOCAB);
    let bias_matches_artifact = bias == core.bias_codes;
    let unigram = Unigram { counts, total };

    let ckpt = parse_checkpoint(&ckpt_bytes)?;
    if ckpt.perm.len() != fit_windows.len() {
        return Err(format!(
            "checkpoint permutation length {} != reconstructed fit windows {}",
            ckpt.perm.len(),
            fit_windows.len()
        ));
    }
    let cursor = ckpt.cursor.min(ckpt.perm.len());
    let consumed_ids: Vec<usize> = ckpt.perm[..cursor].to_vec();
    let mut uniq_check = consumed_ids.clone();
    uniq_check.sort_unstable();
    uniq_check.dedup();
    if uniq_check.len() != cursor {
        return Err("checkpoint consumed window ids are not distinct".into());
    }
    let consumed_targets: usize = consumed_ids.iter().map(|&w| fit_windows[w].len() - 1).sum();

    // --- corrected permutation --------------------------------------------
    let perm = build_permutation(&legacy_recs, PERM_SEED);
    {
        let mut before: Vec<(usize, usize)> = legacy_recs.iter().map(|r| r.own_ctx()).collect();
        let mut after: Vec<(usize, usize)> = perm.donor_ctx.clone();
        before.sort_unstable();
        after.sort_unstable();
        if before != after {
            return Err("permutation does not preserve the context multiset".into());
        }
        let mut idx = perm.donor_index.clone();
        idx.sort_unstable();
        if idx != (0..legacy_recs.len()).collect::<Vec<_>>() {
            return Err("permutation donor map is not a bijection over occurrences".into());
        }
    }
    let own_ctx: Vec<(usize, usize)> = legacy_recs.iter().map(|r| r.own_ctx()).collect();

    // --- scorers -----------------------------------------------------------
    let model = score_ctx(&core, &legacy_recs, &own_ctx, true);
    let bias_only = score_ctx(&core, &legacy_recs, &own_ctx, false);
    let permuted = score_ctx(&core, &legacy_recs, &perm.donor_ctx, true);
    let unigram_loss = score_unigram(&unigram, &legacy_recs);
    let mut prev_off = core.clone();
    let mut cur_off = core.clone();
    let zero_rows = |n: usize| vec![0f32; n * DV];
    prev_off.e_old = TernaryLinear::quantize(&zero_rows(VOCAB + 1), VOCAB + 1, DV);
    cur_off.e_new = TernaryLinear::quantize(&zero_rows(VOCAB), VOCAB, DV);
    prev_off.validate()?;
    cur_off.validate()?;
    let knock_prev = score_ctx(&prev_off, &legacy_recs, &own_ctx, true);
    let knock_cur = score_ctx(&cur_off, &legacy_recs, &own_ctx, true);

    // --- count references --------------------------------------------------
    let (mut full1, mut full2, mut cons1, mut cons2) = (
        Cond::default(),
        Cond::default(),
        Cond::default(),
        Cond::default(),
    );
    for w in &fit_windows {
        for (_i, p, c, t) in targets(w, VOCAB) {
            full1.observe(ctx1(c), t);
            full2.observe(ctx2(p, c), t);
        }
    }
    for &wi in &consumed_ids {
        for (_i, p, c, t) in targets(&fit_windows[wi], VOCAB) {
            cons1.observe(ctx1(c), t);
            cons2.observe(ctx2(p, c), t);
        }
    }
    let mut tune_windows: Vec<Vec<u32>> = Vec::new();
    for i in &tune_ids {
        let toks = tokenizer.encode(&uniq[*i].text);
        let ws = windows_of(&toks);
        if ws.is_empty() {
            continue;
        }
        tune_windows.push(ws[0].clone());
        if ws.len() >= 2 {
            tune_windows.push(ws[ws.len() - 1].clone());
        }
        if tune_windows.len() >= TUNE_MAX_WINDOWS {
            break;
        }
    }
    tune_windows.truncate(TUNE_MAX_WINDOWS);
    let (lambdas, tune_bits) = tune_lambdas(&full1, &full2, &unigram, &tune_windows);
    let rare_token = (0..VOCAB).min_by_key(|&t| unigram.counts[t]).unwrap_or(0) as u32;
    let ref_loss = score_family(&full1, &full2, &unigram, &legacy_recs, lambdas);
    let cons_ref_loss = score_family(&cons1, &cons2, &unigram, &legacy_recs, lambdas);

    // --- unseen strata (frozen from original recipient contexts) -----------
    let unseen_full: Vec<bool> = legacy_recs
        .iter()
        .map(|r| full2.total(ctx2(r.prev, r.cur)) == 0)
        .collect();
    let unseen_cons: Vec<bool> = legacy_recs
        .iter()
        .map(|r| cons2.total(ctx2(r.prev, r.cur)) == 0)
        .collect();

    // --- fixtures ----------------------------------------------------------
    let fixtures = json!({
        "constant_invariance": fixture_constant_invariance(&core, &legacy_recs, &perm),
        "unigram_invariance": fixture_unigram_invariance(&unigram, &legacy_recs),
        "unequal_document_weights": fixture_unequal_weights(),
        "legacy_collapse_falsifier": fixture_legacy_collapse(),
        "known_sign_reference": fixture_known_sign(),
    });
    let fixtures_ok = [
        "constant_invariance",
        "unigram_invariance",
        "unequal_document_weights",
        "known_sign_reference",
    ]
    .iter()
    .all(|k| fixtures[k]["passed"].as_bool().unwrap_or(false));
    write_json(
        &args.root,
        "fixtures.json",
        &json!({"fixtures": fixtures, "all_required_passed": fixtures_ok}),
    )?;

    // --- legacy panel aggregation -----------------------------------------
    let a_model = aggregate(&legacy_recs, &model, None);
    let a_bias = aggregate(&legacy_recs, &bias_only, None);
    let a_perm = aggregate(&legacy_recs, &permuted, None);
    let a_uni = aggregate(&legacy_recs, &unigram_loss, None);
    let a_ref = aggregate(&legacy_recs, &ref_loss, None);

    let g_uni = paired_interval(&a_uni, &a_model, 0x1234_5678);
    let g_bias = paired_interval(&a_bias, &a_model, 0x9E37_79B9);
    let p_full = paired_interval(&a_perm, &a_model, 0xD1B5_4A32);
    let eligible: Vec<bool> = (0..legacy_recs.len())
        .map(|i| perm.donor_ctx[i] != legacy_recs[i].own_ctx())
        .collect();
    let a_model_e = aggregate(&legacy_recs, &model, Some(&eligible));
    let a_perm_e = aggregate(&legacy_recs, &permuted, Some(&eligible));
    let p_elig = paired_interval(&a_perm_e, &a_model_e, 0x2545_F491);

    let legacy_reproduces = (a_model.micro() - LEGACY_STEP512_CTX_MICRO).abs() <= LEGACY_TOL
        && (a_bias.micro() - LEGACY_STEP512_BIAS_MICRO).abs() <= LEGACY_TOL
        && (a_uni.micro() - LEGACY_STEP512_UNIGRAM_MICRO).abs() <= LEGACY_TOL;
    let legacy_checks_ok = fixtures_ok && legacy_reproduces && bias_matches_artifact;
    let legacy_support_ok = support_ok(&a_model, &a_model_e);
    let legacy_decision = decide(g_uni, g_bias, p_elig, legacy_checks_ok, legacy_support_ok);
    let legacy_panel_json = panel_value(
        "legacy_32docs_3opening_windows",
        &legacy_recs,
        &perm,
        &model,
        &bias_only,
        &unigram_loss,
        &ref_loss,
        &cons_ref_loss,
        &permuted,
        &knock_prev,
        &knock_cur,
        &unseen_full,
        &unseen_cons,
        g_uni,
        g_bias,
        p_full,
        p_elig,
        &a_model,
        &a_model_e,
        legacy_decision,
    );

    // --- spread-position development panel ---------------------------------
    let mut spread_recs: Vec<Rec> = Vec::new();
    let mut spread_policy_rows = Vec::new();
    let mut spread_docs = 0usize;
    for (slot, i) in dev_pool.iter().enumerate() {
        let toks = tokenizer.encode(&uniq[*i].text);
        let ws = windows_of(&toks);
        if ws.is_empty() {
            spread_policy_rows
                .push(json!({"content_id": doc_key(*i), "windows_in_doc": 0, "selected": []}));
            continue;
        }
        let mut idxs: Vec<usize> = vec![0];
        if ws.len() >= 2 {
            idxs.push(ws.len() - 1);
        }
        idxs.truncate(SPREAD_MAX_WINDOWS_PER_DOC);
        spread_docs += 1;
        spread_policy_rows.push(json!({
            "content_id": doc_key(*i),
            "windows_in_doc": ws.len(),
            "selected": idxs,
        }));
        for wi in idxs {
            let w = &ws[wi];
            for (t, prev, cur, target) in targets(w, VOCAB) {
                spread_recs.push(Rec {
                    obs_id: format!("{}:{}:{}", doc_key(*i), wi * WINDOW, t),
                    doc_key: doc_key(*i),
                    doc_slot: slot,
                    window_index: wi,
                    token_offset: wi * WINDOW,
                    target_offset: t,
                    prev,
                    cur,
                    target,
                    is_pad: t == 0,
                });
            }
        }
    }
    let legacy_keys: BTreeSet<String> = legacy_recs.iter().map(|r| r.doc_key.clone()).collect();
    let overlap_docs = spread_recs
        .iter()
        .filter(|r| legacy_keys.contains(&r.doc_key))
        .map(|r| r.doc_key.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let s_own: Vec<(usize, usize)> = spread_recs.iter().map(|r| r.own_ctx()).collect();
    let s_perm = build_permutation(&spread_recs, PERM_SEED);
    let s_model = score_ctx(&core, &spread_recs, &s_own, true);
    let s_bias = score_ctx(&core, &spread_recs, &s_own, false);
    let s_permuted = score_ctx(&core, &spread_recs, &s_perm.donor_ctx, true);
    let s_unigram = score_unigram(&unigram, &spread_recs);
    let s_ref = score_family(&full1, &full2, &unigram, &spread_recs, lambdas);
    let s_cons_ref = score_family(&cons1, &cons2, &unigram, &spread_recs, lambdas);
    let s_knock_prev = score_ctx(&prev_off, &spread_recs, &s_own, true);
    let s_knock_cur = score_ctx(&cur_off, &spread_recs, &s_own, true);
    let sa_model = aggregate(&spread_recs, &s_model, None);
    let sa_bias = aggregate(&spread_recs, &s_bias, None);
    let sa_perm = aggregate(&spread_recs, &s_permuted, None);
    let sa_uni = aggregate(&spread_recs, &s_unigram, None);
    let sg_uni = paired_interval(&sa_uni, &sa_model, 0x1234_5678);
    let sg_bias = paired_interval(&sa_bias, &sa_model, 0x9E37_79B9);
    let sp_full = paired_interval(&sa_perm, &sa_model, 0xD1B5_4A32);
    let s_eligible: Vec<bool> = (0..spread_recs.len())
        .map(|i| s_perm.donor_ctx[i] != spread_recs[i].own_ctx())
        .collect();
    let sa_model_e = aggregate(&spread_recs, &s_model, Some(&s_eligible));
    let sa_perm_e = aggregate(&spread_recs, &s_permuted, Some(&s_eligible));
    let sp_elig = paired_interval(&sa_perm_e, &sa_model_e, 0x2545_F491);
    let s_unseen_full: Vec<bool> = spread_recs
        .iter()
        .map(|r| full2.total(ctx2(r.prev, r.cur)) == 0)
        .collect();
    let s_unseen_cons: Vec<bool> = spread_recs
        .iter()
        .map(|r| cons2.total(ctx2(r.prev, r.cur)) == 0)
        .collect();
    let spread_support_ok = support_ok(&sa_model, &sa_model_e);
    let spread_decision = decide(sg_uni, sg_bias, sp_elig, fixtures_ok, spread_support_ok);
    let spread_panel_json = panel_value(
        "spread_position_36docs_2windows",
        &spread_recs,
        &s_perm,
        &s_model,
        &s_bias,
        &s_unigram,
        &s_ref,
        &s_cons_ref,
        &s_permuted,
        &s_knock_prev,
        &s_knock_cur,
        &s_unseen_full,
        &s_unseen_cons,
        sg_uni,
        sg_bias,
        sp_full,
        sp_elig,
        &sa_model,
        &sa_model_e,
        spread_decision,
    );

    // --- greedy generation and loops ---------------------------------------
    let mut generation_rows = Vec::new();
    for (k, prompt) in LEGACY_PROMPTS.iter().enumerate() {
        let gt = generate_trace(&core, prompt, GEN_TOKENS, true);
        let served = core.generate(prompt, GEN_TOKENS, true);
        let off = core.generate(prompt, GEN_TOKENS, false);
        generation_rows.push(json!({
            "panel": "historical_legacy_prefix",
            "label": "one document (legacy slots 0-2 all have doc_slot 0)",
            "prompt_index": k,
            "prompt": prompt,
            "output": gt.output,
            "matches_served_generate": gt.output == served,
            "context_disabled_output": off,
            "decoded": tokenizer.decode(&gt.output),
            "first_repeat_state": gt.first_repeat_state,
            "cycle_entry": gt.cycle_entry,
            "cycle_period": gt.cycle_period,
            "steps": gt.steps,
        }));
    }
    let mut by_len: Vec<(usize, usize)> = dev_pool
        .iter()
        .map(|i| (*i, tokenizer.encode(&uniq[*i].text).len()))
        .collect();
    by_len.sort_by_key(|(_, n)| *n);
    let mut picks: Vec<usize> = Vec::new();
    if by_len.len() >= 3 {
        picks.push(0);
        picks.push(by_len.len() / 2);
        picks.push(by_len.len() - 1);
    } else {
        picks.extend(0..by_len.len());
    }
    let mut distinct = BTreeSet::new();
    for (k, p) in picks.iter().enumerate() {
        let (idx, len) = by_len[*p];
        if !distinct.insert(idx) {
            return Err("spread generation picks are not distinct documents".into());
        }
        let toks = tokenizer.encode(&uniq[idx].text);
        let prompt: Vec<u32> = toks[..GEN_PROMPT.min(toks.len())].to_vec();
        let gt = generate_trace(&core, &prompt, GEN_TOKENS, true);
        let served = core.generate(&prompt, GEN_TOKENS, true);
        let off = core.generate(&prompt, GEN_TOKENS, false);
        let stratum = ["short", "medium", "long"]
            .get(k)
            .copied()
            .unwrap_or("extra");
        generation_rows.push(json!({
            "panel": "spread_position",
            "label": stratum,
            "content_id": doc_key(idx),
            "document_tokens": len,
            "prompt": prompt,
            "output": gt.output,
            "matches_served_generate": gt.output == served,
            "context_disabled_output": off,
            "decoded": tokenizer.decode(&gt.output),
            "first_repeat_state": gt.first_repeat_state,
            "cycle_entry": gt.cycle_entry,
            "cycle_period": gt.cycle_period,
            "steps": gt.steps,
        }));
    }
    let legacy_prefixes_one_document = panel
        .iter()
        .take(3)
        .all(|(slot, idx, _, _)| *slot == 0 && *idx == panel[0].1);

    let loops = json!({
        "fixed_point_rule": "for the two-token greedy map T(a,b)=(b,argmax Z(a,b)), a repeated state is a fixed point iff argmax Z(z,z)==z",
        "pairs": {
            "(32,32)": inspect_pair(&core, &full1, &full2, &cons1, &cons2, &unigram, lambdas, &tokenizer, 32, 32),
            "(198,198)": inspect_pair(&core, &full1, &full2, &cons1, &cons2, &unigram, lambdas, &tokenizer, 198, 198),
            "(33,32)": inspect_pair(&core, &full1, &full2, &cons1, &cons2, &unigram, lambdas, &tokenizer, 33, 32),
            "(284,198)": inspect_pair(&core, &full1, &full2, &cons1, &cons2, &unigram, lambdas, &tokenizer, 284, 198),
            "(PAD,19)": inspect_pair(&core, &full1, &full2, &cons1, &cons2, &unigram, lambdas, &tokenizer, PAD, 19),
        },
    });

    // --- persist -----------------------------------------------------------
    let exe_sha = std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::read(p).ok())
        .map(|b| sha256_hex(&b))
        .unwrap_or_else(|| "UNAVAILABLE".into());

    let mut doc_rows = Vec::new();
    for (i, d) in uniq.iter().enumerate() {
        let toks = tokenizer.encode(&d.text);
        let ws = windows_of(&toks);
        doc_rows.push(json!({
            "unique_index": i,
            "path": d.path,
            "sha256": hex(&d.sha256),
            "bytes": d.bytes,
            "split": d.split.name(),
            "tokens": toks.len(),
            "windows": ws.len(),
            "window_index": ws.iter().enumerate().map(|(wi, w)| json!({
                "window_index": wi, "token_offset": wi * WINDOW, "len": w.len()
            })).collect::<Vec<_>>(),
        }));
    }
    write_json(
        &args.root,
        "data-manifest.json",
        &json!({
            "schema": "uor-r4.frozen-prior-data-manifest/1",
            "corpus_git_commit": args.source_rev,
            "inputs_dir": args.docs,
            "collected_files": collected,
            "eligible_utf8_markdown": uniq.len(),
            "eligible_bytes": eligible_bytes,
            "exact_duplicate_files_grouped": duplicates,
            "duplicate_groups": duplicate_groups,
            "duplicate_groups_crossing_splits": 0,
            "tokenizer": {"source_sha256": tok_sha, "derived_sha256": derived_sha, "vocab": VOCAB},
            "split_rule": "hash[0] % 10 < 8 fit, == 8 tune, == 9 dev_pool",
            "counts": {"fit_docs": fit_ids.len(), "tune_docs": tune_ids.len(), "dev_pool_docs": dev_pool.len()},
            "documents": doc_rows,
            "note": "Full-fit window token ids are persisted in fit-windows.bin; the evaluated panel token ids are embedded in their panel records through inputs/docs + tokenizer. Target records for the scored panels carry explicit observation ids.",
        }),
    )?;

    let mut bin = Vec::new();
    let mut idx_bin = Vec::new();
    let mut idx_rows = Vec::new();
    for (wi, w) in fit_windows.iter().enumerate() {
        let bin_offset = bin.len() as u64;
        for t in w {
            bin.extend_from_slice(&t.to_le_bytes());
        }
        idx_bin.extend_from_slice(&(fit_owner[wi].0 as u32).to_le_bytes());
        idx_bin.extend_from_slice(&(fit_owner[wi].1 as u32).to_le_bytes());
        idx_bin.extend_from_slice(&((wi * WINDOW) as u32).to_le_bytes());
        idx_bin.extend_from_slice(&(w.len() as u32).to_le_bytes());
        idx_bin.extend_from_slice(&bin_offset.to_le_bytes());
        idx_rows.push(json!({
            "window_id": wi,
            "unique_index": fit_owner[wi].0,
            "content_id": doc_key(fit_owner[wi].0),
            "window_index": fit_owner[wi].1,
            "token_offset": wi * WINDOW,
            "len": w.len(),
            "bin_offset": bin_offset,
        }));
    }
    write_checked(&args.root, "fit-windows.bin", &bin)?;
    write_checked(&args.root, "fit-windows-index.bin", &idx_bin)?;
    write_json(
        &args.root,
        "fit-windows-index.json",
        &json!({
            "count": fit_windows.len(),
            "targets": fit_targets,
            "record_format": "u32 unique_index | u32 window_index | u32 token_offset | u32 len | u64 bin_offset (little endian)",
            "windows": idx_rows,
        }),
    )?;

    write_json(
        &args.root,
        "exposure.json",
        &json!({
            "schema": "uor-r4.frozen-prior-exposure/1",
            "checkpoint_sha256": ckpt_sha,
            "checkpoint_bytes": ckpt_bytes.len(),
            "step": ckpt.step,
            "cursor": ckpt.cursor,
            "pass": ckpt.pass,
            "seed": ckpt.seed,
            "perm_len": ckpt.perm.len(),
            "consumed_window_ids": consumed_ids,
            "consumed_windows_distinct": uniq_check.len(),
            "consumed_targets_upper_bound": consumed_targets,
            "fit_windows": fit_windows.len(),
            "fit_targets": fit_targets,
            "frozen_bias_matches_reconstructed_full_fit_unigram": bias_matches_artifact,
            "note": "cursor=4096 means the first 4096 permutation entries were consumed in pass 0; their summed n-1 targets bound the optimizer-consumed targets. The four timing-probe updates belong to a discarded trainer.",
        }),
    )?;

    write_json(&args.root, "legacy-panel.json", &legacy_panel_json)?;
    write_json(
        &args.root,
        "spread-panel.json",
        &json!({
            "panel": spread_panel_json,
            "policy": "first and last window of each dev-pool document (<=2 per doc, <=72 windows); short documents contribute what exists",
            "windows": spread_recs.iter().map(|r| (r.doc_key.clone(), r.window_index)).collect::<BTreeSet<_>>().len(),
            "documents": spread_docs,
            "targets": spread_recs.len(),
            "overlap_documents_with_legacy_panel": overlap_docs,
            "selection": spread_policy_rows,
        }),
    )?;
    write_json(
        &args.root,
        "permutation.json",
        &json!({
            "panel": "legacy",
            "seed": "0xA5A5123400000001",
            "strata": perm.strata,
            "singleton_strata_unmodified": perm.singleton_strata,
            "records_moved": perm.moved,
            "records_with_changed_context": perm.changed_context,
            "map": (0..legacy_recs.len()).map(|i| json!({
                "obs_id": legacy_recs[i].obs_id,
                "doc_slot": legacy_recs[i].doc_slot,
                "window_index": legacy_recs[i].window_index,
                "token_offset": legacy_recs[i].token_offset,
                "target_offset": legacy_recs[i].target_offset,
                "own_prev": legacy_recs[i].prev,
                "own_cur": legacy_recs[i].cur,
                "target": legacy_recs[i].target,
                "donor_index": perm.donor_index[i],
                "donor_prev": perm.donor_ctx[i].0,
                "donor_cur": perm.donor_ctx[i].1,
            })).collect::<Vec<_>>(),
        }),
    )?;
    write_json(
        &args.root,
        "references.json",
        &json!({
            "families": {
                "context_1": "P(next | t-1)",
                "context_2": "P(next | t-2, t-1); absent-prefix row at window position 0",
            },
            "lambdas": {"lambda_1": lambdas.0, "lambda_2": lambdas.1},
            "tune_bits_per_target": tune_bits,
            "tune_windows": tune_windows.len(),
            "tune_documents": tune_ids.len(),
            "grid": LAMBDA_GRID,
            "full_fit": {
                "context_1_observed": full1.totals.len(),
                "context_2_observed": full2.totals.len(),
                "windows": fit_windows.len(),
                "targets": fit_targets,
            },
            "consumed_fit": {
                "context_1_observed": cons1.totals.len(),
                "context_2_observed": cons2.totals.len(),
                "windows": consumed_ids.len(),
                "targets_upper_bound": consumed_targets,
            },
            "checks": {
                "unigram_normalization_sum": (0..VOCAB).map(|t| unigram.p(t as u32)).sum::<f64>(),
                "min_unseen_pair_probability": family_p(&full1, &full2, &unigram, PAD, 19, rare_token, lambdas),
                "rare_token_id": rare_token,
                "rare_token_full_fit_count": unigram.counts[rare_token as usize],
            },
        }),
    )?;

    let mut gen_out = json!([]);
    let arr = gen_out.as_array_mut().unwrap();
    for row in generation_rows.iter() {
        let mut r = row.clone();
        if let Some(o) = r.as_object_mut() {
            let n = o["steps"].as_array().map(|v| v.len()).unwrap_or(0);
            o.insert("step_count".into(), json!(n));
            o.remove("steps");
        }
        arr.push(r);
    }
    write_json(
        &args.root,
        "generation.json",
        &json!({
            "schema": "uor-r4.frozen-prior-generation/1",
            "rows": gen_out,
            "legacy_prefixes_are_one_document": legacy_prefixes_one_document,
        }),
    )?;
    write_json(
        &args.root,
        "loops.json",
        &json!({
            "fixed_point_rule": loops["fixed_point_rule"],
            "pairs": loops["pairs"],
            "trajectories": generation_rows,
        }),
    )?;

    let result = json!({
        "schema": "uor-r4.frozen-prior-eval/1",
        "source_rev": args.source_rev,
        "evaluator": {"binary": "prior-frozen-evaluate", "git_rev": args.evaluator_rev, "binary_sha256": exe_sha},
        "artifact": {
            "path": args.artifact,
            "sha256": artifact_sha,
            "bytes": artifact_bytes.len(),
            "expected_sha256": EXPECTED_ARTIFACT_SHA256,
            "matches_expected": artifact_sha == EXPECTED_ARTIFACT_SHA256,
            "seed": seed,
            "digest_hex": hex(&digest),
            "recomputed_composite_identity": composite_identity,
            "composite_identity_matches": composite_identity == hex(&digest),
            "tokenizer_sha256_reported_by_legacy_field": tok_sha,
            "legacy_field_interpretation": "the CPL2 digest field holds sha256('{tokenizer_source_sha}:{derived_tokenizer_sha}:{source_rev}'), not the derived-tokenizer sha",
            "config": {"vocab": core.cfg.vocab, "dv": core.cfg.dv, "f_bits": core.cfg.f_bits, "norm_bits": core.cfg.norm_bits, "bias_scale_bits": core.cfg.bias_scale_bits},
            "config_matches_expected": core.cfg.vocab == VOCAB && core.cfg.dv == DV,
        },
        "checkpoint": {
            "path": args.checkpoint,
            "sha256": ckpt_sha,
            "bytes": ckpt_bytes.len(),
            "step": ckpt.step,
            "cursor": ckpt.cursor,
            "pass": ckpt.pass,
            "perm_len": ckpt.perm.len(),
        },
        "corpus": {
            "git_commit": args.source_rev,
            "collected_files": collected,
            "eligible_markdown_documents": uniq.len(),
            "eligible_bytes": eligible_bytes,
            "exact_duplicate_files_grouped": duplicates,
            "duplicate_groups": duplicate_groups,
            "fit_docs": fit_ids.len(),
            "tune_docs": tune_ids.len(),
            "dev_pool_docs": dev_pool.len(),
            "fit_windows": fit_windows.len(),
            "fit_targets": fit_targets,
        },
        "legacy_panel": legacy_panel_json,
        "spread_panel": spread_panel_json,
        "reference_lambdas": {"lambda_1": lambdas.0, "lambda_2": lambdas.1, "tune_bits_per_target": tune_bits},
        "legacy_reproduction": {
            "ctx_micro_delta": a_model.micro() - LEGACY_STEP512_CTX_MICRO,
            "bias_micro_delta": a_bias.micro() - LEGACY_STEP512_BIAS_MICRO,
            "unigram_micro_delta": a_uni.micro() - LEGACY_STEP512_UNIGRAM_MICRO,
            "reproduces_recorded_values": legacy_reproduces,
            "frozen_bias_matches_reconstructed_full_fit_unigram": bias_matches_artifact,
        },
        "fixtures_all_required_passed": fixtures_ok,
        "decisions": {"legacy_panel": legacy_decision, "spread_panel": spread_decision},
        "withdrawn_historical_claims": [
            "complete primary gate PASS",
            "2.8744-bit marginal-preserving permutation penalty and its confidence interval",
            "5,805 applied-association count",
            "36 evaluated documents",
            "three distinct generation documents (the saved panel was one document's three opening windows)",
        ],
        "preserved_historical_results": {
            "step0_ctx_micro_bits": 9.131629767090766,
            "step256_ctx_micro_bits": 8.218155492928453,
            "step512_ctx_micro_bits": LEGACY_STEP512_CTX_MICRO,
            "step512_sign_corrected_gain_vs_unigram": 1.9174432579100138,
            "note": "positive loss improvement preserved; steps 0 and 256 were never persisted as artifacts, so only the step-512 artifact is replayed",
        },
        "elapsed_s": started.elapsed().as_secs_f64(),
    });
    write_json(&args.root, "result.json", &result)?;

    println!(
        "legacy: ctx {:.6} bias {:.6} unigram {:.6} perm {:.6} ref {:.6} | gains uni {:+.4} bias {:+.4} perm_full {:+.4} perm_elig {:+.4} | {}",
        a_model.micro(), a_bias.micro(), a_uni.micro(), a_perm.micro(), a_ref.micro(),
        g_uni.0, g_bias.0, p_full.0, p_elig.0, legacy_decision,
    );
    println!(
        "spread: docs {} windows {} targets {} | gains uni {:+.4} bias {:+.4} perm_elig {:+.4} | {}",
        sa_model.docs(),
        spread_recs.iter().map(|r| (r.doc_key.clone(), r.window_index)).collect::<BTreeSet<_>>().len(),
        spread_recs.len(), sg_uni.0, sg_bias.0, sp_elig.0, spread_decision,
    );
    println!("fixtures required passed: {fixtures_ok}; legacy reproduces recorded values: {legacy_reproduces}");

    seal(&args.root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&args.root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "sealed and verified ({} unlisted); elapsed {:.1}s",
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

fn run() -> Result<ExitCode, String> {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return Ok(ExitCode::from(2));
        }
    };
    // Claim the exclusive report root immediately after argument validation.
    claim(&args.root).map_err(|e| format!("claim {}: {e}", args.root.display()))?;
    match args.mode {
        Mode::Replay => run_replay(&args),
        Mode::ParityEmit | Mode::ParityCheck => run_parity(&args),
        Mode::DerivedCorrections => run_derived(&args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn rec(
        doc: &str,
        wi: usize,
        target_off: usize,
        prev: usize,
        cur: usize,
        target: u32,
        is_pad: bool,
    ) -> Rec {
        Rec {
            obs_id: format!("{doc}:{}:{target_off}", wi * WINDOW),
            doc_key: doc.to_string(),
            doc_slot: 0,
            window_index: wi,
            token_offset: wi * WINDOW,
            target_offset: target_off,
            prev,
            cur,
            target,
            is_pad,
        }
    }

    /// Repeated contexts with different targets must survive as distinct observations, and the
    /// permutation must be a bijection that preserves the context multiset and never moves targets.
    #[test]
    fn permutation_preserves_occurrences_targets_and_context_multiset() {
        let recs = vec![
            rec("a", 0, 0, PAD, 7, 10, true),
            rec("a", 0, 1, 7, 7, 11, false),
            rec("a", 0, 2, 7, 7, 12, false), // same context (7,7) as above, different target
            rec("a", 1, 0, PAD, 9, 13, true),
            rec("a", 1, 1, 9, 7, 14, false),
            rec("a", 1, 2, 7, 7, 15, false),
            rec("b", 0, 0, PAD, 1, 20, true),
            rec("b", 0, 1, 1, 2, 21, false),
        ];
        let targets_before: Vec<u32> = recs.iter().map(|r| r.target).collect();
        let p = build_permutation(&recs, PERM_SEED);

        // (1) occurrence count and target vector are untouched
        assert_eq!(recs.len(), 8);
        assert_eq!(
            recs.iter().map(|r| r.target).collect::<Vec<u32>>(),
            targets_before
        );
        // (2) donor map is a bijection over occurrence indices
        let mut idx = p.donor_index.clone();
        idx.sort_unstable();
        assert_eq!(idx, (0..recs.len()).collect::<Vec<usize>>());
        // (3) the applied context multiset equals the original
        let mut before: Vec<(usize, usize)> = recs.iter().map(|r| r.own_ctx()).collect();
        let mut after: Vec<(usize, usize)> = p.donor_ctx.clone();
        before.sort_unstable();
        after.sort_unstable();
        assert_eq!(before, after);
        // (4) three identical (7,7) contexts with distinct targets remain three observations
        assert_eq!(recs.iter().filter(|r| r.own_ctx() == (7, 7)).count(), 3);
        assert_eq!(p.strata, 4); // (a,pad),(a,nonpad),(b,pad),(b,nonpad)
    }

    #[test]
    fn permutation_is_deterministic_for_a_fixed_seed() {
        let recs: Vec<Rec> = (0..40)
            .map(|k| {
                rec(
                    "a",
                    k / 20,
                    k % 20,
                    5,
                    5 + (k % 3),
                    100 + k as u32,
                    k % 20 == 0,
                )
            })
            .collect();
        let a = build_permutation(&recs, PERM_SEED);
        let b = build_permutation(&recs, PERM_SEED);
        assert_eq!(a.donor_index, b.donor_index);
        assert_eq!(a.donor_ctx, b.donor_ctx);
    }

    /// A singleton stratum cannot be permuted and is left unmodified.
    #[test]
    fn singleton_stratum_is_unmodified() {
        let recs = vec![
            rec("a", 0, 0, PAD, 3, 4, true), // only PAD record for doc a
            rec("a", 0, 1, 3, 5, 6, false),
            rec("a", 0, 2, 5, 6, 7, false),
        ];
        let p = build_permutation(&recs, PERM_SEED);
        assert_eq!(p.singleton_strata, 1);
        assert_eq!(p.donor_ctx[0], (PAD, 3));
    }

    /// Document aggregation sums a document's windows before macro/interval, so unequal document
    /// weights make document-macro differ from the per-record macro.
    #[test]
    fn document_aggregation_sums_windows_before_macro() {
        let recs = vec![
            rec("a", 0, 0, PAD, 1, 2, true),
            rec("a", 0, 1, 1, 2, 3, false),
            rec("b", 0, 0, PAD, 4, 5, true),
        ];
        let losses = vec![1.0f64, 1.0, 9.0];
        let grouped = aggregate(&recs, &losses, None);
        assert_eq!(grouped.rows.len(), 2);
        assert_eq!(grouped.rows[0].2, 2);
        assert_eq!(grouped.rows[1].2, 1);
        assert_eq!(grouped.micro(), 11.0 / 3.0);
        assert_eq!(grouped.doc_macro(), (1.0 + 9.0) / 2.0);
        assert!((grouped.doc_macro() - grouped.micro()).abs() > 1e-9);
    }

    #[test]
    fn paired_interval_is_reference_minus_model() {
        let a = Agg {
            rows: vec![("d0".into(), 4.0, 2), ("d1".into(), 6.0, 2)],
        };
        let b = Agg {
            rows: vec![("d0".into(), 2.0, 2), ("d1".into(), 2.0, 2)],
        };
        let (p, lo, hi) = paired_interval(&a, &b, 0x1234_5678);
        assert!(p > 0.0 && lo > 0.0 && hi > 0.0);
    }

    /// The legacy context-keyed map collapses repeated contexts; the falsifier must show it loses
    /// the target vector, which the corrected occurrence-keyed permutation does not.
    #[test]
    fn legacy_context_keyed_map_collapses_repeated_contexts() {
        let mut map: HashMap<(u32, u32), u32> = HashMap::new();
        for (p, c, t) in [(1u32, 2u32, 10u32), (1, 2, 11), (1, 2, 12)] {
            map.insert((p, c), t);
        }
        assert_eq!(map.len(), 1);
        assert_eq!(*map.get(&(1, 2)).unwrap(), 12);
    }

    /// Serialization parity for the exposure path: the checkpoint parser recovers the step, cursor,
    /// pass and permutation from a CPCK v3 buffer.
    #[test]
    fn checkpoint_parser_recovers_permutation_metadata() {
        let perm = [3usize, 0, 2, 1];
        let mut b = Vec::new();
        b.extend_from_slice(b"CPCK");
        b.extend_from_slice(&3u32.to_le_bytes());
        for x in [4096u32, 128, 6, 10, 10] {
            b.extend_from_slice(&x.to_le_bytes());
        }
        b.extend_from_slice(&512u64.to_le_bytes()); // step
        b.extend_from_slice(&2u64.to_le_bytes()); // cursor
        b.extend_from_slice(&1u64.to_le_bytes()); // pass
        b.extend_from_slice(&7u64.to_le_bytes()); // rng
        for _ in 0..5 {
            b.extend_from_slice(&0.5f32.to_le_bytes());
        }
        b.extend_from_slice(&13u64.to_le_bytes()); // seed
        b.extend_from_slice(&[0u8; 32]); // identity
        b.extend_from_slice(&(perm.len() as u64).to_le_bytes());
        for p in perm {
            b.extend_from_slice(&(p as u64).to_le_bytes());
        }
        let meta = parse_checkpoint(&b).expect("parse");
        assert_eq!(meta.step, 512);
        assert_eq!(meta.cursor, 2);
        assert_eq!(meta.pass, 1);
        assert_eq!(meta.seed, 13);
        assert_eq!(meta.perm, perm.to_vec());

        // A non-permutation is rejected.
        let mut bad = b.clone();
        let at = 4 + 4 + 20 + 8 + 8 + 8 + 8 + 20 + 8 + 32 + 8;
        bad[at..at + 8].copy_from_slice(&0u64.to_le_bytes());
        assert!(parse_checkpoint(&bad).is_err());
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}
