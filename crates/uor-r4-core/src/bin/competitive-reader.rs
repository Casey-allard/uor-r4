//! Competing-source geometric read: one shared causal inference path, exercised everywhere.
//!
//! Every entry point (teacher-forced evaluation, autoregressive generation, interventions, timing)
//! calls [`predict_next`] on the actual observed prefix. It needs no target, no sentinel and no
//! precomputed supervised record; losses are computed outside it. The relation encoding travels in
//! the artifact so no arm is fitted at one address and evaluated at another.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::json;

use uor_r4_core::native_geometric::learner::occurrence::{OccurrenceRef, OccurrenceRing};
use uor_r4_core::native_geometric::learner::prefix_artifact::{
    parent_hash_convention, ExactGroupTable,
};
use uor_r4_core::native_geometric::learner::prior_learning::PriorCore;
use uor_r4_core::native_geometric::learner::query_read::QueryHard;
use uor_r4_core::native_geometric::learner::realtext_support::*;
use uor_r4_core::native_geometric::learner::relational::*;
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::bpe_derive::{derive_tokenizer, derive_tokenizer_json};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const DEFAULT_ROOT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/competitive-reader-1";
const E_PATH: &str = "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/head-projection-3/corrected/empirical.cpl2";
const S_PATH: &str = "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/s-attribution-3/corrected/separable_older_query_read.cpx3";
const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json";
const DEFAULT_DOCS: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/inputs/docs";

const E_SHA: &str = "565cf98a01273af38425687adb522704b09a89bd7ab976b58b20bfbdaccfa0bf";
const S_SHA: &str = "ed9affd638ee93670559c626fd6dab91dc2a3c7faa5db66ec9e5a486bcdbdfad";
const TOKENIZER_SHA: &str = "9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c";
const DERIVED_SHA: &str = "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";

const RING_CAP: usize = 128;
const MAX_CAND: usize = 24;
const N_FIT: usize = 220;
const N_FRESH: usize = 140;
const STEPS: usize = 200;
const BATCH: usize = 16;
const LR: f64 = 0.08;
const ROUNDS: usize = 2;
/// Scalar refit steps after the first descriptor refinement: the phase the parent schedule lacked.
const STEPS_REFIT: usize = 200;
/// Development construction used only to select between exported fit phases (fit payload bank).
const N_TUNE: usize = 60;
const SEED_TUNE: u64 = 0x5C0F_7A11;
const SEED_FIT: u64 = 0x5C0F_1E71;
const SEED_FRESH: u64 = 0x5C0F_7F2E;
/// Prospectively declared primary margin: paired hard-action CE improvement over the exact reader.
const MARGIN_HARD_BITS: f64 = 0.05;
/// Prospectively declared central margin: contextual minus global-strength hard-action CE.
const MARGIN_CTX_BITS: f64 = 0.05;
/// Tolerated all-position regression on the natural-text development regression.
const MARGIN_TEXT_BITS: f64 = 0.05;
const GEN_TOKENS: usize = 48;
/// Documents per side of the document-separated reader text split (fit vs held out).
const TEXT_DOCS: usize = 8;
/// Bounded coordinate-descent rounds for the contextual interaction table.
const CTX_ROUNDS: usize = 2;
/// Fixed causal bucket count, matching `relational::CTX_BUCKETS`.
const N_CTX_BUCKETS: usize = 16;
const LEGACY_PROMPTS: [[u32; 16]; 6] = [
    [
        19, 1071, 2615, 3927, 930, 2095, 198, 198, 828, 67, 85, 437, 1548, 216, 39, 287,
    ],
    [
        93, 84, 643, 933, 1855, 1160, 984, 342, 471, 31, 375, 481, 99, 77, 24, 591,
    ],
    [
        3911, 58, 3700, 68, 79, 61, 3872, 30, 93, 84, 25, 808, 30, 669, 3140, 829,
    ],
    [32; 16],
    [198; 16],
    [0; 16],
];

// ---------------------------------------------------------------------------
// The single shared causal inference boundary
// ---------------------------------------------------------------------------

/// Outcome of one target-free read decision.
#[derive(Debug, PartialEq, Eq)]
struct Read {
    /// `None` means NoRead; otherwise `(candidate index, strength)`.
    action: Option<(usize, usize)>,
    source: Option<OccurrenceRef>,
    /// The exact observed payload token of the selected occurrence.
    payload: Option<u32>,
    /// Absolute position of that payload inside the ring, for interventions.
    payload_abs: Option<u32>,
    /// The directed relative element of the selected occurrence.
    rel: usize,
    admitted: usize,
    scanned: usize,
}

/// **The one read path.** Operates on the actual observed prefix; no target or sentinel is needed.
/// A prefix with no admissible candidate returns `action = None`, and the caller emits the local
/// prediction.
fn read_step(
    ring: &OccurrenceRing,
    cur: u32,
    prev: u32,
    prev2: u32,
    sel: &RelationalSelector,
    table: &ExactGroupTable,
    use_reader: bool,
) -> Read {
    if !use_reader {
        return Read {
            action: None,
            source: None,
            payload: None,
            payload_abs: None,
            rel: 0,
            admitted: 0,
            scanned: 0,
        };
    }
    let i = ring.written();
    let (cands, stats) = admit_mixed(ring, i, cur, prev, prev2, MAX_CAND);
    let mut out = Read {
        action: None,
        source: None,
        payload: None,
        payload_abs: None,
        rel: 0,
        admitted: cands.len(),
        scanned: stats.scanned,
    };
    if cands.is_empty() {
        return out;
    }
    let qr = prev as usize;
    let rel: Vec<usize> = cands
        .iter()
        .map(|c| {
            let kr = if c.x_prev == NO_TOKEN {
                return 0;
            } else {
                c.x_prev as usize
            };
            relation_index(
                table,
                sel.mode,
                &sel.code_of,
                *sel.q_roots.get(qr).unwrap_or(&0),
                *sel.q_roots.get(kr).unwrap_or(&0),
                qr,
                kr,
            )
        })
        .collect();
    if let Some((k, st)) = sel.choose(&cands, &rel) {
        out.action = Some((k, st));
        out.source = Some(cands[k].slot_ref);
        out.payload = Some(cands[k].payload);
        out.payload_abs = Some(cands[k].abs + 1);
        out.rel = rel[k];
    }
    out
}

/// Served next-token scores on the actual prefix. Used by evaluation, generation and timing alike.
#[allow(clippy::too_many_arguments)]
fn predict_next(
    ring: &OccurrenceRing,
    cur: u32,
    prev: usize,
    prev2: u32,
    sel: &RelationalSelector,
    table: &ExactGroupTable,
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    use_reader: bool,
) -> (Vec<i32>, Read) {
    let mut z = local_logits(parent, local, u, cur, prev, prev2, ring.written());
    let r = read_step(ring, cur, prev as u32, prev2, sel, table, use_reader);
    if let Some((_, st)) = r.action {
        if let Some(p) = r.payload {
            let p = p as usize;
            if p < z.len() {
                z[p] = z[p].saturating_add(1i32 << AMP_SHIFTS[st]);
            }
        }
    }
    (z, r)
}

/// Frozen local baseline `z_local = z_E + u_S(b)` for `i >= 2`, else `z_E`.
///
/// `i` is the position being predicted; the prefix `x_0..x_{i-1}` is already in the ring.
fn local_logits(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    cur: u32,
    prev: usize,
    _prev2: u32,
    i: u32,
) -> Vec<i32> {
    let v = parent.cfg.vocab;
    let mut z = parent.int_logits(prev.min(v - 1), (cur as usize).min(v - 1), true);
    if i >= 2 {
        let b = local.query_state(cur);
        for (x, y) in z.iter_mut().zip(u[b].iter()) {
            *x += *y;
        }
    }
    z
}

/// Independent S-query-only reconstruction using the retained scorer's own absence behaviour.
fn s_query_only(
    ring: &OccurrenceRing,
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    cur: u32,
    prev: usize,
) -> Vec<i32> {
    let v = parent.cfg.vocab;
    let i = ring.written();
    if i < 2 {
        return parent.int_logits(prev.min(v - 1), (cur as usize).min(v - 1), true);
    }
    let b = local.query_state(cur);
    let e = local.table.identity as usize;
    local
        .int_logits(prev.min(v - 1), (cur as usize).min(v - 1), Some((b, e)))
        .iter()
        .enumerate()
        .map(|(k, z)| z - u[e][k])
        .collect()
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn hex_to_bytes(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("odd hex length".into());
    }
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

fn sha256_bytes(b: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(b).into()
}

fn hex_of(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn write_checked(root: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    let p = root.join(name);
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("mkdir {}: {e}", d.display()))?;
    }
    std::fs::write(&p, bytes).map_err(|e| format!("write {}: {e}", p.display()))
}

fn write_json(root: &Path, name: &str, v: &serde_json::Value) -> Result<(), String> {
    write_checked(
        root,
        name,
        serde_json::to_string_pretty(v)
            .map_err(|e| format!("json: {e}"))?
            .as_bytes(),
    )
}

fn xorshift(st: &mut u64) -> u64 {
    *st ^= *st << 13;
    *st ^= *st >> 7;
    *st ^= *st << 17;
    *st
}

fn pick<T: Copy>(v: &[T], st: &mut u64) -> T {
    v[(xorshift(st) as usize) % v.len()]
}

fn argmax_low(z: &[i32]) -> usize {
    let mut b = 0usize;
    for r in 1..z.len() {
        if z[r] > z[b] {
            b = r;
        }
    }
    b
}

fn bits(z: &[i32], target: u32, f_bits: u32) -> f64 {
    let scale = (-(f_bits as f64)).exp2();
    let t = (target as usize).min(z.len() - 1);
    let mut max = f64::NEG_INFINITY;
    for &v in z {
        max = max.max(v as f64 * scale);
    }
    let mut sum = 0.0;
    for &v in z {
        sum += (v as f64 * scale - max).exp();
    }
    (max + sum.ln() - z[t] as f64 * scale) / std::f64::consts::LN_2
}

fn prob_of(z: &[i32], token: u32, f_bits: u32) -> f64 {
    let scale = (-(f_bits as f64)).exp2();
    let mut max = f64::NEG_INFINITY;
    for &v in z {
        max = max.max(v as f64 * scale);
    }
    let mut sum = 0.0;
    let mut want = 0.0;
    for (k, &v) in z.iter().enumerate() {
        let e = (v as f64 * scale - max).exp();
        sum += e;
        if k == token as usize {
            want = e;
        }
    }
    want / sum
}

struct Banks {
    pairs: Vec<(u32, u32)>,
    keys: Vec<u32>,
    values_fit: Vec<u32>,
    values_held: Vec<u32>,
}

#[derive(Clone)]
struct Seq {
    tokens: Vec<u32>,
    group: u16,
    /// The correct answer for the final query, for scoring only.
    answer: u32,
    /// Construction metadata used only by fixture checks and reporting.
    absent: bool,
    answer_source_abs: Option<usize>,
}

/// A discriminating construction: several **competing plausible sources** share the query key and
/// all use roles from the same paired-role class, so no source-class shortcut separates them. The
/// correct block's role is the query role's partner; the competitors use partners from *other*
/// families. Placement and recency are balanced, and `absent` queries have no correct source.
fn make_seq(banks: &Banks, st: &mut u64, values: &[u32], absent: bool) -> Seq {
    let n_fam = 4 + (xorshift(st) as usize) % 3;
    let mut fams: Vec<(u32, u32)> = Vec::new();
    while fams.len() < n_fam {
        let f = pick(&banks.pairs, st);
        if !fams.contains(&f) {
            fams.push(f);
        }
    }
    let qkey = pick(&banks.keys, st);
    let target_fam = fams[0];
    let (qa, qb) = if xorshift(st) & 1 == 0 {
        (target_fam.0, target_fam.1)
    } else {
        (target_fam.1, target_fam.0)
    };
    let qrole = qb; // the partner must supply the answer
    let aval = pick(values, st);
    // One block per family, all with the same key. Only the target family's block carries the
    // answer; every competitor is a plausible paired-role source with a different value.
    let mut blocks: Vec<(u32, u32, u32, bool)> = Vec::new();
    for (i, (a, b)) in fams.iter().enumerate() {
        if i == 0 && absent {
            continue;
        }
        let role = if i == 0 {
            qa
        } else if xorshift(st) & 1 == 0 {
            *a
        } else {
            *b
        };
        let mut v = pick(values, st);
        if i == 0 {
            v = aval;
        } else if v == aval {
            v = values[(values.iter().position(|x| *x == aval).unwrap_or(0) + 1) % values.len()];
        }
        blocks.push((role, qkey, v, i == 0));
    }
    // A filler block with a different key, so candidates are not only same-key.
    let mut filler_key = pick(&banks.keys, st);
    if filler_key == qkey {
        filler_key = banks.keys
            [(banks.keys.iter().position(|x| *x == qkey).unwrap_or(0) + 1) % banks.keys.len()];
    }
    let mut filler_value = pick(values, st);
    if filler_value == aval {
        filler_value =
            values[(values.iter().position(|x| *x == aval).unwrap_or(0) + 1) % values.len()];
    }
    blocks.push((pick(&banks.pairs, st).0, filler_key, filler_value, false));
    // Randomize the target's position among plausible competitors.
    for i in (1..blocks.len()).rev() {
        let j = (xorshift(st) as usize) % (i + 1);
        blocks.swap(i, j);
    }
    let mut tokens = Vec::new();
    let mut answer_source_abs = None;
    for (r, k, v, is_answer) in blocks.iter() {
        if *is_answer {
            answer_source_abs = Some(tokens.len() + 2);
        }
        tokens.push(*r);
        tokens.push(*k);
        tokens.push(*v);
    }
    // For absence the observed next token is deliberately absent from all source payloads.
    tokens.extend_from_slice(&[qrole, qkey, aval]);
    Seq {
        tokens,
        group: 0,
        answer: aval,
        absent,
        answer_source_abs,
    }
}

fn validate_fixture(seq: &Seq, banks: &Banks) -> Result<(), String> {
    if seq.tokens.len() < 6 || seq.tokens.len() % 3 != 0 {
        return Err("fixture must contain source triples and one query triple".into());
    }
    let end = seq.tokens.len() - 3;
    let q = seq.tokens[end];
    let key = seq.tokens[end + 1];
    let partner = banks
        .pairs
        .iter()
        .find_map(|&(a, b)| {
            if a == q {
                Some(b)
            } else if b == q {
                Some(a)
            } else {
                None
            }
        })
        .ok_or("query role has no declared partner")?;
    let matching: Vec<usize> = seq.tokens[..end]
        .chunks_exact(3)
        .enumerate()
        .filter(|(_, b)| b[0] == partner && b[1] == key)
        .map(|(i, _)| i * 3 + 2)
        .collect();
    if seq.tokens[end + 2] != seq.answer {
        return Err("fixture target disagrees with answer metadata".into());
    }
    if seq.absent {
        if !matching.is_empty()
            || seq.answer_source_abs.is_some()
            || seq.tokens[..end]
                .chunks_exact(3)
                .any(|b| b[2] == seq.answer)
        {
            return Err("absent fixture contains a relevant source or answer payload".into());
        }
    } else if matching.len() != 1
        || seq.answer_source_abs != matching.first().copied()
        || seq.tokens[matching[0]] != seq.answer
    {
        return Err(
            "present fixture must have exactly one partner source carrying the answer".into(),
        );
    }
    Ok(())
}

fn make_pop(banks: &Banks, n: usize, seed: u64, values: &[u32]) -> Vec<Seq> {
    let mut st = seed | 1;
    let mut out = Vec::with_capacity(n);
    for g in 0..n {
        let absent = xorshift(&mut st) % 100 < 15;
        let mut s = make_seq(banks, &mut st, values, absent);
        s.group = g as u16;
        out.push(s);
    }
    out
}

fn build_banks(tok: &HfBpeTokenizer, vocab: usize) -> Result<Banks, String> {
    let mut words: Vec<u32> = Vec::new();
    for id in 0..vocab as u32 {
        let s = tok.decode(&[id]);
        if s.len() < 3 || s.len() > 7 || !s.bytes().all(|b| b.is_ascii_lowercase()) {
            continue;
        }
        if tok.encode(&s) == vec![id] {
            words.push(id);
        }
        if words.len() >= 200 {
            break;
        }
    }
    if words.len() < 80 {
        return Err(format!("only {} single-token words", words.len()));
    }
    Ok(Banks {
        pairs: (0..14).map(|k| (words[k], words[14 + k])).collect(),
        keys: words[28..52].to_vec(),
        values_fit: words[52..76].to_vec(),
        values_held: words[76..88].to_vec(),
    })
}

/// One supervised observation, computed **outside** the inference function.
struct Obs {
    /// Prefix state at the decision point.
    ring: OccurrenceRing,
    cur: u32,
    prev: u32,
    prev2: u32,
    target: u32,
    /// The candidate pool, for losses and counterfactuals only.
    cands: Vec<Cand>,
    group: u16,
}

/// Walk a sequence with the shared path, recording observations. The ring is advanced by
/// `observe` after each decision, so no future token can enter it.
fn observe(seq: &Seq) -> Vec<Obs> {
    let mut ring = OccurrenceRing::new(RING_CAP);
    let mut out = Vec::new();
    for i in 0..seq.tokens.len() {
        let cur = seq.tokens[i];
        let prev = if i >= 1 { seq.tokens[i - 1] } else { 0 };
        let prev2 = if i >= 2 { seq.tokens[i - 2] } else { NO_TOKEN };
        if i >= 2 && i + 1 < seq.tokens.len() {
            let (cands, _) = admit_mixed(&ring, ring.written(), cur, prev, prev2, MAX_CAND);
            if !cands.is_empty() {
                out.push(Obs {
                    ring: ring.clone(),
                    cur,
                    prev,
                    prev2,
                    target: seq.tokens[i + 1],
                    cands,
                    group: seq.group,
                });
            }
        }
        ring.observe(cur);
    }
    out
}

fn cand_list(cands: &[Cand]) -> Vec<Cand> {
    cands.to_vec()
}

/// Reporting-only stratum label for the construction, whose triples are `(role, key, value)`.
/// Never a serving feature and never a filter: harmful NoRead and uncovered positions stay in the
/// all-position totals.
fn stratum_of(o: &Obs, seq: &Seq) -> &'static str {
    let len = seq.tokens.len();
    let t = o.ring.written() as usize + 1;
    if t + 1 == len {
        return if seq.absent {
            "final_absent"
        } else {
            "final_present"
        };
    }
    if t + 2 == len {
        return "query_key";
    }
    if t + 2 < len {
        return match t % 3 {
            0 => "intermediate_role",
            1 => "intermediate_key",
            _ => "intermediate_value",
        };
    }
    "other"
}

/// Relation indices for a whole pool under one selector.
fn rels_for(
    sel: &RelationalSelector,
    o: &Obs,
    table: &ExactGroupTable,
    query_blind: bool,
) -> Vec<usize> {
    let qr = if query_blind { 0usize } else { o.prev as usize };
    o.cands
        .iter()
        .map(|c| {
            let kr = if c.x_prev == NO_TOKEN {
                return 0;
            } else {
                c.x_prev as usize
            };
            relation_index(
                table,
                sel.mode,
                &sel.code_of,
                *sel.q_roots.get(qr).unwrap_or(&0),
                *sel.q_roots.get(kr).unwrap_or(&0),
                qr,
                kr,
            )
        })
        .collect()
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

/// A complete binding digest over every observation the reader was fitted or scored on: the
/// construction split, the text-fit documents, the tune construction and the fresh draw, including
/// candidate payloads, references and targets. A source hash alone is not the binding.
fn binding_digest(
    fit_obs: &[Vec<Obs>],
    text_obs: &[Vec<Obs>],
    tune_obs: &[Vec<Obs>],
    fresh_obs: &[Vec<Obs>],
) -> [u8; 32] {
    let mut b = Vec::new();
    for (label, part) in [
        ("fit", fit_obs),
        ("text_fit", text_obs),
        ("tune", tune_obs),
        ("fresh", fresh_obs),
    ] {
        b.extend_from_slice(label.as_bytes());
        b.push(0);
        for os in part.iter() {
            b.extend_from_slice(&(os.len() as u32).to_le_bytes());
            for o in os.iter() {
                b.extend_from_slice(&o.cur.to_le_bytes());
                b.extend_from_slice(&o.prev.to_le_bytes());
                b.extend_from_slice(&o.prev2.to_le_bytes());
                b.extend_from_slice(&o.target.to_le_bytes());
                b.extend_from_slice(&o.group.to_le_bytes());
                b.extend_from_slice(&o.ring.written().to_le_bytes());
                b.extend_from_slice(&(o.cands.len() as u32).to_le_bytes());
                for c in o.cands.iter() {
                    b.extend_from_slice(&c.abs.to_le_bytes());
                    b.extend_from_slice(&c.payload.to_le_bytes());
                    b.extend_from_slice(&c.x_cur.to_le_bytes());
                    b.extend_from_slice(&c.x_prev.to_le_bytes());
                    b.extend_from_slice(&c.x_prev2.to_le_bytes());
                    b.extend_from_slice(&c.feats);
                }
            }
        }
    }
    sha256_bytes(&b)
}

/// Paired cluster interval of `a - b` on the fresh construction, optionally restricted to one
/// stratum, with the ratio-of-resampled-sums estimator.
#[allow(clippy::too_many_arguments)]
fn paired_fresh_diff(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    fresh_obs: &[Vec<Obs>],
    fresh: &[Seq],
    a_sel: &RelationalSelector,
    b_sel: &RelationalSelector,
    stratum: Option<&str>,
    seed: u64,
) -> serde_json::Value {
    let mut pairs: Vec<(f64, f64, usize)> = Vec::new();
    for (gi, os) in fresh_obs.iter().enumerate() {
        let (mut sa, mut sb, mut n) = (0.0f64, 0.0f64, 0usize);
        for o in os.iter() {
            if let Some(want) = stratum {
                if stratum_of(o, &fresh[gi]) != want {
                    continue;
                }
            }
            let (za, _) = predict_next(
                &o.ring,
                o.cur,
                o.prev as usize,
                o.prev2,
                a_sel,
                table,
                parent,
                local,
                u,
                true,
            );
            let (zb, _) = predict_next(
                &o.ring,
                o.cur,
                o.prev as usize,
                o.prev2,
                b_sel,
                table,
                parent,
                local,
                u,
                true,
            );
            sa += bits(&za, o.target, parent.cfg.f_bits);
            sb += bits(&zb, o.target, parent.cfg.f_bits);
            n += 1;
        }
        if n > 0 {
            pairs.push((sa, sb, n));
        }
    }
    let mut out = boot_diff(&pairs, seed);
    if let Some(w) = stratum {
        out["stratum"] = json!(w);
    }
    out
}

/// Snapshot one fit phase: quantized selector, soft objective, and the **exported** decision quality
/// on the development construction used for phase selection.
fn phase_snapshot(
    tr: &RelationalTrainer,
    table: &ExactGroupTable,
    positions: &[TrainPos],
    tune: &[TrainPos],
) -> (RelationalSelector, f64, f64, usize) {
    let sel = tr.quantize();
    let soft: f64 = positions.iter().map(|p| tr.position_loss(p)).sum();
    let (dev_loss, dev_reads) = sel.exported_objective(table, tune);
    (sel, soft, dev_loss, dev_reads)
}

/// Whether the pool's best finite action belongs to a candidate carrying the target payload.
/// Duplicate correct payloads all qualify; this is never a first-matching-occurrence test.
fn best_is_correct(p: &TrainPos, o: &Obs) -> bool {
    if p.cands.is_empty() {
        return false;
    }
    let lstar = |k: usize| {
        p.delta[k]
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min)
            .min(0.0)
    };
    let best = (0..p.cands.len()).map(lstar).fold(0.0f64, f64::min);
    (0..p.cands.len()).any(|k| lstar(k) <= best + 1e-12 && o.cands[k].payload == o.target)
}

/// Record one fit phase: soft objective, descriptor bookkeeping and the exported development quality.
#[allow(clippy::too_many_arguments)]
fn push_phase(
    tr: &RelationalTrainer,
    table: &ExactGroupTable,
    positions: &[TrainPos],
    tune: &[TrainPos],
    label: &str,
    moves_total: f64,
    phases: &mut Vec<serde_json::Value>,
    checkpoints: &mut Vec<(RelationalSelector, f64, usize)>,
) {
    let (sel, soft, dev_loss, dev_reads) = phase_snapshot(tr, table, positions, tune);
    phases.push(json!({
        "phase": label,
        "soft_objective": soft,
        "descriptor_moves_total": moves_total,
        "descriptor_evaluations_total": tr.descriptor_evaluations,
        "exported_dev_loss_nats": dev_loss,
        "exported_dev_reads": dev_reads,
    }));
    checkpoints.push((sel, dev_loss, dev_reads));
}

/// Apply the **identical** contextual utility procedure to one source arm: compute that arm's own
/// frozen buckets, then fit only the interaction from zero. Returns the arm, its fit receipt, the
/// used-bucket count and whether nothing but `ctx` moved.
fn apply_ctx_procedure(
    table: &ExactGroupTable,
    positions: &[TrainPos],
    base: &RelationalSelector,
    rounds: usize,
    vocab: usize,
) -> Result<(RelationalSelector, CtxFit, usize, bool), String> {
    let mut sel = base.clone();
    let buckets = sel.buckets_of(table, positions);
    let used = (0..CTX_BUCKETS).filter(|b| buckets.contains(b)).count();
    let fit = sel.ctx_fit(table, positions, &buckets, rounds);
    sel.validate_for_vocab(vocab)?;
    let only_ctx = {
        let mut frozen = sel.clone();
        frozen.ctx = Vec::new();
        frozen == *base
    };
    Ok((sel, fit, used, only_ctx))
}

/// Active-candidate probe: the same token stream through the shared path, reporting the observed
/// read/NoRead actions and the work a real serving step would do. This is the cost measurement the
/// zero-candidate text probe cannot supply.
fn active_probe(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    sel: &RelationalSelector,
    table: &ExactGroupTable,
    stream: &[u32],
    use_reader: bool,
) -> (u32, usize, usize, usize) {
    let mut last = 0u32;
    let mut admitted = 0usize;
    let mut reads = 0usize;
    let mut predictions = 0usize;
    if !use_reader {
        for k in 2..stream.len().saturating_sub(1) {
            let z = local_logits(
                parent,
                local,
                u,
                stream[k],
                stream[k - 1] as usize,
                stream[k - 2],
                k as u32,
            );
            last = black_box(argmax_low(&z)) as u32;
            predictions += 1;
        }
        return (last, predictions, 0, 0);
    }
    let mut ring = OccurrenceRing::new(RING_CAP);
    for k in 0..stream.len() {
        let cur = stream[k];
        let prev = if k >= 1 {
            stream[k - 1] as usize
        } else {
            parent.cfg.pad_row()
        };
        let prev2 = if k >= 2 { stream[k - 2] } else { NO_TOKEN };
        if k >= 2 && k + 1 < stream.len() {
            let (z, r) = predict_next(&ring, cur, prev, prev2, sel, table, parent, local, u, true);
            last = black_box(argmax_low(&z)) as u32;
            predictions += 1;
            admitted += r.admitted;
            if r.action.is_some() {
                reads += 1;
            }
        }
        ring.observe(cur);
    }
    (last, predictions, admitted, reads)
}

fn boot_diff(pairs: &[(f64, f64, usize)], seed: u64) -> serde_json::Value {
    if pairs.len() < 8 {
        return json!({"point": null, "lo": null, "hi": null, "note": "too few groups"});
    }
    let positions: usize = pairs.iter().map(|(_, _, n)| n).sum();
    let point = pairs.iter().map(|(a, b, _)| a - b).sum::<f64>() / positions.max(1) as f64;
    let mut st = seed | 1;
    let mut boots = Vec::new();
    for _ in 0..2000 {
        let mut s = 0.0;
        let mut n = 0usize;
        for _ in 0..pairs.len() {
            let k = (xorshift(&mut st) as usize) % pairs.len();
            s += pairs[k].0 - pairs[k].1;
            n += pairs[k].2;
        }
        boots.push(s / n.max(1) as f64);
    }
    boots.sort_by(|a, b| a.partial_cmp(b).unwrap());
    json!({
        "point": point,
        "lo": boots[(boots.len() as f64 * 0.025) as usize],
        "hi": boots[((boots.len() as f64 * 0.975) as usize).min(boots.len() - 1)],
        "draws": boots.len(),
        "groups": pairs.len(),
        "positions": positions,
        "unit": "bits_per_candidate_bearing_position",
        "resampling_unit": "sequence",
    })
}

fn run() -> Result<ExitCode, String> {
    let root = {
        let mut a = std::env::args().skip(1);
        let mut r = PathBuf::from(DEFAULT_ROOT);
        let mut src = None;
        while let Some(k) = a.next() {
            match k.as_str() {
                "--root" => r = PathBuf::from(a.next().ok_or("--root value")?),
                "--source-root" => src = Some(PathBuf::from(a.next().ok_or("value")?)),
                other => return Err(format!("unknown argument {other}")),
            }
        }
        (r, src)
    };
    let (root, source_root) = root;
    claim(&root).map_err(|e| format!("claim {}: {e}", root.display()))?;
    let started = Instant::now();
    let mut marks: Vec<(&'static str, f64)> = Vec::new();
    let mark = |n: &'static str, m: &mut Vec<(&'static str, f64)>| {
        m.push((n, started.elapsed().as_secs_f64()));
    };

    let pb = std::fs::read(E_PATH).map_err(|e| format!("E: {e}"))?;
    if sha256_hex(&pb) != E_SHA {
        return Err("E sha mismatch".into());
    }
    let parent = PriorCore::from_bytes(&pb).map_err(|e| format!("load E: {e}"))?;
    parent.validate()?;
    let (parent_digest, _) = parent_hash_convention(&pb);
    let sb = std::fs::read(S_PATH).map_err(|e| format!("S: {e}"))?;
    if sha256_hex(&sb) != S_SHA {
        return Err("S sha mismatch".into());
    }
    let tb = std::fs::read(DEFAULT_TOKENIZER).map_err(|e| format!("tokenizer: {e}"))?;
    if sha256_hex(&tb) != TOKENIZER_SHA {
        return Err("tokenizer sha mismatch".into());
    }
    let derived = sha256_hex(&derive_tokenizer_json(&tb, VOCAB).map_err(|e| format!("{e}"))?);
    if derived != DERIVED_SHA {
        return Err(format!("derived tokenizer sha {derived} != pinned"));
    }
    let tokenizer = derive_tokenizer(&tb, VOCAB).map_err(|e| format!("derive: {e}"))?;
    let raw_tok: [u8; 32] = hex_to_bytes(&derived)?.try_into().unwrap();
    let local = QueryHard::from_bytes(&sb, &parent, &parent_digest, &raw_tok)
        .map_err(|e| format!("load S: {e}"))?;
    let u: Vec<Vec<i32>> = (0..120).map(|s| local.row_scores(s)).collect();
    let table = ExactGroupTable::build().map_err(|e| format!("table: {e}"))?;
    let executable_sha256 = hex_of(&sha256_bytes(
        &std::env::current_exe()
            .ok()
            .and_then(|p| std::fs::read(p).ok())
            .unwrap_or_default(),
    ));
    let sources: Vec<serde_json::Value> = match &source_root {
        Some(sr) => [
            "crates/uor-r4-core/src/native_geometric/learner/relational.rs",
            "crates/uor-r4-core/src/bin/competitive-reader.rs",
        ]
        .iter()
        .map(|rel| match std::fs::read(sr.join(rel)) {
            Ok(b) => json!({"path": rel, "sha256": sha256_hex(&b)}),
            Err(_) => json!({"path": rel, "sha256": "UNAVAILABLE"}),
        })
        .collect(),
        None => vec![json!({"path": "unspecified", "sha256": "UNAVAILABLE"})],
    };
    mark("pinned inputs", &mut marks);

    let banks = build_banks(&tokenizer, parent.cfg.vocab)?;
    let fit = make_pop(&banks, N_FIT, SEED_FIT, &banks.values_fit);
    let fresh = make_pop(&banks, N_FRESH, SEED_FRESH, &banks.values_held);
    for seq in fit.iter().chain(fresh.iter()) {
        validate_fixture(seq, &banks)?;
    }
    let fit_obs: Vec<Vec<Obs>> = fit.iter().map(observe).collect();
    let fresh_obs: Vec<Vec<Obs>> = fresh.iter().map(observe).collect();
    let total = |v: &[Vec<Obs>]| -> (usize, usize, usize) {
        let n: usize = v.iter().map(|x| x.len()).sum();
        let covered = v
            .iter()
            .flatten()
            .filter(|o| o.cands.iter().any(|c| c.payload == o.target))
            .count();
        let multi = v.iter().flatten().filter(|o| o.cands.len() >= 2).count();
        (n, covered, multi)
    };
    let (fit_n, fit_cov, fit_multi) = total(&fit_obs);
    let (fresh_n, fresh_cov, fresh_multi) = total(&fresh_obs);
    println!(
        "banks {} pairs / {} keys; fit {fit_n} positions ({fit_cov} covered, {fit_multi} competing); fresh {fresh_n} ({fresh_cov} covered, {fresh_multi} competing); absent queries {}",
        banks.pairs.len(),
        banks.keys.len(),
        fresh.iter().filter(|s| s.absent).count()
    );
    write_json(
        &root,
        "population.json",
        &json!({
            "format": "v2: several blocks (role,key,value) sharing the query key with roles drawn from the SAME paired-role class, then a query (role,key). A present query has exactly one partner-role source carrying its answer. An absent query has no partner-role source and its target is absent from all source payloads. Fixture truth is checked before fitting; metadata is never a serving feature.",
            "held_out": "fresh uses a disjoint payload bank and fresh draws; role/context combinations differ from fit",
            "seeds": {"fit": SEED_FIT, "fresh": SEED_FRESH},
            "banks": {"role_pairs": banks.pairs, "keys": banks.keys, "values_fit": banks.values_fit, "values_held_out": banks.values_held},
            "fit_sequences": fit.iter().map(|s| json!({"group": s.group, "tokens": s.tokens, "answer": s.answer, "absent": s.absent, "answer_source_abs": s.answer_source_abs})).collect::<Vec<_>>(),
            "fresh_sequences": fresh.iter().map(|s| json!({"group": s.group, "tokens": s.tokens, "answer": s.answer, "absent": s.absent, "answer_source_abs": s.answer_source_abs})).collect::<Vec<_>>(),
        }),
    )?;
    mark("construction", &mut marks);

    // ---- reader text: document-separated fit and held-out evaluation ---------------
    // The pinned docs corpus is the corpus the frozen local predictor was trained on. That older
    // exposure is disclosed separately and is NOT removed by this split. What the split does
    // guarantee is that the *new reader* never fits and evaluates the same document.
    let (uniq, _, _) = reconstruct_corpus(Path::new(DEFAULT_DOCS));
    let dev: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Dev)
        .collect();
    // Deterministic document order by content hash, so the split cannot follow a listing order.
    let mut dev_sorted: Vec<usize> = dev.clone();
    dev_sorted.sort_by_key(|i| uniq[*i].sha256);
    let text_fit_docs: Vec<usize> = dev_sorted.iter().copied().take(TEXT_DOCS).collect();
    let text_eval_docs: Vec<usize> = dev_sorted
        .iter()
        .copied()
        .skip(TEXT_DOCS)
        .take(TEXT_DOCS)
        .collect();
    if text_fit_docs.len() < TEXT_DOCS || text_eval_docs.len() < TEXT_DOCS {
        return Err(
            "not enough disjoint Dev documents for a document-separated reader split".into(),
        );
    }
    for a in text_fit_docs.iter() {
        for b in text_eval_docs.iter() {
            if uniq[*a].path == uniq[*b].path || uniq[*a].sha256 == uniq[*b].sha256 {
                return Err("reader text fit and held-out documents overlap".into());
            }
        }
    }
    let window_seq = |i: usize, group: u16| -> Option<Seq> {
        windows_of(&tokenizer.encode(&uniq[i].text))
            .into_iter()
            .next()
            .map(|w| Seq {
                tokens: w,
                group,
                answer: 0,
                absent: false,
                answer_source_abs: None,
            })
    };
    let text_fit_seqs: Vec<Seq> = text_fit_docs
        .iter()
        .filter_map(|i| window_seq(*i, 0xF000))
        .collect();
    let text_fit_tokens: usize = text_fit_seqs.iter().map(|s| s.tokens.len()).sum();
    let text_obs: Vec<Vec<Obs>> = text_fit_seqs.iter().map(observe).collect();
    let text_fit_n: usize = text_obs.iter().map(|v| v.len()).sum();
    let doc_id = |i: usize| {
        json!({
            "path": uniq[i].path,
            "sha256": hex_of(&uniq[i].sha256),
            "bytes": uniq[i].bytes,
            "split": uniq[i].split.name(),
        })
    };
    let text_split_report = json!({
        "fit_documents": text_fit_docs.iter().map(|i| doc_id(*i)).collect::<Vec<_>>(),
        "held_out_documents": text_eval_docs.iter().map(|i| doc_id(*i)).collect::<Vec<_>>(),
        "fit_documents_without_a_window": text_fit_docs.len() - text_fit_seqs.len(),
        "disjoint_by_path_and_content_hash": true,
        "fit_tokens": text_fit_tokens,
        "fit_candidate_positions": text_fit_n,
        "scope": "The new reader never fits and evaluates the same document. The frozen local prior E was trained on this pinned corpus, so the prior's historical exposure is disclosed, not removed; this is a reader-held-out local evaluation, not an externally unseen claim.",
    });

    // ---- training positions, computed outside the inference function -------------
    let init_roots: Vec<u8> = local
        .a_codes
        .iter()
        .map(|c| local.palette.elements[*c as usize])
        .collect();
    let init_codes: Vec<u8> = (0..parent.cfg.vocab).map(|t| (t % 120) as u8).collect();
    let cfg_id = sha256_bytes(
        format!("{RING_CAP}|{MAX_CAND}|{STEPS}|{BATCH}|{LR}|{ROUNDS}|{SEED_FIT}").as_bytes(),
    );

    // One supervised position from one observation: constants only, computed outside inference.
    let pos_of = |o: &Obs| -> TrainPos {
        let z = local_logits(
            &parent,
            &local,
            &u,
            o.cur,
            o.prev as usize,
            o.prev2,
            o.ring.written(),
        );
        let delta: Vec<[f64; ACTS]> = o
            .cands
            .iter()
            .map(|c| {
                let p = prob_of(&z, c.payload, parent.cfg.f_bits);
                std::array::from_fn(|a| {
                    let boost = (1i64 << AMP_SHIFTS[a]) as f64 / (1i64 << parent.cfg.f_bits) as f64;
                    action_loss(p, boost, c.payload == o.target)
                })
            })
            .collect();
        TrainPos {
            cands: cand_list(&o.cands),
            q_role: o.prev as usize,
            k_role: o
                .cands
                .iter()
                .map(|c| {
                    if c.x_prev == NO_TOKEN {
                        usize::MAX
                    } else {
                        c.x_prev as usize
                    }
                })
                .collect(),
            delta,
            group: o.group,
        }
    };
    let build_positions =
        |obs: &[Vec<Obs>]| -> Vec<TrainPos> { obs.iter().flatten().map(&pos_of).collect() };
    // One mixture, identical for every arm: constructed fit split + natural-text development.
    let mut positions = build_positions(&fit_obs);
    let constructed_fit_positions = positions.len();
    positions.extend(build_positions(&text_obs));
    let mut data_bytes = Vec::new();
    for (gi, o) in fit_obs.iter().enumerate() {
        for x in o.iter() {
            data_bytes.push(gi as u8);
            data_bytes.extend_from_slice(&x.cur.to_le_bytes());
            data_bytes.extend_from_slice(&x.prev.to_le_bytes());
            data_bytes.extend_from_slice(&x.target.to_le_bytes());
            data_bytes.push(x.cands.len() as u8);
        }
    }
    let data_id = sha256_bytes(&data_bytes);

    // ---- tune construction: development phase selection, disjoint from fit and fresh ------
    let tune = make_pop(&banks, N_TUNE, SEED_TUNE, &banks.values_fit);
    for s in tune.iter() {
        validate_fixture(s, &banks)?;
    }
    let tune_obs: Vec<Vec<Obs>> = tune.iter().map(observe).collect();
    let tune_positions = build_positions(&tune_obs);
    let tune_n: usize = tune_obs.iter().map(|v| v.len()).sum();

    // ---- fit with a bounded alternating schedule; one dose and one selection rule for every arm --
    const PHASE_LABELS: [&str; 4] = [
        "scalar_fit",
        "descriptor_refine_1",
        "scalar_refit",
        "descriptor_refine_2",
    ];
    let mut arms: BTreeMap<&'static str, RelationalSelector> = BTreeMap::new();
    let mut arm_report = Vec::new();
    for name in ["exact", "categorical", "relational"] {
        let mode = match name {
            "exact" => RelMode::ExactOnly,
            "categorical" => RelMode::Categorical,
            _ => RelMode::Geometric,
        };
        let mut tr = RelationalTrainer::new(
            parent.cfg.vocab,
            table.clone(),
            LR,
            SEED_FIT,
            data_id,
            cfg_id,
            if mode == RelMode::Geometric {
                &init_roots
            } else {
                &init_codes
            },
        )
        .with_mode(
            mode,
            if mode == RelMode::Categorical {
                init_codes.clone()
            } else {
                Vec::new()
            },
        );
        let t0 = Instant::now();
        let mut phases: Vec<serde_json::Value> = Vec::new();
        let mut checkpoints: Vec<(RelationalSelector, f64, usize)> = Vec::new();
        // Phase 1: scalar fit on the soft expected-action objective.
        for _ in 0..STEPS {
            let b = tr.next_batch(positions.len(), BATCH);
            tr.step(&positions, &b);
        }
        push_phase(
            &tr,
            &table,
            &positions,
            &tune_positions,
            PHASE_LABELS[0],
            0.0,
            &mut phases,
            &mut checkpoints,
        );
        // Phase 2: bounded discrete descriptor refinement.
        let m2 = if mode != RelMode::ExactOnly {
            tr.refine_descriptor(&positions, ROUNDS)
        } else {
            0.0
        };
        push_phase(
            &tr,
            &table,
            &positions,
            &tune_positions,
            PHASE_LABELS[1],
            m2,
            &mut phases,
            &mut checkpoints,
        );
        // Phase 3: scalar **refit** after the descriptors moved: the phase the parent lacked, so the
        // rank/feature/strength scores are no longer stale with respect to the new assignments.
        for _ in 0..STEPS_REFIT {
            let b = tr.next_batch(positions.len(), BATCH);
            tr.step(&positions, &b);
        }
        push_phase(
            &tr,
            &table,
            &positions,
            &tune_positions,
            PHASE_LABELS[2],
            m2,
            &mut phases,
            &mut checkpoints,
        );
        // Phase 4: descriptor refinement again against the refitted scores.
        let m4 = if mode != RelMode::ExactOnly {
            tr.refine_descriptor(&positions, ROUNDS)
        } else {
            0.0
        };
        push_phase(
            &tr,
            &table,
            &positions,
            &tune_positions,
            PHASE_LABELS[3],
            m2 + m4,
            &mut phases,
            &mut checkpoints,
        );
        // Select by **exported** decision quality on the development construction: lowest exported
        // loss, then fewest reads. This is not the soft surrogate.
        let chosen = checkpoints
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.1.partial_cmp(&b.1).unwrap().then(a.2.cmp(&b.2)))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let sel = checkpoints[chosen].0.clone();
        sel.validate_for_vocab(parent.cfg.vocab)?;
        arm_report.push(json!({
            "arm": name,
            "mode": format!("{mode:?}"),
            "seconds": t0.elapsed().as_secs_f64(),
            "steps": tr.step,
            "phases": phases,
            "selected_phase_index": chosen,
            "selected_phase": PHASE_LABELS[chosen],
            "descriptor_assignments_moved": tr.descriptor_moved(),
            "descriptor_evaluations": tr.descriptor_evaluations,
            "descriptor_refine_1_change": m2,
            "descriptor_refine_2_change": m4,
            "code_search_change": if mode == RelMode::Categorical { m2 + m4 } else { 0.0 },
            "quantized": {"w": sel.w.to_vec(), "bias": sel.bias, "sb": sel.sb.to_vec(), "noread": sel.noread,
                          "rank_nonzero": sel.rank.iter().filter(|v| **v != 0).count()},
        }));
        arms.insert(name, sel);
    }
    let relational = arms["relational"].clone();
    let exact = arms["exact"].clone();
    let categorical = arms["categorical"].clone();
    mark("fit", &mut marks);

    // ---- contextual utility controller: the same procedure for BOTH source arms --------------
    // A frozen table does not freeze its function: the ranker's winner and margin determine the
    // utility bucket, so `ctx` must be re-fitted to each arm's own scorer. Both arms get the same
    // procedure, rounds, data and starting point (all-zero = that arm's factorised baseline).
    let (relational_ctx, rel_ctx_fit, rel_ctx_used, rel_ctx_only) = apply_ctx_procedure(
        &table,
        &positions,
        &relational,
        CTX_ROUNDS,
        parent.cfg.vocab,
    )?;
    let (categorical_ctx, cat_ctx_fit, cat_ctx_used, cat_ctx_only) = apply_ctx_procedure(
        &table,
        &positions,
        &categorical,
        CTX_ROUNDS,
        parent.cfg.vocab,
    )?;
    // A relation-channel lesion of the contextual arm: reliance check, not query blindness.
    let relational_ctx_lesion = relational_ctx.without_relation();
    // An all-zero table must be **behaviourally** identical to that arm's baseline. Struct equality
    // is the wrong check: the empty and all-zero encodings differ as vectors while acting the same.
    let zero_table_matches = |base: &RelationalSelector, arm: &RelationalSelector| -> bool {
        let mut zero = base.clone();
        zero.ctx = vec![[0i32; ACTS + 1]; N_CTX_BUCKETS];
        let mut same = true;
        for os in fresh_obs.iter().chain(fit_obs.iter()) {
            for o in os.iter() {
                let rel = rels_for(base, o, &table, false);
                if zero.choose(&o.cands, &rel) != base.choose(&o.cands, &rel) {
                    same = false;
                }
            }
        }
        let _ = arm;
        same
    };
    let ctx_report = json!({
        "procedure": "identical for both source arms: compute that arm's own frozen buckets, then fit only ctx from zero by bounded coordinate descent on the same positions",
        "buckets": N_CTX_BUCKETS,
        "rounds": CTX_ROUNDS,
        "relational": {
            "buckets_used": rel_ctx_used, "entries_moved": rel_ctx_fit.entries_moved,
            "evaluations": rel_ctx_fit.evaluations,
            "fit_objective_before": rel_ctx_fit.objective_before,
            "fit_objective_after": rel_ctx_fit.objective_after,
            "table": relational_ctx.ctx.clone(),
            "only_ctx_changed": rel_ctx_only,
            "zero_table_behaviourally_equals_baseline": zero_table_matches(&relational, &relational_ctx),
        },
        "categorical": {
            "buckets_used": cat_ctx_used, "entries_moved": cat_ctx_fit.entries_moved,
            "evaluations": cat_ctx_fit.evaluations,
            "fit_objective_before": cat_ctx_fit.objective_before,
            "fit_objective_after": cat_ctx_fit.objective_after,
            "table": categorical_ctx.ctx.clone(),
            "only_ctx_changed": cat_ctx_only,
            "zero_table_behaviourally_equals_baseline": zero_table_matches(&categorical, &categorical_ctx),
        },
        "constructed_fit_positions": constructed_fit_positions,
        "text_fit_positions": text_fit_n,
        "tune_positions": tune_n,
        "soft_objective_note": "the ctx fit objective is the softmax surrogate on the served integers; the exported hard decision quality is reported per arm",
    });
    mark("ctx fit (both arms)", &mut marks);

    // ---- artifact export / independent reload; the RELOADED selectors are what gets exercised --
    let mut export_bytes: BTreeMap<&'static str, Vec<u8>> = BTreeMap::new();
    let mut reloaded_arms: BTreeMap<&'static str, RelationalSelector> = BTreeMap::new();
    let mut arm_parity_failures = 0usize;
    for (name, sel) in [
        ("exact", &exact),
        ("categorical", &categorical),
        ("categorical_ctx", &categorical_ctx),
        ("relational", &relational),
        ("relational_ctx", &relational_ctx),
    ] {
        let a = RelationalArtifact {
            selector: sel.clone(),
            local_artifact_digest: sha256_bytes(&sb),
            tokenizer_digest: raw_tok,
            data_digest: data_id,
        };
        let bytes = a.to_bytes();
        write_checked(&root, &format!("artifacts/{name}.rlr2"), &bytes)?;
        match RelationalArtifact::from_bytes(&bytes, &sha256_bytes(&sb), &raw_tok) {
            Ok(b) => {
                if b.selector != *sel || b.selector.validate_for_vocab(parent.cfg.vocab).is_err() {
                    arm_parity_failures += 1;
                }
                reloaded_arms.insert(name, b.selector);
            }
            Err(_) => arm_parity_failures += 1,
        }
        export_bytes.insert(name, bytes);
    }
    let relational_reload = reloaded_arms["relational"].clone();
    let relational_ctx_reload = reloaded_arms["relational_ctx"].clone();
    let categorical_reload = reloaded_arms["categorical"].clone();
    let categorical_ctx_reload = reloaded_arms["categorical_ctx"].clone();
    let exact_reload = reloaded_arms["exact"].clone();
    let reload_matches_ctx =
        relational_ctx_reload == relational_ctx && relational_ctx_reload.ctx.len() == N_CTX_BUCKETS;
    check_reload(
        reloaded_arms.get("categorical").ok_or("cat")?,
        &table,
        &template_obs(&fresh_obs),
    )?;

    // ---- evaluation through the shared path ------------------------------------
    let mut panel = Vec::new();
    let mut text_rows = Vec::new();
    for (panel_name, obs, pop) in [("fit", &fit_obs, &fit), ("fresh", &fresh_obs, &fresh)] {
        let mut rows = Vec::new();
        for (name, sel) in arms.iter().chain(std::iter::empty()) {
            let _ = name;
            let _ = sel;
        }
        let arm_specs: Vec<(&'static str, Option<&RelationalSelector>)> = vec![
            ("local", None),
            ("exact", Some(&exact_reload)),
            ("categorical", Some(&categorical_reload)),
            ("categorical_ctx", Some(&categorical_ctx_reload)),
            ("relational", Some(&relational_reload)),
            ("relational_ctx", Some(&relational_ctx_reload)),
            ("relational_ctx_lesion", Some(&relational_ctx_lesion)),
        ];
        for (name, sel) in arm_specs {
            let mut n = 0usize;
            let mut reads = 0usize;
            let mut correct_reads = 0usize;
            let mut covered = 0usize;
            let mut correct_covered = 0usize;
            let mut multi_candidates = 0usize;
            let mut multi_correct = 0usize;
            let mut hard_ce = 0.0f64;
            let mut emitted_ok = 0usize;
            let mut served_not_ungated = 0usize;
            let mut rank_regret = 0.0f64;
            let mut gate_regret = 0.0f64;
            let mut dose_regret = 0.0f64;
            let mut pool_best_correct = 0usize;
            let mut per_group: Vec<(f64, f64)> = Vec::new();
            let mut outcomes = Vec::new();
            let mut strata: BTreeMap<&str, (usize, usize, usize, usize, f64)> = BTreeMap::new();
            for (gi, os) in obs.iter().enumerate() {
                let mut g_hard = 0.0f64;
                let mut g_local = 0.0f64;
                for o in os.iter() {
                    n += 1;
                    let local_z = local_logits(
                        &parent,
                        &local,
                        &u,
                        o.cur,
                        o.prev as usize,
                        o.prev2,
                        o.ring.written(),
                    );
                    let (z, r, detail) = match sel {
                        None => (
                            local_z.clone(),
                            Read {
                                action: None,
                                source: None,
                                payload: None,
                                payload_abs: None,
                                rel: 0,
                                admitted: 0,
                                scanned: 0,
                            },
                            None,
                        ),
                        Some(s) => {
                            let (z, r) = predict_next(
                                &o.ring,
                                o.cur,
                                o.prev as usize,
                                o.prev2,
                                s,
                                &table,
                                &parent,
                                &local,
                                &u,
                                true,
                            );
                            let rel = rels_for(s, o, &table, false);
                            let d = s.decision_detail(&o.cands, &rel);
                            (z, r, Some(d))
                        }
                    };
                    let covered_here = o.cands.iter().any(|c| c.payload == o.target);
                    if covered_here {
                        covered += 1;
                    }
                    if let Some((k, _)) = r.action {
                        reads += 1;
                        // Any correct payload counts, never a first-matching occurrence index.
                        if o.cands[k].payload == o.target {
                            correct_reads += 1;
                        }
                        if covered_here && o.cands[k].payload == o.target {
                            correct_covered += 1;
                        }
                    }
                    let hb = bits(&z, o.target, parent.cfg.f_bits);
                    let lb = bits(&local_z, o.target, parent.cfg.f_bits);
                    hard_ce += hb;
                    g_hard += hb;
                    g_local += lb;
                    let emitted = argmax_low(&z) as u32;
                    if emitted == o.target {
                        emitted_ok += 1;
                    }
                    // Exact regret decomposition on the actual exported decision.
                    let (reg, source_detail) = match sel {
                        None => (
                            Regret::default(),
                            json!({"ungated_top_source": null, "served_source": null,
                                   "payload": null, "bucket": null, "action": null}),
                        ),
                        Some(s) => {
                            let p = pos_of(o);
                            let reg = regret_decomposition(s, &table, &p);
                            if r.action.is_some() && !reg.served_matches_ungated {
                                served_not_ungated += 1;
                            }
                            if best_is_correct(&p, o) {
                                pool_best_correct += 1;
                            }
                            let (top, act, bucket) = detail.unwrap_or((None, None, 0));
                            let sd = json!({
                                "ungated_top_source": top.map(|k| json!({"seq": o.cands[k].slot_ref.seq, "abs": o.cands[k].abs})),
                                "served_source": r.source.map(|s2| json!({"seq": s2.seq, "abs": s2.abs})),
                                "payload": r.payload, "bucket": bucket,
                                "action": act.map(|(k, a)| json!({"candidate": k, "strength": a})),
                            });
                            (reg, sd)
                        }
                    };
                    rank_regret += reg.ranking;
                    gate_regret += reg.gate;
                    dose_regret += reg.dose;
                    let stratum = stratum_of(o, &pop[gi]);
                    let entry = strata.entry(stratum).or_default();
                    if o.cands.len() >= 2 {
                        multi_candidates += 1;
                        if covered_here && r.payload == Some(o.target) {
                            multi_correct += 1;
                        }
                    }
                    entry.0 += 1;
                    entry.1 += usize::from(covered_here);
                    entry.2 += usize::from(r.payload == Some(o.target));
                    entry.3 += usize::from(emitted == o.target);
                    entry.4 += hb;
                    outcomes.push(json!({
                        "group": o.group, "query_abs": o.ring.written(), "stratum": stratum,
                        "target": o.target, "covered": covered_here, "candidates": o.cands.len(),
                        "source": source_detail,
                        "payload_abs": r.payload_abs, "payload": r.payload,
                        "strength": r.action.map(|(_, a)| a), "emitted": emitted,
                        "hard_bits": hb, "local_bits": lb,
                        "ranking_regret_nats": reg.ranking, "gate_regret_nats": reg.gate,
                        "dose_regret_nats": reg.dose, "actual_nats": reg.actual, "lpool_nats": reg.lpool,
                        "served_matches_ungated": reg.served_matches_ungated,
                    }));
                }
                per_group.push((g_hard, g_local));
            }
            let local_total: f64 = per_group.iter().map(|(_, l)| l).sum();
            rows.push(json!({
                "arm": name,
                "positions": n,
                "hard_action_ce_bits": hard_ce / n.max(1) as f64,
                "hard_action_ce_bits_vs_local": (hard_ce - local_total) / n.max(1) as f64,
                "emitted_next_token_correct": emitted_ok,
                "emitted_accuracy": emitted_ok as f64 / n.max(1) as f64,
                "covered_positions": covered,
                "reads": reads,
                "correct_reads": correct_reads,
                "read_precision_all": if reads == 0 { f64::NAN } else { correct_reads as f64 / reads as f64 },
                "read_precision_covered": if covered == 0 { f64::NAN } else { correct_covered as f64 / covered as f64 },
                "covered_decision_success": if covered == 0 { f64::NAN } else { correct_covered as f64 / covered as f64 },
                "multi_candidate_positions": multi_candidates,
                "multi_candidate_correct_reads": multi_correct,
                "no_read_rate": 1.0 - reads as f64 / n.max(1) as f64,
                "read_actions_where_served_source_differs_from_ungated_top": served_not_ungated,
                "mean_ranking_regret_nats": rank_regret / n.max(1) as f64,
                "mean_gate_regret_nats": gate_regret / n.max(1) as f64,
                "mean_dose_regret_nats": dose_regret / n.max(1) as f64,
                "mean_total_regret_nats": (rank_regret + gate_regret + dose_regret) / n.max(1) as f64,
                "positions_where_pool_best_action_is_correct": pool_best_correct,
                "strata": strata.iter().map(|(label, (n, covered, correct, emitted, ce))| json!({
                    "stratum": label, "positions": n, "covered_positions": covered,
                    "correct_reads": correct, "emitted_correct": emitted,
                    "hard_action_ce_bits": ce / (*n).max(1) as f64,
                })).collect::<Vec<_>>(),
            }));
            write_json(
                &root,
                &format!("outcomes-{panel_name}-{name}.json"),
                &json!({"positions": outcomes}),
            )?;
            if panel_name == "fresh" {
                let mut sums: BTreeMap<u16, (f64, f64, usize)> = BTreeMap::new();
                for (gi, os) in obs.iter().enumerate() {
                    let e = sums.entry(pop[gi].group).or_insert((0.0, 0.0, 0));
                    let mut h = 0.0;
                    let mut l = 0.0;
                    for o in os.iter() {
                        let lz = local_logits(
                            &parent,
                            &local,
                            &u,
                            o.cur,
                            o.prev as usize,
                            o.prev2,
                            o.ring.written(),
                        );
                        l += bits(&lz, o.target, parent.cfg.f_bits);
                        h += match sel {
                            None => bits(&lz, o.target, parent.cfg.f_bits),
                            Some(s) => {
                                let rel = rels_for(s, o, &table, false);
                                let mut z = lz.clone();
                                if let Some((k, st)) = s.choose(&o.cands, &rel) {
                                    let p = o.cands[k].payload as usize;
                                    if p < z.len() {
                                        z[p] = z[p].saturating_add(1i32 << AMP_SHIFTS[st]);
                                    }
                                }
                                bits(&z, o.target, parent.cfg.f_bits)
                            }
                        };
                    }
                    e.0 += h;
                    e.1 += l;
                    e.2 += os.len();
                }
                let pairs: Vec<(f64, f64)> = sums.values().map(|(h, l, _k)| (*h, *l)).collect();
                let per_pos_unused = pairs.len();
                let _ = per_pos_unused;
                write_json(
                    &root,
                    &format!("per-group-{name}.json"),
                    &json!({"groups": sums.iter().map(|(g, (h, l, k))| json!({"group": g, "hard_sum_bits": h, "local_sum_bits": l, "positions": k})).collect::<Vec<_>>()}),
                )?;
            }
        }
        panel.push(json!({"panel": panel_name, "arms": rows}));
    }

    // Paired interval of the hard-loss difference against the exact arm, fresh panel.
    let mut pairs: Vec<(f64, f64, usize)> = Vec::new();
    for (_gi, os) in fresh_obs.iter().enumerate() {
        let mut a = 0.0;
        let mut b = 0.0;
        for o in os.iter() {
            let lz = local_logits(
                &parent,
                &local,
                &u,
                o.cur,
                o.prev as usize,
                o.prev2,
                o.ring.written(),
            );
            let rel_r = rels_for(&relational_reload, o, &table, false);
            let rel_e = rels_for(&exact_reload, o, &table, false);
            let mut zr = lz.clone();
            if let Some((k, st)) = relational_reload.choose(&o.cands, &rel_r) {
                let p = o.cands[k].payload as usize;
                zr[p] = zr[p].saturating_add(1i32 << AMP_SHIFTS[st]);
            }
            let mut ze = lz.clone();
            if let Some((k, st)) = exact_reload.choose(&o.cands, &rel_e) {
                let p = o.cands[k].payload as usize;
                ze[p] = ze[p].saturating_add(1i32 << AMP_SHIFTS[st]);
            }
            a += bits(&zr, o.target, parent.cfg.f_bits);
            b += bits(&ze, o.target, parent.cfg.f_bits);
        }
        pairs.push((a, b, os.len()));
    }
    let hard_diff = boot_diff(&pairs, 0x1234_5678);

    // ---- fresh paired comparisons on the independently reloaded artifacts -----------------
    let ctx_diff = paired_fresh_diff(
        &parent,
        &local,
        &u,
        &table,
        &fresh_obs,
        &fresh,
        &relational_ctx_reload,
        &relational_reload,
        None,
        0x0C7A_1E17,
    );
    let ctx_diff_present = paired_fresh_diff(
        &parent,
        &local,
        &u,
        &table,
        &fresh_obs,
        &fresh,
        &relational_ctx_reload,
        &relational_reload,
        Some("final_present"),
        0x0C7A_1E18,
    );
    let ctx_diff_absent = paired_fresh_diff(
        &parent,
        &local,
        &u,
        &table,
        &fresh_obs,
        &fresh,
        &relational_ctx_reload,
        &relational_reload,
        Some("final_absent"),
        0x0C7A_1E19,
    );
    // The matched nongeometric comparison: categorical + the same contextual procedure.
    let cat_ctx_vs_rel_ctx = paired_fresh_diff(
        &parent,
        &local,
        &u,
        &table,
        &fresh_obs,
        &fresh,
        &categorical_ctx_reload,
        &relational_ctx_reload,
        None,
        0x0C7A_1E1A,
    );
    let cat_ctx_vs_rel_ctx_present = paired_fresh_diff(
        &parent,
        &local,
        &u,
        &table,
        &fresh_obs,
        &fresh,
        &categorical_ctx_reload,
        &relational_ctx_reload,
        Some("final_present"),
        0x0C7A_1E1B,
    );
    // Reliance check: the relation channel removed from the contextual arm.
    let rel_ctx_vs_lesion = paired_fresh_diff(
        &parent,
        &local,
        &u,
        &table,
        &fresh_obs,
        &fresh,
        &relational_ctx_reload,
        &relational_ctx_lesion,
        None,
        0x0C7A_1E1C,
    );
    // The categorical + contextual arm against its own global-strength parent.
    let cat_ctx_vs_cat = paired_fresh_diff(
        &parent,
        &local,
        &u,
        &table,
        &fresh_obs,
        &fresh,
        &categorical_ctx_reload,
        &categorical_reload,
        None,
        0x0C7A_1E1D,
    );

    // ---- exact regret decomposition on the reloaded arms (target-using; diagnostic only) ----
    // Replaces the retired "ranking dominance" diagnostic, which measured the global arm and charged
    // NoRead's missing gain to ranking. This decomposes each arm's own exported decisions exactly.
    let decomp_arms: Vec<(&'static str, &RelationalSelector)> = vec![
        ("relational", &relational_reload),
        ("relational_ctx", &relational_ctx_reload),
        ("categorical_ctx", &categorical_ctx_reload),
    ];
    let mut decomp_rows = Vec::new();
    for (name, sel) in decomp_arms.iter() {
        let mut n = 0usize;
        let (mut rk, mut gt, mut ds, mut ac, mut lp) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
        let mut covered = 0usize;
        let mut pool_best_correct = 0usize;
        let mut not_ungated = 0usize;
        let mut strata: BTreeMap<&str, (usize, f64, f64, f64)> = BTreeMap::new();
        for (gi, os) in fresh_obs.iter().enumerate() {
            for o in os.iter() {
                let p = pos_of(o);
                let r = regret_decomposition(sel, &table, &p);
                n += 1;
                rk += r.ranking;
                gt += r.gate;
                ds += r.dose;
                ac += r.actual;
                lp += r.lpool;
                if o.cands.iter().any(|c| c.payload == o.target) {
                    covered += 1;
                }
                if best_is_correct(&p, o) {
                    pool_best_correct += 1;
                }
                if !r.served_matches_ungated {
                    not_ungated += 1;
                }
                let e = strata.entry(stratum_of(o, &fresh[gi])).or_default();
                e.0 += 1;
                e.1 += r.ranking;
                e.2 += r.gate;
                e.3 += r.dose;
            }
        }
        decomp_rows.push(json!({
            "arm": name,
            "positions": n,
            "mean_ranking_regret_nats": rk / n.max(1) as f64,
            "mean_gate_regret_nats": gt / n.max(1) as f64,
            "mean_dose_regret_nats": ds / n.max(1) as f64,
            "mean_total_regret_nats": (rk + gt + ds) / n.max(1) as f64,
            "mean_actual_nats": ac / n.max(1) as f64,
            "mean_lpool_nats": lp / n.max(1) as f64,
            "covered_positions": covered,
            "positions_where_pool_best_action_is_correct": pool_best_correct,
            "read_actions_where_served_source_differs_from_ungated_top": not_ungated,
            "strata": strata.iter().map(|(k, (nn, a, b, c))| json!({
                "stratum": k, "positions": nn,
                "mean_ranking_regret_nats": a / (*nn).max(1) as f64,
                "mean_gate_regret_nats": b / (*nn).max(1) as f64,
                "mean_dose_regret_nats": c / (*nn).max(1) as f64,
            })).collect::<Vec<_>>(),
        }));
    }
    let decomposition = json!({
        "identity": "ranking + gate + dose == actual - lpool, verified per position",
        "unit": "nats_per_candidate_bearing_position",
        "target_using": true,
        "lpool_caveat": "lpool sees only the admitted pool; it cannot diagnose a source excluded by admission",
        "arms": decomp_rows,
    });

    // ---- controls ---------------------------------------------------------------
    let mut controls = Vec::new();
    let mut same_path = true;
    for os in fresh_obs.iter().take(16) {
        for o in os.iter() {
            let (z1, r1) = predict_next(
                &o.ring,
                o.cur,
                o.prev as usize,
                o.prev2,
                &relational_ctx_reload,
                &table,
                &parent,
                &local,
                &u,
                true,
            );
            let rel = rels_for(&relational_ctx_reload, o, &table, false);
            let act = relational_ctx_reload.choose(&o.cands, &rel);
            let mut z2 = local_logits(
                &parent,
                &local,
                &u,
                o.cur,
                o.prev as usize,
                o.prev2,
                o.ring.written(),
            );
            if let Some((k, st)) = act {
                let p = o.cands[k].payload as usize;
                z2[p] = z2[p].saturating_add(1i32 << AMP_SHIFTS[st]);
            }
            if z1 != z2 || r1.action != act {
                same_path = false;
            }
        }
    }
    controls.push(json!({
        "control": "one_shared_inference_path",
        "arm": "relational_ctx (reloaded)",
        "positions": fresh_obs.iter().take(16).map(|v| v.len()).sum::<usize>(),
        "observed_prefix_scores_and_actions_identical": same_path,
        "note": "predict_next/read_step is compared against the independent score construction on identical stored prefix states; generation prefix advancement is separately covered by a runner unit test",
    }));
    let mut scheck_fail = 0usize;
    let mut scheck_n = 0usize;
    for os in fresh_obs.iter() {
        for o in os.iter() {
            let a = local_logits(
                &parent,
                &local,
                &u,
                o.cur,
                o.prev as usize,
                o.prev2,
                o.ring.written(),
            );
            let b = s_query_only(&o.ring, &parent, &local, &u, o.cur, o.prev as usize);
            scheck_n += 1;
            if a != b {
                scheck_fail += 1;
            }
        }
    }
    controls.push(json!({
        "control": "independent_s_query_only_check",
        "positions": scheck_n, "mismatching_positions": scheck_fail,
        "matches": scheck_fail == 0,
        "note": "rebuilt through the retained scorer's own int_logits((b, identity)) with the declared absence behaviour and the identity row removed",
    }));
    // Change x_cut and compare only queries x_i with i < cut: their observed prefixes agree.
    let mut causal = true;
    let mut compared = 0usize;
    let mut predecessors_compared = 0usize;
    for (gi, os) in fresh_obs.iter().enumerate() {
        if os.is_empty() {
            continue;
        }
        let cut = os[os.len() / 2].ring.written() as usize;
        let mut t2 = fresh[gi].tokens.clone();
        if cut >= t2.len() {
            continue;
        }
        t2[cut] = banks.values_held[(cut + 3) % banks.values_held.len()];
        let mut changed = fresh[gi].clone();
        changed.tokens = t2;
        let alt = observe(&changed);
        for o in os.iter() {
            let i = o.ring.written() as usize;
            if i >= cut {
                continue;
            }
            let Some(a1) = alt.iter().find(|x| x.ring.written() == o.ring.written()) else {
                causal = false;
                continue;
            };
            compared += 1;
            predecessors_compared += usize::from(i + 1 == cut);
            let (za, ra) = predict_next(
                &o.ring,
                o.cur,
                o.prev as usize,
                o.prev2,
                &relational_ctx_reload,
                &table,
                &parent,
                &local,
                &u,
                true,
            );
            let (zb, rb) = predict_next(
                &a1.ring,
                a1.cur,
                a1.prev as usize,
                a1.prev2,
                &relational_ctx_reload,
                &table,
                &parent,
                &local,
                &u,
                true,
            );
            if za != zb || ra != rb {
                causal = false;
            }
        }
    }
    controls.push(json!({
        "control": "future_token_intervention",
        "positions_compared": compared,
        "changed_token_predecessor_positions_compared": predecessors_compared,
        "unchanged_scores_action_reference_payload": causal,
        "scope": "candidate-bearing positions i < cut, including cut-1 when candidate-bearing; x_cut itself is excluded because it is changed input",
    }));
    // Selected-source intervention: change ONLY the payload of the occurrence the exported policy
    // actually selected, and only when that occurrence lies outside the fixed recent query context.
    // The query tokens and the admission conditions are preserved. Every attempt, lost support and
    // NoRead is recorded rather than dropped, with an unrelated-source control and read-disabled
    // behaviour on the same intervened prefix.
    let mut sel_attempted = 0usize;
    let mut sel_lost_support = 0usize;
    let mut sel_no_read_after = 0usize;
    let mut sel_selected_new = 0usize;
    let mut sel_emitted_new = 0usize;
    let mut emitted_old_before = 0usize;
    let mut emitted_new_after = 0usize;
    let mut read_disabled_changed = 0usize;
    let mut unrelated_attempted = 0usize;
    let mut unrelated_selected_new = 0usize;
    for (gi, os) in fresh_obs.iter().enumerate() {
        let Some(o) = os.last() else { continue };
        let rel = rels_for(&relational_ctx_reload, o, &table, false);
        let Some((k, _a)) = relational_ctx_reload.choose(&o.cands, &rel) else {
            continue;
        };
        let src = (o.cands[k].abs + 1) as usize;
        // Outside the fixed recent query context: the query triple and its predecessor are untouched.
        if src + 4 > o.ring.written() as usize || src >= fresh[gi].tokens.len() {
            continue;
        }
        let Some(alt) = banks.values_held.iter().find(|v| **v != o.cands[k].payload) else {
            continue;
        };
        sel_attempted += 1;
        let (zb, _) = predict_next(
            &o.ring,
            o.cur,
            o.prev as usize,
            o.prev2,
            &relational_ctx_reload,
            &table,
            &parent,
            &local,
            &u,
            true,
        );
        emitted_old_before += usize::from(argmax_low(&zb) as u32 == o.cands[k].payload);
        let mut t2 = fresh[gi].tokens.clone();
        t2[src] = *alt;
        let mut changed = fresh[gi].clone();
        changed.tokens = t2;
        let alt_obs = observe(&changed);
        let Some(o2) = alt_obs
            .iter()
            .find(|x| x.ring.written() == o.ring.written())
        else {
            sel_lost_support += 1;
            continue;
        };
        let (z2, r2) = predict_next(
            &o2.ring,
            o2.cur,
            o2.prev as usize,
            o2.prev2,
            &relational_ctx_reload,
            &table,
            &parent,
            &local,
            &u,
            true,
        );
        match r2.action {
            None => sel_no_read_after += 1,
            Some((kk, _)) => {
                if o2.cands[kk].payload == *alt {
                    sel_selected_new += 1;
                }
                if argmax_low(&z2) as u32 == *alt {
                    sel_emitted_new += 1;
                }
            }
        }
        emitted_new_after += usize::from(argmax_low(&z2) as u32 == *alt);
        let (zd, _) = predict_next(
            &o2.ring,
            o2.cur,
            o2.prev as usize,
            o2.prev2,
            &relational_ctx_reload,
            &table,
            &parent,
            &local,
            &u,
            false,
        );
        read_disabled_changed += usize::from(argmax_low(&zd) != argmax_low(&zb));
        // Unrelated-source control: mutate a different admitted occurrence, same conditions.
        if let Some(other) = o.cands.iter().position(|c| {
            (c.abs + 1) as usize + 4 <= o.ring.written() as usize && c.abs != o.cands[k].abs
        }) {
            let osrc = (o.cands[other].abs + 1) as usize;
            if osrc < fresh[gi].tokens.len() && fresh[gi].tokens[osrc] != *alt {
                let mut t3 = fresh[gi].tokens.clone();
                t3[osrc] = *alt;
                let mut ch3 = fresh[gi].clone();
                ch3.tokens = t3;
                let obs3 = observe(&ch3);
                if let Some(o3) = obs3.iter().find(|x| x.ring.written() == o.ring.written()) {
                    unrelated_attempted += 1;
                    let (_, r3) = predict_next(
                        &o3.ring,
                        o3.cur,
                        o3.prev as usize,
                        o3.prev2,
                        &relational_ctx_reload,
                        &table,
                        &parent,
                        &local,
                        &u,
                        true,
                    );
                    if r3.payload == Some(*alt) {
                        unrelated_selected_new += 1;
                    }
                }
            }
        }
    }
    controls.push(json!({
        "control": "selected_source_intervention",
        "arm": "relational_ctx (reloaded)",
        "attempted": sel_attempted,
        "lost_support_after_intervention": sel_lost_support,
        "no_read_after_intervention": sel_no_read_after,
        "selected_the_new_payload": sel_selected_new,
        "emitted_the_new_payload": sel_emitted_new,
        "emitted_the_old_payload_before": emitted_old_before,
        "emitted_the_new_payload_after": emitted_new_after,
        "read_disabled_emission_changed_on_the_same_prefix": read_disabled_changed,
        "unrelated_source_control_attempted": unrelated_attempted,
        "unrelated_source_control_selected_the_new_payload": unrelated_selected_new,
        "note": "changes only the payload of the occurrence the exported policy actually selected, outside the four-token query context; query tokens and admission conditions are unchanged; lost support and NoRead are recorded rather than dropped; the first target-bearing occurrence is never silently substituted",
    }));
    let mut ring = OccurrenceRing::new(RING_CAP);
    for t in [1u32, 2, 3] {
        ring.observe(t);
    }
    let r = ring.reference(1).ok_or("ref")?;
    ring.reset();
    controls.push(json!({
        "control": "stale_reference_after_reset",
        "resolved_before_reset": true,
        "resolved_after_reset": ring.resolve(r).is_some(),
    }));

    // ---- reader text: document-separated held-out evaluation plus a fit-document regression ---
    let text_arms: Vec<(&'static str, &RelationalSelector)> = vec![
        ("relational", &relational_reload),
        ("relational_ctx", &relational_ctx_reload),
        ("categorical_ctx", &categorical_ctx_reload),
    ];
    for (label, docs, scope) in [
        (
            "held_out_documents",
            &text_eval_docs,
            "Documents the new reader never fitted. The frozen local prior E was trained on this pinned corpus, so its older exposure is disclosed separately and is not removed; this is a reader-held-out local evaluation, not an externally unseen claim. Harmful NoRead and uncovered positions are not filtered out.",
        ),
        (
            "fit_documents_regression",
            &text_fit_docs,
            "The same documents used for the reader text fit: an in-sample regression row, never transfer evidence.",
        ),
    ] {
        let mut positions = 0usize;
        let mut local_sum = 0.0f64;
        let mut reader_sum = vec![0.0f64; text_arms.len()];
        let mut reads = vec![0usize; text_arms.len()];
        let mut emitted = vec![0usize; text_arms.len()];
        let mut per_doc: Vec<Vec<(f64, f64, usize)>> = vec![Vec::new(); text_arms.len()];
        for i in docs.iter() {
            let ws = windows_of(&tokenizer.encode(&uniq[*i].text));
            if ws.is_empty() {
                continue;
            }
            // Warm every arm over the window before measuring, so no arm pays a cold first pass.
            {
                let mut ring = OccurrenceRing::new(RING_CAP);
                for k in 0..ws[0].len() {
                    let cur = ws[0][k];
                    let prev = if k >= 1 {
                        ws[0][k - 1] as usize
                    } else {
                        parent.cfg.pad_row()
                    };
                    let prev2 = if k >= 2 { ws[0][k - 2] } else { NO_TOKEN };
                    if k >= 2 {
                        for (_n, s) in text_arms.iter() {
                            let _ = predict_next(
                                &ring, cur, prev, prev2, s, &table, &parent, &local, &u, true,
                            );
                        }
                    }
                    ring.observe(cur);
                }
            }
            let mut doc_local = 0.0f64;
            let mut doc_reader = vec![0.0f64; text_arms.len()];
            let mut doc_n = 0usize;
            let mut ring = OccurrenceRing::new(RING_CAP);
            for k in 0..ws[0].len() {
                let cur = ws[0][k];
                let prev = if k >= 1 {
                    ws[0][k - 1] as usize
                } else {
                    parent.cfg.pad_row()
                };
                let prev2 = if k >= 2 { ws[0][k - 2] } else { NO_TOKEN };
                if k >= 2 && k + 1 < ws[0].len() {
                    let target = ws[0][k + 1];
                    let (zl, _) = predict_next(
                        &ring, cur, prev, prev2, &relational_reload, &table, &parent, &local, &u,
                        false,
                    );
                    doc_local += bits(&zl, target, parent.cfg.f_bits);
                    doc_n += 1;
                    for (ai, (_n, s)) in text_arms.iter().enumerate() {
                        let (zr, r) = predict_next(
                            &ring, cur, prev, prev2, s, &table, &parent, &local, &u, true,
                        );
                        doc_reader[ai] += bits(&zr, target, parent.cfg.f_bits);
                        if r.action.is_some() {
                            reads[ai] += 1;
                        }
                        if argmax_low(&zr) as u32 == target {
                            emitted[ai] += 1;
                        }
                    }
                }
                ring.observe(cur);
            }
            positions += doc_n;
            local_sum += doc_local;
            for ai in 0..text_arms.len() {
                reader_sum[ai] += doc_reader[ai];
                if doc_n > 0 {
                    per_doc[ai].push((doc_reader[ai], doc_local, doc_n));
                }
            }
        }
        for (ai, (name, _s)) in text_arms.iter().enumerate() {
            let interval = boot_diff(&per_doc[ai], 0x7E47_0000 + ai as u64);
            text_rows.push(json!({
                "panel": label,
                "arm": name,
                "documents": docs.len(),
                "positions": positions,
                "local_bits_per_token": local_sum / positions.max(1) as f64,
                "reader_bits_per_token": reader_sum[ai] / positions.max(1) as f64,
                "delta_bits": (reader_sum[ai] - local_sum) / positions.max(1) as f64,
                "reader_reads": reads[ai],
                "reader_emitted_correct": emitted[ai],
                "document_cluster_interval": interval,
                "scope": scope,
            }));
        }
    }
    mark("controls + text", &mut marks);

    // ---- generation through the shared path -------------------------------------
    let mut gen_rows = Vec::new();
    let gen_arms: Vec<(&'static str, &RelationalSelector, bool)> = vec![
        ("local", &relational_reload, false),
        ("relational", &relational_reload, true),
        ("relational_ctx", &relational_ctx_reload, true),
        ("categorical_ctx", &categorical_ctx_reload, true),
    ];
    for (k, pr) in LEGACY_PROMPTS.iter().enumerate() {
        let mut out = Vec::new();
        for (name, sel, use_reader) in gen_arms.iter() {
            // The query token is supplied separately, exactly as in teacher-forced evaluation.
            let mut ring = ring_before_current(pr);
            let mut toks: Vec<u32> = pr.to_vec();
            let start = toks.len();
            for _ in 0..GEN_TOKENS {
                let _ = generate_step(
                    &mut ring,
                    &mut toks,
                    sel,
                    &table,
                    &parent,
                    &local,
                    &u,
                    *use_reader,
                )?;
            }
            let tail = &toks[start..];
            let mut seen: Vec<u32> = Vec::new();
            let mut repeats = 0usize;
            for t in tail.iter() {
                if seen.last() == Some(t) {
                    repeats += 1;
                }
                seen.push(*t);
            }
            out.push(json!({
                "arm": name, "tokens": tail.to_vec(),
                "decoded": tokenizer.decode(tail),
                "prompt_index": k,
                "adjacent_repeat_tokens": repeats,
            }));
        }
        gen_rows.push(json!({"prompt_index": k, "arms": out}));
    }

    // ---- cost: honest active-candidate path plus a genuine no-reader-work baseline ------
    // The text window below admits no candidates (the parent run's probe measured 0 over 29
    // predictions), so it is only a local-path baseline. The active-candidate measurement uses a
    // real constructed stream, where admission is non-empty and reads actually occur.
    let text_probe_ws = windows_of(&tokenizer.encode(&uniq[text_eval_docs[0]].text));
    let text_probe: Vec<u32> = text_probe_ws[0][..32.min(text_probe_ws[0].len())].to_vec();
    let active_probe_stream: Vec<u32> = fresh[0].tokens.clone();
    let mut timings = Vec::new();
    for (label, sel, stream, use_reader, protocol) in [
        (
            "local_no_reader_work_text_window",
            &relational_reload,
            &text_probe,
            false,
            "32-token real text window; the local arm builds no ring, performs no admission or feature work and emits from the local baseline alone",
        ),
        (
            "local_no_reader_work_construction_stream",
            &relational_reload,
            &active_probe_stream,
            false,
            "the SAME constructed stream as the reader arms; matched no-reader-work baseline, so the reader increment is the difference between this row and the reader rows",
        ),
        (
            "reader_global_active_candidates",
            &relational_reload,
            &active_probe_stream,
            true,
            "a real constructed stream with non-empty admission; full shared path including admission, bucket, source score and table reads",
        ),
        (
            "reader_contextual_active_candidates",
            &relational_ctx_reload,
            &active_probe_stream,
            true,
            "the reloaded contextual arm on the same stream; the interaction adds one table read per candidate",
        ),
        (
            "reader_categorical_ctx_active_candidates",
            &categorical_ctx_reload,
            &active_probe_stream,
            true,
            "the reloaded matched categorical arm on the same stream",
        ),
    ] {
        let _ = active_probe(&parent, &local, &u, sel, &table, stream, use_reader);
        let mut s = Vec::new();
        for _ in 0..5 {
            let t0 = Instant::now();
            let out = active_probe(&parent, &local, &u, sel, &table, stream, use_reader);
            black_box(&out);
            s.push(t0.elapsed().as_secs_f64());
        }
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (_, predictions, admitted, reads) =
            active_probe(&parent, &local, &u, sel, &table, stream, use_reader);
        timings.push(json!({
            "label": label,
            "predictions": predictions,
            "median_s": s[s.len() / 2],
            "per_prediction_us": s[s.len() / 2] / predictions.max(1) as f64 * 1e6,
            "candidates_admitted_total": admitted,
            "candidates_admitted_per_prediction": admitted as f64 / predictions.max(1) as f64,
            "read_actions": reads,
            "protocol": protocol,
        }));
    }
    let mut serialized = serde_json::Map::new();
    serialized.insert("parent_E".into(), json!(pb.len()));
    serialized.insert("local_query_artifact".into(), json!(sb.len()));
    for (k, v) in export_bytes.iter() {
        serialized.insert((*k).to_string(), json!(v.len()));
    }
    let resident_and_serialized = json!({
        "timings": timings,
        "serialized_bytes": serde_json::Value::Object(serialized),
        "resident_bytes": {
            "local_row_table": 120 * parent.cfg.vocab * 4,
            "parent_scratch": parent.cfg.vocab * 4,
            "descriptor_roots_per_arm": relational.q_roots.len(),
            "rank_entries_per_arm": RANKS * 4,
            "contextual_table_entries_per_ctx_arm": N_CTX_BUCKETS * (ACTS + 1),
            "contextual_table_vec_capacity_caveat": "16 [i32;4] rows are 256 resident entry bytes plus Vec metadata/capacity; the serialized payload is 64 bytes",
        },
        "energy": "UNAVAILABLE",
    });
    mark("generation + cost", &mut marks);

    // ---- decision ---------------------------------------------------------------
    let fresh_panel = panel
        .iter()
        .find(|p| p["panel"] == "fresh")
        .ok_or("fresh")?;
    let rows = fresh_panel["arms"].as_array().ok_or("arms")?;
    let get =
        |n: &str| -> &serde_json::Value { rows.iter().find(|r| r["arm"] == n).unwrap_or(&rows[0]) };
    let hard_point = hard_diff["point"].as_f64().unwrap_or(f64::NAN);
    let hard_hi = hard_diff["hi"].as_f64().unwrap_or(f64::NAN);
    let text_delta = text_rows[0]["delta_bits"].as_f64().unwrap_or(f64::NAN);
    let instrument_ok = same_path
        && scheck_fail == 0
        && causal
        && controls
            .iter()
            .find(|c| c["control"] == "stale_reference_after_reset")
            .is_some_and(|c| c["resolved_after_reset"] == json!(false))
        && arm_parity_failures == 0;
    // Primary: a meaningfully negative hard-loss difference (relational better) with the WHOLE
    // interval below zero, i.e. the upper bound negative.
    let positive = hard_point <= -MARGIN_HARD_BITS
        && hard_hi < 0.0
        && instrument_ok
        && text_delta <= MARGIN_TEXT_BITS;

    // ---- prospective endpoints, frozen in the design note before this draw --------------------
    let text_row = |panel: &str, arm: &str| -> Option<&serde_json::Value> {
        text_rows
            .iter()
            .find(|r| r["panel"] == panel && r["arm"] == arm)
    };
    let text_delta = |panel: &str, arm: &str| -> f64 {
        text_row(panel, arm)
            .and_then(|r| r["delta_bits"].as_f64())
            .unwrap_or(f64::NAN)
    };
    let held_out_rel = text_delta("held_out_documents", "relational");
    let held_out_ctx = text_delta("held_out_documents", "relational_ctx");
    let held_out_cat_ctx = text_delta("held_out_documents", "categorical_ctx");
    let read_rate = |n: &str| -> f64 { 1.0 - get(n)["no_read_rate"].as_f64().unwrap_or(f64::NAN) };
    let ctx_point = ctx_diff["point"].as_f64().unwrap_or(f64::NAN);
    let ctx_hi = ctx_diff["hi"].as_f64().unwrap_or(f64::NAN);
    let present_point = ctx_diff_present["point"].as_f64().unwrap_or(f64::NAN);
    let present_hi = ctx_diff_present["hi"].as_f64().unwrap_or(f64::NAN);
    let absent_point = ctx_diff_absent["point"].as_f64().unwrap_or(f64::NAN);
    let cat_ctx_minus_rel_ctx = cat_ctx_vs_rel_ctx["point"].as_f64().unwrap_or(f64::NAN);
    let cat_ctx_lo = cat_ctx_vs_rel_ctx["lo"].as_f64().unwrap_or(f64::NAN);
    let cat_ctx_hi = cat_ctx_vs_rel_ctx["hi"].as_f64().unwrap_or(f64::NAN);
    let ctx_keeps_reading = read_rate("relational_ctx") >= 0.5 * read_rate("relational");
    let emitted_ok_margin = get("relational_ctx")["emitted_accuracy"]
        .as_f64()
        .unwrap_or(f64::NAN)
        >= get("relational")["emitted_accuracy"]
            .as_f64()
            .unwrap_or(f64::NAN)
            - 0.02;
    let lang_ok = held_out_ctx <= held_out_rel + MARGIN_TEXT_BITS;
    let ctx_instrument_ok = instrument_ok && reload_matches_ctx && arm_parity_failures == 0;
    // Retained component decision, exactly as preregistered in the parent and re-measured here.
    let ctx_positive = ctx_point <= -MARGIN_CTX_BITS
        && ctx_hi < 0.0
        && ctx_instrument_ok
        && ctx_keeps_reading
        && lang_ok
        && emitted_ok_margin;
    // The matched comparison that would license a unique-geometric claim: the categorical arm gets
    // the same contextual procedure, so an interval excluding zero in the geometric arm's favour is
    // required. Reported as its own endpoint, never inferred from the lesion.
    let unique_geometric_benefit = cat_ctx_minus_rel_ctx > 0.0 && cat_ctx_lo > 0.0;
    // An all-NoRead collapse cannot count as attention progress.
    let all_noread_collapse = read_rate("relational_ctx") <= 0.02;

    write_json(
        &root,
        "result.json",
        &json!({
            "schema": "uor-r4.competitive-reader/4",
            "base_revision": "472767dc",
            "running_source": {
                "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
                "git_dirty": std::env::var("UOR_GIT_DIRTY").unwrap_or_else(|_| "unknown".into()),
                "executable_sha256": executable_sha256,
                "source_file_sha256": sources.clone(),
            },
            "inputs": {"E": E_SHA, "local_query_artifact": S_SHA, "tokenizer": {"source": TOKENIZER_SHA, "derived": derived}},
            "baseline": "z_local(v) = z_E(v) + u_S(b) for i >= 2, else z_E, through one shared function",
            "mechanism": {
                "shared_path": "read_step + predict_next; no target, sentinel or supervised record enters them",
                "descriptor_initialisation": "palette.elements[a_codes[t]]: the actual S write element",
                "relation": "inverse(q)*k through the single relation_index function used by fit, inference, export and reload",
                "artifact": "RLR2 v3 carries the mode, the code map and the contextual interaction; an independent loader is exercised for every arm and the reloaded selectors drive evaluation, generation, interventions and timing",
                "strengths": {"amp_shifts": AMP_SHIFTS.to_vec()},
                "ring_cap": RING_CAP, "max_candidates": MAX_CAND,
                "arm_reload_parity_failures": arm_parity_failures,
                "reloaded_contextual_equals_fitted": reload_matches_ctx,
                "contextual_interaction": "ctx[bucket][slot], slot 0 = NoRead offset, slots 1..=ACTS = strength offsets; signed 4-bit integers added by the same integer kernel as sb[a]; bucket frozen from the ctx-free source scores and the candidate exact-context class",
                "optimisation": "bounded alternating scalar fit, descriptor refinement, scalar refit, descriptor refinement, with the exported development decision quality selecting the phase; identical schedule and selection for every learned arm",
                "decomposition": "ranking + gate + dose == actual - lpool per position",
            },
            "fit": arm_report,
            "contextual_fit": ctx_report,
            "panels": panel,
            "hard_action_ce_relational_minus_exact_fresh": hard_diff,
            "hard_action_ce_contextual_minus_global_fresh": ctx_diff,
            "hard_action_ce_contextual_minus_global_final_present": ctx_diff_present,
            "hard_action_ce_contextual_minus_global_final_absent": ctx_diff_absent,
            "hard_action_ce_categorical_ctx_minus_relational_ctx": cat_ctx_vs_rel_ctx,
            "hard_action_ce_categorical_ctx_minus_relational_ctx_final_present": cat_ctx_vs_rel_ctx_present,
            "hard_action_ce_categorical_ctx_minus_categorical": cat_ctx_vs_cat,
            "hard_action_ce_relational_ctx_minus_lesion": rel_ctx_vs_lesion,
            "regret_decomposition": decomposition,
            "natural_text": text_rows,
            "text_split": text_split_report.clone(),
            "controls": controls,
            "generation": gen_rows,
            "cost": resident_and_serialized,
            "decision": {
                "primary": "paired-by-sequence hard-action CE difference, contextual minus frozen global-strength baseline, fresh construction",
                "endpoints": {
                    "all_positions": ctx_diff,
                    "final_present_queries": ctx_diff_present,
                    "final_absent_queries": ctx_diff_absent,
                    "matched_categorical_ctx_minus_relational_ctx": cat_ctx_vs_rel_ctx,
                    "matched_categorical_ctx_minus_relational_ctx_final_present": cat_ctx_vs_rel_ctx_present,
                    "relation_channel_lesion_of_the_contextual_arm": rel_ctx_vs_lesion,
                    "relational_minus_exact": hard_diff,
                },
                "counts": {
                    "relational_ctx_correct_emitted_next_tokens_all_positions": get("relational_ctx")["emitted_next_token_correct"].clone(),
                    "relational_correct_emitted_next_tokens_all_positions": get("relational")["emitted_next_token_correct"].clone(),
                    "categorical_ctx_correct_emitted_next_tokens_all_positions": get("categorical_ctx")["emitted_next_token_correct"].clone(),
                    "relational_ctx_correct_payload_reads_all_positions": get("relational_ctx")["correct_reads"].clone(),
                    "relational_correct_payload_reads_all_positions": get("relational")["correct_reads"].clone(),
                },
                "text": {
                    "held_out_relational_bits_per_token": held_out_rel,
                    "held_out_relational_ctx_bits_per_token": held_out_ctx,
                    "held_out_categorical_ctx_bits_per_token": held_out_cat_ctx,
                    "held_out_document_cluster_interval_relational_ctx": text_row("held_out_documents", "relational_ctx").map(|r| r["document_cluster_interval"].clone()),
                },
                "margin_bits": MARGIN_CTX_BITS,
                "text_margin_bits": MARGIN_TEXT_BITS,
                "instrument_checks_pass": instrument_ok,
                "contextual_instrument_checks_pass": ctx_instrument_ok,
                "contextual_keeps_reading": ctx_keeps_reading,
                "contextual_language_tradeoff_ok": lang_ok,
                "contextual_emitted_margin_ok": emitted_ok_margin,
                "all_noread_collapse": all_noread_collapse,
                "retained_component_positive": ctx_positive,
                "historical_relational_vs_exact_positive": positive,
                "unique_geometric_benefit_over_matched_categorical": unique_geometric_benefit,
                "historical": "the parent positive=true for the strength comparison, the old 0.15 precision margin, the -2.4956 bits/sequence figure and the old positive=false remain historical at their own scope and are not re-applied",
            },
            "phases": marks.iter().fold((0.0f64, Vec::new()), |(prev, mut out), (n, t)| {
                out.push(json!({"phase": n, "seconds": t - prev, "cumulative_s": t}));
                (*t, out)
            }).1,
            "elapsed_s": started.elapsed().as_secs_f64(),
        }),
    )?;
    // ---- cryptographically bound manifest: complete inputs, splits, versions and seeds ------
    let full_binding = binding_digest(&fit_obs, &text_obs, &tune_obs, &fresh_obs);
    write_json(
        &root,
        "binding.json",
        &json!({
            "schema": "uor-r4.relational-learning-binding/1",
            "base_revision": "472767dc",
            "inputs": {
                "E_sha256": E_SHA,
                "local_query_artifact_sha256": S_SHA,
                "tokenizer_source_sha256": TOKENIZER_SHA,
                "tokenizer_derived_sha256": DERIVED_SHA,
                "group_table": "ExactGroupTable::build()",
                "parent_digest": hex_of(&parent_digest),
            },
            "observations": {
                "full_binding_digest": hex_of(&full_binding),
                "fit_sequences": fit.len(),
                "fit_candidate_positions": constructed_fit_positions,
                "text_fit_documents": text_fit_docs.len(),
                "text_fit_candidate_positions": text_fit_n,
                "text_fit_tokens": text_fit_tokens,
                "tune_sequences": tune.len(),
                "tune_candidate_positions": tune_n,
                "fresh_sequences": fresh.len(),
                "fresh_candidate_positions": fresh_n,
                "scope": "candidate payloads, references, exact-context features and targets for every fitted or scored position, including the natural-text fit documents",
            },
            "text_split": text_split_report.clone(),
            "configuration": {
                "ring_cap": RING_CAP, "max_candidates": MAX_CAND,
                "act_shifts": AMP_SHIFTS.to_vec(),
                "steps": STEPS, "steps_refit": STEPS_REFIT, "batch": BATCH, "lr": LR,
                "descriptor_rounds": ROUNDS, "ctx_rounds": CTX_ROUNDS, "ctx_buckets": N_CTX_BUCKETS,
                "seeds": {"fit": SEED_FIT, "tune": SEED_TUNE, "fresh": SEED_FRESH},
                "config_identity": hex_of(&cfg_id),
                "optimisation": "scalar fit, descriptor refinement, scalar refit, descriptor refinement; phase selected by exported development decision quality",
                "selection_data": "SEED_TUNE construction on the fit payload bank; disjoint from the fit and fresh draws",
            },
            "export": {
                "format": "RLR2 v3",
                "artifacts": export_bytes.keys().collect::<Vec<_>>(),
                "semantics": "signed 4-bit selector entries, integer add/compare/table-read only; ctx slot 0 is the NoRead offset",
            },
            "source_files": sources,
            "limitations": [
                "The frozen local prior E was trained on this pinned corpus; the document split separates the new reader's fit from its held-out reader text but does not remove that historical exposure.",
                "The soft training objective is a surrogate; the exported integer decision quality is reported separately.",
                "Physical energy is UNAVAILABLE without measurement.",
            ],
        }),
    )?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "fresh all-pos: ctx {:.6} vs global {:.6} vs exact {:.6} vs local {:.6}; ctx-global {:?} [{:?},{:?}]; present {:?} [{:?},{:?}]; absent {:?}; cat_ctx-rel_ctx {:?} [{:?},{:?}]; heldout text global {:.4} ctx {:.4} cat_ctx {:.4}; reading {}; unique_geom {}; ctx_ok {ctx_positive}; instrument {ctx_instrument_ok}; sealed {} unlisted; elapsed {:.1}s",
        get("relational_ctx")["hard_action_ce_bits"].as_f64().unwrap_or(f64::NAN),
        get("relational")["hard_action_ce_bits"].as_f64().unwrap_or(f64::NAN),
        get("exact")["hard_action_ce_bits"].as_f64().unwrap_or(f64::NAN),
        get("local")["hard_action_ce_bits"].as_f64().unwrap_or(f64::NAN),
        ctx_point,
        ctx_diff["lo"].as_f64().unwrap_or(f64::NAN),
        ctx_hi,
        present_point,
        ctx_diff_present["lo"].as_f64().unwrap_or(f64::NAN),
        present_hi,
        absent_point,
        cat_ctx_minus_rel_ctx,
        cat_ctx_lo,
        cat_ctx_hi,
        held_out_rel,
        held_out_ctx,
        held_out_cat_ctx,
        read_rate("relational_ctx"),
        unique_geometric_benefit,
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

fn template_obs(obs: &[Vec<Obs>]) -> Obs {
    let o = &obs[0][0];
    Obs {
        ring: o.ring.clone(),
        cur: o.cur,
        prev: o.prev,
        prev2: o.prev2,
        target: o.target,
        cands: o.cands.clone(),
        group: o.group,
    }
}

/// Exercise the independent artifact loader for one arm on inputs where the encodings differ.
fn check_reload(sel: &RelationalSelector, table: &ExactGroupTable, o: &Obs) -> Result<(), String> {
    let art = RelationalArtifact {
        selector: sel.clone(),
        local_artifact_digest: [1u8; 32],
        tokenizer_digest: [2u8; 32],
        data_digest: [3u8; 32],
    };
    let back = RelationalArtifact::from_bytes(&art.to_bytes(), &[1u8; 32], &[2u8; 32])?;
    if back.selector != *sel {
        return Err("reload changed the selector".into());
    }
    if back.selector.mode != sel.mode || back.selector.code_of != sel.code_of {
        return Err("reload changed the relation encoding".into());
    }
    if back.selector.rels_of(table, &dummy_pos_for(o)) != sel.rels_of(table, &dummy_pos_for(o)) {
        return Err("reload changed the relation indices".into());
    }
    Ok(())
}

fn dummy_pos_for(o: &Obs) -> TrainPos {
    TrainPos {
        cands: o.cands.clone(),
        q_role: o.prev as usize,
        k_role: o
            .cands
            .iter()
            .map(|c| {
                if c.x_prev == NO_TOKEN {
                    usize::MAX
                } else {
                    c.x_prev as usize
                }
            })
            .collect(),
        delta: o.cands.iter().map(|_| [0.0; ACTS]).collect(),
        group: o.group,
    }
}

/// The separately supplied current token is not duplicated inside the memory prefix.
fn ring_before_current(prefix: &[u32]) -> OccurrenceRing {
    let mut ring = OccurrenceRing::new(RING_CAP);
    for &token in prefix.iter().take(prefix.len().saturating_sub(1)) {
        ring.observe(token);
    }
    ring
}

#[allow(dead_code)]
fn prediction_count(tokens: usize) -> usize {
    // Query positions 2..tokens-1 each have the two-token context and a next-token target.
    tokens.saturating_sub(3)
}

/// Autoregressive caller: predict from the observed prefix, then advance that prefix exactly once.
#[allow(clippy::too_many_arguments)]
fn generate_step(
    ring: &mut OccurrenceRing,
    tokens: &mut Vec<u32>,
    sel: &RelationalSelector,
    table: &ExactGroupTable,
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    use_reader: bool,
) -> Result<(Vec<i32>, Read), String> {
    let i = tokens
        .len()
        .checked_sub(1)
        .ok_or("empty generation prefix")?;
    if i < 2 || ring.written() as usize != i {
        return Err(
            "generation requires two predecessors and a ring excluding the current token".into(),
        );
    }
    let result = predict_next(
        ring,
        tokens[i],
        tokens[i - 1] as usize,
        tokens[i - 2],
        sel,
        table,
        parent,
        local,
        u,
        use_reader,
    );
    let next = argmax_low(&result.0) as u32;
    ring.observe(tokens[i]);
    tokens.push(next);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn banks() -> Banks {
        Banks {
            pairs: (0..14).map(|i| (10 + i, 30 + i)).collect(),
            keys: (60..84).collect(),
            values_fit: (100..124).collect(),
            values_held: (130..142).collect(),
        }
    }

    #[test]
    fn constructed_sources_are_partners_and_absence_is_real() {
        let b = banks();
        let mut seed = 271u64;
        for absent in [false, true] {
            for _ in 0..80 {
                let seq = make_seq(&b, &mut seed, &b.values_held, absent);
                assert_eq!(validate_fixture(&seq, &b), Ok(()));
                let final_obs = observe(&seq)
                    .pop()
                    .expect("final query has competing sources");
                assert_eq!(final_obs.ring.written() as usize, seq.tokens.len() - 2);
                assert_eq!(
                    final_obs.cands.iter().any(|c| c.payload == seq.answer),
                    !absent
                );
            }
        }
    }

    #[test]
    fn generation_advance_and_observation_share_the_same_prefix_boundary() {
        let mut tokens = vec![11, 61, 131, 31, 61];
        let mut ring = ring_before_current(&tokens);
        for next in [132, 11, 61, 131] {
            let i = tokens.len() - 1;
            let expected = ring_before_current(&tokens);
            assert_eq!(ring.written(), i as u32);
            let got = admit_mixed(
                &ring,
                ring.written(),
                tokens[i],
                tokens[i - 1],
                tokens[i - 2],
                MAX_CAND,
            );
            let want = admit_mixed(
                &expected,
                expected.written(),
                tokens[i],
                tokens[i - 1],
                tokens[i - 2],
                MAX_CAND,
            );
            assert_eq!(got.0, want.0);
            assert_eq!(
                ring.get(i as u32),
                None,
                "current token is supplied separately"
            );
            ring.observe(tokens[i]);
            tokens.push(next);
        }
    }

    #[test]
    fn the_contextual_interaction_moves_the_served_strength_and_can_suppress_a_read() {
        let b = banks();
        let mut seed = 99u64;
        let seq = make_seq(&b, &mut seed, &b.values_held, false);
        let obs = observe(&seq);
        let o = obs.last().expect("final query has competing sources");
        let rel: Vec<usize> = vec![0; o.cands.len()];
        let base = RelationalSelector {
            q_roots: vec![0u8; 8],
            mode: RelMode::Geometric,
            code_of: Vec::new(),
            w: [0; EXACT_FEATS],
            rank: [0; RANKS],
            bias: 0,
            sb: [-7, 3, 6],
            noread: -7,
            ctx: Vec::new(),
        };
        let baseline = base.choose(&o.cands, &rel).expect("a read is affordable");
        assert_eq!(
            baseline,
            (0, 2),
            "the factorised arm takes the strongest action"
        );
        // Move every bucket away from the strongest action: same source, a cheaper strength. This
        // is exactly the interaction the controller exists to express.
        let mut softened = base.clone();
        let mut ctx = vec![[0i32; ACTS + 1]; N_CTX_BUCKETS];
        for row in ctx.iter_mut() {
            row[ACTS] = -7;
        }
        softened.ctx = ctx;
        let after = softened
            .choose(&o.cands, &rel)
            .expect("a weaker read is still affordable");
        assert_eq!(
            after.0, baseline.0,
            "the selected source does not change here"
        );
        assert!(after.1 < baseline.1, "the interaction lowers the strength");
        // Rewarding NoRead suppresses the read without touching the ranking.
        let mut quiet = base.clone();
        let mut ctx = vec![[0i32; ACTS + 1]; N_CTX_BUCKETS];
        for row in ctx.iter_mut() {
            row[0] = 7;
            for slot in row.iter_mut().skip(1) {
                *slot = -7;
            }
        }
        quiet.ctx = ctx;
        assert_eq!(
            quiet.choose(&o.cands, &rel),
            None,
            "NoRead is inside the action set"
        );
        assert_eq!(
            quiet.bucket_of(&o.cands, &rel),
            base.bucket_of(&o.cands, &rel),
            "the bucket key is unchanged by the interaction it keys"
        );
    }

    #[test]
    fn the_regret_identity_holds_on_real_constructed_observations() {
        let b = banks();
        let table = ExactGroupTable::build().expect("exact table");
        let mut seed = 4242u64;
        let sel = RelationalSelector {
            q_roots: vec![3u8; 256],
            mode: RelMode::Geometric,
            code_of: Vec::new(),
            w: [1, 0, 2, 0, -1, 3],
            rank: std::array::from_fn(|k| (k as i32 % 7) - 3),
            bias: 1,
            sb: [2, -1, 3],
            noread: 0,
            ctx: Vec::new(),
        };
        let mut checked = 0usize;
        let mut checked_absent = 0usize;
        for absent in [false, true] {
            for _ in 0..12 {
                let seq = make_seq(&b, &mut seed, &b.values_held, absent);
                for o in observe(&seq).iter() {
                    let delta: Vec<[f64; ACTS]> = o
                        .cands
                        .iter()
                        .enumerate()
                        .map(|(k, c)| {
                            let p = 0.002 * (k as f64 + 1.0);
                            let correct = c.payload == o.target;
                            std::array::from_fn(|a| {
                                let boost = (1i64 << AMP_SHIFTS[a]) as f64 / 1024.0;
                                action_loss(p, boost, correct)
                            })
                        })
                        .collect();
                    let p = TrainPos {
                        cands: cand_list(&o.cands),
                        q_role: o.prev as usize,
                        k_role: o
                            .cands
                            .iter()
                            .map(|c| {
                                if c.x_prev == NO_TOKEN {
                                    usize::MAX
                                } else {
                                    c.x_prev as usize
                                }
                            })
                            .collect(),
                        delta,
                        group: o.group,
                    };
                    let r = regret_decomposition(&sel, &table, &p);
                    assert!(
                        (r.ranking + r.gate + r.dose - (r.actual - r.lpool)).abs() < 1e-9,
                        "the decomposition must be exact on observed positions"
                    );
                    assert!(r.ranking >= -1e-12 && r.gate >= -1e-12 && r.dose >= -1e-12);
                    assert!(
                        r.served_matches_ungated,
                        "the served source must be the ungated top source"
                    );
                    checked += 1;
                    if absent {
                        checked_absent += 1;
                    }
                }
            }
        }
        assert!(checked > 0 && checked_absent > 0);
    }

    #[test]
    fn bootstrap_uses_token_ratio_with_sequence_resampling() {
        let mut groups = Vec::new();
        for _ in 0..4 {
            groups.push((1.0, 0.0, 1));
            groups.push((0.0, 0.0, 9));
        }
        let estimate = boot_diff(&groups, 177);
        assert_eq!(estimate["positions"], 40);
        assert_eq!(estimate["groups"], 8);
        assert!((estimate["point"].as_f64().unwrap() - 0.1).abs() < 1e-12);
        assert_eq!(estimate["resampling_unit"], "sequence");
        assert_eq!(prediction_count(32), 29);
        assert_eq!(prediction_count(2), 0);
    }

    #[test]
    #[ignore = "requires the pinned local E/S and PR1321 reader artifacts; explicitly run as a bounded smoke check"]
    fn retained_artifact_generation_matches_teacher_forced_prefixes() -> Result<(), String> {
        let pb = std::fs::read(E_PATH).map_err(|e| e.to_string())?;
        let sb = std::fs::read(S_PATH).map_err(|e| e.to_string())?;
        if sha256_hex(&pb) != E_SHA || sha256_hex(&sb) != S_SHA {
            return Err("pinned smoke-check parent hash mismatch".into());
        }
        let parent = PriorCore::from_bytes(&pb)?;
        let (digest, _) = parent_hash_convention(&pb);
        let tokenizer: [u8; 32] = hex_to_bytes(DERIVED_SHA)?
            .try_into()
            .map_err(|_| "tokenizer digest length")?;
        let local = QueryHard::from_bytes(&sb, &parent, &digest, &tokenizer)?;
        let bytes = std::fs::read(Path::new(DEFAULT_ROOT).join("artifacts/relational_reader.rlr2"))
            .map_err(|e| e.to_string())?;
        let reader =
            RelationalArtifact::from_bytes(&bytes, &sha256_bytes(&sb), &tokenizer)?.selector;
        reader.validate_for_vocab(parent.cfg.vocab)?;
        let table = ExactGroupTable::build()?;
        let u: Vec<Vec<i32>> = (0..120).map(|state| local.row_scores(state)).collect();
        for use_reader in [false, true] {
            let mut tokens = LEGACY_PROMPTS[3].to_vec();
            let mut ring = ring_before_current(&tokens);
            for _ in 0..4 {
                let query_abs = tokens.len() - 1;
                let mut supervised = tokens.clone();
                supervised.push(0); // target for observe only; never passed into inference.
                let seq = Seq {
                    tokens: supervised,
                    group: 0,
                    answer: 0,
                    absent: false,
                    answer_source_abs: None,
                };
                let observations = observe(&seq);
                let o = observations
                    .iter()
                    .find(|o| o.ring.written() as usize == query_abs)
                    .ok_or("smoke prefix unexpectedly has no admitted candidates")?;
                let teacher = predict_next(
                    &o.ring,
                    o.cur,
                    o.prev as usize,
                    o.prev2,
                    &reader,
                    &table,
                    &parent,
                    &local,
                    &u,
                    use_reader,
                );
                let generated = generate_step(
                    &mut ring,
                    &mut tokens,
                    &reader,
                    &table,
                    &parent,
                    &local,
                    &u,
                    use_reader,
                )?;
                assert_eq!(
                    generated, teacher,
                    "real generation and teacher-forced callers disagree"
                );
                assert_eq!(tokens.last().copied(), Some(argmax_low(&teacher.0) as u32));
                println!(
                    "retained-prefix-smoke {}",
                    json!({
                        "reader_enabled": use_reader, "query_abs": query_abs,
                        "emitted": tokens.last(), "source": generated.1.source.map(|r| json!({"seq": r.seq, "abs": r.abs})),
                        "payload": generated.1.payload,
                    })
                );
            }
        }
        Ok(())
    }
}
