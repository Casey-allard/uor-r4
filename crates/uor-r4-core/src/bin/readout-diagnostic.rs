//! Paired output-head diagnostic: empirical versus smoothed conditional targets.
//!
//! Freezes the parent CPL2 embeddings, integer features, normalization, bias and `F`, and trains
//! **only** the existing V4096x128 ternary output head on two matched target distributions over the
//! recovered 4,096-window population:
//!
//! * Arm E: the empirical conditional histogram `p_emp(v|c) = C2[c,v]/N_c`, whose occurrence-weighted
//!   soft loss is exactly the token-CE population objective; and
//! * Arm S: the existing normalized interpolated count family `q(v|c)`.
//!
//! Both arms use identical initialization, order, dose, optimizer and inference parameter budget.
//! The conditional floating relaxation runs only under the prompt's declared branch and is
//! offline-only.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use uor_r4_core::native_geometric::learner::lowbit::TernaryLinear;
use uor_r4_core::native_geometric::learner::prior_learning::{
    bias_codes_from_counts, targets, Config, PriorCore,
};
use uor_r4_core::native_geometric::learner::realtext_support::*;
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::bpe_derive::{derive_tokenizer, derive_tokenizer_json};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json";
const EXPECTED_TOKENIZER_SHA256: &str =
    "9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c";
const EXPECTED_ARTIFACT_SHA256: &str =
    "cd5a3aa1b804ccc3582a4a25090e25c862489e0bc4e5bb86c1d1f363a7338a00";
const UPDATES: usize = 512;
const BATCH: usize = 8;
const WARMUP_PROBE: usize = 8;
const SEED: u64 = 13;
const LR: f32 = 0.05;
const CLIP: f32 = 1.0;
const BETA1: f32 = 0.9;
const BETA2: f32 = 0.999;
const EPS: f32 = 1e-8;
const SCREEN_MARGIN: f64 = 0.10;
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

struct Args {
    root: PathBuf,
    artifact: PathBuf,
    checkpoint: PathBuf,
    docs: PathBuf,
    tokenizer: PathBuf,
    source_rev: String,
    evaluator_rev: String,
    probe: bool,
    resume_test: bool,
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
    let mut resume_test = false;
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
            "--resume-test" => resume_test = true,
            other => return Err(format!("unknown argument {other}")),
        }
        i += if k.starts_with("--probe") || k.starts_with("--resume-test") {
            1
        } else {
            2
        };
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
        resume_test,
    })
}

fn write_checked(root: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {name}: {e}"))?;
    }
    std::fs::write(&path, bytes).map_err(|e| format!("write {name}: {e}"))
}
fn write_json(root: &Path, name: &str, v: &Value) -> Result<(), String> {
    let s = serde_json::to_string_pretty(v).map_err(|e| format!("encode {name}: {e}"))?;
    write_checked(root, name, s.as_bytes())
}
fn write_f64s(root: &Path, name: &str, xs: &[f64]) -> Result<(), String> {
    let mut b = Vec::with_capacity(xs.len() * 8);
    for x in xs {
        b.extend_from_slice(&x.to_le_bytes());
    }
    write_checked(root, name, &b)
}
fn xorshift(st: &mut u64) -> u64 {
    *st ^= *st << 13;
    *st ^= *st >> 7;
    *st ^= *st << 17;
    *st
}

/// The legacy CPCK output masters (block index 2), for verified initialization.
fn cpc_k_output_masters(bytes: &[u8], vocab: usize, dv: usize) -> Result<Vec<f32>, String> {
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
    for _ in 0..perm_len {
        let _ = u64_at(&mut c)?;
    }
    let wants = [
        (vocab + 1) * dv,
        vocab * dv,
        vocab * dv,
        (vocab + 1) * dv,
        vocab * dv,
        vocab * dv,
        (vocab + 1) * dv,
        vocab * dv,
        vocab * dv,
    ];
    for (k, want) in wants.iter().enumerate() {
        let len = u64_at(&mut c)? as usize;
        if len != *want {
            return Err(format!("CPCK block {k} length {len} != {want}"));
        }
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            let x = f32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
            if !x.is_finite() {
                return Err("CPCK holds a non-finite value".into());
            }
            v.push(x);
        }
        if k == 2 {
            return Ok(v);
        }
    }
    Err("CPCK output masters missing".into())
}

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

/// Frozen parent features for one context: normalized hidden `h` and the norm shift.
#[derive(Clone)]
struct Feat {
    h: Vec<i32>,
}

/// Identity that a readout checkpoint must bind and that a resume must not adopt wrongly.
#[derive(Clone, Debug, PartialEq, Eq)]
struct HeadMeta {
    parent_digest: [u8; 32],
    data_digest: [u8; 32],
    /// 0 = empirical, 1 = smoothed.
    teacher: u8,
    lambdas: (u64, u64),
    seed: u64,
    updates: usize,
    batch: usize,
}

/// The output-only head: masters, moments, and the served hard forward.
struct Head {
    vocab: usize,
    dv: usize,
    f_bits: u32,
    bias_codes: Vec<i8>,
    bias_scale_bits: u32,
    master: Vec<f32>,
    m: Vec<f32>,
    v: Vec<f32>,
    age: u64,
    cursor: usize,
    meta: HeadMeta,
}

const RDO_MAGIC: &[u8; 4] = b"RDO1";

impl Head {
    fn checkpoint_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(RDO_MAGIC);
        o.extend_from_slice(&1u32.to_le_bytes());
        o.extend_from_slice(&(self.vocab as u32).to_le_bytes());
        o.extend_from_slice(&(self.dv as u32).to_le_bytes());
        o.extend_from_slice(&self.f_bits.to_le_bytes());
        o.extend_from_slice(&self.bias_scale_bits.to_le_bytes());
        o.extend_from_slice(&self.meta.parent_digest);
        o.extend_from_slice(&self.meta.data_digest);
        o.push(self.meta.teacher);
        o.extend_from_slice(&self.meta.lambdas.0.to_le_bytes());
        o.extend_from_slice(&self.meta.lambdas.1.to_le_bytes());
        o.extend_from_slice(&self.meta.seed.to_le_bytes());
        o.extend_from_slice(&(self.meta.updates as u64).to_le_bytes());
        o.extend_from_slice(&(self.meta.batch as u64).to_le_bytes());
        o.extend_from_slice(&self.age.to_le_bytes());
        o.extend_from_slice(&(self.cursor as u64).to_le_bytes());
        for block in [&self.master, &self.m, &self.v] {
            o.extend_from_slice(&(block.len() as u64).to_le_bytes());
            for f in block.iter() {
                o.extend_from_slice(&f.to_le_bytes());
            }
        }
        o
    }

    /// Load a checkpoint, rejecting any identity mismatch before mutating anything.
    fn resume_from(&mut self, bytes: &[u8], expected: &HeadMeta) -> Result<(), String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated readout checkpoint".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != RDO_MAGIC {
            return Err("bad readout checkpoint magic".into());
        }
        let u32_at = |c: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        let u64_at = |c: &mut usize| -> Result<u64, String> {
            Ok(u64::from_le_bytes(take(c, 8)?.try_into().unwrap()))
        };
        if u32_at(&mut c)? != 1 {
            return Err("unsupported readout checkpoint version".into());
        }
        let vocab = u32_at(&mut c)? as usize;
        let dv = u32_at(&mut c)? as usize;
        let f_bits = u32_at(&mut c)?;
        let bias_scale_bits = u32_at(&mut c)?;
        let mut parent_digest = [0u8; 32];
        parent_digest.copy_from_slice(take(&mut c, 32)?);
        let mut data_digest = [0u8; 32];
        data_digest.copy_from_slice(take(&mut c, 32)?);
        let teacher = take(&mut c, 1)?[0];
        let l0 = u64_at(&mut c)?;
        let l1 = u64_at(&mut c)?;
        let seed = u64_at(&mut c)?;
        let updates = u64_at(&mut c)? as usize;
        let batch = u64_at(&mut c)? as usize;
        let age = u64_at(&mut c)?;
        let cursor = u64_at(&mut c)? as usize;
        let mut blocks: Vec<Vec<f32>> = Vec::new();
        for want in [vocab * dv, vocab * dv, vocab * dv] {
            let len = u64_at(&mut c)? as usize;
            if len != want {
                return Err("readout checkpoint block length differs".into());
            }
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                let x = f32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
                if !x.is_finite() {
                    return Err("readout checkpoint holds a non-finite value".into());
                }
                v.push(x);
            }
            blocks.push(v);
        }
        if c != bytes.len() {
            return Err(format!(
                "{} trailing readout checkpoint bytes",
                bytes.len() - c
            ));
        }
        let got = HeadMeta {
            parent_digest,
            data_digest,
            teacher,
            lambdas: (l0, l1),
            seed,
            updates,
            batch,
        };
        if vocab != self.vocab
            || dv != self.dv
            || f_bits != self.f_bits
            || bias_scale_bits != self.bias_scale_bits
            || got != *expected
        {
            return Err("readout checkpoint identity/config differs; refusing to adopt it".into());
        }
        // Only the second moment must be non-negative; the first moment is signed.
        if blocks[2].iter().any(|x| *x < 0.0) {
            return Err("readout checkpoint second moment is invalid".into());
        }
        let mut it = blocks.into_iter();
        self.master = it.next().unwrap();
        self.m = it.next().unwrap();
        self.v = it.next().unwrap();
        self.age = age;
        self.cursor = cursor;
        self.meta = got;
        Ok(())
    }
}

impl Head {
    fn quantized(&self) -> TernaryLinear {
        TernaryLinear::quantize(&self.master, self.vocab, self.dv)
    }

    /// Hard integer logits for frozen features: `Z[v] = bias[v] + (sum_j code*h[j]) << shift[v]`.
    fn logits(&self, q: &TernaryLinear, h: &[i32]) -> Vec<i32> {
        let acc = q.forward_i32(h);
        (0..self.vocab)
            .map(|v| ((self.bias_codes[v] as i32) << self.bias_scale_bits) + acc[v])
            .collect()
    }
}

fn softmax_scaled(z: &[i32], scale: f64) -> Vec<f64> {
    let mut max = f64::NEG_INFINITY;
    for &v in z {
        max = max.max(v as f64 * scale);
    }
    let mut p = Vec::with_capacity(z.len());
    let mut sum = 0.0;
    for &v in z {
        let e = (v as f64 * scale - max).exp();
        p.push(e);
        sum += e;
    }
    for x in p.iter_mut() {
        *x /= sum;
    }
    p
}

/// One teacher distribution for a context.
enum Teacher {
    /// Sparse empirical histogram `(successor, count)` with total `n`.
    Empirical { pairs: Vec<(u32, u32)>, n: f64 },
    /// Dense interpolated family.
    Smoothed(Vec<f64>),
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
    if artifact_sha != EXPECTED_ARTIFACT_SHA256 {
        return Err(format!("artifact sha {artifact_sha} != pinned"));
    }
    let parent = PriorCore::from_bytes(&artifact_bytes).map_err(|e| format!("load parent: {e}"))?;
    parent.validate()?;
    let ckpt_bytes = std::fs::read(&args.checkpoint).map_err(|e| format!("checkpoint: {e}"))?;
    let ckpt_sha = sha256_hex(&ckpt_bytes);

    let tok_bytes = std::fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    if sha256_hex(&tok_bytes) != EXPECTED_TOKENIZER_SHA256 {
        return Err("tokenizer sha mismatch".into());
    }
    let tokenizer: HfBpeTokenizer =
        derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive tokenizer: {e}"))?;
    let derived_sha = sha256_hex(
        &derive_tokenizer_json(&tok_bytes, VOCAB).map_err(|e| format!("derive json: {e}"))?,
    );

    // --- corpus, fit windows, recovered 4,096-window population -----------------
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

    let mut fit_windows: Vec<Vec<u32>> = Vec::new();
    for i in &fit_ids {
        fit_windows.extend(windows_of(&tokenizer.encode(&uniq[*i].text)));
    }
    let perm = cpc_k_perm(&ckpt_bytes)?;
    if perm.len() != fit_windows.len() {
        return Err("checkpoint permutation length != fit windows".into());
    }
    let consumed: Vec<usize> = perm[..4096.min(perm.len())].to_vec();
    let windows: Vec<Vec<u32>> = consumed.iter().map(|&w| fit_windows[w].clone()).collect();
    let mut occ: Vec<(usize, usize, usize, usize)> = Vec::new(); // (win, i, prev, cur)
    let mut occ_target: Vec<u32> = Vec::new();
    for (wi, w) in windows.iter().enumerate() {
        for (i, prev, cur, target) in targets(w, VOCAB) {
            occ.push((wi, i, prev, cur));
            occ_target.push(target);
        }
    }
    let n_occ = occ.len();
    println!(
        "corpus collected={collected} eligible={} duplicates={duplicates} fit_windows={} consumed=4096 population_targets={n_occ}",
        uniq.len(),
        fit_windows.len()
    );

    // Shared full-fit add-one unigram marginal.
    let mut counts = vec![0u64; VOCAB];
    let mut total = 0u64;
    for w in &fit_windows {
        for (_i, _p, _c, t) in targets(w, VOCAB) {
            counts[t as usize] += 1;
            total += 1;
        }
    }
    let uni = Uni { counts, total };
    let bias = bias_codes_from_counts(&uni.counts, total, VOCAB);
    let bias_matches = bias == parent.bias_codes;

    // Conditional counts from THESE occurrences only.
    let mut c2: HashMap<u64, Vec<(u32, u32)>> = HashMap::new();
    let mut c1: HashMap<u64, Vec<(u32, u32)>> = HashMap::new();
    for (k, &(_wi, _i, prev, cur)) in occ.iter().enumerate() {
        let t = occ_target[k];
        let e2 = c2.entry(ctx2(prev, cur)).or_default();
        match e2.iter_mut().find(|(s, _)| *s == t) {
            Some((_, c)) => *c += 1,
            None => e2.push((t, 1)),
        }
        let e1 = c1.entry(cur as u64).or_default();
        match e1.iter_mut().find(|(s, _)| *s == t) {
            Some((_, c)) => *c += 1,
            None => e1.push((t, 1)),
        }
    }

    // Saved or reconstructed lambdas from the sealed pilot references (full-fit family).
    let lambdas = (0.8f64, 0.7f64);
    let lambda_src = "sealed prefix-pilot-2 references.json full_fit entry (0.8, 0.7)";

    // --- schedule: one pass over 4096 windows, saved seed13 Fisher-Yates ---------
    let mut st = SEED ^ 0xC0FF_EE00_1234_5678;
    let mut order: Vec<usize> = (0..windows.len()).collect();
    for i in (1..order.len()).rev() {
        let j = (xorshift(&mut st) as usize) % (i + 1);
        order.swap(i, j);
    }
    if UPDATES * BATCH != windows.len() {
        return Err("512 updates x batch 8 must equal the 4096-window population".into());
    }
    // Occurrence ranges per window (contiguous by construction).
    let mut win_start = vec![0usize; windows.len() + 1];
    {
        let mut k = 0usize;
        for (wi, w) in windows.iter().enumerate() {
            win_start[wi] = k;
            k += w.len().saturating_sub(1);
        }
        win_start[windows.len()] = k;
    }

    // --- verified initialization -------------------------------------------------
    let wo_masters = cpc_k_output_masters(&ckpt_bytes, VOCAB, parent.cfg.dv)?;
    let quant_inits = TernaryLinear::quantize(&wo_masters, VOCAB, parent.cfg.dv);
    let mut repro = quant_inits.rows == parent.w_o.rows
        && quant_inits.cols == parent.w_o.cols
        && quant_inits.packed() == parent.w_o.packed();
    if repro {
        for r in 0..VOCAB {
            if quant_inits.shift(r) != parent.w_o.shift(r) {
                repro = false;
            }
        }
    }
    let (init_master, init_src) = if repro {
        (wo_masters.clone(), "verified legacy CPCK output masters")
    } else {
        // Declared fallback: 1.25 * code * 2^shift for nonzero codes.
        let mut m = vec![0.0f32; VOCAB * parent.cfg.dv];
        for v in 0..VOCAB {
            let s = parent.w_o.shift(v);
            for j in 0..parent.cfg.dv {
                let code = parent.w_o.weight(v, j);
                if code != 0 {
                    m[v * parent.cfg.dv + j] = 1.25 * (code as f32) * ((1u32 << s) as f32);
                }
            }
        }
        (
            m,
            "declared fallback 1.25*code*2^shift (CPCK masters did not reproduce the parent head)",
        )
    };
    println!("initialization: {init_src} (verified={repro})");

    // Frozen feature cache.
    let mut feat: HashMap<(usize, usize), Feat> = HashMap::new();
    for &(_wi, _i, prev, cur) in occ.iter() {
        feat.entry((prev, cur)).or_insert_with(|| Feat {
            h: parent.trace(prev, cur, true).h,
        });
    }

    let data_digest: [u8; 32] = Sha256::digest(
        windows
            .iter()
            .flat_map(|w| w.iter().copied())
            .flat_map(|t| t.to_le_bytes())
            .chain(consumed.iter().flat_map(|w| (*w as u64).to_le_bytes()))
            .collect::<Vec<u8>>(),
    )
    .into();
    let parent_digest: [u8; 32] = Sha256::digest(&artifact_bytes).into();
    let meta_for = |teacher: u8| HeadMeta {
        parent_digest,
        data_digest,
        teacher,
        lambdas: (lambdas.0.to_bits(), lambdas.1.to_bits()),
        seed: SEED,
        updates: UPDATES,
        batch: BATCH,
    };
    let mut arms: Vec<(&str, Head)> = Vec::new();
    for (idx, name) in ["empirical", "smoothed"].iter().enumerate() {
        arms.push((
            name,
            Head {
                vocab: VOCAB,
                dv: parent.cfg.dv,
                f_bits: parent.cfg.f_bits,
                bias_codes: parent.bias_codes.clone(),
                bias_scale_bits: parent.cfg.bias_scale_bits,
                master: init_master.clone(),
                m: vec![0.0; init_master.len()],
                v: vec![0.0; init_master.len()],
                age: 0,
                cursor: 0,
                meta: meta_for(idx as u8),
            },
        ));
    }

    // Step-zero parity: the hard head must reproduce the parent's integer logits exactly.
    for (name, head) in arms.iter() {
        let q = head.quantized();
        let mut bad = 0usize;
        for &(prev, cur) in feat.keys().take(256) {
            let z = head.logits(&q, &feat[&(prev, cur)].h);
            if z != parent.int_logits(prev, cur, true) {
                bad += 1;
            }
        }
        if bad != 0 {
            return Err(format!(
                "{name}: step-zero parity failed for {bad} contexts"
            ));
        }
    }
    println!("step-zero parity: both arms reproduce the parent integer logits exactly");

    // Teacher validation before any fitting: complete mass, non-negativity, finiteness.
    {
        let mut checked = 0usize;
        for &(prev, cur) in feat.keys().take(2048) {
            let ctx = ctx2(prev, cur);
            for smoothed in [false, true] {
                let t = teacher(
                    &c2, &c1, &uni, ctx, cur as u64, prev, cur, lambdas, smoothed,
                );
                if !teacher_is_valid(&t, 1e-9) {
                    return Err(format!("teacher mass/non-negativity failure at ctx {ctx}"));
                }
                checked += 1;
            }
        }
        println!("teacher validation: {checked} distributions checked, mass within 1e-9 of 1");
    }

    if args.resume_test {
        return resume_test(
            &args,
            &parent,
            &init_master,
            &windows,
            &occ,
            &occ_target,
            &win_start,
            &order,
            &feat,
            &c2,
            &c1,
            &uni,
            lambdas,
            meta_for(0),
            meta_for(1),
        );
    }

    // --- probe ----------------------------------------------------------------
    if args.probe {
        let t0 = Instant::now();
        let mut head = Head {
            vocab: VOCAB,
            dv: parent.cfg.dv,
            f_bits: parent.cfg.f_bits,
            bias_codes: parent.bias_codes.clone(),
            bias_scale_bits: parent.cfg.bias_scale_bits,
            master: init_master.clone(),
            m: vec![0.0; init_master.len()],
            v: vec![0.0; init_master.len()],
            age: 0,
            cursor: 0,
            meta: meta_for(0),
        };
        for u in 0..WARMUP_PROBE {
            let ws: Vec<usize> = order[u * BATCH..(u + 1) * BATCH].to_vec();
            let loss = update(
                &mut head,
                &ws,
                &windows,
                &occ,
                &occ_target,
                &win_start,
                &feat,
                &c2,
                &c1,
                &uni,
                lambdas,
                true,
            )?;
            println!("probe update {u}: soft loss {loss:.4} bits/occurrence");
        }
        println!(
            "probe: {WARMUP_PROBE} updates in {:.2}s (probe state discarded and charged)",
            t0.elapsed().as_secs_f64()
        );
        seal(&args.root).map_err(|e| format!("seal: {e}"))?;
        verify(&args.root).map_err(|e| format!("verify: {e}"))?;
        return Ok(ExitCode::SUCCESS);
    }

    // --- the two matched hard fits ---------------------------------------------
    let dev_panel = build_panel(&uniq, &dev_pool, &tokenizer, 8);
    let tune_windows: Vec<Vec<u32>> = tune_ids
        .iter()
        .flat_map(|i| {
            let ws = windows_of(&tokenizer.encode(&uniq[*i].text));
            let mut v = Vec::new();
            if !ws.is_empty() {
                v.push(ws[0].clone());
                if ws.len() >= 2 {
                    v.push(ws[ws.len() - 1].clone());
                }
            }
            v
        })
        .take(76)
        .collect();
    let mut train_windows: Vec<Vec<u32>> = Vec::new();
    for i in &fit_ids {
        train_windows.extend(windows_of(&tokenizer.encode(&uniq[*i].text)));
    }
    let _ = train_windows;

    let parent_dev_loss = eval_true_label(&parent, &dev_panel);
    let a_parent_dev = aggregate(&dev_panel.recs, &parent_dev_loss, None);

    let mut arm_report = Vec::new();
    let mut gen_rows = Vec::new();
    let full_fit_kl: Vec<f64>;
    {
        let mut kl = Vec::new();
        for (name, head) in arms.iter_mut() {
            let t0 = Instant::now();
            let mut curve = Vec::new();
            let mut last_loss = f64::NAN;
            for u in 0..UPDATES {
                let ws: Vec<usize> = order[u * BATCH..(u + 1) * BATCH].to_vec();
                last_loss = update(
                    head,
                    &ws,
                    &windows,
                    &occ,
                    &occ_target,
                    &win_start,
                    &feat,
                    &c2,
                    &c1,
                    &uni,
                    lambdas,
                    *name == "smoothed",
                )?;
                if u + 1 == 128 || u + 1 == 256 || u + 1 == UPDATES {
                    curve.push(json!({"update": u + 1, "online_soft_loss": last_loss}));
                }
            }
            let q = head.quantized();
            // Objective on the fixed fit population: occurrence-weighted teacher KL/soft CE.
            let mut obj_bits = 0.0f64;
            let mut n_obj = 0usize;
            let mut kl_sum = 0.0f64;
            for (k, &(_wi, _i, prev, cur)) in occ.iter().enumerate() {
                let ctx = ctx2(prev, cur);
                let z = head.logits(&q, &feat[&(prev, cur)].h);
                let p = softmax_scaled(&z, (-(head.f_bits as f64)).exp2());
                let t = teacher(
                    &c2,
                    &c1,
                    &uni,
                    ctx,
                    cur as u64,
                    prev,
                    cur,
                    lambdas,
                    *name == "smoothed",
                );
                obj_bits += teacher_bits(&t, &p);
                kl_sum += teacher_kl(&t, &p);
                n_obj += 1;
                let _ = occ_target[k];
            }
            let soft_ce = obj_bits / n_obj as f64;
            let kl_bits = kl_sum / n_obj as f64;
            kl.push(kl_bits);
            let dev = eval_true_label_core(&parent, &dev_panel, |h| head.logits(&q, h));
            let a_dev = aggregate(&dev_panel.recs, &dev, None);
            let panel_gain = paired_interval(&a_parent_dev, &a_dev, 0x1234_5678);
            let fit = eval_true_label_core(&parent, &dev_panel, |h| head.logits(&q, h));
            let _ = fit;
            // Row scale and ternary-code changes vs the parent head.
            let mut shift_changes = 0usize;
            let mut code_changes = 0usize;
            for v in 0..VOCAB {
                if q.shift(v) != parent.w_o.shift(v) {
                    shift_changes += 1;
                }
                for j in 0..parent.cfg.dv {
                    if q.weight(v, j) != parent.w_o.weight(v, j) {
                        code_changes += 1;
                    }
                }
            }
            let art = PriorCore {
                cfg: Config {
                    vocab: VOCAB,
                    dv: parent.cfg.dv,
                    norm_bits: parent.cfg.norm_bits,
                    f_bits: parent.cfg.f_bits,
                    bias_scale_bits: parent.cfg.bias_scale_bits,
                },
                elements: parent.elements.clone(),
                e_old: parent.e_old.clone(),
                e_new: parent.e_new.clone(),
                w_o: q.clone(),
                bias_codes: parent.bias_codes.clone(),
            };
            let bytes = art.to_bytes(&[7u8; 32], SEED);
            write_checked(&args.root, &format!("hard/{name}.cpl2"), &bytes)?;
            let reloaded =
                PriorCore::from_bytes(&bytes).map_err(|e| format!("{name}: reload: {e}"))?;
            if reloaded.w_o.packed() != q.packed() || reloaded.validate().is_err() {
                return Err(format!("{name}: reloaded export differs"));
            }
            arm_report.push(json!({
                "arm": name,
                "target": if *name == "empirical" { "p_emp(v|c)=C2[c,v]/N_c" } else { "interpolated count family q(v|c)" },
                "online_soft_loss_last": last_loss,
                "fit_objective_bits_per_occurrence": soft_ce,
                "fit_teacher_kl_bits_per_occurrence": kl_bits,
                "curve": curve,
                "dev_micro_bits": a_dev.micro(),
                "dev_document_macro_bits": a_dev.macro_bits(),
                "gain_vs_parent": {"point": panel_gain.0, "lo": panel_gain.1, "hi": panel_gain.2},
                "output_row_shift_changes": shift_changes,
                "ternary_code_changes": code_changes,
                "cpl2": {"path": format!("hard/{name}.cpl2"), "sha256": sha256_hex(&bytes), "bytes": bytes.len()},
                "elapsed_s": t0.elapsed().as_secs_f64(),
            }));
            for (pi, prompt) in LEGACY_PROMPTS.iter().enumerate() {
                let out = reloaded.generate(prompt, GEN_TOKENS, true);
                let (pair, ring) = cycle_certificates(prompt, &out);
                gen_rows.push(json!({
                    "prompt_index": pi, "model": format!("hard_{name}"),
                    "output": out, "decoded": tokenizer.decode(&out),
                    "pair_state_cycle": pair.map(|(e, p)| json!({"entry": e, "period": p})),
                    "ring_state_cycle": ring.map(|(e, p)| json!({"entry": e, "period": p})),
                }));
            }
            write_f64s(&args.root, &format!("vectors/{name}-dev.f64"), &dev)?;
            write_checked(
                &args.root,
                &format!("hard/{name}.ckpt"),
                &head.checkpoint_bytes(),
            )?;
            let _ = tune_windows;
        }
        full_fit_kl = kl;
    }

    // Parent generation + corrected full-vocabulary count reference.
    let mut full1 = Cond::default();
    let mut full2 = Cond::default();
    for w in &fit_windows {
        for (_i, p, c, t) in targets(w, VOCAB) {
            full1.observe(c as u64, t);
            full2.observe(ctx2(p, c), t);
        }
    }
    for (pi, prompt) in LEGACY_PROMPTS.iter().enumerate() {
        let out = parent.generate(prompt, GEN_TOKENS, true);
        let (pair, ring) = cycle_certificates(prompt, &out);
        gen_rows.push(json!({
            "prompt_index": pi, "model": "frozen_parent",
            "output": out, "decoded": tokenizer.decode(&out),
            "pair_state_cycle": pair.map(|(e, p)| json!({"entry": e, "period": p})),
            "ring_state_cycle": ring.map(|(e, p)| json!({"entry": e, "period": p})),
        }));
        let rout = generate_reference_full(&full1, &full2, &uni, lambdas, prompt, GEN_TOKENS);
        gen_rows.push(json!({
            "prompt_index": pi, "model": "count_reference_full_vocabulary",
            "output": rout, "decoded": tokenizer.decode(&rout),
            "note": "argmax over all 4096 entries with lowest-ID ties",
        }));
    }

    // --- screen and conditional floating branch --------------------------------
    let emp = arm_report
        .iter()
        .find(|a| a["arm"].as_str() == Some("empirical"))
        .cloned()
        .unwrap();
    let smo = arm_report
        .iter()
        .find(|a| a["arm"].as_str() == Some("smoothed"))
        .cloned()
        .unwrap();
    let emp_gain = (
        emp["gain_vs_parent"]["point"].as_f64().unwrap(),
        emp["gain_vs_parent"]["lo"].as_f64().unwrap(),
    );
    let smo_gain = (
        smo["gain_vs_parent"]["point"].as_f64().unwrap(),
        smo["gain_vs_parent"]["lo"].as_f64().unwrap(),
    );
    let best = if emp_gain.0 >= smo_gain.0 {
        emp_gain
    } else {
        smo_gain
    };
    let primary_ok = best.0 >= SCREEN_MARGIN && best.1 > 0.0;
    let smoothed_better = smo_gain.0 > emp_gain.0 && smo_gain.1 > 0.0;
    let teacher_gap = full_fit_kl.iter().fold(f64::NEG_INFINITY, |a, b| a.max(*b));
    let float_branch = !(primary_ok && smoothed_better) && teacher_gap > 0.10;
    let float_report = if float_branch {
        let t0 = Instant::now();
        let mut w: Vec<f32> = vec![0.0; VOCAB * parent.cfg.dv];
        for v in 0..VOCAB {
            let s = parent.w_o.shift(v);
            for j in 0..parent.cfg.dv {
                let code = parent.w_o.weight(v, j);
                w[v * parent.cfg.dv + j] = (code as f32) * ((1u32 << s) as f32);
            }
        }
        let mut m = vec![0.0f32; w.len()];
        let mut vv = vec![0.0f32; w.len()];
        let mut age = 0u64;
        for u in 0..UPDATES {
            let ws: Vec<usize> = order[u * BATCH..(u + 1) * BATCH].to_vec();
            float_update(
                &mut w, &mut m, &mut vv, &mut age, &ws, &windows, &occ, &win_start, &feat, &c2,
                &c1, &uni, lambdas, &parent,
            )?;
        }
        // Evaluate the floating head on the same soft objective and true-label populations.
        let mut obj = 0.0f64;
        for &(_wi, _i, prev, cur) in occ.iter() {
            let ctx = ctx2(prev, cur);
            let h = &feat[&(prev, cur)].h;
            let z = float_z(&w, h, &parent);
            let p = softmax_f64_scaled(&z, (-(parent.cfg.f_bits as f64)).exp2());
            let t = teacher(&c2, &c1, &uni, ctx, cur as u64, prev, cur, lambdas, true);
            obj += teacher_bits(&t, &p);
        }
        let dev = eval_true_label_float(&parent, &dev_panel, &w);
        let a_dev = aggregate(&dev_panel.recs, &dev, None);
        let gain = paired_interval(&a_parent_dev, &a_dev, 0x1234_5678);
        write_f64s(&args.root, "vectors/float-dev.f64", &dev)?;
        write_checked(&args.root, "float-head.bin", &{
            let mut b = Vec::with_capacity(w.len() * 4);
            for x in &w {
                b.extend_from_slice(&x.to_le_bytes());
            }
            b
        })?;
        json!({
            "status": "RUN",
            "objective_bits_per_occurrence": obj / n_occ as f64,
            "dev_micro_bits": a_dev.micro(),
            "gain_vs_parent": {"point": gain.0, "lo": gain.1, "hi": gain.2},
            "diagnostic_only": true,
            "never_a_serving_path": true,
            "elapsed_s": t0.elapsed().as_secs_f64(),
        })
    } else {
        json!({
            "status": "NOT_RUN",
            "branch_condition": "Arm S must fail the combined screen AND the final fit teacher KL must exceed 0.10 bits/target",
            "primary_ok": primary_ok,
            "smoothed_better_than_empirical": smoothed_better,
            "max_fit_teacher_kl_bits": teacher_gap,
        })
    };

    // --- reports ---------------------------------------------------------------
    let exe_sha = std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::read(p).ok())
        .map(|b| sha256_hex(&b))
        .unwrap_or_else(|| "UNAVAILABLE".into());
    let panel_vectors: Vec<Value> = ["parent", "empirical", "smoothed", "float"]
        .iter()
        .filter(|n| args.root.join(format!("vectors/{n}-dev.f64")).exists())
        .map(|n| json!({"name": n, "path": format!("vectors/{n}-dev.f64")}))
        .collect();

    write_json(
        &args.root,
        "result.json",
        &json!({
            "schema": "uor-r4.readout-diagnostic/1",
            "source_rev": args.source_rev,
            "evaluator": {"binary": "readout-diagnostic", "git_rev": args.evaluator_rev, "binary_sha256": exe_sha},
            "parent": {"path": args.artifact, "sha256": artifact_sha, "matches_pinned": artifact_sha == EXPECTED_ARTIFACT_SHA256},
            "checkpoint": {"path": args.checkpoint, "sha256": ckpt_sha},
            "tokenizer": {"source_sha256": EXPECTED_TOKENIZER_SHA256, "derived_sha256": derived_sha},
            "corpus": {"git_commit": args.source_rev, "collected": collected, "eligible": uniq.len(), "duplicates": duplicates, "fit_docs": fit_ids.len(), "tune_docs": tune_ids.len(), "dev_pool": dev_pool.len()},
            "population": {"windows": windows.len(), "targets": n_occ, "source": "recovered first 4096 consumed windows from the legacy permutation"},
            "frozen": {"e_old": true, "e_new": true, "norm_and_shift": true, "bias_codes_and_scale": true, "f_bits": parent.cfg.f_bits, "trained_component": "w_o only"},
            "initialization": {"source": init_src, "verified_cpcK_reproduces_parent_head": repro},
            "optimizer": {"adam_beta1": BETA1, "adam_beta2": BETA2, "eps": EPS, "lr": LR, "grad_clip": CLIP, "weight_decay": 0.0, "updates_per_arm": UPDATES, "batch": BATCH, "schedule": "one saved seed13 Fisher-Yates pass over the 4096 window IDs"},
            "lambdas": [lambdas.0, lambdas.1],
            "lambda_source": lambda_src,
            "frozen_bias_matches_full_fit_unigram": bias_matches,
            "parent_dev_micro_bits": a_parent_dev.micro(),
            "arms": arm_report,
            "float_head": float_report,
            "screen": {
                "criterion": ">=0.10 bits/target gain over the frozen parent with paired lower bound > 0",
                "best_hard_arm": if emp_gain.0 >= smo_gain.0 { "empirical" } else { "smoothed" },
                "best_gain_vs_parent": {"point": best.0, "lo": best.1},
                "primary_ok": primary_ok,
                "smoothed_better_than_empirical": smoothed_better,
                "empirical_gain": {"point": emp_gain.0, "lo": emp_gain.1},
                "smoothed_gain": {"point": smo_gain.0, "lo": smo_gain.1},
            },
            "generation": gen_rows,
            "dev_panel": {"documents": dev_panel.docs.iter().filter(|(_, _, s)| *s > 0).count(), "windows": dev_panel.windows.len(), "targets": dev_panel.recs.len()},
            "vectors": panel_vectors,
            "elapsed_s": started.elapsed().as_secs_f64(),
        }),
    )?;
    write_json(
        &args.root,
        "panel.json",
        &json!({
            "records": dev_panel.recs.iter().map(|r| json!({
                "obs": r.obs, "doc": r.doc, "window": r.win, "i": r.i,
                "prev": r.prev, "cur": r.cur, "target": r.target
            })).collect::<Vec<_>>(),
            "vectors": "vectors/<arm>-dev.f64 are little-endian f64 per record, aligned to the record order",
        }),
    )?;

    println!(
        "empirical dev micro {:.6} | smoothed dev micro {:.6} | parent {:.6}",
        emp["dev_micro_bits"].as_f64().unwrap_or(f64::NAN),
        smo["dev_micro_bits"].as_f64().unwrap_or(f64::NAN),
        a_parent_dev.micro()
    );
    println!("screen primary_ok={primary_ok} smoothed_better={smoothed_better} teacher_gap={teacher_gap:.4} float={}", if float_branch { "RUN" } else { "NOT_RUN" });
    seal(&args.root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&args.root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "sealed and verified ({} unlisted); elapsed {:.1}s",
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

fn teacher(
    c2: &HashMap<u64, Vec<(u32, u32)>>,
    c1: &HashMap<u64, Vec<(u32, u32)>>,
    uni: &Uni,
    ctx: u64,
    cur: u64,
    _prev: usize,
    _cur: usize,
    lambdas: (f64, f64),
    smoothed: bool,
) -> Teacher {
    if !smoothed {
        let pairs = c2.get(&ctx).cloned().unwrap_or_default();
        let n: f64 = pairs.iter().map(|(_, c)| *c as f64).sum();
        return Teacher::Empirical { pairs, n };
    }
    let mut t = vec![0.0f64; VOCAB];
    let (l1, l2) = lambdas;
    // Draw: (1-l1) * U
    for v in 0..VOCAB {
        t[v] = (1.0 - l1) * uni.p(v as u32);
    }
    // l1 * p1
    let tot1: f64 = c1
        .get(&cur)
        .map(|v| v.iter().map(|(_, c)| *c as f64).sum())
        .unwrap_or(0.0);
    if tot1 > 0.0 {
        for (s, c) in c1.get(&cur).unwrap() {
            t[*s as usize] += l1 * (*c as f64) / tot1;
        }
    }
    // blend toward the order-2 level
    let mut out = vec![0.0f64; VOCAB];
    for v in 0..VOCAB {
        out[v] = (1.0 - l2) * t[v];
    }
    let tot2: f64 = c2
        .get(&ctx)
        .map(|v| v.iter().map(|(_, c)| *c as f64).sum())
        .unwrap_or(0.0);
    if tot2 > 0.0 {
        for (s, c) in c2.get(&ctx).unwrap() {
            out[*s as usize] += l2 * (*c as f64) / tot2;
        }
    }
    Teacher::Smoothed(out)
}

fn teacher_bits(t: &Teacher, p: &[f64]) -> f64 {
    match t {
        Teacher::Empirical { pairs, n } => {
            if *n == 0.0 {
                return 0.0;
            }
            let mut b = 0.0;
            for (s, c) in pairs {
                b -= (*c as f64 / *n) * p[*s as usize].max(1e-300).log2();
            }
            b
        }
        Teacher::Smoothed(q) => {
            let mut b = 0.0;
            for v in 0..q.len() {
                if q[v] > 0.0 {
                    b -= q[v] * p[v].max(1e-300).log2();
                }
            }
            b
        }
    }
}

fn teacher_kl(t: &Teacher, p: &[f64]) -> f64 {
    match t {
        Teacher::Empirical { pairs, n } => {
            if *n == 0.0 {
                return 0.0;
            }
            let mut b = 0.0;
            for (s, c) in pairs {
                let tv = *c as f64 / *n;
                b += tv * (tv / p[*s as usize].max(1e-300)).log2();
            }
            b
        }
        Teacher::Smoothed(q) => {
            let mut b = 0.0;
            for v in 0..q.len() {
                if q[v] > 0.0 {
                    b += q[v] * (q[v] / p[v].max(1e-300)).log2();
                }
            }
            b
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update(
    head: &mut Head,
    ws: &[usize],
    windows: &[Vec<u32>],
    occ: &[(usize, usize, usize, usize)],
    occ_target: &[u32],
    win_start: &[usize],
    feat: &HashMap<(usize, usize), Feat>,
    c2: &HashMap<u64, Vec<(u32, u32)>>,
    c1: &HashMap<u64, Vec<(u32, u32)>>,
    uni: &Uni,
    lambdas: (f64, f64),
    smoothed: bool,
) -> Result<f64, String> {
    let q = head.quantized();
    let scale = (-(head.f_bits as f64)).exp2();
    let inv_ln2 = 1.0 / std::f64::consts::LN_2;
    let mut gw = vec![0.0f32; head.master.len()];
    let mut loss = 0.0f64;
    let mut n = 0usize;
    for &wi in ws {
        let _ = windows[wi].len();
        let (s, e) = (win_start[wi], win_start[wi + 1]);
        for k in s..e {
            let (_w, _i, prev, cur) = occ[k];
            let _ = occ_target[k];
            let ctx = ctx2(prev, cur);
            let h = &feat[&(prev, cur)].h;
            let z = head.logits(&q, h);
            let p = softmax_scaled(&z, scale);
            let t = teacher(c2, c1, uni, ctx, cur as u64, prev, cur, lambdas, smoothed);
            loss += teacher_bits(&t, &p);
            n += 1;
            // dZ[v] = (p - t) * 2^-F / ln2
            let mut dz = vec![0.0f64; head.vocab];
            match &t {
                Teacher::Empirical { pairs, n: tn } => {
                    for (v, slot) in dz.iter_mut().enumerate() {
                        *slot = p[v];
                    }
                    if *tn > 0.0 {
                        for (succ, c) in pairs {
                            dz[*succ as usize] -= *c as f64 / *tn;
                        }
                    }
                }
                Teacher::Smoothed(tv) => {
                    for (v, slot) in dz.iter_mut().enumerate() {
                        *slot = p[v] - tv[v];
                    }
                }
            }
            for slot in dz.iter_mut() {
                *slot *= scale * inv_ln2;
            }
            // dW[v,j] = dZ[v] * h[j]
            for v in 0..head.vocab {
                let d = dz[v] as f32;
                if d == 0.0 {
                    continue;
                }
                let base = v * head.dv;
                for (j, &hj) in h.iter().enumerate() {
                    gw[base + j] += d * hj as f32;
                }
            }
        }
    }
    if n == 0 {
        return Err("empty batch".into());
    }
    let inv = 1.0 / n as f64;
    loss *= inv;
    let inv32 = inv as f32;
    let mut norm = 0.0f64;
    for x in gw.iter_mut() {
        *x *= inv32;
        norm += (*x as f64) * (*x as f64);
    }
    norm = norm.sqrt();
    if norm > CLIP as f64 {
        let f = (CLIP as f64 / norm) as f32;
        for x in gw.iter_mut() {
            *x *= f;
        }
    }
    // Adam
    head.age += 1;
    head.cursor += 1;
    let bc1 = 1.0 - BETA1.powi(head.age as i32);
    let bc2 = 1.0 - BETA2.powi(head.age as i32);
    for i in 0..head.master.len() {
        let g = gw[i];
        head.m[i] = BETA1 * head.m[i] + (1.0 - BETA1) * g;
        head.v[i] = BETA2 * head.v[i] + (1.0 - BETA2) * g * g;
        let mhat = head.m[i] / bc1;
        let vhat = head.v[i] / bc2;
        head.master[i] -= LR * mhat / (vhat.sqrt() + EPS);
    }
    Ok(loss)
}

/// Floating scores over frozen features (offline diagnostic only).
///
/// With `w[v,j] = code[v,j] * 2^shift[v]` and integer hidden values `h`, this reproduces the
/// parent's integer contribution **exactly**, because the parent computes
/// `(sum_j code*h[j]) << shift = sum_j (code*2^shift)*h[j]`.
fn float_z(w: &[f32], h: &[i32], parent: &PriorCore) -> Vec<f64> {
    let dv = parent.cfg.dv;
    let vocab = parent.cfg.vocab;
    (0..vocab)
        .map(|v| {
            let mut acc = 0.0f64;
            for j in 0..dv {
                acc += w[v * dv + j] as f64 * h[j] as f64;
            }
            ((parent.bias_codes[v] as i32) << parent.cfg.bias_scale_bits) as f64 + acc
        })
        .collect()
}

fn softmax_f64_scaled(z: &[f64], scale: f64) -> Vec<f64> {
    let mut max = f64::NEG_INFINITY;
    for &v in z {
        max = max.max(v * scale);
    }
    let mut p = Vec::with_capacity(z.len());
    let mut sum = 0.0;
    for &v in z {
        let e = (v * scale - max).exp();
        p.push(e);
        sum += e;
    }
    for x in p.iter_mut() {
        *x /= sum;
    }
    p
}

fn ce_bits_f64(z: &[f64], target: usize, f_bits: u32) -> f64 {
    let p = softmax_f64_scaled(z, (-(f_bits as f64)).exp2());
    -(p[target.min(p.len() - 1)].max(1e-300).ln()) / std::f64::consts::LN_2
}

/// Total mass of a teacher distribution. All 4096 units must participate: no top-k, no argmax
/// labels and no dropped tail.
fn teacher_mass(t: &Teacher) -> f64 {
    match t {
        Teacher::Empirical { pairs, n } => {
            if *n == 0.0 {
                0.0
            } else {
                pairs.iter().map(|(_, c)| *c as f64 / *n).sum()
            }
        }
        Teacher::Smoothed(q) => q.iter().sum(),
    }
}

fn teacher_is_valid(t: &Teacher, tol: f64) -> bool {
    let mass = teacher_mass(t);
    let nonneg = match t {
        Teacher::Empirical { pairs, n } => *n > 0.0 && pairs.iter().all(|(_, c)| *c > 0),
        Teacher::Smoothed(q) => q.iter().all(|x| *x >= 0.0),
    };
    nonneg && (mass - 1.0).abs() <= tol
}

#[allow(clippy::too_many_arguments)]
fn float_update(
    w: &mut [f32],
    m: &mut [f32],
    vv: &mut [f32],
    age: &mut u64,
    ws: &[usize],
    windows: &[Vec<u32>],
    occ: &[(usize, usize, usize, usize)],
    win_start: &[usize],
    feat: &HashMap<(usize, usize), Feat>,
    c2: &HashMap<u64, Vec<(u32, u32)>>,
    c1: &HashMap<u64, Vec<(u32, u32)>>,
    uni: &Uni,
    lambdas: (f64, f64),
    parent: &PriorCore,
) -> Result<(), String> {
    let scale = (-(parent.cfg.f_bits as f64)).exp2();
    let inv_ln2 = 1.0 / std::f64::consts::LN_2;
    let vocab = parent.cfg.vocab;
    let dv = parent.cfg.dv;
    let mut gw = vec![0.0f32; w.len()];
    let mut n = 0usize;
    for &wi in ws {
        let _ = windows[wi].len();
        let (s, e) = (win_start[wi], win_start[wi + 1]);
        for k in s..e {
            let (_w, _i, prev, cur) = occ[k];
            let ctx = ctx2(prev, cur);
            let h = &feat[&(prev, cur)].h;
            let z = float_z(w, h, parent);
            let p = softmax_f64_scaled(&z, scale);
            let t = teacher(c2, c1, uni, ctx, cur as u64, prev, cur, lambdas, true);
            let tv = match &t {
                Teacher::Smoothed(tv) => tv.clone(),
                Teacher::Empirical { pairs, n: tn } => {
                    let mut dense = vec![0.0f64; vocab];
                    if *tn > 0.0 {
                        for (succ, c) in pairs {
                            dense[*succ as usize] = *c as f64 / *tn;
                        }
                    }
                    dense
                }
            };
            n += 1;
            for v in 0..vocab {
                let d = ((p[v] - tv[v]) * scale * inv_ln2) as f32;
                if d == 0.0 {
                    continue;
                }
                let base = v * dv;
                for (j, &hj) in h.iter().enumerate() {
                    gw[base + j] += d * hj as f32;
                }
            }
        }
    }
    if n == 0 {
        return Err("empty batch".into());
    }
    let inv32 = 1.0 / n as f32;
    let mut norm = 0.0f64;
    for x in gw.iter_mut() {
        *x *= inv32;
        norm += (*x as f64) * (*x as f64);
    }
    norm = norm.sqrt();
    if norm > CLIP as f64 {
        let f = (CLIP as f64 / norm) as f32;
        for x in gw.iter_mut() {
            *x *= f;
        }
    }
    *age += 1;
    let bc1 = 1.0 - BETA1.powi(*age as i32);
    let bc2 = 1.0 - BETA2.powi(*age as i32);
    for i in 0..w.len() {
        let g = gw[i];
        m[i] = BETA1 * m[i] + (1.0 - BETA1) * g;
        vv[i] = BETA2 * vv[i] + (1.0 - BETA2) * g * g;
        w[i] -= LR * (m[i] / bc1) / ((vv[i] / bc2).sqrt() + EPS);
    }
    Ok(())
}

fn eval_true_label(parent: &PriorCore, panel: &Panel) -> Vec<f64> {
    eval_true_label_core(parent, panel, |h| {
        let _ = h;
        Vec::new()
    })
}

fn eval_true_label_core<F>(parent: &PriorCore, panel: &Panel, head: F) -> Vec<f64>
where
    F: Fn(&[i32]) -> Vec<i32>,
{
    panel
        .recs
        .iter()
        .map(|r| {
            let tr = parent.trace(r.prev, r.cur, true);
            let z = head(&tr.h);
            if z.is_empty() {
                parent.bits_one(&tr.z, r.target)
            } else {
                parent.bits_one(&z, r.target)
            }
        })
        .collect()
}

fn eval_true_label_float(parent: &PriorCore, panel: &Panel, w: &[f32]) -> Vec<f64> {
    panel
        .recs
        .iter()
        .map(|r| {
            let tr = parent.trace(r.prev, r.cur, true);
            let z = float_z(w, &tr.h, parent);
            ce_bits_f64(&z, r.target as usize, parent.cfg.f_bits)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn resume_test(
    args: &Args,
    parent: &PriorCore,
    init: &[f32],
    windows: &[Vec<u32>],
    occ: &[(usize, usize, usize, usize)],
    occ_target: &[u32],
    win_start: &[usize],
    order: &[usize],
    feat: &HashMap<(usize, usize), Feat>,
    c2: &HashMap<u64, Vec<(u32, u32)>>,
    c1: &HashMap<u64, Vec<(u32, u32)>>,
    uni: &Uni,
    lambdas: (f64, f64),
    expect: HeadMeta,
    wrong: HeadMeta,
) -> Result<ExitCode, String> {
    let mk = |meta: HeadMeta| Head {
        vocab: VOCAB,
        dv: parent.cfg.dv,
        f_bits: parent.cfg.f_bits,
        bias_codes: parent.bias_codes.clone(),
        bias_scale_bits: parent.cfg.bias_scale_bits,
        master: init.to_vec(),
        m: vec![0.0; init.len()],
        v: vec![0.0; init.len()],
        age: 0,
        cursor: 0,
        meta,
    };
    let run = |head: &mut Head, from: usize, to: usize| -> Result<(), String> {
        for u in from..to {
            let ws: Vec<usize> = order[u * BATCH..(u + 1) * BATCH].to_vec();
            update(
                head, &ws, windows, occ, occ_target, win_start, feat, c2, c1, uni, lambdas, false,
            )?;
        }
        Ok(())
    };
    let mut a = mk(expect.clone());
    run(&mut a, 0, 12)?;
    let mut b = mk(expect.clone());
    run(&mut b, 0, 8)?;
    let ck = b.checkpoint_bytes();
    let mut c = mk(expect.clone());
    c.resume_from(&ck, &expect)
        .map_err(|e| format!("resume: {e}"))?;
    if c.cursor != 8 || c.age != 8 {
        return Err("resume did not restore the cursor/age".into());
    }
    run(&mut c, 8, 12)?;
    let same =
        a.master == c.master && a.m == c.m && a.v == c.v && a.age == c.age && a.cursor == c.cursor;
    let mut d = mk(wrong.clone());
    let rejected = d.resume_from(&ck, &wrong).is_err();
    let next_a = &order[a.cursor * BATCH..(a.cursor + 1) * BATCH];
    let next_c = &order[c.cursor * BATCH..(c.cursor + 1) * BATCH];
    write_json(
        &args.root,
        "resume-test.json",
        &json!({
            "schema": "uor-r4.readout-continuation/1",
            "split_run_equals_uninterrupted": same,
            "wrong_identity_rejected": rejected,
            "cursor_after_resume": c.cursor,
            "age_after_resume": c.age,
            "next_batch_equal": next_a == next_c,
            "next_batch": next_a,
        }),
    )?;
    println!("resume test: identical={same} wrong_identity_rejected={rejected}");
    seal(&args.root).map_err(|e| format!("seal: {e}"))?;
    verify(&args.root).map_err(|e| format!("verify: {e}"))?;
    if !same || !rejected {
        return Err("continuation test failed".into());
    }
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Focused fixtures
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_core::native_geometric::learner::prior_learning::{bias_codes_from_counts, Config};

    fn tiny_parent(vocab: usize, dv: usize) -> PriorCore {
        let counts: Vec<u64> = (0..vocab).map(|t| (t as u64) + 1).collect();
        let bias = bias_codes_from_counts(&counts, counts.iter().sum(), vocab);
        let mut st = 0x1234_5678u64;
        let mut fill = |n: usize| -> Vec<f32> {
            (0..n)
                .map(|_| {
                    st ^= st << 13;
                    st ^= st >> 7;
                    st ^= st << 17;
                    ((st as f64 / u64::MAX as f64) as f32 * 2.0 - 1.0) * 1.5
                })
                .collect()
        };
        let e_old = fill((vocab + 1) * dv);
        let e_new = fill(vocab * dv);
        let wo = fill(vocab * dv);
        PriorCore {
            cfg: Config::new(vocab, dv),
            elements: (0..vocab).map(|t| (t % 120) as u16).collect(),
            e_old: TernaryLinear::quantize(&e_old, vocab + 1, dv),
            e_new: TernaryLinear::quantize(&e_new, vocab, dv),
            w_o: TernaryLinear::quantize(&wo, vocab, dv),
            bias_codes: bias,
        }
    }

    /// The analytic soft-target gradient used by `float_update` must match a central finite
    /// difference of the same (unrounded) floating linear-head loss.
    #[test]
    fn floating_head_gradient_matches_finite_difference() {
        let vocab = 8usize;
        let dv = 4usize;
        let scale = 2f64.powi(-10);
        let h: Vec<i32> = vec![7, -3, 11, 2];
        let bias: Vec<f64> = vec![0.5, -1.0, 2.0, 0.0, -0.5, 1.0, 0.25, -0.75];
        let t: Vec<f64> = {
            let mut v = vec![0.0; vocab];
            v[2] = 0.5;
            v[5] = 0.25;
            v[7] = 0.25;
            v
        };
        let mut st = 99u64;
        let mut w: Vec<f64> = (0..vocab * dv)
            .map(|_| {
                st ^= st << 13;
                st ^= st >> 7;
                st ^= st << 17;
                ((st as f64 / u64::MAX as f64) - 0.5) * 0.4
            })
            .collect();
        let loss = |w: &[f64]| -> f64 {
            let mut z = vec![0.0f64; vocab];
            for v in 0..vocab {
                z[v] = bias[v];
                for j in 0..dv {
                    z[v] += w[v * dv + j] * h[j] as f64;
                }
            }
            let p = softmax_f64_scaled(&z, scale);
            let mut l = 0.0;
            for v in 0..vocab {
                if t[v] > 0.0 {
                    l -= t[v] * p[v].max(1e-300).log2();
                }
            }
            l
        };
        // Analytic gradient: dL/dw[v,j] = (p[v]-t[v]) * 2^-F/ln2 * h[j]
        let mut z = vec![0.0f64; vocab];
        for v in 0..vocab {
            z[v] = bias[v];
            for j in 0..dv {
                z[v] += w[v * dv + j] * h[j] as f64;
            }
        }
        let p = softmax_f64_scaled(&z, scale);
        let inv_ln2 = 1.0 / std::f64::consts::LN_2;
        let analytic: Vec<f64> = (0..vocab * dv)
            .map(|i| {
                let (v, j) = (i / dv, i % dv);
                (p[v] - t[v]) * scale * inv_ln2 * h[j] as f64
            })
            .collect();
        let eps = 1e-6;
        for i in 0..vocab * dv {
            let orig = w[i];
            w[i] = orig + eps;
            let lp = loss(&w);
            w[i] = orig - eps;
            let lm = loss(&w);
            w[i] = orig;
            let fd = (lp - lm) / (2.0 * eps);
            assert!(
                (fd - analytic[i]).abs() < 1e-5,
                "index {i}: fd {fd} vs analytic {}",
                analytic[i]
            );
        }
    }

    /// With a one-hot empirical teacher the occurrence-weighted soft objective is exactly the
    /// token cross-entropy at the observed successor.
    #[test]
    fn empirical_teacher_reduces_to_token_cross_entropy() {
        let p = vec![0.1, 0.2, 0.3, 0.4];
        let t = Teacher::Empirical {
            pairs: vec![(2, 7)],
            n: 7.0,
        };
        let bits = teacher_bits(&t, &p);
        assert!((bits - (-p[2].log2())).abs() < 1e-12);
        assert!(teacher_is_valid(&t, 1e-12));
    }

    /// Occurrence weighting: doubling every count must not change the objective.
    #[test]
    fn occurrence_weighting_is_scale_invariant() {
        let p = vec![0.25, 0.25, 0.25, 0.25];
        let a = Teacher::Empirical {
            pairs: vec![(0, 3), (1, 1)],
            n: 4.0,
        };
        let b = Teacher::Empirical {
            pairs: vec![(0, 30), (1, 10)],
            n: 40.0,
        };
        assert!((teacher_bits(&a, &p) - teacher_bits(&b, &p)).abs() < 1e-12);
    }

    /// Missing teacher mass must be detectable; complete distributions must be accepted.
    #[test]
    fn missing_teacher_mass_is_detected() {
        let truncated = Teacher::Smoothed(vec![0.5, 0.0, 0.0, 0.0]);
        assert!(!teacher_is_valid(&truncated, 1e-9));
        let mut c1: HashMap<u64, Vec<(u32, u32)>> = HashMap::new();
        let mut c2: HashMap<u64, Vec<(u32, u32)>> = HashMap::new();
        c1.insert(3u64, vec![(1u32, 5u32)]);
        c2.insert(ctx2(2, 3), vec![(1u32, 5u32)]);
        let mut counts = vec![0u64; VOCAB];
        counts[9] = 4;
        counts[1] = 1;
        let uni = Uni { counts, total: 5 };
        let t = teacher(&c2, &c1, &uni, ctx2(2, 3), 3, 2, 3, (0.8, 0.7), true);
        assert!(teacher_is_valid(&t, 1e-9), "mass {}", teacher_mass(&t));
        let e = teacher(&c2, &c1, &uni, ctx2(2, 3), 3, 2, 3, (0.8, 0.7), false);
        assert!(teacher_is_valid(&e, 1e-12));
    }

    /// A hard head initialized from `code * 2^shift` must reproduce the parent's integer logits.
    #[test]
    fn step_zero_hard_head_reproduces_parent_logits() {
        let parent = tiny_parent(8, 4);
        let vocab = 8usize;
        let dv = 4usize;
        let mut master = vec![0.0f32; vocab * dv];
        for v in 0..vocab {
            let s = parent.w_o.shift(v);
            for j in 0..dv {
                let code = parent.w_o.weight(v, j);
                master[v * dv + j] = (code as f32) * ((1u32 << s) as f32);
            }
        }
        let q = TernaryLinear::quantize(&master, vocab, dv);
        assert_eq!(q.packed(), parent.w_o.packed());
        for v in 0..vocab {
            assert_eq!(q.shift(v), parent.w_o.shift(v));
        }
        let head = Head {
            vocab,
            dv,
            f_bits: parent.cfg.f_bits,
            bias_codes: parent.bias_codes.clone(),
            bias_scale_bits: parent.cfg.bias_scale_bits,
            master,
            m: vec![0.0; vocab * dv],
            v: vec![0.0; vocab * dv],
            age: 0,
            cursor: 0,
            meta: HeadMeta {
                parent_digest: [1; 32],
                data_digest: [2; 32],
                teacher: 0,
                lambdas: (0, 0),
                seed: 13,
                updates: 1,
                batch: 1,
            },
        };
        for prev in 0..8usize {
            for cur in 0..8usize {
                let tr = parent.trace(prev, cur, true);
                let z = head.logits(&q, &tr.h);
                assert_eq!(z, parent.int_logits(prev, cur, true));
            }
        }
    }

    /// The floating head at the parent's served codes/shifts reproduces the parent's scores exactly.
    #[test]
    fn floating_head_at_parent_codes_reproduces_parent_scores() {
        let parent = tiny_parent(8, 4);
        let vocab = 8usize;
        let dv = 4usize;
        let mut w = vec![0.0f32; vocab * dv];
        for v in 0..vocab {
            let s = parent.w_o.shift(v);
            for j in 0..dv {
                w[v * dv + j] = (parent.w_o.weight(v, j) as f32) * ((1u32 << s) as f32);
            }
        }
        for prev in 0..8usize {
            for cur in 0..8usize {
                let tr = parent.trace(prev, cur, true);
                let z = float_z(&w, &tr.h, &parent);
                let want = parent.int_logits(prev, cur, true);
                for v in 0..vocab {
                    assert_eq!(z[v], want[v] as f64);
                }
            }
        }
    }
}
