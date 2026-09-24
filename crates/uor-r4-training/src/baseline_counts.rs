//! Train-only count fitting and a prospectively split count/cache comparison.
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::report_output;

use crate::baseline_protocol::{base_report, read_tokens, save_json, Evaluator};
use crate::ngram::{KneserNey5Gram, NgramConfig, TokenCache};
use crate::{invalid, sha256_file, Result};

const DISCOUNTS: [f64; 3] = [0.5, 0.75, 0.9];
const MIXTURES: [f64; 6] = [0.0, 0.05, 0.1, 0.2, 0.4, 0.6];
const CONTEXT: usize = 256;
const TUNE_BLOCKS: usize = 64;
const TOTAL_BLOCKS: usize = 976;

pub fn run_fit(protocol: &Evaluator, out: &Path) -> Result<()> {
    let started = Instant::now();
    let sources = protocol.document["train_sources"]
        .as_array()
        .ok_or_else(|| invalid("missing train sources"))?;
    if sources.len() != 2 {
        return Err(invalid("requires both retained train stores"));
    }
    let sequences = sources
        .iter()
        .map(read_tokens)
        .collect::<Result<Vec<_>>>()?;
    if sequences.iter().map(Vec::len).collect::<Vec<_>>() != [30_000_000, 119_996_416] {
        return Err(invalid("unexpected training population"));
    }
    let slices: Vec<_> = sequences.iter().map(Vec::as_slice).collect();
    let configuration = NgramConfig {
        vocab_size: 4096,
        discount: 0.75,
        unigram_alpha: 0.0001,
        max_train_tokens: 150_000_000,
        row_caps: [4_000_000, 4_000_000, 8_000_000, 8_000_000],
    };
    if protocol.document["ngram"]["row_caps"] != json!(configuration.row_caps)
        || protocol.document["ngram"]["unigram_alpha"] != configuration.unigram_alpha
    {
        return Err(invalid("ngram settings differ from the evaluator"));
    }
    let fit_started = Instant::now();
    let (model, fit) = KneserNey5Gram::fit_with_progress(&slices, configuration, |progress| {
        eprintln!(
            "count fit order {}: {} retained of {} types; {:.2}s",
            progress.completed_order,
            progress.retained_types,
            progress.raw_types,
            fit_started.elapsed().as_secs_f64()
        );
    })
    .map_err(|error| invalid(error.to_string()))?;
    let fit_seconds = fit_started.elapsed().as_secs_f64();
    if model.model_bytes() > 700 * 1024 * 1024 {
        return Err(invalid("count model exceeds declared retained storage"));
    }
    // Retain selected real-input probabilities before dropping the fitted model.
    let mut probes = Vec::new();
    for sequence in &sequences {
        for position in 1usize..=32 {
            let history = sequence[position.saturating_sub(4)..position].to_vec();
            let target = sequence[position];
            let probability = model
                .probability(&history, target)
                .map_err(|error| invalid(error.to_string()))?;
            probes.push((history, target, probability));
        }
    }
    let model_path = out.join("model.ng5");
    model
        .save(&model_path)
        .map_err(|error| invalid(error.to_string()))?;
    drop(model);
    drop(slices);
    drop(sequences);
    let restored = KneserNey5Gram::load(&model_path).map_err(|error| invalid(error.to_string()))?;
    let mut equal = 0;
    for (history, target, before) in &probes {
        let after = restored
            .probability(history, *target)
            .map_err(|error| invalid(error.to_string()))?;
        if before.to_bits() != after.to_bits() {
            return Err(invalid(
                "count export/reload changed an actual-input probability",
            ));
        }
        equal += 1;
    }
    let mut report = base_report(protocol, "ngram-fit")?;
    report["training_sources"] = protocol.document["train_sources"].clone();
    report["fit"] = serde_json::to_value(fit)?;
    report["model_sha256"] = json!(sha256_file(&model_path)?);
    report["model_bytes"] = json!(fs::metadata(&model_path)?.len());
    report["reload_probability_probes"] = json!(equal);
    report["fit_seconds"] = json!(fit_seconds);
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["status"] = json!("FIT_EXPORTED_RELOADED");
    save_json(&out.join("ngram-fit.json"), &report)?;
    eprintln!("count fit complete: {fit_seconds:.2}s, {equal} exact reload probes");
    Ok(())
}

fn probability(model: &KneserNey5Gram, history: &[u16], target: u16, discount: f64) -> Result<f64> {
    let value = model
        .probability_with_discount(history, target, discount)
        .map_err(|error| invalid(error.to_string()))?;
    if !value.is_finite() || value <= 0.0 || value > 1.0 + 1e-12 {
        return Err(invalid("count probability is not in (0,1]"));
    }
    Ok(value)
}

fn lowest(values: &[f64]) -> Result<usize> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err(invalid("nonfinite calibration objective"));
    }
    Ok(values
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.total_cmp(b.1).then(a.0.cmp(&b.0)))
        .ok_or_else(|| invalid("empty calibration grid"))?
        .0)
}

pub fn run_evaluate(protocol: &Evaluator, fit_root: &Path, out: &Path) -> Result<()> {
    let started = Instant::now();
    report_output::verify(fit_root)?;
    let fitted: Value = serde_json::from_slice(&fs::read(fit_root.join("ngram-fit.json"))?)?;
    let model_path = fit_root.join("model.ng5");
    if fitted["evaluator_sha256"] != protocol.sha256
        || fitted["model_sha256"] != sha256_file(&model_path)?
    {
        return Err(invalid("count artifact does not bind this evaluator"));
    }
    let model = KneserNey5Gram::load(&model_path).map_err(|error| invalid(error.to_string()))?;
    let tokens = read_tokens(&protocol.document["dev_source"])?;
    if tokens.len() != 250_000 || (tokens.len() - 1) / CONTEXT != TOTAL_BLOCKS {
        return Err(invalid("development population differs"));
    }
    // The parameter choice is completed on the fixed prefix before later blocks.
    let mut discount_losses = [0.0; DISCOUNTS.len()];
    for block in 0..TUNE_BLOCKS {
        let start = block * CONTEXT;
        for position in 0..CONTEXT {
            let history = &tokens[start + position.saturating_sub(3)..=start + position];
            let target = tokens[start + position + 1];
            for (index, &discount) in DISCOUNTS.iter().enumerate() {
                discount_losses[index] -= probability(&model, history, target, discount)?.ln();
            }
        }
    }
    let selected_discount = DISCOUNTS[lowest(&discount_losses)?];
    let mut cache_losses = [0.0; MIXTURES.len()];
    let mut cache = TokenCache::new(4096, CONTEXT).map_err(|error| invalid(error.to_string()))?;
    for block in 0..TUNE_BLOCKS {
        cache.reset();
        let start = block * CONTEXT;
        for position in 0..CONTEXT {
            cache
                .observe(tokens[start + position])
                .map_err(|error| invalid(error.to_string()))?;
            let history = &tokens[start + position.saturating_sub(3)..=start + position];
            let target = tokens[start + position + 1];
            let base = probability(&model, history, target, selected_discount)?;
            for (index, &mixture) in MIXTURES.iter().enumerate() {
                cache_losses[index] -= cache
                    .mixed_probability(base, target, mixture)
                    .map_err(|error| invalid(error.to_string()))?
                    .ln();
            }
        }
    }
    let selected_mixture = MIXTURES[lowest(&cache_losses)?];
    let selection = json!({"discount_grid":DISCOUNTS,"discount_tune_nll_sums":discount_losses,
        "selected_discount":selected_discount,"mixture_grid":MIXTURES,
        "mixture_tune_nll_sums":cache_losses,"selected_mixture":selected_mixture,
        "tune_blocks":TUNE_BLOCKS,"tune_targets":TUNE_BLOCKS*CONTEXT,
        "selection_scope":"Shared-D cache comparison; choices frozen before later blocks. Entire population is historical neural development."});
    save_json(&out.join("count-selection.json"), &selection)?;
    let mut predictions = BufWriter::new(fs::File::create_new(out.join("count-targets.csv"))?);
    writeln!(predictions,"block,position,target_offset,input,target,ngram_probability,cache_probability,ngram_nll_nats,cache_nll_nats")?;
    let mut blocks = BufWriter::new(fs::File::create_new(out.join("count-blocks.jsonl"))?);
    let mut sums = [[0.0; 2]; 2];
    for block in 0..TOTAL_BLOCKS {
        cache.reset();
        let start = block * CONTEXT;
        let mut block_sums = [0.0; 2];
        for position in 0..CONTEXT {
            cache
                .observe(tokens[start + position])
                .map_err(|error| invalid(error.to_string()))?;
            let history = &tokens[start + position.saturating_sub(3)..=start + position];
            let target = tokens[start + position + 1];
            let base = probability(&model, history, target, selected_discount)?;
            let mixed = cache
                .mixed_probability(base, target, selected_mixture)
                .map_err(|error| invalid(error.to_string()))?;
            let nll = [-base.ln(), -mixed.ln()];
            if nll
                .iter()
                .any(|value| !value.is_finite() || *value < -1e-12)
            {
                return Err(invalid("invalid count/cache likelihood"));
            }
            for index in 0..2 {
                block_sums[index] += nll[index];
            }
            writeln!(
                predictions,
                "{block},{position},{},{},{target},{base:.17},{mixed:.17},{:.17},{:.17}",
                start + position + 1,
                tokens[start + position],
                nll[0],
                nll[1]
            )?;
        }
        let partition = usize::from(block >= TUNE_BLOCKS);
        for index in 0..2 {
            sums[partition][index] += block_sums[index];
        }
        serde_json::to_writer(
            &mut blocks,
            &json!({"block":block,"targets":CONTEXT,
            "ngram_nll_sum_nats":block_sums[0],"cache_nll_sum_nats":block_sums[1]}),
        )?;
        writeln!(blocks)?;
        if block % 128 == 0 {
            eprintln!("count evaluation block {block}/{TOTAL_BLOCKS}");
        }
        if started.elapsed().as_secs() > 2400 {
            return Err(invalid("count evaluation deadline"));
        }
    }
    predictions.flush()?;
    blocks.flush()?;
    let partition = |index: usize, targets: usize| {
        json!({"scored_targets":targets,
        "ngram_nll_nats":sums[index][0]/targets as f64,"cache_nll_nats":sums[index][1]/targets as f64})
    };
    let mut report = base_report(protocol, "ngram-evaluate")?;
    report["selection"] = selection;
    report["model_sha256"] = fitted["model_sha256"].clone();
    report["model_fit_evaluator_sha256"] = fitted["evaluator_sha256"].clone();
    report["tune"] = partition(0, TUNE_BLOCKS * CONTEXT);
    report["comparison"] = partition(1, (TOTAL_BLOCKS - TUNE_BLOCKS) * CONTEXT);
    report["full"] = json!({"scored_targets":TOTAL_BLOCKS*CONTEXT,
        "ngram_nll_nats":(sums[0][0]+sums[1][0])/(TOTAL_BLOCKS*CONTEXT) as f64,
        "cache_nll_nats":(sums[0][1]+sums[1][1])/(TOTAL_BLOCKS*CONTEXT) as f64});
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["status"] = json!("COMPLETE");
    save_json(&out.join("count-evaluation.json"), &report)?;
    Ok(())
}
