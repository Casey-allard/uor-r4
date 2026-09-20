//! Thin real-text prior-only learning curve: the first corrected measured run on documents.
//!
//! One arm, memory disabled, frozen fit-only low-bit bias. Claims an exclusive report root before
//! loading anything, binds source/config/tokenizer/data identities, fits one configuration at
//! checkpoints 0/256/512, evaluates the frozen development set with the predetermined gate and the
//! fixed within-document context permutation, generates greedy continuations and seals the attempt.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::prior_learning::{
    bias_codes_from_counts, targets, Config, PriorCore, PriorTrainer,
};
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::bpe_derive::derive_tokenizer;

const VOCAB: usize = 4096;
const DV: usize = 128;
const WINDOW: usize = 64;
const BATCH: usize = 8;
const SEED: u64 = 13;
const OUTPUT_MASTER_BOUND: f32 = 0.45;
const CHECKPOINTS: [u64; 3] = [0, 256, 512];
const GATE_MARGIN_BITS: f64 = 0.10;
const BOOTSTRAP_DRAWS: usize = 2000;
const DEV_MAX_DOCS: usize = 48;
const DEV_MAX_WINDOWS: usize = 96;
const GEN_TOKENS: usize = 64;
const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json";
const EXPECTED_TOKENIZER_SHA256: &str =
    "9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c";

struct Args {
    root: PathBuf,
    docs: PathBuf,
    tokenizer: PathBuf,
    source_rev: String,
}

fn parse_args() -> Result<Args, String> {
    let mut root = None;
    let mut docs = None;
    let mut tokenizer = PathBuf::from(DEFAULT_TOKENIZER);
    let mut source_rev = String::from("unknown");
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
            "--docs" => docs = Some(PathBuf::from(v()?)),
            "--tokenizer" => tokenizer = PathBuf::from(v()?),
            "--source-rev" => source_rev = v()?,
            other => return Err(format!("unknown argument {other}")),
        }
        i += 2;
    }
    Ok(Args {
        root: root.ok_or("--root is required")?,
        docs: docs.ok_or("--docs is required")?,
        tokenizer,
        source_rev,
    })
}

/// A whole document with its content hash.
struct Doc {
    path: String,
    hash: [u8; 32],
    text: String,
}

fn collect_docs(dir: &Path, rel: &Path, out: &mut Vec<Doc>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut kids: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    kids.sort();
    for k in kids {
        let r = rel.join(k.file_name().unwrap_or_default());
        if k.is_dir() {
            collect_docs(&k, &r, out);
        } else if k.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(text) = std::fs::read_to_string(&k) {
                let hash: [u8; 32] = Sha256::digest(text.as_bytes()).into();
                out.push(Doc {
                    path: r.display().to_string(),
                    hash,
                    text,
                });
            }
        }
    }
}

fn windows_of(tokens: &[u32]) -> Vec<Vec<u32>> {
    tokens
        .chunks(WINDOW)
        .filter(|c| c.len() >= 3)
        .map(|c| c.to_vec())
        .collect()
}

/// Per-document loss sums and counts, so intervals resample documents rather than tokens.
#[derive(Default, Clone)]
struct Eval {
    per_doc: Vec<(f64, usize)>,
}

impl Eval {
    fn total(&self) -> (f64, usize) {
        self.per_doc
            .iter()
            .fold((0.0, 0usize), |(a, b), (l, n)| (a + l, b + n))
    }
    fn micro(&self) -> f64 {
        let (l, n) = self.total();
        if n == 0 {
            f64::NAN
        } else {
            l / n as f64
        }
    }
    fn doc_macro(&self) -> f64 {
        let docs: Vec<f64> = self
            .per_doc
            .iter()
            .filter(|(_, n)| *n > 0)
            .map(|(l, n)| l / *n as f64)
            .collect();
        if docs.is_empty() {
            f64::NAN
        } else {
            docs.iter().sum::<f64>() / docs.len() as f64
        }
    }
}

fn evaluate(
    core: &PriorCore,
    dev: &[(usize, Vec<u32>)],
    pairs: Option<&HashMap<(usize, u32, u32), u32>>,
    use_context: bool,
) -> Eval {
    let mut e = Eval::default();
    for (doc, w) in dev {
        let mut loss = 0f64;
        let mut n = 0usize;
        for (_i, prev, cur, target) in targets(w, VOCAB) {
            let (prev, target) = match pairs {
                Some(map) => match map.get(&(*doc, prev as u32, cur as u32)) {
                    Some(&t) => (prev, t),
                    None => (prev, target),
                },
                None => (prev, target),
            };
            let tr = core.trace(prev, cur, use_context);
            loss += core.bits_one(&tr.z, target);
            n += 1;
        }
        e.per_doc.push((loss, n));
    }
    e
}

/// Paired document-cluster bootstrap of a loss *difference* (a minus b), recomputing the ratio from
/// resampled sums and counts.
fn paired_interval(a: &Eval, b: &Eval, seed: u64) -> (f64, f64, f64) {
    let n = a.per_doc.len().min(b.per_doc.len());
    if n == 0 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    let d: Vec<(f64, f64, f64, f64)> = (0..n)
        .map(|i| {
            let (la, na) = a.per_doc[i];
            let (lb, nb) = b.per_doc[i];
            (la, na as f64, lb, nb as f64)
        })
        .collect();
    let point = (d.iter().map(|x| x.0).sum::<f64>() / d.iter().map(|x| x.1).sum::<f64>())
        - (d.iter().map(|x| x.2).sum::<f64>() / d.iter().map(|x| x.3).sum::<f64>());
    let mut st = seed | 1;
    let mut boots = Vec::with_capacity(BOOTSTRAP_DRAWS);
    for _ in 0..BOOTSTRAP_DRAWS {
        let (mut sa, mut na, mut sb, mut nb) = (0f64, 0f64, 0f64, 0f64);
        for _ in 0..n {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            let (la, n1, lb, n2) = d[(st as usize) % n];
            sa += la;
            na += n1;
            sb += lb;
            nb += n2;
        }
        if na > 0.0 && nb > 0.0 {
            boots.push(sa / na - sb / nb);
        }
    }
    if boots.len() < BOOTSTRAP_DRAWS / 2 {
        return (point, f64::NAN, f64::NAN);
    }
    boots.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let lo = boots[(boots.len() as f64 * 0.025) as usize];
    let hi = boots[(boots.len() as f64 * 0.975) as usize];
    (point, lo, hi)
}

fn greedy(core: &PriorCore, prompt: &[u32], n_new: usize, use_context: bool) -> Vec<u32> {
    core.generate(prompt, n_new, use_context)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    // Claim the exclusive report root immediately after argument validation.
    if let Err(e) = claim(&args.root) {
        eprintln!("error: claim {}: {e}", args.root.display());
        return ExitCode::from(1);
    }
    let started = Instant::now();
    println!("=== real-text prior-only learning curve ===");
    println!(
        "config V={VOCAB} dv={DV} F=10 norm_bits=6 window={WINDOW} batch={BATCH} seed={SEED} memory=disabled"
    );
    println!("source_rev={}", args.source_rev);

    // --- tokenizer identity -------------------------------------------------
    let tok_bytes = match std::fs::read(&args.tokenizer) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: tokenizer: {e}");
            return ExitCode::from(1);
        }
    };
    let tok_sha = hex::encode(Sha256::digest(&tok_bytes));
    if tok_sha != EXPECTED_TOKENIZER_SHA256 {
        eprintln!("error: tokenizer sha256 {tok_sha} != expected {EXPECTED_TOKENIZER_SHA256}");
        return ExitCode::from(1);
    }
    let tokenizer = match derive_tokenizer(&tok_bytes, VOCAB) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: derive tokenizer: {e}");
            return ExitCode::from(1);
        }
    };
    let derived_bytes =
        match uor_r4_core::transformerless::bpe_derive::derive_tokenizer_json(&tok_bytes, VOCAB) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("error: derive bytes: {e}");
                return ExitCode::from(1);
            }
        };
    let derived_sha = hex::encode(Sha256::digest(&derived_bytes));
    let _ = std::fs::write(args.root.join("tokenizer_source.json"), &tok_bytes);
    let _ = std::fs::write(
        args.root.join("tokenizer_derived_v4096.json"),
        &derived_bytes,
    );
    println!("tokenizer source {tok_sha}\ntokenizer derived {derived_sha}");

    // --- corpus -------------------------------------------------------------
    let mut docs: Vec<Doc> = Vec::new();
    collect_docs(&args.docs, Path::new(""), &mut docs);
    let mut seen: HashMap<[u8; 32], usize> = HashMap::new();
    let mut uniq: Vec<Doc> = Vec::new();
    let mut duplicates = 0usize;
    for d in docs {
        if seen.insert(d.hash, uniq.len()).is_some() {
            duplicates += 1;
            continue;
        }
        uniq.push(d);
    }
    // Deterministic split by content hash: buckets 0-7 fit, bucket 8 tune, bucket 9 development.
    let fit_ids: Vec<usize> = (0..uniq.len())
        .filter(|i| (uniq[*i].hash[0] as usize) % 10 < 8)
        .collect();
    let dev_pool: Vec<usize> = (0..uniq.len())
        .filter(|i| (uniq[*i].hash[0] as usize) % 10 == 9)
        .collect();
    println!(
        "corpus docs={} exact_duplicates_grouped={} fit_docs={} dev_pool={}",
        uniq.len(),
        duplicates,
        fit_ids.len(),
        dev_pool.len()
    );

    // Dev selection: stratify by document token length so short/medium/long are represented.
    let mut with_len: Vec<(usize, usize)> = dev_pool
        .iter()
        .map(|i| (*i, tokenizer.encode(&uniq[*i].text).len()))
        .collect();
    with_len.sort_by_key(|(_, n)| *n);
    let mut dev_docs: Vec<usize> = Vec::new();
    if !with_len.is_empty() {
        let take = DEV_MAX_DOCS.min(with_len.len());
        for k in 0..take {
            let idx = k * with_len.len() / take;
            dev_docs.push(with_len[idx].0);
        }
    }
    dev_docs.sort_unstable();
    dev_docs.dedup();
    let mut dev: Vec<(usize, Vec<u32>)> = Vec::new();
    for (slot, i) in dev_docs.iter().enumerate() {
        let toks = tokenizer.encode(&uniq[*i].text);
        for w in windows_of(&toks)
            .into_iter()
            .take(DEV_MAX_WINDOWS.div_ceil(dev_docs.len().max(1)))
        {
            dev.push((slot, w));
        }
    }
    if dev.len() > DEV_MAX_WINDOWS {
        dev.truncate(DEV_MAX_WINDOWS);
    }
    let dev_targets: usize = dev.iter().map(|(_, w)| w.len() - 1).sum();
    println!(
        "development docs={} windows={} scored_targets={}",
        dev_docs.len(),
        dev.len(),
        dev_targets
    );

    // Fit windows.
    let mut fit_windows: Vec<Vec<u32>> = Vec::new();
    for i in &fit_ids {
        let toks = tokenizer.encode(&uniq[*i].text);
        fit_windows.extend(windows_of(&toks));
    }
    let fit_targets: usize = fit_windows.iter().map(|w| w.len() - 1).sum();
    println!(
        "fit windows={} scored_targets={} (distinct documents={})",
        fit_windows.len(),
        fit_targets,
        fit_ids.len()
    );

    // --- frozen fit-only marginal, counts from zero ------------------------
    let mut counts = vec![0u64; VOCAB];
    let mut total = 0u64;
    for w in &fit_windows {
        for (_i, _p, _c, t) in targets(w, VOCAB) {
            counts[t as usize] += 1;
            total += 1;
        }
    }
    assert_eq!(
        counts.iter().sum::<u64>(),
        total,
        "sum(counts) must equal the declared fit target count"
    );
    assert_eq!(total as usize, fit_targets);
    let bias = bias_codes_from_counts(&counts, total, VOCAB);
    println!("fit unigram counted over {total} targets (add-one applied once at read time)");

    // --- timing probe on the real path -------------------------------------
    let cfg = Config::new(VOCAB, DV);
    let mut probe = match PriorTrainer::new(cfg.clone(), SEED, bias.clone(), OUTPUT_MASTER_BOUND) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: trainer: {e}");
            return ExitCode::from(1);
        }
    };
    let batch0: Vec<Vec<u32>> = fit_windows.iter().take(BATCH).cloned().collect();
    let t0 = Instant::now();
    let _ = probe.train_batch_bits(&batch0);
    let cold = t0.elapsed().as_secs_f64();
    let t1 = Instant::now();
    for _ in 0..3 {
        let b = probe.next_batch(&fit_windows, BATCH);
        probe.train_batch_bits(&b);
    }
    let warm = t1.elapsed().as_secs_f64() / 3.0;
    println!("timing: cold step {cold:.4}s, warm step mean {warm:.4}s over 3 steps (probe state discarded)");
    let scored_per_step = BATCH * (WINDOW - 1);
    println!(
        "timing: {scored_per_step} scored targets/step; 512 steps ~= {} s of fitting",
        (warm * 512.0) as u64
    );

    // --- the one learned arm ------------------------------------------------
    let mut t = match PriorTrainer::new(cfg.clone(), SEED, bias.clone(), OUTPUT_MASTER_BOUND) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: trainer: {e}");
            return ExitCode::from(1);
        }
    };
    t.data_identity =
        Sha256::digest(format!("{tok_sha}:{derived_sha}:{}", args.source_rev).as_bytes()).into();

    // Frozen permutation control: shuffle (prev,cur) pairs within each development document,
    // preserving the context marginal and the target list, and the PAD stratum separately.
    let mut perm_map: HashMap<(usize, u32, u32), u32> = HashMap::new();
    let mut perm_seed = 0xA5A5_1234u64;
    let mut perm_nonpad = 0usize;
    let mut perm_pad = 0usize;
    for (doc, w) in &dev {
        let mut ctx: Vec<(usize, usize, usize, u32)> = Vec::new();
        let mut pad_ctx: Vec<(usize, usize, usize, u32)> = Vec::new();
        for (i, prev, cur, target) in targets(w, VOCAB) {
            if i == 0 {
                pad_ctx.push((i, prev, cur, target));
            } else {
                ctx.push((i, prev, cur, target));
            }
        }
        let pad_len = pad_ctx.len();
        for group in [&mut ctx, &mut pad_ctx] {
            if group.len() < 2 {
                continue;
            }
            let targets_only: Vec<u32> = group.iter().map(|x| x.3).collect();
            let mut shuffled_targets = targets_only.clone();
            for i in (1..shuffled_targets.len()).rev() {
                perm_seed ^= perm_seed << 13;
                perm_seed ^= perm_seed >> 7;
                perm_seed ^= perm_seed << 17;
                let j = (perm_seed as usize) % (i + 1);
                shuffled_targets.swap(i, j);
            }
            for (k, (_, prev, cur, original)) in group.iter().enumerate() {
                let t = shuffled_targets[k];
                if t != *original {
                    if k == 0 && group.len() == pad_len {
                        perm_pad += 1;
                    } else {
                        perm_nonpad += 1;
                    }
                }
                perm_map.insert((*doc, *prev as u32, *cur as u32), t);
            }
        }
    }
    println!(
        "permutation: {} changed context-target associations ({} non-PAD, {} PAD); mapping frozen before fitting",
        perm_nonpad + perm_pad,
        perm_nonpad,
        perm_pad
    );

    // Exact smoothed unigram reference, computed directly.
    let mut uni_eval = Eval::default();
    for (doc, w) in &dev {
        let mut loss = 0f64;
        let mut n = 0usize;
        for (_i, _p, _c, target) in targets(w, VOCAB) {
            let p = (counts[target as usize] as f64 + 1.0) / (total as f64 + VOCAB as f64);
            loss += -p.log2();
            n += 1;
        }
        uni_eval.per_doc.push((loss, n));
        let _ = doc;
    }

    let mut curve = Vec::new();
    let mut ctx_doc_macro;
    let mut finals: Vec<PriorCore> = Vec::new();
    for cp in CHECKPOINTS {
        while t.step < cp {
            let b = t.next_batch(&fit_windows, BATCH);
            if b.is_empty() {
                break;
            }
            t.train_batch_bits(&b);
        }
        let core = t.to_core().expect("core");
        let ctx = evaluate(&core, &dev, None, true);
        let bias_only = evaluate(&core, &dev, None, false);
        let perm = evaluate(&core, &dev, Some(&perm_map), true);
        // Position knockouts: zero one embedding table before the shared nonlinearity.
        let mut prev_off = core.clone();
        let mut cur_off = core.clone();
        let zero_rows = |rows: usize| vec![0f32; rows * DV];
        prev_off.e_old = uor_r4_core::native_geometric::learner::TernaryLinear::quantize(
            &zero_rows(VOCAB + 1),
            VOCAB + 1,
            DV,
        );
        cur_off.e_new = uor_r4_core::native_geometric::learner::TernaryLinear::quantize(
            &zero_rows(VOCAB),
            VOCAB,
            DV,
        );
        let prev_k = evaluate(&prev_off, &dev, None, true);
        let cur_k = evaluate(&cur_off, &dev, None, true);
        ctx_doc_macro = ctx.doc_macro();
        // `paired_interval(a, b)` returns the paired difference a - b, so passing the reference first makes
        // a positive value mean "the reference is worse", i.e. a gain for the model. The earlier build had
        // the arguments the other way round, which only mislabelled the verdict.
        let (u_pt, u_lo, u_hi) = paired_interval(&uni_eval, &ctx, 0x1234_5678);
        let (b_pt, b_lo, b_hi) = paired_interval(&bias_only, &ctx, 0x9E37_79B9);
        let (p_pt, p_lo, p_hi) = paired_interval(&perm, &ctx, 0xD1B5_4A32);
        println!(
            "step {cp:>4}: dev ctx micro {:.4} macro {:.4} | bias-only {:.4} | unigram {:.4} | perm {:.4} | knockouts prev {:.4} cur {:.4}",
            ctx.micro(),
            ctx.doc_macro(),
            bias_only.micro(),
            uni_eval.micro(),
            perm.micro(),
            prev_k.micro(),
            cur_k.micro()
        );
        println!(
            "          gain vs unigram {u_pt:+.4} [{u_lo:+.4},{u_hi:+.4}] | gain vs quantized bias {b_pt:+.4} [{b_lo:+.4},{b_hi:+.4}] | perm penalty {p_pt:+.4} [{p_lo:+.4},{p_hi:+.4}]"
        );
        curve.push(serde_json::json!({
            "step": cp,
            "dev_micro_bits": ctx.micro(), "dev_macro_bits": ctx_doc_macro,
            "bias_only_micro_bits": bias_only.micro(), "exact_unigram_micro_bits": uni_eval.micro(),
            "permuted_micro_bits": perm.micro(),
            "knockout_prev_micro_bits": prev_k.micro(), "knockout_cur_micro_bits": cur_k.micro(),
            "gain_vs_unigram": {"point": u_pt, "lo": u_lo, "hi": u_hi},
            "gain_vs_bias": {"point": b_pt, "lo": b_lo, "hi": b_hi},
            "perm_penalty": {"point": p_pt, "lo": p_lo, "hi": p_hi},
            "per_doc_loss_sums": ctx.per_doc,
        }));
        finals.push(core);
    }

    // --- predetermined step-512 gate ---------------------------------------
    let last = curve.last().unwrap();
    let g1 = last["gain_vs_unigram"]["point"].as_f64().unwrap();
    let g1lo = last["gain_vs_unigram"]["lo"].as_f64().unwrap();
    let g2 = last["gain_vs_bias"]["point"].as_f64().unwrap();
    let g2lo = last["gain_vs_bias"]["lo"].as_f64().unwrap();
    let pp = last["perm_penalty"]["point"].as_f64().unwrap();
    let pplo = last["perm_penalty"]["lo"].as_f64().unwrap();
    let gate = g1 >= GATE_MARGIN_BITS
        && g1lo > 0.0
        && g2 >= GATE_MARGIN_BITS
        && g2lo > 0.0
        && pp > 0.0
        && pplo > 0.0;
    println!(
        "STEP-512 GATE: unigram gain {} | bias gain {} | permutation penalty {} | OVERALL {}",
        if g1 >= GATE_MARGIN_BITS && g1lo > 0.0 {
            "PASS"
        } else {
            "FAIL"
        },
        if g2 >= GATE_MARGIN_BITS && g2lo > 0.0 {
            "PASS"
        } else {
            "FAIL"
        },
        if pp > 0.0 && pplo > 0.0 {
            "PASS"
        } else {
            "FAIL"
        },
        if gate { "PASS" } else { "FAIL" }
    );

    // --- generation ---------------------------------------------------------
    let mut gens = Vec::new();
    for (k, (doc, w)) in dev.iter().take(3).enumerate() {
        let prompt: Vec<u32> = w[..16.min(w.len())].to_vec();
        for (cp, core) in CHECKPOINTS.iter().zip(finals.iter()) {
            let out = greedy(core, &prompt, GEN_TOKENS, true);
            let off = greedy(core, &prompt, GEN_TOKENS, false);
            let dec = tokenizer.decode(&out);
            println!(
                "gen step {cp} prompt#{k} doc#{doc}: {} new ids | decoded {:?}",
                out.len(),
                &dec[..dec.len().min(120)]
            );
            gens.push(serde_json::json!({
                "step": cp, "prompt_index": k, "doc_slot": doc,
                "prompt_ids": prompt, "output_ids": out, "decoded": dec,
                "context_disabled_ids": off,
            }));
        }
    }

    // --- persist, seal, verify ---------------------------------------------
    let result = serde_json::json!({
        "schema": "uor-r4.realtext-prior/1",
        "source_rev": args.source_rev,
        "tokenizer": {"source_sha256": tok_sha, "derived_sha256": derived_sha, "vocab": VOCAB},
        "config": {"dv": DV, "window": WINDOW, "batch": BATCH, "seed": SEED, "f_bits": cfg.f_bits, "norm_bits": cfg.norm_bits, "bias_scale_bits": cfg.bias_scale_bits, "output_master_bound": OUTPUT_MASTER_BOUND, "memory": "disabled"},
        "data": {"fit_docs": fit_ids.len(), "fit_windows": fit_windows.len(), "fit_targets": fit_targets, "dev_docs": dev_docs.len(), "dev_windows": dev.len(), "dev_targets": dev_targets, "exact_duplicates_grouped": duplicates},
        "gate_frozen": {"margin_bits": GATE_MARGIN_BITS, "bootstrap_draws": BOOTSTRAP_DRAWS, "checkpoint":"512"},
        "timing": {"cold_step_s": cold, "warm_step_s": warm, "scored_targets_per_step": scored_per_step},
        "curve": curve,
        "gate": {"unigram_gain": g1, "quantized_gain": g2, "perm_penalty": pp, "passed": gate},
    });
    if let Err(e) = std::fs::write(
        args.root.join("result.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    ) {
        eprintln!("error: result: {e}");
        return ExitCode::from(1);
    }
    if let Err(e) = std::fs::write(
        args.root.join("generation.json"),
        serde_json::to_string_pretty(&gens).unwrap(),
    ) {
        eprintln!("error: generation: {e}");
        return ExitCode::from(1);
    }
    let core = finals.last().unwrap();
    let artifact = core.to_bytes(&t.data_identity, SEED);
    let ckpt = t.checkpoint_bytes();
    let _ = std::fs::write(args.root.join("prior_realtext.cpl2"), &artifact);
    let _ = std::fs::write(args.root.join("prior_realtext.ckpt"), &ckpt);
    println!(
        "artifact {} bytes | checkpoint {} bytes | elapsed {:.1}s",
        artifact.len(),
        ckpt.len(),
        started.elapsed().as_secs_f32()
    );
    match seal(&args.root) {
        Ok(m) => println!("sealed {}", m.display()),
        Err(e) => {
            eprintln!("error: seal: {e}");
            return ExitCode::from(1);
        }
    }
    match verify(&args.root) {
        Ok(u) => println!("verified ({} unlisted)", u.len()),
        Err(e) => {
            eprintln!("error: verify: {e}");
            return ExitCode::from(1);
        }
    }
    ExitCode::SUCCESS
}
