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
use uor_r4_core::native_geometric::learner::grounded_session::*;
use uor_r4_core::native_geometric::learner::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};
use uor_r4_core::native_geometric::learner::lexical_realization::*;
use uor_r4_core::native_geometric::learner::observed_text_session::*;
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
use uor_r4_core::native_geometric::learner::relational_session::*;
use uor_r4_core::native_geometric::learner::result_decoder::*;
use uor_r4_core::native_geometric::learner::scoped_memory::*;
use uor_r4_core::native_geometric::learner::shared_transition::*;
use uor_r4_core::native_geometric::learner::state_lexical::*;
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
                        &ring,
                        cur,
                        prev,
                        prev2,
                        &relational_reload,
                        &table,
                        &parent,
                        &local,
                        &u,
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
        get("relational_ctx")["hard_action_ce_bits"]
            .as_f64()
            .unwrap_or(f64::NAN),
        get("relational")["hard_action_ce_bits"]
            .as_f64()
            .unwrap_or(f64::NAN),
        get("exact")["hard_action_ce_bits"]
            .as_f64()
            .unwrap_or(f64::NAN),
        get("local")["hard_action_ce_bits"]
            .as_f64()
            .unwrap_or(f64::NAN),
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
    fn observed_byte_alignment_checks_content_and_preserves_lexical_identity() {
        let raw = std::fs::read(DEFAULT_TOKENIZER).expect("required local tokenizer fixture");
        assert_eq!(sha256_hex(&raw), TOKENIZER_SHA);
        let tokenizer = derive_tokenizer(&raw, VOCAB).unwrap();
        let plain = tokenizer.encode("Ivo");
        let spaced = tokenizer.encode(" Ivo");
        assert_ne!(
            plain, spaced,
            "the identity test must cross a real BPE boundary"
        );
        let a = ob_byte_lengths(&tokenizer, "Ivo", &plain).unwrap();
        let b = ob_byte_lengths(&tokenizer, " Ivo", &spaced).unwrap();
        assert_eq!(
            lexical_key("Ivo", &a, 0, plain.len()),
            lexical_key(" Ivo", &b, 0, spaced.len())
        );
        assert!(ob_byte_lengths(&tokenizer, " Ova", &spaced).is_err());
        for text in ["Cedar", " Cedar Annex", " café", " naïve"] {
            let tokens = tokenizer.encode(text);
            let lengths = ob_byte_lengths(&tokenizer, text, &tokens).unwrap();
            assert_eq!(
                lexical_key(text, &lengths, 0, tokens.len()).unwrap(),
                text.trim_matches(|c: char| c.is_ascii_whitespace())
                    .as_bytes()
            );
        }
    }

    #[test]
    fn object_intervention_preserves_suffix_and_rebases_later_roles() {
        let raw = std::fs::read(DEFAULT_TOKENIZER).expect("required local tokenizer fixture");
        assert_eq!(sha256_hex(&raw), TOKENIZER_SHA);
        let tokenizer = derive_tokenizer(&raw, VOCAB).unwrap();
        let names = ObNames {
            persons: ["Mara", "Ivo", "Cedar", "Oren"],
            offices: ["Fen", "Office Park", "Cedar Annex"],
            projects: ["Atlas", "Lumen", "Project Bay"],
        };
        for style in 0..3 {
            let (clause, label) = ob_build_clause(
                &tokenizer,
                7,
                &ob_assert_form(&names, Goal::Office, "Mara", "Fen", style),
            )
            .unwrap();
            let world = ObWorld {
                id: 1,
                version: 1,
                clauses: vec![clause],
                labels: vec![label],
                oracle: vec![],
                texts: vec![],
            };
            let edited = ob_edit_object(&world, &tokenizer, 7, "Cedar Annex").unwrap();
            assert_eq!(
                edited.clauses[0].text,
                world.clauses[0].text.replace("Fen", "Cedar Annex")
            );
            let c = &edited.clauses[0];
            let l = &edited.labels[0];
            for (span, expected) in [
                (l.subject, "Mara"),
                (l.marker, "office"),
                (l.object.unwrap(), "Cedar Annex"),
            ] {
                assert_eq!(
                    lexical_key(&c.text, &c.byte_lengths, span.0 as usize, span.1).unwrap(),
                    expected.as_bytes()
                );
            }
            assert_eq!(tokenizer.encode(&c.text), c.tokens);
            assert_eq!(edited.texts, vec![c.text.clone()]);
            let restored = ob_edit_object(&edited, &tokenizer, 7, "Fen").unwrap();
            assert_eq!(restored.clauses[0].text, world.clauses[0].text);
            assert_eq!(restored.labels[0].subject, world.labels[0].subject);
            assert_eq!(restored.labels[0].marker, world.labels[0].marker);
        }
    }

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
        d_text_h4,
        d_text_cat,
        corpus(&txt_one_nat, "delta_bits_per_position"),
        d_text_parent,
        present_h4["emitted_correct"].as_u64().unwrap_or(0),
        present_parent["emitted_correct"].as_u64().unwrap_or(0),
        absent_reads_h4,
        absent_reads_parent,
        present_ok,
        absent_ok,
        harm_containment,
        useful_transfer,
        fitted_beats_one_nat_construction,
        fitted_beats_one_nat_text,
        verified_digest,
        unlisted.len(),
        started.elapsed().as_secs_f32()
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

/// An evaluation receipt made only after target-free inference has returned its actual scores.
/// Targets/intended references are scoring metadata and never enter the predictor.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct CePredictionRecord {
    prediction: u32,
    target: u32,
    target_logit: i32,
    strongest_competitor: u32,
    strongest_competitor_logit: i32,
    loss_bits: f64,
    f_bits: u32,
    q0: usize,
    q1: usize,
    read: bool,
    relation: usize,
    selected_payload: Option<u32>,
    selected_source_seq_abs: Option<[u32; 2]>,
    selected_payload_seq_abs: Option<[u32; 2]>,
    intended_source_seq_abs: [u32; 2],
    intended_payload_seq_abs: [u32; 2],
}

#[allow(clippy::too_many_arguments)]
fn ce_prediction_record(
    z: &[i32],
    read: &Read,
    q0: usize,
    q1: usize,
    target: u32,
    f_bits: u32,
    seq: u32,
    expected_payload_abs: u32,
) -> Result<CePredictionRecord, String> {
    if z.len() < 2 || target as usize >= z.len() || expected_payload_abs == 0 {
        return Err("invalid prediction receipt target, vocabulary or intended occurrence".into());
    }
    let mut competitor = if target == 0 { 1 } else { 0 };
    for i in 0..z.len() {
        if i != target as usize && z[i] > z[competitor] {
            competitor = i;
        }
    }
    Ok(CePredictionRecord {
        prediction: argmax_low(z) as u32,
        target,
        target_logit: z[target as usize],
        strongest_competitor: competitor as u32,
        strongest_competitor_logit: z[competitor],
        loss_bits: bits(z, target, f_bits),
        f_bits,
        q0,
        q1,
        read: read.action.is_some(),
        relation: read.rel,
        selected_payload: read.payload,
        selected_source_seq_abs: read.source.map(|r| [r.seq, r.abs]),
        selected_payload_seq_abs: read.payload_abs.map(|abs| [seq, abs]),
        intended_source_seq_abs: [seq, expected_payload_abs - 1],
        intended_payload_seq_abs: [seq, expected_payload_abs],
    })
}

fn ce_prediction_counts(records: &[CePredictionRecord]) -> (usize, usize) {
    (
        records.iter().filter(|r| r.prediction == r.target).count(),
        records.iter().filter(|r| r.read).count(),
    )
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
    let ambiguous_targets = m
        .values()
        .filter(|t| t.len() > 1)
        .map(|t| t.len())
        .sum::<usize>();
    let ambiguous_positions = examples
        .iter()
        .filter(|ex| {
            ex.read
                && m.get(&(ex.q0, ex.r % GROUP_ORDER, ex.payload))
                    .is_some_and(|t| t.len() > 1)
        })
        .count();
    json!({
        "positions": examples.iter().filter(|e| e.read).count(),
        "distinct_signatures": m.len(),
        "ambiguous_signatures": ambiguous,
        "targets_on_ambiguous_signatures": ambiguous_targets,
        "positions_on_ambiguous_signatures": ambiguous_positions,
    })
}

/// Runtime source checkout observation, distinct from caller-supplied build provenance. The
/// executable digest and source-file digests remain authoritative identities; observing a checkout
/// does not by itself prove that this binary was built from it.
fn ce_source_checkout(source_root: Option<&std::path::Path>) -> serde_json::Value {
    let Some(root) = source_root else {
        return json!({"status": "UNAVAILABLE", "reason": "--source-root absent"});
    };
    let git = |args: &[&str]| -> Option<String> {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        String::from_utf8(out.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    };
    let head = git(&["rev-parse", "HEAD"]);
    let changes = git(&["status", "--porcelain"]);
    json!({"path": root, "observed_head": head, "observed_dirty": changes.as_ref().map(|s| !s.is_empty()),
        "scope": "source checkout observed before fitting; binary-to-source build binding is not established by this observation",
        "caller_claimed_git_rev": std::env::var("UOR_GIT_REV").ok(),
        "caller_claimed_git_dirty": std::env::var("UOR_GIT_DIRTY").ok()})
}

/// **Observed served-feature aliases.** Test the real feature
/// used by the emission readout, `R(q0*T[r]*V[v]) - R(q0)`, not the injectivity of the value codes.
/// Among positions whose intended source was selected, how many demand different targets while
/// sharing an identical integer service feature? Distinct state ids and injective value maps are
/// insufficient: two different `q1` can still yield the same `R` difference. The majority-vote
/// ceiling is descriptive on these labelled examples, not a trained decoder or a certificate
/// that a linear decoder is feasible/infeasible. Local logits are not inputs to this grouping.
fn ce_feature_alias_report(
    examples: &[CeExample],
    eligible: &[bool],
    params: &ReadConditionedParams,
    res: &EmissionResidual,
    cyclic: bool,
) -> serde_json::Value {
    let mut m: BTreeMap<Vec<i32>, BTreeMap<u32, usize>> = BTreeMap::new();
    let mut considered = 0usize;
    for (i, ex) in examples.iter().enumerate() {
        if !ex.read || !eligible.get(i).copied().unwrap_or(false) {
            continue;
        }
        let q1 = ex.q1(params, cyclic);
        *m.entry(res.delta(ex.q0, q1).to_vec())
            .or_default()
            .entry(ex.target)
            .or_default() += 1;
        considered += 1;
    }
    let conflicting = m.values().filter(|t| t.len() > 1).count();
    let on_conflict = m
        .values()
        .filter(|t| t.len() > 1)
        .map(|t| t.values().sum::<usize>())
        .sum::<usize>();
    let best_case = m
        .values()
        .map(|t| t.values().copied().max().unwrap_or(0))
        .sum::<usize>();
    json!({
        "source_correct_positions": considered,
        "distinct_service_features": m.len(),
        "features_with_conflicting_targets": conflicting,
        "positions_on_conflicting_features": on_conflict,
        "best_case_from_features_alone": best_case,
        "feature_only_ceiling_fraction": if considered == 0 { serde_json::Value::Null } else { json!(best_case as f64 / considered as f64) },
        "note": "feature = integer R(q1)-R(q0) of the deployed residual; descriptive majority-vote ceiling on the labelled source-correct subset only. Not a development-fitted decoder, held-out generalization result, or linear-feasibility certificate; the full predictor also receives local logits.",
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
    let source_checkout = ce_source_checkout(source_root.as_deref());
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

    let eval_arm =
        |params: &ReadConditionedParams,
         res: &EmissionResidual,
         rows: &[usize],
         cyclic: bool,
         use_update: bool,
         allowed: bool,
         items: &[CeItem]|
         -> Result<(usize, usize, Vec<(usize, bool)>, Vec<CePredictionRecord>), String> {
            let mut records = Vec::new();
            let mut per_pair = Vec::new();
            for it in items.iter() {
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
                    params,
                    res,
                    rows,
                    cyclic,
                    use_update,
                    allowed,
                );
                let record = ce_prediction_record(
                    &z,
                    &r,
                    q0,
                    q1,
                    it.seq.answer,
                    parent.cfg.f_bits,
                    ring.seq(),
                    it.expected_payload_abs,
                )?;
                per_pair.push((it.pair_id, record.prediction == record.target));
                records.push(record);
            }
            let (hits, reads) = ce_prediction_counts(&records);
            Ok((hits, reads, per_pair, records))
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
    let copy_arm =
        |items: &[CeItem]| -> Result<(usize, Vec<(usize, bool)>, Vec<CePredictionRecord>), String> {
            let mut records = Vec::new();
            let mut per_pair = Vec::new();
            for it in items.iter() {
                let prefix = &it.seq.tokens[..it.seq.tokens.len() - 1];
                let ring = ring_before_current(prefix);
                let i = prefix.len() - 1;
                let (z, r) = predict_next(
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
                let q0 = local.query_state(prefix[i]).min(u.len() - 1);
                let record = ce_prediction_record(
                    &z,
                    &r,
                    q0,
                    q0,
                    it.seq.answer,
                    parent.cfg.f_bits,
                    ring.seq(),
                    it.expected_payload_abs,
                )?;
                per_pair.push((it.pair_id, record.prediction == record.target));
                records.push(record);
            }
            let (hits, _) = ce_prediction_counts(&records);
            Ok((hits, per_pair, records))
        };

    let mut comparisons: Vec<serde_json::Value> = Vec::new();
    let mut prediction_records: BTreeMap<(&str, &str), Vec<CePredictionRecord>> = BTreeMap::new();
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
        let (dh, dr, dp, de) =
            eval_arm(params, res, rows, cyclic, use_update, allowed, &items_dev)?;
        let (th, _, _, te) = eval_arm(params, res, rows, cyclic, use_update, allowed, &items_tune)?;
        let (fh, fr, fp, fe) =
            eval_arm(params, res, rows, cyclic, use_update, allowed, &items_fresh)?;
        prediction_records.insert(("dev", label), de);
        prediction_records.insert(("tune", label), te);
        prediction_records.insert(("final", label), fe);
        comparisons.push(json!({
            "arm": label,
            "dev_hits": dh, "dev_positions": dev_pos.len(), "dev_reads": dr,
            "dev_pairs_both_correct": both(&dp),
            "tune_hits": th, "tune_positions": tune_pos.len(),
            "fresh_hits": fh, "fresh_positions": fresh_pos.len(), "fresh_reads": fr,
            "fresh_pairs_both_correct": both(&fp),
        }));
    }
    let (copy_dev, copy_dp, copy_de) = copy_arm(&items_dev)?;
    let (copy_fresh, copy_fp, copy_fe) = copy_arm(&items_fresh)?;
    prediction_records.insert(("dev", "scalar_copy_parent"), copy_de);
    prediction_records.insert(("final", "scalar_copy_parent"), copy_fe);
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

    // One row per actual prefix; arm receipts come from the same predictions counted above.
    let mut rows_text = String::new();
    let mut push_rows = |split: &str, poss: &[CePos], items: &[CeItem]| -> Result<(), String> {
        if poss.len() != items.len() {
            return Err("association row population length differs".into());
        }
        for (i, (p, it)) in poss.iter().zip(items).enumerate() {
            let mut arms = serde_json::Map::new();
            for ((record_split, arm), records) in &prediction_records {
                if *record_split == split {
                    let record = records
                        .get(i)
                        .ok_or("association prediction receipt missing")?;
                    arms.insert(
                        (*arm).into(),
                        serde_json::to_value(record).map_err(|e| e.to_string())?,
                    );
                }
            }
            let row = json!({"split": split, "position": i, "pair": it.pair_id,
                "prefix": &it.seq.tokens[..it.seq.tokens.len()-1], "value": it.value,
                "target": p.target, "selected_payload": p.payload, "read": p.read,
                "correct_exact_occurrence": p.correct_source, "correct_payload_value": p.correct_value,
                "q0": p.q0, "relation": p.r, "target_margin": p.target_margin, "arms": arms,
                "direct_decoder_predictions": {"development_constant": const_target,
                    "categorical_selected_value": cat_of(p)},
                "direct_decoder_note": "direct label decisions have no logit/loss or update state; copy parent is recorded only where evaluated"});
            rows_text.push_str(&serde_json::to_string(&row).map_err(|e| e.to_string())?);
            rows_text.push('\n');
        }
        Ok(())
    };
    push_rows("dev", &dev_pos, &items_dev)?;
    push_rows("tune", &tune_pos, &items_tune)?;
    push_rows("final", &fresh_pos, &items_fresh)?;
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
        "note": "historical strict association comparison retained for continuity, not a requirement that geometry outperform an appropriate learned lexical dictionary and not a capability promotion criterion",
    });

    let result = json!({
        "schema": "uor-r4.contextual-emission/2",
        "parent_review_revision": "036e9c4310ca3d8d2440ac367c577a1df7edf58f",
        "running_source": {
            "source_checkout_observed_before_fit": source_checkout,
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
            "served_feature_separability_dev": ce_feature_alias_report(&dev_ex, &dev_pos.iter().map(|p| p.correct_source).collect::<Vec<_>>(), &h4_params, &h4_res, false),
            "served_feature_separability_fresh": ce_feature_alias_report(&fresh_ex, &fresh_pos.iter().map(|p| p.correct_source).collect::<Vec<_>>(), &h4_params, &h4_res, false),
            "note": "signature = (q0, relation, selected payload). Ambiguous signatures demand different targets from an identical update input; the update cannot separate them without help from the unchanged local logits. The served_feature report tests the actual R(q1)-R(q0) the readout consumes.",
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
        dev_pos.len(),
        absent_ok,
        decisive,
        reads,
        identical_local,
        row_hits,
        dev_pos.len(),
        shift,
        h4["dev_hits"],
        h4["dev_positions"],
        h4["fresh_hits"],
        h4["fresh_positions"],
        h4["fresh_pairs_both_correct"],
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Relation composition: a derived answer from a query operation and retrieved content
// ---------------------------------------------------------------------------
//
// The association instrument answers `out = label(value)`. A payload-only table is the natural
// sufficient mechanism there, so it cannot measure a geometric contribution. This instrument
// instead makes the answer a function of the **query operation composed with the retrieved
// content**: `class = (op + vi) mod 10`, where `op` is the directed relation between the query role
// and the selected source role, and `vi` is the selected value. Selected operation/value cells are
// withheld; every evaluated primitive and output class occurs in development. The declared rule is
// realisable by the served H4 update: `2I` contains an element `h` of order 10, so with
// `T[rel(op)] = h^op` and `V[value(vi)] = h^vi` the served state is `q0 * h^((op+vi) mod 10)`.
// Two cells that share a class therefore share one served state under this witness. Both H4 and
// C120 contain an order-10 subgroup: this cyclic task tests composition, not H4-specific advantage.

const RC2_N_OPS: usize = 8;
const RC2_N_VALUES_DEV: usize = 6;
const RC2_N_VALUES_TEST: usize = 2;
const RC2_DISTRACTORS: usize = 2;
const RC2_CLASS_MODULUS: usize = 10;
/// Repeated contexts per `(op, vi)` cell: the query key, query role and source role are held fixed,
/// so the frozen query state is constant and only the distractor context and placement vary.
const RC2_REPEATS: usize = 4;
const RC2_MAX_W_ROWS: usize = 64;
const RC2_EPOCHS: usize = 600;
const RC2_LR: f64 = 1.0;
/// Fixture v2 repairs the old modulus/split contract. The old roots remain exposed diagnostics.
const RC2_SEED: u64 = 0xC0F1_0002;

#[derive(Clone)]
struct Rc2Item {
    tokens: Vec<u32>,
    answer: u32,
    op: usize,
    vi: usize,
    test: bool,
    source_value: u32,
    expected_payload_abs: u32,
}

struct Rc2Pos {
    q0: usize,
    r: usize,
    payload: u32,
    read: bool,
    target: u32,
    op: usize,
    vi: usize,
    test: bool,
    correct_source: bool,
    correct_value: bool,
    target_margin: i32,
}

/// The declared composition rule. `op` and `vi` are the two independent inputs; the class merges
/// cells whose operation and value indices sum to the same residue.
fn rc2_class(op: usize, vi: usize, k: usize) -> usize {
    (op + vi) % k
}

/// Keep the modulus identical to the witnessed subgroup order. Class coverage is supplied by
/// the cell split rather than by changing the task to an unwitnessed modulus.
fn rc2_choose_modulus(n_ops: usize) -> Result<usize, String> {
    if !(2..=RC2_N_OPS).contains(&n_ops) {
        return Err("composition requires 2..=8 observed operation descriptors".into());
    }
    rc2_validate_split(n_ops, RC2_CLASS_MODULUS)?;
    Ok(RC2_CLASS_MODULUS)
}

/// Two held-out cells per operation. Alternating masks keep every value familiar elsewhere.
fn rc2_test_cell(op: usize, vi: usize) -> bool {
    match op % 2 {
        0 => vi == 1 || vi == 4,
        _ => vi == 2 || vi == 5,
    }
}

/// Check the actual evaluated cell population, not merely coverage of hypothetical residues.
fn rc2_validate_split(n_ops: usize, k: usize) -> Result<(), String> {
    if !(2..=RC2_N_OPS).contains(&n_ops) || k != RC2_CLASS_MODULUS {
        return Err(
            "composition split needs a supported operation count and witnessed modulus 10".into(),
        );
    }
    let n_values = RC2_N_VALUES_DEV + RC2_N_VALUES_TEST;
    let mut dev_ops = std::collections::BTreeSet::new();
    let mut dev_values = std::collections::BTreeSet::new();
    let mut dev_classes = std::collections::BTreeSet::new();
    for op in 0..n_ops {
        for vi in 0..n_values {
            if !rc2_test_cell(op, vi) {
                dev_ops.insert(op);
                dev_values.insert(vi);
                dev_classes.insert(rc2_class(op, vi, k));
            }
        }
    }
    for op in 0..n_ops {
        let held_out = (0..n_values).filter(|vi| rc2_test_cell(op, *vi)).count();
        if held_out != RC2_N_VALUES_TEST {
            return Err("composition split has the wrong held-out cell count".into());
        }
        for vi in 0..n_values {
            if !dev_ops.contains(&op)
                || !dev_values.contains(&vi)
                || !dev_classes.contains(&rc2_class(op, vi, k))
            {
                return Err(format!(
                    "composition cell ({op},{vi}) contains an unseen primitive or output class"
                ));
            }
        }
    }
    Ok(())
}

/// Witness that `2I` contains the order-10 element the declared rule needs, and that power
/// composition reproduces it for every reachable exponent pair. This makes the rule an *exact*
/// property of the served algebra rather than an aspiration, so the instrument is realisable.
fn rc2_order_ten_witness() -> Result<usize, String> {
    let t = group_table();
    let identity = t.identity as usize;
    let mul = |a: usize, b: usize| t.product[a * ROW_STRIDE + b] as usize;
    let pow = |g: usize, e: usize| -> usize {
        let mut x = identity;
        for _ in 0..e {
            x = mul(x, g);
        }
        x
    };
    let mut found = None;
    for g in 0..GROUP_ORDER {
        let mut x = identity;
        let mut order = 0usize;
        for k in 1..=RC2_CLASS_MODULUS + 1 {
            x = mul(x, g);
            if x == identity {
                order = k;
                break;
            }
        }
        if order == RC2_CLASS_MODULUS {
            found = Some(g);
            break;
        }
    }
    let g = found.ok_or("2I has no order-10 element")?;
    for a in 0..RC2_CLASS_MODULUS {
        for b in 0..RC2_CLASS_MODULUS {
            if mul(pow(g, a), pow(g, b)) != pow(g, (a + b) % RC2_CLASS_MODULUS) {
                return Err("order-10 powers do not realise the declared class".into());
            }
        }
    }
    Ok(g)
}

/// Exercise the actual update functions over the selected modulus and every fixture cell.
/// This is a constructed state-representation witness, never learner initialization or serving
/// supervision. It proves neither successful fitting nor successful residual lexical emission.
fn rc2_validate_algebra(
    relations: &[usize],
    values: &[u32],
    k: usize,
    witness: usize,
    q0: usize,
) -> Result<(), String> {
    rc2_validate_split(relations.len(), k)?;
    if values.len() != RC2_N_VALUES_DEV + RC2_N_VALUES_TEST
        || relations.iter().any(|r| *r >= GROUP_ORDER)
        || relations
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != relations.len()
        || GROUP_ORDER % k != 0
    {
        return Err("composition witness has an invalid value/relation domain".into());
    }
    let t = group_table();
    let mul = |a: usize, b: usize| t.product[a * ROW_STRIDE + b] as usize;
    let pow = |e: usize| (0..e).fold(t.identity as usize, |x, _| mul(x, witness));
    let mut h4 = ReadConditionedParams::identity();
    h4.value_domain = values.to_vec();
    h4.value_code = (0..values.len()).map(|vi| pow(vi) as u8).collect();
    let mut cyclic = h4.clone();
    let stride = GROUP_ORDER / k;
    cyclic.value_code = (0..values.len())
        .map(|vi| (vi * stride % GROUP_ORDER) as u8)
        .collect();
    for (op, &rel) in relations.iter().enumerate() {
        h4.transport[rel] = pow(op) as u8;
        cyclic.transport[rel] = (op * stride % GROUP_ORDER) as u8;
    }
    for (op, &rel) in relations.iter().enumerate() {
        for (vi, &value) in values.iter().enumerate() {
            let class = rc2_class(op, vi, k);
            if h4.update(q0, rel, value) != mul(q0, pow(class))
                || cyclic.update_cyclic(q0, rel, value) != (q0 + class * stride) % GROUP_ORDER
            {
                return Err(format!(
                    "actual update does not realize declared cell ({op},{vi}) modulo {k}"
                ));
            }
        }
    }
    Ok(())
}

/// Both population members for every `(op, vi)` cell with a fixed query key, so the frozen query
/// state `q0` is constant and the only varying inputs are the operation and the value. Distractor
/// blocks share the key and use other operations' source roles, so the reader faces a genuine
/// competing-source pool.
fn make_rc2_items(
    banks: &Banks,
    op_pairs: &[(u32, u32)],
    n_ops: usize,
    values: &[u32],
    out_bank: &[u32],
    k: usize,
    seed: u64,
) -> Result<Vec<Rc2Item>, String> {
    rc2_validate_split(n_ops, k)?;
    if op_pairs.len() < n_ops
        || values.len() < RC2_N_VALUES_DEV + RC2_N_VALUES_TEST
        || out_bank.len() < k
    {
        return Err("insufficient bank for the composition instrument".into());
    }
    let key = banks.keys[0];
    let mut st = seed | 1;
    let mut items = Vec::new();
    for op in 0..n_ops {
        let (qrole, source_role) = op_pairs[op];
        for vi in 0..(RC2_N_VALUES_DEV + RC2_N_VALUES_TEST) {
            let test = rc2_test_cell(op, vi);
            for _rep in 0..RC2_REPEATS {
                let mut blocks: Vec<(u32, u32, u32)> = Vec::new();
                for _ in 0..RC2_DISTRACTORS {
                    let other = (op + 1 + (xorshift(&mut st) as usize) % (n_ops - 1)) % n_ops;
                    let role = op_pairs[other].1;
                    let v = values[(xorshift(&mut st) as usize) % values.len()];
                    blocks.push((role, key, v));
                }
                let pos = (xorshift(&mut st) as usize) % (blocks.len() + 1);
                let mut seq_blocks = blocks.clone();
                seq_blocks.insert(pos, (source_role, key, values[vi]));
                let mut tokens = Vec::new();
                let mut expected_payload_abs = 0u32;
                for (i, (r, k, v)) in seq_blocks.iter().enumerate() {
                    if i == pos {
                        expected_payload_abs = (i * 3 + 2) as u32;
                    }
                    tokens.push(*r);
                    tokens.push(*k);
                    tokens.push(*v);
                }
                tokens.push(qrole);
                tokens.push(key);
                let answer = out_bank[rc2_class(op, vi, k)];
                tokens.push(answer);
                items.push(Rc2Item {
                    tokens,
                    answer,
                    op,
                    vi,
                    test,
                    source_value: values[vi],
                    expected_payload_abs,
                });
            }
        }
    }
    Ok(items)
}

fn rc2_extract(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    sel: &RelationalSelector,
    items: &[Rc2Item],
) -> Result<(Vec<Rc2Pos>, Vec<CeExample>), String> {
    let mut positions = Vec::new();
    let mut examples = Vec::new();
    for it in items.iter() {
        let seq = Seq {
            tokens: it.tokens.clone(),
            group: 0,
            answer: it.answer,
            absent: false,
            answer_source_abs: None,
        };
        let obs = observe_full(&seq);
        let want = it.tokens.len() - 2;
        let o = obs
            .iter()
            .find(|o| o.ring.written() as usize == want)
            .ok_or("composition decision point is missing")?;
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
        let q0 = local.query_state(o.cur).min(u.len() - 1);
        let t = (it.answer as usize).min(z.len() - 1);
        let best_other = z
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != t)
            .map(|(_, v)| *v)
            .max()
            .unwrap_or(i32::MIN);
        positions.push(Rc2Pos {
            q0,
            r: selected.rel,
            payload: selected.payload.unwrap_or(0),
            read: selected.action.is_some(),
            target: it.answer,
            op: it.op,
            vi: it.vi,
            test: it.test,
            correct_source: selected.payload_abs == Some(it.expected_payload_abs),
            correct_value: selected.payload == Some(it.source_value),
            target_margin: z[t] - best_other,
        });
        examples.push(CeExample {
            z_local: z,
            q0,
            r: selected.rel,
            payload: selected.payload.unwrap_or(0),
            read: selected.action.is_some(),
            target: it.answer,
        });
    }
    Ok((positions, examples))
}

fn rc2_run() -> Result<ExitCode, String> {
    let mut root = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/relation-composition-v2-1",
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
    let source_checkout = ce_source_checkout(source_root.as_deref());
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

    let parent_root = PathBuf::from(PARENT_ROOT);
    let selector_bytes = std::fs::read(parent_root.join("artifacts/relational_ctx.rlr2"))
        .map_err(|e| format!("parent artifact: {e}"))?;
    let selector_sha256 = sha256_hex(&selector_bytes);
    if selector_sha256 != PARENT_SHA_RELATIONAL_CTX {
        return Err("contextual selector hash mismatch".into());
    }
    let sel =
        RelationalArtifact::from_bytes(&selector_bytes, &sha256_bytes(&sb), &raw_tok)?.selector;

    // Measure the frozen selector's relation projection on these declared pairs. This is not a
    // universal operation capacity: local logits still observe the query role, and a separate
    // learned operation operand could preserve distinctions unnecessary for source selection.
    let rel_of = |qrole: u32, source_role: u32| -> usize {
        relation_index(
            &table,
            sel.mode,
            &sel.code_of,
            *sel.q_roots.get(qrole as usize).unwrap_or(&0),
            *sel.q_roots.get(source_role as usize).unwrap_or(&0),
            qrole as usize,
            source_role as usize,
        )
    };
    let mut op_pairs: Vec<(u32, u32)> = Vec::new();
    let mut seen_rel: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    let mut all_rels: Vec<(u32, u32, usize, u8, u8)> = Vec::new();
    for &(a, b) in banks.pairs.iter() {
        let r = rel_of(a, b) % GROUP_ORDER;
        all_rels.push((
            a,
            b,
            r,
            *sel.q_roots.get(a as usize).unwrap_or(&0),
            *sel.q_roots.get(b as usize).unwrap_or(&0),
        ));
        if seen_rel.insert(r) && op_pairs.len() < RC2_N_OPS {
            op_pairs.push((a, b));
        }
    }
    let available_relations = seen_rel.len();
    let n_ops = op_pairs.len().min(RC2_N_OPS);
    if n_ops < 2 {
        return Err(format!(
            "the served relation interface exposes {available_relations} distinct relative element(s); an operation-conditioned composition needs at least two"
        ));
    }
    let k = rc2_choose_modulus(n_ops)?;

    let witness = rc2_order_ten_witness()?;
    let values: Vec<u32> = banks.values_fit.iter().copied().take(8).collect();
    let mut used: std::collections::BTreeSet<u32> = values.iter().copied().collect();
    used.insert(banks.keys[0]);
    for (a, b) in op_pairs.iter() {
        used.insert(*a);
        used.insert(*b);
    }
    let out_bank: Vec<u32> = (0..parent.cfg.vocab as u32)
        .rev()
        .filter(|t| !used.contains(t))
        .take(k)
        .collect();
    if out_bank.len() < k {
        return Err("no disjoint composition output bank".into());
    }
    let items = make_rc2_items(&banks, &op_pairs, n_ops, &values, &out_bank, k, RC2_SEED)?;
    let dev_items: Vec<Rc2Item> = items.iter().filter(|i| !i.test).cloned().collect();
    let test_items: Vec<Rc2Item> = items.iter().filter(|i| i.test).cloned().collect();

    let relation_exposure: Vec<serde_json::Value> = all_rels
        .iter()
        .map(|(a, b, r, qr, kr)| json!({"qrole": a, "source_role": b, "relation": r, "q_root": qr, "source_root": kr}))
        .collect();

    let (dev_pos, dev_ex) = rc2_extract(&parent, &local, &u, &table, &sel, &dev_items)?;
    let (test_pos, test_ex) = rc2_extract(&parent, &local, &u, &table, &sel, &test_items)?;
    if dev_pos.len() != dev_items.len() || test_pos.len() != test_items.len() {
        return Err("composition extraction lost a declared position".into());
    }
    let all_read = dev_pos.iter().chain(test_pos.iter()).all(|p| p.read);
    let all_source = dev_pos
        .iter()
        .chain(test_pos.iter())
        .all(|p| p.correct_source);
    let q0_values: std::collections::BTreeSet<usize> = dev_pos
        .iter()
        .chain(test_pos.iter())
        .map(|p| p.q0)
        .collect();
    let rels: std::collections::BTreeSet<usize> =
        dev_pos.iter().map(|p| p.r % GROUP_ORDER).collect();
    if q0_values.len() != 1 {
        return Err("composition instrument does not hold a single frozen query state".into());
    }
    let witness_relations: Vec<usize> = op_pairs
        .iter()
        .map(|&(a, b)| rel_of(a, b) % GROUP_ORDER)
        .collect();
    rc2_validate_algebra(
        &witness_relations,
        &values,
        k,
        witness,
        *q0_values.first().ok_or("composition query state absent")?,
    )?;
    // The reader may expose a relation outside the declared operation budget (a distractor or an
    // empty-predecessor candidate). Record the exposure rather than asserting it away.
    let relations_observed = rels.len();

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
    let learn = |cyclic: bool| -> Result<
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
        let (res_before_maps, nll_initial, nll_before_maps, flips_before_maps) = fit_output_block(
            &dev_ex,
            &params0,
            &res0,
            cyclic,
            RC2_EPOCHS,
            RC2_LR,
            parent.cfg.f_bits,
            RC2_MAX_W_ROWS,
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
        let (res, refit_initial, nll_final, flips_after_maps) = fit_output_block(
            &dev_ex,
            &params,
            &res_before_maps,
            cyclic,
            RC2_EPOCHS,
            RC2_LR,
            parent.cfg.f_bits,
            RC2_MAX_W_ROWS,
            5,
        );
        if (refit_initial - nll_after_maps).abs() > 1e-12 || nll_final > refit_initial + 1e-12 {
            return Err(
                "composition post-map output refit violated the incumbent CE contract".into(),
            );
        }
        let rows = res.active_rows();
        let report = json!({
            "algebra": if cyclic { "cyclic_c120" } else { "signed_h4" },
            "shift": shift,
            "f_bits": parent.cfg.f_bits,
            "loss_units": "bits after fixed-point normalization",
            "output_rows": rows.len(),
            "nll_bits": {
                "initial": nll_initial, "served_output_before_map_search": nll_before_maps,
                "served_after_map_search": nll_after_maps, "post_map_refit_incumbent": refit_initial,
                "served_final": nll_final,
            },
            "distinct_payload_alias_pairs_dev_before": ce_collisions(&dev_ex, &params0),
            "distinct_payload_alias_pairs_dev_after": ce_collisions(&dev_ex, &params),
            "accepted_ternary_flips": flips_before_maps + flips_after_maps,
            "accepted_map_changes": accepted,
            "post_map_output_refit_executed": true,
            "dev_hits": ce_hits(&dev_ex, &params, &res, cyclic, &rows),
            "output_after_maps_refit_sha256": sha256_hex(&res.to_bytes()),
        });
        Ok((report, params, res, rows))
    };
    let (h4_rep, h4_params0, h4_res0, _h4_rows0) = learn(false)?;
    let (c120_rep, c120_params0, c120_res0, _c120_rows0) = learn(true)?;

    let reload = |name: &str,
                  params: ReadConditionedParams,
                  res: EmissionResidual,
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
            return Err("composition independent artifact load mismatch".into());
        }
        let loaded_rows = loaded_r.active_rows();
        let unloaded_rows = res.active_rows();
        let mut parity = 0usize;
        for it in items.iter() {
            let prefix = &it.tokens[..it.tokens.len() - 1];
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
                &unloaded_rows,
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
                return Err("composition loaded predictor parity failure".into());
            }
            parity += 1;
        }
        let receipt = json!({"arm": name, "params_sha256": sha256_hex(&pbytes), "residual_sha256": sha256_hex(&rbytes),
            "params_reload_identical": true, "residual_reload_identical": true,
            "loaded_full_predictor_parity_positions": parity,
            "loaded_objects_used_for_evaluation": true,
            "algebra": if cyclic {"cyclic_c120"} else {"signed_h4"}, "f_bits": parent.cfg.f_bits,
            "selector_sha256": selector_sha256, "group_digest": group_digest});
        Ok((loaded_p, loaded_r, loaded_rows, receipt))
    };
    let (h4_params, h4_res, h4_rows, h4_artifact) =
        reload("h4_composition", h4_params0, h4_res0, false)?;
    let (c120_params, c120_res, c120_rows, c120_artifact) =
        reload("cyclic_c120_composition", c120_params0, c120_res0, true)?;
    let artifacts = vec![h4_artifact, c120_artifact];

    let eval_arm = |params: &ReadConditionedParams,
                    res: &EmissionResidual,
                    rows: &[usize],
                    cyclic: bool,
                    use_update: bool,
                    allowed: bool,
                    items: &[Rc2Item]|
     -> Result<(usize, usize, Vec<CePredictionRecord>), String> {
        let mut records = Vec::new();
        for it in items.iter() {
            let prefix = &it.tokens[..it.tokens.len() - 1];
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
                params,
                res,
                rows,
                cyclic,
                use_update,
                allowed,
            );
            records.push(ce_prediction_record(
                &z,
                &r,
                q0,
                q1,
                it.answer,
                parent.cfg.f_bits,
                ring.seq(),
                it.expected_payload_abs,
            )?);
        }
        let (hits, reads) = ce_prediction_counts(&records);
        Ok((hits, reads, records))
    };

    let mut comparisons: Vec<serde_json::Value> = Vec::new();
    let mut prediction_records: BTreeMap<(&str, &str), Vec<CePredictionRecord>> = BTreeMap::new();
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
            "h4_composition",
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
            "cyclic_c120_composition",
            &c120_params,
            &c120_res,
            &c120_rows,
            true,
            true,
            true,
        ),
    ] {
        let (dh, dr, de) = eval_arm(params, res, rows, cyclic, use_update, allowed, &dev_items)?;
        let (th, tr, te) = eval_arm(params, res, rows, cyclic, use_update, allowed, &test_items)?;
        prediction_records.insert(("dev", label), de);
        prediction_records.insert(("held_out_cells", label), te);
        comparisons.push(json!({
            "arm": label,
            "dev_hits": dh, "dev_positions": dev_items.len(), "dev_reads": dr,
            "test_hits": th, "test_positions": test_items.len(), "test_reads": tr,
        }));
    }

    // Fair comparators fitted on development only and given the same observable inputs.
    let mut const_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut pay: BTreeMap<u32, BTreeMap<u32, usize>> = BTreeMap::new();
    let mut two: BTreeMap<(usize, u32), BTreeMap<u32, usize>> = BTreeMap::new();
    for p in dev_pos.iter() {
        *const_counts.entry(p.target).or_default() += 1;
        *pay.entry(p.payload)
            .or_default()
            .entry(p.target)
            .or_default() += 1;
        *two.entry((p.r % GROUP_ORDER, p.payload))
            .or_default()
            .entry(p.target)
            .or_default() += 1;
    }
    let const_target = const_counts
        .iter()
        .max_by_key(|(_, c)| **c)
        .map(|(t, _)| *t)
        .unwrap_or(0);
    let majority = |m: &BTreeMap<u32, usize>| -> u32 {
        m.iter()
            .max_by_key(|(_, c)| **c)
            .map(|(t, _)| *t)
            .unwrap_or(const_target)
    };
    let pay_table: BTreeMap<u32, u32> = pay.iter().map(|(k, m)| (*k, majority(m))).collect();
    let two_table: BTreeMap<(usize, u32), u32> =
        two.iter().map(|(k, m)| (*k, majority(m))).collect();
    let count_hits = |poss: &[Rc2Pos], f: &dyn Fn(&Rc2Pos) -> u32| -> usize {
        poss.iter().filter(|p| f(p) == p.target).count()
    };
    for (label, f) in [
        (
            "development_constant",
            &(|_: &Rc2Pos| const_target) as &dyn Fn(&Rc2Pos) -> u32,
        ),
        (
            "payload_only_table",
            &(|p: &Rc2Pos| pay_table.get(&p.payload).copied().unwrap_or(const_target)),
        ),
        (
            "relation_and_payload_table",
            &(|p: &Rc2Pos| {
                two_table
                    .get(&(p.r % GROUP_ORDER, p.payload))
                    .copied()
                    .unwrap_or(const_target)
            }),
        ),
    ] {
        comparisons.push(json!({
            "arm": label,
            "dev_hits": count_hits(&dev_pos, f), "dev_positions": dev_pos.len(),
            "test_hits": count_hits(&test_pos, f), "test_positions": test_pos.len(),
            "blind_to_operation": label == "payload_only_table",
        }));
    }

    // Diagnostic of exact state reuse, not a necessary condition for learned readout transfer.
    // Honor NoRead exactly as the actual predictor does.
    let state_of = |p: &Rc2Pos| -> usize {
        if p.read {
            h4_params.update(p.q0, p.r, p.payload)
        } else {
            p.q0
        }
    };
    let dev_states: std::collections::BTreeSet<usize> = dev_pos.iter().map(&state_of).collect();
    let test_shared = test_pos
        .iter()
        .filter(|p| dev_states.contains(&state_of(p)))
        .count();
    let test_classes: std::collections::BTreeSet<usize> =
        test_pos.iter().map(|p| rc2_class(p.op, p.vi, k)).collect();
    let dev_classes: std::collections::BTreeSet<usize> =
        dev_pos.iter().map(|p| rc2_class(p.op, p.vi, k)).collect();
    let feature_dev = ce_feature_alias_report(
        &dev_ex,
        &dev_pos.iter().map(|p| p.correct_source).collect::<Vec<_>>(),
        &h4_params,
        &h4_res,
        false,
    );
    let feature_held_out = ce_feature_alias_report(
        &test_ex,
        &test_pos
            .iter()
            .map(|p| p.correct_source)
            .collect::<Vec<_>>(),
        &h4_params,
        &h4_res,
        false,
    );

    let mut rows_text = String::new();
    for (split, poss, row_items) in [
        ("dev", &dev_pos, &dev_items),
        ("held_out_cells", &test_pos, &test_items),
    ] {
        for (i, (p, it)) in poss.iter().zip(row_items).enumerate() {
            let q1 = state_of(p);
            let mut arms = serde_json::Map::new();
            for ((record_split, arm), records) in &prediction_records {
                if *record_split == split {
                    let record = records
                        .get(i)
                        .ok_or("composition prediction receipt missing")?;
                    arms.insert(
                        (*arm).into(),
                        serde_json::to_value(record).map_err(|e| e.to_string())?,
                    );
                }
            }
            let row = json!({"split": split, "position": i, "op": p.op, "value_index": p.vi,
                "class": rc2_class(p.op, p.vi, k), "test": p.test,
                "prefix": &it.tokens[..it.tokens.len()-1], "target": p.target,
                "selected_payload": p.payload, "read": p.read, "correct_source": p.correct_source,
                "correct_value": p.correct_value, "q0": p.q0, "relation": p.r % GROUP_ORDER,
                "q1": q1, "q1_state_seen_in_dev": dev_states.contains(&q1),
                "target_margin": p.target_margin, "arms": arms,
                "direct_decoder_predictions": {"development_constant": const_target,
                    "payload_only_table": pay_table.get(&p.payload).copied().unwrap_or(const_target),
                    "relation_and_payload_table": two_table.get(&(p.r % GROUP_ORDER, p.payload)).copied().unwrap_or(const_target)},
                "direct_decoder_note": "direct label decisions have no logit/loss or update state"});
            rows_text.push_str(&serde_json::to_string(&row).map_err(|e| e.to_string())?);
            rows_text.push('\n');
        }
    }
    write_checked(&root, "rows.jsonl", rows_text.as_bytes())?;

    let find_h = |name: &str| -> u64 {
        comparisons
            .iter()
            .find(|c| c["arm"] == json!(name))
            .and_then(|c| c["test_hits"].as_u64())
            .unwrap_or(0)
    };
    let h4_test = find_h("h4_composition");
    let control_max = [
        "local_noread",
        "h4_update_disabled",
        "h4_read_disabled",
        "development_constant",
        "payload_only_table",
        "relation_and_payload_table",
        "cyclic_c120_composition",
    ]
    .iter()
    .map(|n| find_h(n))
    .max()
    .unwrap_or(0);
    let noncompositional_control_max = [
        "local_noread",
        "h4_update_disabled",
        "h4_read_disabled",
        "development_constant",
        "payload_only_table",
        "relation_and_payload_table",
    ]
    .iter()
    .map(|n| find_h(n))
    .max()
    .unwrap_or(0);
    let screen = json!({
        "comparison_scope": "exploratory familiar-primitive held-out cells on a cyclic task supported by both algebras",
        "h4_test_hits": h4_test, "test_positions": test_items.len(),
        "best_control_test_hits": control_max,
        "best_noncompositional_control_test_hits": noncompositional_control_max,
        "exceeds_noncompositional_controls": h4_test > noncompositional_control_max,
        "cyclic_c120_test_hits": find_h("cyclic_c120_composition"),
        "h4_exceeds_matched_cyclic_arm": h4_test > find_h("cyclic_c120_composition"),
        "historical_strict_all_control_comparison": h4_test > control_max,
        "met": false, "competence_qualification": "NOT_RUN",
        "required_qualification": "complete generated answers and paired changed-source/changed-operation causal controls in addition to familiar-primitive cell transfer; these controls are not executed by this diagnostic mode",
        "note": "H4 need not beat C120 to demonstrate cyclic computation competence. Unique algebra advantage is a separate claim; aggregate hits on this authored fixture do not qualify complete computation or language capability.",
    });

    let result = json!({
        "schema": "uor-r4.relation-composition/2",
        "parent_review_revision": "036e9c4310ca3d8d2440ac367c577a1df7edf58f",
        "running_source": {
            "source_checkout_observed_before_fit": source_checkout,
            "executable_sha256": executable_sha256,
            "source_files": source_files,
        },
        "inputs": {"E_sha256": E_SHA, "S_sha256": S_SHA, "tokenizer_derived": DERIVED_SHA, "group_digest": group_digest, "selector_sha256": selector_sha256, "f_bits": parent.cfg.f_bits},
        "instrument": {
            "fixture_version": 2, "seed": RC2_SEED,
            "exposure": "new repaired fixture after exposed v1 diagnostics; generated held-out cells are not a sealed final language evaluation",
            "design": "fixed query key, four contexts per operation/value cell; alternating held-out cell masks keep every evaluated operation, value and output class familiar in development; distractor blocks share the key and use other operations' source roles",
            "declared_rule": "class = (op + vi) mod 10; the answer token is a disjoint output bank entry per class, absent from every prefix",
            "order_ten_witness_element": witness,
            "realisability": "constructed H4 order-10 and C120 stride-12 maps both pass the actual update function on every fixture cell modulo 10; this witnesses state representation only, not learning, reader correctness or residual output realizability",
            "values": values, "output_bank": out_bank,
            "ops": n_ops, "available_relations": available_relations, "relation_budget_saturating": available_relations < RC2_N_OPS,
            "relation_budget_scope": "frozen selector projection on these enumerated role pairs only; query role still reaches local logits; not a whole-model operation limit",
            "class_modulus": k, "distinct_values": values.len(),
            "development_cells_per_operation": RC2_N_VALUES_DEV, "held_out_cells_per_operation": RC2_N_VALUES_TEST,
            "held_out_cell_rule": "even operation indices withhold values 1 and 4; odd operation indices withhold values 2 and 5",
            "relation_exposure": relation_exposure,
        },
        "validity": {
            "dev_positions": dev_pos.len(), "held_out_positions": test_pos.len(),
            "all_read": all_read, "all_source_selected": all_source,
            "distinct_frozen_query_states": q0_values.len(), "distinct_relations": relations_observed,
            "dev_classes": dev_classes.len(), "held_out_classes": test_classes.len(),
            "held_out_states_already_reached_in_dev": test_shared,
            "max_target_deficit_units": deficit, "declared_shift": shift,
        },
        "diagnostics": {"served_feature_separability_dev": feature_dev, "served_feature_separability_held_out": feature_held_out},
        "learning": {"h4": h4_rep, "cyclic_c120": c120_rep},
        "comparisons": comparisons,
        "screen": screen,
        "artifacts": artifacts,
        "scope": "bounded authored cyclic-composition instrument, equally state-representable in H4 and C120. Not independent evidence that H4 is the best language geometry. Learned transfer and unique algebra advantage are separate questions. Energy UNAVAILABLE; whole-path D0-b not claimed.",
        "elapsed_s": started.elapsed().as_secs_f64(),
    });
    write_json(&root, "result.json", &result)?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "relation-composition: dev {} | held-out {} | all-read {} | all-source {} | frozen q0 {} | dev classes {} held-out classes {} | held-out states shared {} | H4 dev {} held-out {} | best control {} | sealed {} unlisted | {:.1}s",
        dev_pos.len(),
        test_pos.len(),
        all_read,
        all_source,
        q0_values.len(),
        dev_classes.len(),
        test_classes.len(),
        test_shared,
        comparisons
            .iter()
            .find(|c| c["arm"] == json!("h4_composition"))
            .and_then(|c| c["dev_hits"].as_u64())
            .unwrap_or(0),
        h4_test,
        control_max,
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Derived-state decoding: a learned lexical decoder on the relative result
// ---------------------------------------------------------------------------

const DSD_CE_N_DEV: usize = 90;
const DSD_CE_N_TUNE: usize = 60;
const DSD_CE_N_FINAL: usize = 60;
const DSD_SEED_DEV: u64 = 0xC0F2_0011;
const DSD_SEED_TUNE: u64 = 0xC0F2_0012;
const DSD_SEED_FINAL: u64 = 0xC0F2_0021;
const DSD_PASSES: usize = 3;
const DSD_GEN_TOKENS: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DsdPrediction {
    emitted: u32,
    decoder_token: Option<u32>,
    decoder_score: i32,
    read: bool,
    rel: usize,
    payload: Option<u32>,
    payload_abs: Option<u32>,
    source_abs: Option<u32>,
    source_seq: Option<u32>,
    outcome: &'static str,
    computed_state: Option<usize>,
    q0: usize,
    s: usize,
    z: Vec<i32>,
    local_z: Vec<i32>,
}

/// **The one derived-state step.** Teacher-forced evaluation, interventions and generation all call
/// this on the actual observed prefix. The operation input is the observed query role token, never a
/// fixture label. A grounded learned emission wins the argmax; `NoRead` leaves the local prior.
#[allow(clippy::too_many_arguments)]
fn dsd_step(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    sel: &RelationalSelector,
    model: &DerivedStateModel,
    tokens: &[u32],
    use_reader: bool,
    use_update: bool,
) -> Result<DsdPrediction, String> {
    if tokens.len() < 4 {
        return Err("derived-state step needs a complete query prefix".into());
    }
    let ring = ring_before_current(tokens);
    let i = tokens.len() - 1;
    let mut z = local_logits(
        parent,
        local,
        u,
        tokens[i],
        tokens[i - 1] as usize,
        tokens[i - 2],
        ring.written(),
    );
    let r = read_step(
        &ring,
        tokens[i],
        tokens[i - 1],
        tokens[i - 2],
        sel,
        table,
        use_reader,
        &z,
    );
    let qrole = tokens[i - 1];
    let q0 = local.query_state(tokens[i]).min(u.len() - 1);
    let local_z = z.clone();
    let computed = if r.action.is_none() {
        Err("NoRead")
    } else if !use_update {
        Err("UpdateDisabled")
    } else if let Some(payload) = r.payload {
        model.checked_relative_result(r.rel, qrole, payload)
    } else {
        Err("MissingPayload")
    };
    let computed_state = computed.ok();
    let s = computed_state.unwrap_or(group_table().identity as usize);
    let (decoder_token, decoder_score, outcome) = match computed {
        Ok(s) => match model.decoder.decode(s) {
            DecOutcome::Emit { token, score } => (Some(token), score, "Emit"),
            DecOutcome::NoRead => (None, 0, "MissingDecoderState"),
        },
        Err(reason) => (None, 0, reason),
    };
    if let Some(t) = decoder_token {
        if (t as usize) < z.len() {
            let top = z.iter().copied().max().unwrap_or(0);
            z[t as usize] =
                z[t as usize].saturating_add(top.saturating_sub(z[t as usize]).saturating_add(1));
        }
    }
    Ok(DsdPrediction {
        emitted: argmax_low(&z) as u32,
        decoder_token,
        decoder_score,
        read: r.action.is_some(),
        rel: r.rel,
        payload: r.payload,
        payload_abs: r.payload_abs,
        source_abs: r.source.map(|x| x.abs),
        source_seq: r.source.map(|x| x.seq),
        outcome,
        computed_state,
        q0,
        s,
        z,
        local_z,
    })
}

/// Scoring metadata assembled from the returned target-free prediction, never an inference input.
fn dsd_event(
    panel: &str,
    split: &str,
    arm: &str,
    prefix: &[u32],
    pred: &DsdPrediction,
    target: u32,
    intended_payload_abs: u32,
    intended_payload: u32,
) -> serde_json::Value {
    json!({"panel": panel, "split": split, "arm": arm, "prefix": prefix,
        "target": target, "emitted": pred.emitted, "decoder_token": pred.decoder_token,
        "decoder_score": pred.decoder_score, "outcome": pred.outcome,
        "s": pred.computed_state, "q0": pred.q0, "read": pred.read, "relation": pred.rel,
        "query_role": prefix[prefix.len()-2], "selected_payload": pred.payload,
        "selected_source_seq_abs": pred.source_seq.zip(pred.source_abs).map(|(s,a)| [s,a]),
        "selected_payload_seq_abs": pred.source_seq.zip(pred.payload_abs).map(|(s,a)| [s,a]),
        "intended_payload_abs": intended_payload_abs, "intended_payload": intended_payload,
        "correct_source": pred.payload_abs == Some(intended_payload_abs),
        "correct_value": pred.payload == Some(intended_payload),
        "local_emitted": argmax_low(&pred.local_z), "preserves_local_logits": pred.z == pred.local_z,
        "emission_contract": "direct learned label selection overrides the local argmax; decoder counts are not calibrated probabilities"})
}

/// Witness that the declared class rule is realisable through the **derived-state interface**
/// `s = T_bind[r] * U_op[observed query] * V[value]`, and that the served update really is
/// `q1 = q0 * s`, so `inverse(q0) * q1 = s` for every fixture cell. This is a constructed state
/// witness, never learner initialisation or serving supervision.
fn dsd_validate_algebra(
    relations: &[usize],
    qroles: &[u32],
    values: &[u32],
    k: usize,
    witness: usize,
    q0: usize,
) -> Result<(), String> {
    let n_ops = qroles.len();
    if relations.len() != n_ops || values.len() < 8 || GROUP_ORDER % k != 0 {
        return Err("derived-state witness has an invalid domain".into());
    }
    let t = group_table();
    let mul = |a: usize, b: usize| t.product[a * ROW_STRIDE + b] as usize;
    let pow = |e: usize| (0..e).fold(t.identity as usize, |x, _| mul(x, witness));
    let m = DerivedStateModel {
        transport: vec![t.identity; GROUP_ORDER],
        cyclic: false,
        value_only: false,
        strict_operands: true,
        op_domain: qroles.to_vec(),
        op_code: (0..n_ops).map(|op| pow(op) as u8).collect(),
        value_domain: values.to_vec(),
        value_code: (0..values.len()).map(|vi| pow(vi) as u8).collect(),
        decoder: ResultDecoder::default(),
    };
    for op in 0..n_ops {
        for (vi, &value) in values.iter().enumerate() {
            let sv = m.relative_result(relations[op], qroles[op], value);
            if sv != pow(rc2_class(op, vi, k)) {
                return Err(format!(
                    "derived-state interface does not realize cell ({op},{vi})"
                ));
            }
            let q1 = mul(
                mul(q0, m.transport[relations[op]] as usize),
                mul(m.op_state(qroles[op]), m.value_state(value)),
            );
            if mul(t.inverse[q0] as usize, q1) != sv {
                return Err("relative result is not inverse(q0)*q1".into());
            }
        }
    }
    Ok(())
}

fn dsd_fit_json(f: &DecFitReport) -> serde_json::Value {
    json!({
        "initial_hits": f.initial_hits,
        "final_hits": f.final_hits,
        "examples": f.examples,
        "grounded_states": f.states,
        "accepted_moves": f.accepted_moves,
        "supervised_outer_hits": f.probe_hits,
        "supervised_outer_examples": f.probe_examples,
        "objective": "supervised outer hits, then decoder-calibration fit hits, then fewer states",
    })
}

fn dsd_run() -> Result<ExitCode, String> {
    let mut root = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/derived-state-decoder-v2-1",
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
    let source_checkout = ce_source_checkout(source_root.as_deref());
    let source_files: Vec<serde_json::Value> = match &source_root {
        Some(sr) => [
            "crates/uor-r4-core/src/native_geometric/learner/relational.rs",
            "crates/uor-r4-core/src/native_geometric/learner/read_conditioned.rs",
            "crates/uor-r4-core/src/native_geometric/learner/contextual_emission.rs",
            "crates/uor-r4-core/src/native_geometric/learner/result_decoder.rs",
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
    let parent_root = PathBuf::from(PARENT_ROOT);
    let selector_bytes = std::fs::read(parent_root.join("artifacts/relational_ctx.rlr2"))
        .map_err(|e| format!("parent artifact: {e}"))?;
    let selector_sha256 = sha256_hex(&selector_bytes);
    if selector_sha256 != PARENT_SHA_RELATIONAL_CTX {
        return Err("contextual selector hash mismatch".into());
    }
    let sel =
        RelationalArtifact::from_bytes(&selector_bytes, &sha256_bytes(&sb), &raw_tok)?.selector;

    let values: Vec<u32> = banks.values_fit.iter().copied().take(8).collect();

    // ---------------- Panel A: the association regression ----------------
    let specs_dev = make_ce_specs(&banks, DSD_CE_N_DEV, DSD_SEED_DEV, &values);
    let specs_tune = make_ce_specs(&banks, DSD_CE_N_TUNE, DSD_SEED_TUNE, &values);
    let specs_final = make_ce_specs(&banks, DSD_CE_N_FINAL, DSD_SEED_FINAL, &values);
    let mut used: std::collections::BTreeSet<u32> = values.iter().copied().collect();
    for spec in specs_dev
        .iter()
        .chain(specs_tune.iter())
        .chain(specs_final.iter())
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
    let out_bank: Vec<u32> = (0..parent.cfg.vocab as u32)
        .rev()
        .filter(|t| !used.contains(t))
        .take(values.len())
        .collect();
    if out_bank.len() < values.len() {
        return Err("no disjoint association output bank".into());
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
    let items_final = materialize(&specs_final);
    let (a_dev_pos, _, _) = ce_extract(&parent, &local, &u, &table, &sel, &items_dev)?;
    let (a_tune_pos, _, _) = ce_extract(&parent, &local, &u, &table, &sel, &items_tune)?;
    let (a_final_pos, _, _) = ce_extract(&parent, &local, &u, &table, &sel, &items_final)?;
    let a_example = |p: &CePos, it: &CeItem| -> DecExample {
        DecExample {
            r: p.r % GROUP_ORDER,
            qrole: it.seq.tokens[it.seq.tokens.len() - 3],
            payload: p.payload,
            target: p.target,
            read: p.read,
            grounded: p.read && p.correct_source,
        }
    };
    let a_dev_ex: Vec<DecExample> = a_dev_pos
        .iter()
        .zip(items_dev.iter())
        .map(|(p, it)| a_example(p, it))
        .collect();
    let a_tune_ex: Vec<DecExample> = a_tune_pos
        .iter()
        .zip(items_tune.iter())
        .map(|(p, it)| a_example(p, it))
        .collect();
    let a_final_ex: Vec<DecExample> = a_final_pos
        .iter()
        .zip(items_final.iter())
        .map(|(p, it)| a_example(p, it))
        .collect();
    let (a_model, a_fit) =
        fit_derived_state_model(&a_dev_ex, &[], DEC_MAX_STATES, DSD_PASSES, false, false);
    let (a_value_only, a_vo_fit) =
        fit_derived_state_model(&a_dev_ex, &[], DEC_MAX_STATES, DSD_PASSES, false, true);
    // Persist every candidate that is reported and retain the independently loaded objects.
    let export_loaded = |name: &str,
                         model: &DerivedStateModel,
                         prefixes: &[&[u32]]|
     -> Result<(DerivedStateModel, serde_json::Value), String> {
        let bytes = model.to_bytes();
        let path = format!("artifacts/{name}.rlds");
        write_checked(&root, &path, &bytes)?;
        let loaded = DerivedStateModel::from_bytes(
            &std::fs::read(root.join(&path)).map_err(|e| e.to_string())?,
            parent.cfg.vocab,
        )?;
        if loaded != *model {
            return Err("derived-state reload mismatch".into());
        }
        for prefix in prefixes {
            let before = dsd_step(&parent, &local, &u, &table, &sel, model, prefix, true, true)?;
            let after = dsd_step(
                &parent, &local, &u, &table, &sel, &loaded, prefix, true, true,
            )?;
            if before != after {
                return Err("full loaded derived-state predictor parity failure".into());
            }
        }
        let receipt = json!({"arm": name, "artifact_sha256": sha256_hex(&bytes),
            "reload_identical": true, "loaded_parity_positions": prefixes.len(),
            "loaded_candidate_used_after_export": true, "format": "RLDSv2",
            "value_only": loaded.value_only, "strict_operands": loaded.strict_operands,
            "algebra": if loaded.cyclic {"cyclic_c120"} else {"signed_h4"},
            "f_bits": parent.cfg.f_bits, "selector_sha256": selector_sha256, "group_digest": group_digest});
        Ok((loaded, receipt))
    };
    let association_prefixes: Vec<&[u32]> = items_dev
        .iter()
        .chain(&items_tune)
        .chain(&items_final)
        .map(|it| &it.seq.tokens[..it.seq.tokens.len() - 1])
        .collect();
    let (a_model, a_artifact) =
        export_loaded("association_derived_state", &a_model, &association_prefixes)?;
    let (a_value_only, a_value_artifact) = export_loaded(
        "association_value_only",
        &a_value_only,
        &association_prefixes,
    )?;
    let a_artifacts = vec![a_artifact, a_value_artifact];
    // Comparators on the same panel: the frozen local prior and a development-fitted
    // selected-value dictionary (the strongest arbitrary-label mechanism).
    let a_local_hits = |items: &[CeItem]| -> usize {
        items
            .iter()
            .filter(|it| {
                let prefix = &it.seq.tokens[..it.seq.tokens.len() - 1];
                let p = dsd_step(
                    &parent, &local, &u, &table, &sel, &a_model, prefix, false, false,
                )
                .map(|x| x.emitted)
                .unwrap_or(u32::MAX);
                p == it.seq.answer
            })
            .count()
    };
    let mut a_dict: BTreeMap<u32, BTreeMap<u32, usize>> = BTreeMap::new();
    for (p, it) in a_dev_pos.iter().zip(items_dev.iter()) {
        *a_dict
            .entry(p.payload)
            .or_default()
            .entry(it.seq.answer)
            .or_default() += 1;
    }
    let a_majority = |m: &BTreeMap<u32, usize>| -> u32 {
        m.iter()
            .max_by_key(|(_, c)| **c)
            .map(|(t, _)| *t)
            .unwrap_or(0)
    };
    let a_table: BTreeMap<u32, u32> = a_dict.iter().map(|(k, m)| (*k, a_majority(m))).collect();
    let mut prediction_events: Vec<serde_json::Value> = Vec::new();
    let mut a_arms: Vec<serde_json::Value> = Vec::new();
    for (split, ex, poss, items) in [
        ("dev", &a_dev_ex, &a_dev_pos, &items_dev),
        ("tune", &a_tune_ex, &a_tune_pos, &items_tune),
        ("final", &a_final_ex, &a_final_pos, &items_final),
    ] {
        let mut full = Vec::new();
        let mut value_only = Vec::new();
        for it in items.iter() {
            let prefix = &it.seq.tokens[..it.seq.tokens.len() - 1];
            for (arm, model, records) in [
                ("association_derived_state", &a_model, &mut full),
                ("association_value_only", &a_value_only, &mut value_only),
            ] {
                let pred = dsd_step(&parent, &local, &u, &table, &sel, model, prefix, true, true)?;
                records.push((
                    pred.emitted == it.seq.answer,
                    pred.decoder_token == Some(it.seq.answer),
                    pred.decoder_token.is_none(),
                ));
                let mut event = dsd_event(
                    "association",
                    split,
                    arm,
                    prefix,
                    &pred,
                    it.seq.answer,
                    it.expected_payload_abs,
                    it.value,
                );
                event["categorical_selected_value_prediction"] = json!(pred
                    .payload
                    .filter(|_| pred.read)
                    .and_then(|v| a_table.get(&v).copied())
                    .unwrap_or_else(|| argmax_low(&pred.local_z) as u32));
                event["pair"] = json!(it.pair_id);
                event["member"] = json!(it.member);
                prediction_events.push(event);
            }
        }
        let dh = full.iter().filter(|r| r.1).count();
        let dt = full.len();
        let n = poss.len();
        let no_read = full.iter().filter(|r| r.2).count();
        let vh = value_only.iter().filter(|r| r.1).count();
        let dictionary_hits = prediction_events
            .iter()
            .filter(|e| {
                e["panel"] == "association"
                    && e["split"] == split
                    && e["arm"] == "association_value_only"
                    && e["categorical_selected_value_prediction"] == e["target"]
            })
            .count();
        a_arms.push(
            json!({"arm": "association_derived_state", "split": split, "positions": n,
            "decoder_hits": dh, "decoder_total": dt, "no_read_positions": no_read,
            "value_only_decoder_hits": vh,
            "emitted_hits": full.iter().filter(|r| r.0).count(),
            "value_only_emitted_hits": value_only.iter().filter(|r| r.0).count(),
            "evaluation": "actual target-free step on independently loaded candidates",
            "local_hits": a_local_hits(items), "selected_value_dictionary_hits": dictionary_hits,
            "grounded_positions": ex.iter().filter(|e| e.grounded).count(),
            "correct_source_positions": poss.iter().filter(|p| p.correct_source).count()}),
        );
    }

    // ---------------- Panel B: the repaired composition fixture ----------------
    let rel_of = |qrole: u32, source_role: u32| -> usize {
        relation_index(
            &table,
            sel.mode,
            &sel.code_of,
            *sel.q_roots.get(qrole as usize).unwrap_or(&0),
            *sel.q_roots.get(source_role as usize).unwrap_or(&0),
            qrole as usize,
            source_role as usize,
        )
    };
    // The operation is carried by the **observed query role token**, not by the binding relation:
    // the frozen 14 role pairs expose only two relative elements, which is a property of those
    // descriptors, not of the algebra. Operation identity therefore uses as many observed query
    // role tokens as the declared fixture, while `r` keeps the source-compatibility role.
    let op_pairs: Vec<(u32, u32)> = banks.pairs.iter().copied().take(RC2_N_OPS).collect();
    let n_ops = op_pairs.len().min(RC2_N_OPS);
    if n_ops < 2 {
        return Err("composition needs at least two declared operations".into());
    }
    let relations: Vec<usize> = op_pairs
        .iter()
        .map(|(a, b)| rel_of(*a, *b) % GROUP_ORDER)
        .collect();
    let binding_budget = relations
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let k = rc2_choose_modulus(n_ops)?;
    let witness = rc2_order_ten_witness()?;
    let mut comp_used: std::collections::BTreeSet<u32> = values.iter().copied().collect();
    comp_used.insert(banks.keys[0]);
    for (qa, qb) in op_pairs.iter() {
        comp_used.insert(*qa);
        comp_used.insert(*qb);
    }
    let comp_out: Vec<u32> = (0..parent.cfg.vocab as u32)
        .rev()
        .filter(|t| !comp_used.contains(t))
        .take(k)
        .collect();
    if comp_out.len() < k {
        return Err("no disjoint composition output bank".into());
    }
    let comp_items = make_rc2_items(&banks, &op_pairs, n_ops, &values, &comp_out, k, RC2_SEED)?;
    let comp_dev: Vec<Rc2Item> = comp_items.iter().filter(|i| !i.test).cloned().collect();
    let comp_test: Vec<Rc2Item> = comp_items.iter().filter(|i| i.test).cloned().collect();
    let (b_dev_pos, _) = rc2_extract(&parent, &local, &u, &table, &sel, &comp_dev)?;
    let (b_test_pos, _) = rc2_extract(&parent, &local, &u, &table, &sel, &comp_test)?;
    let comp_q0 = b_dev_pos
        .first()
        .map(|p| p.q0)
        .ok_or("composition probe lost its query state")?;
    let op_qroles: Vec<u32> = op_pairs.iter().map(|(a, _)| *a).collect();
    dsd_validate_algebra(&relations, &op_qroles, &values, k, witness, comp_q0)?;
    let b_example = |p: &Rc2Pos, it: &Rc2Item| -> DecExample {
        DecExample {
            r: p.r % GROUP_ORDER,
            qrole: it.tokens[it.tokens.len() - 3],
            payload: p.payload,
            target: p.target,
            read: p.read,
            grounded: p.read && p.correct_source,
        }
    };
    let b_dev_ex: Vec<DecExample> = b_dev_pos
        .iter()
        .zip(comp_dev.iter())
        .map(|(p, it)| b_example(p, it))
        .collect();
    let b_test_ex: Vec<DecExample> = b_test_pos
        .iter()
        .zip(comp_test.iter())
        .map(|(p, it)| b_example(p, it))
        .collect();
    // Supervised outer-development operations: their labels guide every coordinate search.
    // Decoder calibration uses fit cells; both sets initialize learnable operand domains.
    let b_cells: Vec<usize> = comp_dev.iter().map(|it| it.op * 64 + it.vi).collect();
    let probe_ops: std::collections::BTreeSet<usize> =
        (n_ops.saturating_sub(2).max(1)..n_ops).collect();
    let b_mask: Vec<bool> = comp_dev
        .iter()
        .map(|it| !probe_ops.contains(&it.op))
        .collect();
    {
        let fit_targets: std::collections::BTreeSet<u32> = comp_dev
            .iter()
            .zip(b_mask.iter())
            .filter(|(_, m)| **m)
            .map(|(it, _)| it.answer)
            .collect();
        let probe_targets: std::collections::BTreeSet<u32> = comp_dev
            .iter()
            .zip(b_mask.iter())
            .filter(|(_, m)| !**m)
            .map(|(it, _)| it.answer)
            .collect();
        if !probe_targets.iter().all(|t| fit_targets.contains(t)) || b_mask.iter().all(|m| *m) {
            return Err("development probe operations do not cover the fit classes".into());
        }
    }
    let b_fit_ex: Vec<DecExample> = b_dev_ex
        .iter()
        .zip(b_mask.iter())
        .filter(|(_, m)| **m)
        .map(|(e, _)| *e)
        .collect();
    let b_probe_ex: Vec<DecExample> = b_dev_ex
        .iter()
        .zip(b_mask.iter())
        .filter(|(_, m)| !**m)
        .map(|(e, _)| *e)
        .collect();
    let (b_h4, b_h4_fit) = fit_derived_state_model(
        &b_fit_ex,
        &b_probe_ex,
        DEC_MAX_STATES,
        DSD_PASSES,
        false,
        false,
    );
    let (b_c120, b_c120_fit) = fit_derived_state_model(
        &b_fit_ex,
        &b_probe_ex,
        DEC_MAX_STATES,
        DSD_PASSES,
        true,
        false,
    );
    let composition_prefixes: Vec<&[u32]> = comp_items
        .iter()
        .map(|it| &it.tokens[..it.tokens.len() - 1])
        .collect();
    let (b_h4, b_h4_artifact) =
        export_loaded("composition_derived_state_h4", &b_h4, &composition_prefixes)?;
    let (b_c120, b_c120_artifact) = export_loaded(
        "composition_derived_state_c120",
        &b_c120,
        &composition_prefixes,
    )?;
    let b_artifacts = vec![b_h4_artifact, b_c120_artifact];
    // Both-input comparators with an explicit unseen-cell fallback.
    let mut b_const_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut b_pay: BTreeMap<u32, BTreeMap<u32, usize>> = BTreeMap::new();
    let mut b_two: BTreeMap<(u32, u32), BTreeMap<u32, usize>> = BTreeMap::new();
    for (p, it) in b_dev_pos.iter().zip(comp_dev.iter()) {
        let qrole = it.tokens[it.tokens.len() - 3];
        *b_const_counts.entry(p.target).or_default() += 1;
        *b_pay
            .entry(p.payload)
            .or_default()
            .entry(p.target)
            .or_default() += 1;
        *b_two
            .entry((qrole, p.payload))
            .or_default()
            .entry(p.target)
            .or_default() += 1;
    }
    let b_const = b_const_counts
        .iter()
        .max_by_key(|(_, c)| **c)
        .map(|(t, _)| *t)
        .unwrap_or(0);
    let b_majority = |m: &BTreeMap<u32, usize>| -> u32 {
        m.iter()
            .max_by_key(|(_, c)| **c)
            .map(|(t, _)| *t)
            .unwrap_or(b_const)
    };
    let b_pay_table: BTreeMap<u32, u32> = b_pay.iter().map(|(x, m)| (*x, b_majority(m))).collect();
    let b_two_table: BTreeMap<(u32, u32), u32> =
        b_two.iter().map(|(x, m)| (*x, b_majority(m))).collect();
    let mut b_arm = |split: &str,
                     label: &str,
                     model: Option<&DerivedStateModel>,
                     items: &[Rc2Item],
                     f: &dyn Fn(&Rc2Item, &DsdPrediction) -> u32|
     -> Result<serde_json::Value, String> {
        let mut hits = 0usize;
        let mut reads = 0usize;
        let mut no_read = 0usize;
        for it in items.iter() {
            let prefix = &it.tokens[..it.tokens.len() - 1];
            let pred = match model {
                Some(m) => dsd_step(&parent, &local, &u, &table, &sel, m, prefix, true, true)?,
                None => dsd_step(
                    &parent, &local, &u, &table, &sel, &b_h4, prefix, false, false,
                )?,
            };
            if pred.read {
                reads += 1;
            }
            if pred.decoder_token.is_none() {
                no_read += 1;
            }
            let emitted = f(it, &pred);
            if emitted == it.answer {
                hits += 1;
            }
            let mut event = dsd_event(
                "composition",
                split,
                label,
                prefix,
                &pred,
                it.answer,
                it.expected_payload_abs,
                it.source_value,
            );
            event["arm_emitted"] = json!(emitted);
            event["op"] = json!(it.op);
            event["value_index"] = json!(it.vi);
            event["class"] = json!(rc2_class(it.op, it.vi, k));
            prediction_events.push(event);
        }
        Ok(
            json!({"arm": label, "split": split, "positions": items.len(), "hits": hits,
            "reads": reads, "decoder_no_read": no_read}),
        )
    };
    let identity_map = |_: &Rc2Item, p: &DsdPrediction| p.emitted;
    let mut b_arms: Vec<serde_json::Value> = Vec::new();
    for (split, items) in [("dev_cells", &comp_dev), ("held_out_cells", &comp_test)] {
        let payload_only = |_: &Rc2Item, p: &DsdPrediction| {
            b_pay_table
                .get(&p.payload.unwrap_or(0))
                .copied()
                .unwrap_or(b_const)
        };
        let two_input = |it: &Rc2Item, p: &DsdPrediction| {
            let qrole = it.tokens[it.tokens.len() - 3];
            b_two_table
                .get(&(qrole, p.payload.unwrap_or(0)))
                .copied()
                .unwrap_or(b_const)
        };
        b_arms.push(b_arm(split, "local_noread", None, items, &identity_map)?);
        b_arms.push(b_arm(
            split,
            "h4_derived_state",
            Some(&b_h4),
            items,
            &identity_map,
        )?);
        b_arms.push(b_arm(
            split,
            "c120_derived_state",
            Some(&b_c120),
            items,
            &identity_map,
        )?);
        b_arms.push(b_arm(
            split,
            "payload_only_table",
            Some(&b_h4),
            items,
            &payload_only,
        )?);
        b_arms.push(b_arm(
            split,
            "operation_and_payload_table",
            Some(&b_h4),
            items,
            &two_input,
        )?);
        b_arms.push(b_arm(
            split,
            "development_constant",
            Some(&b_h4),
            items,
            &(|_: &Rc2Item, _: &DsdPrediction| b_const),
        )?);
    }
    // ---------------- Interventions on the shared step ----------------
    let key0 = banks.keys[0];
    let key1 = banks.keys.get(1).copied().unwrap_or(key0);
    let (q0role, s0role) = op_pairs[0];
    let (q1role, _s1role) = op_pairs[1 % n_ops];
    let drole = op_pairs[2 % n_ops].1;
    let base =
        |qrole: u32, srole: u32, value: u32| -> Vec<u32> { vec![srole, key0, value, qrole, key0] };
    let with_distractor = |qrole: u32, srole: u32, value: u32, dv: u32| -> Vec<u32> {
        vec![drole, key0, dv, srole, key0, value, qrole, key0]
    };
    let filler_only = |qrole: u32| -> Vec<u32> { vec![drole, key1, values[0], qrole, key0] };
    let run = |tokens: &[u32], model: &DerivedStateModel, use_reader: bool, use_update: bool| {
        dsd_step(
            &parent, &local, &u, &table, &sel, model, tokens, use_reader, use_update,
        )
    };
    let v0 = values[0];
    let v1 = values[1 % values.len()];
    let payload_a = base(q0role, s0role, v0);
    let payload_b = base(q0role, s0role, v1);
    let op_a = base(q0role, s0role, v0);
    // Only the query operation changes; source role, key and payload remain byte-identical.
    let op_b = base(q1role, s0role, v0);
    let dist_a = with_distractor(q0role, s0role, v0, values[2 % values.len()]);
    let dist_b = with_distractor(q0role, s0role, v0, values[3 % values.len()]);
    let removed = filler_only(q0role);
    let (pa, pb) = (
        run(&payload_a, &b_h4, true, true)?,
        run(&payload_b, &b_h4, true, true)?,
    );
    let (oa, ob) = (
        run(&op_a, &b_h4, true, true)?,
        run(&op_b, &b_h4, true, true)?,
    );
    let (da, db) = (
        run(&dist_a, &b_h4, true, true)?,
        run(&dist_b, &b_h4, true, true)?,
    );
    let rem = run(&removed, &b_h4, true, true)?;
    let rd = run(&payload_a, &b_h4, false, true)?;
    let ud = run(&payload_a, &b_h4, true, false)?;
    let expected_a = comp_out[rc2_class(0, 0, k)];
    let expected_b = comp_out[rc2_class(0, 1 % values.len(), k)];
    let expected_op_b = comp_out[rc2_class(1 % n_ops, 0, k)];
    let identity_cell = (0..n_ops)
        .flat_map(|op| (0..values.len()).map(move |vi| (op, vi)))
        .find(|(op, vi)| rc2_class(*op, *vi, k) == 0)
        .ok_or("composition fixture has no identity-class cell")?;
    let identity_tokens = base(
        op_pairs[identity_cell.0].0,
        op_pairs[identity_cell.0].1,
        values[identity_cell.1],
    );
    let identity_pred = run(&identity_tokens, &b_h4, true, true)?;
    let controlled_operation = if let Some(payload) = oa.payload.filter(|_| oa.read) {
        let first = b_h4.decode(oa.rel, q0role, payload);
        let second = b_h4.decode(oa.rel, q1role, payload);
        let token = |d| match d {
            DecOutcome::Emit { token, .. } => Some(token),
            DecOutcome::NoRead => None,
        };
        json!({"scope": "decoder computation only; binding relation and actually selected payload are held fixed, source selection is not rerun",
            "fixed_relation": oa.rel, "fixed_payload": payload,
            "emitted_a": token(first), "emitted_b": token(second),
            "expected_a": expected_a, "expected_b": expected_op_b,
            "both_expected": token(first) == Some(expected_a) && token(second) == Some(expected_op_b)})
    } else {
        json!({"status": "UNAVAILABLE", "reason": "base query did not select a source"})
    };
    let interventions = json!({
        "payload_changed_identical_query": {
            "emitted_a": pa.emitted, "emitted_b": pb.emitted,
            "expected_a": expected_a, "expected_b": expected_b,
            "changed": pa.emitted != pb.emitted,
            "both_expected": pa.emitted == expected_a && pb.emitted == expected_b,
            "selected_a": pa.payload, "selected_b": pb.payload,
        },
        "operation_changed_fixed_evidence": {
            "prefix_a": op_a, "prefix_b": op_b,
            "only_query_operation_changes": op_a[..3] == op_b[..3],
            "selected_payload_a": oa.payload, "selected_payload_b": ob.payload,
            "selected_source_abs_a": oa.source_abs, "selected_source_abs_b": ob.source_abs,
            "expected_source_abs": 1, "expected_payload_abs": 2,
            "outcome_a": oa.outcome, "outcome_b": ob.outcome,
            "emitted_a": oa.emitted, "emitted_b": ob.emitted,
            "expected_a": expected_a, "expected_b": expected_op_b,
            "changed": oa.emitted != ob.emitted,
            "both_expected": oa.emitted == expected_a && ob.emitted == expected_op_b,
        },
        "operation_changed_fixed_selected_computation": controlled_operation,
        "irrelevant_distractor_changed": {
            "emitted_a": da.emitted, "emitted_b": db.emitted,
            "preserved": da.emitted == db.emitted,
            "matches_clean": da.emitted == expected_a && db.emitted == expected_a,
        },
        "required_source_removed": {
            "emitted": rem.emitted, "read": rem.read,
            "decoder_noread": rem.decoder_token.is_none(),
            "equals_local": rem.z == rem.local_z && rem.emitted == argmax_low(&rem.local_z) as u32,
            "outcome": rem.outcome,
        },
        "read_disabled": {"read": rd.read, "decoder_noread": rd.decoder_token.is_none(),
            "equals_local": rd.z == rd.local_z && rd.emitted == argmax_low(&rd.local_z) as u32, "outcome": rd.outcome},
        "update_disabled": {"read": ud.read, "decoder_noread": ud.decoder_token.is_none(),
            "equals_local": ud.z == ud.local_z && ud.emitted == argmax_low(&ud.local_z) as u32, "outcome": ud.outcome},
        "fixture_class_zero_versus_absence": {
            "identity_cell": [identity_cell.0, identity_cell.1],
            "identity_class": rc2_class(identity_cell.0, identity_cell.1, k),
            "identity_expected": comp_out[0],
            "identity_emitted": identity_pred.emitted,
            "identity_is_emit_not_noread": identity_pred.decoder_token.is_some(),
            "actual_computed_state": identity_pred.computed_state,
            "actual_group_identity": group_table().identity,
            "is_actual_group_identity": identity_pred.computed_state == Some(group_table().identity as usize),
            "scope": "fixture class zero does not guarantee the learned latent state is the group identity; actual identity contract is separately unit tested",
            "identity_correct": identity_pred.emitted == comp_out[0],
            "absent_read_noread": rem.decoder_token.is_none(),
        },
    });
    // ---------------- Short loaded rollouts (one supervised answer, three generated tokens) ----------------
    let mut generation: Vec<serde_json::Value> = Vec::new();
    let association_inputs: Vec<(Vec<u32>, u32)> = items_final
        .iter()
        .take(3)
        .map(|it| {
            (
                it.seq.tokens[..it.seq.tokens.len() - 1].to_vec(),
                it.seq.answer,
            )
        })
        .collect();
    let composition_inputs: Vec<(Vec<u32>, u32)> = comp_test
        .iter()
        .take(3)
        .map(|it| (it.tokens[..it.tokens.len() - 1].to_vec(), it.answer))
        .collect();
    for (panel, arm, model, inputs) in [
        (
            "association",
            "value_only",
            &a_value_only,
            &association_inputs,
        ),
        ("composition", "h4", &b_h4, &composition_inputs),
        ("composition", "c120", &b_c120, &composition_inputs),
    ] {
        for (prefix, answer) in inputs {
            let mut tokens = prefix.clone();
            let mut emitted = Vec::new();
            let mut steps = Vec::new();
            for _ in 0..DSD_GEN_TOKENS {
                let pred = run(&tokens, model, true, true)?;
                steps.push(json!({"outcome": pred.outcome, "read": pred.read, "s": pred.computed_state,
                "emitted": pred.emitted, "selected_payload": pred.payload,
                "selected_source_seq_abs": pred.source_seq.zip(pred.source_abs).map(|(s,a)| [s,a])}));
                emitted.push(pred.emitted);
                tokens.push(pred.emitted);
            }
            generation.push(
                json!({"panel": panel, "arm": arm, "prefix": prefix, "answer": answer,
            "emitted": emitted, "first_correct": emitted.first() == Some(answer),
            "decoded": tokenizer.decode(&emitted), "steps": steps,
            "scope": "short rollout, no learned termination or complete-response qualification"}),
            );
        }
    }
    // Persist the same actual prediction events consumed by the metric loops above.
    let mut rows_text = String::new();
    for event in &prediction_events {
        rows_text.push_str(&serde_json::to_string(event).map_err(|e| e.to_string())?);
        rows_text.push('\n');
    }
    write_checked(&root, "rows.jsonl", rows_text.as_bytes())?;

    // Decoder-only hits and factual, operand-valid result-state overlap. Neither
    // overlap nor an operand-valid result implies a grounded decoder or correct source.
    let h4_dev_only = model_hits(&b_h4, &b_dev_ex);
    let c120_dev_only = model_hits(&b_c120, &b_dev_ex);
    let h4_held_only = model_hits(&b_h4, &b_test_ex);
    let c120_held_only = model_hits(&b_c120, &b_test_ex);
    let b_dev_states: std::collections::BTreeSet<usize> = b_dev_ex
        .iter()
        .filter(|e| e.read)
        .filter_map(|e| b_h4.checked_relative_result(e.r, e.qrole, e.payload).ok())
        .collect();
    let b_test_states: std::collections::BTreeSet<usize> = b_test_ex
        .iter()
        .filter(|e| e.read)
        .filter_map(|e| b_h4.checked_relative_result(e.r, e.qrole, e.payload).ok())
        .collect();
    let b_test_shared = b_test_states.intersection(&b_dev_states).count();
    let b_dev_cell_count = comp_dev
        .iter()
        .map(|it| (it.op, it.vi))
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let b_test_cell_count = comp_test
        .iter()
        .map(|it| (it.op, it.vi))
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    let b_hits = |label: &str, split: &str| -> u64 {
        b_arms
            .iter()
            .find(|c| c["arm"] == json!(label) && c["split"] == json!(split))
            .and_then(|c| c["hits"].as_u64())
            .unwrap_or(0)
    };
    let h4_test = b_hits("h4_derived_state", "held_out_cells");
    let control_max = [
        "local_noread",
        "c120_derived_state",
        "payload_only_table",
        "operation_and_payload_table",
        "development_constant",
    ]
    .iter()
    .map(|n| b_hits(n, "held_out_cells"))
    .max()
    .unwrap_or(0);
    let noncompositional_max = [
        "local_noread",
        "payload_only_table",
        "operation_and_payload_table",
        "development_constant",
    ]
    .iter()
    .map(|n| b_hits(n, "held_out_cells"))
    .max()
    .unwrap_or(0);
    let screen = json!({
        "comparison_scope": "exposed cyclic-fixture comparisons; computation competence and unique algebra advantage are separate",
        "h4_held_out_hits": h4_test,
        "held_out_positions": comp_test.len(),
        "best_control_held_out_hits": control_max,
        "historical_exceeds_all_controls": h4_test > control_max,
        "exceeds_noncompositional_controls": h4_test > noncompositional_max,
        "cyclic_c120_hits": b_hits("c120_derived_state", "held_out_cells"),
        "met": false, "competence_qualification": "NOT_RUN",
        "required_qualification": "prospectively selected familiar-primitive transfer criterion plus correct source/operation causal pairs and complete generated output; changed tokens or score superiority alone are insufficient",
    });

    let result = json!({
        "schema": "uor-r4.derived-state-decoder/2",
        "parent_review_revision": "fdc1607f",
        "running_source": {
            "source_checkout_observed_before_fit": source_checkout,
            "executable_sha256": executable_sha256,
            "source_files": source_files,
        },
        "inputs": {"E_sha256": E_SHA, "S_sha256": S_SHA, "tokenizer_derived": DERIVED_SHA,
            "group_digest": group_digest, "selector_sha256": selector_sha256, "f_bits": parent.cfg.f_bits},
        "mechanism": {
            "relative_result": "s = inverse(q0) * ((q0 * T_bind[r]) * U_op[observed_query_role] * V[payload]) = T_bind[r] * U_op[query] * V[value]",
            "operation_input": "the observed query role token of the actual causal prefix; the fixture's hidden operation index is never read",
            "decoder": "bounded learned per-state token shortlist; NoRead preserves the frozen local prior and is distinct from a valid identity result",
            "state_bound": DEC_MAX_STATES, "shortlist_k": DEC_K,
        },
        "association_panel": {"fit": dsd_fit_json(&a_fit), "value_only_fit": dsd_fit_json(&a_vo_fit),
            "arms": a_arms, "artifacts": a_artifacts},
        "composition_panel": {
            "fixture_version": 2, "seed": RC2_SEED, "ops": n_ops, "class_modulus": k,
            "binding_relation_budget": binding_budget, "witness_element": witness,
            "dev_positions": comp_dev.len(), "held_out_positions": comp_test.len(),
            "dev_cells": b_dev_cell_count, "held_out_cells": b_test_cell_count,
            "cell_count_scope": "distinct operation-value pairs; positions include repeated contexts per cell",
            "fit_h4": dsd_fit_json(&b_h4_fit), "fit_c120": dsd_fit_json(&b_c120_fit),
            "development_probe_split": {
                "role": "supervised outer development objective; labels queried throughout map search, not held-out generalization",
                "all_outer_operations_in_model_domain": b_probe_ex.iter().all(|e| b_h4.op_domain.contains(&e.qrole)),
                "fit_examples": b_fit_ex.len(), "probe_examples": b_probe_ex.len(),
                "fit_cells": b_cells.iter().zip(b_mask.iter()).filter(|(_, m)| **m).map(|(c, _)| *c).collect::<Vec<_>>(),
                "probe_cells": b_cells.iter().zip(b_mask.iter()).filter(|(_, m)| !**m).map(|(c, _)| *c).collect::<Vec<_>>(),
                "h4_probe_hits": model_hits(&b_h4, &b_probe_ex).0,
                "c120_probe_hits": model_hits(&b_c120, &b_probe_ex).0,
            },
            "arms": b_arms, "artifacts": b_artifacts,
            "decoder_only_hits": {
                "dev": {"h4": h4_dev_only.0, "total": h4_dev_only.1, "c120": c120_dev_only.0},
                "held_out": {"h4": h4_held_only.0, "total": h4_held_only.1, "c120": c120_held_only.0},
            },
            "states": {"scope": "actual reads with valid active operands only; result-state overlap does not imply decoder support or correct source selection",
                "dev_distinct": b_dev_states.len(), "held_out_distinct": b_test_states.len(),
                "held_out_states_shared_with_dev": b_test_shared,
                "h4_grounded_states": b_h4.decoder.state_count(), "c120_grounded_states": b_c120.decoder.state_count()},
            "interventions": interventions, "generated": generation,
        },
        "screen": screen,
        "scope": "bounded authored instruments, exposed association regression seeds; the composition rule was constructed to be realisable by the served algebra, so a pass is a composition-circuit result, not unique geometric advantage or language capability. Energy UNAVAILABLE; whole-path D0-b not claimed.",
        "elapsed_s": started.elapsed().as_secs_f64(),
    });
    write_json(&root, "result.json", &result)?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    let a_final = a_arms.last().cloned().unwrap_or(json!(null));
    println!(
        "derived-state: association decoder dev {} tune {} final {} (dict final {}) | composition dev {} held-out {} best control {} | sealed {} unlisted | {:.1}s",
        a_arms[0]["decoder_hits"],
        a_arms[1]["decoder_hits"],
        a_final["decoder_hits"],
        a_final["selected_value_dictionary_hits"],
        b_hits("h4_derived_state", "dev_cells"),
        h4_test,
        control_max,
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Shared-transition continuation: read -> compute -> emit -> stop
// ---------------------------------------------------------------------------

const ST_VALUES: usize = 4;
const ST_PRIMITIVES: usize = 8;
const ST_SEED: u64 = 0xC0F3_0001;
const ST_PASSES: usize = 3;
const ST_RESTARTS: usize = 6;
const ST_DEV_LEN2: usize = 32;
const ST_DEV_LEN3: usize = 32;
const ST_HELD_LEN4: usize = 16;
const ST_HELD_REV: usize = 16;

#[derive(Clone)]
struct StItem {
    item_id: usize,
    instruction_start: usize,
    prefix: Vec<u32>,
    primitives: Vec<u32>,
    targets: Vec<u32>,
    value: u32,
    split: &'static str,
}

struct StFixture {
    pub items: Vec<StItem>,
    pub stop_token: u32,
    pub primitives: Vec<u32>,
    pub q8: [u8; 8],
    pub noncommuting_pair: (usize, usize),
}

fn st_primitive_seqs(lens: &[usize], stride: usize, seed: u64) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut st = seed | 1;
    for &l in lens.iter() {
        let total = ST_PRIMITIVES.pow(l as u32);
        let mut i = (xorshift(&mut st) as usize) % stride.max(1);
        while i < total {
            let mut seq = Vec::with_capacity(l);
            let mut x = i;
            for _ in 0..l {
                seq.push(x % ST_PRIMITIVES);
                x /= ST_PRIMITIVES;
            }
            out.push(seq);
            i += stride.max(1);
        }
    }
    out
}

fn st_fixture(
    banks: &Banks,
    vocab: usize,
    labels: &[u32],
    q8: [u8; 8],
) -> Result<StFixture, String> {
    let t = group_table();
    let mul = |a: usize, b: usize| t.product[a * ROW_STRIDE + b] as usize;
    let idx_of = |g: usize| q8.iter().position(|x| *x as usize == g).unwrap_or(0);
    // True data codes live only in the generator; the model never sees them.
    let vcode = |i: usize| q8[(i * 3) % 8] as usize;
    let pcode = |j: usize| q8[(j * 5 + 1) % 8] as usize;
    let values: Vec<u32> = banks.values_fit.iter().copied().take(ST_VALUES).collect();
    let primitives: Vec<u32> = banks
        .values_fit
        .iter()
        .copied()
        .skip(ST_VALUES)
        .take(ST_PRIMITIVES)
        .collect();
    if values.len() < ST_VALUES || primitives.len() < ST_PRIMITIVES {
        return Err("insufficient bank for the shared-transition fixture".into());
    }
    let key = banks.keys[0];
    let qrole = banks.pairs[0].0;
    let source_role = banks.pairs[0].1;
    let mut stop_token = 0u32;
    for tk in (0..vocab as u32).rev() {
        if !values.contains(&tk) && !primitives.contains(&tk) && !labels.contains(&tk) {
            stop_token = tk;
            break;
        }
    }
    if stop_token == 0 {
        return Err("no disjoint stop token".into());
    }
    let mut noncommuting_pair = (0usize, 0usize);
    'outer: for a in 0..ST_PRIMITIVES {
        for b in 0..ST_PRIMITIVES {
            if mul(pcode(a), pcode(b)) != mul(pcode(b), pcode(a)) {
                noncommuting_pair = (a, b);
                break 'outer;
            }
        }
    }
    if noncommuting_pair == (0usize, 0usize) && mul(pcode(0), pcode(0)) == mul(pcode(0), pcode(0)) {
        // A single generator may still be noncommuting with itself only if it is not; check all.
        let any = (0..ST_PRIMITIVES).any(|a| {
            (0..ST_PRIMITIVES).any(|b| mul(pcode(a), pcode(b)) != mul(pcode(b), pcode(a)))
        });
        if !any {
            return Err("the declared primitives do not contain a noncommuting pair".into());
        }
    }
    let mut items: Vec<StItem> = Vec::new();
    let mut build =
        |seq: &[usize], value_idx: usize, split: &'static str, items: &mut Vec<StItem>| {
            let mut state = vcode(value_idx);
            let mut targets = Vec::with_capacity(seq.len());
            for &p in seq.iter() {
                state = mul(pcode(p), state);
                targets.push(labels[idx_of(state)]);
            }
            let mut prefix: Vec<u32> = vec![source_role, key, values[value_idx]];
            prefix.extend(seq.iter().map(|p| primitives[*p]));
            prefix.push(qrole);
            prefix.push(key);
            items.push(StItem {
                item_id: items.len(),
                instruction_start: 3,
                prefix,
                primitives: seq.iter().map(|p| primitives[*p]).collect(),
                targets,
                value: values[value_idx],
                split,
            });
        };
    for seq in st_primitive_seqs(&[1], 1, ST_SEED) {
        for v in 0..ST_VALUES {
            build(&seq, v, "dev", &mut items);
        }
    }
    for seq in st_primitive_seqs(&[2], 2, ST_SEED ^ 0x11) {
        for v in 0..ST_VALUES {
            build(&seq, v, "dev", &mut items);
        }
    }
    for seq in st_primitive_seqs(&[3], 16, ST_SEED ^ 0x22) {
        for v in 0..ST_VALUES {
            build(&seq, v, "dev", &mut items);
        }
    }
    for seq in st_primitive_seqs(&[4], 128, ST_SEED ^ 0x33) {
        for v in 0..ST_VALUES {
            build(&seq, v, "held_out_length4", &mut items);
        }
    }
    // Order-reversal held-out population: reversed development pairs that do **not** occur in
    // development at any length, so the reversal is genuinely unseen.
    let dev_pairs = st_primitive_seqs(&[2], 2, ST_SEED ^ 0x11);
    let dev_set: std::collections::BTreeSet<Vec<usize>> = {
        let mut set = std::collections::BTreeSet::new();
        for seq in st_primitive_seqs(&[1], 1, ST_SEED) {
            set.insert(seq);
        }
        for seq in dev_pairs.iter() {
            set.insert(seq.clone());
        }
        for seq in st_primitive_seqs(&[3], 16, ST_SEED ^ 0x22) {
            set.insert(seq);
        }
        set
    };
    let mut rev_built = 0usize;
    for seq in dev_pairs.iter() {
        if rev_built >= ST_HELD_REV {
            break;
        }
        let rev: Vec<usize> = seq.iter().rev().copied().collect();
        if dev_set.contains(&rev) {
            continue;
        }
        for v in 0..ST_VALUES {
            build(&rev, v, "held_out_reversal", &mut items);
        }
        rev_built += 1;
    }
    let _ = (ST_DEV_LEN2, ST_DEV_LEN3, ST_HELD_LEN4);
    Ok(StFixture {
        items,
        stop_token,
        primitives,
        q8,
        noncommuting_pair,
    })
}

/// One served response produced from the shared read, with the selected occurrence recorded.
struct StServed {
    response: Response,
    payload: Option<u32>,
    read: bool,
    source_abs: Option<u32>,
    source_seq: Option<u32>,
    payload_abs: Option<u32>,
    initial_state: Option<usize>,
    local_token: u32,
}

fn st_instructions(item: &StItem) -> Result<&[u32], String> {
    let end = item
        .prefix
        .len()
        .checked_sub(2)
        .ok_or("missing query boundary")?;
    if item.prefix.len() < 3 || item.instruction_start > end {
        return Err("invalid declared instruction span".into());
    }
    Ok(&item.prefix[item.instruction_start..end])
}

#[allow(clippy::too_many_arguments)]
fn st_read_and_serve(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    sel: &RelationalSelector,
    model: &SharedTransitionModel,
    item: &StItem,
    use_reader: bool,
) -> Result<StServed, String> {
    let primitives = st_instructions(item)?;
    let ring = ring_before_current(&item.prefix);
    let i = item.prefix.len() - 1;
    let z = local_logits(
        parent,
        local,
        u,
        item.prefix[i],
        item.prefix[i - 1] as usize,
        item.prefix[i - 2],
        ring.written(),
    );
    let r = read_step(
        &ring,
        item.prefix[i],
        item.prefix[i - 1],
        item.prefix[i - 2],
        sel,
        table,
        use_reader,
        &z,
    );
    let local_token = argmax_low(&z) as u32;
    let response = match (use_reader, r.payload) {
        (true, Some(p)) => model.serve(p, primitives),
        _ => Response {
            tokens: Vec::new(),
            states: Vec::new(),
            steps: vec![StepKind::NoRead],
            stopped: false,
        },
    };
    Ok(StServed {
        response,
        payload: r.payload,
        read: r.action.is_some(),
        source_abs: r.source.map(|x| x.abs),
        source_seq: r.source.map(|x| x.seq),
        payload_abs: r.payload_abs,
        initial_state: r.payload.and_then(|p| model.initial_state(p).ok()),
        local_token,
    })
}

fn st_complete(item: &StItem, r: &Response) -> bool {
    r.stopped && r.tokens == item.targets
}

fn st_tokens_response(tokens: Option<Vec<u32>>) -> Response {
    let Some(tokens) = tokens else {
        return Response {
            tokens: vec![],
            states: vec![],
            steps: vec![StepKind::NoRead],
            stopped: false,
        };
    };
    if tokens.is_empty() {
        return Response {
            tokens,
            states: vec![],
            steps: vec![StepKind::NoGrounding],
            stopped: false,
        };
    }
    let mut steps = vec![StepKind::Emit; tokens.len()];
    steps.push(StepKind::Stop);
    Response {
        tokens,
        states: vec![],
        steps,
        stopped: true,
    }
}

fn st_event(arm: &str, item: &StItem, served: &StServed) -> serde_json::Value {
    let events: Vec<_> = served.response.steps.iter().enumerate().map(|(j, kind)| {
        let computing = matches!(kind, StepKind::Emit | StepKind::NoGrounding);
        json!({"step": j, "kind": format!("{kind:?}"),
            "primitive": st_instructions(item).ok().and_then(|p| p.get(j)).copied(),
            "pre_state": if j == 0 { served.initial_state } else { served.response.states.get(j - 1).copied() },
            "post_state": if computing { served.response.states.get(j).copied() } else { None },
            "emitted": served.response.tokens.get(j), "expected": item.targets.get(j)})
    }).collect();
    json!({"arm": arm, "item_id": item.item_id, "split": item.split,
        "prefix": item.prefix, "instruction_start": item.instruction_start,
        "observed_primitives": st_instructions(item).ok(), "expected": item.targets,
        "tokens": served.response.tokens, "steps": events, "stopped": served.response.stopped,
        "read": served.read, "selected_payload": served.payload,
        "source_seq_abs": served.source_seq.zip(served.source_abs).map(|(s,a)| [s,a]),
        "payload_seq_abs": served.source_seq.zip(served.payload_abs).map(|(s,a)| [s,a]),
        "intended_payload": item.value, "intended_payload_abs": if item.instruction_start == 3 { Some(2u32) } else { None },
        "local_token": served.local_token,
        "complete": st_complete(item, &served.response),
        "state_scope": if matches!(arm, "h4_shared_transition" | "c120_shared_transition" | "grounded_factorization" | "fitted_shared_recurrence") { "actual served algebra states" } else { "not an algebra-state arm" }})
}

fn st_event_complete(event: &serde_json::Value) -> bool {
    match (event["tokens"].as_array(), event["expected"].as_array()) {
        (Some(tokens), Some(expected)) => event["stopped"] == json!(true) && tokens == expected,
        _ => false,
    }
}

fn st_score<F>(
    items: &[StItem],
    serve: &F,
    arm: &str,
    events: &mut Vec<serde_json::Value>,
) -> Result<(usize, usize, usize, usize), String>
where
    F: Fn(&StItem) -> Result<StServed, String>,
{
    let (mut complete, mut hits, mut total, mut stop_ok) = (0, 0, 0, 0);
    for item in items {
        let served = serve(item)?;
        let event = st_event(arm, item, &served);
        complete += usize::from(event["complete"] == json!(true));
        stop_ok += usize::from(
            served.response.stopped && served.response.tokens.len() == item.targets.len(),
        );
        for (j, expected) in item.targets.iter().enumerate() {
            total += 1;
            hits += usize::from(served.response.tokens.get(j) == Some(expected));
        }
        events.push(event);
    }
    Ok((complete, hits, total, stop_ok))
}

fn st_run() -> Result<ExitCode, String> {
    let mut root = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/shared-transition-principal-1",
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
            "crates/uor-r4-core/src/native_geometric/learner/result_decoder.rs",
            "crates/uor-r4-core/src/native_geometric/learner/shared_transition.rs",
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
    let source_checkout = ce_source_checkout(source_root.as_deref());
    let executable_sha256 = hex_of(&sha256_bytes(
        &std::env::current_exe()
            .ok()
            .and_then(|p| std::fs::read(p).ok())
            .unwrap_or_default(),
    ));
    let parent_root = PathBuf::from(PARENT_ROOT);
    let selector_bytes = std::fs::read(parent_root.join("artifacts/relational_ctx.rlr2"))
        .map_err(|e| format!("parent artifact: {e}"))?;
    let selector_sha256 = sha256_hex(&selector_bytes);
    if selector_sha256 != PARENT_SHA_RELATIONAL_CTX {
        return Err("contextual selector hash mismatch".into());
    }
    let sel =
        RelationalArtifact::from_bytes(&selector_bytes, &sha256_bytes(&sb), &raw_tok)?.selector;

    let q8 = q8_witness()?;
    let labels: Vec<u32> = (0..parent.cfg.vocab as u32).rev().take(64).collect();
    let fixture = st_fixture(&banks, parent.cfg.vocab, &labels, q8)?;
    let dev: Vec<StItem> = fixture
        .items
        .iter()
        .filter(|i| i.split == "dev")
        .cloned()
        .collect();
    let held_len: Vec<StItem> = fixture
        .items
        .iter()
        .filter(|i| i.split == "held_out_length4")
        .cloned()
        .collect();
    let held_rev: Vec<StItem> = fixture
        .items
        .iter()
        .filter(|i| i.split == "held_out_reversal")
        .cloned()
        .collect();
    let dev_examples: Vec<StExample> = dev
        .iter()
        .map(|i| StExample {
            payload: i.value,
            primitives: i.primitives.clone(),
            targets: i.targets.clone(),
        })
        .collect();

    // Value-disjoint supervised fitting: the fixture orders all four values within each
    // sequence, so item parity selects values 0/2 for calibration and 1/3 for the outer
    // map objective. Both sets contain every development sequence; neither is final data.
    let dev_fit: Vec<StExample> = dev_examples
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, e)| e.clone())
        .collect();
    let dev_probe: Vec<StExample> = dev_examples
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, e)| e.clone())
        .collect();
    let (h4, h4_fit) = fit_shared_transition(
        &dev_fit,
        &dev_probe,
        ST_MAX_STATES,
        ST_PASSES,
        false,
        ST_RESTARTS,
        ST_SEED,
    );
    let (c120, c120_fit) = fit_shared_transition(
        &dev_fit,
        &dev_probe,
        ST_MAX_STATES,
        ST_PASSES,
        true,
        ST_RESTARTS,
        ST_SEED,
    );

    // Export and independently reload before any reported response.
    let mut artifacts: Vec<serde_json::Value> = Vec::new();
    let mut reload =
        |name: &str, model: SharedTransitionModel| -> Result<SharedTransitionModel, String> {
            let bytes = model.to_bytes();
            let rel = format!("artifacts/{name}.rlst");
            write_checked(&root, &rel, &bytes)?;
            let loaded = SharedTransitionModel::from_bytes(
                &std::fs::read(root.join(&rel)).map_err(|e| e.to_string())?,
                parent.cfg.vocab,
            )?;
            if loaded != model {
                return Err("shared-transition reload mismatch".into());
            }
            let mut parity = 0usize;
            for it in fixture.items.iter() {
                let a = model.serve(it.value, &it.primitives);
                let b = loaded.serve(it.value, &it.primitives);
                if a != b {
                    return Err("shared-transition loaded parity failure".into());
                }
                parity += 1;
            }
            artifacts.push(json!({"arm": name, "artifact_sha256": sha256_hex(&bytes),
            "reload_identical": true, "loaded_parity_items": parity,
            "algebra": if model.cyclic {"cyclic_c120"} else {"signed_h4"},
            "selector_sha256": selector_sha256, "group_digest": group_digest}));
            Ok(loaded)
        };
    let h4 = reload("shared_transition_h4", h4)?;
    let c120 = reload("shared_transition_c120", c120)?;

    // Retained lexical component: the value-only derived-state decoder as a one-token emitter.
    let vo_fit: Vec<DecExample> = dev
        .iter()
        .filter_map(|i| {
            i.targets.first().map(|t| DecExample {
                r: 0,
                qrole: 0,
                payload: i.value,
                target: *t,
                read: true,
                grounded: true,
            })
        })
        .collect();
    let (vo_model, _vo_report) =
        fit_derived_state_model(&vo_fit, &[], DEC_MAX_STATES, ST_PASSES, false, true);
    let vo_bytes = vo_model.to_bytes();
    write_checked(&root, "artifacts/value_only_lexical.rlds", &vo_bytes)?;
    let vo_loaded = DerivedStateModel::from_bytes(
        &std::fs::read(root.join("artifacts/value_only_lexical.rlds"))
            .map_err(|e| e.to_string())?,
        parent.cfg.vocab,
    )?;
    if vo_loaded.value_only != true || vo_loaded != vo_model {
        return Err("value-only lexical reload or factor contract mismatch".into());
    }
    artifacts.push(
        json!({"arm": "value_only_lexical", "artifact_sha256": sha256_hex(&vo_bytes),
        "reload_identical": true, "value_only": vo_loaded.value_only,
        "strict_operands": vo_loaded.strict_operands}),
    );

    // Comparators.
    let mut dict: BTreeMap<(u32, Vec<u32>), Vec<u32>> = BTreeMap::new();
    let mut value_first: BTreeMap<u32, BTreeMap<u32, usize>> = BTreeMap::new();
    for i in dev.iter() {
        dict.insert((i.value, i.primitives.clone()), i.targets.clone());
        if let Some(t) = i.targets.first() {
            *value_first
                .entry(i.value)
                .or_default()
                .entry(*t)
                .or_default() += 1;
        }
    }
    let value_first: BTreeMap<u32, u32> = value_first
        .iter()
        .map(|(v, m)| {
            (
                *v,
                m.iter()
                    .max_by_key(|(_, c)| **c)
                    .map(|(t, _)| *t)
                    .unwrap_or(0),
            )
        })
        .collect();

    let finite = FiniteTransitionModel::fit(&dev_examples);
    let finite_bytes = finite.to_bytes()?;
    write_checked(
        &root,
        "artifacts/finite_shared_transition.json",
        &finite_bytes,
    )?;
    let finite_loaded = FiniteTransitionModel::from_bytes(
        &std::fs::read(root.join("artifacts/finite_shared_transition.json"))
            .map_err(|e| e.to_string())?,
        parent.cfg.vocab,
    )?;
    if finite != finite_loaded {
        return Err("finite transition reload mismatch".into());
    }
    artifacts.push(json!({"arm": "finite_shared_transition", "artifact_sha256": sha256_hex(&finite_bytes),
        "bytes": finite_bytes.len(), "table_sizes_initial_recurrent": finite_loaded.table_sizes(),
        "reload_identical": true, "fit_scope": "all declared development intermediate labels; shared initial and recurrent transitions"}));
    let mut prediction_events: Vec<serde_json::Value> = Vec::new();
    let mut arms: Vec<serde_json::Value> = Vec::new();
    for (split, items) in [
        ("dev", &dev),
        ("held_out_length4", &held_len),
        ("held_out_reversal", &held_rev),
    ] {
        let h4_score = st_score(
            items,
            &|it| st_read_and_serve(&parent, &local, &u, &table, &sel, &h4, it, true),
            "h4_shared_transition",
            &mut prediction_events,
        )?;
        let c120_score = st_score(
            items,
            &|it| st_read_and_serve(&parent, &local, &u, &table, &sel, &c120, it, true),
            "c120_shared_transition",
            &mut prediction_events,
        )?;
        let local_score = st_score(
            items,
            &|it| {
                let s = st_read_and_serve(&parent, &local, &u, &table, &sel, &h4, it, false)?;
                Ok(StServed {
                    response: st_tokens_response(Some(vec![s.local_token])),
                    initial_state: None,
                    ..s
                })
            },
            "local_noread",
            &mut prediction_events,
        )?;
        let dict_score = st_score(
            items,
            &|it| {
                let s = st_read_and_serve(&parent, &local, &u, &table, &sel, &h4, it, true)?;
                let instructions = st_instructions(it)?;
                let tokens = s.payload.map(|payload| {
                    dict.get(&(payload, instructions.to_vec()))
                        .cloned()
                        .unwrap_or_else(|| value_first.get(&payload).copied().into_iter().collect())
                });
                Ok(StServed {
                    response: st_tokens_response(tokens),
                    initial_state: None,
                    ..s
                })
            },
            "whole_sequence_dictionary",
            &mut prediction_events,
        )?;
        let vo_score = st_score(
            items,
            &|it| {
                let s = st_read_and_serve(&parent, &local, &u, &table, &sel, &h4, it, true)?;
                let tokens = s
                    .payload
                    .map(|payload| match vo_loaded.decode(0, 0, payload) {
                        DecOutcome::Emit { token, .. } => vec![token],
                        DecOutcome::NoRead => vec![],
                    });
                Ok(StServed {
                    response: st_tokens_response(tokens),
                    initial_state: None,
                    ..s
                })
            },
            "value_only_lexical",
            &mut prediction_events,
        )?;
        let finite_score = st_score(
            items,
            &|it| {
                let s = st_read_and_serve(&parent, &local, &u, &table, &sel, &h4, it, true)?;
                Ok(StServed {
                    response: finite_loaded.serve(s.payload, st_instructions(it)?),
                    initial_state: None,
                    ..s
                })
            },
            "finite_shared_transition",
            &mut prediction_events,
        )?;
        for (label, sc) in [
            ("h4_shared_transition", h4_score),
            ("c120_shared_transition", c120_score),
            ("local_noread", local_score),
            ("whole_sequence_dictionary", dict_score),
            ("value_only_lexical", vo_score),
            ("finite_shared_transition", finite_score),
        ] {
            arms.push(json!({"arm": label, "split": split, "items": items.len(),
                "complete": sc.0, "token_hits": sc.1, "token_total": sc.2, "stopped_correctly": sc.3}));
        }
    }

    // Interventions use the loaded candidate and actual causal read boundary. Fixture
    // expectations are computed separately and are never inputs to the predictor.
    let nc = fixture.noncommuting_pair;
    let expected_for = |payload: u32, primitives: &[u32]| -> Option<Vec<u32>> {
        let vi = banks
            .values_fit
            .iter()
            .take(ST_VALUES)
            .position(|v| *v == payload)?;
        let mut state = fixture.q8[(vi * 3) % 8] as usize;
        let mut out = Vec::new();
        for primitive in primitives {
            let pi = fixture.primitives.iter().position(|p| p == primitive)?;
            let action = fixture.q8[(pi * 5 + 1) % 8] as usize;
            state = group_table().product[action * ROW_STRIDE + state] as usize;
            let si = fixture.q8.iter().position(|s| *s as usize == state)?;
            out.push(labels[si]);
        }
        Some(out)
    };
    let mut interventions = json!({});
    if let Some(base) = dev.iter().find(|i| i.primitives.len() == 2) {
        let actual = st_read_and_serve(&parent, &local, &u, &table, &sel, &h4, base, true)?;
        if let Some(other) = dev
            .iter()
            .find(|i| i.primitives == base.primitives && i.value != base.value)
        {
            let changed = st_read_and_serve(&parent, &local, &u, &table, &sel, &h4, other, true)?;
            interventions["payload_changed_same_query_and_primitives"] = json!({
                "scope": "actual loaded read and compute; only the relevant payload token differs",
                "a": st_event("h4_shared_transition", base, &actual),
                "b": st_event("h4_shared_transition", other, &changed),
                "changed": actual.response.tokens != changed.response.tokens,
                "both_complete_correct": st_complete(base, &actual.response) && st_complete(other, &changed.response)});
        }
        let disabled = st_read_and_serve(&parent, &local, &u, &table, &sel, &h4, base, false)?;
        let mut removed = base.clone();
        removed.prefix.drain(..3);
        removed.instruction_start = 0;
        let absent = st_read_and_serve(&parent, &local, &u, &table, &sel, &h4, &removed, true)?;
        interventions["read_disabled"] = st_event("h4_read_disabled", base, &disabled);
        interventions["source_removed"] = st_event("h4_source_removed", &removed, &absent);
        interventions["absence_scope"] = json!(
            "removal and read disabling are distinct; the component returns NoRead with no response and records the actual local token; it does not claim local-output parity or eviction"
        );
        interventions["finite_no_read"] = json!({"steps": finite_loaded.serve(None, st_instructions(base)?).steps.iter().map(|k| format!("{k:?}")).collect::<Vec<_>>()});
        let mut reversed = base.clone();
        reversed.primitives.reverse();
        let end = reversed.prefix.len() - 2;
        reversed.prefix[reversed.instruction_start..end].reverse();
        reversed.targets = expected_for(reversed.value, st_instructions(&reversed)?)
            .ok_or("reversal expectation missing")?;
        let reversed_actual =
            st_read_and_serve(&parent, &local, &u, &table, &sel, &h4, &reversed, true)?;
        interventions["order_changed_fixed_evidence_end_to_end"] = json!({
            "forward": st_event("h4_shared_transition", base, &actual),
            "reversed": st_event("h4_shared_transition", &reversed, &reversed_actual),
            "same_selected_source": actual.source_seq == reversed_actual.source_seq && actual.source_abs == reversed_actual.source_abs && actual.payload_abs == reversed_actual.payload_abs,
            "both_complete_correct": st_complete(base, &actual.response) && st_complete(&reversed, &reversed_actual.response)});
        if let Some(payload) = actual.payload {
            let mut perturbed = h4.clone();
            for state in &mut perturbed.decoder.states {
                for token in &mut state.tokens {
                    if let Some(i) = labels.iter().position(|t| t == token) {
                        *token = labels[(i + 1) % 8];
                    }
                }
            }
            let original = h4.serve(payload, st_instructions(base)?);
            let modified = perturbed.serve(payload, st_instructions(base)?);
            interventions["retained_state_under_decoder_label_permutation"] = json!({
                "scope": "controlled permutation of the loaded decoder labels; selected operand and action codes fixed",
                "selected_payload": payload, "original_tokens": original.tokens, "modified_tokens": modified.tokens,
                "original_states": original.states, "modified_states": modified.states,
                "emissions_changed": original.tokens != modified.tokens,
                "states_identical": original.states == modified.states});
            let mut witness = json!({"found": false});
            'search: for x in &fixture.primitives {
                for y in &fixture.primitives {
                    if x == y {
                        continue;
                    }
                    let fwd = h4.serve(payload, &[*x, *y]);
                    let rev = h4.serve(payload, &[*y, *x]);
                    if fwd.states.len() != 2
                        || rev.states.len() != 2
                        || fwd.states[1] == rev.states[1]
                    {
                        continue;
                    }
                    let cf = c120.serve(payload, &[*x, *y]);
                    let cr = c120.serve(payload, &[*y, *x]);
                    let ef = expected_for(payload, &[*x, *y]);
                    let er = expected_for(payload, &[*y, *x]);
                    let c_state = |ops: [u32; 2]| {
                        c120.initial_state(payload)
                            .and_then(|s| c120.apply(s, ops[0]))
                            .and_then(|s| c120.apply(s, ops[1]))
                            .ok()
                    };
                    witness = json!({"found":true,"scope":"searched latent-order witness on one actual selected operand; correctness reported separately",
                        "primitives":[x,y],"selected_payload":payload,
                        "h4_final_state_forward":fwd.states[1],"h4_final_state_reversed":rev.states[1],
                        "h4_tokens_forward":fwd.tokens,"h4_tokens_reversed":rev.tokens,
                        "expected_forward":ef,"expected_reversed":er,
                        "forward_complete_correct":ef.as_ref().map(|e| fwd.stopped && fwd.tokens == *e),
                        "reversed_complete_correct":er.as_ref().map(|e| rev.stopped && rev.tokens == *e),
                        "forward_first_correct":ef.as_ref().map(|e| fwd.tokens.first() == e.first()),
                        "reversed_first_correct":er.as_ref().map(|e| rev.tokens.first() == e.first()),
                        "forward_final_correct":ef.as_ref().map(|e| fwd.tokens.len() == e.len() && fwd.tokens.last() == e.last()),
                        "reversed_final_correct":er.as_ref().map(|e| rev.tokens.len() == e.len() && rev.tokens.last() == e.last()),
                        "c120_tokens_forward":cf.tokens,"c120_tokens_reversed":cr.tokens,
                        "c120_final_state_forward":c_state([*x,*y]),"c120_final_state_reversed":c_state([*y,*x]),
                        "note":"additive final states commute; intermediate output trajectories need not be identical"});
                    break 'search;
                }
            }
            interventions["noncommuting_final_state_witness"] = witness;
            let unknown = (0..parent.cfg.vocab as u32)
                .find(|t| !h4.action_domain.contains(t))
                .ok_or("no unknown primitive")?;
            interventions["unknown_primitive_is_typed"] = json!({"token":unknown,"steps":h4.serve(payload,&[unknown]).steps.iter().map(|k|format!("{k:?}")).collect::<Vec<_>>()});
            interventions["undefined_value_is_typed"] = json!({"steps":h4.serve(u32::MAX - 3,st_instructions(base)?).steps.iter().map(|k|format!("{k:?}")).collect::<Vec<_>>()});
            let lengths:Vec<_> = (1..=4).map(|n| {let response=h4.serve(payload,&fixture.primitives[..n]);json!({"input_length":n,"emitted":response.tokens,"stopped":response.stopped,"terminal_kind":response.steps.last().map(|k|format!("{k:?}"))})}).collect();
            interventions["input_exhaustion_policy"] = json!({"scope":"supervised table on observable remaining primitive count, with explicit Exhausted fallback; no learned general continuation", "lengths":lengths});
        }
    }

    // Complete generated responses (the whole trajectory, not one next-token score).
    let mut generation: Vec<serde_json::Value> = Vec::new();
    for it in held_len.iter().take(4) {
        let s = st_read_and_serve(&parent, &local, &u, &table, &sel, &h4, it, true)?;
        generation.push(json!({
            "split": it.split, "value": it.value, "primitives": it.primitives,
            "expected": it.targets, "emitted": s.response.tokens,
            "stopped": s.response.stopped, "states": s.response.states,
            "steps": s.response.steps.iter().map(|k| format!("{k:?}")).collect::<Vec<_>>(),
            "complete_correct": st_complete(it, &s.response),
        }));
    }

    // Persist actual all-arm item/step events from the metric passes, without another inference.
    let mut rows_text = String::new();
    for event in &prediction_events {
        rows_text.push_str(&serde_json::to_string(event).map_err(|e| e.to_string())?);
        rows_text.push('\n');
    }
    write_checked(&root, "rows.jsonl", rows_text.as_bytes())?;
    let saved_events: Vec<serde_json::Value> = std::fs::read_to_string(root.join("rows.jsonl"))
        .map_err(|e| e.to_string())?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let mut recount = Vec::new();
    for arm in &arms {
        let matching: Vec<_> = saved_events
            .iter()
            .filter(|e| e["arm"] == arm["arm"] && e["split"] == arm["split"])
            .collect();
        let count = matching.iter().filter(|e| st_event_complete(e)).count();
        if count as u64 != arm["complete"].as_u64().ok_or("missing complete metric")? {
            return Err("saved actual-event complete count mismatch".into());
        }
        recount.push(
            json!({"arm":arm["arm"],"split":arm["split"],"items":matching.len(),"complete":count}),
        );
    }

    let find = |arm: &str, split: &str| -> u64 {
        arms.iter()
            .find(|a| a["arm"] == json!(arm) && a["split"] == json!(split))
            .and_then(|a| a["complete"].as_u64())
            .unwrap_or(0)
    };
    let h4_len = find("h4_shared_transition", "held_out_length4");
    let control_max = [
        "c120_shared_transition",
        "local_noread",
        "whole_sequence_dictionary",
        "value_only_lexical",
        "finite_shared_transition",
    ]
    .iter()
    .map(|a| find(a, "held_out_length4"))
    .max()
    .unwrap_or(0);
    let screen = json!({
        "declared": "held-out unseen ordered combinations must be completed correctly and must exceed every matched control on the same items",
        "h4_held_out_length4_complete": h4_len,
        "held_out_length4_items": held_len.len(),
        "best_control_complete": control_max,
        "beats_controls": h4_len > control_max,
        "met": false, "competence_qualification": "NOT_RUN",
        "scope": "exposed regression with added shared finite transition comparator; learned general continuation and source-owned dependent computation not qualified",
    });

    let result = json!({
        "schema": "uor-r4.shared-transition/2",
        "parent_review_revision": "832c1c33",
        "running_source": {
            "source_checkout_observed_before_fit": source_checkout,
            "executable_sha256": executable_sha256,
            "source_files": source_files,
        },
        "inputs": {"E_sha256": E_SHA, "S_sha256": S_SHA, "tokenizer_derived": DERIVED_SHA,
            "group_digest": group_digest, "selector_sha256": selector_sha256, "f_bits": parent.cfg.f_bits},
        "mechanism": {
            "core": "s0 = E[selected payload]; s_j = A[observed primitive j] * s_{j-1}; token = D(s_j); Stop from supervised input-exhaustion table; absent policy returns Exhausted",
            "action_reuse": "one action code per observed primitive token, reused at every occurrence and every position",
            "state_retention": "one local result state is retained across primitives within serve; emitted tokens are never re-read. Persistent session/owned-source leases remain NOT_RUN",
            "typed_outcomes": "Emit, Stop, Exhausted, NoRead, UnknownValue, UnknownPrimitive, InvalidState, NoGrounding are distinct",
            "noncommuting_witness": [fixture.primitives[nc.0], fixture.primitives[nc.1]],
            "q8_subgroup": fixture.q8.to_vec(),
        },
        "fixture": {
            "seed": ST_SEED, "values": ST_VALUES, "primitives": ST_PRIMITIVES,
            "stop_token": fixture.stop_token,
            "dev_items": dev.len(), "held_out_length4_items": held_len.len(),
            "held_out_reversal_items": held_rev.len(),
            "layout": "[source_role, key, value, primitive.., query_role, key] then the response",
        },
        "learning": {"h4": dsd_fit_json_st(&h4_fit), "c120": dsd_fit_json_st(&c120_fit),
            "development_probe": {"calibration_examples": dev_fit.len(), "probe_examples": dev_probe.len(),
                "note": "value-disjoint supervised fitting: values0/2 calibrate, values1/3 guide map search; every development sequence occurs in both, and the final decoder fits all development"}},
        "arms": arms,
        "interventions": interventions,
        "generated": generation,
        "recount": {"arms": recount, "item_rows": saved_events.len(), "step_events": saved_events.iter().filter_map(|e| e["steps"].as_array()).map(|s| s.len()).sum::<usize>(), "all_arm_counts_match": true},
        "screen": screen,
        "artifacts": artifacts,
        "scope": "bounded authored ordered-primitive fixture with a witness non-abelian action subgroup. An authored finite circuit is a circuit result, not language capability or unique geometric advantage. Energy UNAVAILABLE; whole-path D0-b not claimed.",
        "elapsed_s": started.elapsed().as_secs_f64(),
    });
    write_json(&root, "result.json", &result)?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "shared-transition: dev {} | length4 {} | reversal {} | H4 complete dev {} len4 {} rev {} | best control len4 {} | sealed {} unlisted | {:.1}s",
        dev.len(),
        held_len.len(),
        held_rev.len(),
        find("h4_shared_transition", "dev"),
        h4_len,
        find("h4_shared_transition", "held_out_reversal"),
        control_max,
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

fn dsd_fit_json_st(f: &StFitReport) -> serde_json::Value {
    json!({
        "initial_complete": f.initial_complete,
        "final_complete": f.final_complete,
        "final_tokens": f.final_tokens,
        "token_total": f.token_total,
        "examples": f.examples,
        "grounded_states": f.states,
        "historical_restart_tiebreak_pre_refit_states": f.pre_refit_states,
        "accepted_moves": f.accepted_moves,
    })
}

// ---------------------------------------------------------------------------
// Grounded computation in an owned dependent session
// ---------------------------------------------------------------------------

const GS_SEED: u64 = 0xC0F4_0001;

/// Serve the shared-transition fixture through an arbitrary response function, sharing one read.
fn gs_serve_with<F>(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    table: &ExactGroupTable,
    sel: &RelationalSelector,
    item: &StItem,
    serve: &F,
    initial_of: &dyn Fn(u32) -> Option<usize>,
    use_reader: bool,
) -> Result<StServed, String>
where
    F: Fn(Option<u32>, &[u32]) -> Response,
{
    let primitives = st_instructions(item)?;
    let ring = ring_before_current(&item.prefix);
    let i = item.prefix.len() - 1;
    let z = local_logits(
        parent,
        local,
        u,
        item.prefix[i],
        item.prefix[i - 1] as usize,
        item.prefix[i - 2],
        ring.written(),
    );
    let r = read_step(
        &ring,
        item.prefix[i],
        item.prefix[i - 1],
        item.prefix[i - 2],
        sel,
        table,
        use_reader,
        &z,
    );
    let local_token = argmax_low(&z) as u32;
    let response = if use_reader {
        serve(r.payload, primitives)
    } else {
        Response {
            tokens: Vec::new(),
            states: Vec::new(),
            steps: vec![StepKind::NoRead],
            stopped: false,
        }
    };
    Ok(StServed {
        response,
        payload: r.payload,
        read: r.action.is_some(),
        source_abs: r.source.map(|x| x.abs),
        source_seq: r.source.map(|x| x.seq),
        payload_abs: r.payload_abs,
        initial_state: r.payload.and_then(|p| initial_of(p)),
        local_token,
    })
}

// ---- dependent two-hop relation chain -------------------------------------

#[derive(Clone)]
struct GsItem {
    /// Evidence tokens (records) followed by the observed request `[role_q1, key1, p1, p2]`.
    prefix: Vec<u32>,
    evidence: Vec<u32>,
    key1: u32,
    prime: [u32; 2],
    split: &'static str,
    id: usize,
}

struct GsRead {
    payload: Option<u32>,
    read: bool,
    source_abs: Option<u32>,
    states: Vec<usize>,
    emits: Vec<u32>,
    steps: Vec<Terminal>,
}

fn gs_run() -> Result<ExitCode, String> {
    let mut root = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/grounded-session-principal-1",
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
            "crates/uor-r4-core/src/native_geometric/learner/result_decoder.rs",
            "crates/uor-r4-core/src/native_geometric/learner/shared_transition.rs",
            "crates/uor-r4-core/src/native_geometric/learner/grounded_session.rs",
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
    let source_checkout = ce_source_checkout(source_root.as_deref());
    let executable_sha256 = hex_of(&sha256_bytes(
        &std::env::current_exe()
            .ok()
            .and_then(|p| std::fs::read(p).ok())
            .unwrap_or_default(),
    ));
    let parent_root = PathBuf::from(PARENT_ROOT);
    let selector_bytes = std::fs::read(parent_root.join("artifacts/relational_ctx.rlr2"))
        .map_err(|e| format!("parent artifact: {e}"))?;
    let selector_sha256 = sha256_hex(&selector_bytes);
    if selector_sha256 != PARENT_SHA_RELATIONAL_CTX {
        return Err("contextual selector hash mismatch".into());
    }
    let sel =
        RelationalArtifact::from_bytes(&selector_bytes, &sha256_bytes(&sb), &raw_tok)?.selector;

    // ---------------- Panel 1: grounded factorization vs the finite-state control -------------
    let q8 = q8_witness()?;
    let labels: Vec<u32> = (0..parent.cfg.vocab as u32).rev().take(64).collect();
    let fixture = st_fixture(&banks, parent.cfg.vocab, &labels, q8)?;
    let dev: Vec<StItem> = fixture
        .items
        .iter()
        .filter(|i| i.split == "dev")
        .cloned()
        .collect();
    let held_len: Vec<StItem> = fixture
        .items
        .iter()
        .filter(|i| i.split == "held_out_length4")
        .cloned()
        .collect();
    let held_rev: Vec<StItem> = fixture
        .items
        .iter()
        .filter(|i| i.split == "held_out_reversal")
        .cloned()
        .collect();
    let dev_examples: Vec<StExample> = dev
        .iter()
        .map(|i| StExample {
            payload: i.value,
            primitives: i.primitives.clone(),
            targets: i.targets.clone(),
        })
        .collect();
    let (gf, gf_report) = factor_observed_graph(&dev_examples, false)?;
    let finite = FiniteTransitionModel::fit(&dev_examples);
    let (fitted, fitted_report) = fit_shared_transition(
        &dev_examples,
        &[],
        ST_MAX_STATES,
        ST_PASSES,
        false,
        2,
        GS_SEED,
    );
    let gf_bytes = gf.to_bytes();
    write_checked(&root, "artifacts/grounded_factorization.rlgf", &gf_bytes)?;
    let gf_loaded = GroundedFactorization::from_bytes(
        &std::fs::read(root.join("artifacts/grounded_factorization.rlgf"))
            .map_err(|e| e.to_string())?,
        parent.cfg.vocab,
    )?;
    if gf_loaded != gf {
        return Err("grounded-factorization reload mismatch".into());
    }
    let finite_bytes = finite.to_bytes().map_err(|e| e)?;
    write_checked(&root, "artifacts/finite_transition.json", &finite_bytes)?;
    let finite_loaded = FiniteTransitionModel::from_bytes(
        &std::fs::read(root.join("artifacts/finite_transition.json")).map_err(|e| e.to_string())?,
        parent.cfg.vocab,
    )?;
    if finite_loaded != finite {
        return Err("finite transition reload mismatch".into());
    }
    let fitted_bytes = fitted.to_bytes();
    write_checked(&root, "artifacts/fitted_recurrence.rlst", &fitted_bytes)?;
    let fitted_loaded = SharedTransitionModel::from_bytes(
        &std::fs::read(root.join("artifacts/fitted_recurrence.rlst")).map_err(|e| e.to_string())?,
        parent.cfg.vocab,
    )?;
    if fitted_loaded != fitted {
        return Err("fitted recurrence reload mismatch".into());
    }
    let gf = gf_loaded;
    let finite = finite_loaded;
    let fitted = fitted_loaded;
    let gf_sha = sha256_hex(&gf_bytes);
    let loaded_artifacts = json!([
        {"arm":"grounded_factorization","sha256":gf_sha,"serialized_bytes":gf_bytes.len(),"parameter_bytes_estimate":gf.parameter_bytes(),"actual_loaded_object_used":true},
        {"arm":"finite_transition_control","sha256":sha256_hex(&finite_bytes),"serialized_bytes":finite_bytes.len(),"actual_loaded_object_used":true},
        {"arm":"fitted_shared_recurrence","sha256":sha256_hex(&fitted_bytes),"serialized_bytes":fitted_bytes.len(),"actual_loaded_object_used":true,
            "fit_scope":"new all-development refit with two restarts and GS_SEED; not the retained previous-milestone artifact"}
    ]);
    let mut panel1: Vec<serde_json::Value> = Vec::new();
    let mut events: Vec<serde_json::Value> = Vec::new();
    for (split, items) in [
        ("dev", &dev),
        ("held_out_length4", &held_len),
        ("held_out_reversal", &held_rev),
    ] {
        let mut gf_events: Vec<serde_json::Value> = Vec::new();
        let gf_score = st_score(
            items,
            &|it| {
                gs_serve_with(
                    &parent,
                    &local,
                    &u,
                    &table,
                    &sel,
                    it,
                    &|p, pr: &[u32]| gf.serve(p, pr),
                    &|p| gf.initial_state(p),
                    true,
                )
            },
            "grounded_factorization",
            &mut gf_events,
        )?;
        events.extend(gf_events);
        let mut fin_events: Vec<serde_json::Value> = Vec::new();
        let finite_score = st_score(
            items,
            &|it| {
                gs_serve_with(
                    &parent,
                    &local,
                    &u,
                    &table,
                    &sel,
                    it,
                    &|p, pr: &[u32]| finite.serve(p, pr),
                    &|_| None,
                    true,
                )
            },
            "finite_transition_control",
            &mut fin_events,
        )?;
        events.extend(fin_events);
        let mut fit_events: Vec<serde_json::Value> = Vec::new();
        let fitted_score = st_score(
            items,
            &|it| {
                gs_serve_with(
                    &parent,
                    &local,
                    &u,
                    &table,
                    &sel,
                    it,
                    &|p, pr: &[u32]| {
                        p.map(|payload| fitted.serve(payload, pr))
                            .unwrap_or(Response {
                                tokens: Vec::new(),
                                states: Vec::new(),
                                steps: vec![StepKind::NoRead],
                                stopped: false,
                            })
                    },
                    &|p| fitted.initial_state(p).ok(),
                    true,
                )
            },
            "fitted_shared_recurrence",
            &mut fit_events,
        )?;
        events.extend(fit_events);
        for (label, sc, params) in [
            ("grounded_factorization", gf_score, gf_bytes.len()),
            (
                "finite_transition_control",
                finite_score,
                finite_bytes.len(),
            ),
            (
                "fitted_shared_recurrence",
                fitted_score,
                fitted.to_bytes().len(),
            ),
        ] {
            panel1.push(json!({"arm": label, "split": split, "items": items.len(),
                "complete": sc.0, "token_hits": sc.1, "token_total": sc.2,
                "stopped_correctly": sc.3, "serialized_artifact_bytes": params}));
        }
    }

    // ---------------- Panel 2: dependent two-hop relation chain -------------------------------
    // Evidence is a set of `(role, key, value)` records. The first key is a request key; the second
    // hop's keys are the *typed outcome labels*, so the first computed result forms the second
    // query. The final answer is a label token, absent from every record payload.
    let g_values: Vec<u32> = banks.values_fit.iter().copied().take(ST_VALUES).collect();
    let hop1_keys: Vec<u32> = banks.keys.iter().copied().take(ST_VALUES).collect();
    let outcomes: Vec<u32> = gf.outcome_domain.clone();
    let role_rec = banks.pairs[10].1;
    let role_q = banks.pairs[10].0;
    let p1 = fixture.primitives[0];
    let p2 = fixture.primitives[1];
    let build_evidence = |keys: &[u32], vals: &[u32], drop_first: bool| -> Vec<u32> {
        let mut v = Vec::new();
        for (j, k) in keys.iter().enumerate() {
            if drop_first && j == 0 {
                continue;
            }
            v.push(role_rec);
            v.push(*k);
            v.push(vals[j]);
        }
        for (m, y) in outcomes.iter().enumerate() {
            v.push(role_rec);
            v.push(*y);
            v.push(vals[m % vals.len()]);
        }
        v
    };
    // Checkpoints bind the existing RLSF payload to the actual loaded model and its
    // declared geometry/tokenizer. The fixed continuation location is supplied protocol.
    let checkpoint =
        |name: &str, stage: &str, frame: &SessionFrame| -> Result<SessionFrame, String> {
            frame.validate(&gf)?;
            let bytes = frame.to_bytes();
            let rel = format!("frames/{name}.json");
            write_json(
                &root,
                &rel,
                &json!({"schema":"uor-r4.grounded-checkpoint/1","stage":stage,
            "model_sha256":gf_sha,"group_digest":group_digest,"tokenizer_sha256":DERIVED_SHA,
            "frame_sha256":sha256_hex(&bytes),"frame_bytes":bytes}),
            )?;
            let stored: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join(&rel)).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            if stored["stage"] != json!(stage)
                || stored["model_sha256"] != json!(gf_sha)
                || stored["group_digest"] != json!(group_digest)
                || stored["tokenizer_sha256"] != json!(DERIVED_SHA)
            {
                return Err("session checkpoint binding mismatch".into());
            }
            let bytes: Vec<u8> =
                serde_json::from_value(stored["frame_bytes"].clone()).map_err(|e| e.to_string())?;
            if stored["frame_sha256"] != json!(sha256_hex(&bytes)) {
                return Err("session checkpoint byte digest mismatch".into());
            }
            let restored = SessionFrame::from_bytes(&bytes, parent.cfg.vocab)?;
            restored.validate(&gf)?;
            Ok(restored)
        };
    // One shared second-stage continuation consumes the restored computed outcome.
    // It does not receive a first source, first key or first primitive and cannot rerun hop1.
    let continue_from_first = |ring: &OccurrenceRing,
                               frame: &mut SessionFrame,
                               primitive: u32,
                               second_reader: bool,
                               second_update: bool,
                               pause_name: Option<&str>|
     -> Result<serde_json::Value, String> {
        frame.validate(&gf)?;
        let first_snapshot = frame.lease;
        let first_state = frame.state;
        let first = frame.outcome;
        let Some(query) = first else {
            if frame.terminal.is_none() {
                frame.stop(Terminal::NoGrounding);
            }
            return Ok(
                json!({"first":first,"second_query":null,"second_payload":null,"answer":null,
                "terminal":format!("{:?}",frame.terminal),"second_read":false,
                "first_snapshot":first_snapshot.map(|l|[l.seq,l.abs,l.payload])}),
            );
        };
        let z = local_logits(
            &parent,
            &local,
            &u,
            query,
            role_q as usize,
            NO_TOKEN,
            ring.written(),
        );
        let read = read_step(
            ring,
            query,
            role_q,
            NO_TOKEN,
            &sel,
            &table,
            second_reader,
            &z,
        );
        let mut second_snapshot = None;
        let mut answer = None;
        if let Some(payload) = read.payload {
            let abs = read
                .payload_abs
                .ok_or("second read lacks an exact payload reference")?;
            let snapshot = SourceLease::acquire(ring, abs).map_err(|t| format!("{t:?}"))?;
            if snapshot.payload != payload {
                return Err("second snapshot disagrees with selected payload".into());
            }
            second_snapshot = Some(snapshot);
            if second_update {
                frame.read(snapshot, &gf).map_err(|t| format!("{t:?}"))?;
            }
            frame
                .apply_primitive(primitive, &gf)
                .map_err(|t| format!("{t:?}"))?;
            if let Some(name) = pause_name {
                *frame = checkpoint(
                    &format!("{name}-after-second"),
                    "after_second_compute_before_emit",
                    frame,
                )?;
            }
            answer = Some(frame.emit().map_err(|t| format!("{t:?}"))?);
            frame.stop(Terminal::Exhausted);
        } else {
            frame.stop(Terminal::NoRead);
        }
        Ok(
            json!({"first":first,"first_state":first_state,"second_query":query,
            "second_payload":read.payload,"answer":answer,"terminal":format!("{:?}",frame.terminal),
            "second_read":read.action.is_some(),"second_source_abs":read.payload_abs,
            "second_key_seq_abs":read.source.map(|r|[r.seq,r.abs]),
            "first_snapshot":first_snapshot.map(|l|[l.seq,l.abs,l.payload]),
            "second_snapshot":second_snapshot.map(|l|[l.seq,l.abs,l.payload]),
            "active_snapshot":frame.lease.map(|l|[l.seq,l.abs,l.payload]),
            "second_admitted":read.admitted,"second_scanned":read.scanned,
            "second_update_enabled":second_update,"final_state":frame.state,
            "emitted":frame.emitted,"phase":frame.phase}),
        )
    };
    let chain = |evidence: &[u32],
                 key1: u32,
                 primitive1: u32,
                 primitive2: u32,
                 reader: bool,
                 first_update: bool,
                 second_reader: bool,
                 second_update: bool,
                 pause_name: Option<&str>|
     -> Result<serde_json::Value, String> {
        let mut ring = OccurrenceRing::new(RING_CAP);
        for token in evidence {
            ring.observe(*token);
        }
        let z = local_logits(
            &parent,
            &local,
            &u,
            key1,
            role_q as usize,
            NO_TOKEN,
            ring.written(),
        );
        let read = read_step(&ring, key1, role_q, NO_TOKEN, &sel, &table, reader, &z);
        let mut frame = SessionFrame::start();
        let mut first_snapshot = None;
        if let Some(payload) = read.payload {
            let abs = read
                .payload_abs
                .ok_or("first read lacks an exact payload reference")?;
            let snapshot = SourceLease::acquire(&ring, abs).map_err(|t| format!("{t:?}"))?;
            if snapshot.payload != payload {
                return Err("first snapshot disagrees with selected payload".into());
            }
            first_snapshot = Some(snapshot);
            frame.read(snapshot, &gf).map_err(|t| format!("{t:?}"))?;
            if first_update {
                frame
                    .apply_primitive(primitive1, &gf)
                    .map_err(|t| format!("{t:?}"))?;
            }
        } else {
            frame.stop(Terminal::NoRead);
        }
        let after_first = frame.clone();
        let mut observed = if frame.terminal.is_none() {
            continue_from_first(
                &ring,
                &mut frame,
                primitive2,
                second_reader,
                second_update,
                None,
            )?
        } else {
            json!({"first":null,"second_query":null,"second_payload":null,"answer":null,
            "terminal":format!("{:?}",frame.terminal),"second_read":false,"second_source_abs":null,
            "first_snapshot":null,"second_snapshot":null})
        };
        let mut resumed_identical = None;
        if let Some(name) = pause_name {
            if after_first.terminal.is_none() {
                let mut restored = checkpoint(
                    &format!("{name}-after-first"),
                    "after_first_compute_before_second_read",
                    &after_first,
                )?;
                let resumed = continue_from_first(
                    &ring,
                    &mut restored,
                    primitive2,
                    second_reader,
                    second_update,
                    Some(name),
                )?;
                resumed_identical = Some(resumed == observed && restored == frame);
                if resumed_identical != Some(true) {
                    return Err(
                        "restored continuation differs from uninterrupted continuation".into(),
                    );
                }
            }
        }
        observed["first_read"] = json!(read.action.is_some());
        observed["first_source_abs"] = json!(read.payload_abs);
        observed["first_key_seq_abs"] = json!(read.source.map(|r| [r.seq, r.abs]));
        observed["first_snapshot"] = json!(first_snapshot.map(|l| [l.seq, l.abs, l.payload]));
        observed["first_admitted"] = json!(read.admitted);
        observed["first_scanned"] = json!(read.scanned);
        Ok(
            json!({"evidence":evidence,"request":{"key":key1,"primitives":[primitive1,primitive2]},
            "observed":observed,"resumed_identical":resumed_identical,
            "checkpoint_prefix":pause_name,
            "scope":"supplied fixed two-hop protocol; restored continuation begins after first computation, never rereads hop1; completion is explicit program exhaustion"}),
        )
    };
    // Independent evaluator: use the declared first-transition labels, not candidate outputs.
    // Evidence is a supplied unique-key record layout; exact positions are evaluator metadata.
    let expected_step = |value: u32, primitive: u32| -> Result<u32, String> {
        let targets: std::collections::BTreeSet<u32> = dev_examples
            .iter()
            .filter(|e| e.payload == value && e.primitives.first() == Some(&primitive))
            .filter_map(|e| e.targets.first().copied())
            .collect();
        if targets.len() != 1 {
            return Err("independent transition expectation is missing or ambiguous".into());
        }
        targets
            .iter()
            .next()
            .copied()
            .ok_or("missing expected transition".into())
    };
    let expected_chain = |evidence: &[u32], key: u32| -> Result<serde_json::Value, String> {
        let lookup = |query: u32| -> Option<(u32, u32)> {
            evidence
                .chunks_exact(3)
                .enumerate()
                .find(|(_, r)| r[1] == query)
                .map(|(i, r)| ((3 * i + 2) as u32, r[2]))
        };
        let Some((first_abs, value)) = lookup(key) else {
            return Ok(
                json!({"first":null,"second_payload_abs":null,"second_payload":null,"answer":null}),
            );
        };
        let first = expected_step(value, p1)?;
        let Some((second_abs, value)) = lookup(first) else {
            return Ok(
                json!({"first":first,"first_payload_abs":first_abs,"second_payload_abs":null,"second_payload":null,"answer":null}),
            );
        };
        Ok(
            json!({"first":first,"first_payload_abs":first_abs,"second_payload_abs":second_abs,
            "second_payload":value,"answer":expected_step(value,p2)?}),
        )
    };
    let ev0 = build_evidence(&hop1_keys, &g_values, false);
    let mut chain_rows = Vec::new();
    let (mut first_ok, mut second_ok, mut answer_ok, mut resume_ok) = (0, 0, 0, 0);
    for (j, key) in hop1_keys.iter().enumerate() {
        let actual = chain(
            &ev0,
            *key,
            p1,
            p2,
            true,
            true,
            true,
            true,
            Some(&format!("chain-{j}")),
        )?;
        let expected = expected_chain(&ev0, *key)?;
        let observed = &actual["observed"];
        first_ok += usize::from(
            observed["first"] == expected["first"]
                && observed["first_source_abs"] == expected["first_payload_abs"],
        );
        second_ok += usize::from(
            observed["second_payload"] == expected["second_payload"]
                && observed["second_source_abs"] == expected["second_payload_abs"],
        );
        answer_ok += usize::from(observed["answer"] == expected["answer"]);
        resume_ok += usize::from(actual["resumed_identical"] == json!(true));
        chain_rows.push(json!({"key1":key,"expected":expected,"actual":actual}));
    }
    let base = chain(&ev0, hop1_keys[0], p1, p2, true, true, true, true, None)?;
    let mut ev_alt = ev0.clone();
    ev_alt[2] = g_values[1]; // Exactly the first source payload; every hop2 record stays fixed.
    let alt = chain(&ev_alt, hop1_keys[0], p1, p2, true, true, true, true, None)?;
    let expected_base = expected_chain(&ev0, hop1_keys[0])?;
    let expected_alt = expected_chain(&ev_alt, hop1_keys[0])?;
    let mut ev_removed = ev0.clone();
    ev_removed.drain(..3);
    let removed = chain(
        &ev_removed,
        hop1_keys[0],
        p1,
        p2,
        true,
        true,
        true,
        true,
        None,
    )?;
    let second_abs = expected_base["second_payload_abs"]
        .as_u64()
        .ok_or("missing expected second source")? as usize;
    let mut ev_second_removed = ev0.clone();
    ev_second_removed.drain(second_abs - 2..second_abs + 1);
    let second_removed = chain(
        &ev_second_removed,
        hop1_keys[0],
        p1,
        p2,
        true,
        true,
        true,
        true,
        None,
    )?;
    let mut ev_distractor = ev0.clone();
    ev_distractor.extend([
        role_rec,
        *banks.keys.last().ok_or("no distractor key")?,
        g_values[2],
    ]);
    let distractor = chain(
        &ev_distractor,
        hop1_keys[0],
        p1,
        p2,
        true,
        true,
        true,
        true,
        None,
    )?;
    let no_reader = chain(&ev0, hop1_keys[0], p1, p2, false, true, true, true, None)?;
    let no_second_reader = chain(&ev0, hop1_keys[0], p1, p2, true, true, false, true, None)?;
    let no_first_update = chain(&ev0, hop1_keys[0], p1, p2, true, false, true, true, None)?;
    let no_second_update = chain(&ev0, hop1_keys[0], p1, p2, true, true, true, false, None)?;
    let chain_interventions = json!({
        "changed_first_source": {"a":base,"b":alt,"expected_a":expected_base,"expected_b":expected_alt,
            "changed_positions":ev0.iter().zip(&ev_alt).enumerate().filter(|(_, (a,b))|a!=b).map(|(i,_)|i).collect::<Vec<_>>(),
            "first_changed":base["observed"]["first"]!=alt["observed"]["first"],
            "second_query_changed":base["observed"]["second_query"]!=alt["observed"]["second_query"],
            "second_selection_changed":base["observed"]["second_source_abs"]!=alt["observed"]["second_source_abs"],
            "answer_changed":base["observed"]["answer"]!=alt["observed"]["answer"],
            "both_answers_correct":base["observed"]["answer"]==expected_base["answer"] && alt["observed"]["answer"]==expected_alt["answer"]},
        "irrelevant_distractor_added":{"base":base,"changed":distractor,
            "answer_preserved":base["observed"]["answer"]==distractor["observed"]["answer"],
            "first_preserved":base["observed"]["first"]==distractor["observed"]["first"],
            "second_identity_preserved":base["observed"]["second_snapshot"]==distractor["observed"]["second_snapshot"]},
        "required_first_source_removed":removed,"required_second_source_removed":second_removed,
        "read_disabled":no_reader,"second_read_disabled":no_second_reader,
        "first_computation_disabled":no_first_update,"second_payload_update_disabled":no_second_update,
        "control_scope":"fixed supplied two-hop schedule; update controls change actual computation and do not train or qualify a learned controller"
    });

    let result = json!({
        "schema": "uor-r4.grounded-session/2",
        "parent_review_revision": "17d5072c",
        "running_source": {
            "source_checkout_observed_before_fit": source_checkout,
            "executable_sha256": executable_sha256,
            "source_files": source_files,
        },
        "inputs": {"E_sha256": E_SHA, "S_sha256": S_SHA, "tokenizer_derived": DERIVED_SHA,
            "group_digest": group_digest, "selector_sha256": selector_sha256, "f_bits": parent.cfg.f_bits},
        "factorization": gf_report,
        "loaded_artifacts": loaded_artifacts,
        "panel1_program_completion": panel1,
        "panel1_events": events,
        "panel2_dependent_chain": {
            "records": {"hop1": hop1_keys.len(), "hop2": outcomes.len(), "evidence_tokens": ev0.len()},
            "first_correct": first_ok, "second_selection_correct": second_ok,
            "answer_correct": answer_ok, "resumed_identical": resume_ok,
            "chains": hop1_keys.len(),
            "rows": chain_rows,
            "controls": chain_interventions,
            "note": "computed first outcome supplies second exact-key query; fixed two-hop schedule and record layout are supplied, not learned. Snapshots copy content and survive origin eviction; references are local to the owning ring. The final label is absent from record payloads but may occur as a record key.",
        },
        "fitted_recurrence_report": dsd_fit_json_st(&fitted_report),
        "scope": "bounded authored fixtures; a finite circuit result, not language capability. Energy UNAVAILABLE; whole-path D0-b not claimed.",
        "elapsed_s": started.elapsed().as_secs_f64(),
    });
    write_json(&root, "result.json", &result)?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    let find = |arm: &str, split: &str| -> u64 {
        panel1
            .iter()
            .find(|a| a["arm"] == json!(arm) && a["split"] == json!(split))
            .and_then(|a| a["complete"].as_u64())
            .unwrap_or(0)
    };
    println!(
        "grounded-session: gf dev {} len4 {} rev {} | finite dev {} len4 {} rev {} | iso candidates {} transitions {}/{} | sealed {} unlisted | {:.1}s",
        find("grounded_factorization", "dev"),
        find("grounded_factorization", "held_out_length4"),
        find("grounded_factorization", "held_out_reversal"),
        find("finite_transition_control", "dev"),
        find("finite_transition_control", "held_out_length4"),
        find("finite_transition_control", "held_out_reversal"),
        gf_report.isomorphic_candidates_checked,
        gf_report.transitions_consistent,
        gf_report.transitions_checked,
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Learned relational access and content-dependent session control
// ---------------------------------------------------------------------------

const REL_N_ENTITIES: usize = 8;
const REL_N_LITERALS: usize = 4;
const REL_N_RELATIONS: usize = 4;
const REL_DEV_WORLDS: u64 = 4;
const REL_FINAL_WORLDS: u64 = 4;
const REL_DEV_SEED: u64 = 0xC0F5_0001;
const REL_FINAL_SEED: u64 = 0xC0F5_0101;

/// A supplied memory world. Records are `(role, key, value)` token triples; the role token encodes
/// the relation, and the request names the relation, so the role/relation correspondence must be
/// learned rather than read off.
#[derive(Clone, serde::Serialize)]
struct RelWorld {
    records: Vec<(u32, u32, u32)>,
    entities: Vec<u32>,
    version: u32,
}

impl RelWorld {
    fn keys(&self) -> std::collections::BTreeSet<u32> {
        self.records.iter().map(|(_, k, _)| *k).collect()
    }
    fn admit(&self, key: u32) -> Vec<Candidate> {
        self.records
            .iter()
            .filter(|(_, k, _)| *k == key)
            .map(|(role, k, v)| Candidate {
                role: *role,
                key: *k,
                value: *v,
                exact_key: true,
            })
            .collect()
    }
    fn record_abs(&self, index: usize) -> u32 {
        index as u32
    }
}

fn rel_make_world(
    seed: u64,
    relations: &[u32],
    roles: &[u32],
    entities: &[u32],
    literals: &[u32],
    version: u32,
) -> RelWorld {
    let mut st = seed | 1;
    let mut records = Vec::new();
    for e in entities.iter() {
        for (r, role) in roles.iter().enumerate() {
            let _ = relations;
            let value = if r < 2 {
                // Entity-valued relations: never a self loop, so the chain always advances.
                let mut v = entities[(xorshift(&mut st) as usize) % entities.len()];
                if v == *e {
                    let i = entities.iter().position(|x| *x == v).unwrap_or(0);
                    v = entities[(i + 1) % entities.len()];
                }
                v
            } else {
                literals[(xorshift(&mut st) as usize) % literals.len()]
            };
            records.push((*role, *e, value));
        }
    }
    RelWorld {
        records,
        entities: entities.to_vec(),
        version,
    }
}

/// The declared task semantics, computed from the world alone: never from the model.
fn rel_oracle(
    world: &RelWorld,
    roles: &[u32],
    relation: usize,
    entity: u32,
) -> Option<(u32, u32, usize, u32, u8)> {
    let first = world
        .records
        .iter()
        .position(|(role, key, _)| *role == roles[relation] && *key == entity)?;
    let value = world.records[first].2;
    if world.entities.contains(&value) && relation < 2 {
        let follow = relation + 2;
        let second = world
            .records
            .iter()
            .position(|(role, key, _)| *role == roles[follow] && *key == value)?;
        Some((value, world.records[second].2, second, roles[follow], 2))
    } else {
        Some((value, value, first, 0, 1))
    }
}

/// One step of the single causal serving path. `freeze` carries declared lesion switches; the
/// production path uses the default.
#[derive(Clone, Copy, Default)]
struct RelLesion {
    reads_disabled: bool,
    freeze_relation_match: bool,
    freeze_continuation_to_continue: bool,
    fixed_depth: Option<u8>,
}

#[derive(Default, serde::Serialize)]
struct RelTrace {
    selected_roles: Vec<u32>,
    selected_abs: Vec<u32>,
    retained: Vec<u32>,
    actions: Vec<RelAction>,
    events: Vec<serde_json::Value>,
}

struct RelOutcome {
    emitted: Option<u32>,
    selected_roles: Vec<u32>,
    selected_abs: Vec<u32>,
    retained: Vec<u32>,
    actions: Vec<RelAction>,
    events: Vec<serde_json::Value>,
    terminal: RelAction,
    hops: u8,
    frame_bytes: usize,
    snapshots: Vec<Vec<u8>>,
}

fn rel_binding(model: &RelationalModel, world: &RelWorld) -> Result<RelBinding, String> {
    let t = group_table();
    let mut geometry = t.product.to_vec();
    geometry.extend_from_slice(&t.inverse);
    geometry.push(t.identity);
    Ok(RelBinding {
        model: sha256_bytes(&model.to_bytes()),
        geometry: sha256_bytes(&geometry),
        tokenizer: hex_to_bytes(DERIVED_SHA)?
            .try_into()
            .map_err(|_| "tokenizer digest length")?,
        world_namespace: world.version,
    })
}

/// The sole incremental session transition. It consumes the serialized phase and observed world;
/// no selected candidate or previous relation is retained in the caller's control stack.
fn rel_step(
    model: &RelationalModel,
    world: &RelWorld,
    frame: &mut RelFrame,
    lesion: RelLesion,
    trace: &mut RelTrace,
) -> Result<(), String> {
    frame.validate(VOCAB, &rel_binding(model, world)?)?;
    if frame.terminal.is_some() {
        return Ok(());
    }
    let before = frame.clone();
    match frame.pending.ok_or("session has no pending phase")? {
        RelAction::Read => {
            if frame.hops >= REL_MAX_HOPS {
                frame.stop(RelAction::Exhausted);
            } else if lesion.reads_disabled {
                frame.stop(RelAction::Unresolved);
            } else {
                trace.actions.push(RelAction::Read);
                let entity = frame.entity.ok_or("session lost query entity")?;
                let relation = frame.relation.ok_or("session lost relation")?;
                let admitted = world.admit(entity);
                let pick = if lesion.freeze_relation_match {
                    (!admitted.is_empty()).then_some(0)
                } else {
                    rank(
                        model,
                        relation,
                        frame.retained.is_some_and(|v| world.entities.contains(&v)),
                        &admitted,
                    )
                };
                match pick {
                    None => frame.stop(RelAction::Unresolved),
                    Some(pick) => {
                        let chosen = admitted[pick];
                        // A relation-disabled lesion removes BOTH matching rank and matching gate.
                        if !lesion.freeze_relation_match
                            && model.relation_matches(relation, chosen.role) <= 0
                        {
                            frame.stop(RelAction::Unresolved);
                        } else {
                            let abs = world
                                .records
                                .iter()
                                .position(|(r, k, v)| {
                                    *r == chosen.role && *k == chosen.key && *v == chosen.value
                                })
                                .ok_or("selected candidate lost its exact occurrence")?
                                as u32;
                            frame.capture(CapturedPayload {
                                seq: world.version,
                                abs,
                                payload: chosen.value,
                                version: world.version,
                            })?;
                            trace.selected_roles.push(chosen.role);
                            trace.selected_abs.push(abs);
                            trace.retained.push(chosen.value);
                        }
                    }
                }
            }
        }
        RelAction::Continue => {
            let value = frame.retained.ok_or("decision lost owned operand")?;
            let relation = frame.relation.ok_or("decision lost request relation")?;
            // Declared type registry is causal input, independent of whether any facts survive.
            let content_is_entity = world.entities.contains(&value);
            let proceed = if lesion.freeze_continuation_to_continue {
                true
            } else if let Some(depth) = lesion.fixed_depth {
                frame.hops < depth
            } else {
                model.should_continue(content_is_entity)
            };
            if proceed {
                match model.follow_of(relation) {
                    Some(next) => {
                        trace.actions.push(RelAction::Continue);
                        frame.advance_relation(next)?;
                    }
                    None => frame.stop(RelAction::Unresolved),
                }
            } else {
                frame.pending = Some(RelAction::Emit);
            }
        }
        RelAction::Emit => {
            frame.emit().map_err(|e| format!("emit failed: {e:?}"))?;
            trace.actions.push(RelAction::Emit);
        }
        _ => return Err("unsupported active relational phase".into()),
    }
    if let Some(terminal) = frame.terminal {
        trace.actions.push(terminal);
    }
    frame.validate(VOCAB, &rel_binding(model, world)?)?;
    trace.events.push(json!({"before": before, "after": frame,
        "observed_content_is_entity": before.retained.map(|v| world.entities.contains(&v))}));
    Ok(())
}

fn rel_finish(
    model: &RelationalModel,
    world: &RelWorld,
    mut frame: RelFrame,
    lesion: RelLesion,
    mut trace: RelTrace,
    snapshots: Vec<Vec<u8>>,
) -> Result<RelOutcome, String> {
    // Each read can cause only one decision and one emission. Execution cap is distinct from Stop.
    while frame.terminal.is_none() {
        rel_step(model, world, &mut frame, lesion, &mut trace)?;
    }
    let frame_bytes = frame.to_bytes()?.len();
    Ok(RelOutcome {
        emitted: frame.emitted.first().copied(),
        selected_roles: trace.selected_roles,
        selected_abs: trace.selected_abs,
        retained: trace.retained,
        actions: trace.actions,
        events: trace.events,
        terminal: frame.terminal.ok_or("missing final reason")?,
        hops: frame.hops,
        frame_bytes,
        snapshots,
    })
}

#[allow(clippy::too_many_arguments)]
fn rel_serve(
    model: &RelationalModel,
    world: &RelWorld,
    _roles: &[u32],
    relation: u32,
    entity: u32,
    lesion: RelLesion,
    resume_probe: bool,
) -> Result<RelOutcome, String> {
    let binding = rel_binding(model, world)?;
    let mut frame = RelFrame::start(relation, entity, binding);
    let mut trace = RelTrace::default();
    if resume_probe {
        rel_step(model, world, &mut frame, lesion, &mut trace)?;
        let bytes = frame.to_bytes()?;
        // Discard the live frame before calling the independent continuation entry.
        drop(frame);
        let restored = RelFrame::from_bytes(&bytes, VOCAB, &binding)?;
        rel_finish(model, world, restored, lesion, trace, vec![bytes])
    } else {
        rel_finish(model, world, frame, lesion, trace, Vec::new())
    }
}

fn rel_run() -> Result<ExitCode, String> {
    let mut root = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/relational-session-1",
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
            "crates/uor-r4-core/src/native_geometric/learner/result_decoder.rs",
            "crates/uor-r4-core/src/native_geometric/learner/shared_transition.rs",
            "crates/uor-r4-core/src/native_geometric/learner/grounded_session.rs",
            "crates/uor-r4-core/src/native_geometric/learner/relational_session.rs",
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

    // Banks: request relation names, record role tokens, entity values, literal values.
    let relations: Vec<u32> = banks.keys.iter().copied().take(REL_N_RELATIONS).collect();
    let roles: Vec<u32> = banks
        .keys
        .iter()
        .copied()
        .skip(REL_N_RELATIONS)
        .take(REL_N_RELATIONS)
        .collect();
    let entities: Vec<u32> = banks
        .values_fit
        .iter()
        .copied()
        .take(REL_N_ENTITIES)
        .collect();
    let literals: Vec<u32> = banks
        .values_fit
        .iter()
        .copied()
        .skip(REL_N_ENTITIES)
        .take(REL_N_LITERALS)
        .collect();
    if relations.len() < REL_N_RELATIONS
        || roles.len() < REL_N_RELATIONS
        || entities.len() < REL_N_ENTITIES
        || literals.len() < REL_N_LITERALS
    {
        return Err("insufficient bank for the relational fixture".into());
    }

    let dev_worlds: Vec<RelWorld> = (0..REL_DEV_WORLDS)
        .map(|i| {
            rel_make_world(
                REL_DEV_SEED + i * 0x9E37,
                &relations,
                &roles,
                &entities,
                &literals,
                1 + i as u32,
            )
        })
        .collect();
    let final_worlds: Vec<RelWorld> = (0..REL_FINAL_WORLDS)
        .map(|i| {
            rel_make_world(
                REL_FINAL_SEED + i * 0x9E37,
                &relations,
                &roles,
                &entities,
                &literals,
                100 + i as u32,
            )
        })
        .collect();

    // Declared offline supervision: gold intermediate actions from the development worlds.
    let mut dev_examples: Vec<RelExample> = Vec::new();
    for w in dev_worlds.iter() {
        for (r, _) in relations.iter().enumerate() {
            for e in entities.iter() {
                if let Some((v1, v2, _abs, second_role, depth)) = rel_oracle(w, &roles, r, *e) {
                    dev_examples.push(RelExample {
                        relation: relations[r],
                        entity: *e,
                        first_role: roles[r],
                        first_value: v1,
                        follow_relation: if depth >= 2 { relations[r + 2] } else { 0 },
                        second_role,
                        second_value: v2,
                        depth,
                        content_is_entity: w.entities.contains(&v1),
                    });
                }
            }
        }
    }
    // Capable relation-only schedule fitted from the same development action labels.
    let fixed_depth_by_relation: BTreeMap<u32, u8> = relations
        .iter()
        .map(|relation| {
            let mut counts = BTreeMap::<u8, usize>::new();
            for example in dev_examples.iter().filter(|e| e.relation == *relation) {
                *counts.entry(example.depth).or_default() += 1;
            }
            let depth = counts
                .into_iter()
                .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
                .map(|(depth, _)| depth)
                .unwrap_or(1);
            (*relation, depth)
        })
        .collect();
    let (model, fit) = learn_relational_model(&dev_examples, false, false);
    let (categorical, cat_fit) = learn_relational_model(&dev_examples, false, true);
    let (cyclic_model, cyc_fit) = learn_relational_model(&dev_examples, true, false);

    // Export and independently reload before any reported answer.
    let mut artifacts: Vec<serde_json::Value> = Vec::new();
    let mut reload = |name: &str, m: &RelationalModel| -> Result<RelationalModel, String> {
        let bytes = m.to_bytes();
        let rel = format!("artifacts/{name}.rlrm");
        write_checked(&root, &rel, &bytes)?;
        let loaded = RelationalModel::from_bytes(
            &std::fs::read(root.join(&rel)).map_err(|e| e.to_string())?,
            parent.cfg.vocab,
        )?;
        if loaded != *m {
            return Err("relational artifact reload mismatch".into());
        }
        artifacts.push(json!({"arm": name, "artifact_sha256": sha256_hex(&bytes),
            "artifact_bytes": bytes.len(), "reload_identical": true,
            "algebra": if m.cyclic {"cyclic_c120"} else if m.categorical {"categorical"} else {"signed_h4"},
            "group_digest": group_digest}));
        Ok(loaded)
    };
    let model_pre = model.clone();
    let model = reload("relational_primary", &model_pre)?;
    let categorical = reload("relational_categorical_control", &categorical)?;
    let cyclic_model = reload("relational_cyclic_control", &cyclic_model)?;

    // Full-predictor parity: the pre-export and the independently loaded model must agree on every
    // request of every world before any reported result.
    let mut parity = 0usize;
    for w in dev_worlds.iter().chain(final_worlds.iter()) {
        for (r, _) in relations.iter().enumerate() {
            for e in entities.iter() {
                let a = rel_serve(
                    &model_pre,
                    w,
                    &roles,
                    relations[r],
                    *e,
                    RelLesion::default(),
                    false,
                )?;
                let b = rel_serve(
                    &model,
                    w,
                    &roles,
                    relations[r],
                    *e,
                    RelLesion::default(),
                    false,
                )?;
                if a.emitted != b.emitted
                    || a.hops != b.hops
                    || a.selected_abs != b.selected_abs
                    || a.terminal != b.terminal
                {
                    return Err("loaded relational full-predictor parity failure".into());
                }
                parity += 1;
            }
        }
    }

    // ---------------- Evaluation ----------------
    let evaluate = |m: &RelationalModel,
                    worlds: &[RelWorld],
                    lesion: RelLesion|
     -> Result<(usize, usize, usize, Vec<serde_json::Value>), String> {
        let mut complete = 0usize;
        let mut total = 0usize;
        let mut depth_ok = 0usize;
        let mut rows = Vec::new();
        for (wi, w) in worlds.iter().enumerate() {
            for (r, _) in relations.iter().enumerate() {
                for e in entities.iter() {
                    total += 1;
                    let expected = rel_oracle(w, &roles, r, *e);
                    let served = rel_serve(m, w, &roles, relations[r], *e, lesion, false)?;
                    let (want_answer, want_depth) = match expected {
                        Some((_, answer, _, _, depth)) => (Some(answer), depth),
                        None => (None, 0),
                    };
                    let ok = served.emitted == want_answer && served.terminal == RelAction::Stop;
                    if ok {
                        complete += 1;
                    }
                    if served.hops == want_depth {
                        depth_ok += 1;
                    }
                    rows.push(json!({
                        "world": wi, "world_version": w.version, "relation": relations[r],
                        "entity": e, "expected_answer": want_answer, "expected_depth": want_depth,
                        "emitted": served.emitted,
                        "decoded": served.emitted.map(|t| tokenizer.decode(&[t])).unwrap_or_default(),
                        "selected_roles": served.selected_roles, "selected_abs": served.selected_abs,
                        "retained": served.retained, "hops": served.hops,
                        "terminal": format!("{:?}", served.terminal),
                        "actions": served.actions.iter().map(|a| format!("{a:?}")).collect::<Vec<_>>(),
                        "events": served.events,
                        "correct": ok,
                    }));
                }
            }
        }
        Ok((complete, total, depth_ok, rows))
    };

    let (dev_ok, dev_total, dev_depth, dev_rows) =
        evaluate(&model, &dev_worlds, RelLesion::default())?;
    let (fin_ok, fin_total, fin_depth, fin_rows) =
        evaluate(&model, &final_worlds, RelLesion::default())?;
    let (cat_ok, cat_total, _, cat_rows) =
        evaluate(&categorical, &final_worlds, RelLesion::default())?;
    let (cyc_ok, cyc_total, _, cyc_rows) =
        evaluate(&cyclic_model, &final_worlds, RelLesion::default())?;
    let (no_rel_ok, _, _, no_rel_rows) = evaluate(
        &model,
        &final_worlds,
        RelLesion {
            freeze_relation_match: true,
            ..RelLesion::default()
        },
    )?;
    let (always_ok, _, _, always_rows) = evaluate(
        &model,
        &final_worlds,
        RelLesion {
            freeze_continuation_to_continue: true,
            ..RelLesion::default()
        },
    )?;
    let (no_read_ok, _, _, no_read_rows) = evaluate(
        &model,
        &final_worlds,
        RelLesion {
            reads_disabled: true,
            ..RelLesion::default()
        },
    )?;

    let arms = json!([
        {"arm": "learned_relational", "split": "development", "complete": dev_ok, "total": dev_total, "depth_correct": dev_depth},
        {"arm": "learned_relational", "split": "final", "complete": fin_ok, "total": fin_total, "depth_correct": fin_depth},
        {"arm": "categorical_control", "split": "final", "complete": cat_ok, "total": cat_total},
        {"arm": "cyclic_control", "split": "final", "complete": cyc_ok, "total": cyc_total},
        {"arm": "relation_rank_and_gate_disabled", "split": "final", "complete": no_rel_ok, "total": fin_total},
        {"arm": "continuation_frozen_continue", "split": "final", "complete": always_ok, "total": fin_total},
        {"arm": "reads_disabled", "split": "final", "complete": no_read_ok, "total": fin_total},
    ]);

    // ---------------- Causal interventions ----------------
    let w0 = &final_worlds[0];
    let req_rel = relations[0];
    let req_ent = entities[0];
    let base_out = rel_serve(
        &model,
        w0,
        &roles,
        req_rel,
        req_ent,
        RelLesion::default(),
        false,
    )?;
    // One-position source edit: change exactly the answering record's value, keep every other record
    // fixed, and independently derive the expected continuation from the edited world.
    let pos = w0
        .records
        .iter()
        .position(|(role, key, _)| *role == roles[0] && *key == req_ent)
        .ok_or("intervention could not locate the answering record")?;
    let mut edited = w0.clone();
    let mut replacement = entities[1];
    if replacement == w0.records[pos].2 {
        replacement = entities[2];
    }
    edited.records[pos].2 = replacement;
    let edited_out = rel_serve(
        &model,
        &edited,
        &roles,
        req_rel,
        req_ent,
        RelLesion::default(),
        false,
    )?;
    let edited_oracle = rel_oracle(&edited, &roles, 0, req_ent);
    // Request relation change with the entity and every record held fixed.
    let other_rel_out = rel_serve(
        &model,
        w0,
        &roles,
        relations[2],
        req_ent,
        RelLesion::default(),
        false,
    )?;
    let other_rel_oracle = rel_oracle(w0, &roles, 2, req_ent);
    // Required record removed.
    let mut removed = w0.clone();
    removed.records.remove(pos);
    let removed_out = rel_serve(
        &model,
        &removed,
        &roles,
        req_rel,
        req_ent,
        RelLesion::default(),
        false,
    )?;
    // Plausible distractor added: same key, an unseen role token.
    let mut with_distractor = w0.clone();
    let decoy_role = banks.keys[8];
    with_distractor
        .records
        .push((decoy_role, req_ent, literals[0]));
    let distractor_out = rel_serve(
        &model,
        &with_distractor,
        &roles,
        req_rel,
        req_ent,
        RelLesion::default(),
        false,
    )?;
    // Exact origin identity is separate from the validity of an immutable owned copy.
    let binding = rel_binding(&model, w0)?;
    let mut frame = RelFrame::start(req_rel, req_ent, binding);
    let owned = CapturedPayload {
        seq: w0.version,
        abs: pos as u32,
        payload: w0.records[pos].2,
        version: w0.version,
    };
    frame.capture(owned)?;
    let overwritten = CapturedPayload {
        payload: literals[3],
        version: w0.version + 1,
        ..owned
    };
    let same_value_new_version = CapturedPayload {
        version: w0.version + 1,
        ..owned
    };
    let origin_now = overwritten.payload;
    let mut resume_identical = 0usize;
    let mut resume_total = 0usize;
    let mut saved_snapshots = Vec::new();
    for w in final_worlds.iter() {
        for relation in relations.iter() {
            for entity in entities.iter() {
                resume_total += 1;
                let plain = rel_serve(
                    &model,
                    w,
                    &roles,
                    *relation,
                    *entity,
                    RelLesion::default(),
                    false,
                )?;
                let resumed = rel_serve(
                    &model,
                    w,
                    &roles,
                    *relation,
                    *entity,
                    RelLesion::default(),
                    true,
                )?;
                if plain.emitted == resumed.emitted
                    && plain.hops == resumed.hops
                    && plain.selected_abs == resumed.selected_abs
                    && plain.terminal == resumed.terminal
                    && plain.actions == resumed.actions
                    && plain.events == resumed.events
                {
                    resume_identical += 1;
                }
                for bytes in resumed.snapshots {
                    let name = format!(
                        "snapshots/world-{}-rel-{}-entity-{}.rlrf",
                        w.version, relation, entity
                    );
                    write_checked(&root, &name, &bytes)?;
                    saved_snapshots.push(
                        json!({"path": name, "sha256": sha256_hex(&bytes), "bytes": bytes.len(),
                        "world_namespace": w.version, "relation": relation, "entity": entity}),
                    );
                }
            }
        }
    }
    let mut s1 = RelFrame::start(relations[0], entities[0], binding);
    let mut s2 = RelFrame::start(relations[2], entities[1], binding);
    let mut tr1 = RelTrace::default();
    let mut tr2 = RelTrace::default();
    while s1.terminal.is_none() || s2.terminal.is_none() {
        if s1.terminal.is_none() {
            rel_step(&model, w0, &mut s1, RelLesion::default(), &mut tr1)?;
        }
        if s2.terminal.is_none() {
            rel_step(&model, w0, &mut s2, RelLesion::default(), &mut tr2)?;
        }
    }
    let isolated1 = rel_serve(
        &model,
        w0,
        &roles,
        relations[0],
        entities[0],
        RelLesion::default(),
        false,
    )?;
    let isolated2 = rel_serve(
        &model,
        w0,
        &roles,
        relations[2],
        entities[1],
        RelLesion::default(),
        false,
    )?;
    let interleaved_independent = s1.emitted.first().copied() == isolated1.emitted
        && s2.emitted.first().copied() == isolated2.emitted
        && tr1.events == isolated1.events
        && tr2.events == isolated2.events;
    let snapshot = frame.to_bytes()?;
    let bad_binding = RelBinding {
        model: [0; 32],
        ..binding
    };
    let wrong_model_rejected = RelFrame::from_bytes(&snapshot, VOCAB, &bad_binding).is_err();
    let wrong_world_rejected = RelFrame::from_bytes(
        &snapshot,
        VOCAB,
        &RelBinding {
            world_namespace: binding.world_namespace + 1,
            ..binding
        },
    )
    .is_err();
    let mut malformed = frame.clone();
    malformed.terminal = Some(RelAction::Stop);
    let malformed_phase_rejected =
        RelFrame::from_bytes(&malformed.to_bytes()?, VOCAB, &binding).is_err();

    // Development diagnostic: exactly the same request and record positions, but direct versus
    // entity-valued content requires different depths. The declared entity registry stays fixed.
    let mut direct = w0.clone();
    direct.records[pos].2 = literals[0];
    let middle = w0.records[pos].2;
    let mut orphan = w0.clone();
    orphan.records.retain(|(_, key, _)| *key != middle);
    let cases = [
        ("entity", w0.clone()),
        ("direct_literal", direct),
        ("orphan_entity", orphan),
    ];
    let mut content_diagnostic = Vec::new();
    for (name, world) in cases {
        let expected = rel_oracle(&world, &roles, 0, req_ent);
        let mut rows = Vec::new();
        for (arm, lesion) in [
            ("learned_type_policy", RelLesion::default()),
            (
                "relation_only_fixed_depth",
                RelLesion {
                    fixed_depth: fixed_depth_by_relation.get(&req_rel).copied(),
                    ..RelLesion::default()
                },
            ),
        ] {
            let result = rel_serve(&model, &world, &roles, req_rel, req_ent, lesion, false)?;
            let expected_terminal = if expected.is_some() {
                RelAction::Stop
            } else {
                RelAction::Unresolved
            };
            rows.push(json!({"arm": arm, "emitted": result.emitted, "hops": result.hops,
                "terminal": result.terminal, "expected_answer": expected.map(|x| x.1),
                "expected_terminal": expected_terminal,
                "correct": result.emitted == expected.map(|x| x.1) && result.terminal == expected_terminal,
                "selected_abs": result.selected_abs, "actions": result.actions, "events": result.events}));
        }
        content_diagnostic.push(json!({"case": name, "world": world,
            "request": {"relation": req_rel, "entity": req_ent}, "rows": rows}));
    }
    let mut unresolved_ok = 0usize;
    let mut unresolved_total = 0usize;
    for w in final_worlds.iter() {
        for (r, _) in relations.iter().enumerate() {
            let e = entities[r % entities.len()];
            let Some(p) = w
                .records
                .iter()
                .position(|(role, key, _)| *role == roles[r] && *key == e)
            else {
                continue;
            };
            let mut incomplete = w.clone();
            incomplete.records.remove(p);
            unresolved_total += 1;
            let out = rel_serve(
                &model,
                &incomplete,
                &roles,
                relations[r],
                e,
                RelLesion::default(),
                false,
            )?;
            if out.terminal == RelAction::Unresolved && out.emitted.is_none() {
                unresolved_ok += 1;
            }
        }
    }
    let interventions = json!({
        "incomplete_world_missing_required_fact": {
            "unresolved_and_silent": unresolved_ok, "total": unresolved_total,
            "note": "the required record is removed from the world; the session must stop unresolved with no emission",
        },
        "one_position_source_edit": {
            "world": 0, "relation": req_rel, "entity": req_ent, "changed_record_index": pos,
            "value_before": w0.records[pos].2, "value_after": replacement,
            "emitted_before": base_out.emitted, "emitted_after": edited_out.emitted,
            "retained_before": base_out.retained, "retained_after": edited_out.retained,
            "selected_abs_before": base_out.selected_abs, "selected_abs_after": edited_out.selected_abs,
            "world_before": w0, "world_after": edited,
            "events_before": base_out.events, "events_after": edited_out.events,
            "independent_expected_after": edited_oracle.map(|o| o.1),
            "answer_changed": base_out.emitted != edited_out.emitted,
            "matches_independent_expectation": edited_out.emitted == edited_oracle.map(|o| o.1),
        },
        "request_relation_change_fixed_evidence": {
            "relation_before": req_rel, "relation_after": relations[2], "entity": req_ent,
            "emitted_before": base_out.emitted, "emitted_after": other_rel_out.emitted,
            "independent_expected_after": other_rel_oracle.map(|o| o.1),
            "selected_roles_before": base_out.selected_roles, "selected_roles_after": other_rel_out.selected_roles,
            "changed": base_out.emitted != other_rel_out.emitted,
            "matches_independent_expectation": other_rel_out.emitted == other_rel_oracle.map(|o| o.1),
        },
        "required_record_removed": {
            "removed_index": pos, "terminal": format!("{:?}", removed_out.terminal),
            "emitted": removed_out.emitted,
        },
        "plausible_distractor_added": {
            "answer_preserved": distractor_out.emitted == base_out.emitted,
            "selected_roles_preserved": distractor_out.selected_roles == base_out.selected_roles,
        },
        "owned_capture_versus_live_reference": {
            "owned_payload": owned.payload, "origin_now": origin_now,
            "owned_still_usable": frame.retained == Some(owned.payload),
            "origin_is_live": frame.origin_is_live(Some(overwritten)),
            "same_value_new_version_is_live": frame.origin_is_live(Some(same_value_new_version)),
        },
        "pause_resume_identical": {"identical": resume_identical, "total": resume_total},
        "interleaved_sessions_independent": interleaved_independent,
        "interleaved_events": [tr1.events, tr2.events],
        "snapshot_validation": {"wrong_model_rejected": wrong_model_rejected, "wrong_world_rejected": wrong_world_rejected, "malformed_phase_rejected": malformed_phase_rejected},
        "same_request_content_diagnostic": content_diagnostic,
        "saved_snapshots": saved_snapshots,
    });

    write_checked(
        &root,
        "rows.jsonl",
        fin_rows
            .iter()
            .map(|r| serde_json::to_string(r).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
            .as_bytes(),
    )?;

    let result = json!({
        "schema": "uor-r4.relational-session/2",
        "base_revision": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
        "running_source": {
            "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
            "git_dirty": std::env::var("UOR_GIT_DIRTY").unwrap_or_else(|_| "unknown".into()),
            "executable_sha256": executable_sha256,
            "source_files": source_files,
        },
        "inputs": {"E_sha256": E_SHA, "S_sha256": S_SHA, "tokenizer_derived": DERIVED_SHA,
            "group_digest": group_digest, "f_bits": parent.cfg.f_bits},
        "task": {
            "records": "role/key/value triples in a supplied world; the role token encodes the relation while the request names it",
            "chain": "read requested relation; observed declared entity type chooses learned continuation even when entity facts are missing; otherwise emit",
            "relations": relations, "roles": roles, "entities": entities, "literals": literals,
            "authored_structure": "token layout, record triples, entity type registry and follow-up convention supplied; role/relation codes and follow map fitted; continuation fitted from observed type and separate action label; scorer coefficients fixed",
        },
        "learning": {"primary": fit, "categorical": cat_fit, "cyclic": cyc_fit, "fixed_score_coefficients": [1,1,0,0], "fitted_relation_only_depths": fixed_depth_by_relation},
        "exposure": "Original worlds repeatedly exposed by PR1341; corrected replay and new same-request cases are development diagnostics, not fresh confirmation",
        "worlds": {"development": dev_worlds, "exposed_original_final": final_worlds},
        "all_arm_rows": {"development_primary": dev_rows, "exposed_primary": fin_rows, "categorical": cat_rows, "cyclic": cyc_rows, "relation_disabled": no_rel_rows, "always_continue": always_rows, "read_disabled": no_read_rows},
        "arms": arms,
        "artifacts": artifacts,
        "loaded_full_predictor_parity_requests": parity,
        "interventions": interventions,
        "generated_final": fin_rows.iter().filter(|r| r["correct"] == json!(true)).take(6).cloned().collect::<Vec<_>>(),
        "scope": "bounded authored memory worlds; supplied layout and follow-up convention. Not broad language understanding. Energy UNAVAILABLE; whole-path D0-b not claimed.",
        "elapsed_s": started.elapsed().as_secs_f64(),
    });
    write_json(&root, "result.json", &result)?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "relational-session: dev {}/{} final {}/{} depth {} | cat {} cyc {} | no-rel {} always {} no-read {} | sealed {} unlisted | {:.1}s",
        dev_ok,
        dev_total,
        fin_ok,
        fin_total,
        fin_depth,
        cat_ok,
        cyc_ok,
        no_rel_ok,
        always_ok,
        no_read_ok,
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod relational_session_tests {
    use super::*;

    fn fixture() -> (RelationalModel, RelWorld, Vec<u32>, Vec<u32>) {
        let relations = vec![200, 201, 202, 203];
        let roles = vec![300, 301, 302, 303];
        let entities: Vec<u32> = (10..18).collect();
        let world = rel_make_world(71, &relations, &roles, &entities, &[40, 41, 42, 43], 7);
        let mut examples = Vec::new();
        for r in 0..4 {
            for entity in &entities {
                let (v1, v2, _, second_role, depth) =
                    rel_oracle(&world, &roles, r, *entity).unwrap();
                examples.push(RelExample {
                    relation: relations[r],
                    entity: *entity,
                    first_role: roles[r],
                    first_value: v1,
                    follow_relation: if depth == 2 { relations[r + 2] } else { 0 },
                    second_role,
                    second_value: v2,
                    depth,
                    content_is_entity: world.entities.contains(&v1),
                });
            }
        }
        (
            learn_relational_model(&examples, false, false).0,
            world,
            relations,
            roles,
        )
    }

    #[test]
    fn frame_drives_resumed_and_interleaved_inference() {
        let (model, world, relations, roles) = fixture();
        let plain = rel_serve(
            &model,
            &world,
            &roles,
            relations[0],
            10,
            RelLesion::default(),
            false,
        )
        .unwrap();
        let resumed = rel_serve(
            &model,
            &world,
            &roles,
            relations[0],
            10,
            RelLesion::default(),
            true,
        )
        .unwrap();
        assert_eq!(plain.hops, 2);
        assert_eq!(
            plain
                .actions
                .iter()
                .filter(|a| **a == RelAction::Read)
                .count(),
            2
        );
        assert_eq!(plain.events, resumed.events);
        assert_eq!(plain.emitted, resumed.emitted);
        let binding = rel_binding(&model, &world).unwrap();
        let mut a = RelFrame::start(relations[0], 10, binding);
        let mut b = RelFrame::start(relations[2], 11, binding);
        let mut at = RelTrace::default();
        let mut bt = RelTrace::default();
        while a.terminal.is_none() || b.terminal.is_none() {
            rel_step(&model, &world, &mut a, RelLesion::default(), &mut at).unwrap();
            rel_step(&model, &world, &mut b, RelLesion::default(), &mut bt).unwrap();
        }
        assert_eq!(at.events, plain.events);
        assert_eq!(a.emitted.first().copied(), plain.emitted);
        let bplain = rel_serve(
            &model,
            &world,
            &roles,
            relations[2],
            11,
            RelLesion::default(),
            false,
        )
        .unwrap();
        assert_eq!(bt.events, bplain.events);
    }

    #[test]
    fn same_request_content_and_missing_entity_change_control() {
        let (model, world, relations, roles) = fixture();
        let entity = rel_serve(
            &model,
            &world,
            &roles,
            relations[0],
            10,
            RelLesion::default(),
            false,
        )
        .unwrap();
        let mut direct = world.clone();
        direct.records[0].2 = 40;
        let direct_out = rel_serve(
            &model,
            &direct,
            &roles,
            relations[0],
            10,
            RelLesion::default(),
            false,
        )
        .unwrap();
        assert_eq!(entity.hops, 2);
        assert_eq!((direct_out.hops, direct_out.emitted), (1, Some(40)));
        let fixed = rel_serve(
            &model,
            &direct,
            &roles,
            relations[0],
            10,
            RelLesion {
                fixed_depth: Some(2),
                ..RelLesion::default()
            },
            false,
        )
        .unwrap();
        assert_ne!(fixed.emitted, Some(40));
        let mut orphan = world.clone();
        orphan
            .records
            .retain(|(_, key, _)| *key != world.records[0].2);
        let missing = rel_serve(
            &model,
            &orphan,
            &roles,
            relations[0],
            10,
            RelLesion::default(),
            false,
        )
        .unwrap();
        assert_eq!(missing.terminal, RelAction::Unresolved);
        assert_eq!(missing.emitted, None);
        assert!(rel_oracle(&orphan, &roles, 0, 10).is_none());
    }
}

// ---------------------------------------------------------------------------
// Contextual occurrence roles and exact spans from readable text
// ---------------------------------------------------------------------------

const OB_PERSONS: usize = 4;
const OB_EOS: u32 = u32::MAX - 1;

/// Authored vocabulary. Case and BPE boundaries matter: visual cue/name similarity is not
/// automatically a shared learned feature. Exact full-span identity distinguishes overlapping names.
struct ObNames {
    persons: [&'static str; OB_PERSONS],
    offices: [&'static str; 3],
    projects: [&'static str; 3],
}

impl ObNames {
    fn cue(&self, goal: Goal, redirect: bool) -> &'static str {
        match (goal, redirect) {
            (Goal::Office, false) => "works in",
            (Goal::Office, true) => "office follows",
            (Goal::Project, false) => "is on project",
            (Goal::Project, true) => "project follows",
        }
    }
    fn targets(&self, goal: Goal, target_is_key: bool) -> Vec<String> {
        match goal {
            Goal::Office if target_is_key => self.persons.iter().map(|s| s.to_string()).collect(),
            Goal::Office => self.offices.iter().map(|s| s.to_string()).collect(),
            Goal::Project => self.projects.iter().map(|s| s.to_string()).collect(),
        }
    }
    fn role_of(goal: Goal, redirect: bool) -> usize {
        match (goal, redirect) {
            (Goal::Office, false) => 0,
            (Goal::Office, true) => 1,
            (Goal::Project, false) => 2,
            (Goal::Project, true) => 3,
        }
    }
}

struct ObWorld {
    id: u32,
    version: u32,
    clauses: Vec<Clause>,
    labels: Vec<ClauseLabel>,
    /// Independent expectation per request: answer tokens and the number of reads required.
    oracle: Vec<((usize, Goal), (Vec<u32>, u8))>,
    texts: Vec<String>,
}

/// Encode one readable clause and locate its declared spans by exact sub-sequence identity. The cue
/// and the object are encoded as they appear after a space, and the located extents are checked.
/// Verify actual token bytes against the original input, not merely the total byte length.
/// This authored-text adapter requires exact alignment; legacy token-only input is explicit elsewhere.
fn ob_byte_lengths(
    tokenizer: &HfBpeTokenizer,
    text: &str,
    tokens: &[u32],
) -> Result<Vec<u32>, String> {
    let lens: Vec<u32> = tokens
        .iter()
        .map(|t| tokenizer.decode_bytes(&[*t]).len() as u32)
        .collect();
    if lens.iter().all(|n| *n > 0)
        && tokenizer.decode_bytes(tokens) == text.as_bytes()
        && lens.iter().map(|b| *b as usize).sum::<usize>() == text.len()
    {
        Ok(lens)
    } else {
        Err("observed token bytes do not exactly reproduce source text".into())
    }
}

/// Which declared argument a clause part realizes. Filler parts declare no span.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ObPart {
    Subject,
    Cue,
    Object,
}

/// One authored ordinary-form clause: an ordered list of `(declared role or filler, literal text)`.
struct ObForm {
    parts: Vec<(Option<ObPart>, String)>,
    role: usize,
    goal: Goal,
    action: RelAction,
}

/// Realize one authored form against the pinned tokenizer. Declared spans are located by exact byte
/// offsets computed from actual token bytes, so a boundary that is not a real token boundary is a
/// hard error instead of a silently approximate extent. No gold field is stored in the clause itself.
fn ob_build_clause(
    tokenizer: &HfBpeTokenizer,
    seg: u32,
    form: &ObForm,
) -> Result<(Clause, ClauseLabel), String> {
    let text: String = form.parts.iter().map(|(_, s)| s.as_str()).collect();
    let tokens = tokenizer.encode(&text);
    if tokens.is_empty() || tokens.len() > OB_MAX_CLAUSE {
        return Err(format!("clause outside the declared bound: {text}"));
    }
    let byte_lengths = ob_byte_lengths(tokenizer, &text, &tokens)?;
    let mut cumulative = Vec::with_capacity(tokens.len() + 1);
    cumulative.push(0usize);
    for b in &byte_lengths {
        cumulative.push(cumulative.last().copied().unwrap_or(0) + *b as usize);
    }
    if cumulative.last().copied() != Some(text.len()) {
        return Err(format!("token bytes do not tile the clause text: {text}"));
    }
    let boundary = |offset: usize| -> Result<u32, String> {
        cumulative
            .iter()
            .position(|c| *c == offset)
            .map(|i| i as u32)
            .ok_or_else(|| format!("declared boundary is not a token boundary in {text:?}"))
    };
    let mut spans: [Option<(u32, usize)>; 3] = [None; 3];
    let mut offset = 0usize;
    for (part, content) in &form.parts {
        let start = offset;
        offset += content.len();
        let Some(part) = part else { continue };
        let (i0, i1) = (boundary(start)?, boundary(offset)?);
        if i1 <= i0 {
            return Err(format!("empty declared span in {text:?}"));
        }
        let slot = match part {
            ObPart::Subject => 0,
            ObPart::Cue => 1,
            ObPart::Object => 2,
        };
        if spans[slot].is_some() {
            return Err(format!("declared span repeated in {text:?}"));
        }
        spans[slot] = Some((i0, (i1 - i0) as usize));
    }
    let subject = spans[0].ok_or("form declares no subject")?;
    let marker = spans[1].ok_or("form declares no cue")?;
    let object = spans[2];
    if marker.1 > OB_MAX_MARKER {
        return Err(format!(
            "cue span exceeds the declared feature bound in {text:?}"
        ));
    }
    Ok((
        Clause {
            seg,
            tokens,
            text,
            byte_lengths,
        },
        ClauseLabel {
            seg,
            subject,
            marker,
            object,
            role: form.role,
            goal: form.goal,
            action: form.action,
        },
    ))
}

/// Ordinary assertion forms. Style 0 is the canonical possessor order, style 1 puts the object before
/// its subject, and style 2 appends trailing nonanswer material. The cue phrase is the **same** in
/// every style, so the learned role semantics are shared across argument orders.
fn ob_assert_form(names: &ObNames, goal: Goal, person: &str, target: &str, style: usize) -> ObForm {
    let _ = names;
    let (noun, trailing) = match goal {
        Goal::Office => ("office", " downtown"),
        Goal::Project => ("project", " this year"),
    };
    let parts = match style % 3 {
        0 => vec![
            (Some(ObPart::Subject), person.to_string()),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), format!(" {noun}")),
            (None, " is".to_string()),
            (Some(ObPart::Object), format!(" {target}")),
        ],
        1 => vec![
            (Some(ObPart::Object), target.to_string()),
            (None, " is".to_string()),
            (Some(ObPart::Subject), format!(" {person}")),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), format!(" {noun}")),
        ],
        _ => vec![
            (Some(ObPart::Subject), person.to_string()),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), format!(" {noun}")),
            (None, " is".to_string()),
            (Some(ObPart::Object), format!(" {target}")),
            (None, trailing.to_string()),
        ],
    };
    ObForm {
        parts,
        role: ObNames::role_of(goal, false),
        goal,
        action: RelAction::Emit,
    }
}

/// The redirect form. Its subject is clause-initial, so the object of the previous hop (` Name`) and
/// this subject (`Name`) are different BPE sequences with the same exact lexical identity.
fn ob_redirect_form(_names: &ObNames, goal: Goal, person: &str, next: &str) -> ObForm {
    let noun = match goal {
        Goal::Office => "office",
        Goal::Project => "project",
    };
    ObForm {
        parts: vec![
            (Some(ObPart::Subject), person.to_string()),
            (Some(ObPart::Cue), format!(" {noun} follows")),
            (Some(ObPart::Object), format!(" {next}")),
        ],
        role: ObNames::role_of(goal, true),
        goal,
        action: RelAction::Continue,
    }
}

/// A genuine interrogative: the subject is interior, the cue is interior and the sentence-final `?`
/// is trailing nonanswer material. The cue phrase is the same one the assertion forms use.
fn ob_question_form(_names: &ObNames, person: &str, goal: Goal) -> ObForm {
    let noun = match goal {
        Goal::Office => "office",
        Goal::Project => "project",
    };
    ObForm {
        parts: vec![
            (None, "What is".to_string()),
            (Some(ObPart::Subject), format!(" {person}")),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), format!(" {noun}")),
            (None, "?".to_string()),
        ],
        role: ObNames::role_of(goal, false),
        goal,
        action: RelAction::Emit,
    }
}

fn ob_make_world(
    tokenizer: &HfBpeTokenizer,
    names: &ObNames,
    id: u32,
    version: u32,
    office_chains: &[Vec<usize>],
    project_chains: &[Vec<usize>],
    target_is_key: bool,
    style0: usize,
    target0: usize,
) -> Result<ObWorld, String> {
    let mut clauses = Vec::new();
    let mut labels = Vec::new();
    let mut texts = Vec::new();
    let mut oracle = Vec::new();
    let mut seg = 0u32;
    for (goal, chains) in [
        (Goal::Office, office_chains),
        (Goal::Project, project_chains),
    ] {
        for (chain_index, chain) in chains.iter().enumerate() {
            if chain.is_empty() {
                continue;
            }
            let targets = names.targets(goal, target_is_key);
            // Deterministic target/style cycling rather than a random draw: development must cover the
            // ordinary forms systematically instead of leaving a form/target pair to chance.
            let target = targets[(target0 + chain_index) % targets.len()].clone();
            let len = chain.len();
            let mut answer: Vec<u32> = Vec::new();
            for (hop, person) in chain.iter().enumerate() {
                let last = hop + 1 == len;
                let style = (style0 + chain_index + hop) % 3;
                let form = if last {
                    ob_assert_form(names, goal, names.persons[*person], &target, style)
                } else {
                    ob_redirect_form(
                        names,
                        goal,
                        names.persons[*person],
                        names.persons[chain[hop + 1]],
                    )
                };
                let (clause, label) = ob_build_clause(tokenizer, seg, &form)?;
                if last {
                    let (s, l) = label.object.ok_or("terminal clause declares no object")?;
                    answer = clause.tokens[s as usize..s as usize + l].to_vec();
                }
                texts.push(clause.text.clone());
                clauses.push(clause);
                labels.push(label);
                oracle.push((
                    (*person, goal),
                    (
                        if last { answer.clone() } else { Vec::new() },
                        (len - hop) as u8,
                    ),
                ));
                seg += 1;
            }
            for (hop, person) in chain.iter().enumerate() {
                if hop + 1 == len {
                    continue;
                }
                if let Some(e) = oracle
                    .iter_mut()
                    .find(|((p, g), _)| *p == *person && *g == goal)
                {
                    e.1 .0 = answer.clone();
                }
            }
        }
    }
    Ok(ObWorld {
        id,
        version,
        clauses,
        labels,
        oracle,
        texts,
    })
}

fn ob_question_labeled(
    tokenizer: &HfBpeTokenizer,
    names: &ObNames,
    person: usize,
    goal: Goal,
) -> Result<(Clause, ClauseLabel), String> {
    ob_build_clause(
        tokenizer,
        999,
        &ob_question_form(names, names.persons[person], goal),
    )
}

struct ObOutcome {
    emitted: Vec<u32>,
    actions: Vec<RelAction>,
    reads: u8,
    terminal: RelAction,
    selected: Vec<u32>,
    roles: Vec<u64>,
    events: Vec<serde_json::Value>,
    snapshots: Vec<serde_json::Value>,
    final_frame: TextSession,
    resume_identical: usize,
}

/// Evaluation adapter: the executable runtime sees only loaded parameters, document tokens,
/// observed question and explicitly named control. Expected goal/person/answer stay outside it.
fn ob_serve(
    model: &ObservedTextModel,
    world: &ObWorld,
    question: &Clause,
    control: TextControl,
    resume: bool,
) -> Result<ObOutcome, String> {
    let bytes = serde_json::to_vec(model).map_err(|e| e.to_string())?;
    let runtime = ObservedTextRuntime::load(
        &bytes,
        DERIVED_SHA,
        world.clauses.clone(),
        world.id,
        world.version,
        VOCAB,
        Some(OB_EOS),
    )?
    .with_control(control)?;
    let mut session = runtime.start(question)?;
    let mut snapshots = vec![runtime.snapshot(&session)?];
    let mut events = Vec::new();
    while session.terminal.is_none() {
        let before = serde_json::to_value(&session).map_err(|e| e.to_string())?;
        let effect = runtime.step(&mut session)?;
        events.push(json!({"before":before,"effect":effect,"after":session}));
        snapshots.push(runtime.snapshot(&session)?);
        if events.len() > 3 * OB_MAX_READS as usize + OB_MAX_CLAUSE + 4 {
            return Err("runtime exceeded declared read/phrase action bound".into());
        }
    }
    let mut resume_identical = 0;
    let mut saved_snapshots = Vec::new();
    if resume {
        // Each continuation is reconstructed from only the saved bytes and immutable runtime.
        // No original loop counter, previous-read flag or pending source is carried across.
        for (at, bytes) in snapshots.iter().enumerate() {
            let mut restored = runtime.restore(bytes)?;
            let mut suffix = Vec::new();
            while restored.terminal.is_none() {
                let before = serde_json::to_value(&restored).map_err(|e| e.to_string())?;
                let effect = runtime.step(&mut restored)?;
                suffix.push(json!({"before":before,"effect":effect,"after":restored}));
                if suffix.len() > 3 * OB_MAX_READS as usize + OB_MAX_CLAUSE + 4 {
                    return Err("restored runtime exceeded action bound".into());
                }
            }
            let identical = suffix == events[at..] && restored == session;
            resume_identical += usize::from(identical);
            saved_snapshots.push(json!({"checkpoint_index":at,
                "snapshot_utf8":String::from_utf8(bytes.clone()).map_err(|e|e.to_string())?,
                "snapshot_sha256":sha256_hex(bytes),"resumed_suffix":suffix,
                "resumed_final_frame":restored,"identical":identical}));
        }
    }
    let actions = events
        .iter()
        .map(|v| serde_json::from_value(v["effect"]["action"].clone()).map_err(|e| e.to_string()))
        .collect::<Result<Vec<RelAction>, _>>()?;
    let selected = events
        .iter()
        .filter(|v| {
            v["effect"]["action"] == "Read"
                && v["after"]["reads"].as_u64() > v["before"]["reads"].as_u64()
        })
        .filter_map(|v| v["effect"]["selected_segment"].as_u64().map(|n| n as u32))
        .collect();
    let roles = events
        .iter()
        .filter(|v| {
            v["effect"]["action"] == "Read"
                && v["after"]["reads"].as_u64() > v["before"]["reads"].as_u64()
        })
        .filter_map(|v| v["effect"]["selected_role"].as_u64())
        .collect();
    Ok(ObOutcome {
        emitted: session.emitted.clone(),
        actions,
        reads: session.reads,
        terminal: session.terminal.ok_or("missing runtime terminal")?,
        selected,
        roles,
        events,
        snapshots: saved_snapshots,
        final_frame: session,
        resume_identical,
    })
}

// Independent evaluation oracle over supplied labels and exact lexical identities. Never called by
// the runtime or a feature extractor. Missing records, cycles and successful answers stay distinct,
// and the traversal follows the declared lexical key so it crosses real BPE boundaries.
fn ob_expected(world: &ObWorld, entity: &[u8], goal: Goal) -> Result<serde_json::Value, String> {
    let mut key = entity.to_vec();
    let mut selected = Vec::<u32>::new();
    let mut visited = std::collections::BTreeSet::new();
    for _ in 0..OB_MAX_READS {
        let matching: Vec<_> = world
            .labels
            .iter()
            .filter(|label| {
                label.goal == goal
                    && world
                        .clauses
                        .iter()
                        .find(|c| c.seg == label.seg)
                        .and_then(|c| {
                            lexical_key(
                                &c.text,
                                &c.byte_lengths,
                                label.subject.0 as usize,
                                label.subject.1,
                            )
                        })
                        .as_deref()
                        == Some(key.as_slice())
            })
            .collect();
        if matching.is_empty() {
            return Ok(
                json!({"answer":[],"terminal":"Unresolved","reads":selected.len(),"selected_segments":selected}),
            );
        }
        if matching.len() != 1 {
            return Err("oracle observed ambiguous exact subject/goal".into());
        }
        let label = matching[0];
        if !visited.insert(label.seg) {
            return Ok(
                json!({"answer":[],"terminal":"Exhausted","reads":selected.len(),"selected_segments":selected}),
            );
        }
        let clause = world
            .clauses
            .iter()
            .find(|c| c.seg == label.seg)
            .ok_or("oracle clause missing")?;
        let (start, len) = label.object.ok_or("oracle statement object missing")?;
        let object = clause
            .tokens
            .get(start as usize..start as usize + len)
            .ok_or("oracle object outside clause")?
            .to_vec();
        selected.push(label.seg);
        if label.action == RelAction::Emit {
            return Ok(
                json!({"answer":object,"terminal":"Stop","reads":selected.len(),"selected_segments":selected}),
            );
        }
        if label.action != RelAction::Continue {
            return Err("oracle invalid statement action".into());
        }
        key = lexical_key(&clause.text, &clause.byte_lengths, start as usize, len)
            .ok_or("oracle object has no lexical identity")?;
    }
    Ok(
        json!({"answer":[],"terminal":"Exhausted","reads":selected.len(),"selected_segments":selected}),
    )
}

fn ob_outcome_record(out: &ObOutcome) -> serde_json::Value {
    json!({"emitted":out.emitted,"reads":out.reads,"terminal":out.terminal,
        "selected_segments":out.selected,"selected_roles":out.roles,"events":out.events,
        "snapshots":out.snapshots,"final_frame":out.final_frame,"resume_identical":out.resume_identical})
}

fn ob_check_oracle(out: &ObOutcome, expected: &serde_json::Value) -> bool {
    let mut answer: Vec<u32> =
        serde_json::from_value(expected["answer"].clone()).unwrap_or_default();
    if expected["terminal"] == "Stop" {
        answer.push(OB_EOS);
    }
    out.emitted == answer
        && json!(out.terminal) == expected["terminal"]
        && json!(out.reads) == expected["reads"]
        && json!(out.selected) == expected["selected_segments"]
}

struct ObPanel {
    complete: usize,
    total: usize,
    depth_ok: usize,
    rows: Vec<serde_json::Value>,
}

/// Development supervision: every statement clause of the development worlds plus the observed
/// question clauses. Question clauses contribute their own spans and cue roles, never their answers.
fn ob_training_set(
    tokenizer: &HfBpeTokenizer,
    names: &ObNames,
    worlds: &[ObWorld],
) -> Result<(Vec<Clause>, Vec<ClauseLabel>), String> {
    let mut clauses = Vec::new();
    let mut labels = Vec::new();
    for (wi, w) in worlds.iter().enumerate() {
        let base = wi as u32 * 100;
        for c in &w.clauses {
            clauses.push(Clause {
                seg: c.seg + base,
                tokens: c.tokens.clone(),
                text: c.text.clone(),
                byte_lengths: c.byte_lengths.clone(),
            });
        }
        for l in &w.labels {
            let mut l2 = l.clone();
            l2.seg += base;
            labels.push(l2);
        }
        for person in 0..OB_PERSONS {
            for goal in [Goal::Office, Goal::Project] {
                let (mut q, mut label) = ob_question_labeled(tokenizer, names, person, goal)?;
                let seg = 900_000 + wi as u32 * 100 + (person as u32) * 2 + goal.index() as u32;
                q.seg = seg;
                label.seg = seg;
                clauses.push(q);
                labels.push(label);
            }
        }
    }
    Ok((clauses, labels))
}

/// Evaluate every person/goal request of one world through the loaded runtime. The authored oracle
/// and an independent exact traversal must agree before any model output is looked at.
fn ob_evaluate(
    tokenizer: &HfBpeTokenizer,
    names: &ObNames,
    model: &ObservedTextModel,
    world: &ObWorld,
    control: &TextControl,
    split: &str,
    arm: &str,
) -> Result<ObPanel, String> {
    let mut panel = ObPanel {
        complete: 0,
        total: 0,
        depth_ok: 0,
        rows: Vec::new(),
    };
    for person in 0..OB_PERSONS {
        for goal in [Goal::Office, Goal::Project] {
            let (q, qlabel) = ob_question_labeled(tokenizer, names, person, goal)?;
            let (want, want_depth) = world
                .oracle
                .iter()
                .find(|((p, g), _)| *p == person && *g == goal)
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            let oracle = ob_expected(world, names.persons[person].as_bytes(), goal)?;
            if oracle["answer"] != json!(want)
                || oracle["reads"] != json!(want_depth)
                || oracle["terminal"] != "Stop"
            {
                return Err("world expectation differs from independent exact traversal".into());
            }
            let query_key = lexical_key(
                &q.text,
                &q.byte_lengths,
                qlabel.subject.0 as usize,
                qlabel.subject.1,
            )
            .map(|k| String::from_utf8_lossy(&k).into_owned());
            let question_text = q.text.clone();
            let out = ob_serve(model, world, &q, control.clone(), true)?;
            let mut want_eos = want.clone();
            want_eos.push(OB_EOS);
            let ok = out.emitted == want_eos && out.terminal == RelAction::Stop;
            panel.complete += usize::from(ok);
            panel.depth_ok += usize::from(out.reads == want_depth);
            panel.total += 1;
            let emitted_text = tokenizer.decode(
                &out.emitted
                    .iter()
                    .copied()
                    .filter(|t| *t != OB_EOS)
                    .collect::<Vec<_>>(),
            );
            panel.rows.push(json!({
                "arm": arm, "split": split, "world": world.id, "world_version": world.version,
                "person": person, "goal": goal,
                "question": q, "question_text": question_text, "query_subject_key": query_key,
                "expected_answer": want, "expected_text": tokenizer.decode(&want),
                "expected_depth": want_depth, "independent_expected": oracle,
                "emitted": out.emitted, "emitted_text_without_eos": emitted_text,
                "reads": out.reads, "terminal": out.terminal,
                "selected_segments": out.selected, "selected_roles": out.roles,
                "events": out.events, "snapshots": out.snapshots, "final_frame": out.final_frame,
                "resume_identical": out.resume_identical, "actions": out.actions, "correct": ok
            }));
        }
    }
    Ok(panel)
}

/// Replace one clause's object span with a different literal, preserving the observed leading-space
/// convention and re-deriving text, alignment and retokenization together.
fn ob_edit_object(
    world: &ObWorld,
    tokenizer: &HfBpeTokenizer,
    seg: u32,
    replacement: &str,
) -> Result<ObWorld, String> {
    let mut clauses = world.clauses.clone();
    let mut labels = world.labels.clone();
    let label = labels
        .iter_mut()
        .find(|l| l.seg == seg)
        .ok_or("missing source label")?;
    let (start, len) = label.object.ok_or("source clause has no object")?;
    let clause = clauses
        .iter_mut()
        .find(|c| c.seg == seg)
        .ok_or("missing source clause")?;
    let end = (start as usize)
        .checked_add(len)
        .filter(|end| len > 0 && *end <= clause.tokens.len())
        .ok_or("source object extent is invalid")?;
    let old = tokenizer.decode_bytes(&clause.tokens[start as usize..end]);
    let lead = if old.first() == Some(&b' ') { " " } else { "" };
    let new_tokens = tokenizer.encode(&format!("{lead}{replacement}"));
    if new_tokens.is_empty() {
        return Err("edited source object is empty".into());
    }
    // Replace only the object. Trailing adjuncts and a later subject/cue are causal controls,
    // not disposable suffixes. Rebase all later supervised extents by the actual token delta.
    for span in [&mut label.subject, &mut label.marker] {
        let span_end = (span.0 as usize)
            .checked_add(span.1)
            .ok_or("source argument extent overflow")?;
        if (span.0 as usize) < end && span_end > start as usize {
            return Err("edited object overlaps another declared argument".into());
        }
        if span.0 as usize >= end {
            span.0 = u32::try_from(span.0 as usize - len + new_tokens.len())
                .map_err(|_| "edited argument offset overflow")?;
        }
    }
    clause
        .tokens
        .splice(start as usize..end, new_tokens.iter().copied());
    if clause.tokens.len() > OB_MAX_CLAUSE {
        return Err("edited clause exceeds the declared token bound".into());
    }
    clause.text = String::from_utf8(tokenizer.decode_bytes(&clause.tokens))
        .map_err(|e| format!("edited source UTF-8: {e}"))?;
    if tokenizer.encode(&clause.text) != clause.tokens {
        return Err("edited source tokenization changed the declared span boundaries".into());
    }
    clause.byte_lengths = ob_byte_lengths(tokenizer, &clause.text, &clause.tokens)?;
    label.object = Some((start, new_tokens.len()));
    let texts = clauses.iter().map(|c| c.text.clone()).collect();
    Ok(ObWorld {
        id: world.id,
        version: world.version,
        clauses,
        labels,
        oracle: Vec::new(),
        texts,
    })
}

fn ob_without_segment(world: &ObWorld, seg: u32) -> ObWorld {
    ObWorld {
        id: world.id,
        version: world.version,
        clauses: world
            .clauses
            .iter()
            .filter(|c| c.seg != seg)
            .cloned()
            .collect(),
        labels: world
            .labels
            .iter()
            .filter(|l| l.seg != seg)
            .cloned()
            .collect(),
        oracle: Vec::new(),
        texts: Vec::new(),
    }
}

fn ob_run() -> Result<ExitCode, String> {
    let mut root = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/contextual-text-roles-principal-1",
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
    let _u: Vec<Vec<i32>> = (0..120).map(|s| local.row_scores(s)).collect();
    let table = ExactGroupTable::build().map_err(|e| format!("table: {e}"))?;
    let group_digest = group_table_digest(&table);
    let source_files: Vec<serde_json::Value> = match &source_root {
        Some(sr) => [
            "crates/uor-r4-core/src/native_geometric/learner/observed_text_session.rs",
            "crates/uor-r4-core/src/native_geometric/learner/relational_session.rs",
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
    let names = ObNames {
        persons: ["Mara", "Ivo", "Cedar", "Oren"],
        offices: ["Cedar Annex", "Office Park", "Fen"],
        projects: ["Atlas", "Project Bay", "Lumen"],
    };

    // Development: one- and two-read chains over all ordinary statement styles, so a three-read
    // composition is withheld from fitting.
    let dev_worlds: Vec<ObWorld> = vec![
        ob_make_world(
            &tokenizer,
            &names,
            0,
            10,
            &[vec![0, 1], vec![2, 3]],
            &[vec![0], vec![1, 2], vec![3]],
            false,
            0,
            0,
        )?,
        ob_make_world(
            &tokenizer,
            &names,
            1,
            11,
            &[vec![0], vec![1, 2], vec![3]],
            &[vec![0, 1], vec![2, 3]],
            false,
            1,
            1,
        )?,
        ob_make_world(
            &tokenizer,
            &names,
            2,
            12,
            &[vec![0, 1], vec![2, 3]],
            &[vec![0, 2], vec![1, 3]],
            false,
            2,
            2,
        )?,
        // Development also covers the declared control family in which the terminal value is itself a
        // key elsewhere, so the topicalized form is exercised with a person-valued object too.
        ob_make_world(
            &tokenizer,
            &names,
            3,
            13,
            &[vec![0, 2], vec![1, 3]],
            &[vec![0], vec![1, 2], vec![3]],
            true,
            0,
            1,
        )?,
        ob_make_world(
            &tokenizer,
            &names,
            4,
            14,
            &[vec![0, 2], vec![1, 3]],
            &[vec![0], vec![1, 2], vec![3]],
            true,
            0,
            3,
        )?,
        ob_make_world(
            &tokenizer,
            &names,
            5,
            15,
            &[vec![0, 2], vec![1, 3]],
            &[vec![0], vec![1, 2], vec![3]],
            true,
            2,
            1,
        )?,
    ];
    // Exposed regression: three-read compositions and a terminal value that is also a key elsewhere.
    let exposed_worlds: Vec<ObWorld> = vec![
        ob_make_world(
            &tokenizer,
            &names,
            100,
            300,
            &[vec![0, 1, 2], vec![3]],
            &[vec![0], vec![1, 2, 3]],
            false,
            0,
            1,
        )?,
        ob_make_world(
            &tokenizer,
            &names,
            101,
            301,
            &[vec![0], vec![1, 2, 3]],
            &[vec![0, 1, 2], vec![3]],
            false,
            1,
            2,
        )?,
        ob_make_world(
            &tokenizer,
            &names,
            102,
            302,
            &[vec![0, 1, 2], vec![3]],
            &[vec![0], vec![1, 2, 3]],
            true,
            2,
            0,
        )?,
    ];
    // Exposed regression panel: the three-read compositions and the key-valued terminal family. This
    // panel informed the development-form coverage above, so it is reported as **exposed**.
    let heldout_worlds: Vec<ObWorld> = vec![
        ob_make_world(
            &tokenizer,
            &names,
            200,
            400,
            &[vec![0, 3, 1], vec![2]],
            &[vec![0, 1, 3], vec![2]],
            false,
            1,
            0,
        )?,
        ob_make_world(
            &tokenizer,
            &names,
            201,
            401,
            &[vec![1, 2, 3], vec![0]],
            &[vec![2, 0, 1], vec![3]],
            true,
            2,
            1,
        )?,
    ];
    // Final draw: a genuinely new composition/style draw made **after** the design was frozen, and
    // not used to change any development choice or fitted parameter.
    let final_worlds: Vec<ObWorld> = vec![
        ob_make_world(
            &tokenizer,
            &names,
            300,
            500,
            &[vec![1, 3, 2], vec![0]],
            &[vec![3, 0, 2], vec![1]],
            false,
            1,
            2,
        )?,
        ob_make_world(
            &tokenizer,
            &names,
            301,
            501,
            &[vec![2, 0, 3], vec![1]],
            &[vec![3, 1, 2], vec![0]],
            false,
            2,
            0,
        )?,
        ob_make_world(
            &tokenizer,
            &names,
            302,
            502,
            &[vec![2, 1, 0], vec![3]],
            &[vec![1, 3, 0], vec![2]],
            true,
            0,
            3,
        )?,
    ];

    let (dev_clauses, dev_labels) = ob_training_set(&tokenizer, &names, &dev_worlds)?;
    let (cat_model, cat_fit) = fit_observed_text_model(&dev_clauses, &dev_labels, false)
        .map_err(|e| format!("fit categorical: {e}"))?;
    let (h4_model, h4_fit) = fit_observed_text_model(&dev_clauses, &dev_labels, true)
        .map_err(|e| format!("fit h4: {e}"))?;
    let cat_bytes = cat_model.to_bytes().map_err(|e| format!("{e}"))?;
    let h4_bytes = h4_model.to_bytes().map_err(|e| format!("{e}"))?;
    write_checked(
        &root,
        "artifacts/ordinary_forms_categorical_order.json",
        &cat_bytes,
    )?;
    write_checked(&root, "artifacts/ordinary_forms_h4_hybrid.json", &h4_bytes)?;
    let cat_reloaded = ObservedTextModel::from_bytes(
        &std::fs::read(root.join("artifacts/ordinary_forms_categorical_order.json"))
            .map_err(|e| e.to_string())?,
        VOCAB,
    )
    .map_err(|e| format!("{e}"))?;
    let h4_reloaded = ObservedTextModel::from_bytes(
        &std::fs::read(root.join("artifacts/ordinary_forms_h4_hybrid.json"))
            .map_err(|e| e.to_string())?,
        VOCAB,
    )
    .map_err(|e| format!("{e}"))?;
    if cat_reloaded != cat_model || h4_reloaded != h4_model {
        return Err("model artifact reload mismatch".into());
    }

    // ---- open development evaluation selects the primary arm ----
    let mut arms = Vec::<serde_json::Value>::new();
    let mut all_rows = Vec::<serde_json::Value>::new();
    let mut dev_complete = [0usize; 2];
    for (arm_index, (arm, model)) in [("categorical_order", &cat_model), ("h4_hybrid", &h4_model)]
        .into_iter()
        .enumerate()
    {
        for w in &dev_worlds {
            let panel = ob_evaluate(
                &tokenizer,
                &names,
                model,
                w,
                &TextControl::Normal,
                "development",
                arm,
            )?;
            dev_complete[arm_index] += panel.complete;
            all_rows.extend(panel.rows);
        }
        arms.push(json!({"arm": arm, "split": "development",
            "complete": dev_complete[arm_index],
            "total": OB_PERSONS * 2 * dev_worlds.len(),
            "depth_correct": null}));
    }
    let primary_is_categorical = dev_complete[0] >= dev_complete[1];
    let primary_arm = if primary_is_categorical {
        "categorical_order"
    } else {
        "h4_hybrid"
    };

    // ---- exposed regression (design-informed) and the final fresh draw, both arms ----
    let exposed_panel: Vec<&ObWorld> = exposed_worlds.iter().chain(heldout_worlds.iter()).collect();
    let mut depth_by_arm = [0usize; 2];
    for (split, worlds) in [
        ("exposed_regression", exposed_panel.clone()),
        ("final", final_worlds.iter().collect::<Vec<&ObWorld>>()),
    ] {
        for (arm_index, (arm, model)) in
            [("categorical_order", &cat_model), ("h4_hybrid", &h4_model)]
                .into_iter()
                .enumerate()
        {
            let mut complete = 0usize;
            let mut total = 0usize;
            let mut depth_ok = 0usize;
            for w in &worlds {
                let panel = ob_evaluate(
                    &tokenizer,
                    &names,
                    model,
                    w,
                    &TextControl::Normal,
                    split,
                    arm,
                )?;
                complete += panel.complete;
                total += panel.total;
                depth_ok += panel.depth_ok;
                all_rows.extend(panel.rows);
            }
            if split == "exposed_regression" {
                depth_by_arm[arm_index] += depth_ok;
            }
            arms.push(json!({"arm": arm, "split": split, "complete": complete,
                "total": total, "depth_correct": depth_ok}));
        }
    }

    let primary_model = if primary_is_categorical {
        &cat_model
    } else {
        &h4_model
    };

    // ---- controls on the exposed regression panel ----
    let membership_entities: Vec<Vec<u32>> = names
        .persons
        .iter()
        .map(|p| tokenizer.encode(&format!(" {p}")))
        .collect();
    let mut controls = Vec::<serde_json::Value>::new();
    for (label, control) in [
        (
            "membership_continuation",
            TextControl::Membership {
                entities: membership_entities.clone(),
            },
        ),
        (
            "lexical_membership_continuation",
            TextControl::LexicalMembership {
                entities: names
                    .persons
                    .iter()
                    .map(|p| p.as_bytes().to_vec())
                    .collect(),
            },
        ),
        ("maximum_two_read", TextControl::ReadCap { max_reads: 2 }),
        ("reads_disabled", TextControl::ReadsDisabled),
    ] {
        let mut complete = 0usize;
        let mut total = 0usize;
        let mut depth_ok = 0usize;
        for w in &exposed_panel {
            let panel = ob_evaluate(
                &tokenizer,
                &names,
                primary_model,
                w,
                &control,
                "exposed_regression",
                label,
            )?;
            complete += panel.complete;
            total += panel.total;
            depth_ok += panel.depth_ok;
            all_rows.extend(panel.rows);
        }
        controls.push(
            json!({"arm": label, "split": "exposed_regression", "complete": complete,
            "total": total, "depth_correct": depth_ok}),
        );
    }

    // ---- interventions through the same loaded runtime ----
    let w0 = &exposed_worlds[0];
    let (q0, q0_label) = ob_question_labeled(&tokenizer, &names, 0, Goal::Office)?;
    let q0_key = names.persons[0].as_bytes();
    let base_expected = ob_expected(w0, q0_key, Goal::Office)?;
    let base = ob_serve(primary_model, w0, &q0, TextControl::Normal, true)?;
    let base_segments: Vec<u32> = base_expected["selected_segments"]
        .as_array()
        .map(|v| {
            v.iter()
                .filter_map(|n| n.as_u64().map(|x| x as u32))
                .collect()
        })
        .unwrap_or_default();
    let first_seg = *base_segments
        .first()
        .ok_or("missing first oracle segment")?;
    let terminal_seg = *base_segments.last().ok_or("missing terminal source")?;
    let last_redirect_seg = if base_segments.len() >= 2 {
        base_segments[base_segments.len() - 2]
    } else {
        first_seg
    };

    // Changed source: the terminal location of the same person is different.
    let terminal_object = w0
        .labels
        .iter()
        .find(|l| l.seg == terminal_seg)
        .and_then(|l| l.object)
        .map(|(s, l)| {
            tokenizer.decode_bytes(
                &w0.clauses
                    .iter()
                    .find(|c| c.seg == terminal_seg)
                    .map(|c| c.tokens[s as usize..s as usize + l].to_vec())
                    .unwrap_or_default(),
            )
        })
        .map(|b| String::from_utf8_lossy(&b).trim().to_string())
        .unwrap_or_default();
    let replacement_target = names
        .offices
        .iter()
        .chain(names.projects.iter())
        .find(|n| **n != terminal_object)
        .ok_or("no alternative target")?;
    let edited = ob_edit_object(w0, &tokenizer, terminal_seg, replacement_target)?;
    let edit_expected = ob_expected(&edited, q0_key, Goal::Office)?;
    let edit = ob_serve(primary_model, &edited, &q0, TextControl::Normal, true)?;

    // Required terminal fact removed: unresolved after depth-1 reads.
    let removed = ob_without_segment(w0, terminal_seg);
    let removed_expected = ob_expected(&removed, q0_key, Goal::Office)?;
    let absent = ob_serve(primary_model, &removed, &q0, TextControl::Normal, true)?;

    // Cycle: the last redirect points back at the query person.
    let cycle_world = ob_edit_object(w0, &tokenizer, last_redirect_seg, names.persons[0])?;
    let cycle_expected = ob_expected(&cycle_world, q0_key, Goal::Office)?;
    let cycle = ob_serve(primary_model, &cycle_world, &q0, TextControl::Normal, true)?;

    // Valid redirect to a different existing person: the same question follows a different path.
    let first_object = w0
        .labels
        .iter()
        .find(|l| l.seg == first_seg)
        .and_then(|l| l.object)
        .and_then(|(s, l)| {
            w0.clauses
                .iter()
                .find(|c| c.seg == first_seg)
                .map(|c| c.tokens[s as usize..s as usize + l].to_vec())
        })
        .map(|tokens| tokenizer.decode(&tokens).trim().to_string())
        .unwrap_or_default();
    let redirect_target = *names
        .persons
        .iter()
        .find(|p| {
            **p != first_object.as_str()
                && **p != names.persons[0]
                && w0.labels.iter().any(|l| {
                    l.goal == Goal::Office
                        && w0
                            .clauses
                            .iter()
                            .find(|c| c.seg == l.seg)
                            .and_then(|c| {
                                lexical_key(
                                    &c.text,
                                    &c.byte_lengths,
                                    l.subject.0 as usize,
                                    l.subject.1,
                                )
                            })
                            .as_deref()
                            == Some(p.as_bytes())
                })
        })
        .ok_or("no alternative office subject")?;
    let redirect_world = ob_edit_object(w0, &tokenizer, first_seg, redirect_target)?;
    let redirect_expected = ob_expected(&redirect_world, q0_key, Goal::Office)?;
    let redirect_outcome = ob_serve(
        primary_model,
        &redirect_world,
        &q0,
        TextControl::Normal,
        true,
    )?;

    // Request-goal change: the same person's other goal selects different evidence.
    let (q_project, _) = ob_question_labeled(&tokenizer, &names, 0, Goal::Project)?;
    let project_expected = ob_expected(w0, q0_key, Goal::Project)?;
    let project = ob_serve(primary_model, w0, &q_project, TextControl::Normal, true)?;

    // Subword-order perturbation: not semantic word-order evidence.
    let mut swapped = q0.clone();
    let n = swapped.tokens.len();
    swapped.tokens.swap(n - 1, n - 2);
    swapped.text = String::from_utf8(tokenizer.decode_bytes(&swapped.tokens))
        .map_err(|e| format!("perturbed input UTF-8: {e}"))?;
    swapped.byte_lengths = ob_byte_lengths(&tokenizer, &swapped.text, &swapped.tokens)?;
    let perturbation = match ob_serve(primary_model, w0, &swapped, TextControl::Normal, false) {
        Ok(out) => ob_outcome_record(&out),
        Err(e) => json!({"error": e}),
    };

    // ---- cross-BPE identity witness ----
    let clause_of = |seg: u32| w0.clauses.iter().find(|c| c.seg == seg);
    let label_of = |seg: u32| w0.labels.iter().find(|l| l.seg == seg);
    let mut joins = Vec::<serde_json::Value>::new();
    for pair in base_segments.windows(2) {
        let (prev, next) = (pair[0], pair[1]);
        let (Some(pc), Some(pl), Some(nc), Some(nl)) = (
            clause_of(prev),
            label_of(prev),
            clause_of(next),
            label_of(next),
        ) else {
            continue;
        };
        let Some((ps, plen)) = pl.object else {
            continue;
        };
        let prev_object_tokens = pc.tokens[ps as usize..ps as usize + plen].to_vec();
        let next_subject_tokens =
            nc.tokens[nl.subject.0 as usize..nl.subject.0 as usize + nl.subject.1].to_vec();
        let prev_key = lexical_key(&pc.text, &pc.byte_lengths, ps as usize, plen);
        let next_key = lexical_key(
            &nc.text,
            &nc.byte_lengths,
            nl.subject.0 as usize,
            nl.subject.1,
        );
        let keys_equal = prev_key.is_some() && prev_key == next_key;
        let different_bpe_forms = prev_object_tokens != next_subject_tokens;
        joins.push(json!({
            "from_segment": prev, "to_segment": next,
            "object_surface": prev_object_tokens,
            "subject_surface": next_subject_tokens,
            "object_key": prev_key.as_ref().map(|k| String::from_utf8_lossy(k).into_owned()),
            "subject_key": next_key.as_ref().map(|k| String::from_utf8_lossy(k).into_owned()),
            "keys_equal": keys_equal,
            "different_bpe_forms": different_bpe_forms,
        }));
    }
    let chain_cross_bpe = !joins.is_empty()
        && joins.iter().all(|j| j["keys_equal"] == json!(true))
        && joins
            .iter()
            .any(|j| j["different_bpe_forms"] == json!(true));

    let (first_clause, first_label) = (
        clause_of(first_seg).ok_or("missing first clause")?,
        label_of(first_seg).ok_or("missing first label")?,
    );
    let query_subject_tokens = q0.tokens
        [q0_label.subject.0 as usize..q0_label.subject.0 as usize + q0_label.subject.1]
        .to_vec();
    let source_subject_tokens = first_clause.tokens
        [first_label.subject.0 as usize..first_label.subject.0 as usize + first_label.subject.1]
        .to_vec();
    let query_key_value = lexical_key(
        &q0.text,
        &q0.byte_lengths,
        q0_label.subject.0 as usize,
        q0_label.subject.1,
    );
    let source_key_value = lexical_key(
        &first_clause.text,
        &first_clause.byte_lengths,
        first_label.subject.0 as usize,
        first_label.subject.1,
    );
    let query_keys_equal = query_key_value.is_some() && query_key_value == source_key_value;
    let query_forms_differ = query_subject_tokens != source_subject_tokens;
    let query_cross_bpe = query_keys_equal && query_forms_differ;
    let identity = json!({
        "convention": "Exact original bytes with exterior ASCII whitespace excluded; case and interior bytes preserved. Surface extents remain the provenance and emission source.",
        "query_join": {
            "query_subject_surface": query_subject_tokens,
            "source_subject_surface": source_subject_tokens,
            "query_key": query_key_value.as_ref().map(|k| String::from_utf8_lossy(k).into_owned()),
            "source_key": source_key_value.as_ref().map(|k| String::from_utf8_lossy(k).into_owned()),
            "keys_equal": query_keys_equal,
            "different_bpe_forms": query_forms_differ,
        },
        "chain_joins": joins,
        "chain_cross_bpe": chain_cross_bpe,
    });

    let goal_invariant = |out: &ObOutcome| {
        out.events
            .iter()
            .all(|e| e["before"]["goal"] == e["after"]["goal"])
    };
    let checks = json!({
        "base_matches": ob_check_oracle(&base, &base_expected),
        "changed_payload_same_path": ob_check_oracle(&edit, &edit_expected)
            && edit.terminal == RelAction::Stop
            && edit.emitted != base.emitted
            && edit.selected == base.selected,
        "valid_redirect_changes_path": ob_check_oracle(&redirect_outcome, &redirect_expected)
            && redirect_outcome.terminal == RelAction::Stop
            && redirect_outcome.selected != base.selected,
        "later_absence_matches": ob_check_oracle(&absent, &removed_expected)
            && base.reads >= 1
            && absent.reads + 1 == base.reads,
        "cycle_matches": ob_check_oracle(&cycle, &cycle_expected),
        "goal_change_matches": ob_check_oracle(&project, &project_expected)
            && project.selected != base.selected,
        "cross_bpe_identity": query_cross_bpe && chain_cross_bpe,
        "goal_invariant": goal_invariant(&base) && goal_invariant(&edit),
    });
    let interventions_all_expected = checks
        .as_object()
        .map(|m| m.values().all(|v| v == &json!(true)))
        .unwrap_or(false);
    let interventions = json!({
        "changed_terminal_source": {
            "segment": terminal_seg, "replacement": replacement_target,
            "before_clauses": w0.clauses, "after_clauses": edited.clauses,
            "before_expected": base_expected, "after_expected": edit_expected,
            "before": ob_outcome_record(&base), "after": ob_outcome_record(&edit),
        },
        "required_terminal_fact_removed": {
            "removed_segment": terminal_seg, "clauses": removed.clauses,
            "expected": removed_expected, "outcome": ob_outcome_record(&absent),
        },
        "valid_redirect_to_existing_person": {
            "edited_segment": first_seg, "replacement": redirect_target,
            "clauses": redirect_world.clauses, "expected": redirect_expected,
            "outcome": ob_outcome_record(&redirect_outcome),
        },
        "cycle": {
            "edited_segment": last_redirect_seg, "clauses": cycle_world.clauses,
            "expected": cycle_expected, "outcome": ob_outcome_record(&cycle),
        },
        "request_goal_change": {
            "question": q_project, "expected": project_expected, "outcome": ob_outcome_record(&project),
        },
        "subword_order_perturbation": {
            "question": swapped, "decoded": tokenizer.decode(&swapped.tokens), "outcome": perturbation,
            "scope": "Perturbation only; no claim of semantic word-order generalization",
        },
        "checks": checks,
    });

    write_json(
        &root,
        "worlds.json",
        &json!({
            "development": dev_worlds.iter().map(|w| json!({"id": w.id, "version": w.version, "clauses": w.clauses, "labels_for_evaluation_only": w.labels})).collect::<Vec<_>>(),
            "exposed_regression": exposed_worlds.iter().map(|w| json!({"id": w.id, "version": w.version, "clauses": w.clauses, "labels_for_evaluation_only": w.labels})).collect::<Vec<_>>(),
            "exposed_regression_second_draw": heldout_worlds.iter().map(|w| json!({"id": w.id, "version": w.version, "clauses": w.clauses, "labels_for_evaluation_only": w.labels})).collect::<Vec<_>>(),
            "final": final_worlds.iter().map(|w| json!({"id": w.id, "version": w.version, "clauses": w.clauses, "labels_for_evaluation_only": w.labels})).collect::<Vec<_>>(),
            "fit_clauses": dev_clauses,
            "fit_labels": dev_labels,
            "membership_entities": membership_entities,
            "lexical_membership_entities": names.persons,
            "forms": {
                "assertion_styles": ["{P}'s office is {T}", "{T} is {P}'s office", "{P}'s office is {T} downtown"],
                "redirect": "{P} office follows {Q}",
                "question": "What is {P}'s office?",
                "note": "Ordinary forms realized against the pinned tokenizer; declared spans are located by exact token-byte offsets and every boundary must be a real token boundary."
            }
        }),
    )?;

    write_checked(
        &root,
        "rows.jsonl",
        all_rows
            .iter()
            .map(|r| serde_json::to_string(r).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
            .as_bytes(),
    )?;

    let result = json!({
        "schema": "uor-r4.ordinary-form-argument-binding/1",
        "base_revision": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
        "running_source": {
            "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
            "git_dirty": std::env::var("UOR_GIT_DIRTY").unwrap_or_else(|_| "unknown".into()),
            "executable_sha256": executable_sha256,
            "source_files": source_files,
        },
        "inputs": {"E_sha256": E_SHA, "S_sha256": S_SHA, "tokenizer_derived": DERIVED_SHA,
            "group_digest": group_digest, "f_bits": parent.cfg.f_bits},
        "task": {
            "serving_input": "readable ordinary-form statement clauses and an observed interrogative clause, both encoded by the pinned BPE tokenizer; no gold role, span, goal, pointer, depth or target",
            "vocabulary": {"persons": names.persons, "offices": names.offices, "projects": names.projects,
                "office_assert_cue": " office", "office_redirect_cue": names.cue(Goal::Office, true),
                "project_assert_cue": " project", "project_redirect_cue": names.cue(Goal::Project, true)},
            "eos": OB_EOS,
            "dev_worlds": dev_worlds.iter().map(|w| w.texts.clone()).collect::<Vec<_>>(),
            "exposed_worlds": exposed_worlds.iter().map(|w| w.texts.clone()).collect::<Vec<_>>(),
            "exposed_regression_second_draw": heldout_worlds.iter().map(|w| w.texts.clone()).collect::<Vec<_>>(),
            "final_worlds": final_worlds.iter().map(|w| w.texts.clone()).collect::<Vec<_>>(),
        },
        "learning": {"categorical_order": cat_fit, "h4_hybrid": h4_fit},
        "primary_arm": primary_arm,
        "primary_arm_selected_on": "development complete answers only; ties keep the order-aware categorical comparator. The final composition panel first appeared after fitting/design selection in historical attempt7; subsequent executions are exposed replays, not new independent final draws.",
        "development_complete": {"categorical_order": dev_complete[0], "h4_hybrid": dev_complete[1]},
        "arms": arms,
        "controls": controls,
        "identity": identity,
        "interventions": interventions,
        "interventions_all_expected": interventions_all_expected,
        "all_arm_rows": all_rows.len(),
        "exposed_depth_by_arm": {"categorical_order": depth_by_arm[0], "h4_hybrid": depth_by_arm[1]},
        "scope": "authored ordinary readable text over a small memory domain; declared bounded span proposals and a same-question causal session. This corrected execution replays the exposed final composition panel. Not broad language understanding, general reasoning or frontier capability. Historical membership uses token identity; lexical_membership_continuation uses the same exact lexical boundary as the primary. Energy UNAVAILABLE; whole-path D0-b not claimed.",
        "elapsed_s": started.elapsed().as_secs_f64(),
    });
    write_json(&root, "result.json", &result)?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    if !interventions_all_expected {
        return Err(format!("sealed intervention contract failure: {checks}"));
    }
    let find = |arm: &str, split: &str| -> (u64, u64) {
        arms.iter()
            .find(|a| a["arm"] == json!(arm) && a["split"] == json!(split))
            .map(|a| {
                (
                    a["complete"].as_u64().unwrap_or(0),
                    a["total"].as_u64().unwrap_or(0),
                )
            })
            .unwrap_or((0, 0))
    };
    let (cc, ct) = find("categorical_order", "development");
    let (hc, ht) = find("h4_hybrid", "development");
    let (ec, et) = find("categorical_order", "exposed_regression");
    let (xe, xt) = find("h4_hybrid", "exposed_regression");
    let (fc, ft) = find("categorical_order", "final");
    let (gf, gt) = find("h4_hybrid", "final");
    println!(
        "ordinary-form-binding: dev cat {cc}/{ct} h4 {hc}/{ht} | exposed cat {ec}/{et} h4 {xe}/{xt} | final cat {fc}/{ft} h4 {gf}/{gt} | primary {primary_arm} | spans {} -> {} | cue-roles {} of {} | seg-pot {} role-w {} h4-moves {} | sealed {} unlisted | {:.1}s",
        cat_fit.span_initial_correct,
        cat_fit.span_final_correct,
        cat_fit.cue_role_correct,
        cat_fit.clauses,
        cat_fit.segment_potentials,
        cat_fit.role_weights,
        h4_fit.h4_moves,
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Scoped correction memory: learned ingestion intent over exact versioned storage
// ---------------------------------------------------------------------------

const SCM_EOS: u32 = u32::MAX - 1;

/// Development and final entity/value vocabularies. The final population uses strings that never
/// appear in the fitted supervision, so lexical transfer is measured rather than assumed.
const SCM_DEV_ENTITIES: [&str; 8] = [
    "Mara", "Ivo", "Cedar", "Oren", "Bo", "Odette", "Wren", "Casper",
];
const SCM_DEV_OFFICES: [&str; 5] = ["Fen", "Office Park", "Cedar Annex", "Bay", "Vega"];
const SCM_DEV_PROJECTS: [&str; 5] = ["Atlas", "Project Bay", "Lumen", "Tarn", "Io"];
/// Panel whose failures informed the training-recipe change; reported as exposed.
const SCM_EXPOSED_ENTITIES: [&str; 4] = ["Ova", "Nia", "Rin", "Tessa"];
const SCM_EXPOSED_OFFICES: [&str; 3] = ["Bramble", "Quarry", "Vale"];
const SCM_EXPOSED_PROJECTS: [&str; 3] = ["Marsh", "Harbor", "Ledge"];
/// Fresh final population drawn after the change: disjoint entities and values.
const SCM_FINAL_ENTITIES: [&str; 4] = ["Una", "Pia", "Soren", "Kestrel"];
const SCM_FINAL_OFFICES: [&str; 3] = ["Cobalt", "Mica", "Ridge"];
const SCM_FINAL_PROJECTS: [&str; 3] = ["Delta", "Basalt", "Dune"];

/// The authored clause forms. Assertions, explicit corrections, declared nonasserting statements and
/// three temporal question forms, in both relations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ScmForm {
    AssertOffice,
    CorrectOffice,
    NegateOffice,
    RedirectOffice,
    AssertProject,
    CorrectProject,
    NegateProject,
    RedirectProject,
    AskCurrentOffice,
    AskPreviousOffice,
    AskInitialOffice,
    AskCurrentProject,
    AskPreviousProject,
    AskInitialProject,
}

const SCM_STATEMENT_FORMS: [ScmForm; 8] = [
    ScmForm::AssertOffice,
    ScmForm::CorrectOffice,
    ScmForm::NegateOffice,
    ScmForm::RedirectOffice,
    ScmForm::AssertProject,
    ScmForm::CorrectProject,
    ScmForm::NegateProject,
    ScmForm::RedirectProject,
];

const SCM_QUESTION_FORMS: [ScmForm; 6] = [
    ScmForm::AskCurrentOffice,
    ScmForm::AskPreviousOffice,
    ScmForm::AskInitialOffice,
    ScmForm::AskCurrentProject,
    ScmForm::AskPreviousProject,
    ScmForm::AskInitialProject,
];

impl ScmForm {
    fn relation(self) -> u8 {
        match self {
            ScmForm::AssertOffice
            | ScmForm::CorrectOffice
            | ScmForm::NegateOffice
            | ScmForm::RedirectOffice
            | ScmForm::AskCurrentOffice
            | ScmForm::AskPreviousOffice
            | ScmForm::AskInitialOffice => 0,
            _ => 1,
        }
    }
    fn is_question(self) -> bool {
        SCM_QUESTION_FORMS.contains(&self)
    }
    fn is_redirect(self) -> bool {
        matches!(self, ScmForm::RedirectOffice | ScmForm::RedirectProject)
    }
    fn is_negation(self) -> bool {
        matches!(self, ScmForm::NegateOffice | ScmForm::NegateProject)
    }
    fn is_correction(self) -> bool {
        matches!(self, ScmForm::CorrectOffice | ScmForm::CorrectProject)
    }
    fn history(self) -> Option<uor_r4_core::native_geometric::learner::scoped_memory::HistoryView> {
        use uor_r4_core::native_geometric::learner::scoped_memory::HistoryView;
        match self {
            ScmForm::AskCurrentOffice | ScmForm::AskCurrentProject => Some(HistoryView::Current),
            ScmForm::AskPreviousOffice | ScmForm::AskPreviousProject => {
                Some(HistoryView::PreviousAssertion)
            }
            ScmForm::AskInitialOffice | ScmForm::AskInitialProject => Some(HistoryView::Initial),
            _ => None,
        }
    }
    fn declared_statement_intent(self) -> Option<usize> {
        use uor_r4_core::native_geometric::learner::scoped_memory::{
            STMT_ASSERT, STMT_CORRECT, STMT_NONASSERTING,
        };
        match self {
            // A redirect is a bare assertion whose value is itself an entity key.
            ScmForm::AssertOffice
            | ScmForm::AssertProject
            | ScmForm::RedirectOffice
            | ScmForm::RedirectProject => Some(STMT_ASSERT),
            ScmForm::CorrectOffice | ScmForm::CorrectProject => Some(STMT_CORRECT),
            ScmForm::NegateOffice | ScmForm::NegateProject => Some(STMT_NONASSERTING),
            _ => None,
        }
    }
    fn declared_question_intent(self) -> Option<usize> {
        use uor_r4_core::native_geometric::learner::scoped_memory::{
            ASK_CURRENT, ASK_INITIAL, ASK_PREVIOUS,
        };
        match self {
            ScmForm::AskCurrentOffice | ScmForm::AskCurrentProject => Some(ASK_CURRENT),
            ScmForm::AskPreviousOffice | ScmForm::AskPreviousProject => Some(ASK_PREVIOUS),
            ScmForm::AskInitialOffice | ScmForm::AskInitialProject => Some(ASK_INITIAL),
            _ => None,
        }
    }
    fn noun(self) -> &'static str {
        if self.relation() == 0 {
            "office"
        } else {
            "project"
        }
    }
    /// The role the retained binder must learn: relation plus whether the value is an entity key.
    fn role(self) -> usize {
        match (self.relation(), self.is_redirect()) {
            (0, false) => 0,
            (0, true) => 1,
            (_, false) => 2,
            (_, true) => 3,
        }
    }
    fn goal(self) -> Goal {
        if self.relation() == 0 {
            Goal::Office
        } else {
            Goal::Project
        }
    }
    fn action(self) -> RelAction {
        if self.is_redirect() {
            RelAction::Continue
        } else {
            RelAction::Emit
        }
    }
}

/// Realize one authored form as the retained part-based clause. The cue span carries the relation and
/// the statement/request intent, so the learned intent table reads it from the same observation.
fn scm_ob_form(form: ScmForm, entity: &str, value: Option<&str>) -> ObForm {
    let noun = form.noun();
    let object = |v: Option<&str>| (Some(ObPart::Object), format!(" {}", v.unwrap_or("")));
    let parts = match form {
        ScmForm::AssertOffice | ScmForm::AssertProject => vec![
            (Some(ObPart::Subject), entity.to_string()),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), format!(" {noun}")),
            (None, " is".to_string()),
            object(value),
        ],
        // Correction and nonassertion cues deliberately contain no copula: a cue ending in " is" would
        // share its prefix with " is <value>", letting it absorb the copula and the value's first token
        // on unseen values. Every cue here ends immediately before the object.
        ScmForm::CorrectOffice | ScmForm::CorrectProject => vec![
            (Some(ObPart::Subject), entity.to_string()),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), format!(" {noun} became")),
            object(value),
        ],
        ScmForm::NegateOffice | ScmForm::NegateProject => vec![
            (Some(ObPart::Subject), entity.to_string()),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), format!(" {noun} might be")),
            object(value),
        ],
        ScmForm::RedirectOffice | ScmForm::RedirectProject => vec![
            (Some(ObPart::Subject), entity.to_string()),
            (Some(ObPart::Cue), format!(" {noun} follows")),
            object(value),
        ],
        ScmForm::AskCurrentOffice | ScmForm::AskCurrentProject => vec![
            (None, "What is".to_string()),
            (Some(ObPart::Subject), format!(" {entity}")),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), format!(" {noun}")),
            (None, "?".to_string()),
        ],
        ScmForm::AskPreviousOffice | ScmForm::AskPreviousProject => vec![
            (None, "What was".to_string()),
            (Some(ObPart::Subject), format!(" {entity}")),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), format!(" {noun} before")),
            (None, "?".to_string()),
        ],
        ScmForm::AskInitialOffice | ScmForm::AskInitialProject => vec![
            (None, "What was".to_string()),
            (Some(ObPart::Subject), format!(" {entity}")),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), format!(" {noun} originally")),
            (None, "?".to_string()),
        ],
    };
    ObForm {
        parts,
        role: form.role(),
        goal: form.goal(),
        action: form.action(),
    }
}

/// One authored operation in an interaction script. The runtime is never told which kind it is; the
/// learned observation and intent decide whether anything is written.
#[derive(Clone, Copy, Debug)]
enum ScmOp {
    Ingest {
        scope: u8,
        form: ScmForm,
        entity: &'static str,
        value: &'static str,
    },
    Ask {
        scope: u8,
        form: ScmForm,
        entity: &'static str,
    },
    /// An exact historical view that is not reachable from learned question intent.
    AskView {
        scope: u8,
        entity: &'static str,
        relation: u8,
        history: uor_r4_core::native_geometric::learner::scoped_memory::HistoryView,
    },
}

const SCM_SCOPES: [&str; 2] = ["alpha", "beta"];

/// Expected answer under the declared semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
enum ScmExpected {
    Complete { value: String, hops: u8 },
    Unresolved,
    NoHistory,
    Evicted,
    Cycle,
    Exhausted,
}

#[derive(Clone, Debug)]
struct ScmRefRecord {
    value: String,
    continues: bool,
    conflict: bool,
    commit: u64,
    evicted: bool,
}

/// Exact declared-semantics reference. It diagnoses store bookkeeping and harness agreement; it is
/// **not** evidence of learned intent, which is counted separately from the loaded path.
struct ScmOracle {
    chains: std::collections::BTreeMap<(u8, String, u8), Vec<ScmRefRecord>>,
    commit: u64,
    capacity: usize,
}

impl ScmOracle {
    fn new(capacity: usize) -> Self {
        ScmOracle {
            chains: std::collections::BTreeMap::new(),
            commit: 0,
            capacity: capacity.max(1),
        }
    }

    /// Apply the declared semantics of one asserting form.
    fn declared(&mut self, scope: u8, form: ScmForm, entity: &str, value: &str) -> bool {
        if form.is_question() || form.is_negation() {
            return false;
        }
        self.commit += 1;
        let key = (scope, entity.to_string(), form.relation());
        let chain = self.chains.entry(key).or_default();
        let conflict = chain
            .last()
            .filter(|head| !head.evicted)
            .is_some_and(|head| !form.is_correction() && head.value != value);
        chain.push(ScmRefRecord {
            value: value.to_string(),
            continues: form.is_redirect(),
            conflict,
            commit: self.commit,
            evicted: false,
        });
        // Independent age/rank rule: only the newest capacity revisions retain payloads.
        let retained_start = chain.len().saturating_sub(self.capacity);
        for (index, record) in chain.iter_mut().enumerate() {
            record.evicted = index < retained_start;
        }
        true
    }

    fn answer(
        &self,
        scope: u8,
        entity: &str,
        relation: u8,
        history: uor_r4_core::native_geometric::learner::scoped_memory::HistoryView,
        view: u64,
    ) -> ScmExpected {
        use uor_r4_core::native_geometric::learner::scoped_memory::HistoryView;
        use uor_r4_core::native_geometric::learner::scoped_memory::SCOPED_MAX_HOPS;
        let mut current = entity.to_string();
        let mut visited: Vec<String> = Vec::new();
        let mut hop = 0u8;
        loop {
            if hop >= SCOPED_MAX_HOPS {
                return ScmExpected::Exhausted;
            }
            if visited.contains(&current) {
                return ScmExpected::Cycle;
            }
            let hop_history = if hop == 0 {
                history
            } else {
                HistoryView::Current
            };
            let Some(chain) = self.chains.get(&(scope, current.clone(), relation)) else {
                return ScmExpected::Unresolved;
            };
            let visible: Vec<&ScmRefRecord> = chain.iter().filter(|r| r.commit <= view).collect();
            let Some(head) = visible.last() else {
                return ScmExpected::Unresolved;
            };
            let selected = match hop_history {
                HistoryView::Current => *head,
                HistoryView::Initial => visible[0],
                HistoryView::PreviousAssertion => {
                    if visible.len() < 2 {
                        return ScmExpected::NoHistory;
                    }
                    visible[visible.len() - 2]
                }
                HistoryView::PreviousDistinctValue => {
                    let mut found = None;
                    for record in visible.iter().rev().skip(1) {
                        if record.evicted {
                            return ScmExpected::Evicted;
                        }
                        if record.value != head.value {
                            found = Some(*record);
                            break;
                        }
                    }
                    match found {
                        Some(record) => record,
                        None => return ScmExpected::NoHistory,
                    }
                }
            };
            if selected.evicted {
                return ScmExpected::Evicted;
            }
            if selected.continues {
                visited.push(current);
                current = selected.value.clone();
                hop += 1;
                continue;
            }
            return ScmExpected::Complete {
                value: selected.value.clone(),
                hops: hop + 1,
            };
        }
    }
}

struct ScmSupervision {
    label: ClauseLabel,
    question: bool,
    intent: usize,
}

/// Development supervision: every form in both relations, with disjoint entity/value pools.
fn scm_development_supervision(
    tokenizer: &HfBpeTokenizer,
) -> Result<(Vec<Clause>, Vec<ScmSupervision>), String> {
    let mut clauses = Vec::new();
    let mut supervision = Vec::new();
    let mut seg = 0u32;
    for (index, form) in SCM_STATEMENT_FORMS.iter().enumerate() {
        for k in 0..6usize {
            let entity = SCM_DEV_ENTITIES[(index + k) % SCM_DEV_ENTITIES.len()];
            let value = if form.is_redirect() {
                SCM_DEV_ENTITIES[(index + k + 1) % SCM_DEV_ENTITIES.len()]
            } else if form.relation() == 0 {
                SCM_DEV_OFFICES[(index + k) % SCM_DEV_OFFICES.len()]
            } else {
                SCM_DEV_PROJECTS[(index + k) % SCM_DEV_PROJECTS.len()]
            };
            let (clause, label) =
                ob_build_clause(tokenizer, seg, &scm_ob_form(*form, entity, Some(value)))?;
            clauses.push(clause);
            supervision.push(ScmSupervision {
                label,
                question: false,
                intent: form
                    .declared_statement_intent()
                    .ok_or("statement form has no declared intent")?,
            });
            seg += 1;
        }
    }
    for (index, form) in SCM_QUESTION_FORMS.iter().enumerate() {
        for k in 0..6usize {
            let entity = SCM_DEV_ENTITIES[(index + k) % SCM_DEV_ENTITIES.len()];
            let (clause, label) =
                ob_build_clause(tokenizer, seg, &scm_ob_form(*form, entity, None))?;
            clauses.push(clause);
            supervision.push(ScmSupervision {
                label,
                question: true,
                intent: form
                    .declared_question_intent()
                    .ok_or("question form has no declared intent")?,
            });
            seg += 1;
        }
    }
    Ok((clauses, supervision))
}

/// Open development interaction script. Covers scope independence, relation independence,
/// nonasserting inputs, same-value reassertion, a conflicting bare assertion, an explicit correction,
/// the three temporal questions, the exact previous-distinct query, a dependent read and a missing fact.
fn scm_development_script() -> Vec<ScmOp> {
    use ScmForm::*;
    vec![
        // Independent scopes: identical entity bytes, different values.
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Mara", value: "Fen" },
        ScmOp::Ingest { scope: 1, form: AssertOffice, entity: "Mara", value: "Office Park" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Mara" },
        ScmOp::Ask { scope: 1, form: AskCurrentOffice, entity: "Mara" },
        // Relation independence: correcting the office must not change the project.
        ScmOp::Ingest { scope: 0, form: AssertProject, entity: "Mara", value: "Atlas" },
        ScmOp::Ingest { scope: 0, form: CorrectOffice, entity: "Mara", value: "Cedar Annex" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Mara" },
        ScmOp::Ask { scope: 0, form: AskCurrentProject, entity: "Mara" },
        // Declared nonasserting inputs write nothing.
        ScmOp::Ingest { scope: 0, form: NegateOffice, entity: "Mara", value: "Fen" },
        // A question ingested as if it were input must also write nothing.
        ScmOp::Ingest { scope: 0, form: AskCurrentOffice, entity: "Mara", value: "" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Mara" },
        // Same-value reassertion, conflicting bare assertion, explicit correction, history.
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Ivo", value: "Fen" },
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Ivo", value: "Fen" },
        ScmOp::Ask { scope: 0, form: AskPreviousOffice, entity: "Ivo" },
        ScmOp::AskView { scope: 0, entity: "Ivo", relation: 0, history: uor_r4_core::native_geometric::learner::scoped_memory::HistoryView::PreviousDistinctValue },
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Ivo", value: "Office Park" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Ivo" },
        ScmOp::Ask { scope: 0, form: AskInitialOffice, entity: "Ivo" },
        ScmOp::Ask { scope: 0, form: AskPreviousOffice, entity: "Ivo" },
        ScmOp::AskView { scope: 0, entity: "Ivo", relation: 0, history: uor_r4_core::native_geometric::learner::scoped_memory::HistoryView::PreviousDistinctValue },
        ScmOp::Ingest { scope: 0, form: CorrectOffice, entity: "Ivo", value: "Cedar Annex" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Ivo" },
        ScmOp::Ask { scope: 0, form: AskInitialOffice, entity: "Ivo" },
        // Missing fact before any dependent read is written.
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Oren" },
        // Dependent read: Cedar's office follows Oren, and Oren's office is Fen.
        ScmOp::Ingest { scope: 0, form: RedirectOffice, entity: "Cedar", value: "Oren" },
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Oren", value: "Fen" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Cedar" },
        // A correction of the dependent terminal changes the first entity's answer too.
        ScmOp::Ingest { scope: 0, form: CorrectOffice, entity: "Oren", value: "Cedar Annex" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Cedar" },
    ]
}

/// Former final population used to diagnose the boundary; now exposed.
fn scm_exposed_script() -> Vec<ScmOp> {
    use ScmForm::*;
    vec![
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Ova", value: "Bramble" },
        ScmOp::Ingest { scope: 0, form: CorrectOffice, entity: "Ova", value: "Quarry" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Ova" },
        ScmOp::Ask { scope: 0, form: AskPreviousOffice, entity: "Ova" },
        ScmOp::Ask { scope: 0, form: AskInitialOffice, entity: "Ova" },
        // Held out: an unrelated-scope update must not move the answer.
        ScmOp::Ingest { scope: 1, form: AssertOffice, entity: "Ova", value: "Vale" },
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Ova", value: "Quarry" },
        ScmOp::Ask { scope: 1, form: AskCurrentOffice, entity: "Ova" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Ova" },
        ScmOp::AskView { scope: 0, entity: "Ova", relation: 0, history: uor_r4_core::native_geometric::learner::scoped_memory::HistoryView::PreviousDistinctValue },
        // Held out: a dependent chain whose intermediate link is corrected.
        ScmOp::Ingest { scope: 0, form: RedirectOffice, entity: "Nia", value: "Rin" },
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Rin", value: "Marsh" },
        ScmOp::Ingest { scope: 0, form: CorrectOffice, entity: "Rin", value: "Harbor" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Nia" },
        // Held out: a dependent chain whose terminal is corrected, and a project relation alongside.
        ScmOp::Ingest { scope: 0, form: RedirectOffice, entity: "Tessa", value: "Ova" },
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Ova", value: "Harbor" },
        ScmOp::Ingest { scope: 0, form: CorrectOffice, entity: "Ova", value: "Ledge" },
        ScmOp::Ingest { scope: 0, form: AssertProject, entity: "Ova", value: "Marsh" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Tessa" },
        ScmOp::Ask { scope: 0, form: AskCurrentProject, entity: "Ova" },
        // Held out: a negation between two assertions must not disturb the current value.
        ScmOp::Ingest { scope: 0, form: NegateOffice, entity: "Tessa", value: "Vale" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Tessa" },
        ScmOp::Ask { scope: 0, form: AskInitialOffice, entity: "Ova" },
    ]
}

/// Fresh final population: new unseen entity and value strings, same held-out interaction structure.
fn scm_final_script() -> Vec<ScmOp> {
    use ScmForm::*;
    vec![
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Una", value: "Cobalt" },
        ScmOp::Ingest { scope: 0, form: CorrectOffice, entity: "Una", value: "Mica" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Una" },
        ScmOp::Ask { scope: 0, form: AskPreviousOffice, entity: "Una" },
        ScmOp::Ask { scope: 0, form: AskInitialOffice, entity: "Una" },
        ScmOp::Ingest { scope: 1, form: AssertOffice, entity: "Una", value: "Ridge" },
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Una", value: "Mica" },
        ScmOp::Ask { scope: 1, form: AskCurrentOffice, entity: "Una" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Una" },
        ScmOp::AskView { scope: 0, entity: "Una", relation: 0, history: uor_r4_core::native_geometric::learner::scoped_memory::HistoryView::PreviousDistinctValue },
        ScmOp::Ingest { scope: 0, form: RedirectOffice, entity: "Pia", value: "Soren" },
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Soren", value: "Delta" },
        ScmOp::Ingest { scope: 0, form: CorrectOffice, entity: "Soren", value: "Basalt" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Pia" },
        ScmOp::Ingest { scope: 0, form: RedirectOffice, entity: "Kestrel", value: "Una" },
        ScmOp::Ingest { scope: 0, form: AssertOffice, entity: "Una", value: "Basalt" },
        ScmOp::Ingest { scope: 0, form: CorrectOffice, entity: "Una", value: "Dune" },
        ScmOp::Ingest { scope: 0, form: AssertProject, entity: "Una", value: "Delta" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Kestrel" },
        ScmOp::Ask { scope: 0, form: AskCurrentProject, entity: "Una" },
        ScmOp::Ingest { scope: 0, form: NegateOffice, entity: "Kestrel", value: "Mica" },
        ScmOp::Ask { scope: 0, form: AskCurrentOffice, entity: "Kestrel" },
        ScmOp::Ask { scope: 0, form: AskInitialOffice, entity: "Una" },
    ]
}

fn scm_clause(
    tokenizer: &HfBpeTokenizer,
    seg: u32,
    form: ScmForm,
    entity: &str,
    value: Option<&str>,
) -> Result<(Clause, ClauseLabel), String> {
    ob_build_clause(tokenizer, seg, &scm_ob_form(form, entity, value))
}

/// The emitted answer as an exact lexical identity: exterior ASCII whitespace is excluded, exactly as
/// the store's identity policy does.
fn scm_emitted_key(tokenizer: &HfBpeTokenizer, session: &ScopedSession) -> String {
    let tokens: Vec<u32> = session
        .emitted
        .iter()
        .copied()
        .filter(|t| Some(*t) != session.eos)
        .collect();
    tokenizer
        .decode(&tokens)
        .trim_matches(|c: char| c.is_ascii_whitespace())
        .to_string()
}

fn scm_terminal_of(expected: &ScmExpected) -> ScopedTerminal {
    match expected {
        ScmExpected::Complete { .. } => ScopedTerminal::Complete,
        ScmExpected::Unresolved => ScopedTerminal::Unresolved,
        ScmExpected::NoHistory => ScopedTerminal::NoHistory,
        ScmExpected::Evicted => ScopedTerminal::Evicted,
        ScmExpected::Cycle => ScopedTerminal::Cycle,
        ScmExpected::Exhausted => ScopedTerminal::Exhausted,
    }
}

#[derive(Default)]
struct ScmReplay {
    questions: usize,
    complete: usize,
    branch_total: usize,
    branch_correct: usize,
    intent_total: usize,
    intent_correct: usize,
    nowrite_total: usize,
    nowrite_correct: usize,
    rows: Vec<serde_json::Value>,
    store: Vec<u8>,
}

/// Replay one interaction script through the loaded path under one control and declared capacity.
/// A fresh runtime and store are built every time, so controls are compared on identical inputs.
#[allow(clippy::too_many_arguments)]
fn scm_replay(
    tokenizer: &HfBpeTokenizer,
    model_bytes: &[u8],
    intent_bytes: &[u8],
    lineage: u64,
    control: MemoryControl,
    capacity: usize,
    script: &[ScmOp],
    script_name: &str,
    control_name: &str,
) -> Result<ScmReplay, String> {
    let memory = Memory::new(lineage, capacity);
    let mut runtime = ScopedMemoryRuntime::load(
        model_bytes,
        intent_bytes,
        memory,
        lineage,
        control,
        VOCAB,
        Some(SCM_EOS),
    )
    .map_err(|e| format!("scoped runtime load: {e}"))?;
    let mut oracle = ScmOracle::new(capacity);
    let mut out = ScmReplay::default();
    for (index, op) in script.iter().enumerate() {
        match *op {
            ScmOp::Ingest {
                scope,
                form,
                entity,
                value,
            } => {
                let (clause, _) = scm_clause(tokenizer, index as u32, form, entity, Some(value))?;
                let observed = runtime
                    .observe(&clause)
                    .map_err(|e| format!("observe {entity:?}: {e}"))?;
                out.branch_total += 1;
                if observed.is_question == form.is_question() {
                    out.branch_correct += 1;
                }
                let declared = if form.is_question() {
                    form.declared_question_intent()
                } else {
                    form.declared_statement_intent()
                };
                if let Some(declared) = declared {
                    out.intent_total += 1;
                    if observed.intent == declared {
                        out.intent_correct += 1;
                    }
                }
                let expected_write = !form.is_question() && !form.is_negation();
                let outcome = runtime
                    .ingest(&clause, SCM_SCOPES[scope as usize].as_bytes(), index as u64)
                    .map_err(|e| {
                        format!(
                            "ingest {entity:?} text={:?} is_q={} intent={} rel={} value={:?}: {e}",
                            clause.text,
                            observed.is_question,
                            observed.intent,
                            observed.relation,
                            observed
                                .value_key
                                .as_ref()
                                .map(|v| String::from_utf8_lossy(v).into_owned())
                        )
                    })?;
                let wrote = matches!(outcome, IngestOutcome::Wrote(_));
                if !expected_write {
                    out.nowrite_total += 1;
                    if !wrote {
                        out.nowrite_correct += 1;
                    }
                }
                let declared_wrote = oracle.declared(scope, form, entity, value);
                if declared_wrote != expected_write {
                    return Err(format!("harness/oracle disagreement on {entity:?}"));
                }
                out.rows.push(json!({
                    "script": script_name, "control": control_name, "turn": index,
                    "kind": "ingest", "scope": SCM_SCOPES[scope as usize], "text": clause.text,
                    "entity_key": String::from_utf8_lossy(&observed.entity_key).into_owned(),
                    "value_key": observed.value_key.as_ref().map(|v| String::from_utf8_lossy(v).into_owned()),
                    "learned_is_question": observed.is_question,
                    "declared_is_question": form.is_question(),
                    "learned_relation": observed.relation,
                    "declared_relation": form.relation(),
                    "learned_intent": observed.intent,
                    "declared_intent": declared,
                    "learned_continues": observed.continues,
                    "declared_continues": form.is_redirect(),
                    "expected_write": expected_write,
                    "wrote": wrote,
                    "outcome": outcome,
                }));
            }
            ScmOp::Ask {
                scope,
                form,
                entity,
            } => {
                let (clause, _) = scm_clause(tokenizer, index as u32, form, entity, None)?;
                let query_key = runtime
                    .observe(&clause)
                    .map_err(|e| format!("observe question {entity:?}: {e}"))?;
                let session = runtime
                    .ask(&clause, SCM_SCOPES[scope as usize].as_bytes())
                    .map_err(|e| format!("ask {entity:?}: {e}"))?;
                let view = session.view;
                let mut session = session;
                let effects = runtime
                    .run(&mut session)
                    .map_err(|e| format!("run {entity:?}: {e}"))?;
                let answer = scm_emitted_key(tokenizer, &session);
                // The expectation is taken at the *declared* commit snapshot, so a control that
                // writes nothing cannot silently agree with an oracle that also saw nothing.
                let oracle_view = oracle.commit;
                let expected = oracle.answer(
                    scope,
                    entity,
                    form.relation(),
                    form.history().ok_or("ask form has no history")?,
                    oracle_view,
                );
                out.questions += 1;
                let matched = match (&expected, session.terminal) {
                    (ScmExpected::Complete { value, hops }, Some(ScopedTerminal::Complete)) => {
                        // `hop` counts continuations, so the read count is one more than the hop.
                        answer == *value && session.hop + 1 == *hops
                    }
                    (_, Some(terminal)) => terminal == scm_terminal_of(&expected),
                    _ => false,
                };
                if matched {
                    out.complete += 1;
                }
                out.rows.push(json!({
                    "script": script_name, "control": control_name, "turn": index,
                    "kind": "ask", "scope": SCM_SCOPES[scope as usize], "text": clause.text,
                    "query_key": String::from_utf8_lossy(&query_key.entity_key).into_owned(),
                    "query_relation": query_key.relation,
                    "query_is_question": query_key.is_question,
                    "pin_view": view, "oracle_view": oracle_view,
                    "view_agrees": view == oracle_view, "answer": answer,
                    "expected": format!("{expected:?}"), "terminal": session.terminal,
                    "hop": session.hop, "matched": matched,
                    "effects": effects,
                    "final_frame": session,
                }));
            }
            ScmOp::AskView {
                scope,
                entity,
                relation,
                history,
            } => {
                let session = runtime
                    .ask_view(
                        SCM_SCOPES[scope as usize].as_bytes(),
                        relation,
                        entity.as_bytes(),
                        history,
                    )
                    .map_err(|e| format!("ask_view {entity:?}: {e}"))?;
                let view = session.view;
                let mut session = session;
                let effects = runtime
                    .run(&mut session)
                    .map_err(|e| format!("run view {entity:?}: {e}"))?;
                let answer = scm_emitted_key(tokenizer, &session);
                let oracle_view = oracle.commit;
                let expected = oracle.answer(scope, entity, relation, history, oracle_view);
                out.questions += 1;
                let matched = match (&expected, session.terminal) {
                    (ScmExpected::Complete { value, hops }, Some(ScopedTerminal::Complete)) => {
                        answer == *value && session.hop + 1 == *hops
                    }
                    (_, Some(terminal)) => terminal == scm_terminal_of(&expected),
                    _ => false,
                };
                if matched {
                    out.complete += 1;
                }
                out.rows.push(json!({
                    "script": script_name, "control": control_name, "turn": index,
                    "kind": "ask_view", "scope": SCM_SCOPES[scope as usize],
                    "text": format!("exact {history:?} of {entity:?} relation {relation}"),
                    "pin_view": view, "oracle_view": oracle_view,
                    "view_agrees": view == oracle_view, "answer": answer,
                    "expected": format!("{expected:?}"), "terminal": session.terminal,
                    "hop": session.hop, "matched": matched,
                    "effects": effects,
                    "final_frame": session,
                }));
            }
        }
    }
    out.store = runtime
        .store_bytes()
        .map_err(|e| format!("store bytes: {e}"))?;
    Ok(out)
}

fn scm_answer_questions(
    tokenizer: &HfBpeTokenizer,
    runtime: &ScopedMemoryRuntime,
    script: &[ScmOp],
) -> Result<Vec<serde_json::Value>, String> {
    let mut answers = Vec::new();
    for (index, op) in script.iter().enumerate() {
        match *op {
            ScmOp::Ask {
                scope,
                form,
                entity,
            } => {
                let (clause, _) = scm_clause(tokenizer, index as u32, form, entity, None)?;
                let mut session = runtime
                    .ask(&clause, SCM_SCOPES[scope as usize].as_bytes())
                    .map_err(|e| format!("ask {entity:?}: {e}"))?;
                runtime
                    .run(&mut session)
                    .map_err(|e| format!("run {entity:?}: {e}"))?;
                answers.push(json!({
                    "kind": "ask", "text": clause.text,
                    "answer": scm_emitted_key(tokenizer, &session),
                    "emitted": session.emitted,
                    "terminal": session.terminal, "hop": session.hop,
                }));
            }
            ScmOp::AskView {
                scope,
                entity,
                relation,
                history,
            } => {
                let mut session = runtime
                    .ask_view(
                        SCM_SCOPES[scope as usize].as_bytes(),
                        relation,
                        entity.as_bytes(),
                        history,
                    )
                    .map_err(|e| format!("ask_view {entity:?}: {e}"))?;
                runtime
                    .run(&mut session)
                    .map_err(|e| format!("run view {entity:?}: {e}"))?;
                answers.push(json!({
                    "kind": "ask_view",
                    "text": format!("exact {history:?} of {entity:?} relation {relation}"),
                    "answer": scm_emitted_key(tokenizer, &session),
                    "emitted": session.emitted,
                    "terminal": session.terminal, "hop": session.hop,
                }));
            }
            ScmOp::Ingest { .. } => {}
        }
    }
    Ok(answers)
}

/// Convert actual text into observed tokens and byte alignment, without semantic annotations.
fn scm_raw_clause(tokenizer: &HfBpeTokenizer, text: &str) -> Result<Clause, String> {
    let tokens = tokenizer.encode(text);
    if tokens.is_empty() || tokens.len() > OB_MAX_CLAUSE {
        return Err("raw clause outside declared token bound".into());
    }
    Ok(Clause {
        seg: 0,
        byte_lengths: ob_byte_lengths(tokenizer, text, &tokens)?,
        tokens,
        text: text.to_string(),
    })
}

fn scm_answer_json(session: &ScopedSession) -> serde_json::Value {
    json!({"emitted": session.emitted, "terminal": session.terminal, "hop": session.hop,
        "selected": session.captured.as_ref().map(|c| c.record_id)})
}

/// Fresh-process reload runs learned text interpretation, typed API controls separately, actual
/// in-flight restoration, and a learned post-restart write. Gold semantic fields never route asks.
fn scm_reload_check(dir: &std::path::Path) -> Result<ExitCode, String> {
    let model_bytes = std::fs::read(dir.join("model.json")).map_err(|e| e.to_string())?;
    let intent_bytes = std::fs::read(dir.join("intent.json")).map_err(|e| e.to_string())?;
    let store_bytes = std::fs::read(dir.join("store.json")).map_err(|e| e.to_string())?;
    let queries: Vec<serde_json::Value> = serde_json::from_slice(
        &std::fs::read(dir.join("queries.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let tb = std::fs::read(DEFAULT_TOKENIZER).map_err(|e| e.to_string())?;
    if sha256_hex(&tb) != TOKENIZER_SHA {
        return Err("reload tokenizer mismatch".into());
    }
    let tokenizer = derive_tokenizer(&tb, VOCAB).map_err(|e| e.to_string())?;
    let memory = Memory::from_bytes(&store_bytes).map_err(|e| e.to_string())?;
    let lineage = memory.lineage;
    let mut runtime = ScopedMemoryRuntime::load(
        &model_bytes,
        &intent_bytes,
        memory,
        lineage,
        MemoryControl::Normal,
        VOCAB,
        Some(SCM_EOS),
    )
    .map_err(|e| e.to_string())?;
    let required = |v: &serde_json::Value, key: &str| -> Result<String, String> {
        v[key]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("missing reload {key}"))
    };
    let mut out = Vec::new();
    for query in queries {
        let scope = required(&query, "scope")?;
        let mut session = match required(&query, "kind")?.as_str() {
            "ask" => {
                if query.get("entity").is_some()
                    || query.get("relation").is_some()
                    || query.get("history").is_some()
                {
                    return Err("semantic fields on raw ask".into());
                }
                let clause = scm_raw_clause(&tokenizer, &required(&query, "text")?)?;
                runtime
                    .ask(&clause, scope.as_bytes())
                    .map_err(|e| e.to_string())?
            }
            "ask_view" => {
                let relation = query["relation"]
                    .as_u64()
                    .filter(|r| *r <= u8::MAX as u64)
                    .ok_or("invalid typed relation")? as u8;
                let history: HistoryView =
                    serde_json::from_value(query["history"].clone()).map_err(|e| e.to_string())?;
                runtime
                    .ask_view(
                        scope.as_bytes(),
                        relation,
                        required(&query, "entity")?.as_bytes(),
                        history,
                    )
                    .map_err(|e| e.to_string())?
            }
            _ => return Err("unknown reload query kind".into()),
        };
        runtime.run(&mut session).map_err(|e| e.to_string())?;
        out.push(scm_answer_json(&session));
    }
    // One learned correction after restart, then continue all pre-write saved phases under their pin.
    let update = scm_raw_clause(&tokenizer, "Oren's office became Office Park")?;
    let before = runtime.revision();
    let outcome = runtime
        .ingest(&update, b"alpha", 1900)
        .map_err(|e| e.to_string())?;
    let wrote = matches!(outcome, IngestOutcome::Wrote(_)) && runtime.revision() == before + 1;
    let checkpoints: Vec<serde_json::Value> = serde_json::from_slice(
        &std::fs::read(dir.join("checkpoints.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut resumed = Vec::new();
    for checkpoint in checkpoints {
        let bytes = serde_json::to_vec(&checkpoint["session"]).map_err(|e| e.to_string())?;
        let mut session = runtime.restore(&bytes).map_err(|e| e.to_string())?;
        runtime.run(&mut session).map_err(|e| e.to_string())?;
        let answer = scm_answer_json(&session);
        resumed.push(json!({"phase": checkpoint["session"]["pending"],
            "matches": answer == checkpoint["expected"], "answer": answer}));
    }
    let question = scm_raw_clause(&tokenizer, "What is Cedar's office?")?;
    let mut session = runtime
        .ask(&question, b"alpha")
        .map_err(|e| e.to_string())?;
    runtime.run(&mut session).map_err(|e| e.to_string())?;
    let updated_answer = scm_emitted_key(&tokenizer, &session);
    println!(
        "{}",
        serde_json::to_string(&json!({"answers": out, "resumed": resumed,
        "post_restart": {"wrote": wrote, "answer": updated_answer, "frame": session},
        "learned_restart": wrote && updated_answer == "Office Park"}))
        .map_err(|e| e.to_string())?
    );
    Ok(ExitCode::SUCCESS)
}

fn scm_queries(script: &[ScmOp]) -> Vec<serde_json::Value> {
    script.iter().filter_map(|op| match op {
        ScmOp::Ask {scope, form, entity} => {
            let text: String = scm_ob_form(*form, entity, None).parts.into_iter().map(|(_,s)| s).collect();
            Some(json!({"kind":"ask", "scope": SCM_SCOPES[*scope as usize], "text": text}))
        }
        ScmOp::AskView {scope, entity, relation, history} => Some(json!({"kind":"ask_view",
            "scope":SCM_SCOPES[*scope as usize], "entity":entity,"relation":relation,"history":history})),
        ScmOp::Ingest {..} => None,
    }).collect()
}

#[cfg(test)]
mod scoped_runner_checks {
    use super::*;
    #[test]
    fn capacity_oracle_retains_only_latest_two_after_each_write() {
        let mut o = ScmOracle::new(2);
        for value in ["a", "b", "c", "d", "e"] {
            o.declared(0, ScmForm::CorrectOffice, "entity", value);
            let chain = &o.chains[&(0, "entity".to_string(), 0)];
            assert_eq!(
                chain.iter().filter(|r| !r.evicted).count(),
                chain.len().min(2)
            );
        }
        assert_eq!(
            o.answer(0, "entity", 0, HistoryView::Initial, o.commit),
            ScmExpected::Evicted
        );
        assert_eq!(
            o.answer(0, "entity", 0, HistoryView::PreviousAssertion, o.commit),
            ScmExpected::Complete {
                value: "d".into(),
                hops: 1
            }
        );
    }
    #[test]
    fn restart_text_questions_have_no_semantic_routing_fields() {
        let queries = scm_queries(&scm_development_script());
        assert_eq!(queries.iter().filter(|q| q["kind"] == "ask").count(), 14);
        assert_eq!(
            queries.iter().filter(|q| q["kind"] == "ask_view").count(),
            2
        );
        for q in queries.iter().filter(|q| q["kind"] == "ask") {
            assert!(q["text"].as_str().is_some());
            assert!(
                q.get("entity").is_none()
                    && q.get("relation").is_none()
                    && q.get("history").is_none()
            );
        }
    }
}

fn scm_run() -> Result<ExitCode, String> {
    let mut root = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/scoped-correction-memory-1",
    );
    let mut source_root: Option<PathBuf> = None;
    let mut reload_dir: Option<PathBuf> = None;
    {
        let mut a = std::env::args().skip(1);
        while let Some(k) = a.next() {
            match k.as_str() {
                "--root" => root = PathBuf::from(a.next().ok_or("--root value")?),
                "--source-root" => source_root = Some(PathBuf::from(a.next().ok_or("value")?)),
                "--reload-check" => reload_dir = Some(PathBuf::from(a.next().ok_or("value")?)),
                other if other.starts_with("--mode") => {}
                other => return Err(format!("unknown argument {other}")),
            }
        }
    }
    if let Some(dir) = reload_dir {
        return scm_reload_check(&dir);
    }
    claim(&root).map_err(|e| format!("claim {}: {e}", root.display()))?;
    let started = Instant::now();
    let tb = std::fs::read(DEFAULT_TOKENIZER).map_err(|e| format!("tokenizer: {e}"))?;
    if sha256_hex(&tb) != TOKENIZER_SHA {
        return Err("tokenizer sha mismatch".into());
    }
    let derived = sha256_hex(&derive_tokenizer_json(&tb, VOCAB).map_err(|e| format!("{e}"))?);
    if derived != DERIVED_SHA {
        return Err("derived tokenizer sha mismatch".into());
    }
    let tokenizer = derive_tokenizer(&tb, VOCAB).map_err(|e| format!("derive: {e}"))?;
    let source_files: Vec<serde_json::Value> = match &source_root {
        Some(sr) => [
            "crates/uor-r4-core/src/native_geometric/learner/scoped_memory.rs",
            "crates/uor-r4-core/src/native_geometric/learner/observed_text_session.rs",
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

    // ---- fit the retained categorical binder and the new intent tables ----
    let (dev_clauses, supervision) = scm_development_supervision(&tokenizer)?;
    let labels: Vec<ClauseLabel> = supervision.iter().map(|s| s.label.clone()).collect();
    let (model, fit) = fit_observed_text_model(&dev_clauses, &labels, false)
        .map_err(|e| format!("binder fit: {e}"))?;
    let cue_spans: Vec<(usize, usize)> = supervision
        .iter()
        .map(|s| (s.label.marker.0 as usize, s.label.marker.1))
        .collect();
    let examples: Vec<IntentExample> = supervision
        .iter()
        .map(|s| IntentExample {
            question: s.question,
            intent: s.intent,
        })
        .collect();
    let (intent, intent_fit) = fit_intent_model(&dev_clauses, &cue_spans, &examples, VOCAB)
        .map_err(|e| format!("intent fit: {e}"))?;
    let model_bytes = model.to_bytes().map_err(|e| format!("{e}"))?;
    let intent_bytes = intent.to_bytes().map_err(|e| format!("{e}"))?;
    write_checked(&root, "artifacts/binder.json", &model_bytes)?;
    write_checked(&root, "artifacts/intent.json", &intent_bytes)?;
    let binder_reloaded = ObservedTextModel::from_bytes(
        &std::fs::read(root.join("artifacts/binder.json")).map_err(|e| e.to_string())?,
        VOCAB,
    )
    .map_err(|e| format!("{e}"))?;
    let intent_reloaded = IntentModel::from_bytes(
        &std::fs::read(root.join("artifacts/intent.json")).map_err(|e| e.to_string())?,
        VOCAB,
    )
    .map_err(|e| format!("{e}"))?;
    if binder_reloaded != model || intent_reloaded != intent {
        return Err("artifact reload mismatch".into());
    }

    // ---- populations ----
    let dev_script = scm_development_script();
    let exposed_script = scm_exposed_script();
    let final_script = scm_final_script();
    const DEV_LINEAGE: u64 = 0x5C0F_0000_0001;
    const FINAL_LINEAGE: u64 = 0x5C0F_0000_0002;
    const EXPOSED_LINEAGE: u64 = 0x5C0F_0000_0003;
    let dev = scm_replay(
        &tokenizer,
        &model_bytes,
        &intent_bytes,
        DEV_LINEAGE,
        MemoryControl::Normal,
        DEFAULT_CHAIN_CAPACITY,
        &dev_script,
        "development",
        "normal",
    )?;
    let final_pop = scm_replay(
        &tokenizer,
        &model_bytes,
        &intent_bytes,
        FINAL_LINEAGE,
        MemoryControl::Normal,
        DEFAULT_CHAIN_CAPACITY,
        &final_script,
        "final",
        "normal",
    )?;
    // The earlier final draw whose two failures informed the wider training recipe. Retained and
    // reported as exposed, never as a fresh win.
    let exposed = scm_replay(
        &tokenizer,
        &model_bytes,
        &intent_bytes,
        EXPOSED_LINEAGE,
        MemoryControl::Normal,
        DEFAULT_CHAIN_CAPACITY,
        &exposed_script,
        "exposed_regression",
        "normal",
    )?;

    // ---- controls on both populations ----
    let mut arms = vec![
        json!({"arm": "primary", "script": "development", "questions": dev.questions,
            "complete": dev.complete, "branch_correct": dev.branch_correct,
            "branch_total": dev.branch_total, "intent_correct": dev.intent_correct,
            "intent_total": dev.intent_total, "nowrite_correct": dev.nowrite_correct,
            "nowrite_total": dev.nowrite_total}),
        json!({"arm": "primary", "script": "exposed_regression", "questions": exposed.questions,
            "complete": exposed.complete, "branch_correct": exposed.branch_correct,
            "branch_total": exposed.branch_total, "intent_correct": exposed.intent_correct,
            "intent_total": exposed.intent_total, "nowrite_correct": exposed.nowrite_correct,
            "nowrite_total": exposed.nowrite_total}),
        json!({"arm": "primary", "script": "final", "questions": final_pop.questions,
            "complete": final_pop.complete, "branch_correct": final_pop.branch_correct,
            "branch_total": final_pop.branch_total, "intent_correct": final_pop.intent_correct,
            "intent_total": final_pop.intent_total, "nowrite_correct": final_pop.nowrite_correct,
            "nowrite_total": final_pop.nowrite_total}),
    ];
    let mut all_rows: Vec<serde_json::Value> = Vec::new();
    all_rows.extend(dev.rows.clone());
    all_rows.extend(final_pop.rows.clone());
    all_rows.extend(exposed.rows.clone());
    for (name, control) in [
        ("no_read", MemoryControl::NoRead),
        ("update_disabled", MemoryControl::UpdateDisabled),
        ("unpinned", MemoryControl::Unpinned),
        ("unscoped", MemoryControl::Unscoped),
        ("parse_score_authority", MemoryControl::ParseScoreAuthority),
    ] {
        for (script_name, script, lineage) in [
            ("development", &dev_script, DEV_LINEAGE),
            ("final", &final_script, FINAL_LINEAGE),
            ("exposed_regression", &exposed_script, EXPOSED_LINEAGE),
        ] {
            let replay = scm_replay(
                &tokenizer,
                &model_bytes,
                &intent_bytes,
                lineage,
                control.clone(),
                DEFAULT_CHAIN_CAPACITY,
                script,
                script_name,
                name,
            )?;
            arms.push(
                json!({"arm": name, "script": script_name, "questions": replay.questions,
                "complete": replay.complete, "branch_correct": replay.branch_correct,
                "branch_total": replay.branch_total, "intent_correct": replay.intent_correct,
                "intent_total": replay.intent_total, "nowrite_correct": replay.nowrite_correct,
                "nowrite_total": replay.nowrite_total}),
            );
            all_rows.extend(replay.rows);
        }
    }
    // Declared-capacity arm on the development population.
    let capacity_replay = scm_replay(
        &tokenizer,
        &model_bytes,
        &intent_bytes,
        DEV_LINEAGE,
        MemoryControl::Normal,
        2,
        &dev_script,
        "development",
        "capacity_2",
    )?;
    arms.push(json!({"arm": "capacity_2", "script": "development",
        "questions": capacity_replay.questions, "complete": capacity_replay.complete,
        "branch_correct": capacity_replay.branch_correct, "branch_total": capacity_replay.branch_total,
        "intent_correct": capacity_replay.intent_correct, "intent_total": capacity_replay.intent_total,
        "nowrite_correct": capacity_replay.nowrite_correct, "nowrite_total": capacity_replay.nowrite_total}));
    let capacity_changes = capacity_replay
        .rows
        .iter()
        .filter(|r| r["kind"] != "ingest")
        .zip(dev.rows.iter().filter(|r| r["kind"] != "ingest"))
        .filter(|(a, b)| {
            a["turn"] == b["turn"] && (a["answer"] != b["answer"] || a["terminal"] != b["terminal"])
        })
        .count();
    all_rows.extend(capacity_replay.rows);

    // ---- procedural interventions on one committed development store ----
    let dev_memory = Memory::from_bytes(&dev.store).map_err(|e| format!("{e}"))?;
    let dev_base = ScopedMemoryRuntime::load(
        &model_bytes,
        &intent_bytes,
        dev_memory.clone(),
        DEV_LINEAGE,
        MemoryControl::Normal,
        VOCAB,
        Some(SCM_EOS),
    )
    .map_err(|e| format!("{e}"))?;
    let baseline_answers = scm_answer_questions(&tokenizer, &dev_base, &dev_script)?;
    let cedar_question = scm_clause(&tokenizer, 0, ScmForm::AskCurrentOffice, "Cedar", None)?.0;
    let baseline_cedar = {
        let mut s = dev_base
            .ask(&cedar_question, SCM_SCOPES[0].as_bytes())
            .map_err(|e| format!("{e}"))?;
        dev_base.run(&mut s).map_err(|e| format!("{e}"))?;
        scm_emitted_key(&tokenizer, &s)
    };

    // (a) Physical storage order must be irrelevant to authority.
    let mut reordered_memory = dev_memory.clone();
    reordered_memory.records.reverse();
    reordered_memory.validate().map_err(|e| format!("{e}"))?;
    let reordered_runtime = dev_base.with_memory(reordered_memory, MemoryControl::Normal)?;
    let reorder_identical =
        scm_answer_questions(&tokenizer, &reordered_runtime, &dev_script)? == baseline_answers;

    // (b) A pinned read view is not spliced by a correction that arrives mid-answer.
    let mid_correction = scm_clause(
        &tokenizer,
        0,
        ScmForm::CorrectOffice,
        "Oren",
        Some("Office Park"),
    )?
    .0;
    let disconnect = scm_raw_clause(&tokenizer, "Cedar's office became Fen")?;
    let mut pinned_runtime = ScopedMemoryRuntime::load(
        &model_bytes,
        &intent_bytes,
        dev_memory.clone(),
        DEV_LINEAGE,
        MemoryControl::Normal,
        VOCAB,
        Some(SCM_EOS),
    )
    .map_err(|e| format!("{e}"))?;
    let mut pinned_session = pinned_runtime
        .ask(&cedar_question, SCM_SCOPES[0].as_bytes())
        .map_err(|e| format!("{e}"))?;
    let pinned_view = pinned_session.view;
    pinned_runtime
        .step(&mut pinned_session)
        .map_err(|e| format!("{e}"))?;
    pinned_runtime
        .ingest(&disconnect, SCM_SCOPES[0].as_bytes(), 899)
        .map_err(|e| format!("{e}"))?;
    pinned_runtime
        .ingest(&mid_correction, SCM_SCOPES[0].as_bytes(), 900)
        .map_err(|e| format!("{e}"))?;
    pinned_runtime
        .run(&mut pinned_session)
        .map_err(|e| format!("{e}"))?;
    let pinned_answer = scm_emitted_key(&tokenizer, &pinned_session);
    let pinned_terminal = pinned_session.terminal;

    let mut oracle_to_pin = ScmOracle::new(DEFAULT_CHAIN_CAPACITY);
    for op in &dev_script {
        if let ScmOp::Ingest {
            scope,
            form,
            entity,
            value,
        } = op
        {
            oracle_to_pin.declared(*scope, *form, entity, value);
        }
    }
    let pinned_expected = oracle_to_pin.answer(0, "Cedar", 0, HistoryView::Current, pinned_view);
    let pinned_matches_expected = matches!(
        (&pinned_expected, pinned_terminal),
        (ScmExpected::Complete { value, .. }, Some(ScopedTerminal::Complete)) if *value == pinned_answer
    );

    let mut unpinned_runtime = ScopedMemoryRuntime::load(
        &model_bytes,
        &intent_bytes,
        dev_memory.clone(),
        DEV_LINEAGE,
        MemoryControl::Unpinned,
        VOCAB,
        Some(SCM_EOS),
    )
    .map_err(|e| format!("{e}"))?;
    let mut unpinned_session = unpinned_runtime
        .ask(&cedar_question, SCM_SCOPES[0].as_bytes())
        .map_err(|e| format!("{e}"))?;
    unpinned_runtime
        .step(&mut unpinned_session)
        .map_err(|e| format!("{e}"))?;
    unpinned_runtime
        .ingest(&disconnect, SCM_SCOPES[0].as_bytes(), 899)
        .map_err(|e| format!("{e}"))?;
    unpinned_runtime
        .ingest(&mid_correction, SCM_SCOPES[0].as_bytes(), 901)
        .map_err(|e| format!("{e}"))?;
    unpinned_runtime
        .run(&mut unpinned_session)
        .map_err(|e| format!("{e}"))?;
    let unpinned_answer = scm_emitted_key(&tokenizer, &unpinned_session);
    let mut consistent_answers = vec![format!("{pinned_expected:?}")];
    oracle_to_pin.declared(0, ScmForm::CorrectOffice, "Cedar", "Fen");
    consistent_answers.push(format!(
        "{:?}",
        oracle_to_pin.answer(0, "Cedar", 0, HistoryView::Current, oracle_to_pin.commit)
    ));
    oracle_to_pin.declared(0, ScmForm::CorrectOffice, "Oren", "Office Park");
    consistent_answers.push(format!(
        "{:?}",
        oracle_to_pin.answer(0, "Cedar", 0, HistoryView::Current, oracle_to_pin.commit)
    ));
    let unpinned_splices = pinned_answer == "Cedar Annex"
        && unpinned_answer == "Office Park"
        && consistent_answers
            .iter()
            .all(|s| !s.contains("Office Park"));

    // (c) An owned capture survives replacement of its origin.
    let mut capture_runtime = ScopedMemoryRuntime::load(
        &model_bytes,
        &intent_bytes,
        dev_memory.clone(),
        DEV_LINEAGE,
        MemoryControl::Normal,
        VOCAB,
        Some(SCM_EOS),
    )
    .map_err(|e| format!("{e}"))?;
    let mara_question = scm_clause(&tokenizer, 0, ScmForm::AskCurrentOffice, "Mara", None)?.0;
    let mut capture_session = capture_runtime
        .ask(&mara_question, SCM_SCOPES[0].as_bytes())
        .map_err(|e| format!("{e}"))?;
    capture_runtime
        .step(&mut capture_session)
        .map_err(|e| format!("{e}"))?;
    let live_before = capture_session.origin_is_live(capture_runtime.memory());
    let capture_before = capture_session.captured.clone();
    let mara_replacement = scm_clause(&tokenizer, 0, ScmForm::AssertOffice, "Mara", Some("Fen"))?.0;
    capture_runtime
        .ingest(&mara_replacement, SCM_SCOPES[0].as_bytes(), 902)
        .map_err(|e| format!("{e}"))?;
    let live_after = capture_session.origin_is_live(capture_runtime.memory());
    capture_runtime
        .run(&mut capture_session)
        .map_err(|e| format!("{e}"))?;
    let capture_answer = scm_emitted_key(&tokenizer, &capture_session);
    let capture_complete = capture_session.terminal == Some(ScopedTerminal::Complete);
    let capture_preserved = capture_before == capture_session.captured;
    let capture_liveness_distinct = live_before && !live_after;

    // (d) A changed source changes the dependent answer and keeps the oracle agreement.
    let mut changed_script = dev_script.clone();
    for op in changed_script.iter_mut() {
        if let ScmOp::Ingest {
            scope: 0,
            form: ScmForm::CorrectOffice,
            entity: "Oren",
            value,
        } = op
        {
            *value = "Office Park";
        }
    }
    let changed = scm_replay(
        &tokenizer,
        &model_bytes,
        &intent_bytes,
        DEV_LINEAGE,
        MemoryControl::Normal,
        DEFAULT_CHAIN_CAPACITY,
        &changed_script,
        "development_changed_source",
        "normal",
    )?;
    let changed_runtime = ScopedMemoryRuntime::load(
        &model_bytes,
        &intent_bytes,
        Memory::from_bytes(&changed.store).map_err(|e| format!("{e}"))?,
        DEV_LINEAGE,
        MemoryControl::Normal,
        VOCAB,
        Some(SCM_EOS),
    )
    .map_err(|e| format!("{e}"))?;
    let changed_cedar = {
        let mut s = changed_runtime
            .ask(&cedar_question, SCM_SCOPES[0].as_bytes())
            .map_err(|e| format!("{e}"))?;
        changed_runtime.run(&mut s).map_err(|e| format!("{e}"))?;
        scm_emitted_key(&tokenizer, &s)
    };
    all_rows.extend(changed.rows.clone());
    let changed_source_ok = changed_cedar != baseline_cedar
        && changed.complete == changed.questions
        && changed_cedar == "Office Park";

    // (e) A typed cycle.
    let cycle_script = vec![
        ScmOp::Ingest {
            scope: 0,
            form: ScmForm::RedirectOffice,
            entity: "Mara",
            value: "Ivo",
        },
        ScmOp::Ingest {
            scope: 0,
            form: ScmForm::RedirectOffice,
            entity: "Ivo",
            value: "Mara",
        },
        ScmOp::Ask {
            scope: 0,
            form: ScmForm::AskCurrentOffice,
            entity: "Mara",
        },
    ];
    let cycle = scm_replay(
        &tokenizer,
        &model_bytes,
        &intent_bytes,
        DEV_LINEAGE,
        MemoryControl::Normal,
        DEFAULT_CHAIN_CAPACITY,
        &cycle_script,
        "cycle",
        "normal",
    )?;
    all_rows.extend(cycle.rows.clone());
    let cycle_typed = cycle.complete == cycle.questions && cycle.questions == 1;

    // Historical intent applies to the first relation only; later hops use the pinned current view.
    let history_script = vec![
        ScmOp::Ingest {
            scope: 0,
            form: ScmForm::AssertOffice,
            entity: "Ivo",
            value: "Fen",
        },
        ScmOp::Ingest {
            scope: 0,
            form: ScmForm::AssertOffice,
            entity: "Oren",
            value: "Office Park",
        },
        ScmOp::Ingest {
            scope: 0,
            form: ScmForm::RedirectOffice,
            entity: "Cedar",
            value: "Ivo",
        },
        ScmOp::Ingest {
            scope: 0,
            form: ScmForm::RedirectOffice,
            entity: "Cedar",
            value: "Oren",
        },
        ScmOp::Ingest {
            scope: 0,
            form: ScmForm::CorrectOffice,
            entity: "Ivo",
            value: "Cedar Annex",
        },
        ScmOp::Ask {
            scope: 0,
            form: ScmForm::AskPreviousOffice,
            entity: "Cedar",
        },
        ScmOp::Ask {
            scope: 0,
            form: ScmForm::AskInitialOffice,
            entity: "Cedar",
        },
        ScmOp::Ask {
            scope: 0,
            form: ScmForm::AskCurrentOffice,
            entity: "Cedar",
        },
    ];
    let historical = scm_replay(
        &tokenizer,
        &model_bytes,
        &intent_bytes,
        DEV_LINEAGE,
        MemoryControl::Normal,
        DEFAULT_CHAIN_CAPACITY,
        &history_script,
        "historical_dependent_review",
        "normal",
    )?;
    let historical_answers: Vec<String> = historical
        .rows
        .iter()
        .filter(|r| r["kind"] == "ask")
        .filter_map(|r| r["answer"].as_str().map(str::to_string))
        .collect();
    let historical_ok = historical.complete == 3
        && historical.questions == 3
        && historical_answers == ["Cedar Annex", "Cedar Annex", "Office Park"];
    all_rows.extend(historical.rows);

    // ---- save / reload ----
    let reload_dir = root.join("reload");
    std::fs::create_dir_all(&reload_dir).map_err(|e| format!("{e}"))?;
    write_checked(&root, "reload/model.json", &model_bytes)?;
    write_checked(&root, "reload/intent.json", &intent_bytes)?;
    write_checked(&root, "reload/store.json", &dev.store)?;
    write_json(
        &root,
        "reload/queries.json",
        &json!(scm_queries(&dev_script)),
    )?;
    let mut checkpoints = Vec::new();
    let mut phase = dev_base
        .ask(&cedar_question, b"alpha")
        .map_err(|e| e.to_string())?;
    loop {
        let snapshot = dev_base.snapshot(&phase).map_err(|e| e.to_string())?;
        let mut continued = dev_base.restore(&snapshot).map_err(|e| e.to_string())?;
        dev_base.run(&mut continued).map_err(|e| e.to_string())?;
        checkpoints.push(
            json!({"session":serde_json::from_slice::<serde_json::Value>(&snapshot)
            .map_err(|e| e.to_string())?, "expected":scm_answer_json(&continued)}),
        );
        if phase.terminal.is_some() {
            break;
        }
        dev_base.step(&mut phase).map_err(|e| e.to_string())?;
    }
    write_json(&root, "reload/checkpoints.json", &json!(checkpoints))?;
    let reloaded_memory = Memory::from_bytes(
        &std::fs::read(root.join("reload/store.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{e}"))?;
    let reloaded_runtime = ScopedMemoryRuntime::load(
        &model_bytes,
        &intent_bytes,
        reloaded_memory,
        DEV_LINEAGE,
        MemoryControl::Normal,
        VOCAB,
        Some(SCM_EOS),
    )
    .map_err(|e| format!("{e}"))?;
    let reload_answers = scm_answer_questions(&tokenizer, &reloaded_runtime, &dev_script)?;
    let disk_reload_identical = reload_answers == baseline_answers;
    let child = std::process::Command::new(
        std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?,
    )
    .arg("--mode=scoped-correction-memory")
    .arg("--reload-check")
    .arg(&reload_dir)
    .output()
    .map_err(|e| format!("fresh process: {e}"))?;
    let child_stdout = String::from_utf8_lossy(&child.stdout).into_owned();
    let child_json: serde_json::Value =
        serde_json::from_str(child_stdout.trim()).map_err(|e| format!("child json: {e}"))?;
    let child_answers = child_json["answers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let fresh_process_identical = child.status.success()
        && child_answers.len() == reload_answers.len()
        && child_answers
            .iter()
            .zip(reload_answers.iter())
            .all(|(c, p)| {
                c["emitted"] == p["emitted"]
                    && c["terminal"] == p["terminal"]
                    && c["hop"] == p["hop"]
            });

    let mut terminal_counts = std::collections::BTreeMap::<String, usize>::new();
    for row in &all_rows {
        let kind = row["kind"].as_str().unwrap_or_default();
        if kind == "ask" || kind == "ask_view" {
            let key = format!("{}", row["terminal"]);
            *terminal_counts.entry(key).or_default() += 1;
        }
    }
    let arm_total = |name: &str| -> u64 {
        arms.iter()
            .filter(|a| a["arm"] == json!(name))
            .map(|a| a["complete"].as_u64().unwrap_or(0))
            .sum()
    };
    let primary_total = arm_total("primary");
    let checks = json!({
        "development_complete": dev.complete == dev.questions && dev.questions > 0,
        "final_complete": final_pop.complete == final_pop.questions && final_pop.questions > 0,
        "exposed_reported": exposed.questions > 0,
        "intent_learned_exactly": dev.intent_correct == dev.intent_total
            && final_pop.intent_correct == final_pop.intent_total
            && exposed.intent_correct == exposed.intent_total
            && dev.intent_total > 0,
        "binder_branch_exact": dev.branch_correct == dev.branch_total
            && final_pop.branch_correct == final_pop.branch_total
            && exposed.branch_correct == exposed.branch_total,
        "nonasserting_writes_nothing": dev.nowrite_correct == dev.nowrite_total
            && dev.nowrite_total > 0,
        "capacity_arm_agrees": capacity_replay.complete == capacity_replay.questions,
        "reordered_storage_invariant": reorder_identical,
        "pinned_view_matches_oracle": pinned_matches_expected,
        "unpinned_control_splices": unpinned_splices,
        "owned_capture_preserved": capture_preserved && capture_liveness_distinct && capture_complete,
        "changed_source_changes_answer": changed_source_ok,
        "cycle_typed": cycle_typed,
        "historical_dependent_first_hop": historical_ok,
        // Each named control must actually change the measured answers on the same inputs.
        "scope_control_contaminates": arm_total("unscoped") < primary_total,
        "phase_and_controls_differ": arm_total("parse_score_authority") < primary_total,
        "history_control_differs": capacity_changes > 0,
        "write_control_differs": arm_total("update_disabled") < primary_total,
        "read_control_differs": arm_total("no_read") < primary_total,
        "disk_reload_identical": disk_reload_identical,
        "fresh_process_identical": fresh_process_identical,
        "fresh_process_learned_update": child_json["learned_restart"] == true,
        "fresh_process_phases_after_update": child_json["resumed"].as_array()
            .is_some_and(|a| a.len() == checkpoints.len() && !a.is_empty() && a.iter().all(|c| c["matches"] == true)),
    });
    let all_expected = checks
        .as_object()
        .map(|m| m.values().all(|v| v == &json!(true)))
        .unwrap_or(false);
    let result = json!({
        "schema": "uor-r4.scoped-correction-memory/2",
        "evaluation_scope": "Principal corrected exposed replay of submitted panels; original final lexical population is already exposed. Typed AskView and expected failure matches are separate from emitted learned-language answers.",
        "restart_child": child_json,
        "historical_dependent_answers": historical_answers,
        "capacity_changes_same_development_inputs": capacity_changes,
        "base_revision": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
        "running_source": {
            "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
            "git_dirty": std::env::var("UOR_GIT_DIRTY").unwrap_or_else(|_| "unknown".into()),
            "executable_sha256": executable_sha256,
            "source_files": source_files,
        },
        "inputs": {"tokenizer_derived": DERIVED_SHA, "vocab": VOCAB, "eos": SCM_EOS,
            "capacity": DEFAULT_CHAIN_CAPACITY, "hop_bound": SCOPED_MAX_HOPS},
        "task": {
            "serving_input": "observed readable clauses (assertions, explicit corrections, negations, redirects and three temporal question forms) with a host-authenticated scope tag; no gold intent, relation, expected answer or preselected fact",
            "development_entities": SCM_DEV_ENTITIES,
            "development_values": {"offices": SCM_DEV_OFFICES, "projects": SCM_DEV_PROJECTS},
            "final_entities": SCM_FINAL_ENTITIES,
            "final_values": {"offices": SCM_FINAL_OFFICES, "projects": SCM_FINAL_PROJECTS},
            "scopes": SCM_SCOPES,
            "novelty": "the final population's entity and value strings never appear in fitting; the forms and cue phrases are shared",
        },
        "semantics": {
            "assert": "appends a revision; a differing bare assertion becomes current and is marked conflict",
            "correct": "explicit correction supersedes with the stated value, never marked conflict",
            "same_value_reassertion": "a new revision with the same value; history is never deduplicated",
            "previous": "previous assertion (immediate predecessor)",
            "previous_distinct": "separate exact query; not reachable from learned question intent",
            "initial": "earliest retained revision; an evicted one is typed, never absent",
            "historical_dependent_read": "first hop uses the requested view; later hops resolve Current at the same pinned view",
            "pin": "the read view is fixed when the answer starts",
        },
        "learning": {"binder": fit, "intent": intent_fit},
        "panels": {
            "development": {"questions": dev.questions, "complete": dev.complete},
            "exposed_regression": {"questions": exposed.questions, "complete": exposed.complete},
            "final": {"questions": final_pop.questions, "complete": final_pop.complete},
        },
        "arms": arms,
        "checks": checks,
        "checks_all_expected": all_expected,
        "ask_terminal_counts": terminal_counts,
        "interventions": {
            "reordered_storage": {"answers_identical": reorder_identical},
            "pinned_view": {"pin": pinned_view, "answer": pinned_answer, "terminal": pinned_terminal,
                "expected": format!("{pinned_expected:?}"), "unpinned_answer": unpinned_answer, "consistent_answers": consistent_answers},
            "owned_capture": {"live_before": live_before, "live_after": live_after,
                "answer": capture_answer, "payload_preserved": capture_preserved},
            "changed_source": {"baseline_cedar": baseline_cedar, "changed_cedar": changed_cedar,
                "complete": changed.complete, "questions": changed.questions},
            "cycle": {"complete": cycle.complete, "questions": cycle.questions},
            "save_reload": {"disk_reload_identical": disk_reload_identical,
                "fresh_process_identical": fresh_process_identical,
                "child_status": child.status.code(),"raw_text_entry":true,"child":child_json},
        },
        "all_rows": all_rows.len(),
        "elapsed_s": started.elapsed().as_secs_f64(),
        "scope": "authored ordinary readable text in a small declared memory domain; learned intent feeding exact versioned storage with a pinned read view. Scope is host-authenticated, not inferred from language. Not broad language understanding, general reasoning or frontier capability. Energy UNAVAILABLE; whole-path D0-b not claimed.",
    });
    write_json(&root, "result.json", &result)?;
    write_checked(
        &root,
        "rows.jsonl",
        all_rows
            .iter()
            .map(|r| serde_json::to_string(r).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
            .as_bytes(),
    )?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    if !all_expected {
        return Err(format!("sealed scoped-memory contract failure: {checks}"));
    }
    println!(
        "scoped-correction-memory: dev {}/{} final {}/{} | intent {}/{} branch {}/{} nowrite {}/{} | controls: no_read {} update_disabled {} unpinned {} unscoped {} parse_score {} capacity2 {} | intent-w {} binder-w {} | sealed {} unlisted | {:.1}s",
        dev.complete,
        dev.questions,
        final_pop.complete,
        final_pop.questions,
        dev.intent_correct + final_pop.intent_correct,
        dev.intent_total + final_pop.intent_total,
        dev.branch_correct + final_pop.branch_correct,
        dev.branch_total + final_pop.branch_total,
        dev.nowrite_correct + final_pop.nowrite_correct,
        dev.nowrite_total + final_pop.nowrite_total,
        arms.iter()
            .filter(|a| a["arm"] == json!("no_read"))
            .map(|a| a["complete"].as_u64().unwrap_or(0))
            .sum::<u64>(),
        arms.iter()
            .filter(|a| a["arm"] == json!("update_disabled"))
            .map(|a| a["complete"].as_u64().unwrap_or(0))
            .sum::<u64>(),
        arms.iter()
            .filter(|a| a["arm"] == json!("unpinned"))
            .map(|a| a["complete"].as_u64().unwrap_or(0))
            .sum::<u64>(),
        arms.iter()
            .filter(|a| a["arm"] == json!("unscoped"))
            .map(|a| a["complete"].as_u64().unwrap_or(0))
            .sum::<u64>(),
        arms.iter()
            .filter(|a| a["arm"] == json!("parse_score_authority"))
            .map(|a| a["complete"].as_u64().unwrap_or(0))
            .sum::<u64>(),
        capacity_replay.complete,
        intent_fit.statement_weights + intent_fit.question_weights,
        fit.segment_potentials,
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Consumed geometric state: an observed request drives a shared computation whose
// grounded result changes a later exact read and the complete answer
// ---------------------------------------------------------------------------

const CGS_EOS: u32 = u32::MAX - 1;
/// The eight retained state labels. They double as entity names, so a grounded result can address a
/// memory record. The vocabulary is familiar in development by design.
const CGS_LABELS: [&str; 8] = [
    "Alma", "Bert", "Cora", "Dane", "Elin", "Frey", "Gwen", "Holt",
];
/// The operation words and the opaque primitive identifiers they are declared with. The mapping
/// from word to identifier is *learned* into the lexicon; the identifier's group element is
/// recovered from observed transitions.
const CGS_OPS: [(&str, u32); 3] = [("i", 100), ("j", 200), ("e", 300)];
/// Development people and destinations (familiar strings).
const CGS_DEV_PEOPLE: [&str; 4] = ["Mara", "Ivo", "Cedar", "Oren"];
const CGS_DEV_DESTS: [&str; 8] = [
    "Bramble", "Quarry", "Vale", "Marsh", "Tarn", "Ledge", "Ridge", "Stone",
];
/// Final people and destinations: disjoint strings, plus withheld ordered operation combinations.
const CGS_FINAL_PEOPLE: [&str; 4] = ["Una", "Pia", "Soren", "Kestrel"];
const CGS_FINAL_DESTS: [&str; 8] = [
    "Cobalt", "Mica", "Dune", "Basalt", "Flint", "Slate", "Amber", "Onyx",
];
/// Operation sequences exercised in development.
const CGS_DEV_SEQS: [&[&str]; 6] = [
    &["i"],
    &["j"],
    &["i", "j"],
    &["j", "i"],
    &["e"],
    &["i", "e"],
];
/// Withheld ordered combinations used only by the final population.
const CGS_FINAL_SEQS: [&[&str]; 2] = [&["i", "j", "i"], &["j", "i", "j"]];
/// A fresh withheld population: people and destinations disjoint from every earlier population, and
/// three-operation orders that appear in no development or earlier exposed population. Its worlds
/// also carry a person-level redirect, so a requested operation must follow an observed redirect to
/// reach its operand: that routing composition is absent from fitting.
const CGS_FRESH_PEOPLE: [&str; 4] = ["Juno", "Rhea", "Silas", "Perrin"];
const CGS_FRESH_DESTS: [&str; 8] = [
    "Larkspur", "Nettle", "Umber", "Vellum", "Wren", "Yarrow", "Zephyr", "Cinder",
];
const CGS_FRESH_SEQS: [&[&str]; 3] = [&["i", "i", "j"], &["j", "j", "i"], &["j", "i", "i"]];

/// The observed clause forms.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CgsForm {
    /// "What is {person}'s office?" - an ordinary memory question.
    AskOffice,
    /// "{person}'s office then {ops}" - a computation request whose object is the operation span.
    Compute,
    /// "{entity}'s office is {value}" - an assertion that builds the document.
    AssertOffice,
    /// "{entity} office follows {value}" - a redirect whose value is another entity key.
    RedirectOffice,
}

impl CgsForm {
    fn role(self) -> usize {
        if self == CgsForm::RedirectOffice {
            1
        } else {
            0
        }
    }
    fn action(self) -> RelAction {
        if self == CgsForm::RedirectOffice {
            RelAction::Continue
        } else {
            RelAction::Emit
        }
    }
}

fn cgs_ob_form(form: CgsForm, entity: &str, tail: &str) -> ObForm {
    let parts = match form {
        CgsForm::AskOffice => vec![
            (None, "What is".to_string()),
            (Some(ObPart::Subject), format!(" {entity}")),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), " office".to_string()),
            (None, "?".to_string()),
        ],
        CgsForm::Compute => vec![
            (Some(ObPart::Subject), entity.to_string()),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), " office then".to_string()),
            (Some(ObPart::Object), format!(" {tail}")),
        ],
        CgsForm::AssertOffice => vec![
            (Some(ObPart::Subject), entity.to_string()),
            (None, "'s".to_string()),
            (Some(ObPart::Cue), " office".to_string()),
            (None, " is".to_string()),
            (Some(ObPart::Object), format!(" {tail}")),
        ],
        CgsForm::RedirectOffice => vec![
            (Some(ObPart::Subject), entity.to_string()),
            (Some(ObPart::Cue), " office follows".to_string()),
            (Some(ObPart::Object), format!(" {tail}")),
        ],
    };
    ObForm {
        parts,
        role: form.role(),
        goal: Goal::Office,
        action: form.action(),
    }
}

fn cgs_clause(
    tokenizer: &HfBpeTokenizer,
    seg: u32,
    form: CgsForm,
    entity: &str,
    tail: &str,
) -> Result<(Clause, ClauseLabel), String> {
    ob_build_clause(tokenizer, seg, &cgs_ob_form(form, entity, tail))
}

/// The declared Q8 action used to *generate* development observations. The constructive
/// factorization never reads these codes: it recovers the action from the observed transitions, and
/// this table is used only to build the fixture and to write independent expectations.
struct CgsAction {
    /// label -> element index
    element: Vec<usize>,
    /// primitive -> element index
    code: Vec<(u32, usize)>,
}

impl CgsAction {
    fn build() -> Result<Self, String> {
        let witness = uor_r4_core::native_geometric::learner::shared_transition::q8_witness()
            .map_err(|e| format!("q8 witness: {e}"))?;
        let t = ExactGroupTable::build().map_err(|e| format!("table: {e}"))?;
        let product = |a: usize, b: usize| t.product[a * 128 + b] as usize;
        // Two noncommuting generators plus the identity.
        let mut pair = None;
        for a in witness.iter() {
            for b in witness.iter() {
                if product(*a as usize, *b as usize) != product(*b as usize, *a as usize) {
                    pair = Some((*a as u32, *b as u32));
                    break;
                }
            }
            if pair.is_some() {
                break;
            }
        }
        let (gi, gj) = pair.ok_or("q8 witness has no noncommuting pair")?;
        let identity = t.identity;
        Ok(CgsAction {
            element: CGS_LABELS
                .iter()
                .enumerate()
                .map(|(k, _)| witness[k] as usize)
                .collect(),
            code: vec![
                (100, gi as usize),
                (200, gj as usize),
                (300, identity as usize),
            ],
        })
    }

    fn element_of_label(&self, label: u32) -> Option<usize> {
        self.element.get(label as usize).copied()
    }
    fn label_of_element(&self, element: usize) -> Option<u32> {
        self.element
            .iter()
            .position(|e| *e == element)
            .map(|k| k as u32)
    }
    fn apply(&self, state: usize, primitive: u32) -> Option<usize> {
        let t = ExactGroupTable::build().ok()?;
        let code = self
            .code
            .iter()
            .find(|(p, _)| *p == primitive)
            .map(|(_, c)| *c)?;
        Some(t.product[code * 128 + state] as usize)
    }
}

/// The declared world: which label each person holds, and each label's own office record.
#[derive(Clone)]
struct CgsWorld {
    id: u32,
    version: u32,
    people: &'static [&'static str],
    dests: &'static [&'static str],
    /// person index -> label index
    assignment: Vec<usize>,
    /// label index -> Some(dest index) for a terminal office, or None for a redirect to label k
    office: Vec<Option<usize>>,
    /// label index -> redirect target label index
    redirect: Vec<usize>,
    /// person index -> another person index whose office this person's office follows. This is the
    /// declared structure that requires *following an observed redirect to the operand* before a
    /// requested operation is applied, so the request alone cannot predict the number of reads.
    person_redirect: Vec<Option<usize>>,
    scope: &'static str,
}

/// An independently computed expectation for one request, using the declared action and world.
#[derive(Clone, Debug, PartialEq, Eq)]
enum CgsExpected {
    /// A complete answer with its surface text and the exact selected record ids.
    Answer {
        text: String,
        reads: usize,
    },
    Unresolved,
    Exhausted,
    UnknownOperation,
    Ungrounded,
}

impl CgsExpected {
    fn label(&self) -> String {
        match self {
            Self::Answer { text, .. } => format!("Answer({text})"),
            other => format!("{other:?}"),
        }
    }
}

/// Evaluate one request independently: walk the declared world, apply the declared action and read
/// the resulting record. This never consults the fitted artifact or the loaded runtime.
fn cgs_expectation(
    world: &CgsWorld,
    action: &CgsAction,
    person_index: usize,
    ops: &[&str],
) -> CgsExpected {
    let mut entity = world.people[person_index].to_string();
    let mut reads = 0usize;
    let mut guard = 0;
    // Declared semantics: a redirect is followed before anything else, so a requested operation is
    // applied to the *terminal* value reached from the requested entity, never to an intermediate
    // reference. The request alone therefore cannot predict the number of reads.
    let mut ops_done = false;
    loop {
        guard += 1;
        if guard > 8 {
            return CgsExpected::Exhausted;
        }
        // Locate the record for this entity.
        let (value, continues) = if let Some(p) = world
            .people
            .iter()
            .position(|name| *name == entity.as_str())
        {
            match world.person_redirect[p] {
                Some(target) => (world.people[target].to_string(), true),
                None => (CGS_LABELS[world.assignment[p]].to_string(), false),
            }
        } else if let Some(l) = CGS_LABELS.iter().position(|name| *name == entity.as_str()) {
            match world.office[l] {
                Some(dest) => (world.dests[dest].to_string(), false),
                None => (CGS_LABELS[world.redirect[l]].to_string(), true),
            }
        } else {
            return CgsExpected::Unresolved;
        };
        reads += 1;
        if !ops_done && !ops.is_empty() && !continues {
            // The operand label grounds to a retained state; apply the observed operations in order.
            let Some(label) = CGS_LABELS.iter().position(|name| *name == value.as_str()) else {
                return CgsExpected::Ungrounded;
            };
            let Some(mut state) = action.element_of_label(label as u32) else {
                return CgsExpected::Ungrounded;
            };
            for op in ops {
                let Some(primitive) = CGS_OPS.iter().find(|(word, _)| word == op).map(|(_, p)| *p)
                else {
                    return CgsExpected::UnknownOperation;
                };
                match action.apply(state, primitive) {
                    Some(next) => state = next,
                    None => return CgsExpected::UnknownOperation,
                }
            }
            let Some(derived) = action.label_of_element(state) else {
                return CgsExpected::Ungrounded;
            };
            entity = CGS_LABELS[derived as usize].to_string();
            ops_done = true;
            continue;
        }
        if continues {
            entity = value;
            continue;
        }
        return CgsExpected::Answer { text: value, reads };
    }
}

/// Development supervision for the binder and the intent tables, over the declared forms.
struct CgsSupervision {
    label: ClauseLabel,
    question: bool,
    intent: usize,
}

fn cgs_development_supervision(
    tokenizer: &HfBpeTokenizer,
    seg_start: u32,
) -> Result<(Vec<Clause>, Vec<CgsSupervision>), String> {
    let mut clauses = Vec::new();
    let mut supervision = Vec::new();
    let mut seg = seg_start;
    let mut push = |clauses: &mut Vec<Clause>,
                    supervision: &mut Vec<CgsSupervision>,
                    seg: &mut u32,
                    form: CgsForm,
                    entity: &str,
                    tail: &str,
                    question: bool,
                    intent: usize|
     -> Result<(), String> {
        let (clause, label) = cgs_clause(tokenizer, *seg, form, entity, tail)?;
        clauses.push(clause);
        supervision.push(CgsSupervision {
            label,
            question,
            intent,
        });
        *seg += 1;
        Ok(())
    };
    for (index, person) in CGS_DEV_PEOPLE.iter().enumerate() {
        push(
            &mut clauses,
            &mut supervision,
            &mut seg,
            CgsForm::AskOffice,
            person,
            "",
            true,
            ASK_CURRENT,
        )?;
        push(
            &mut clauses,
            &mut supervision,
            &mut seg,
            CgsForm::AssertOffice,
            person,
            CGS_LABELS[index],
            false,
            STMT_ASSERT,
        )?;
        for ops in CGS_DEV_SEQS.iter() {
            push(
                &mut clauses,
                &mut supervision,
                &mut seg,
                CgsForm::Compute,
                person,
                &ops.join(" "),
                false,
                STMT_COMPUTE,
            )?;
        }
    }
    for label in CGS_LABELS.iter() {
        push(
            &mut clauses,
            &mut supervision,
            &mut seg,
            CgsForm::AssertOffice,
            label,
            CGS_DEV_DESTS[0],
            false,
            STMT_ASSERT,
        )?;
        push(
            &mut clauses,
            &mut supervision,
            &mut seg,
            CgsForm::RedirectOffice,
            label,
            CGS_LABELS[0],
            false,
            STMT_ASSERT,
        )?;
    }
    Ok((clauses, supervision))
}

/// **One combined supervision set** over both retained form families: the scoped
/// correction/history/relationship forms and the consumed-computation forms. One binder and one
/// intent table therefore have to bind ordinary arguments *and* observe an operation request, which is
/// what lets a single loaded artifact compose them. The submitted design fitted the same two heads on
/// the computation forms alone; that ablation is retained below as a named comparison.
fn combined_development_supervision(
    tokenizer: &HfBpeTokenizer,
) -> Result<(Vec<Clause>, Vec<CgsSupervision>), String> {
    let (mut clauses, scm) = scm_development_supervision(tokenizer)?;
    let mut supervision: Vec<CgsSupervision> = scm
        .into_iter()
        .map(|s| CgsSupervision {
            label: s.label,
            question: s.question,
            intent: s.intent,
        })
        .collect();
    let seg_start = clauses.iter().map(|c| c.seg).max().map_or(0, |m| m + 1);
    let (cgs_clauses, cgs) = cgs_development_supervision(tokenizer, seg_start)?;
    clauses.extend(cgs_clauses);
    supervision.extend(cgs);
    Ok((clauses, supervision))
}

/// Build Q8 development observations from the declared action. The factorization reads only these
/// transitions, never the generator's codes.
fn cgs_examples(action: &CgsAction) -> Result<Vec<StExample>, String> {
    let t = ExactGroupTable::build().map_err(|e| format!("table: {e}"))?;
    let product = |a: usize, b: usize| t.product[a * 128 + b] as usize;
    let mut examples = Vec::new();
    for (k, element) in action.element.iter().enumerate() {
        let label = k as u32;
        for (a, b) in [(0usize, 1usize), (1, 0)] {
            let (pa, pb) = (action.code[a].0, action.code[b].0);
            let first = product(action.code[a].1, *element);
            let second = product(action.code[b].1, first);
            examples.push(StExample {
                payload: label,
                primitives: vec![pa, pb],
                targets: vec![
                    action
                        .label_of_element(first)
                        .ok_or("first element is unlabelled")?,
                    action
                        .label_of_element(second)
                        .ok_or("second element is unlabelled")?,
                ],
            });
        }
        // The identity primitive is observed too, so the artifact grounds it. It must leave every
        // outcome unchanged, which is the correct negative control for the apply phase.
        let identity = action
            .code
            .iter()
            .find(|(p, c)| *p == 300 && *c == t.identity as usize)
            .map(|(p, _)| *p)
            .ok_or("the declared action has no identity primitive")?;
        let after_i = product(action.code[0].1, *element);
        // The identity must appear in a non-initial position to ground its permutation.
        let after_label = action.label_of_element(after_i).ok_or("unlabelled")?;
        examples.push(StExample {
            payload: label,
            primitives: vec![action.code[0].0, identity],
            targets: vec![after_label, after_label],
        });
    }
    Ok(examples)
}

struct CgsPanel {
    requests: usize,
    complete: usize,
    rows: Vec<serde_json::Value>,
}

#[allow(clippy::too_many_arguments)]
fn cgs_panel(
    tokenizer: &HfBpeTokenizer,
    runtime: &ScopedMemoryRuntime,
    action: &CgsAction,
    world: &CgsWorld,
    seqs: &[&[&str]],
    panel: &str,
    arm: &str,
) -> Result<CgsPanel, String> {
    let mut out = CgsPanel {
        requests: 0,
        complete: 0,
        rows: Vec::new(),
    };
    for (person_index, person) in world.people.iter().enumerate() {
        let (clause, _) = cgs_clause(tokenizer, 0, CgsForm::AskOffice, person, "")?;
        let mut session = runtime
            .ask(&clause, world.scope.as_bytes())
            .map_err(|e| format!("ask {person}: {e}"))?;
        let effects = runtime
            .run(&mut session)
            .map_err(|e| format!("run {person}: {e}"))?;
        let answer = cgs_text(tokenizer, &session);
        let expected = cgs_expectation(world, action, person_index, &[]);
        out.requests += 1;
        let matched = matches!(&expected, CgsExpected::Answer { text, reads }
                if *text == answer && *reads == (session.hop as usize) + 1)
            && session.terminal == Some(ScopedTerminal::Complete);
        if matched {
            out.complete += 1;
        }
        out.rows.push(serde_json::json!({
            "panel": panel, "arm": arm, "world": world.id, "scope": world.scope,
            "person": person, "ops": Vec::<String>::new(), "request": clause.text,
            "expected": expected.label(), "answer": answer, "terminal": session.terminal,
            "reads": session.hop + 1, "matched": matched,
            "effects": effects, "final_frame": session,
        }));
        for seq in seqs {
            let text = seq.join(" ");
            let (clause, _) = cgs_clause(tokenizer, 0, CgsForm::Compute, person, &text)?;
            let mut session = runtime
                .ask(&clause, world.scope.as_bytes())
                .map_err(|e| format!("ask compute {person}: {e}"))?;
            let effects = runtime
                .run(&mut session)
                .map_err(|e| format!("run compute {person}: {e}"))?;
            let answer = cgs_text(tokenizer, &session);
            let expected = cgs_expectation(world, action, person_index, seq);
            out.requests += 1;
            let matched = match (&expected, session.terminal) {
                (CgsExpected::Answer { text, reads }, Some(ScopedTerminal::Complete)) => {
                    answer == *text && (session.hop as usize) + 1 == *reads
                }
                (CgsExpected::Unresolved, Some(ScopedTerminal::Unresolved)) => true,
                (CgsExpected::Exhausted, Some(ScopedTerminal::Exhausted)) => true,
                (CgsExpected::UnknownOperation, Some(ScopedTerminal::UnknownOperation)) => true,
                (CgsExpected::Ungrounded, Some(ScopedTerminal::UngroundedResult)) => true,
                _ => false,
            };
            if matched {
                out.complete += 1;
            }
            out.rows.push(serde_json::json!({
                "panel": panel, "arm": arm, "world": world.id, "scope": world.scope,
                "person": person, "ops": seq, "request": clause.text,
                "expected": expected.label(), "answer": answer, "terminal": session.terminal,
                "reads": session.hop + 1, "matched": matched,
                "computed": session.computation, "effects": effects, "final_frame": session,
            }));
        }
    }
    Ok(out)
}

fn cgs_text(tokenizer: &HfBpeTokenizer, session: &ScopedSession) -> String {
    let tokens: Vec<u32> = session
        .emitted
        .iter()
        .copied()
        .filter(|t| Some(*t) != session.eos)
        .collect();
    tokenizer
        .decode(&tokens)
        .trim_matches(|c: char| c.is_ascii_whitespace())
        .to_string()
}

/// Build the declared development worlds with a deterministic person->label assignment and a mix of
/// terminal and redirect label records (so the same request form needs a content-dependent read path).
fn cgs_worlds(
    id_base: u32,
    people: &'static [&'static str],
    dests: &'static [&'static str],
    scope: &'static str,
    redirect_at: usize,
) -> Vec<CgsWorld> {
    // Preserve the actually evaluated worlds: the former equal-length test overrode
    // both development assignments because both people arrays have length four.
    let assignment = match id_base {
        4 => vec![0, 5, 2, 7],
        _ => vec![6, 1, 4, 3],
    };
    let mut office: Vec<Option<usize>> = (0..8).map(Some).collect();
    // One label redirects to another label, so a derived read can require a second hop.
    office[redirect_at] = None;
    let mut redirect = vec![0usize; 8];
    redirect[redirect_at] = (redirect_at + 1) % 8;
    vec![CgsWorld {
        id: id_base,
        version: 1,
        people,
        dests,
        assignment,
        office,
        redirect,
        person_redirect: vec![None; people.len()],
        scope,
    }]
}

/// A fresh world whose first person's office record is itself a redirect to the second person, so the
/// requested operand is only reachable by following an observed redirect before applying.
fn cgs_fresh_worlds(
    id_base: u32,
    people: &'static [&'static str],
    dests: &'static [&'static str],
    scope: &'static str,
    redirect_at: usize,
) -> Vec<CgsWorld> {
    let mut worlds = cgs_worlds(id_base, people, dests, scope, redirect_at);
    worlds[0].person_redirect[0] = Some(1);
    worlds
}

/// Deterministic document construction from the declared world. This is document preparation, not a
/// learned decision: the observed requests and their consumption are what the model must handle.
fn cgs_memory(tokenizer: &HfBpeTokenizer, world: &CgsWorld) -> Result<Memory, String> {
    let mut memory = Memory::new(world.id as u64, 16);
    let mut commit = 0u64;
    let payload = |text: &str| tokenizer.encode(&format!(" {text}"));
    let mut write = |memory: &mut Memory,
                     entity: &str,
                     value: &str,
                     continues: bool,
                     tokens: Vec<u32>|
     -> Result<(), String> {
        commit += 1;
        memory
            .write(
                world.scope.as_bytes(),
                entity.as_bytes(),
                0,
                value.as_bytes(),
                &tokens,
                commit,
                Update::Assert,
                continues,
                0,
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    };
    for (person_index, label_index) in world.assignment.iter().enumerate() {
        match world.person_redirect[person_index] {
            Some(target) => {
                let name = world.people[target];
                write(
                    &mut memory,
                    world.people[person_index],
                    name,
                    true,
                    payload(name),
                )?;
            }
            None => {
                let label = CGS_LABELS[*label_index];
                write(
                    &mut memory,
                    world.people[person_index],
                    label,
                    false,
                    payload(label),
                )?;
            }
        }
    }
    for (label_index, office) in world.office.iter().enumerate() {
        let label = CGS_LABELS[label_index];
        match office {
            Some(dest) => write(
                &mut memory,
                label,
                world.dests[*dest],
                false,
                payload(world.dests[*dest]),
            )?,
            None => {
                let target = CGS_LABELS[world.redirect[label_index]];
                write(&mut memory, label, target, true, payload(target))?;
            }
        }
    }
    memory.validate().map_err(|e| e.to_string())?;
    Ok(memory)
}

/// The declared assertions of a serving world, in the order they are observed.
fn cgs_world_facts(world: &CgsWorld) -> Vec<(CgsForm, &'static str, &'static str)> {
    let mut facts: Vec<(CgsForm, &'static str, &'static str)> = Vec::new();
    for (person_index, label_index) in world.assignment.iter().enumerate() {
        match world.person_redirect[person_index] {
            Some(target) => facts.push((
                CgsForm::RedirectOffice,
                world.people[person_index],
                world.people[target],
            )),
            None => facts.push((
                CgsForm::AssertOffice,
                world.people[person_index],
                CGS_LABELS[*label_index],
            )),
        }
    }
    for (label_index, office) in world.office.iter().enumerate() {
        let label = CGS_LABELS[label_index];
        match office {
            Some(dest) => facts.push((CgsForm::AssertOffice, label, world.dests[*dest])),
            None => facts.push((
                CgsForm::RedirectOffice,
                label,
                CGS_LABELS[world.redirect[label_index]],
            )),
        }
    }
    facts
}

/// Build one serving world by passing its **actual observed assertions** through the candidate's
/// loaded ingest path. What the panel then reads is what the learned interface actually committed,
/// not a typed construction: the raw text, tokenizer alignment, learned relation/intent and the
/// committed record are all retained as receipts. `cgs_memory` remains available as the named
/// typed-record comparator that localizes an observed-ingest failure from a computation failure.
#[allow(clippy::too_many_arguments)]
fn cgs_ingest_world(
    tokenizer: &HfBpeTokenizer,
    model: &[u8],
    intent: &[u8],
    lexicon: &[u8],
    backend: &ComputationBackend,
    control: &MemoryControl,
    world: &CgsWorld,
) -> Result<(ScopedMemoryRuntime, Vec<serde_json::Value>), String> {
    let mut runtime = ScopedMemoryRuntime::load_with_computation(
        model,
        intent,
        Some(lexicon),
        backend.clone(),
        Memory::new(world.id as u64, 16),
        world.id as u64,
        control.clone(),
        VOCAB,
        Some(CGS_EOS),
    )
    .map_err(|e| e.to_string())?;
    let mut receipts = Vec::with_capacity(16);
    for (turn, (form, entity, tail)) in cgs_world_facts(world).into_iter().enumerate() {
        let (clause, _) = cgs_clause(tokenizer, 0, form, entity, tail)?;
        let observed = runtime.observe(&clause).map_err(|e| e.to_string())?;
        let outcome = runtime
            .ingest(&clause, world.scope.as_bytes(), turn as u64 + 1)
            .map_err(|e| e.to_string())?;
        let recorded = match &outcome {
            IngestOutcome::Wrote(written) => Some(serde_json::json!({
                "id": written.id, "commit": written.commit, "superseded": written.superseded,
                "revision": written.revision, "action": format!("{:?}", written.action),
            })),
            _ => None,
        };
        receipts.push(serde_json::json!({
            "turn": turn,
            "text": clause.text,
            "declared_form": format!("{form:?}"),
            "declared_entity": entity,
            "declared_value": tail,
            "declared_continues": matches!(form, CgsForm::RedirectOffice),
            "entity": String::from_utf8_lossy(&observed.entity_key),
            "value": observed.value_key.as_ref().map(|v| String::from_utf8_lossy(v).into_owned()),
            "relation": observed.relation,
            "intent": observed.intent,
            "role": observed.role,
            "continues": observed.continues,
            "outcome": format!("{outcome:?}").chars().take(40).collect::<String>(),
            "committed": recorded,
        }));
    }
    Ok((runtime, receipts))
}

fn cgs_reload_check(dir: &std::path::Path) -> Result<ExitCode, String> {
    let model_bytes = std::fs::read(dir.join("model.json")).map_err(|e| e.to_string())?;
    let intent_bytes = std::fs::read(dir.join("intent.json")).map_err(|e| e.to_string())?;
    let lexicon_bytes = std::fs::read(dir.join("lexicon.json")).map_err(|e| e.to_string())?;
    let store_bytes = std::fs::read(dir.join("store.json")).map_err(|e| e.to_string())?;
    let request: serde_json::Value = serde_json::from_slice(
        &std::fs::read(dir.join("request.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let artifact: serde_json::Value = serde_json::from_slice(
        &std::fs::read(dir.join("artifact.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let backend = cgs_backend_from_json(&artifact)?;
    let memory = Memory::from_bytes(&store_bytes).map_err(|e| e.to_string())?;
    let lineage = memory.lineage;
    let realization_bytes = std::fs::read(dir.join("realization.json")).ok();
    let runtime = match realization_bytes.as_deref() {
        Some(realization) => ScopedMemoryRuntime::load_grounded(
            &model_bytes,
            &intent_bytes,
            Some(&lexicon_bytes),
            backend,
            Some(realization),
            OutputContract::RealizedV1,
            memory,
            lineage,
            MemoryControl::Normal,
            VOCAB,
            Some(CGS_EOS),
        ),
        None => ScopedMemoryRuntime::load_with_computation(
            &model_bytes,
            &intent_bytes,
            Some(&lexicon_bytes),
            backend,
            memory,
            lineage,
            MemoryControl::Normal,
            VOCAB,
            Some(CGS_EOS),
        ),
    }
    .map_err(|e| e.to_string())?;
    let object = request.as_object().ok_or("request must be an object")?;
    if object.keys().any(|k| k != "text" && k != "scope") {
        return Err("raw request contains fields beyond text and scope".into());
    }
    let text = request["text"].as_str().ok_or("missing raw request text")?;
    let scope = request["scope"].as_str().ok_or("missing request scope")?;
    let tb = std::fs::read(DEFAULT_TOKENIZER).map_err(|e| e.to_string())?;
    if sha256_hex(&tb) != TOKENIZER_SHA {
        return Err("tokenizer sha mismatch".into());
    }
    let tokenizer = derive_tokenizer(&tb, VOCAB).map_err(|e| e.to_string())?;
    let clause = scm_raw_clause(&tokenizer, text)?;
    let mut session = runtime
        .ask(&clause, scope.as_bytes())
        .map_err(|e| e.to_string())?;
    runtime.run(&mut session).map_err(|e| e.to_string())?;
    let snapshots: Vec<Vec<u8>> = serde_json::from_slice(
        &std::fs::read(dir.join("checkpoints.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut resumed = Vec::new();
    for bytes in snapshots {
        let mut restored = runtime.restore(&bytes).map_err(|e| e.to_string())?;
        let phase = restored.pending;
        runtime.run(&mut restored).map_err(|e| e.to_string())?;
        resumed.push(json!({"phase":phase,"matches":restored == session,
            "emitted":restored.emitted,"terminal":restored.terminal,"computed":restored.computation,"final_frame":restored}));
    }
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "emitted": session.emitted, "terminal": session.terminal, "hop": session.hop,
            "computed": session.computation, "resumed":resumed,"final_frame":session,
        }))
        .map_err(|e| e.to_string())?
    );
    Ok(ExitCode::SUCCESS)
}

/// Rebuild a computation artifact from its serialized identity (used by the fresh-process reload).
fn cgs_backend_from_json(value: &serde_json::Value) -> Result<ComputationBackend, String> {
    ComputationBackend::from_identity_json(value).map_err(|e| e.to_string())
}

/// Exposed preservation audit: both artifacts receive the same raw prior conversation scripts.
/// Errors are retained per turn; a failed parser cannot shorten the denominator.
fn cgs_prior_lifecycle(
    tokenizer: &HfBpeTokenizer,
    model: &[u8],
    intent: &[u8],
    lexicon: &[u8],
    backend: &ComputationBackend,
    artifact: &str,
) -> Result<serde_json::Value, String> {
    let mut rows = Vec::new();
    let (mut questions, mut matched, mut language_questions, mut api_questions) = (0, 0, 0, 0);
    let (mut ingest_total, mut ingest_correct, mut errors) = (0, 0, 0);
    for (name, script) in [
        ("development", scm_development_script()),
        ("exposed_regression", scm_exposed_script()),
        ("final_lexical_exposed", scm_final_script()),
    ] {
        let mut runtime = ScopedMemoryRuntime::load_with_computation(
            model,
            intent,
            Some(lexicon),
            backend.clone(),
            Memory::new(801, 8),
            801,
            MemoryControl::Normal,
            VOCAB,
            Some(CGS_EOS),
        )
        .map_err(|e| format!("preservation load {artifact}: {e}"))?;
        let mut oracle = ScmOracle::new(8);
        for (turn, op) in script.iter().enumerate() {
            match *op {
                ScmOp::Ingest {
                    scope,
                    form,
                    entity,
                    value,
                } => {
                    let text = scm_clause(tokenizer, turn as u32, form, entity, Some(value))?
                        .0
                        .text;
                    let clause = scm_raw_clause(tokenizer, &text)?;
                    let expected_write = oracle.declared(scope, form, entity, value);
                    let declared_intent = if form.is_question() {
                        form.declared_question_intent()
                    } else {
                        form.declared_statement_intent()
                    };
                    let observed = runtime.observe(&clause);
                    let observation_ok = observed.as_ref().is_ok_and(|o| {
                        o.is_question == form.is_question()
                            && Some(o.intent) == declared_intent
                            && o.relation == form.relation()
                            && o.entity_key == entity.as_bytes()
                            && o.continues == form.is_redirect()
                            && (!expected_write || o.value_key.as_deref() == Some(value.as_bytes()))
                    });
                    let observed_json = match &observed {
                        Ok(o) => json!({"entity":String::from_utf8_lossy(&o.entity_key),
                            "value":o.value_key.as_ref().map(|v|String::from_utf8_lossy(v).into_owned()),
                            "relation":o.relation,"intent":o.intent,"is_question":o.is_question,"continues":o.continues}),
                        Err(e) => json!({"error":e.to_string()}),
                    };
                    let outcome =
                        runtime.ingest(&clause, SCM_SCOPES[scope as usize].as_bytes(), turn as u64);
                    let wrote = matches!(&outcome, Ok(IngestOutcome::Wrote(_)));
                    let ok = observation_ok && outcome.is_ok() && wrote == expected_write;
                    ingest_total += 1;
                    ingest_correct += usize::from(ok);
                    errors += usize::from(outcome.is_err());
                    rows.push(json!({"artifact":artifact,"script":name,"turn":turn,"kind":"ingest",
                        "text":text,"scope":SCM_SCOPES[scope as usize],"expected_write":expected_write,
                        "observed":observed_json,"wrote":wrote,"observation_and_write_match":ok,
                        "error":outcome.as_ref().err().map(|e|e.to_string())}));
                }
                _ => {
                    let (scope, entity, relation, history, kind, text, start) = match *op {
                        ScmOp::Ask {
                            scope,
                            form,
                            entity,
                        } => {
                            let text = scm_clause(tokenizer, turn as u32, form, entity, None)?
                                .0
                                .text;
                            let clause = scm_raw_clause(tokenizer, &text)?;
                            let start = runtime.ask(&clause, SCM_SCOPES[scope as usize].as_bytes());
                            language_questions += 1;
                            (
                                scope,
                                entity,
                                form.relation(),
                                form.history().ok_or("history missing")?,
                                "ask",
                                text,
                                start,
                            )
                        }
                        ScmOp::AskView {
                            scope,
                            entity,
                            relation,
                            history,
                        } => {
                            api_questions += 1;
                            let start = runtime.ask_view(
                                SCM_SCOPES[scope as usize].as_bytes(),
                                relation,
                                entity.as_bytes(),
                                history,
                            );
                            (
                                scope,
                                entity,
                                relation,
                                history,
                                "ask_view",
                                format!("exact {history:?} of {entity:?} relation {relation}"),
                                start,
                            )
                        }
                        ScmOp::Ingest { .. } => return Err("unexpected ingest branch".into()),
                    };
                    let expected = oracle.answer(scope, entity, relation, history, oracle.commit);
                    let actual = start.and_then(|mut session| {
                        let effects = runtime.run(&mut session)?;
                        Ok((session, effects))
                    });
                    questions += 1;
                    let (ok, answer, terminal, frame, effects, error) = match actual {
                        Ok((session, effects)) => {
                            let answer = cgs_text(tokenizer, &session);
                            let ok = match (&expected, session.terminal) {
                                (
                                    ScmExpected::Complete { value, hops },
                                    Some(ScopedTerminal::Complete),
                                ) => answer == *value && session.hop + 1 == *hops,
                                (_, Some(terminal)) => terminal == scm_terminal_of(&expected),
                                _ => false,
                            };
                            (
                                ok,
                                answer,
                                serde_json::to_value(session.terminal)
                                    .map_err(|e| e.to_string())?,
                                serde_json::to_value(session).map_err(|e| e.to_string())?,
                                serde_json::to_value(effects).map_err(|e| e.to_string())?,
                                None,
                            )
                        }
                        Err(e) => {
                            errors += 1;
                            (
                                false,
                                String::new(),
                                json!(null),
                                json!(null),
                                json!([]),
                                Some(e.to_string()),
                            )
                        }
                    };
                    matched += usize::from(ok);
                    rows.push(json!({"artifact":artifact,"script":name,"turn":turn,"kind":kind,
                        "text":text,"scope":SCM_SCOPES[scope as usize],"expected":format!("{expected:?}"),
                        "matched":ok,"answer":answer,"terminal":terminal,"final_frame":frame,
                        "effects":effects,"error":error}));
                }
            }
        }
    }
    Ok(
        json!({"artifact":artifact,"questions":questions,"matched":matched,
        "language_questions":language_questions,"api_questions":api_questions,
        "ingest_total":ingest_total,"ingest_observation_and_write_correct":ingest_correct,
        "errors":errors,"all_preserved":matched==questions && ingest_correct==ingest_total && errors==0,
        "rows":rows}),
    )
}

fn cgs_run() -> Result<ExitCode, String> {
    let mut root = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/consumed-geometric-state-1",
    );
    let mut source_root: Option<PathBuf> = None;
    let mut reload_dir: Option<PathBuf> = None;
    {
        let mut a = std::env::args().skip(1);
        while let Some(k) = a.next() {
            match k.as_str() {
                "--root" => root = PathBuf::from(a.next().ok_or("--root value")?),
                "--source-root" => source_root = Some(PathBuf::from(a.next().ok_or("value")?)),
                "--reload-check" => reload_dir = Some(PathBuf::from(a.next().ok_or("value")?)),
                other if other.starts_with("--mode") => {}
                other => return Err(format!("unknown argument {other}")),
            }
        }
    }
    if let Some(dir) = reload_dir {
        return cgs_reload_check(&dir);
    }
    claim(&root).map_err(|e| format!("claim {}: {e}", root.display()))?;
    let started = Instant::now();
    let tb = std::fs::read(DEFAULT_TOKENIZER).map_err(|e| format!("tokenizer: {e}"))?;
    if sha256_hex(&tb) != TOKENIZER_SHA {
        return Err("tokenizer sha mismatch".into());
    }
    let derived = sha256_hex(&derive_tokenizer_json(&tb, VOCAB).map_err(|e| format!("{e}"))?);
    if derived != DERIVED_SHA {
        return Err("derived tokenizer sha mismatch".into());
    }
    let tokenizer = derive_tokenizer(&tb, VOCAB).map_err(|e| format!("derive: {e}"))?;
    let source_files: Vec<serde_json::Value> = match &source_root {
        Some(sr) => [
            "crates/uor-r4-core/src/native_geometric/learner/scoped_memory.rs",
            "crates/uor-r4-core/src/native_geometric/learner/grounded_session.rs",
            "crates/uor-r4-core/src/bin/competitive-reader.rs",
        ]
        .iter()
        .map(|rel| match std::fs::read(sr.join(rel)) {
            Ok(b) => serde_json::json!({"path": rel, "sha256": sha256_hex(&b)}),
            Err(_) => serde_json::json!({"path": rel, "sha256": "UNAVAILABLE"}),
        })
        .collect(),
        None => vec![serde_json::json!({"path": "unspecified", "sha256": "UNAVAILABLE"})],
    };
    let executable_sha256 = hex_of(&sha256_bytes(
        &std::env::current_exe()
            .ok()
            .and_then(|p| std::fs::read(p).ok())
            .unwrap_or_default(),
    ));

    // ---- the declared action, its constructive recovery and the matched comparator ----
    let action = CgsAction::build()?;
    let examples = cgs_examples(&action)?;
    let (factorization, factor_report) =
        factor_observed_graph(&examples, false).map_err(|e| format!("factorization: {e}"))?;
    let folded_group = FoldedGroup::recover(&factorization).map_err(|e| format!("fold: {e}"))?;
    // The matched comparator is fitted with the project's standard shared-transition recipe.
    let (finite_model, finite_report) =
        fit_shared_transition(&examples, &examples, 8, 12, false, 16, ST_SEED);
    let finite_summary = serde_json::json!({
        "initial_complete": finite_report.initial_complete,
        "final_complete": finite_report.final_complete,
        "final_tokens": finite_report.final_tokens,
        "token_total": finite_report.token_total,
        "examples": finite_report.examples,
        "states": finite_report.states,
        "accepted_moves": finite_report.accepted_moves,
    });
    let finite_shape = (
        finite_model.value_domain.len(),
        finite_model.action_domain.len(),
    );
    let signed = ComputationBackend::Signed(Box::new(factorization.clone()));
    let folded =
        ComputationBackend::Folded(Box::new(factorization.clone()), Box::new(folded_group));
    let finite = ComputationBackend::Finite(Box::new(finite_model));
    let tabulated = ComputationBackend::Tabulated(Box::new(
        TabulatedControl::fit(&examples).map_err(|e| format!("tabulated control: {e}"))?,
    ));

    // Every compared computation backend is independently serialized and loaded below.
    for backend in [&signed, &folded, &finite, &tabulated] {
        write_json(
            &root,
            &format!("artifacts/backend-{}.json", backend.name()),
            &backend.identity_json(),
        )?;
    }

    // ---- the learned lexicon over exact lexical bytes ----
    let mut surface: Vec<(Vec<u8>, u32)> = CGS_OPS
        .iter()
        .map(|(word, id)| (word.as_bytes().to_vec(), *id))
        .collect();
    for (index, label) in CGS_LABELS.iter().enumerate() {
        surface.push((label.as_bytes().to_vec(), index as u32));
    }
    let canonical: Vec<(u32, Vec<u8>)> = CGS_LABELS
        .iter()
        .enumerate()
        .map(|(index, label)| (index as u32, label.as_bytes().to_vec()))
        .collect();
    let lexicon = GroundingLexicon::from_observations(&surface, &canonical)
        .map_err(|e| format!("lexicon: {e}"))?;

    // ---- the learned binder and intent tables: ONE combined bundle over both form families ----
    let (dev_clauses, supervision) = combined_development_supervision(&tokenizer)?;
    let labels: Vec<ClauseLabel> = supervision.iter().map(|s| s.label.clone()).collect();
    let (model, fit) = fit_observed_text_model(&dev_clauses, &labels, false)
        .map_err(|e| format!("binder fit: {e}"))?;
    let cue_spans: Vec<(usize, usize)> = supervision
        .iter()
        .map(|s| (s.label.marker.0 as usize, s.label.marker.1))
        .collect();
    let intent_examples: Vec<IntentExample> = supervision
        .iter()
        .map(|s| IntentExample {
            question: s.question,
            intent: s.intent,
        })
        .collect();
    let (intent, intent_fit) = fit_intent_model(&dev_clauses, &cue_spans, &intent_examples, VOCAB)
        .map_err(|e| format!("intent fit: {e}"))?;

    // ---- retained ablation: the submitted design fitted the same two heads on the computation
    // forms alone. Fitting both on identical inputs makes "combined support" a measured change, not
    // an assertion, and keeps the previous candidate's scope reproducible. ----
    let (cgs_only_clauses, cgs_only_sup) = cgs_development_supervision(&tokenizer, 0)?;
    let cgs_only_labels: Vec<ClauseLabel> = cgs_only_sup.iter().map(|s| s.label.clone()).collect();
    let (cgs_only_model, cgs_only_fit) =
        fit_observed_text_model(&cgs_only_clauses, &cgs_only_labels, false)
            .map_err(|e| format!("cgs-only binder fit: {e}"))?;
    let cgs_only_spans: Vec<(usize, usize)> = cgs_only_sup
        .iter()
        .map(|s| (s.label.marker.0 as usize, s.label.marker.1))
        .collect();
    let cgs_only_examples: Vec<IntentExample> = cgs_only_sup
        .iter()
        .map(|s| IntentExample {
            question: s.question,
            intent: s.intent,
        })
        .collect();
    let (cgs_only_intent, cgs_only_intent_fit) = fit_intent_model(
        &cgs_only_clauses,
        &cgs_only_spans,
        &cgs_only_examples,
        VOCAB,
    )
    .map_err(|e| format!("cgs-only intent fit: {e}"))?;
    let cgs_only_model_bytes = cgs_only_model.to_bytes().map_err(|e| format!("{e}"))?;
    let cgs_only_intent_bytes = cgs_only_intent.to_bytes().map_err(|e| format!("{e}"))?;
    let model_bytes = model.to_bytes().map_err(|e| format!("{e}"))?;
    let intent_bytes = intent.to_bytes().map_err(|e| format!("{e}"))?;
    let lexicon_bytes = lexicon.to_bytes().map_err(|e| format!("{e}"))?;
    write_checked(&root, "artifacts/model.json", &model_bytes)?;
    write_checked(&root, "artifacts/intent.json", &intent_bytes)?;
    write_checked(&root, "artifacts/lexicon.json", &lexicon_bytes)?;

    // ---- the learned grounded lexical realization policy ----
    // One compact finite context-indexed table fitted from declared development response text.
    // `realization` is the candidate; `realization-context-disabled` is the falsifier ablation that
    // holds the owned-evidence and derived flags at zero, so the vocabulary effect cannot depend on
    // them. Both are fitted on identical examples and differ only in those flags.
    let real_slots = cgs_real_slots(&tokenizer)?;
    let real_examples = cgs_realization_examples(&tokenizer, &real_slots)?;
    let realization = fit_realization(&real_examples, VOCAB, real_slots, 8)
        .map_err(|e| format!("realization fit: {e}"))?;
    let realization_bytes = realization.to_bytes().map_err(|e| format!("{e}"))?;
    write_checked(&root, "artifacts/realization.json", &realization_bytes)?;
    let realization_reloaded = RealizationModel::from_bytes(
        &std::fs::read(root.join("artifacts/realization.json")).map_err(|e| e.to_string())?,
        VOCAB,
    )
    .map_err(|e| format!("{e}"))?;
    if realization_reloaded != realization {
        return Err("realization artifact reload mismatch".into());
    }
    write_json(
        &root,
        "artifacts/artifact.json",
        &backend_identity_json(&signed),
    )?;
    let model_reloaded = ObservedTextModel::from_bytes(
        &std::fs::read(root.join("artifacts/model.json")).map_err(|e| e.to_string())?,
        VOCAB,
    )
    .map_err(|e| format!("{e}"))?;
    if model_reloaded != model {
        return Err("model artifact reload mismatch".into());
    }
    let intent_reloaded = IntentModel::from_bytes(
        &std::fs::read(root.join("artifacts/intent.json")).map_err(|e| e.to_string())?,
        VOCAB,
    )
    .map_err(|e| format!("{e}"))?;
    if intent_reloaded != intent {
        return Err("intent artifact reload mismatch".into());
    }

    // ---- worlds ----
    let dev_worlds = cgs_worlds(0, &CGS_DEV_PEOPLE, &CGS_DEV_DESTS, "alpha", 5);
    let dev_world_beta = cgs_worlds(1, &CGS_DEV_PEOPLE, &CGS_DEV_DESTS, "beta", 2);
    let final_worlds = cgs_worlds(4, &CGS_FINAL_PEOPLE, &CGS_FINAL_DESTS, "alpha", 3);
    let final_world_beta = cgs_worlds(5, &CGS_FINAL_PEOPLE, &CGS_FINAL_DESTS, "beta", 6);
    // A genuinely fresh withheld population: disjoint strings, an observed person-level redirect
    // before the operand, and three-operation orders absent from every earlier population.
    let fresh_worlds = cgs_fresh_worlds(8, &CGS_FRESH_PEOPLE, &CGS_FRESH_DESTS, "alpha", 6);
    let fresh_world_beta = cgs_fresh_worlds(9, &CGS_FRESH_PEOPLE, &CGS_FRESH_DESTS, "beta", 1);

    // Every compared backend is loaded from its own serialized identity before it is used.
    let load_backend = |backend: &ComputationBackend| -> Result<ComputationBackend, String> {
        let serialized: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("artifacts/backend-{}.json", backend.name())))
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        cgs_backend_from_json(&serialized)
    };
    let load = |memory: Memory,
                control: MemoryControl,
                backend: &ComputationBackend|
     -> Result<ScopedMemoryRuntime, String> {
        let lineage = memory.lineage;
        let loaded_backend = load_backend(backend)?;
        ScopedMemoryRuntime::load_with_computation(
            &model_bytes,
            &intent_bytes,
            Some(&lexicon_bytes),
            loaded_backend,
            memory,
            lineage,
            control,
            VOCAB,
            Some(CGS_EOS),
        )
        .map_err(|e| e.to_string())
    };
    // The primary serving world is built by the candidate's own loaded ingest path.
    let ingest_world = |world: &CgsWorld,
                        control: &MemoryControl,
                        backend: &ComputationBackend|
     -> Result<(ScopedMemoryRuntime, Vec<serde_json::Value>), String> {
        let loaded_backend = load_backend(backend)?;
        cgs_ingest_world(
            &tokenizer,
            &model_bytes,
            &intent_bytes,
            &lexicon_bytes,
            &loaded_backend,
            control,
            world,
        )
    };

    // The cheap actual-loaded mixed-session check, before the preservation/final campaigns.
    let mixed = cgs_mixed_session(
        &tokenizer,
        &model_bytes,
        &intent_bytes,
        &lexicon_bytes,
        &load_backend(&signed)?,
        &action,
    )?;

    let mut arms = Vec::<serde_json::Value>::new();
    let mut all_rows = Vec::<serde_json::Value>::new();
    let mut panels = Vec::<serde_json::Value>::new();
    let mut ingest_receipts = Vec::<serde_json::Value>::new();
    let mut typed_arm = Vec::<serde_json::Value>::new();
    let mut ingest_disagreements = Vec::<serde_json::Value>::new();
    for (panel, worlds, seqs) in [
        (
            "development",
            vec![&dev_worlds[0], &dev_world_beta[0]],
            CGS_DEV_SEQS.to_vec(),
        ),
        (
            "final",
            vec![&final_worlds[0], &final_world_beta[0]],
            CGS_FINAL_SEQS.to_vec(),
        ),
        (
            "fresh_withheld",
            vec![&fresh_worlds[0], &fresh_world_beta[0]],
            CGS_FRESH_SEQS.to_vec(),
        ),
    ] {
        let (mut requests, mut complete) = (0usize, 0usize);
        for world in worlds {
            // Primary: the world itself is constructed by the candidate's loaded ingest path.
            let (runtime, receipts) = ingest_world(world, &MemoryControl::Normal, &signed)?;
            ingest_receipts.extend(receipts);
            let result = cgs_panel(
                &tokenizer,
                &runtime,
                &action,
                world,
                &seqs,
                panel,
                "primary_signed",
            )?;
            requests += result.requests;
            complete += result.complete;
            // Named comparator arm on the identical requests, to localize an observed-ingest
            // failure from a computation/consumption failure.
            let typed = load(
                cgs_memory(&tokenizer, world)?,
                MemoryControl::Normal,
                &signed,
            )?;
            let typed_result = cgs_panel(
                &tokenizer,
                &typed,
                &action,
                world,
                &seqs,
                panel,
                "typed_records_only",
            )?;
            typed_arm.push(serde_json::json!({
                "panel": panel, "world": world.id, "requests": typed_result.requests,
                "complete": typed_result.complete,
            }));
            // Row-by-row agreement between the two constructions.
            for (a, b) in result.rows.iter().zip(typed_result.rows.iter()) {
                if a["matched"] != b["matched"] || a["answer"] != b["answer"] {
                    ingest_disagreements.push(serde_json::json!({
                        "panel": panel, "world": world.id, "request": a["request"],
                        "learned": a["answer"], "typed": b["answer"],
                    }));
                }
            }
            all_rows.extend(result.rows);
        }
        panels.push(serde_json::json!({
            "panel": panel, "arm": "primary_signed",
            "requests": requests, "complete": complete,
        }));
    }

    // ---- matched controls and comparator arms on the same inputs ----
    let mut controls = Vec::<serde_json::Value>::new();
    for (name, control, backend) in [
        ("no_read", MemoryControl::NoRead, &signed),
        ("apply_disabled", MemoryControl::ApplyDisabled, &signed),
        ("consume_disabled", MemoryControl::ConsumeDisabled, &signed),
        ("unscoped", MemoryControl::Unscoped, &signed),
        ("finite_transition", MemoryControl::Normal, &finite),
        ("tabulated_finite", MemoryControl::Normal, &tabulated),
        ("folded_central_sign", MemoryControl::Normal, &folded),
    ] {
        let mut requests = 0usize;
        let mut complete = 0usize;
        for world in [&dev_worlds[0], &final_worlds[0]] {
            let (runtime, _) = ingest_world(world, &control, backend)?;
            let seqs: Vec<&[&str]> = if world.id >= 4 {
                CGS_FINAL_SEQS.to_vec()
            } else {
                CGS_DEV_SEQS.to_vec()
            };
            let panel = cgs_panel(&tokenizer, &runtime, &action, world, &seqs, "control", name)?;
            requests += panel.requests;
            complete += panel.complete;
            all_rows.extend(panel.rows);
        }
        controls.push(serde_json::json!({"arm": name, "requests": requests, "complete": complete}));
    }

    // ---- causal comparisons on one committed development world ----
    let dev_world = &dev_worlds[0];
    let dev_memory = cgs_memory(&tokenizer, dev_world)?;
    let store_bytes = dev_memory.to_bytes().map_err(|e| e.to_string())?;
    let rt = load(dev_memory.clone(), MemoryControl::Normal, &signed)?;
    let person = dev_world.people[0];
    let scope = dev_world.scope;
    let q_ij = cgs_clause(&tokenizer, 0, CgsForm::Compute, person, "i j")?.0;
    let q_ji = cgs_clause(&tokenizer, 0, CgsForm::Compute, person, "j i")?.0;
    let q_e = cgs_clause(&tokenizer, 0, CgsForm::Compute, person, "e")?.0;
    let (a_ij, t_ij, c_ij, s_ij) = cgs_ask(&tokenizer, &rt, &q_ij, scope)?;
    let (a_ji, _, c_ji, _) = cgs_ask(&tokenizer, &rt, &q_ji, scope)?;
    let (_, _, c_e, _) = cgs_ask(&tokenizer, &rt, &q_e, scope)?;

    // (1) A changed operand changes the computed key and the complete answer.
    let mut changed_world = dev_world.clone();
    changed_world.assignment[0] = (changed_world.assignment[0] + 1) % 8;
    let changed_rt = load(
        cgs_memory(&tokenizer, &changed_world)?,
        MemoryControl::Normal,
        &signed,
    )?;
    let (a_changed, _, c_changed, _) = cgs_ask(&tokenizer, &changed_rt, &q_ij, scope)?;

    // (2) A changed operation order changes the retained state; the central-sign fold does not.
    let folded_rt = load(dev_memory.clone(), MemoryControl::Normal, &folded)?;
    let (f_ij, _, fc_ij, _) = cgs_ask(&tokenizer, &folded_rt, &q_ij, scope)?;
    let (f_ji, _, fc_ji, _) = cgs_ask(&tokenizer, &folded_rt, &q_ji, scope)?;

    // (3) Identity is the correct negative control: the retained state is unchanged.
    let identity_ok = c_e
        .as_ref()
        .is_some_and(|c| c.operand_state == c.state && c.derived_key == c.operand_key);

    // (4) Consumption disabled keeps the computed state but does not use it.
    let skip_rt = load(dev_memory.clone(), MemoryControl::ConsumeDisabled, &signed)?;
    let (a_skip, t_skip, c_skip, _) = cgs_ask(&tokenizer, &skip_rt, &q_ij, scope)?;

    // (5) Owned resume across the computation: snapshot after the first Apply and finish elsewhere.
    let mut partial = rt
        .ask(&q_ij, scope.as_bytes())
        .map_err(|e| format!("resume ask: {e}"))?;
    for _ in 0..8 {
        let effect = rt
            .step(&mut partial)
            .map_err(|e| format!("resume step: {e}"))?;
        if effect.action == SessionAction::Apply {
            break;
        }
    }
    let mid_state = partial.computation.clone();
    let snapshot = rt
        .snapshot(&partial)
        .map_err(|e| format!("snapshot: {e}"))?;
    let fresh_rt = load(
        Memory::from_bytes(&store_bytes).map_err(|e| e.to_string())?,
        MemoryControl::Normal,
        &signed,
    )?;
    let mut restored = fresh_rt
        .restore(&snapshot)
        .map_err(|e| format!("restore: {e}"))?;
    fresh_rt
        .run(&mut restored)
        .map_err(|e| format!("resume run: {e}"))?;
    let resume_ok = restored == s_ij;

    // (6) A real disk reload and a separate process reproduce the same answer.
    let reload_dir = root.join("reload");
    std::fs::create_dir_all(&reload_dir).map_err(|e| format!("{e}"))?;
    write_checked(&root, "reload/model.json", &model_bytes)?;
    write_checked(&root, "reload/intent.json", &intent_bytes)?;
    write_checked(&root, "reload/lexicon.json", &lexicon_bytes)?;
    write_checked(&root, "reload/store.json", &store_bytes)?;
    write_json(
        &root,
        "reload/artifact.json",
        &backend_identity_json(&signed),
    )?;
    write_json(
        &root,
        "reload/request.json",
        &serde_json::json!({"text": q_ij.text, "scope": scope}),
    )?;
    let mut phase = rt.ask(&q_ij, scope.as_bytes()).map_err(|e| e.to_string())?;
    let mut snapshots = Vec::<Vec<u8>>::new();
    loop {
        snapshots.push(rt.snapshot(&phase).map_err(|e| e.to_string())?);
        if phase.terminal.is_some() {
            break;
        }
        rt.step(&mut phase).map_err(|e| e.to_string())?;
    }
    write_json(&root, "reload/checkpoints.json", &json!(snapshots))?;
    let disk_memory = Memory::from_bytes(
        &std::fs::read(root.join("reload/store.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{e}"))?;
    let disk_rt = load(disk_memory, MemoryControl::Normal, &signed)?;
    let (a_disk, t_disk, c_disk, s_disk) = cgs_ask(&tokenizer, &disk_rt, &q_ij, scope)?;
    let disk_ok = a_disk == a_ij && t_disk == t_ij && c_disk == c_ij && s_disk == s_ij;
    let child = std::process::Command::new(
        std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?,
    )
    .arg("--mode=consumed-geometric-state")
    .arg("--reload-check")
    .arg(&reload_dir)
    .output()
    .map_err(|e| format!("fresh process: {e}"))?;
    let child_json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&child.stdout).trim())
            .map_err(|e| format!("child json: {e}"))?;
    let child_ok = child.status.success()
        && child_json["emitted"] == serde_json::json!(s_ij.emitted)
        && child_json["terminal"]
            == serde_json::to_value(s_ij.terminal).map_err(|e| e.to_string())?
        && child_json["final_frame"] == json!(s_ij)
        && child_json["computed"] == json!(s_ij.computation)
        && child_json["hop"] == json!(s_ij.hop)
        && child_json["resumed"].as_array().is_some_and(|rs| {
            rs.len() == snapshots.len() && rs.iter().all(|r| r["matches"] == json!(true))
        });

    // ---- ordinary memory lifecycle through the learned path (preservation panel) ----
    let ordinary = cgs_ordinary(
        &tokenizer,
        &model_bytes,
        &intent_bytes,
        &lexicon_bytes,
        &signed,
    )?;

    // Test the actual new artifact against every prior primary interaction, not the two-assertion smoke.
    let preservation = cgs_prior_lifecycle(
        &tokenizer,
        &model_bytes,
        &intent_bytes,
        &lexicon_bytes,
        &signed,
        "new_computation_artifact",
    )?;
    // The submitted design's ablation on the identical prior panel: computation-forms-only
    // supervision, so "combined support" is measured rather than asserted.
    let ablation = cgs_prior_lifecycle(
        &tokenizer,
        &cgs_only_model_bytes,
        &cgs_only_intent_bytes,
        &lexicon_bytes,
        &signed,
        "ablation_computation_forms_only",
    )?;
    let realization_report = cgs_realized_section(
        &tokenizer,
        &root,
        &model_bytes,
        &intent_bytes,
        &lexicon_bytes,
        &realization_bytes,
        &signed,
        &realization,
        &real_examples,
        &dev_worlds[0],
    )?;
    let prior_root = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/scoped-memory-principal-3",
    );
    let prior_model =
        std::fs::read(prior_root.join("artifacts/binder.json")).map_err(|e| e.to_string())?;
    let prior_intent =
        std::fs::read(prior_root.join("artifacts/intent.json")).map_err(|e| e.to_string())?;
    let prior_preservation = cgs_prior_lifecycle(
        &tokenizer,
        &prior_model,
        &prior_intent,
        &lexicon_bytes,
        &signed,
        "prior_v1_intent_migrated",
    )?;
    write_json(
        &root,
        "preservation.json",
        &json!({"candidate":preservation,"prior_migrated":prior_preservation,
        "ablation_computation_forms_only":ablation,
        "prior_model_sha256":sha256_hex(&prior_model),"prior_intent_sha256":sha256_hex(&prior_intent)}),
    )?;

    let consumption = serde_json::json!({
        "rows": panels.iter().map(|p| p["complete"].as_u64().unwrap_or(0)).sum::<u64>(),
        "requests": panels.iter().map(|p| p["requests"].as_u64().unwrap_or(0)).sum::<u64>(),
        "changed_operand": {"base": a_ij, "changed": a_changed,
            "base_key": c_ij.as_ref().map(|c| String::from_utf8_lossy(&c.derived_key).into_owned()),
            "changed_key": c_changed.as_ref().map(|c| String::from_utf8_lossy(&c.derived_key).into_owned())},
        "changed_order": {"i_then_j": a_ij, "j_then_i": a_ji,
            "ij_key": c_ij.as_ref().map(|c| String::from_utf8_lossy(&c.derived_key).into_owned()),
            "ji_key": c_ji.as_ref().map(|c| String::from_utf8_lossy(&c.derived_key).into_owned()),
            "folded_i_then_j": f_ij, "folded_j_then_i": f_ji,
            "folded_ij_key": fc_ij.as_ref().map(|c| String::from_utf8_lossy(&c.derived_key).into_owned()),
            "folded_ji_key": fc_ji.as_ref().map(|c| String::from_utf8_lossy(&c.derived_key).into_owned())},
        "identity": {"key": c_e.as_ref().map(|c| String::from_utf8_lossy(&c.derived_key).into_owned()),
            "operand_state": c_e.as_ref().map(|c| c.operand_state),
            "state": c_e.as_ref().map(|c| c.state)},
        "consume_disabled": {"answer": a_skip, "terminal": t_skip,
            "consumed": c_skip.as_ref().map(|c| c.consumed)},
        "resume": {"mid": mid_state, "identical": resume_ok},
        "reload": {"disk": disk_ok, "fresh_process": child_ok,
            "child_status": child.status.code(),"raw_text_entry":true,"child":child_json},
    });

    // Every observed world assertion must be committed by the learned ingest path with the declared
    // entity/value/relation/continuation ownership.
    let ingest_ok = ingest_receipts.iter().all(|r| {
        r["committed"].is_object()
            && r["entity"] == r["declared_entity"]
            && r["value"] == r["declared_value"]
            && r["continues"] == r["declared_continues"]
            && r["relation"] == json!(0)
            && r["intent"] == json!(STMT_ASSERT)
    });
    let checks = serde_json::json!({
        "development_complete": panels[0]["complete"] == panels[0]["requests"],
        "final_complete": panels[1]["complete"] == panels[1]["requests"],
        "fresh_withheld_complete": panels[2]["complete"] == panels[2]["requests"],
        "mixed_session": mixed["ok"] == json!(true),
        "consumption_necessary": a_changed != a_ij,
        "order_changes_state": c_ij.as_ref().map(|c| c.derived_key.clone())
            != c_ji.as_ref().map(|c| c.derived_key.clone()),
        "fold_loses_the_order_distinction": fc_ij.as_ref().map(|c| c.derived_key.clone())
            == fc_ji.as_ref().map(|c| c.derived_key.clone()),
        "identity_unchanged": identity_ok,
        "consume_disabled_changes_answer": a_skip != a_ij && t_skip == t_ij
            && !c_skip.as_ref().is_some_and(|c| c.consumed)
            && c_skip.as_ref().map(|c| c.state) == c_ij.as_ref().map(|c| c.state),
        "owned_resume_identical": resume_ok,
        "disk_reload_identical": disk_ok,
        "fresh_process_identical": child_ok,
        "ordinary_smoke": ordinary["ok"] == serde_json::json!(true),
        "prior_model_migration_preserves_lifecycle": prior_preservation["all_preserved"] == json!(true),
        // The milestone: the same candidate that consumes the computation must retain the earlier
        // correction/history/relationship lifecycle, and its primary worlds must be built by the
        // learned ingest path rather than by typed construction.
        "combined_support_restores_prior_lifecycle": preservation["all_preserved"] == json!(true),
        "learned_ingest_commits_every_assertion": ingest_ok,
        "learned_ingest_matches_typed_records": ingest_disagreements.is_empty(),
        // Falsifier for attributing the repair to combined support: the submitted
        // computation-forms-only supervision must *not* restore the prior lifecycle.
        "ablation_computation_forms_only_does_not_restore_prior_lifecycle":
            ablation["all_preserved"] == json!(false),
        // The milestone: a learned evidence-conditioned lexical interface in the *same* session.
        "grounded_realization_all_expected": realization_report["checks_all_expected"] == json!(true),
    });
    let all_expected = checks
        .as_object()
        .map(|m| m.values().all(|v| v == &serde_json::json!(true)))
        .unwrap_or(false);
    let result = serde_json::json!({
        "schema": "uor-r4.consumed-geometric-state/2",
        "evaluation_scope": "Corrected exposed replay; selected final first appeared before design/comparator selection. Prior same-artifact lifecycle measured separately, not included in computation-component success.",
        "same_artifact_lifecycle_preserved": preservation["all_preserved"],
        "same_artifact_lifecycle_matches": preservation["matched"],
        "integration_complete": all_expected && preservation["all_preserved"] == json!(true),
        "base": "origin/main b5d36c9b8318e4fe2084ef49cc75cff55c1b2b6f",
        "running_source": {
            "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
            "git_dirty": std::env::var("UOR_GIT_DIRTY").unwrap_or_else(|_| "unknown".into()),
            "executable_sha256": executable_sha256,
            "source_files": source_files,
        },
        "inputs": {"tokenizer_derived": DERIVED_SHA, "vocab": VOCAB, "eos": CGS_EOS},
        "task": {
            "serving_input": "observed readable requests (an ordinary memory question or an observed operation sequence) over a scoped document; no gold operation, state, derived key or expected answer",
            "labels": CGS_LABELS,
            "operations": CGS_OPS.iter().map(|(w, id)| format!("{w}:{id}")).collect::<Vec<_>>(),
            "development_people": CGS_DEV_PEOPLE,
            "development_dests": CGS_DEV_DESTS,
            "final_people": CGS_FINAL_PEOPLE,
            "final_dests": CGS_FINAL_DESTS,
            "development_sequences": CGS_DEV_SEQS,
            "final_sequences": CGS_FINAL_SEQS,
            "fresh_people": CGS_FRESH_PEOPLE,
            "fresh_dests": CGS_FRESH_DESTS,
            "fresh_sequences": CGS_FRESH_SEQS,
            "fresh_novelty": {
                "lexical": "fresh people and destinations are disjoint from every earlier population",
                "operation_order": "three-operation orders absent from development and from the earlier exposed final",
                "routing_depth": "the first person's office record is itself an observed redirect, so the requested operand needs one followed read before the operation starts; that composition is absent from fitting",
                "syntax": "the observed clause forms are the familiar declared forms; no new surface syntax is introduced",
            },
            "novelty": "the final people and destinations are disjoint from fitting and the withheld ordered combinations never appear in development; the label and operation vocabularies are familiar by design",
        },
        "learning": {"binder": fit, "intent": intent_fit, "factorization": factor_report,
            "finite_control_shape": {"values": finite_shape.0, "actions": finite_shape.1},
            "finite_control_fit": finite_summary,
            "ablation_computation_forms_only": {"binder": cgs_only_fit, "intent": cgs_only_intent_fit}},
        "learned_ingest": {
            "path": "ScopedMemoryRuntime::observe + ingest over the observed assertion text",
            "receipts": ingest_receipts,
            "all_committed": ingest_ok,
        },
        "localization": {
            "typed_records_only": typed_arm,
            "learned_vs_typed_disagreements": ingest_disagreements,
        },
        "panels": panels,
        "controls": controls,
        "consumption": consumption,
        "mixed_session": mixed,
        "grounded_realization": realization_report,
        "ordinary_lifecycle": ordinary,
        "all_backend_arms_loaded_from_saved_artifacts": true,
        "checks": checks,
        "checks_all_expected": all_expected,
        "all_rows": all_rows.len(),
        "elapsed_s": started.elapsed().as_secs_f64(),
        "scope": "authored ordinary readable requests over a small declared state world; one loaded path consumes a shared finite computation whose grounded result selects the next exact read. Not broad language understanding, general reasoning, frontier capability or a whole-program dictionary. Energy UNAVAILABLE; whole-path D0-b not claimed.",
    });
    write_checked(
        &root,
        "rows.jsonl",
        all_rows
            .iter()
            .map(|r| serde_json::to_string(r).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
            .as_bytes(),
    )?;
    cgs_finish_run(&tokenizer, &root, started, result, all_expected)
}

fn backend_identity_json(backend: &ComputationBackend) -> serde_json::Value {
    backend.identity_json()
}

// ---------------------------------------------------------------------------
// Learned grounded lexical realization: one learned Copy/Vocabulary/Stop policy
// emits shared vocabulary words around the exact owned span, with the word chosen
// from causal owned evidence rather than from the recent request
// ---------------------------------------------------------------------------

/// Declared development realization vocabulary: one learned shared slot per word. Slot 0 is a shared
/// opener every declared response starts with; the other slots are the *uncopied* connector words
/// whose selection is exactly what the learned policy has to make. Each must be one token of the
/// bound tokenizer, so one slot is one learned word.
const CGS_REAL_WORDS: [&str; 4] = [" it", " is", " was", " now"];

/// A declared development response class. The class is a property of the observed session state
/// (owned committed history and computed consumption), never of the request text or an evaluator
/// label.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CgsRealClass {
    /// Current value, no superseded older evidence, not a derived result.
    Direct,
    /// The current value superseded a *different* older committed value for the same address.
    Superseded,
    /// A same-value reassertion: history exists but the value never changed.
    Reasserted,
    /// The answer reads the consumed computed result.
    Derived,
}

impl CgsRealClass {
    fn flags(self) -> (bool, bool) {
        (
            matches!(self, Self::Superseded),
            matches!(self, Self::Derived),
        )
    }
    /// The declared development response text for this class and value.
    fn text(self, value: &str) -> String {
        match self {
            Self::Direct | Self::Reasserted => format!("it is {value}"),
            Self::Superseded => format!("it was {value}"),
            Self::Derived => format!("it is now {value}"),
        }
    }
}

const CGS_REAL_CLASSES: [CgsRealClass; 4] = [
    CgsRealClass::Direct,
    CgsRealClass::Superseded,
    CgsRealClass::Reasserted,
    CgsRealClass::Derived,
];

/// The declared slot tokens, each required to be exactly one token so a slot is one learned word.
/// Every candidate is reported together, so one run identifies all unusable words.
fn cgs_real_slots(tokenizer: &HfBpeTokenizer) -> Result<Vec<u32>, String> {
    let encoded: Vec<(u32, usize)> = CGS_REAL_WORDS
        .iter()
        .map(|word| {
            let tokens = tokenizer.encode(word);
            (tokens.first().copied().unwrap_or(0), tokens.len())
        })
        .collect();
    if let Some((_, _)) = encoded.iter().find(|(_, len)| *len != 1) {
        return Err(format!(
            "declared realization words tokenize to {:?} (word, token count); each must be one token",
            CGS_REAL_WORDS
                .iter()
                .zip(encoded.iter())
                .map(|(word, (_, len))| (word, len))
                .collect::<Vec<_>>()
        ));
    }
    Ok(encoded.into_iter().map(|(token, _)| token).collect())
}

/// Derive the realization fit examples from the **declared development response text**: tokenize each
/// declared response, locate the exact owned payload inside it, and read one example per emitted
/// token. The payload is never given to the policy; it is the label the fit extracts from text.
fn cgs_realization_examples(
    tokenizer: &HfBpeTokenizer,
    slots: &[u32],
) -> Result<Vec<RealizationExample>, String> {
    let slot_of = |token: u32| -> Result<u8, String> {
        slots
            .iter()
            .position(|s| *s == token)
            .map(|i| i as u8)
            .ok_or_else(|| format!("response word {token} is not a declared realization slot"))
    };
    let mut examples = Vec::new();
    for value in CGS_DEV_DESTS.iter() {
        let payload = tokenizer.encode(&format!(" {value}"));
        // The declared wording rule is defined by the owned-evidence flags, so the same declared
        // response is fitted at every learned historical position. Otherwise a `previous` or
        // `initial` request would fall back to the declared default and emit no learned word.
        for history in [0u8, 1, 2, 3] {
            for class in CGS_REAL_CLASSES {
                let (prior_differs, derived) = class.flags();
                let text = class.text(value);
                // The bound convention is an explicit leading space, matching the owned payload encoding.
                let tokens = tokenizer.encode(&format!(" {text}"));
                let pos = tokens
                    .windows(payload.len())
                    .position(|w| w == payload.as_slice())
                    .ok_or_else(|| {
                        format!("declared response {text:?} does not contain its owned payload")
                    })?;
                let base = |copy_stage: u8, emitted_bucket: u8| RealizationContext {
                    relation: 0,
                    history,
                    derived,
                    prior_differs,
                    evidence_class: 0,
                    copy_stage,
                    emitted_bucket,
                };
                for (i, token) in tokens[..pos].iter().enumerate() {
                    examples.push(RealizationExample {
                        context: base(0, (i as u8).min(3)),
                        action: RealizationAction::Insert(slot_of(*token)?),
                    });
                }
                let words = pos as u8;
                for i in 0..payload.len() {
                    // The first payload token is emitted with the copy cursor still at zero.
                    examples.push(RealizationExample {
                        context: base(u8::from(i > 0), words.min(3)),
                        action: RealizationAction::Copy,
                    });
                }
                let suffix = tokens.len() - pos - payload.len();
                for (i, token) in tokens[pos + payload.len()..].iter().enumerate() {
                    examples.push(RealizationExample {
                        context: base(2, (words + i as u8).min(3)),
                        action: RealizationAction::Insert(slot_of(*token)?),
                    });
                }
                examples.push(RealizationExample {
                    context: base(2, (words + suffix as u8).min(3)),
                    action: RealizationAction::Stop,
                });
            }
        }
    }
    Ok(examples)
}

/// One declared realized case: a scope, a lineage, the observed assertions that build its store, and
/// the person the request asks about.
struct CgsRealCase {
    name: &'static str,
    scope: &'static str,
    lineage: u64,
    person: &'static str,
    facts: &'static [(&'static str, &'static str)],
}

const CGS_REAL_CASES: [CgsRealCase; 5] = [
    CgsRealCase {
        name: "direct",
        scope: "alpha",
        lineage: 400,
        person: "Mara",
        facts: &[("Mara", "Harbor")],
    },
    CgsRealCase {
        name: "superseded",
        scope: "alpha",
        lineage: 401,
        person: "Mara",
        facts: &[("Mara", "Quarry"), ("Mara", "Harbor")],
    },
    CgsRealCase {
        name: "reasserted",
        scope: "alpha",
        lineage: 402,
        person: "Mara",
        facts: &[("Mara", "Harbor"), ("Mara", "Harbor")],
    },
    CgsRealCase {
        name: "fresh_direct",
        scope: "gamma",
        lineage: 410,
        person: "Juno",
        facts: &[("Juno", "Larkspur")],
    },
    CgsRealCase {
        name: "fresh_superseded",
        scope: "gamma",
        lineage: 411,
        person: "Juno",
        facts: &[("Juno", "Nettle"), ("Juno", "Larkspur")],
    },
];

/// Ingest the declared assertions of one realized case through the loaded observe+ingest path.
fn cgs_real_ingest(
    runtime: &mut ScopedMemoryRuntime,
    tokenizer: &HfBpeTokenizer,
    scope: &str,
    facts: &[(&str, &str)],
) -> Result<Vec<serde_json::Value>, String> {
    let mut receipts = Vec::new();
    for (turn, (entity, value)) in facts.iter().enumerate() {
        let (clause, _) = cgs_clause(tokenizer, 0, CgsForm::AssertOffice, entity, value)?;
        let observed = runtime.observe(&clause).map_err(|e| e.to_string())?;
        let outcome = runtime
            .ingest(&clause, scope.as_bytes(), turn as u64 + 1)
            .map_err(|e| e.to_string())?;
        receipts.push(json!({
            "text": clause.text,
            "entity": String::from_utf8_lossy(&observed.entity_key),
            "value": observed.value_key.as_ref().map(|v| String::from_utf8_lossy(v).into_owned()),
            "intent": observed.intent,
            "declared_value": value,
            "outcome": format!("{outcome:?}").chars().take(28).collect::<String>(),
        }));
    }
    Ok(receipts)
}

/// Trimmed decoded surface of one token list.
fn cgs_decode_trim(tokenizer: &HfBpeTokenizer, tokens: &[u32]) -> String {
    tokenizer
        .decode(tokens)
        .trim_matches(|c: char| c.is_ascii_whitespace())
        .to_string()
}

/// The per-emission trace of one realized run: (token, decision) for every emitted token.
fn cgs_real_trace(effects: &[ScopedStepEffect]) -> Vec<(u32, Option<RealizationDecision>)> {
    effects
        .iter()
        .filter_map(|e| e.emitted.map(|t| (t, e.realization)))
        .collect()
}

/// Compare two emission traces and describe the first differing decision and the fixed prefix before
/// it. This is the shape of the reduced causal test.
fn cgs_real_divergence(
    a: &[(u32, Option<RealizationDecision>)],
    b: &[(u32, Option<RealizationDecision>)],
) -> serde_json::Value {
    let prefix = a
        .iter()
        .zip(b.iter())
        .take_while(|(x, y)| x.0 == y.0)
        .count();
    let at = |t: &[(u32, Option<RealizationDecision>)], i: usize| {
        t.get(i).map(|(token, decision)| {
            json!({
                "token": token,
                "text": decision.map(|d| format!("{:?}", d.action)),
                "from_table": decision.map(|d| d.from_table),
                "evidence_class": decision.map(|d| d.evidence_class),
            })
        })
    };
    json!({
        "identical_prefix_tokens": prefix,
        "same_length": a.len() == b.len(),
        "a_emitted": a.iter().map(|(t, _)| *t).collect::<Vec<_>>(),
        "b_emitted": b.iter().map(|(t, _)| *t).collect::<Vec<_>>(),
        "a_at_divergence": at(a, prefix),
        "b_at_divergence": at(b, prefix),
    })
}

/// Every realized assertion the milestone must make, evaluated from actual emitted token records
/// rather than from summary booleans.
#[allow(clippy::too_many_arguments)]
fn cgs_realized_section(
    tokenizer: &HfBpeTokenizer,
    root: &std::path::Path,
    model_bytes: &[u8],
    intent_bytes: &[u8],
    lexicon_bytes: &[u8],
    realization_bytes: &[u8],
    backend: &ComputationBackend,
    realization: &RealizationModel,
    examples: &[RealizationExample],
    world: &CgsWorld,
) -> Result<serde_json::Value, String> {
    let load = |lineage: u64, control: MemoryControl, real: &[u8], contract: OutputContract| {
        ScopedMemoryRuntime::load_grounded(
            model_bytes,
            intent_bytes,
            Some(lexicon_bytes),
            backend.clone(),
            Some(real),
            contract,
            Memory::new(lineage, 16),
            lineage,
            control,
            VOCAB,
            Some(CGS_EOS),
        )
        .map_err(|e| e.to_string())
    };

    let mut cases = Vec::new();
    let mut traces =
        std::collections::BTreeMap::<String, Vec<(u32, Option<RealizationDecision>)>>::new();
    let mut disabled_traces =
        std::collections::BTreeMap::<String, Vec<(u32, Option<RealizationDecision>)>>::new();
    let mut disabled_cases = std::collections::BTreeMap::<String, String>::new();
    let mut payload_text = std::collections::BTreeMap::<String, String>::new();
    let mut direct_session: Option<ScopedSession> = None;

    for case in CGS_REAL_CASES.iter() {
        let mut runtime = load(
            case.lineage,
            MemoryControl::Normal,
            realization_bytes,
            OutputContract::RealizedV1,
        )?;
        let receipts = cgs_real_ingest(&mut runtime, tokenizer, case.scope, case.facts)?;
        let (clause, _) = cgs_clause(tokenizer, 0, CgsForm::AskOffice, case.person, "")?;
        let mut session = runtime
            .ask(&clause, case.scope.as_bytes())
            .map_err(|e| format!("realized ask {}: {e}", case.name))?;
        let effects = runtime
            .run(&mut session)
            .map_err(|e| format!("realized run {}: {e}", case.name))?;
        let answer = cgs_text(tokenizer, &session);
        let payload = session
            .captured
            .as_ref()
            .map(|c| c.payload.clone())
            .unwrap_or_default();
        let prelude = session.prelude_words as usize;
        let span = session
            .emitted
            .get(prelude..prelude + session.cursor)
            .map(|s| s.to_vec())
            .unwrap_or_default();
        let payload_surface = cgs_decode_trim(tokenizer, &payload);
        let trace = cgs_real_trace(&effects);

        // The same artifact under the retained legacy contract, for byte-for-byte retention.
        let mut legacy = load(
            case.lineage,
            MemoryControl::Normal,
            realization_bytes,
            OutputContract::LegacyWords,
        )?;
        cgs_real_ingest(&mut legacy, tokenizer, case.scope, case.facts)?;
        let mut legacy_session = legacy
            .ask(&clause, case.scope.as_bytes())
            .map_err(|e| format!("legacy ask {}: {e}", case.name))?;
        legacy.run(&mut legacy_session).map_err(|e| e.to_string())?;
        let legacy_answer = cgs_text(tokenizer, &legacy_session);

        cases.push(json!({
            "case": case.name,
            "scope": case.scope,
            "request": clause.text,
            "observed": receipts,
            "answer": answer,
            "legacy_answer": legacy_answer,
            "payload_surface": payload_surface,
            "copied_span_equals_payload": span == payload && !payload.is_empty(),
            "terminal": session.terminal,
            "emitted": &session.emitted,
            "vocabulary_words": session.vocabulary_words,
            "prelude_words": session.prelude_words,
            "derived_capture": session.captured.as_ref().map(|c| c.derived).unwrap_or(false),
            "effects": &effects,
        }));
        payload_text.insert(case.name.to_string(), payload_surface);
        traces.insert(case.name.to_string(), trace);

        // The context-disabled arm: identical artifact shape, fitted with the owned-evidence and
        // derived flags held at zero, so the vocabulary effect cannot depend on them.
        let mut disabled = load(
            case.lineage,
            MemoryControl::RealizationContextDisabled,
            realization_bytes,
            OutputContract::RealizedV1,
        )?;
        cgs_real_ingest(&mut disabled, tokenizer, case.scope, case.facts)?;
        let mut disabled_session = disabled
            .ask(&clause, case.scope.as_bytes())
            .map_err(|e| format!("disabled ask {}: {e}", case.name))?;
        let disabled_effects = disabled
            .run(&mut disabled_session)
            .map_err(|e| e.to_string())?;
        disabled_cases.insert(
            case.name.to_string(),
            cgs_text(tokenizer, &disabled_session),
        );
        disabled_traces.insert(case.name.to_string(), cgs_real_trace(&disabled_effects));

        if case.name == "direct" {
            direct_session = Some(session);
        }
    }

    // The consumed-computation case: one observed operation applied to the declared world's operand,
    // then the grounded result consumed as the next exact read.
    let mut derived_runtime = load(
        420,
        MemoryControl::Normal,
        realization_bytes,
        OutputContract::RealizedV1,
    )?;
    let mut derived_receipts = Vec::new();
    for (turn, (form, entity, tail)) in cgs_world_facts(world).into_iter().enumerate() {
        let (clause, _) = cgs_clause(tokenizer, 0, form, entity, tail)?;
        let outcome = derived_runtime
            .ingest(&clause, world.scope.as_bytes(), turn as u64 + 1)
            .map_err(|e| e.to_string())?;
        derived_receipts.push(json!({
            "text": clause.text,
            "declared_form": format!("{form:?}"),
            "committed": format!("{outcome:?}").chars().take(28).collect::<String>(),
        }));
    }
    let operand = world.people[0];
    let (compute_clause, _) = cgs_clause(tokenizer, 0, CgsForm::Compute, operand, "i")?;
    let mut derived_session = derived_runtime
        .ask(&compute_clause, world.scope.as_bytes())
        .map_err(|e| format!("realized compute ask: {e}"))?;
    let derived_effects = derived_runtime
        .run(&mut derived_session)
        .map_err(|e| format!("realized compute run: {e}"))?;
    let derived_answer = cgs_text(tokenizer, &derived_session);
    let derived_trace = cgs_real_trace(&derived_effects);
    let derived_payload = derived_session
        .captured
        .as_ref()
        .map(|c| c.payload.clone())
        .unwrap_or_default();
    let derived_prelude = derived_session.prelude_words as usize;
    let derived_span = derived_session
        .emitted
        .get(derived_prelude..derived_prelude + derived_session.cursor)
        .map(|s| s.to_vec())
        .unwrap_or_default();
    let derived_payload_surface = cgs_decode_trim(tokenizer, &derived_payload);

    // A source-access-disabled control over the identical realized store: no capture, no emission.
    let mut noread = load(
        400,
        MemoryControl::NoRead,
        realization_bytes,
        OutputContract::RealizedV1,
    )?;
    cgs_real_ingest(&mut noread, tokenizer, "alpha", CGS_REAL_CASES[0].facts)?;
    let (noread_clause, _) = cgs_clause(
        tokenizer,
        0,
        CgsForm::AskOffice,
        CGS_REAL_CASES[0].person,
        "",
    )?;
    let mut noread_session = noread
        .ask(&noread_clause, b"alpha")
        .map_err(|e| e.to_string())?;
    noread.run(&mut noread_session).map_err(|e| e.to_string())?;

    // ---- reduced causal tests ----
    let older = cgs_real_divergence(
        traces.get("direct").map(Vec::as_slice).unwrap_or(&[]),
        traces.get("superseded").map(Vec::as_slice).unwrap_or(&[]),
    );
    let reassert = cgs_real_divergence(
        traces.get("direct").map(Vec::as_slice).unwrap_or(&[]),
        traces.get("reasserted").map(Vec::as_slice).unwrap_or(&[]),
    );
    let derived_div = cgs_real_divergence(
        traces.get("direct").map(Vec::as_slice).unwrap_or(&[]),
        derived_trace.as_slice(),
    );
    let fresh_older = cgs_real_divergence(
        traces.get("fresh_direct").map(Vec::as_slice).unwrap_or(&[]),
        traces
            .get("fresh_superseded")
            .map(Vec::as_slice)
            .unwrap_or(&[]),
    );
    let disabled_older = cgs_real_divergence(
        disabled_traces
            .get("direct")
            .map(Vec::as_slice)
            .unwrap_or(&[]),
        disabled_traces
            .get("superseded")
            .map(Vec::as_slice)
            .unwrap_or(&[]),
    );

    // ---- mid-copy and mid-vocabulary restart, in process and in a fresh process ----
    let session = direct_session.ok_or("no realized direct session")?;
    let mut runtime = load(
        400,
        MemoryControl::Normal,
        realization_bytes,
        OutputContract::RealizedV1,
    )?;
    cgs_real_ingest(&mut runtime, tokenizer, "alpha", CGS_REAL_CASES[0].facts)?;
    let (restart_clause, _) = cgs_clause(
        tokenizer,
        0,
        CgsForm::AskOffice,
        CGS_REAL_CASES[0].person,
        "",
    )?;
    let mut phase = runtime
        .ask(&restart_clause, b"alpha")
        .map_err(|e| e.to_string())?;
    let mut snapshots = Vec::<Vec<u8>>::new();
    let mut boundaries = Vec::<serde_json::Value>::new();
    loop {
        snapshots.push(runtime.snapshot(&phase).map_err(|e| e.to_string())?);
        boundaries.push(json!({
            "pending": phase.pending,
            "cursor": phase.cursor,
            "vocabulary_words": phase.vocabulary_words,
            "emitted": phase.emitted,
        }));
        if phase.terminal.is_some() {
            break;
        }
        runtime.step(&mut phase).map_err(|e| e.to_string())?;
    }
    let mut restarts = Vec::new();
    for bytes in &snapshots {
        let mut restored = runtime.restore(bytes).map_err(|e| e.to_string())?;
        let boundary = json!({
            "cursor": restored.cursor,
            "vocabulary_words": restored.vocabulary_words,
            "pending": restored.pending,
        });
        runtime.run(&mut restored).map_err(|e| e.to_string())?;
        restarts.push(json!({
            "boundary": boundary,
            "matches": restored == session,
            "emitted": restored.emitted,
            "terminal": restored.terminal,
        }));
    }

    let reload_dir = root.join("realized_reload");
    std::fs::create_dir_all(&reload_dir).map_err(|e| e.to_string())?;
    let store_bytes = runtime.store_bytes().map_err(|e| e.to_string())?;
    write_checked(root, "realized_reload/model.json", model_bytes)?;
    write_checked(root, "realized_reload/intent.json", intent_bytes)?;
    write_checked(root, "realized_reload/lexicon.json", lexicon_bytes)?;
    write_checked(root, "realized_reload/realization.json", realization_bytes)?;
    write_checked(root, "realized_reload/store.json", &store_bytes)?;
    write_json(
        root,
        "realized_reload/artifact.json",
        &backend_identity_json(backend),
    )?;
    write_json(
        root,
        "realized_reload/request.json",
        &json!({"text": restart_clause.text, "scope": "alpha"}),
    )?;
    write_json(root, "realized_reload/checkpoints.json", &json!(&snapshots))?;
    let child = std::process::Command::new(
        std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?,
    )
    .arg("--mode=consumed-geometric-state")
    .arg("--reload-check")
    .arg(&reload_dir)
    .output()
    .map_err(|e| format!("realized fresh process: {e}"))?;
    let child_json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&child.stdout).trim()).map_err(|e| {
            format!(
                "realized child json: {e} (stderr {})",
                String::from_utf8_lossy(&child.stderr)
            )
        })?;
    let child_ok = child.status.success()
        && child_json["emitted"] == json!(&session.emitted)
        && child_json["terminal"]
            == serde_json::to_value(session.terminal).map_err(|e| e.to_string())?
        && child_json["final_frame"] == json!(&session)
        && child_json["resumed"].as_array().is_some_and(|rs| {
            rs.len() == snapshots.len() && rs.iter().all(|r| r["matches"] == json!(true))
        });

    // ---- one actual loaded multi-turn mixed interaction over the realized contract ----
    // observe, ordinary answer, correction, pinned in-flight continuation across that correction,
    // computed answer, previous/initial requests, and a mid-copy restart. Inspected as decoded text.
    let mut mixed_rt = load(
        430,
        MemoryControl::Normal,
        realization_bytes,
        OutputContract::RealizedV1,
    )?;
    let mut mixed_turns = Vec::<serde_json::Value>::new();
    let mut observe_turn = |rt: &mut ScopedMemoryRuntime,
                            entity: &str,
                            value: &str,
                            commit: u64,
                            out: &mut Vec<serde_json::Value>| {
        let (clause, _) = cgs_clause(tokenizer, 0, CgsForm::AssertOffice, entity, value)?;
        let outcome = rt
            .ingest(&clause, b"delta", commit)
            .map_err(|e| e.to_string())?;
        out.push(json!({
            "kind": "observe", "commit": commit, "text": clause.text,
            "outcome": format!("{outcome:?}").chars().take(24).collect::<String>(),
        }));
        Ok::<(), String>(())
    };
    observe_turn(&mut mixed_rt, "Mara", "Harbor", 1, &mut mixed_turns)?;
    let (ordinary_clause, _) = cgs_clause(tokenizer, 0, CgsForm::AskOffice, "Mara", "")?;
    let mut ordinary_session = mixed_rt
        .ask(&ordinary_clause, b"delta")
        .map_err(|e| e.to_string())?;
    mixed_rt
        .run(&mut ordinary_session)
        .map_err(|e| e.to_string())?;
    mixed_turns.push(json!({
        "kind": "ordinary", "request": ordinary_clause.text,
        "answer": cgs_text(tokenizer, &ordinary_session), "terminal": ordinary_session.terminal,
    }));
    // A pinned in-flight answer is started now and run only after a later correction commits.
    let mut pinned = mixed_rt
        .ask(&ordinary_clause, b"delta")
        .map_err(|e| e.to_string())?;
    observe_turn(&mut mixed_rt, "Mara", "Quarry", 2, &mut mixed_turns)?;
    mixed_rt.run(&mut pinned).map_err(|e| e.to_string())?;
    mixed_turns.push(json!({
        "kind": "pinned_in_flight", "request": ordinary_clause.text,
        "answer": cgs_text(tokenizer, &pinned), "terminal": pinned.terminal,
    }));
    let mut corrected = mixed_rt
        .ask(&ordinary_clause, b"delta")
        .map_err(|e| e.to_string())?;
    mixed_rt.run(&mut corrected).map_err(|e| e.to_string())?;
    mixed_turns.push(json!({
        "kind": "ordinary_after_correction", "request": ordinary_clause.text,
        "answer": cgs_text(tokenizer, &corrected), "terminal": corrected.terminal,
    }));
    // A consumed computation and the historical positions in the same loaded session. The Alma
    // assertion commits first, so previous is a different value from current and initial is not.
    observe_turn(&mut mixed_rt, "Mara", "Alma", 3, &mut mixed_turns)?;
    // A previous-assertion and an initial request through the same policy.
    let mut previous_session = mixed_rt
        .ask_view(b"delta", 0, b"Mara", HistoryView::PreviousAssertion)
        .map_err(|e| e.to_string())?;
    mixed_rt
        .run(&mut previous_session)
        .map_err(|e| e.to_string())?;
    let mut initial_session = mixed_rt
        .ask_view(b"delta", 0, b"Mara", HistoryView::Initial)
        .map_err(|e| e.to_string())?;
    mixed_rt
        .run(&mut initial_session)
        .map_err(|e| e.to_string())?;
    mixed_turns.push(json!({
        "kind": "previous", "answer": cgs_text(tokenizer, &previous_session),
        "terminal": previous_session.terminal,
        "kind_initial": "initial", "initial_answer": cgs_text(tokenizer, &initial_session),
        "initial_terminal": initial_session.terminal,
    }));
    // The grounded label offices the consumed computation will read.
    for (index, label) in CGS_LABELS.iter().enumerate() {
        observe_turn(
            &mut mixed_rt,
            label,
            CGS_DEV_DESTS[index],
            4 + index as u64,
            &mut mixed_turns,
        )?;
    }
    // A consumed computation in the same loaded session.
    let (compute_clause, _) = cgs_clause(tokenizer, 0, CgsForm::Compute, "Mara", "i")?;
    let mut computed_session = mixed_rt
        .ask(&compute_clause, b"delta")
        .map_err(|e| e.to_string())?;
    mixed_rt
        .run(&mut computed_session)
        .map_err(|e| e.to_string())?;
    let computed_payload = computed_session
        .captured
        .as_ref()
        .map(|c| c.payload.clone())
        .unwrap_or_default();
    let computed_prelude = computed_session.prelude_words as usize;
    let computed_span = computed_session
        .emitted
        .get(computed_prelude..computed_prelude + computed_session.cursor)
        .map(|s| s.to_vec())
        .unwrap_or_default();
    mixed_turns.push(json!({
        "kind": "computed", "request": compute_clause.text,
        "answer": cgs_text(tokenizer, &computed_session),
        "terminal": computed_session.terminal,
        "copied_span_equals_payload": computed_span == computed_payload && !computed_payload.is_empty(),
        "derived_capture": computed_session.captured.as_ref().map(|c| c.derived).unwrap_or(false),
    }));
    // Mid-copy restart inside the same loaded session.
    let mut mixed_phase = mixed_rt
        .ask(&ordinary_clause, b"delta")
        .map_err(|e| e.to_string())?;
    let mut mixed_snapshots = Vec::<Vec<u8>>::new();
    loop {
        mixed_snapshots.push(mixed_rt.snapshot(&mixed_phase).map_err(|e| e.to_string())?);
        if mixed_phase.terminal.is_some() {
            break;
        }
        mixed_rt.step(&mut mixed_phase).map_err(|e| e.to_string())?;
    }
    let mut mixed_restarts = Vec::new();
    for bytes in &mixed_snapshots {
        let mut restored = mixed_rt.restore(bytes).map_err(|e| e.to_string())?;
        mixed_rt.run(&mut restored).map_err(|e| e.to_string())?;
        mixed_restarts.push(restored == mixed_phase);
    }
    let ordinary_text = cgs_text(tokenizer, &ordinary_session);
    let pinned_text = cgs_text(tokenizer, &pinned);
    let corrected_text = cgs_text(tokenizer, &corrected);
    let previous_text = cgs_text(tokenizer, &previous_session);
    let initial_text = cgs_text(tokenizer, &initial_session);
    let computed_text = cgs_text(tokenizer, &computed_session);
    let mixed_ok = ordinary_text == "it is Harbor"
        // The pinned answer was started before the correction, so it keeps the old view; the fresh
        // request after the same correction sees the new value and its differing predecessor.
        && pinned_text == "it is Harbor"
        && corrected_text == "it was Quarry"
        && previous_text == "it was Quarry"
        && initial_text == "it is Harbor"
        && computed_session.terminal == Some(ScopedTerminal::Complete)
        && computed_span == computed_payload
        && !computed_payload.is_empty()
        && computed_session.captured.as_ref().map(|c| c.derived) == Some(true)
        && mixed_restarts.iter().all(|m| *m)
        && !mixed_restarts.is_empty();
    mixed_turns.push(json!({
        "kind": "decoded",
        "ordinary": ordinary_text,
        "pinned_in_flight": pinned_text,
        "ordinary_after_correction": corrected_text,
        "previous": previous_text,
        "initial": initial_text,
        "computed": computed_text,
        "ok": mixed_ok,
    }));

    let at = |v: &serde_json::Value| -> serde_json::Value {
        json!({
            "a": v["a_at_divergence"],
            "b": v["b_at_divergence"],
            "identical_prefix_tokens": v["identical_prefix_tokens"],
            "same_length": v["same_length"],
        })
    };
    let diverged = |v: &serde_json::Value| -> bool {
        let prefix = v["identical_prefix_tokens"].as_u64().unwrap_or(0);
        prefix > 0
            && v["a_at_divergence"].is_object()
            && v["b_at_divergence"].is_object()
            && v["a_at_divergence"]["token"] != v["b_at_divergence"]["token"]
    };
    let insert_at = |v: &serde_json::Value| -> bool {
        let is_insert = |side: &serde_json::Value| {
            side["text"]
                .as_str()
                .is_some_and(|t| t.starts_with("Insert"))
        };
        (is_insert(&v["a_at_divergence"]) || is_insert(&v["b_at_divergence"]))
            && v["a_at_divergence"]["from_table"] == json!(true)
            && v["b_at_divergence"]["from_table"] == json!(true)
    };

    let checks = json!({
        "realization_fit_is_nonempty": !examples.is_empty() && !realization.table.is_empty(),
        "realized_answers_are_complete": cases.iter().all(|c| c["terminal"] == json!(Some(ScopedTerminal::Complete))
            && c["vocabulary_words"].as_u64().unwrap_or(0) > 0),
        "realized_copied_span_is_exact": cases.iter().all(|c| c["copied_span_equals_payload"] == json!(true))
            && derived_span == derived_payload && !derived_payload.is_empty(),
        "legacy_contract_is_byte_exact": cases.iter().all(|c| c["legacy_answer"] == c["payload_surface"]),
        // The milestone's causal claim: older owned evidence changes an *uncopied* word while the
        // request and the generated prefix up to that decision are identical.
        "older_evidence_changes_an_uncopied_word": diverged(&older) && insert_at(&older),
        "fresh_older_evidence_changes_an_uncopied_word": diverged(&fresh_older) && insert_at(&fresh_older),
        "same_value_reassertion_keeps_the_direct_word": reassert["a_emitted"] == reassert["b_emitted"],
        "consumed_result_changes_an_uncopied_choice": diverged(&derived_div) && insert_at(&derived_div),
        // The context-disabled falsifier: with the owned-evidence flags held at zero the pair
        // collapses, so the effect is not a copy placement or a template shortcut.
        "context_disabled_collapses_the_pair": disabled_older["a_emitted"] == disabled_older["b_emitted"]
            && !diverged(&disabled_older),
        "source_access_disabled_emits_nothing": noread_session.terminal == Some(ScopedTerminal::NoRead)
            && noread_session.emitted.is_empty(),
        "restart_reproduces_the_complete_realized_frame": restarts.iter().all(|r| r["matches"] == json!(true))
            && restarts.len() == snapshots.len(),
        "fresh_process_restart_reproduces_the_complete_realized_frame": child_ok,
        "every_realized_emission_reports_its_context": cases.iter().all(|c| {
            c["effects"].as_array().is_some_and(|es| {
                es.iter()
                    .filter(|e| e["action"] == json!("Emit"))
                    .all(|e| e["realization"].is_object())
            })
        }),
        "realized_mixed_interaction_ok": mixed_ok,
    });

    Ok(json!({
        "artifact": {
            "version": realization.version,
            "slots": realization.slots,
            "slot_words": CGS_REAL_WORDS,
            "evidence_buckets": realization.evidence_buckets,
            "table_rows": realization.table.len(),
            "examples": examples.len(),
            "sha256": sha256_hex(realization_bytes),
            "context_disabled_control": "MemoryControl::RealizationContextDisabled",
        },
        "policy": "learned hard action policy over {Copy, Insert(slot), Stop}; owned structural invariants keep the exact span contiguous and complete, so the learned choice is the words around it and where to end",
        "context": "causal session state only: observed relation, requested history position, derived-result flag, superseded-older-evidence flag, copy stage and emitted-word count",
        "cases": cases,
        "derived_case": {
            "request": compute_clause.text,
            "observed": derived_receipts,
            "answer": derived_answer,
            "payload_surface": derived_payload_surface,
            "copied_span_equals_payload": derived_span == derived_payload,
            "terminal": derived_session.terminal,
            "emitted": &derived_session.emitted,
            "vocabulary_words": derived_session.vocabulary_words,
            "derived_capture": derived_session.captured.as_ref().map(|c| c.derived).unwrap_or(false),
            "effects": &derived_effects,
        },
        "no_read_control": {"terminal": noread_session.terminal, "emitted": noread_session.emitted},
        "context_disabled_answers": disabled_cases,
        "mixed_interaction": {
            "turns": mixed_turns,
            "restart_matches": mixed_restarts.iter().all(|m| *m),
            "restart_boundaries": mixed_restarts.len(),
            "ok": mixed_ok,
        },
        "causal": {
            "older_evidence_vs_direct": at(&older),
            "fresh_older_evidence_vs_direct": at(&fresh_older),
            "same_value_reassertion_vs_direct": at(&reassert),
            "consumed_result_vs_direct": at(&derived_div),
            "context_disabled_older_evidence_vs_direct": at(&disabled_older),
            "held_fixed": "the requested clause text, the copied payload and every emitted token before the flagged decision",
        },
        "restart": {
            "boundaries": boundaries,
            "resumed": restarts,
            "fresh_process_matches": child_ok,
            "snapshots": snapshots.len(),
        },
        "payload_text": payload_text,
        "checks": checks,
        "checks_all_expected": checks.as_object().map(|m| m.values().all(|v| v == &json!(true))).unwrap_or(false),
        "scope": "authored development responses over a small declared state world; a finite learned association table selects which shared vocabulary word follows the owned history. Not general prose, not semantic generalization of arbitrary values, no geometric advantage claimed.",
    }))
}

/// Ask one request and return (emitted text, terminal, computed result).
fn cgs_ask(
    tokenizer: &HfBpeTokenizer,
    runtime: &ScopedMemoryRuntime,
    clause: &Clause,
    scope: &str,
) -> Result<
    (
        String,
        Option<ScopedTerminal>,
        Option<ComputedState>,
        ScopedSession,
    ),
    String,
> {
    let mut session = runtime
        .ask(clause, scope.as_bytes())
        .map_err(|e| format!("ask: {e}"))?;
    runtime.run(&mut session).map_err(|e| format!("run: {e}"))?;
    Ok((
        cgs_text(tokenizer, &session),
        session.terminal,
        session.computation.clone(),
        session,
    ))
}

fn cgs_finish_run(
    tokenizer: &HfBpeTokenizer,
    root: &std::path::Path,
    started: Instant,
    payload: serde_json::Value,
    all_expected: bool,
) -> Result<ExitCode, String> {
    write_json(root, "result.json", &payload)?;
    seal(root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(root).map_err(|e| format!("verify: {e}"))?;
    if !all_expected {
        return Err(format!("sealed contract failure: {}", payload["checks"]));
    }
    let panels = &payload["panels"];
    println!(
        "consumed-geometric-state: dev {}/{} final {}/{} | consumed {} | sealed {} unlisted | {:.1}s",
        panels[0]["complete"],
        panels[0]["requests"],
        panels[1]["complete"],
        panels[1]["requests"],
        payload["consumption"]["rows"],
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    let _ = tokenizer;
    Ok(ExitCode::SUCCESS)
}

/// The retained ordinary memory lifecycle through the same learned path: two scoped assertions, a
/// question that must write nothing, and scope-independent answers.
fn cgs_ordinary(
    tokenizer: &HfBpeTokenizer,
    model_bytes: &[u8],
    intent_bytes: &[u8],
    lexicon_bytes: &[u8],
    backend: &ComputationBackend,
) -> Result<serde_json::Value, String> {
    let mut runtime = ScopedMemoryRuntime::load_with_computation(
        model_bytes,
        intent_bytes,
        Some(lexicon_bytes),
        backend.clone(),
        Memory::new(77, 8),
        77,
        MemoryControl::Normal,
        VOCAB,
        Some(CGS_EOS),
    )
    .map_err(|e| e.to_string())?;
    let mut wrote = 0usize;
    for (scope, label) in [("alpha", CGS_LABELS[0]), ("beta", CGS_LABELS[1])] {
        let (clause, _) = cgs_clause(
            tokenizer,
            0,
            CgsForm::AssertOffice,
            CGS_DEV_PEOPLE[0],
            label,
        )?;
        if matches!(
            runtime
                .ingest(&clause, scope.as_bytes(), 1)
                .map_err(|e| e.to_string())?,
            IngestOutcome::Wrote(_)
        ) {
            wrote += 1;
        }
    }
    let (question, _) = cgs_clause(tokenizer, 0, CgsForm::AskOffice, CGS_DEV_PEOPLE[0], "")?;
    let nowrite = matches!(
        runtime
            .ingest(&question, b"alpha", 2)
            .map_err(|e| e.to_string())?,
        IngestOutcome::NonAsserting { .. }
    );
    let (alpha, terminal, _, _) = cgs_ask(tokenizer, &runtime, &question, "alpha")?;
    let (beta, _, _, _) = cgs_ask(tokenizer, &runtime, &question, "beta")?;
    let ok = wrote == 2
        && nowrite
        && alpha == CGS_LABELS[0]
        && beta == CGS_LABELS[1]
        && terminal == Some(ScopedTerminal::Complete);
    Ok(serde_json::json!({
        "ok": ok, "wrote": wrote, "question_writes_nothing": nowrite,
        "alpha": alpha, "beta": beta,
    }))
}

/// Run one observed clause through the loaded session path and report the complete answer.
fn cgs_run_clause(
    tokenizer: &HfBpeTokenizer,
    runtime: &ScopedMemoryRuntime,
    clause: &Clause,
    scope: &str,
) -> Result<(String, Option<ScopedTerminal>, u8, ScopedSession), String> {
    let mut session = runtime
        .ask(clause, scope.as_bytes())
        .map_err(|e| e.to_string())?;
    runtime.run(&mut session).map_err(|e| e.to_string())?;
    let reads = session.hop + 1;
    Ok((
        cgs_text(tokenizer, &session),
        session.terminal,
        reads,
        session,
    ))
}

/// The cheap actual-loaded multi-turn check, run as soon as a candidate is constructed. One
/// conversation observes assertions, a redirect, a declared nonasserting input and a correction;
/// answers ordinary and historical questions; runs a requested computation whose operand is only
/// reachable through an observed redirect; keeps a pinned in-flight answer stable across a later
/// correction while a fresh question sees the new value; and continues identically after a real
/// save/reload. Expectations are produced by the declared reference semantics, never by the model.
fn cgs_mixed_session(
    tokenizer: &HfBpeTokenizer,
    model: &[u8],
    intent: &[u8],
    lexicon: &[u8],
    backend: &ComputationBackend,
    action: &CgsAction,
) -> Result<serde_json::Value, String> {
    use uor_r4_core::native_geometric::learner::scoped_memory::HistoryView;
    let scope = "alpha";
    let mut runtime = ScopedMemoryRuntime::load_with_computation(
        model,
        intent,
        Some(lexicon),
        backend.clone(),
        Memory::new(913, 8),
        913,
        MemoryControl::Normal,
        VOCAB,
        Some(CGS_EOS),
    )
    .map_err(|e| e.to_string())?;
    let mut oracle = ScmOracle::new(8);
    let mut turn = 0u64;
    let mut rows: Vec<serde_json::Value> = Vec::new();

    // ---- observed ingestion through the loaded path ----
    let ingests: Vec<(ScmForm, &str, &str)> = vec![
        (ScmForm::AssertOffice, "Alma", "Bramble"),
        (ScmForm::AssertOffice, "Bert", "Quarry"),
        (ScmForm::AssertOffice, "Cora", "Vale"),
        (ScmForm::AssertOffice, "Dane", "Marsh"),
        (ScmForm::AssertOffice, "Elin", "Tarn"),
        (ScmForm::AssertOffice, "Frey", "Ledge"),
        (ScmForm::AssertOffice, "Gwen", "Ridge"),
        (ScmForm::AssertOffice, "Holt", "Stone"),
        (ScmForm::AssertOffice, "Ivo", "Alma"),
        (ScmForm::RedirectOffice, "Mara", "Ivo"),
        (ScmForm::AssertOffice, "Oren", "Quarry"),
        (ScmForm::NegateOffice, "Oren", "Vale"),
    ];
    let mut ingest_total = 0usize;
    let mut ingest_ok = 0usize;
    for (form, entity, value) in ingests {
        turn += 1;
        let (clause, _) = scm_clause(tokenizer, 0, form, entity, Some(value))?;
        let observed = runtime.observe(&clause).map_err(|e| e.to_string())?;
        let outcome = runtime
            .ingest(&clause, scope.as_bytes(), turn)
            .map_err(|e| e.to_string())?;
        let declared_write = oracle.declared(0, form, entity, value);
        let wrote = matches!(outcome, IngestOutcome::Wrote(_));
        let ok = declared_write == wrote
            && observed.relation == form.relation()
            && observed.continues == form.is_redirect()
            && (!declared_write || observed.value_key.as_deref() == Some(value.as_bytes()));
        ingest_total += 1;
        ingest_ok += usize::from(ok);
        rows.push(serde_json::json!({
            "kind": "ingest", "turn": turn, "text": clause.text,
            "declared_write": declared_write, "wrote": wrote, "matched": ok,
            "relation": observed.relation, "continues": observed.continues,
            "value": observed.value_key.as_ref().map(|v| String::from_utf8_lossy(v).into_owned()),
        }));
    }

    // A question offered as input must write nothing.
    let (question_clause, _) = scm_clause(tokenizer, 0, ScmForm::AskCurrentOffice, "Mara", None)?;
    let question_nowrite = matches!(
        runtime
            .ingest(&question_clause, scope.as_bytes(), 999)
            .map_err(|e| e.to_string())?,
        IngestOutcome::NonAsserting { .. }
    );

    // ---- ordinary, historical and redirect-following questions ----
    let mut question_total = 0usize;
    let mut question_matched = 0usize;
    let ask = |rows: &mut Vec<serde_json::Value>,
               runtime: &ScopedMemoryRuntime,
               oracle: &ScmOracle,
               view: u64,
               form: ScmForm,
               entity: &str|
     -> Result<bool, String> {
        let (clause, _) = scm_clause(tokenizer, 0, form, entity, None)?;
        let history = form
            .history()
            .ok_or("question form has no declared history")?;
        let expected = oracle.answer(0, entity, form.relation(), history, view);
        let (answer, terminal, reads, _) = cgs_run_clause(tokenizer, runtime, &clause, scope)?;
        let ok = match (&expected, terminal) {
            (ScmExpected::Complete { value, hops }, Some(ScopedTerminal::Complete)) => {
                answer == *value && reads == *hops
            }
            (_, Some(found)) => found == scm_terminal_of(&expected),
            _ => false,
        };
        rows.push(serde_json::json!({
            "kind": "ask", "form": format!("{form:?}"), "text": clause.text,
            "expected": format!("{expected:?}"), "answer": answer, "terminal": terminal,
            "reads": reads, "matched": ok,
        }));
        Ok(ok)
    };
    for (form, entity) in [
        (ScmForm::AskCurrentOffice, "Mara"),
        (ScmForm::AskCurrentOffice, "Ivo"),
        (ScmForm::AskCurrentOffice, "Alma"),
        (ScmForm::AskCurrentOffice, "Oren"),
        (ScmForm::AskInitialOffice, "Oren"),
        (ScmForm::AskPreviousOffice, "Oren"),
    ] {
        question_total += 1;
        question_matched += usize::from(ask(
            &mut rows,
            &runtime,
            &oracle,
            oracle.commit,
            form,
            entity,
        )?);
    }

    // ---- a requested computation whose operand is reached through an observed redirect ----
    let compute_expect = |oracle: &ScmOracle,
                          view: u64,
                          entity: &str,
                          ops: &[&str]|
     -> Result<(String, u8), String> {
        let first = oracle.answer(0, entity, 0, HistoryView::Current, view);
        let ScmExpected::Complete { value, hops: h1 } = first else {
            return Err(format!("mixed computation operand walk: {first:?}"));
        };
        let label = CGS_LABELS
            .iter()
            .position(|l| *l == value.as_str())
            .ok_or_else(|| format!("operand {value} is not a grounded label"))?;
        let mut state = action
            .element_of_label(label as u32)
            .ok_or("operand label has no element")?;
        for op in ops {
            let primitive = CGS_OPS
                .iter()
                .find(|(w, _)| w == op)
                .map(|(_, p)| *p)
                .ok_or_else(|| format!("unknown operation {op}"))?;
            state = action
                .apply(state, primitive)
                .ok_or("declared action rejected the operation")?;
        }
        let derived = action
            .label_of_element(state)
            .ok_or("derived element is unlabelled")?;
        let second = oracle.answer(
            0,
            CGS_LABELS[derived as usize],
            0,
            HistoryView::Current,
            view,
        );
        let ScmExpected::Complete {
            value: answer,
            hops: h2,
        } = second
        else {
            return Err(format!("mixed computation derived walk: {second:?}"));
        };
        // The derived read is one further read at the same relation, exactly as the declared
        // reference walk counts it.
        Ok((answer, h1 + h2))
    };
    let mut computation_total = 0usize;
    let mut computation_matched = 0usize;
    let compute = |rows: &mut Vec<serde_json::Value>,
                   runtime: &ScopedMemoryRuntime,
                   oracle: &ScmOracle,
                   person: &str,
                   ops: &[&str]|
     -> Result<bool, String> {
        let text = ops.join(" ");
        let (clause, _) = cgs_clause(tokenizer, 0, CgsForm::Compute, person, &text)?;
        let (expected, hops) = compute_expect(oracle, oracle.commit, person, ops)?;
        let (answer, terminal, reads, session) =
            cgs_run_clause(tokenizer, runtime, &clause, scope)?;
        let routed_hops = session.captured.as_ref().map(|c| c.read_hop);
        let ok = terminal == Some(ScopedTerminal::Complete)
            && answer == expected
            && reads == hops
            && session.computation.as_ref().is_some_and(|c| c.consumed);
        rows.push(serde_json::json!({
            "kind": "compute", "text": clause.text, "ops": ops,
            "expected": expected, "answer": answer, "reads": reads, "expected_reads": hops,
            "terminal": terminal, "matched": ok,
            "operand_read_hop": routed_hops,
            "computed": session.computation,
        }));
        Ok(ok)
    };
    // `Mara` redirects to `Ivo`, so the operand requires one followed read before the operation runs.
    computation_total += 1;
    computation_matched += usize::from(compute(&mut rows, &runtime, &oracle, "Mara", &["i"])?);
    // A direct operand on the same request form.
    computation_total += 1;
    computation_matched += usize::from(compute(&mut rows, &runtime, &oracle, "Ivo", &["i", "j"])?);

    // ---- pinned in-flight answer across a later correction ----
    let (pinned_clause, _) = scm_clause(tokenizer, 0, ScmForm::AskCurrentOffice, "Oren", None)?;
    let pinned_view = oracle.commit;
    let pinned_expect = oracle.answer(0, "Oren", 0, HistoryView::Current, pinned_view);
    let mut pinned_session = runtime
        .ask(&pinned_clause, scope.as_bytes())
        .map_err(|e| e.to_string())?;
    // The correction arrives while the pinned answer is in flight.
    turn += 1;
    let (correction_clause, _) =
        scm_clause(tokenizer, 0, ScmForm::CorrectOffice, "Oren", Some("Harbor"))?;
    let correction_outcome = runtime
        .ingest(&correction_clause, scope.as_bytes(), turn)
        .map_err(|e| e.to_string())?;
    let correction_declared = oracle.declared(0, ScmForm::CorrectOffice, "Oren", "Harbor");
    runtime
        .run(&mut pinned_session)
        .map_err(|e| e.to_string())?;
    let pinned_answer = cgs_text(tokenizer, &pinned_session);
    let pinned_ok = matches!(&pinned_expect, ScmExpected::Complete { value, .. } if *value == pinned_answer)
        && pinned_session.terminal == Some(ScopedTerminal::Complete);
    rows.push(serde_json::json!({
        "kind": "pinned", "text": pinned_clause.text, "expected": format!("{pinned_expect:?}"),
        "answer": pinned_answer, "matched": pinned_ok, "view": pinned_view,
        "correction_written": matches!(correction_outcome, IngestOutcome::Wrote(_)),
        "correction_declared": correction_declared,
    }));
    let mut after_correction_total = 0usize;
    let mut after_correction_matched = 0usize;
    for (form, entity) in [
        (ScmForm::AskCurrentOffice, "Oren"),
        (ScmForm::AskInitialOffice, "Oren"),
        (ScmForm::AskPreviousOffice, "Oren"),
    ] {
        after_correction_total += 1;
        after_correction_matched += usize::from(ask(
            &mut rows,
            &runtime,
            &oracle,
            oracle.commit,
            form,
            entity,
        )?);
    }
    let correction_changed_answer = pinned_answer != "Harbor";

    // ---- real save/reload, then repeat the mixed requests ----
    let store = runtime.store_bytes().map_err(|e| e.to_string())?;
    let reloaded = ScopedMemoryRuntime::load_with_computation(
        model,
        intent,
        Some(lexicon),
        backend.clone(),
        Memory::from_bytes(&store).map_err(|e| e.to_string())?,
        913,
        MemoryControl::Normal,
        VOCAB,
        Some(CGS_EOS),
    )
    .map_err(|e| e.to_string())?;
    let mut reload_total = 0usize;
    let mut reload_matched = 0usize;
    for (form, entity) in [
        (ScmForm::AskCurrentOffice, "Oren"),
        (ScmForm::AskCurrentOffice, "Mara"),
    ] {
        reload_total += 1;
        reload_matched += usize::from(ask(
            &mut rows,
            &reloaded,
            &oracle,
            oracle.commit,
            form,
            entity,
        )?);
    }
    reload_total += 1;
    reload_matched += usize::from(compute(&mut rows, &reloaded, &oracle, "Mara", &["i"])?);

    let checks = serde_json::json!({
        "ingest_declared_write_and_observation": ingest_ok == ingest_total,
        "question_input_writes_nothing": question_nowrite,
        "ordinary_and_historical_questions": question_matched == question_total,
        "computation_follows_redirect_to_operand": computation_matched == computation_total,
        "pinned_answer_retained_across_correction": pinned_ok,
        "correction_writes_and_changes_new_answers":
            correction_declared
                && matches!(correction_outcome, IngestOutcome::Wrote(_))
                && after_correction_matched == after_correction_total
                && correction_changed_answer,
        "save_reload_continues_identically": reload_matched == reload_total,
    });
    let ok = checks
        .as_object()
        .map(|m| m.values().all(|v| v == &serde_json::json!(true)))
        .unwrap_or(false);
    Ok(serde_json::json!({
        "ok": ok,
        "checks": checks,
        "ingest": {"total": ingest_total, "matched": ingest_ok},
        "questions": {"total": question_total, "matched": question_matched},
        "computation": {"total": computation_total, "matched": computation_matched},
        "after_correction": {"total": after_correction_total, "matched": after_correction_matched},
        "reload": {"total": reload_total, "matched": reload_matched},
        "pinned_answer": pinned_answer,
        "rows": rows,
    }))
}

fn main() -> ExitCode {
    let mode = std::env::args().any(|a| a == "--mode=utility-transfer");
    let ce = std::env::args().any(|a| a == "--mode=contextual-emission");
    let comp = std::env::args().any(|a| a == "--mode=relation-composition");
    let dsd = std::env::args().any(|a| a == "--mode=derived-state-decoder");
    let stm = std::env::args().any(|a| a == "--mode=shared-transition");
    let gsm = std::env::args().any(|a| a == "--mode=grounded-session");
    let rsm = std::env::args().any(|a| a == "--mode=relational-session");
    let obm = std::env::args().any(|a| a == "--mode=observed-text-session");
    let scm = std::env::args().any(|a| a == "--mode=scoped-correction-memory");
    let cgs = std::env::args().any(|a| a == "--mode=consumed-geometric-state");
    let slx = std::env::args().any(|a| a == "--mode=state-lexical");
    let result = if slx {
        slx_run()
    } else if cgs {
        cgs_run()
    } else if scm {
        scm_run()
    } else if obm {
        ob_run()
    } else if rsm {
        rel_run()
    } else if gsm {
        gs_run()
    } else if stm {
        st_run()
    } else if dsd {
        dsd_run()
    } else if comp {
        rc2_run()
    } else if ce {
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
mod shared_transition_tests {
    use super::*;

    fn item() -> StItem {
        StItem {
            item_id: 1,
            instruction_start: 3,
            prefix: vec![90, 91, 10, 1, 2, 92, 91],
            primitives: vec![88, 89],
            targets: vec![20, 40],
            value: 10,
            split: "test",
        }
    }

    #[test]
    fn instructions_come_from_the_declared_observed_span() {
        let mut example = item();
        assert_eq!(st_instructions(&example).unwrap(), &[1, 2]);
        example.prefix[3] = 3;
        assert_eq!(st_instructions(&example).unwrap(), &[3, 2]);
        example.instruction_start = 99;
        assert!(st_instructions(&example).is_err());
    }

    #[test]
    fn saved_actual_events_recount_outputs_and_include_terminal_and_source_events() {
        let example = item();
        let served = StServed {
            response: Response {
                tokens: vec![20, 40],
                states: vec![7, 8],
                steps: vec![StepKind::Emit, StepKind::Emit, StepKind::Stop],
                stopped: true,
            },
            payload: Some(10),
            read: true,
            source_abs: Some(1),
            source_seq: Some(7),
            payload_abs: Some(2),
            initial_state: Some(6),
            local_token: 99,
        };
        let event = st_event("h4_shared_transition", &example, &served);
        let saved: serde_json::Value =
            serde_json::from_slice(&serde_json::to_vec(&event).unwrap()).unwrap();
        assert!(st_event_complete(&saved));
        assert_eq!(saved["source_seq_abs"], json!([7, 1]));
        assert_eq!(saved["steps"][0]["pre_state"], json!(6));
        assert_eq!(saved["steps"][1]["post_state"], json!(8));
        assert_eq!(saved["steps"][2]["kind"], json!("Stop"));
        assert_eq!(saved["steps"].as_array().unwrap().len(), 3);
        let absent = StServed {
            response: st_tokens_response(None),
            payload: None,
            read: false,
            source_abs: None,
            source_seq: None,
            payload_abs: None,
            initial_state: None,
            local_token: 99,
        };
        let no_read = st_event("whole_sequence_dictionary", &example, &absent);
        assert!(!st_event_complete(&no_read));
        assert_eq!(no_read["steps"][0]["kind"], json!("NoRead"));
        assert_eq!(no_read["tokens"], json!([]));
        assert!(!st_event_complete(&json!({})));
    }
}

#[cfg(test)]
mod relation_composition_tests {
    use super::*;

    #[test]
    fn declared_class_merges_only_the_shared_residue() {
        // Cells that share a residue must share a class; a value change must move it.
        assert_eq!(
            rc2_class(1, 4, RC2_CLASS_MODULUS),
            rc2_class(5, 0, RC2_CLASS_MODULUS)
        );
        assert_eq!(
            rc2_class(3, 7, RC2_CLASS_MODULUS),
            rc2_class(0, 0, RC2_CLASS_MODULUS)
        );
        assert_ne!(
            rc2_class(2, 1, RC2_CLASS_MODULUS),
            rc2_class(2, 2, RC2_CLASS_MODULUS)
        );
        for op in 0..RC2_N_OPS {
            for vi in 0..(RC2_N_VALUES_DEV + RC2_N_VALUES_TEST) {
                assert!(rc2_class(op, vi, RC2_CLASS_MODULUS) < RC2_CLASS_MODULUS);
            }
        }
    }

    #[test]
    fn the_rule_is_realisable_by_the_served_group() {
        // A witnessed order-10 element whose powers compose exactly as the rule declares.
        let g = rc2_order_ten_witness().expect("2I must contain an order-10 element");
        let t = group_table();
        let identity = t.identity as usize;
        let mul = |a: usize, b: usize| t.product[a * ROW_STRIDE + b] as usize;
        let pow = |e: usize| -> usize {
            let mut x = identity;
            for _ in 0..e {
                x = mul(x, g);
            }
            x
        };
        assert_eq!(pow(RC2_CLASS_MODULUS), pow(0));
        // Distinct residues 0..9 are distinct states, so the ten classes are separable.
        let mut seen = std::collections::BTreeSet::new();
        for e in 0..RC2_CLASS_MODULUS {
            assert!(seen.insert(pow(e)), "residue {e} collides");
        }
        let values: Vec<u32> = (100..108).collect();
        for n_ops in 2..=RC2_N_OPS {
            let k = rc2_choose_modulus(n_ops).unwrap();
            assert_eq!(k, 10);
            let relations: Vec<usize> = (0..n_ops).collect();
            for q0 in [identity, 7, 63, 119] {
                rc2_validate_algebra(&relations, &values, k, g, q0).unwrap();
            }
        }
        // This was the actual v1 defect: its reduced fixture labelled sum 7 as class 0
        // although the witnessed order-10 state at exponent 7 is not the identity.
        assert_ne!(pow(7), pow(rc2_class(1, 6, 7)));
        assert!(rc2_validate_algebra(&[80, 85], &values, 7, g, identity).is_err());
    }

    #[test]
    fn held_out_cells_share_a_class_with_development() {
        // Every held-out residue must be covered by an all-operations development population, for
        // the full operation budget and for the reduced budgets the served relation set may force.
        for n_ops in 2..=RC2_N_OPS {
            let k = rc2_choose_modulus(n_ops).expect("a covering modulus must exist");
            let dev: std::collections::BTreeSet<usize> = (0..n_ops)
                .flat_map(|op| {
                    (0..(RC2_N_VALUES_DEV + RC2_N_VALUES_TEST))
                        .filter(move |vi| !rc2_test_cell(op, *vi))
                        .map(move |vi| rc2_class(op, vi, k))
                })
                .collect();
            for op in 0..n_ops {
                for vi in
                    (0..(RC2_N_VALUES_DEV + RC2_N_VALUES_TEST)).filter(|vi| rc2_test_cell(op, *vi))
                {
                    assert!(
                        dev.contains(&rc2_class(op, vi, k)),
                        "held-out cell ({op},{vi}) has no development class at n_ops={n_ops}"
                    );
                }
            }
            rc2_validate_split(n_ops, k).unwrap();
        }
    }

    #[test]
    fn prediction_receipts_recount_actual_scores_and_preserve_occurrence_identity() {
        let mut read = Read {
            action: None,
            source: None,
            payload: None,
            payload_abs: None,
            rel: 0,
            admitted: 0,
            scanned: 0,
        };
        let first = ce_prediction_record(&[0, 0, 0], &read, 7, 7, 1, 1, 4, 8).unwrap();
        assert_eq!(first.prediction, 0);
        assert_eq!(first.strongest_competitor, 0);
        assert!((first.loss_bits - 3f64.log2()).abs() < 1e-12);
        read.action = Some((0, 1));
        read.source = Some(OccurrenceRef { seq: 4, abs: 7 });
        read.payload = Some(42);
        read.payload_abs = Some(8);
        let second = ce_prediction_record(&[2, 0, -2], &read, 7, 19, 0, 1, 4, 8).unwrap();
        let expected = (1.0 + (-1f64).exp() + (-2f64).exp()).log2();
        assert!((second.loss_bits - expected).abs() < 1e-12);
        let encoded = serde_json::to_string(&vec![first, second]).unwrap();
        let loaded: Vec<CePredictionRecord> = serde_json::from_str(&encoded).unwrap();
        assert_eq!(ce_prediction_counts(&loaded), (1, 1));
        assert_eq!(loaded[1].selected_source_seq_abs, Some([4, 7]));
        assert_eq!(loaded[1].intended_payload_seq_abs, [4, 8]);
        assert_eq!(loaded[1].q1, 19);
    }

    #[test]
    fn signature_report_counts_positions_separately_from_distinct_labels() {
        let make = |target| CeExample {
            z_local: vec![0; 2],
            q0: 7,
            r: 80,
            payload: 42,
            read: true,
            target,
        };
        let report = ce_signature_report(&[make(0), make(0), make(0), make(1), make(1)]);
        assert_eq!(report["targets_on_ambiguous_signatures"], 2);
        assert_eq!(report["positions_on_ambiguous_signatures"], 5);
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

#[cfg(test)]
mod loaded_lifecycle_tests {
    use super::*;

    fn bytes(root: &Path, name: &str) -> Result<Vec<u8>, String> {
        std::fs::read(root.join(name)).map_err(|e| format!("{name}: {e}"))
    }
    fn tokenizer() -> Result<HfBpeTokenizer, String> {
        let bytes = std::fs::read(DEFAULT_TOKENIZER).map_err(|e| e.to_string())?;
        if sha256_hex(&bytes) != TOKENIZER_SHA {
            return Err("review tokenizer identity mismatch".into());
        }
        derive_tokenizer(&bytes, VOCAB).map_err(|e| e.to_string())
    }
    fn loaded(root: &Path, memory: Memory) -> Result<ScopedMemoryRuntime, String> {
        let backend = cgs_backend_from_json(
            &serde_json::from_slice(&bytes(root, "backend-signed_factorization.json")?)
                .map_err(|e| e.to_string())?,
        )?;
        let lineage = memory.lineage;
        ScopedMemoryRuntime::load_with_computation(
            &bytes(root, "model.json")?,
            &bytes(root, "intent.json")?,
            Some(&bytes(root, "lexicon.json")?),
            backend,
            memory,
            lineage,
            MemoryControl::Normal,
            VOCAB,
            Some(CGS_EOS),
        )
        .map_err(|e| e.to_string())
    }
    fn ingest(
        runtime: &mut ScopedMemoryRuntime,
        tokenizer: &HfBpeTokenizer,
        oracle: &mut ScmOracle,
        form: ScmForm,
        entity: &str,
        value: &str,
    ) -> Result<serde_json::Value, String> {
        // Only the literal text and scope cross the serving boundary. Form/entity/value fields
        // independently define the reference world, and never select the runtime's route.
        let text: String = scm_ob_form(form, entity, Some(value))
            .parts
            .into_iter()
            .map(|(_, s)| s)
            .collect();
        let clause = scm_raw_clause(tokenizer, &text)?;
        let observed = runtime.observe(&clause).map_err(|e| e.to_string())?;
        let expected_write = oracle.declared(0, form, entity, value);
        let result = runtime
            .ingest(&clause, b"alpha", oracle.commit)
            .map_err(|e| e.to_string())?;
        let wrote = matches!(result, IngestOutcome::Wrote(_));
        if wrote != expected_write
            || observed.entity_key != entity.as_bytes()
            || observed.value_key.as_deref() != Some(value.as_bytes())
            || observed.continues != form.is_redirect()
        {
            return Err(format!(
                "loaded observation mismatch for {text:?}: {observed:?}"
            ));
        }
        Ok(
            json!({"text": text, "tokens": clause.tokens, "byte_lengths": clause.byte_lengths,
            "entity": entity, "value": value, "continues": observed.continues,
            "intent": observed.intent, "wrote": wrote, "commit": oracle.commit}),
        )
    }
    fn derived(action: &CgsAction, operand: &str) -> Result<String, String> {
        let label = CGS_LABELS
            .iter()
            .position(|l| *l == operand)
            .ok_or("reference operand")?;
        let mut state = action
            .element_of_label(label as u32)
            .ok_or("reference element")?;
        for primitive in [100, 200] {
            state = action.apply(state, primitive).ok_or("reference action")?;
        }
        Ok(CGS_LABELS[action.label_of_element(state).ok_or("reference result")? as usize].into())
    }
    fn expected(action: &CgsAction, oracle: &ScmOracle) -> Result<(String, u8), String> {
        let first = oracle.answer(0, "Mara", 0, HistoryView::Current, oracle.commit);
        let ScmExpected::Complete { value, hops } = first else {
            return Err(format!("reference operand walk: {first:?}"));
        };
        let key = derived(action, &value)?;
        let second = oracle.answer(0, &key, 0, HistoryView::Current, oracle.commit);
        let ScmExpected::Complete { value, hops: after } = second else {
            return Err(format!("reference derived walk: {second:?}"));
        };
        Ok((value, hops + after))
    }

    /// A separately invoked test process loads only persisted model/store/request/snapshots.
    /// No expected answer or gold semantic query field is supplied to its runtime.
    #[test]
    #[ignore = "child of the explicitly invoked loaded-artifact lifecycle audit"]
    fn child_restores_pinned_computation() -> Result<(), String> {
        let root = PathBuf::from(std::env::var("UOR_LIFECYCLE_CHILD").map_err(|e| e.to_string())?);
        let tokenizer = tokenizer()?;
        let memory = Memory::from_bytes(&bytes(&root, "store.json")?).map_err(|e| e.to_string())?;
        let runtime = loaded(&root.join("artifacts"), memory)?;
        let request: serde_json::Value =
            serde_json::from_slice(&bytes(&root, "request.json")?).map_err(|e| e.to_string())?;
        if request.as_object().is_none_or(|v| v.len() != 2)
            || request.get("text").and_then(|v| v.as_str()).is_none()
            || request["scope"] != json!("alpha")
        {
            return Err("child needs only raw text and scope".into());
        }
        let clause = scm_raw_clause(&tokenizer, request["text"].as_str().ok_or("text")?)?;
        let (_, _, _, fresh) = cgs_run_clause(&tokenizer, &runtime, &clause, "alpha")?;
        let snapshots: Vec<Vec<u8>> =
            serde_json::from_slice(&bytes(&root, "snapshots.json")?).map_err(|e| e.to_string())?;
        let mut resumed = Vec::new();
        for snapshot in snapshots {
            let mut session = runtime.restore(&snapshot).map_err(|e| e.to_string())?;
            runtime.run(&mut session).map_err(|e| e.to_string())?;
            resumed.push(session);
        }
        write_json(
            &root,
            "child.json",
            &json!({"fresh": fresh, "resumed": resumed}),
        )?;
        Ok(())
    }

    #[test]
    #[ignore = "requires explicitly pinned PR1348 artifacts and UOR_LIFECYCLE_AUDIT_ROOT; no fitting"]
    fn loaded_redirect_computation_retains_pins_across_relevant_corrections() -> Result<(), String>
    {
        let input =
            PathBuf::from(std::env::var("UOR_LIFECYCLE_ARTIFACT_ROOT").map_err(|e| e.to_string())?);
        let root =
            PathBuf::from(std::env::var("UOR_LIFECYCLE_AUDIT_ROOT").map_err(|e| e.to_string())?);
        claim(&root).map_err(|e| e.to_string())?;
        let tokenizer = tokenizer()?;
        let action = CgsAction::build()?;
        let mut runtime = loaded(&input, Memory::new(1348, 8))?;
        let mut oracle = ScmOracle::new(8);
        let mut ingests = Vec::new();
        for (label, dest) in CGS_LABELS.iter().zip(CGS_DEV_DESTS) {
            ingests.push(ingest(
                &mut runtime,
                &tokenizer,
                &mut oracle,
                ScmForm::AssertOffice,
                label,
                dest,
            )?);
        }
        // Alma is deliberately both a familiar geometric label and an entity reference.
        // Its learned source role determines whether to follow it or apply immediately.
        ingests.push(ingest(
            &mut runtime,
            &tokenizer,
            &mut oracle,
            ScmForm::CorrectOffice,
            "Alma",
            "Bert",
        )?);
        ingests.push(ingest(
            &mut runtime,
            &tokenizer,
            &mut oracle,
            ScmForm::RedirectOffice,
            "Mara",
            "Alma",
        )?);
        let old_derived = derived(&action, "Bert")?;
        let immediate_derived = derived(&action, "Alma")?;
        if old_derived == "Alma" || immediate_derived == "Alma" || old_derived == immediate_derived
        {
            return Err("reference fixture needs distinct non-Alma derived keys".into());
        }
        ingests.push(ingest(
            &mut runtime,
            &tokenizer,
            &mut oracle,
            ScmForm::RedirectOffice,
            &old_derived,
            "Oren",
        )?);
        ingests.push(ingest(
            &mut runtime,
            &tokenizer,
            &mut oracle,
            ScmForm::AssertOffice,
            "Oren",
            "Quarry",
        )?);
        let baseline_store = runtime.store_bytes().map_err(|e| e.to_string())?;
        let clause = scm_raw_clause(&tokenizer, "Mara's office then i j")?;
        let baseline_expected = expected(&action, &oracle)?;
        let mut phase = runtime.ask(&clause, b"alpha").map_err(|e| e.to_string())?;
        let mut snapshots = Vec::new();
        let mut phases = Vec::new();
        loop {
            snapshots.push(runtime.snapshot(&phase).map_err(|e| e.to_string())?);
            phases.push(json!({"pending": phase.pending, "hop": phase.hop,
                "computation": phase.computation, "captured": phase.captured}));
            if phase.terminal.is_some() {
                break;
            }
            runtime.step(&mut phase).map_err(|e| e.to_string())?;
        }
        let baseline = phase;
        if baseline.terminal != Some(ScopedTerminal::Complete)
            || cgs_text(&tokenizer, &baseline) != baseline_expected.0
            || baseline.hop + 1 != baseline_expected.1
        {
            return Err("baseline reference mismatch".into());
        }
        let corrected_operand = CGS_LABELS
            .iter()
            .copied()
            .find(|label| {
                *label != "Bert"
                    && *label != "Alma"
                    && derived(&action, label).is_ok_and(|key| key != "Alma" && key != old_derived)
            })
            .ok_or("need a distinct corrected operand")?;
        let mut scenarios = Vec::new();
        for (name, form, entity, value) in [
            (
                "operand_correction",
                ScmForm::CorrectOffice,
                "Alma",
                corrected_operand,
            ),
            (
                "downstream_correction",
                ScmForm::CorrectOffice,
                "Oren",
                "Harbor",
            ),
            (
                "same_key_role_change",
                ScmForm::CorrectOffice,
                "Mara",
                "Alma",
            ),
        ] {
            let dir = root.join(name);
            let mut changed = loaded(
                &input,
                Memory::from_bytes(&baseline_store).map_err(|e| e.to_string())?,
            )?;
            let mut changed_oracle = ScmOracle {
                chains: oracle.chains.clone(),
                commit: oracle.commit,
                capacity: oracle.capacity,
            };
            let observation = ingest(
                &mut changed,
                &tokenizer,
                &mut changed_oracle,
                form,
                entity,
                value,
            )?;
            let target = expected(&action, &changed_oracle)?;
            let (answer, terminal, reads, fresh) =
                cgs_run_clause(&tokenizer, &changed, &clause, "alpha")?;
            if terminal != Some(ScopedTerminal::Complete)
                || answer != target.0
                || reads != target.1
                || answer == baseline_expected.0
            {
                return Err(format!(
                    "{name} changed-source reference mismatch: {answer:?} != {target:?}"
                ));
            }
            let mut pinned = Vec::new();
            for snapshot in &snapshots {
                let mut resumed = changed
                    .restore(snapshot)
                    .map_err(|e| format!("{name}: {e}"))?;
                changed.run(&mut resumed).map_err(|e| e.to_string())?;
                if resumed != baseline {
                    return Err(format!("{name}: pinned full-frame changed"));
                }
                pinned.push(resumed);
            }
            for filename in [
                "model.json",
                "intent.json",
                "lexicon.json",
                "backend-signed_factorization.json",
            ] {
                write_checked(
                    &dir,
                    &format!("artifacts/{filename}"),
                    &bytes(&input, filename)?,
                )?;
            }
            write_checked(
                &dir,
                "store.json",
                &changed.store_bytes().map_err(|e| e.to_string())?,
            )?;
            write_json(
                &dir,
                "request.json",
                &json!({"text": clause.text, "scope": "alpha"}),
            )?;
            write_json(&dir, "snapshots.json", &json!(snapshots))?;
            let child =
                std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
                    .args([
                        "--exact",
                        "loaded_lifecycle_tests::child_restores_pinned_computation",
                        "--ignored",
                    ])
                    .env("UOR_LIFECYCLE_CHILD", &dir)
                    .output()
                    .map_err(|e| e.to_string())?;
            write_checked(&dir, "child.stdout", &child.stdout)?;
            write_checked(&dir, "child.stderr", &child.stderr)?;
            if !child.status.success() {
                return Err(format!("{name}: child failed"));
            }
            let checked: serde_json::Value =
                serde_json::from_slice(&bytes(&dir, "child.json")?).map_err(|e| e.to_string())?;
            if checked["fresh"] != json!(fresh) || checked["resumed"] != json!(pinned) {
                return Err(format!("{name}: child full-frame mismatch"));
            }
            scenarios.push(json!({"name": name, "observation": observation,
                "expected": target.0, "expected_reads": target.1, "fresh": fresh,
                "pinned_resumes": pinned.len(), "separate_process_full_frame_parity": true}));
        }
        let artifact_hashes: BTreeMap<&str, String> = [
            "model.json",
            "intent.json",
            "lexicon.json",
            "backend-signed_factorization.json",
        ]
        .into_iter()
        .map(|name| bytes(&input, name).map(|b| (name, sha256_hex(&b))))
        .collect::<Result<_, _>>()?;
        write_json(
            &root,
            "result.json",
            &json!({"artifact_root": input,
            "artifact_hashes": artifact_hashes, "ingests": ingests, "request": clause.text,
            "baseline_expected": baseline_expected.0, "baseline": baseline, "phases": phases,
            "scenarios": scenarios, "classification": "exposed principal interaction regression; no refit",
            "phase_scope": "grounding and consumption are one atomic Apply; no separate between-state exists"}),
        )?;
        seal(&root).map_err(|e| e.to_string())?;
        let unlisted = verify(&root).map_err(|e| e.to_string())?;
        if !unlisted.is_empty() {
            return Err("unlisted review evidence".into());
        }
        Ok(())
    }

    #[test]
    #[ignore = "requires saved candidate and prior scoped evidence; no fitting"]
    fn loaded_bundle_retains_scoped_control_matrix() -> Result<(), String> {
        let input =
            PathBuf::from(std::env::var("UOR_LIFECYCLE_ARTIFACT_ROOT").map_err(|e| e.to_string())?);
        let root =
            PathBuf::from(std::env::var("UOR_LIFECYCLE_CONTROL_ROOT").map_err(|e| e.to_string())?);
        let prior =
            PathBuf::from(std::env::var("UOR_LIFECYCLE_PRIOR_ROOT").map_err(|e| e.to_string())?);
        claim(&root).map_err(|e| e.to_string())?;
        let tokenizer = tokenizer()?;
        let model = bytes(&input, "model.json")?;
        let intent = bytes(&input, "intent.json")?;
        let prior_rows = bytes(&prior, "rows.jsonl")?;
        let mut old = BTreeMap::new();
        for line in std::str::from_utf8(&prior_rows)
            .map_err(|e| e.to_string())?
            .lines()
        {
            let row: serde_json::Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
            let key = (
                row["script"].to_string(),
                row["control"].to_string(),
                row["turn"].to_string(),
                row["kind"].to_string(),
            );
            old.insert(key, row);
        }
        let scripts = [
            ("development", scm_development_script(), 0x5C0F_0000_0001),
            ("exposed_regression", scm_exposed_script(), 0x5C0F_0000_0003),
            ("final", scm_final_script(), 0x5C0F_0000_0002),
        ];
        let mut rows = Vec::new();
        let mut summaries = Vec::new();
        for (name, control, capacity) in [
            ("normal", MemoryControl::Normal, 8),
            ("no_read", MemoryControl::NoRead, 8),
            ("update_disabled", MemoryControl::UpdateDisabled, 8),
            ("unpinned", MemoryControl::Unpinned, 8),
            ("unscoped", MemoryControl::Unscoped, 8),
            (
                "parse_score_authority",
                MemoryControl::ParseScoreAuthority,
                8,
            ),
            ("capacity_2", MemoryControl::Normal, 2),
        ] {
            for (script_name, script, lineage) in &scripts {
                if name == "capacity_2" && *script_name != "development" {
                    continue;
                }
                let replay = scm_replay(
                    &tokenizer,
                    &model,
                    &intent,
                    *lineage,
                    control.clone(),
                    capacity,
                    script,
                    script_name,
                    name,
                )?;
                summaries.push(json!({"script": script_name, "control": name,
                    "questions": replay.questions, "matched": replay.complete,
                    "intent_correct": replay.intent_correct, "intent_total": replay.intent_total,
                    "branch_correct": replay.branch_correct, "branch_total": replay.branch_total,
                    "nowrite_correct": replay.nowrite_correct, "nowrite_total": replay.nowrite_total}));
                rows.extend(replay.rows);
            }
        }
        // Different learned bytes and their history digests are expected. Compare every other
        // observation, write, answer, exact selected source, action and final owned-frame field.
        let semantic = |row: &serde_json::Value| {
            let mut compared = row.clone();
            if let Some(frame) = compared
                .get_mut("final_frame")
                .and_then(|v| v.as_object_mut())
            {
                frame.remove("binding");
                frame.remove("view_sha256");
            }
            compared
        };
        let mut differences = Vec::new();
        let mut missing = Vec::new();
        for row in &rows {
            let key = (
                row["script"].to_string(),
                row["control"].to_string(),
                row["turn"].to_string(),
                row["kind"].to_string(),
            );
            match old.get(&key) {
                None => missing.push(json!(key)),
                Some(previous) if semantic(row) != semantic(previous) => {
                    differences.push(json!({"key": key, "prior": previous, "candidate": row}));
                }
                _ => {}
            }
        }
        let encoded = rows
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
            .join("\n");
        write_checked(&root, "rows.jsonl", encoded.as_bytes())?;
        write_json(
            &root,
            "result.json",
            &json!({"artifact_root": input, "prior_root": prior,
            "model_sha256": sha256_hex(&model), "intent_sha256": sha256_hex(&intent),
            "prior_rows_sha256": sha256_hex(&prior_rows), "compared_rows": rows.len(),
            "summaries": summaries, "missing_prior_rows": missing, "differences": differences,
            "comparison_excludes": ["final_frame.binding", "final_frame.view_sha256"],
            "scope": "same saved observation and intent parameters in unchanged ordinary scoped runtime; arithmetic backend absent because requests carry no operations; original scripted controls, not fresh language acceptance"}),
        )?;
        seal(&root).map_err(|e| e.to_string())?;
        if !verify(&root).map_err(|e| e.to_string())?.is_empty() {
            return Err("unlisted control evidence".into());
        }
        if !missing.is_empty() || !differences.is_empty() {
            return Err(format!(
                "retained matrix: {} missing, {} different rows; sealed for review",
                missing.len(),
                differences.len()
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod loaded_realization_tests {
    use super::*;

    const FILES: [&str; 5] = [
        "model.json",
        "intent.json",
        "lexicon.json",
        "backend-signed_factorization.json",
        "realization.json",
    ];

    fn bytes(root: &Path, name: &str) -> Result<Vec<u8>, String> {
        std::fs::read(root.join(name)).map_err(|e| format!("{name}: {e}"))
    }
    fn tokenizer() -> Result<HfBpeTokenizer, String> {
        let data = std::fs::read(DEFAULT_TOKENIZER).map_err(|e| e.to_string())?;
        if sha256_hex(&data) != TOKENIZER_SHA {
            return Err("realization audit tokenizer identity differs".into());
        }
        derive_tokenizer(&data, VOCAB).map_err(|e| e.to_string())
    }
    fn loaded(
        input: &Path,
        memory: Memory,
        contract: OutputContract,
    ) -> Result<ScopedMemoryRuntime, String> {
        let backend = cgs_backend_from_json(
            &serde_json::from_slice(&bytes(input, "backend-signed_factorization.json")?)
                .map_err(|e| e.to_string())?,
        )?;
        let lineage = memory.lineage;
        ScopedMemoryRuntime::load_grounded(
            &bytes(input, "model.json")?,
            &bytes(input, "intent.json")?,
            Some(&bytes(input, "lexicon.json")?),
            backend,
            Some(&bytes(input, "realization.json")?),
            contract,
            memory,
            lineage,
            MemoryControl::Normal,
            VOCAB,
            Some(CGS_EOS),
        )
        .map_err(|e| e.to_string())
    }

    /// A separate test process receives saved artifacts, store, raw request and snapshots only.
    /// Full actual frames are retained, with no expected response supplied to inference.
    #[test]
    #[ignore = "child of the explicitly invoked saved-realizer audit"]
    fn child_restores_realized_frames() -> Result<(), String> {
        let root =
            PathBuf::from(std::env::var("UOR_REALIZATION_CHILD").map_err(|e| e.to_string())?);
        let tokenizer = tokenizer()?;
        let memory = Memory::from_bytes(&bytes(&root, "store.json")?).map_err(|e| e.to_string())?;
        let runtime = loaded(&root.join("artifacts"), memory, OutputContract::RealizedV1)?;
        let request: serde_json::Value =
            serde_json::from_slice(&bytes(&root, "request.json")?).map_err(|e| e.to_string())?;
        if request.as_object().is_none_or(|o| o.len() != 2) {
            return Err("child request must contain raw text and scope only".into());
        }
        let clause = scm_raw_clause(&tokenizer, request["text"].as_str().ok_or("request text")?)?;
        let scope = request["scope"].as_str().ok_or("request scope")?;
        let mut fresh = runtime
            .ask(&clause, scope.as_bytes())
            .map_err(|e| e.to_string())?;
        let fresh_effects = runtime.run(&mut fresh).map_err(|e| e.to_string())?;
        let snapshots: Vec<Vec<u8>> =
            serde_json::from_slice(&bytes(&root, "snapshots.json")?).map_err(|e| e.to_string())?;
        let mut resumed = Vec::new();
        for bytes in snapshots {
            let mut frame = runtime.restore(&bytes).map_err(|e| e.to_string())?;
            let effects = runtime.run(&mut frame).map_err(|e| e.to_string())?;
            resumed.push(json!({"frame": frame, "effects": effects}));
        }
        write_json(
            &root,
            "child.json",
            &json!({
                "fresh": fresh, "fresh_effects": fresh_effects, "resumed": resumed,
            }),
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn persist_checkpoints(
        root: &Path,
        input: &Path,
        runtime: &ScopedMemoryRuntime,
        text: &str,
        scope: &str,
        snapshots: &[Vec<u8>],
        baseline: &ScopedSession,
        fresh: &ScopedSession,
    ) -> Result<usize, String> {
        for name in FILES {
            write_checked(root, &format!("artifacts/{name}"), &bytes(input, name)?)?;
        }
        write_checked(
            root,
            "store.json",
            &runtime.store_bytes().map_err(|e| e.to_string())?,
        )?;
        write_json(root, "request.json", &json!({"text": text, "scope": scope}))?;
        write_json(root, "snapshots.json", &json!(snapshots))?;
        let mut parent = Vec::new();
        for snapshot in snapshots {
            let mut frame = runtime.restore(snapshot).map_err(|e| e.to_string())?;
            let effects = runtime.run(&mut frame).map_err(|e| e.to_string())?;
            if frame != *baseline {
                return Err("saved realizer parent full-frame mismatch".into());
            }
            parent.push(json!({"frame": frame, "effects": effects}));
        }
        write_json(
            root,
            "parent.json",
            &json!({"fresh": fresh, "resumed": parent}),
        )?;
        let child = std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .args([
                "--exact",
                "loaded_realization_tests::child_restores_realized_frames",
                "--ignored",
            ])
            .env("UOR_REALIZATION_CHILD", root)
            .output()
            .map_err(|e| e.to_string())?;
        write_checked(root, "child.stdout", &child.stdout)?;
        write_checked(root, "child.stderr", &child.stderr)?;
        if !child.status.success() {
            return Err(format!(
                "realizer child failed: {}",
                String::from_utf8_lossy(&child.stderr)
            ));
        }
        let actual: serde_json::Value =
            serde_json::from_slice(&bytes(root, "child.json")?).map_err(|e| e.to_string())?;
        if actual["fresh"] != json!(fresh) || actual["resumed"] != json!(parent) {
            return Err("saved realizer child full-frame/effect mismatch".into());
        }
        Ok(snapshots.len())
    }

    #[test]
    #[ignore = "requires UOR_REALIZATION_ARTIFACT_ROOT and exclusive UOR_REALIZATION_AUDIT_ROOT; no fitting"]
    fn saved_realizer_preserves_exposed_outputs_and_owned_checkpoints() -> Result<(), String> {
        let input = PathBuf::from(
            std::env::var("UOR_REALIZATION_ARTIFACT_ROOT").map_err(|e| e.to_string())?,
        );
        let root =
            PathBuf::from(std::env::var("UOR_REALIZATION_AUDIT_ROOT").map_err(|e| e.to_string())?);
        claim(&root).map_err(|e| e.to_string())?;
        let tokenizer = tokenizer()?;
        let mut rows = Vec::new();
        let mut total_phases = 0;
        // Exact prior observed bytes are exposure/regression targets, explicitly NOT a semantic pass.
        for (index, case) in CGS_REAL_CASES.iter().enumerate() {
            let mut runtime = loaded(
                &input,
                Memory::new(case.lineage, 16),
                OutputContract::RealizedV1,
            )?;
            let ingests = cgs_real_ingest(&mut runtime, &tokenizer, case.scope, case.facts)?;
            let (clause, _) = cgs_clause(&tokenizer, 0, CgsForm::AskOffice, case.person, "")?;
            let mut phase = runtime
                .ask(&clause, case.scope.as_bytes())
                .map_err(|e| e.to_string())?;
            let mut snapshots = Vec::new();
            let mut effects = Vec::new();
            loop {
                snapshots.push(runtime.snapshot(&phase).map_err(|e| e.to_string())?);
                if phase.terminal.is_some() {
                    break;
                }
                effects.push(runtime.step(&mut phase).map_err(|e| e.to_string())?);
            }
            let prior_output = [
                "it is Harbor",
                "it was Harbor",
                "it is Harbor",
                "it is Larkspur",
                "it was Larkspur",
            ][index];
            if cgs_text(&tokenizer, &phase) != prior_output
                || phase.terminal != Some(ScopedTerminal::Complete)
            {
                return Err(format!("{} changed exposed output", case.name));
            }
            let store = runtime.store_bytes().map_err(|e| e.to_string())?;
            let legacy = loaded(
                &input,
                Memory::from_bytes(&store).map_err(|e| e.to_string())?,
                OutputContract::LegacyWords,
            )?;
            let mut legacy_frame = legacy
                .ask(&clause, case.scope.as_bytes())
                .map_err(|e| e.to_string())?;
            legacy.run(&mut legacy_frame).map_err(|e| e.to_string())?;
            let payload = phase.captured.as_ref().ok_or("missing owned payload")?;
            if cgs_text(&tokenizer, &legacy_frame) != String::from_utf8_lossy(&payload.value) {
                return Err("bound realizer legacy contract changed owned value".into());
            }
            let phases = persist_checkpoints(
                &root.join(case.name),
                &input,
                &runtime,
                &clause.text,
                case.scope,
                &snapshots,
                &phase,
                &phase,
            )?;
            total_phases += phases;
            rows.push(
                json!({"case": case.name, "ingests": ingests, "request": clause.text,
                "frame": phase, "effects": effects, "legacy_frame": legacy_frame,
                "exposed_prior_output": prior_output, "parent_and_child_phases": phases}),
            );
        }
        // Same loaded policy receives observed computation; final exact value remains owned evidence.
        let world = cgs_worlds(0, &CGS_DEV_PEOPLE, &CGS_DEV_DESTS, "alpha", 5).remove(0);
        let mut runtime = loaded(&input, Memory::new(420, 16), OutputContract::RealizedV1)?;
        let mut observations = Vec::new();
        for (turn, (form, entity, tail)) in cgs_world_facts(&world).into_iter().enumerate() {
            let (clause, _) = cgs_clause(&tokenizer, 0, form, entity, tail)?;
            let outcome = runtime
                .ingest(&clause, b"alpha", turn as u64 + 1)
                .map_err(|e| e.to_string())?;
            observations.push(json!({"text": clause.text, "outcome": format!("{outcome:?}")}));
        }
        let clause = scm_raw_clause(&tokenizer, "Mara's office then i")?;
        let mut phase = runtime.ask(&clause, b"alpha").map_err(|e| e.to_string())?;
        let mut snapshots = Vec::new();
        let mut effects = Vec::new();
        loop {
            snapshots.push(runtime.snapshot(&phase).map_err(|e| e.to_string())?);
            if phase.terminal.is_some() {
                break;
            }
            effects.push(runtime.step(&mut phase).map_err(|e| e.to_string())?);
        }
        if cgs_text(&tokenizer, &phase) != "it is now Tarn"
            || !phase.captured.as_ref().is_some_and(|c| c.derived)
        {
            return Err("computed exposed output changed".into());
        }
        let phases = persist_checkpoints(
            &root.join("derived"),
            &input,
            &runtime,
            &clause.text,
            "alpha",
            &snapshots,
            &phase,
            &phase,
        )?;
        total_phases += phases;
        rows.push(json!({"case": "derived", "ingests": observations, "request": clause.text,
            "frame": phase, "effects": effects, "exposed_prior_output": "it is now Tarn", "parent_and_child_phases": phases}));
        // Preserve the already owned contextual decision when both its predecessor payload and
        // original source payload are actually evicted. The snapshot is before `was`, so a
        // recomputed false prior-difference flag changes a vocabulary token and is observable.
        let mut pinned_rt = loaded(&input, Memory::new(1349, 2), OutputContract::RealizedV1)?;
        let pin_ingests = cgs_real_ingest(
            &mut pinned_rt,
            &tokenizer,
            "alpha",
            &[("Mara", "Quarry"), ("Mara", "Harbor")],
        )?;
        let pin_clause = scm_raw_clause(&tokenizer, "What is Mara's office?")?;
        let mut pin_phase = pinned_rt
            .ask(&pin_clause, b"alpha")
            .map_err(|e| e.to_string())?;
        while pin_phase.vocabulary_words == 0 && pin_phase.terminal.is_none() {
            pinned_rt.step(&mut pin_phase).map_err(|e| e.to_string())?;
        }
        if pin_phase.vocabulary_words != 1 || pin_phase.cursor != 0 {
            return Err("pin witness did not reach first vocabulary word".into());
        }
        let pin_snapshot = pinned_rt.snapshot(&pin_phase).map_err(|e| e.to_string())?;
        let mut pin_baseline = pin_phase;
        pinned_rt
            .run(&mut pin_baseline)
            .map_err(|e| e.to_string())?;
        let (same_value, _) = cgs_clause(&tokenizer, 0, CgsForm::AssertOffice, "Mara", "Harbor")?;
        for commit in [3, 4] {
            pinned_rt
                .ingest(&same_value, b"alpha", commit)
                .map_err(|e| e.to_string())?;
        }
        let evicted_store =
            Memory::from_bytes(&pinned_rt.store_bytes().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if !(1..=2).all(|id| {
            evicted_store
                .record_ref(id)
                .is_some_and(|r| r.evicted && r.payload.is_empty())
        }) {
            return Err("predecessor/source payload eviction was not witnessed".into());
        }
        let mut pin_fresh = pinned_rt
            .ask(&pin_clause, b"alpha")
            .map_err(|e| e.to_string())?;
        pinned_rt.run(&mut pin_fresh).map_err(|e| e.to_string())?;
        if cgs_text(&tokenizer, &pin_baseline) != "it was Harbor"
            || cgs_text(&tokenizer, &pin_fresh) != "it is Harbor"
        {
            return Err("pinned/fresh exposed-realizer contrast changed".into());
        }
        let pin_phases = persist_checkpoints(
            &root.join("owned_context_after_eviction"),
            &input,
            &pinned_rt,
            &pin_clause.text,
            "alpha",
            &[pin_snapshot],
            &pin_baseline,
            &pin_fresh,
        )?;
        total_phases += pin_phases;
        rows.push(json!({"case": "owned_context_after_eviction", "ingests": pin_ingests,
            "evicted_record_ids": [1,2], "pinned": pin_baseline, "fresh": pin_fresh,
            "parent_and_child_phases": pin_phases,
            "classification": "Owned-state preservation only; original tense semantics remain a retained negative"}));
        let hashes: BTreeMap<&str, String> = FILES
            .into_iter()
            .map(|name| bytes(&input, name).map(|data| (name, sha256_hex(&data))))
            .collect::<Result<_, _>>()?;
        write_json(
            &root,
            "result.json",
            &json!({
                "classification": "Exposed actual-artifact runtime regression; no fitting, no new semantic acceptance",
                "artifact_root": input, "artifact_sha256": hashes, "cases": rows,
                "complete_parent_and_child_frames_checked": total_phases,
                "semantic_failure_retained": "Prior-difference falsely chooses was for current Harbor/Larkspur; preserving bytes here does not accept that training target.",
                "source_executable_sha256": sha256_hex(&std::fs::read(std::env::current_exe().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?),
            }),
        )?;
        seal(&root).map_err(|e| e.to_string())?;
        if !verify(&root).map_err(|e| e.to_string())?.is_empty() {
            return Err("unlisted realizer audit evidence".into());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// State-conditioned lexical realization on authored response targets
//
// The retained `RealizedV1` finite table changed a word from a history/provenance
// *flag*: its key omitted the actual emitted symbols and the evidence content, so
// a word selected after the third insertion aliased and an external cap had to
// end the response; two values with identical flags could not differ, and the
// authored targets used `was` for the current value and `now` for a computation.
//
// This section learns a state-conditioned decoder from declared response text
// and serves it through the retained scoped session. The decoder consumes the
// actual emitted-token row and a content embedding of selected evidence. Its
// computed-target wording is keyed to derived-address movement in self-valued
// fixtures; this is not an independently validated temporal proposition.
// ---------------------------------------------------------------------------

/// The protocol terminator the session binds.
const SLX_EOS: u32 = u32::MAX - 1;

/// The declared learned vocabulary: one shared slot per word, each a single token of the bound
/// tokenizer. Slot order is the learned action order and is never derived from an evaluator field.
const SLX_WORDS: [&str; 8] = [
    " it", " is", " was", " still", " became", " first", " at", " now",
];
/// Declared bound on learned insert words per answer, matching the session contract.
const SLX_MAX_INSERT: u8 = 6;
/// Declared unfamiliar token ids used by the unknown-content control. They are excluded from the
/// fitted content index, so the control exercises the decoder's reserved OOV rows rather than being
/// silently fitted, and the held-out source documents remain genuinely unfamiliar.
const SLX_OOV_TOKENS: [u32; 2] = [(VOCAB - 1) as u32, (VOCAB - 2) as u32];

/// One declared source document: a scope, the entity it is about, and the successive office values
/// the document asserts. `computed` documents use groundable labels so a computation can consume
/// their current value; plain documents may use any unfamiliar value string.
struct SlxDoc {
    name: &'static str,
    scope: &'static str,
    lineage: u64,
    entity: &'static str,
    values: &'static [&'static str],
    fit: bool,
    computed: bool,
}

/// Four fitting documents and three **held-out source documents** that share no entity and, for the
/// plain documents, no value string with the fitting set. The split is by document, before fitting.
const SLX_DOCS: [SlxDoc; 7] = [
    SlxDoc {
        name: "mara",
        scope: "alpha",
        lineage: 501,
        entity: "Mara",
        values: &["Alma", "Bert"],
        fit: true,
        computed: true,
    },
    SlxDoc {
        name: "ivo",
        scope: "beta",
        lineage: 502,
        entity: "Ivo",
        values: &["Cora"],
        fit: true,
        computed: true,
    },
    SlxDoc {
        name: "cedar",
        scope: "gamma",
        lineage: 503,
        entity: "Cedar",
        values: &["Dane", "Dane"],
        fit: true,
        computed: true,
    },
    SlxDoc {
        name: "oren",
        scope: "delta",
        lineage: 504,
        entity: "Oren",
        values: &["Elin", "Frey", "Gwen"],
        fit: true,
        computed: true,
    },
    SlxDoc {
        name: "una",
        scope: "epsilon",
        lineage: 601,
        entity: "Una",
        values: &["Larkspur"],
        fit: false,
        computed: false,
    },
    SlxDoc {
        name: "pia",
        scope: "zeta",
        lineage: 602,
        entity: "Pia",
        values: &["Nettle", "Umber"],
        fit: false,
        computed: false,
    },
    SlxDoc {
        name: "silas",
        scope: "eta",
        lineage: 603,
        entity: "Silas",
        values: &["Vellum", "Wren", "Yarrow"],
        fit: false,
        computed: false,
    },
];

/// The retained historical position a case requests.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SlxView {
    Current,
    Previous,
    Initial,
}

impl SlxView {
    fn history(self) -> HistoryView {
        match self {
            Self::Current => HistoryView::Current,
            Self::Previous => HistoryView::PreviousAssertion,
            Self::Initial => HistoryView::Initial,
        }
    }
    fn code(self) -> u8 {
        match self {
            Self::Current => 0,
            Self::Previous => 1,
            Self::Initial => 3,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Previous => "previous",
            Self::Initial => "initial",
        }
    }
}

/// The declared slot tokens, each required to be exactly one token so a slot is one learned word.
fn slx_slots(tokenizer: &HfBpeTokenizer) -> Result<Vec<u32>, String> {
    let encoded: Vec<(u32, usize)> = SLX_WORDS
        .iter()
        .map(|word| {
            let tokens = tokenizer.encode(word);
            (tokens.first().copied().unwrap_or(0), tokens.len())
        })
        .collect();
    if encoded.iter().any(|(_, len)| *len != 1) {
        return Err(format!(
            "declared state-lexical words tokenize to {:?}; each must be one token",
            SLX_WORDS
                .iter()
                .zip(encoded.iter())
                .map(|(word, (_, len))| (word, len))
                .collect::<Vec<_>>()
        ));
    }
    Ok(encoded.into_iter().map(|(token, _)| token).collect())
}

fn slx_slot_of(slots: &[u32], token: u32) -> Result<u8, String> {
    slots
        .iter()
        .position(|s| *s == token)
        .map(|i| i as u8)
        .ok_or_else(|| format!("declared response word {token} is not a learned slot"))
}

/// Historical authored target sentences, retained for exposed replay. Current/history wording is
/// explicit; computed `became`/`still` distinguish route keys on self-valued fixtures, not a
/// committed temporal change. These targets do not constitute an independent semantic oracle.
fn slx_render(view: SlxView, derived: bool, changed: bool, value: &str) -> String {
    match (view, derived) {
        (SlxView::Initial, false) => format!("at first it was {value}"),
        (SlxView::Previous, false) => format!("it was {value}"),
        (SlxView::Current, false) => format!("it is {value}"),
        // A changed derived address is marked after the owned span in this authored
        // response family. The marker does not prove a committed temporal mutation.
        (SlxView::Current, true) if changed => format!("it became {value} now"),
        (SlxView::Current, true) => format!("it is still {value}"),
        _ => format!("it is {value}"),
    }
}

/// Derive one teacher-forced sequence from the **declared response text**: tokenize the sentence,
/// locate the exact owned payload inside it, and read one action per emitted token. `res` carries the
/// computation operand's tokens so the learner can see the value the computation started from.
fn slx_text_sequence(
    tokenizer: &HfBpeTokenizer,
    slots: &[u32],
    view: SlxView,
    derived: bool,
    changed: bool,
    value: &str,
    res: &[u32],
    prior_differs: bool,
    committed: bool,
    key_changed: bool,
) -> Result<SlSequence, String> {
    let payload = tokenizer.encode(&format!(" {value}"));
    let text = slx_render(view, derived, changed, value);
    let tokens = tokenizer.encode(&format!(" {text}"));
    let pos = tokens
        .windows(payload.len())
        .position(|w| w == payload.as_slice())
        .ok_or_else(|| format!("declared response {text:?} does not contain its owned payload"))?;
    let mut actions = Vec::with_capacity(tokens.len() + 1);
    for token in &tokens[..pos] {
        actions.push(RealizationAction::Insert(slx_slot_of(slots, *token)?));
    }
    actions.extend(vec![RealizationAction::Copy; payload.len()]);
    for token in &tokens[pos + payload.len()..] {
        actions.push(RealizationAction::Insert(slx_slot_of(slots, *token)?));
    }
    actions.push(RealizationAction::Stop);
    Ok(SlSequence {
        sel: payload,
        res: res.to_vec(),
        history: view.code(),
        derived,
        prior_differs,
        committed,
        key_changed,
        actions,
    })
}

/// The grounded computation backend and its lexicon, built exactly as the retained consumed-state
/// path builds them, so the state-lexical section composes with the same artifacts.
#[allow(clippy::type_complexity)]
fn slx_artifacts(
    tokenizer: &HfBpeTokenizer,
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>, ComputationBackend, Vec<u32>), String> {
    let action = CgsAction::build()?;
    let examples = cgs_examples(&action)?;
    let (factorization, _) =
        factor_observed_graph(&examples, false).map_err(|e| format!("factorization: {e}"))?;
    let backend = ComputationBackend::Signed(Box::new(factorization));

    let mut surface: Vec<(Vec<u8>, u32)> = CGS_OPS
        .iter()
        .map(|(word, id)| (word.as_bytes().to_vec(), *id))
        .collect();
    for (index, label) in CGS_LABELS.iter().enumerate() {
        surface.push((label.as_bytes().to_vec(), index as u32));
    }
    let canonical: Vec<(u32, Vec<u8>)> = CGS_LABELS
        .iter()
        .enumerate()
        .map(|(index, label)| (index as u32, label.as_bytes().to_vec()))
        .collect();
    let lexicon = GroundingLexicon::from_observations(&surface, &canonical)
        .map_err(|e| format!("lexicon: {e}"))?;

    let (dev_clauses, supervision) = combined_development_supervision(tokenizer)?;
    let labels: Vec<ClauseLabel> = supervision.iter().map(|s| s.label.clone()).collect();
    let (model, _) = fit_observed_text_model(&dev_clauses, &labels, false)
        .map_err(|e| format!("binder: {e}"))?;
    let cue_spans: Vec<(usize, usize)> = supervision
        .iter()
        .map(|s| (s.label.marker.0 as usize, s.label.marker.1))
        .collect();
    let intent_examples: Vec<IntentExample> = supervision
        .iter()
        .map(|s| IntentExample {
            question: s.question,
            intent: s.intent,
        })
        .collect();
    let (intent, _) = fit_intent_model(&dev_clauses, &cue_spans, &intent_examples, VOCAB)
        .map_err(|e| format!("intent: {e}"))?;

    let slots = slx_slots(tokenizer)?;
    Ok((
        model.to_bytes().map_err(|e| e.to_string())?,
        intent.to_bytes().map_err(|e| e.to_string())?,
        lexicon.to_bytes().map_err(|e| e.to_string())?,
        backend,
        slots,
    ))
}

/// Build one document's exact store by passing its assertions through the **loaded observe+ingest
/// path**. Every label gets an office equal to itself so a consumed computation can read its result;
/// the document's own entity then takes the declared successive values.
fn slx_doc_store(
    tokenizer: &HfBpeTokenizer,
    doc: &SlxDoc,
    model_bytes: &[u8],
    intent_bytes: &[u8],
    lexicon_bytes: &[u8],
    backend: &ComputationBackend,
) -> Result<Vec<u8>, String> {
    let mut runtime = ScopedMemoryRuntime::load_grounded(
        model_bytes,
        intent_bytes,
        Some(lexicon_bytes),
        backend.clone(),
        None,
        OutputContract::LegacyWords,
        Memory::new(doc.lineage, 16),
        doc.lineage,
        MemoryControl::Normal,
        VOCAB,
        Some(SLX_EOS),
    )
    .map_err(|e| e.to_string())?;
    let mut turn = 0u64;
    for label in CGS_LABELS {
        let (clause, _) = cgs_clause(tokenizer, 0, CgsForm::AssertOffice, label, label)?;
        runtime.observe(&clause).map_err(|e| e.to_string())?;
        runtime
            .ingest(&clause, doc.scope.as_bytes(), turn)
            .map_err(|e| e.to_string())?;
        turn += 1;
    }
    for value in doc.values {
        let (clause, _) = cgs_clause(tokenizer, 0, CgsForm::AssertOffice, doc.entity, value)?;
        runtime.observe(&clause).map_err(|e| e.to_string())?;
        runtime
            .ingest(&clause, doc.scope.as_bytes(), turn)
            .map_err(|e| e.to_string())?;
        turn += 1;
    }
    runtime.store_bytes().map_err(|e| e.to_string())
}

fn slx_load(
    model_bytes: &[u8],
    intent_bytes: &[u8],
    lexicon_bytes: &[u8],
    backend: &ComputationBackend,
    lexical_bytes: Option<&[u8]>,
    contract: OutputContract,
    control: MemoryControl,
    store: &[u8],
    lineage: u64,
) -> Result<ScopedMemoryRuntime, String> {
    let memory = Memory::from_bytes(store).map_err(|e| e.to_string())?;
    ScopedMemoryRuntime::load_grounded(
        model_bytes,
        intent_bytes,
        Some(lexicon_bytes),
        backend.clone(),
        lexical_bytes,
        contract,
        memory,
        lineage,
        control,
        VOCAB,
        Some(SLX_EOS),
    )
    .map_err(|e| e.to_string())
}

/// Historical fixture target membership. Shares target design with `slx_render`; it does not
/// independently establish truthful meaning for arbitrary downstream computed values.
fn slx_oracle(
    slots: &[u32],
    view: SlxView,
    derived: bool,
    changed: bool,
    payload: &[u32],
) -> Vec<u32> {
    let slot = |i: usize| slots[i];
    let mut out = Vec::new();
    // A word that follows the owned span, when the declared meaning places one there.
    let mut tail: Vec<u32> = Vec::new();
    match (view, derived) {
        (SlxView::Initial, false) => {
            out.extend([slot(6), slot(5), slot(0), slot(2)]);
        }
        (SlxView::Previous, false) => {
            out.extend([slot(0), slot(2)]);
        }
        (SlxView::Current, false) => {
            out.extend([slot(0), slot(1)]);
        }
        (SlxView::Current, true) if changed => {
            // The authored route-change case has a trailing marker after the copied span.
            out.extend([slot(0), slot(4)]);
            tail.push(slot(7));
        }
        // The consumed computation left the same office in force, so the answer reports persistence.
        (SlxView::Current, true) => {
            out.extend([slot(0), slot(1), slot(3)]);
        }
        (_, true) => {
            out.extend([slot(0), slot(1)]);
        }
    }
    out.extend_from_slice(payload);
    out.extend(tail);
    out
}

/// A copy of `seq` whose owned payload is replaced by `len` declared-unfamiliar tokens, with the
/// action sequence's copy run resized to match. The unknown-content control is presented across the
/// payload lengths the language can present rather than only the fitting documents' lengths, so an
/// unfamiliar value of any length meets a fitted OOV path.
fn slx_reblank(seq: &SlSequence, len: usize, unfitted: u32) -> SlSequence {
    let mut actions: Vec<RealizationAction> = Vec::new();
    let mut copied = false;
    for action in &seq.actions {
        if matches!(action, RealizationAction::Copy) {
            if !copied {
                actions.extend(std::iter::repeat(RealizationAction::Copy).take(len));
                copied = true;
            }
            continue;
        }
        actions.push(*action);
    }
    let mut out = seq.clone();
    out.sel = vec![unfitted; len];
    out.actions = actions;
    out
}

/// One probed computation for a document: the op word, whether it changed the consumed address, the
/// exact answer payload, and the operand payload the computation started from.
struct SlxProbe {
    op: &'static str,
    changed: bool,
    payload: Vec<u32>,
    operand_payload: Vec<u32>,
}

/// Run one computation request under the retained legacy contract and read its actual effect.
fn slx_probe_compute(
    tokenizer: &HfBpeTokenizer,
    model_bytes: &[u8],
    intent_bytes: &[u8],
    lexicon_bytes: &[u8],
    backend: &ComputationBackend,
    store: &[u8],
    doc: &SlxDoc,
    op: &'static str,
) -> Result<Option<SlxProbe>, String> {
    let mut runtime = slx_load(
        model_bytes,
        intent_bytes,
        lexicon_bytes,
        backend,
        None,
        OutputContract::LegacyWords,
        MemoryControl::Normal,
        store,
        doc.lineage,
    )?;
    let (clause, _) = cgs_clause(tokenizer, 0, CgsForm::Compute, doc.entity, op)?;
    let mut session = runtime
        .ask(&clause, doc.scope.as_bytes())
        .map_err(|e| format!("compute ask: {e}"))?;
    runtime
        .run(&mut session)
        .map_err(|e| format!("compute run: {e}"))?;
    if session.terminal != Some(ScopedTerminal::Complete) {
        return Ok(None);
    }
    let Some(computed) = session.computation.as_ref() else {
        return Ok(None);
    };
    let Some(capture) = session.captured.as_ref() else {
        return Ok(None);
    };
    Ok(Some(SlxProbe {
        op,
        changed: computed.derived_key != computed.operand_key,
        payload: capture.payload.clone(),
        operand_payload: computed.operand_payload.clone(),
    }))
}

/// One evaluated case.
#[allow(clippy::too_many_arguments)]
fn slx_case(
    tokenizer: &HfBpeTokenizer,
    model_bytes: &[u8],
    intent_bytes: &[u8],
    lexicon_bytes: &[u8],
    backend: &ComputationBackend,
    slots: &[u32],
    realization_bytes: Option<&[u8]>,
    contract: OutputContract,
    control: MemoryControl,
    doc: &SlxDoc,
    store: &[u8],
    case: &str,
    view: SlxView,
    op: Option<&str>,
    expected_payload: &[u32],
    derived: bool,
    changed: bool,
) -> Result<serde_json::Value, String> {
    let mut runtime = slx_load(
        model_bytes,
        intent_bytes,
        lexicon_bytes,
        backend,
        realization_bytes,
        contract,
        control.clone(),
        store,
        doc.lineage,
    )?;
    let control_name = format!("{control:?}");
    let mut session = if let Some(op) = op {
        let (clause, _) = cgs_clause(tokenizer, 0, CgsForm::Compute, doc.entity, op)?;
        runtime
            .ask(&clause, doc.scope.as_bytes())
            .map_err(|e| format!("ask: {e}"))?
    } else {
        runtime
            .ask_view(
                doc.scope.as_bytes(),
                0,
                doc.entity.as_bytes(),
                view.history(),
            )
            .map_err(|e| format!("ask_view: {e}"))?
    };
    let effects = runtime.run(&mut session).map_err(|e| format!("run: {e}"))?;
    let mut expected_with_eos = if matches!(contract, OutputContract::LegacyWords) {
        expected_payload.to_vec()
    } else {
        slx_oracle(slots, view, derived, changed, expected_payload)
    };
    expected_with_eos.push(SLX_EOS);
    let stop_from_table = effects
        .iter()
        .filter(|e| {
            e.realization
                .is_some_and(|d| d.action == RealizationAction::Stop && d.from_table)
        })
        .count();
    let stop_total = effects
        .iter()
        .filter(|e| {
            e.realization
                .is_some_and(|d| d.action == RealizationAction::Stop)
        })
        .count();
    Ok(json!({
        "document": doc.name,
        "fit_document": doc.fit,
        "case": case,
        "view": view.name(),
        "op": op,
        "derived": derived,
        "changed": changed,
        "contract": format!("{contract:?}"),
        "control": control_name,
        "terminal": format!("{:?}", session.terminal),
        "emitted": session.emitted,
        "expected": expected_with_eos,
        "ok": session.terminal == Some(ScopedTerminal::Complete) && session.emitted == expected_with_eos,
        "learned_stop": stop_from_table,
        "stop_steps": stop_total,
        "sl_state_len": session.sl_state.len(),
        "final_session": session,
        "effects": effects,
        "text": cgs_text(tokenizer, &session),
    }))
}

/// Small declared-environment readers so a fit configuration can be swept without a rebuild.
fn slx_env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn slx_env_f32(key: &str, default: f32) -> f32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// The full state-lexical section.
fn slx_run() -> Result<ExitCode, String> {
    let mut root = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/state-lexical-1",
    );
    let mut source_root: Option<PathBuf> = None;
    let mut reload_dir: Option<PathBuf> = None;
    let mut reuse_root: Option<PathBuf> = None;
    {
        let mut a = std::env::args().skip(1);
        while let Some(k) = a.next() {
            match k.as_str() {
                "--root" => root = PathBuf::from(a.next().ok_or("--root value")?),
                "--source-root" => source_root = Some(PathBuf::from(a.next().ok_or("value")?)),
                "--reload-check" => reload_dir = Some(PathBuf::from(a.next().ok_or("value")?)),
                "--reuse-artifact-root" => {
                    reuse_root = Some(PathBuf::from(a.next().ok_or("value")?))
                }
                other if other.starts_with("--mode") => {}
                other => return Err(format!("unknown argument {other}")),
            }
        }
    }
    if let Some(dir) = reload_dir {
        return slx_reload_check(&dir);
    }
    claim(&root).map_err(|e| format!("claim {}: {e}", root.display()))?;
    let started = Instant::now();
    let tb = std::fs::read(DEFAULT_TOKENIZER).map_err(|e| format!("tokenizer: {e}"))?;
    if sha256_hex(&tb) != TOKENIZER_SHA {
        return Err("tokenizer sha mismatch".into());
    }
    if sha256_hex(&derive_tokenizer_json(&tb, VOCAB).map_err(|e| format!("{e}"))?) != DERIVED_SHA {
        return Err("derived tokenizer sha mismatch".into());
    }
    let tokenizer = derive_tokenizer(&tb, VOCAB).map_err(|e| format!("derive: {e}"))?;
    let source_files: Vec<serde_json::Value> = match &source_root {
        Some(sr) => [
            "crates/uor-r4-core/src/native_geometric/learner/scoped_memory.rs",
            "crates/uor-r4-core/src/native_geometric/learner/state_lexical.rs",
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

    let (model_bytes, intent_bytes, lexicon_bytes, backend, slots) = slx_artifacts(&tokenizer)?;
    write_checked(&root, "artifacts/model.json", &model_bytes)?;
    write_checked(&root, "artifacts/intent.json", &intent_bytes)?;
    write_checked(&root, "artifacts/lexicon.json", &lexicon_bytes)?;

    // ---- per-document stores and computation probes ----
    let mut stores: BTreeMap<&'static str, Vec<u8>> = BTreeMap::new();
    let mut probes: BTreeMap<&'static str, Vec<SlxProbe>> = BTreeMap::new();
    let mut ingest_receipts = Vec::new();
    for doc in SLX_DOCS.iter() {
        let store = slx_doc_store(
            &tokenizer,
            doc,
            &model_bytes,
            &intent_bytes,
            &lexicon_bytes,
            &backend,
        )?;
        let mut doc_probes = Vec::new();
        if doc.computed {
            for op in ["e", "i", "j"] {
                if let Some(probe) = slx_probe_compute(
                    &tokenizer,
                    &model_bytes,
                    &intent_bytes,
                    &lexicon_bytes,
                    &backend,
                    &store,
                    doc,
                    op,
                )? {
                    doc_probes.push(probe);
                }
            }
        }
        ingest_receipts.push(json!({
            "document": doc.name,
            "scope": doc.scope,
            "entity": doc.entity,
            "values": doc.values,
            "fit": doc.fit,
            "computed": doc.computed,
            "store_bytes": store.len(),
            "probes": doc_probes.iter().map(|p| json!({
                "op": p.op, "changed": p.changed,
                "answer": cgs_decode_trim(&tokenizer, &p.payload),
            })).collect::<Vec<_>>(),
        }));
        stores.insert(doc.name, store);
        probes.insert(doc.name, doc_probes);
    }

    // ---- declared development text -> teacher-forced sequences (fit documents only) ----
    //
    // Each tag records the declared *meaning* the sentence expresses, so the training set can be
    // balanced and the unidentified-content regime exercised without inventing targets.
    let mut tagged: Vec<(SlSequence, &'static str, RealizationContext)> = Vec::new();
    let mut fit_text_receipts = Vec::new();
    for doc in SLX_DOCS.iter().filter(|d| d.fit) {
        // Observe the comparator's actual serving context from the same loaded session. Its
        // capture/history observations come from serving, rather than the authored answer family;
        // a historical capture can itself have a different predecessor.
        // Observe the comparator's actual serving context from the same loaded session. Its
        // capture/history observations come from serving, rather than the authored answer family;
        // a historical capture can itself have a different predecessor. The typed causal facts are
        // read from the same executed session, so no training observation is manufactured from an
        // answer-family label.
        let observe = |view: SlxView,
                       op: Option<&str>|
         -> Result<(RealizationContext, (bool, bool, bool)), String> {
            let mut runtime = slx_load(
                &model_bytes,
                &intent_bytes,
                &lexicon_bytes,
                &backend,
                None,
                OutputContract::LegacyWords,
                MemoryControl::Normal,
                &stores[doc.name],
                doc.lineage,
            )?;
            let mut session = if let Some(op) = op {
                let (clause, _) = cgs_clause(&tokenizer, 0, CgsForm::Compute, doc.entity, op)?;
                runtime
                    .ask(&clause, doc.scope.as_bytes())
                    .map_err(|e| e.to_string())?
            } else {
                runtime
                    .ask_view(
                        doc.scope.as_bytes(),
                        0,
                        doc.entity.as_bytes(),
                        view.history(),
                    )
                    .map_err(|e| e.to_string())?
            };
            runtime.run(&mut session).map_err(|e| e.to_string())?;
            Ok((
                runtime.realization_context(&session),
                runtime.observed_lexical_facts(&session),
            ))
        };
        // current
        let current = doc.values.last().copied().unwrap();
        let (ctx, (committed, prior_differs, key_changed)) = observe(SlxView::Current, None)?;
        let seq = slx_text_sequence(
            &tokenizer,
            &slots,
            SlxView::Current,
            false,
            false,
            current,
            &[],
            prior_differs,
            committed,
            key_changed,
        )?;
        fit_text_receipts.push(json!({"document": doc.name, "case": "current", "text": slx_render(SlxView::Current, false, false, current)}));
        tagged.push((seq, "flag", ctx));
        // previous and initial
        if doc.values.len() >= 2 {
            let prev = doc.values[doc.values.len() - 2];
            let (ctx, (committed, prior_differs, key_changed)) = observe(SlxView::Previous, None)?;
            let seq = slx_text_sequence(
                &tokenizer,
                &slots,
                SlxView::Previous,
                false,
                false,
                prev,
                &[],
                prior_differs,
                committed,
                key_changed,
            )?;
            fit_text_receipts.push(json!({"document": doc.name, "case": "previous", "text": slx_render(SlxView::Previous, false, false, prev)}));
            tagged.push((seq, "flag", ctx));
        }
        let first = doc.values[0];
        let (ctx, (committed, prior_differs, key_changed)) = observe(SlxView::Initial, None)?;
        let seq = slx_text_sequence(
            &tokenizer,
            &slots,
            SlxView::Initial,
            false,
            false,
            first,
            &[],
            prior_differs,
            committed,
            key_changed,
        )?;
        fit_text_receipts.push(json!({"document": doc.name, "case": "initial", "text": slx_render(SlxView::Initial, false, false, first)}));
        tagged.push((seq, "flag", ctx));
        // consumed computations discovered through the retained session
        for probe in probes.get(doc.name).map(Vec::as_slice).unwrap_or(&[]) {
            let answer = cgs_decode_trim(&tokenizer, &probe.payload);
            let (ctx, (committed, prior_differs, key_changed)) =
                observe(SlxView::Current, Some(probe.op))?;
            let seq = slx_text_sequence(
                &tokenizer,
                &slots,
                SlxView::Current,
                true,
                probe.changed,
                &answer,
                &probe.operand_payload,
                prior_differs,
                committed,
                key_changed,
            )?;
            let tag = if probe.changed {
                "changed"
            } else {
                "unchanged"
            };
            fit_text_receipts.push(json!({
                "document": doc.name, "case": if probe.changed {"derived_changed"} else {"derived_unchanged"},
                "op": probe.op, "text": slx_render(SlxView::Current, true, probe.changed, &answer),
            }));
            tagged.push((seq, tag, ctx));
        }
    }
    // Two declared balancing decisions, made before fitting and reported with the corpus:
    // * every flag-only sentence is also presented with no fitted content, so an unfamiliar value
    //   degrades to the flag-driven sequence instead of an arbitrary one;
    // * the change-reporting branch is rare in the declared text (one identity operation per
    //   document against two moving ones), so it is repeated to the same order as its siblings
    //   rather than left for the majority class to drown.
    let declared_sequences = tagged.len();
    let mut fit_sequences: Vec<SlSequence> = Vec::new();
    let mut comparator_contexts = Vec::new();
    for (seq, tag, context) in &tagged {
        fit_sequences.push(seq.clone());
        comparator_contexts.push(*context);
        if *tag == "flag" {
            // Present the same sentence with unfamiliar-but-shaped content at every payload length the
            // language can present, so an unfamiliar value of any length meets a fitted OOV path
            // rather than a zero-length payload the session never serves.
            for len in 1..=(SL_MAX_CONTENT_TOKENS + 1) {
                fit_sequences.push(slx_reblank(seq, len, SLX_OOV_TOKENS[0]));
                comparator_contexts.push(*context);
            }
        }
        if *tag == "unchanged" {
            for _ in 0..3 {
                fit_sequences.push(seq.clone());
                comparator_contexts.push(*context);
            }
        }
        if *tag == "changed" {
            // The moving operations are twice as frequent as the identity one, so the identity branch
            // is repeated above. The change-reporting answer also carries the sole word that follows
            // the owned span, which is otherwise drowned by the many copy steps, so that branch is
            // repeated to give the post-copy decision comparable support.
            for _ in 0..4 {
                fit_sequences.push(seq.clone());
                comparator_contexts.push(*context);
            }
        }
    }

    let cfg = SlFitConfig {
        epochs: slx_env_usize("SLX_EPOCHS", 16000),
        h_dim: slx_env_usize("SLX_H", 48),
        e_dim: slx_env_usize("SLX_E", 48),
        lr: slx_env_f32("SLX_LR", 0.01),
        seed: slx_env_usize("SLX_SEED", 0x5eed_1eaf) as u64,
        stop_weight: slx_env_f32("SLX_STOP", 4.0),
        insert_weight: slx_env_f32("SLX_INS", 3.0),
    };
    let outcome = if let Some(parent) = &reuse_root {
        let unlisted = verify(parent).map_err(|e| format!("parent seal: {e}"))?;
        if !unlisted.is_empty() {
            return Err(format!("parent has unlisted files: {unlisted:?}"));
        }
        let result: serde_json::Value = serde_json::from_slice(
            &std::fs::read(parent.join("result.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        for (name, bytes) in [
            ("model.json", model_bytes.as_slice()),
            ("intent.json", intent_bytes.as_slice()),
            ("lexicon.json", lexicon_bytes.as_slice()),
        ] {
            if std::fs::read(parent.join("artifacts").join(name)).map_err(|e| e.to_string())?
                != bytes
            {
                return Err(format!("reconstructed parent {name} differs"));
            }
        }
        SlFitOutcome {
            model: StateLexicalModel::from_bytes(
                &std::fs::read(parent.join("artifacts/state_lexical.json"))
                    .map_err(|e| e.to_string())?,
                VOCAB,
            )
            .map_err(|e| e.to_string())?,
            // Historical diagnostics are preserved as inherited, never labelled as a new fit.
            report: serde_json::from_value(result["fit"].clone()).map_err(|e| e.to_string())?,
        }
    } else {
        let tries = slx_env_usize("SLX_TRIES", 12);
        let mut best: Option<SlFitOutcome> = None;
        for k in 0..tries {
            let mut attempt = cfg;
            attempt.seed = cfg
                .seed
                .wrapping_add((k as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
            let out = fit_state_lexical(
                &fit_sequences,
                VOCAB,
                slots.clone(),
                SLX_MAX_INSERT,
                &attempt,
                &SLX_OOV_TOKENS,
            )
            .map_err(|e| format!("state-lexical fit: {e}"))?;
            // Selection is on the declared *training* objective only; the evaluation cases below are not
            // consulted. A tiny ternary objective is seed-sensitive, so the best of a few declared seeds
            // is taken rather than a single draw.
            let better = match &best {
                None => true,
                Some(b) => {
                    out.report.action_correct > b.report.action_correct
                        || (out.report.action_correct == b.report.action_correct
                            && out.report.served_loss < b.report.served_loss)
                }
            };
            if better {
                best = Some(out);
            }
        }
        best.ok_or("no state-lexical fit attempt")?
    };
    let realization_bytes = outcome.model.to_bytes().map_err(|e| e.to_string())?;
    write_checked(&root, "artifacts/state_lexical.json", &realization_bytes)?;
    let reloaded = StateLexicalModel::from_bytes(
        &std::fs::read(root.join("artifacts/state_lexical.json")).map_err(|e| e.to_string())?,
        VOCAB,
    )
    .map_err(|e| e.to_string())?;
    if reloaded != outcome.model {
        return Err("state-lexical artifact reload mismatch".into());
    }

    let teacher_forced_replay = reloaded.teacher_forced_agreement(&fit_sequences);

    // ---- the retained finite-table comparator, fitted on the same declared text ----
    let mut comparator_examples: Vec<RealizationExample> = Vec::new();
    assert_eq!(fit_sequences.len(), comparator_contexts.len());
    for (ex, context) in fit_sequences.iter().zip(&comparator_contexts) {
        let stages = ex.copy_stages();
        let mut inserted = 0usize;
        for (i, action) in ex.actions.iter().enumerate() {
            comparator_examples.push(RealizationExample {
                context: RealizationContext {
                    copy_stage: stages[i],
                    emitted_bucket: inserted.min(3) as u8,
                    ..*context
                },
                action: *action,
            });
            if matches!(action, RealizationAction::Insert(_)) {
                inserted += 1;
            }
        }
    }
    let comparator = fit_realization(&comparator_examples, VOCAB, slots.clone(), 8)
        .map_err(|e| format!("comparator fit: {e}"))?;
    let comparator_bytes = comparator.to_bytes().map_err(|e| e.to_string())?;
    write_checked(&root, "artifacts/comparator.json", &comparator_bytes)?;

    // ---- evaluation: retained copy, retained finite table, learned state decoder, and controls ----
    let mut cases: Vec<serde_json::Value> = Vec::new();
    for doc in SLX_DOCS.iter() {
        let store = &stores[doc.name];
        let current = doc.values.last().copied().unwrap();
        // current / previous / initial
        let mut views = vec![(SlxView::Current, doc.values.len() - 1)];
        if doc.values.len() >= 2 {
            views.push((SlxView::Previous, doc.values.len() - 2));
        }
        views.push((SlxView::Initial, 0));
        for (view, idx) in views {
            let value = doc.values[idx];
            let payload = tokenizer.encode(&format!(" {value}"));
            for (contract, control, lex) in [
                (OutputContract::LegacyWords, MemoryControl::Normal, None),
                (
                    OutputContract::RealizedV1,
                    MemoryControl::Normal,
                    Some(comparator_bytes.as_slice()),
                ),
                (
                    OutputContract::StateLexicalV1,
                    MemoryControl::Normal,
                    Some(realization_bytes.as_slice()),
                ),
                (
                    OutputContract::StateLexicalV1,
                    MemoryControl::RealizationContextDisabled,
                    Some(realization_bytes.as_slice()),
                ),
                (
                    OutputContract::StateLexicalV1,
                    MemoryControl::RecurrenceDisabled,
                    Some(realization_bytes.as_slice()),
                ),
            ] {
                let case = slx_case(
                    &tokenizer,
                    &model_bytes,
                    &intent_bytes,
                    &lexicon_bytes,
                    &backend,
                    &slots,
                    lex,
                    contract,
                    control,
                    doc,
                    store,
                    view.name(),
                    view,
                    None,
                    &payload,
                    false,
                    false,
                )?;
                cases.push(case);
            }
        }
        // consumed computations on the documents whose values are groundable
        for probe in probes.get(doc.name).map(Vec::as_slice).unwrap_or(&[]) {
            for (contract, control, lex) in [
                (OutputContract::LegacyWords, MemoryControl::Normal, None),
                (
                    OutputContract::RealizedV1,
                    MemoryControl::Normal,
                    Some(comparator_bytes.as_slice()),
                ),
                (
                    OutputContract::StateLexicalV1,
                    MemoryControl::Normal,
                    Some(realization_bytes.as_slice()),
                ),
                (
                    OutputContract::StateLexicalV1,
                    MemoryControl::RealizationContextDisabled,
                    Some(realization_bytes.as_slice()),
                ),
                (
                    OutputContract::StateLexicalV1,
                    MemoryControl::RecurrenceDisabled,
                    Some(realization_bytes.as_slice()),
                ),
            ] {
                let case = slx_case(
                    &tokenizer,
                    &model_bytes,
                    &intent_bytes,
                    &lexicon_bytes,
                    &backend,
                    &slots,
                    lex,
                    contract,
                    control,
                    doc,
                    store,
                    if probe.changed {
                        "derived_changed"
                    } else {
                        "derived_unchanged"
                    },
                    SlxView::Current,
                    Some(probe.op),
                    &probe.payload,
                    true,
                    probe.changed,
                )?;
                cases.push(case);
            }
        }
    }

    // ---- panels ----
    let panel = |filter: &dyn Fn(&serde_json::Value) -> bool| -> serde_json::Value {
        let rows: Vec<&serde_json::Value> = cases.iter().filter(|c| filter(c)).collect();
        let ok = rows.iter().filter(|c| c["ok"] == json!(true)).count();
        json!({"cases": rows.len(), "ok": ok, "complete": ok == rows.len() && !rows.is_empty()})
    };
    let panels = json!({
        "legacy_copy_retained": panel(&|c| c["contract"] == json!("LegacyWords")),
        "retained_finite_table": panel(&|c| c["contract"] == json!("RealizedV1") && c["control"] == json!("Normal")),
        "learned_state_lexical": panel(&|c| c["contract"] == json!("StateLexicalV1") && c["control"] == json!("Normal")),
        "learned_fit_documents": panel(&|c| c["contract"] == json!("StateLexicalV1") && c["control"] == json!("Normal") && c["fit_document"] == json!(true)),
        "learned_held_out_documents": panel(&|c| c["contract"] == json!("StateLexicalV1") && c["control"] == json!("Normal") && c["fit_document"] == json!(false)),
        "context_disabled": panel(&|c| c["control"] == json!("RealizationContextDisabled")),
        "recurrence_disabled": panel(&|c| c["control"] == json!("RecurrenceDisabled")),
    });

    // ---- paired computed requests: not a payload-only intervention ----
    let mut value_sensitive = Vec::new();
    for doc in SLX_DOCS.iter().filter(|d| d.fit && d.computed) {
        let doc_probes = probes.get(doc.name).map(Vec::as_slice).unwrap_or(&[]);
        let unchanged = doc_probes.iter().find(|p| !p.changed);
        let changed = doc_probes.iter().find(|p| p.changed);
        if let (Some(u), Some(c)) = (unchanged, changed) {
            let run = |probe: &SlxProbe| -> Result<(Vec<u32>, Vec<i32>), String> {
                let mut runtime = slx_load(
                    &model_bytes,
                    &intent_bytes,
                    &lexicon_bytes,
                    &backend,
                    Some(&realization_bytes),
                    OutputContract::StateLexicalV1,
                    MemoryControl::Normal,
                    &stores[doc.name],
                    doc.lineage,
                )?;
                let (clause, _) =
                    cgs_clause(&tokenizer, 0, CgsForm::Compute, doc.entity, probe.op)?;
                let mut session = runtime
                    .ask(&clause, doc.scope.as_bytes())
                    .map_err(|e| e.to_string())?;
                runtime.run(&mut session).map_err(|e| e.to_string())?;
                Ok((session.emitted, session.sl_state))
            };
            let (u_emitted, u_state) = run(u)?;
            let (c_emitted, c_state) = run(c)?;
            value_sensitive.push(json!({
                "document": doc.name,
                "legacy_flags_held_equal": "derived=true, history=current, prior_differs=false for both; key_changed and operation differ",
                "unchanged_op": u.op, "changed_op": c.op,
                "unchanged_emitted": u_emitted, "changed_emitted": c_emitted,
                "differ": u_emitted != c_emitted,
                "final_recurrent_state_differs": u_state != c_state,
            }));
        }
    }

    // ---- restart: every emission boundary in process, and one cross-process resume ----
    let restart_doc = SLX_DOCS
        .iter()
        .find(|d| d.fit && d.computed)
        .ok_or("no computed fitting document")?;
    let restart_runtime = slx_load(
        &model_bytes,
        &intent_bytes,
        &lexicon_bytes,
        &backend,
        Some(&realization_bytes),
        OutputContract::StateLexicalV1,
        MemoryControl::Normal,
        &stores[restart_doc.name],
        restart_doc.lineage,
    )?;
    let restart_store = &stores[restart_doc.name];
    let mut session = restart_runtime
        .ask_view(
            restart_doc.scope.as_bytes(),
            0,
            restart_doc.entity.as_bytes(),
            HistoryView::Initial,
        )
        .map_err(|e| e.to_string())?;
    // Collect a snapshot at every boundary, then restore each and run to completion.
    let mut frames: Vec<(Vec<u8>, serde_json::Value)> = Vec::new();
    loop {
        let bytes = restart_runtime
            .snapshot(&session)
            .map_err(|e| e.to_string())?;
        let mut copy = restart_runtime.restore(&bytes).map_err(|e| e.to_string())?;
        let remaining = restart_runtime.run(&mut copy).map_err(|e| e.to_string())?;
        frames.push((
            bytes,
            json!({"session": copy, "remaining_effects": remaining}),
        ));
        if session.terminal.is_some() {
            break;
        }
        restart_runtime
            .step(&mut session)
            .map_err(|e| e.to_string())?;
    }
    let final_emitted = session.emitted.clone();
    let restart_ok = frames
        .iter()
        .all(|(_, continuation)| continuation["session"] == json!(session));
    // Persist the frames and the artifacts so a separate process can resume them.
    let restart_dir = root.join("restart");
    std::fs::create_dir_all(&restart_dir).map_err(|e| e.to_string())?;
    write_checked(&root, "restart/model.json", &model_bytes)?;
    write_checked(&root, "restart/intent.json", &intent_bytes)?;
    write_checked(&root, "restart/lexicon.json", &lexicon_bytes)?;
    write_checked(&root, "restart/state_lexical.json", &realization_bytes)?;
    write_checked(&root, "restart/store.bin", &stores[restart_doc.name])?;
    let snapshots: Vec<serde_json::Value> = frames
        .iter()
        .enumerate()
        .map(|(i, (bytes, continuation))| -> Result<serde_json::Value, String> {
            let name = format!("restart/frame-{i:03}.json");
            write_checked(&root, &name, bytes)?;
            Ok(json!({"index": i, "frame": format!("frame-{i:03}.json"), "continuation": continuation}))
        })
        .collect::<Result<_, _>>()?;
    write_json(
        &root,
        "restart/manifest.json",
        &json!({
            "lineage": restart_doc.lineage,
            "scope": restart_doc.scope,
            "entity": restart_doc.entity,
            "view": "initial",
            "expected": final_emitted,
            "frames": snapshots,
        }),
    )?;
    let child = std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
        .args([
            "--mode=state-lexical",
            "--reload-check",
            restart_dir.to_str().ok_or("restart dir")?,
        ])
        .output()
        .map_err(|e| format!("child: {e}"))?;
    let child_ok = child.status.success();
    let child_report: serde_json::Value = if child_ok {
        serde_json::from_slice(
            &std::fs::read(restart_dir.join("child.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?
    } else {
        json!({"error": String::from_utf8_lossy(&child.stderr).to_string()})
    };

    // Exact representational diagnostics on the loaded candidate, not language acceptance.
    let unknown: Vec<u32> = (0..VOCAB as u32)
        .filter(|t| !reloaded.content_tokens.contains(t))
        .take(2)
        .collect();
    let known: Vec<u32> = reloaded
        .content_tokens
        .iter()
        .skip(1)
        .copied()
        .take(4)
        .collect();
    let mut reversed = known.clone();
    reversed.reverse();
    let mut longer = known.clone();
    if let Some(t) = known.get(1) {
        longer.push(*t);
    }
    // The decoder consumes the content feature *and* the typed causal block, so the representational
    // witness is measured over the full observation, not the content feature alone.
    let observe = |sel: &[u32], res: &[u32]| -> (Vec<i32>, Vec<i32>) {
        (
            reloaded.content_feature(sel, res),
            reloaded.typed_block(sel, res, SlFacts::default()),
        )
    };
    let unknown_alias = if unknown.len() == 2 {
        Some(observe(&unknown[..1], &unknown[..1]) == observe(&unknown[..1], &unknown[1..]))
    } else {
        None
    };
    let unknown_content_alias = if unknown.len() == 2 {
        Some(
            reloaded.content_feature(&unknown[..1], &unknown[..1])
                == reloaded.content_feature(&unknown[..1], &unknown[1..]),
        )
    } else {
        None
    };
    let representation_diagnostics = json!({
        "unknown_token_ids": unknown,
        "content_only_unknown_pair_alias": unknown_content_alias,
        "equal_and_unequal_unknown_pairs_alias": unknown_alias,
        "known_token_ids": known,
        "reordered_known_tokens_alias": reloaded.content_feature(&known, &[]) == reloaded.content_feature(&reversed, &[]),
        "suffix_after_four_tokens_alias": if known.len() == 4 { Some(reloaded.content_feature(&known, &[]) == reloaded.content_feature(&longer, &[])) } else { None },
        "copy_token_feedback": "the emitted token's shared learned identity enters the recurrence; an unfitted token shares the reserved OOV row",
        "tail_position": "a payload longer than four tokens carries its final token in a dedicated tail position",
        "scope": "loaded representation witnesses only; not a language-quality panel",
    });

    // ---- loaded E/S reference assessment ----
    // Exercise each loaded donor at its own validated interface. This is a
    // component probe, not a matched scoped-span or language-quality comparison.
    let es_reference = {
        let es_started = Instant::now();
        let pb = std::fs::read(E_PATH).map_err(|e| format!("E: {e}"))?;
        let sb = std::fs::read(S_PATH).map_err(|e| format!("S: {e}"))?;
        let e_sha_ok = sha256_hex(&pb) == E_SHA;
        let s_sha_ok = sha256_hex(&sb) == S_SHA;
        let parent = PriorCore::from_bytes(&pb).map_err(|e| format!("load E: {e}"))?;
        let e_valid = parent.validate().is_ok();
        let (parent_digest, _) = parent_hash_convention(&pb);
        let raw_tok: [u8; 32] = hex_to_bytes(DERIVED_SHA)?
            .try_into()
            .map_err(|_| "derived tokenizer digest width".to_string())?;
        let query = QueryHard::from_bytes(&sb, &parent, &parent_digest, &raw_tok)
            .map_err(|e| format!("load S: {e}"))?;
        // Three prompt tokens make the S older-prefix query eligible on the
        // first generated step. The row receipt proves S was actually invoked.
        let prompt: [u32; 3] = [slots[0], slots[4], slots[1]];
        let s_rows = query
            .inference_rows_checked(&prompt, prompt.len() - 1)
            .map_err(|e| format!("S inference rows: {e}"))?;
        let continued = parent.generate(&prompt, 4, true);
        let s_continued = query
            .generate_checked(&prompt, 4)
            .map_err(|e| format!("S generation: {e}"))?;
        let slot_set: std::collections::BTreeSet<u32> = slots.iter().copied().collect();
        let in_slots = continued.iter().filter(|t| slot_set.contains(t)).count();
        json!({
            "status": "E_AND_S_COMPONENT_PROBED",
            "e_bytes": pb.len(),
            "s_bytes": sb.len(),
            "e_sha_matches_pin": e_sha_ok,
            "s_sha_matches_pin": s_sha_ok,
            "e_valid": e_valid,
            "probe_prompt": prompt,
            "donor_continuation": continued,
            "s_inference_rows": s_rows,
            "s_continuation": s_continued,
            "continuation_tokens_in_declared_slots": in_slots,
            "load_and_probe_ms": es_started.elapsed().as_millis() as u64,
            "verdict": "E and S load and execute separate full-vocabulary continuation probes. S older-prefix inference rows are recorded. Neither donor emits or owns the session's exact copied span, and this is not a matched lexical-quality comparison. A donor-conditioned vocabulary decision over an exactly owned span remains an interface hypothesis.",
        })
    };

    // ---- ordinary-text exposure on a source-separated prose population ----
    // A measured statement of the ordinary-text remainder: how much of real prose the fitted content
    // index covers. This is exposure accounting, not a language-quality panel.
    let ordinary_text = {
        let docs = [
            "docs/integration/current-state.md",
            "docs/integration/project-track.md",
        ];
        let mut rows = Vec::new();
        for rel in docs {
            let Some(root_dir) = source_root.as_ref() else {
                rows.push(json!({"path": rel, "status": "UNAVAILABLE (no source root)"}));
                continue;
            };
            match std::fs::read_to_string(root_dir.join(rel)) {
                Ok(text) => {
                    let toks = tokenizer.encode(&text);
                    let fitted = toks
                        .iter()
                        .filter(|t| reloaded.content_tokens.iter().skip(1).any(|c| c == *t))
                        .count();
                    let fraction = if toks.is_empty() {
                        0.0
                    } else {
                        fitted as f64 / toks.len() as f64
                    };
                    rows.push(json!({
                        "path": rel,
                        "tokens": toks.len(),
                        "fitted_tokens": fitted,
                        "fitted_fraction": fraction,
                    }));
                }
                Err(e) => rows.push(json!({"path": rel, "status": format!("UNAVAILABLE ({e})")})),
            }
        }
        json!({
            "population": "source-separated local project documentation, distinct from the authored state world",
            "documents": rows,
            "verdict": "NOT_RUN as a learning task. The scoped contract supervises an exactly owned span, not open prose continuation, and the fitted content index does not cover ordinary vocabulary (measured coverage above). Ordinary-text learning through the retained session remains the open remainder of the milestone and is the first item of the recorded next step.",
        })
    };

    // ---- checks ----
    let holds = |name: &str| panels[name]["complete"] == json!(true);
    let checks = json!({
        "learned_fit_documents_all_expected": holds("learned_fit_documents"),
        "legacy_copy_retained": holds("legacy_copy_retained"),
        "value_sensitive_effect": value_sensitive.iter().any(|v| v["differ"] == json!(true)),
        "recurrence_ablation_changes_output": panels["recurrence_disabled"]["cases"]
            .as_u64()
            .unwrap_or(0)
            > 0
            && panels["recurrence_disabled"]["complete"] == json!(false),
        "context_ablation_changes_output": panels["context_disabled"]["cases"]
            .as_u64()
            .unwrap_or(0)
            > 0
            && panels["context_disabled"]["complete"] == json!(false),
        "restart_all_boundaries": restart_ok,
        "restart_child_process": child_ok && child_report["ok"] == json!(true),
        "artifact_reload_identity": true,
    });

    let payload = json!({
        "schema": "uor-r4.state-lexical/1",
        "source_files": source_files,
        "executable_sha256": sha256_hex(&std::fs::read(std::env::current_exe().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?),
        "reuse_artifact_root": reuse_root,
        "fit_diagnostics_inherited_without_neural_refit": reuse_root.is_some(),
        "teacher_forced_replay": {"correct": teacher_forced_replay.0, "total": teacher_forced_replay.1},
        "evaluation_exposure": "exposed authored panel; replay is not a fresh acceptance draw",
        "finite_comparator_fit": "shared runtime causal context; actual inserted-count and copy-stage progress",
        "finite_comparator_training_contexts": comparator_contexts,

        "artifact_sha256": {
            "model.json": sha256_hex(&model_bytes),
            "intent.json": sha256_hex(&intent_bytes),
            "lexicon.json": sha256_hex(&lexicon_bytes),
            "state_lexical.json": sha256_hex(&realization_bytes),
            "comparator.json": sha256_hex(&comparator_bytes),
        },
        "slots": slots,
        "fit": {
            "sequences": outcome.report.sequences,
            "steps": outcome.report.steps,
            "epochs": outcome.report.epochs,
            "first_loss": outcome.report.first_loss,
            "final_loss": outcome.report.final_loss,
            "served_loss": outcome.report.served_loss,
            "action_correct": outcome.report.action_correct,
            "action_total": outcome.report.action_total,
            "h_dim": outcome.report.h_dim,
            "e_dim": outcome.report.e_dim,
            "content_tokens": outcome.report.content_tokens,
        },
        "corpus": {
            "documents": SLX_DOCS.len(),
            "fit_documents": SLX_DOCS.iter().filter(|d| d.fit).count(),
            "held_out_documents": SLX_DOCS.iter().filter(|d| !d.fit).count(),
            "declared_sequences": declared_sequences,
            "training_sequences": outcome.report.sequences,
            "declared_text": fit_text_receipts,
            "documents_detail": ingest_receipts,
        },
        "panels": panels,
        "value_sensitive": value_sensitive,
        "representation_diagnostics": representation_diagnostics,
        "restart": {
            "boundaries": frames.len(),
            "all_equal_final": restart_ok,
            "child_process": child_ok,
            "child": child_report,
        },
        "cases": cases,
        "checks": checks,
        "checks_all_expected": checks.as_object().map(|m| m.values().all(|v| v == &json!(true))).unwrap_or(false),
        "scope": "exposed authored response fixture over a small declared state world; a learned low-bit state-conditioned decoder selects shared vocabulary words around the exact owned span, and continues after the span, from position-resolved content, the actual emitted token identity, and an exact typed causal block. Not general prose and no geometric advantage claimed; the held-out-document panel transfers the sentence form to unfamiliar entity/value strings of unseen length, not an unfamiliar computed relation.",
        "es_reference": es_reference,
        "ordinary_text_exposure": ordinary_text,
    });
    write_json(&root, "result.json", &payload)?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "state-lexical: fit {} seq {} steps {}/{} | panels {:?} | sealed {} unlisted | {:.1}s",
        outcome.report.sequences,
        outcome.report.steps,
        outcome.report.action_correct,
        outcome.report.action_total,
        panels,
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    if !(payload["checks_all_expected"] == json!(true)) {
        return Err(format!("sealed contract failure: {}", payload["checks"]));
    }
    let _ = (started, unlisted);
    Ok(ExitCode::SUCCESS)
}

/// Resume saved frames in a separate process and report the completed emissions for each.
fn slx_reload_check(dir: &std::path::Path) -> Result<ExitCode, String> {
    let model_bytes = std::fs::read(dir.join("model.json")).map_err(|e| e.to_string())?;
    let intent_bytes = std::fs::read(dir.join("intent.json")).map_err(|e| e.to_string())?;
    let lexicon_bytes = std::fs::read(dir.join("lexicon.json")).map_err(|e| e.to_string())?;
    let realization_bytes =
        std::fs::read(dir.join("state_lexical.json")).map_err(|e| e.to_string())?;
    let store = std::fs::read(dir.join("store.bin")).map_err(|e| e.to_string())?;
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(dir.join("manifest.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let lineage = manifest["lineage"].as_u64().ok_or("lineage")?;
    let expected = manifest["expected"].clone();

    let tokenizer = {
        let tb = std::fs::read(DEFAULT_TOKENIZER).map_err(|e| e.to_string())?;
        derive_tokenizer(&tb, VOCAB).map_err(|e| e.to_string())?
    };
    let _ = tokenizer;
    let action = CgsAction::build()?;
    let examples = cgs_examples(&action)?;
    let (factorization, _) =
        factor_observed_graph(&examples, false).map_err(|e| format!("factorization: {e}"))?;
    let backend = ComputationBackend::Signed(Box::new(factorization));

    let mut frames_ok = 0usize;
    let mut frames = Vec::new();
    for entry in manifest["frames"].as_array().ok_or("frames")? {
        let path = dir.join(entry["frame"].as_str().ok_or("frame path")?);
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let runtime = slx_load(
            &model_bytes,
            &intent_bytes,
            &lexicon_bytes,
            &backend,
            Some(&realization_bytes),
            OutputContract::StateLexicalV1,
            MemoryControl::Normal,
            &store,
            lineage,
        )?;
        let mut session = runtime.restore(&bytes).map_err(|e| e.to_string())?;
        let remaining = runtime.run(&mut session).map_err(|e| e.to_string())?;
        let continuation = json!({"session": session, "remaining_effects": remaining});
        let full_parity = entry.get("continuation").map(|v| *v == continuation);
        let ok = json!(session.emitted) == expected && full_parity.unwrap_or(true);
        if ok {
            frames_ok += 1;
        }
        frames.push(json!({"index": entry["index"], "emitted": session.emitted, "ok": ok, "full_continuation_equal": full_parity, "continuation": continuation}));
    }
    write_json(
        dir,
        "child.json",
        &json!({
            "frames": frames,
            "frames_ok": frames_ok,
            "expected": expected,
            "ok": frames_ok == manifest["frames"].as_array().map(Vec::len).unwrap_or(0),
            "pid": std::process::id(),
        }),
    )?;
    if frames.is_empty() || frames_ok != frames.len() {
        return Err("state-lexical child continuation mismatch".into());
    }
    Ok(ExitCode::SUCCESS)
}
