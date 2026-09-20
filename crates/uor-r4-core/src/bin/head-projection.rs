//! Frozen output-head projection: Q0 (the existing quantizer) versus QG (one fit-activation-aware
//! dyadic ternary calibration).
//!
//! Reuses the retained floating output head and the frozen integer features. Only the output head
//! changes: `e_old`, `e_new`, their shifts, normalization, bias codes/scale, `elements`, vocabulary,
//! `F` and the argmax tie rule stay byte-identical. No gradient fit, wider layer, new precision,
//! new corpus or prefix refit is performed.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::{json, Value};

use uor_r4_core::native_geometric::learner::head_projection::{
    floating_gate, practical_gain, practical_screen, project_row, reconstruction_error,
    smoothing_attribution, Gram, RowCandidate, DV, MAX_SAFE_SHIFT,
};
use uor_r4_core::native_geometric::learner::lowbit::TernaryLinear;
use uor_r4_core::native_geometric::learner::prior_learning::{targets, PriorCore};
use uor_r4_core::native_geometric::learner::realtext_support::*;
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::bpe_derive::{derive_tokenizer, derive_tokenizer_json};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json";
const EXPECTED_TOKENIZER_SHA256: &str =
    "9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c";
const EXPECTED_DERIVED_SHA256: &str =
    "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";
const EXPECTED_PARENT_SHA256: &str =
    "cd5a3aa1b804ccc3582a4a25090e25c862489e0bc4e5bb86c1d1f363a7338a00";
const EXPECTED_CHECKPOINT_SHA256: &str =
    "1c3dfedb2248ec76c21f320b7ecec585c7a522a2d47e35ed5dc9eaca2454bff2";
const EXPECTED_EMPIRICAL_SHA256: &str =
    "d8a1fb14457481d48faa47b3f0794f42c6ca265b8de11b40af795fa03cd0215c";
const EXPECTED_SMOOTHED_SHA256: &str =
    "8a0950332b6f3638d8e46e797c6dafb7dda1a20cd2cd84901cdad4f2591447f8";
const EXPECTED_FLOAT_SHA256: &str =
    "84f28bbd04b9d14cdedbfc2f9c13c24ad27f271d92401f467da5cf960a7bb1c0";
/// The actual teacher smoothing used by the retained S/F runs: the 4,096-window conditional counts
/// with the retained full-fit smoothing (the prescribed (0.7,0.5) belongs to the 4,096 *reference*
/// family and was not used by S/F).
const ACTUAL_LAMBDAS: (f64, f64) = (0.8, 0.7);
const PRESCRIBED_LAMBDAS: (f64, f64) = (0.7, 0.5);
/// Retained per-exposure reference smoothing from the sealed `prefix-pilot-2` references.
const LAMBDA_512: (f64, f64) = (0.5, 0.4);
const LAMBDA_4096: (f64, f64) = (0.7, 0.5);
const LAMBDA_FULL: (f64, f64) = (0.8, 0.7);
const PLACEHOLDER_TOKENIZER_DIGEST: [u8; 32] = [7u8; 32];
const SCREEN_MARGIN: f64 = 0.10;
const FLOAT_GATE_MARGIN: f64 = 0.10;
const PAIR_PERM_SEED: u64 = 0xA5A5_1234;
const TOLERANCE_BITS: f64 = 1e-8;
const GEN_TOKENS: usize = 64;
const PROBE_ROWS: usize = 64;
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
    parent: PathBuf,
    checkpoint: PathBuf,
    empirical: PathBuf,
    smoothed: PathBuf,
    float_head: PathBuf,
    docs: PathBuf,
    tokenizer: PathBuf,
    source_rev: String,
    evaluator_rev: String,
    probe: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut root = None;
    let mut parent = None;
    let mut checkpoint = None;
    let mut empirical = None;
    let mut smoothed = None;
    let mut float_head = None;
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
            "--parent" => parent = Some(PathBuf::from(v()?)),
            "--checkpoint" => checkpoint = Some(PathBuf::from(v()?)),
            "--empirical" => empirical = Some(PathBuf::from(v()?)),
            "--smoothed" => smoothed = Some(PathBuf::from(v()?)),
            "--float-head" => float_head = Some(PathBuf::from(v()?)),
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
        parent: parent.ok_or("--parent is required")?,
        checkpoint: checkpoint.ok_or("--checkpoint is required")?,
        empirical: empirical.ok_or("--empirical is required")?,
        smoothed: smoothed.ok_or("--smoothed is required")?,
        float_head: float_head.ok_or("--float-head is required")?,
        docs: docs.ok_or("--docs is required")?,
        tokenizer,
        source_rev,
        evaluator_rev,
        probe,
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
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Exactly `rows*cols` finite little-endian f32 values.
fn read_f32_le(path: &Path, rows: usize, cols: usize) -> Result<Vec<f32>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let want = rows * cols * 4;
    if bytes.len() != want {
        return Err(format!(
            "float head is {} bytes, expected {want}",
            bytes.len()
        ));
    }
    let mut out = Vec::with_capacity(rows * cols);
    for chunk in bytes.chunks_exact(4) {
        let x = f32::from_le_bytes(chunk.try_into().unwrap());
        if !x.is_finite() {
            return Err("float head holds a non-finite value".into());
        }
        out.push(x);
    }
    Ok(out)
}

/// Pack explicit ternary codes with the module's two-bit encoding.
fn pack_codes(codes: &[i8], rows: usize, cols: usize) -> Vec<u8> {
    let mut packed = vec![0u8; (rows * cols).div_ceil(4)];
    for (flat, &c) in codes.iter().enumerate() {
        let code = match c {
            1 => 1u8,
            -1 => 2u8,
            _ => 0u8,
        };
        packed[flat >> 2] |= code << (2 * (flat & 3) as u32);
    }
    packed
}

fn head_logits(parent: &PriorCore, head: &TernaryLinear, h: &[i32]) -> Vec<i32> {
    let acc = head.forward_i32(h);
    (0..parent.cfg.vocab)
        .map(|v| ((parent.bias_codes[v] as i32) << parent.cfg.bias_scale_bits) + acc[v])
        .collect()
}

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

fn bits_f64(z: &[f64], target: usize, f_bits: u32) -> f64 {
    let scale = (-(f_bits as f64)).exp2();
    let mut max = f64::NEG_INFINITY;
    for &v in z {
        max = max.max(v * scale);
    }
    let mut sum = 0.0;
    for &v in z {
        sum += (v * scale - max).exp();
    }
    let t = target.min(z.len() - 1);
    (max + sum.ln() - z[t] * scale) / std::f64::consts::LN_2
}

fn softmax_f64(z: &[f64], scale: f64) -> Vec<f64> {
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

/// The legacy CPCK permutation, for recovered exposure only.
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

/// The per-document / PAD-stratum occurrence permutation used as the local-head control.
///
/// This is the `prior-frozen-evaluate` semantics: a recipient keeps its target and receives a donor
/// context from within its own `(document, PAD-status)` stratum. It is **not** the older-prefix
/// conditional permutation, which groups identical local pairs and cannot test a local head.
fn pair_permutation(recs: &[Rec], seed: u64) -> (Vec<(usize, usize)>, Vec<bool>, usize, usize) {
    let mut groups: Vec<((String, bool), Vec<usize>)> = Vec::new();
    let mut index: HashMap<(String, bool), usize> = HashMap::new();
    for (i, r) in recs.iter().enumerate() {
        let key = (r.doc.clone(), r.i == 0);
        let slot = match index.get(&key) {
            Some(&s) => s,
            None => {
                let s = groups.len();
                index.insert(key.clone(), s);
                groups.push((key, Vec::new()));
                s
            }
        };
        groups[slot].1.push(i);
    }
    let mut donor: Vec<(usize, usize)> = recs.iter().map(|r| r.own_ctx()).collect();
    let mut eligible = vec![false; recs.len()];
    let mut st = seed | 1;
    let mut strata = 0usize;
    let mut changed = 0usize;
    for (_, members) in groups.iter() {
        strata += 1;
        if members.len() < 2 {
            continue;
        }
        let mut perm: Vec<usize> = (0..members.len()).collect();
        for i in (1..perm.len()).rev() {
            let j = (xorshift(&mut st) as usize) % (i + 1);
            perm.swap(i, j);
        }
        for (k, &rec) in members.iter().enumerate() {
            let d = members[perm[k]];
            donor[rec] = recs[d].own_ctx();
            eligible[rec] = true;
            if donor[rec] != recs[rec].own_ctx() {
                changed += 1;
            }
        }
    }
    (donor, eligible, strata, changed)
}

/// One occurrence of the fit population.
struct Occ {
    ctx: u32,
    target: u32,
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

fn ctx_key(prev: usize, cur: usize) -> u32 {
    ((prev as u32) * VOCAB as u32) + cur as u32
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

    // --- verify retained bytes -------------------------------------------------
    let parent_bytes = std::fs::read(&args.parent).map_err(|e| format!("parent: {e}"))?;
    let parent_sha = sha256_hex(&parent_bytes);
    if parent_sha != EXPECTED_PARENT_SHA256 {
        return Err(format!("parent sha {parent_sha} != pinned"));
    }
    let ckpt_bytes = std::fs::read(&args.checkpoint).map_err(|e| format!("checkpoint: {e}"))?;
    if sha256_hex(&ckpt_bytes) != EXPECTED_CHECKPOINT_SHA256 {
        return Err("checkpoint sha mismatch".into());
    }
    for (path, want, label) in [
        (&args.empirical, EXPECTED_EMPIRICAL_SHA256, "empirical"),
        (&args.smoothed, EXPECTED_SMOOTHED_SHA256, "smoothed"),
        (&args.float_head, EXPECTED_FLOAT_SHA256, "float head"),
    ] {
        let bytes = std::fs::read(path).map_err(|e| format!("{label}: {e}"))?;
        let got = sha256_hex(&bytes);
        if got != want {
            return Err(format!("{label} sha {got} != pinned {want}"));
        }
    }
    let parent = PriorCore::from_bytes(&parent_bytes).map_err(|e| format!("load parent: {e}"))?;
    parent.validate()?;
    let e_bytes = std::fs::read(&args.empirical).map_err(|e| format!("{e}"))?;
    let empirical = PriorCore::from_bytes(&e_bytes).map_err(|e| format!("load E: {e}"))?;
    let s_bytes = std::fs::read(&args.smoothed).map_err(|e| format!("{e}"))?;
    let smoothed = PriorCore::from_bytes(&s_bytes).map_err(|e| format!("load S: {e}"))?;
    let w_f = read_f32_le(&args.float_head, VOCAB, DV)?;
    // The retained E/S exports carry a placeholder tokenizer digest.
    let mut e_meta = [0u8; 32];
    let (_, e_digest) = empirical
        .metadata(&e_bytes)
        .map_err(|e| format!("E metadata: {e}"))?;
    e_meta.copy_from_slice(&e_digest);
    let placeholder_confirmed = e_meta == PLACEHOLDER_TOKENIZER_DIGEST;

    let tok_bytes = std::fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    if sha256_hex(&tok_bytes) != EXPECTED_TOKENIZER_SHA256 {
        return Err("tokenizer sha mismatch".into());
    }
    let tokenizer: HfBpeTokenizer =
        derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive tokenizer: {e}"))?;
    let derived_hex = sha256_hex(
        &derive_tokenizer_json(&tok_bytes, VOCAB).map_err(|e| format!("derive json: {e}"))?,
    );
    if derived_hex != EXPECTED_DERIVED_SHA256 {
        return Err(format!("derived tokenizer sha {derived_hex} != pinned"));
    }
    let real_digest_bytes: [u8; 32] = hex_to_bytes(&derived_hex)?.try_into().unwrap();

    // --- corpus, fit windows, recovered population -----------------------------
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
    let mut occ: Vec<Occ> = Vec::new();
    for w in windows.iter() {
        for (_i, prev, cur, target) in targets(w, VOCAB) {
            occ.push(Occ {
                ctx: ctx_key(prev, cur),
                target,
            });
        }
    }
    let n_occ = occ.len();
    println!(
        "corpus collected={collected} eligible={} duplicates={duplicates} fit_windows={} population_windows={} population_targets={n_occ}",
        uniq.len(),
        fit_windows.len(),
        windows.len()
    );

    // Frozen feature cache.
    let mut feat: HashMap<u32, Vec<i32>> = HashMap::new();
    for w in windows.iter() {
        for (_i, prev, cur, _t) in targets(w, VOCAB) {
            feat.entry(ctx_key(prev, cur))
                .or_insert_with(|| parent.trace(prev, cur, true).h);
        }
    }
    // Occurrence multiplicity per context, deterministic (sorted keys).
    let mut ctx_counts: HashMap<u32, u64> = HashMap::new();
    for o in occ.iter() {
        *ctx_counts.entry(o.ctx).or_insert(0) += 1;
    }
    let mut ctx_keys: Vec<u32> = ctx_counts.keys().copied().collect();
    ctx_keys.sort_unstable();
    println!(
        "distinct fit contexts={} feature cache entries={}",
        ctx_keys.len(),
        feat.len()
    );

    // --- Q0: the existing quantizer applied exactly once to the floating head ---
    let q0 = TernaryLinear::quantize(&w_f, VOCAB, DV);
    // Envelope validation through the parent's shared validator.
    {
        let probe = PriorCore {
            cfg: parent.cfg.clone(),
            elements: parent.elements.clone(),
            e_old: parent.e_old.clone(),
            e_new: parent.e_new.clone(),
            w_o: q0.clone(),
            bias_codes: parent.bias_codes.clone(),
        };
        probe
            .validate()
            .map_err(|e| format!("Q0 violates the declared arithmetic envelope: {e}"))?;
    }

    // --- Gram over the fit population -----------------------------------------
    let t_gram = Instant::now();
    let mut gram = Gram::new(DV);
    for key in ctx_keys.iter() {
        let h = &feat[key];
        gram.add_occurrence(h, ctx_counts[key])
            .map_err(|e| format!("gram: {e}"))?;
    }
    gram.finalize().map_err(|e| format!("gram: {e}"))?;
    let gram_s = t_gram.elapsed().as_secs_f64();
    println!("gram built over N={} in {gram_s:.2}s", gram.n);

    // --- QG: one activation-aware dyadic ternary projection --------------------
    let t_proj = Instant::now();
    let mut rows: Vec<RowCandidate> = Vec::with_capacity(VOCAB);
    let mut empirical_rows: Vec<Vec<i8>> = Vec::with_capacity(VOCAB);
    for r in 0..VOCAB {
        empirical_rows.push((0..DV).map(|j| empirical.w_o.weight(r, j) as i8).collect());
    }
    let s_e: Vec<u32> = (0..VOCAB).map(|r| empirical.w_o.shift(r)).collect();
    for r in 0..VOCAB {
        let w: Vec<f32> = (0..DV).map(|j| w_f[r * DV + j]).collect();
        let s0 = q0.shift(r);
        rows.push(project_row(
            &w,
            s0,
            s_e[r],
            Some(&empirical_rows[r]),
            &gram,
            MAX_SAFE_SHIFT,
        ));
        if args.probe && r + 1 == PROBE_ROWS {
            let el = t_proj.elapsed().as_secs_f64();
            println!(
                "probe: {PROBE_ROWS} rows projected in {el:.3}s -> full 4096 rows ~{:.1}s",
                el * (VOCAB as f64) / (PROBE_ROWS as f64)
            );
            write_json(
                &args.root,
                "probe.json",
                &json!({
                    "schema": "uor-r4.head-projection-probe/1",
                    "probe_rows": PROBE_ROWS,
                    "probe_seconds": el,
                    "projected_full_seconds": el * (VOCAB as f64) / (PROBE_ROWS as f64),
                    "gram_seconds": gram_s,
                    "gram_n": gram.n,
                    "distinct_contexts": ctx_keys.len(),
                    "probe_rows_detail": rows.iter().map(|c| json!({
                        "shift": c.shift, "seed": match c.seed { uor_r4_core::native_geometric::learner::head_projection::Seed::Nearest => "nearest", _ => "empirical" }, "j": c.j
                    })).collect::<Vec<_>>(),
                }),
            )?;
            seal(&args.root).map_err(|e| format!("seal: {e}"))?;
            verify(&args.root).map_err(|e| format!("verify: {e}"))?;
            return Ok(ExitCode::SUCCESS);
        }
    }
    let projection_s = t_proj.elapsed().as_secs_f64();

    let qg_codes: Vec<i8> = rows.iter().flat_map(|r| r.codes.clone()).collect();
    let qg_shifts: Vec<u32> = rows.iter().map(|r| r.shift).collect();
    let qg = TernaryLinear::from_packed(
        pack_codes(&qg_codes, VOCAB, DV),
        qg_shifts.clone(),
        VOCAB,
        DV,
    )?;
    {
        let probe = PriorCore {
            cfg: parent.cfg.clone(),
            elements: parent.elements.clone(),
            e_old: parent.e_old.clone(),
            e_new: parent.e_new.clone(),
            w_o: qg.clone(),
            bias_codes: parent.bias_codes.clone(),
        };
        probe
            .validate()
            .map_err(|e| format!("QG violates the declared arithmetic envelope: {e}"))?;
    }
    let q0_j: f64 = (0..VOCAB)
        .map(|r| {
            let w: Vec<f32> = (0..DV).map(|j| w_f[r * DV + j]).collect();
            let codes: Vec<i8> = (0..DV).map(|j| q0.weight(r, j) as i8).collect();
            reconstruction_error(&w, &codes, q0.shift(r), &gram)
        })
        .sum();
    let qg_j: f64 = rows.iter().map(|c| c.j).sum();
    let e_j: f64 = (0..VOCAB)
        .map(|r| {
            let w: Vec<f32> = (0..DV).map(|j| w_f[r * DV + j]).collect();
            let codes: Vec<i8> = (0..DV).map(|j| empirical.w_o.weight(r, j) as i8).collect();
            reconstruction_error(&w, &codes, empirical.w_o.shift(r), &gram)
        })
        .sum();
    println!(
        "projection done in {projection_s:.1}s; total fit J vs F: E {:.6e}, Q0 {:.6e}, QG {:.6e} (QG non-increasing vs Q0: {})",
        e_j,
        q0_j,
        qg_j,
        qg_j <= q0_j + 1e-9
    );

    // --- exports: corrected E/S plus Q0/QG ------------------------------------
    let export = |head: &TernaryLinear| -> Vec<u8> {
        PriorCore {
            cfg: parent.cfg.clone(),
            elements: parent.elements.clone(),
            e_old: parent.e_old.clone(),
            e_new: parent.e_new.clone(),
            w_o: head.clone(),
            bias_codes: parent.bias_codes.clone(),
        }
        .to_bytes(&real_digest_bytes, 13)
    };
    let e_fixed = export(&empirical.w_o);
    let s_fixed = export(&smoothed.w_o);
    let q0_bytes = export(&q0);
    let qg_bytes = export(&qg);
    write_checked(&args.root, "corrected/empirical.cpl2", &e_fixed)?;
    write_checked(&args.root, "corrected/smoothed.cpl2", &s_fixed)?;
    write_checked(&args.root, "hard/q0.cpl2", &q0_bytes)?;
    write_checked(&args.root, "hard/qg.cpl2", &qg_bytes)?;
    write_checked(
        &args.root,
        "hard/qg-codes.bin",
        &qg_codes.iter().map(|c| *c as u8).collect::<Vec<u8>>(),
    )?;
    write_json(
        &args.root,
        "hard/qg-shifts.json",
        &json!({"values": qg_shifts, "seeds": rows.iter().map(|r| match r.seed { uor_r4_core::native_geometric::learner::head_projection::Seed::Nearest => "nearest", _ => "empirical" }).collect::<Vec<_>>()}),
    )?;
    write_checked(
        &args.root,
        "float-head.bin",
        &w_f.iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<u8>>(),
    )?;

    // Corrected E/S: numerical fields must be identical to the retained files.
    let e_reload = PriorCore::from_bytes(&e_fixed).map_err(|e| format!("reload E: {e}"))?;
    let s_reload = PriorCore::from_bytes(&s_fixed).map_err(|e| format!("reload S: {e}"))?;
    let mut e_same = e_reload.w_o.packed() == empirical.w_o.packed()
        && e_reload.bias_codes == empirical.bias_codes
        && e_reload.e_old.packed() == empirical.e_old.packed()
        && e_reload.e_new.packed() == empirical.e_new.packed()
        && e_reload.elements == empirical.elements;
    let mut s_same = s_reload.w_o.packed() == smoothed.w_o.packed()
        && s_reload.bias_codes == smoothed.bias_codes
        && s_reload.e_old.packed() == smoothed.e_old.packed()
        && s_reload.e_new.packed() == smoothed.e_new.packed()
        && s_reload.elements == smoothed.elements;
    for r in 0..VOCAB {
        if e_reload.w_o.shift(r) != empirical.w_o.shift(r)
            || s_reload.w_o.shift(r) != smoothed.w_o.shift(r)
        {
            e_same = false;
            s_same = false;
        }
    }

    // --- development panel ------------------------------------------------------
    let panel = build_panel(&uniq, &dev_pool, &tokenizer, 8);
    let dev_recs = &panel.recs;
    let dev_ctx: Vec<u32> = dev_recs.iter().map(|r| ctx_key(r.prev, r.cur)).collect();
    println!(
        "dev panel docs={} windows={} targets={}",
        panel.docs.iter().filter(|(_, _, s)| *s > 0).count(),
        panel.windows.len(),
        dev_recs.len()
    );

    // Feature cache for the panel (parent features only).
    for r in dev_recs.iter() {
        feat.entry(ctx_key(r.prev, r.cur))
            .or_insert_with(|| parent.trace(r.prev, r.cur, true).h);
    }

    // --- shared evaluation ------------------------------------------------------
    let t_eval = Instant::now();
    let eval_head = |head: &TernaryLinear| -> Vec<f64> {
        let mut by_ctx: HashMap<u32, Vec<usize>> = HashMap::new();
        for (i, c) in dev_ctx.iter().enumerate() {
            by_ctx.entry(*c).or_default().push(i);
        }
        let mut out = vec![0.0f64; dev_recs.len()];
        let mut keys: Vec<u32> = by_ctx.keys().copied().collect();
        keys.sort_unstable();
        for k in keys {
            let h = &feat[&k];
            let z = head_logits(&parent, head, h);
            for &i in by_ctx[&k].iter() {
                out[i] = parent.bits_one(&z, dev_recs[i].target);
            }
        }
        out
    };
    let eval_float = || -> Vec<f64> {
        let mut by_ctx: HashMap<u32, Vec<usize>> = HashMap::new();
        for (i, c) in dev_ctx.iter().enumerate() {
            by_ctx.entry(*c).or_default().push(i);
        }
        let mut out = vec![0.0f64; dev_recs.len()];
        let mut keys: Vec<u32> = by_ctx.keys().copied().collect();
        keys.sort_unstable();
        for k in keys {
            let h = &feat[&k];
            let z = float_z(&w_f, h, &parent);
            for &i in by_ctx[&k].iter() {
                out[i] = bits_f64(&z, dev_recs[i].target as usize, parent.cfg.f_bits);
            }
        }
        out
    };

    let loss_parent = eval_head(&parent.w_o);
    let loss_e = eval_head(&empirical.w_o);
    let loss_s = eval_head(&smoothed.w_o);
    let loss_q0 = eval_head(&q0);
    let loss_qg = eval_head(&qg);
    let loss_f = eval_float();
    for (name, v) in [
        ("parent", &loss_parent),
        ("empirical", &loss_e),
        ("smoothed", &loss_s),
        ("float", &loss_f),
        ("q0", &loss_q0),
        ("qg", &loss_qg),
    ] {
        write_f64s(&args.root, &format!("vectors/dev-{name}.f64"), v)?;
    }
    let a_parent = aggregate(dev_recs, &loss_parent, None);
    let a_e = aggregate(dev_recs, &loss_e, None);
    let a_s = aggregate(dev_recs, &loss_s, None);
    let a_f = aggregate(dev_recs, &loss_f, None);
    let a_q0 = aggregate(dev_recs, &loss_q0, None);
    let a_qg = aggregate(dev_recs, &loss_qg, None);
    println!(
        "dev micro: parent {:.6} E {:.6} S {:.6} F {:.6} Q0 {:.6} QG {:.6}",
        a_parent.micro(),
        a_e.micro(),
        a_s.micro(),
        a_f.micro(),
        a_q0.micro(),
        a_qg.micro()
    );

    // Reproduction tolerances against the recorded values.
    let want = [
        ("E", 7.170815718_f64, a_e.micro()),
        ("S", 7.188584609, a_s.micro()),
        ("F", 6.874144256, a_f.micro()),
    ];
    let mut repro = Vec::new();
    let mut repro_ok = true;
    for (n, w, got) in want {
        let d = (got - w).abs();
        if d > TOLERANCE_BITS {
            repro_ok = false;
        }
        repro.push(json!({"arm": n, "recorded": w, "reproduced": got, "abs_delta": d}));
    }

    // --- reference comparators on the same panel --------------------------------
    let mut counts = vec![0u64; VOCAB];
    let mut total = 0u64;
    for w in fit_windows.iter() {
        for (_i, _p, _c, t) in targets(w, VOCAB) {
            counts[t as usize] += 1;
            total += 1;
        }
    }
    let uni = Uni { counts, total };
    let c2_from = |ids: &[usize]| -> Cond {
        let mut c = Cond::default();
        for &wi in ids {
            for (_i, _p, cur, t) in targets(&fit_windows[wi], VOCAB) {
                c.observe(cur as u64, t);
            }
        }
        c
    };
    let c2_pairs = |ids: &[usize]| -> Cond {
        let mut c = Cond::default();
        for &wi in ids {
            for (_i, p, cur, t) in targets(&fit_windows[wi], VOCAB) {
                c.observe(ctx2(p, cur), t);
            }
        }
        c
    };
    let mut c1_full = Cond::default();
    let mut c2_full = Cond::default();
    for w in fit_windows.iter() {
        for (_i, p, cur, t) in targets(w, VOCAB) {
            c1_full.observe(cur as u64, t);
            c2_full.observe(ctx2(p, cur), t);
        }
    }
    let c1_4096 = c2_from(&consumed);
    let c2_4096 = c2_pairs(&consumed);
    let c1_512 = c2_from(&consumed[..512]);
    let c2_512 = c2_pairs(&consumed[..512]);
    let ref_loss = |c1: &Cond, c2: &Cond, l: (f64, f64)| -> Vec<f64> {
        dev_recs
            .iter()
            .map(|r| {
                -family_p(c1, c2, &uni, r.prev, r.cur, r.target, l)
                    .max(1e-300)
                    .log2()
            })
            .collect()
    };
    let l_uni: Vec<f64> = dev_recs.iter().map(|r| -uni.p(r.target).log2()).collect();
    let l_bias: Vec<f64> = dev_recs
        .iter()
        .map(|r| {
            let z = parent.int_logits(r.prev, r.cur, false);
            parent.bits_one(&z, r.target)
        })
        .collect();
    let l_512 = ref_loss(&c1_512, &c2_512, LAMBDA_512);
    let l_4096 = ref_loss(&c1_4096, &c2_4096, LAMBDA_4096);
    let l_full = ref_loss(&c1_full, &c2_full, LAMBDA_FULL);
    let refs = [
        ("unigram", &l_uni),
        ("quantized_bias", &l_bias),
        ("ref_512_windows", &l_512),
        ("ref_4096_windows", &l_4096),
        ("ref_full_fit", &l_full),
    ];
    let mut ref_json = Vec::new();
    for (n, v) in refs.iter() {
        write_f64s(&args.root, &format!("vectors/ref-dev-{n}.f64"), v)?;
        let a = aggregate(dev_recs, v, None);
        ref_json.push(json!({
            "reference": n,
            "exposure": match *n { "ref_512_windows" => "512-window conditional counts, lambda (0.5,0.4)", "ref_4096_windows" => "4096-window conditional counts, lambda (0.7,0.5)", "ref_full_fit" => "full-fit conditional counts, lambda (0.8,0.7)", _ => "shared full-fit marginal" },
            "micro_bits": a.micro(),
            "document_macro_bits": a.macro_bits(),
        }));
    }

    // --- descriptive tune-window CE (never selects anything) --------------------
    let mut tune_windows: Vec<Vec<u32>> = Vec::new();
    for i in &tune_ids {
        let ws = windows_of(&tokenizer.encode(&uniq[*i].text));
        if ws.is_empty() {
            continue;
        }
        tune_windows.push(ws[0].clone());
        if ws.len() >= 2 {
            tune_windows.push(ws[ws.len() - 1].clone());
        }
        if tune_windows.len() >= 76 {
            break;
        }
    }
    tune_windows.truncate(76);
    let mut tune_recs: Vec<Rec> = Vec::new();
    for (wi, w) in tune_windows.iter().enumerate() {
        for (i, prev, cur, target) in targets(w, VOCAB) {
            tune_recs.push(Rec {
                obs: format!("tune:{wi}:{i}"),
                doc: format!("tune:{wi}"),
                win: wi,
                i,
                prev,
                cur,
                target,
                older_len: older_len(i),
            });
        }
    }
    for r in tune_recs.iter() {
        feat.entry(ctx_key(r.prev, r.cur))
            .or_insert_with(|| parent.trace(r.prev, r.cur, true).h);
    }
    let tune_ce = |head: &TernaryLinear| -> f64 {
        let mut by_ctx: HashMap<u32, Vec<usize>> = HashMap::new();
        for (i, r) in tune_recs.iter().enumerate() {
            by_ctx.entry(ctx_key(r.prev, r.cur)).or_default().push(i);
        }
        let (mut bits, mut n) = (0.0f64, 0usize);
        let mut keys: Vec<u32> = by_ctx.keys().copied().collect();
        keys.sort_unstable();
        for k in keys {
            let z = head_logits(&parent, head, &feat[&k]);
            for &i in by_ctx[&k].iter() {
                bits += parent.bits_one(&z, tune_recs[i].target);
                n += 1;
            }
        }
        bits / n.max(1) as f64
    };
    let tune_float_ce = {
        let mut by_ctx: HashMap<u32, Vec<usize>> = HashMap::new();
        for (i, r) in tune_recs.iter().enumerate() {
            by_ctx.entry(ctx_key(r.prev, r.cur)).or_default().push(i);
        }
        let (mut bits, mut n) = (0.0f64, 0usize);
        let mut keys: Vec<u32> = by_ctx.keys().copied().collect();
        keys.sort_unstable();
        for k in keys {
            let z = float_z(&w_f, &feat[&k], &parent);
            for &i in by_ctx[&k].iter() {
                bits += bits_f64(&z, tune_recs[i].target as usize, parent.cfg.f_bits);
                n += 1;
            }
        }
        bits / n.max(1) as f64
    };
    let tune_json = json!({
        "descriptive_only": true,
        "tune_documents": tune_ids.len(),
        "windows": tune_windows.len(),
        "targets": tune_recs.len(),
        "micro_bits": {
            "parent": tune_ce(&parent.w_o),
            "empirical": tune_ce(&empirical.w_o),
            "smoothed": tune_ce(&smoothed.w_o),
            "float": tune_float_ce,
            "q0": tune_ce(&q0),
            "qg": tune_ce(&qg),
        },
    });

    // --- common-objective matrix on the fit population -------------------------
    let t_matrix = Instant::now();
    let mut per_ctx: HashMap<u32, Vec<(u32, u64)>> = HashMap::new();
    for o in occ.iter() {
        let e = per_ctx.entry(o.ctx).or_default();
        match e.iter_mut().find(|(s, _)| *s == o.target) {
            Some((_, c)) => *c += 1,
            None => e.push((o.target, 1)),
        }
    }
    for v in per_ctx.values_mut() {
        v.sort_by_key(|(s, _)| *s);
    }
    // Population-level one-token conditional counts: the teacher the retained S/F runs used.
    let mut pop_c1: HashMap<u32, Vec<(u32, u64)>> = HashMap::new();
    for o in occ.iter() {
        let cur = (o.ctx % VOCAB as u32) as u32;
        let e = pop_c1.entry(cur).or_default();
        match e.iter_mut().find(|(s, _)| *s == o.target) {
            Some((_, c)) => *c += 1,
            None => e.push((o.target, 1)),
        }
    }
    for v in pop_c1.values_mut() {
        v.sort_by_key(|(s, _)| *s);
    }
    let mut max_teacher_mass_error = 0.0f64;
    let teacher_target = |key: u32, smoothed_arm: bool, cur: usize| -> Vec<f64> {
        if !smoothed_arm {
            let pairs = per_ctx.get(&key).cloned().unwrap_or_default();
            let n: f64 = pairs.iter().map(|(_, c)| *c as f64).sum();
            let mut t = vec![0.0f64; VOCAB];
            if n > 0.0 {
                for (s, c) in pairs {
                    t[s as usize] = c as f64 / n;
                }
            }
            t
        } else {
            let mut t = vec![0.0f64; VOCAB];
            for v in 0..VOCAB {
                t[v] = (1.0 - ACTUAL_LAMBDAS.0) * uni.p(v as u32);
            }
            let tot1: f64 = pop_c1
                .get(&(cur as u32))
                .map(|v| v.iter().map(|(_, c)| *c as f64).sum())
                .unwrap_or(0.0);
            if tot1 > 0.0 {
                for (s, c) in pop_c1[&(cur as u32)].iter() {
                    t[*s as usize] += ACTUAL_LAMBDAS.0 * (*c as f64) / tot1;
                }
            }
            let mut out = vec![0.0f64; VOCAB];
            for v in 0..VOCAB {
                out[v] = (1.0 - ACTUAL_LAMBDAS.1) * t[v];
            }
            let tot2: f64 = per_ctx
                .get(&key)
                .map(|v| v.iter().map(|(_, c)| *c as f64).sum())
                .unwrap_or(0.0);
            if tot2 > 0.0 {
                for (s, c) in per_ctx[&key].iter() {
                    out[*s as usize] += ACTUAL_LAMBDAS.1 * (*c as f64) / tot2;
                }
            }
            out
        }
    };
    let cur_of = |key: u32| -> usize { (key % VOCAB as u32) as usize };
    // `float` is dispatched by name inside the matrix loop; this value is never read as an integer head.
    let w_f_as_head = q0.clone();
    let mut matrix = Vec::new();
    for (name, head) in [
        ("parent", None),
        ("empirical", Some(&empirical.w_o)),
        ("smoothed", Some(&smoothed.w_o)),
        ("float", Some(&w_f_as_head)),
        ("q0", Some(&q0)),
        ("qg", Some(&qg)),
    ] {
        let mut true_ce = 0.0f64;
        let mut emp_ce = 0.0f64;
        let mut emp_kl = 0.0f64;
        let mut smo_ce = 0.0f64;
        let mut smo_kl = 0.0f64;
        for key in ctx_keys.iter() {
            let h = &feat[key];
            let z = match head {
                Some(hd) if name != "float" => head_logits(&parent, hd, h),
                Some(_) => {
                    // The floating head is evaluated with the recorded floating scorer and its
                    // intermediate logits are never rounded to integers.
                    let zf = float_z(&w_f, h, &parent);
                    let t = (key % VOCAB as u32) as usize;
                    let _ = t;
                    let q = softmax_f64(&zf, (-(parent.cfg.f_bits as f64)).exp2());
                    let n = ctx_counts[key] as f64;
                    let t_emp = teacher_target(*key, false, cur_of(*key));
                    let t_smo = teacher_target(*key, true, cur_of(*key));
                    let mut te = 0.0;
                    let mut ke = 0.0;
                    for v in 0..VOCAB {
                        if t_emp[v] > 0.0 {
                            te -= t_emp[v] * q[v].max(1e-300).log2();
                            ke += t_emp[v] * (t_emp[v] / q[v].max(1e-300)).log2();
                        }
                    }
                    let mut ts = 0.0;
                    let mut ks = 0.0;
                    for v in 0..VOCAB {
                        if t_smo[v] > 0.0 {
                            ts -= t_smo[v] * q[v].max(1e-300).log2();
                            ks += t_smo[v] * (t_smo[v] / q[v].max(1e-300)).log2();
                        }
                    }
                    true_ce += te * n;
                    emp_ce += te * n;
                    emp_kl += ke * n;
                    smo_ce += ts * n;
                    smo_kl += ks * n;
                    continue;
                }
                None => parent.int_logits((*key as usize) / VOCAB, cur_of(*key), true),
            };
            let p = softmax_scaled(&z, (-(parent.cfg.f_bits as f64)).exp2());
            let n = ctx_counts[key] as f64;
            let t_emp = teacher_target(*key, false, cur_of(*key));
            let t_smo = teacher_target(*key, true, cur_of(*key));
            let me: f64 = t_emp.iter().sum();
            let ms: f64 = t_smo.iter().sum();
            max_teacher_mass_error = max_teacher_mass_error
                .max((me - 1.0).abs())
                .max((ms - 1.0).abs());
            let mut te = 0.0;
            let mut ke = 0.0;
            for v in 0..VOCAB {
                if t_emp[v] > 0.0 {
                    te -= t_emp[v] * p[v].max(1e-300).log2();
                    ke += t_emp[v] * (t_emp[v] / p[v].max(1e-300)).log2();
                }
            }
            let mut ts = 0.0;
            let mut ks = 0.0;
            for v in 0..VOCAB {
                if t_smo[v] > 0.0 {
                    ts -= t_smo[v] * p[v].max(1e-300).log2();
                    ks += t_smo[v] * (t_smo[v] / p[v].max(1e-300)).log2();
                }
            }
            true_ce += te * n;
            emp_ce += te * n;
            emp_kl += ke * n;
            smo_ce += ts * n;
            smo_kl += ks * n;
        }
        let nn = n_occ as f64;
        matrix.push(json!({
            "head": name,
            "true_label_fit_ce_bits": true_ce / nn,
            "empirical_target_ce_bits": emp_ce / nn,
            "empirical_target_kl_bits": emp_kl / nn,
            "smoothed_target_ce_bits": smo_ce / nn,
            "smoothed_target_kl_bits": smo_kl / nn,
        }));
    }
    let matrix_s = t_matrix.elapsed().as_secs_f64();

    // --- controls ---------------------------------------------------------------
    let t_ctl = Instant::now();
    let (donor_ctx, eligible, strata, changed) = pair_permutation(dev_recs, PAIR_PERM_SEED);
    let donor_h: Vec<&Vec<i32>> = donor_ctx
        .iter()
        .map(|(p, c)| {
            feat.get(&ctx_key(*p, *c))
                .ok_or_else(|| "donor context not in cache".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let perm_loss = |head: &TernaryLinear| -> Vec<f64> {
        dev_recs
            .iter()
            .zip(donor_h.iter())
            .map(|(r, h)| {
                let z = head_logits(&parent, head, h);
                parent.bits_one(&z, r.target)
            })
            .collect()
    };
    let p_e = perm_loss(&empirical.w_o);
    let p_q0 = perm_loss(&q0);
    let p_qg = perm_loss(&qg);
    write_f64s(&args.root, "vectors/dev-perm-empirical.f64", &p_e)?;
    write_f64s(&args.root, "vectors/dev-perm-q0.f64", &p_q0)?;
    write_f64s(&args.root, "vectors/dev-perm-qg.f64", &p_qg)?;
    let a_p_e = aggregate(dev_recs, &p_e, None);
    let a_p_q0 = aggregate(dev_recs, &p_q0, None);
    let a_p_qg = aggregate(dev_recs, &p_qg, None);
    let penalty = |permuted: &Agg, own: &Agg| {
        let t = paired_interval(permuted, own, 0x9E37_79B9);
        json!({"point": t.0, "lo": t.1, "hi": t.2})
    };
    // Context-disabled invariant: every candidate's disabled logits equal the frozen bias.
    let bias_only: Vec<i32> = (0..VOCAB)
        .map(|v| (parent.bias_codes[v] as i32) << parent.cfg.bias_scale_bits)
        .collect();
    let mut disabled_invariant = true;
    for head in [&empirical.w_o, &q0, &qg, &parent.w_o] {
        for r in dev_recs.iter().take(64) {
            if head_logits(&parent, head, &vec![0i32; DV]) != bias_only {
                disabled_invariant = false;
            }
            let _ = r;
        }
    }
    let ctl_s = t_ctl.elapsed().as_secs_f64();

    // --- generation -------------------------------------------------------------
    let mut gen_rows = Vec::new();
    let gen_arms: Vec<(&str, PriorCore)> = vec![
        ("corrected_empirical", e_reload.clone()),
        (
            "q0",
            PriorCore::from_bytes(&q0_bytes).map_err(|e| format!("reload Q0: {e}"))?,
        ),
        (
            "qg",
            PriorCore::from_bytes(&qg_bytes).map_err(|e| format!("reload QG: {e}"))?,
        ),
    ];
    for (pi, prompt) in LEGACY_PROMPTS.iter().enumerate() {
        let pout = parent.generate(prompt, GEN_TOKENS, true);
        let (pp, pr) = cycle_certificates(prompt, &pout);
        gen_rows.push(json!({
            "prompt_index": pi, "model": "frozen_parent",
            "output": pout, "decoded": tokenizer.decode(&pout),
            "pair_state_cycle": pp.map(|(e, p)| json!({"entry": e, "period": p})),
            "ring_state_cycle": pr.map(|(e, p)| json!({"entry": e, "period": p})),
        }));
        for (name, core) in gen_arms.iter() {
            let out = core.generate(prompt, GEN_TOKENS, true);
            let (pair, ring) = cycle_certificates(prompt, &out);
            gen_rows.push(json!({
                "prompt_index": pi, "model": name,
                "output": out, "decoded": tokenizer.decode(&out),
                "pair_state_cycle": pair.map(|(e, p)| json!({"entry": e, "period": p})),
                "ring_state_cycle": ring.map(|(e, p)| json!({"entry": e, "period": p})),
            }));
        }
    }

    // --- cost -------------------------------------------------------------------
    let cost = |core: &PriorCore| -> f64 {
        let prompt = &LEGACY_PROMPTS[0];
        let t0 = Instant::now();
        for _ in 0..8 {
            let _ = core.generate(prompt, GEN_TOKENS, true);
        }
        t0.elapsed().as_secs_f64() / 8.0
    };
    let lat_e = cost(&e_reload);
    let lat_q0 = cost(&PriorCore::from_bytes(&q0_bytes).map_err(|e| format!("{e}"))?);
    let lat_qg = cost(&PriorCore::from_bytes(&qg_bytes).map_err(|e| format!("{e}"))?);

    // --- row/scale statistics and reconstruction errors -------------------------
    let mut q0_shift_changes = 0usize;
    let mut q0_code_changes = 0usize;
    let mut qg_shift_changes = 0usize;
    let mut qg_code_changes = 0usize;
    let mut q0_zeros = 0usize;
    let mut qg_zeros = 0usize;
    let mut qg_per_row = Vec::with_capacity(VOCAB);
    for r in 0..VOCAB {
        if q0.shift(r) != empirical.w_o.shift(r) {
            q0_shift_changes += 1;
        }
        if qg.shift(r) != empirical.w_o.shift(r) {
            qg_shift_changes += 1;
        }
        for j in 0..DV {
            if q0.weight(r, j) != empirical.w_o.weight(r, j) {
                q0_code_changes += 1;
            }
            if qg.weight(r, j) != empirical.w_o.weight(r, j) {
                qg_code_changes += 1;
            }
            if q0.weight(r, j) == 0 {
                q0_zeros += 1;
            }
            if qg.weight(r, j) == 0 {
                qg_zeros += 1;
            }
        }
        qg_per_row.push(json!({
            "row": r, "shift": qg.shift(r), "seed": match rows[r].seed { uor_r4_core::native_geometric::learner::head_projection::Seed::Nearest => "nearest", _ => "empirical" },
            "j": rows[r].j,
            "zero_fraction": (0..DV).filter(|&j| qg.weight(r, j) == 0).count() as f64 / DV as f64,
        }));
    }

    // --- screen -----------------------------------------------------------------
    let gain_q0 = practical_gain(&a_e, &a_q0);
    let gain_qg = practical_gain(&a_e, &a_qg);
    let method = paired_interval(&a_q0, &a_qg, 0x1234_5678);
    let attr = smoothing_attribution(&a_e, &a_s);
    let s_kl = matrix
        .iter()
        .find(|m| m["head"] == "smoothed")
        .and_then(|m| m["smoothed_target_kl_bits"].as_f64())
        .unwrap_or(f64::NAN);
    let screen = json!({
        "criterion": "practical: CE_E - CE_candidate >= 0.10 with paired document-bootstrap lower bound > 0; method attribution: CE_Q0 - CE_QG lower bound > 0",
        "primary_candidate": "qg",
        "incumbent": "empirical",
        "qg_practical_gain_vs_E": {"point": gain_qg.point, "lo": gain_qg.lo, "hi": gain_qg.hi},
        "qg_practical_pass": practical_screen(gain_qg, SCREEN_MARGIN),
        "q0_practical_gain_vs_E": {"point": gain_q0.point, "lo": gain_q0.lo, "hi": gain_q0.hi},
        "q0_practical_pass": practical_screen(gain_q0, SCREEN_MARGIN),
        "method_gain_q0_minus_qg": {"point": method.0, "lo": method.1, "hi": method.2},
        "method_attribution_pass": method.1 > 0.0,
        "smoothing_attribution_E_minus_S": {"point": attr.point, "lo": attr.lo, "hi": attr.hi},
        "smoothing_attribution_pass": practical_screen(attr, 0.0),
        "floating_gate_smoothed_teacher_kl": s_kl,
        "floating_gate_run": floating_gate(s_kl, FLOAT_GATE_MARGIN),
    });

    // --- reports ----------------------------------------------------------------
    let exe_sha = std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::read(p).ok())
        .map(|b| sha256_hex(&b))
        .unwrap_or_else(|| "UNAVAILABLE".into());
    write_json(
        &args.root,
        "result.json",
        &json!({
            "schema": "uor-r4.head-projection/1",
            "source_rev": args.source_rev,
            "evaluator": {"binary": "head-projection", "git_rev": args.evaluator_rev, "binary_sha256": exe_sha},
            "parent": {"path": args.parent, "sha256": parent_sha, "matches_pinned": parent_sha == EXPECTED_PARENT_SHA256},
            "tokenizer": {"source_sha256": EXPECTED_TOKENIZER_SHA256, "derived_sha256": derived_hex, "digest_field": hex(&real_digest_bytes)},
            "metadata_correction": {
                "retained_placeholder_confirmed": placeholder_confirmed,
                "sole_change": format!("CPL2 tokenizer digest field {} -> {}", hex(&e_meta), hex(&real_digest_bytes)),
                "empirical_numerical_fields_identical": e_same,
                "smoothed_numerical_fields_identical": s_same,
                "old_empirical_sha256": EXPECTED_EMPIRICAL_SHA256,
                "new_empirical_sha256": sha256_hex(&e_fixed),
                "old_smoothed_sha256": EXPECTED_SMOOTHED_SHA256,
                "new_smoothed_sha256": sha256_hex(&s_fixed),
            },
            "float_manifest": {
                "path": "float-head.bin",
                "sha256": sha256_hex(&w_f.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>()),
                "shape": [VOCAB, DV],
                "dtype": "little-endian f32, finite",
                "parent_sha256": parent_sha,
                "tokenizer_derived_sha256": derived_hex,
                "f_bits": parent.cfg.f_bits,
                "bias_scale_bits": parent.cfg.bias_scale_bits,
                "actual_teacher_lambdas": [ACTUAL_LAMBDAS.0, ACTUAL_LAMBDAS.1],
                "prescribed_lambdas_not_used": [PRESCRIBED_LAMBDAS.0, PRESCRIBED_LAMBDAS.1],
                "offline_only": true,
                "not_a_standalone_served_model": true,
            },
            "population": {"windows": windows.len(), "targets": n_occ, "distinct_contexts": ctx_keys.len(), "source": "recovered first 4096 consumed windows"},
            "gram": {"n": gram.n, "dv": DV, "uncentered": true, "singularity": "NOT_MEASURED", "singular_inputs_supported": true, "inverted": false, "seconds": gram_s},
            "projection": {
                "algorithm": "two coordinate sweeps (ascending then descending) per (shift, seed) candidate; strictly-negative acceptance; ascending tie-break; Q0 included as an explicit candidate; the nearest-code seed is swept at every admissible shift including s0",
                "shifts_tried": "sorted unique {s0-1, s0, s0+1, sE} filtered to 0..=15",
                "max_safe_shift": MAX_SAFE_SHIFT,
                "comparison_tolerance": 1e-9,
                "verification_tolerance": 1e-9,
                "seconds": projection_s,
                "total_fit_j_E": e_j,
                "total_fit_j_q0": q0_j,
                "total_fit_j_qg": qg_j,
                "qg_not_worse_than_q0": qg_j <= q0_j + 1e-9,
                "seed_counts": {
                    "nearest": rows.iter().filter(|r| matches!(r.seed, uor_r4_core::native_geometric::learner::head_projection::Seed::Nearest)).count(),
                    "empirical": rows.iter().filter(|r| matches!(r.seed, uor_r4_core::native_geometric::learner::head_projection::Seed::Empirical)).count(),
                },
            },
            "artifacts": {
                "q0": {"path": "hard/q0.cpl2", "sha256": sha256_hex(&q0_bytes), "bytes": q0_bytes.len()},
                "qg": {"path": "hard/qg.cpl2", "sha256": sha256_hex(&qg_bytes), "bytes": qg_bytes.len()},
                "empirical": {"bytes": e_fixed.len()},
                "smoothed": {"bytes": s_fixed.len()},
            },
            "row_statistics": {
                "q0_shift_changes_vs_E": q0_shift_changes,
                "q0_code_changes_vs_E": q0_code_changes,
                "qg_shift_changes_vs_E": qg_shift_changes,
                "qg_code_changes_vs_E": qg_code_changes,
                "q0_zero_fraction": q0_zeros as f64 / (VOCAB * DV) as f64,
                "qg_zero_fraction": qg_zeros as f64 / (VOCAB * DV) as f64,
                "gain_vs_E": {"point": gain_qg.point, "lo": gain_qg.lo, "hi": gain_qg.hi},
                "latency_s_per_64_tokens": {"E": lat_e, "Q0": lat_q0, "QG": lat_qg},
            },
            "panel": {
                "documents": panel.docs.iter().filter(|(_, _, s)| *s > 0).count(),
                "windows": panel.windows.len(),
                "targets": dev_recs.len(),
                "micro": {"parent": a_parent.micro(), "empirical": a_e.micro(), "smoothed": a_s.micro(), "float": a_f.micro(), "q0": a_q0.micro(), "qg": a_qg.micro()},
                "document_macro": {"parent": a_parent.macro_bits(), "empirical": a_e.macro_bits(), "smoothed": a_s.macro_bits(), "float": a_f.macro_bits(), "q0": a_q0.macro_bits(), "qg": a_qg.macro_bits()},
                "per_document": a_qg.rows.iter().map(|(k, l, n)| json!({"content_id": k, "loss_sum": l, "count": n})).collect::<Vec<_>>(),
            },
            "references": ref_json,
            "tune_panel": tune_json,
            "reproduction": {"tolerance_bits": TOLERANCE_BITS, "ok": repro_ok, "rows": repro},
            "common_objective_matrix": {"seconds": matrix_s, "teacher_mass_max_abs_error": max_teacher_mass_error, "actual_teacher": "lambda2*P2(4096-window conditional) + (1-lambda2)*(lambda1*P1(4096-window) + (1-lambda1)*U(full-fit)) with lambda (0.8,0.7)", "rows": matrix},
            "controls": {
                "pair_permutation": {
                    "rule": "per (document, PAD-status) Fisher-Yates bijection over occurrence indices; recipient target fixed, donor context substituted; the prior-frozen-evaluate semantics, NOT the older-prefix conditional permutation",
                    "seed": "0xA5A51234",
                    "strata": strata,
                    "eligible_records": eligible.iter().filter(|x| **x).count(),
                    "changed_context_records": changed,
                    "penalty_E": penalty(&a_p_e, &a_e),
                    "penalty_Q0": penalty(&a_p_q0, &a_q0),
                    "penalty_QG": penalty(&a_p_qg, &a_qg),
                },
                "context_disabled_invariant_to_permutation": disabled_invariant,
                "seconds": ctl_s,
            },
            "screen": screen,
            "generation": gen_rows,
            "elapsed_s": started.elapsed().as_secs_f64(),
            "eval_seconds": t_eval.elapsed().as_secs_f64(),
        }),
    )?;
    write_json(
        &args.root,
        "panel.json",
        &json!({
            "records": dev_recs.iter().map(|r| json!({
                "obs": r.obs, "doc": r.doc, "window": r.win, "i": r.i,
                "prev": r.prev, "cur": r.cur, "target": r.target
            })).collect::<Vec<_>>(),
            "donor": donor_ctx.iter().map(|(p, c)| json!([p, c])).collect::<Vec<_>>(),
            "vectors": "vectors/*.f64 are little-endian f64 aligned to the record order",
        }),
    )?;
    write_json(&args.root, "qg-rows.json", &json!({"rows": qg_per_row}))?;

    println!(
        "screen: {}",
        serde_json::to_string(&screen).unwrap_or_default()
    );
    seal(&args.root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&args.root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "sealed and verified ({} unlisted); elapsed {:.1}s",
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

fn hex_to_bytes(h: &str) -> Result<Vec<u8>, String> {
    if h.len() % 2 != 0 {
        return Err("odd hex length".into());
    }
    (0..h.len() / 2)
        .map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).map_err(|e| format!("hex: {e}")))
        .collect()
}
