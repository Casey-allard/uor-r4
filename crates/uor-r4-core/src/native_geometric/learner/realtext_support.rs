//! Shared real-text pilot support: corpus reconstruction, panel construction, document
//! aggregation, paired bootstrap intervals, conditional permutation and count references.
//!
//! Extracted so the recovery replay and the output-head diagnostic reuse one implementation rather
//! than copying another training/evaluation stack. The collection order and split rule are the
//! legacy ones, so populations stay comparable across the recorded runs.
#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::prior_learning::targets;
use crate::transformerless::hf_bpe::HfBpeTokenizer;

pub const VOCAB: usize = 4096;
pub const WINDOW: usize = 64;
pub const BOOTSTRAP_DRAWS: usize = 2000;
pub const LAMBDA_GRID: [f64; 10] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Split {
    Fit,
    Tune,
    Dev,
}

impl Split {
    pub fn name(self) -> &'static str {
        match self {
            Split::Fit => "fit",
            Split::Tune => "tune",
            Split::Dev => "dev_pool",
        }
    }
}

pub struct DocRec {
    pub path: String,
    pub sha256: [u8; 32],
    pub bytes: usize,
    pub text: String,
    pub split: Split,
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn sha256_hex(b: &[u8]) -> String {
    hex(&Sha256::digest(b))
}

pub fn xorshift(st: &mut u64) -> u64 {
    *st ^= *st << 13;
    *st ^= *st >> 7;
    *st ^= *st << 17;
    *st
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

/// All collected Markdown files, then the exact-duplicate-grouped unique population in collection
/// order. Split rule: `hash[0] % 10 < 8` fit, `== 8` tune, `== 9` dev pool.
pub fn reconstruct_corpus(dir: &Path) -> (Vec<DocRec>, usize, usize) {
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

pub fn windows_of(tokens: &[u32]) -> Vec<Vec<u32>> {
    tokens
        .chunks(WINDOW)
        .filter(|c| c.len() >= 3)
        .map(|c| c.to_vec())
        .collect()
}

/// One scored observation.
#[derive(Clone)]
pub struct Rec {
    pub obs: String,
    pub doc: String,
    pub win: usize,
    pub i: usize,
    pub prev: usize,
    pub cur: usize,
    pub target: u32,
    pub older_len: usize,
}

impl Rec {
    pub fn own_ctx(&self) -> (usize, usize) {
        (self.prev, self.cur)
    }
}

#[derive(Clone, Default)]
pub struct Agg {
    pub rows: Vec<(String, f64, usize)>,
}

impl Agg {
    pub fn total(&self) -> (f64, usize) {
        self.rows
            .iter()
            .fold((0.0f64, 0usize), |(a, b), (_, l, n)| (a + l, b + n))
    }
    pub fn micro(&self) -> f64 {
        let (l, n) = self.total();
        if n == 0 {
            f64::NAN
        } else {
            l / n as f64
        }
    }
    pub fn macro_bits(&self) -> f64 {
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
    pub fn docs(&self) -> usize {
        self.rows.iter().filter(|(_, _, n)| *n > 0).count()
    }
}

pub fn aggregate(recs: &[Rec], losses: &[f64], keep: Option<&[bool]>) -> Agg {
    assert_eq!(recs.len(), losses.len());
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

/// Paired document-cluster bootstrap of a loss difference (`a` minus `b`).
pub fn paired_interval(a: &Agg, b: &Agg, seed: u64) -> (f64, f64, f64) {
    assert_eq!(a.rows.len(), b.rows.len(), "scorers must be aligned");
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

pub struct Perm {
    pub donor: Vec<usize>,
    pub strata: usize,
    pub eligible: Vec<bool>,
    pub excluded_no_history: usize,
}

/// Seeded Fisher-Yates bijection within `(exact prev, exact cur, exact older-prefix length)`.
/// No-history records are excluded; donors may cross documents.
pub fn build_perm(recs: &[Rec], seed: u64) -> Perm {
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
    }
}

#[derive(Default)]
pub struct Cond {
    counts: HashMap<(u64, u32), u32>,
    pub totals: HashMap<u64, u64>,
}

impl Cond {
    pub fn observe(&mut self, ctx: u64, next: u32) {
        *self.counts.entry((ctx, next)).or_insert(0) += 1;
        *self.totals.entry(ctx).or_insert(0) += 1;
    }
    pub fn total(&self, ctx: u64) -> u64 {
        self.totals.get(&ctx).copied().unwrap_or(0)
    }
    pub fn count_of(&self, ctx: u64, next: u32) -> u32 {
        self.counts.get(&(ctx, next)).copied().unwrap_or(0)
    }
    pub fn successors(&self, ctx: u64) -> Vec<(u32, u32)> {
        let mut v: Vec<(u32, u32)> = self
            .counts
            .iter()
            .filter(|((c, _), _)| *c == ctx)
            .map(|((_, n), c)| (*n, *c))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }
    /// All distinct successors observed in this level's context.
    pub fn candidate_set(&self, ctx: u64) -> Vec<u32> {
        self.successors(ctx).into_iter().map(|(n, _)| n).collect()
    }
}

pub struct Uni {
    pub counts: Vec<u64>,
    pub total: u64,
}

impl Uni {
    pub fn p(&self, next: u32) -> f64 {
        (self.counts[next as usize] as f64 + 1.0) / (self.total as f64 + VOCAB as f64)
    }
    pub fn argmax(&self) -> u32 {
        let mut best = 0usize;
        for r in 1..self.counts.len() {
            if self.counts[r] > self.counts[best] {
                best = r;
            }
        }
        best as u32
    }
}

pub fn ctx2(prev: usize, cur: usize) -> u64 {
    (prev as u64) * VOCAB as u64 + cur as u64
}

pub fn family_p(
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

pub fn tune_lambdas(c1: &Cond, c2: &Cond, uni: &Uni, tune: &[Vec<u32>]) -> ((f64, f64), f64) {
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

/// Full-vocabulary argmax of an interpolated count family, with lowest-ID tie handling.
///
/// The earlier `generate_reference` scanned only `{unigram mode, prev, cur}`; that is not the count
/// model's greedy behaviour because the true maximizer can lie outside that set.
pub fn reference_argmax(
    c1: &Cond,
    c2: &Cond,
    uni: &Uni,
    prev: usize,
    cur: usize,
    l: (f64, f64),
) -> u32 {
    let mut best = 0u32;
    let mut best_p = f64::NEG_INFINITY;
    for v in 0..VOCAB as u32 {
        let p = family_p(c1, c2, uni, prev, cur, v, l);
        if p > best_p {
            best_p = p;
            best = v;
        }
    }
    best
}

/// Greedy generation under a full-vocabulary count reference.
pub fn generate_reference_full(
    c1: &Cond,
    c2: &Cond,
    uni: &Uni,
    l: (f64, f64),
    prompt: &[u32],
    n_new: usize,
) -> Vec<u32> {
    let mut toks = prompt.to_vec();
    for _ in 0..n_new {
        let i = toks.len() - 1;
        let prev = if i == 0 {
            VOCAB
        } else {
            (toks[i - 1] as usize).min(VOCAB - 1)
        };
        let cur = (toks[i] as usize).min(VOCAB - 1);
        toks.push(reference_argmax(c1, c2, uni, prev, cur, l));
    }
    toks[prompt.len()..].to_vec()
}

/// Panel construction: up to `max_per_doc` evenly spaced windows per dev-pool document.
pub struct Panel {
    pub windows: Vec<Vec<u32>>,
    pub window_docs: Vec<String>,
    pub docs: Vec<(String, usize, usize)>,
    pub recs: Vec<Rec>,
}

pub fn build_panel(
    uniq: &[DocRec],
    dev_pool: &[usize],
    tokenizer: &HfBpeTokenizer,
    max_per_doc: usize,
) -> Panel {
    let key_of = |i: usize| hex(&uniq[i].sha256);
    let mut windows: Vec<Vec<u32>> = Vec::new();
    let mut window_docs: Vec<String> = Vec::new();
    let mut docs: Vec<(String, usize, usize)> = Vec::new();
    for i in dev_pool {
        let key = key_of(*i);
        let ws = windows_of(&tokenizer.encode(&uniq[*i].text));
        if ws.is_empty() {
            docs.push((key, 0, 0));
            continue;
        }
        let count = max_per_doc.min(ws.len());
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
        docs.push((key.clone(), ws.len(), idxs.len()));
        for wi in idxs {
            windows.push(ws[wi].clone());
            window_docs.push(key.clone());
        }
    }
    let mut recs = Vec::new();
    for (wi, w) in windows.iter().enumerate() {
        for (i, prev, cur, target) in targets(w, VOCAB) {
            recs.push(Rec {
                obs: format!("{}:{}:{}", window_docs[wi], wi, i),
                doc: window_docs[wi].clone(),
                win: wi,
                i,
                prev,
                cur,
                target,
                older_len: older_len(i),
            });
        }
    }
    Panel {
        windows,
        window_docs,
        docs,
        recs,
    }
}

/// Number of older-prefix tokens for prediction position `i` (the local pair is excluded).
#[inline]
pub fn older_len(i: usize) -> usize {
    if i < 2 {
        0
    } else {
        (i - 1) - i.saturating_sub(WINDOW - 1)
    }
}

/// Cycle certificates computed from a generated stream.
///
/// `pair` is the sufficient state of the two-token parent; `ring` (ordered last <=64 tokens) is the
/// sufficient state of the bounded-context arms. A repeated pair is NOT a certificate for the new
/// arms.
pub fn cycle_certificates(
    prompt: &[u32],
    output: &[u32],
) -> (Option<(usize, usize)>, Option<(usize, usize)>) {
    let mut seen_pair: HashMap<(u32, u32), usize> = HashMap::new();
    let mut seen_ring: HashMap<Vec<u32>, usize> = HashMap::new();
    let mut pair = None;
    let mut ring = None;
    for t in 0..output.len() {
        let last = if t == 0 {
            prompt[prompt.len() - 1]
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
            let len = prompt.len() + t;
            let start = len.saturating_sub(WINDOW);
            for j in start..prompt.len() {
                key.push(prompt[j]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_argmax_scans_the_whole_vocabulary() {
        // A family whose true maximizer is neither the unigram argmax nor prev/cur.
        let mut c1 = Cond::default();
        let mut c2 = Cond::default();
        // Context (prev=7,cur=9) always emits 1234; the unigram mode is 5.
        for _ in 0..10 {
            c1.observe(9, 1234);
            c2.observe(ctx2(7, 9), 1234);
        }
        // Unigram: token 5 is the mode.
        let mut counts = vec![0u64; VOCAB];
        counts[5] = 100;
        counts[1234] = 1;
        let uni = Uni { counts, total: 101 };
        let got = reference_argmax(&c1, &c2, &uni, 7, 9, (0.9, 0.9));
        assert_eq!(got, 1234, "the true maximizer must be found by a full scan");
        assert!(got != 5 && got != 7 && got != 9);
    }

    #[test]
    fn older_len_excludes_the_local_pair() {
        assert_eq!(older_len(0), 0);
        assert_eq!(older_len(1), 0);
        assert_eq!(older_len(2), 1);
        assert_eq!(older_len(63), 62);
        assert_eq!(older_len(64), 62);
        assert_eq!(older_len(100), 62);
    }

    #[test]
    fn cycle_certificates_distinguish_pair_from_ring() {
        // Output that repeats a pair immediately but never repeats a 64-token ring.
        let prompt: Vec<u32> = (0..16).collect();
        let output: Vec<u32> = std::iter::once(7u32)
            .chain(std::iter::repeat(32).take(63))
            .collect();
        let (pair, ring) = cycle_certificates(&prompt, &output);
        assert!(pair.is_some(), "a repeated pair must be reported");
        assert!(ring.is_none(), "a repeated pair is not a ring certificate");
    }
}
