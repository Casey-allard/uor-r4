//! Recovery replay for the three saved negative prefix candidates.
//!
//! Reconstructs each pinned `CPXS` checkpoint into a versioned, executable `CPX2` artifact with a
//! bound exact `2I` product table, verifies the recovered hard parameters against the old
//! descriptors, reloads the artifact and checks integer parity, then saves per-occurrence and
//! per-document control vectors, arm-aware generation and full-vocabulary count-reference outputs.
//! No prefix fit is restarted and the sealed pilot roots are untouched.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use uor_r4_core::native_geometric::learner::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};
use uor_r4_core::native_geometric::learner::lowbit::TernaryLinear;
use uor_r4_core::native_geometric::learner::prefix_artifact::{
    parent_hash_convention, ExactGroupTable, PrefixHard, OUTPUT_SHIFT, READER_SHIFT, READER_WIDTH,
};
use uor_r4_core::native_geometric::learner::prefix_state::{
    pack_fixed, palette, Arm, PALETTE_SIZE,
};
use uor_r4_core::native_geometric::learner::prior_learning::{
    bias_codes_from_counts, targets, PriorCore,
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
    docs: PathBuf,
    tokenizer: PathBuf,
    source_rev: String,
    evaluator_rev: String,
    pilot_root: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let mut root = None;
    let mut artifact = None;
    let mut docs = None;
    let mut tokenizer = PathBuf::from(DEFAULT_TOKENIZER);
    let mut source_rev = String::from("unknown");
    let mut evaluator_rev = String::from("unknown");
    let mut pilot_root = None;
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
            "--docs" => docs = Some(PathBuf::from(v()?)),
            "--tokenizer" => tokenizer = PathBuf::from(v()?),
            "--source-rev" => source_rev = v()?,
            "--evaluator-rev" => evaluator_rev = v()?,
            "--pilot-root" => pilot_root = Some(PathBuf::from(v()?)),
            other => return Err(format!("unknown argument {other}")),
        }
        i += 2;
    }
    Ok(Args {
        root: root.ok_or("--root is required")?,
        artifact: artifact.ok_or("--artifact is required")?,
        docs: docs.ok_or("--docs is required")?,
        tokenizer,
        source_rev,
        evaluator_rev,
        pilot_root: pilot_root.ok_or("--pilot-root is required")?,
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

/// A parsed CPXS checkpoint: only what recovery needs.
struct Cpxs {
    arm: u8,
    seed: u64,
    window: usize,
    batch: usize,
    updates: usize,
    warmup: usize,
    palette: [u8; PALETTE_SIZE],
    palette_identity: u8,
    reader_updates: u64,
    action_updates: u64,
    pass: u32,
    parent_sha256: [u8; 32],
    data_identity: [u8; 32],
    r_master: Vec<f32>,
    w_master: Vec<f32>,
    a_logits: Vec<f32>,
}

fn parse_cpxs(bytes: &[u8], vocab: usize, dv: usize) -> Result<Cpxs, String> {
    let mut c = 0usize;
    let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
        let end = c.checked_add(n).ok_or("size overflow")?;
        if end > bytes.len() {
            return Err("truncated CPXS".into());
        }
        let s = &bytes[*c..end];
        *c = end;
        Ok(s)
    };
    if take(&mut c, 4)? != b"CPXS" {
        return Err("bad CPXS magic".into());
    }
    let u32_at = |c: &mut usize| -> Result<u32, String> {
        Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
    };
    let u64_at = |c: &mut usize| -> Result<u64, String> {
        Ok(u64::from_le_bytes(take(c, 8)?.try_into().unwrap()))
    };
    if u32_at(&mut c)? != 1 {
        return Err("unsupported CPXS version".into());
    }
    let cvocab = u32_at(&mut c)? as usize;
    let window = u32_at(&mut c)? as usize;
    let batch = u32_at(&mut c)? as usize;
    let updates = u32_at(&mut c)? as usize;
    let warmup = u32_at(&mut c)? as usize;
    let arm = take(&mut c, 1)?[0];
    let seed = u64_at(&mut c)?;
    let mut pal = [0u8; PALETTE_SIZE];
    pal.copy_from_slice(take(&mut c, PALETTE_SIZE)?);
    let palette_identity = take(&mut c, 1)?[0];
    let reader_updates = u64_at(&mut c)?;
    let action_updates = u64_at(&mut c)?;
    let pass = u32_at(&mut c)?;
    let mut parent_sha256 = [0u8; 32];
    parent_sha256.copy_from_slice(take(&mut c, 32)?);
    let mut data_identity = [0u8; 32];
    data_identity.copy_from_slice(take(&mut c, 32)?);
    if cvocab != vocab {
        return Err("CPXS vocab mismatch".into());
    }
    let wants = [
        GROUP_ORDER * READER_WIDTH,
        vocab * READER_WIDTH,
        vocab * PALETTE_SIZE,
        GROUP_ORDER * READER_WIDTH,
        GROUP_ORDER * READER_WIDTH,
        vocab * READER_WIDTH,
        vocab * READER_WIDTH,
        vocab * PALETTE_SIZE,
        vocab * PALETTE_SIZE,
    ];
    let mut blocks: Vec<Vec<f32>> = Vec::new();
    for want in wants {
        let len = u64_at(&mut c)? as usize;
        if len != want {
            return Err(format!("CPXS block length {len} != {want}"));
        }
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            let x = f32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
            if !x.is_finite() {
                return Err("CPXS holds a non-finite value".into());
            }
            v.push(x);
        }
        blocks.push(v);
    }
    let _ = dv;
    let mut it = blocks.into_iter();
    let r_master = it.next().unwrap();
    let w_master = it.next().unwrap();
    let a_logits = it.next().unwrap();
    Ok(Cpxs {
        arm,
        seed,
        window,
        batch,
        updates,
        warmup,
        palette: pal,
        palette_identity,
        reader_updates,
        action_updates,
        pass,
        parent_sha256,
        data_identity,
        r_master,
        w_master,
        a_logits,
    })
}

fn argmax8(a: &[f32]) -> usize {
    let mut best = 0usize;
    for k in 1..PALETTE_SIZE {
        if a[k] > a[best] {
            best = k;
        }
    }
    best
}

/// The floating-table reference composition used by the earlier pilot.
fn state_older_floating(
    actions: &[u8],
    pal: &[u8; PALETTE_SIZE],
    tokens: &[u32],
    i: usize,
) -> Option<usize> {
    if i < 2 {
        return None;
    }
    let start = i.saturating_sub(WINDOW - 1);
    let end = i - 1;
    if end <= start {
        return None;
    }
    let t = group_table();
    let mut q = t.identity as usize;
    for &tok in &tokens[start..end] {
        let k = actions[(tok as usize).min(actions.len() - 1)] as usize;
        q = t.product[q * ROW_STRIDE + pal[k] as usize] as usize;
    }
    Some(q)
}

fn state_tail_floating(
    actions: &[u8],
    pal: &[u8; PALETTE_SIZE],
    tokens: &[u32],
    i: usize,
) -> Option<usize> {
    if i < 2 {
        return None;
    }
    let t = group_table();
    let mut q = t.identity as usize;
    for &tok in &tokens[i - 1..=i] {
        let k = actions[(tok as usize).min(actions.len() - 1)] as usize;
        q = t.product[q * ROW_STRIDE + pal[k] as usize] as usize;
    }
    Some(q)
}

fn state_reversed(hard: &PrefixHard, tokens: &[u32], i: usize) -> Option<usize> {
    if i < 2 {
        return None;
    }
    let start = i.saturating_sub(WINDOW - 1);
    let end = i - 1;
    if end <= start {
        return None;
    }
    let mut q = hard.table.identity;
    for &tok in tokens[start..end].iter().rev() {
        q = hard
            .table
            .compose(q, hard.palette.elements[hard.action(tok)]);
    }
    Some(q as usize)
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
        return Err(format!(
            "artifact sha {artifact_sha} != pinned {EXPECTED_ARTIFACT_SHA256}"
        ));
    }
    let parent = PriorCore::from_bytes(&artifact_bytes).map_err(|e| format!("load parent: {e}"))?;
    parent.validate()?;
    let (parent_file_digest, legacy_parent_digest) = parent_hash_convention(&artifact_bytes);

    let tok_bytes = std::fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    let tok_sha = sha256_hex(&tok_bytes);
    if tok_sha != EXPECTED_TOKENIZER_SHA256 {
        return Err("tokenizer sha mismatch".into());
    }
    let tokenizer: HfBpeTokenizer =
        derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive tokenizer: {e}"))?;
    let derived_sha = sha256_hex(
        &derive_tokenizer_json(&tok_bytes, VOCAB).map_err(|e| format!("derive json: {e}"))?,
    );
    let tokenizer_digest: [u8; 32] = Sha256::digest(derived_sha.as_bytes()).into();

    let exact = ExactGroupTable::build().map_err(|e| format!("exact table: {e}"))?;
    println!(
        "exact table: 14400 products verified; mismatches vs floating table = {}",
        exact.max_mismatches_vs_floating
    );

    let (uniq, collected, duplicates) = reconstruct_corpus(&args.docs);
    let dev_pool: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Dev)
        .collect();
    let panel = build_panel(&uniq, &dev_pool, &tokenizer, 8);
    println!(
        "corpus collected={collected} eligible={} duplicates={duplicates}; panel docs={} windows={} targets={}",
        uniq.len(),
        panel.docs.iter().filter(|(_, _, s)| *s > 0).count(),
        panel.windows.len(),
        panel.recs.len()
    );
    let perm = build_perm(&panel.recs, 0xA5A5_1234u64);

    // Frozen unigram from all fit windows (the declared shared marginal).
    let fit_ids: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Fit)
        .collect();
    let mut counts = vec![0u64; VOCAB];
    let mut total = 0u64;
    for i in &fit_ids {
        for w in windows_of(&tokenizer.encode(&uniq[*i].text)) {
            for (_p, _c, t) in targets(&w, VOCAB).map(|(_i, p, c, t)| (p, c, t)) {
                counts[t as usize] += 1;
                total += 1;
            }
        }
    }
    let uni = Uni { counts, total };
    let bias = bias_codes_from_counts(&uni.counts, total, VOCAB);
    let bias_matches = bias == parent.bias_codes;

    // Count references from the same fit wording, tuned on the tune documents' first/last windows.
    let mut full1 = Cond::default();
    let mut full2 = Cond::default();
    for i in &fit_ids {
        for w in windows_of(&tokenizer.encode(&uniq[*i].text)) {
            for (_i, p, c, t) in targets(&w, VOCAB) {
                full1.observe(c as u64, t);
                full2.observe(ctx2(p, c), t);
            }
        }
    }
    let tune_ids: Vec<usize> = (0..uniq.len())
        .filter(|i| uniq[*i].split == Split::Tune)
        .collect();
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
    let (lambdas, tune_bits) = tune_lambdas(&full1, &full2, &uni, &tune_windows);

    // Parent (state-disabled) losses, shared by every arm comparison.
    let mut cache: HashMap<(usize, usize), Vec<i32>> = HashMap::new();
    let parent_loss: Vec<f64> = panel
        .recs
        .iter()
        .map(|r| {
            let z = cache
                .entry((r.prev, r.cur))
                .or_insert_with(|| parent.int_logits(r.prev, r.cur, true));
            parent.bits_one(z, r.target)
        })
        .collect();
    write_f64s(&args.root, "vectors/parent.f64", &parent_loss)?;
    let a_parent = aggregate(&panel.recs, &parent_loss, None);

    let arm_files = [
        ("arm0-learned_older_prefix", Arm::LearnedOlder),
        ("arm1-fixed_action_older_prefix", Arm::FixedOlder),
        ("arm2-learned_local_tail", Arm::LearnedTail),
    ];
    let mut arm_report = Vec::new();
    let mut gen_rows = Vec::new();
    let mut actions_by_arm: HashMap<u8, Vec<u8>> = HashMap::new();

    for (stem, arm) in arm_files {
        let ckpt_path = args.pilot_root.join(format!("{stem}.ckpt"));
        let desc_path = args.pilot_root.join(format!("{stem}.cpl2.json"));
        let ckpt = std::fs::read(&ckpt_path).map_err(|e| format!("read {stem}: {e}"))?;
        let desc: Value = serde_json::from_slice(
            &std::fs::read(&desc_path).map_err(|e| format!("read desc: {e}"))?,
        )
        .map_err(|e| format!("parse desc: {e}"))?;
        let cpxs = parse_cpxs(&ckpt, VOCAB, parent.cfg.dv)?;
        if Arm::from_u8(cpxs.arm)? != arm {
            return Err(format!("{stem}: CPXS arm {} != expected", cpxs.arm));
        }
        if cpxs.palette != palette().elements || cpxs.palette_identity != palette().identity {
            return Err(format!("{stem}: CPXS palette differs"));
        }
        // The legacy `parent_sha256` field holds SHA256 of the hex parent-hash string.
        if cpxs.parent_sha256 != legacy_parent_digest {
            return Err(format!(
                "{stem}: legacy parent binding does not match the recorded convention"
            ));
        }

        // Reconstruct hard parameters offline.
        let (r_packed, r_shift) =
            pack_fixed(&cpxs.r_master, GROUP_ORDER, READER_WIDTH, READER_SHIFT);
        let (w_packed, w_shift) = pack_fixed(&cpxs.w_master, VOCAB, READER_WIDTH, OUTPUT_SHIFT);
        let reader =
            TernaryLinear::from_packed(r_packed.clone(), r_shift, GROUP_ORDER, READER_WIDTH)?;
        let wg = TernaryLinear::from_packed(w_packed.clone(), w_shift, VOCAB, READER_WIDTH)?;
        let actions: Vec<u8> = (0..VOCAB)
            .map(|t| argmax8(&cpxs.a_logits[t * PALETTE_SIZE..(t + 1) * PALETTE_SIZE]) as u8)
            .collect();

        // Verify the recovered parameters against the old descriptors before evaluating anything.
        let reader_sha = sha256_hex(&r_packed);
        let output_sha = sha256_hex(&w_packed);
        let action_sha = sha256_hex(&actions);
        let want_reader = desc["reader_codes_sha256"].as_str().unwrap_or("");
        let want_output = desc["output_codes_sha256"].as_str().unwrap_or("");
        let want_action = desc["action_codes_sha256"].as_str().unwrap_or("");
        if reader_sha != want_reader || output_sha != want_output || action_sha != want_action {
            return Err(format!(
                "{stem}: recovered hard parameters do not match the recorded descriptor"
            ));
        }
        if let Some(arr) = desc["action_codes"].as_array() {
            let want: Vec<u8> = arr.iter().map(|v| v.as_u64().unwrap_or(9) as u8).collect();
            if want != actions {
                return Err(format!("{stem}: recovered RAW action array differs"));
            }
        }

        actions_by_arm.insert(cpxs.arm, actions.clone());
        let hard = PrefixHard {
            parent: parent.clone(),
            reader,
            wg,
            action_codes: actions.clone(),
            palette: palette().clone(),
            arm,
            table: exact.clone(),
            parent_file_digest,
            tokenizer_digest,
        };
        let bytes = hard.to_bytes();
        write_checked(&args.root, &format!("artifacts/{stem}.cpx2"), &bytes)?;
        let reloaded = PrefixHard::from_bytes(&bytes, &parent, &parent_file_digest)
            .map_err(|e| format!("{stem}: reload: {e}"))?;

        // Integer parity: artifact-bound table vs the historical floating table, plus logits.
        let mut state_mismatch = 0usize;
        let mut logit_mismatch = 0usize;
        for r in panel.recs.iter() {
            let w = &panel.windows[r.win];
            let (a_state, b_state) = match arm {
                Arm::LearnedTail => (
                    reloaded.state_tail(w, r.i),
                    state_tail_floating(&actions, &palette().elements, w, r.i),
                ),
                _ => (
                    reloaded.state_older(w, r.i),
                    state_older_floating(&actions, &palette().elements, w, r.i),
                ),
            };
            if a_state != b_state {
                state_mismatch += 1;
            }
            let za = reloaded.int_logits(r.prev, r.cur, a_state);
            // Reference logits through the floating table.
            let mut zb = parent.int_logits(r.prev, r.cur, true);
            if let Some(q) = b_state {
                let mut h = vec![0i32; READER_WIDTH];
                for (j, slot) in h.iter_mut().enumerate() {
                    *slot = reloaded.reader.weight(q, j) << reloaded.reader.shift(q);
                }
                let res = reloaded.wg.forward_i32(&h);
                for (x, y) in zb.iter_mut().zip(res.iter()) {
                    *x += *y;
                }
            }
            if za != zb {
                logit_mismatch += 1;
            }
        }
        if state_mismatch != 0 || logit_mismatch != 0 {
            return Err(format!(
                "{stem}: parity failure (states {state_mismatch}, logits {logit_mismatch})"
            ));
        }

        // Control vectors on the reloaded artifact.
        let own: Vec<Option<usize>> = panel
            .recs
            .iter()
            .map(|r| reloaded.state_for_arm(&panel.windows[r.win], r.i))
            .collect();
        let donor: Vec<Option<usize>> = (0..panel.recs.len())
            .map(|i| {
                if perm.eligible[i] {
                    let d = perm.donor[i];
                    reloaded.state_for_arm(&panel.windows[panel.recs[d].win], panel.recs[d].i)
                } else {
                    own[i]
                }
            })
            .collect();
        let rev: Vec<Option<usize>> = panel
            .recs
            .iter()
            .map(|r| state_reversed(&reloaded, &panel.windows[r.win], r.i))
            .collect();
        let disabled: Vec<Option<usize>> = vec![None; panel.recs.len()];

        let res_table: Vec<Vec<i32>> = (0..GROUP_ORDER)
            .map(|q| reloaded.residual_scores(q))
            .collect();
        let mut score = |states: &[Option<usize>]| -> Vec<f64> {
            panel
                .recs
                .iter()
                .zip(states.iter())
                .map(|(r, st)| {
                    let zp = cache
                        .entry((r.prev, r.cur))
                        .or_insert_with(|| parent.int_logits(r.prev, r.cur, true));
                    match st {
                        Some(q) => {
                            let mut z: Vec<i32> = zp.clone();
                            for (x, y) in z.iter_mut().zip(res_table[*q].iter()) {
                                *x += *y;
                            }
                            parent.bits_one(&z, r.target)
                        }
                        None => parent.bits_one(zp, r.target),
                    }
                })
                .collect()
        };
        let own_loss = score(&own);
        let perm_loss = score(&donor);
        let rev_loss = score(&rev);
        let dis_loss = score(&disabled);
        for (name, v) in [
            ("own", &own_loss),
            ("perm", &perm_loss),
            ("rev", &rev_loss),
            ("dis", &dis_loss),
        ] {
            write_f64s(&args.root, &format!("vectors/{stem}-{name}.f64"), v)?;
        }
        let changed = (0..panel.recs.len())
            .filter(|&i| perm.eligible[i] && donor[i] != own[i])
            .count();

        let a_own = aggregate(&panel.recs, &own_loss, None);
        let a_perm = aggregate(&panel.recs, &perm_loss, None);
        let a_rev = aggregate(&panel.recs, &rev_loss, None);
        let a_dis = aggregate(&panel.recs, &dis_loss, None);
        let elig = perm.eligible.clone();
        let a_own_e = aggregate(&panel.recs, &own_loss, Some(&elig));
        let a_perm_e = aggregate(&panel.recs, &perm_loss, Some(&elig));
        // Gains standardised as CE_reference - CE_candidate (positive = candidate better).
        let gain_parent = paired_interval(&a_parent, &a_own, 0x1234_5678);
        let perm_penalty = paired_interval(&a_perm, &a_own, 0xD1B5_4A32);
        let perm_penalty_e = paired_interval(&a_perm_e, &a_own_e, 0x9E37_79B9);
        let rev_penalty = paired_interval(&a_rev, &a_own, 0x2545_F491);

        arm_report.push(json!({
            "arm": arm.name(),
            "checkpoint": {"path": ckpt_path, "sha256": sha256_hex(&ckpt)},
            "descriptor_verified": true,
            "legacy_parent_binding_convention": "SHA256 of the hex parent-artifact-digest string",
            "raw_parent_file_digest": hex(&parent_file_digest),
            "artifact": {"path": format!("artifacts/{stem}.cpx2"), "sha256": sha256_hex(&bytes), "bytes": bytes.len()},
            "reader_packed_sha256": reader_sha,
            "output_packed_sha256": output_sha,
            "raw_action_array_sha256": action_sha,
            "action_transitions_vs_learned_older": Value::Null,
            "state_parity_mismatches": state_mismatch,
            "logit_parity_mismatches": logit_mismatch,
            "panel_micro_bits": a_own.micro(),
            "panel_document_macro_bits": a_own.macro_bits(),
            "disabled_micro_bits": a_dis.micro(),
            "gain_vs_parent": {"point": gain_parent.0, "lo": gain_parent.1, "hi": gain_parent.2},
            "perm_penalty_full": {"point": perm_penalty.0, "lo": perm_penalty.1, "hi": perm_penalty.2},
            "perm_penalty_eligible": {"point": perm_penalty_e.0, "lo": perm_penalty_e.1, "hi": perm_penalty_e.2},
            "reverse_penalty": if arm == Arm::LearnedTail {
                // The local-tail arm has no older-prefix channel, so reversing an older prefix the
                // arm never reads is out-of-distribution rather than a sensitivity control.
                Value::Null
            } else {
                json!({"point": rev_penalty.0, "lo": rev_penalty.1, "hi": rev_penalty.2})
            },
            "reverse_note": if arm == Arm::LearnedTail {
                "NOT_APPLICABLE: the local-tail arm reads no older prefix; reversing it measures an out-of-distribution state, and reversing the arm's own pair state would change the local tail"
            } else { "reverse only the older-prefix order, preserving its multiset and the local tail" },
            "conditional_permutation_note": if arm == Arm::LearnedTail {
                "the local-tail arm is exactly invariant under the older-prefix intervention (penalty 0.0), which is what distinguishes it from the primary"
            } else { "state donors exchanged within exact (prev, cur, older-prefix length)" },
            "changed_state_recipients": changed,
            "per_document": a_own.rows.iter().map(|(k, l, n)| json!({"content_id": k, "loss_sum": l, "count": n})).collect::<Vec<_>>(),
            "checkpoint_metadata": {
                "seed": cpxs.seed, "window": cpxs.window, "batch": cpxs.batch,
                "updates": cpxs.updates, "warmup": cpxs.warmup,
                "reader_updates": cpxs.reader_updates, "action_updates": cpxs.action_updates,
                "pass": cpxs.pass,
                "continuation_note": "recovery is final-weight reconstruction; this legacy CPXS has no schedule/cursor binding and its resume adopts the stored data identity"
            },
            "weight_bytes": reloaded.weight_bytes(),
        }));

        // Arm-aware generation from the reloaded artifact.
        for (pi, prompt) in LEGACY_PROMPTS.iter().enumerate() {
            let out = reloaded.generate(prompt, GEN_TOKENS);
            let (pair, ring) = cycle_certificates(prompt, &out);
            gen_rows.push(json!({
                "prompt_index": pi,
                "model": format!("recovered_{}", arm.name()),
                "arm_aware_state": true,
                "output": out,
                "decoded": tokenizer.decode(&out),
                "pair_state_cycle": pair.map(|(e, p)| json!({"entry": e, "period": p})),
                "ring_state_cycle": ring.map(|(e, p)| json!({"entry": e, "period": p})),
            }));
        }
    }

    // Cross-arm hard-action differences (changed final action IDs, not transitions during learning).
    {
        let base = actions_by_arm
            .get(&(Arm::LearnedOlder as u8))
            .cloned()
            .unwrap_or_default();
        for row in arm_report.iter_mut() {
            let name = row["arm"].as_str().unwrap_or("").to_string();
            let key = match name.as_str() {
                "learned_older_prefix" => Arm::LearnedOlder as u8,
                "fixed_action_older_prefix" => Arm::FixedOlder as u8,
                "learned_local_tail" => Arm::LearnedTail as u8,
                _ => 255,
            };
            if let (Some(arr), Some(o)) = (actions_by_arm.get(&key), row.as_object_mut()) {
                let changed = arr.iter().zip(base.iter()).filter(|(a, b)| a != b).count();
                o.insert(
                    "changed_final_action_ids_vs_learned_older".into(),
                    json!(changed),
                );
            }
        }
    }
    let pilot_stats = std::fs::read(args.pilot_root.join("result.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .map(|v| json!({
            "hard_action_transitions_during_learning": v["fit"].as_array().map(|a| a.iter().map(|f| f["hard_action_transitions"].clone()).collect::<Vec<_>>()),
            "hard_action_occupancy": v["fit"].as_array().map(|a| a.iter().map(|f| f["hard_action_occupancy"].clone()).collect::<Vec<_>>()),
            "initial_parent_cache": v["training"].get("parent_cache").cloned(),
            "note": "copied from the sealed prefix-pilot-2 result for corrected labelling; final cache was 21,251 entries / 348,176,384 bytes",
        }));

    // Parent generation and corrected full-vocabulary count-reference generation.
    for (pi, prompt) in LEGACY_PROMPTS.iter().enumerate() {
        let out = parent.generate(prompt, GEN_TOKENS, true);
        let (pair, ring) = cycle_certificates(prompt, &out);
        gen_rows.push(json!({
            "prompt_index": pi, "model": "frozen_parent", "arm_aware_state": false,
            "output": out, "decoded": tokenizer.decode(&out),
            "pair_state_cycle": pair.map(|(e, p)| json!({"entry": e, "period": p})),
            "ring_state_cycle": ring.map(|(e, p)| json!({"entry": e, "period": p})),
        }));
        let rout = generate_reference_full(&full1, &full2, &uni, lambdas, prompt, GEN_TOKENS);
        let (rp, rr) = cycle_certificates(prompt, &rout);
        gen_rows.push(json!({
            "prompt_index": pi, "model": "count_reference_full_vocabulary", "arm_aware_state": false,
            "output": rout, "decoded": tokenizer.decode(&rout),
            "pair_state_cycle": rp.map(|(e, p)| json!({"entry": e, "period": p})),
            "ring_state_cycle": rr.map(|(e, p)| json!({"entry": e, "period": p})),
            "note": "argmax over all 4096 vocabulary entries with lowest-ID ties",
        }));
    }

    write_json(
        &args.root,
        "recovery.json",
        &json!({
            "schema": "uor-r4.prefix-recovery/1",
            "source_rev": args.source_rev,
            "evaluator": {"binary": "prefix-recover", "git_rev": args.evaluator_rev},
            "parent": {"path": args.artifact, "sha256": artifact_sha, "file_digest": hex(&parent_file_digest)},
            "tokenizer": {"source_sha256": tok_sha, "derived_sha256": derived_sha},
            "exact_group_table": {
                "order": GROUP_ORDER,
                "products_verified": GROUP_ORDER * GROUP_ORDER,
                "mismatches_vs_floating_table": exact.max_mismatches_vs_floating,
                "identity_historical_index": exact.identity,
                "palette": palette().elements,
                "palette_identity": palette().identity,
                "state_order": "historical learner::embedding enumeration; exact table transported through an explicit bijection",
            },
            "panel": {"documents": panel.docs.iter().filter(|(_, _, s)| *s > 0).count(), "windows": panel.windows.len(), "targets": panel.recs.len()},
            "permutation": {"strata": perm.strata, "excluded_no_history": perm.excluded_no_history, "eligible": perm.eligible.iter().filter(|x| **x).count()},
            "references": {"lambdas": [lambdas.0, lambdas.1], "tune_windows": tune_windows.len(), "tune_bits_per_target": tune_bits},
            "frozen_bias_matches_full_fit_unigram": bias_matches,
            "parent_micro_bits": a_parent.micro(),
            "arms": arm_report,
            "generation": gen_rows,
            "pilot_saved_statistics": pilot_stats,
            "label_corrections": {
                "probe": "two batch updates over sixteen windows, not sixteen updates",
                "action_norm": "the previously quoted action-gradient norm was the pre-softmax-Jacobian palette adjoint norm, not the Adam action-logit gradient norm",
                "cache": "the previously quoted 9,891-entry / 162 MB figure was the initial panel cache; the final saved cache was 21,251 entries / 348,176,384 bytes",
                "gains": "gains are standardised as CE_reference - CE_candidate; the earlier references' gain_vs_parent used the opposite convention",
                "screen": "a failed main screen can contain a passed component (the primary beat the fixed-action control)",
            },
            "elapsed_s": started.elapsed().as_secs_f64(),
        }),
    )?;
    write_json(
        &args.root,
        "panel.json",
        &json!({
            "records": panel.recs.iter().map(|r| json!({
                "obs": r.obs, "doc": r.doc, "window": r.win, "i": r.i,
                "prev": r.prev, "cur": r.cur, "target": r.target, "older_len": r.older_len
            })).collect::<Vec<_>>(),
            "windows": panel.windows,
            "window_docs": panel.window_docs,
            "donor_map": (0..panel.recs.len()).map(|i| json!({
                "obs": panel.recs[i].obs, "eligible": perm.eligible[i], "donor_index": perm.donor[i]
            })).collect::<Vec<_>>(),
            "vectors": "vectors/*.f64 are little-endian f64 per record, aligned to the record order above",
        }),
    )?;
    write_json(
        &args.root,
        "generation.json",
        &json!({"rows": gen_rows, "tokens": GEN_TOKENS}),
    )?;

    seal(&args.root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&args.root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "recovery sealed and verified ({} unlisted); elapsed {:.1}s",
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}
