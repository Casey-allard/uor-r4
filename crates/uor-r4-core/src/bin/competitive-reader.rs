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
const SEED_FIT: u64 = 0x5C0F_1E71;
const SEED_FRESH: u64 = 0x5C0F_7F2E;
/// Prospectively declared primary margin: paired hard-action CE improvement over the exact reader.
const MARGIN_HARD_BITS: f64 = 0.05;
/// Tolerated all-position regression on the natural-text development regression.
const MARGIN_TEXT_BITS: f64 = 0.05;
const GEN_TOKENS: usize = 48;
const TEXT_WINDOWS: usize = 8;
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

    let build_positions = |mode: RelMode, code_of: &[u8], roots: &[u8]| -> Vec<TrainPos> {
        let mut v = Vec::new();
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
            let mut delta = Vec::new();
            for c in o.cands.iter() {
                let p = prob_of(&z, c.payload, parent.cfg.f_bits);
                delta.push(std::array::from_fn(|a| {
                    let boost = (1i64 << AMP_SHIFTS[a]) as f64 / (1i64 << parent.cfg.f_bits) as f64;
                    action_loss(p, boost, c.payload == o.target)
                }));
            }
            v.push(TrainPos {
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
            });
        }
        let _ = (mode, code_of, roots);
        v
    };
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

    // ---- fit the arms on one pool, one dose, one objective ----------------------
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
        let positions = build_positions(mode, &init_codes, &init_roots);
        let t0 = Instant::now();
        for _ in 0..STEPS {
            let b = tr.next_batch(positions.len(), BATCH);
            tr.step(&positions, &b);
        }
        // Both learned arms get the same mode-aware bounded coordinate-search opportunity.
        let before_geom: f64 = positions.iter().map(|p| tr.position_loss(p)).sum();
        let search_change = if mode != RelMode::ExactOnly {
            tr.refine_descriptor(&positions, ROUNDS)
        } else {
            0.0
        };
        let after: f64 = positions.iter().map(|p| tr.position_loss(p)).sum();
        let sel = tr.quantize();
        sel.validate_for_vocab(parent.cfg.vocab)?;
        arm_report.push(json!({
            "arm": name,
            "mode": format!("{mode:?}"),
            "steps": tr.step,
            "seconds": t0.elapsed().as_secs_f64(),
            "descriptor_assignments_moved": tr.descriptor_moved(),
            "descriptor_evaluations": tr.descriptor_evaluations,
            "fit_objective_before_search": before_geom,
            "fit_objective_after_search": after,
            "search_change": search_change,
            "code_search_change": if mode == RelMode::Categorical { search_change } else { 0.0 },
            "quantized": {"w": sel.w.to_vec(), "bias": sel.bias, "sb": sel.sb.to_vec(), "noread": sel.noread,
                          "rank_nonzero": sel.rank.iter().filter(|v| **v != 0).count()},
        }));
        arms.insert(name, sel);
    }
    let relational = arms["relational"].clone();
    let exact = arms["exact"].clone();
    let categorical = arms["categorical"].clone();
    mark("fit", &mut marks);

    // ---- artifact export / independent reload ----------------------------------
    let art = RelationalArtifact {
        selector: relational.clone(),
        local_artifact_digest: sha256_bytes(&sb),
        tokenizer_digest: raw_tok,
        data_digest: data_id,
    };
    let bytes = art.to_bytes();
    write_checked(&root, "artifacts/relational_reader.rlr2", &bytes)?;
    let reloaded = RelationalArtifact::from_bytes(&bytes, &sha256_bytes(&sb), &raw_tok)?;
    reloaded.selector.validate_for_vocab(parent.cfg.vocab)?;
    if reloaded.selector != relational {
        return Err("reloaded artifact differs from the fitted selector".into());
    }
    // Every arm must survive the same independent reload path.
    let mut arm_parity_failures = 0usize;
    for (name, sel) in arms.iter() {
        let a = RelationalArtifact {
            selector: sel.clone(),
            local_artifact_digest: sha256_bytes(&sb),
            tokenizer_digest: raw_tok,
            data_digest: data_id,
        };
        match RelationalArtifact::from_bytes(&a.to_bytes(), &sha256_bytes(&sb), &raw_tok) {
            Ok(b) => {
                if b.selector != *sel || b.selector.validate_for_vocab(parent.cfg.vocab).is_err() {
                    arm_parity_failures += 1;
                }
            }
            Err(_) => arm_parity_failures += 1,
        }
        let _ = name;
    }
    check_reload(
        arms.get("categorical").ok_or("cat")?,
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
        let arm_specs: Vec<(&'static str, Option<&RelationalSelector>, bool)> = vec![
            ("local", None, false),
            ("exact", Some(&exact), false),
            ("categorical", Some(&categorical), false),
            ("relational", Some(&relational), false),
            ("relational_query_blind", Some(&relational), true),
        ];
        for (name, sel, blind) in arm_specs {
            let mut n = 0usize;
            let mut reads = 0usize;
            let mut correct_reads = 0usize;
            let mut covered = 0usize;
            let mut correct_covered = 0usize;
            let mut hard_ce = 0.0f64;
            let mut emitted_ok = 0usize;
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
                    let (z, r) = match sel {
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
                        ),
                        Some(s) if !blind => predict_next(
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
                        ),
                        Some(s) => {
                            let mut z = local_z.clone();
                            let rel = rels_for(s, o, &table, blind);
                            let r = if s.mode == RelMode::ExactOnly && !blind {
                                read_step(&o.ring, o.cur, o.prev, o.prev2, s, &table, true)
                            } else {
                                let act = s.choose(&o.cands, &rel);
                                Read {
                                    action: act,
                                    source: act.map(|(k, _)| o.cands[k].slot_ref),
                                    payload: act.map(|(k, _)| o.cands[k].payload),
                                    payload_abs: act.map(|(k, _)| o.cands[k].abs + 1),
                                    rel: act.map(|(k, _)| rel[k]).unwrap_or(0),
                                    admitted: o.cands.len(),
                                    scanned: 0,
                                }
                            };
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
                    };
                    let covered_here = o.cands.iter().any(|c| c.payload == o.target);
                    if covered_here {
                        covered += 1;
                    }
                    if let Some((k, _)) = r.action {
                        reads += 1;
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
                    let stratum = if o.ring.written() as usize + 2 == pop[gi].tokens.len() {
                        if pop[gi].absent {
                            "final_absent"
                        } else {
                            "final_present"
                        }
                    } else {
                        "intermediate"
                    };
                    let entry = strata.entry(stratum).or_default();
                    entry.0 += 1;
                    entry.1 += usize::from(covered_here);
                    entry.2 += usize::from(r.payload == Some(o.target));
                    entry.3 += usize::from(emitted == o.target);
                    entry.4 += hb;
                    outcomes.push(json!({
                        "group": o.group, "query_abs": o.ring.written(), "stratum": stratum,
                        "target": o.target, "covered": covered_here, "candidates": o.cands.len(),
                        "selected_source": r.source.map(|s| json!({"seq": s.seq, "abs": s.abs})),
                        "payload_abs": r.payload_abs, "payload": r.payload,
                        "strength": r.action.map(|(_, a)| a), "emitted": emitted,
                        "hard_bits": hb, "local_bits": lb,
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
                "no_read_rate": 1.0 - reads as f64 / n.max(1) as f64,
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
                                let rel = rels_for(s, o, &table, blind);
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
            let rel_r = rels_for(&relational, o, &table, false);
            let rel_e = rels_for(&exact, o, &table, false);
            let mut zr = lz.clone();
            if let Some((k, st)) = relational.choose(&o.cands, &rel_r) {
                let p = o.cands[k].payload as usize;
                zr[p] = zr[p].saturating_add(1i32 << AMP_SHIFTS[st]);
            }
            let mut ze = lz.clone();
            if let Some((k, st)) = exact.choose(&o.cands, &rel_e) {
                let p = o.cands[k].payload as usize;
                ze[p] = ze[p].saturating_add(1i32 << AMP_SHIFTS[st]);
            }
            a += bits(&zr, o.target, parent.cfg.f_bits);
            b += bits(&ze, o.target, parent.cfg.f_bits);
        }
        pairs.push((a, b, os.len()));
    }
    let hard_diff = boot_diff(&pairs, 0x1234_5678);

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
                &relational,
                &table,
                &parent,
                &local,
                &u,
                true,
            );
            let rel = rels_for(&relational, o, &table, false);
            let act = relational.choose(&o.cands, &rel);
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
                &relational,
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
                &relational,
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
    // Altered source payload: record selected payload AND actually emitted output.
    let mut altered_n = 0usize;
    let mut altered_attempted = 0usize;
    let mut altered_query_context = 0usize;
    let mut altered_selected = 0usize;
    let mut altered_emitted = 0usize;
    for (gi, os) in fresh_obs.iter().enumerate() {
        for o in os.iter() {
            let Some(k) = o.cands.iter().position(|c| c.payload == o.target) else {
                continue;
            };
            let Some(alt) = banks.values_held.iter().find(|v| **v != o.target) else {
                continue;
            };
            let src = (o.cands[k].abs + 1) as usize;
            if src >= fresh[gi].tokens.len() {
                continue;
            }
            let mut t2 = fresh[gi].tokens.clone();
            t2[src] = *alt;
            altered_attempted += 1;
            altered_query_context += usize::from(src + 2 >= o.ring.written() as usize);
            let mut changed = fresh[gi].clone();
            changed.tokens = t2;
            let alt_obs = observe(&changed);
            let Some(o2) = alt_obs
                .iter()
                .find(|x| x.ring.written() == o.ring.written())
            else {
                continue;
            };
            let (z, r) = predict_next(
                &o2.ring,
                o2.cur,
                o2.prev as usize,
                o2.prev2,
                &relational,
                &table,
                &parent,
                &local,
                &u,
                true,
            );
            if let Some((kk, _)) = r.action {
                altered_n += 1;
                if o2.cands[kk].payload == *alt {
                    altered_selected += 1;
                }
                if argmax_low(&z) as u32 == *alt {
                    altered_emitted += 1;
                }
            }
        }
    }
    controls.push(json!({
        "control": "altered_source_payload",
        "changed_source_positions": altered_attempted,
        "positions_with_post_intervention_read": altered_n,
        "interventions_also_changing_recent_query_context": altered_query_context,
        "selected_the_new_payload": altered_selected,
        "emitted_the_new_payload": altered_emitted,
        "note": "mutates the first admitted occurrence carrying the target payload, which need not be the previously selected source; records all attempts separately from post-intervention reads and query-context changes; not a paired correct-source attribution test",
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

    // ---- natural-text development regression ------------------------------------
    let (uniq, _, _) = reconstruct_corpus(Path::new(DEFAULT_DOCS));
    let dev: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Dev)
        .collect();
    let mut text_positions = 0usize;
    let mut text_local = 0.0f64;
    let mut text_reader = 0.0f64;
    let mut text_reads = 0usize;
    for i in dev.iter().take(TEXT_WINDOWS) {
        let ws = windows_of(&tokenizer.encode(&uniq[*i].text));
        if ws.is_empty() {
            continue;
        }
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
                let _z = local_logits(&parent, &local, &u, cur, prev, prev2, ring.written());
                let zr = predict_next(
                    &ring,
                    cur,
                    prev,
                    prev2,
                    &relational,
                    &table,
                    &parent,
                    &local,
                    &u,
                    true,
                )
                .1;
                let _ = zr;
            }
            ring.observe(cur);
        }
        // Score every next-token position on the window through the shared path.
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
                let (zr, r) = predict_next(
                    &ring,
                    cur,
                    prev,
                    prev2,
                    &relational,
                    &table,
                    &parent,
                    &local,
                    &u,
                    true,
                );
                let (zl, _) = predict_next(
                    &ring,
                    cur,
                    prev,
                    prev2,
                    &relational,
                    &table,
                    &parent,
                    &local,
                    &u,
                    false,
                );
                text_local += bits(&zl, target, parent.cfg.f_bits);
                text_reader += bits(&zr, target, parent.cfg.f_bits);
                text_positions += 1;
                if r.action.is_some() {
                    text_reads += 1;
                }
            }
            ring.observe(cur);
        }
    }
    text_rows.push(json!({
        "panel": "natural_text_development_regression",
        "documents": dev.iter().take(TEXT_WINDOWS).count(),
        "positions": text_positions,
        "local_bits_per_token": text_local / text_positions.max(1) as f64,
        "reader_bits_per_token": text_reader / text_positions.max(1) as f64,
        "delta_bits": (text_reader - text_local) / text_positions.max(1) as f64,
        "reader_reads": text_reads,
        "scope": "pinned repository documentation, open development; already inspected, not a fresh external benchmark",
    }));
    mark("controls + text", &mut marks);

    // ---- generation through the shared path -------------------------------------
    let mut gen_rows = Vec::new();
    for (k, pr) in LEGACY_PROMPTS.iter().enumerate() {
        let mut out = Vec::new();
        for name in ["local", "relational"] {
            let use_reader = name == "relational";
            // The query token is supplied separately, exactly as in teacher-forced evaluation.
            let mut ring = ring_before_current(pr);
            let mut toks: Vec<u32> = pr.to_vec();
            let start = toks.len();
            for _ in 0..GEN_TOKENS {
                let _ = generate_step(
                    &mut ring,
                    &mut toks,
                    &relational,
                    &table,
                    &parent,
                    &local,
                    &u,
                    use_reader,
                )?;
            }
            out.push(json!({
                "arm": name, "tokens": toks[start..].to_vec(),
                "decoded": tokenizer.decode(&toks[start..]),
                "prompt_index": k,
            }));
        }
        gen_rows.push(json!({"prompt_index": k, "arms": out}));
    }

    // ---- cost: correct denominators, reader-free baseline -----------------------
    let probe_ws = windows_of(&tokenizer.encode(&uniq[dev[0]].text));
    let probe: Vec<u32> = probe_ws[0][..32].to_vec();
    let mut timings = Vec::new();
    for (label, use_reader) in [("local_no_reader_work", false), ("reader_full_path", true)] {
        let _ = run_probe(&parent, &local, &u, &relational, &table, &probe, use_reader);
        let mut s = Vec::new();
        for _ in 0..5 {
            let t0 = Instant::now();
            let out = run_probe(&parent, &local, &u, &relational, &table, &probe, use_reader);
            black_box(&out);
            s.push(t0.elapsed().as_secs_f64());
        }
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let predictions = prediction_count(probe.len());
        let (scanned, admitted) =
            run_probe_counts(&parent, &local, &u, &relational, &table, &probe, use_reader);
        timings.push(json!({
            "label": label,
            "predictions": predictions,
            "median_s": s[s.len() / 2],
            "per_prediction_us": s[s.len() / 2] / predictions as f64 * 1e6,
            "ring_records_scanned": scanned,
            "candidates_admitted": admitted,
            "protocol": "32-token real window, one discarded warm-up then five repeats, E computed per position with no token-pair cache, black_box on the result; the local arm builds no ring and performs no admission or feature work",
        }));
    }
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
    write_json(
        &root,
        "result.json",
        &json!({
            "schema": "uor-r4.competitive-reader/2",
            "base_revision": "a1fadd1f",
            "running_source": {
                "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
                "git_dirty": std::env::var("UOR_GIT_DIRTY").unwrap_or_else(|_| "unknown".into()),
                "executable_sha256": executable_sha256,
                "source_file_sha256": sources,
            },
            "inputs": {"E": E_SHA, "local_query_artifact": S_SHA, "tokenizer": {"source": TOKENIZER_SHA, "derived": derived}},
            "baseline": "z_local(v) = z_E(v) + u_S(b) for i >= 2, else z_E, through one shared function",
            "mechanism": {
                "shared_path": "read_step + predict_next; no target, sentinel or supervised record enters them",
                "descriptor_initialisation": "palette.elements[a_codes[t]]: the actual S write element, named explicitly (the old run copied slot labels as group IDs)",
                "relation": "inverse(q)*k through the single relation_index function used by fit, inference, export and reload",
                "artifact": "RLR2 carries the mode and the code map; an independent loader is exercised for every arm",
                "strengths": {"amp_shifts": AMP_SHIFTS.to_vec()},
                "ring_cap": RING_CAP, "max_candidates": MAX_CAND,
                "arm_reload_parity_failures": arm_parity_failures,
            },
            "fit": arm_report,
            "panels": panel,
            "hard_action_ce_relational_minus_exact_fresh": hard_diff,
            "natural_text": text_rows,
            "controls": controls,
            "generation": gen_rows,
            "cost": {
                "timings": timings,
                "serialized_bytes": {"parent_E": pb.len(), "local_query_artifact": sb.len(), "relational_artifact": bytes.len()},
                "resident_bytes": {"local_row_table": 120 * parent.cfg.vocab * 4, "parent_scratch": parent.cfg.vocab * 4, "descriptor_roots": relational.q_roots.len(), "rank_table": RANKS * 4},
                "energy": "UNAVAILABLE",
            },
            "decision": {
                "primary": "paired-by-sequence hard-action CE difference, relational minus exact-recurrence, fresh construction",
                "margin_bits": MARGIN_HARD_BITS,
                "text_margin_bits": MARGIN_TEXT_BITS,
                "relational_vs_exact": hard_diff,
                "relational_hard_ce": get("relational")["hard_action_ce_bits"].clone(),
                "exact_hard_ce": get("exact")["hard_action_ce_bits"].clone(),
                "local_hard_ce": get("local")["hard_action_ce_bits"].clone(),
                "query_blind_hard_ce": get("relational_query_blind")["hard_action_ce_bits"].clone(),
                "instrument_checks_pass": instrument_ok,
                "positive": positive,
                "historical": "the old 0.15 precision margin and positive=false remain historical and are not re-applied here",
            },
            "phases": marks.iter().fold((0.0f64, Vec::new()), |(prev, mut out), (n, t)| {
                out.push(json!({"phase": n, "seconds": t - prev, "cumulative_s": t}));
                (*t, out)
            }).1,
            "elapsed_s": started.elapsed().as_secs_f64(),
        }),
    )?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "fresh: relational {:.6} vs exact {:.6} vs local {:.6} bits; paired diff {:?} [{:?},{:?}]; text delta {text_delta:.6}; instrument_ok {instrument_ok}; positive {positive}; sealed {} unlisted; elapsed {:.1}s",
        get("relational")["hard_action_ce_bits"].as_f64().unwrap_or(f64::NAN),
        get("exact")["hard_action_ce_bits"].as_f64().unwrap_or(f64::NAN),
        get("local")["hard_action_ce_bits"].as_f64().unwrap_or(f64::NAN),
        hard_point,
        hard_diff["lo"].as_f64().unwrap_or(f64::NAN),
        hard_hi,
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

fn run_probe(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    sel: &RelationalSelector,
    table: &ExactGroupTable,
    probe: &[u32],
    use_reader: bool,
) -> u32 {
    let mut last = 0u32;
    if !use_reader {
        // The matched local baseline does no memory allocation, update, admission or feature work.
        for k in 2..probe.len().saturating_sub(1) {
            let z = local_logits(
                parent,
                local,
                u,
                probe[k],
                probe[k - 1] as usize,
                probe[k - 2],
                k as u32,
            );
            last = black_box(argmax_low(&z)) as u32;
        }
        return last;
    }
    let mut ring = OccurrenceRing::new(RING_CAP);
    for k in 0..probe.len() {
        let cur = probe[k];
        let prev = if k >= 1 {
            probe[k - 1] as usize
        } else {
            parent.cfg.pad_row()
        };
        let prev2 = if k >= 2 { probe[k - 2] } else { NO_TOKEN };
        if k >= 2 && k + 1 < probe.len() {
            let (z, _) = predict_next(
                &ring, cur, prev, prev2, sel, table, parent, local, u, use_reader,
            );
            last = black_box(argmax_low(&z)) as u32;
        }
        ring.observe(cur);
    }
    last
}

fn run_probe_counts(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    sel: &RelationalSelector,
    table: &ExactGroupTable,
    probe: &[u32],
    use_reader: bool,
) -> (usize, usize) {
    if !use_reader {
        return (0, 0);
    }
    let mut ring = OccurrenceRing::new(RING_CAP);
    let (mut scanned, mut admitted) = (0usize, 0usize);
    for k in 0..probe.len() {
        let cur = probe[k];
        let prev = if k >= 1 {
            probe[k - 1] as usize
        } else {
            parent.cfg.pad_row()
        };
        let prev2 = if k >= 2 { probe[k - 2] } else { NO_TOKEN };
        if k >= 2 && k + 1 < probe.len() {
            let r = read_step(&ring, cur, prev as u32, prev2, sel, table, true);
            scanned += r.scanned;
            admitted += r.admitted;
            let _ = local_logits(parent, local, u, cur, prev, prev2, ring.written());
        }
        ring.observe(cur);
    }
    (scanned, admitted)
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
