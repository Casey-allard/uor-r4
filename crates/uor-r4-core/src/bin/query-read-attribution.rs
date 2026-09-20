//! Frozen S attribution — **evaluation only**.
//!
//! Loads the retained corrected parent E and the three legacy `CPX3` query-read artifacts, repairs
//! their tokenizer binding into new descendants, reproduces the old saved vectors, freezes a
//! new-position panel and its donor map before scoring, and evaluates the exact identity-anchored
//! decomposition of the separable reader.
//!
//! No optimizer update, no fitting, no reset/write experiment, no new corpus and no decoder change
//! is reachable from this entry point: it never constructs a trainer.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::json;

use uor_r4_core::native_geometric::learner::prefix_artifact::{
    hex_lower, parent_hash_convention, ExactGroupTable,
};
use uor_r4_core::native_geometric::learner::prefix_state::palette;
use uor_r4_core::native_geometric::learner::prior_learning::{targets, PriorCore};
use uor_r4_core::native_geometric::learner::query_read::{
    QueryArm, QueryHard, CONTEXT, READER_WIDTH, RESIDUAL_MAX_ABS, ZERO_DIGEST,
};
use uor_r4_core::native_geometric::learner::realtext_support::*;
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::bpe_derive::{derive_tokenizer, derive_tokenizer_json};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const DEFAULT_ROOT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/s-attribution-1";
const ROOT_DIR: &str = "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20";
const DEFAULT_PARENT: &str = "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/head-projection-3/corrected/empirical.cpl2";
const DEFAULT_CHECKPOINT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/attempt-1/prior_realtext.ckpt";
const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json";
const DEFAULT_DOCS: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/inputs/docs";

const E_SHA: &str = "565cf98a01273af38425687adb522704b09a89bd7ab976b58b20bfbdaccfa0bf";
const Q_SHA: &str = "341f07211bb8695604c5cae50f3a529a1178def83d4d431f9031b2636a78a3fc";
const S_SHA: &str = "9f8cda09bfb562221b5718a8330cc4a296215b8e94d3e0c2b8abe2ce31fae851";
const L_SHA: &str = "b0e5088794c2dedc87adabfe988cf2530fe5605863ca33ad3ce8078924b372a4";
const PANEL2_SHA: &str = "b58d104083164dba30bc8251f0957ac1da9b3bec7d9c3b6ea299d6a9c7e1f79f";
const RESULT2_SHA: &str = "4aa6ed817f1e277007101ac06a56065c7050e2a281e36346ecdb07c2e58b42fc";
const CKPT_SHA: &str = "1c3dfedb2248ec76c21f320b7ecec585c7a522a2d47e35ed5dc9eaca2454bff2";
const TOKENIZER_SHA: &str = "9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c";
const DERIVED_SHA: &str = "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";

const OLD_E_MICRO: f64 = 7.170815717556;

const FIT_WINDOWS: usize = 4096;
const OLD_PANEL_TARGETS: usize = 17342;
const OLD_PANEL_WINDOWS: usize = 288;
const OLD_PANEL_DOCS: usize = 36;
/// The declared new-position donor seed, bound before scoring.
const NEW_DONOR_SEED: u64 = 0x5E9A_11B7;
/// The common newly declared comparison seed.
const COMMON_SEED: u64 = 0x1234_5678;
const OLD_DONOR_SEED: u64 = 0xD1B5_4A32;
/// The newly declared component-level practical margin, in bits per eligible target.
const COMPONENT_MARGIN: f64 = 0.01;
/// Repaired-artifact tolerance (bits) for old-vector reproduction.
const REPRO_TOL: f64 = 1e-8;

const GEN_TOKENS: usize = 64;
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
// Conditions
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Cond {
    E,
    S11,
    S01,
    S10,
    S00,
    Qonly,
    Honly,
}

const ALL_CONDS: [Cond; 7] = [
    Cond::E,
    Cond::S11,
    Cond::S01,
    Cond::S10,
    Cond::S00,
    Cond::Qonly,
    Cond::Honly,
];

impl Cond {
    fn label(self) -> &'static str {
        match self {
            Cond::E => "E",
            Cond::S11 => "S11",
            Cond::S01 => "S01",
            Cond::S10 => "S10",
            Cond::S00 => "S00",
            Cond::Qonly => "Qonly",
            Cond::Honly => "Honly",
        }
    }
    /// Whether the condition consumes the individual older state `q`.
    fn takes_older(self) -> bool {
        matches!(self, Cond::S11 | Cond::S10 | Cond::Honly)
    }
    /// Whether the condition consumes the current-token query `b`.
    fn takes_query(self) -> bool {
        matches!(self, Cond::S11 | Cond::S01 | Cond::Qonly)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
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
    let bytes: Vec<u8> = xs.iter().flat_map(|x| x.to_le_bytes()).collect();
    write_checked(root, name, &bytes)
}

fn read_f64s(path: &Path) -> Result<Vec<f64>, String> {
    let b = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if b.len() % 8 != 0 {
        return Err(format!("{}: not f64 aligned", path.display()));
    }
    Ok(b.chunks_exact(8)
        .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
        .collect())
}

fn argmax_low(z: &[i32]) -> usize {
    let mut best = 0usize;
    for r in 1..z.len() {
        if z[r] > z[best] {
            best = r;
        }
    }
    best
}

/// Stable uncapped log-sum-exp bits over f64 diagnostic logits.
fn bits_f64(logits: &[f64], target: usize) -> f64 {
    let mut max = f64::NEG_INFINITY;
    for &x in logits {
        if x > max {
            max = x;
        }
    }
    let mut sum = 0.0f64;
    for &x in logits {
        sum += (x - max).exp();
    }
    (max + sum.ln() - logits[target]) / std::f64::consts::LN_2
}

fn win_bytes(w: &[u32]) -> Vec<u8> {
    w.iter().flat_map(|t| t.to_le_bytes()).collect()
}

/// Paired document bootstrap with explicit zero-support accounting.
fn paired_detailed(a: &Agg, b: &Agg, seed: u64) -> (f64, f64, f64, usize, usize) {
    assert_eq!(a.rows.len(), b.rows.len(), "scorers must be aligned");
    let n = a.rows.len();
    if n == 0 {
        return (f64::NAN, f64::NAN, f64::NAN, 0, 0);
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
    let mut dropped = 0usize;
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
        } else {
            dropped += 1;
        }
    }
    if boots.len() < BOOTSTRAP_DRAWS / 2 {
        return (point, f64::NAN, f64::NAN, boots.len(), dropped);
    }
    boots.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let lo = boots[(boots.len() as f64 * 0.025) as usize];
    let hi = boots[((boots.len() as f64 * 0.975) as usize).min(boots.len() - 1)];
    (point, lo, hi, boots.len(), dropped)
}

fn iv(t: (f64, f64, f64, usize, usize)) -> serde_json::Value {
    json!({"point": t.0, "lo": t.1, "hi": t.2, "draws_used": t.3, "draws_dropped_zero_support": t.4})
}

fn f(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or(f64::NAN)
}

/// The permutation embedded in the retained `CPCK` v3 checkpoint.
fn cpc_k_perm(bytes: &[u8]) -> Result<Vec<usize>, String> {
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
        return Err("bad CPCK magic".into());
    }
    let u32_at = |c: &mut usize| -> Result<u32, String> {
        Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
    };
    let u64_at = |c: &mut usize| -> Result<u64, String> {
        Ok(u64::from_le_bytes(take(c, 8)?.try_into().unwrap()))
    };
    if u32_at(&mut c)? != 3 {
        return Err("unsupported CPCK version".into());
    }
    for _ in 0..5 {
        let _ = u32_at(&mut c)?;
    }
    for _ in 0..4 {
        let _ = u64_at(&mut c)?;
    }
    for _ in 0..5 {
        let _ = u32_at(&mut c)?;
    }
    let _ = u64_at(&mut c)?;
    let _ = take(&mut c, 32)?;
    let perm_len = u64_at(&mut c)? as usize;
    let mut perm = Vec::with_capacity(perm_len);
    let mut seen = vec![false; perm_len];
    for _ in 0..perm_len {
        let p = u64_at(&mut c)? as usize;
        if p >= perm_len || std::mem::replace(&mut seen[p], true) {
            return Err("CPCK permutation is not a permutation".into());
        }
        perm.push(p);
    }
    Ok(perm)
}

// ---------------------------------------------------------------------------
// Evaluation environment
// ---------------------------------------------------------------------------

/// One frozen arm's reader-row table plus cached frozen-parent logits.
struct Env {
    parent: PriorCore,
    art: QueryHard,
    /// `u[s][v]`: the single-row residual of the separable reader at state `s`.
    u: Vec<Vec<i32>>,
    e: usize,
    z_cache: HashMap<(usize, usize), Vec<i32>>,
}

impl Env {
    fn new(parent: PriorCore, art: QueryHard) -> Self {
        let u = (0..120usize).map(|s| art.row_scores(s)).collect();
        let e = art.table.identity as usize;
        Self {
            parent,
            art,
            u,
            e,
            z_cache: HashMap::new(),
        }
    }

    fn z_e(&mut self, prev: usize, cur: usize) -> Vec<i32> {
        if !self.z_cache.contains_key(&(prev, cur)) {
            let z = self.parent.int_logits(prev, cur, true);
            self.z_cache.insert((prev, cur), z);
        }
        self.z_cache[&(prev, cur)].clone()
    }

    /// The arm's own two-row residual read.
    fn z_rows(&mut self, prev: usize, cur: usize, rows: Option<(usize, usize)>) -> Vec<i32> {
        let mut z = self.z_e(prev, cur);
        if let Some((a, b)) = rows {
            let (ua, ub) = (&self.u[a], &self.u[b]);
            for (i, x) in z.iter_mut().enumerate() {
                *x += ua[i] + ub[i];
            }
        }
        z
    }

    /// The frozen separable decomposition. `rows` is the separable arm's `(q, b)`; absent-prefix
    /// positions are `None` and every condition then equals `E`.
    fn z_cond(
        &mut self,
        prev: usize,
        cur: usize,
        rows: Option<(usize, usize)>,
        cond: Cond,
    ) -> Vec<i32> {
        let mut z = self.z_e(prev, cur);
        let (q, b) = match rows {
            None => return z,
            Some(rb) => rb,
        };
        let e = self.e;
        let add = |z: &mut Vec<i32>, r: usize, u: &Vec<Vec<i32>>| {
            for (i, x) in z.iter_mut().enumerate() {
                *x += u[r][i];
            }
        };
        match cond {
            Cond::E => {}
            Cond::S11 => {
                add(&mut z, q, &self.u);
                add(&mut z, b, &self.u);
            }
            Cond::S01 => {
                add(&mut z, e, &self.u);
                add(&mut z, b, &self.u);
            }
            Cond::S10 => {
                add(&mut z, q, &self.u);
                add(&mut z, e, &self.u);
            }
            Cond::S00 => {
                add(&mut z, e, &self.u);
                add(&mut z, e, &self.u);
            }
            Cond::Qonly => add(&mut z, b, &self.u),
            Cond::Honly => add(&mut z, q, &self.u),
        }
        z
    }
}

/// Rows with the query neutralised to the identity element.
fn rows_identity_query(core: &QueryHard, tokens: &[u32], i: usize) -> Option<(usize, usize)> {
    let e = core.table.identity as usize;
    match core.arm {
        QueryArm::L => core.tail_state(tokens, i).map(|t| (t, e)),
        _ => core.older_state(tokens, i).map(|q| (q, e)),
    }
}

/// Rows with the older prefix reversed in time; the local arm is a no-op.
fn rows_reversed(core: &QueryHard, tokens: &[u32], i: usize) -> Option<(usize, usize)> {
    if i < 2 || i >= tokens.len() {
        return None;
    }
    let e = core.table.identity as usize;
    match core.arm {
        QueryArm::L => core.inference_rows(tokens, i),
        QueryArm::S => {
            let q = reversed_older(core, tokens, i)?;
            Some((q, core.query_state(tokens[i])))
        }
        QueryArm::Q => {
            let q = reversed_older(core, tokens, i)?;
            Some((
                core.table
                    .compose(q as u8, core.query_state(tokens[i]) as u8) as usize,
                e,
            ))
        }
    }
}

fn reversed_older(core: &QueryHard, tokens: &[u32], i: usize) -> Option<usize> {
    if i < 2 || i - 1 <= i.saturating_sub(CONTEXT - 1) {
        return None;
    }
    let start = i.saturating_sub(CONTEXT - 1);
    let mut q = core.table.identity as usize;
    for &tok in tokens[start..i - 1].iter().rev() {
        q = core.table.compose(q as u8, core.write_state(tok) as u8) as usize;
    }
    Some(q)
}

/// Greedy generation for one frozen condition, lowest-ID ties, no decoder change.
fn generate_cond(env: &mut Env, prompt: &[u32], n_new: usize, cond: Cond) -> Vec<u32> {
    generate_cond_with(env, prompt, n_new, cond, false)
}

/// As [`generate_cond`], optionally through the retained training-path row selection. The two paths
/// must agree exactly; only their cost differs.
fn generate_cond_with(
    env: &mut Env,
    prompt: &[u32],
    n_new: usize,
    cond: Cond,
    old_path: bool,
) -> Vec<u32> {
    let v = env.parent.cfg.vocab;
    let mut toks = prompt.to_vec();
    for _ in 0..n_new {
        let i = toks.len() - 1;
        let prev = if i == 0 { v } else { toks[i - 1] as usize };
        let cur = toks[i] as usize;
        let rows = if old_path {
            env.art.read_path(&toks, i).map(|p| (p.first, p.second))
        } else {
            env.art.inference_rows(&toks, i)
        };
        let z = env.z_cond(prev, cur, rows, cond);
        toks.push(black_box(argmax_low(&z)) as u32);
    }
    toks[prompt.len()..].to_vec()
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
    let (root, source_root) = {
        let mut a = std::env::args().skip(1);
        let mut root = PathBuf::from(DEFAULT_ROOT);
        let mut source_root: Option<PathBuf> = None;
        while let Some(k) = a.next() {
            match k.as_str() {
                "--root" => {
                    root = PathBuf::from(a.next().ok_or("--root needs a value")?);
                }
                "--source-root" => {
                    source_root = Some(PathBuf::from(
                        a.next().ok_or("--source-root needs a value")?,
                    ));
                }
                other => return Err(format!("unknown argument {other}")),
            }
        }
        (root, source_root)
    };
    claim(&root).map_err(|e| format!("claim {}: {e}", root.display()))?;
    let started = Instant::now();
    let base = Path::new(ROOT_DIR);
    let mut marks: Vec<(&str, f64)> = Vec::new();
    let mark = |name: &'static str, t: Instant, marks: &mut Vec<(&'static str, f64)>| {
        marks.push((name, t.elapsed().as_secs_f64()));
    };

    // ---- pinned inputs -------------------------------------------------------
    let parent_bytes = std::fs::read(DEFAULT_PARENT).map_err(|e| format!("parent: {e}"))?;
    if sha256_hex(&parent_bytes) != E_SHA {
        return Err("parent E sha mismatch".into());
    }
    let parent = PriorCore::from_bytes(&parent_bytes).map_err(|e| format!("load E: {e}"))?;
    parent.validate()?;
    let (parent_file_digest, _) = parent_hash_convention(&parent_bytes);
    let v = parent.cfg.vocab;

    let ckpt = std::fs::read(DEFAULT_CHECKPOINT).map_err(|e| format!("ckpt: {e}"))?;
    if sha256_hex(&ckpt) != CKPT_SHA {
        return Err("checkpoint sha mismatch".into());
    }
    let tok_bytes = std::fs::read(DEFAULT_TOKENIZER).map_err(|e| format!("tokenizer: {e}"))?;
    if sha256_hex(&tok_bytes) != TOKENIZER_SHA {
        return Err("tokenizer sha mismatch".into());
    }
    let derived_hex = sha256_hex(
        &derive_tokenizer_json(&tok_bytes, VOCAB).map_err(|e| format!("derive json: {e}"))?,
    );
    if derived_hex != DERIVED_SHA {
        return Err(format!("derived tokenizer sha {derived_hex} != pinned"));
    }
    let tokenizer: HfBpeTokenizer =
        derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive tokenizer: {e}"))?;
    let raw_tok: [u8; 32] = hex_to_bytes(&derived_hex)?.try_into().unwrap();
    if raw_tok == ZERO_DIGEST {
        return Err("the raw tokenizer digest is the placeholder".into());
    }
    let legacy: [(&str, QueryArm, &str); 3] = [
        ("query_conditioned_older_read", QueryArm::Q, Q_SHA),
        ("separable_older_query_read", QueryArm::S, S_SHA),
        ("local_only_read", QueryArm::L, L_SHA),
    ];
    let mut legacy_bytes: HashMap<&str, Vec<u8>> = HashMap::new();
    for (stem, _, sha) in legacy.iter() {
        let p = base.join(format!("query-read-2/artifacts/{stem}.cpx3"));
        let b = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        if sha256_hex(&b) != *sha {
            return Err(format!("{stem}: sha mismatch"));
        }
        legacy_bytes.insert(stem, b);
    }
    let panel2: serde_json::Value = serde_json::from_slice(
        &std::fs::read(base.join("query-read-2/panel.json")).map_err(|e| format!("{e}"))?,
    )
    .map_err(|e| format!("panel2 parse: {e}"))?;
    let result2: serde_json::Value = serde_json::from_slice(
        &std::fs::read(base.join("query-read-2/result.json")).map_err(|e| format!("{e}"))?,
    )
    .map_err(|e| format!("result2 parse: {e}"))?;
    if sha256_hex(&std::fs::read(base.join("query-read-2/panel.json")).unwrap()) != PANEL2_SHA
        || sha256_hex(&std::fs::read(base.join("query-read-2/result.json")).unwrap()) != RESULT2_SHA
    {
        return Err("retained query-read-2 manifest sha mismatch".into());
    }
    // Executable identity: the running binary hashes itself, so the report identifies the actual
    // executable rather than a symbolic revision name.
    let executable_sha256 = match std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::read(p).ok())
    {
        Some(b) => sha256_hex(&b),
        None => "UNAVAILABLE".to_string(),
    };
    let declared_sources = [
        "crates/uor-r4-core/src/native_geometric/learner/query_read.rs",
        "crates/uor-r4-core/src/bin/query-read-attribution.rs",
    ];
    let source_hashes: Vec<serde_json::Value> = match &source_root {
        Some(sr) => declared_sources
            .iter()
            .map(|rel| {
                let p = sr.join(rel);
                match std::fs::read(&p) {
                    Ok(b) => json!({"path": rel, "sha256": sha256_hex(&b)}),
                    Err(_) => json!({"path": rel, "sha256": "UNAVAILABLE"}),
                }
            })
            .collect(),
        None => declared_sources
            .iter()
            .map(|rel| json!({"path": rel, "sha256": "UNAVAILABLE — no --source-root supplied"}))
            .collect(),
    };
    let exact = ExactGroupTable::build().map_err(|e| format!("exact table: {e}"))?;
    let pal = palette().clone();
    if pal.identity != exact.identity {
        return Err("palette identity differs from the bound table".into());
    }
    mark("pinned inputs + corpus setup", started, &mut marks);

    // ---- import legacy, repair the tokenizer binding -------------------------
    let mut repaired: BTreeMap<&str, QueryHard> = BTreeMap::new();
    let mut repair_rows: Vec<serde_json::Value> = Vec::new();
    for (stem, arm, sha) in legacy.iter() {
        let bytes = &legacy_bytes[stem];
        if bytes.len() < 105 || &bytes[73..105] != &ZERO_DIGEST {
            return Err(format!(
                "{stem}: tokenizer field is not the legacy placeholder"
            ));
        }
        let imported = QueryHard::import_legacy(bytes, &parent, &parent_file_digest)
            .map_err(|e| format!("{stem}: legacy import: {e}"))?;
        if imported.arm != *arm {
            return Err(format!("{stem}: arm mismatch"));
        }
        if QueryHard::from_bytes(bytes, &parent, &parent_file_digest, &raw_tok).is_ok() {
            return Err(format!("{stem}: production load accepted the placeholder"));
        }
        let fixed = QueryHard::from_parts(
            imported.parent.clone(),
            imported.reader.clone(),
            imported.wg.clone(),
            imported.a_codes.clone(),
            imported.b_codes.clone(),
            imported.palette.clone(),
            imported.arm,
            imported.table.clone(),
            parent_file_digest,
            raw_tok,
        )
        .map_err(|e| format!("{stem}: repair: {e}"))?;
        let fixed_bytes = fixed.to_bytes();
        if fixed_bytes.len() != bytes.len() {
            return Err(format!("{stem}: repaired size differs"));
        }
        let diff: Vec<usize> = (0..fixed_bytes.len())
            .filter(|&k| fixed_bytes[k] != bytes[k])
            .collect();
        if diff.len() != 32 || !diff.iter().all(|k| (73..105).contains(k)) {
            return Err(format!("{stem}: repair changed bytes outside 73..105"));
        }
        let reloaded = QueryHard::from_bytes(&fixed_bytes, &parent, &parent_file_digest, &raw_tok)
            .map_err(|e| format!("{stem}: repaired reload: {e}"))?;
        // Full adversarial row/logit parity against the imported legacy artifact, including absent
        // and identity states.
        let mut checked = 0usize;
        for k in 0..64usize {
            let mut w: Vec<u32> = (0..64usize)
                .map(|t| ((k * 31 + t * 7) % v) as u32)
                .collect();
            if k % 4 == 1 {
                w.truncate(3 + k % 5);
            }
            for i in 0..w.len() {
                let a = imported.inference_rows(&w, i);
                let b = reloaded.inference_rows(&w, i);
                if a != b {
                    return Err(format!("{stem}: repaired rows differ at {k}/{i}"));
                }
                let prev = if i == 0 { v } else { w[i - 1] as usize };
                if imported.int_logits(prev, w[i] as usize, a)
                    != reloaded.int_logits(prev, w[i] as usize, b)
                {
                    return Err(format!("{stem}: repaired logits differ at {k}/{i}"));
                }
                checked += 1;
            }
        }
        repair_rows.push(json!({
            "arm": arm.name(),
            "old_path": format!("query-read-2/artifacts/{stem}.cpx3"),
            "old_sha256": sha,
            "new_path": format!("corrected/{stem}.cpx3"),
            "new_sha256": sha256_hex(&fixed_bytes),
            "bytes": fixed_bytes.len(),
            "changed_offsets": diff.len(),
            "changed_offset_range": [73, 105],
            "tokenizer_digest": hex_lower(&raw_tok),
            "row_and_logit_parity_positions": checked,
            "production_loader_rejected_placeholder": true,
        }));
        write_checked(&root, &format!("corrected/{stem}.cpx3"), &fixed_bytes)?;
        repaired.insert(stem, reloaded);
    }
    mark(
        "legacy import + tokenizer repair + parity",
        started,
        &mut marks,
    );

    // ---- populations ---------------------------------------------------------
    let (uniq, collected, duplicates) = reconstruct_corpus(Path::new(DEFAULT_DOCS));
    let fit_ids: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Fit)
        .collect();
    let dev_pool: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Dev)
        .collect();
    let mut fit_windows: Vec<Vec<u32>> = Vec::new();
    for i in &fit_ids {
        fit_windows.extend(windows_of(&tokenizer.encode(&uniq[*i].text)));
    }
    let perm = cpc_k_perm(&ckpt)?;
    if perm.len() != fit_windows.len() {
        return Err("checkpoint permutation length != fit windows".into());
    }
    let consumed: Vec<usize> = perm[..FIT_WINDOWS.min(perm.len())].to_vec();
    let consumed_windows: Vec<Vec<u32>> =
        consumed.iter().map(|&w| fit_windows[w].clone()).collect();
    let consumed_targets: usize = consumed_windows.iter().map(|w| w.len() - 1).sum();

    let old_panel = build_panel(&uniq, &dev_pool, &tokenizer, 8);
    if old_panel.recs.len() != OLD_PANEL_TARGETS
        || old_panel.windows.len() != OLD_PANEL_WINDOWS
        || old_panel.docs.iter().filter(|(_, _, s)| *s > 0).count() != OLD_PANEL_DOCS
    {
        return Err("old panel reconstruction mismatch".into());
    }
    let old_donor = build_perm(&old_panel.recs, 0xA5A5_1234u64);
    {
        let recs = panel2["records"].as_array().ok_or("panel2 records")?;
        if recs.len() != old_panel.recs.len() {
            return Err("panel2 record count differs".into());
        }
        for (k, r) in old_panel.recs.iter().enumerate() {
            let j = &recs[k];
            if j["obs"] != r.obs
                || j["doc"] != r.doc
                || j["window"].as_u64() != Some(r.win as u64)
                || j["i"].as_u64() != Some(r.i as u64)
                || j["prev"].as_u64() != Some(r.prev as u64)
                || j["cur"].as_u64() != Some(r.cur as u64)
                || j["target"].as_u64() != Some(r.target as u64)
                || j["older_len"].as_u64() != Some(r.older_len as u64)
            {
                return Err(format!("panel2 record {k} differs"));
            }
        }
        let donors = panel2["donor_map"].as_array().ok_or("panel2 donors")?;
        for (k, d) in donors.iter().enumerate() {
            if d["eligible"].as_bool() != Some(old_donor.eligible[k])
                || d["donor_index"].as_u64() != Some(old_donor.donor[k] as u64)
            {
                return Err(format!("panel2 donor {k} differs"));
            }
        }
    }
    mark(
        "populations + pinned panel verification",
        started,
        &mut marks,
    );

    // ---- reproduce the old saved vectors -------------------------------------
    let mut envs: BTreeMap<&str, Env> = BTreeMap::new();
    for (stem, _, _) in legacy.iter() {
        envs.insert(stem, Env::new(parent.clone(), repaired[stem].clone()));
    }
    let mut repro_rows: Vec<serde_json::Value> = Vec::new();
    {
        let env = envs.get_mut("separable_older_query_read").unwrap();
        let e_loss: Vec<f64> = old_panel
            .recs
            .iter()
            .map(|r| {
                let z = env.z_e(r.prev, r.cur);
                env.parent.bits_one(&z, r.target)
            })
            .collect();
        let saved = read_f64s(&base.join("query-read-2/vectors/E.f64"))?;
        let max_delta = e_loss
            .iter()
            .zip(saved.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        let agg = aggregate(&old_panel.recs, &e_loss, None);
        if (agg.micro() - OLD_E_MICRO).abs() > REPRO_TOL {
            return Err(format!("E micro {} does not reproduce", agg.micro()));
        }
        repro_rows.push(json!({
            "vector": "E", "micro": agg.micro(), "old_micro": OLD_E_MICRO,
            "max_abs_delta": max_delta, "within_tolerance": max_delta <= REPRO_TOL,
        }));
    }
    for (stem, arm, _) in legacy.iter() {
        let env = envs.get_mut(stem).unwrap();
        let own: Vec<Option<(usize, usize)>> = old_panel
            .recs
            .iter()
            .map(|r| env.art.inference_rows(&old_panel.windows[r.win], r.i))
            .collect();
        let donor: Vec<Option<(usize, usize)>> = (0..old_panel.recs.len())
            .map(|k| {
                if old_donor.eligible[k] {
                    let d = old_donor.donor[k];
                    let dw = &old_panel.windows[old_panel.recs[d].win];
                    match env.art.older_state(dw, old_panel.recs[d].i) {
                        Some(o) => env.art.rows_from_older(
                            &old_panel.windows[old_panel.recs[k].win],
                            old_panel.recs[k].i,
                            o,
                        ),
                        None => own[k],
                    }
                } else {
                    own[k]
                }
            })
            .collect();
        let identity: Vec<Option<(usize, usize)>> = old_panel
            .recs
            .iter()
            .map(|r| rows_identity_query(&env.art, &old_panel.windows[r.win], r.i))
            .collect();
        let reversed: Vec<Option<(usize, usize)>> = old_panel
            .recs
            .iter()
            .map(|r| rows_reversed(&env.art, &old_panel.windows[r.win], r.i))
            .collect();
        let disabled: Vec<Option<(usize, usize)>> = vec![None; old_panel.recs.len()];
        for (name, rows) in [
            ("own", &own),
            ("donor", &donor),
            ("identity_query", &identity),
            ("reversed", &reversed),
            ("disabled", &disabled),
        ] {
            let losses: Vec<f64> = old_panel
                .recs
                .iter()
                .zip(rows.iter())
                .map(|(r, st)| {
                    let z = env.z_rows(r.prev, r.cur, *st);
                    env.parent.bits_one(&z, r.target)
                })
                .collect();
            let saved =
                read_f64s(&base.join(format!("query-read-2/vectors/{}-{name}.f64", arm.name())))?;
            let max_delta = losses
                .iter()
                .zip(saved.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f64, f64::max);
            if max_delta > REPRO_TOL {
                return Err(format!(
                    "{}: {name} vector differs by {max_delta}",
                    arm.name()
                ));
            }
            let agg = aggregate(&old_panel.recs, &losses, None);
            repro_rows.push(json!({
                "arm": arm.name(), "vector": name, "micro": agg.micro(),
                "max_abs_delta": max_delta, "within_tolerance": true,
            }));
        }
    }
    mark("old-vector reproduction", started, &mut marks);

    // ---- fit-only history means (M01) ----------------------------------------
    let s_core = repaired["separable_older_query_read"].clone();
    let mut sums: BTreeMap<usize, Vec<i64>> = BTreeMap::new();
    let mut counts: BTreeMap<usize, u64> = BTreeMap::new();
    for w in consumed_windows.iter() {
        for i in 2..w.len() {
            let Some((q, _b)) = s_core.inference_rows(w, i) else {
                continue;
            };
            let len = older_len(i);
            let e = sums.entry(len).or_insert_with(|| vec![0i64; READER_WIDTH]);
            for (j, slot) in e.iter_mut().enumerate() {
                *slot += s_core.reader.weight(q, j) as i64;
            }
            *counts.entry(len).or_insert(0) += 1;
        }
    }
    let means: BTreeMap<usize, Vec<f64>> = sums
        .iter()
        .map(|(len, s)| {
            (
                *len,
                s.iter().map(|x| *x as f64 / counts[len] as f64).collect(),
            )
        })
        .collect();
    write_json(
        &root,
        "fit-history-means.json",
        &json!({
            "provenance": "4096 consumed fit windows, frozen S A map and reader codes; no labels, tune or dev windows, no code optimization, no refitting",
            "lengths": means.iter().map(|(len, mu)| json!({
                "older_len": len, "count": counts[len], "sums": sums[len], "mean": mu,
            })).collect::<Vec<_>>(),
        }),
    )?;
    let supported: Vec<usize> = means.keys().copied().collect();
    mark("fit history means", started, &mut marks);

    // ---- freeze the new-position selection -----------------------------------
    let mut old_hashes: BTreeSet<String> = BTreeSet::new();
    for i in &dev_pool {
        let ws = windows_of(&tokenizer.encode(&uniq[*i].text));
        if ws.is_empty() {
            continue;
        }
        for idx in spread_indices(ws.len()) {
            old_hashes.insert(sha256_hex(&win_bytes(&ws[idx])));
        }
    }
    let mut new_windows: Vec<Vec<u32>> = Vec::new();
    let mut new_docs: Vec<String> = Vec::new();
    let mut selection: Vec<serde_json::Value> = Vec::new();
    let mut per_doc: Vec<serde_json::Value> = Vec::new();
    for i in &dev_pool {
        let key = hex(&uniq[*i].sha256);
        let ws = windows_of(&tokenizer.encode(&uniq[*i].text));
        if ws.is_empty() {
            per_doc.push(json!({"doc": key, "available_chunks": 0, "selected": 0, "reason": "no chunk of at least 3 tokens"}));
            continue;
        }
        let used = spread_indices(ws.len());
        let mut cands: Vec<(String, usize)> = Vec::new();
        for idx in 0..ws.len() {
            if used.contains(&idx) || old_hashes.contains(&sha256_hex(&win_bytes(&ws[idx]))) {
                continue;
            }
            let h = sha256_hex(format!("uor-r4-S-attribution-v1|{key}|{idx}").as_bytes());
            cands.push((h, idx));
        }
        cands.sort();
        let picked: Vec<(String, usize)> = cands.into_iter().take(8).collect();
        per_doc.push(json!({
            "doc": key, "available_chunks": ws.len(), "old_selected": used.len(),
            "selected": picked.len(), "shortfall": 8usize.saturating_sub(picked.len()),
        }));
        for (h, idx) in picked {
            new_windows.push(ws[idx].clone());
            new_docs.push(key.clone());
            selection.push(json!({
                "doc": key, "local_chunk_index": idx,
                "token_start": idx * WINDOW, "token_end": idx * WINDOW + ws[idx].len(),
                "token_window_sha256": sha256_hex(&win_bytes(&ws[idx])), "rank_digest": h,
            }));
        }
    }
    if new_windows.len() > OLD_PANEL_WINDOWS {
        return Err("new panel exceeded the declared window bound".into());
    }
    let mut new_recs: Vec<Rec> = Vec::new();
    for (wi, w) in new_windows.iter().enumerate() {
        for (i, prev, cur, target) in targets(w, VOCAB) {
            new_recs.push(Rec {
                obs: format!("{}:{}:{}", new_docs[wi], wi, i),
                doc: new_docs[wi].clone(),
                win: wi,
                i,
                prev,
                cur,
                target,
                older_len: older_len(i),
            });
        }
    }
    write_json(
        &root,
        "selection-manifest.json",
        &json!({
            "rule": "SHA256(\"uor-r4-S-attribution-v1|<doc_sha256_hex>|<decimal_local_chunk_index>\") lexicographic digest order, index tie-break, top 8 per document, excluding every old selected chunk and every exact duplicate token window",
            "frozen_before_scoring": true,
            "dev_documents": dev_pool.len(),
            "windows": new_windows.len(),
            "targets": new_recs.len(),
            "selection": selection,
            "per_document": per_doc,
        }),
    )?;
    let new_donor = build_perm(&new_recs, NEW_DONOR_SEED);
    let new_strata = strata_stats(&new_recs);
    mark(
        "frozen new-position selection + donor map",
        started,
        &mut marks,
    );
    println!(
        "corpus collected={collected} eligible={} duplicates={duplicates} fit_windows={} consumed={} targets={consumed_targets}; old panel {}/{}; new panel {} windows / {} targets / {} docs; donor seed {NEW_DONOR_SEED:#x} eligible {} strata {}",
        uniq.len(),
        fit_windows.len(),
        consumed_windows.len(),
        old_panel.windows.len(),
        old_panel.recs.len(),
        new_windows.len(),
        new_recs.len(),
        new_docs.iter().collect::<BTreeSet<_>>().len(),
        new_donor.eligible.iter().filter(|x| **x).count(),
        new_donor.strata,
    );

    // ======================= evaluation (no fitting below) ====================
    let mut env = envs.remove("separable_older_query_read").unwrap();
    let scale = (-(parent.cfg.f_bits as f64)).exp2();
    let mut panels: Vec<serde_json::Value> = Vec::new();
    for (name, recs, windows) in [
        ("old", &old_panel.recs, &old_panel.windows),
        ("new", &new_recs, &new_windows),
    ] {
        let mut loss: HashMap<&'static str, Vec<f64>> = HashMap::new();
        for cond in ALL_CONDS.iter() {
            loss.insert(cond.label(), Vec::with_capacity(recs.len()));
        }
        let mut m01: Vec<Option<f64>> = vec![None; recs.len()];
        let mut q_ids = Vec::with_capacity(recs.len());
        let mut b_ids = Vec::with_capacity(recs.len());
        let mut identity_failures = 0usize;
        for (k, r) in recs.iter().enumerate() {
            let w = &windows[r.win];
            let rows = env.art.inference_rows(w, r.i);
            for cond in ALL_CONDS.iter() {
                let z = env.z_cond(r.prev, r.cur, rows, *cond);
                let b = parent.bits_one(&z, r.target);
                loss.get_mut(cond.label()).unwrap().push(b);
            }
            let Some((q, b)) = rows else {
                q_ids.push(usize::MAX);
                b_ids.push(usize::MAX);
                continue;
            };
            q_ids.push(q);
            b_ids.push(b);
            // Exact integer identity Z11 + Z00 = Z10 + Z01 on every vocabulary row (wide ints).
            let z11 = env.z_cond(r.prev, r.cur, rows, Cond::S11);
            let z00 = env.z_cond(r.prev, r.cur, rows, Cond::S00);
            let z10 = env.z_cond(r.prev, r.cur, rows, Cond::S10);
            let z01 = env.z_cond(r.prev, r.cur, rows, Cond::S01);
            for rv in 0..z11.len() {
                let l = z11[rv] as i64 + z00[rv] as i64;
                let rr = z10[rv] as i64 + z01[rv] as i64;
                if l != rr {
                    identity_failures += 1;
                }
            }
            // M01: fit-average history at the same length, offline diagnostic only.
            if let Some(mu) = means.get(&r.older_len) {
                let e = env.z_e(r.prev, r.cur);
                let mut zf = vec![0.0f64; v];
                for (row, slot) in zf.iter_mut().enumerate() {
                    let mut acc = 0.0f64;
                    for j in 0..READER_WIDTH {
                        let code = s_core.wg.weight(row, j) as f64;
                        if code != 0.0 {
                            acc += code * (mu[j] + s_core.reader.weight(b, j) as f64);
                        }
                    }
                    *slot = e[row] as f64 * scale + 128.0 * acc * scale;
                }
                m01[k] = Some(bits_f64(&zf, (r.target as usize).min(v - 1)));
            }
        }
        let older_mask: Vec<bool> = recs.iter().map(|r| r.older_len > 0).collect();
        let support_mask: Vec<bool> = m01.iter().map(|x| x.is_some()).collect();
        let matched: Vec<bool> = older_mask
            .iter()
            .zip(support_mask.iter())
            .map(|(a, b)| *a && *b)
            .collect();
        let m01_vals: Vec<f64> = m01
            .iter()
            .zip(matched.iter())
            .map(|(x, m)| if *m { x.unwrap() } else { 0.0 })
            .collect();
        let mut cond_json = Vec::new();
        for cond in ALL_CONDS.iter() {
            let l = &loss[cond.label()];
            let all = aggregate(recs, l, None);
            let present = aggregate(recs, l, Some(&older_mask));
            cond_json.push(json!({
                "condition": cond.label(),
                "takes_older": cond.takes_older(),
                "takes_query": cond.takes_query(),
                "micro_all": all.micro(),
                "macro_all": all.macro_bits(),
                "micro_older_present": present.micro(),
                "macro_older_present": present.macro_bits(),
            }));
        }
        let m01_agg = aggregate(recs, &m01_vals, Some(&matched));
        cond_json.push(json!({
            "condition": "M01",
            "offline_only": true,
            "micro_matched": m01_agg.micro(),
            "macro_matched": m01_agg.macro_bits(),
            "matched_targets": matched.iter().filter(|x| **x).count(),
        }));
        let cmp = |a: &[f64], b: &[f64]| {
            iv(paired_detailed(
                &aggregate(recs, a, Some(&matched)),
                &aggregate(recs, b, Some(&matched)),
                COMMON_SEED,
            ))
        };
        let comparisons = json!({
            "mask": "older-present positions with fit-mean support on this panel",
            "matched_targets": matched.iter().filter(|x| **x).count(),
            "sign_convention": "CE_a - CE_b; positive means the second condition is better",
            "primary_CE_M01_minus_CE_S11": cmp(&m01_vals, &loss["S11"]),
            "anchor_S01_minus_S11": cmp(&loss["S01"], &loss["S11"]),
            "offset_Qonly_minus_S11": cmp(&loss["Qonly"], &loss["S11"]),
            "S00_minus_S11": cmp(&loss["S00"], &loss["S11"]),
            "S10_minus_S11": cmp(&loss["S10"], &loss["S11"]),
            "Honly_minus_S11": cmp(&loss["Honly"], &loss["S11"]),
            "E_minus_S11": cmp(&loss["E"], &loss["S11"]),
            "E_minus_S01": cmp(&loss["E"], &loss["S01"]),
            "E_minus_Qonly": cmp(&loss["E"], &loss["Qonly"]),
        });
        panels.push(json!({
            "panel": name,
            "targets": recs.len(),
            "older_present_targets": older_mask.iter().filter(|x| **x).count(),
            "m01_matched_targets": matched.iter().filter(|x| **x).count(),
            "identity": {
                "rule": "Z11 + Z00 = Z10 + Z01 per vocabulary row in wide integers",
                "failures": identity_failures,
            },
            "conditions": cond_json,
            "comparisons": comparisons,
        }));
        for cond in ALL_CONDS.iter() {
            write_f64s(
                &root,
                &format!("vectors/{name}-{}.f64", cond.label()),
                &loss[cond.label()],
            )?;
        }
        write_f64s(&root, &format!("vectors/{name}-M01.f64"), &m01_vals)?;
        write_json(
            &root,
            &format!("ids-{name}.json"),
            &json!({
                "q_row": q_ids, "b_row": b_ids,
                "older_present": older_mask,
                "m01_matched": matched,
                "note": "q_row 18446744073709551615 marks an absent-prefix position",
            }),
        )?;
    }

    // ---- matched interventions ----------------------------------------------
    let mut controls: Vec<serde_json::Value> = Vec::new();
    for (name, recs, windows, donor) in [
        ("old", &old_panel.recs, &old_panel.windows, &old_donor),
        ("new", &new_recs, &new_windows, &new_donor),
    ] {
        let own: Vec<Option<(usize, usize)>> = recs
            .iter()
            .map(|r| env.art.inference_rows(&windows[r.win], r.i))
            .collect();
        let donor_rows: Vec<Option<(usize, usize)>> = (0..recs.len())
            .map(|k| {
                if donor.eligible[k] {
                    let d = donor.donor[k];
                    match env.art.older_state(&windows[recs[d].win], recs[d].i) {
                        Some(o) => env.art.rows_from_older(&windows[recs[k].win], recs[k].i, o),
                        None => own[k],
                    }
                } else {
                    own[k]
                }
            })
            .collect();
        let rev: Vec<Option<(usize, usize)>> = recs
            .iter()
            .map(|r| rows_reversed(&env.art, &windows[r.win], r.i))
            .collect();
        let mut s11_own = Vec::with_capacity(recs.len());
        let mut s11_donor = Vec::with_capacity(recs.len());
        let mut s11_rev = Vec::with_capacity(recs.len());
        for (k, r) in recs.iter().enumerate() {
            s11_own.push(parent.bits_one(&env.z_cond(r.prev, r.cur, own[k], Cond::S11), r.target));
            s11_donor.push(parent.bits_one(
                &env.z_cond(r.prev, r.cur, donor_rows[k], Cond::S11),
                r.target,
            ));
            s11_rev.push(parent.bits_one(&env.z_cond(r.prev, r.cur, rev[k], Cond::S11), r.target));
        }
        // Content-independent conditions must not move under either intervention.
        let mut invariance = Vec::new();
        for cond in [Cond::E, Cond::S01, Cond::S00, Cond::Qonly] {
            let mut donor_same = true;
            let mut rev_same = true;
            for (k, r) in recs.iter().enumerate() {
                let a = env.z_cond(r.prev, r.cur, own[k], cond);
                if env.z_cond(r.prev, r.cur, donor_rows[k], cond) != a {
                    donor_same = false;
                }
                if env.z_cond(r.prev, r.cur, rev[k], cond) != a {
                    rev_same = false;
                }
            }
            invariance.push(json!({"condition": cond.label(), "donor_invariant": donor_same, "reversal_invariant": rev_same}));
        }
        // Identity q/b and row-coincidence accounting.
        let e = env.e;
        let ident_q = own.iter().filter(|x| x.map(|r| r.0) == Some(e)).count();
        let ident_b = own.iter().filter(|x| x.map(|r| r.1) == Some(e)).count();
        let coincident = own
            .iter()
            .filter(|x| x.map(|r| r.0 == r.1).unwrap_or(false))
            .count();
        let elig = donor.eligible.clone();
        let changed = (0..recs.len())
            .filter(|&k| donor.eligible[k] && donor_rows[k] != own[k])
            .count();
        let elig_docs: BTreeSet<&str> = recs
            .iter()
            .enumerate()
            .filter(|(k, _)| elig[*k])
            .map(|(_, r)| r.doc.as_str())
            .collect();
        controls.push(json!({
            "panel": name,
            "strata": donor.strata,
            "eligible": elig.iter().filter(|x| **x).count(),
            "changed_reads": changed,
            "excluded_no_history": donor.excluded_no_history,
            "eligible_documents": elig_docs.len(),
            "singleton_strata": if name == "old" {
                strata_stats(recs).1
            } else {
                new_strata.1
            },
            "S11_donor_penalty_eligible": iv(paired_detailed(
                &aggregate(recs, &s11_donor, Some(&elig)),
                &aggregate(recs, &s11_own, Some(&elig)),
                if name == "old" { OLD_DONOR_SEED } else { COMMON_SEED })),
            "S11_reversal_penalty_all": iv(paired_detailed(
                &aggregate(recs, &s11_rev, None),
                &aggregate(recs, &s11_own, None),
                if name == "old" { OLD_DONOR_SEED } else { COMMON_SEED })),
            "invariance": invariance,
            "identity_rows": {"q_equals_identity": ident_q, "b_equals_identity": ident_b, "coincident_q_b": coincident},
        }));
        if name == "new" {
            write_f64s(&root, "vectors/new-S11-own.f64", &s11_own)?;
            write_f64s(&root, "vectors/new-S11-donor.f64", &s11_donor)?;
            write_f64s(&root, "vectors/new-S11-reversed.f64", &s11_rev)?;
        }
    }

    // ---- generation ----------------------------------------------------------
    let mut gen_rows: Vec<serde_json::Value> = Vec::new();
    for cond in ALL_CONDS.iter() {
        let mut outputs = Vec::new();
        let mut certs = Vec::new();
        for (k, p) in LEGACY_PROMPTS.iter().enumerate() {
            let prompt: Vec<u32> = p.to_vec();
            let out = generate_cond(&mut env, &prompt, GEN_TOKENS, *cond);
            let (pair, ring) = cycle_certificates(&prompt, &out);
            certs.push(json!({"prompt_index": k, "pair_cycle": pair, "ring_cycle": ring}));
            outputs
                .push(json!({"prompt_index": k, "output": out, "decoded": tokenizer.decode(&out)}));
        }
        let pair_cert = certs.iter().filter(|c| !c["pair_cycle"].is_null()).count();
        let ring_cert = certs.iter().filter(|c| !c["ring_cycle"].is_null()).count();
        gen_rows.push(json!({
            "condition": cond.label(),
            "history_dependent": cond.takes_older(),
            "pair_cycle_prompts": pair_cert,
            "full_ring_cycle_prompts": ring_cert,
            "certificate_scope": if cond.takes_older() {
                "a repeated local pair cannot certify this condition; the complete bounded ring is the sufficient state"
            } else {
                "content-independent condition; its true state is the local inputs plus the absence mask"
            },
            "certificates": certs,
            "outputs": outputs,
        }));
    }
    // Reproduce S11's retained old generation exactly.
    let s_old = result2["arms"]
        .as_array()
        .ok_or("result2 arms")?
        .iter()
        .find(|a| a["arm"] == "separable_older_query_read")
        .ok_or("result2 S arm")?
        .clone();
    let mut s11_reproduction = true;
    for g in s_old["generation"].as_array().ok_or("result2 generation")? {
        let k = g["prompt_index"].as_u64().unwrap_or(99) as usize;
        if k >= LEGACY_PROMPTS.len() {
            return Err("unexpected prompt index".into());
        }
        let want: Vec<u32> = g["output"]
            .as_array()
            .ok_or("out")?
            .iter()
            .map(|x| x.as_u64().unwrap_or(0) as u32)
            .collect();
        let got = generate_cond(&mut env, &LEGACY_PROMPTS[k].to_vec(), GEN_TOKENS, Cond::S11);
        if got != want {
            s11_reproduction = false;
        }
    }
    gen_rows
        .push(json!({"condition": "S11_old_output_reproduction", "identical": s11_reproduction}));
    if !s11_reproduction {
        return Err("S11 did not reproduce its retained old generation".into());
    }

    // ---- cost ---------------------------------------------------------------
    // Declared identical protocol: one prompt, 64 greedy tokens, one discarded warm-up repeat,
    // then five timed repeats, `black_box` on inputs and outputs, results consumed.
    let mut timings = Vec::new();
    let mut z_cache_entries = 0usize;
    for label in [
        "E_generate",
        "S11_generate_inference_seam",
        "S11_generate_old_read_path",
        "S01_generate_inference_seam",
        "Qonly_generate_inference_seam",
    ] {
        let (cond, use_old_path) = match label {
            "E_generate" => (Cond::E, false),
            "S11_generate_old_read_path" => (Cond::S11, true),
            _ => (Cond::S11, false),
        };
        let cond = if label.starts_with("S01") {
            Cond::S01
        } else if label.starts_with("Qonly") {
            Cond::Qonly
        } else {
            cond
        };
        let prompt: Vec<u32> = LEGACY_PROMPTS[0].to_vec();
        // Discarded warm-up repeat: fills the frozen-parent score cache.
        let _ = generate_cond_with(&mut env, black_box(&prompt), GEN_TOKENS, cond, use_old_path);
        let mut samples = Vec::new();
        for _ in 0..5 {
            let t0 = Instant::now();
            let out =
                generate_cond_with(&mut env, black_box(&prompt), GEN_TOKENS, cond, use_old_path);
            black_box(&out);
            samples.push(t0.elapsed().as_secs_f64());
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        z_cache_entries = env.z_cache.len();
        timings.push(json!({
            "label": label,
            "samples_s": samples,
            "min_s": samples[0],
            "median_s": samples[samples.len() / 2],
            "spread_s": samples[samples.len() - 1] - samples[0],
            "protocol": "one retained prompt, 64 greedy tokens, one discarded warm-up repeat then five timed repeats, black_box on input and output, results consumed, complete numerical path including the frozen parent score",
        }));
    }

    // ---- decision -----------------------------------------------------------
    let np = panels
        .iter()
        .find(|p| p["panel"] == "new")
        .ok_or("new panel")?;
    let op = panels
        .iter()
        .find(|p| p["panel"] == "old")
        .ok_or("old panel")?;
    let ncmp = &np["comparisons"];
    let ocmp = &op["comparisons"];
    let primary = f(&ncmp["primary_CE_M01_minus_CE_S11"]["point"]);
    let primary_lo = f(&ncmp["primary_CE_M01_minus_CE_S11"]["lo"]);
    let donor_point = f(&controls[1]["S11_donor_penalty_eligible"]["point"]);
    let donor_lo = f(&controls[1]["S11_donor_penalty_eligible"]["lo"]);
    let o_primary = f(&ocmp["primary_CE_M01_minus_CE_S11"]["point"]);
    let o_primary_lo = f(&ocmp["primary_CE_M01_minus_CE_S11"]["lo"]);
    let history_lead = primary >= COMPONENT_MARGIN
        && primary_lo > 0.0
        && donor_point >= COMPONENT_MARGIN
        && donor_lo > 0.0
        && !(o_primary <= -COMPONENT_MARGIN && o_primary_lo < 0.0);
    let s01_point = f(&ncmp["anchor_S01_minus_S11"]["point"]);
    let s01_lo = f(&ncmp["anchor_S01_minus_S11"]["lo"]);
    let s01_hi = f(&ncmp["anchor_S01_minus_S11"]["hi"]);
    let local_equiv = s01_point.abs() <= COMPONENT_MARGIN && s01_lo <= 0.0 && s01_hi >= 0.0;
    let classification = if history_lead {
        "S11's improvement is not explained by its local query alone: a bounded positive history lead, warranting a selective-maintenance test with fresh behavioural criteria. Not model promotion."
    } else if local_equiv {
        "S11 is best described as a frozen local query/calibration result at this support; target the state-maintenance problem directly and do not advertise the group product as useful memory."
    } else {
        "inconclusive at this support: identify the smallest missing causal/measurement condition rather than widening strata or sweeping capacity."
    };

    let abs_lines: Vec<String> = ["old", "new"]
        .iter()
        .map(|n| {
            let p = panels.iter().find(|x| x["panel"] == *n).unwrap();
            format!(
                "{n}: identity_failures={} matched={}",
                p["identity"]["failures"], p["m01_matched_targets"]
            )
        })
        .collect();

    write_json(
        &root,
        "result.json",
        &json!({
            "schema": "uor-r4.s-attribution/1",
            "analysis_kind": "evaluation_only",
            "utc_start": utc_now(),
            "base_revision": "8bbdb63d1c686d078292562bb879c0bf4dadde34",
            "running_source": {
                "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
                "git_dirty": std::env::var("UOR_GIT_DIRTY").unwrap_or_else(|_| "unknown".into()),
                "source_files": [
                    "crates/uor-r4-core/src/native_geometric/learner/query_read.rs",
                    "crates/uor-r4-core/src/bin/query-read-attribution.rs",
                ],
                "executable_sha256": executable_sha256,
                "source_file_sha256": source_hashes,
                "old_binary_identity": "UNAVAILABLE — the superseded run bound no executable digest",
            },
            "parent_E": {"path": DEFAULT_PARENT, "sha256": E_SHA, "tokenizer_digest": hex_lower(&raw_tok)},
            "tokenizer": {"source_sha256": TOKENIZER_SHA, "derived_sha256": DERIVED_SHA, "raw_digest": hex_lower(&raw_tok)},
            "repairs": repair_rows,
            "population": {
                "documents": uniq.len(), "collected": collected, "duplicates": duplicates,
                "fit_windows": fit_windows.len(),
                "consumed_fit_windows": consumed_windows.len(),
                "consumed_fit_targets": consumed_targets,
                "old_panel": {"documents": OLD_PANEL_DOCS, "windows": old_panel.windows.len(), "targets": old_panel.recs.len()},
                "new_panel": {
                    "documents": new_docs.iter().collect::<BTreeSet<_>>().len(),
                    "windows": new_windows.len(), "targets": new_recs.len(),
                    "donor_seed": NEW_DONOR_SEED,
                },
                "replication_scope": "new positions in the same open-development documents; not independent documents and not final held-out qualification",
            },
            "reproduced_old_vectors": repro_rows,
            "fit_history_means": {
                "lengths_supported": supported.len(),
                "first_length": supported.first(),
                "last_length": supported.last(),
                "note": "M01 is an offline diagnostic comparator and is never a serving incumbent",
            },
            "panels": panels,
            "controls": controls,
            "generation": gen_rows,
            "cost": {
                "timings": timings,
                "generation_64_tokens_s": "see the S11_generate_inference_seam row of `timings`; the same protocol covers every label",
                "residual_max_abs": RESIDUAL_MAX_ABS,
                "corrected_artifact_bytes": repaired.iter().map(|(k, val)| json!({"arm": k, "bytes": val.to_bytes().len()})).collect::<Vec<_>>(),
                "loaded_parent_bytes": parent_bytes.len(),
                "frozen_parent_cache_entries": z_cache_entries,
                "note": "steady-state inference only; no fit-time cache in these timings. Whole-process RSS is reported by the invoking harness, not here. Physical energy UNAVAILABLE.",
            },
            "decision": {
                "component_margin_bits": COMPONENT_MARGIN,
                "primary": "CE_M01 - CE_S11 on matched older-present positions",
                "primary_new": ncmp["primary_CE_M01_minus_CE_S11"].clone(),
                "primary_old": ocmp["primary_CE_M01_minus_CE_S11"].clone(),
                "conditional_donor_new": controls[1]["S11_donor_penalty_eligible"].clone(),
                "conditional_donor_old": controls[0]["S11_donor_penalty_eligible"].clone(),
                "S01_minus_S11_new": ncmp["anchor_S01_minus_S11"].clone(),
                "Qonly_minus_S11_new": ncmp["offset_Qonly_minus_S11"].clone(),
                "history_lead": history_lead,
                "local_equivalence_band": local_equiv,
                "classification": classification,
            },
            "phases": phase_json(&marks),
            "elapsed_s": started.elapsed().as_secs_f64(),
        }),
    )?;
    write_json(
        &root,
        "generation.json",
        &json!({"rows": gen_rows, "tokens": GEN_TOKENS}),
    )?;
    seal(&root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "{} | primary {:.6} [{:.6},{:.6}] | donor {:.6} [{:.6},{:.6}] | S01-S11 {:.6} [{:.6},{:.6}] | history_lead={} local_equiv={} | sealed {} unlisted | elapsed {:.1}s",
        abs_lines.join(" ; "),
        primary,
        primary_lo,
        f(&ncmp["primary_CE_M01_minus_CE_S11"]["hi"]),
        donor_point,
        donor_lo,
        f(&controls[1]["S11_donor_penalty_eligible"]["hi"]),
        s01_point,
        s01_lo,
        s01_hi,
        history_lead,
        local_equiv,
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

/// The evenly spaced chunk indices the retained panel selection used for a document.
fn spread_indices(len: usize) -> Vec<usize> {
    let count = 8.min(len);
    let mut idxs: Vec<usize> = Vec::new();
    for k in 0..count {
        let idx = if count == 1 {
            0
        } else {
            (k * (len - 1) + (count - 1) / 2) / (count - 1)
        };
        if !idxs.contains(&idx) {
            idxs.push(idx);
        }
    }
    idxs
}

/// `(strata, singleton strata, no-history positions)` using the same key as `build_perm`.
fn strata_stats(recs: &[Rec]) -> (usize, usize, usize) {
    let mut groups: BTreeMap<(usize, usize, usize), usize> = BTreeMap::new();
    for r in recs {
        *groups.entry((r.prev, r.cur, r.older_len)).or_insert(0) += 1;
    }
    let mut singletons = 0usize;
    let mut no_history = 0usize;
    for ((_, _, len), n) in groups.iter() {
        if *len == 0 {
            no_history += *n;
        } else if *n < 2 {
            singletons += 1;
        }
    }
    (groups.len(), singletons, no_history)
}

/// Convert cumulative marks into non-overlapping phase durations.
fn phase_json(marks: &[(&str, f64)]) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    let mut prev = 0.0f64;
    for (name, t) in marks {
        out.push(json!({"phase": name, "cumulative_s": t, "seconds": t - prev}));
        prev = *t;
    }
    out
}

fn utc_now() -> String {
    // Seconds since the Unix epoch, formatted without a timezone crate.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("epoch+{secs}s")
}
