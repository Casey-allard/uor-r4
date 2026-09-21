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

use uor_r4_core::native_geometric::learner::contextual_emission::*;
use uor_r4_core::native_geometric::learner::group_table::{group_table, GROUP_ORDER};
use uor_r4_core::native_geometric::learner::occurrence::{OccurrenceRef, OccurrenceRing};
use uor_r4_core::native_geometric::learner::policy_feasibility::*;
use uor_r4_core::native_geometric::learner::prefix_artifact::{
    parent_hash_convention, ExactGroupTable,
};
use uor_r4_core::native_geometric::learner::prior_learning::PriorCore;
use uor_r4_core::native_geometric::learner::query_read::QueryHard;
use uor_r4_core::native_geometric::learner::read_conditioned::*;
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
/// Documents per side of the document-separated reader text split (fit / tune / final).
const TEXT_DOCS: usize = 8;
/// Frozen parent artifacts: the improved relation learners this run only reads.
const PARENT_ROOT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/relational-learning-4";
const DEFAULT_UTIL_ROOT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/reader-utility-3";
const PARENT_SHA_EXACT: &str = "4795042636d57a360d7396c2f2939b79b1a28173c7228896a620cf53285a860d";
const PARENT_SHA_RELATIONAL: &str =
    "8d33a48888af8d224533b3634a52a868313df41364ec2dde6e6f8da7bd3043d7";
const PARENT_SHA_RELATIONAL_CTX: &str =
    "d4ab183d3975be563139ba42f3a5fb9ec61fa1691603bfcc3d603787d7ab5b55";
const PARENT_SHA_CATEGORICAL: &str =
    "2a60d9376702126060d3f6c86a0358bde723b45dd419b4bb50dd8d6ee04296a3";
/// New frozen construction seed for final assessment (declared in the design note).
const SEED_FINAL: u64 = 0x5C0F_F1A1;
/// The **new** frozen seed for this run's final assessment. `SEED_FINAL` was inspected by PR #1328 and
/// is retained as development/regression evidence only.
const SEED_FINAL2: u64 = 0x5C0F_F2B2;
/// Prospectively declared fresh construction seed for the read-confidence interface evaluation.
const SEED_CONF_FRESH: u64 = 0x5C0F_C0DE;
/// Declared window count per reader document before the preparation probe may extend it.
const WINDOWS_PER_DOC: usize = 4;
/// Declared minimum information target for natural-text fit positions.
const MIN_TEXT_FIT_POSITIONS: usize = 600;
/// Declared admission-regret trigger for naming admission as a future candidate (nats/position).
const ADMISSION_TRIGGER_NATS: f64 = 0.25;
/// Prospectively declared component margin in bits per position.
const MARGIN_PRESENT_BITS: f64 = 0.05;
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
    local_z: &[i32],
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
    if let Some((k, st)) = sel.choose_with(&cands, &rel, Some(local_z)) {
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
    let r = read_step(ring, cur, prev as u32, prev2, sel, table, use_reader, &z);
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
            "crates/uor-r4-core/src/native_geometric/learner/policy_feasibility.rs",
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

    #[test]
    fn contextual_map_search_uses_lexicographic_alias_priority() {
        assert!(!ce_objective_better((1, 0.0), (0, 1.0e12)));
        assert!(ce_objective_better((0, 1.0e12), (1, 0.0)));
        assert!(ce_objective_better((0, 1.0), (0, 2.0)));
        assert!(!ce_objective_better((0, 2.0), (0, 2.0)));
        let mut p = ReadConditionedParams::identity();
        p.value_domain = vec![10, 11];
        p.value_code = vec![3, 3];
        let ex = |payload, target| CeExample {
            z_local: vec![0; 2],
            q0: 0,
            r: 0,
            payload,
            read: true,
            target,
        };
        assert_eq!(
            ce_collisions(&[ex(10, 0), ex(10, 1)], &p),
            0,
            "contradictory targets for the same selected payload are not a value-code alias"
        );
        assert_eq!(ce_collisions(&[ex(10, 0), ex(11, 1)], &p), 1);
        p.value_code[1] = 4;
        assert_eq!(ce_collisions(&[ex(10, 0), ex(11, 1)], &p), 0);
    }

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
            policy: Vec::new(),
            gap_thresholds: [0; 3],
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
            policy: Vec::new(),
            gap_thresholds: [0; 3],
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
    fn the_shared_read_step_uses_the_direct_policy_and_its_gap_observation() {
        let b = banks();
        let table = ExactGroupTable::build().expect("exact table");
        let mut seed = 7u64;
        let seq = make_seq(&b, &mut seed, &b.values_held, false);
        let obs = observe_full(&seq);
        let o = obs
            .iter()
            .rev()
            .find(|o| !o.cands.is_empty())
            .expect("a candidate-bearing position");
        let sel = RelationalSelector {
            q_roots: vec![0u8; 256],
            mode: RelMode::Geometric,
            code_of: Vec::new(),
            w: [0; EXACT_FEATS],
            rank: [0; RANKS],
            bias: 0,
            sb: [0; ACTS],
            noread: 0,
            ctx: Vec::new(),
            policy: vec![1u8; UTIL_BUCKETS],
            gap_thresholds: [-24, -8, -2],
        };
        sel.validate().unwrap();
        let z = vec![0i32; 256];
        let r = read_step(&o.ring, o.cur, o.prev, o.prev2, &sel, &table, true, &z);
        let (k, a) = r.action.expect("the policy reads");
        assert_eq!(a, 0, "opcode 1 is the weakest strength");
        assert_eq!(r.payload, Some(o.cands[k].payload));
        // The disabled reader is not affected by the policy at all.
        assert!(
            read_step(&o.ring, o.cur, o.prev, o.prev2, &sel, &table, false, &z)
                .action
                .is_none()
        );
        // An all-NoRead policy abstains through the same shared path.
        let mut quiet = sel.clone();
        quiet.policy = vec![0u8; UTIL_BUCKETS];
        assert!(
            read_step(&o.ring, o.cur, o.prev, o.prev2, &quiet, &table, true, &z)
                .action
                .is_none()
        );
        // Indexing: changing only the declared local-gap observation moves the bucket, and the
        // shared path follows the new bucket's opcode.
        let rel: Vec<usize> = o
            .cands
            .iter()
            .map(|c| {
                relation_index(
                    &table,
                    sel.mode,
                    &sel.code_of,
                    *sel.q_roots.get(o.prev as usize).unwrap_or(&0),
                    *sel.q_roots.get(c.x_prev as usize).unwrap_or(&0),
                    o.prev as usize,
                    c.x_prev as usize,
                )
            })
            .collect();
        let mut sel2 = sel.clone();
        sel2.policy = vec![0u8; UTIL_BUCKETS];
        let (top, b0) = sel2.policy_bucket(&o.cands, &rel, &z).expect("bucket");
        let mut z2 = z.clone();
        let p = o.cands[top].payload as usize;
        z2[p] = z2.iter().copied().max().unwrap_or(0) - 100;
        let (_, b1) = sel2.policy_bucket(&o.cands, &rel, &z2).expect("bucket");
        assert_ne!(b0, b1, "the declared gap observation must move the bucket");
        sel2.policy[b0] = 2;
        sel2.policy[b1] = 3;
        assert_eq!(
            read_step(&o.ring, o.cur, o.prev, o.prev2, &sel2, &table, true, &z)
                .action
                .expect("read")
                .1,
            1
        );
        assert_eq!(
            read_step(&o.ring, o.cur, o.prev, o.prev2, &sel2, &table, true, &z2)
                .action
                .expect("read")
                .1,
            2
        );
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
// ---------------------------------------------------------------------------
// Utility-transfer mode: learn read influence for the frozen improved source maps
// ---------------------------------------------------------------------------

/// Every position of the full prediction stream, **including empty candidate pools**, which take the
/// local prediction. This is the one fixed population every pool/policy comparison uses.
fn observe_full(seq: &Seq) -> Vec<Obs> {
    let mut ring = OccurrenceRing::new(RING_CAP);
    let mut out = Vec::new();
    for i in 0..seq.tokens.len() {
        let cur = seq.tokens[i];
        let prev = if i >= 1 { seq.tokens[i - 1] } else { 0 };
        let prev2 = if i >= 2 { seq.tokens[i - 2] } else { NO_TOKEN };
        if i >= 2 && i + 1 < seq.tokens.len() {
            let (cands, _) = admit_mixed(&ring, ring.written(), cur, prev, prev2, MAX_CAND);
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
        ring.observe(cur);
    }
    out
}

fn observe_full_all(seqs: &[Seq]) -> Vec<Vec<Obs>> {
    seqs.iter().map(observe_full).collect()
}

/// A real digest of the exact group table bytes, replacing the builder-name string.
fn group_table_digest(t: &ExactGroupTable) -> String {
    let mut b = Vec::new();
    b.extend_from_slice(b"r4g1");
    b.push(t.identity);
    b.extend_from_slice(&t.product);
    b.extend_from_slice(&t.inverse);
    sha256_hex(&b)
}

/// Target-using ring diagnostic: is the target an observable successor somewhere in the **whole**
/// causal ring, and what is the best finite action over that whole ring (`Lring <= 0`)? Never a
/// serving feature.
fn ring_utility(ring: &OccurrenceRing, z: &[i32], target: u32, f_bits: u32) -> (bool, f64, usize) {
    let written = ring.written();
    let low = ring.valid_from();
    let (mut eligible, mut lring, mut occ) = (false, 0.0f64, 0usize);
    for abs in low..written {
        if let Some(payload) = ring.get(abs + 1) {
            occ += 1;
            if payload == target {
                eligible = true;
            }
            let p = prob_of(z, payload, f_bits);
            for a in 0..ACTS {
                let boost = (1i64 << AMP_SHIFTS[a]) as f64 / (1i64 << f_bits) as f64;
                let d = action_loss(p, boost, payload == target);
                if d < lring {
                    lring = d;
                }
            }
        }
    }
    (eligible, lring, occ)
}

/// One prepared position: the local baseline logits, coverage, the supervised position constants and
/// the ring bound. Computed **once** per full stream and reused by every arm.
struct PrepPos {
    z0: Vec<i32>,
    covered: bool,
    pos: Option<TrainPos>,
    ring: (bool, f64),
}

fn prep_stream(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    obs: &[Vec<Obs>],
) -> Vec<PrepPos> {
    let mut out = Vec::new();
    for os in obs.iter() {
        for o in os.iter() {
            let z0 = local_logits(
                parent,
                local,
                u,
                o.cur,
                o.prev as usize,
                o.prev2,
                o.ring.written(),
            );
            let covered = o.cands.iter().any(|c| c.payload == o.target);
            let rng = ring_utility(&o.ring, &z0, o.target, parent.cfg.f_bits);
            let pos = if o.cands.is_empty() {
                None
            } else {
                Some(pos_of_obs_with(parent, u, o, &z0))
            };
            out.push(PrepPos {
                z0,
                covered,
                pos,
                ring: (rng.0, rng.1),
            });
        }
    }
    out
}

fn pos_of_obs_with(parent: &PriorCore, u: &[Vec<i32>], o: &Obs, z: &[i32]) -> TrainPos {
    let _ = u;
    let delta: Vec<[f64; ACTS]> = o
        .cands
        .iter()
        .map(|c| {
            let p = prob_of(z, c.payload, parent.cfg.f_bits);
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
}

/// The expected result field set, validated **before** the expensive pass.
fn result_schema() -> serde_json::Value {
    json!({
        "schema": "uor-r4.reader-utility/1",
        "required": [
            "schema", "base_revision", "running_source", "inputs", "parents", "group_digest",
            "text_split", "preparation_probe", "fit", "panels", "ring_diagnostics", "controls",
            "generation", "cost", "manifest_digest", "decision", "phases", "elapsed_s"
        ],
    })
}

fn validate_result_fields(schema: &serde_json::Value) -> Result<(), String> {
    let required = schema["required"].as_array().ok_or("schema.required")?;
    let skeleton = json!({
        "schema": 0, "base_revision": 0, "running_source": 0, "inputs": 0, "parents": 0,
        "group_digest": 0, "text_split": 0, "preparation_probe": 0, "fit": 0, "panels": 0,
        "ring_diagnostics": 0, "controls": 0, "generation": 0, "cost": 0, "manifest_digest": 0,
        "decision": 0, "phases": 0, "elapsed_s": 0,
    });
    for k in required {
        let name = k.as_str().ok_or("schema key")?;
        if skeleton.get(name).is_none() {
            return Err(format!(
                "result schema is missing the required field {name}"
            ));
        }
    }
    Ok(())
}

/// Aggregate of one arm over one fixed full stream.
#[derive(Default)]
struct ArmAgg {
    positions: usize,
    empty_pool: usize,
    cand_bearing: usize,
    reads: usize,
    strengths: [usize; ACTS],
    correct_payload: usize,
    emitted_correct_local: usize,
    emitted_correct_reader: usize,
    covered: usize,
    served_ne_ungated: usize,
    hard_bits: f64,
    local_bits: f64,
    rank_regret: f64,
    gate_regret: f64,
    dose_regret: f64,
    lpool_sum: f64,
    admission_regret: f64,
    admission_regret_cand: f64,
    admission_regret_complete: f64,
    actual_logit_mismatches: usize,
    per_doc: BTreeMap<usize, (f64, f64, usize)>,
    strata: BTreeMap<String, (usize, usize, usize, f64, f64)>,
    /// Per stratum: reads, NoRead, correct payload, emitted correct, emitted correct (local), and
    /// the action counts for the three strengths.
    strata_detail: BTreeMap<String, [usize; 8]>,
}

fn agg_json(a: &ArmAgg) -> serde_json::Value {
    json!({
        "positions": a.positions,
        "empty_pool_positions": a.empty_pool,
        "candidate_bearing_positions": a.cand_bearing,
        "reads": a.reads,
        "strength_counts": a.strengths.to_vec(),
        "covered_positions": a.covered,
        "correct_payload_reads": a.correct_payload,
        "emitted_correct_local": a.emitted_correct_local,
        "emitted_correct_reader": a.emitted_correct_reader,
        "hard_bits": a.hard_bits,
        "local_bits": a.local_bits,
        "hard_bits_per_position": a.hard_bits / a.positions.max(1) as f64,
        "delta_bits_per_position": (a.hard_bits - a.local_bits) / a.positions.max(1) as f64,
        "mean_ranking_regret_nats": a.rank_regret / a.positions.max(1) as f64,
        "mean_gate_regret_nats": a.gate_regret / a.positions.max(1) as f64,
        "mean_dose_regret_nats": a.dose_regret / a.positions.max(1) as f64,
        "mean_lpool_nats": a.lpool_sum / a.positions.max(1) as f64,
        "mean_admission_regret_nats": a.admission_regret / a.positions.max(1) as f64,
        "mean_admission_regret_candidate_only_nats": a.admission_regret_cand / a.cand_bearing.max(1) as f64,
        "mean_admission_regret_complete_stream_nats": a.admission_regret_complete / a.positions.max(1) as f64,
        "actual_vs_emitted_logit_mismatches": a.actual_logit_mismatches,
        "read_actions_where_served_source_differs_from_ungated_top": a.served_ne_ungated,
        "strata": a.strata.iter().map(|(k, (n, cov, corr, hb, lb))| {
            let d = a.strata_detail.get(k).copied().unwrap_or([0; 8]);
            json!({
                "stratum": k, "positions": n, "covered_positions": cov,
                "correct_payload_reads": corr,
                "reads": d[0], "no_read": d[1], "correct_payload": d[2],
                "emitted_correct": d[3], "emitted_correct_local": d[4],
                "strength_counts": [d[5], d[6], d[7]],
                "hard_bits_per_position": hb / (*n).max(1) as f64,
                "delta_bits_per_position": (hb - lb) / (*n).max(1) as f64,
            })
        }).collect::<Vec<_>>(),
        "documents": a.per_doc.iter().map(|(d, (l, r, n))| json!({
            "document": d, "positions": n, "local_bits": l, "reader_bits": r,
        })).collect::<Vec<_>>(),
    })
}

/// Evaluate one arm (or the local baseline when `sel` is `None`) over one prepared full stream.
#[allow(clippy::too_many_arguments)]
fn eval_stream(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    seqs: &[Seq],
    obs: &[Vec<Obs>],
    kind: &str,
    doc_of: Option<&[usize]>,
    sel: Option<&RelationalSelector>,
    prep: &[PrepPos],
) -> ArmAgg {
    let mut a = ArmAgg::default();
    let mut idx = 0usize;
    for (gi, os) in obs.iter().enumerate() {
        let seq = &seqs[gi];
        for o in os.iter() {
            let pp = &prep[idx];
            a.positions += 1;
            let (z, r) = match sel {
                None => (
                    pp.z0.clone(),
                    Read {
                        action: None,
                        source: None,
                        payload: None,
                        payload_abs: None,
                        rel: 0,
                        admitted: 0,
                        scanned: 0,
                    },
                ),
                Some(s) => predict_next(
                    &o.ring,
                    o.cur,
                    o.prev as usize,
                    o.prev2,
                    s,
                    table,
                    parent,
                    local,
                    u,
                    true,
                ),
            };
            let lb = bits(&pp.z0, o.target, parent.cfg.f_bits);
            let hb = bits(&z, o.target, parent.cfg.f_bits);
            a.local_bits += lb;
            a.hard_bits += hb;
            let emitted_local = argmax_low(&pp.z0) as u32;
            let emitted = argmax_low(&z) as u32;
            if emitted_local == o.target {
                a.emitted_correct_local += 1;
            }
            if emitted == o.target {
                a.emitted_correct_reader += 1;
            }
            if o.cands.is_empty() {
                a.empty_pool += 1;
            } else {
                a.cand_bearing += 1;
            }
            if pp.covered {
                a.covered += 1;
            }
            if let Some((_k, st)) = r.action {
                a.reads += 1;
                a.strengths[st.min(ACTS - 1)] += 1;
            }
            if r.payload == Some(o.target) {
                a.correct_payload += 1;
            }
            if let (Some(s), Some(p)) = (sel, pp.pos.as_ref()) {
                // **The actual served action**, not `choose`: a policy-bearing selector returns `None`
                // from `choose`, so the scored helper would measure NoRead for an executed read.
                let reg = regret_decomposition_acted(s, table, p, r.action);
                a.rank_regret += reg.ranking;
                a.gate_regret += reg.gate;
                a.dose_regret += reg.dose;
                a.lpool_sum += reg.lpool;
                // `actual` must equal the loss change implied by the actually emitted logits.
                let served_delta_nats = (hb - lb) * std::f64::consts::LN_2;
                if (reg.actual - served_delta_nats).abs() > 1e-6 {
                    a.actual_logit_mismatches += 1;
                }
                if r.action.is_some() && !reg.served_matches_ungated {
                    a.served_ne_ungated += 1;
                }
            }
            // Admission regret has two declared denominators: candidate-bearing and the complete
            // stream. An empty pool has `Lpool = 0` and still contributes to the complete-stream mean.
            if sel.is_some() {
                let lpool_here = match pp.pos.as_ref() {
                    Some(p) => (0..p.cands.len())
                        .map(|k| {
                            p.delta[k]
                                .iter()
                                .cloned()
                                .fold(f64::INFINITY, f64::min)
                                .min(0.0)
                        })
                        .fold(0.0f64, f64::min),
                    None => 0.0,
                };
                let term = lpool_here - pp.ring.1;
                a.admission_regret += term;
                a.admission_regret_complete += term;
                if !o.cands.is_empty() {
                    a.admission_regret_cand += term;
                }
            }
            let label: String = if kind == "construction" {
                stratum_of(o, seq).to_string()
            } else if pp.covered {
                "text_copy_available".into()
            } else {
                "text_copy_unavailable".into()
            };
            let e = a.strata.entry(label.clone()).or_default();
            e.0 += 1;
            e.1 += usize::from(pp.covered);
            e.2 += usize::from(r.payload == Some(o.target));
            e.3 += hb;
            e.4 += lb;
            let reads_here = usize::from(r.action.is_some());
            let no_read_here = usize::from(r.action.is_none());
            let emitted_ok_here = usize::from(emitted == o.target);
            let correct_payload_here = usize::from(r.payload == Some(o.target));
            let se = a.strata_detail.entry(label).or_insert([0usize; 8]);
            se[0] += reads_here;
            se[1] += no_read_here;
            se[2] += correct_payload_here;
            se[3] += emitted_ok_here;
            se[4] += usize::from(emitted_local == o.target);
            if let Some((_k, st)) = r.action {
                se[5 + st.min(ACTS - 1)] += 1;
            }
            if let Some(d) = doc_of {
                if let Some(&doc) = d.get(idx) {
                    let de = a.per_doc.entry(doc).or_insert((0.0, 0.0, 0));
                    de.0 += lb;
                    de.1 += hb;
                    de.2 += 1;
                }
            }
            idx += 1;
        }
    }
    a
}

/// One direct-action fit event from one observation under one frozen source arm **and the configured
/// feature contract**. Delegates to the library extractor so the train/serve contract is one function.
fn policy_event(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    sel: &RelationalSelector,
    cfg: &PolicyConfig,
    o: &Obs,
    weight: f64,
) -> Option<PolicyEvent> {
    if o.cands.is_empty() {
        return None;
    }
    let z = local_logits(
        parent,
        local,
        u,
        o.cur,
        o.prev as usize,
        o.prev2,
        o.ring.written(),
    );
    let rel = rels_for(sel, o, table, false);
    policy_event_for(
        sel,
        cfg,
        &o.cands,
        &rel,
        &z,
        o.target,
        parent.cfg.f_bits,
        weight,
    )
}

/// Greedy continuation from an observed prefix through the shared path.
#[allow(clippy::too_many_arguments)]
fn greedy_continue(
    prefix: &[u32],
    sel: &RelationalSelector,
    table: &ExactGroupTable,
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    use_reader: bool,
    n: usize,
) -> Vec<u32> {
    let mut ring = ring_before_current(prefix);
    let mut toks: Vec<u32> = prefix.to_vec();
    let start = toks.len();
    for _ in 0..n {
        if generate_step(
            &mut ring, &mut toks, sel, table, parent, local, u, use_reader,
        )
        .is_err()
        {
            break;
        }
    }
    toks[start..].to_vec()
}

/// Document-cluster interval of `a - b` from serialized per-document sums.
fn doc_cluster(pairs: &[(f64, f64, usize)], seed: u64) -> serde_json::Value {
    let mut v = boot_diff(pairs, seed);
    v["resampling_unit"] = json!("document");
    v["unit"] = json!("bits_per_token");
    v
}

/// Recompute and verify the manifest digest a loader would check.
fn verify_manifest(root: &Path) -> Result<String, String> {
    let p = root.join("binding.json");
    let text = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let mut v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let claimed = v
        .get("manifest_digest")
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string();
    v["manifest_digest"] = json!("");
    let canonical = serde_json::to_string(&v).map_err(|e| e.to_string())?;
    let got = sha256_hex(canonical.as_bytes());
    if got != claimed {
        return Err(format!("manifest digest {claimed} != recomputed {got}"));
    }
    Ok(got)
}

/// Bool -> {0.0, 1.0}.
fn b2f(b: bool) -> f64 {
    if b {
        1.0
    } else {
        0.0
    }
}

/// Apply a chosen action table to a `(bucket, action)` contribution matrix.
fn pick_mat(m: &[[f64; ACTS + 1]], actions: &[usize]) -> f64 {
    let mut s = 0.0f64;
    for b in 0..m.len().min(actions.len()) {
        s += m[b][actions[b].min(ACTS)];
    }
    s
}

/// Per-position counterfactual sufficient statistics for one arm over one population. Filled by
/// `accumulate_feas` from the actual integer logits, so no model pass is needed per optimizer step.
#[derive(Clone, Debug)]
struct FeasStats {
    /// Text CE change in bits per `(bucket, action)`, summed over the fit/tune reader documents.
    text_bits: Vec<[f64; ACTS + 1]>,
    /// Present-query emitted-correct counts per `(bucket, action)`.
    present_correct: Vec<[f64; ACTS + 1]>,
    /// Present-query CE change in bits per `(bucket, action)`.
    present_delta: Vec<[f64; ACTS + 1]>,
    /// Absent-query read counts per `(bucket, action)`.
    absent_read: Vec<[f64; ACTS + 1]>,
    /// Absent-query CE change in bits per `(bucket, action)`.
    absent_delta: Vec<[f64; ACTS + 1]>,
    /// Per-document text CE change in bits, for independently reconstructible intervals.
    text_docs: BTreeMap<usize, Vec<[f64; ACTS + 1]>>,
    /// Candidate-bearing position count per address (the declared support mask input).
    support: Vec<f64>,
    text_candidate_positions: usize,
    present_positions: usize,
    absent_positions: usize,
    counterfactual_positions: usize,
    max_resid: f64,
}

impl FeasStats {
    fn new(nb: usize) -> Self {
        FeasStats {
            text_bits: vec![[0.0f64; ACTS + 1]; nb],
            present_correct: vec![[0.0f64; ACTS + 1]; nb],
            present_delta: vec![[0.0f64; ACTS + 1]; nb],
            absent_read: vec![[0.0f64; ACTS + 1]; nb],
            absent_delta: vec![[0.0f64; ACTS + 1]; nb],
            text_docs: BTreeMap::new(),
            support: vec![0.0f64; nb],
            text_candidate_positions: 0,
            present_positions: 0,
            absent_positions: 0,
            counterfactual_positions: 0,
            max_resid: 0.0,
        }
    }
    fn doc_pick(&self, actions: &[usize]) -> Vec<(usize, f64)> {
        let mut v: Vec<(usize, f64)> = Vec::new();
        for (d, row) in self.text_docs.iter() {
            let mut s = 0.0f64;
            for b in 0..row.len() {
                s += row[b][actions[b].min(ACTS)];
            }
            v.push((*d, s));
        }
        v
    }
}

/// Realized aggregates of the frozen parent on the same populations.
#[derive(Clone, Copy, Debug, Default)]
struct ParentAgg {
    present_correct: f64,
    present_delta: f64,
    absent_read: f64,
    absent_delta: f64,
    present_positions: usize,
    absent_positions: usize,
}

/// Accumulate the four counterfactual action outcomes at every candidate-bearing position under one
/// configured selector. Construction positions are split into the `final_present` / `final_absent`
/// strata; text positions are split by document.
#[allow(clippy::too_many_arguments)]
fn accumulate_feas(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    cfg: &PolicyConfig,
    sel: &RelationalSelector,
    obs: &[Vec<Obs>],
    seqs: &[Seq],
    kind: &str,
    doc_of: Option<&[usize]>,
    conf: bool,
    st: &mut FeasStats,
) -> Result<(), String> {
    let f_bits = parent.cfg.f_bits;
    let mut idx = 0usize;
    for (gi, os) in obs.iter().enumerate() {
        let seq = &seqs[gi];
        for o in os.iter() {
            let doc = doc_of.map(|d| d[idx]);
            idx += 1;
            if o.cands.is_empty() {
                continue;
            }
            let z = local_logits(
                parent,
                local,
                u,
                o.cur,
                o.prev as usize,
                o.prev2,
                o.ring.written(),
            );
            let rel = rels_for(sel, o, table, false);
            let (top, bucket) = if conf {
                match sel.confidence_address(&o.cands, &rel, &z) {
                    Some((top, addr, _d)) => (top, addr),
                    None => continue,
                }
            } else {
                match sel.policy_bucket_with(cfg, &o.cands, &rel, &z) {
                    Some((top, bucket)) => (top, bucket),
                    None => continue,
                }
            };
            let payload = o.cands[top].payload;
            let (out, resid) = position_action_outcomes(&z, payload, o.target, f_bits);
            if resid > 1e-6 {
                return Err(format!(
                    "policy identity residual {resid} on a {kind} position"
                ));
            }
            st.max_resid = st.max_resid.max(resid);
            st.counterfactual_positions += 1;
            let nb = st.text_bits.len();
            let b = bucket.min(nb - 1);
            st.support[b] += 1.0;
            if kind == "text" {
                st.text_candidate_positions += 1;
                for a in 0..=ACTS {
                    st.text_bits[b][a] += out[a].delta_bits;
                }
                if let Some(d) = doc {
                    let row = st
                        .text_docs
                        .entry(d)
                        .or_insert_with(|| vec![[0.0f64; ACTS + 1]; nb]);
                    for a in 0..=ACTS {
                        row[b][a] += out[a].delta_bits;
                    }
                }
            } else {
                match stratum_of(o, seq) {
                    "final_present" => {
                        st.present_positions += 1;
                        for a in 0..=ACTS {
                            st.present_correct[b][a] += b2f(out[a].correct);
                            st.present_delta[b][a] += out[a].delta_bits;
                        }
                    }
                    "final_absent" => {
                        st.absent_positions += 1;
                        for a in 0..=ACTS {
                            st.absent_read[b][a] += b2f(out[a].read);
                            st.absent_delta[b][a] += out[a].delta_bits;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

/// Realized parent aggregates on the `final_present` / `final_absent` construction strata.
#[allow(clippy::too_many_arguments)]
fn accumulate_parent(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    sel: &RelationalSelector,
    obs: &[Vec<Obs>],
    seqs: &[Seq],
    agg: &mut ParentAgg,
) {
    let f_bits = parent.cfg.f_bits;
    for (gi, os) in obs.iter().enumerate() {
        let seq = &seqs[gi];
        for o in os.iter() {
            if o.cands.is_empty() {
                continue;
            }
            let stratum = stratum_of(o, seq);
            if stratum != "final_present" && stratum != "final_absent" {
                continue;
            }
            let z = local_logits(
                parent,
                local,
                u,
                o.cur,
                o.prev as usize,
                o.prev2,
                o.ring.written(),
            );
            let rel = rels_for(sel, o, table, false);
            let chosen = sel.choose_scored(&o.cands, &rel);
            let (payload, strength) = match chosen {
                Some((k, st)) => (Some(o.cands[k].payload), Some(st)),
                None => (None, None),
            };
            let oc = realized_outcome(&z, payload, strength, o.target, f_bits);
            if stratum == "final_present" {
                agg.present_positions += 1;
                agg.present_correct += b2f(oc.correct);
                agg.present_delta += oc.delta_bits;
            } else {
                agg.absent_positions += 1;
                agg.absent_read += b2f(oc.read);
                agg.absent_delta += oc.delta_bits;
            }
        }
    }
}

/// The witnessed parent rule on a confidence address: eight nats iff the parent's causal advantage
/// is positive (odd address), otherwise NoRead.
fn parent_rule_op(b: usize) -> usize {
    if b % 2 == 1 {
        3
    } else {
        0
    }
}

fn parent_rule_table(nb: usize) -> Vec<usize> {
    (0..nb).map(parent_rule_op).collect()
}

/// Test a table against the original constraint senses and tolerances.
fn table_feasible(p: &FeasProblem, x: &[usize]) -> bool {
    p.cons.iter().enumerate().all(|(j, c)| {
        let v = realized(p, x, j);
        match c.sense {
            ConSense::AtMost => v <= c.rhs + c.tol,
            ConSense::AtLeast => v >= c.rhs - c.tol,
        }
    })
}

fn table_obj(p: &FeasProblem, x: &[usize]) -> f64 {
    x.iter().enumerate().map(|(b, a)| p.obj[b][*a]).sum()
}

fn require_witness_parity(checked: usize, mismatches: usize) -> Result<(), String> {
    if checked == 0 || mismatches != 0 {
        return Err(format!(
            "confidence witness has {mismatches} mismatches over {checked} checked candidate-bearing positions"
        ));
    }
    Ok(())
}

/// A failed confidence load cannot become `None`, which means the local baseline to panel callers.
fn require_matching_confidence_load(
    name: &str,
    expected: &RelationalSelector,
    loaded: Result<RelationalSelector, String>,
) -> Result<RelationalSelector, String> {
    let got = loaded.map_err(|e| format!("confidence load failed for {name}: {e}"))?;
    if got.policy.len() != CONF_ADDRESSES || got != *expected {
        return Err(format!(
            "loaded confidence selector {name} differs from the selected 64-address operator"
        ));
    }
    Ok(got)
}

fn required_confidence_arm<'a>(
    arms: &'a BTreeMap<&str, RelationalSelector>,
    name: &str,
) -> Result<&'a RelationalSelector, String> {
    arms.get(name)
        .ok_or_else(|| format!("required confidence arm {name} was not selected and verified"))
}

/// Verify the parent-preserving witness: the configured selector's confidence sign must reproduce the
/// frozen scored parent's candidate-index/action pair on candidate-bearing causal development
/// prefixes. This check does not exercise empty pools or independently loaded integer-logit parity.
#[allow(clippy::too_many_arguments)]
fn witness_parity(
    configured: &RelationalSelector,
    parent_sel: &RelationalSelector,
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    obs: &[Vec<Obs>],
) -> Result<serde_json::Value, String> {
    let (mut checked, mut mismatches, mut parent_reads, mut d_pos) =
        (0usize, 0usize, 0usize, 0usize);
    let mut examples: Vec<serde_json::Value> = Vec::new();
    for os in obs.iter() {
        for o in os.iter() {
            if o.cands.is_empty() {
                continue;
            }
            let z = local_logits(
                parent,
                local,
                u,
                o.cur,
                o.prev as usize,
                o.prev2,
                o.ring.written(),
            );
            let rel = rels_for(configured, o, table, false);
            let Some((top, addr, d)) = configured.confidence_address(&o.cands, &rel, &z) else {
                continue;
            };
            let actual = parent_sel.choose_scored(&o.cands, &rel);
            let witness = if d > 0 { Some((top, 2usize)) } else { None };
            checked += 1;
            if d > 0 {
                d_pos += 1;
            }
            if actual.is_some() {
                parent_reads += 1;
            }
            let same = match (actual, witness) {
                (None, None) => true,
                (Some((k1, a1)), Some((k2, a2))) => k1 == k2 && a1 == a2,
                _ => false,
            };
            if !same {
                mismatches += 1;
                if examples.len() < 5 {
                    examples.push(json!({
                        "cur": o.cur, "candidates": o.cands.len(),
                        "actual": actual, "witness": witness, "address": addr, "d": d,
                    }));
                }
            }
        }
    }
    require_witness_parity(checked, mismatches)?;
    Ok(json!({
        "checked_positions": checked,
        "mismatches": mismatches,
        "parent_reads": parent_reads,
        "d_positive": d_pos,
        "examples": examples,
        "note": "candidate-bearing index/action parity only; witness = ungated top source at eight nats iff D>0; D uses the parent bucket and widened i64; empty-pool, independent loaded-logit and rollout parity are not tested here",
    }))
}

// ---------------------------------------------------------------------------
// Read-conditioned geometric emission: instrument, oracle ceiling, learning
// ---------------------------------------------------------------------------

const N_RC_DEV: usize = 60;
const N_RC_TUNE: usize = 30;
const N_RC_FRESH: usize = 30;
const SEED_RC_DEV: u64 = 0x5C0F_D001;
const SEED_RC_TUNE: u64 = 0x5C0F_D002;
const SEED_RC_FRESH: u64 = 0x5C0F_D003;

/// Historical derived-role diagnostic. The answer is the fixed partner of the query role and is
/// therefore determined by the recent suffix alone. This does not test whether older context is
/// necessary. `tokens` is the complete observed prefix: it does not contain an answer sentinel.
fn make_derive_seq(banks: &Banks, st: &mut u64) -> Seq {
    let n_fam = 3 + (xorshift(st) as usize) % 2;
    let mut fams: Vec<(u32, u32)> = Vec::new();
    while fams.len() < n_fam {
        let f = pick(&banks.pairs, st);
        if !fams.contains(&f) {
            fams.push(f);
        }
    }
    let qkey = pick(&banks.keys, st);
    let (a0, b0) = fams[0];
    let (qrole, partner) = if xorshift(st) & 1 == 0 {
        (a0, b0)
    } else {
        (b0, a0)
    };
    let mut blocks: Vec<(u32, u32, u32)> = Vec::new();
    for (i, (a, b)) in fams.iter().enumerate() {
        // The target family's block carries the query role; competitors use their own first role.
        let role = if i == 0 { qrole } else { *a };
        if role == partner {
            // Keep the partner absent from every role in the prefix.
            continue;
        }
        let mut v = pick(&banks.values_held, st);
        let mut guard = 0;
        while blocks.iter().any(|(_, _, x)| *x == v) && guard < 64 {
            v = pick(&banks.values_held, st);
            guard += 1;
        }
        blocks.push((role, qkey, v));
    }
    for i in (1..blocks.len()).rev() {
        let j = (xorshift(st) as usize) % (i + 1);
        blocks.swap(i, j);
    }
    let mut tokens = Vec::new();
    for (r, k, v) in blocks.iter() {
        tokens.push(*r);
        tokens.push(*k);
        tokens.push(*v);
    }
    tokens.extend_from_slice(&[qrole, qkey]);
    Seq {
        tokens,
        group: 0,
        answer: partner,
        absent: false,
        answer_source_abs: None,
    }
}

fn make_rc_pop(banks: &Banks, n: usize, seed: u64) -> Vec<Seq> {
    let mut st = seed;
    let mut out = Vec::new();
    for _ in 0..n {
        out.push(make_derive_seq(banks, &mut st));
    }
    out
}

/// One instrument decision point with the frozen local base (the query row removed) so a candidate
/// row can be evaluated in `O(vocab)`.
struct RcPos {
    z_base: Vec<i32>,
    q0: usize,
    rel: usize,
    payload: u32,
    read: bool,
    answer: u32,
    covered: bool,
    cands: usize,
}

/// The target-free boundary for a complete observed prefix. Its final token remains the current
/// input; no supervised next-token observation or last-nonempty-candidate search is involved.
fn rc_prefix_boundary(prefix: &[u32]) -> Result<(OccurrenceRing, u32, u32, u32), String> {
    if prefix.len() < 3 {
        return Err("read-conditioned prefix needs two predecessors".into());
    }
    let i = prefix.len() - 1;
    Ok((
        ring_before_current(prefix),
        prefix[i],
        prefix[i - 1],
        prefix[i - 2],
    ))
}

fn first_continuation_token(tokens: &[u32], prefix_len: usize) -> Result<u32, String> {
    tokens
        .get(prefix_len)
        .copied()
        .ok_or_else(|| "rollout did not emit a first continuation token".into())
}

fn reload_read_conditioned(
    bytes: &[u8],
    expected: &ReadConditionedParams,
) -> Result<ReadConditionedParams, String> {
    let loaded = ReadConditionedParams::from_bytes(bytes)?;
    if loaded != *expected {
        return Err("read-conditioned reload changed the parameters".into());
    }
    Ok(loaded)
}

/// Build instrument positions under one frozen read rule. Returns the positions and validity counts.
#[allow(clippy::too_many_arguments)]
fn rc_positions(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    sel: &RelationalSelector,
    seqs: &[Seq],
) -> Result<(Vec<RcPos>, usize, usize, usize), String> {
    let mut out = Vec::new();
    let (mut covered_violations, mut decisive, mut no_read) = (0usize, 0usize, 0usize);
    for seq in seqs.iter() {
        let (ring, cur, prev, prev2) = rc_prefix_boundary(&seq.tokens)?;
        let z = local_logits(parent, local, u, cur, prev as usize, prev2, ring.written());
        let selected = read_step(&ring, cur, prev, prev2, sel, table, true, &z);
        let (cands, _) = admit_mixed(&ring, ring.written(), cur, prev, prev2, MAX_CAND);
        let read = selected.action.is_some();
        let covered = cands.iter().any(|c| c.payload == seq.answer);
        if covered {
            covered_violations += 1;
        }
        let q0 = local.query_state(cur).min(u.len() - 1);
        let z_base: Vec<i32> = z.iter().zip(u[q0].iter()).map(|(a, b)| a - b).collect();
        let local_argmax = argmax_low(&z) as u32;
        if local_argmax != seq.answer {
            decisive += 1;
        }
        if !read {
            no_read += 1;
        }
        out.push(RcPos {
            z_base,
            q0,
            rel: selected.rel,
            payload: selected.payload.unwrap_or(0),
            read,
            answer: seq.answer,
            covered,
            cands: cands.len(),
        });
    }
    Ok((out, covered_violations, decisive, no_read))
}

#[inline]
fn rc_argmax_row(z_base: &[i32], row: &[i32]) -> usize {
    let mut best = 0usize;
    let mut bv = z_base[0] + row[0];
    for i in 1..z_base.len() {
        let v = z_base[i] + row[i];
        if v > bv {
            bv = v;
            best = i;
        }
    }
    best
}

fn rc_argmax(params: &ReadConditionedParams, pos: &RcPos, u: &[Vec<i32>]) -> usize {
    let q1 = if pos.read {
        params.update(pos.q0, pos.rel, pos.payload)
    } else {
        pos.q0
    };
    rc_argmax_row(&pos.z_base, &u[q1.min(u.len() - 1)])
}

fn rc_accuracy(params: &ReadConditionedParams, positions: &[RcPos], u: &[Vec<i32>]) -> f64 {
    if positions.is_empty() {
        return f64::NAN;
    }
    let hit = positions
        .iter()
        .filter(|p| rc_argmax(params, p, u) as u32 == p.answer)
        .count();
    hit as f64 / positions.len() as f64
}

/// The oracle ceiling: the fraction of positions where *some* shared row emits the required answer.
fn rc_oracle(positions: &[RcPos], u: &[Vec<i32>]) -> (usize, usize) {
    let mut hit = 0usize;
    for p in positions.iter() {
        for q in 0..u.len() {
            if rc_argmax_row(&p.z_base, &u[q]) as u32 == p.answer {
                hit += 1;
                break;
            }
        }
    }
    (hit, positions.len())
}

/// A different fixed initialization of the same H4 operator, not a categorical-state control.
fn alternate_h4_value_seed(token: u32) -> usize {
    ((token as usize) * 7 + 3) % GROUP_ORDER
}

/// Bounded discrete coordinate search over the transport and value-code maps under final-token
/// accuracy on the development instrument. No gradients, no new readout parameters.
fn rc_learn(
    positions: &[RcPos],
    u: &[Vec<i32>],
    alternate_initialization: bool,
) -> ReadConditionedParams {
    let t = group_table();
    let relations: Vec<usize> = {
        let mut v: Vec<usize> = positions
            .iter()
            .filter(|p| p.read)
            .map(|p| p.rel % GROUP_ORDER)
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    let domain: Vec<u32> = {
        let mut v: Vec<u32> = positions
            .iter()
            .filter(|p| p.read)
            .map(|p| p.payload)
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    let mut params = if alternate_initialization {
        // Same H4 products, relation encoding and optimizer; only the initial value codes differ.
        let mut p = ReadConditionedParams::identity();
        p.value_domain = domain.clone();
        p.value_code = domain
            .iter()
            .map(|tok| (alternate_h4_value_seed(*tok)) as u8)
            .collect();
        p
    } else {
        let mut p = ReadConditionedParams::identity();
        p.value_domain = domain.clone();
        p.value_code = vec![t.identity; domain.len()];
        p
    };
    // Seed transport from the relation index itself so the start is not a dead no-op.
    for r in relations.iter() {
        params.transport[*r] = (*r % GROUP_ORDER) as u8;
    }
    let score = |p: &ReadConditionedParams| -> f64 {
        positions
            .iter()
            .filter(|pos| rc_argmax(p, pos, u) as u32 == pos.answer)
            .count() as f64
    };
    let mut best = score(&params);
    for _pass in 0..3 {
        let mut improved = false;
        for r in relations.iter() {
            let saved = params.transport[*r];
            let mut local_best = best;
            let mut local_val = saved;
            for cand in 0..GROUP_ORDER {
                params.transport[*r] = cand as u8;
                let v = score(&params);
                if v > local_best + 0.5 {
                    local_best = v;
                    local_val = cand as u8;
                }
            }
            params.transport[*r] = local_val;
            if local_best > best + 0.5 {
                best = local_best;
                improved = true;
            }
        }
        for i in 0..params.value_code.len() {
            let saved = params.value_code[i];
            let mut local_best = best;
            let mut local_val = saved;
            for cand in 0..GROUP_ORDER {
                params.value_code[i] = cand as u8;
                let v = score(&params);
                if v > local_best + 0.5 {
                    local_best = v;
                    local_val = cand as u8;
                }
            }
            params.value_code[i] = local_val;
            if local_best > best + 0.5 {
                best = local_best;
                improved = true;
            }
        }
        if !improved {
            break;
        }
    }
    params
}

/// Autoregressive rollout recording the served action and exact selected occurrence per step.
#[allow(clippy::too_many_arguments)]
fn rollout_prefix(
    prefix: &[u32],
    sel: &RelationalSelector,
    table: &ExactGroupTable,
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    use_reader: bool,
    n: usize,
) -> Result<(Vec<u32>, Vec<serde_json::Value>), String> {
    let mut ring = ring_before_current(prefix);
    let mut toks = prefix.to_vec();
    let mut steps = Vec::new();
    for s in 0..n {
        let (z, r) = generate_step(
            &mut ring, &mut toks, sel, table, parent, local, u, use_reader,
        )?;
        let next = *toks.last().unwrap_or(&0);
        steps.push(json!({
            "step": s,
            "read": r.action.is_some(),
            "action": r.action.map(|(_, a)| a),
            "admitted": r.admitted,
            "scanned": r.scanned,
            "source_ref": r.source.map(|s| json!({"seq": s.seq, "abs": s.abs})),
            "payload": r.payload,
            "emitted": next,
            "argmax": argmax_low(&z),
        }));
    }
    Ok((toks, steps))
}

/// The read-conditioned prediction: the shared read selects a source, then one learned geometric
/// update replaces the query row. `UpdateDisabled`/`NoRead` leave `q1 = q0`, so the residual is
/// exactly zero.
#[allow(clippy::too_many_arguments)]
fn rc_predict_next(
    ring: &OccurrenceRing,
    cur: u32,
    prev: usize,
    prev2: u32,
    sel: &RelationalSelector,
    table: &ExactGroupTable,
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    params: &ReadConditionedParams,
    use_update: bool,
    allowed: bool,
) -> (Vec<i32>, Read, usize, usize) {
    let mut z = local_logits(parent, local, u, cur, prev, prev2, ring.written());
    let r = read_step(ring, cur, prev as u32, prev2, sel, table, allowed, &z);
    let q0 = local.query_state(cur).min(u.len() - 1);
    let mut q1 = q0;
    if use_update {
        if let (Some(_), Some(p)) = (r.action, r.payload) {
            q1 = params.update(q0, r.rel, p);
        }
    }
    if q1 != q0 {
        let (a, b) = (&u[q1.min(u.len() - 1)], &u[q0]);
        for i in 0..z.len() {
            z[i] = z[i].saturating_add(a[i]).saturating_sub(b[i]);
        }
    }
    (z, r, q0, q1)
}

/// Read-conditioned rollout recording the served action, the update and the emitted tokens.
#[allow(clippy::too_many_arguments)]
fn rc_rollout(
    prefix: &[u32],
    sel: &RelationalSelector,
    table: &ExactGroupTable,
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    params: &ReadConditionedParams,
    use_update: bool,
    allowed: bool,
    n: usize,
) -> Result<(Vec<u32>, Vec<serde_json::Value>), String> {
    let (mut ring, _, _, _) = rc_prefix_boundary(prefix)?;
    let mut toks = prefix.to_vec();
    let mut steps = Vec::new();
    for s in 0..n {
        let i = toks
            .len()
            .checked_sub(1)
            .ok_or("empty read-conditioned prefix")?;
        if i < 2 || ring.written() as usize != i {
            return Err("read-conditioned rollout needs two predecessors".into());
        }
        let (z, r, q0, q1) = rc_predict_next(
            &ring,
            toks[i],
            toks[i - 1] as usize,
            toks[i - 2],
            sel,
            table,
            parent,
            local,
            u,
            params,
            use_update,
            allowed,
        );
        let next = argmax_low(&z) as u32;
        steps.push(json!({
            "step": s, "read": r.action.is_some(), "updated": q1 != q0,
            "q0": q0, "q1": q1, "admitted": r.admitted,
            "source_ref": r.source.map(|source| json!({"seq": source.seq, "abs": source.abs})),
            "payload_abs": r.payload_abs,
            "payload": r.payload, "emitted": next,
        }));
        ring.observe(toks[i]);
        toks.push(next);
    }
    Ok((toks, steps))
}

/// Teacher-forced CE of the required answer in bits under one read-conditioned table.
fn rc_nll_bits(pos: &RcPos, params: &ReadConditionedParams, u: &[Vec<i32>], f_bits: u32) -> f64 {
    let q1 = if pos.read {
        params.update(pos.q0, pos.rel, pos.payload)
    } else {
        pos.q0
    };
    let row = &u[q1.min(u.len() - 1)];
    let z: Vec<i32> = pos
        .z_base
        .iter()
        .zip(row.iter())
        .map(|(a, b)| a.saturating_add(*b))
        .collect();
    bits(&z, pos.answer, f_bits)
}

fn utility_transfer_run() -> Result<ExitCode, String> {
    // ---- arguments -------------------------------------------------------------
    let mut root = PathBuf::from(DEFAULT_UTIL_ROOT);
    let mut parent_root = PathBuf::from(PARENT_ROOT);
    let mut source_root: Option<PathBuf> = None;
    {
        let mut a = std::env::args().skip(1);
        while let Some(k) = a.next() {
            match k.as_str() {
                "--root" => root = PathBuf::from(a.next().ok_or("--root value")?),
                "--parent-root" => parent_root = PathBuf::from(a.next().ok_or("value")?),
                "--source-root" => source_root = Some(PathBuf::from(a.next().ok_or("value")?)),
                other if other.starts_with("--mode") => {}
                other => return Err(format!("unknown argument {other}")),
            }
        }
    }
    // Claim every output root immediately after argument validation, before any model load.
    claim(&root).map_err(|e| format!("claim {}: {e}", root.display()))?;
    let started = Instant::now();
    let mut marks: Vec<(&'static str, f64)> = Vec::new();
    let mark = |n: &'static str, m: &mut Vec<(&'static str, f64)>| {
        m.push((n, started.elapsed().as_secs_f64()));
    };
    let schema = result_schema();
    validate_result_fields(&schema)?;
    write_json(&root, "schema.json", &schema)?;
    let mut controls: Vec<serde_json::Value> = Vec::new();
    mark("schema", &mut marks);

    // ---- pinned inputs ---------------------------------------------------------
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
    let group_digest = group_table_digest(&table);
    let executable_sha256 = hex_of(&sha256_bytes(
        &std::env::current_exe()
            .ok()
            .and_then(|p| std::fs::read(p).ok())
            .unwrap_or_default(),
    ));
    let sources: Vec<serde_json::Value> = match &source_root {
        Some(sr) => [
            "crates/uor-r4-core/src/native_geometric/learner/relational.rs",
            "crates/uor-r4-core/src/native_geometric/learner/policy_feasibility.rs",
            "crates/uor-r4-core/src/native_geometric/learner/read_conditioned.rs",
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

    // ---- frozen parent artifacts, digest-pinned ---------------------------------
    let e_digest = sha256_bytes(&sb);
    let load_parent = |name: &str, pin: &str| -> Result<RelationalSelector, String> {
        let path = parent_root.join("artifacts").join(format!("{name}.rlr2"));
        let b = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let got = sha256_hex(&b);
        if got != pin {
            return Err(format!(
                "parent artifact {name} digest {got} != pinned {pin}"
            ));
        }
        let sel = RelationalArtifact::from_bytes(&b, &e_digest, &raw_tok)?.selector;
        sel.validate_for_vocab(parent.cfg.vocab)?;
        if !sel.policy.is_empty() {
            return Err(format!(
                "parent artifact {name} unexpectedly carries a policy"
            ));
        }
        Ok(sel)
    };
    let mut parents_meta: Vec<serde_json::Value> = Vec::new();
    let mut base_arms: BTreeMap<&'static str, RelationalSelector> = BTreeMap::new();
    for (name, pin) in [
        ("exact", PARENT_SHA_EXACT),
        ("relational", PARENT_SHA_RELATIONAL),
        ("relational_ctx", PARENT_SHA_RELATIONAL_CTX),
        ("categorical", PARENT_SHA_CATEGORICAL),
    ] {
        let sel = load_parent(name, pin)?;
        parents_meta.push(json!({"arm": name, "sha256": pin, "policy_present": false}));
        base_arms.insert(name, sel);
    }
    let exact_base = base_arms["exact"].clone();
    let relational_base = base_arms["relational"].clone();
    let relational_ctx_base = base_arms["relational_ctx"].clone();
    let categorical_base = base_arms["categorical"].clone();
    mark("parent artifacts", &mut marks);

    // ---- populations -----------------------------------------------------------
    let banks = build_banks(&tokenizer, parent.cfg.vocab)?;
    let fit = make_pop(&banks, N_FIT, SEED_FIT, &banks.values_fit);
    let tune = make_pop(&banks, N_TUNE, SEED_TUNE, &banks.values_fit);
    let regression = make_pop(&banks, N_FRESH, SEED_FRESH, &banks.values_held);
    // `prev_pop` is the population inspected by PR #1328: development/regression evidence only.
    let prev_pop = make_pop(&banks, N_FRESH, SEED_FINAL, &banks.values_held);
    // The **new** final draw for this run.
    let final_pop = make_pop(&banks, N_FRESH, SEED_FINAL2, &banks.values_held);
    for s in fit
        .iter()
        .chain(tune.iter())
        .chain(regression.iter())
        .chain(prev_pop.iter())
        .chain(final_pop.iter())
    {
        validate_fixture(s, &banks)?;
    }
    let fit_obs = observe_full_all(&fit);
    let tune_obs = observe_full_all(&tune);
    let regression_obs = observe_full_all(&regression);
    let prev_obs = observe_full_all(&prev_pop);
    let final_obs = observe_full_all(&final_pop);
    let pop_counts = |obs: &[Vec<Obs>]| -> (usize, usize) {
        let n: usize = obs.iter().map(|v| v.len()).sum();
        let cand: usize = obs.iter().flatten().filter(|o| !o.cands.is_empty()).count();
        (n, cand)
    };
    let (fit_n, fit_cand) = pop_counts(&fit_obs);
    let (tune_n, tune_cand) = pop_counts(&tune_obs);
    let _ = tune_cand;
    let (reg_n, reg_cand) = pop_counts(&regression_obs);
    let _ = reg_cand;
    let (prev_n, prev_cand) = pop_counts(&prev_obs);
    let _ = (prev_n, prev_cand);
    let (fin_n, fin_cand) = pop_counts(&final_obs);
    mark("construction", &mut marks);

    // ---- reader text: document-disjoint fit / tune / final ----------------------
    let (uniq, _, _) = reconstruct_corpus(Path::new(DEFAULT_DOCS));
    let mut dev: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Dev)
        .collect();
    dev.sort_by_key(|i| uniq[*i].sha256);
    if dev.len() < 3 * TEXT_DOCS {
        return Err("not enough Dev documents for fit/tune/final reader splits".into());
    }
    let fit_docs: Vec<usize> = dev[0..TEXT_DOCS].to_vec();
    let tune_docs: Vec<usize> = dev[TEXT_DOCS..2 * TEXT_DOCS].to_vec();
    // The previously inspected reader text is development evidence; the new final documents are disjoint.
    let prev_final_docs: Vec<usize> = dev[2 * TEXT_DOCS..3 * TEXT_DOCS].to_vec();
    let final_docs: Vec<usize> = dev[3 * TEXT_DOCS..4 * TEXT_DOCS].to_vec();
    if dev.len() < 4 * TEXT_DOCS {
        return Err("not enough Dev documents for fit/tune/previous/final reader splits".into());
    }
    for (a, b) in [
        (&fit_docs, &tune_docs),
        (&fit_docs, &final_docs),
        (&tune_docs, &final_docs),
        (&fit_docs, &prev_final_docs),
        (&prev_final_docs, &final_docs),
    ] {
        for x in a.iter() {
            for y in b.iter() {
                if uniq[*x].path == uniq[*y].path || uniq[*x].sha256 == uniq[*y].sha256 {
                    return Err("reader text document splits overlap".into());
                }
            }
        }
    }
    let doc_id = |i: usize| json!({"path": uniq[i].path, "sha256": hex_of(&uniq[i].sha256), "bytes": uniq[i].bytes});

    // Preparation probe, then a window count that clears the declared minimum information target.
    let probe_encode_start = Instant::now();
    let probe_tokens = tokenizer.encode(&uniq[fit_docs[0]].text);
    let probe_encode_s = probe_encode_start.elapsed().as_secs_f64().max(1e-9);
    let probe_windows = windows_of(&probe_tokens);
    let probe_observe_start = Instant::now();
    let probe_cand: usize = probe_windows
        .iter()
        .take(WINDOWS_PER_DOC)
        .map(|w| {
            let seq = Seq {
                tokens: w.clone(),
                group: 0xF000,
                answer: 0,
                absent: false,
                answer_source_abs: None,
            };
            observe_full(&seq)
                .iter()
                .filter(|o| !o.cands.is_empty())
                .count()
        })
        .sum();
    let probe_observe_s = probe_observe_start.elapsed().as_secs_f64().max(1e-9);
    let per_window_cand = probe_cand as f64 / WINDOWS_PER_DOC.max(1) as f64;
    let mut per_doc = WINDOWS_PER_DOC;
    while (per_doc as f64 * per_window_cand * TEXT_DOCS as f64) < MIN_TEXT_FIT_POSITIONS as f64
        && per_doc < probe_windows.len().max(WINDOWS_PER_DOC)
    {
        per_doc += 1;
    }
    let preparation_probe = json!({
        "fit_document": uniq[fit_docs[0]].path,
        "encode_seconds": probe_encode_s,
        "tokens": probe_tokens.len(),
        "windows_available": probe_windows.len(),
        "observe_seconds": probe_observe_s,
        "candidate_positions_per_window": per_window_cand,
        "declared_minimum_text_fit_positions": MIN_TEXT_FIT_POSITIONS,
        "chosen_windows_per_document": per_doc,
        "rationale": "declared minimum information target plus measured observe rate; no large corpus download",
    });

    let encode_doc = |i: usize| -> Vec<u32> { tokenizer.encode(&uniq[i].text) };
    let build_streams = |docs: &[usize], per: usize| -> Vec<(usize, Vec<u32>)> {
        let mut v = Vec::new();
        for i in docs.iter() {
            for w in windows_of(&encode_doc(*i)).into_iter().take(per) {
                v.push((*i, w));
            }
        }
        v
    };
    let fit_streams = build_streams(&fit_docs, per_doc);
    let tune_streams = build_streams(&tune_docs, per_doc);
    let final_streams = build_streams(&final_docs, per_doc);
    let prev_final_streams = build_streams(&prev_final_docs, per_doc);
    let to_seqs = |s: &[(usize, Vec<u32>)], group: u16| -> Vec<Seq> {
        s.iter()
            .map(|(_, t)| Seq {
                tokens: t.clone(),
                group,
                answer: 0,
                absent: false,
                answer_source_abs: None,
            })
            .collect()
    };
    let fit_text_seqs = to_seqs(&fit_streams, 0xF000);
    let tune_text_seqs = to_seqs(&tune_streams, 0xF100);
    let final_text_seqs = to_seqs(&final_streams, 0xF200);
    let prev_final_seqs = to_seqs(&prev_final_streams, 0xF300);
    let fit_text_obs = observe_full_all(&fit_text_seqs);
    let tune_text_obs = observe_full_all(&tune_text_seqs);
    let final_text_obs = observe_full_all(&final_text_seqs);
    let prev_final_obs = observe_full_all(&prev_final_seqs);
    // Per-*position* document index for document-cluster intervals (one document per window).
    let per_position_docs = |streams: &[(usize, Vec<u32>)], obs: &[Vec<Obs>]| -> Vec<usize> {
        streams
            .iter()
            .zip(obs.iter())
            .flat_map(|((d, _), os)| std::iter::repeat(*d).take(os.len()))
            .collect()
    };
    let fit_text_doc_pos = per_position_docs(&fit_streams, &fit_text_obs);
    let final_text_doc_pos = per_position_docs(&final_streams, &final_text_obs);
    let prev_final_doc_pos = per_position_docs(&prev_final_streams, &prev_final_obs);
    let doc_of = |s: &[(usize, Vec<u32>)]| -> Vec<usize> { s.iter().map(|(d, _)| *d).collect() };
    let _fit_text_docs = doc_of(&fit_streams);
    let tune_text_docs = doc_of(&tune_streams);
    let _ = &tune_text_docs;
    let final_text_docs = doc_of(&final_streams);
    let text_counts = |obs: &[Vec<Obs>]| -> (usize, usize) {
        let n: usize = obs.iter().map(|v| v.len()).sum();
        let cand: usize = obs.iter().flatten().filter(|o| !o.cands.is_empty()).count();
        (n, cand)
    };
    let (fit_text_n, fit_text_cand) = text_counts(&fit_text_obs);
    let (tune_text_n, tune_text_cand) = text_counts(&tune_text_obs);
    let (final_text_n, final_text_cand) = text_counts(&final_text_obs);
    let text_split = json!({
        "fit_documents": fit_docs.iter().map(|i| doc_id(*i)).collect::<Vec<_>>(),
        "tune_documents": tune_docs.iter().map(|i| doc_id(*i)).collect::<Vec<_>>(),
        "final_documents": final_docs.iter().map(|i| doc_id(*i)).collect::<Vec<_>>(),
        "windows_per_document": per_doc,
        "disjoint_by_path_and_content_hash": true,
        "fit_windows": fit_streams.len(),
        "fit_positions": fit_text_n,
        "fit_candidate_positions": fit_text_cand,
        "tune_positions": tune_text_n,
        "tune_candidate_positions": tune_text_cand,
        "final_positions": final_text_n,
        "final_candidate_positions": final_text_cand,
        "minimum_information_target_met": fit_text_cand >= MIN_TEXT_FIT_POSITIONS,
        "scope": "The frozen local prior was trained on this pinned corpus; that exposure is separate and is not removed. Reader-held-out transfer, not necessarily externally unseen language.",
    });
    mark("text split + prep probe", &mut marks);

    // ---- declared gap thresholds from the TUNE mixture --------------------------
    let mut tune_gaps: Vec<i32> = Vec::new();
    for (obs, sel) in [
        (&tune_obs, &relational_base),
        (&tune_text_obs, &relational_base),
    ] {
        for os in obs.iter() {
            for o in os.iter() {
                if o.cands.is_empty() {
                    continue;
                }
                let z = local_logits(
                    &parent,
                    &local,
                    &u,
                    o.cur,
                    o.prev as usize,
                    o.prev2,
                    o.ring.written(),
                );
                let rel = rels_for(sel, o, &table, false);
                if let Some(g) = sel.top_payload_gap(&o.cands, &rel, &z) {
                    tune_gaps.push(g);
                }
            }
        }
    }
    let gap_thresholds = choose_gap_thresholds(&mut tune_gaps);
    // **The feature contract is fixed before any event exists.** Both the fit-time and the serve-time
    // bucket partition are this one configuration, so they cannot drift apart.
    let policy_cfg = PolicyConfig::new(gap_thresholds);
    let policy_cfg_digest = policy_cfg.digest();
    mark("gap thresholds + feature contract", &mut marks);

    // ---- direct hard-action policy for each frozen source arm -------------------
    let mut fit_report: Vec<serde_json::Value> = Vec::new();
    let mut arm_fit_meta: BTreeMap<&'static str, (Vec<f64>, usize)> = BTreeMap::new();
    let mut policies: BTreeMap<&'static str, RelationalSelector> = BTreeMap::new();
    let mut contract_buckets: BTreeMap<&'static str, serde_json::Value> = BTreeMap::new();
    for (name, base) in [
        ("h4_policy", &relational_base),
        ("categorical_policy", &categorical_base),
    ] {
        let t0 = Instant::now();
        // The configured selector is used for event extraction, not the raw legacy parent.
        let configured = policy_cfg.apply(base);
        let const_ev: Vec<PolicyEvent> = fit_obs
            .iter()
            .flatten()
            .filter_map(|o| {
                policy_event(
                    &parent,
                    &local,
                    &u,
                    &table,
                    &configured,
                    &policy_cfg,
                    o,
                    1.0,
                )
            })
            .collect();
        let text_raw: Vec<PolicyEvent> = fit_text_obs
            .iter()
            .flatten()
            .filter_map(|o| {
                policy_event(
                    &parent,
                    &local,
                    &u,
                    &table,
                    &configured,
                    &policy_cfg,
                    o,
                    1.0,
                )
            })
            .collect();
        // Declared weight: construction and natural text carry equal total weight.
        let w_text = if text_raw.is_empty() {
            0.0
        } else {
            const_ev.len() as f64 / text_raw.len() as f64
        };
        let mut events = const_ev.clone();
        for mut e in text_raw {
            e.weight = w_text;
            events.push(e);
        }
        let (policy, fit) = fit_policy(&events);
        arm_fit_meta.insert(name, (fit.support.clone(), fit.global_action));
        let mut sel = configured.clone();
        sel.policy = policy.clone();
        sel.validate_for_vocab(parent.cfg.vocab)?;
        sel.verify_policy_contract(&policy_cfg)?;
        let mut action_hist = [0usize; ACTS + 1];
        for code in policy.iter() {
            action_hist[*code as usize] += 1;
        }
        // Per-bucket fit occupancy, and the occupancy the served addresses actually address.
        let mut raw_examples = vec![0usize; UTIL_BUCKETS];
        for e in events.iter() {
            raw_examples[e.bucket.min(UTIL_BUCKETS - 1)] += 1;
        }
        contract_buckets.insert(
            name,
            json!({
                "buckets_with_examples": (0..UTIL_BUCKETS).filter(|b| raw_examples[*b] > 0).count(),
                "buckets_unobserved": (0..UTIL_BUCKETS).filter(|b| raw_examples[*b] == 0).count(),
                "buckets_below_min_support": fit.support.iter().filter(|s| **s > 0.0 && **s < UTIL_MIN_SUPPORT).count(),
                "buckets_supported": fit.support.iter().filter(|s| **s >= UTIL_MIN_SUPPORT).count(),
                "raw_examples_per_bucket": raw_examples,
                "weighted_support_per_bucket": fit.support.clone(),
            }),
        );
        fit_report.push(json!({
            "arm": name,
            "base_arm": if name == "h4_policy" { "relational" } else { "categorical" },
            "seconds": t0.elapsed().as_secs_f64(),
            "feature_contract": {
                "bucket_formula": POLICY_BUCKET_FORMULA,
                "gap_units": POLICY_GAP_UNITS,
                "comparison": POLICY_COMPARISON,
                "context_bits": POLICY_CONTEXT_BITS,
                "margin_rule": POLICY_MARGIN_RULE,
                "action_encoding": POLICY_ACTION_ENCODING,
                "gap_thresholds": gap_thresholds,
                "config_digest": hex_of(&policy_cfg_digest),
                "configured_before_events": true,
            },
            "construction_events": const_ev.len(),
            "text_events": events.len() - const_ev.len(),
            "declared_text_weight": w_text,
            "total_declared_weight": fit.total_weight,
            "global_action": fit.global_action,
            "buckets_with_own_action": fit.buckets_with_own_action,
            "fallback_buckets": fit.fallback_buckets,
            "bucket_action_histogram": action_hist,
            "bucket_mean_cost": fit.mean_cost,
            "policy_actions": policy,
        }));
        policies.insert(name, sel);
    }
    let h4_policy = policies["h4_policy"].clone();
    let categorical_policy = policies["categorical_policy"].clone();
    // The fixed generic comparator, on the same ungated source and the same configured contract:
    // every bucket applies the one-nat strength (opcode 2). It tests whether the fitted table adds
    // anything beyond a constant small boost.
    let mut h4_one_nat = policy_cfg.apply(&relational_base);
    h4_one_nat.policy = vec![2u8; UTIL_BUCKETS];
    h4_one_nat.validate_for_vocab(parent.cfg.vocab)?;
    h4_one_nat.verify_policy_contract(&policy_cfg)?;
    mark("direct policy fit", &mut marks);
    // ---- joint finite-policy feasibility ----------------------------------------
    // One development extraction of the counterfactual statistics the executed-policy aggregates
    // cannot supply, then a bounded constrained solve. The declared operational class uses the same
    // support mask and global fallback opcode as the fitted table; a wider-class diagnostic frees
    // every bucket and is labelled separately.
    let feas_t0 = Instant::now();
    let configured_h4 = policy_cfg.apply(&relational_base);
    let configured_cat = policy_cfg.apply(&categorical_base);
    let mut parent_fit = ParentAgg::default();
    accumulate_parent(
        &parent,
        &local,
        &u,
        &table,
        &relational_ctx_base,
        &fit_obs,
        &fit,
        &mut parent_fit,
    );
    let mut parent_tune = ParentAgg::default();
    accumulate_parent(
        &parent,
        &local,
        &u,
        &table,
        &relational_ctx_base,
        &tune_obs,
        &tune,
        &mut parent_tune,
    );
    let tune_text_doc_pos = per_position_docs(&tune_streams, &tune_text_obs);
    let mut feas_arms: Vec<serde_json::Value> = Vec::new();
    let mut feasibility_selection = json!(null);
    for (name, configured) in [
        ("h4_policy", &configured_h4),
        ("categorical_policy", &configured_cat),
    ] {
        let (support, fallback) = arm_fit_meta
            .get(name)
            .cloned()
            .unwrap_or((vec![0.0f64; UTIL_BUCKETS], 0usize));
        let mut fit_stats = FeasStats::new(UTIL_BUCKETS);
        accumulate_feas(
            &parent,
            &local,
            &u,
            &table,
            &policy_cfg,
            configured,
            &fit_obs,
            &fit,
            "construction",
            None,
            false,
            &mut fit_stats,
        )?;
        accumulate_feas(
            &parent,
            &local,
            &u,
            &table,
            &policy_cfg,
            configured,
            &fit_text_obs,
            &fit_text_seqs,
            "text",
            Some(&fit_text_doc_pos),
            false,
            &mut fit_stats,
        )?;
        let mut tune_stats = FeasStats::new(UTIL_BUCKETS);
        accumulate_feas(
            &parent,
            &local,
            &u,
            &table,
            &policy_cfg,
            configured,
            &tune_obs,
            &tune,
            "construction",
            None,
            false,
            &mut tune_stats,
        )?;
        accumulate_feas(
            &parent,
            &local,
            &u,
            &table,
            &policy_cfg,
            configured,
            &tune_text_obs,
            &tune_text_seqs,
            "text",
            Some(&tune_text_doc_pos),
            false,
            &mut tune_stats,
        )?;

        let p_count = fit_stats.present_positions;
        let margin_count = (2.0 * p_count as f64 / 121.0).ceil();
        let present_loss_margin = MARGIN_PRESENT_BITS * p_count as f64;
        let obj: Vec<[f64; ACTS + 1]> = (0..UTIL_BUCKETS).map(|b| fit_stats.text_bits[b]).collect();
        let cons = vec![
            ConSpec {
                name: "present_emitted_correct",
                coeff: fit_stats.present_correct.clone(),
                sense: ConSense::AtLeast,
                rhs: parent_fit.present_correct - margin_count,
                tol: 1e-6,
            },
            ConSpec {
                name: "present_delta_bits",
                coeff: fit_stats.present_delta.clone(),
                sense: ConSense::AtMost,
                rhs: parent_fit.present_delta + present_loss_margin,
                tol: 1e-6,
            },
            ConSpec {
                name: "absent_reads",
                coeff: fit_stats.absent_read.clone(),
                sense: ConSense::AtMost,
                rhs: parent_fit.absent_read,
                tol: 1e-6,
            },
            ConSpec {
                name: "absent_delta_bits",
                coeff: fit_stats.absent_delta.clone(),
                sense: ConSense::AtMost,
                rhs: parent_fit.absent_delta,
                tol: 1e-6,
            },
        ];
        let fixed: Vec<Option<usize>> = (0..UTIL_BUCKETS)
            .map(|b| {
                if support[b] >= UTIL_MIN_SUPPORT {
                    None
                } else {
                    Some(fallback)
                }
            })
            .collect();
        let problem = FeasProblem {
            obj: obj.clone(),
            cons: cons.clone(),
            fixed: fixed.clone(),
            support: support.clone(),
        };
        let sol = solve_feasible(&problem, 45.0, 4_000_000);
        let mut wide = problem.clone();
        for f in wide.fixed.iter_mut() {
            *f = None;
        }
        let wide_sol = solve_feasible(&wide, 45.0, 4_000_000);

        let fit_tokens: usize = fit_text_obs.iter().map(|v| v.len()).sum();
        let tune_tokens: usize = tune_text_obs.iter().map(|v| v.len()).sum();
        let one_nat = vec![2usize; UTIL_BUCKETS];
        let text_per_token = sol.obj / fit_tokens.max(1) as f64;
        let tune_text_per_token =
            pick_mat(&tune_stats.text_bits, &sol.actions) / tune_tokens.max(1) as f64;
        let doc_sums = fit_stats.doc_pick(&sol.actions);
        let report_actions: Vec<usize> = if sol.feasible {
            sol.actions.clone()
        } else {
            Vec::new()
        };
        let realized_fit_json = if sol.feasible {
            json!({
                "text_delta_bits": pick_mat(&fit_stats.text_bits, &sol.actions),
                "present_emitted_correct": pick_mat(&fit_stats.present_correct, &sol.actions),
                "present_delta_bits": pick_mat(&fit_stats.present_delta, &sol.actions),
                "absent_reads": pick_mat(&fit_stats.absent_read, &sol.actions),
                "absent_delta_bits": pick_mat(&fit_stats.absent_delta, &sol.actions),
            })
        } else {
            json!(null)
        };
        let realized_tune_json = if sol.feasible {
            json!({
                "text_bits_per_token": tune_text_per_token,
                "present_emitted_correct": pick_mat(&tune_stats.present_correct, &sol.actions),
                "present_delta_bits": pick_mat(&tune_stats.present_delta, &sol.actions),
                "absent_reads": pick_mat(&tune_stats.absent_read, &sol.actions),
                "absent_delta_bits": pick_mat(&tune_stats.absent_delta, &sol.actions),
            })
        } else {
            json!(null)
        };
        let doc_sums_json: Vec<serde_json::Value> = if sol.feasible {
            doc_sums
                .iter()
                .map(|(d, v)| json!({"document": d, "delta_bits": v}))
                .collect()
        } else {
            Vec::new()
        };
        feas_arms.push(json!({
            "arm": name,
            "declared_class": {
                "supported_buckets": (0..UTIL_BUCKETS).filter(|b| support[*b] >= UTIL_MIN_SUPPORT).count(),
                "fallback_opcode": fallback,
                "free_buckets": fixed.iter().filter(|f| f.is_none()).count(),
            },
            "development_populations": {
                "text_tokens": fit_tokens,
                "text_candidate_positions": fit_stats.text_candidate_positions,
                "present_positions": fit_stats.present_positions,
                "absent_positions": fit_stats.absent_positions,
                "tune_text_tokens": tune_tokens,
                "tune_present_positions": tune_stats.present_positions,
                "tune_absent_positions": tune_stats.absent_positions,
            },
            "parent_reference": {
                "present_emitted_correct": parent_fit.present_correct,
                "present_delta_bits": parent_fit.present_delta,
                "absent_reads": parent_fit.absent_read,
                "absent_delta_bits": parent_fit.absent_delta,
                "tune_present_emitted_correct": parent_tune.present_correct,
                "tune_absent_reads": parent_tune.absent_read,
            },
            "margins": {"present_count_margin": margin_count, "present_loss_margin_bits": present_loss_margin},
            "solve": {
                "status": sol.status,
                "feasible": sol.feasible,
                "objective_bits": sol.obj,
                "lower_bound_bits": sol.lower_bound,
                "text_bits_per_token": text_per_token,
                "screen_bits_per_token": MARGIN_TEXT_BITS,
                "meets_text_screen": text_per_token <= MARGIN_TEXT_BITS,
                "improves_over_local": sol.obj < 0.0,
                "nodes": sol.nodes,
                "exhaustive": sol.exhaustive,
                "seconds": sol.seconds,
                "actions": report_actions,
                "realized_fit": realized_fit_json,
                "realized_tune": realized_tune_json,
                "document_text_bits": doc_sums_json,
                "obstruction_proved_by_bound": sol.lower_bound / fit_tokens.max(1) as f64 > MARGIN_TEXT_BITS,
            },
            "wider_class_relaxed_fallback": {
                "status": wide_sol.status,
                "feasible": wide_sol.feasible,
                "objective_bits": wide_sol.obj,
                "lower_bound_bits": wide_sol.lower_bound,
                "text_bits_per_token": wide_sol.obj / fit_tokens.max(1) as f64,
            },
            "constant_one_nat_reference": {
                "text_bits_per_token": pick_mat(&fit_stats.text_bits, &one_nat) / fit_tokens.max(1) as f64,
                "present_emitted_correct": pick_mat(&fit_stats.present_correct, &one_nat),
                "absent_reads": pick_mat(&fit_stats.absent_read, &one_nat),
            },
            "identity_max_residual_bits": fit_stats.max_resid.max(tune_stats.max_resid),
            "counterfactual_positions": fit_stats.counterfactual_positions + tune_stats.counterfactual_positions,
            "statistics": {
                "unit": "bits per (bucket, action); present/absent over the fit construction population; text over the fit reader documents",
                "buckets": UTIL_BUCKETS,
                "actions": {"0": "NoRead", "1": "0.0625 nats", "2": "1 nat", "3": "8 nats"},
                "text_delta_bits": fit_stats.text_bits,
                "present_emitted_correct": fit_stats.present_correct,
                "present_delta_bits": fit_stats.present_delta,
                "absent_reads": fit_stats.absent_read,
                "absent_delta_bits": fit_stats.absent_delta,
            },
        }));
        if name == "h4_policy" {
            feasibility_selection = json!({
                "arm": name,
                "status": sol.status,
                "feasible": sol.feasible,
                "actions": report_actions,
                "text_bits_per_token": text_per_token,
                "lower_bound_bits": sol.lower_bound,
                "meets_text_screen": text_per_token <= MARGIN_TEXT_BITS,
                "obstruction_proved_by_bound": sol.lower_bound / fit_tokens.max(1) as f64 > MARGIN_TEXT_BITS,
                "tune_text_bits_per_token": sol.feasible.then_some(tune_text_per_token),
                "wider_class_feasible": wide_sol.feasible,
                "wider_class_text_bits_per_token": wide_sol.obj / fit_tokens.max(1) as f64,
            });
        }
    }
    let feasibility_report = json!({
        "design": "minimize complete-stream development natural-text loss (bits/token on the complete fit text stream) subject to present emitted-correct >= parent - translated margin, present CE <= parent + 0.05 bits/position, absent reads <= parent, absent CE <= parent; supported buckets free, sparse/unseen buckets fixed to the declared global fallback opcode",
        "development": "fit construction (SEED_FIT) + fit reader documents; the tune pools are held-out development checks",
        "solver": "Lagrangian lower bound (coordinate ascent) + bounded branch-and-bound with node and wall-time caps",
        "screen_bits_per_token": MARGIN_TEXT_BITS,
        "identity": "delta_loss(a) = log(1 + p*(exp(a)-1)) - a*1[payload==target], asserted against the actual integer logits at every position",
        "seconds": feas_t0.elapsed().as_secs_f64(),
        "arms": feas_arms,
        "selection": feasibility_selection,
        "scope": "Development feasibility of the declared class only; not generalization. Teacher-forced additivity does not extend to rollout. A solver timeout is UNRESOLVED, not INFEASIBLE.",
    });
    eprintln!("feasibility: {} s", feas_t0.elapsed().as_secs_f32());
    mark("joint feasibility", &mut marks);

    // ---- export, independent reload; the RELOADED arms are what get exercised ----
    // Acyclic binding: the artifact carries the digest of the **fit inputs** it was produced from; the
    // surrounding delivery manifest carries each artifact's byte hash. The config digest depends only
    // on the declared feature semantics plus the thresholds, so no hash cycle exists.
    let mut fit_input_bytes = Vec::new();
    for o in fit_obs
        .iter()
        .flatten()
        .chain(fit_text_obs.iter().flatten())
    {
        fit_input_bytes.extend_from_slice(&o.cur.to_le_bytes());
        fit_input_bytes.extend_from_slice(&o.prev.to_le_bytes());
        fit_input_bytes.extend_from_slice(&o.prev2.to_le_bytes());
        fit_input_bytes.extend_from_slice(&o.target.to_le_bytes());
        fit_input_bytes.extend_from_slice(&(o.cands.len() as u32).to_le_bytes());
        for c in o.cands.iter() {
            fit_input_bytes.extend_from_slice(&c.payload.to_le_bytes());
            fit_input_bytes.extend_from_slice(&c.feats);
        }
    }
    let fit_input_digest = sha256_bytes(&fit_input_bytes);
    let mut export_bytes: BTreeMap<&'static str, Vec<u8>> = BTreeMap::new();
    let mut reloaded: BTreeMap<&'static str, RelationalSelector> = BTreeMap::new();
    let mut reload_failures = 0usize;
    for (name, sel) in [
        ("h4_policy", &h4_policy),
        ("categorical_policy", &categorical_policy),
    ] {
        let a = RelationalArtifact {
            selector: sel.clone(),
            local_artifact_digest: e_digest,
            tokenizer_digest: raw_tok,
            data_digest: fit_input_digest,
        };
        let bytes = a.to_bytes();
        write_checked(&root, &format!("artifacts/{name}.rlr2"), &bytes)?;
        // The verified loader contract, exercised **before any prediction**.
        match RelationalArtifact::from_bytes(&bytes, &e_digest, &raw_tok) {
            Ok(b) => {
                let mut ok = b.selector == *sel
                    && b.selector.validate_for_vocab(parent.cfg.vocab).is_ok()
                    && b.selector.verify_policy_contract(&policy_cfg).is_ok()
                    && b.data_digest == fit_input_digest;
                // Rejections the consumer must actually perform.
                let changed_cfg = PolicyConfig::new([
                    gap_thresholds[0] - 64,
                    gap_thresholds[1],
                    gap_thresholds[2],
                ]);
                if b.selector.verify_policy_contract(&changed_cfg).is_ok() {
                    ok = false;
                }
                let mut swapped = b.selector.clone();
                // Swap two entries that actually differ, so the byte comparison is meaningful.
                if let Some(j) = (0..UTIL_BUCKETS).find(|j| swapped.policy[*j] != swapped.policy[0])
                {
                    swapped.policy.swap(0, j);
                    let bytes_swapped = RelationalArtifact {
                        selector: swapped,
                        local_artifact_digest: e_digest,
                        tokenizer_digest: raw_tok,
                        data_digest: fit_input_digest,
                    }
                    .to_bytes();
                    if bytes_swapped == bytes {
                        ok = false;
                    }
                }
                let mut wrong_data = sha256_bytes(b"not-the-fit-inputs");
                if wrong_data == fit_input_digest {
                    wrong_data[0] ^= 1;
                }
                if RelationalArtifact::from_bytes(&bytes, &e_digest, &raw_tok)
                    .map(|x| x.data_digest == wrong_data)
                    .unwrap_or(false)
                {
                    ok = false;
                }
                if !ok {
                    reload_failures += 1;
                }
                reloaded.insert(name, b.selector);
            }
            Err(_) => reload_failures += 1,
        }
        export_bytes.insert(name, bytes);
    }
    let _h4_load = reloaded["h4_policy"].clone();
    let _cat_load = reloaded["categorical_policy"].clone();
    // Exercise the one-nat comparator through the same configured contract.
    let one_nat_bytes = RelationalArtifact {
        selector: h4_one_nat.clone(),
        local_artifact_digest: e_digest,
        tokenizer_digest: raw_tok,
        data_digest: fit_input_digest,
    }
    .to_bytes();
    write_checked(&root, "artifacts/h4_one_nat.rlr2", &one_nat_bytes)?;
    let one_nat_load =
        RelationalArtifact::from_bytes(&one_nat_bytes, &e_digest, &raw_tok)?.selector;
    // ---- expected-manifest verification of the served artifacts ------------------
    // The consumer verifies actual bytes, fit-input identity and the configured feature contract
    // *before* it returns a predictor. Every rejection is exercised through that consumer, not by a
    // byte comparison or a dummy digest.
    let mut verify_failures = 0usize;
    let mut verified_arms: BTreeMap<&'static str, RelationalSelector> = BTreeMap::new();
    for (name, bytes) in export_bytes.iter() {
        let exp = ExpectedArtifact {
            name: (*name).to_string(),
            bytes_sha256: artifact_sha256(bytes),
            data_digest: fit_input_digest,
        };
        match load_expected_artifact(
            &root,
            &exp,
            &e_digest,
            &raw_tok,
            &policy_cfg,
            parent.cfg.vocab,
        ) {
            Ok(sel) => {
                let fitted = if *name == "h4_policy" {
                    &h4_policy
                } else {
                    &categorical_policy
                };
                if sel != *fitted {
                    verify_failures += 1;
                }
                verified_arms.insert(*name, sel);
            }
            Err(e) => {
                eprintln!("verified load failed for {name}: {e}");
                verify_failures += 1;
            }
        }
    }
    let one_nat_exp = ExpectedArtifact {
        name: "h4_one_nat".to_string(),
        bytes_sha256: artifact_sha256(&one_nat_bytes),
        data_digest: fit_input_digest,
    };
    if load_expected_artifact(
        &root,
        &one_nat_exp,
        &e_digest,
        &raw_tok,
        &policy_cfg,
        parent.cfg.vocab,
    )
    .is_err()
    {
        verify_failures += 1;
    }
    // Rejection probes the consumer must actually perform.
    let base_exp = ExpectedArtifact {
        name: "h4_policy".to_string(),
        bytes_sha256: artifact_sha256(&export_bytes["h4_policy"]),
        data_digest: fit_input_digest,
    };
    let drifted_cfg =
        PolicyConfig::new([gap_thresholds[0] - 64, gap_thresholds[1], gap_thresholds[2]]);
    if load_expected_artifact(
        &root,
        &base_exp,
        &e_digest,
        &raw_tok,
        &drifted_cfg,
        parent.cfg.vocab,
    )
    .is_ok()
    {
        verify_failures += 1;
    }
    let mut wrong_hash = base_exp.clone();
    wrong_hash.bytes_sha256 = "0".repeat(64);
    if load_expected_artifact(
        &root,
        &wrong_hash,
        &e_digest,
        &raw_tok,
        &policy_cfg,
        parent.cfg.vocab,
    )
    .is_ok()
    {
        verify_failures += 1;
    }
    let mut wrong_digest = base_exp.clone();
    wrong_digest.data_digest = [0xABu8; 32];
    if load_expected_artifact(
        &root,
        &wrong_digest,
        &e_digest,
        &raw_tok,
        &policy_cfg,
        parent.cfg.vocab,
    )
    .is_ok()
    {
        verify_failures += 1;
    }
    // A truncated artifact must fail structural load even when its own hash is the expected one.
    let tmp_root = std::env::temp_dir().join(format!("uor-feas-verify-{}", std::process::id()));
    let _ = std::fs::create_dir_all(tmp_root.join("artifacts"));
    let mut tampered = export_bytes["h4_policy"].clone();
    tampered.truncate(tampered.len().saturating_sub(4));
    if std::fs::write(tmp_root.join("artifacts/h4_policy.rlr2"), &tampered).is_ok() {
        let tamper_exp = ExpectedArtifact {
            name: "h4_policy".to_string(),
            bytes_sha256: artifact_sha256(&tampered),
            data_digest: fit_input_digest,
        };
        if load_expected_artifact(
            &tmp_root,
            &tamper_exp,
            &e_digest,
            &raw_tok,
            &policy_cfg,
            parent.cfg.vocab,
        )
        .is_ok()
        {
            verify_failures += 1;
        }
    }
    let _ = std::fs::remove_dir_all(&tmp_root);
    controls.push(json!({
        "control": "expected_manifest_loader",
        "arms": ["h4_policy", "categorical_policy", "h4_one_nat"],
        "failures": verify_failures,
        "checks": ["artifact byte hash", "fit-input identity", "configured feature contract", "vocabulary"],
        "rejection_probes": ["drifted gap thresholds", "wrong expected byte hash", "wrong expected fit-input identity", "truncated artifact bytes"],
        "note": "the consumer verifies bytes, fit-input identity and the feature contract before returning a predictor; all four rejections are exercised through that consumer",
    }));
    reload_failures += verify_failures;
    let h4_load = verified_arms["h4_policy"].clone();
    let cat_load = verified_arms["categorical_policy"].clone();
    // The served predictor is the one the expected-manifest consumer actually returned.
    let one_nat_load = match load_expected_artifact(
        &root,
        &one_nat_exp,
        &e_digest,
        &raw_tok,
        &policy_cfg,
        parent.cfg.vocab,
    ) {
        Ok(sel) => sel,
        Err(e) => {
            eprintln!("verified one-nat load failed: {e}");
            reload_failures += 1;
            one_nat_load
        }
    };
    if one_nat_load != h4_one_nat {
        reload_failures += 1;
    }
    export_bytes.insert("h4_one_nat", one_nat_bytes);
    mark("export + reload", &mut marks);
    // ---- learned read-confidence influence interface ----------------------------
    // Restore the parent's learned Read/NoRead decision at the influence boundary: the address is the
    // five-bit utility bucket plus the sign of the parent's own causal integer advantage D. The
    // witnessed parent rule is an explicit candidate and the unsupported-entry fallback; a compact
    // learned table may trade dose while preserving the parent-preserving construction witness.
    let conf_t0 = Instant::now();
    let configured_ctx = policy_cfg.apply(&relational_ctx_base);
    let mut conf_arms: Vec<serde_json::Value> = Vec::new();
    let mut conf_tables: BTreeMap<&'static str, (RelationalSelector, f64)> = BTreeMap::new();
    let mut conf_fit_bytes: Vec<u8> = Vec::new();
    conf_fit_bytes.extend_from_slice(&policy_cfg_digest);
    for (name, configured, parent_sel) in [
        ("h4_confidence", &configured_ctx, &relational_ctx_base),
        ("categorical_confidence", &configured_cat, &categorical_base),
    ] {
        let witness = witness_parity(
            configured, parent_sel, &parent, &local, &u, &table, &fit_obs,
        )?;
        let mut fit_stats = FeasStats::new(CONF_ADDRESSES);
        accumulate_feas(
            &parent,
            &local,
            &u,
            &table,
            &policy_cfg,
            configured,
            &fit_obs,
            &fit,
            "construction",
            None,
            true,
            &mut fit_stats,
        )?;
        accumulate_feas(
            &parent,
            &local,
            &u,
            &table,
            &policy_cfg,
            configured,
            &fit_text_obs,
            &fit_text_seqs,
            "text",
            Some(&fit_text_doc_pos),
            true,
            &mut fit_stats,
        )?;
        let mut tune_stats = FeasStats::new(CONF_ADDRESSES);
        accumulate_feas(
            &parent,
            &local,
            &u,
            &table,
            &policy_cfg,
            configured,
            &tune_obs,
            &tune,
            "construction",
            None,
            true,
            &mut tune_stats,
        )?;
        accumulate_feas(
            &parent,
            &local,
            &u,
            &table,
            &policy_cfg,
            configured,
            &tune_text_obs,
            &tune_text_seqs,
            "text",
            Some(&tune_text_doc_pos),
            true,
            &mut tune_stats,
        )?;
        let mut parent_fit = ParentAgg::default();
        accumulate_parent(
            &parent,
            &local,
            &u,
            &table,
            parent_sel,
            &fit_obs,
            &fit,
            &mut parent_fit,
        );
        let mut parent_tune = ParentAgg::default();
        accumulate_parent(
            &parent,
            &local,
            &u,
            &table,
            parent_sel,
            &tune_obs,
            &tune,
            &mut parent_tune,
        );

        let p_count = fit_stats.present_positions;
        let margin_count = (2.0 * p_count as f64 / 121.0).ceil();
        let present_loss_margin = MARGIN_PRESENT_BITS * p_count as f64;
        let obj: Vec<[f64; ACTS + 1]> = (0..CONF_ADDRESSES)
            .map(|b| fit_stats.text_bits[b])
            .collect();
        let cons = vec![
            ConSpec {
                name: "present_emitted_correct",
                coeff: fit_stats.present_correct.clone(),
                sense: ConSense::AtLeast,
                rhs: parent_fit.present_correct - margin_count,
                tol: 1e-6,
            },
            ConSpec {
                name: "present_delta_bits",
                coeff: fit_stats.present_delta.clone(),
                sense: ConSense::AtMost,
                rhs: parent_fit.present_delta + present_loss_margin,
                tol: 1e-6,
            },
            ConSpec {
                name: "absent_reads",
                coeff: fit_stats.absent_read.clone(),
                sense: ConSense::AtMost,
                rhs: parent_fit.absent_read,
                tol: 1e-6,
            },
            ConSpec {
                name: "absent_delta_bits",
                coeff: fit_stats.absent_delta.clone(),
                sense: ConSense::AtMost,
                rhs: parent_fit.absent_delta,
                tol: 1e-6,
            },
        ];
        let fixed: Vec<Option<usize>> = (0..CONF_ADDRESSES)
            .map(|b| {
                if fit_stats.support[b] >= UTIL_MIN_SUPPORT {
                    None
                } else {
                    Some(parent_rule_op(b))
                }
            })
            .collect();
        let problem = FeasProblem {
            obj: obj.clone(),
            cons: cons.clone(),
            fixed: fixed.clone(),
            support: fit_stats.support.clone(),
        };
        let sol = solve_feasible(&problem, 45.0, 6_000_000);
        // The witnessed parent rule is an explicit feasible candidate and the fallback.
        let pr_table = parent_rule_table(CONF_ADDRESSES);
        let pr_feasible = table_feasible(&problem, &pr_table);
        // Select the feasible candidate with the lowest development text loss.
        let mut chosen = pr_table.clone();
        let mut chosen_src = "parent_rule";
        if sol.feasible && (!pr_feasible || sol.obj < table_obj(&problem, &pr_table) - 1e-12) {
            chosen = sol.actions.clone();
            chosen_src = "solver_incumbent";
        }
        let chosen_feasible = table_feasible(&problem, &chosen);
        let fit_tokens: usize = fit_text_obs.iter().map(|v| v.len()).sum();
        let tune_tokens: usize = tune_text_obs.iter().map(|v| v.len()).sum();
        let realized_row = |x: &[usize]| -> (f64, f64, f64, f64, f64) {
            (
                pick_mat(&fit_stats.text_bits, x) / fit_tokens.max(1) as f64,
                pick_mat(&fit_stats.present_correct, x),
                pick_mat(&fit_stats.present_delta, x),
                pick_mat(&fit_stats.absent_read, x),
                pick_mat(&fit_stats.absent_delta, x),
            )
        };
        let (pr_text, pr_pc, pr_pd, pr_ar, pr_ad) = realized_row(&pr_table);
        let (ch_text, ch_pc, ch_pd, ch_ar, ch_ad) = realized_row(&chosen);
        let (_, tu_pc, tu_pd, tu_ar, tu_ad) = (
            0.0,
            pick_mat(&tune_stats.present_correct, &chosen),
            pick_mat(&tune_stats.present_delta, &chosen),
            pick_mat(&tune_stats.absent_read, &chosen),
            pick_mat(&tune_stats.absent_delta, &chosen),
        );
        let tune_text = pick_mat(&tune_stats.text_bits, &chosen) / tune_tokens.max(1) as f64;
        let supported = (0..CONF_ADDRESSES)
            .filter(|b| fit_stats.support[*b] >= UTIL_MIN_SUPPORT)
            .count();
        conf_arms.push(json!({
            "arm": name,
            "addresses": CONF_ADDRESSES,
            "supported_addresses": supported,
            "witness_parity": witness,
            "development_populations": {
                "present_positions": p_count, "absent_positions": fit_stats.absent_positions,
                "text_tokens": fit_tokens, "text_candidate_positions": fit_stats.text_candidate_positions,
                "tune_present_positions": tune_stats.present_positions, "tune_absent_positions": tune_stats.absent_positions,
            },
            "parent_reference": {
                "present_emitted_correct": parent_fit.present_correct,
                "present_delta_bits": parent_fit.present_delta,
                "absent_reads": parent_fit.absent_read,
                "absent_delta_bits": parent_fit.absent_delta,
                "tune_present_emitted_correct": parent_tune.present_correct,
                "tune_absent_reads": parent_tune.absent_read,
            },
            "margins": {"present_count_margin": margin_count, "present_loss_margin_bits": present_loss_margin},
            "solve": {
                "status": sol.status, "feasible": sol.feasible, "exhaustive": sol.exhaustive,
                "nodes": sol.nodes, "seconds": sol.seconds, "lower_bound_bits": sol.lower_bound,
            },
            "selected": {
                "source": chosen_src,
                "feasible": chosen_feasible,
                "table": chosen,
                "text_bits_per_token": ch_text,
                "present_emitted_correct": ch_pc,
                "present_delta_bits": ch_pd,
                "absent_reads": ch_ar,
                "absent_delta_bits": ch_ad,
                "meets_text_screen": ch_text <= MARGIN_TEXT_BITS,
                "improves_over_local": ch_text < 0.0,
                "tune_text_bits_per_token": tune_text,
                "tune_present_emitted_correct": tu_pc,
                "tune_absent_reads": tu_ar,
                "tune_present_delta_bits": tu_pd,
                "tune_absent_delta_bits": tu_ad,
            },
            "witness_table_realized": {
                "text_bits_per_token": pr_text, "present_emitted_correct": pr_pc,
                "present_delta_bits": pr_pd, "absent_reads": pr_ar, "absent_delta_bits": pr_ad,
                "feasible": pr_feasible,
            },
            "identity_max_residual_bits": fit_stats.max_resid.max(tune_stats.max_resid),
            "counterfactual_positions": fit_stats.counterfactual_positions + tune_stats.counterfactual_positions,
        }));
        if chosen_feasible {
            let mut sel = configured.clone();
            sel.policy = chosen.iter().map(|a| *a as u8).collect();
            sel.validate_for_vocab(parent.cfg.vocab)?;
            sel.verify_policy_contract(&policy_cfg)?;
            conf_tables.insert(name, (sel, ch_text));
        }
        // This partial construction-observation digest does not bind text fit inputs, sequence/
        // document boundaries, selected occurrence references or all optimizer configuration.
        for o in fit_obs.iter().flatten() {
            let z = local_logits(
                &parent,
                &local,
                &u,
                o.cur,
                o.prev as usize,
                o.prev2,
                o.ring.written(),
            );
            let rel = rels_for(configured, o, &table, false);
            conf_fit_bytes.extend_from_slice(&o.cur.to_le_bytes());
            conf_fit_bytes.extend_from_slice(&o.prev.to_le_bytes());
            conf_fit_bytes.extend_from_slice(&o.target.to_le_bytes());
            conf_fit_bytes.extend_from_slice(&(o.cands.len() as u32).to_le_bytes());
            if let Some((_top, addr, d)) = configured.confidence_address(&o.cands, &rel, &z) {
                conf_fit_bytes.extend_from_slice(&(addr as u32).to_le_bytes());
                conf_fit_bytes.extend_from_slice(&(u8::from(d > 0)).to_le_bytes());
            }
        }
    }
    let conf_fit_digest = sha256_bytes(&conf_fit_bytes);
    // Export and independently reload the selected confidence operators.
    let mut conf_load: BTreeMap<&'static str, RelationalSelector> = BTreeMap::new();
    for (name, (sel, _tpt)) in conf_tables.iter() {
        let bytes = RelationalArtifact {
            selector: sel.clone(),
            local_artifact_digest: e_digest,
            tokenizer_digest: raw_tok,
            data_digest: conf_fit_digest,
        }
        .to_bytes();
        write_checked(&root, &format!("artifacts/{name}.rlr2"), &bytes)?;
        let exp = ExpectedArtifact {
            name: (*name).to_string(),
            bytes_sha256: artifact_sha256(&bytes),
            data_digest: conf_fit_digest,
        };
        let got = require_matching_confidence_load(
            name,
            sel,
            load_expected_artifact(
                &root,
                &exp,
                &e_digest,
                &raw_tok,
                &policy_cfg,
                parent.cfg.vocab,
            ),
        )?;
        conf_load.insert(name, got);
        export_bytes.insert(name, bytes);
    }
    let h4_confidence_load = required_confidence_arm(&conf_load, "h4_confidence")?;
    let categorical_confidence_load =
        required_confidence_arm(&conf_load, "categorical_confidence")?;
    controls.push(json!({
        "control": "read_confidence_interface",
        "addresses": CONF_ADDRESSES,
        "arms": conf_tables.keys().collect::<Vec<_>>(),
        "reload_failures": reload_failures,
        "note": "address = five-bit utility bucket plus the sign of the retained parent's learned integer advantage D; the witnessed parent rule is the unsupported-entry fallback and an explicit candidate",
    }));

    // Fresh, prospectively declared populations for the one selected evaluation.
    let conf_fresh_pop = make_pop(&banks, N_FRESH, SEED_CONF_FRESH, &banks.values_held);
    for s in conf_fresh_pop.iter() {
        validate_fixture(s, &banks)?;
    }
    let conf_fresh_obs = observe_full_all(&conf_fresh_pop);
    let conf_fresh_docs: Vec<usize> = dev[4 * TEXT_DOCS..].to_vec();
    let conf_fresh_streams = build_streams(&conf_fresh_docs, per_doc);
    let conf_fresh_seqs = to_seqs(&conf_fresh_streams, 0xF400);
    let conf_fresh_text_obs = observe_full_all(&conf_fresh_seqs);
    let conf_fresh_doc_pos = per_position_docs(&conf_fresh_streams, &conf_fresh_text_obs);
    let conf_report = json!({
        "design": "restore the parent's learned Read/NoRead decision at the influence boundary as a sixth address bit; minimize complete-stream development text loss subject to the same present-emission/loss and absent-read/loss constraints, with the witnessed parent rule as candidate and fallback",
        "identity": "D = max_strength strength_score(candidate, relation, strength, parent_bucket) - noread_score(parent_bucket), widened i64; read iff D>0",
        "policy_semantics": "64 opcodes select the confidence interface; 32 opcodes retain the coarse interface; opcode count is serialized in RLR2 v4",
        "binding": {
            "partial_construction_digest": hex_of(&conf_fit_digest),
            "scope": "shared coarse PolicyConfig digest plus construction cur/prev/target/candidate-count/address/D-sign for each fitted arm",
            "not_bound_by_this_digest": ["text fit inputs", "sequence/document boundaries", "selected occurrence and payload references", "full optimizer configuration", "confidence feature semantics"],
        },
        "arms": conf_arms,
        "fresh_population": {
            "construction_seed": SEED_CONF_FRESH,
            "construction_sequences": conf_fresh_pop.len(),
            "text_documents": conf_fresh_docs.len(),
            "note": "previously used Dev documents are development history; only the remaining eligible Dev documents are used and disclosed as a small honest final population",
        },
        "seconds": conf_t0.elapsed().as_secs_f64(),
        "scope": "Development selection plus one declared fresh evaluation; not generalization.",
    });
    mark("read-confidence interface", &mut marks);

    // ---- prepared streams, ring diagnostics, panels -----------------------------
    let prep_fit = prep_stream(&parent, &local, &u, &fit_obs);
    let prep_reg = prep_stream(&parent, &local, &u, &regression_obs);
    let prep_final = prep_stream(&parent, &local, &u, &final_obs);
    let prep_prev = prep_stream(&parent, &local, &u, &prev_obs);
    let prep_fit_text = prep_stream(&parent, &local, &u, &fit_text_obs);
    let prep_final_text = prep_stream(&parent, &local, &u, &final_text_obs);
    let prep_prev_text = prep_stream(&parent, &local, &u, &prev_final_obs);
    let prep_conf_fresh = prep_stream(&parent, &local, &u, &conf_fresh_obs);
    let prep_conf_fresh_text = prep_stream(&parent, &local, &u, &conf_fresh_text_obs);
    mark("prepared streams", &mut marks);

    let arms: Vec<(&'static str, Option<&RelationalSelector>)> = vec![
        ("local", None),
        ("exact_parent", Some(&exact_base)),
        ("relational_parent", Some(&relational_base)),
        ("relational_ctx_parent", Some(&relational_ctx_base)),
        ("categorical_parent", Some(&categorical_base)),
        ("h4_policy", Some(&h4_load)),
        ("categorical_policy", Some(&cat_load)),
        ("h4_one_nat_fixed", Some(&one_nat_load)),
        ("h4_confidence", Some(h4_confidence_load)),
        ("categorical_confidence", Some(categorical_confidence_load)),
    ];
    let mut panels: Vec<serde_json::Value> = Vec::new();
    let mut agg_store: BTreeMap<(String, String), ArmAgg> = BTreeMap::new();
    for (label, kind, seqs, obs, prep, docs) in [
        (
            "construction_fit",
            "construction",
            &fit,
            &fit_obs,
            &prep_fit,
            None,
        ),
        (
            "construction_regression",
            "construction",
            &regression,
            &regression_obs,
            &prep_reg,
            None,
        ),
        (
            "construction_previous_seed",
            "construction",
            &prev_pop,
            &prev_obs,
            &prep_prev,
            None,
        ),
        (
            "construction_final",
            "construction",
            &final_pop,
            &final_obs,
            &prep_final,
            None,
        ),
        (
            "text_fit",
            "text",
            &fit_text_seqs,
            &fit_text_obs,
            &prep_fit_text,
            Some(&fit_text_doc_pos),
        ),
        (
            "text_previous_documents",
            "text",
            &prev_final_seqs,
            &prev_final_obs,
            &prep_prev_text,
            Some(&prev_final_doc_pos),
        ),
        (
            "text_final",
            "text",
            &final_text_seqs,
            &final_text_obs,
            &prep_final_text,
            Some(&final_text_doc_pos),
        ),
        (
            "construction_confidence_fresh",
            "construction",
            &conf_fresh_pop,
            &conf_fresh_obs,
            &prep_conf_fresh,
            None,
        ),
        (
            "text_confidence_fresh",
            "text",
            &conf_fresh_seqs,
            &conf_fresh_text_obs,
            &prep_conf_fresh_text,
            Some(&conf_fresh_doc_pos),
        ),
    ] {
        let mut rows = Vec::new();
        for (name, sel) in arms.iter() {
            let a = eval_stream(
                &parent,
                &local,
                &u,
                &table,
                seqs,
                obs,
                kind,
                docs.map(|d| d.as_slice()),
                *sel,
                prep,
            );
            rows.push(json!({"arm": name, "aggregate": agg_json(&a)}));
            agg_store.insert((label.to_string(), name.to_string()), a);
        }
        panels.push(json!({"panel": label, "kind": kind, "arms": rows}));
    }
    mark("panels", &mut marks);

    // ---- ring diagnostics ------------------------------------------------------
    let ring_row = |label: &str, obs: &[Vec<Obs>], prep: &[PrepPos]| -> serde_json::Value {
        let n = prep.len();
        let eligible = prep.iter().filter(|p| p.ring.0).count();
        let cand = obs.iter().flatten().filter(|o| !o.cands.is_empty()).count();
        json!({
            "stream": label, "positions": n, "ring_eligible_targets": eligible,
            "candidate_bearing_positions": cand,
            "mean_lring_nats_on_candidate_bearing": prep.iter()
                .map(|p| p.ring.1).sum::<f64>() / n.max(1) as f64,
        })
    };
    let mut ring_means: Vec<serde_json::Value> = Vec::new();
    for (label, obs, prep) in [
        ("construction_fit", &fit_obs, &prep_fit),
        ("construction_regression", &regression_obs, &prep_reg),
        ("construction_final", &final_obs, &prep_final),
        ("construction_previous_seed", &prev_obs, &prep_prev),
        ("text_fit", &fit_text_obs, &prep_fit_text),
        ("text_final", &final_text_obs, &prep_final_text),
        ("text_previous_documents", &prev_final_obs, &prep_prev_text),
    ] {
        ring_means.push(ring_row(label, obs, prep));
    }
    let admission_regret = |panel: &str, arm: &str| -> serde_json::Value {
        agg_store
            .get(&(panel.to_string(), arm.to_string()))
            .map(|a| {
                json!({
                    "candidate_only_nats_per_candidate_position": a.admission_regret_cand
                        / a.cand_bearing.max(1) as f64,
                    "complete_stream_nats_per_position": a.admission_regret_complete
                        / a.positions.max(1) as f64,
                    "empty_pool_positions_included": a.empty_pool,
                })
            })
            .unwrap_or(json!(null))
    };
    let ring_diagnostics = json!({
        "unit": "nats; target-using oracle opportunity, never a serving observation or a promised gain",
        "streams": ring_means,
        "admission_regret_definition": "Lpool - Lring >= 0 for EVERY position; an empty admitted pool has Lpool = 0",
        "declared_trigger_nats": ADMISSION_TRIGGER_NATS,
        "declared_trigger_note": "the parent's 0.25 trigger was candidate-only; both denominators are now reported separately and no admission change is made in this run",
        "mean_admission_regret_by_arm": {
            "construction_final_h4_policy": admission_regret("construction_final", "h4_policy"),
            "construction_final_h4_one_nat_fixed": admission_regret("construction_final", "h4_one_nat_fixed"),
            "construction_final_categorical_policy": admission_regret("construction_final", "categorical_policy"),
            "text_final_h4_policy": admission_regret("text_final", "h4_policy"),
            "text_final_h4_one_nat_fixed": admission_regret("text_final", "h4_one_nat_fixed"),
            "text_final_categorical_policy": admission_regret("text_final", "categorical_policy"),
        },
    });
    mark("ring diagnostics", &mut marks);

    // ---- paired construction comparisons on the same full stream ----------------
    let mut paired: Vec<serde_json::Value> = Vec::new();
    for (label, obs, seqs, a, b, stratum, seed) in [
        (
            "final: h4_policy minus relational_ctx_parent (all positions)",
            &final_obs,
            &final_pop,
            &h4_load,
            &relational_ctx_base,
            None,
            0xA001u64,
        ),
        (
            "final: h4_policy minus relational_ctx_parent (final present)",
            &final_obs,
            &final_pop,
            &h4_load,
            &relational_ctx_base,
            Some("final_present"),
            0xA002,
        ),
        (
            "final: h4_policy minus relational_ctx_parent (final absent)",
            &final_obs,
            &final_pop,
            &h4_load,
            &relational_ctx_base,
            Some("final_absent"),
            0xA003,
        ),
        (
            "regression: h4_policy minus relational_ctx_parent (final present)",
            &regression_obs,
            &regression,
            &h4_load,
            &relational_ctx_base,
            Some("final_present"),
            0xA004,
        ),
        (
            "final: h4_policy minus relational_parent (all positions)",
            &final_obs,
            &final_pop,
            &h4_load,
            &relational_base,
            None,
            0xA005,
        ),
        (
            "final: categorical_policy minus h4_policy (all positions)",
            &final_obs,
            &final_pop,
            &cat_load,
            &h4_load,
            None,
            0xA006,
        ),
    ] {
        let mut d = paired_fresh_diff(&parent, &local, &u, &table, obs, seqs, a, b, stratum, seed);
        d["comparison"] = json!(label);
        d["unit"] = json!("bits_per_position");
        d["resampling_unit"] = json!("sequence");
        paired.push(d);
    }

    // ---- document-cluster intervals from serialized per-document sums -----------
    let doc_pairs = |panel: &str, arm: &str| -> Vec<(f64, f64, usize)> {
        agg_store
            .get(&(panel.to_string(), arm.to_string()))
            .map(|a| {
                a.per_doc
                    .values()
                    .map(|(l, r, n)| (*r, *l, *n))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    let text_intervals = json!({
        "text_final": {
            "h4_policy_vs_local": doc_cluster(&doc_pairs("text_final", "h4_policy"), 0xB001),
            "categorical_policy_vs_local": doc_cluster(&doc_pairs("text_final", "categorical_policy"), 0xB002),
            "relational_ctx_parent_vs_local": doc_cluster(&doc_pairs("text_final", "relational_ctx_parent"), 0xB003),
            "h4_policy_vs_relational_ctx_parent": doc_cluster(
                &{
                    let a = doc_pairs("text_final", "h4_policy");
                    let b = doc_pairs("text_final", "relational_ctx_parent");
                    a.iter()
                        .zip(b.iter())
                        .map(|(x, y)| (x.0, y.0, x.2))
                        .collect::<Vec<_>>()
                },
                0xB004,
            ),
        },
        "text_fit": {
            "h4_policy_vs_local": doc_cluster(&doc_pairs("text_fit", "h4_policy"), 0xB005),
        },
    });

    // ---- four-condition selected-source intervention ---------------------------
    let mut iv_attempted = 0usize;
    let mut iv_invariant = 0usize;
    let mut iv_support_lost = 0usize;
    let mut iv_no_read_after = 0usize;
    let mut iv_payload_follows = 0usize;
    let mut iv_emitted_follows = 0usize;
    let mut iv_ref_identical = 0usize;
    let mut iv_enabled_changes_emission = 0usize;
    let mut iv_original_still_admitted = 0usize;
    let mut iv_pool_size_changed = 0usize;
    let mut iv_neighbor_changed = 0usize;
    let mut iv_neighbors_added = 0usize;
    let mut iv_neighbors_removed = 0usize;
    let mut iv_lost_admission = 0usize;
    for (gi, seq) in final_pop.iter().enumerate() {
        let os = &final_obs[gi];
        let mut chosen: Option<(usize, usize)> = None;
        for (j, o) in os.iter().enumerate() {
            if o.cands.is_empty() {
                continue;
            }
            let z = local_logits(
                &parent,
                &local,
                &u,
                o.cur,
                o.prev as usize,
                o.prev2,
                o.ring.written(),
            );
            let rel = rels_for(&h4_load, o, &table, false);
            if let Some((k, _)) = h4_load.choose_with(&o.cands, &rel, Some(&z)) {
                // Outside the fixed recent query context.
                if o.cands[k].abs + 4 < o.ring.written() {
                    chosen = Some((j, k));
                }
            }
        }
        let Some((j, k)) = chosen else { continue };
        let o = &os[j];
        let payload = o.cands[k].payload;
        let Some(alt) = banks
            .values_held
            .iter()
            .find(|v| **v != payload && !seq.tokens.contains(v))
        else {
            continue;
        };
        let idx = (o.cands[k].abs + 1) as usize;
        if idx >= seq.tokens.len() {
            continue;
        }
        iv_attempted += 1;
        let (z_eo, _r_eo) = predict_next(
            &o.ring,
            o.cur,
            o.prev as usize,
            o.prev2,
            &h4_load,
            &table,
            &parent,
            &local,
            &u,
            true,
        );
        let (z_do, _) = predict_next(
            &o.ring,
            o.cur,
            o.prev as usize,
            o.prev2,
            &h4_load,
            &table,
            &parent,
            &local,
            &u,
            false,
        );
        let mut t2 = seq.tokens.clone();
        t2[idx] = *alt;
        let mut changed = seq.clone();
        changed.tokens = t2;
        let alt_obs = observe_full(&changed);
        let Some(o2) = alt_obs
            .iter()
            .find(|x| x.ring.written() == o.ring.written())
        else {
            iv_support_lost += 1;
            continue;
        };
        let (z_ec, r_ec) = predict_next(
            &o2.ring,
            o2.cur,
            o2.prev as usize,
            o2.prev2,
            &h4_load,
            &table,
            &parent,
            &local,
            &u,
            true,
        );
        let (z_dc, _) = predict_next(
            &o2.ring,
            o2.cur,
            o2.prev as usize,
            o2.prev2,
            &h4_load,
            &table,
            &parent,
            &local,
            &u,
            false,
        );
        // Disabled-before and disabled-after must be invariant: only an older payload changed.
        if z_do == z_dc {
            iv_invariant += 1;
        }
        if r_ec.action.is_none() {
            iv_no_read_after += 1;
        }
        if r_ec.payload == Some(*alt) {
            iv_payload_follows += 1;
        }
        // The original occurrence is still the served source when it is still selected.
        if r_ec.source == Some(o.cands[k].slot_ref) {
            iv_ref_identical += 1;
        }
        // Support and neighborhood: an `observe_full` row does not prove the original occurrence is
        // still admitted, and admission/features of neighboring records can change.
        if o2.cands.iter().any(|c| c.slot_ref == o.cands[k].slot_ref) {
            iv_original_still_admitted += 1;
        }
        if o2.cands.len() != o.cands.len() {
            iv_pool_size_changed += 1;
        }
        // The deliberately edited occurrence is not a neighbour. Count changed, added and removed
        // neighbours separately; count lost admission separately from a missing row and from NoRead.
        let edited_abs = o.cands[k].abs;
        let neighbor_changed = o.cands.iter().filter(|c| c.abs != edited_abs).any(|c| {
            match o2.cands.iter().find(|d| d.abs == c.abs) {
                Some(d) => d.feats != c.feats || d.payload != c.payload,
                None => true,
            }
        });
        if neighbor_changed {
            iv_neighbor_changed += 1;
        }
        iv_neighbors_added += o2
            .cands
            .iter()
            .filter(|d| !o.cands.iter().any(|c| c.abs == d.abs))
            .count();
        iv_neighbors_removed += o
            .cands
            .iter()
            .filter(|c| c.abs != edited_abs && !o2.cands.iter().any(|d| d.abs == c.abs))
            .count();
        if !o2.cands.iter().any(|c| c.slot_ref == o.cands[k].slot_ref) {
            iv_lost_admission += 1;
        }
        if argmax_low(&z_ec) as u32 == *alt {
            iv_emitted_follows += 1;
        }
        if argmax_low(&z_eo) != argmax_low(&z_ec) {
            iv_enabled_changes_emission += 1;
        }
    }
    controls.push(json!({
        "control": "selected_source_intervention_four_conditions",
        "arm": "h4_policy (reloaded)",
        "attempted": iv_attempted,
        "invariant_disabled_before_equals_disabled_after": iv_invariant,
        "lost_support_after_intervention": iv_support_lost,
        "no_read_after_intervention": iv_no_read_after,
        "enabled_selected_the_new_payload": iv_payload_follows,
        "enabled_emitted_the_new_payload": iv_emitted_follows,
        "original_source_reference_still_served": iv_ref_identical,
        "original_occurrence_still_admitted": iv_original_still_admitted,
        "candidate_pool_size_changed": iv_pool_size_changed,
        "neighbor_payload_or_feature_changed": iv_neighbor_changed,
        "neighbors_added": iv_neighbors_added,
        "neighbors_removed": iv_neighbors_removed,
        "original_occurrence_lost_admission": iv_lost_admission,
        "enabled_emission_changed_by_the_intervention": iv_enabled_changes_emission,
        "note": "four matched conditions on the same prefix: enabled-original, enabled-changed, disabled-original, disabled-changed; the replacement payload is absent from the whole sequence and the query is fixed; the deliberately edited occurrence is excluded from neighbour-change counts; lost admission, missing rows and NoRead are counted separately",
    }));

    // ---- frozen-confidence rollout: complete responses through the shared path -------------
    let mut rollout_rows: Vec<serde_json::Value> = Vec::new();
    {
        let mut prompts: Vec<(&'static str, Vec<u32>)> = Vec::new();
        for seq in conf_fresh_pop.iter().filter(|q| !q.absent).take(3) {
            prompts.push((
                "present_construction",
                seq.tokens[..seq.tokens.len() - 1].to_vec(),
            ));
        }
        for seq in conf_fresh_pop.iter().filter(|q| q.absent).take(2) {
            prompts.push((
                "absent_construction",
                seq.tokens[..seq.tokens.len() - 1].to_vec(),
            ));
        }
        for seq in conf_fresh_seqs.iter().take(2) {
            if seq.tokens.len() > 8 {
                prompts.push(("ordinary_text", seq.tokens[..seq.tokens.len() - 1].to_vec()));
            }
        }
        let arms: Vec<(&'static str, &RelationalSelector, bool)> = vec![
            ("local", &h4_load, false),
            ("relational_ctx_parent", &relational_ctx_base, true),
            ("h4_confidence", h4_confidence_load, true),
            ("categorical_confidence", categorical_confidence_load, true),
            ("h4_confidence_read_disabled", h4_confidence_load, false),
        ];
        for (pi, (kind, prefix)) in prompts.iter().enumerate() {
            let mut rows = Vec::new();
            for (name, sel, use_reader) in arms.iter() {
                let (toks, steps) =
                    rollout_prefix(prefix, sel, &table, &parent, &local, &u, *use_reader, 6)?;
                rows.push(json!({
                    "arm": name,
                    "tokens": toks,
                    "decoded": tokenizer.decode(&toks),
                    "reads": steps.iter().filter(|x| x["read"] == json!(true)).count(),
                    "steps": steps,
                }));
            }
            rollout_rows.push(json!({
                "prompt": kind, "index": pi, "prefix_len": prefix.len(), "arms": rows,
            }));
        }
    }
    mark("frozen-confidence rollout", &mut marks);

    // ---- read-conditioned geometric emission --------------------------------------------------
    // Historical suffix-solvable diagnostic. Corrected full-prefix/first-token semantics differ
    // from read-conditioned-1; rerunning this block does not validate context-required attention.
    let rc_t0 = Instant::now();
    let rc_dev_seqs = make_rc_pop(&banks, N_RC_DEV, SEED_RC_DEV);
    let rc_tune_seqs = make_rc_pop(&banks, N_RC_TUNE, SEED_RC_TUNE);
    let rc_fresh_seqs = make_rc_pop(&banks, N_RC_FRESH, SEED_RC_FRESH);
    let (rc_dev, rc_dev_cov, rc_dev_decisive, rc_dev_noread) = rc_positions(
        &parent,
        &local,
        &u,
        &table,
        &relational_ctx_base,
        &rc_dev_seqs,
    )?;
    let (rc_tune, _, _, _) = rc_positions(
        &parent,
        &local,
        &u,
        &table,
        &relational_ctx_base,
        &rc_tune_seqs,
    )?;
    let (rc_fresh, _, _, _) = rc_positions(
        &parent,
        &local,
        &u,
        &table,
        &relational_ctx_base,
        &rc_fresh_seqs,
    )?;
    let (rc_oracle_hit, rc_oracle_total) = rc_oracle(&rc_dev, &u);
    let rc_identity = ReadConditionedParams::identity();
    let rc_h4 = rc_learn(&rc_dev, &u, false);
    let rc_alt_h4 = rc_learn(&rc_dev, &u, true);
    // Parameter serialization only: write, hash, independently reload and compare. RLRC v1 does
    // not bind the parent/reader/tokenizer/fit identities; the successor must supply that contract.
    let rc_h4_bytes = rc_h4.to_bytes();
    write_checked(&root, "artifacts/h4_read_conditioned.rlrc", &rc_h4_bytes)?;
    let rc_h4_hash = artifact_sha256(&rc_h4_bytes);
    let rc_h4 = reload_read_conditioned(&rc_h4_bytes, &rc_h4)?;
    let rc_alt_h4_bytes = rc_alt_h4.to_bytes();
    write_checked(
        &root,
        "artifacts/h4_alternate_init_read_conditioned.rlrc",
        &rc_alt_h4_bytes,
    )?;
    let rc_alt_h4 = reload_read_conditioned(&rc_alt_h4_bytes, &rc_alt_h4)?;
    let acc = |p: &ReadConditionedParams, pos: &[RcPos]| rc_accuracy(p, pos, &u);
    let nll = |p: &ReadConditionedParams, pos: &[RcPos]| -> f64 {
        if pos.is_empty() {
            return f64::NAN;
        }
        pos.iter()
            .map(|q| rc_nll_bits(q, p, &u, parent.cfg.f_bits))
            .sum::<f64>()
            / pos.len() as f64
    };
    // Causal dependence of the emitted token on the update inputs.
    let (mut upd_positions, mut v_causal, mut rel_causal) = (0usize, 0usize, 0usize);
    for pos in rc_fresh.iter().filter(|q| q.read) {
        let q1 = rc_h4.update(pos.q0, pos.rel, pos.payload);
        if q1 == pos.q0 {
            continue;
        }
        upd_positions += 1;
        let base = rc_argmax_row(&pos.z_base, &u[q1]);
        let alt_v = rc_h4
            .value_domain
            .iter()
            .find(|t| **t != pos.payload)
            .copied()
            .unwrap_or(pos.payload);
        let q_alt_v = rc_h4.update(pos.q0, pos.rel, alt_v);
        if rc_argmax_row(&pos.z_base, &u[q_alt_v]) != base {
            v_causal += 1;
        }
        let q_alt_r = rc_h4.update(pos.q0, (pos.rel + 1) % GROUP_ORDER, pos.payload);
        if rc_argmax_row(&pos.z_base, &u[q_alt_r]) != base {
            rel_causal += 1;
        }
    }
    // Actual generated behavior on the instrument: does the first emitted token equal the answer?
    let mut rc_gen_hit = 0usize;
    let mut rc_gen_local_hit = 0usize;
    let mut rc_gen_total = 0usize;
    let mut rc_gen_samples: Vec<serde_json::Value> = Vec::new();
    for seq in rc_fresh_seqs.iter().take(30) {
        let prefix = &seq.tokens[..];
        if prefix.len() < 3 {
            continue;
        }
        rc_gen_total += 1;
        let (toks, steps) = rc_rollout(
            prefix,
            &relational_ctx_base,
            &table,
            &parent,
            &local,
            &u,
            &rc_h4,
            true,
            true,
            4,
        )?;
        let emitted = first_continuation_token(&toks, prefix.len())?;
        if emitted == seq.answer {
            rc_gen_hit += 1;
        }
        let (local_toks, _) = rc_rollout(
            prefix,
            &relational_ctx_base,
            &table,
            &parent,
            &local,
            &u,
            &rc_identity,
            false,
            true,
            4,
        )?;
        if first_continuation_token(&local_toks, prefix.len())? == seq.answer {
            rc_gen_local_hit += 1;
        }
        if rc_gen_samples.len() < 4 {
            rc_gen_samples.push(json!({
                "answer": seq.answer,
                "prefix_len": prefix.len(),
                "first_emitted": emitted,
                "tokens": toks,
                "decoded": tokenizer.decode(&toks),
                "steps": steps,
            }));
        }
    }
    let rc_report = json!({
        "instrument": "historical derived role-partner: answer is determined by the recent query role alone; not a context-required attention test",
        "semantics": "full observed prefix retained; actual first continuation token; no last-nonempty fallback; principal-corrected diagnostic differs from read-conditioned-1",
        "categorical_control": "NOT_RUN: the alternate arm changes initialization of the same H4 operator",
        "populations": {"dev": rc_dev_seqs.len(), "tune": rc_tune_seqs.len(), "fresh": rc_fresh_seqs.len()},
        "validity": {
            "dev_positions": rc_dev.len(),
            "dev_answer_covered_by_a_payload": rc_dev_cov,
            "dev_local_argmax_wrong": rc_dev_decisive,
            "dev_no_read": rc_dev_noread,
        },
        "oracle_ceiling": {"positions": rc_oracle_total, "some_row_emits_the_answer": rc_oracle_hit},
        "accuracy": {
            "local_no_update": {"dev": acc(&rc_identity, &rc_dev), "tune": acc(&rc_identity, &rc_tune), "fresh": acc(&rc_identity, &rc_fresh)},
            "h4_read_conditioned": {"dev": acc(&rc_h4, &rc_dev), "tune": acc(&rc_h4, &rc_tune), "fresh": acc(&rc_h4, &rc_fresh)},
            "h4_alternate_initialization": {"dev": acc(&rc_alt_h4, &rc_dev), "tune": acc(&rc_alt_h4, &rc_tune), "fresh": acc(&rc_alt_h4, &rc_fresh)},
        },
        "answer_nll_bits": {
            "local_no_update": nll(&rc_identity, &rc_fresh),
            "h4_read_conditioned": nll(&rc_h4, &rc_fresh),
            "h4_alternate_initialization": nll(&rc_alt_h4, &rc_fresh),
        },
        "causal": {
            "fresh_read_positions_updated": upd_positions,
            "emitted_changes_when_value_code_input_changes": v_causal,
            "emitted_changes_when_relation_changes": rel_causal,
        },
        "generated_first_token": {
            "positions": rc_gen_total,
            "h4_read_conditioned_hits": rc_gen_hit,
            "local_no_update_hits": rc_gen_local_hit,
            "samples": rc_gen_samples,
        },
        "artifact": {
            "h4_sha256": rc_h4_hash, "bytes": rc_h4_bytes.len(),
            "alternate_h4_sha256": artifact_sha256(&rc_alt_h4_bytes),
            "binding_scope": "RLRC v1 parameters only; no parent/reader/tokenizer/fit identity validation",
        },
        "seconds": rc_t0.elapsed().as_secs_f64(),
        "scope": "suffix-solvable diagnostic only; fixed-readout reachability at these prefixes cannot isolate admission, selection or geometric advantage; no context-required or broad language claim",
    });
    controls.push(json!({
        "control": "read_conditioned_update",
        "instrument": "derived_role_partner",
        "update_disabled_residual_exact_zero": ReadConditionedParams::identity().update(7, 3, 100) == 7,
        "h4_artifact_sha256": rc_h4_hash,
        "note": "The reported boolean checks an identity parameter map only; actual NoRead/UpdateDisabled predictor parity requires its own executed control",
    }));
    mark("read-conditioned emission", &mut marks);

    // ---- generation from the reloaded artifacts --------------------------------
    let gen_arms: [(&str, &RelationalSelector); 4] = [
        ("relational_ctx_parent", &relational_ctx_base),
        ("h4_policy", &h4_load),
        ("categorical_policy", &cat_load),
        ("categorical_parent", &categorical_base),
    ];
    let mut generation: Vec<serde_json::Value> = Vec::new();
    for (gi, seq) in final_pop.iter().enumerate().take(6) {
        let prefix = &seq.tokens[..seq.tokens.len() - 1];
        let mut rows = Vec::new();
        rows.push(json!({
            "arm": "local",
            "tokens": greedy_continue(prefix, &h4_load, &table, &parent, &local, &u, false, 8),
        }));
        for (name, sel) in gen_arms.iter() {
            rows.push(json!({
                "arm": name,
                "tokens": greedy_continue(prefix, sel, &table, &parent, &local, &u, true, 8),
            }));
        }
        let mut seen: Vec<Vec<u32>> = Vec::new();
        for r in rows.iter_mut() {
            let toks: Vec<u32> = r["tokens"]
                .as_array()
                .map(|v| {
                    v.iter()
                        .filter_map(|x| x.as_u64().map(|y| y as u32))
                        .collect()
                })
                .unwrap_or_default();
            let repeats = toks.windows(2).filter(|w| w[0] == w[1]).count();
            r["decoded"] = json!(tokenizer.decode(&toks));
            r["adjacent_repeats"] = json!(repeats);
            r["loop_period_1"] = json!(toks.len() >= 2 && toks.iter().all(|t| *t == toks[0]));
            seen.push(toks);
        }
        generation.push(json!({
            "prompt": "construction_final",
            "index": gi,
            "answer": seq.answer,
            "absent": seq.absent,
            "arms": rows,
        }));
    }
    for (ti, seq) in final_text_seqs.iter().enumerate().take(2) {
        let prefix = &seq.tokens[..seq.tokens.len().min(24)];
        let mut rows = Vec::new();
        rows.push(json!({
            "arm": "local",
            "tokens": greedy_continue(prefix, &h4_load, &table, &parent, &local, &u, false, 16),
        }));
        for (name, sel) in gen_arms.iter() {
            rows.push(json!({
                "arm": name,
                "tokens": greedy_continue(prefix, sel, &table, &parent, &local, &u, true, 16),
            }));
        }
        for r in rows.iter_mut() {
            let toks: Vec<u32> = r["tokens"]
                .as_array()
                .map(|v| {
                    v.iter()
                        .filter_map(|x| x.as_u64().map(|y| y as u32))
                        .collect()
                })
                .unwrap_or_default();
            r["decoded"] = json!(tokenizer.decode(&toks));
            r["adjacent_repeats"] = json!(toks.windows(2).filter(|w| w[0] == w[1]).count());
            r["loop_period_1"] = json!(toks.len() >= 2 && toks.iter().all(|t| *t == toks[0]));
        }
        generation.push(json!({
            "prompt": "natural_text_final",
            "index": ti,
            "document": uniq[final_text_docs[ti]].path,
            "arms": rows,
        }));
    }
    mark("controls + generation", &mut marks);

    // ---- paired interleaved cost measurement -----------------------------------
    let active_stream: Vec<u32> = final_pop[0].tokens.clone();
    let text_stream: Vec<u32> =
        final_text_seqs[0].tokens[..24.min(final_text_seqs[0].tokens.len())].to_vec();
    let mut cost_rows: Vec<serde_json::Value> = Vec::new();
    for (label, stream) in [
        ("construction_stream", &active_stream),
        ("text_window", &text_stream),
    ] {
        let mut local_s = Vec::new();
        let mut reader_s = Vec::new();
        let mut paired_d = Vec::new();
        let _ = active_probe(&parent, &local, &u, &h4_load, &table, stream, true);
        for _ in 0..7 {
            let t0 = Instant::now();
            let lo = active_probe(&parent, &local, &u, &h4_load, &table, stream, false);
            black_box(&lo);
            let t1 = t0.elapsed().as_secs_f64();
            let rd = active_probe(&parent, &local, &u, &h4_load, &table, stream, true);
            black_box(&rd);
            let t2 = t0.elapsed().as_secs_f64();
            local_s.push(t1);
            reader_s.push(t2 - t1);
            paired_d.push((t2 - t1) - t1);
        }
        local_s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        reader_s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        paired_d.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (_, preds, admitted, reads) =
            active_probe(&parent, &local, &u, &h4_load, &table, stream, true);
        cost_rows.push(json!({
            "stream": label,
            "tokens": stream.len(),
            "predictions": preds,
            "candidates_admitted_total": admitted,
            "candidates_admitted_per_prediction": admitted as f64 / preds.max(1) as f64,
            "read_actions": reads,
            "median_local_s": local_s[local_s.len() / 2],
            "median_reader_s": reader_s[reader_s.len() / 2],
            "median_paired_reader_minus_local_s": paired_d[paired_d.len() / 2],
            "median_local_us_per_prediction": local_s[local_s.len() / 2] / preds.max(1) as f64 * 1e6,
            "median_reader_us_per_prediction": reader_s[reader_s.len() / 2] / preds.max(1) as f64 * 1e6,
            "protocol": "paired interleaved repeats on the same stream, alternating no-reader-work local and full reader passes; medians reported; no token-pair cache",
        }));
    }
    let mut serialized = serde_json::Map::new();
    serialized.insert("parent_E".into(), json!(pb.len()));
    serialized.insert("local_query_artifact".into(), json!(sb.len()));
    for (k, v) in export_bytes.iter() {
        serialized.insert((*k).to_string(), json!(v.len()));
    }
    let cost = json!({
        "measurements": cost_rows,
        "measured_reader_arm": "h4_policy (32-address coarse policy)",
        "confidence_arm_timing": "NOT_RUN",
        "serialized_bytes": serde_json::Value::Object(serialized),
        "resident_bytes": {
            "local_row_table": 120 * parent.cfg.vocab * 4,
            "parent_scratch": parent.cfg.vocab * 4,
            "policy_opcodes_per_arm": {"coarse": UTIL_BUCKETS, "confidence": CONF_ADDRESSES},
            "gap_thresholds_per_arm": 4 * (UTIL_GAP_BINS - 1),
        },
        "energy": "UNAVAILABLE",
    });
    mark("cost", &mut marks);

    // ---- manifest with a digest the loader verifies -----------------------------
    let full_binding = binding_digest(&fit_obs, &fit_text_obs, &tune_obs, &final_obs);
    let mut manifest = json!({
        "schema": "uor-r4.reader-utility-binding/2",
        "base_revision": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
        "parent_root": parent_root.display().to_string(),
        "parents": parents_meta,
        "policy_config": {
            "config_digest": hex_of(&policy_cfg_digest),
            "digest_scope": "coarse 32-address observation/action contract and shared gap thresholds; does not identify the added confidence-bit semantics",
            "gap_thresholds": gap_thresholds,
            "bucket_formula": POLICY_BUCKET_FORMULA,
            "gap_units": POLICY_GAP_UNITS,
            "comparison": POLICY_COMPARISON,
            "context_bits": POLICY_CONTEXT_BITS,
            "margin_rule": POLICY_MARGIN_RULE,
            "action_encoding": POLICY_ACTION_ENCODING,
            "configured_before_event_extraction": true,
        },
        "fit_input_digest": hex_of(&fit_input_digest),
        "fit_input_digest_scope": "coarse policy fit events; not the confidence artifacts' data_digest",
        "confidence_interface": {
            "addresses": CONF_ADDRESSES,
            "dispatch": "serialized policy length 64 selects confidence semantics; length 32 selects the unchanged coarse policy",
            "address_formula": "2 * coarse_utility_bucket + 1[D > 0]",
            "D": "max_strength strength_score(ungated top source, relation, strength, parent bucket) minus noread_score(parent bucket), in widened i64",
            "parent_bucket": "selector bucket_of including newest bit and its own single-candidate margin rule; not the coarse utility bucket",
            "action_encoding": POLICY_ACTION_ENCODING,
            "partial_construction_digest": hex_of(&conf_fit_digest),
            "binding_limit": "this digest omits text fit inputs, sequence/document boundaries, selected occurrence/payload references, confidence feature semantics and full optimizer configuration; complete fit dependency closure is not established",
        },
        "artifact_bytes_sha256": export_bytes
            .iter()
            .map(|(k, v)| ((*k).to_string(), json!(sha256_hex(v))))
            .collect::<serde_json::Map<_, _>>(),
        "inputs": {
            "E_sha256": E_SHA, "local_query_artifact_sha256": S_SHA,
            "tokenizer_source_sha256": TOKENIZER_SHA, "tokenizer_derived_sha256": DERIVED_SHA,
            "parent_digest": hex_of(&parent_digest),
        },
        "group_digest": group_digest,
        "observations": {
            "full_binding_digest": hex_of(&full_binding),
            "construction_fit_positions": fit_n, "construction_fit_candidate_positions": fit_cand,
            "construction_tune_positions": tune_n, "construction_regression_positions": reg_n,
            "construction_final_positions": fin_n, "construction_final_candidate_positions": fin_cand,
            "construction_previous_seed_positions": prev_n, "construction_previous_seed_candidate_positions": prev_cand,
            "text_fit_positions": fit_text_n, "text_fit_candidate_positions": fit_text_cand,
            "text_final_positions": final_text_n, "text_final_candidate_positions": final_text_cand,
        },
        "text_split": text_split.clone(),
        "features": {
            "scope": "coarse policy; see confidence_interface for the 64-address extension",
            "buckets": UTIL_BUCKETS,
            "bucket_formula": "gap_bin*8 + ctx_class*2 + margin_bit",
            "gap_bin": "count of declared integer thresholds below the selected payload's local logit gap",
            "ctx_class": "bit0 = payload equals the current input token, bit1 = ordered two-neighbour agreement",
            "margin_bit": "1 iff the ungated top source score strictly exceeds the runner-up's",
            "gap_thresholds": gap_thresholds,
            "actions": {"0": "NoRead", "1": "0.0625 nats", "2": "1 nat", "3": "8 nats"},
            "bucket_occupancy": contract_buckets,
        },
        "configuration": {
            "ring_cap": RING_CAP, "max_candidates": MAX_CAND,
            "seeds": {"fit": SEED_FIT, "tune": SEED_TUNE, "regression": SEED_FRESH, "previous_final": SEED_FINAL, "final": SEED_FINAL2, "confidence_fresh": SEED_CONF_FRESH},
            "min_support": UTIL_MIN_SUPPORT,
            "coarse_policy_fit": "equal construction/text total weight; a read is chosen only if its mean cost is strictly below NoRead's",
            "confidence_policy_fit": "minimize full-stream fit text delta subject to present emitted-count/loss and absent read-count/loss constraints; unsupported addresses keep the witnessed parent rule",
        },
        "export": {"format": "RLR2 v4", "artifacts": export_bytes.keys().collect::<Vec<_>>()},
        "source_files": sources.clone(),
        "manifest_digest": "",
        "limitations": [
            "The frozen local prior was trained on this pinned corpus; reader-held-out is not externally unseen.",
            "Offline probabilities are used only for fitting; serving uses integer comparisons and table lookups.",
            "Physical energy is UNAVAILABLE without measurement.",
        ],
    });
    let canonical = serde_json::to_string(&manifest).map_err(|e| e.to_string())?;
    let manifest_digest = sha256_hex(canonical.as_bytes());
    manifest["manifest_digest"] = json!(manifest_digest.clone());
    write_json(&root, "binding.json", &manifest)?;
    let verified_digest = verify_manifest(&root)?;
    controls.push(json!({
        "control": "artifact_reload_parity",
        "arms": ["h4_policy", "categorical_policy"],
        "reload_failures": reload_failures,
        "confidence_arms": ["h4_confidence", "categorical_confidence"],
        "note": "confidence artifacts must load and equal the selected selectors before their teacher-forced panels run; generation, interventions and timing still use coarse policies and do not validate confidence rollout",
    }));

    // ---- decision --------------------------------------------------------------
    let agg = |panel: &str, arm: &str| -> serde_json::Value {
        agg_store
            .get(&(panel.to_string(), arm.to_string()))
            .map(agg_json)
            .unwrap_or(json!(null))
    };
    let corpus = |v: &serde_json::Value, f: &str| -> f64 { v[f].as_f64().unwrap_or(f64::NAN) };
    let txt_final_h4 = agg("text_final", "h4_policy");
    let txt_final_cat = agg("text_final", "categorical_policy");
    let txt_final_parent = agg("text_final", "relational_ctx_parent");
    let fin_h4 = agg("construction_final", "h4_policy");
    let fin_parent = agg("construction_final", "relational_ctx_parent");
    let reg_h4 = agg("construction_regression", "h4_policy");
    let reg_parent = agg("construction_regression", "relational_ctx_parent");
    let stratum_of_arm = |v: &serde_json::Value, label: &str| -> serde_json::Value {
        v["strata"]
            .as_array()
            .and_then(|a| a.iter().find(|s| s["stratum"] == label))
            .cloned()
            .unwrap_or(json!(null))
    };
    let present_h4 = stratum_of_arm(&fin_h4, "final_present");
    let present_parent = stratum_of_arm(&fin_parent, "final_present");
    let absent_h4 = stratum_of_arm(&fin_h4, "final_absent");
    let absent_parent = stratum_of_arm(&fin_parent, "final_absent");
    let d_text_h4 = corpus(&txt_final_h4, "delta_bits_per_position");
    let d_text_cat = corpus(&txt_final_cat, "delta_bits_per_position");
    let d_text_parent = corpus(&txt_final_parent, "delta_bits_per_position");
    let h4_text_interval = text_intervals["text_final"]["h4_policy_vs_local"].clone();
    let harm_containment = d_text_h4 <= MARGIN_TEXT_BITS;
    let useful_transfer =
        d_text_h4 < 0.0 && h4_text_interval["hi"].as_f64().unwrap_or(f64::NAN) < 0.0;
    // Preservation uses final-present **emitted correctness** and loss, as declared.
    let present_loss_ok = corpus(&present_h4, "hard_bits_per_position")
        <= corpus(&present_parent, "hard_bits_per_position") + MARGIN_PRESENT_BITS;
    let present_emitted_ok = present_h4["emitted_correct"].as_u64().unwrap_or(0) + 2
        >= present_parent["emitted_correct"].as_u64().unwrap_or(0);
    let present_ok = present_loss_ok && present_emitted_ok;
    // Absence: actual read counts AND loss, reported separately. Unknown absent answers are not
    // expected to be guessed, so payload correctness is not part of this predicate.
    let absent_reads_h4 = absent_h4["reads"].as_u64().unwrap_or(0);
    let absent_reads_parent = absent_parent["reads"].as_u64().unwrap_or(0);
    let absent_reads_available = !absent_parent["reads"].is_null();
    let absent_loss_h4 = corpus(&absent_h4, "delta_bits_per_position");
    let absent_loss_parent = corpus(&absent_parent, "delta_bits_per_position");
    let absent_loss_ok = absent_loss_h4 <= absent_loss_parent;
    let absent_reads_ok = absent_reads_available && absent_reads_h4 <= absent_reads_parent;
    let absent_ok = absent_loss_ok && absent_reads_ok;
    // A generic one-nat boost over the same ungated source: does the fitted table add value?
    let fin_one_nat = agg("construction_final", "h4_one_nat_fixed");
    let txt_one_nat = agg("text_final", "h4_one_nat_fixed");
    let fitted_beats_one_nat_construction =
        corpus(&fin_h4, "hard_bits_per_position") < corpus(&fin_one_nat, "hard_bits_per_position");
    let fitted_beats_one_nat_text = corpus(&txt_final_h4, "delta_bits_per_position")
        < corpus(&txt_one_nat, "delta_bits_per_position");
    let read_rate_h4 =
        corpus(&fin_h4, "reads") / corpus(&fin_h4, "candidate_bearing_positions").max(1.0);
    let all_noread_collapse =
        corpus(&fin_h4, "reads") / corpus(&fin_h4, "positions").max(1.0) <= 0.02;

    write_json(
        &root,
        "result.json",
        &json!({
            "schema": "uor-r4.reader-utility/1",
            "base_revision": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
            "running_source": {
                "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
                "git_dirty": std::env::var("UOR_GIT_DIRTY").unwrap_or_else(|_| "unknown".into()),
                "executable_sha256": executable_sha256,
                "source_file_sha256": sources,
            },
            "inputs": {"E": E_SHA, "local_query_artifact": S_SHA, "tokenizer": {"source": TOKENIZER_SHA, "derived": derived}},
            "parents": parents_meta,
            "group_digest": group_digest,
            "text_split": text_split,
            "preparation_probe": preparation_probe,
            "fit": fit_report,
            "feasibility": feasibility_report,
            "read_confidence": conf_report,
            "confidence_rollout": rollout_rows,
            "read_conditioned": rc_report,
            "panels": panels,
            "paired": paired,
            "text_intervals": text_intervals,
            "ring_diagnostics": ring_diagnostics,
            "controls": controls,
            "generation": generation,
            "cost": cost,
            "manifest_digest": verified_digest,
            "reload_failures": reload_failures,
            "decision": {
                "unit": "bits_per_position on the full fixed stream; text in bits_per_token",
                "endpoints": {
                    "text_final_h4_policy": txt_final_h4,
                    "text_final_categorical_policy": txt_final_cat,
                    "text_final_relational_ctx_parent": txt_final_parent,
                    "construction_final_h4_policy": fin_h4,
                    "construction_final_relational_ctx_parent": fin_parent,
                    "construction_regression_h4_policy": reg_h4,
                    "construction_regression_relational_ctx_parent": reg_parent,
                    "final_present_h4_policy": present_h4,
                    "final_present_relational_ctx_parent": present_parent,
                    "final_absent_h4_policy": absent_h4,
                    "final_absent_relational_ctx_parent": absent_parent,
                },
                "text_delta_bits_per_token": {
                    "h4_policy": d_text_h4,
                    "categorical_policy": d_text_cat,
                    "relational_ctx_parent": d_text_parent,
                },
                "margins": {"text_bits_per_token": MARGIN_TEXT_BITS, "component_bits_per_position": MARGIN_PRESENT_BITS},
                "read_rate_among_candidate_bearing_final_construction_h4_policy": read_rate_h4,
                "present_query_detail": {
                    "loss_margin_ok": present_loss_ok,
                    "emitted_correctness_ok": present_emitted_ok,
                    "h4_policy_emitted_correct": present_h4["emitted_correct"].clone(),
                    "parent_emitted_correct": present_parent["emitted_correct"].clone(),
                    "h4_policy_reads": present_h4["reads"].clone(),
                    "parent_reads": present_parent["reads"].clone(),
                },
                "absence_detail": {
                    "loss_ok": absent_loss_ok,
                    "read_count_ok": absent_reads_ok,
                    "read_count_check_available": absent_reads_available,
                    "h4_policy_reads": absent_reads_h4,
                    "parent_reads": absent_reads_parent,
                    "h4_policy_loss_bits_per_absent_query": absent_loss_h4,
                    "parent_loss_bits_per_absent_query": absent_loss_parent,
                    "note": "payload correctness is deliberately NOT part of this predicate: an unknowable absent answer is not expected to be guessed",
                },
                "constant_comparator": {
                    "arm": "h4_one_nat_fixed",
                    "fitted_beats_one_nat_on_construction": fitted_beats_one_nat_construction,
                    "fitted_beats_one_nat_on_text": fitted_beats_one_nat_text,
                    "one_nat_construction_hard_bits_per_position": corpus(&fin_one_nat, "hard_bits_per_position"),
                    "one_nat_text_delta_bits_per_token": corpus(&txt_one_nat, "delta_bits_per_position"),
                },
                "categories": {
                    "harm_containment": harm_containment,
                    "useful_transfer": useful_transfer,
                    "relational_behaviour_preserved": present_ok,
                    "absence_improved": absent_ok,
                    "all_noread_collapse": all_noread_collapse,
                    "matched_categorical_reported": true,
                },
                "historical": "PR #1323's narrow controller positive, PR #1325's retained_component_positive=false and unique_geometric_benefit=false all stand at their own scope and are not re-applied; PR #1328's useful_transfer=true is retained under its stated CE rule with the fit/serving indexing defect attached",
            },
            "phases": marks.iter().fold((0.0f64, Vec::new()), |(prev, mut out), (n, t)| {
                out.push(json!({"phase": n, "seconds": t - prev, "cumulative_s": t}));
                (*t, out)
            }).1,
            "elapsed_s": started.elapsed().as_secs_f64(),
        }),
    )?;
    // Preflight the **serialized** structure before the expensive public step ends: every declared
    // path must exist in the object actually written, not in a hand-authored field list.
    let written: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("result.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let declared_paths = [
        "schema",
        "parents",
        "group_digest",
        "text_split",
        "preparation_probe",
        "fit",
        "panels",
        "ring_diagnostics",
        "controls",
        "generation",
        "cost",
        "manifest_digest",
        "decision",
        "feasibility",
        "read_confidence",
        "confidence_rollout",
        "read_conditioned",
    ];
    let mut missing: Vec<&str> = Vec::new();
    for p in declared_paths {
        if written.get(p).is_none() {
            missing.push(p);
        }
    }
    for (panel, arm) in [
        ("construction_final", "h4_policy"),
        ("construction_final", "h4_one_nat_fixed"),
        ("text_final", "h4_policy"),
    ] {
        let found = written["panels"]
            .as_array()
            .map(|ps| {
                ps.iter().any(|p| {
                    p["panel"] == panel
                        && p["arms"]
                            .as_array()
                            .map(|a| a.iter().any(|x| x["arm"] == arm))
                            .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        if !found {
            missing.push("panel/arm");
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "serialized result is missing declared fields: {missing:?}"
        ));
    }
    write_json(
        &root,
        "preflight.json",
        &json!({
            "declared_paths_present": true,
            "declared_paths": declared_paths,
            "panels_checked": [["construction_final", "h4_policy"], ["construction_final", "h4_one_nat_fixed"], ["text_final", "h4_policy"]],
            "note": "validated against the object actually written, not against a second hand-authored list",
        }),
    )?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "utility-transfer: text_final h4 {:.4} cat {:.4} one_nat {:.4} parent {:.4} bits/token | present emitted h4 {} parent {} | absent reads h4 {} parent {} | rel_preserved {} absence_ok {} harm {} transfer {} | fitted>1nat c={} t={} | manifest {} | sealed {} unlisted | {:.1}s",
        d_text_h4, d_text_cat, corpus(&txt_one_nat, "delta_bits_per_position"), d_text_parent,
        present_h4["emitted_correct"].as_u64().unwrap_or(0),
        present_parent["emitted_correct"].as_u64().unwrap_or(0),
        absent_reads_h4, absent_reads_parent,
        present_ok, absent_ok, harm_containment, useful_transfer,
        fitted_beats_one_nat_construction, fitted_beats_one_nat_text,
        verified_digest, unlisted.len(), started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Contextual emission: a context-required Read -> Update -> Emit pass
// ---------------------------------------------------------------------------

const CE_N_DEV: usize = 90;
const CE_N_TUNE: usize = 60;
const CE_N_FRESH: usize = 60;
const CE_DISTRACTORS: usize = 3;
const CE_SEED_DEV: u64 = 0xC0F0_0011;
const CE_SEED_TUNE: u64 = 0xC0F0_0012;
/// Untouched final draw: seeds 0xC0F00001..03 are the exposed regression populations.
const CE_SEED_FINAL: u64 = 0xC0F0_0021;
const CE_MAX_W_ROWS: usize = 64;
const CE_EPOCHS: usize = 600;
const CE_LR: f64 = 1.0;

struct CeItem {
    seq: Seq,
    value: u32,
    out: u32,
    pair_id: usize,
    #[allow(dead_code)]
    member: usize,
    expected_payload_abs: u32,
}

struct CePairSpec {
    blocks: Vec<(u32, u32, u32)>,
    qrole: u32,
    key: u32,
    pos: usize,
    source_role: u32,
    value_idx: [usize; 2],
}

/// Identical structure for both members of a pair: same key, same distractor roles/values/order,
/// same relevant block position. Only the relevant block's value differs.
fn make_ce_specs(banks: &Banks, n_pairs: usize, seed: u64, values: &[u32]) -> Vec<CePairSpec> {
    let mut st = seed;
    let mut out = Vec::new();
    for _ in 0..n_pairs {
        let key = pick(&banks.keys, &mut st);
        let (fa, fb) = pick(&banks.pairs, &mut st);
        let (qrole, source_role) = if xorshift(&mut st) & 1 == 0 {
            (fa, fb)
        } else {
            (fb, fa)
        };
        let i0 = (xorshift(&mut st) as usize) % values.len();
        let mut i1 = (xorshift(&mut st) as usize) % values.len();
        if i1 == i0 {
            i1 = (i0 + 1) % values.len();
        }
        let pool: Vec<(u32, u32)> = banks
            .pairs
            .iter()
            .copied()
            .filter(|q| *q != (fa, fb) && *q != (fb, fa))
            .collect();
        let mut blocks: Vec<(u32, u32, u32)> = Vec::new();
        for _ in 0..CE_DISTRACTORS {
            let (a, b) = pick(&pool, &mut st);
            let role = if xorshift(&mut st) & 1 == 0 { a } else { b };
            let v = pick(values, &mut st);
            blocks.push((role, key, v));
        }
        // Keep the relevant block off the final position so both members share identical local input.
        let pos = (xorshift(&mut st) as usize) % blocks.len().max(1);
        out.push(CePairSpec {
            blocks,
            qrole,
            key,
            pos,
            source_role,
            value_idx: [i0, i1],
        });
    }
    out
}

fn ce_item_tokens(spec: &CePairSpec, value: u32, out: u32) -> Vec<u32> {
    let mut blocks = spec.blocks.clone();
    blocks.insert(
        spec.pos.min(blocks.len()),
        (spec.source_role, spec.key, value),
    );
    let mut tokens = Vec::new();
    for (r, k, v) in blocks.iter() {
        tokens.push(*r);
        tokens.push(*k);
        tokens.push(*v);
    }
    tokens.push(spec.qrole);
    tokens.push(spec.key);
    tokens.push(out);
    tokens
}

struct CePos {
    z_local: Vec<i32>,
    q0: usize,
    r: usize,
    payload: u32,
    read: bool,
    target: u32,
    decisive: bool,
    absent: bool,
    target_margin: i32,
    correct_source: bool,
    correct_value: bool,
}

#[allow(clippy::too_many_arguments)]
fn ce_extract(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    sel: &RelationalSelector,
    items: &[CeItem],
) -> Result<(Vec<CePos>, Vec<CeExample>, Vec<usize>), String> {
    let mut positions = Vec::new();
    let mut examples = Vec::new();
    let mut pair_ids = Vec::new();
    for item in items.iter() {
        if item.seq.tokens.len() < 4 {
            return Err("contextual item lacks a complete query".into());
        }
        let obs = observe_full(&item.seq);
        let want = item.seq.tokens.len() - 2;
        let o = obs
            .iter()
            .find(|o| o.ring.written() as usize == want)
            .ok_or("contextual decision point is missing")?;
        let z = local_logits(
            parent,
            local,
            u,
            o.cur,
            o.prev as usize,
            o.prev2,
            o.ring.written(),
        );
        let selected = read_step(&o.ring, o.cur, o.prev, o.prev2, sel, table, true, &z);
        let r = selected.rel;
        let payload = selected.payload.unwrap_or(0);
        let read = selected.action.is_some();
        let q0 = local.query_state(o.cur).min(u.len() - 1);
        let absent = !o.cands.iter().any(|c| c.payload == item.seq.answer);
        let t = (item.seq.answer as usize).min(z.len() - 1);
        let best_other = z
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != t)
            .map(|(_, v)| *v)
            .max()
            .unwrap_or(i32::MIN);
        positions.push(CePos {
            z_local: z.clone(),
            q0,
            r,
            payload,
            read,
            target: item.seq.answer,
            decisive: argmax_low(&z) as u32 != item.seq.answer,
            absent,
            target_margin: z[t] - best_other,
            correct_source: selected.payload_abs == Some(item.expected_payload_abs),
            correct_value: selected.payload == Some(item.value),
        });
        examples.push(CeExample {
            z_local: z,
            q0,
            r,
            payload,
            read,
            target: item.seq.answer,
        });
        pair_ids.push(item.pair_id);
    }
    Ok((positions, examples, pair_ids))
}

fn ce_logits(
    ex: &CeExample,
    params: &ReadConditionedParams,
    res: &EmissionResidual,
    cyclic: bool,
    rows: &[usize],
) -> Vec<i32> {
    let q1 = ex.q1(params, cyclic);
    let mut z = ex.z_local.clone();
    res.add_to_logits(ex.q0, q1, rows, &mut z);
    z
}

fn ce_hits(
    examples: &[CeExample],
    params: &ReadConditionedParams,
    res: &EmissionResidual,
    cyclic: bool,
    rows: &[usize],
) -> usize {
    examples
        .iter()
        .filter(|ex| argmax_low(&ce_logits(ex, params, res, cyclic, rows)) as u32 == ex.target)
        .count()
}

/// Incompatible-output collisions: for the same `(q0, relation)`, distinct payloads that must
/// produce different targets but share the same value state collapse the update. This is the
/// diagnosed information loss, and it is a property of the value map, not of readout width.
fn ce_collisions(examples: &[CeExample], params: &ReadConditionedParams) -> usize {
    let mut m: std::collections::BTreeMap<(usize, usize), Vec<(u32, u32)>> =
        std::collections::BTreeMap::new();
    for ex in examples.iter().filter(|e| e.read) {
        m.entry((ex.q0, ex.r % GROUP_ORDER))
            .or_default()
            .push((ex.payload, ex.target));
    }
    let mut c = 0usize;
    for v in m.values() {
        for i in 0..v.len() {
            for j in (i + 1)..v.len() {
                if v[i].0 != v[j].0
                    && v[i].1 != v[j].1
                    && params.value_state(v[i].0) == params.value_state(v[j].0)
                {
                    c += 1;
                }
            }
        }
    }
    c
}

/// Map search minimizes a lexicographic pair: first observed distinct-payload aliases, then the
/// exact calibrated served CE. Same selected-payload contradictory targets are reader/input
/// ambiguity, not a value-code alias. This is not universal injectivity or a full feature test.
fn ce_objective(
    examples: &[CeExample],
    params: &ReadConditionedParams,
    res: &EmissionResidual,
    cyclic: bool,
    rows: &[usize],
    f_bits: u32,
) -> (usize, f64) {
    (
        ce_collisions(examples, params),
        served_nll_bits(examples, params, res, cyclic, rows, f_bits),
    )
}

fn ce_objective_better(candidate: (usize, f64), incumbent: (usize, f64)) -> bool {
    candidate.1.is_finite()
        && (candidate.0 < incumbent.0
            || (candidate.0 == incumbent.0 && candidate.1 < incumbent.1 - 1e-12))
}

/// Discrete search over update maps, preserving observed distinctions before minimizing served CE.
fn ce_search_maps(
    examples: &[CeExample],
    params: &ReadConditionedParams,
    res: &EmissionResidual,
    cyclic: bool,
    rows: &[usize],
    passes: usize,
    f_bits: u32,
) -> (ReadConditionedParams, usize) {
    let mut p = params.clone();
    let mut best = ce_objective(examples, &p, res, cyclic, rows, f_bits);
    let mut accepted = 0usize;
    let relations: Vec<usize> = {
        let mut v: Vec<usize> = examples
            .iter()
            .filter(|e| e.read)
            .map(|e| e.r % GROUP_ORDER)
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    for _ in 0..passes {
        let mut improved = false;
        for r in relations.iter() {
            let saved = p.transport[*r];
            let mut local_best = best;
            let mut local_val = saved;
            for cand in 0..GROUP_ORDER {
                p.transport[*r] = cand as u8;
                let m = ce_objective(examples, &p, res, cyclic, rows, f_bits);
                if ce_objective_better(m, local_best) {
                    local_best = m;
                    local_val = cand as u8;
                }
            }
            p.transport[*r] = local_val;
            if ce_objective_better(local_best, best) {
                best = ce_objective(examples, &p, res, cyclic, rows, f_bits);
                accepted += 1;
                improved = true;
            }
        }
        for i in 0..p.value_code.len() {
            let saved = p.value_code[i];
            let mut local_best = best;
            let mut local_val = saved;
            for cand in 0..GROUP_ORDER {
                p.value_code[i] = cand as u8;
                let m = ce_objective(examples, &p, res, cyclic, rows, f_bits);
                if ce_objective_better(m, local_best) {
                    local_best = m;
                    local_val = cand as u8;
                }
            }
            p.value_code[i] = local_val;
            if ce_objective_better(local_best, best) {
                best = ce_objective(examples, &p, res, cyclic, rows, f_bits);
                accepted += 1;
                improved = true;
            }
        }
        if !improved {
            break;
        }
    }
    (p, accepted)
}

/// The one shared read -> update -> emit step used by evaluation, interventions and generation.
#[allow(clippy::too_many_arguments)]
fn ce_predict(
    ring: &OccurrenceRing,
    cur: u32,
    prev: usize,
    prev2: u32,
    sel: &RelationalSelector,
    table: &ExactGroupTable,
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    params: &ReadConditionedParams,
    res: &EmissionResidual,
    rows: &[usize],
    cyclic: bool,
    use_update: bool,
    allowed: bool,
) -> (Vec<i32>, Read, usize, usize) {
    let mut z = local_logits(parent, local, u, cur, prev, prev2, ring.written());
    let r = read_step(ring, cur, prev as u32, prev2, sel, table, allowed, &z);
    let q0 = local.query_state(cur).min(u.len() - 1);
    let mut q1 = q0;
    if use_update && allowed {
        if let (Some(_), Some(payload)) = (r.action, r.payload) {
            q1 = if cyclic {
                params.update_cyclic(q0, r.rel, payload)
            } else {
                params.update(q0, r.rel, payload)
            };
        }
    }
    res.add_to_logits(q0, q1, rows, &mut z);
    (z, r, q0, q1)
}

/// Input-signature collision report for the update: how many `(q0, relation, payload)` signatures
/// demand more than one distinct target.
fn ce_signature_report(examples: &[CeExample]) -> serde_json::Value {
    let mut m: std::collections::BTreeMap<(usize, usize, u32), std::collections::BTreeSet<u32>> =
        std::collections::BTreeMap::new();
    for ex in examples.iter().filter(|e| e.read) {
        m.entry((ex.q0, ex.r % GROUP_ORDER, ex.payload))
            .or_default()
            .insert(ex.target);
    }
    let ambiguous = m.values().filter(|t| t.len() > 1).count();
    let ambiguous_examples = m
        .values()
        .filter(|t| t.len() > 1)
        .map(|t| t.len())
        .sum::<usize>();
    json!({
        "positions": examples.iter().filter(|e| e.read).count(),
        "distinct_signatures": m.len(),
        "ambiguous_signatures": ambiguous,
        "targets_on_ambiguous_signatures": ambiguous_examples,
    })
}

fn contextual_emission_run() -> Result<ExitCode, String> {
    let mut root = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/contextual-emission-1",
    );
    let mut source_root: Option<PathBuf> = None;
    {
        let mut a = std::env::args().skip(1);
        while let Some(k) = a.next() {
            match k.as_str() {
                "--root" => root = PathBuf::from(a.next().ok_or("--root value")?),
                "--source-root" => source_root = Some(PathBuf::from(a.next().ok_or("value")?)),
                other if other.starts_with("--mode") => {}
                other => return Err(format!("unknown argument {other}")),
            }
        }
    }
    claim(&root).map_err(|e| format!("claim {}: {e}", root.display()))?;
    let started = Instant::now();

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
        return Err("derived tokenizer sha mismatch".into());
    }
    let tokenizer = derive_tokenizer(&tb, VOCAB).map_err(|e| format!("derive: {e}"))?;
    let raw_tok: [u8; 32] = hex_to_bytes(&derived)?.try_into().unwrap();
    let local = QueryHard::from_bytes(&sb, &parent, &parent_digest, &raw_tok)
        .map_err(|e| format!("load S: {e}"))?;
    let u: Vec<Vec<i32>> = (0..120).map(|s| local.row_scores(s)).collect();
    let table = ExactGroupTable::build().map_err(|e| format!("table: {e}"))?;
    let group_digest = group_table_digest(&table);
    let banks = build_banks(&tokenizer, parent.cfg.vocab)?;
    let source_files: Vec<serde_json::Value> = match &source_root {
        Some(sr) => [
            "crates/uor-r4-core/src/native_geometric/learner/relational.rs",
            "crates/uor-r4-core/src/native_geometric/learner/read_conditioned.rs",
            "crates/uor-r4-core/src/native_geometric/learner/contextual_emission.rs",
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
    let executable_sha256 = hex_of(&sha256_bytes(
        &std::env::current_exe()
            .ok()
            .and_then(|p| std::fs::read(p).ok())
            .unwrap_or_default(),
    ));

    let values: Vec<u32> = banks.values_fit.iter().copied().take(8).collect();
    let specs_dev = make_ce_specs(&banks, CE_N_DEV, CE_SEED_DEV, &values);
    let specs_tune = make_ce_specs(&banks, CE_N_TUNE, CE_SEED_TUNE, &values);
    let specs_fresh = make_ce_specs(&banks, CE_N_FRESH, CE_SEED_FINAL, &values);
    let mut used: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for spec in specs_dev
        .iter()
        .chain(specs_tune.iter())
        .chain(specs_fresh.iter())
    {
        used.insert(spec.key);
        used.insert(spec.qrole);
        used.insert(spec.source_role);
        for (r, k, v) in spec.blocks.iter() {
            used.insert(*r);
            used.insert(*k);
            used.insert(*v);
        }
    }
    for v in values.iter() {
        used.insert(*v);
    }
    let out_bank: Vec<u32> = (0..parent.cfg.vocab as u32)
        .rev()
        .filter(|t| !used.contains(t))
        .take(values.len())
        .collect();
    if out_bank.len() < values.len() {
        return Err("no disjoint output bank".into());
    }
    let materialize = |specs: &[CePairSpec]| -> Vec<CeItem> {
        let mut items = Vec::new();
        for (pid, spec) in specs.iter().enumerate() {
            for (member, &vi) in spec.value_idx.iter().enumerate() {
                let tokens = ce_item_tokens(spec, values[vi], out_bank[vi]);
                items.push(CeItem {
                    seq: Seq {
                        tokens,
                        group: 0,
                        answer: out_bank[vi],
                        absent: false,
                        answer_source_abs: None,
                    },
                    value: values[vi],
                    out: out_bank[vi],
                    pair_id: pid,
                    member,
                    expected_payload_abs: (3 * spec.pos + 2) as u32,
                });
            }
        }
        items
    };
    let items_dev = materialize(&specs_dev);
    let items_tune = materialize(&specs_tune);
    let items_fresh = materialize(&specs_fresh);

    let parent_root = PathBuf::from(PARENT_ROOT);
    let selector_bytes = std::fs::read(parent_root.join("artifacts/relational_ctx.rlr2"))
        .map_err(|e| format!("parent artifact: {e}"))?;
    let selector_sha256 = sha256_hex(&selector_bytes);
    if selector_sha256 != PARENT_SHA_RELATIONAL_CTX {
        return Err("contextual selector hash mismatch".into());
    }
    let sel =
        RelationalArtifact::from_bytes(&selector_bytes, &sha256_bytes(&sb), &raw_tok)?.selector;

    let (dev_pos, dev_ex, dev_pairs) = ce_extract(&parent, &local, &u, &table, &sel, &items_dev)?;
    let (tune_pos, tune_ex, tune_pairs) =
        ce_extract(&parent, &local, &u, &table, &sel, &items_tune)?;
    let (fresh_pos, fresh_ex, fresh_pairs) =
        ce_extract(&parent, &local, &u, &table, &sel, &items_fresh)?;

    let absent_ok = dev_pos
        .iter()
        .chain(tune_pos.iter())
        .chain(fresh_pos.iter())
        .all(|p| p.absent);
    let decisive = dev_pos.iter().filter(|p| p.decisive).count();
    let reads = dev_pos.iter().filter(|p| p.read).count();
    let dev_by_pair: std::collections::BTreeMap<usize, Vec<usize>> = {
        let mut m: std::collections::BTreeMap<usize, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (i, pid) in dev_pairs.iter().enumerate() {
            m.entry(*pid).or_default().push(i);
        }
        m
    };
    let identical_local = dev_by_pair
        .values()
        .filter(|v| v.len() == 2 && dev_pos[v[0]].z_local == dev_pos[v[1]].z_local)
        .count();
    if dev_pos.len() != items_dev.len()
        || tune_pos.len() != items_tune.len()
        || fresh_pos.len() != items_fresh.len()
    {
        return Err("contextual extraction lost a declared position".into());
    }
    for (items, positions) in [
        (&items_dev, &dev_pos),
        (&items_tune, &tune_pos),
        (&items_fresh, &fresh_pos),
    ] {
        for (pair, p) in items.chunks_exact(2).zip(positions.chunks_exact(2)) {
            let end = pair[0].seq.tokens.len() - 1;
            let differences: Vec<usize> = pair[0].seq.tokens[..end]
                .iter()
                .zip(&pair[1].seq.tokens[..end])
                .enumerate()
                .filter_map(|(i, (x, y))| (x != y).then_some(i))
                .collect();
            if differences != vec![pair[0].expected_payload_abs as usize]
                || pair[0].out == pair[1].out
                || p[0].z_local != p[1].z_local
                || !p.iter().all(|x| x.absent && x.decisive)
                || pair.iter().any(|it| it.seq.tokens[..end].contains(&it.out))
            {
                return Err("contextual paired instrument is invalid".into());
            }
        }
    }
    let row_hits = dev_pos
        .iter()
        .filter(|p| {
            let b: Vec<i32> = p
                .z_local
                .iter()
                .zip(u[p.q0].iter())
                .map(|(a, c)| a - c)
                .collect();
            (0..u.len()).any(|q| {
                let z: Vec<i32> = b
                    .iter()
                    .zip(u[q].iter())
                    .map(|(a, c)| a.saturating_add(*c))
                    .collect();
                argmax_low(&z) as u32 == p.target
            })
        })
        .count();

    let deficit = dev_pos
        .iter()
        .map(|p| (-p.target_margin).max(0) as i64)
        .max()
        .unwrap_or(0);
    let mut shift = 0u32;
    while shift < 12 && (CE_WIDTH as i64) * 2 * (1i64 << shift) < deficit {
        shift += 1;
    }
    let base_params = {
        let mut p = ReadConditionedParams::identity();
        p.value_domain = values.clone();
        p.value_code = (0..values.len()).map(|i| (i % GROUP_ORDER) as u8).collect();
        for r in 0..GROUP_ORDER {
            p.transport[r] = (r % GROUP_ORDER) as u8;
        }
        p
    };
    let mut learn = |cyclic: bool| -> Result<
        (
            serde_json::Value,
            ReadConditionedParams,
            EmissionResidual,
            Vec<usize>,
        ),
        String,
    > {
        let params0 = base_params.clone();
        let mut res0 = EmissionResidual::seeded(parent.cfg.vocab, 0x51E5_0000);
        res0.shift = shift;
        // One serving-support projection and exact CE are used by every output-fitting forward.
        let (res_before_maps, nll_initial, nll_before_maps, flips_before_maps) = fit_output_block(
            &dev_ex,
            &params0,
            &res0,
            cyclic,
            CE_EPOCHS,
            CE_LR,
            parent.cfg.f_bits,
            CE_MAX_W_ROWS,
            5,
        );
        let rows_before_maps = res_before_maps.active_rows();
        let (params, accepted) = ce_search_maps(
            &dev_ex,
            &params0,
            &res_before_maps,
            cyclic,
            &rows_before_maps,
            4,
            parent.cfg.f_bits,
        );
        let nll_after_maps = served_nll_bits(
            &dev_ex,
            &params,
            &res_before_maps,
            cyclic,
            &rows_before_maps,
            parent.cfg.f_bits,
        );
        // Consume the changed map and retain its incumbent output unless exact served CE improves.
        // The final fitted residual below, rather than the stale pre-map residual, is serialized.
        let (res, refit_initial, nll_final, flips_after_maps) = fit_output_block(
            &dev_ex,
            &params,
            &res_before_maps,
            cyclic,
            CE_EPOCHS,
            CE_LR,
            parent.cfg.f_bits,
            CE_MAX_W_ROWS,
            5,
        );
        if (refit_initial - nll_after_maps).abs() > 1e-12 || nll_final > refit_initial + 1e-12 {
            return Err("post-map output refit violated incumbent CE contract".into());
        }
        let rows = res.active_rows();
        let report = json!({
            "algebra": if cyclic { "cyclic_c120" } else { "signed_h4" },
            "shift": shift,
            "f_bits": parent.cfg.f_bits,
            "loss_units": "bits after fixed-point normalization",
            "output_rows": rows.len(),
            "embedding_seed": 0x51E5_0000u64,
            "coefficient_storage": "one byte per ternary coefficient in RLCE; not bit-packed",
            "nonzero_coefficients": rows.iter().map(|o| res.w[*o].iter().filter(|v| **v != 0).count()).sum::<usize>(),
            "residual_range_bound": res.range_bound(),
            "nll_bits": {
                "initial": nll_initial, "served_output_before_map_search": nll_before_maps,
                "served_after_map_search": nll_after_maps, "post_map_refit_incumbent": refit_initial,
                "served_final": nll_final,
            },
            "distinct_payload_alias_pairs_dev_before": ce_collisions(&dev_ex, &params0),
            "distinct_payload_alias_pairs_dev_after": ce_collisions(&dev_ex, &params),
            "distinct_payload_alias_pairs_final_after": ce_collisions(&fresh_ex, &params),
            "accepted_ternary_flips": flips_before_maps + flips_after_maps,
            "accepted_ternary_flips_before_maps": flips_before_maps,
            "accepted_ternary_flips_after_maps": flips_after_maps,
            "post_map_output_refit_executed": true,
            "output_before_maps_sha256": sha256_hex(&res_before_maps.to_bytes()),
            "output_after_maps_refit_sha256": sha256_hex(&res.to_bytes()),
            "optimization": "shared top-k ternary support; surrogate nats gradients; exact served bits for incumbent retention and coordinate search; observed alias count then CE for map search",
            "accepted_map_changes": accepted,
            "dev_hits": ce_hits(&dev_ex, &params, &res, cyclic, &rows),
            "tune_hits": ce_hits(&tune_ex, &params, &res, cyclic, &rows),
        });
        Ok((report, params, res, rows))
    };
    let (h4_rep, h4_params, h4_res, h4_rows) = learn(false)?;
    let (c120_rep, c120_params, c120_res, c120_rows) = learn(true)?;

    // Serialize and independently load before any reported prediction. The sparse row set is
    // recovered from the loaded artifact. A full predictor comparison fails closed on mismatch.
    let reload = |name: &str,
                  params: ReadConditionedParams,
                  res: EmissionResidual,
                  rows: Vec<usize>,
                  cyclic: bool|
     -> Result<
        (
            ReadConditionedParams,
            EmissionResidual,
            Vec<usize>,
            serde_json::Value,
        ),
        String,
    > {
        let pbytes = params.to_bytes();
        let rbytes = res.to_bytes();
        let pp = format!("artifacts/{name}.rlrc");
        let rp = format!("artifacts/{name}.rlce");
        write_checked(&root, &pp, &pbytes)?;
        write_checked(&root, &rp, &rbytes)?;
        let loaded_p = ReadConditionedParams::from_bytes(
            &std::fs::read(root.join(&pp)).map_err(|e| e.to_string())?,
        )?;
        let loaded_r = EmissionResidual::from_bytes(
            &std::fs::read(root.join(&rp)).map_err(|e| e.to_string())?,
            parent.cfg.vocab,
        )?;
        if params != loaded_p || res != loaded_r || loaded_r.w.len() != parent.cfg.vocab {
            return Err("contextual independent artifact load mismatch".into());
        }
        let loaded_rows = loaded_r.active_rows();
        let mut parity_positions = 0usize;
        for it in items_dev.iter().chain(&items_tune).chain(&items_fresh) {
            let prefix = &it.seq.tokens[..it.seq.tokens.len() - 1];
            let ring = ring_before_current(prefix);
            let i = prefix.len() - 1;
            let before = ce_predict(
                &ring,
                prefix[i],
                prefix[i - 1] as usize,
                prefix[i - 2],
                &sel,
                &table,
                &parent,
                &local,
                &u,
                &params,
                &res,
                &rows,
                cyclic,
                true,
                true,
            );
            let after = ce_predict(
                &ring,
                prefix[i],
                prefix[i - 1] as usize,
                prefix[i - 2],
                &sel,
                &table,
                &parent,
                &local,
                &u,
                &loaded_p,
                &loaded_r,
                &loaded_rows,
                cyclic,
                true,
                true,
            );
            if before != after {
                return Err("contextual loaded predictor parity failure".into());
            }
            parity_positions += 1;
        }
        let receipt = json!({"arm": name, "params_sha256": sha256_hex(&pbytes), "residual_sha256": sha256_hex(&rbytes),
            "params_reload_identical": true, "residual_reload_identical": true,
            "loaded_full_predictor_parity_positions": parity_positions, "loaded_objects_used_for_evaluation_and_generation": true,
            "algebra": if cyclic {"cyclic_c120"} else {"signed_h4"}, "f_bits": parent.cfg.f_bits,
            "selector_sha256": selector_sha256, "group_digest": group_digest});
        Ok((loaded_p, loaded_r, loaded_rows, receipt))
    };
    let (h4_params, h4_res, h4_rows, h4_artifact) =
        reload("h4_emission", h4_params, h4_res, h4_rows, false)?;
    let (c120_params, c120_res, c120_rows, c120_artifact) = reload(
        "cyclic_c120_emission",
        c120_params,
        c120_res,
        c120_rows,
        true,
    )?;
    let artifacts = vec![h4_artifact, c120_artifact];
    let artifact_ok = true;

    let eval_arm = |params: &ReadConditionedParams,
                    res: &EmissionResidual,
                    rows: &[usize],
                    cyclic: bool,
                    use_update: bool,
                    allowed: bool,
                    items: &[CeItem]|
     -> Result<(usize, usize, Vec<(usize, bool)>), String> {
        let mut hits = 0usize;
        let mut reads = 0usize;
        let mut per_pair = Vec::new();
        for it in items.iter() {
            let prefix = &it.seq.tokens[..it.seq.tokens.len() - 1];
            let ring = ring_before_current(prefix);
            let i = prefix.len() - 1;
            let (z, r, _q0, _q1) = ce_predict(
                &ring,
                prefix[i],
                prefix[i - 1] as usize,
                prefix[i - 2],
                &sel,
                &table,
                &parent,
                &local,
                &u,
                params,
                res,
                rows,
                cyclic,
                use_update,
                allowed,
            );
            if r.action.is_some() {
                reads += 1;
            }
            let ok = argmax_low(&z) as u32 == it.seq.answer;
            if ok {
                hits += 1;
            }
            per_pair.push((it.pair_id, ok));
        }
        Ok((hits, reads, per_pair))
    };
    let both = |per_pair: &[(usize, bool)]| -> usize {
        let mut m: std::collections::BTreeMap<usize, Vec<bool>> = std::collections::BTreeMap::new();
        for (pid, ok) in per_pair.iter() {
            m.entry(*pid).or_default().push(*ok);
        }
        m.values()
            .filter(|v| v.len() == 2 && v.iter().all(|x| *x))
            .count()
    };
    let copy_arm = |items: &[CeItem]| -> Result<(usize, Vec<(usize, bool)>), String> {
        let mut hits = 0usize;
        let mut per_pair = Vec::new();
        for it in items.iter() {
            let prefix = &it.seq.tokens[..it.seq.tokens.len() - 1];
            let ring = ring_before_current(prefix);
            let i = prefix.len() - 1;
            let (z, _r) = predict_next(
                &ring,
                prefix[i],
                prefix[i - 1] as usize,
                prefix[i - 2],
                &sel,
                &table,
                &parent,
                &local,
                &u,
                true,
            );
            let ok = argmax_low(&z) as u32 == it.seq.answer;
            if ok {
                hits += 1;
            }
            per_pair.push((it.pair_id, ok));
        }
        Ok((hits, per_pair))
    };

    let mut comparisons: Vec<serde_json::Value> = Vec::new();
    for (label, params, res, rows, cyclic, use_update, allowed) in [
        (
            "local_noread",
            &h4_params,
            &h4_res,
            &h4_rows,
            false,
            false,
            false,
        ),
        (
            "h4_read_conditioned",
            &h4_params,
            &h4_res,
            &h4_rows,
            false,
            true,
            true,
        ),
        (
            "h4_update_disabled",
            &h4_params,
            &h4_res,
            &h4_rows,
            false,
            false,
            true,
        ),
        (
            "h4_read_disabled",
            &h4_params,
            &h4_res,
            &h4_rows,
            false,
            true,
            false,
        ),
        (
            "cyclic_c120_read_conditioned",
            &c120_params,
            &c120_res,
            &c120_rows,
            true,
            true,
            true,
        ),
    ] {
        let (dh, dr, dp) = eval_arm(params, res, rows, cyclic, use_update, allowed, &items_dev)?;
        let (th, _, _) = eval_arm(params, res, rows, cyclic, use_update, allowed, &items_tune)?;
        let (fh, fr, fp) = eval_arm(params, res, rows, cyclic, use_update, allowed, &items_fresh)?;
        comparisons.push(json!({
            "arm": label,
            "dev_hits": dh, "dev_positions": dev_pos.len(), "dev_reads": dr,
            "dev_pairs_both_correct": both(&dp),
            "tune_hits": th, "tune_positions": tune_pos.len(),
            "fresh_hits": fh, "fresh_positions": fresh_pos.len(), "fresh_reads": fr,
            "fresh_pairs_both_correct": both(&fp),
        }));
    }
    let (copy_dev, copy_dp) = copy_arm(&items_dev)?;
    let (copy_fresh, copy_fp) = copy_arm(&items_fresh)?;
    comparisons.push(json!({
        "arm": "scalar_copy_parent",
        "dev_hits": copy_dev, "dev_positions": dev_pos.len(),
        "dev_pairs_both_correct": both(&copy_dp),
        "fresh_hits": copy_fresh, "fresh_positions": fresh_pos.len(),
        "fresh_pairs_both_correct": both(&copy_fp),
    }));

    // Real changed-source causal pairs: identical prefixes but one older payload.
    let mut pair_rows: Vec<serde_json::Value> = Vec::new();
    for spec_idx in 0..specs_fresh.len().min(4) {
        let a = &items_fresh[spec_idx * 2];
        let b = &items_fresh[spec_idx * 2 + 1];
        let first = |it: &CeItem| -> Result<(u32, bool, bool), String> {
            let prefix = &it.seq.tokens[..it.seq.tokens.len() - 1];
            let ring = ring_before_current(prefix);
            let i = prefix.len() - 1;
            let (z, r, q0, q1) = ce_predict(
                &ring,
                prefix[i],
                prefix[i - 1] as usize,
                prefix[i - 2],
                &sel,
                &table,
                &parent,
                &local,
                &u,
                &h4_params,
                &h4_res,
                &h4_rows,
                false,
                true,
                true,
            );
            Ok((argmax_low(&z) as u32, r.action.is_some(), q1 != q0))
        };
        let (fa, ra, ua) = first(a)?;
        let (fb, rb, ub) = first(b)?;
        let za = ce_extract(&parent, &local, &u, &table, &sel, std::slice::from_ref(a))?.0;
        let zb = ce_extract(&parent, &local, &u, &table, &sel, std::slice::from_ref(b))?.0;
        pair_rows.push(json!({
            "pair": spec_idx,
            "value_a": a.value, "answer_a": a.out, "emitted_a": fa, "correct_a": fa == a.out, "read_a": ra, "updated_a": ua,
            "value_b": b.value, "answer_b": b.out, "emitted_b": fb, "correct_b": fb == b.out, "read_b": rb, "updated_b": ub,
            "both_correct": fa == a.out && fb == b.out,
            "identical_local_logits": za.first().map(|p| p.z_local.clone()) == zb.first().map(|p| p.z_local.clone()),
            "different_answers": a.out != b.out,
        }));
    }

    // Short generated continuations on the same shared boundary.
    let mut gen_rows: Vec<serde_json::Value> = Vec::new();
    for it in items_fresh.iter().take(3) {
        let prefix = &it.seq.tokens[..it.seq.tokens.len() - 1];
        let mut ring = ring_before_current(prefix);
        let mut toks = prefix.to_vec();
        let mut steps = Vec::new();
        for _ in 0..3 {
            let i = toks.len() - 1;
            let (z, r, q0, q1) = ce_predict(
                &ring,
                toks[i],
                toks[i - 1] as usize,
                toks[i - 2],
                &sel,
                &table,
                &parent,
                &local,
                &u,
                &h4_params,
                &h4_res,
                &h4_rows,
                false,
                true,
                true,
            );
            let next = argmax_low(&z) as u32;
            steps.push(json!({"read": r.action.is_some(), "updated": q1 != q0, "emitted": next}));
            ring.observe(toks[i]);
            toks.push(next);
        }
        gen_rows.push(json!({
            "answer": it.seq.answer, "emitted": toks[prefix.len()..].to_vec(),
            "decoded": tokenizer.decode(&toks[prefix.len()..]), "steps": steps,
        }));
    }

    // Required controls: a development-fitted constant and a categorical selected-value emitter that
    // uses only the observed selected payload (no intended value reaches serving).
    let mut const_counts: std::collections::BTreeMap<u32, usize> =
        std::collections::BTreeMap::new();
    let mut cat_counts: std::collections::BTreeMap<u32, std::collections::BTreeMap<u32, usize>> =
        std::collections::BTreeMap::new();
    for p in dev_pos.iter() {
        *const_counts.entry(p.target).or_default() += 1;
        *cat_counts
            .entry(p.payload)
            .or_default()
            .entry(p.target)
            .or_default() += 1;
    }
    let const_target = const_counts
        .iter()
        .max_by_key(|(_, c)| **c)
        .map(|(t, _)| *t)
        .unwrap_or(0);
    let cat_table: std::collections::BTreeMap<u32, u32> = cat_counts
        .iter()
        .map(|(payload, m)| {
            (
                *payload,
                m.iter()
                    .max_by_key(|(_, c)| **c)
                    .map(|(t, _)| *t)
                    .unwrap_or(const_target),
            )
        })
        .collect();
    let count_hits =
        |poss: &[CePos], ids: &[usize], f: &dyn Fn(&CePos) -> u32| -> (usize, Vec<(usize, bool)>) {
            let mut hits = 0usize;
            let mut pp = Vec::new();
            for (p, id) in poss.iter().zip(ids.iter()) {
                let ok = f(p) == p.target;
                if ok {
                    hits += 1;
                }
                pp.push((*id, ok));
            }
            (hits, pp)
        };
    let (const_dev, const_dp) = count_hits(&dev_pos, &dev_pairs, &|_| const_target);
    let (const_tune, _) = count_hits(&tune_pos, &tune_pairs, &|_| const_target);
    let (const_final, const_fp) = count_hits(&fresh_pos, &fresh_pairs, &|_| const_target);
    let cat_of = |p: &CePos| cat_table.get(&p.payload).copied().unwrap_or(const_target);
    let (cat_dev, cat_dp) = count_hits(&dev_pos, &dev_pairs, &cat_of);
    let (cat_tune, _) = count_hits(&tune_pos, &tune_pairs, &cat_of);
    let (cat_final, cat_fp) = count_hits(&fresh_pos, &fresh_pairs, &cat_of);
    comparisons.push(json!({
        "arm": "development_constant",
        "dev_hits": const_dev, "dev_positions": dev_pos.len(), "dev_pairs_both_correct": both(&const_dp),
        "tune_hits": const_tune, "tune_positions": tune_pos.len(),
        "fresh_hits": const_final, "fresh_positions": fresh_pos.len(),
        "fresh_pairs_both_correct": both(&const_fp),
        "learned_target": const_target,
    }));
    comparisons.push(json!({
        "arm": "categorical_selected_value",
        "dev_hits": cat_dev, "dev_positions": dev_pos.len(), "dev_pairs_both_correct": both(&cat_dp),
        "tune_hits": cat_tune, "tune_positions": tune_pos.len(),
        "fresh_hits": cat_final, "fresh_positions": fresh_pos.len(),
        "fresh_pairs_both_correct": both(&cat_fp),
        "table_entries": cat_table.len(),
    }));

    // Reader-localization: correct exact occurrence and correct payload value, per split.
    let source_stratum = |poss: &[CePos]| -> serde_json::Value {
        json!({
            "positions": poss.len(),
            "reads": poss.iter().filter(|p| p.read).count(),
            "correct_exact_occurrence": poss.iter().filter(|p| p.correct_source).count(),
            "correct_payload_value": poss.iter().filter(|p| p.correct_value).count(),
        })
    };

    // Compact row file: one line per position with the exact served references and outcomes.
    let mut rows_text = String::new();
    let mut push_rows = |split: &str, poss: &[CePos], items: &[CeItem]| {
        for (i, p) in poss.iter().enumerate() {
            let it = items.get(i);
            rows_text.push_str(&format!(
                "{{\"split\":\"{split}\",\"pair\":{},\"value\":{},\"target\":{},\"selected_payload\":{},\"read\":{},\"correct_exact_occurrence\":{},\"correct_payload_value\":{},\"q0\":{},\"relation\":{},\"target_margin\":{}}}\n",
                it.map(|x| x.pair_id).unwrap_or(0), it.map(|x| x.value).unwrap_or(0), p.target, p.payload,
                p.read, p.correct_source, p.correct_value, p.q0, p.r, p.target_margin
            ));
        }
    };
    push_rows("dev", &dev_pos, &items_dev);
    push_rows("tune", &tune_pos, &items_tune);
    push_rows("final", &fresh_pos, &items_fresh);
    write_checked(&root, "rows.jsonl", rows_text.as_bytes())?;

    let h4_final = comparisons
        .iter()
        .find(|c| c["arm"] == json!("h4_read_conditioned"))
        .cloned()
        .unwrap_or(json!(null));
    let final_pairs = h4_final["fresh_pairs_both_correct"].as_u64().unwrap_or(0);
    let final_total_pairs = (fresh_pos.len() / 2).max(1) as u64;
    let ctrl_pairs = |name: &str| -> u64 {
        comparisons
            .iter()
            .find(|c| c["arm"] == json!(name))
            .and_then(|c| c["fresh_pairs_both_correct"].as_u64())
            .unwrap_or(0)
    };
    let controls_max = [
        "local_noread",
        "scalar_copy_parent",
        "development_constant",
        "categorical_selected_value",
        "cyclic_c120_read_conditioned",
    ]
    .iter()
    .map(|n| ctrl_pairs(n))
    .max()
    .unwrap_or(0);
    let fraction = final_pairs as f64 / final_total_pairs as f64;
    let screen = json!({
        "declared_before_final": {"minimum_both_members_correct_fraction": 0.5,
            "must_exceed": ["local_noread", "scalar_copy_parent", "development_constant", "categorical_selected_value", "cyclic_c120_read_conditioned"]},
        "h4_final_pairs_both_correct": final_pairs,
        "h4_final_pairs": final_total_pairs,
        "h4_final_fraction": fraction,
        "best_control_pairs_both_correct": controls_max,
        "beats_controls": final_pairs > controls_max,
        "met": fraction >= 0.5 && final_pairs > controls_max,
        "note": "engineering screen, not a theorem or alpha threshold; a partial mechanism below it is retained with explicit limitations",
    });

    let result = json!({
        "schema": "uor-r4.contextual-emission/2",
        "base_revision": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
        "running_source": {
            "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
            "git_dirty": std::env::var("UOR_GIT_DIRTY").unwrap_or_else(|_| "unknown".into()),
            "executable_sha256": executable_sha256,
            "source_files": source_files,
        },
        "inputs": {"E_sha256": E_SHA, "S_sha256": S_SHA, "tokenizer_derived": DERIVED_SHA, "group_digest": group_digest, "selector_sha256": selector_sha256, "f_bits": parent.cfg.f_bits},
        "instrument": {
            "design": "paired prefixes identical in query role/key, recent suffix, relevant-block position, distractor roles/values/order; only the relevant source block's value changes",
            "answer": "learned output class of the selected value, from a bank disjoint from every prefix token and every admitted payload",
            "values": values, "output_bank": out_bank,
            "populations": {"dev_pairs": specs_dev.len(), "tune_pairs": specs_tune.len(), "fresh_pairs": specs_fresh.len()},
        },
        "validity": {
            "dev_positions": dev_pos.len(), "absent_targets_held": absent_ok,
            "decisive_local_positions": decisive, "any_read": reads,
            "source_selection_by_split": [
                {"split": "dev", "positions": dev_pos.len(), "correct_occurrence": dev_pos.iter().filter(|p| p.correct_source).count(), "correct_value": dev_pos.iter().filter(|p| p.correct_value).count()},
                {"split": "tune", "positions": tune_pos.len(), "correct_occurrence": tune_pos.iter().filter(|p| p.correct_source).count(), "correct_value": tune_pos.iter().filter(|p| p.correct_value).count()},
                {"split": "reused_acceptance", "positions": fresh_pos.len(), "correct_occurrence": fresh_pos.iter().filter(|p| p.correct_source).count(), "correct_value": fresh_pos.iter().filter(|p| p.correct_value).count()},
            ],
            "pairs_with_identical_local_logits": identical_local,
            "max_target_deficit_units": deficit, "declared_shift": shift,
        },
        "frozen_row_ceiling": {"positions": dev_pos.len(), "some_row_emits_target": row_hits},
        "diagnostics": {
            "distinct_update_signatures_dev": ce_signature_report(&dev_ex),
            "distinct_update_signatures_fresh": ce_signature_report(&fresh_ex),
            "note": "signature = (q0, relation, selected payload). Ambiguous signatures demand different targets from an identical update input; the update cannot separate them without help from the unchanged local logits.",
        },
        "learning": {"h4": h4_rep, "cyclic_c120": c120_rep},
        "comparisons": comparisons,
        "correct_source_stratum": {"dev": source_stratum(&dev_pos), "tune": source_stratum(&tune_pos), "final": source_stratum(&fresh_pos)},
        "screen": screen,
        "changed_source_pairs_fresh": pair_rows,
        "generated": gen_rows,
        "artifacts": artifacts,
        "artifact_reload_all_ok": artifact_ok,
        "scope": "bounded authored context-required instrument; not general language or reasoning. Fixed seeds are now reused development populations; a new held-out result requires separately frozen new data. Schema1 loss/optimizer receipts remain historical, not normalized CE or convergence evidence. Energy UNAVAILABLE; whole-path D0-b not claimed.",
        "elapsed_s": started.elapsed().as_secs_f64(),
    });
    write_json(&root, "result.json", &result)?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    let h4 = comparisons
        .iter()
        .find(|c| c["arm"] == json!("h4_read_conditioned"))
        .cloned()
        .unwrap_or(json!(null));
    println!(
        "contextual-emission: dev {} pos | absent {} | decisive {} | reads {} | identical-local pairs {} | frozen-row ceiling {}/{} | shift {} | H4 dev {}/{} fresh {}/{} both {} | sealed {} unlisted | {:.1}s",
        dev_pos.len(), absent_ok, decisive, reads, identical_local, row_hits, dev_pos.len(), shift,
        h4["dev_hits"], h4["dev_positions"], h4["fresh_hits"], h4["fresh_positions"],
        h4["fresh_pairs_both_correct"], unlisted.len(), started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    let mode = std::env::args().any(|a| a == "--mode=utility-transfer");
    let ce = std::env::args().any(|a| a == "--mode=contextual-emission");
    let result = if ce {
        contextual_emission_run()
    } else if mode {
        utility_transfer_run()
    } else {
        run()
    };
    match result {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod read_conditioned_boundary_tests {
    use super::*;

    #[test]
    fn complete_prefix_keeps_query_key_and_empty_pool_position() {
        // The last two observed tokens are the role and key, not role and an answer sentinel.
        let prefix = [10, 20, 30, 40, 50];
        let (ring, cur, prev, prev2) = rc_prefix_boundary(&prefix).unwrap();
        assert_eq!((cur, prev, prev2, ring.written()), (50, 40, 30, 4));
        let (cands, _) = admit_mixed(&ring, ring.written(), cur, prev, prev2, MAX_CAND);
        assert!(cands.is_empty());
        // No candidate does not move the prediction back to an earlier token or drop the row.
        assert_eq!(ring.written() as usize, prefix.len() - 1);
        assert!(rc_prefix_boundary(&[]).is_err());
        assert!(rc_prefix_boundary(&[10, 20]).is_err());
    }

    #[test]
    fn continuation_accuracy_uses_first_emission_not_last() {
        let tokens = [10, 20, 30, 41, 42, 43, 44];
        assert_eq!(first_continuation_token(&tokens, 3).unwrap(), 41);
        assert_ne!(first_continuation_token(&tokens, 3).unwrap(), tokens[6]);
        assert!(first_continuation_token(&tokens[..3], 3).is_err());
    }

    #[test]
    fn read_conditioned_reload_returns_validated_parameters_or_fails() {
        let expected = ReadConditionedParams::identity();
        let bytes = expected.to_bytes();
        assert_eq!(
            reload_read_conditioned(&bytes, &expected).unwrap(),
            expected
        );
        assert!(reload_read_conditioned(&bytes[..4], &expected).is_err());
        let mut changed = expected.clone();
        changed.transport[0] = (changed.transport[0] + 1) % GROUP_ORDER as u8;
        assert!(reload_read_conditioned(&changed.to_bytes(), &expected).is_err());
    }
}

#[cfg(test)]
mod confidence_boundary_tests {
    use super::*;

    fn selector() -> RelationalSelector {
        RelationalSelector {
            q_roots: vec![0],
            mode: RelMode::Geometric,
            code_of: Vec::new(),
            w: [0; EXACT_FEATS],
            rank: [0; RANKS],
            bias: 0,
            sb: [0; ACTS],
            noread: 0,
            ctx: Vec::new(),
            policy: vec![0; CONF_ADDRESSES],
            gap_thresholds: [-8, -4, -1],
        }
    }

    #[test]
    fn confidence_failures_cannot_be_served_as_the_local_baseline() {
        let expected = selector();
        assert!(require_matching_confidence_load(
            "h4_confidence",
            &expected,
            Err("expected-manifest rejection".into()),
        )
        .is_err());
        let mut changed = expected.clone();
        changed.policy[1] = 3;
        assert!(require_matching_confidence_load("h4_confidence", &expected, Ok(changed)).is_err());
        let mut coarse = expected.clone();
        coarse.policy.truncate(UTIL_BUCKETS);
        assert!(
            require_matching_confidence_load("h4_confidence", &coarse, Ok(coarse.clone())).is_err()
        );
        let got =
            require_matching_confidence_load("h4_confidence", &expected, Ok(expected.clone()))
                .expect("the exact verified selector must remain usable");
        let mut arms = BTreeMap::new();
        assert!(required_confidence_arm(&arms, "h4_confidence").is_err());
        arms.insert("h4_confidence", got);
        assert_eq!(
            required_confidence_arm(&arms, "h4_confidence").unwrap(),
            &expected
        );
        assert!(required_confidence_arm(&arms, "categorical_confidence").is_err());
    }

    #[test]
    fn confidence_witness_requires_nonempty_successful_comparison() {
        assert!(require_witness_parity(0, 0).is_err());
        assert!(require_witness_parity(20, 1).is_err());
        assert!(require_witness_parity(20, 0).is_ok());
    }
}
