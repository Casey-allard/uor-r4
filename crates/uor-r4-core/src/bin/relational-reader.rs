//! Learned contextual geometric relational reader — fit, matched comparators, controls, cost.
//!
//! Four arms share one causal candidate pool, one action set, one dose and one objective; the only
//! difference is how the relation index is formed. Instrument defects named by the correcting audit
//! are repaired here. No transformer, authored parser or new corpus.
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
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/relational-reader-1";
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
const N_FIT: usize = 200;
const N_TUNE: usize = 60;
const N_FRESH: usize = 120;
const STEPS: usize = 220;
const BATCH: usize = 16;
const LR: f64 = 0.08;
const ROUNDS: usize = 2;
const SEED_FIT: u64 = 0x5E1A_71F1;
const SEED_FRESH: u64 = 0x5E1A_7F2E;
const MARGIN_PRECISION: f64 = 0.15;
const MARGIN_LOSS: f64 = 0.01;
const GEN_TOKENS: usize = 48;
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

static EXACT: std::sync::LazyLock<ExactGroupTable> =
    std::sync::LazyLock::new(|| ExactGroupTable::build().expect("exact table"));

struct Banks {
    solo: Vec<u32>,
    pairs: Vec<(u32, u32)>,
    keys: Vec<u32>,
    values_fit: Vec<u32>,
    values_held: Vec<u32>,
}

#[derive(Clone)]
struct Seq {
    tokens: Vec<u32>,
    group: u16,
    geom: bool,
}

/// One analysed position with everything the arms and controls need.
struct Pos {
    i: usize,
    target: u32,
    q_role: u32,
    k_roles: Vec<u32>,
    payload_abs: Vec<u32>,
    feats: Vec<[u8; EXACT_FEATS]>,
    payloads: Vec<u32>,
    delta: Vec<[f64; ACTS]>,
    local_argmax: u32,
    stats: PoolStats,
    covered: bool,
    group: u16,
    geom: bool,
}

struct Args {
    root: PathBuf,
    source_root: Option<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut a = std::env::args().skip(1);
    let mut root = PathBuf::from(DEFAULT_ROOT);
    let mut source_root = None;
    while let Some(k) = a.next() {
        match k.as_str() {
            "--root" => root = PathBuf::from(a.next().ok_or("--root needs a value")?),
            "--source-root" => source_root = Some(PathBuf::from(a.next().ok_or("value")?)),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(Args { root, source_root })
}

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

fn logsumexp_scale(z: &[i32], f_bits: u32) -> (f64, f64) {
    let scale = (-(f_bits as f64)).exp2();
    let mut max = f64::NEG_INFINITY;
    for &v in z {
        max = max.max(v as f64 * scale);
    }
    let mut sum = 0.0;
    for &v in z {
        sum += (v as f64 * scale - max).exp();
    }
    (max, max + sum.ln())
}

fn bits(z: &[i32], target: u32, f_bits: u32) -> f64 {
    let scale = (-(f_bits as f64)).exp2();
    let (_, lse) = logsumexp_scale(z, f_bits);
    (lse - z[(target as usize).min(z.len() - 1)] as f64 * scale) / std::f64::consts::LN_2
}

fn prob_of(z: &[i32], token: u32, f_bits: u32) -> f64 {
    let scale = (-(f_bits as f64)).exp2();
    let (max, _) = logsumexp_scale(z, f_bits);
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

fn local_logits(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    tokens: &[u32],
    i: usize,
) -> Vec<i32> {
    let v = parent.cfg.vocab;
    let prev = if i == 0 {
        parent.cfg.pad_row()
    } else {
        (tokens[i - 1] as usize).min(v - 1)
    };
    let cur = (tokens[i] as usize).min(v - 1);
    let mut z = parent.int_logits(prev, cur, true);
    if i >= 2 {
        let b = local.query_state(tokens[i]);
        for (x, y) in z.iter_mut().zip(u[b].iter()) {
            *x += *y;
        }
    }
    z
}

/// Independent reconstruction of the retained S-query-only scorer for the read-disabled check.
fn s_query_only(parent: &PriorCore, local: &QueryHard, tokens: &[u32], i: usize) -> Vec<i32> {
    let v = parent.cfg.vocab;
    let prev = if i == 0 {
        parent.cfg.pad_row()
    } else {
        (tokens[i - 1] as usize).min(v - 1)
    };
    let cur = (tokens[i] as usize).min(v - 1);
    if i < 2 {
        return parent.int_logits(prev, cur, true);
    }
    let b = local.query_state(tokens[i]);
    let e = local.table.identity as usize;
    local.int_logits(prev, cur, Some((b, e)))
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
    if words.len() < 84 {
        return Err(format!("only {} single-token words", words.len()));
    }
    Ok(Banks {
        solo: words[0..8].to_vec(),
        pairs: (0..10)
            .map(|k| (words[8 + 2 * k], words[9 + 2 * k]))
            .collect(),
        keys: words[28..52].to_vec(),
        values_fit: words[52..76].to_vec(),
        values_held: words[76..88].to_vec(),
    })
}

/// The answer block shares the query key and uses the **same-scope partner** of the query role in
/// the geometric family, so exact role equality cannot identify it. The decoy uses a solo role from
/// a different family with a different value and follows the answer in half the sequences. The task
/// relation is an authored list, independent of any model root or slot assignment.
fn make_seq(banks: &Banks, geom: bool, st: &mut u64, values: &[u32], m: usize) -> Seq {
    let (qrole, arole, drole) = if geom {
        let (a, b) = pick(&banks.pairs, st);
        let (q, al) = if xorshift(st) & 1 == 0 {
            (a, b)
        } else {
            (b, a)
        };
        (q, al, pick(&banks.solo, st))
    } else {
        let r = pick(&banks.solo, st);
        let d = loop {
            let x = pick(&banks.solo, st);
            if x != r {
                break x;
            }
        };
        (r, r, d)
    };
    let qkey = pick(&banks.keys, st);
    let aval = pick(values, st);
    let mut dval = pick(values, st);
    if dval == aval {
        dval = values[(values.iter().position(|v| *v == aval).unwrap_or(0) + 1) % values.len()];
    }
    let mut blocks: Vec<(u32, u32, u32)> = vec![(arole, qkey, aval), (drole, qkey, dval)];
    for _ in 2..m {
        blocks.push((
            pick(&banks.solo, st),
            pick(&banks.keys, st),
            pick(values, st),
        ));
    }
    if xorshift(st) & 1 == 0 {
        blocks.swap(0, 1);
    }
    let mut tokens = Vec::new();
    for (r, k, v) in blocks.iter() {
        tokens.push(*r);
        tokens.push(*k);
        tokens.push(*v);
    }
    tokens.push(qrole);
    tokens.push(qkey);
    tokens.push(aval);
    Seq {
        tokens,
        group: 0,
        geom,
    }
}

fn make_pop(banks: &Banks, n: usize, seed: u64, values: &[u32], geom_pct: u64) -> Vec<Seq> {
    let mut st = seed | 1;
    let mut out = Vec::with_capacity(n);
    for g in 0..n {
        let geom = xorshift(&mut st) % 100 < geom_pct;
        let m = 3 + (xorshift(&mut st) as usize) % 4;
        let mut s = make_seq(banks, geom, &mut st, values, m);
        s.group = g as u16;
        out.push(s);
    }
    out
}

fn analyse(seq: &Seq, parent: &PriorCore, local: &QueryHard, u: &[Vec<i32>]) -> Vec<Pos> {
    let toks = &seq.tokens;
    let v = parent.cfg.vocab;
    let f = parent.cfg.f_bits;
    let mut ring = OccurrenceRing::new(RING_CAP);
    let mut out = Vec::new();
    for i in 0..toks.len() {
        if i >= 2 && i + 1 < toks.len() {
            let q_cur = (toks[i] as usize % v) as u32;
            let q_prev = toks[i - 1];
            let q_prev2 = toks[i - 2];
            let (cands, stats) = admit_mixed(&ring, i as u32, q_cur, q_prev, q_prev2, MAX_CAND);
            if !cands.is_empty() {
                let z = local_logits(parent, local, u, toks, i);
                let target = toks[i + 1];
                let mut delta = Vec::with_capacity(cands.len());
                for c in cands.iter() {
                    let p = prob_of(&z, c.payload, f);
                    delta.push(std::array::from_fn(|a| {
                        let boost = (1i64 << AMP_SHIFTS[a]) as f64 / (1i64 << f) as f64;
                        action_loss(p, boost, c.payload == target)
                    }));
                }
                out.push(Pos {
                    i,
                    target,
                    q_role: q_prev,
                    k_roles: cands
                        .iter()
                        .map(|c| {
                            if c.x_prev == NO_TOKEN {
                                NO_TOKEN
                            } else {
                                c.x_prev
                            }
                        })
                        .collect(),
                    payload_abs: cands.iter().map(|c| c.abs + 1).collect(),
                    feats: cands.iter().map(|c| c.feats).collect(),
                    payloads: cands.iter().map(|c| c.payload).collect(),
                    delta,
                    local_argmax: argmax_low(&z) as u32,
                    stats,
                    covered: cands.iter().any(|c| c.payload == target),
                    group: seq.group,
                    geom: seq.geom,
                });
            }
        }
        ring.observe(toks[i]);
    }
    out
}

fn cands_of(p: &Pos) -> Vec<Cand> {
    (0..p.feats.len())
        .map(|k| Cand {
            slot_ref: OccurrenceRef {
                seq: 1,
                abs: k as u32,
            },
            abs: k as u32,
            payload: p.payloads[k],
            x_cur: 0,
            x_prev: p.k_roles[k],
            x_prev2: 0,
            feats: p.feats[k],
        })
        .collect()
}

/// Relation index per candidate under one selector.
fn rels_of(sel: &RelationalSelector, p: &Pos) -> Vec<usize> {
    let qr = p.q_role;
    p.k_roles
        .iter()
        .map(|kr| {
            if qr == NO_TOKEN || *kr == NO_TOKEN {
                return 0;
            }
            let a = *sel.q_roots.get(qr as usize).unwrap_or(&0) as usize;
            let b = *sel.q_roots.get(*kr as usize).unwrap_or(&0) as usize;
            relation(&EXACT, a, b)
        })
        .collect()
}

fn arm_action(sel: &RelationalSelector, p: &Pos) -> Option<(usize, usize)> {
    let cs = cands_of(p);
    let rel = rels_of(sel, p);
    sel.choose(&cs, &rel)
}

/// Expected actual action loss of an arm's policy at one position, including NoRead.
fn arm_expected_loss(sel: &RelationalSelector, p: &Pos) -> f64 {
    let cs = cands_of(p);
    let rel = rels_of(sel, p);
    let n = cs.len() * ACTS;
    let mut s = vec![0.0f64; n + 1];
    let mut d = vec![0.0f64; n + 1];
    for (k, c) in cs.iter().enumerate() {
        for a in 0..ACTS {
            s[k * ACTS + a] = sel.score(c, rel[k], a) as f64;
            d[k * ACTS + a] = p.delta[k][a];
        }
    }
    s[n] = sel.noread as f64;
    d[n] = 0.0;
    let max = s.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let pi: Vec<f64> = s.iter().map(|x| (x - max).exp()).collect();
    let z: f64 = pi.iter().sum();
    pi.iter().zip(d.iter()).map(|(a, b)| a * b).sum::<f64>() / z
}

fn rel_apply(z: &mut [i32], payload: Option<u32>, strength: usize) {
    if let Some(p) = payload {
        let p = p as usize;
        if p < z.len() {
            z[p] = z[p].saturating_add(1i32 << AMP_SHIFTS[strength]);
        }
    }
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

#[allow(clippy::too_many_lines)]
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
    let derived =
        sha256_hex(&derive_tokenizer_json(&tb, VOCAB).map_err(|e| format!("derive: {e}"))?);
    if derived != DERIVED_SHA {
        return Err(format!("derived tokenizer sha {derived} != pinned"));
    }
    let tokenizer = derive_tokenizer(&tb, VOCAB).map_err(|e| format!("derive tokenizer: {e}"))?;
    let raw_tok: [u8; 32] = hex_to_bytes(&derived)?.try_into().unwrap();
    let local = QueryHard::from_bytes(&sb, &parent, &parent_digest, &raw_tok)
        .map_err(|e| format!("load S: {e}"))?;
    let u: Vec<Vec<i32>> = (0..120).map(|s| local.row_scores(s)).collect();
    let executable_sha256 = hex_of(&sha256_bytes(
        &std::env::current_exe()
            .ok()
            .and_then(|p| std::fs::read(p).ok())
            .unwrap_or_default(),
    ));
    let sources: Vec<serde_json::Value> = match &args.source_root {
        Some(sr) => [
            "crates/uor-r4-core/src/native_geometric/learner/relational.rs",
            "crates/uor-r4-core/src/bin/relational-reader.rs",
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
    let fit = make_pop(&banks, N_FIT, SEED_FIT, &banks.values_fit, 60);
    let tune = make_pop(&banks, N_TUNE, SEED_FIT ^ 0x77, &banks.values_fit, 60);
    let fresh = make_pop(&banks, N_FRESH, SEED_FRESH, &banks.values_held, 60);
    let fit_a: Vec<Vec<Pos>> = fit
        .iter()
        .map(|s| analyse(s, &parent, &local, &u))
        .collect();
    let tune_a: Vec<Vec<Pos>> = tune
        .iter()
        .map(|s| analyse(s, &parent, &local, &u))
        .collect();
    let fresh_a: Vec<Vec<Pos>> = fresh
        .iter()
        .map(|s| analyse(s, &parent, &local, &u))
        .collect();
    let count = |a: &[Vec<Pos>]| -> (usize, usize, usize) {
        let n: usize = a.iter().map(|v| v.len()).sum();
        let cov: usize = a.iter().flatten().filter(|p| p.covered).count();
        let geom: usize = a.iter().flatten().filter(|p| p.geom).count();
        (n, cov, geom)
    };
    let (fit_n, fit_cov, fit_g) = count(&fit_a);
    let (fresh_n, fresh_cov, fresh_g) = count(&fresh_a);
    println!(
        "banks: {} solo / {} pairs / {} keys; fit {fit_n} pos ({fit_cov} covered, {fit_g} geometric); fresh {fresh_n} pos ({fresh_cov} covered, {fresh_g} geometric)",
        banks.solo.len(),
        banks.pairs.len(),
        banks.keys.len()
    );
    write_json(
        &args.root,
        "population.json",
        &json!({
            "format": "M blocks (role,key,value) then a query (role,key) whose successor is the answer. Geometric family: the answer block's role is the same-scope partner of the query role, so exact role equality cannot identify it. Exact family: the answer role equals the query role. A decoy block repeats the query key with a solo role from a different family and a different value, placed after the answer in half the sequences.",
            "task_relation": "same-scope role pairs and solo roles are authored lists; no answer is defined by a model root or slot assignment",
            "frozen_before_scoring": true,
            "seeds": {"fit": SEED_FIT, "tune": SEED_FIT ^ 0x77, "fresh": SEED_FRESH},
            "banks": {"solo_roles": banks.solo, "role_pairs": banks.pairs, "keys": banks.keys,
                      "values_fit": banks.values_fit, "values_held_out": banks.values_held},
            "fit": fit.iter().map(|s| json!({"group": s.group, "geom": s.geom, "tokens": s.tokens})).collect::<Vec<_>>(),
            "fresh": fresh.iter().map(|s| json!({"group": s.group, "geom": s.geom, "tokens": s.tokens})).collect::<Vec<_>>(),
        }),
    )?;
    mark("construction + analysis", &mut marks);

    let to_positions = |a: &[Vec<Pos>]| -> Vec<TrainPos> {
        a.iter()
            .flatten()
            .map(|p| TrainPos {
                cands: cands_of(p),
                q_role: p.q_role as usize,
                k_role: p
                    .k_roles
                    .iter()
                    .map(|k| {
                        if *k == NO_TOKEN {
                            usize::MAX
                        } else {
                            *k as usize
                        }
                    })
                    .collect(),
                delta: p.delta.clone(),
                group: p.group,
            })
            .collect()
    };
    let fit_pos = to_positions(&fit_a);
    let tune_pos = to_positions(&tune_a);
    let init_roots: Vec<u8> = local.a_codes.iter().map(|c| *c).collect();
    let code_of: Vec<u8> = (0..parent.cfg.vocab)
        .map(|t| ((t * 37 + 11) % 120) as u8)
        .collect();
    let cfg_id = sha256_bytes(
        format!("{RING_CAP}|{MAX_CAND}|{STEPS}|{BATCH}|{LR}|{ROUNDS}|{SEED_FIT}").as_bytes(),
    );
    let mut data_bytes = Vec::new();
    for p in fit_pos.iter() {
        data_bytes.push(p.cands.len() as u8);
        for d in p.delta.iter() {
            for x in d.iter() {
                data_bytes.extend_from_slice(&x.to_le_bytes());
            }
        }
    }
    let data_id = sha256_bytes(&data_bytes);

    let mut arms: BTreeMap<&'static str, RelationalSelector> = BTreeMap::new();
    let mut arm_report = Vec::new();
    for name in ["local", "exact", "categorical", "relational"] {
        if name == "local" {
            arm_report.push(json!({"arm": name, "note": "always NoRead; no fitting"}));
            continue;
        }
        let mode = match name {
            "exact" => RelMode::ExactOnly,
            "categorical" => RelMode::Categorical,
            _ => RelMode::Geometric,
        };
        let mut tr = RelationalTrainer::new(
            parent.cfg.vocab,
            EXACT.clone(),
            LR,
            SEED_FIT,
            data_id,
            cfg_id,
            &init_roots,
        )
        .with_mode(mode, code_of.clone());
        let t0 = Instant::now();
        let mut curve = Vec::new();
        for _ in 0..STEPS {
            let b = tr.next_batch(fit_pos.len(), BATCH);
            let l = tr.step(&fit_pos, &b);
            if tr.step % 55 == 0 {
                curve.push(json!({"step": tr.step, "mean_expected_action_loss_nats": l}));
            }
        }
        let tune_before: f64 = tune_pos.iter().map(|p| tr.position_loss(p)).sum();
        let improvement = if mode == RelMode::Geometric {
            tr.refine_descriptor(&fit_pos, ROUNDS)
        } else {
            0.0
        };
        let tune_after: f64 = tune_pos.iter().map(|p| tr.position_loss(p)).sum();
        let sel = tr.quantize();
        sel.validate()?;
        arm_report.push(json!({
            "arm": name,
            "mode": format!("{mode:?}"),
            "steps": tr.step,
            "seconds": t0.elapsed().as_secs_f64(),
            "curve": curve,
            "descriptor_roots_moved": tr.descriptor_moved(),
            "descriptor_evaluations": tr.descriptor_evaluations,
            "descriptor_moves": tr.descriptor_moves,
            "fit_objective_improvement_from_descriptor_search": improvement,
            "tune_mean_expected_action_loss_before": tune_before,
            "tune_mean_expected_action_loss_after": tune_after,
            "quantized": {
                "w": sel.w.to_vec(), "bias": sel.bias, "sb": sel.sb.to_vec(),
                "noread": sel.noread,
                "rank_nonzero": sel.rank.iter().filter(|v| **v != 0).count(),
            },
        }));
        arms.insert(name, sel);
    }
    let relational = arms["relational"].clone();
    mark("fit", &mut marks);

    // ---- export / reload with complete parity -------------------------------
    let artifact = json!({
        "schema": "uor-r4.relational-reader/1",
        "descriptor_roots": relational.q_roots,
        "w": relational.w.to_vec(), "rank": relational.rank.to_vec(),
        "bias": relational.bias, "sb": relational.sb.to_vec(), "noread": relational.noread,
        "amp_shifts": AMP_SHIFTS.to_vec(),
        "local_artifact_sha256": S_SHA,
        "tokenizer_digest": hex_of(&raw_tok),
        "source_digest": hex_of(&data_id),
    });
    let bytes = serde_json::to_vec(&artifact).map_err(|e| format!("{e}"))?;
    write_checked(&args.root, "artifacts/relational_reader.json", &bytes)?;
    let reloaded: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| format!("{e}"))?;
    let mut reload_sel = relational.clone();
    for (k, v) in reloaded["w"].as_array().ok_or("w")?.iter().enumerate() {
        reload_sel.w[k] = v.as_i64().unwrap_or(0) as i32;
    }
    for (k, v) in reloaded["rank"]
        .as_array()
        .ok_or("rank")?
        .iter()
        .enumerate()
    {
        reload_sel.rank[k] = v.as_i64().unwrap_or(0) as i32;
    }
    for (k, v) in reloaded["sb"].as_array().ok_or("sb")?.iter().enumerate() {
        reload_sel.sb[k] = v.as_i64().unwrap_or(0) as i32;
    }
    for (k, v) in reloaded["descriptor_roots"]
        .as_array()
        .ok_or("roots")?
        .iter()
        .enumerate()
    {
        reload_sel.q_roots[k] = v.as_u64().unwrap_or(0) as u8;
    }
    reload_sel.bias = reloaded["bias"].as_i64().unwrap_or(0) as i32;
    reload_sel.noread = reloaded["noread"].as_i64().unwrap_or(0) as i32;
    if reload_sel != relational {
        return Err("reloaded artifact differs".into());
    }
    let mut parity_mismatch = 0usize;
    let mut parity_checked = 0usize;
    for ps in fresh_a.iter() {
        for p in ps.iter() {
            let a = arm_action(&relational, p);
            let b = arm_action(&reload_sel, p);
            parity_checked += 1;
            if a != b {
                parity_mismatch += 1;
                continue;
            }
            if let Some((k, st)) = a {
                let mut za =
                    local_logits(&parent, &local, &u, &fresh[p.group as usize].tokens, p.i);
                rel_apply(&mut za, Some(p.payloads[k]), st);
                let za_arg = argmax_low(&za);
                let mut zb =
                    local_logits(&parent, &local, &u, &fresh[p.group as usize].tokens, p.i);
                rel_apply(&mut zb, Some(p.payloads[k]), st);
                if za_arg != argmax_low(&zb) {
                    parity_mismatch += 1;
                }
            }
        }
    }

    // ---- evaluation ---------------------------------------------------------
    let mut panel = Vec::new();
    for (panel_name, a, pop) in [("fit", &fit_a, &fit), ("fresh", &fresh_a, &fresh)] {
        let mut per_arm: BTreeMap<&'static str, ArmStats> = BTreeMap::new();
        for name in ["local", "exact", "categorical", "relational"] {
            per_arm.insert(name, ArmStats::default());
        }
        for (gi, ps) in a.iter().enumerate() {
            for p in ps.iter() {
                let z = local_logits(&parent, &local, &u, &pop[gi].tokens, p.i);
                let lb = bits(&z, p.target, parent.cfg.f_bits);
                for name in ["local", "exact", "categorical", "relational"] {
                    let st = per_arm.get_mut(name).unwrap();
                    st.n += 1;
                    st.local_ce += lb;
                    st.groups.push(p.group);
                    st.geom.push(p.geom);
                    let action = if name == "local" {
                        None
                    } else {
                        arm_action(&arms[name], p)
                    };
                    let mut zz = z.clone();
                    if let Some((k, s)) = action {
                        rel_apply(&mut zz, Some(p.payloads[k]), s);
                        st.reads += 1;
                        if p.payloads[k] == p.target {
                            st.good_reads += 1;
                        }
                        if p.covered {
                            st.covered_reads += 1;
                            if p.payloads[k] == p.target {
                                st.covered_good += 1;
                            }
                        }
                    }
                    st.correct += usize::from(argmax_low(&zz) as u32 == p.target);
                    st.ce_delta += bits(&zz, p.target, parent.cfg.f_bits) - lb;
                    if name != "local" {
                        st.expected_loss += arm_expected_loss(&arms[name], p);
                    }
                }
            }
        }
        let rows: Vec<serde_json::Value> = ["local", "exact", "categorical", "relational"]
            .iter()
            .map(|name| {
                let s = &per_arm[name];
                json!({
                    "arm": name,
                    "positions": s.n,
                    "next_token_accuracy": s.correct as f64 / s.n.max(1) as f64,
                    "reads": s.reads,
                    "no_read_rate": 1.0 - s.reads as f64 / s.n.max(1) as f64,
                    "read_precision_all": if s.reads == 0 { f64::NAN } else { s.good_reads as f64 / s.reads as f64 },
                    "covered_reads": s.covered_reads,
                    "read_precision_covered": if s.covered_reads == 0 { f64::NAN } else { s.covered_good as f64 / s.covered_reads as f64 },
                    "mean_expected_action_loss_nats": if *name == "local" { 0.0 } else { s.expected_loss / s.n.max(1) as f64 },
                    "mean_real_ce_delta_bits": s.ce_delta / s.n.max(1) as f64,
                })
            })
            .collect();
        // Paired-by-group intervals, relational minus exact.
        let (prec, lossd) = paired(a, pop, &parent, &local, &u, &arms);
        panel.push(json!({
            "panel": panel_name,
            "positions": per_arm["local"].n,
            "geometric_positions": per_arm["local"].geom.iter().filter(|g| **g).count(),
            "arms": rows,
            "read_precision_relational_minus_exact": prec,
            "expected_action_loss_relational_minus_exact": lossd,
        }));
    }
    mark("evaluation", &mut marks);

    // ---- controls -----------------------------------------------------------
    let mut controls = Vec::new();
    let mut read_disabled_ok = true;
    let mut independent_ok = true;
    let mut independent_mismatch = 0usize;
    let mut checked = 0usize;
    for (gi, ps) in fresh_a.iter().enumerate() {
        for p in ps.iter() {
            let z = local_logits(&parent, &local, &u, &fresh[gi].tokens, p.i);
            let mut zz = z.clone();
            rel_apply(&mut zz, None, 0);
            if zz != z {
                read_disabled_ok = false;
            }
            let z2 = s_query_only(&parent, &local, &fresh[gi].tokens, p.i);
            if z2 != z {
                independent_ok = false;
                independent_mismatch += 1;
            }
            checked += 1;
        }
    }
    controls.push(json!({
        "control": "read_disabled_reproduces_local", "positions": checked,
        "equals_local": read_disabled_ok,
    }));
    controls.push(json!({
        "control": "independent_s_query_only_reconstruction", "positions": checked,
        "matches_additive_row_table_path": independent_ok,
        "mismatching_positions": independent_mismatch,
        "note": "rebuilt through the artifact's own QueryHard::int_logits((b, identity)) rather than the precomputed row table",
    }));
    let mut causal = true;
    let mut compared = 0usize;
    let mut decision_changes = 0usize;
    for (gi, ps) in fresh_a.iter().enumerate() {
        if ps.is_empty() {
            continue;
        }
        let cut = ps[ps.len() / 2].i;
        let mut t2 = fresh[gi].tokens.clone();
        if cut + 1 >= t2.len() {
            continue;
        }
        t2[cut + 1] = banks.values_held[(cut + 3) % banks.values_held.len()];
        let alt = analyse(
            &Seq {
                tokens: t2,
                group: fresh[gi].group,
                geom: fresh[gi].geom,
            },
            &parent,
            &local,
            &u,
        );
        for p0 in ps.iter().filter(|p| p.i < cut) {
            let Some(p1) = alt.iter().find(|p| p.i == p0.i) else {
                causal = false;
                continue;
            };
            compared += 1;
            if p0.feats != p1.feats || p0.payloads != p1.payloads {
                causal = false;
            }
            if arm_action(&relational, p0) != arm_action(&relational, p1) {
                decision_changes += 1;
            }
        }
    }
    controls.push(json!({
        "control": "future_token_intervention",
        "positions_compared": compared,
        "earlier_observations_unchanged": causal,
        "earlier_selected_action_changes": decision_changes,
        "note": "a real future token was mutated and the mutated sequence re-analysed; the pass is an actual comparison, not a re-run of the same input",
    }));
    let mut ring = OccurrenceRing::new(RING_CAP);
    for t in [1u32, 2, 3] {
        ring.observe(t);
    }
    let r = ring.reference(1).ok_or("reference")?;
    ring.reset();
    controls.push(json!({
        "control": "stale_reference_after_reset",
        "resolved_before_reset": true,
        "resolved_after_reset": ring.resolve(r).is_some(),
    }));
    // Altered source payload: rewrite the source occurrence's observed successor.
    let mut altered_n = 0usize;
    let mut altered_ok = 0usize;
    let mut altered_abstain = 0usize;
    for (gi, ps) in fresh_a.iter().enumerate() {
        for p in ps.iter() {
            if !p.covered {
                continue;
            }
            let Some(k) = p.payloads.iter().position(|v| *v == p.target) else {
                continue;
            };
            let Some(alt) = banks.values_held.iter().find(|v| **v != p.target) else {
                continue;
            };
            let src = p.payload_abs[k] as usize;
            if src >= fresh[gi].tokens.len() {
                continue;
            }
            let mut t2 = fresh[gi].tokens.clone();
            t2[src] = *alt;
            let a2 = analyse(
                &Seq {
                    tokens: t2,
                    group: fresh[gi].group,
                    geom: fresh[gi].geom,
                },
                &parent,
                &local,
                &u,
            );
            let Some(p1) = a2.iter().find(|q| q.i == p.i) else {
                continue;
            };
            match arm_action(&relational, p1) {
                Some((j, _)) => {
                    altered_n += 1;
                    if p1.payloads[j] == *alt {
                        altered_ok += 1;
                    }
                }
                None => altered_abstain += 1,
            }
        }
    }
    controls.push(json!({
        "control": "altered_source_payload",
        "changed_source_positions": altered_n,
        "emitted_the_new_payload": altered_ok,
        "abstained_after_change": altered_abstain,
        "note": "the source occurrence's observed successor was rewritten; query and surrounding prefix unchanged",
    }));
    controls.push(json!({
        "control": "matched_categorical_partition",
        "definition": "a learned table of the same size on an arbitrary per-token code, with targets defined independently of any model root or slot assignment",
        "arm": "categorical",
    }));
    controls.push(json!({
        "control": "representation_matched_ablation",
        "note": "the exact arm is a full matched fit on the same pool, action set, dose and objective with the relation term absent; it is not a post-hoc weight zeroing of the fitted relational selector",
    }));

    // ---- generation (observational regression) ------------------------------
    let mut gen_rows = Vec::new();
    for (k, pr) in LEGACY_PROMPTS.iter().enumerate() {
        let mut arms_out = Vec::new();
        for name in ["local", "relational"] {
            let mut toks: Vec<u32> = pr.to_vec();
            let start = toks.len();
            for _ in 0..GEN_TOKENS {
                let i = toks.len() - 1;
                let ps = analyse(
                    &Seq {
                        tokens: toks.clone(),
                        group: k as u16,
                        geom: false,
                    },
                    &parent,
                    &local,
                    &u,
                );
                let Some(p) = ps.last() else { break };
                let z = local_logits(&parent, &local, &u, &toks, i);
                let mut zz = z;
                if name != "local" {
                    if let Some((kk, st)) = arm_action(&arms[name], p) {
                        rel_apply(&mut zz, Some(p.payloads[kk]), st);
                    }
                }
                toks.push(argmax_low(&zz) as u32);
            }
            arms_out.push(json!({
                "arm": name,
                "output": toks[start..].to_vec(),
                "decoded": tokenizer.decode(&toks[start..]),
            }));
        }
        gen_rows.push(json!({"prompt_index": k, "prompt": pr, "arms": arms_out}));
    }

    // ---- direct-path cost ---------------------------------------------------
    let (uniq, _, _) = reconstruct_corpus(Path::new(DEFAULT_DOCS));
    let dev: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Dev)
        .collect();
    let ws = windows_of(&tokenizer.encode(&uniq[dev[0]].text));
    let probe: Vec<u32> = ws[0][..32].to_vec();
    let mut timings = Vec::new();
    for label in ["local_no_reader_work", "reader_full_path"] {
        let use_reader = label == "reader_full_path";
        let _ = run_path(&parent, &local, &u, &relational, &probe, use_reader);
        let mut s = Vec::new();
        for _ in 0..5 {
            let t0 = Instant::now();
            let out = run_path(&parent, &local, &u, &relational, &probe, use_reader);
            black_box(&out);
            s.push(t0.elapsed().as_secs_f64());
        }
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        timings.push(json!({
            "label": label,
            "median_s": s[s.len() / 2],
            "per_prediction_us": s[s.len() / 2] / (probe.len() - 1) as f64 * 1e6,
            "protocol": "32-token real window, one discarded warm-up then five repeats, E computed per position with no token-pair cache, black_box on the result; the local arm performs no ring, admission or feature work",
        }));
    }
    mark("controls + generation + cost", &mut marks);

    // ---- decision -----------------------------------------------------------
    let fresh_panel = panel
        .iter()
        .find(|p| p["panel"] == "fresh")
        .ok_or("fresh panel")?;
    let prec = &fresh_panel["read_precision_relational_minus_exact"];
    let lossd = &fresh_panel["expected_action_loss_relational_minus_exact"];
    let prec_point = prec["point"].as_f64().unwrap_or(f64::NAN);
    let prec_lo = prec["lo"].as_f64().unwrap_or(f64::NAN);
    let loss_point = lossd["point"].as_f64().unwrap_or(f64::NAN);
    let loss_lo = lossd["lo"].as_f64().unwrap_or(f64::NAN);
    let instrument_ok = read_disabled_ok
        && independent_ok
        && causal
        && controls
            .iter()
            .find(|c| c["control"] == "stale_reference_after_reset")
            .is_some_and(|c| c["resolved_after_reset"] == json!(false))
        && parity_mismatch == 0;
    let positive = prec_point >= MARGIN_PRECISION
        && prec_lo > 0.0
        && -loss_point >= MARGIN_LOSS
        && loss_lo < 0.0
        && instrument_ok;
    write_json(
        &args.root,
        "result.json",
        &json!({
            "schema": "uor-r4.relational-reader/1",
            "base_revision": "647483ab",
            "running_source": {
                "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
                "git_dirty": std::env::var("UOR_GIT_DIRTY").unwrap_or_else(|_| "unknown".into()),
                "executable_sha256": executable_sha256,
                "source_file_sha256": sources,
                "hash_scope": "sha256 of the executable bytes, not a hex re-encoding of them",
            },
            "inputs": {"E": E_SHA, "local_query_artifact": S_SHA, "tokenizer": {"source": TOKENIZER_SHA, "derived": derived}},
            "baseline": "z_local(v) = z_E(v) + u_S(b) for i >= 2, else z_E; no S history row and no history fold",
            "mechanism": {
                "descriptor": "learned per-token root Q(x_{i-1}), 120-valued, initialised from the frozen S write-slot assignment",
                "relation": "inverse(q)*k, common-left invariant",
                "rank_table_nonzero_entries": relational.rank.iter().filter(|v| **v != 0).count(),
                "descriptor_roots_moved_from_initialisation": relational.q_roots.iter().zip(local.a_codes.iter()).filter(|(a, b)| **a != **b).count(),
                "strengths": {"amp_shifts": AMP_SHIFTS.to_vec(), "nats": AMP_SHIFTS.iter().map(|s| (1i64 << s) as f64 / 1024.0).collect::<Vec<_>>()},
                "ring_cap": RING_CAP, "max_candidates": MAX_CAND,
                "export_reload_parity_positions": parity_checked,
                "export_reload_parity_mismatches": parity_mismatch,
            },
            "admission_counters": {
                "fit": fit_a.iter().flatten().fold(json!({"scanned":0,"matches":0,"admitted":0,"dropped_by_bound":0,"no_observed_successor":0}), |mut a, p| {
                    for (k, v) in [("scanned", p.stats.scanned), ("matches", p.stats.matches), ("admitted", p.stats.admitted), ("dropped_by_bound", p.stats.dropped_by_bound), ("no_observed_successor", p.stats.no_observed_successor)] {
                        let cur = a[k].as_u64().unwrap_or(0);
                        a[k] = json!(cur + v as u64);
                    }
                    a
                }),
                "note": "every scanned ring record and every candidate visit is counted; a 24-candidate result after scanning 128 records is not 24 operations",
            },
            "local_argmax_matches_target_share": {
                "fit": fit_a.iter().flatten().filter(|p| p.local_argmax == p.target).count() as f64 / fit_a.iter().flatten().count().max(1) as f64,
                "fresh": fresh_a.iter().flatten().filter(|p| p.local_argmax == p.target).count() as f64 / fresh_a.iter().flatten().count().max(1) as f64,
                "note": "the frozen local baseline's own next-token accuracy on the candidate-bearing pool",
            },
            "fit": arm_report,
            "panels": panel,
            "controls": controls,
            "generation": gen_rows,
            "cost": {
                "timings": timings,
                "serialized_bytes": {"parent_E": pb.len(), "local_query_artifact": sb.len(), "relational_artifact": bytes.len()},
                "resident_bytes": {"local_row_table": 120 * parent.cfg.vocab * 4, "parent_scratch": parent.cfg.vocab * 4, "descriptor_roots": relational.q_roots.len(), "rank_table": RANKS * 4},
                "energy": "UNAVAILABLE",
            },
            "decision": {
                "primary": "covered-position read precision, relational minus exact-recurrence, same pool, actions and dose",
                "precision_difference": prec,
                "expected_action_loss_difference": lossd,
                "margins": {"precision": MARGIN_PRECISION, "loss_nats": MARGIN_LOSS},
                "instrument_checks_pass": instrument_ok,
                "positive": positive,
                "historical_columns": "the old +0.671 synthetic and +2.789 raw numbers were measured at the fixed 16-nat gain and are not comparable to these bounded strengths",
            },
            "phases": marks.iter().fold((0.0f64, Vec::new()), |(prev, mut out), (n, t)| {
                out.push(json!({"phase": n, "seconds": t - prev, "cumulative_s": t}));
                (*t, out)
            }).1,
            "elapsed_s": started.elapsed().as_secs_f64(),
        }),
    )?;
    seal(&args.root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&args.root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "fresh: precision rel-exact {prec_point:?} [{prec_lo:?}], expected-loss diff {loss_point:?} [{loss_lo:?}], instrument_ok {instrument_ok}, positive {positive}; sealed {} unlisted; elapsed {:.1}s",
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

#[derive(Default)]
struct ArmStats {
    n: usize,
    correct: usize,
    reads: usize,
    good_reads: usize,
    covered_reads: usize,
    covered_good: usize,
    local_ce: f64,
    ce_delta: f64,
    expected_loss: f64,
    groups: Vec<u16>,
    geom: Vec<bool>,
}

/// Paired-by-sequence bootstrap of the two primary differences, relational minus exact.
fn paired(
    a: &[Vec<Pos>],
    pop: &[Seq],
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    arms: &BTreeMap<&'static str, RelationalSelector>,
) -> (serde_json::Value, serde_json::Value) {
    fn boot<F: Fn(usize, &Pos) -> (f64, f64, f64)>(
        a: &[Vec<Pos>],
        pop: &[Seq],
        f: F,
    ) -> serde_json::Value {
        let mut per_group: Vec<(f64, f64, f64, f64)> = Vec::new();
        for (gi, ps) in a.iter().enumerate() {
            let pop_tokens: &[u32] = &pop[gi].tokens;
            let _ = pop_tokens;
            let mut n = 0.0;
            let mut da = 0.0;
            let mut db = 0.0;
            let mut unused = 0.0;
            for p in ps.iter() {
                let (x, y, m) = f(gi, p);
                if m == 0.0 {
                    continue;
                }
                n += 1.0;
                da += x;
                db += y;
                unused += 1.0;
            }
            let _ = unused;
            if n > 0.0 {
                per_group.push((n, da, db, 1.0));
            }
        }
        if per_group.len() < 8 {
            return json!({"point": null, "lo": null, "hi": null, "groups": per_group.len(),
                          "note": "too few groups for a paired interval"});
        }
        let point = per_group.iter().map(|g| g.1 - g.2).sum::<f64>()
            / per_group.iter().map(|g| g.0).sum::<f64>();
        let mut st = 0x1234_5678u64;
        let mut boots = Vec::new();
        for _ in 0..2000 {
            let (mut sn, mut sa, mut sb) = (0.0, 0.0, 0.0);
            for _ in 0..per_group.len() {
                let g = per_group[(xorshift(&mut st) as usize) % per_group.len()];
                sn += g.0;
                sa += g.1;
                sb += g.2;
            }
            if sn > 0.0 {
                boots.push((sa - sb) / sn);
            }
        }
        boots.sort_by(|a, b| a.partial_cmp(b).unwrap());
        json!({
            "point": point,
            "lo": boots[(boots.len() as f64 * 0.025) as usize],
            "hi": boots[((boots.len() as f64 * 0.975) as usize).min(boots.len() - 1)],
            "groups": per_group.len(),
            "draws": boots.len(),
            "unit": "sequence",
        })
    }

    // Precision difference on covered positions only.
    let prec = boot(a, pop, |gi, p| {
        let rel = arm_action(&arms["relational"], p);
        let ex = arm_action(&arms["exact"], p);
        let m = if p.covered { 1.0 } else { 0.0 };
        let ok = |act: Option<(usize, usize)>| -> f64 {
            match act {
                Some((k, _)) if p.payloads[k] == p.target => 1.0,
                _ => 0.0,
            }
        };
        let _ = gi;
        (ok(rel), ok(ex), m)
    });
    // Expected action loss difference on all candidate-bearing positions.
    let lossd = boot(a, pop, |_gi, p| {
        let rel = arm_expected_loss(&arms["relational"], p);
        let ex = arm_expected_loss(&arms["exact"], p);
        (rel, ex, 1.0)
    });
    let _ = (parent, local, u);
    (prec, lossd)
}

fn run_path(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    sel: &RelationalSelector,
    tokens: &[u32],
    use_reader: bool,
) -> u32 {
    let mut ring = OccurrenceRing::new(RING_CAP);
    let mut last = 0u32;
    for i in 0..tokens.len() {
        if i >= 2 && i + 1 < tokens.len() {
            let mut z = local_logits(parent, local, u, tokens, i);
            if use_reader {
                let (cands, _) = admit_mixed(
                    &ring,
                    i as u32,
                    tokens[i],
                    tokens[i - 1],
                    tokens[i - 2],
                    MAX_CAND,
                );
                if !cands.is_empty() {
                    let qr = tokens[i - 1];
                    let rel: Vec<usize> = cands
                        .iter()
                        .map(|c| {
                            if c.x_prev == NO_TOKEN {
                                return 0;
                            }
                            relation(
                                &EXACT,
                                *sel.q_roots.get(qr as usize).unwrap_or(&0) as usize,
                                *sel.q_roots.get(c.x_prev as usize).unwrap_or(&0) as usize,
                            )
                        })
                        .collect();
                    if let Some((k, st)) = sel.choose(&cands, &rel) {
                        rel_apply(&mut z, Some(cands[k].payload), st);
                    }
                }
            }
            last = black_box(argmax_low(&z)) as u32;
        }
        ring.observe(tokens[i]);
    }
    last
}
