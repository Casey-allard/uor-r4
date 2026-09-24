//! Protocol-bound reference evaluation and generation campaigns.
//!
//! The CLI owns claiming and sealing the output directory. These routines
//! create each file exclusively, preserve partial streams on failure, and
//! never use historical generation outputs while producing continuations.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use candle_core::Tensor;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

use crate::baseline_protocol::{
    base_report, device, read_tokens, save_json, verify_identity, Evaluator,
};
use crate::reference_eval::{
    evaluate_tokens, generate_prompts, BlockEvaluation, GenerationPrompt, GenerationResult,
    GenerationStop, EVALUATION_CONTEXT, SEEDED_SAMPLER_POLICY,
};
use crate::{invalid, ReferenceModel, Result};

const DEADLINE_SECONDS: u64 = 2400;

/// Run the complete historical development population after causal/batch checks.
pub fn run_evaluate(
    protocol: &Evaluator,
    out: &Path,
    device_name: &str,
    batch: usize,
) -> Result<()> {
    let started = Instant::now();
    validate_protocol(protocol)?;
    let document = &protocol.document;
    if batch == 0 || batch > usize_field(&document["reference_gate"], "batch_max")? {
        return Err(invalid(
            "reference batch is outside the evaluator's pinned limit",
        ));
    }
    let mut report = base_report(protocol, "reference-evaluate")?;
    let snapshot = verify_reference_inputs(protocol)?;
    let tokens = read_tokens(&document["dev_source"])?;
    let full_blocks = usize_field(document, "full_blocks")?;
    let tune_blocks = usize_field(document, "tune_blocks")?;
    if tokens.len().saturating_sub(1) / EVALUATION_CONTEXT != full_blocks {
        return Err(invalid(
            "development population does not have all declared full blocks",
        ));
    }
    check_deadline(started)?;
    let selected_device = device(device_name)?;
    report["requested_device"] = json!(device_name);
    report["actual_device"] = json!(format!("{:?}", selected_device.location()));
    report["inputs"] =
        json!({"reference":document["reference_inputs"],"development":document["dev_source"]});
    report["deadline_seconds"] = json!(DEADLINE_SECONDS);
    let model = ReferenceModel::load_pinned(&snapshot, &selected_device)?;
    check_deadline(started)?;
    let preflight = preflight_checks(&model, &tokens, document, started)?;
    save_json(&out.join("preflight-checks.json"), &preflight)?;
    report["preflight"] = preflight.clone();
    if preflight["passes"] != true {
        report["status"] = json!("REFERENCE_EVALUATION_FAILED_PREFLIGHT");
        report["complete_population"] = json!(false);
        report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
        save_json(&out.join("reference-evaluation.json"), &report)?;
        return Err(invalid(
            "reference batch-isolation or future-edit gate failed; checks retained",
        ));
    }

    let mut targets = BufWriter::new(File::create_new(out.join("reference-targets.csv"))?);
    let mut blocks = BufWriter::new(File::create_new(out.join("reference-blocks.jsonl"))?);
    writeln!(targets, "block,position,input_offset,target_offset,input_token,target_token,predicted_token,target_logit,log_normalizer,nll_nats,correct")?;
    let mut tune = Partition::default();
    let mut comparison = Partition::default();
    let evaluation = evaluate_tokens(&model, &tokens, batch, |block, predictions| {
        check_deadline(started)?;
        for prediction in predictions {
            let score = &prediction.score;
            writeln!(
                targets,
                "{},{},{},{},{},{},{},{},{},{},{}",
                prediction.block_index,
                prediction.position_in_block,
                prediction.input_offset,
                prediction.target_offset,
                prediction.input_token,
                score.target_token,
                score.predicted_token,
                score.target_logit,
                score.log_normalizer,
                score.nll_nats,
                u8::from(score.correct)
            )?;
        }
        serde_json::to_writer(&mut blocks, block)?;
        blocks.write_all(b"\n")?;
        targets.flush()?;
        blocks.flush()?;
        if block.block_index < tune_blocks {
            tune.add(block);
        } else {
            comparison.add(block);
        }
        check_deadline(started)
    });
    // Flush both streams even when evaluation stopped on a callback/tensor error.
    finish_stream(&mut targets)?;
    finish_stream(&mut blocks)?;
    report["partitions"] = json!({"tune":tune.report(),"comparison":comparison.report()});
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    let metrics = match evaluation {
        Ok(metrics) => metrics,
        Err(error) => {
            report["status"] = json!("REFERENCE_EVALUATION_FAILED_INCOMPLETE");
            report["complete_population"] = json!(false);
            report["error"] = json!(error.to_string());
            save_json(&out.join("reference-evaluation.json"), &report)?;
            return Err(error);
        }
    };
    let expected = f64_field(&document["reference_gate"], "historical_dev_nll_nats")?;
    let tolerance = f64_field(&document["reference_gate"], "max_absolute_nll_delta")?;
    let delta = (metrics.mean_nll_nats - expected).abs();
    let population_complete = metrics.completed_blocks == full_blocks
        && metrics.scored_targets == usize_field(document, "scored_targets")?
        && tune.blocks == tune_blocks
        && comparison.blocks == usize_field(document, "comparison_blocks")?;
    let passed = population_complete && delta.is_finite() && delta <= tolerance;
    report["complete_population"] = json!(population_complete);
    report["full"] = serde_json::to_value(&metrics)?;
    report["historical_nll_gate"] = json!({
        "actual_nll_nats":metrics.mean_nll_nats,
        "historical_nll_nats":expected,
        "absolute_delta":delta,
        "maximum_absolute_delta":tolerance,
        "passes":passed,
        "reduction":"F64 stable full-vocabulary logsumexp over actual F32 logits, with compensated corpus accumulation; historical PyTorch used F32 batch-mean loss"
    });
    report["status"] = json!(if passed {
        "REFERENCE_EVALUATION_PASS"
    } else {
        "REFERENCE_EVALUATION_FAILED"
    });
    save_json(&out.join("reference-evaluation.json"), &report)?;
    if !passed {
        return Err(invalid(
            "full-population historical NLL gate failed; complete records retained",
        ));
    }
    check_deadline(started)
}

/// Produce and persist all five continuations before opening any golden report.
pub fn run_generate(protocol: &Evaluator, out: &Path, device_name: &str) -> Result<()> {
    let started = Instant::now();
    validate_protocol(protocol)?;
    let document = &protocol.document;
    validate_generation_contract(&document["generation"])?;
    let mut report = base_report(protocol, "reference-generate")?;
    let snapshot = verify_reference_inputs(protocol)?;
    let prompt_path = verify_identity(&document["prompt_source"])?;
    let prompt_document: Value = serde_json::from_slice(&fs::read(&prompt_path)?)?;
    let tokenizer = HfBpeTokenizer::from_dir(&snapshot)
        .map_err(|error| invalid(format!("retained reference tokenizer: {error}")))?;
    if prompt_document["tokenizer_cid"] != tokenizer.address() {
        return Err(invalid(
            "historical prompt fixture tokenizer identity differs",
        ));
    }
    let prompt_records = prompt_document["prompts"]
        .as_array()
        .ok_or_else(|| invalid("historical prompt fixture has no prompt array"))?;
    let seeds = document["generation"]["seeds"]
        .as_array()
        .ok_or_else(|| invalid("generation seeds are not an array"))?;
    if prompt_records.len() != 5 || seeds.len() != prompt_records.len() {
        return Err(invalid(
            "generation requires exactly five pinned prompts and seeds",
        ));
    }
    let mut prompts = Vec::with_capacity(prompt_records.len());
    for (index, record) in prompt_records.iter().enumerate() {
        let text = record["text"]
            .as_str()
            .ok_or_else(|| invalid("historical prompt text"))?;
        let expected_tokens: Vec<u32> = serde_json::from_value(record["token_ids"].clone())?;
        if expected_tokens.len() != usize_field(&document["generation"], "prompt_content_tokens")?
            || tokenizer.encode(text) != expected_tokens
        {
            return Err(invalid(format!(
                "prompt {index} tokenizer IDs differ from the retained fixture"
            )));
        }
        prompts.push(GenerationPrompt {
            id: format!("prompt-{index}"),
            text: text.to_owned(),
            seed: Some(
                seeds[index]
                    .as_u64()
                    .ok_or_else(|| invalid("invalid generation seed"))?,
            ),
        });
    }
    check_deadline(started)?;
    let selected_device = device(device_name)?;
    report["requested_device"] = json!(device_name);
    report["actual_device"] = json!(format!("{:?}", selected_device.location()));
    report["inputs"] =
        json!({"reference":document["reference_inputs"],"prompts":document["prompt_source"]});
    report["deadline_seconds"] = json!(DEADLINE_SECONDS);
    report["goldens_accessed"] = json!(false);
    let model = ReferenceModel::load_pinned(&snapshot, &selected_device)?;
    check_deadline(started)?;
    let mut decisions = BufWriter::new(File::create_new(out.join("generation-predictions.jsonl"))?);
    let generated = generate_prompts(
        &model,
        &tokenizer,
        &prompts,
        usize_field(&document["generation"], "max_new_tokens")?,
        |prediction| {
            check_deadline(started)?;
            serde_json::to_writer(&mut decisions, prediction)?;
            decisions.write_all(b"\n")?;
            decisions.flush()?;
            check_deadline(started)
        },
    );
    finish_stream(&mut decisions)?;
    let generated = match generated {
        Ok(generated) => generated,
        Err(error) => {
            report["status"] = json!("REFERENCE_GENERATION_FAILED_INCOMPLETE");
            report["error"] = json!(error.to_string());
            report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
            save_json(&out.join("reference-generation.json"), &report)?;
            return Err(error);
        }
    };
    // The protocol requires this persistence boundary before any golden access.
    save_json(&out.join("generation-actual.json"), &generated)?;
    for (index, actual) in generated.iter().enumerate() {
        save_json(&out.join(format!("generation-actual-{index}.json")), actual)?;
    }
    check_deadline(started)?;
    let comparison = compare_goldens(protocol, &generated, started);
    report["actual_generations_file"] = json!("generation-actual.json");
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    let comparison = match comparison {
        Ok(comparison) => comparison,
        Err(error) => {
            report["status"] = json!("REFERENCE_GENERATION_COMPARISON_UNAVAILABLE");
            // A deadline or first identity failure can stop comparison before
            // any golden was opened; do not fabricate completed access here.
            report["goldens_accessed"] = Value::Null;
            report["golden_access_status"] = json!("Comparison attempted after actual persistence; zero or more golden inputs may have been read before the recorded failure");
            report["goldens_accessed_after_actual_persistence"] = json!(true);
            report["error"] = json!(error.to_string());
            save_json(&out.join("reference-generation.json"), &report)?;
            return Err(error);
        }
    };
    let passed = comparison["passes"] == true;
    save_json(&out.join("generation-replay.json"), &comparison)?;
    report["goldens_accessed"] = json!(true);
    report["goldens_accessed_after_actual_persistence"] = json!(true);
    report["comparison"] = comparison;
    report["status"] = json!(if passed {
        "REFERENCE_GENERATION_PASS"
    } else {
        "REFERENCE_GENERATION_FAILED"
    });
    save_json(&out.join("reference-generation.json"), &report)?;
    if !passed {
        return Err(invalid(
            "cross-implementation historical generation equality failed; actual outputs retained",
        ));
    }
    check_deadline(started)
}

fn validate_protocol(protocol: &Evaluator) -> Result<()> {
    let value = &protocol.document;
    if value["schema"] != "uor-r4.reference-evaluator/2"
        || value["context"] != 256
        || value["stride"] != 256
        || value["vocabulary"] != 4096
        || value["full_blocks"] != 976
        || value["tune_blocks"] != 64
        || value["comparison_blocks"] != 912
        || value["scored_targets"] != 249856
        || value["reference_gate"]["historical_dev_nll_nats"] != json!(1.580241072373312f64)
        || value["reference_gate"]["max_absolute_nll_delta"] != json!(0.0001f64)
        || value["reference_gate"]["batch_max"] != 16
        || value["reference_gate"]["batch_isolation_max_logit_delta"] != json!(0.005f64)
        || value["reference_gate"]["future_edit_max_prefix_logit_delta"] != json!(1e-5f64)
        || value["reference_gate"]["gradients_tracked_during_evaluation"] != false
    {
        return Err(invalid(
            "reference campaign requires the unchanged evaluator v2 contract",
        ));
    }
    Ok(())
}

fn validate_generation_contract(value: &Value) -> Result<()> {
    if value["seeds"] != json!([2014, 2015, 2016, 2017, 2018])
        || value["max_new_tokens"] != 128
        || value["top_k"] != 40
        || value["temperature"] != json!(0.8f64)
        || value["sampler"] != "r4-local-top-k-q32-splitmix64/1"
        || value["bos"] != 0
        || value["eos"] != 1
        || value["prompt_content_tokens"] != 24
    {
        return Err(invalid(
            "reference generation requires the unchanged historical sampling contract",
        ));
    }
    Ok(())
}

fn verify_reference_inputs(protocol: &Evaluator) -> Result<PathBuf> {
    let snapshot = PathBuf::from(
        protocol.document["snapshot"]
            .as_str()
            .ok_or_else(|| invalid("reference snapshot path"))?,
    );
    let inputs = protocol.document["reference_inputs"]
        .as_array()
        .ok_or_else(|| invalid("reference input identities"))?;
    let expected: BTreeSet<_> = ["config.json", "model.safetensors", "tokenizer.json"]
        .into_iter()
        .map(|name| snapshot.join(name))
        .collect();
    let mut observed = BTreeSet::new();
    for identity in inputs {
        if !observed.insert(verify_identity(identity)?) {
            return Err(invalid("duplicate reference input identity"));
        }
    }
    if observed != expected {
        return Err(invalid(
            "reference input identities do not match the loaded snapshot files",
        ));
    }
    Ok(snapshot)
}

fn preflight_checks(
    model: &ReferenceModel,
    tokens: &[u16],
    protocol: &Value,
    started: Instant,
) -> Result<Value> {
    let original: Vec<u32> = tokens[..EVALUATION_CONTEXT]
        .iter()
        .map(|&id| u32::from(id))
        .collect();
    let distinct_index = (1..tokens.len().saturating_sub(1) / EVALUATION_CONTEXT)
        .find(|&block| {
            tokens[block * EVALUATION_CONTEXT..(block + 1) * EVALUATION_CONTEXT]
                != tokens[..EVALUATION_CONTEXT]
        })
        .ok_or_else(|| invalid("no distinct retained block for batch isolation check"))?;
    let start = distinct_index * EVALUATION_CONTEXT;
    let distinct: Vec<u32> = tokens[start..start + EVALUATION_CONTEXT]
        .iter()
        .map(|&id| u32::from(id))
        .collect();
    let mut batch_inputs = original.clone();
    batch_inputs.extend_from_slice(&distinct);
    check_deadline(started)?;
    let batched = detached_flat(
        model.forward_eval_batch(&batch_inputs, 2, EVALUATION_CONTEXT)?,
        &[2, EVALUATION_CONTEXT, model.config.vocab_size],
    )?;
    check_deadline(started)?;
    let first = detached_flat(
        model.forward_eval(&original)?,
        &[EVALUATION_CONTEXT, model.config.vocab_size],
    )?;
    check_deadline(started)?;
    let second = detached_flat(
        model.forward_eval(&distinct)?,
        &[EVALUATION_CONTEXT, model.config.vocab_size],
    )?;
    let mut singles = first.clone();
    singles.extend_from_slice(&second);
    let batch_comparison = compare_logits(
        &batched,
        &singles,
        f64_field(
            &protocol["reference_gate"],
            "batch_isolation_max_logit_delta",
        )?,
    )?;

    let mut edited = original.clone();
    for token in &mut edited[128..] {
        *token = (*token + 1) % 4096;
    }
    check_deadline(started)?;
    let future = detached_flat(
        model.forward_eval(&edited)?,
        &[EVALUATION_CONTEXT, model.config.vocab_size],
    )?;
    check_deadline(started)?;
    let prefix_values = 128 * model.config.vocab_size;
    let future_comparison = compare_logits(
        &first[..prefix_values],
        &future[..prefix_values],
        f64_field(
            &protocol["reference_gate"],
            "future_edit_max_prefix_logit_delta",
        )?,
    )?;
    let passes = batch_comparison.passes && future_comparison.passes;
    Ok(json!({
        "schema":"uor-r4.reference-evaluation-preflight/1",
        "source_population":"Pinned development token store; no optimization",
        "block_indices":[0,distinct_index],
        "batch_size":2,
        "context":EVALUATION_CONTEXT,
        "original_block_tokens":original,
        "distinct_block_tokens":distinct,
        "batch_isolation":batch_comparison,
        "future_edit":{
            "edited_input_positions":"128..256 (exclusive end)",
            "edit":"Each affected token becomes (token+1)%4096; prefix inputs unchanged",
            "compared_output_positions":"0..128 (exclusive end)",
            "original_prefix_logits_sha256_le_f32":logit_hash(&first[..prefix_values]),
            "edited_prefix_logits_sha256_le_f32":logit_hash(&future[..prefix_values]),
            "comparison":future_comparison
        },
        "gradients_tracked":false,
        "passes":passes
    }))
}

fn detached_flat(tensor: Tensor, expected: &[usize]) -> Result<Vec<f32>> {
    if tensor.track_op() || tensor.dims() != expected {
        return Err(invalid(
            "preflight forward is not detached or has the wrong logit shape",
        ));
    }
    Ok(tensor.flatten_all()?.to_vec1::<f32>()?)
}

#[derive(Serialize)]
struct LogitComparison {
    compared_values: usize,
    finite: bool,
    maximum_absolute_delta: Option<f64>,
    maximum_allowed_absolute_delta: f64,
    passes: bool,
}
fn compare_logits(left: &[f32], right: &[f32], tolerance: f64) -> Result<LogitComparison> {
    if left.len() != right.len() || left.is_empty() {
        return Err(invalid("preflight logit comparison shape mismatch"));
    }
    let finite = left.iter().chain(right).all(|value| value.is_finite());
    let maximum = finite.then(|| {
        left.iter()
            .zip(right)
            .map(|(&a, &b)| (f64::from(a) - f64::from(b)).abs())
            .fold(0.0f64, f64::max)
    });
    Ok(LogitComparison {
        compared_values: left.len(),
        finite,
        maximum_absolute_delta: maximum,
        maximum_allowed_absolute_delta: tolerance,
        passes: maximum.is_some_and(|value| value <= tolerance),
    })
}

fn logit_hash(logits: &[f32]) -> String {
    let mut digest = Sha256::new();
    for value in logits {
        digest.update(value.to_le_bytes());
    }
    hex::encode(digest.finalize())
}

fn compare_goldens(
    protocol: &Evaluator,
    actual: &[GenerationResult],
    started: Instant,
) -> Result<Value> {
    let goldens = protocol.document["golden_generations"]
        .as_array()
        .ok_or_else(|| invalid("historical generation identity array"))?;
    if goldens.len() != actual.len() {
        return Err(invalid("historical and actual generation counts differ"));
    }
    let mut rows = Vec::with_capacity(actual.len());
    let mut all_equal = true;
    for (index, (identity, actual)) in goldens.iter().zip(actual).enumerate() {
        check_deadline(started)?;
        let path = verify_identity(identity)?;
        let golden: Value = serde_json::from_slice(&fs::read(path)?)?;
        let expected_ids: Vec<u32> =
            serde_json::from_value(golden["transcript"]["generated_token_ids"].clone())?;
        let expected_prompt_ids: Vec<u32> =
            serde_json::from_value(golden["prompt_token_ids"].clone())?;
        let expected_text = golden["transcript"]["response_text"]
            .as_str()
            .ok_or_else(|| invalid("historical generated response text"))?;
        let actual_stop = historical_stop(&actual.stop);
        let prompt_equal = golden["prompt"] == actual.prompt.text
            && expected_prompt_ids == actual.prompt_token_ids;
        let sampler_equal = golden["decode_audit"]["seed"] == json!(actual.prompt.seed)
            && golden["decode_audit"]["sampler_policy"] == SEEDED_SAMPLER_POLICY;
        let ids_equal = expected_ids == actual.generated_token_ids;
        let text_equal = expected_text == actual.response_text;
        let stop_equal = golden["stop_reason"] == actual_stop;
        let passes = prompt_equal && sampler_equal && ids_equal && text_equal && stop_equal;
        all_equal &= passes;
        let first_difference = actual
            .generated_token_ids
            .iter()
            .zip(&expected_ids)
            .position(|(a, b)| a != b)
            .or_else(|| {
                (actual.generated_token_ids.len() != expected_ids.len())
                    .then_some(actual.generated_token_ids.len().min(expected_ids.len()))
            });
        rows.push(json!({
            "index":index,
            "prompt_id":actual.prompt.id,
            "seed":actual.prompt.seed,
            "golden_identity":identity,
            "prompt_and_prompt_ids_equal":prompt_equal,
            "sampler_and_seed_equal":sampler_equal,
            "generated_ids_equal":ids_equal,
            "response_text_equal":text_equal,
            "stop_equal":stop_equal,
            "actual_stop":actual_stop,
            "historical_stop":golden["stop_reason"],
            "actual_generated_tokens":actual.generated_token_ids.len(),
            "historical_generated_tokens":expected_ids.len(),
            "first_differing_generated_position":first_difference,
            "historical_generated_token_ids":expected_ids,
            "historical_response_text":expected_text,
            "passes":passes
        }));
    }
    Ok(json!({
        "schema":"uor-r4.reference-generation-replay/1",
        "scope":"Cross-implementation output equality against retained historical Rust generations; no equality claim for R4-gauge audit metadata or runtime internals",
        "golden_access":"All actual continuations and decision streams persisted before historical outputs were opened",
        "comparisons":rows,
        "passes":all_equal
    }))
}

fn historical_stop(stop: &GenerationStop) -> Value {
    match stop {
        GenerationStop::Eos => json!("eos"),
        GenerationStop::MaximumNewTokens => json!("maximum_new_tokens"),
        GenerationStop::ShortCycle { period } => json!({"short_cycle":{"period":period}}),
    }
}

#[derive(Default)]
struct Partition {
    blocks: usize,
    targets: usize,
    nll_sum: f64,
    compensation: f64,
    correct: usize,
}
impl Partition {
    fn add(&mut self, block: &BlockEvaluation) {
        self.blocks += 1;
        self.targets += block.scored_targets;
        self.correct += block.correct;
        let adjusted = block.nll_sum_nats - self.compensation;
        let sum = self.nll_sum + adjusted;
        self.compensation = (sum - self.nll_sum) - adjusted;
        self.nll_sum = sum;
    }
    fn report(&self) -> Value {
        json!({"blocks":self.blocks,"scored_targets":self.targets,"nll_sum_nats":self.nll_sum,
            "mean_nll_nats":(self.targets>0).then(||self.nll_sum/self.targets as f64),
            "correct":self.correct,"top1_accuracy":(self.targets>0).then(||self.correct as f64/self.targets as f64)})
    }
}

fn finish_stream(stream: &mut BufWriter<File>) -> Result<()> {
    stream.flush()?;
    stream.get_ref().sync_all()?;
    Ok(())
}

fn check_deadline(started: Instant) -> Result<()> {
    if started.elapsed() >= Duration::from_secs(DEADLINE_SECONDS) {
        return Err(invalid(
            "reference campaign reached its 2400-second deadline; partial outputs retained",
        ));
    }
    Ok(())
}

fn usize_field(value: &Value, name: &str) -> Result<usize> {
    value[name]
        .as_u64()
        .and_then(|number| usize::try_from(number).ok())
        .ok_or_else(|| invalid(format!("invalid evaluator integer {name}")))
}
fn f64_field(value: &Value, name: &str) -> Result<f64> {
    value[name]
        .as_f64()
        .filter(|number| number.is_finite())
        .ok_or_else(|| invalid(format!("invalid evaluator scalar {name}")))
}
