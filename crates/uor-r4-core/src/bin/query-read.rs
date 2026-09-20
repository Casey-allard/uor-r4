//! Matched geometric query-read experiment: Q/S/L fitted against the frozen corrected E.
//!
//! Loads the corrected empirical head E unchanged, reconstructs the pinned
//! 4096 consumed fit windows and the 36-document / 288-window / 17,342-target open-dev panel, fits
//! one arm per residual arrangement with an identical parameter shape, initialization protocol,
//! optimizer and dose, exports each as a versioned `CPX3` artifact, reloads it and only then
//! reports final CE, matched controls, generation and cost.
//!
//! No prefix/head calibration is restarted and every sealed root is untouched.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::json;
use sha2::{Digest, Sha256};

use uor_r4_core::native_geometric::learner::prefix_artifact::{
    parent_hash_convention, ExactGroupTable,
};
use uor_r4_core::native_geometric::learner::prefix_state::{palette, ParentCache, PALETTE_SIZE};
use uor_r4_core::native_geometric::learner::prior_learning::{targets, PriorCore};
use uor_r4_core::native_geometric::learner::query_read::{
    QueryArm, QueryHard, QueryTrainer, CONTEXT, READER_WIDTH, RESIDUAL_MAX_ABS, RESIDUAL_SHIFT,
    TOTAL_UPDATES, WARMUP_UPDATES,
};
use uor_r4_core::native_geometric::learner::realtext_support::*;
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::bpe_derive::{derive_tokenizer, derive_tokenizer_json};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const DEFAULT_ROOT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/query-read-1";
const DEFAULT_PARENT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/head-projection-3/corrected/empirical.cpl2";
const DEFAULT_CHECKPOINT: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/attempt-1/prior_realtext.ckpt";
const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json";
const DEFAULT_DOCS: &str =
    "/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/inputs/docs";

const EXPECTED_PARENT_SHA256: &str =
    "565cf98a01273af38425687adb522704b09a89bd7ab976b58b20bfbdaccfa0bf";
const EXPECTED_CHECKPOINT_SHA256: &str =
    "1c3dfedb2248ec76c21f320b7ecec585c7a522a2d47e35ed5dc9eaca2454bff2";
const EXPECTED_TOKENIZER_SHA256: &str =
    "9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c";
const EXPECTED_DERIVED_SHA256: &str =
    "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";
/// Frozen E dev micro CE, reproduced before any new comparison is claimed.
const EXPECTED_E_MICRO: f64 = 7.170815717556;
const FIT_WINDOWS: usize = 4096;
const PANEL_TARGETS: usize = 17342;
const PANEL_WINDOWS: usize = 288;
const PANEL_DOCS: usize = 36;
const GEN_TOKENS: usize = 64;
const GEN_REPEATS: usize = 8;
const CURVE_WINDOWS: usize = 8;
/// Hard bound on the fit-time parent-score cache, in `(prev, cur)` rows.
const CATALOGUE_CACHE_ENTRIES: usize = 32_768;
/// The fixed seed for the single Fisher-Yates pass over the consumed windows.
const DOSE_SEED: u64 = 13;
/// The retained document-bootstrap seed.
const BOOT_SEED: u64 = 0x1234_5678;
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
    tokenizer: PathBuf,
    docs: PathBuf,
    total_updates: usize,
    warmup: usize,
}

fn parse_args() -> Result<Args, String> {
    let mut root = PathBuf::from(DEFAULT_ROOT);
    let mut parent = PathBuf::from(DEFAULT_PARENT);
    let mut checkpoint = PathBuf::from(DEFAULT_CHECKPOINT);
    let mut tokenizer = PathBuf::from(DEFAULT_TOKENIZER);
    let mut docs = PathBuf::from(DEFAULT_DOCS);
    let mut total_updates = TOTAL_UPDATES;
    let mut warmup = WARMUP_UPDATES;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        let mut take = |name: &str| -> Result<String, String> {
            it.next().ok_or_else(|| format!("{name} needs a value"))
        };
        match a.as_str() {
            "--root" => root = PathBuf::from(take("--root")?),
            "--parent" => parent = PathBuf::from(take("--parent")?),
            "--checkpoint" => checkpoint = PathBuf::from(take("--checkpoint")?),
            "--tokenizer" => tokenizer = PathBuf::from(take("--tokenizer")?),
            "--docs" => docs = PathBuf::from(take("--docs")?),
            "--updates" => {
                total_updates = take("--updates")?.parse().map_err(|e| format!("{e}"))?
            }
            "--warmup" => warmup = take("--warmup")?.parse().map_err(|e| format!("{e}"))?,
            other => return Err(format!("unknown argument {other}")),
        }
    }
    if warmup > total_updates {
        return Err("warmup exceeds the total schedule".into());
    }
    Ok(Args {
        root,
        parent,
        checkpoint,
        tokenizer,
        docs,
        total_updates,
        warmup,
    })
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

/// The permutation embedded in the retained `CPCK` v3 checkpoint.
///
/// Copied from the reviewed `head-projection` runner, which validated this layout against
/// `prior_realtext.ckpt`; the permutation is the recovered exposure order, not a re-derivation.
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

/// One fixed Fisher-Yates pass with the declared dose seed.
fn dose_order(n: usize) -> Vec<usize> {
    let mut st = DOSE_SEED | 1;
    let mut p: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (xorshift(&mut st) as usize) % (i + 1);
        p.swap(i, j);
    }
    p
}

/// Rows with the query neutralised to the identity element.
fn rows_identity_query(core: &QueryHard, tokens: &[u32], i: usize) -> Option<(usize, usize)> {
    let e = core.table.identity as usize;
    match core.arm {
        QueryArm::L => core.tail_state(tokens, i).map(|t| (t, e)),
        _ => core.older_state(tokens, i).map(|q| (q, e)),
    }
}

/// Rows with the older-prefix order reversed; the local-only arm is invariant by construction.
fn rows_reversed(core: &QueryHard, tokens: &[u32], i: usize) -> Option<(usize, usize)> {
    if i < 2 {
        return None;
    }
    let e = core.table.identity as usize;
    match core.arm {
        // The local-only arm does not consume the older prefix, so reversing it is a no-op: its own
        // read path is the correct reversed read, query transport included.
        QueryArm::L => core.read_path(tokens, i).map(|p| (p.first, p.second)),
        arm => {
            let start = i.saturating_sub(CONTEXT - 1);
            let mut q = e;
            for &tok in tokens[start..i - 1].iter().rev() {
                q = core.table.compose(q as u8, core.write_state(tok) as u8) as usize;
            }
            if arm == QueryArm::S {
                Some((q, core.query_state(tokens[i])))
            } else {
                Some((
                    core.table
                        .compose(q as u8, core.query_state(tokens[i]) as u8)
                        as usize,
                    e,
                ))
            }
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
    let mut phases: Vec<serde_json::Value> = Vec::new();

    // --- pinned inputs --------------------------------------------------------
    let parent_bytes = std::fs::read(&args.parent).map_err(|e| format!("parent: {e}"))?;
    let parent_sha = sha256_hex(&parent_bytes);
    if parent_sha != EXPECTED_PARENT_SHA256 {
        return Err(format!("parent sha {parent_sha} != pinned"));
    }
    let parent = PriorCore::from_bytes(&parent_bytes).map_err(|e| format!("load E: {e}"))?;
    parent.validate()?;
    let (parent_file_digest, _) = parent_hash_convention(&parent_bytes);

    let ckpt = std::fs::read(&args.checkpoint).map_err(|e| format!("checkpoint: {e}"))?;
    if sha256_hex(&ckpt) != EXPECTED_CHECKPOINT_SHA256 {
        return Err("checkpoint sha mismatch".into());
    }
    let tok_bytes = std::fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    if sha256_hex(&tok_bytes) != EXPECTED_TOKENIZER_SHA256 {
        return Err("tokenizer sha mismatch".into());
    }
    let derived_hex = sha256_hex(
        &derive_tokenizer_json(&tok_bytes, VOCAB).map_err(|e| format!("derive json: {e}"))?,
    );
    if derived_hex != EXPECTED_DERIVED_SHA256 {
        return Err(format!("derived tokenizer sha {derived_hex} != pinned"));
    }
    let tokenizer: HfBpeTokenizer =
        derive_tokenizer(&tok_bytes, VOCAB).map_err(|e| format!("derive tokenizer: {e}"))?;
    let tokenizer_digest: [u8; 32] = Sha256::digest(derived_hex.as_bytes()).into();

    let exact = ExactGroupTable::build().map_err(|e| format!("exact table: {e}"))?;
    let pal = palette().clone();
    if pal.identity != exact.identity {
        return Err("palette identity differs from the bound table identity".into());
    }

    // --- population -----------------------------------------------------------
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
    let perm = cpc_k_perm(&ckpt)?;
    if perm.len() != fit_windows.len() {
        return Err("checkpoint permutation length != fit windows".into());
    }
    let consumed: Vec<usize> = perm[..FIT_WINDOWS.min(perm.len())].to_vec();
    let windows: Vec<Vec<u32>> = consumed.iter().map(|&w| fit_windows[w].clone()).collect();
    let consumed_targets: usize = windows.iter().map(|w| w.len() - 1).sum();

    let panel = build_panel(&uniq, &dev_pool, &tokenizer, 8);
    if panel.recs.len() != PANEL_TARGETS
        || panel.windows.len() != PANEL_WINDOWS
        || panel.docs.iter().filter(|(_, _, s)| *s > 0).count() != PANEL_DOCS
    {
        return Err(format!(
            "panel mismatch: targets {} windows {} docs {}",
            panel.recs.len(),
            panel.windows.len(),
            panel.docs.iter().filter(|(_, _, s)| *s > 0).count()
        ));
    }
    eprintln!("[phase] population {:.1}s", started.elapsed().as_secs_f32());
    let donor = build_perm(&panel.recs, 0xA5A5_1234u64);
    println!(
        "corpus collected={collected} eligible={eligible} duplicates={duplicates} fit_windows={fw} consumed={consumed_n} consumed_targets={consumed_targets} panel={pt}/{pw}",
        eligible = uniq.len(),
        fw = fit_windows.len(),
        consumed_n = windows.len(),
        pt = panel.recs.len(),
        pw = panel.windows.len(),
    );

    // Frozen parent losses: the numerical incumbent, reproduced before any new comparison.
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
    write_f64s(&args.root, "vectors/E.f64", &parent_loss)?;
    eprintln!(
        "[phase] E panel losses {:.1}s",
        started.elapsed().as_secs_f32()
    );
    let a_e = aggregate(&panel.recs, &parent_loss, None);
    let e_micro = a_e.micro();
    let e_reproduced = (e_micro - EXPECTED_E_MICRO).abs() <= 1e-8;
    println!("E dev micro {e_micro:.9} bits/target (expected {EXPECTED_E_MICRO}); reproduced={e_reproduced}");

    // Declared descriptive fit/tune curve panel, fixed before any score.
    let mut curve_windows: Vec<Vec<u32>> = Vec::new();
    for i in &tune_ids {
        let ws = windows_of(&tokenizer.encode(&uniq[*i].text));
        if ws.is_empty() {
            continue;
        }
        curve_windows.push(ws[0].clone());
        if curve_windows.len() >= CURVE_WINDOWS {
            break;
        }
    }

    let order = dose_order(windows.len());
    let mut cache = ParentCache::default();
    let mut arm_rows: Vec<serde_json::Value> = Vec::new();
    let mut screens: HashMap<String, bool> = HashMap::new();

    for arm in [QueryArm::Q, QueryArm::S, QueryArm::L] {
        let arm_started = Instant::now();
        let mut tr = QueryTrainer::new(
            parent.clone(),
            pal.clone(),
            exact.clone(),
            arm,
            13,
            8,
            args.total_updates,
            args.warmup,
            parent_file_digest,
        )?;
        // Step 0 must reproduce E exactly on a sample before any update.
        {
            let core = tr.hard_core()?;
            let mut mismatch = 0usize;
            for r in panel.recs.iter().take(512) {
                let w = &panel.windows[r.win];
                let rows = core.read_path(w, r.i).map(|p| (p.first, p.second));
                if core.int_logits(r.prev, r.cur, rows) != parent.int_logits(r.prev, r.cur, true) {
                    mismatch += 1;
                }
            }
            if mismatch != 0 {
                return Err(format!(
                    "{}: step 0 differs from E at {mismatch} positions",
                    arm.name()
                ));
            }
        }
        write_checked(
            &args.root,
            &format!("checkpoints/{}-0000.cpqk", arm.name()),
            &tr.checkpoint_bytes(),
        )?;

        let mut curve: Vec<(usize, f64)> = vec![(0, curve_bits(&tr, &curve_windows)?)];
        let mut first_loss = f64::NAN;
        let mut cache_resets = 0usize;
        let mut cache_peak_entries = 0usize;
        let mut cache_peak_bytes = 0usize;
        for step in 1..=args.total_updates {
            let base = ((step - 1) * tr.batch) % order.len();
            let batch: Vec<Vec<u32>> = (0..tr.batch)
                .map(|k| windows[order[(base + k) % order.len()]].clone())
                .collect();
            let apply_ab = step > args.warmup;
            let loss = tr.train_batch(&batch, &mut cache, apply_ab)?;
            if step == 1 {
                first_loss = loss;
            }
            // Bound the fit-time parent-score cache: ~16 KiB per V=4096 context, so 32,768 entries
            // is ~512 MiB worst case. Recomputation, not retained memory, absorbs the overrun.
            if cache.entries() >= CATALOGUE_CACHE_ENTRIES {
                cache = ParentCache::default();
                cache_resets += 1;
            }
            cache_peak_entries = cache_peak_entries.max(cache.entries());
            cache_peak_bytes = cache_peak_bytes.max(cache.bytes());
            if step == args.warmup || step == 128 || step == 256 || step == args.total_updates {
                write_checked(
                    &args.root,
                    &format!("checkpoints/{}-{step:04}.cpqk", arm.name()),
                    &tr.checkpoint_bytes(),
                )?;
                curve.push((step, curve_bits(&tr, &curve_windows)?));
            }
        }
        eprintln!(
            "[phase] {} fit {:.1}s",
            arm.name(),
            arm_started.elapsed().as_secs_f32()
        );
        let core = tr.hard_core()?;
        let artifact = core.to_bytes();
        write_checked(
            &args.root,
            &format!("artifacts/{}.cpx3", arm.name()),
            &artifact,
        )?;
        let reloaded = QueryHard::from_bytes(&artifact, &parent, &parent_file_digest)
            .map_err(|e| format!("{}: reload: {e}", arm.name()))?;

        // Full-panel integer parity: reloaded vs in-memory, every field.
        let mut logit_mismatch = 0usize;
        for r in panel.recs.iter() {
            let w = &panel.windows[r.win];
            let a = core.read_path(w, r.i).map(|p| (p.first, p.second));
            let b = reloaded.read_path(w, r.i).map(|p| (p.first, p.second));
            if a != b || core.int_logits(r.prev, r.cur, a) != reloaded.int_logits(r.prev, r.cur, b)
            {
                logit_mismatch += 1;
            }
        }
        if logit_mismatch != 0 {
            return Err(format!(
                "{}: reloaded full-panel parity failed at {logit_mismatch} positions",
                arm.name()
            ));
        }

        // Matched controls, all on the reloaded artifact.
        let own_rows: Vec<Option<(usize, usize)>> = panel
            .recs
            .iter()
            .map(|r| {
                reloaded
                    .read_path(&panel.windows[r.win], r.i)
                    .map(|p| (p.first, p.second))
            })
            .collect();
        let donor_rows: Vec<Option<(usize, usize)>> = (0..panel.recs.len())
            .map(|i| {
                if donor.eligible[i] {
                    let d = donor.donor[i];
                    reloaded
                        .read_path(&panel.windows[panel.recs[d].win], panel.recs[d].i)
                        .map(|p| (p.first, p.second))
                } else {
                    own_rows[i]
                }
            })
            .collect();
        let identity_rows: Vec<Option<(usize, usize)>> = panel
            .recs
            .iter()
            .map(|r| rows_identity_query(&reloaded, &panel.windows[r.win], r.i))
            .collect();
        let reversed_rows: Vec<Option<(usize, usize)>> = panel
            .recs
            .iter()
            .map(|r| rows_reversed(&reloaded, &panel.windows[r.win], r.i))
            .collect();

        let mut score_cache: HashMap<(usize, usize), Vec<i32>> = HashMap::new();
        let mut score = |rows: &[Option<(usize, usize)>]| -> Vec<f64> {
            panel
                .recs
                .iter()
                .zip(rows.iter())
                .map(|(r, st)| {
                    let zp = score_cache
                        .entry((r.prev, r.cur))
                        .or_insert_with(|| parent.int_logits(r.prev, r.cur, true));
                    match st {
                        Some(rows) => {
                            let mut z: Vec<i32> = zp.clone();
                            for (x, y) in z
                                .iter_mut()
                                .zip(reloaded.residual_scores(rows.0, rows.1).iter())
                            {
                                *x += *y;
                            }
                            parent.bits_one(&z, r.target)
                        }
                        None => parent.bits_one(zp, r.target),
                    }
                })
                .collect()
        };
        let own_loss = score(&own_rows);
        let donor_loss = score(&donor_rows);
        let id_loss = score(&identity_rows);
        let rev_loss = score(&reversed_rows);
        let disabled: Vec<Option<(usize, usize)>> = vec![None; panel.recs.len()];
        let disabled_loss = score(&disabled);
        for (name, v) in [
            ("own", &own_loss),
            ("donor", &donor_loss),
            ("identity_query", &id_loss),
            ("reversed", &rev_loss),
            ("disabled", &disabled_loss),
        ] {
            write_f64s(&args.root, &format!("vectors/{}-{name}.f64", arm.name()), v)?;
        }
        // Residual off must reproduce E byte-for-byte in loss.
        let disabled_identical = disabled_loss == parent_loss;
        if !disabled_identical {
            return Err(format!(
                "{}: read-disabled output differs from E",
                arm.name()
            ));
        }
        // The local-only arm must be exactly invariant under the donor and reversal maps.
        // The invariance claim is only defined for the local-only arm; Q/S are expected to move.
        let l_invariant = if arm.uses_older_prefix() {
            None
        } else {
            Some(
                donor_loss == own_loss
                    && rev_loss == own_loss
                    && donor_rows == own_rows
                    && reversed_rows == own_rows,
            )
        };

        let a_own = aggregate(&panel.recs, &own_loss, None);
        let elig = donor.eligible.clone();
        let a_own_e = aggregate(&panel.recs, &own_loss, Some(&elig));
        let a_donor_e = aggregate(&panel.recs, &donor_loss, Some(&elig));
        let a_id = aggregate(&panel.recs, &id_loss, None);

        let old_penalty = paired_interval(&a_donor_e, &a_own_e, 0xD1B5_4A32);
        let query_penalty = paired_interval(&a_id, &a_own, 0x9E37_79B9);
        let gain_over_e = paired_interval(&a_e, &a_own, BOOT_SEED);

        // Generation from the reloaded artifact: the six retained prompts.
        let mut gen_rows = Vec::new();
        for (k, p) in LEGACY_PROMPTS.iter().enumerate() {
            let prompt: Vec<u32> = p.to_vec();
            let out = reloaded.generate(&prompt, GEN_TOKENS);
            let (pair, ring) = cycle_certificates(&prompt, &out);
            gen_rows.push(json!({
                "prompt_index": k,
                "prompt": prompt,
                "output": out,
                "pair_cycle": pair,
                "ring_cycle": ring,
                "decoded": tokenizer.decode(&out),
            }));
        }
        // Timing: 64-token generation from one prompt, repeated.
        let t0 = Instant::now();
        for _ in 0..GEN_REPEATS {
            let _ = reloaded.generate(&LEGACY_PROMPTS[0], GEN_TOKENS);
        }
        let gen_seconds = t0.elapsed().as_secs_f64() / GEN_REPEATS as f64;

        // Older-fold work: cost of the older product versus the local pair, measured.
        let w = &panel.windows[0];
        let i = w.len() - 2;
        let t0 = Instant::now();
        for _ in 0..10_000 {
            let _ = reloaded.older_state(w, i);
        }
        let older_fold_ns = t0.elapsed().as_nanos() as f64 / 10_000.0;
        let t0 = Instant::now();
        for _ in 0..10_000 {
            let _ = reloaded.tail_state(w, i);
        }
        let tail_fold_ns = t0.elapsed().as_nanos() as f64 / 10_000.0;

        let a_changes = tr
            .initial_a_codes()
            .iter()
            .zip(tr.hard_a_codes().iter())
            .filter(|(x, y)| x != y)
            .count();
        let b_changes = tr
            .initial_b_codes()
            .iter()
            .zip(tr.hard_b_codes().iter())
            .filter(|(x, y)| x != y)
            .count();

        eprintln!(
            "[phase] {} eval {:.1}s",
            arm.name(),
            arm_started.elapsed().as_secs_f32()
        );
        arm_rows.push(json!({
            "arm": arm.name(),
            "artifact": {
                "path": format!("artifacts/{}.cpx3", arm.name()),
                "sha256": sha256_hex(&artifact),
                "bytes": artifact.len(),
            },
            "reader_width": READER_WIDTH,
            "reader_shift": uor_r4_core::native_geometric::learner::query_read::READER_SHIFT,
            "output_shift": uor_r4_core::native_geometric::learner::query_read::OUTPUT_SHIFT,
            "residual_shift": RESIDUAL_SHIFT,
            "palette_size": PALETTE_SIZE,
            "residual_max_abs": RESIDUAL_MAX_ABS,
            "parameter_bytes": tr.state_bytes(),
            "hard_a_changes": a_changes,
            "hard_b_changes": b_changes,
            "rw_updates": tr.rw_updates,
            "ab_updates": tr.ab_updates,
            "first_batch_bits": first_loss,
            "parent_cache": {
                "bounded_entries": CATALOGUE_CACHE_ENTRIES,
                "resets": cache_resets,
                "peak_entries": cache_peak_entries,
                "peak_bytes": cache_peak_bytes,
            },
            "curve": curve.iter().map(|(s, b)| json!({"step": s, "bits": b})).collect::<Vec<_>>(),
            "reload_parity": {"positions": panel.recs.len(), "mismatches": logit_mismatch},
            "ce": {
                "micro": a_own.micro(),
                "macro": a_own.macro_bits(),
                "docs": a_own.docs(),
            },
            "read_disabled_equals_E": disabled_identical,
            "l_invariant_under_donor_and_reversal": match l_invariant {
                Some(v) => json!(v),
                None => serde_json::Value::Null,
            },
            "l_invariance_scope": "defined only for the local-only arm; Q/S are expected to move",
            "gain_over_E": {"point": gain_over_e.0, "lo": gain_over_e.1, "hi": gain_over_e.2},
            "older_donor_penalty_eligible": {
                "point": old_penalty.0, "lo": old_penalty.1, "hi": old_penalty.2,
                "eligible": elig.iter().filter(|x| **x).count(),
                "changed": (0..panel.recs.len()).filter(|&i| donor.eligible[i] && donor_rows[i] != own_rows[i]).count(),
            },
            "identity_query_penalty": {"point": query_penalty.0, "lo": query_penalty.1, "hi": query_penalty.2},
            "generation": gen_rows,
            "generation_seconds_per_64_tokens": gen_seconds,
            "older_fold_ns": older_fold_ns,
            "tail_fold_ns": tail_fold_ns,
            "elapsed_s": arm_started.elapsed().as_secs_f64(),
        }));
        phases.push(json!({"phase": format!("fit+eval {}", arm.name()), "seconds": arm_started.elapsed().as_secs_f64()}));
    }

    // --- on-disk split-versus-uninterrupted continuation check ---------------
    let continuation = {
        let mk = |arm: QueryArm| -> QueryTrainer {
            QueryTrainer::new(
                parent.clone(),
                pal.clone(),
                exact.clone(),
                arm,
                13,
                8,
                8,
                4,
                parent_file_digest,
            )
            .expect("mini trainer")
        };
        let batch_of = |step: usize| -> Vec<Vec<u32>> {
            let base = ((step - 1) * 8) % order.len();
            (0..8)
                .map(|k| windows[order[(base + k) % order.len()]].clone())
                .collect()
        };
        let mut full = mk(QueryArm::Q);
        let mut c1 = ParentCache::default();
        let mut full_losses = Vec::new();
        for step in 1..=8 {
            full_losses.push(full.train_batch(&batch_of(step), &mut c1, step > 4)?);
        }
        let full_artifact = full.hard_core()?.to_bytes();

        let mut first = mk(QueryArm::Q);
        let mut c2 = ParentCache::default();
        let mut split_losses = Vec::new();
        for step in 1..=4 {
            split_losses.push(first.train_batch(&batch_of(step), &mut c2, false)?);
        }
        let ck = first.checkpoint_bytes();
        write_checked(&args.root, "continuation/mid-0004.cpqk", &ck)?;
        let mut second = mk(QueryArm::Q);
        let mut c3 = ParentCache::default();
        second.resume_from(&ck)?;
        for step in 5..=8 {
            split_losses.push(second.train_batch(&batch_of(step), &mut c3, step > 4)?);
        }
        let split_artifact = second.hard_core()?.to_bytes();
        json!({
            "updates": 8,
            "split_at": 4,
            "warmup": 4,
            "next_batch_loss_equal": full_losses[4] == split_losses[4],
            "losses_equal": full_losses == split_losses,
            "artifact_bytes_equal": full_artifact == split_artifact,
            "artifact_sha256": sha256_hex(&full_artifact),
            "inference_only_load_is_not_training_continuation": true,
        })
    };
    phases.push(json!({"phase": "continuation check", "seconds": 0.0}));

    // --- predeclared decision rules ------------------------------------------
    let get = |name: &str| -> &serde_json::Value {
        arm_rows.iter().find(|r| r["arm"] == name).expect("arm row")
    };
    let ce = |name: &str| -> f64 { get(name)["ce"]["micro"].as_f64().unwrap() };
    // Recompute the Q-versus-S and Q-versus-L intervals from the saved vectors.
    let read_vec = |name: &str| -> Result<Vec<f64>, String> {
        let p = args.root.join(format!("vectors/{}-own.f64", name));
        let bytes = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        Ok(bytes
            .chunks_exact(8)
            .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
            .collect())
    };
    let q = read_vec("query_conditioned_older_read")?;
    let s = read_vec("separable_older_query_read")?;
    let l = read_vec("local_only_read")?;
    let a_q = aggregate(&panel.recs, &q, None);
    let a_s = aggregate(&panel.recs, &s, None);
    let a_l = aggregate(&panel.recs, &l, None);
    let q_minus_s = paired_interval(&a_s, &a_q, BOOT_SEED);
    let q_minus_l = paired_interval(&a_l, &a_q, BOOT_SEED);
    let e_minus_q = paired_interval(&a_e, &a_q, BOOT_SEED);

    let practical = e_minus_q.0 >= 0.10 && e_minus_q.1 > 0.0;
    let matched_s = q_minus_s.0 > 0.0 && q_minus_s.1 > 0.0;
    let matched_l = q_minus_l.0 > 0.0 && q_minus_l.1 > 0.0;
    let old_point = get("query_conditioned_older_read")["older_donor_penalty_eligible"]["point"]
        .as_f64()
        .unwrap();
    let old_lo = get("query_conditioned_older_read")["older_donor_penalty_eligible"]["lo"]
        .as_f64()
        .unwrap();
    let older_content = old_point >= 0.10 && old_lo > 0.0;
    let qp_point = get("query_conditioned_older_read")["identity_query_penalty"]["point"]
        .as_f64()
        .unwrap();
    let qp_lo = get("query_conditioned_older_read")["identity_query_penalty"]["lo"]
        .as_f64()
        .unwrap();
    let query_use = qp_point > 0.0 && qp_lo > 0.0;
    let instrument_ok = arm_rows.iter().all(|r| {
        r["reload_parity"]["mismatches"] == 0
            && r["read_disabled_equals_E"] == true
            && (r["arm"] != "local_only_read"
                || r["l_invariant_under_donor_and_reversal"] == serde_json::Value::Bool(true))
    });
    screens.insert("E_reproduced".into(), e_reproduced);
    screens.insert("practical_E_minus_Q".into(), practical);
    screens.insert("matched_Q_over_S".into(), matched_s);
    screens.insert("matched_Q_over_L".into(), matched_l);
    screens.insert("older_content".into(), older_content);
    screens.insert("query_route_use".into(), query_use);
    screens.insert("instrument_checks".into(), instrument_ok);
    let retained =
        practical && matched_s && matched_l && older_content && query_use && instrument_ok;

    println!(
        "E {:.6} Q {:.6} S {:.6} L {:.6}; E-Q {:.6} [{:.6},{:.6}] Q>S {:.6} Q>L {:.6}; screens {:?}",
        e_micro,
        ce("query_conditioned_older_read"),
        ce("separable_older_query_read"),
        ce("local_only_read"),
        e_minus_q.0,
        e_minus_q.1,
        e_minus_q.2,
        q_minus_s.0,
        q_minus_l.0,
        screens
    );

    write_json(
        &args.root,
        "result.json",
        &json!({
            "schema": "uor-r4.query-read/1",
            "source_rev": "92240a8619927d287bb90ee50aeebf0c944bc597",
            "evaluator": {
                "binary": "query-read",
                "git_rev": std::env::var("UOR_GIT_REV").unwrap_or_else(|_| "unset".into()),
            },
            "parent_E": {
                "path": args.parent, "sha256": parent_sha,
                "micro_bits": e_micro, "expected_micro_bits": EXPECTED_E_MICRO,
                "reproduced": e_reproduced,
            },
            "tokenizer": {
                "source_sha256": EXPECTED_TOKENIZER_SHA256,
                "derived_sha256": derived_hex,
                "digest_field": hex(&tokenizer_digest),
            },
            "exact_group_table": {
                "order": 120, "identity": exact.identity,
                "products_verified": 120 * 120,
                "mismatches_vs_floating_table": exact.max_mismatches_vs_floating,
                "palette": pal.elements.to_vec(),
                "palette_identity": pal.identity,
            },
            "population": {
                "documents": uniq.len(), "collected": collected, "duplicates": duplicates,
                "fit_documents": fit_ids.len(), "tune_documents": tune_ids.len(), "dev_documents": dev_pool.len(),
                "fit_windows": fit_windows.len(),
                "consumed_fit_windows": windows.len(), "consumed_fit_targets": consumed_targets,
                "panel_documents": panel.docs.iter().filter(|(_, _, s)| *s > 0).count(),
                "panel_windows": panel.windows.len(), "panel_targets": panel.recs.len(),
            },
            "dose": {
                "updates_per_arm": args.total_updates, "batch": 8, "warmup": args.warmup,
                "seed": DOSE_SEED, "adam": [0.9, 0.999, 1e-8],
                "rw_lr": 0.03, "ab_lr": 0.003,
                "clips": ["global R/W block at 1", "post-Jacobian A/B block at 1"],
                "master_clamp": [-1.0, 1.0],
                "quantizer": "|master| >= 0.5 to sign, else 0",
            },
            "donor_map": {
                "strata": donor.strata, "eligible": donor.eligible.iter().filter(|x| **x).count(),
                "excluded_no_history": donor.excluded_no_history,
            },
            "arms": arm_rows,
            "screens": screens,
            "decision": {
                "retained_query_read_candidate": retained,
                "e_minus_q": {"point": e_minus_q.0, "lo": e_minus_q.1, "hi": e_minus_q.2},
                "s_minus_q": {"point": q_minus_s.0, "lo": q_minus_s.1, "hi": q_minus_s.2},
                "l_minus_q": {"point": q_minus_l.0, "lo": q_minus_l.1, "hi": q_minus_l.2},
                "note": "nominal repeated open-dev comparisons on correlated repository documents",
            },
            "continuation": continuation,
            "phases": phases,
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
            "donor_map": (0..panel.recs.len()).map(|i| json!({
                "obs": panel.recs[i].obs, "eligible": donor.eligible[i], "donor_index": donor.donor[i]
            })).collect::<Vec<_>>(),
        }),
    )?;

    seal(&args.root).map_err(|e| format!("seal: {e}"))?;
    let unlisted = verify(&args.root).map_err(|e| format!("verify: {e}"))?;
    println!(
        "sealed and verified ({} unlisted); elapsed {:.1}s",
        unlisted.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(ExitCode::SUCCESS)
}

/// Descriptive fit/tune bits over the declared curve panel, from the current hard core.
fn curve_bits(tr: &QueryTrainer, windows: &[Vec<u32>]) -> Result<f64, String> {
    let core = tr.hard_core()?;
    let mut total = 0.0f64;
    let mut n = 0usize;
    for w in windows {
        for (i, prev, cur, target) in targets(w, VOCAB) {
            let rows = core.read_path(w, i).map(|p| (p.first, p.second));
            let z = core.int_logits(prev, cur, rows);
            total += core.parent.bits_one(&z, target);
            n += 1;
        }
    }
    if n == 0 {
        return Err("empty curve panel".into());
    }
    Ok(total / n as f64)
}
