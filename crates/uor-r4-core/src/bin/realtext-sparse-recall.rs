//! Bounded real-text opportunity test for a sparse exact-pair episodic read.
//! The float probabilities are offline diagnostic scoring, not a serving kernel.
#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashMap};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::realtext_support::{
    ctx2, family_p, paired_interval, reconstruct_corpus, tune_lambdas, Agg, Cond, Split, Uni, VOCAB,
};
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const TOKENIZER_SHA: &str = "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";
const FIT_CAP: usize = 400_000;
const TUNE_CAP: usize = 100_000;
const DEV_DOCS: usize = 24;
const DEV_PER_DOC: usize = 4_096;
const PREFIX: usize = 8;
const ALPHAS: [u8; 4] = [0, 1, 2, 3];

#[derive(Clone, Serialize)]
struct Doc {
    path: String,
    sha256: String,
    tokens: Vec<u32>,
}

#[derive(Clone, Copy)]
struct Entry {
    value: u32,
    position: usize,
    hits: u32,
    conflicts: u32,
}

#[derive(Clone)]
struct Obs {
    doc: usize,
    position: usize,
    prev: u32,
    cur: u32,
    target: u32,
    count_p: f64,
    count_top: u32,
    hit: Option<Entry>,
    bucket: Option<usize>,
}

struct Counts {
    c1: Cond,
    c2: Cond,
    uni: Uni,
    one_successors: HashMap<u32, BTreeSet<u32>>,
    two_successors: HashMap<(u32, u32), BTreeSet<u32>>,
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn keep(path: &str) -> bool {
    !path.ends_with("realtext-sparse-recall-plan-2026-09-24.md")
        && !path.ends_with("realtext-sparse-recall-result-2026-09-24.md")
        && !path.ends_with("resource-ledger-2026-09-19.md")
}

fn corpus(
    root: &Path,
    tok: &HfBpeTokenizer,
) -> Result<(Vec<Doc>, Vec<Doc>, Vec<Doc>, usize, usize), String> {
    let (records, collected, duplicates) = reconstruct_corpus(root);
    let mut fit = Vec::new();
    let mut tune = Vec::new();
    let mut dev = Vec::new();
    let mut fit_total = 0usize;
    let mut tune_total = 0usize;
    for rec in records.into_iter().filter(|r| keep(&r.path)) {
        let destination = match rec.split {
            Split::Fit if fit_total < FIT_CAP => &mut fit,
            Split::Tune if tune_total < TUNE_CAP => &mut tune,
            Split::Dev if dev.len() < DEV_DOCS => &mut dev,
            _ => continue,
        };
        let mut tokens = tok.encode(&rec.text);
        let cap = match rec.split {
            Split::Fit => FIT_CAP.saturating_sub(fit_total),
            Split::Tune => TUNE_CAP.saturating_sub(tune_total),
            Split::Dev => DEV_PER_DOC,
        };
        tokens.truncate(cap);
        if tokens.len() <= PREFIX {
            continue;
        }
        match rec.split {
            Split::Fit => fit_total += tokens.len(),
            Split::Tune => tune_total += tokens.len(),
            Split::Dev => {}
        }
        destination.push(Doc {
            path: rec.path,
            sha256: hex::encode(rec.sha256),
            tokens,
        });
    }
    if fit.is_empty() || tune.is_empty() || dev.is_empty() {
        return Err("source split has an empty selected population".into());
    }
    Ok((fit, tune, dev, collected, duplicates))
}

fn fit_counts(docs: &[Doc]) -> Result<Counts, String> {
    let mut out = Counts {
        c1: Cond::default(),
        c2: Cond::default(),
        uni: Uni {
            counts: vec![0; VOCAB],
            total: 0,
        },
        one_successors: HashMap::new(),
        two_successors: HashMap::new(),
    };
    for doc in docs {
        for (i, &next) in doc.tokens.iter().enumerate() {
            let Some(count) = out.uni.counts.get_mut(next as usize) else {
                return Err(format!("out-of-vocabulary token {next} in {}", doc.path));
            };
            *count += 1;
            out.uni.total += 1;
            if i < 2 {
                continue;
            }
            let prev = doc.tokens[i - 2];
            let cur = doc.tokens[i - 1];
            out.c1.observe(cur as u64, next);
            out.c2.observe(ctx2(prev as usize, cur as usize), next);
            out.one_successors.entry(cur).or_default().insert(next);
            out.two_successors
                .entry((prev, cur))
                .or_default()
                .insert(next);
        }
    }
    Ok(out)
}

fn count_top(counts: &Counts, prev: u32, cur: u32, lambdas: (f64, f64)) -> u32 {
    let mut best = counts.uni.argmax();
    let mut score = family_p(
        &counts.c1,
        &counts.c2,
        &counts.uni,
        prev as usize,
        cur as usize,
        best,
        lambdas,
    );
    let one = counts.one_successors.get(&cur);
    let two = counts.two_successors.get(&(prev, cur));
    for candidate in one.into_iter().flatten().chain(two.into_iter().flatten()) {
        let p = family_p(
            &counts.c1,
            &counts.c2,
            &counts.uni,
            prev as usize,
            cur as usize,
            *candidate,
            lambdas,
        );
        if p > score || (p == score && *candidate < best) {
            best = *candidate;
            score = p;
        }
    }
    best
}

fn age_bucket(age: usize) -> usize {
    if age < 64 {
        0
    } else if age < 256 {
        1
    } else {
        2
    }
}

fn observe(docs: &[Doc], counts: &Counts, lambdas: (f64, f64)) -> Vec<Obs> {
    let mut rows = Vec::new();
    let mut top_cache: HashMap<(u32, u32), u32> = HashMap::new();
    for (doc_idx, doc) in docs.iter().enumerate() {
        let mut memory: HashMap<(u32, u32), Entry> = HashMap::new();
        for t in 2..doc.tokens.len() {
            let prev = doc.tokens[t - 2];
            let cur = doc.tokens[t - 1];
            let target = doc.tokens[t];
            let key = (prev, cur);
            let hit = memory.get(&key).copied();
            if t >= PREFIX {
                let count_p = family_p(
                    &counts.c1,
                    &counts.c2,
                    &counts.uni,
                    prev as usize,
                    cur as usize,
                    target,
                    lambdas,
                );
                let count_top = *top_cache
                    .entry(key)
                    .or_insert_with(|| count_top(counts, prev, cur, lambdas));
                rows.push(Obs {
                    doc: doc_idx,
                    position: t,
                    prev,
                    cur,
                    target,
                    count_p,
                    count_top,
                    hit,
                    bucket: hit.map(|h| age_bucket(t - h.position)),
                });
            }
            let next = match hit {
                Some(old) => Entry {
                    value: target,
                    position: t,
                    hits: old.hits + 1,
                    conflicts: old.conflicts + u32::from(old.value != target),
                },
                None => Entry {
                    value: target,
                    position: t,
                    hits: 1,
                    conflicts: 0,
                },
            };
            memory.insert(key, next);
        }
    }
    rows
}

fn mixed(p: f64, target: u32, candidate: u32, alpha_quarters: u8) -> f64 {
    let a = f64::from(alpha_quarters) / 4.0;
    (1.0 - a) * p + if target == candidate { a } else { 0.0 }
}

fn fitted_gate(tune: &[Obs]) -> ([u8; 3], [usize; 3]) {
    let mut losses = [[0.0; 4]; 3];
    let mut n = [0usize; 3];
    for row in tune {
        if let (Some(hit), Some(bucket)) = (row.hit, row.bucket) {
            n[bucket] += 1;
            for (j, &a) in ALPHAS.iter().enumerate() {
                losses[bucket][j] -= mixed(row.count_p, row.target, hit.value, a)
                    .max(1e-300)
                    .log2();
            }
        }
    }
    let mut chosen = [0u8; 3];
    for bucket in 0..3 {
        if n[bucket] < 32 {
            continue;
        }
        let best = (0..4)
            .min_by(|&a, &b| {
                losses[bucket][a]
                    .total_cmp(&losses[bucket][b])
                    .then(a.cmp(&b))
            })
            .unwrap_or(0);
        chosen[bucket] = ALPHAS[best];
    }
    (chosen, n)
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    std::fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{}: {e}", path.display()))
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        return Err(
            "usage: realtext-sparse-recall NEW_ROOT DOCS_DIR DERIVED_TOKENIZER.json".into(),
        );
    }
    let root = PathBuf::from(&args[1]);
    let docs_dir = PathBuf::from(&args[2]);
    let tok_path = PathBuf::from(&args[3]);
    claim(&root).map_err(|e| e.to_string())?;
    let tok_bytes = std::fs::read(&tok_path).map_err(|e| e.to_string())?;
    if sha(&tok_bytes) != TOKENIZER_SHA {
        return Err("tokenizer SHA-256 mismatch".into());
    }
    let tok =
        HfBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes).ok_or("invalid derived tokenizer")?;
    if tok.vocab_size() != VOCAB {
        return Err("tokenizer vocabulary mismatch".into());
    }
    let (fit, tune, dev, collected, duplicates) = corpus(&docs_dir, &tok)?;
    let counts = fit_counts(&fit)?;
    let tune_tokens: Vec<Vec<u32>> = tune.iter().map(|d| d.tokens.clone()).collect();
    let (lambdas, _) = tune_lambdas(&counts.c1, &counts.c2, &counts.uni, &tune_tokens);
    let tune_rows = observe(&tune, &counts, lambdas);
    let (gate, tune_hits) = fitted_gate(&tune_rows);
    let rows = observe(&dev, &counts, lambdas);
    if rows.is_empty() {
        return Err("no scored development positions".into());
    }
    let mut eligible_by_bucket = [Vec::new(), Vec::new(), Vec::new()];
    for (i, row) in rows.iter().enumerate() {
        if let Some(b) = row.bucket {
            eligible_by_bucket[b].push(i);
        }
    }
    let mut rotated = vec![None; rows.len()];
    for members in &eligible_by_bucket {
        for (j, &i) in members.iter().enumerate() {
            rotated[i] = rows[members[(j + 1) % members.len()]].hit.map(|h| h.value);
        }
    }
    let mut out =
        BufWriter::new(std::fs::File::create(root.join("rows.csv")).map_err(|e| e.to_string())?);
    writeln!(out,"doc,position,prev,cur,target,count_p,count_top,candidate,age,hits,conflicts,bucket,alpha_quarters,gate_p,gate_top,rotated_candidate,rotated_p")
        .map_err(|e|e.to_string())?;
    let mut count_loss = vec![0.0; dev.len()];
    let mut gate_loss = vec![0.0; dev.len()];
    let mut null_loss = vec![0.0; dev.len()];
    let mut doc_n = vec![0usize; dev.len()];
    let mut count_correct = 0usize;
    let mut gate_correct = 0usize;
    let mut eligible = [0usize; 3];
    let mut rescues = [0usize; 3];
    let mut harms = [0usize; 3];
    let mut conflict_hits = [0usize; 3];
    for (i, row) in rows.iter().enumerate() {
        let (
            candidate,
            age,
            hits,
            conflicts,
            bucket,
            alpha,
            gate_p,
            gate_top,
            null_candidate,
            null_p,
        ) = if let (Some(hit), Some(b)) = (row.hit, row.bucket) {
            eligible[b] += 1;
            if hit.conflicts > 0 {
                conflict_hits[b] += 1;
            }
            if hit.value == row.target && row.count_top != row.target {
                rescues[b] += 1;
            }
            if hit.value != row.target && row.count_top == row.target {
                harms[b] += 1;
            }
            let a = gate[b];
            let p = mixed(row.count_p, row.target, hit.value, a);
            let cbase = family_p(
                &counts.c1,
                &counts.c2,
                &counts.uni,
                row.prev as usize,
                row.cur as usize,
                hit.value,
                lambdas,
            );
            let stop = family_p(
                &counts.c1,
                &counts.c2,
                &counts.uni,
                row.prev as usize,
                row.cur as usize,
                row.count_top,
                lambdas,
            );
            let top = if (4 - u32::from(a)) as f64 * cbase + f64::from(a)
                > (4 - u32::from(a)) as f64 * stop
            {
                hit.value
            } else {
                row.count_top
            };
            let nc = rotated[i].unwrap_or(hit.value);
            let np = mixed(row.count_p, row.target, nc, a);
            (
                hit.value as i64,
                (row.position - hit.position) as i64,
                hit.hits as i64,
                hit.conflicts as i64,
                b as i64,
                a,
                p,
                top,
                nc as i64,
                np,
            )
        } else {
            (
                -1,
                -1,
                0,
                0,
                -1,
                0,
                row.count_p,
                row.count_top,
                -1,
                row.count_p,
            )
        };
        let cp = row.count_p.max(1e-300);
        let gp = gate_p.max(1e-300);
        let np = null_p.max(1e-300);
        count_loss[row.doc] -= cp.log2();
        gate_loss[row.doc] -= gp.log2();
        null_loss[row.doc] -= np.log2();
        doc_n[row.doc] += 1;
        count_correct += usize::from(row.count_top == row.target);
        gate_correct += usize::from(gate_top == row.target);
        writeln!(
            out,
            "{},{},{},{},{},{:.12},{},{},{},{},{},{},{},{:.12},{},{},{:.12}",
            row.doc,
            row.position,
            row.prev,
            row.cur,
            row.target,
            cp,
            row.count_top,
            candidate,
            age,
            hits,
            conflicts,
            bucket,
            alpha,
            gp,
            gate_top,
            null_candidate,
            np
        )
        .map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())?;
    let sum = |v: &[f64]| v.iter().sum::<f64>() / rows.len() as f64;
    let count_bits = sum(&count_loss);
    let gate_bits = sum(&gate_loss);
    let null_bits = sum(&null_loss);
    let count_agg = Agg {
        rows: dev
            .iter()
            .enumerate()
            .map(|(i, d)| (d.path.clone(), count_loss[i], doc_n[i]))
            .collect(),
    };
    let gate_agg = Agg {
        rows: dev
            .iter()
            .enumerate()
            .map(|(i, d)| (d.path.clone(), gate_loss[i], doc_n[i]))
            .collect(),
    };
    let interval = paired_interval(&gate_agg, &count_agg, 91240924);
    let mut families: HashMap<String, (f64, f64, usize)> = HashMap::new();
    for (i, d) in dev.iter().enumerate() {
        let family = d.path.split('/').next().unwrap_or("?").to_owned();
        let e = families.entry(family).or_default();
        e.0 += gate_loss[i];
        e.1 += count_loss[i];
        e.2 += doc_n[i];
    }
    let worst_family = families
        .values()
        .filter(|x| x.2 > 0)
        .map(|x| (x.0 - x.1) / x.2 as f64)
        .fold(f64::NEG_INFINITY, f64::max);
    let distant = eligible[1] + eligible[2];
    let distant_rescues = rescues[1] + rescues[2];
    let decision = distant as f64 / rows.len() as f64 >= 0.01
        && distant_rescues as f64 / rows.len() as f64 >= 0.005
        && gate_bits - count_bits <= -0.03
        && interval.2 < 0.0
        && gate_bits < null_bits
        && worst_family <= 0.2;
    let identity = |docs: &[Doc]| {
        docs.iter()
            .map(|d| json!({"path":d.path,"sha256":d.sha256,"tokens":d.tokens.len()}))
            .collect::<Vec<_>>()
    };
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let receipt = json!({
        "schema":"uor-r4.realtext-sparse-recall/1","plan":"docs/integration/realtext-sparse-recall-plan-2026-09-24.md",
        "decision":if decision{"ADVANCE_EXACT_PAIR_CACHE_COMPONENT"}else{"DO_NOT_ADVANCE_EXACT_PAIR_CACHE"},
        "population":{"collected":collected,"duplicate_groups":duplicates,"fit":identity(&fit),"tune":identity(&tune),"dev":identity(&dev),
            "scored_positions":rows.len(),"retained_dev_population_exposed":true},
        "tokenizer":{"path":tok_path,"sha256":TOKENIZER_SHA},
        "fit":{"lambdas":lambdas,"age_gate_quarters":gate,"tune_bucket_hits":tune_hits},
        "dev":{"count_bits":count_bits,"gate_bits":gate_bits,"rotated_null_bits":null_bits,
            "gate_minus_count_bits":gate_bits-count_bits,"gate_minus_count_95pct_doc_interval":[interval.1,interval.2],
            "count_top1":count_correct,"gate_top1":gate_correct,"eligible_by_age":eligible,
            "rescues_by_age":rescues,"harms_by_age":harms,"conflict_hits_by_age":conflict_hits,
            "distant_eligible_fraction":distant as f64/rows.len() as f64,
            "distant_rescue_fraction":distant_rescues as f64/rows.len() as f64,
            "family_gate_minus_count":families.iter().map(|(k,x)|(k.clone(),(x.0-x.1)/x.2 as f64)).collect::<HashMap<_,_>>()},
        "source_sha256":sha(include_bytes!("realtext-sparse-recall.rs")),
        "executable_sha256":sha(&std::fs::read(exe).map_err(|e|e.to_string())?),
        "scope":"Exact pair key and last successor; Tune-fitted dyadic age gate; diagnostic on retained project Markdown. No loaded TlModel, geometry comparator, general language, or whole-model serving result."
    });
    write_json(&root.join("receipt.json"), &receipt)?;
    seal(&root).map_err(|e| e.to_string())?;
    let unlisted = verify(&root).map_err(|e| e.to_string())?;
    if !unlisted.is_empty() {
        return Err(format!("unlisted files: {unlisted:?}"));
    }
    println!("decision={} rows={} count_bits={:.6} gate_bits={:.6} null_bits={:.6} gate={gate:?} distant={distant} distant_rescues={distant_rescues}",receipt["decision"],rows.len(),count_bits,gate_bits,null_bits);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(2);
    }
}
