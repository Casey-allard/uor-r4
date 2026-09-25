//! Execution comparison for the full-context integer bridge. This deliberately
//! reuses the retained evaluator, source probes and sampling rule. Its default
//! four development windows are numerical qualification, not a language gate.
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::answer_oracle;
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

use crate::baseline_protocol::{
    base_report, load_evaluator, read_tokens, save_json, verify_identity,
};
use crate::joint_admission::AdmissionPolicy;
use crate::joint_campaign::{finish_attempt, load_hard_export};
use crate::joint_evaluation::{
    self, JointGenerationStop, SplitMix64, StoryVariant, EVALUATION_CONTEXT, MAX_NEW_TOKENS,
    STORY_PROBE_MAX_NEW_TOKENS, STORY_PROBE_SCOPE, STORY_STOP_POLICY,
};
use crate::joint_integer::{IntegerModel, IntegerStep};
use crate::joint_model::{JointModel, ReadMode};
use crate::reference_eval::short_cycle_period;
use crate::{invalid, sha256_file, Result};

const PROBABILITY_TOTAL: u64 = 1 << 48;
const STATE_SCALE: f64 = 2048.0;
const GATE_SCALE: f64 = 32768.0;

/// Return false for another command so the normal dispatcher can continue.
pub fn run(args: &[String]) -> Result<bool> {
    if args.first().map(String::as_str) != Some("joint-integer-evaluate") {
        return Ok(false);
    }
    if !(args.len() == 5 || args.len() == 6) {
        return Err(invalid(
            "usage: joint-integer-evaluate PACKED TABLES EVALUATOR NEW_ROOT [MAX_WINDOWS]",
        ));
    }
    let windows = match args.get(5) {
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| invalid("integer evaluation window count"))?,
        None => 4,
    };
    if !(1..=976).contains(&windows) {
        return Err(invalid("integer evaluation windows must be 1..976"));
    }
    let out = Path::new(&args[4]);
    report_output::claim(out)?;
    let result = evaluate(
        Path::new(&args[1]),
        Path::new(&args[2]),
        Path::new(&args[3]),
        out,
        windows,
    );
    finish_attempt(out, result)?;
    Ok(true)
}

fn evaluate(
    packed: &Path,
    tables: &Path,
    evaluator_path: &Path,
    out: &Path,
    windows: usize,
) -> Result<()> {
    let started = Instant::now();
    let evaluator = load_evaluator(evaluator_path)?;
    report_output::verify(tables)?;
    let reference = load_hard_export(packed, &evaluator)?;
    let integer = IntegerModel::load_with_tables(packed, tables)?;
    if &reference.model.config != integer.config()
        || integer.config().context != EVALUATION_CONTEXT
        || reference.model.admission_policy() != AdmissionPolicy::Full
        || reference.campaign.context != EVALUATION_CONTEXT
    {
        return Err(invalid(
            "integer comparison requires matched full256 accepted artifact",
        ));
    }
    let mut report = base_report(&evaluator, "joint-integer-evaluate")?;
    report["schema"] = json!("uor-r4.joint-integer-execution-evaluation/1");
    report["scope"] = json!("Same learned packed parameters; numerical execution comparison on previously exposed development and unchanged source/continuation panels. No training, final holdout, language qualification, promotion, energy result or geometric advantage is inferred.");
    report["packed_path"] = json!(packed);
    report["packed_manifest_sha256"] = json!(sha256_file(&packed.join("hard-model.json"))?);
    report["tables_path"] = json!(tables);
    report["tables_metadata_sha256"] = json!(sha256_file(&tables.join("tables.json"))?);
    report["tables_payload_sha256"] = json!(sha256_file(&tables.join("tables.bin"))?);
    report["artifact"] = reference.artifact;
    report["checkpoint_binding"] = reference.binding;
    report["model_config"] = serde_json::to_value(&integer.config())?;
    report["training_context"] = json!(reference.campaign.context);
    report["training_batch"] = json!(reference.campaign.batch);
    report["evaluation_context"] = json!(EVALUATION_CONTEXT);
    report["evaluation_stride"] = json!(EVALUATION_CONTEXT);
    report["session_context"] = json!(integer.config().context);
    report["admission_policy"] = json!("full");
    report["prospective_numerical_bounds"] = json!({
        "maximum_state_absolute_delta":0.01,"maximum_probability_absolute_delta":0.01,
        "scope":"Principal-selected engineering drift limits before execution; not a theoretical error bound, new language acceptance gate, or permission to change the existing source-answer retention criteria."
    });
    report["read_vector_coordinates"] = json!(integer.config().read_width);
    report["numerical_boundary"] = json!({
        "integer_model_step":"Q48 probabilities; Q11 state; Q15 copy gate; numerical kernel implementation is audited separately",
        "outside_integer_kernel":["F32 reference emulator","tokenizer and text decoding","probability/state comparisons and NLL in host floating point","historical seeded top-k sampler uses host ln/exp and SplitMix multiplication","serialization and provenance hashing"],
        "seeded_generation_is_complete_integer_serving":false,
        "greedy_source_selection":"maximum Q48 probability; ascending token on ties",
        "allocator_and_machine_instruction_claim":"not established by this evaluation"
    });
    let tokenizer = load_tokenizer(&evaluator.document)?;
    let prompts_path = verify_identity(&evaluator.document["prompt_source"])?;
    let prompts: Value = serde_json::from_slice(&fs::read(prompts_path)?)?;
    let prompts = prompts["prompts"]
        .as_array()
        .ok_or_else(|| invalid("integer evaluation prompts"))?;
    let dev = read_tokens(&evaluator.document["dev_source"])?;
    if dev.len() < windows * EVALUATION_CONTEXT + 1 {
        return Err(invalid("insufficient fixed development tokens"));
    }
    report["setup_seconds"] = json!(started.elapsed().as_secs_f64());
    let mut modes = Vec::new();
    for (mode, label) in [(ReadMode::Enabled, "read"), (ReadMode::NoRead, "no-read")] {
        let panels_started = Instant::now();
        let f32_probes = joint_evaluation::run_story_probes(&reference.model, &tokenizer, mode)?;
        let mut integer_probes = Vec::new();
        let mut probe_rows = BufWriter::new(File::create_new(
            out.join(format!("story-probes-{label}-integer.jsonl")),
        )?);
        let mut first_noun_losses = 0usize;
        let mut complete_losses = 0usize;
        let mut complete_gains = 0usize;
        let mut integer_complete = 0usize;
        let mut f32_complete = 0usize;
        for paired in &f32_probes {
            let original = judge_story(&integer, &tokenizer, &paired.probe.original, mode)?;
            let edited = judge_story(&integer, &tokenizer, &paired.probe.edited, mode)?;
            for (actual, baseline) in [(&original, &paired.original), (&edited, &paired.edited)] {
                let first = actual["first_noun_correct"] == true;
                let complete = actual["complete_correct"] == true;
                first_noun_losses += usize::from(baseline.first_noun_correct && !first);
                complete_losses += usize::from(baseline.complete_correct && !complete);
                complete_gains += usize::from(!baseline.complete_correct && complete);
                integer_complete += usize::from(complete);
                f32_complete += usize::from(baseline.complete_correct);
            }
            let probe_row = json!({
                "schema":"uor-r4.joint-integer-story-source-edit/1",
                "scope":STORY_PROBE_SCOPE,"stop_policy":STORY_STOP_POLICY,
                "probe":paired.probe,"mode":mode,
                "pair_complete_correct":original["complete_correct"] == true && edited["complete_correct"] == true,
                "outputs_differ":original["generation"]["response_text"] != edited["generation"]["response_text"],
                "original":original,"edited":edited
            });
            serde_json::to_writer(&mut probe_rows, &probe_row)?;
            writeln!(probe_rows)?;
            probe_rows.flush()?;
            integer_probes.push(probe_row);
            eprintln!(
                "integer execution {label}: source pair {} of16 complete",
                integer_probes.len()
            );
        }
        probe_rows.get_ref().sync_all()?;
        save_json(
            &out.join(format!("story-probes-{label}-f32.json")),
            &f32_probes,
        )?;
        save_json(
            &out.join(format!("story-probes-{label}-integer.json")),
            &integer_probes,
        )?;
        let mut generations = Vec::new();
        for (index, prompt) in prompts.iter().enumerate() {
            let prompt = prompt["text"]
                .as_str()
                .ok_or_else(|| invalid("integer prompt text"))?;
            let seed = Some(2014 + index as u64);
            let baseline = joint_evaluation::generate(
                &reference.model,
                &tokenizer,
                prompt,
                mode,
                seed,
                MAX_NEW_TOKENS,
            )?;
            let actual = generate(
                &integer,
                &tokenizer,
                prompt,
                mode,
                seed,
                MAX_NEW_TOKENS,
                false,
            )?;
            generations.push(json!({
                "prompt_index":index,"mode":mode,
                "generated_token_ids_equal":serde_json::to_value(&baseline.generated_token_ids)? == actual["generated_token_ids"],
                "f32":baseline,"integer":actual
            }));
        }
        save_json(&out.join(format!("generations-{label}.json")), &generations)?;
        let panels_seconds = panels_started.elapsed().as_secs_f64();
        eprintln!("integer execution {label}: source and continuation panels saved in {panels_seconds:.2}s; starting {windows} full256 numerical windows");
        let numerical = compare_windows(
            &integer,
            &reference.model,
            &dev,
            mode,
            windows,
            &out.join(format!("targets-{label}.jsonl")),
        )?;
        let mode_report = json!({
            "mode":mode,"source_variants":32,"integer_complete_source_answers":integer_complete,
            "f32_complete_source_answers":f32_complete,"first_noun_losses_vs_f32":first_noun_losses,
            "complete_losses_vs_f32":complete_losses,"complete_gains_vs_f32":complete_gains,
            "continuations":generations.len(),"panels_seconds":panels_seconds,"numerical":numerical
        });
        save_json(&out.join(format!("mode-{label}-report.json")), &mode_report)?;
        modes.push(mode_report);
        eprintln!("integer execution {label}: {windows} full256 windows; {integer_complete}/32 source answers; {first_noun_losses} first-noun losses");
    }
    report["modes"] = json!(modes);
    report["optimizer_steps"] = json!(0);
    report["requested_windows"] = json!(windows);
    report["full_fixed_development_complete"] = json!(windows == 976);
    report["numerical_scope"] = json!(if windows == 976 {
        "All 976 existing full256 development windows; no new holdout."
    } else {
        "Fixed corpus prefix only; not the full development/comparison gate. First64 windows are existing calibration partition; later windows, if requested, are comparison partition."
    });
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["status"] = json!("EXECUTION_COMPARISON_COMPLETE_REQUIRES_PRINCIPAL_ADJUDICATION");
    save_json(&out.join("evaluation-report.json"), &report)?;
    Ok(())
}

fn load_tokenizer(evaluator: &Value) -> Result<HfBpeTokenizer> {
    let identity = evaluator["reference_inputs"]
        .as_array()
        .and_then(|entries| {
            entries.iter().find(|entry| {
                entry["path"]
                    .as_str()
                    .is_some_and(|path| path.ends_with("/tokenizer.json"))
            })
        })
        .ok_or_else(|| invalid("integer evaluator tokenizer identity missing"))?;
    let path = verify_identity(identity)?;
    HfBpeTokenizer::from_dir(path.parent().ok_or_else(|| invalid("tokenizer parent"))?)
        .map_err(|error| invalid(format!("integer tokenizer: {error}")))
}

fn probability_row(step: &IntegerStep, vocab: usize, mode: ReadMode) -> Result<Vec<f32>> {
    if step.probabilities.len() != vocab
        || step
            .probabilities
            .iter()
            .map(|&v| u128::from(v))
            .sum::<u128>()
            != u128::from(PROBABILITY_TOTAL)
        || step.no_read_mass > PROBABILITY_TOTAL
        || !(0..=32768).contains(&step.copy_gate)
        || (mode == ReadMode::NoRead && step.no_read_mass != PROBABILITY_TOTAL)
    {
        return Err(invalid(
            "integer probability normalization or NoRead contract differs",
        ));
    }
    Ok(step
        .probabilities
        .iter()
        .map(|&p| (p as f64 / PROBABILITY_TOTAL as f64) as f32)
        .collect())
}

/// The integer interface orders one mass per earlier occurrence, oldest first.
/// A zero mass is a scored slot with zero resulting weight, not an omitted slot.
fn check_read_slots(step: &IntegerStep, prior_positions: usize, mode: ReadMode) -> Result<()> {
    let read_total = step
        .read_masses
        .iter()
        .map(|&mass| u128::from(mass))
        .sum::<u128>();
    if step.read_masses.len() != prior_positions
        || read_total + u128::from(step.no_read_mass) != u128::from(PROBABILITY_TOTAL)
        || (mode == ReadMode::NoRead && read_total != 0)
    {
        return Err(invalid(
            "integer full causal slot count, read normalization or NoRead differs",
        ));
    }
    Ok(())
}

fn greedy(probabilities: &[u64]) -> Result<u32> {
    let mut best = None;
    for (token, &probability) in probabilities.iter().enumerate() {
        if best.is_none_or(|(_, value)| probability > value) {
            best = Some((token, probability));
        }
    }
    u32::try_from(best.ok_or_else(|| invalid("integer empty vocabulary"))?.0)
        .map_err(|_| invalid("integer vocabulary exceeds u32"))
}

#[allow(clippy::too_many_arguments)]
fn generate(
    model: &IntegerModel,
    tokenizer: &HfBpeTokenizer,
    prompt: &str,
    mode: ReadMode,
    seed: Option<u64>,
    max_new_tokens: usize,
    first_sentence: bool,
) -> Result<Value> {
    let started = Instant::now();
    let content = tokenizer.encode(prompt);
    if content.is_empty()
        || tokenizer.vocab_size() != model.config().vocab_size
        || !(1..=MAX_NEW_TOKENS).contains(&max_new_tokens)
        || content
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_add(max_new_tokens))
            .is_none_or(|n| n > model.config().context)
        || content
            .iter()
            .any(|&token| token as usize >= model.config().vocab_size)
    {
        return Err(invalid("integer prompt/tokenizer/horizon differs"));
    }
    let mut inputs = vec![0];
    inputs.extend(content);
    let prompt_token_ids = inputs.clone();
    let mut session = model.new_session();
    let mut current = None;
    for (position, &token) in inputs.iter().enumerate() {
        let step = model.step(&mut session, token, mode)?;
        check_read_slots(&step, position, mode)?;
        current = Some(step);
    }
    let mut calls = inputs.len();
    let mut sampler = seed.map(SplitMix64::new);
    let mut decisions = Vec::new();
    let mut generated = Vec::new();
    let mut stop = JointGenerationStop::MaximumNewTokens;
    for decision in 0..max_new_tokens {
        let step = current
            .take()
            .ok_or_else(|| invalid("integer generation missing prediction"))?;
        let row = probability_row(&step, model.config().vocab_size, mode)?;
        let sampler_state_before = sampler.as_ref().map(|rng| rng.state);
        let selected = match sampler.as_mut() {
            Some(rng) => joint_evaluation::sample_top_k_q32(&row, rng)?,
            None => greedy(&step.probabilities)?,
        };
        let selected_probability =
            step.probabilities[selected as usize] as f64 / PROBABILITY_TOTAL as f64;
        if selected_probability <= 0.0 {
            return Err(invalid("integer generated zero-probability token"));
        }
        let mut digest = Sha256::new();
        for probability in &step.probabilities {
            digest.update(probability.to_le_bytes());
        }
        decisions.push(json!({
            "decision":decision,"input_positions":inputs.len(),"selected_token":selected,
            "greedy_token":greedy(&step.probabilities)?,"selected_probability":selected_probability,
            "selected_model_nll_nats":-selected_probability.ln(),"probability_sum_q48":PROBABILITY_TOTAL,
            "probabilities_sha256_le_u64":hex::encode(digest.finalize()),
            "no_read_mass":step.no_read_mass as f64 / PROBABILITY_TOTAL as f64,
            "exposed_causal_slots":step.read_masses.len(),
            "copy_gate":f64::from(step.copy_gate) / GATE_SCALE,
            "sampler_state_before":sampler_state_before,"sampler_state_after":sampler.as_ref().map(|rng| rng.state)
        }));
        generated.push(selected);
        if selected == 1 {
            stop = JointGenerationStop::Eos;
            break;
        }
        if first_sentence && tokenizer.decode_bytes(&generated).contains(&b'.') {
            stop = JointGenerationStop::FirstSentenceBoundary;
            break;
        }
        if let Some(period) = short_cycle_period(&generated) {
            stop = JointGenerationStop::ShortCycle { period };
            break;
        }
        if decision + 1 < max_new_tokens {
            inputs.push(selected);
            let step = model.step(&mut session, selected, mode)?;
            check_read_slots(&step, inputs.len() - 1, mode)?;
            current = Some(step);
            calls += 1;
        }
    }
    let end = generated
        .iter()
        .position(|&token| token == 1)
        .unwrap_or(generated.len());
    let raw_bytes = tokenizer.decode_bytes(&generated);
    let response_bytes = tokenizer.decode_bytes(&generated[..end]);
    Ok(json!({
        "schema":"uor-r4.joint-integer-incremental-generation/1","prompt":prompt,"mode":mode,"seed":seed,
        "tokenizer_cid":tokenizer.address(),"sampler_policy":if seed.is_some() { joint_evaluation::SEEDED_POLICY } else { "integer-Q48-greedy-token-asc/1" },
        "sampler_boundary":"Seeded sampler is the retained host F32-probability/F64-ln-exp procedure; greedy selection compares integer probabilities. Neither reports whole-path integer serving.",
        "bos_policy":"prepend checkpoint BOS0 exactly once; EOS1 ends output",
        "prompt_token_ids":prompt_token_ids,"generated_token_ids":generated,"raw_decoded_bytes":raw_bytes,
        "raw_decoded":String::from_utf8_lossy(&raw_bytes),"response_text":String::from_utf8_lossy(&response_bytes).trim(),
        "utf8_decodable":std::str::from_utf8(&raw_bytes).is_ok() && std::str::from_utf8(&response_bytes).is_ok(),
        "stop":stop,"max_new_tokens":max_new_tokens,"context_capacity":model.config().context,
        "incremental_step_calls":calls,"fresh_session":true,"whole_prompt_read_mode":true,
        "all_causal_slot_counts_verified":true,"maximum_exposed_causal_slots":calls - 1,
        "gradients_tracked":false,"decisions":decisions,"elapsed_seconds":started.elapsed().as_secs_f64()
    }))
}

fn judge_story(
    model: &IntegerModel,
    tokenizer: &HfBpeTokenizer,
    variant: &StoryVariant,
    mode: ReadMode,
) -> Result<Value> {
    let generation = generate(
        model,
        tokenizer,
        &variant.prompt,
        mode,
        None,
        STORY_PROBE_MAX_NEW_TOKENS,
        true,
    )?;
    let text = generation["response_text"]
        .as_str()
        .ok_or_else(|| invalid("integer story response"))?;
    let first = text
        .split(|c: char| !c.is_alphabetic())
        .next()
        .unwrap_or("");
    let complete = generation["utf8_decodable"] == true
        && matches!(
            generation["stop"]["reason"].as_str(),
            Some("eos" | "first_sentence_boundary")
        )
        && answer_oracle::accepts(&variant.accepted_continuations, text);
    Ok(
        json!({"first_noun_correct":first == variant.item.noun(),"complete_correct":complete,"generation":generation}),
    )
}

fn compare_windows(
    integer: &IntegerModel,
    reference: &JointModel,
    tokens: &[u16],
    mode: ReadMode,
    windows: usize,
    output: &Path,
) -> Result<Value> {
    let started = Instant::now();
    let mut rows = BufWriter::new(File::create_new(output)?);
    let mut total_variation_sum = 0.0f64;
    let mut maximum_total_variation = 0.0f64;
    let mut maximum_probability_delta = 0.0f64;
    let mut maximum_state_delta = 0.0f64;
    let mut maximum_gate_delta = 0.0f64;
    let mut maximum_no_read_delta = 0.0f64;
    let mut maximum_exposed_causal_slots = 0usize;
    let mut integer_nll = 0.0f64;
    let mut f32_nll = 0.0f64;
    let mut top1_disagreement = 0usize;
    let mut integer_seconds = 0.0f64;
    let mut f32_seconds = 0.0f64;
    let mut partitions = [[0.0f64; 3]; 2];
    for block in 0..windows {
        let mut integer_session = integer.new_session();
        let mut f32_session = reference.new_session(1)?;
        for position in 0..EVALUATION_CONTEXT {
            let offset = block * EVALUATION_CONTEXT + position;
            let input = u32::from(tokens[offset]);
            let target = u32::from(tokens[offset + 1]);
            let clock = Instant::now();
            let actual = integer.step(&mut integer_session, input, mode)?;
            integer_seconds += clock.elapsed().as_secs_f64();
            check_read_slots(&actual, position, mode)?;
            maximum_exposed_causal_slots =
                maximum_exposed_causal_slots.max(actual.read_masses.len());
            probability_row(&actual, integer.config().vocab_size, mode)?;
            let clock = Instant::now();
            let baseline = reference.step(&mut f32_session, &[input], mode)?;
            f32_seconds += clock.elapsed().as_secs_f64();
            let probabilities = baseline.probabilities.flatten_all()?.to_vec1::<f32>()?;
            let state = baseline.state.flatten_all()?.to_vec1::<f32>()?;
            if actual.state.len() != state.len()
                || state.len() != integer.config().width
                || probabilities.len() != integer.config().vocab_size
                || state.iter().any(|v| !v.is_finite())
                || baseline.probabilities.track_op()
                || baseline.state.track_op()
            {
                return Err(invalid(
                    "integer/F32 state or probability shape/gradient mismatch",
                ));
            }
            let score = joint_evaluation::score_probabilities(&probabilities, target)?;
            let mut probability_delta = 0.0f64;
            let mut total_variation = 0.0f64;
            let mut integer_digest = Sha256::new();
            let mut f32_digest = Sha256::new();
            for (&code, &value) in actual.probabilities.iter().zip(&probabilities) {
                let delta = (code as f64 / PROBABILITY_TOTAL as f64 - f64::from(value)).abs();
                probability_delta = probability_delta.max(delta);
                total_variation += delta * 0.5;
                integer_digest.update(code.to_le_bytes());
                f32_digest.update(value.to_le_bytes());
            }
            let state_delta = actual
                .state
                .iter()
                .zip(&state)
                .map(|(&code, &value)| (f64::from(code) / STATE_SCALE - f64::from(value)).abs())
                .fold(0.0f64, f64::max);
            let probability =
                actual.probabilities[target as usize] as f64 / PROBABILITY_TOTAL as f64;
            if probability <= 0.0 {
                return Err(invalid(
                    "integer target has zero probability; no evaluation floor permitted",
                ));
            }
            let nll = -probability.ln();
            let prediction = greedy(&actual.probabilities)?;
            let baseline_no_read = f64::from(baseline.no_read_mass.to_vec1::<f32>()?[0]);
            let baseline_gate = f64::from(baseline.copy_gate.to_vec1::<f32>()?[0]);
            let no_read = actual.no_read_mass as f64 / PROBABILITY_TOTAL as f64;
            let gate = f64::from(actual.copy_gate) / GATE_SCALE;
            maximum_no_read_delta = maximum_no_read_delta.max((no_read - baseline_no_read).abs());
            maximum_gate_delta = maximum_gate_delta.max((gate - baseline_gate).abs());
            maximum_state_delta = maximum_state_delta.max(state_delta);
            maximum_probability_delta = maximum_probability_delta.max(probability_delta);
            maximum_total_variation = maximum_total_variation.max(total_variation);
            total_variation_sum += total_variation;
            integer_nll += nll;
            f32_nll += score.nll_nats;
            top1_disagreement += usize::from(prediction != score.predicted_token);
            let partition = usize::from(block >= joint_evaluation::CALIBRATION_BLOCKS);
            partitions[partition][0] += 1.0;
            partitions[partition][1] += nll;
            partitions[partition][2] += score.nll_nats;
            serde_json::to_writer(
                &mut rows,
                &json!({
                    "block_index":block,"position_in_block":position,"input_offset":offset,"target_offset":offset+1,
                    "input_token":input,"target_token":target,"integer_prediction":prediction,"f32_prediction":score.predicted_token,
                    "integer_target_probability":probability,"f32_target_probability":score.target_probability,
                    "integer_nll_nats":nll,"f32_nll_nats":score.nll_nats,
                    "maximum_probability_delta":probability_delta,"total_variation":total_variation,
                    "maximum_state_delta":state_delta,"integer_no_read_mass":no_read,"f32_no_read_mass":baseline_no_read,
                    "exposed_causal_slots":actual.read_masses.len(),
                    "integer_copy_gate":gate,"f32_copy_gate":baseline_gate,
                    "integer_probability_sha256_le_u64":hex::encode(integer_digest.finalize()),
                    "f32_probability_sha256_le_f32":hex::encode(f32_digest.finalize())
                }),
            )?;
            writeln!(rows)?;
        }
    }
    rows.flush()?;
    rows.get_ref().sync_all()?;
    let targets = windows * EVALUATION_CONTEXT;
    Ok(json!({
        "same_observed_tokens":true,"full_context_access":true,"windows":windows,"scored_targets":targets,
        "all_causal_slot_counts_verified":true,"maximum_exposed_causal_slots":maximum_exposed_causal_slots,
        "causal_slot_scope":"One Q48 mass per prior occurrence, oldest to newest, with exact count checked at every step. Slots remain exposed when their resulting weight is zero; positive influence on every slot is not asserted.",
        "integer_mean_nll_nats":integer_nll / targets as f64,"f32_mean_nll_nats":f32_nll / targets as f64,
        "mean_nll_delta_nats":(integer_nll-f32_nll) / targets as f64,
        "mean_total_variation":total_variation_sum / targets as f64,"maximum_total_variation":maximum_total_variation,
        "maximum_probability_delta":maximum_probability_delta,"maximum_state_delta":maximum_state_delta,
        "within_prospective_state_bound":maximum_state_delta <= 0.01,
        "within_prospective_probability_bound":maximum_probability_delta <= 0.01,
        "maximum_no_read_mass_delta":maximum_no_read_delta,"maximum_copy_gate_delta":maximum_gate_delta,
        "top1_disagreements":top1_disagreement,"top1_disagreement_fraction":top1_disagreement as f64 / targets as f64,
        "partitions":[
            {"name":"calibration_prefix","targets":partitions[0][0] as usize,"integer_nll_sum":partitions[0][1],"f32_nll_sum":partitions[0][2]},
            {"name":"comparison_tail","targets":partitions[1][0] as usize,"integer_nll_sum":partitions[1][1],"f32_nll_sum":partitions[1][2]}
        ],
        "integer_step_seconds":integer_seconds,"f32_step_seconds":f32_seconds,
        "timing_scope":"Same-process batch1 incremental calls; excludes session allocation, loading, audit, generation and serialization; not a population throughput or energy claim",
        "elapsed_seconds":started.elapsed().as_secs_f64(),"target_rows_sha256":sha256_file(output)?
    }))
}
