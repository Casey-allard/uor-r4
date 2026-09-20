//! Learned geometric selection of exact token occurrences — fit, controls, probe and cost.
//!
//! Integrates one bounded exact-occurrence reader with the frozen local baseline
//! `z_local = z_E + u_S(b)`. Fits only the ten-scalar selector, exports a versioned artifact,
//! reproduces the declared controls, probes bounded raw text and measures the **uncached** served
//! path. No output projection, width/precision search or reset campaign is reachable here.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Instant;

use serde_json::json;

use uor_r4_core::native_geometric::learner::occurrence::*;
use uor_r4_core::native_geometric::learner::prefix_artifact::parent_hash_convention;
use uor_r4_core::native_geometric::learner::prefix_state::palette;
use uor_r4_core::native_geometric::learner::prior_learning::PriorCore;
use uor_r4_core::native_geometric::learner::query_read::QueryHard;
use uor_r4_core::native_geometric::learner::realtext_support::*;
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::bpe_derive::{derive_tokenizer, derive_tokenizer_json};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const DEFAULT_ROOT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/occurrence-reader-1";
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

const SEED_FIT: u64 = 0x5A17_F17E;
const SEED_TUNE: u64 = 0x5A17_7A6E;
const SEED_FRESH: u64 = 0x5A17_F2E5;
const N_FIT: usize = 420;
const N_TUNE: usize = 120;
const N_FRESH: usize = 240;
const BATCH: usize = 32;
const STEPS: usize = 400;
const LR: f64 = 0.05;
/// Predeclared practical margin for the primary geometric-family endpoint, in absolute accuracy.
const PRIMARY_MARGIN: f64 = 0.20;
/// Tolerated all-position local regression, in bits per token.
const CE_TOL_SYNTH: f64 = 0.01;
const CE_TOL_RAW: f64 = 0.02;
const RAW_PROBE_WINDOWS: usize = 8;
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fam {
    /// The correct occurrence's role token equals the query's role token (exact identity suffices).
    Exact,
    /// The correct occurrence's role token is *different* but shares its transported write slot;
    /// only the geometric feature can identify it. The decoy's role is in another slot.
    Geom,
}

impl Fam {
    fn label(self) -> &'static str {
        match self {
            Fam::Exact => "exact_role",
            Fam::Geom => "geom_slot_only",
        }
    }
}

struct Banks {
    roles: Vec<u32>,
    keys: Vec<u32>,
    values_fit: Vec<u32>,
    values_held: Vec<u32>,
    /// Role pairs `(query_role, source_role)` that are distinct tokens with equal write slots.
    geom_pairs: Vec<(u32, u32)>,
}

struct Seq {
    tokens: Vec<u32>,
    group: u16,
    fam: Fam,
    /// Position whose next token is the answer (the query key position).
    query: usize,
}

struct Args {
    root: PathBuf,
    source_root: Option<PathBuf>,
    mode: String,
    a_ckpt: Option<PathBuf>,
    b_out: Option<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut a = std::env::args().skip(1);
    let mut root = PathBuf::from(DEFAULT_ROOT);
    let mut source_root = None;
    let mut mode = "full".to_string();
    let mut a_ckpt = None;
    let mut b_out = None;
    while let Some(k) = a.next() {
        let mut val = |n: &str| -> Result<String, String> {
            a.next().ok_or_else(|| format!("{n} needs a value"))
        };
        match k.as_str() {
            "--root" => root = PathBuf::from(val("--root")?),
            "--source-root" => source_root = Some(PathBuf::from(val("--source-root")?)),
            "--mode" => mode = val("--mode")?,
            "--ckpt" => a_ckpt = Some(PathBuf::from(val("--ckpt")?)),
            "--out" => b_out = Some(PathBuf::from(val("--out")?)),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(Args {
        root,
        source_root,
        mode,
        a_ckpt,
        b_out,
    })
}

fn hex_to_bytes(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("odd hex length".into());
    }
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|e| e.to_string()))
        .collect()
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

fn write_f64s(root: &Path, name: &str, xs: &[f64]) -> Result<(), String> {
    let b: Vec<u8> = xs.iter().flat_map(|x| x.to_le_bytes()).collect();
    write_checked(root, name, &b)
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

fn xorshift(st: &mut u64) -> u64 {
    *st ^= *st << 13;
    *st ^= *st >> 7;
    *st ^= *st << 17;
    *st
}

fn pick<T: Copy>(v: &[T], st: &mut u64) -> T {
    v[(xorshift(st) as usize) % v.len()]
}

// ---------------------------------------------------------------------------
// Word inventory over the real BPE path
// ---------------------------------------------------------------------------

/// Single-token lowercase words from the real vocabulary, so every synthetic role/key/value is a
/// genuine BPE token id that round-trips through the tokenizer.
fn build_banks(tok: &HfBpeTokenizer, wm: &WriteMap<'_>, vocab: usize) -> Result<Banks, String> {
    let mut words: Vec<u32> = Vec::new();
    for id in 0..vocab as u32 {
        let s = tok.decode(&[id]);
        if s.len() < 3 || s.len() > 7 || !s.bytes().all(|b| b.is_ascii_lowercase()) {
            continue;
        }
        if tok.encode(&s) == vec![id] {
            words.push(id);
        }
        if words.len() >= 240 {
            break;
        }
    }
    if words.len() < 60 {
        return Err(format!("only {} single-token words found", words.len()));
    }
    let roles: Vec<u32> = words[0..12].to_vec();
    let keys: Vec<u32> = words[12..36].to_vec();
    let values_fit: Vec<u32> = words[36..64].to_vec();
    let values_held: Vec<u32> = words[64..76].to_vec();
    // Distinct role tokens that share a transported write slot: only these can carry the
    // geometry-only family.
    let mut geom_pairs = Vec::new();
    for &q in roles.iter() {
        for &s in roles.iter() {
            if q != s && wm.slot(q) == wm.slot(s) {
                geom_pairs.push((q, s));
            }
        }
    }
    if geom_pairs.len() < 4 {
        return Err(format!("only {} slot-sharing role pairs", geom_pairs.len()));
    }
    Ok(Banks {
        roles,
        keys,
        values_fit,
        values_held,
        geom_pairs,
    })
}

// ---------------------------------------------------------------------------
// Synthetic construction population
// ---------------------------------------------------------------------------

/// Build one sequence.
///
/// Layout: `M` assignment blocks `(role, key, value)` then a query `(qrole, qkey)` whose successor is
/// the answer. In `Exact` the answer block uses `qrole`; in `Geom` it uses a *different* role token
/// with the same write slot. A decoy block repeats the query key with another role and another value;
/// the decoy is placed *after* the answer block in half the sequences so recency alone cannot solve
/// the panel.
fn make_seq(banks: &Banks, fam: Fam, st: &mut u64, values: &[u32], m: usize) -> Seq {
    let (qrole, arole) = if fam == Fam::Geom {
        let (q, a) = pick(&banks.geom_pairs, st);
        (q, a)
    } else {
        let r = pick(&banks.roles, st);
        (r, r)
    };
    let qkey = pick(&banks.keys, st);
    let aval = pick(values, st);
    // A decoy role must differ from the answer role and, for the geometric family, must not share
    // the query's slot.
    let decoy_role = loop {
        let r = pick(&banks.roles, st);
        if r == arole {
            continue;
        }
        if fam == Fam::Geom {
            // The decoy must be *rejected* geometrically as well.
            break r;
        }
        if r == qrole {
            continue;
        }
        break r;
    };
    let mut dval = pick(values, st);
    if dval == aval {
        dval = values[(values.iter().position(|v| *v == aval).unwrap_or(0) + 1) % values.len()];
    }

    let mut blocks: Vec<(u32, u32, u32)> = Vec::new();
    blocks.push((arole, qkey, aval));
    blocks.push((decoy_role, qkey, dval));
    for _ in 2..m {
        let r = pick(&banks.roles, st);
        let k = pick(&banks.keys, st);
        let v = pick(values, st);
        blocks.push((r, k, v));
    }
    // Order the two special blocks so that recency is wrong half the time.
    let decoy_later = xorshift(st) & 1 == 0;
    if decoy_later {
        blocks.swap(0, 1);
    }
    let mut tokens: Vec<u32> = Vec::new();
    for (r, k, v) in blocks.iter() {
        tokens.push(*r);
        tokens.push(*k);
        tokens.push(*v);
    }
    let query = tokens.len() + 1;
    tokens.push(qrole);
    tokens.push(qkey);
    // The answer is the next token; the sequence ends with it so the query has a target.
    tokens.push(aval);
    let _ = decoy_later;
    Seq {
        tokens,
        group: 0,
        fam,
        query,
    }
}

fn make_population(
    banks: &Banks,
    n: usize,
    seed: u64,
    values: &[u32],
    geom_share: u64,
) -> Vec<Seq> {
    let mut st = seed | 1;
    let mut out = Vec::with_capacity(n);
    for g in 0..n {
        let fam = if xorshift(&mut st) % 100 < geom_share {
            Fam::Geom
        } else {
            Fam::Exact
        };
        let m = 3 + (xorshift(&mut st) as usize) % 5;
        let mut s = make_seq(banks, fam, &mut st, values, m);
        s.group = g as u16;
        out.push(s);
    }
    out
}

// ---------------------------------------------------------------------------
// Position analysis
// ---------------------------------------------------------------------------

struct Pos {
    group: u16,
    fam: Fam,
    i: usize,
    target: u32,
    local_argmax: u32,
    feats: Vec<[u8; FEATS]>,
    payloads: Vec<u32>,
    /// Absolute position of each candidate's observed successor (the payload token).
    payload_pos: Vec<u32>,
    stats: AdmitStats,
    /// Some admitted candidate's payload is the target.
    covered: bool,
    /// The local baseline already predicts the target.
    local_ok: bool,
    /// The target is not the local argmax but some admitted candidate covers it.
    relevant: bool,
}

/// Causal analysis of one sequence: the ring is built strictly from observed tokens, and the
/// prediction at `i` is recorded before `x_i` is observed.
fn analyze(
    seq: &Seq,
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    wm: &WriteMap<'_>,
) -> Vec<Pos> {
    let toks = &seq.tokens;
    let mut ring = OccurrenceRing::new(RING_CAP);
    let mut out = Vec::new();
    let v = parent.cfg.vocab;
    for i in 0..toks.len() {
        let ctx = QueryContext {
            i: i as u32,
            cur: (toks[i] as usize % v) as u32,
            prev: if i >= 1 { Some(toks[i - 1]) } else { None },
            prev2: if i >= 2 { Some(toks[i - 2]) } else { None },
        };
        if i >= 2 && i + 1 < toks.len() {
            let (cands, stats) = admit(&ring, &ctx, wm, MAX_CANDIDATES);
            let z = local_logits(parent, local, u, toks, i);
            let target = toks[i + 1];
            let la = argmax_low(&z) as u32;
            let covered = cands.iter().any(|c| c.payload == target);
            out.push(Pos {
                group: seq.group,
                fam: seq.fam,
                i,
                target,
                local_argmax: la,
                feats: cands.iter().map(|c| c.feats).collect(),
                payloads: cands.iter().map(|c| c.payload).collect(),
                payload_pos: cands.iter().map(|c| c.abs + 1).collect(),
                stats,
                covered,
                local_ok: la == target,
                relevant: la != target && covered,
            });
        }
        ring.observe(toks[i]);
    }
    out
}

enum Choice {
    Read(usize),
    NoRead,
}

fn choose_selector(s: &Selector, p: &Pos) -> Choice {
    let mut best: Option<(usize, i32)> = None;
    for (k, f) in p.feats.iter().enumerate() {
        let sc = s.score(f);
        if best.is_none_or(|(_, pr)| sc > pr) {
            best = Some((k, sc));
        }
    }
    match best {
        Some((k, sc)) if sc > s.mu => Choice::Read(k),
        _ => Choice::NoRead,
    }
}

fn predict(p: &Pos, z: &[i32], c: &Choice, amp_shift: u32) -> u32 {
    let payload = match c {
        Choice::NoRead => None,
        Choice::Read(k) => Some(p.payloads[*k]),
    };
    let mut zz = z.to_vec();
    apply_residual(&mut zz, payload, amp_shift);
    argmax_low(&zz) as u32
}

// ---------------------------------------------------------------------------

fn main() -> ExitCode {
    match run() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return Ok(ExitCode::from(2));
        }
    };
    let started = Instant::now();
    let mut marks: Vec<(&'static str, f64)> = Vec::new();
    let mark = |n: &'static str, s: &Instant, m: &mut Vec<(&'static str, f64)>| {
        m.push((n, s.elapsed().as_secs_f64()));
    };

    // ---- pinned inputs -------------------------------------------------------
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
    let tsha = sha256_hex(&tb);
    if tsha != TOKENIZER_SHA {
        return Err(format!("tokenizer sha {tsha} != pinned"));
    }
    let derived =
        sha256_hex(&derive_tokenizer_json(&tb, VOCAB).map_err(|e| format!("derive json: {e}"))?);
    if derived != DERIVED_SHA {
        return Err(format!("derived tokenizer sha {derived} != pinned"));
    }
    let tokenizer = derive_tokenizer(&tb, VOCAB).map_err(|e| format!("derive tokenizer: {e}"))?;
    let raw_tok: [u8; 32] = hex_to_bytes(&derived)?.try_into().unwrap();
    let local = QueryHard::from_bytes(&sb, &parent, &parent_digest, &raw_tok)
        .map_err(|e| format!("load S: {e}"))?;
    let u: Vec<Vec<i32>> = (0..120).map(|s| local.row_scores(s)).collect();
    let pal = palette().clone();
    let wm = WriteMap {
        a_codes: &local.a_codes,
        elements: &pal.elements,
    };
    mark("pinned inputs", &started, &mut marks);

    // ---- continuation sub-modes (real cross-process file resume) -------------
    match args.mode.as_str() {
        "continuation-a" => {
            let ckpt = args.a_ckpt.ok_or("--ckpt required")?;
            let (mut tr, examples, _) = build_trainer(&parent, &local, &u, &wm, &tokenizer)?;
            for _ in 0..3 {
                let b = tr.next_batch(examples.len(), BATCH);
                tr.step(&examples, &b);
            }
            std::fs::write(&ckpt, tr.checkpoint_bytes()).map_err(|e| format!("{e}"))?;
            println!(
                "continuation-a wrote {} after {} steps",
                ckpt.display(),
                tr.step
            );
            return Ok(ExitCode::SUCCESS);
        }
        "continuation-b" => {
            let ckpt = args.a_ckpt.ok_or("--ckpt required")?;
            let out = args.b_out.ok_or("--out required")?;
            let (mut tr, examples, _) = build_trainer(&parent, &local, &u, &wm, &tokenizer)?;
            let bytes = std::fs::read(&ckpt).map_err(|e| format!("{e}"))?;
            tr.resume_from(&bytes)?;
            for _ in 0..3 {
                let b = tr.next_batch(examples.len(), BATCH);
                tr.step(&examples, &b);
            }
            std::fs::write(
                &out,
                serde_json::to_vec(&json!({
                    "step": tr.step, "w": tr.w.to_vec(), "mu": tr.mu,
                    "selector": {"w": tr.quantize().w.to_vec(), "mu": tr.quantize().mu},
                    "checkpoint": hex_of(&tr.checkpoint_bytes()),
                }))
                .map_err(|e| format!("{e}"))?,
            )
            .map_err(|e| format!("{e}"))?;
            println!("continuation-b done at step {}", tr.step);
            return Ok(ExitCode::SUCCESS);
        }
        "continuation-full" => {
            let out = args.b_out.ok_or("--out required")?;
            let (mut tr, examples, _) = build_trainer(&parent, &local, &u, &wm, &tokenizer)?;
            for _ in 0..6 {
                let b = tr.next_batch(examples.len(), BATCH);
                tr.step(&examples, &b);
            }
            std::fs::write(
                &out,
                serde_json::to_vec(&json!({
                    "step": tr.step, "w": tr.w.to_vec(), "mu": tr.mu,
                    "selector": {"w": tr.quantize().w.to_vec(), "mu": tr.quantize().mu},
                    "checkpoint": hex_of(&tr.checkpoint_bytes()),
                }))
                .map_err(|e| format!("{e}"))?,
            )
            .map_err(|e| format!("{e}"))?;
            println!("continuation-full done at step {}", tr.step);
            return Ok(ExitCode::SUCCESS);
        }
        "continuation-check" => return continuation_check(&args, &started),
        _ => {}
    }

    claim(&args.root).map_err(|e| format!("claim {}: {e}", args.root.display()))?;
    let executable_sha256 = hex_of(
        &std::env::current_exe()
            .ok()
            .and_then(|p| std::fs::read(p).ok())
            .unwrap_or_default(),
    );
    let source_hashes: Vec<serde_json::Value> = match &args.source_root {
        Some(sr) => [
            "crates/uor-r4-core/src/native_geometric/learner/occurrence.rs",
            "crates/uor-r4-core/src/bin/occurrence-reader.rs",
        ]
        .iter()
        .map(|rel| match std::fs::read(sr.join(rel)) {
            Ok(b) => json!({"path": rel, "sha256": sha256_hex(&b)}),
            Err(_) => json!({"path": rel, "sha256": "UNAVAILABLE"}),
        })
        .collect(),
        None => vec![json!({"path": "unspecified", "sha256": "UNAVAILABLE"})],
    };

    // ---- banks and populations ----------------------------------------------
    let banks = build_banks(&tokenizer, &wm, parent.cfg.vocab)?;
    let fit = make_population(&banks, N_FIT, SEED_FIT, &banks.values_fit, 40);
    let tune = make_population(&banks, N_TUNE, SEED_TUNE, &banks.values_fit, 40);
    let fresh = make_population(&banks, N_FRESH, SEED_FRESH, &banks.values_held, 40);
    let pop_json = |v: &[Seq]| -> Vec<serde_json::Value> {
        v.iter()
            .map(|s| json!({"group": s.group, "family": s.fam.label(), "tokens": s.tokens, "query": s.query}))
            .collect()
    };
    write_json(
        &args.root,
        "population.json",
        &json!({
            "format": "M assignment blocks (role,key,value) then a query (role,key) whose successor is the answer; the answer block's role equals the query role in the exact family and is a different token with the same transported write slot in the geometric family; a decoy block repeats the query key with another role and another value and is placed after the answer in half the sequences",
            "frozen_before_scoring": true,
            "seeds": {"fit": SEED_FIT, "tune": SEED_TUNE, "fresh": SEED_FRESH},
            "banks": {
                "roles": banks.roles, "keys": banks.keys,
                "values_fit": banks.values_fit, "values_held_out": banks.values_held,
                "geom_role_pairs": banks.geom_pairs,
                "note": "held-out value ids are never used as payloads during selector fitting; they appear only in the fresh panel",
            },
            "fit": pop_json(&fit), "tune": pop_json(&tune), "fresh": pop_json(&fresh),
        }),
    )?;
    mark("banks + populations", &started, &mut marks);
    println!(
        "banks: {} roles / {} keys / {} fit values / {} held-out values / {} slot-sharing role pairs",
        banks.roles.len(),
        banks.keys.len(),
        banks.values_fit.len(),
        banks.values_held.len(),
        banks.geom_pairs.len()
    );

    // ---- admission coverage before fitting ----------------------------------
    let mut coverage = BTreeMap::<&'static str, AdmitStats>::new();
    let mut totals = BTreeMap::<&'static str, (usize, usize, usize, usize, usize)>::new();
    for (name, pop) in [("fit", &fit), ("tune", &tune), ("fresh", &fresh)] {
        let mut total = 0usize;
        let mut admitted_positions = 0usize;
        let mut covered = 0usize;
        let mut relevant = 0usize;
        let mut stats = AdmitStats::default();
        for s in pop.iter() {
            for p in analyze(s, &parent, &local, &u, &wm) {
                total += 1;
                if !p.feats.is_empty() {
                    admitted_positions += 1;
                }
                if p.covered {
                    covered += 1;
                }
                if p.relevant {
                    relevant += 1;
                }
                stats.scanned += p.stats.scanned;
                stats.matching += p.stats.matching;
                stats.admitted += p.stats.admitted;
                stats.dropped_by_bound += p.stats.dropped_by_bound;
                stats.no_observed_successor += p.stats.no_observed_successor;
            }
        }
        coverage.insert(name, stats);
        totals.insert(
            name,
            (total, admitted_positions, covered, relevant, pop.len()),
        );
    }
    println!(
        "admission coverage (positions / with candidates / covered / reader-relevant / sequences): {totals:?}"
    );
    println!("admission counters: {coverage:?}");

    // ---- fit ----------------------------------------------------------------
    let (mut trainer, examples, ex_identity) = build_trainer(&parent, &local, &u, &wm, &tokenizer)?;
    let mut curve = Vec::new();
    let probe_b = trainer.next_batch(examples.len(), BATCH);
    let probe_loss = trainer.step(&examples, &probe_b);
    curve.push(json!({"step": trainer.step, "fit_ce_nats": probe_loss}));
    let t0 = Instant::now();
    for _ in 1..STEPS {
        let b = trainer.next_batch(examples.len(), BATCH);
        let l = trainer.step(&examples, &b);
        if trainer.step % 50 == 0 {
            curve.push(json!({"step": trainer.step, "fit_ce_nats": l}));
        }
    }
    let fit_seconds = t0.elapsed().as_secs_f64();
    let mut selector = trainer.quantize();
    selector.validate()?;
    mark("fit", &started, &mut marks);
    println!(
        "fit: {} examples, {} steps in {:.1}s; float w {:?} mu {:.3}; quantized {:?} mu {}",
        examples.len(),
        trainer.step,
        fit_seconds,
        trainer.w.map(|x| (x * 1000.0).round() / 1000.0),
        trainer.mu,
        selector.w,
        selector.mu
    );

    // ---- amplitude from the fit/tune margin distribution --------------------
    let mut margins: Vec<i64> = Vec::new();
    for s in fit.iter().chain(tune.iter()) {
        for p in analyze(s, &parent, &local, &u, &wm) {
            if p.covered {
                let z = local_logits(&parent, &local, &u, &s.tokens, p.i);
                let best = z.iter().cloned().max().unwrap_or(0);
                margins.push(best as i64 - z[p.target as usize] as i64);
            }
        }
    }
    margins.sort_unstable();
    let q = |f: f64| -> i64 {
        if margins.is_empty() {
            0
        } else {
            margins[((margins.len() as f64 * f) as usize).min(margins.len() - 1)]
        }
    };
    let amp_shift = {
        let mut sh = 1u32;
        while (1i64 << sh) < q(0.9).max(1) && sh < 20 {
            sh += 1;
        }
        sh
    };
    let covered_fit = margins.len();
    let winners = margins.iter().filter(|m| **m < (1i64 << amp_shift)).count();
    println!(
        "margins: n={covered_fit} p50={} p90={} p99={}; amp_shift={amp_shift} (A={}); correct source can win at {:.3} of covered fit/tune positions",
        q(0.5),
        q(0.9),
        q(0.99),
        1i64 << amp_shift,
        winners as f64 / covered_fit.max(1) as f64
    );
    write_json(
        &args.root,
        "amplitude.json",
        &json!({
            "definition": "m = max_v z_local(v) - z_local(target) at covered fit/tune positions",
            "count": covered_fit, "p50": q(0.5), "p90": q(0.9), "p99": q(0.99), "max": margins.last(),
            "amp_shift": amp_shift, "amplitude": 1i64 << amp_shift,
            "fraction_where_correct_source_can_win": winners as f64 / covered_fit.max(1) as f64,
        }),
    )?;

    // Declared post-quantization threshold calibration on the **tune** split only: the surrogate
    // objective is not the hard endpoint, and rounding places the threshold on a score boundary.
    // The weights are untouched; only the integer NoRead threshold `mu` is chosen.
    let tune_rows = decision_rows(&selector, &tune, &parent, &local, &u, &wm, amp_shift);
    let mut mu_sweep = Vec::new();
    let mut best_mu = selector.mu;
    let mut best_acc = f64::NEG_INFINITY;
    for mu in -WEIGHT_MAX..=WEIGHT_MAX {
        let acc = sweep_accuracy(&tune_rows, mu);
        mu_sweep.push(json!({"mu": mu, "hard_accuracy": acc}));
        if acc > best_acc {
            best_acc = acc;
            best_mu = mu;
        }
    }
    selector.mu = best_mu;
    selector.validate()?;
    // ---- export, reload, verify --------------------------------------------
    let artifact = OccurrenceArtifact {
        selector,
        amp_shift,
        ring_cap: RING_CAP as u32,
        max_candidates: MAX_CANDIDATES as u32,
        local_artifact_digest: sha256_32(&sb),
        tokenizer_digest: raw_tok,
        source_digest: ex_identity,
    };
    let bytes = artifact.to_bytes();
    write_checked(&args.root, "artifacts/occurrence_reader.ocq1", &bytes)?;
    let reloaded = OccurrenceArtifact::from_bytes(&bytes, &sha256_32(&sb), &raw_tok)?;
    if reloaded.selector != selector || reloaded.amp_shift != amp_shift {
        return Err("reloaded artifact differs".into());
    }
    // The exported selector must reproduce the in-memory one at every decision.
    let mut decision_mismatches = 0usize;
    for s in fit.iter().take(64).chain(fresh.iter().take(64)) {
        for p in analyze(s, &parent, &local, &u, &wm) {
            let a = matches!(choose_selector(&selector, &p), Choice::Read(_));
            let b = matches!(choose_selector(&reloaded.selector, &p), Choice::Read(_));
            if a != b {
                decision_mismatches += 1;
            }
        }
    }
    if decision_mismatches != 0 {
        return Err(format!(
            "export/reload decisions differ at {decision_mismatches} positions"
        ));
    }

    // ---- synthetic evaluation ----------------------------------------------
    let mut panel_json = Vec::new();
    for (name, pop) in [("fit", &fit), ("tune", &tune), ("fresh", &fresh)] {
        // Recompute without leaking: iterate again with owned positions.
        let mut rows: Vec<(Fam, Pos, Vec<i32>)> = Vec::new();
        for s in pop.iter() {
            for p in analyze(s, &parent, &local, &u, &wm) {
                let z = local_logits(&parent, &local, &u, &s.tokens, p.i);
                rows.push((s.fam, p, z));
            }
        }
        let mut conds: BTreeMap<&'static str, (Vec<f64>, Vec<f64>, Vec<u32>, Vec<u32>)> =
            BTreeMap::new();
        // (losses, correct flags, predictions, targets) per condition
        let add = |m: &mut BTreeMap<&'static str, (Vec<f64>, Vec<f64>, Vec<u32>, Vec<u32>)>,
                   k: &'static str,
                   l: f64,
                   ok: f64,
                   pred: u32,
                   tgt: u32| {
            let e = m.entry(k).or_default();
            e.0.push(l);
            e.1.push(ok);
            e.2.push(pred);
            e.3.push(tgt);
        };
        for (fam, p, z) in rows.iter() {
            let l = bits(z, p.target, parent.cfg.f_bits);
            add(
                &mut conds,
                "local",
                l,
                f64::from(p.local_ok),
                p.local_argmax,
                p.target,
            );
            let cs = choose_selector(&selector, p);
            add(
                &mut conds,
                "reader",
                bits(
                    &{
                        let mut zz = z.clone();
                        apply_residual(
                            &mut zz,
                            match cs {
                                Choice::Read(k) => Some(p.payloads[k]),
                                Choice::NoRead => None,
                            },
                            amp_shift,
                        );
                        zz
                    },
                    p.target,
                    parent.cfg.f_bits,
                ),
                f64::from(predict(p, z, &cs, amp_shift) == p.target),
                predict(p, z, &cs, amp_shift),
                p.target,
            );
            // Fixed latest-occurrence control.
            let lc = match Selector::latest(&cands_of(p)) {
                Some(k) => Choice::Read(k),
                None => Choice::NoRead,
            };
            add(
                &mut conds,
                "latest_occurrence",
                bits(
                    &{
                        let mut zz = z.clone();
                        apply_residual(
                            &mut zz,
                            match lc {
                                Choice::Read(k) => Some(p.payloads[k]),
                                Choice::NoRead => None,
                            },
                            amp_shift,
                        );
                        zz
                    },
                    p.target,
                    parent.cfg.f_bits,
                ),
                f64::from(predict(p, z, &lc, amp_shift) == p.target),
                predict(p, z, &lc, amp_shift),
                p.target,
            );
            // Fixed exact-role-match control.
            let ec = match Selector::exact_role(&cands_of(p)) {
                Some(k) => Choice::Read(k),
                None => Choice::NoRead,
            };
            add(
                &mut conds,
                "exact_role_only",
                bits(
                    &{
                        let mut zz = z.clone();
                        apply_residual(
                            &mut zz,
                            match ec {
                                Choice::Read(k) => Some(p.payloads[k]),
                                Choice::NoRead => None,
                            },
                            amp_shift,
                        );
                        zz
                    },
                    p.target,
                    parent.cfg.f_bits,
                ),
                f64::from(predict(p, z, &ec, amp_shift) == p.target),
                predict(p, z, &ec, amp_shift),
                p.target,
            );
            // Geometry-disabled: zero the geometric and collision weights.
            let mut gd = selector;
            gd.w[2] = 0;
            gd.w[3] = 0;
            gd.w[7] = 0;
            gd.w[8] = 0;
            let gc = choose_selector(&gd, p);
            add(
                &mut conds,
                "geometry_disabled",
                bits(
                    &{
                        let mut zz = z.clone();
                        apply_residual(
                            &mut zz,
                            match gc {
                                Choice::Read(k) => Some(p.payloads[k]),
                                Choice::NoRead => None,
                            },
                            amp_shift,
                        );
                        zz
                    },
                    p.target,
                    parent.cfg.f_bits,
                ),
                f64::from(predict(p, z, &gc, amp_shift) == p.target),
                predict(p, z, &gc, amp_shift),
                p.target,
            );
            let _ = fam;
        }
        let mut cjson = Vec::new();
        for (k, (l, ok, _p, _t)) in conds.iter() {
            let micro: f64 = l.iter().sum::<f64>() / l.len() as f64;
            let acc: f64 = ok.iter().sum::<f64>() / ok.len() as f64;
            cjson.push(json!({"condition": k, "micro_ce_bits": micro, "accuracy": acc}));
            write_f64s(&args.root, &format!("vectors/{name}-{k}.f64"), l)?;
        }
        // Family split for accuracy.
        let mut fam_rows = Vec::new();
        for fam in [Fam::Exact, Fam::Geom] {
            let idx: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter(|(_, (f, _, _))| *f == fam)
                .map(|(k, _)| k)
                .collect();
            if idx.is_empty() {
                continue;
            }
            let acc = |k: &str| -> f64 {
                let v = &conds[k].1;
                idx.iter().map(|i| v[*i]).sum::<f64>() / idx.len() as f64
            };
            let ce = |k: &str| -> f64 {
                let v = &conds[k].0;
                idx.iter().map(|i| v[*i]).sum::<f64>() / idx.len() as f64
            };
            fam_rows.push(json!({
                "family": fam.label(), "positions": idx.len(),
                "accuracy": {
                    "local": acc("local"), "reader": acc("reader"),
                    "latest_occurrence": acc("latest_occurrence"),
                    "exact_role_only": acc("exact_role_only"),
                    "geometry_disabled": acc("geometry_disabled"),
                },
                "micro_ce_bits": {
                    "local": ce("local"), "reader": ce("reader"),
                    "exact_role_only": ce("exact_role_only"),
                    "geometry_disabled": ce("geometry_disabled"),
                },
            }));
        }
        // Reader-relevant positions, and the primary geometric-family endpoint.
        let relevant: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, (_, p, _))| p.relevant)
            .map(|(k, _)| k)
            .collect();
        let geom_relevant: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, (f, p, _))| *f == Fam::Geom && p.relevant)
            .map(|(k, _)| k)
            .collect();
        let acc_on = |idx: &[usize], k: &str| -> f64 {
            if idx.is_empty() {
                f64::NAN
            } else {
                let v = &conds[k].1;
                idx.iter().map(|i| v[*i]).sum::<f64>() / idx.len() as f64
            }
        };
        // Paired-by-sequence bootstrap of the accuracy difference.
        let mut by_group: BTreeMap<u16, (usize, usize, usize, usize)> = BTreeMap::new();
        for (k, (f, p, _)) in rows.iter().enumerate() {
            if *f != Fam::Geom || !p.relevant {
                continue;
            }
            let e = by_group.entry(p.group).or_default();
            e.0 += 1; // n
            e.1 += if conds["reader"].1[k] > 0.5 { 1 } else { 0 };
            e.2 += if conds["local"].1[k] > 0.5 { 1 } else { 0 };
            e.3 += if conds["exact_role_only"].1[k] > 0.5 {
                1
            } else {
                0
            };
        }
        let mut st = 0x1234_5678u64;
        let mut diffs = Vec::new();
        let groups: Vec<(usize, usize, usize, usize)> = by_group.values().copied().collect();
        if groups.len() >= 8 {
            for _ in 0..2000 {
                let (mut n, mut r, mut l, mut _e) = (0usize, 0usize, 0usize, 0usize);
                for _ in 0..groups.len() {
                    let g = groups[(xorshift(&mut st) as usize) % groups.len()];
                    n += g.0;
                    r += g.1;
                    l += g.2;
                    _e += g.3;
                }
                if n > 0 {
                    diffs.push((r as f64 - l as f64) / n as f64);
                }
            }
            diffs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }
        let dci = if diffs.len() >= 100 {
            json!({"point": acc_on(&geom_relevant, "reader") - acc_on(&geom_relevant, "local"),
                   "lo": diffs[(diffs.len() as f64 * 0.025) as usize],
                   "hi": diffs[((diffs.len() as f64 * 0.975) as usize).min(diffs.len()-1)],
                   "draws": diffs.len(), "unit": "sequence"})
        } else {
            json!({"point": acc_on(&geom_relevant, "reader") - acc_on(&geom_relevant, "local"),
                   "lo": null, "hi": null, "draws": 0, "unit": "sequence",
                   "note": "insufficient geometric-family sequences for a paired interval"})
        };
        let ce_local: f64 = conds["local"].0.iter().sum::<f64>() / conds["local"].0.len() as f64;
        let ce_reader: f64 = conds["reader"].0.iter().sum::<f64>() / conds["reader"].0.len() as f64;
        panel_json.push(json!({
            "panel": name,
            "positions": rows.len(),
            "conditions": cjson,
            "families": fam_rows,
            "reader_relevant_positions": relevant.len(),
            "geom_family_relevant_positions": geom_relevant.len(),
            "accuracy_reader_relevant": {
                "local": acc_on(&relevant, "local"),
                "reader": acc_on(&relevant, "reader"),
                "latest_occurrence": acc_on(&relevant, "latest_occurrence"),
                "exact_role_only": acc_on(&relevant, "exact_role_only"),
                "geometry_disabled": acc_on(&relevant, "geometry_disabled"),
            },
            "accuracy_geom_family_relevant": {
                "local": acc_on(&geom_relevant, "local"),
                "reader": acc_on(&geom_relevant, "reader"),
                "latest_occurrence": acc_on(&geom_relevant, "latest_occurrence"),
                "exact_role_only": acc_on(&geom_relevant, "exact_role_only"),
                "geometry_disabled": acc_on(&geom_relevant, "geometry_disabled"),
            },
            "primary_reader_minus_local_accuracy": dci,
            "micro_ce": {"local": ce_local, "reader": ce_reader, "delta": ce_reader - ce_local},
        }));
    }
    mark("synthetic evaluation", &started, &mut marks);

    // ---- controls -----------------------------------------------------------
    let mut controls = Vec::new();
    {
        // ReadDisabled must reproduce the local predictor exactly.
        let mut same = true;
        let mut checked = 0usize;
        for s in fresh.iter().take(64) {
            for p in analyze(s, &parent, &local, &u, &wm) {
                let z = local_logits(&parent, &local, &u, &s.tokens, p.i);
                let mut zr = z.clone();
                apply_residual(&mut zr, None, amp_shift);
                if zr != z {
                    same = false;
                }
                checked += 1;
            }
        }
        controls
            .push(json!({"control": "read_disabled", "positions": checked, "equals_local": same}));
        // Causality: changing or appending future tokens cannot change an earlier prediction.
        let mut causal = true;
        for s in fresh.iter().take(64) {
            let base = analyze(s, &parent, &local, &u, &wm);
            for cut in [3usize, 5] {
                if s.tokens.len() <= cut + 2 {
                    continue;
                }
                let mut t2 = s.tokens[..cut].to_vec();
                t2.extend_from_slice(&s.tokens[cut..]);
                let alt = analyze(s, &parent, &local, &u, &wm);
                for (a, b) in base.iter().zip(alt.iter()) {
                    if a.i < cut && (a.feats != b.feats || a.payloads != b.payloads) {
                        causal = false;
                    }
                }
            }
        }
        controls.push(json!({"control": "future_token_causality", "invariant": causal}));
        // Stale references: after reset every earlier reference must be rejected.
        let mut ring = OccurrenceRing::new(RING_CAP);
        for t in [1u32, 2, 3] {
            ring.observe(t);
        }
        let r = ring.reference(1).unwrap();
        ring.reset();
        controls.push(json!({
            "control": "stale_reference_after_reset",
            "resolved_before_reset": true,
            "resolved_after_reset": ring.resolve(r).is_some(),
        }));
        // Altered source payload: rewriting the *source* occurrence's successor must change the
        // emitted payload, with the local pair and the rest of the prefix unchanged.
        let mut altered_ok = 0usize;
        let mut altered_n = 0usize;
        let mut altered_skipped = 0usize;
        for s in fresh.iter().take(96) {
            let ps = analyze(s, &parent, &local, &u, &wm);
            let Some(p0) = ps.into_iter().find(|p| p.fam == Fam::Geom && p.relevant) else {
                continue;
            };
            // The correct source is the candidate whose payload is the target.
            let Some(k) = p0.payloads.iter().position(|v| *v == p0.target) else {
                continue;
            };
            let src_pos = p0.payload_pos[k] as usize;
            let alt = match banks.values_held.iter().find(|v| **v != p0.target) {
                Some(v) => *v,
                None => continue,
            };
            let mut t2 = s.tokens.clone();
            t2[src_pos] = alt;
            let q = analyze(
                &Seq {
                    tokens: t2,
                    group: s.group,
                    fam: s.fam,
                    query: s.query,
                },
                &parent,
                &local,
                &u,
                &wm,
            );
            let Some(p1) = q.iter().find(|p| p.i == p0.i) else {
                continue;
            };
            match choose_selector(&selector, p1) {
                Choice::Read(j) => {
                    altered_n += 1;
                    if p1.payloads[j] == alt {
                        altered_ok += 1;
                    }
                }
                Choice::NoRead => altered_skipped += 1,
            }
        }
        controls.push(json!({
            "control": "altered_source_payload",
            "changed_source_positions": altered_n,
            "emitted_the_new_payload": altered_ok,
            "abstained_after_change": altered_skipped,
            "note": "the source occurrence's successor was rewritten; the query and the surrounding prefix are unchanged",
        }));
    }
    // ---- bounded raw-text probe --------------------------------------------
    let (uniq, _, _) = reconstruct_corpus(Path::new(DEFAULT_DOCS));
    let dev: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Dev)
        .collect();
    let mut raw_rows: Vec<(u16, Pos, Vec<i32>)> = Vec::new();
    let mut raw_windows = 0usize;
    for (k, i) in dev.iter().enumerate() {
        if raw_windows >= RAW_PROBE_WINDOWS {
            break;
        }
        let ws = windows_of(&tokenizer.encode(&uniq[*i].text));
        if ws.is_empty() {
            continue;
        }
        let w = &ws[0];
        let seq = Seq {
            tokens: w.clone(),
            group: k as u16,
            fam: Fam::Exact,
            query: 0,
        };
        for p in analyze(&seq, &parent, &local, &u, &wm) {
            let z = local_logits(&parent, &local, &u, &w, p.i);
            raw_rows.push((k as u16, p, z));
        }
        raw_windows += 1;
    }
    let mut raw_cond: BTreeMap<&'static str, (f64, usize)> = BTreeMap::new();
    let mut admitted = 0usize;
    let mut read_count = 0usize;
    let mut false_copy = 0usize;
    let mut covered = 0usize;
    let mut rank_hit = 0usize;
    for (_g, p, z) in raw_rows.iter() {
        if !p.feats.is_empty() {
            admitted += 1;
        }
        if p.covered {
            covered += 1;
        }
        let cs = choose_selector(&selector, p);
        let l = bits(z, p.target, parent.cfg.f_bits);
        let mut zz = z.clone();
        let pay = match cs {
            Choice::Read(k) => Some(p.payloads[k]),
            Choice::NoRead => None,
        };
        apply_residual(&mut zz, pay, amp_shift);
        let rl = bits(&zz, p.target, parent.cfg.f_bits);
        if let Choice::Read(k) = cs {
            read_count += 1;
            if p.payloads[k] != p.target {
                false_copy += 1;
            }
        } else if p.covered {
            // The reader abstained although a covering candidate existed.
        }
        if p.covered && matches!(cs, Choice::Read(_)) {
            rank_hit += 1;
        }
        let e = raw_cond.entry("local").or_default();
        e.0 += l;
        e.1 += 1;
        let e = raw_cond.entry("reader").or_default();
        e.0 += rl;
        e.1 += 1;
    }
    let raw_local = raw_cond["local"].0 / raw_cond["local"].1 as f64;
    let raw_reader = raw_cond["reader"].0 / raw_cond["reader"].1 as f64;
    let raw_positions = raw_rows.len();
    // Actual raw-text continuations from the reader and the local baseline.
    let mut raw_gen = Vec::new();
    if let Some(i) = dev.first() {
        let ws = windows_of(&tokenizer.encode(&uniq[*i].text));
        if let Some(w) = ws.first() {
            let mut toks = w[..(w.len().min(24))].to_vec();
            let mut base = toks.clone();
            let start = toks.len();
            for _ in 0..GEN_TOKENS {
                let s = Seq {
                    tokens: toks.clone(),
                    group: 0,
                    fam: Fam::Exact,
                    query: 0,
                };
                let ps = analyze(&s, &parent, &local, &u, &wm);
                let Some(p) = ps.last() else { break };
                let z = local_logits(&parent, &local, &u, &toks, p.i);
                let cs = choose_selector(&selector, p);
                let nt = predict(p, &z, &cs, amp_shift);
                toks.push(nt);
            }
            for _ in 0..GEN_TOKENS {
                let s = Seq {
                    tokens: base.clone(),
                    group: 0,
                    fam: Fam::Exact,
                    query: 0,
                };
                let ps = analyze(&s, &parent, &local, &u, &wm);
                let Some(p) = ps.last() else { break };
                let z = local_logits(&parent, &local, &u, &base, p.i);
                let nt = predict(p, &z, &Choice::NoRead, amp_shift);
                base.push(nt);
            }
            raw_gen.push(json!({
                "prompt_tokens": w[..start].to_vec(),
                "reader_output": toks[start..].to_vec(),
                "reader_decoded": tokenizer.decode(&toks[start..]),
                "local_output": base[start..].to_vec(),
                "local_decoded": tokenizer.decode(&base[start..]),
            }));
        }
    }
    mark("raw-text probe", &started, &mut marks);
    println!(
        "raw probe: {raw_windows} windows / {raw_positions} positions; admitted {admitted}; covered {covered}; read {read_count}; false copy {false_copy}; local {raw_local:.6} -> reader {raw_reader:.6} bits/token"
    );

    // ---- observational generation regression --------------------------------
    let mut gen_rows = Vec::new();
    for (k, pr) in LEGACY_PROMPTS.iter().enumerate() {
        let mut toks: Vec<u32> = pr.to_vec();
        let start = toks.len();
        for _ in 0..GEN_TOKENS {
            let s = Seq {
                tokens: toks.clone(),
                group: k as u16,
                fam: Fam::Exact,
                query: 0,
            };
            let ps = analyze(&s, &parent, &local, &u, &wm);
            let Some(p) = ps.last() else { break };
            let z = local_logits(&parent, &local, &u, &toks, p.i);
            let cs = choose_selector(&selector, p);
            toks.push(predict(p, &z, &cs, amp_shift));
        }
        let (pair, ring) = cycle_certificates(pr, &toks[start..]);
        gen_rows.push(json!({
            "prompt_index": k,
            "prompt": pr,
            "output": toks[start..].to_vec(),
            "decoded": tokenizer.decode(&toks[start..]),
            "pair_cycle": pair, "ring_cycle": ring,
        }));
    }

    // ---- direct serving cost (uncached) -------------------------------------
    let probe_seq: Vec<u32> = {
        let ws = windows_of(&tokenizer.encode(&uniq[dev[0]].text));
        ws[0][..32].to_vec()
    };
    let mut timings = Vec::new();
    for label in ["local_uncached", "reader_uncached"] {
        let _ = run_uncached(
            &parent,
            &local,
            &u,
            &wm,
            &selector,
            &probe_seq,
            amp_shift,
            label == "reader_uncached",
        );
        let mut samples = Vec::new();
        for _ in 0..5 {
            let t0 = Instant::now();
            let out = run_uncached(
                &parent,
                &local,
                &u,
                &wm,
                &selector,
                &probe_seq,
                amp_shift,
                label == "reader_uncached",
            );
            black_box(&out);
            samples.push(t0.elapsed().as_secs_f64());
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        timings.push(json!({
            "label": label,
            "positions": probe_seq.len(),
            "samples_s": samples,
            "median_s": samples[samples.len()/2],
            "per_position_us": samples[samples.len()/2] / probe_seq.len() as f64 * 1e6,
            "protocol": "32-token real window, one discarded warm-up then five timed repeats, E computed per position with no token-pair cache, ring and selection included, black_box on the result",
        }));
    }
    mark("cost", &started, &mut marks);

    // ---- decision -----------------------------------------------------------
    let fresh_json = panel_json
        .iter()
        .find(|p| p["panel"] == "fresh")
        .ok_or("fresh panel")?;
    let gacc = &fresh_json["accuracy_geom_family_relevant"];
    let reader_acc = gacc["reader"].as_f64().unwrap_or(f64::NAN);
    let local_acc = gacc["local"].as_f64().unwrap_or(f64::NAN);
    let latest_acc = gacc["latest_occurrence"].as_f64().unwrap_or(f64::NAN);
    let geom_dis_acc = gacc["geometry_disabled"].as_f64().unwrap_or(f64::NAN);
    let primary = fresh_json["primary_reader_minus_local_accuracy"].clone();
    let primary_lo = primary["lo"].as_f64().unwrap_or(f64::NAN);
    let ce_tol_ok = fresh_json["micro_ce"]["delta"].as_f64().unwrap_or(f64::NAN) <= CE_TOL_SYNTH;
    let raw_tol_ok = raw_reader - raw_local <= CE_TOL_RAW;
    let primary_ok = reader_acc - local_acc >= PRIMARY_MARGIN && primary_lo > 0.0;
    let beats_latest = reader_acc > latest_acc;
    let geometry_helps = reader_acc - geom_dis_acc >= PRIMARY_MARGIN;
    let instrument_ok = controls
        .iter()
        .all(|c| c["equals_local"] != json!(false) && c["invariant"] != json!(false))
        && controls
            .iter()
            .find(|c| c["control"] == "stale_reference_after_reset")
            .is_none_or(|c| c["resolved_after_reset"] == json!(false));

    write_json(
        &args.root,
        "result.json",
        &json!({
            "schema": "uor-r4.occurrence-reader/1",
            "base_revision": "1dae322ce7770673c0046597c10f4361564cd3a0",
            "running_source": {
                "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
                "git_dirty": std::env::var("UOR_GIT_DIRTY").unwrap_or_else(|_| "unknown".into()),
                "executable_sha256": executable_sha256,
                "source_file_sha256": source_hashes,
            },
            "inputs": {
                "E": {"path": E_PATH, "sha256": E_SHA},
                "S_local_query_artifact": {"path": S_PATH, "sha256": S_SHA},
                "tokenizer": {"source_sha256": tsha, "derived_sha256": derived, "raw_digest": hex_of(&raw_tok)},
            },
            "baseline_definition": "z_local(v) = z_E(v) + u_S(b) for i >= 2, else z_E; no S history row; no history fold executed",
            "mechanism": {
                "ring_cap": RING_CAP, "max_candidates": MAX_CANDIDATES, "features": FEATS,
                "weight_bound": WEIGHT_MAX,
                "amplitude": {"amp_shift": amp_shift, "A": 1i64 << amp_shift},
                "quantized_selector": {"w": selector.w.to_vec(), "mu": selector.mu},
                "mu_calibration": {
                    "split": "tune only",
                    "uncalibrated_mu": trainer.quantize().mu,
                    "chosen_mu": selector.mu,
                    "sweep": mu_sweep,
                    "declared": "weights are not changed; only the integer NoRead threshold is chosen, on tune, because the softmax surrogate is not the hard endpoint and rounding lands on a score boundary",
                },
                "float_selector": {"w": trainer.w.to_vec(), "mu": trainer.mu},
                "export_reload_decision_mismatches": decision_mismatches,
            },
            "admission_coverage": coverage.iter().map(|(k, v)| json!({
                "panel": k,
                "ring_entries_scanned": v.scanned,
                "query_token_matches": v.matching,
                "admitted": v.admitted,
                "dropped_by_candidate_bound": v.dropped_by_bound,
                "matches_without_observed_successor": v.no_observed_successor,
            })).collect::<Vec<_>>(),
            "admission_totals": totals.iter().map(|(k, v)| json!({
                "panel": k, "sequences": v.4, "positions": v.0,
                "with_candidates": v.1, "covered": v.2, "reader_relevant": v.3,
            })).collect::<Vec<_>>(),
            "fit": {"examples": examples.len(), "steps": trainer.step, "seconds": fit_seconds, "curve": curve},
            "panels": panel_json,
            "controls": controls,
            "raw_text_probe": {
                "windows": raw_windows, "positions": raw_positions,
                "admitted_positions": admitted, "covered_positions": covered,
                "read_positions": read_count, "false_copies": false_copy,
                "covering_and_read": rank_hit,
                "micro_ce_bits": {"local": raw_local, "reader": raw_reader, "delta": raw_reader - raw_local},
                "continuations": raw_gen,
            },
            "generation": gen_rows,
            "cost": {
                "timings": timings,
                "serialized_bytes": {
                    "parent_E": pb.len(), "local_query_artifact": sb.len(),
                    "occurrence_artifact": bytes.len(),
                    "total": pb.len() + sb.len() + bytes.len(),
                },
                "resident_bytes": {
                    "parent_logits_scratch": parent.cfg.vocab * 4,
                    "local_row_table": 120 * parent.cfg.vocab * 4,
                    "ring": RING_CAP * 4,
                    "selector": 10 * 4,
                    "note": "steady-state inference only; no token-pair logit cache is used in these timings",
                },
                "energy": "UNAVAILABLE",
            },
            "decision": {
                "primary_endpoint": "reader hard accuracy minus local accuracy on reader-relevant positions of the geometry-only family, fresh panel",
                "primary": primary,
                "primary_margin": PRIMARY_MARGIN,
                "primary_pass": primary_ok,
                "reader_beats_latest_occurrence": beats_latest,
                "geometry_helps_over_disabled": geometry_helps,
                "synthetic_ce_tolerance_pass": ce_tol_ok,
                "raw_ce_tolerance_pass": raw_tol_ok,
                "instrument_checks_pass": instrument_ok,
                "classification": if primary_ok && beats_latest && geometry_helps
                    && ce_tol_ok && raw_tol_ok && instrument_ok
                {
                    "bounded positive: the learned reader recovers exact occurrences, the geometric slot feature carries accuracy the exact-identity control cannot, and neither the synthetic nor the raw-text all-position loss regresses beyond its declared tolerance"
                } else if primary_ok && beats_latest && geometry_helps && instrument_ok {
                    "targeted selection works and geometry contributes, but abstention is uncalibrated: the all-position loss regresses beyond the declared tolerance on at least one panel"
                } else if beats_latest && instrument_ok {
                    "exact-memory integration works; a geometric ranking advantage is not established at this scale"
                } else {
                    "negative or inconclusive at this design; see the per-family rows"
                },
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
        "fresh geom-family relevant: local {local_acc:.3} reader {reader_acc:.3} latest {latest_acc:.3} geom-disabled {geom_dis_acc:.3}; primary {primary:?}; sealed {} unlisted; elapsed {:.1}s",
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    let _ = ex_identity;
    Ok(ExitCode::SUCCESS)
}

fn cands_of(p: &Pos) -> Vec<Candidate> {
    (0..p.feats.len())
        .map(|k| Candidate {
            slot_ref: OccurrenceRef {
                seq: 1,
                abs: k as u32,
            },
            abs: k as u32,
            payload: p.payloads[k],
            feats: p.feats[k],
        })
        .collect()
}

/// Uncached E computation plus ring, selection and emission — the declared served path.
fn run_uncached(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    wm: &WriteMap<'_>,
    selector: &Selector,
    tokens: &[u32],
    amp_shift: u32,
    use_reader: bool,
) -> u32 {
    let mut ring = OccurrenceRing::new(RING_CAP);
    let mut last = 0u32;
    for i in 0..tokens.len() {
        if i >= 2 && i + 1 < tokens.len() {
            let ctx = QueryContext {
                i: i as u32,
                cur: tokens[i],
                prev: Some(tokens[i - 1]),
                prev2: Some(tokens[i - 2]),
            };
            let (cands, _) = admit(&ring, &ctx, wm, MAX_CANDIDATES);
            let mut z = local_logits(parent, local, u, tokens, i);
            if use_reader {
                let mut best: Option<(usize, i32)> = None;
                for (k, c) in cands.iter().enumerate() {
                    let s = selector.score(&c.feats);
                    if best.is_none_or(|(_, pr)| s > pr) {
                        best = Some((k, s));
                    }
                }
                if let Some((k, s)) = best {
                    if s > selector.mu {
                        apply_residual(&mut z, Some(cands[k].payload), amp_shift);
                    }
                }
            }
            last = black_box(argmax_low(&z)) as u32;
        }
        ring.observe(tokens[i]);
    }
    last
}

/// Build the trainer, its example set and the example-set identity.
///
/// Deterministic in the pinned seeds and populations only, so a resumed process reconstructs the
/// identical example set from the same pinned inputs. The example order is reconstructed from
/// `(rng, epoch)`, which is the declared cache reconstruction.
fn build_trainer(
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    wm: &WriteMap<'_>,
    tokenizer: &HfBpeTokenizer,
) -> Result<(SelectorTrainer, Vec<Example>, [u8; 32]), String> {
    let banks = build_banks(tokenizer, wm, parent.cfg.vocab)?;
    let fit = make_population(&banks, N_FIT, SEED_FIT, &banks.values_fit, 40);
    let mut examples: Vec<Example> = Vec::new();
    let mut canonical = Vec::new();
    let mut seen_non_covered = 0usize;
    for s in fit.iter() {
        for p in analyze(s, parent, local, u, wm) {
            if p.feats.is_empty() {
                continue;
            }
            // Declared class balance: every covered position is kept, and every second
            // non-covered position is kept, so the NoRead class does not swamp the read class.
            if !p.covered {
                seen_non_covered += 1;
                if seen_non_covered % 2 == 1 {
                    continue;
                }
            }
            let label = p
                .payloads
                .iter()
                .position(|v| *v == p.target)
                .unwrap_or(p.feats.len());
            canonical.extend_from_slice(&(p.feats.len() as u8).to_le_bytes());
            for f in p.feats.iter() {
                canonical.extend_from_slice(f);
            }
            canonical.extend_from_slice(&(label as u8).to_le_bytes());
            canonical.extend_from_slice(&s.group.to_le_bytes());
            examples.push(Example {
                feats: p.feats.clone(),
                label,
                group: s.group,
            });
        }
    }
    if examples.is_empty() {
        return Err("no training examples were admitted".into());
    }
    let data_identity = sha256_32(&canonical);
    let mut cfg = Vec::new();
    for v in [
        RING_CAP as u64,
        MAX_CANDIDATES as u64,
        FEATS as u64,
        BATCH as u64,
        STEPS as u64,
        SEED_FIT,
    ] {
        cfg.extend_from_slice(&v.to_le_bytes());
    }
    cfg.extend_from_slice(&LR.to_le_bytes());
    let config_identity = sha256_32(&cfg);
    let trainer = SelectorTrainer::new(LR, SEED_FIT, data_identity, config_identity);
    Ok((trainer, examples, data_identity))
}

/// Per-position decision data for threshold sweeps: the best candidate score (or `i32::MIN`),
/// the hard prediction when reading, and the hard prediction without reading.
fn decision_rows(
    sel: &Selector,
    pop: &[Seq],
    parent: &PriorCore,
    local: &QueryHard,
    u: &[Vec<i32>],
    wm: &WriteMap<'_>,
    amp_shift: u32,
) -> Vec<(i32, bool, bool)> {
    let mut out = Vec::new();
    for s in pop.iter() {
        for p in analyze(s, parent, local, u, wm) {
            let z = local_logits(parent, local, u, &s.tokens, p.i);
            let local_ok = argmax_low(&z) as u32 == p.target;
            let mut best: Option<(i32, u32)> = None;
            for (k, f) in p.feats.iter().enumerate() {
                let sc = sel.score(f);
                if best.is_none_or(|(pr, _)| sc > pr) {
                    best = Some((sc, p.payloads[k]));
                }
            }
            match best {
                Some((sc, pay)) => {
                    let mut zz = z.clone();
                    apply_residual(&mut zz, Some(pay), amp_shift);
                    out.push((sc, argmax_low(&zz) as u32 == p.target, local_ok));
                }
                None => out.push((i32::MIN, local_ok, local_ok)),
            }
        }
    }
    out
}

fn sweep_accuracy(rows: &[(i32, bool, bool)], mu: i32) -> f64 {
    if rows.is_empty() {
        return f64::NAN;
    }
    let hits: usize = rows
        .iter()
        .map(|(sc, read_ok, local_ok)| usize::from(if *sc > mu { *read_ok } else { *local_ok }))
        .sum();
    hits as f64 / rows.len() as f64
}

fn sha256_32(b: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(b).into()
}

fn hex_of(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Real cross-process continuation check for the trainer actually used.
///
/// Three separate processes: A runs three updates and writes a checkpoint file; B starts from a
/// fresh process, reads that file and runs three more; a third runs all six uninterrupted. The
/// final parameters, the exported selector and the checkpoint bytes must be identical.
fn continuation_check(args: &Args, started: &Instant) -> Result<ExitCode, String> {
    let root = args.root.clone();
    claim(&root).map_err(|e| format!("claim {}: {e}", root.display()))?;
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let a_ckpt = root.join("a.ckpt");
    let b_out = root.join("b.json");
    let full_out = root.join("full.json");
    let run = |extra: &[&str]| -> Result<(), String> {
        let out = Command::new(&exe)
            .arg("--mode")
            .args(extra)
            .output()
            .map_err(|e| format!("spawn: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "subprocess failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        Ok(())
    };
    let ac = a_ckpt.to_str().ok_or("path")?.to_string();
    let bo = b_out.to_str().ok_or("path")?.to_string();
    let fo = full_out.to_str().ok_or("path")?.to_string();
    run(&["continuation-a", "--ckpt", &ac])?;
    run(&["continuation-b", "--ckpt", &ac, "--out", &bo])?;
    run(&["continuation-full", "--out", &fo])?;
    let b: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&b_out).map_err(|e| format!("{e}"))?)
            .map_err(|e| format!("{e}"))?;
    let f: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&full_out).map_err(|e| format!("{e}"))?)
            .map_err(|e| format!("{e}"))?;
    let same_step = b["step"] == f["step"];
    let same_params = b["w"] == f["w"] && b["mu"] == f["mu"];
    let same_selector = b["selector"] == f["selector"];
    let same_ckpt = b["checkpoint"] == f["checkpoint"];
    let ok = same_step && same_params && same_selector && same_ckpt;
    write_json(
        &root,
        "continuation.json",
        &json!({
            "kind": "cross_process_file_resume",
            "processes": 3,
            "split_at_step": 3,
            "total_steps": 6,
            "same_step": same_step, "same_float_parameters": same_params,
            "same_exported_selector": same_selector, "same_checkpoint_bytes": same_ckpt,
            "pass": ok,
            "declared_cache_reconstruction": "the per-epoch Fisher-Yates order is reconstructed from (rng, epoch); only cursor and RNG are carried in the checkpoint",
            "note": "process B loads the checkpoint from disk in a fresh process; no in-memory vector is passed",
        }),
    )?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "continuation-check pass={ok} (step {same_step}, params {same_params}, selector {same_selector}, ckpt {same_ckpt}); sealed {} unlisted; elapsed {:.1}s",
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}
