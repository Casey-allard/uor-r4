//! One bounded three-arm pilot for the learned older-prefix group-state channel.
//!
//! Freezes the retained step-512 prior, fits three matched arms above it on the first 512 entries of
//! the recovered consumed-window permutation, evaluates them on a new 36-document spread-position
//! panel with conditional permutation / reverse-order / state-disabled controls, and generates raw
//! greedy continuations. Never retrains the parent and never rewrites a sealed root.
#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use uor_r4_core::native_geometric::learner::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};
use uor_r4_core::native_geometric::learner::prefix_state::{
    palette, Arm, ParentCache, PrefixCore, PrefixTrainer,
};
use uor_r4_core::native_geometric::learner::prior_learning::{
    bias_codes_from_counts, targets, PriorCore,
};
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::bpe_derive::{derive_tokenizer, derive_tokenizer_json};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const VOCAB: usize = 4096;
const WINDOW: usize = 64;
const DEV_MAX_WINDOWS_PER_DOC: usize = 8;
const TRAIN_WINDOW_COUNT: usize = 512;
const UPDATES: usize = 256;
const BATCH: usize = 8;
const WARMUP_UPDATES: usize = 64;
const PASSES: usize = 4;
const SEED: u64 = 13;
const BOOTSTRAP_DRAWS: usize = 2000;
const SCREEN_MARGIN_BITS: f64 = 0.10;
const SUPPORT_MIN_DOCS: usize = 20;
const SUPPORT_MIN_TARGETS: usize = 1024;
const GEN_TOKENS: usize = 64;
const GEN_PROMPT: usize = 16;

const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json";
const EXPECTED_TOKENIZER_SHA256: &str =
    "9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c";
const EXPECTED_ARTIFACT_SHA256: &str =
    "cd5a3aa1b804ccc3582a4a25090e25c862489e0bc4e5bb86c1d1f363a7338a00";
const LEGACY_PROMPTS: [[u32; GEN_PROMPT]; 6] = [
    [
        19, 1071, 2615, 3927, 930, 2095, 198, 198, 828, 67, 85, 437, 1548, 216, 39, 287,
    ],
    [
        93, 84, 643, 933, 1855, 1160, 984, 342, 471, 31, 375, 481, 99, 77, 24, 591,
    ],
    [
        3911, 58, 3700, 68, 79, 61, 3872, 30, 93, 84, 25, 808, 30, 669, 3140, 829,
    ],
    [
        32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    ],
    [
        198, 198, 198, 198, 198, 198, 198, 198, 198, 198, 198, 198, 198, 198, 198, 198,
    ],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn sha256_hex(b: &[u8]) -> String {
    hex(&Sha256::digest(b))
}
fn write_checked(root: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(root.join(name), bytes).map_err(|e| format!("write {name}: {e}"))
}
fn write_json(root: &Path, name: &str, v: &Value) -> Result<(), String> {
    let s = serde_json::to_string_pretty(v).map_err(|e| format!("encode {name}: {e}"))?;
    write_checked(root, name, s.as_bytes())
}
fn xorshift(st: &mut u64) -> u64 {
    *st ^= *st << 13;
    *st ^= *st >> 7;
    *st ^= *st << 17;
    *st
}

struct Args {
    root: PathBuf,
    artifact: PathBuf,
    checkpoint: PathBuf,
    docs: PathBuf,
    tokenizer: PathBuf,
    source_rev: String,
    evaluator_rev: String,
    probe: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut root = None;
    let mut artifact = None;
    let mut checkpoint = None;
    let mut docs = None;
    let mut tokenizer = PathBuf::from(DEFAULT_TOKENIZER);
    let mut source_rev = String::from("unknown");
    let mut evaluator_rev = String::from("unknown");
    let mut probe = false;
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
            "--probe" => probe = true,
            other => return Err(format!("unknown argument {other}")),
        }
        i += if k == "--probe" { 1 } else { 2 };
    }
    Ok(Args {
        root: root.ok_or("--root is required")?,
        artifact: artifact.ok_or("--artifact is required")?,
        checkpoint: checkpoint.ok_or("--checkpoint is required")?,
        docs: docs.ok_or("--docs is required")?,
        tokenizer,
        source_rev,
        evaluator_rev,
        probe,
    })
}

// ---------------------------------------------------------------------------
// Corpus
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Split {
    Fit,
    Tune,
    Dev,
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

fn reconstruct_corpus(dir: &Path) -> (Vec<DocRec>, usize, usize) {
    let mut raw: Vec<(String, [u8; 32], usize, String)> = Vec::new();
    collect_docs(dir, Path::new(""), &mut raw);
    let collected = raw.len();
    let mut seen: BTreeSet<[u8; 32]> = BTreeSet::new();
    let mut uniq = Vec::new();
    let mut duplicates = 0usize;
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
    (uniq, collected, duplicates)
}

fn windows_of(tokens: &[u32]) -> Vec<Vec<u32>> {
    tokens
        .chunks(WINDOW)
        .filter(|c| c.len() >= 3)
        .map(|c| c.to_vec())
        .collect()
}

/// The recovered CPCK v3 permutation, for exposure recovery only.
fn parse_checkpoint_perm(bytes: &[u8]) -> Result<Vec<usize>, String> {
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
    for _ in 0..5 {
        let _ = u32_at(&mut c)?;
    }
    let _step = u64_at(&mut c)?;
    let _cursor = u64_at(&mut c)?;
    let _pass = u64_at(&mut c)?;
    let _rng = u64_at(&mut c)?;
    for _ in 0..5 {
        let _ = u32_at(&mut c)?;
    }
    let _seed = u64_at(&mut c)?;
    let _ = take(&mut c, 32)?;
    let perm_len = u64_at(&mut c)? as usize;
    let mut perm = Vec::with_capacity(perm_len);
    let mut seen = vec![false; perm_len];
    for _ in 0..perm_len {
        let p = u64_at(&mut c)? as usize;
        if p >= perm_len || std::mem::replace(&mut seen[p], true) {
            return Err("checkpoint permutation is not a permutation".into());
        }
        perm.push(p);
    }
    Ok(perm)
}

// ---------------------------------------------------------------------------
// Records, aggregation, intervals
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Rec {
    obs: String,
    doc: String,
    win: usize,
    i: usize,
    prev: usize,
    cur: usize,
    target: u32,
    older_len: usize,
}

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
    fn macro_bits(&self) -> f64 {
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

fn aggregate(recs: &[Rec], losses: &[f64], keep: Option<&[bool]>) -> Agg {
    let mut index: HashMap<&str, usize> = HashMap::new();
    let mut rows: Vec<(String, f64, usize)> = Vec::new();
    for (i, r) in recs.iter().enumerate() {
        if let Some(k) = keep {
            if !k[i] {
                continue;
            }
        }
        let slot = match index.get(r.doc.as_str()) {
            Some(&s) => s,
            None => {
                let s = rows.len();
                index.insert(r.doc.as_str(), s);
                rows.push((r.doc.clone(), 0.0, 0));
                s
            }
        };
        rows[slot].1 += losses[i];
        rows[slot].2 += 1;
    }
    Agg { rows }
}

fn paired_interval(a: &Agg, b: &Agg, seed: u64) -> (f64, f64, f64) {
    assert_eq!(a.rows.len(), b.rows.len());
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
        let (mut sa, mut na, mut sb, mut nb) = (0.0, 0.0, 0.0, 0.0);
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

fn iv(t: (f64, f64, f64)) -> Value {
    json!({"point": t.0, "lo": t.1, "hi": t.2})
}

// ---------------------------------------------------------------------------
// Conditional permutation
// ---------------------------------------------------------------------------

struct Perm {
    donor: Vec<usize>,
    strata: usize,
    eligible: Vec<bool>,
    excluded_no_history: usize,
    changed_state_recipients: usize,
}

/// Seeded Fisher-Yates bijection per `(exact prev, exact cur, exact older-prefix length)` stratum,
/// excluding no-history records. Donors may cross documents; donor identities are frozen before any
/// fitting and are independent of learned states.
fn build_perm(recs: &[Rec], seed: u64) -> Perm {
    let mut groups: Vec<((usize, usize, usize), Vec<usize>)> = Vec::new();
    let mut index: HashMap<(usize, usize, usize), usize> = HashMap::new();
    for (i, r) in recs.iter().enumerate() {
        let key = (r.prev, r.cur, r.older_len);
        let slot = match index.get(&key) {
            Some(&s) => s,
            None => {
                let s = groups.len();
                index.insert(key, s);
                groups.push((key, Vec::new()));
                s
            }
        };
        groups[slot].1.push(i);
    }
    let mut donor: Vec<usize> = (0..recs.len()).collect();
    let mut eligible = vec![false; recs.len()];
    let mut st = seed | 1;
    let mut strata = 0usize;
    let mut excluded = 0usize;
    for ((_, _, len), members) in groups.iter() {
        strata += 1;
        if *len == 0 {
            excluded += members.len();
            continue;
        }
        if members.len() < 2 {
            continue;
        }
        let mut perm: Vec<usize> = (0..members.len()).collect();
        for i in (1..perm.len()).rev() {
            let j = (xorshift(&mut st) as usize) % (i + 1);
            perm.swap(i, j);
        }
        for (k, &rec_idx) in members.iter().enumerate() {
            donor[rec_idx] = members[perm[k]];
            eligible[rec_idx] = true;
        }
    }
    Perm {
        donor,
        strata,
        eligible,
        excluded_no_history: excluded,
        changed_state_recipients: 0,
    }
}

// ---------------------------------------------------------------------------
// Count references
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Cond {
    counts: HashMap<(u64, u32), u32>,
    totals: HashMap<u64, u64>,
}
impl Cond {
    fn observe(&mut self, ctx: u64, next: u32) {
        *self.counts.entry((ctx, next)).or_insert(0) += 1;
        *self.totals.entry(ctx).or_insert(0) += 1;
    }
    fn total(&self, ctx: u64) -> u64 {
        self.totals.get(&ctx).copied().unwrap_or(0)
    }
    fn count_of(&self, ctx: u64, next: u32) -> u32 {
        self.counts.get(&(ctx, next)).copied().unwrap_or(0)
    }
}

struct Uni {
    counts: Vec<u64>,
    total: u64,
}
impl Uni {
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

fn ctx2(prev: usize, cur: usize) -> u64 {
    (prev as u64) * VOCAB as u64 + cur as u64
}

fn family_p(
    c1: &Cond,
    c2: &Cond,
    uni: &Uni,
    prev: usize,
    cur: usize,
    next: u32,
    l: (f64, f64),
) -> f64 {
    let t1 = c1.total(cur as u64);
    let p1 = if t1 == 0 {
        uni.p(next)
    } else {
        c1.count_of(cur as u64, next) as f64 / t1 as f64
    };
    let t2 = c2.total(ctx2(prev, cur));
    let p2 = if t2 == 0 {
        uni.p(next)
    } else {
        c2.count_of(ctx2(prev, cur), next) as f64 / t2 as f64
    };
    l.1 * p2 + (1.0 - l.1) * (l.0 * p1 + (1.0 - l.0) * uni.p(next))
}

const LAMBDA_GRID: [f64; 10] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];

fn tune_lambdas(c1: &Cond, c2: &Cond, uni: &Uni, tune: &[Vec<u32>]) -> ((f64, f64), f64) {
    let mut best = (0.5, 0.5);
    let mut best_bits = f64::INFINITY;
    for &l1 in LAMBDA_GRID.iter() {
        for &l2 in LAMBDA_GRID.iter() {
            let mut bits = 0.0;
            let mut n = 0usize;
            for w in tune {
                for (_i, p, c, t) in targets(w, VOCAB) {
                    bits -= family_p(c1, c2, uni, p, c, t, (l1, l2)).max(1e-300).log2();
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
// Scoring
// ---------------------------------------------------------------------------

/// Per-arm residual table over the 120 states, so evaluation is a table lookup per target.
fn residual_table(core: &PrefixCore) -> Vec<Vec<i32>> {
    (0..GROUP_ORDER).map(|q| core.residual_scores(q)).collect()
}

fn score(
    parent: &PriorCore,
    residuals: Option<&[Vec<i32>]>,
    recs: &[Rec],
    states: &[Option<usize>],
    cache: &mut ParentCache,
) -> Vec<f64> {
    let v = parent.cfg.vocab;
    recs.iter()
        .zip(states.iter())
        .map(|(r, st)| {
            let zp = cache.z_for(parent, r.prev, r.cur);
            let loss = match (residuals, st) {
                (Some(table), Some(q)) => {
                    let res = &table[*q];
                    // bits via a stable log-sum-exp over parent+residual.
                    let scale = (-(parent.cfg.f_bits as f64)).exp2();
                    let t = (r.target as usize).min(v - 1);
                    let mut max = f64::NEG_INFINITY;
                    for i in 0..v {
                        let s = (zp[i] + res[i]) as f64 * scale;
                        if s > max {
                            max = s;
                        }
                    }
                    let mut sum = 0.0;
                    for i in 0..v {
                        sum += ((zp[i] + res[i]) as f64 * scale - max).exp();
                    }
                    let zt = (zp[t] + res[t]) as f64 * scale;
                    (max + sum.ln() - zt) / std::f64::consts::LN_2
                }
                _ => {
                    let z: Vec<i32> = zp.to_vec();
                    parent.bits_one(&z, r.target)
                }
            };
            loss
        })
        .collect()
}

fn state_own(core: &PrefixCore, tail: bool, tokens: &[u32], i: usize) -> Option<usize> {
    if tail {
        core.state_tail(tokens, i)
    } else {
        core.state_older(tokens, i)
    }
}

fn state_reversed(core: &PrefixCore, tokens: &[u32], i: usize) -> Option<usize> {
    if i < 2 {
        return None;
    }
    let start = i.saturating_sub(63);
    let end = i - 1;
    if end <= start {
        return None;
    }
    let t = group_table();
    let mut q = t.identity as usize;
    for &tok in tokens[start..end].iter().rev() {
        let g = core.palette.elements[core.action(tok)] as usize;
        q = t.product[q * ROW_STRIDE + g] as usize;
    }
    Some(q)
}

// ---------------------------------------------------------------------------
// Generation with a complete-state cycle detector
// ---------------------------------------------------------------------------

struct GenOut {
    output: Vec<u32>,
    decoded: String,
    pair_cycle: Option<(usize, usize)>,
    ring_cycle: Option<(usize, usize)>,
    first_repeat_output_index: Option<usize>,
}

/// Cycle certificates computed from the generated token stream.
///
/// `pair` uses the two-token state `(x[t-1], x[t])`, which is the *sufficient* state of the frozen
/// parent. `ring` uses the ordered last-`<=64` tokens, which is the sufficient state of the bounded
/// context model. A repeated pair is not a cycle certificate for the new model; a repeated ring is.
fn cycle_certificates(
    prompt: &[u32],
    output: &[u32],
) -> (Option<(usize, usize)>, Option<(usize, usize)>) {
    let mut seen_pair: HashMap<(u32, u32), usize> = HashMap::new();
    let mut seen_ring: HashMap<Vec<u32>, usize> = HashMap::new();
    let mut pair = None;
    let mut ring = None;
    for t in 0..output.len() {
        let len = prompt.len() + t;
        let toks = &prompt[..];
        let last = if t == 0 {
            toks[toks.len() - 1]
        } else {
            output[t - 1]
        };
        let here = output[t];
        if pair.is_none() {
            if let Some(&t0) = seen_pair.get(&(last, here)) {
                pair = Some((t0, t - t0));
            } else {
                seen_pair.insert((last, here), t);
            }
        }
        if ring.is_none() {
            let mut key: Vec<u32> = Vec::new();
            let start = len.saturating_sub(WINDOW);
            for j in start..toks.len() {
                key.push(toks[j]);
            }
            for j in 0..t {
                key.push(output[j]);
            }
            if key.len() > WINDOW {
                key.remove(0);
            }
            if let Some(&t0) = seen_ring.get(&key) {
                ring = Some((t0, t - t0));
            } else {
                seen_ring.insert(key, t);
            }
        }
    }
    (pair, ring)
}

/// Greedy generation. A cycle is reported only when the *complete* sufficient state repeats: for
/// the bounded-context model that is the ordered ring of the last `<=64` tokens including its
/// length, not merely a repeated pair.
fn generate_full_state(core: &PrefixCore, prompt: &[u32], n_new: usize) -> GenOut {
    let mut toks = prompt.to_vec();
    let v = core.parent.cfg.vocab;
    for _ in 0..n_new {
        let i = toks.len() - 1;
        let prev = if i == 0 {
            core.parent.cfg.pad_row()
        } else {
            (toks[i - 1] as usize).min(v - 1)
        };
        let cur = (toks[i] as usize).min(v - 1);
        let q = core.state_older(&toks, i);
        let z = core.int_logits(prev, cur, q);
        let mut best = 0usize;
        for r in 1..z.len() {
            if z[r] > z[best] {
                best = r;
            }
        }
        toks.push(best as u32);
    }
    let output = toks[prompt.len()..].to_vec();
    let (pair_cycle, ring_cycle) = cycle_certificates(prompt, &output);
    let mut first_pair = None;
    for k in 1..output.len() {
        if output[k] == output[k - 1] && first_pair.is_none() {
            first_pair = Some(k);
        }
    }
    GenOut {
        output,
        decoded: String::new(),
        pair_cycle,
        ring_cycle,
        first_repeat_output_index: first_pair,
    }
}

fn generate_parent(core: &PriorCore, prompt: &[u32], n_new: usize) -> GenOut {
    let out = core.generate(prompt, n_new, true);
    let (pair_cycle, ring_cycle) = cycle_certificates(prompt, &out);
    let mut first_pair = None;
    for k in 1..out.len() {
        if out[k] == out[k - 1] && first_pair.is_none() {
            first_pair = Some(k);
        }
    }
    GenOut {
        output: out,
        decoded: String::new(),
        pair_cycle,
        ring_cycle,
        first_repeat_output_index: first_pair,
    }
}

/// Greedy generation under an interpolated count reference (diagnostic only).
fn generate_reference(
    c1: &Cond,
    c2: &Cond,
    uni: &Uni,
    l: (f64, f64),
    prompt: &[u32],
    n_new: usize,
) -> Vec<u32> {
    let mut toks = prompt.to_vec();
    let v = VOCAB;
    for _ in 0..n_new {
        let i = toks.len() - 1;
        let prev = if i == 0 {
            v
        } else {
            (toks[i - 1] as usize).min(v - 1)
        };
        let cur = (toks[i] as usize).min(v - 1);
        let mut best = uni.argmax();
        let mut best_p = family_p(c1, c2, uni, prev, cur, best, l);
        for cand in [cur as u32, prev.min(v - 1) as u32, uni.argmax()] {
            let p = family_p(c1, c2, uni, prev, cur, cand, l);
            if p > best_p {
                best_p = p;
                best = cand;
            }
        }
        toks.push(best);
    }
    toks[prompt.len()..].to_vec()
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn run() -> Result<ExitCode, String> {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return Ok(ExitCode::from(2));
        }
    };
    claim(&args.root).map_err(|e| format!("claim {}: {e}", args.root.display()))?;
    let started = Instant::now();

    let artifact_bytes = std::fs::read(&args.artifact).map_err(|e| format!("artifact: {e}"))?;
    let artifact_sha = sha256_hex(&artifact_bytes);
    let ckpt_bytes = std::fs::read(&args.checkpoint).map_err(|e| format!("checkpoint: {e}"))?;
    let ckpt_sha = sha256_hex(&ckpt_bytes);
    if artifact_sha != EXPECTED_ARTIFACT_SHA256 {
        return Err(format!(
            "artifact sha {artifact_sha} != pinned {EXPECTED_ARTIFACT_SHA256}"
        ));
    }
    let parent = PriorCore::from_bytes(&artifact_bytes).map_err(|e| format!("load parent: {e}"))?;
    parent.validate()?;

    // Tokenizer.
    let tok_bytes = std::fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    let tok_sha = sha256_hex(&tok_bytes);
    if tok_sha != EXPECTED_TOKENIZER_SHA256 {
        return Err("tokenizer sha mismatch".into());
    }
    let tokenizer: HfBpeTokenizer =
        derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive tokenizer: {e}"))?;
    let derived_sha = sha256_hex(
        &derive_tokenizer_json(&tok_bytes, VOCAB).map_err(|e| format!("derive json: {e}"))?,
    );

    let pal = palette().clone();
    if pal.closure_size() != GROUP_ORDER {
        return Err("palette closure is not the full group".into());
    }

    // Corpus.
    let (uniq, collected, duplicates) = reconstruct_corpus(&args.docs);
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
        "corpus collected={collected} eligible={} fit={} tune={} dev={}",
        uniq.len(),
        fit_ids.len(),
        tune_ids.len(),
        dev_pool.len()
    );

    // Fit windows and the 512-window training dose.
    let mut fit_windows: Vec<Vec<u32>> = Vec::new();
    for i in &fit_ids {
        fit_windows.extend(windows_of(&tokenizer.encode(&uniq[*i].text)));
    }
    let fit_targets: usize = fit_windows.iter().map(|w| w.len() - 1).sum();
    let ckpt_perm = parse_checkpoint_perm(&ckpt_bytes)?;
    if ckpt_perm.len() != fit_windows.len() {
        return Err("checkpoint permutation length != fit windows".into());
    }
    let consumed: Vec<usize> = ckpt_perm[..4096.min(ckpt_perm.len())].to_vec();
    let train_ids: Vec<usize> = consumed[..TRAIN_WINDOW_COUNT.min(consumed.len())].to_vec();
    let train_windows: Vec<Vec<u32>> = train_ids.iter().map(|&w| fit_windows[w].clone()).collect();
    let train_targets: usize = train_windows.iter().map(|w| w.len() - 1).sum();
    println!(
        "fit windows={} targets={} train windows={} targets={} (first {TRAIN_WINDOW_COUNT} of the recovered {}-window consumed permutation)",
        fit_windows.len(),
        fit_targets,
        train_windows.len(),
        train_targets,
        consumed.len()
    );

    // Frozen fit-only unigram.
    let mut counts = vec![0u64; VOCAB];
    let mut total = 0u64;
    for w in &fit_windows {
        for (_i, _p, _c, t) in targets(w, VOCAB) {
            counts[t as usize] += 1;
            total += 1;
        }
    }
    if total as usize != fit_targets {
        return Err("fit count mismatch".into());
    }
    let uni = Uni {
        counts: counts.clone(),
        total,
    };
    let bias = bias_codes_from_counts(&counts, total, VOCAB);
    let bias_matches = bias == parent.bias_codes;

    // Primary panel: all 36 dev-pool documents, up to 8 evenly spaced windows each.
    let mut panel_windows: Vec<Vec<u32>> = Vec::new();
    let mut panel_window_docs: Vec<String> = Vec::new();
    let mut panel_docs: Vec<(String, usize, usize)> = Vec::new();
    for i in &dev_pool {
        let key = doc_key(*i);
        let toks = tokenizer.encode(&uniq[*i].text);
        let ws = windows_of(&toks);
        if ws.is_empty() {
            panel_docs.push((key, 0, 0));
            continue;
        }
        let count = DEV_MAX_WINDOWS_PER_DOC.min(ws.len());
        let mut idxs: Vec<usize> = Vec::new();
        for k in 0..count {
            let idx = if count == 1 {
                0
            } else {
                (k * (ws.len() - 1) + (count - 1) / 2) / (count - 1)
            };
            if !idxs.contains(&idx) {
                idxs.push(idx);
            }
        }
        panel_docs.push((key.clone(), ws.len(), idxs.len()));
        for wi in idxs {
            panel_windows.push(ws[wi].clone());
            panel_window_docs.push(key.clone());
        }
    }
    let mut recs: Vec<Rec> = Vec::new();
    for (wi, w) in panel_windows.iter().enumerate() {
        for (i, prev, cur, target) in targets(w, VOCAB) {
            recs.push(Rec {
                obs: format!("{}:{}:{}", panel_window_docs[wi], wi, i),
                doc: panel_window_docs[wi].clone(),
                win: wi,
                i,
                prev,
                cur,
                target,
                older_len: PrefixCore::older_len(i),
            });
        }
    }
    let panel_targets = recs.len();
    let panel_doc_count = recs
        .iter()
        .map(|r| r.doc.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    // Overlap against the legacy 32-document panel (the 32 lowest dev-pool indices).
    let legacy_keys: BTreeSet<String> = {
        let mut dd: Vec<usize> = dev_pool.clone();
        dd.sort_unstable();
        dd.truncate(32);
        dd.iter().map(|i| doc_key(*i)).collect()
    };
    let overlap = legacy_keys
        .intersection(&recs.iter().map(|r| r.doc.clone()).collect::<BTreeSet<_>>())
        .count();
    println!(
        "panel documents={} windows={} targets={} overlap_with_legacy_docs={overlap}",
        panel_doc_count,
        panel_windows.len(),
        panel_targets
    );

    let mut cache = ParentCache::default();
    for r in recs.iter() {
        let _ = cache.z_for(&parent, r.prev, r.cur);
    }
    println!(
        "parent cache entries={} bytes={}",
        cache.entries(),
        cache.bytes()
    );

    // Data identity for the new arm checkpoints.
    let data_identity: [u8; 32] = Sha256::digest(
        format!(
            "{artifact_sha}:{tok_sha}:{derived_sha}:{}:{}:{}",
            args.source_rev,
            train_ids
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("-"),
            palette_digest(&pal)
        )
        .as_bytes(),
    )
    .into();

    // Schedule: four saved Fisher-Yates passes over the training windows.
    let schedule = build_schedule(train_windows.len(), SEED);

    // --- probe -------------------------------------------------------------
    if args.probe {
        let t0 = Instant::now();
        let mut tr = PrefixTrainer::new(
            parent.clone(),
            pal.clone(),
            Arm::LearnedOlder,
            SEED,
            WINDOW,
            BATCH,
            UPDATES,
            WARMUP_UPDATES,
            parent_hash(&artifact_sha),
        )?;
        tr.data_identity = data_identity;
        let mut probe_cache = ParentCache::default();
        for p in 0..2 {
            let batch: Vec<Vec<u32>> = schedule[p][..BATCH]
                .iter()
                .map(|&j| train_windows[j].clone())
                .collect();
            let loss = tr.train_batch(&batch, &mut probe_cache, true)?;
            println!("probe pass {p} loss {loss:.4} bits/target");
        }
        let core = tr.hard_core()?;
        let _ = core.int_logits(1, 2, Some(0));
        println!(
            "probe: 16 updates in {:.2}s (probe optimizer state discarded and charged)",
            t0.elapsed().as_secs_f64()
        );
        seal(&args.root).map_err(|e| format!("seal: {e}"))?;
        verify(&args.root).map_err(|e| format!("verify: {e}"))?;
        return Ok(ExitCode::SUCCESS);
    }

    // --- three matched fits ------------------------------------------------
    let arms = [Arm::LearnedOlder, Arm::FixedOlder, Arm::LearnedTail];
    let mut fitted: HashMap<Arm, PrefixTrainer> = HashMap::new();
    let mut fit_rows = Vec::new();
    for (ai, arm) in arms.iter().enumerate() {
        let t0 = Instant::now();
        let mut tr = PrefixTrainer::new(
            parent.clone(),
            pal.clone(),
            *arm,
            SEED,
            WINDOW,
            BATCH,
            UPDATES,
            WARMUP_UPDATES,
            parent_hash(&artifact_sha),
        )?;
        tr.data_identity = data_identity;
        let initial_actions = tr.initial_actions();
        let mut pass_losses: Vec<Vec<f64>> = Vec::new();
        let mut checkpoints = Vec::new();
        let mut update = 0usize;
        let mut grad_reader = Vec::new();
        let mut grad_action = Vec::new();
        for pass in 0..PASSES {
            let mut pl = Vec::new();
            for b in 0..(train_windows.len() / BATCH) {
                if update >= UPDATES {
                    break;
                }
                let batch: Vec<Vec<u32>> = schedule[pass][b * BATCH..(b + 1) * BATCH]
                    .iter()
                    .map(|&j| train_windows[j].clone())
                    .collect();
                let apply = update >= WARMUP_UPDATES;
                let loss = tr.train_batch(&batch, &mut cache, apply)?;
                pl.push(loss);
                grad_reader.push(tr.last_grad_norm_reader);
                grad_action.push(tr.last_grad_norm_action);
                update += 1;
                if update == WARMUP_UPDATES || update == 128 || update == UPDATES {
                    let core = tr.hard_core()?;
                    let res = residual_table(&core);
                    let states: Vec<Option<usize>> = recs
                        .iter()
                        .map(|r| {
                            state_own(
                                &core,
                                arm.uses_older_prefix() == false,
                                &panel_windows[r.win],
                                r.i,
                            )
                        })
                        .collect();
                    let losses = score(&parent, Some(&res), &recs, &states, &mut cache);
                    let agg = aggregate(&recs, &losses, None);
                    checkpoints.push(json!({
                        "update": update,
                        "panel_micro_bits": agg.micro(),
                        "panel_document_macro_bits": agg.macro_bits(),
                        "last_batch_loss": loss,
                        "action_updates": tr.action_updates,
                    }));
                }
            }
            tr.pass = pass as u32 + 1;
            pass_losses.push(pl);
        }
        let elapsed = t0.elapsed().as_secs_f64();
        let final_core = tr.hard_core()?;
        let hard = tr.hard_actions();
        let transitions = hard
            .iter()
            .zip(initial_actions.iter())
            .filter(|(a, b)| a != b)
            .count();
        let res = residual_table(&final_core);
        let nonzero_residual_rows = res.iter().filter(|r| r.iter().any(|x| *x != 0)).count();
        let wg_nonzero = (0..VOCAB)
            .filter(|&r| (0..16).any(|j| final_core.wg.weight(r, j) != 0))
            .count();
        let mut occ = [0usize; 8];
        for &c in hard.iter() {
            occ[c as usize] += 1;
        }
        let ckpt = tr.checkpoint_bytes();
        write_checked(&args.root, &format!("arm{ai}-{}.ckpt", arm.name()), &ckpt)?;
        let art = final_core;
        let art_bytes = serde_json::to_vec(&json!({
            "arm": arm.name(),
            "palette": art.palette.elements,
            "identity": art.palette.identity,
            "reader_codes_sha256": sha256_hex(art.reader.packed()),
            "output_codes_sha256": sha256_hex(art.wg.packed()),
            "action_codes_sha256": sha256_hex(&art.action_codes),
            "action_codes": art.action_codes,
        }))
        .map_err(|e| e.to_string())?;
        write_checked(
            &args.root,
            &format!("arm{ai}-{}.cpl2.json", arm.name()),
            &art_bytes,
        )?;
        let mean_loss: f64 = pass_losses.iter().flatten().copied().sum::<f64>()
            / pass_losses.iter().flatten().count().max(1) as f64;
        let warm_loss: f64 = pass_losses
            .iter()
            .flatten()
            .take(WARMUP_UPDATES)
            .sum::<f64>()
            / WARMUP_UPDATES as f64;
        let post_loss: f64 = pass_losses
            .iter()
            .flatten()
            .skip(WARMUP_UPDATES)
            .sum::<f64>()
            / (UPDATES - WARMUP_UPDATES) as f64;
        fit_rows.push(json!({
            "arm": arm.name(),
            "index": ai,
            "mean_fit_loss_bits": mean_loss,
            "warmup_mean_loss_bits": warm_loss,
            "post_warmup_mean_loss_bits": post_loss,
            "reader_updates": tr.reader_updates,
            "action_updates": tr.action_updates,
            "hard_action_transitions": transitions,
            "hard_action_occupancy": occ,
            "distinct_hard_actions": occ.iter().filter(|c| **c > 0).count(),
            "residual_nonzero_states": nonzero_residual_rows,
            "output_rows_nonzero": wg_nonzero,
            "grad_norm_reader_last": grad_reader.last().copied().unwrap_or(0.0),
            "grad_norm_action_last": grad_action.last().copied().unwrap_or(0.0),
            "grad_norm_action_mean_post_warmup": grad_action.iter().skip(WARMUP_UPDATES).sum::<f64>() / (UPDATES - WARMUP_UPDATES) as f64,
            "elapsed_s": elapsed,
            "state_bytes": tr.state_bytes(),
            "checkpoints": checkpoints,
            "initial_actions_sha256": sha256_hex(&initial_actions),
        }));
        println!(
            "arm {}: fit {mean_loss:.4} bits | warm {warm_loss:.4} post {post_loss:.4} | action updates {} | hard transitions {transitions} | residual states {nonzero_residual_rows} | {elapsed:.1}s",
            arm.name(),
            tr.action_updates,
        );
        fitted.insert(*arm, tr);
    }

    // --- control diagnostics on the primary artifact ------------------------
    let primary = fitted
        .get(&Arm::LearnedOlder)
        .ok_or("missing primary arm")?
        .hard_core()?;
    let primary_res = residual_table(&primary);
    let perm = build_perm(&recs, 0xA5A5_1234u64);

    let own_states: Vec<Option<usize>> = recs
        .iter()
        .map(|r| state_own(&primary, false, &panel_windows[r.win], r.i))
        .collect();
    let donor_states: Vec<Option<usize>> = (0..recs.len())
        .map(|i| {
            if perm.eligible[i] {
                let d = perm.donor[i];
                state_own(&primary, false, &panel_windows[recs[d].win], recs[d].i)
            } else {
                own_states[i]
            }
        })
        .collect();
    let rev_states: Vec<Option<usize>> = recs
        .iter()
        .map(|r| state_reversed(&primary, &panel_windows[r.win], r.i))
        .collect();
    let disabled: Vec<Option<usize>> = vec![None; recs.len()];

    let own_loss = score(&parent, Some(&primary_res), &recs, &own_states, &mut cache);
    let perm_loss = score(
        &parent,
        Some(&primary_res),
        &recs,
        &donor_states,
        &mut cache,
    );
    let rev_loss = score(&parent, Some(&primary_res), &recs, &rev_states, &mut cache);
    let dis_loss = score(&parent, Some(&primary_res), &recs, &disabled, &mut cache);
    let parent_loss = score(&parent, None, &recs, &disabled, &mut cache);

    // State-disabled parity: the disabled arm must reproduce the parent's integer logits exactly.
    let mut disabled_parity = true;
    for r in recs.iter().take(64) {
        let a = primary.int_logits(r.prev, r.cur, None);
        let b = parent.int_logits(r.prev, r.cur, true);
        if a != b {
            disabled_parity = false;
        }
    }
    let changed_state_recipients = (0..recs.len())
        .filter(|&i| perm.eligible[i] && donor_states[i] != own_states[i])
        .count();

    let a_own = aggregate(&recs, &own_loss, None);
    let a_perm = aggregate(&recs, &perm_loss, None);
    let a_rev = aggregate(&recs, &rev_loss, None);
    let a_dis = aggregate(&recs, &dis_loss, None);
    let a_parent = aggregate(&recs, &parent_loss, None);

    let g_parent = paired_interval(&a_parent, &a_own, 0x1234_5678);
    let p_perm = paired_interval(&a_perm, &a_own, 0xD1B5_4A32);
    let p_rev = paired_interval(&a_rev, &a_own, 0x2545_F491);

    let elig = perm.eligible.clone();
    let a_own_e = aggregate(&recs, &own_loss, Some(&elig));
    let a_perm_e = aggregate(&recs, &perm_loss, Some(&elig));
    let p_perm_e = paired_interval(&a_perm_e, &a_own_e, 0x9E37_79B9);
    let eligibility_ok =
        a_own_e.docs() >= SUPPORT_MIN_DOCS && a_own_e.total().1 >= SUPPORT_MIN_TARGETS;

    // --- per-arm evaluation and control comparisons -------------------------
    let mut arm_rows = Vec::new();
    for arm in arms.iter() {
        let tr = fitted.get(arm).ok_or("missing arm")?;
        let core = tr.hard_core()?;
        let tail = !arm.uses_older_prefix();
        let res = residual_table(&core);
        let states: Vec<Option<usize>> = recs
            .iter()
            .map(|r| state_own(&core, tail, &panel_windows[r.win], r.i))
            .collect();
        let losses = score(&parent, Some(&res), &recs, &states, &mut cache);
        let agg = aggregate(&recs, &losses, None);
        let gp = paired_interval(&a_parent, &agg, 0x1234_5678);
        arm_rows.push((*arm, agg, gp, losses));
    }
    let mut arm_json = Vec::new();
    for (arm, agg, gp, losses) in arm_rows.iter() {
        let better_than: Vec<Value> = arm_rows
            .iter()
            .filter(|(b, _, _, _)| b != arm)
            .map(|(b, bg, _, _)| {
                let t = paired_interval(bg, agg, 0x0BAD_F00D);
                json!({"control": b.name(), "gain": t.0, "lo": t.1, "hi": t.2})
            })
            .collect();
        arm_json.push(json!({
            "arm": arm.name(),
            "panel_micro_bits": agg.micro(),
            "panel_document_macro_bits": agg.macro_bits(),
            "gain_vs_parent": iv(*gp),
            "paired_vs_controls": better_than,
            "per_document": agg.rows.iter().map(|(k, l, n)| json!({
                "content_id": k, "loss_sum": l, "count": n,
                "loss_per_target": if *n > 0 { json!(l / *n as f64) } else { Value::Null }
            })).collect::<Vec<_>>(),
            "losses_sha256": sha256_hex(
                &losses.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>()
            ),
        }));
    }

    // --- references --------------------------------------------------------
    let mut full1 = Cond::default();
    let mut full2 = Cond::default();
    for w in &fit_windows {
        for (_i, p, c, t) in targets(w, VOCAB) {
            full1.observe(c as u64, t);
            full2.observe(ctx2(p, c), t);
        }
    }
    let mut cons1 = Cond::default();
    let mut cons2 = Cond::default();
    for &wi in &consumed {
        for (_i, p, c, t) in targets(&fit_windows[wi], VOCAB) {
            cons1.observe(c as u64, t);
            cons2.observe(ctx2(p, c), t);
        }
    }
    let mut t512_1 = Cond::default();
    let mut t512_2 = Cond::default();
    for w in &train_windows {
        for (_i, p, c, t) in targets(w, VOCAB) {
            t512_1.observe(c as u64, t);
            t512_2.observe(ctx2(p, c), t);
        }
    }
    // Tune on all 38 tune documents via a newly named first/last-window selection (<=76).
    let mut tune_new: Vec<Vec<u32>> = Vec::new();
    for i in &tune_ids {
        let ws = windows_of(&tokenizer.encode(&uniq[*i].text));
        if ws.is_empty() {
            continue;
        }
        tune_new.push(ws[0].clone());
        if ws.len() >= 2 {
            tune_new.push(ws[ws.len() - 1].clone());
        }
        if tune_new.len() >= 76 {
            break;
        }
    }
    tune_new.truncate(76);
    let lam_full = tune_lambdas(&full1, &full2, &uni, &tune_new).0;
    let lam_cons = tune_lambdas(&cons1, &cons2, &uni, &tune_new).0;
    let lam_512 = tune_lambdas(&t512_1, &t512_2, &uni, &tune_new).0;
    let ref_loss = |c1: &Cond, c2: &Cond, l: (f64, f64)| -> Vec<f64> {
        recs.iter()
            .map(|r| {
                -family_p(c1, c2, &uni, r.prev, r.cur, r.target, l)
                    .max(1e-300)
                    .log2()
            })
            .collect()
    };
    let uni_loss: Vec<f64> = recs.iter().map(|r| -uni.p(r.target).log2()).collect();
    let r_full = ref_loss(&full1, &full2, lam_full);
    let r_cons = ref_loss(&cons1, &cons2, lam_cons);
    let r_512 = ref_loss(&t512_1, &t512_2, lam_512);
    let bias_loss: Vec<f64> = recs
        .iter()
        .map(|r| {
            let z = parent.int_logits(r.prev, r.cur, false);
            parent.bits_one(&z, r.target)
        })
        .collect();
    let aggs = [
        ("exact_unigram", aggregate(&recs, &uni_loss, None)),
        ("quantized_bias", aggregate(&recs, &bias_loss, None)),
        ("ref_full_fit", aggregate(&recs, &r_full, None)),
        ("ref_4096_consumed", aggregate(&recs, &r_cons, None)),
        ("ref_512_windows", aggregate(&recs, &r_512, None)),
    ];
    let ref_json: Vec<Value> = aggs
        .iter()
        .map(|(name, a)| {
            json!({
                "reference": name,
                "micro_bits": a.micro(),
                "document_macro_bits": a.macro_bits(),
                "gain_vs_parent": iv(paired_interval(a, &a_parent, 0x1111_2222)),
                "per_document": a.rows.iter().map(|(k, l, n)| json!({
                    "content_id": k, "loss_sum": l, "count": n
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    // --- screen -----------------------------------------------------------
    let primary_row = arm_json
        .iter()
        .find(|r| r["arm"] == Arm::LearnedOlder.name())
        .cloned()
        .unwrap_or(Value::Null);
    let fixed_lo = primary_row["paired_vs_controls"]
        .as_array()
        .map(|a| {
            a.iter()
                .find(|x| x["control"] == Arm::FixedOlder.name())
                .map(|x| {
                    (
                        x["gain"].as_f64().unwrap_or(f64::NAN),
                        x["lo"].as_f64().unwrap_or(f64::NAN),
                    )
                })
                .unwrap_or((f64::NAN, f64::NAN))
        })
        .unwrap_or((f64::NAN, f64::NAN));
    let tail_lo = primary_row["paired_vs_controls"]
        .as_array()
        .map(|a| {
            a.iter()
                .find(|x| x["control"] == Arm::LearnedTail.name())
                .map(|x| {
                    (
                        x["gain"].as_f64().unwrap_or(f64::NAN),
                        x["lo"].as_f64().unwrap_or(f64::NAN),
                    )
                })
                .unwrap_or((f64::NAN, f64::NAN))
        })
        .unwrap_or((f64::NAN, f64::NAN));
    let screen = json!({
        "criterion": ">=0.10 bits/target over the frozen parent with paired interval excluding zero; positive paired gains over both fitted controls with intervals excluding zero; positive conditional-permutation penalty with interval excluding zero and adequate support",
        "gain_vs_parent": iv(g_parent),
        "beats_parent": g_parent.0 >= SCREEN_MARGIN_BITS && g_parent.1 > 0.0,
        "gain_vs_fixed_action": {"point": fixed_lo.0, "lo": fixed_lo.1},
        "beats_fixed_action": fixed_lo.0 > 0.0 && fixed_lo.1 > 0.0,
        "gain_vs_local_tail": {"point": tail_lo.0, "lo": tail_lo.1},
        "beats_local_tail": tail_lo.0 > 0.0 && tail_lo.1 > 0.0,
        "perm_penalty_full": iv(p_perm),
        "perm_penalty_eligible": iv(p_perm_e),
        "perm_penalty_positive": p_perm_e.0 > 0.0 && p_perm_e.1 > 0.0,
        "eligibility_ok": eligibility_ok,
        "disabled_parity": disabled_parity,
        "passed": g_parent.0 >= SCREEN_MARGIN_BITS
            && g_parent.1 > 0.0
            && fixed_lo.0 > 0.0
            && fixed_lo.1 > 0.0
            && tail_lo.0 > 0.0
            && tail_lo.1 > 0.0
            && p_perm_e.0 > 0.0
            && p_perm_e.1 > 0.0
            && eligibility_ok
            && disabled_parity,
    });

    // --- generation --------------------------------------------------------
    let mut gen_rows = Vec::new();
    for (pi, prompt) in LEGACY_PROMPTS.iter().enumerate() {
        let mut push = |model: String, g: GenOut| {
            gen_rows.push(json!({
                "prompt_index": pi,
                "model": model,
                "output": g.output,
                "decoded": tokenizer.decode(&g.output),
                "pair_state_cycle": g.pair_cycle.map(|(e, p)| json!({"entry": e, "period": p})),
                "ring_state_cycle": g.ring_cycle.map(|(e, p)| json!({"entry": e, "period": p})),
                "first_repeated_output_pair_index": g.first_repeat_output_index,
                "cycle_certificate": "ring_state_cycle for bounded-context arms; pair_state_cycle is a sufficient-state certificate only for the two-token parent",
            }));
        };
        let mut pg = generate_parent(&parent, prompt, GEN_TOKENS);
        pg.decoded = tokenizer.decode(&pg.output);
        push("frozen_parent".into(), pg);
        for (ai, arm) in arms.iter().enumerate() {
            let core = fitted.get(arm).ok_or("missing arm")?.hard_core()?;
            let mut g = generate_full_state(&core, prompt, GEN_TOKENS);
            g.decoded = tokenizer.decode(&g.output);
            let _ = ai;
            push(arm.name().to_string(), g);
        }
        let rout = generate_reference(&full1, &full2, &uni, lam_full, prompt, GEN_TOKENS);
        gen_rows.push(json!({
            "prompt_index": pi,
            "model": "count_reference_full_fit",
            "output": rout,
            "decoded": tokenizer.decode(&rout),
            "diagnostic_only": true,
        }));
    }

    // --- persist -----------------------------------------------------------
    let exe_sha = std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::read(p).ok())
        .map(|b| sha256_hex(&b))
        .unwrap_or_else(|| "UNAVAILABLE".into());

    write_json(
        &args.root,
        "result.json",
        &json!({
            "schema": "uor-r4.prefix-state-pilot/1",
            "source_rev": args.source_rev,
            "evaluator": {"binary": "prefix-state-pilot", "git_rev": args.evaluator_rev, "binary_sha256": exe_sha},
            "parent": {
                "path": args.artifact,
                "sha256": artifact_sha,
                "matches_pinned": artifact_sha == EXPECTED_ARTIFACT_SHA256,
                "config": {"vocab": parent.cfg.vocab, "dv": parent.cfg.dv, "f_bits": parent.cfg.f_bits, "norm_bits": parent.cfg.norm_bits, "bias_scale_bits": parent.cfg.bias_scale_bits},
                "frozen_bias_matches_full_fit_unigram": bias_matches,
            },
            "checkpoint_exposure": {"sha256": ckpt_sha, "perm_len": ckpt_perm.len(), "consumed": consumed.len(), "train_windows": train_windows.len(), "train_targets": train_targets},
            "tokenizer": {"source_sha256": tok_sha, "derived_sha256": derived_sha, "vocab": VOCAB},
            "group": {
                "order": GROUP_ORDER,
                "palette": pal.elements,
                "identity": pal.identity,
                "closure": pal.closure_size(),
                "palette_digest": palette_digest(&pal),
            },
            "corpus": {
                "git_commit": args.source_rev,
                "collected_files": collected,
                "eligible_documents": uniq.len(),
                "exact_duplicates_grouped": duplicates,
                "fit_docs": fit_ids.len(),
                "tune_docs": tune_ids.len(),
                "dev_pool_docs": dev_pool.len(),
                "fit_windows": fit_windows.len(),
                "fit_targets": fit_targets,
            },
            "panel": {
                "documents": panel_doc_count,
                "windows": panel_windows.len(),
                "targets": panel_targets,
                "overlap_documents_with_legacy_panel": overlap,
                "short_document_policy": "up to 8 evenly spaced window indices min(8, W) per document; shortest documents contribute what exists",
            },
            "training": {"updates": UPDATES, "batch": BATCH, "warmup_updates": WARMUP_UPDATES, "passes": PASSES, "seed": SEED, "schedule": schedule},
            "fit": fit_rows,
            "arms": arm_json,
            "references": ref_json,
            "controls": {
                "conditional_permutation": {
                    "rule": "seeded Fisher-Yates bijection within (exact prev, exact cur, exact older-prefix length); donors may cross documents; no-history records excluded",
                    "strata": perm.strata,
                    "no_history_records_excluded": perm.excluded_no_history,
                    "eligible_records": perm.eligible.iter().filter(|x| **x).count(),
                    "changed_state_recipients": changed_state_recipients,
                    "full_population_penalty": iv(p_perm),
                    "eligible_penalty": iv(p_perm_e),
                    "eligible_documents": a_own_e.docs(),
                    "eligible_targets": a_own_e.total().1,
                    "donor_map": (0..recs.len()).map(|i| json!({
                        "obs": recs[i].obs, "recipient_doc": recs[i].doc,
                        "donor_index": perm.donor[i],
                        "donor_doc": recs[perm.donor[i]].doc,
                        "eligible": perm.eligible[i],
                    })).collect::<Vec<_>>(),
                },
                "reverse_older_order": {
                    "penalty": iv(p_rev),
                    "note": "sensitivity under an altered distribution, not a standalone usefulness or noncommutativity proof",
                },
                "state_disabled": {
                    "parity_with_parent_exact": disabled_parity,
                    "micro_bits": a_dis.micro(),
                },
            },
            "screen": screen,
            "timings": {"total_s": started.elapsed().as_secs_f64()},
            "limits": [
                "single 120-state register: at most log2(120)=6.91 bits",
                "pure group transitions are bijections and cannot selectively erase; the 64-token context supplies bounded forgetting",
                "palette restriction and one register/width/dose are disclosed capacity restrictions",
                "2I superiority over another algebra is NOT_TESTED",
                "no energy or latency claim; arithmetic counts only",
            ],
        }),
    )?;

    write_json(
        &args.root,
        "training.json",
        &json!({
            "schedule": schedule,
            "train_window_ids": train_ids,
            "train_windows": train_windows,
            "panel_windows": panel_windows,
            "panel_documents": panel_docs.iter().map(|(k, w, s)| json!({"content_id": k, "windows_in_doc": w, "selected": s})).collect::<Vec<_>>(),
            "parent_cache": {"entries": cache.entries(), "bytes": cache.bytes()},
        }),
    )?;

    let mut controls = serde_json::Map::new();
    controls.insert("own_micro_bits".into(), json!(a_own.micro()));
    controls.insert("parent_micro_bits".into(), json!(a_parent.micro()));
    controls.insert("permuted_micro_bits".into(), json!(a_perm.micro()));
    controls.insert("reversed_micro_bits".into(), json!(a_rev.micro()));
    write_json(&args.root, "controls.json", &Value::Object(controls))?;

    write_json(
        &args.root,
        "references.json",
        &json!({
            "families": {"context_1": "P(next|t-1)", "context_2": "P(next|t-2,t-1)"},
            "tune": {"documents": tune_ids.len(), "windows": tune_new.len(), "selection": "first and last window of each of the 38 tune documents (<=76), distinct from the old 32-document sample"},
            "lambdas": {"full_fit": [lam_full.0, lam_full.1], "consumed_4096": [lam_cons.0, lam_cons.1], "train_512": [lam_512.0, lam_512.1]},
            "full_fit": {"context_1_observed": full1.totals.len(), "context_2_observed": full2.totals.len()},
            "consumed_4096": {"context_1_observed": cons1.totals.len(), "context_2_observed": cons2.totals.len()},
            "train_512": {"context_1_observed": t512_1.totals.len(), "context_2_observed": t512_2.totals.len()},
            "rows": ref_json,
        }),
    )?;

    write_json(
        &args.root,
        "generation.json",
        &json!({"rows": gen_rows, "tokens": GEN_TOKENS, "prompts": LEGACY_PROMPTS}),
    )?;

    println!("legacy-parent micro {:.5} | panel parent {:.5} | primary {:.5} | perm {:.5} | rev {:.5} | disabled {:.5}",
        a_parent.micro(), a_parent.micro(), a_own.micro(), a_perm.micro(), a_rev.micro(), a_dis.micro());
    println!("screen: {:?}", screen["passed"]);
    seal(&args.root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&args.root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "sealed and verified ({} unlisted); elapsed {:.1}s",
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

fn parent_hash(artifact_sha: &str) -> [u8; 32] {
    Sha256::digest(artifact_sha.as_bytes()).into()
}

fn palette_digest(p: &uor_r4_core::native_geometric::learner::prefix_state::Palette) -> String {
    let mut b = p.elements.to_vec();
    b.push(p.identity);
    sha256_hex(&b)
}

fn build_schedule(n: usize, seed: u64) -> Vec<Vec<usize>> {
    let mut st = seed ^ 0xC0FF_EE00_1234_5678;
    let mut passes = Vec::new();
    for _ in 0..PASSES {
        let mut perm: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = (xorshift(&mut st) as usize) % (i + 1);
            perm.swap(i, j);
        }
        passes.push(perm);
    }
    passes
}

fn main() -> ExitCode {
    match run() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}
