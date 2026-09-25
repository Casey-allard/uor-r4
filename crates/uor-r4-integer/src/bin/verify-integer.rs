//! Replay retained integer distributions without a floating model or new data.
#![forbid(unsafe_code)]

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::time::Instant;
use uor_r4_integer::bundle::Bundle;
use uor_r4_integer::generation::Selection;
use uor_r4_integer::{
    format, report_output, sha256_file, IntegerError, ReadMode, Result, SamplePolicy, Sampler,
    PROBABILITY_TOTAL,
};

const WINDOWS: usize = 4;
const CONTEXT: usize = 256;

#[derive(Deserialize)]
struct RetainedRow {
    block_index: usize,
    position_in_block: usize,
    input_token: u32,
    integer_prediction: usize,
    integer_probability_sha256_le_u64: String,
    exposed_causal_slots: usize,
}

fn invalid(message: impl Into<String>) -> IntegerError {
    IntegerError::Invalid(message.into())
}

fn digest_u64(values: &[u64]) -> String {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.to_le_bytes());
    }
    hex::encode(hash.finalize())
}

fn digest_i32(values: &[i32]) -> String {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.to_le_bytes());
    }
    hex::encode(hash.finalize())
}

fn replay_mode(bundle: &Bundle, prior: &Path, output: &Path, mode: ReadMode) -> Result<Value> {
    let label = if mode == ReadMode::Enabled {
        "read"
    } else {
        "no-read"
    };
    let input = prior.join(format!("targets-{label}.jsonl"));
    let output_path = output.join(format!("replay-{label}.jsonl"));
    let mut writer = BufWriter::new(File::create_new(&output_path)?);
    let started = Instant::now();
    let model = bundle.model();
    let mut session = model.new_session();
    let mut sampler = Sampler::new(0);
    let mut count = 0usize;
    let mut mismatches = 0usize;
    let mut probability_mismatches = 0usize;
    let mut prediction_mismatches = 0usize;
    let mut invariant_mismatches = 0usize;
    let mut maximum_slots = 0usize;
    let mut model_nanos = 0u128;
    for line in BufReader::new(File::open(&input)?).lines() {
        let row: RetainedRow = serde_json::from_str(&line?)?;
        if count >= WINDOWS * CONTEXT
            || row.block_index != count / CONTEXT
            || row.position_in_block != count % CONTEXT
            || row.exposed_causal_slots != row.position_in_block
        {
            return Err(invalid(format!(
                "retained {label} row {count} ordering/context differs"
            )));
        }
        if row.position_in_block == 0 {
            session = model.new_session();
        }
        let clock = Instant::now();
        let step = model.step(&mut session, row.input_token, mode)?;
        let step_nanos = clock.elapsed().as_nanos();
        model_nanos += step_nanos;
        let actual_hash = digest_u64(&step.probabilities);
        let total = step.probabilities.iter().try_fold(0u64, |total, &value| {
            total
                .checked_add(value)
                .ok_or_else(|| invalid("probability total overflow"))
        })?;
        let read_total = step
            .read_masses
            .iter()
            .try_fold(step.no_read_mass, |total, &value| {
                total
                    .checked_add(value)
                    .ok_or_else(|| invalid("attention total overflow"))
            })?;
        let selection = sampler.select(&step.probabilities, SamplePolicy::Greedy);
        let prediction = selection.as_ref().ok().copied();
        let probability_equal = actual_hash == row.integer_probability_sha256_le_u64;
        let prediction_equal = prediction == Some(row.integer_prediction);
        let invariant_valid = total == PROBABILITY_TOTAL
            && read_total == PROBABILITY_TOTAL
            && step.probabilities.len() == model.config().vocab_size
            && step.state.len() == model.config().width
            && step.read_masses.len() == row.position_in_block
            && session.len() == row.position_in_block + 1
            && (mode != ReadMode::NoRead
                || (step.no_read_mass == PROBABILITY_TOTAL
                    && step.read_masses.iter().all(|&x| x == 0)));
        probability_mismatches += usize::from(!probability_equal);
        prediction_mismatches += usize::from(!prediction_equal);
        invariant_mismatches += usize::from(!invariant_valid);
        mismatches += usize::from(!(probability_equal && prediction_equal && invariant_valid));
        maximum_slots = maximum_slots.max(step.read_masses.len());
        let result = json!({
            "block_index":row.block_index, "position_in_block":row.position_in_block,
            "input_token":row.input_token, "prediction":prediction,
            "retained_prediction":row.integer_prediction,
            "probability_sha256_le_u64":actual_hash,
            "retained_probability_sha256_le_u64":row.integer_probability_sha256_le_u64,
            "state_sha256_le_i32":digest_i32(&step.state),
            "probability_equal":probability_equal, "prediction_equal":prediction_equal,
            "invariants_valid":invariant_valid, "probability_mass":total,
            "attention_mass":read_total, "exposed_causal_slots":step.read_masses.len(),
            "model_step_ns":step_nanos, "selection_error":selection.err().map(|e| e.to_string())
        });
        serde_json::to_writer(&mut writer, &result)?;
        writer.write_all(b"\n")?;
        count += 1;
    }
    writer.flush()?;
    if count != WINDOWS * CONTEXT {
        return Err(invalid(format!(
            "retained {label} has {count} rows; expected1024"
        )));
    }
    Ok(json!({
        "mode":label, "targets":count, "windows":WINDOWS, "context":CONTEXT,
        "mismatches":mismatches, "probability_mismatches":probability_mismatches,
        "prediction_mismatches":prediction_mismatches, "invariant_mismatches":invariant_mismatches,
        "maximum_exposed_causal_slots":maximum_slots, "model_calls":count,
        "model_call_ns":model_nanos, "mode_wall_ns":started.elapsed().as_nanos(),
        "input_file_sha256":sha256_file(&input)?, "output_file_sha256":sha256_file(&output_path)?,
        "pass":mismatches == 0
    }))
}

// One loaded-artifact boundary check, independent of language-quality scoring.
fn check_text_session(bundle: &Bundle, prior: &Path) -> Result<Value> {
    let started = Instant::now();
    let source = prior.join("story-probes-read-integer.jsonl");
    let line = BufReader::new(File::open(&source)?)
        .lines()
        .next()
        .ok_or_else(|| invalid("missing retained source prompt"))??;
    let row: Value = serde_json::from_str(&line)?;
    let prompt = row["original"]["generation"]["prompt"]
        .as_str()
        .ok_or_else(|| invalid("retained source row lacks natural prompt"))?;
    let followup = " Then they went home.";
    let first_ids = bundle.tokenizer().encode(prompt);
    let followup_ids = bundle.tokenizer().encode(followup);
    let mut session = bundle.text_session(ReadMode::Enabled)?;
    let first_appended = session.append(prompt)?;
    let before_first = session.len();
    let first = session.generate(1, Selection::Greedy, false)?;
    let first_token = *first
        .generated_token_ids
        .first()
        .ok_or_else(|| invalid("first text call emitted no token"))?;
    let after_first = session.len();
    let second_appended = session.append(followup)?;
    let after_append = session.len();
    let second = session.generate(1, Selection::Greedy, false)?;
    let second_decision = second
        .decisions
        .first()
        .ok_or_else(|| invalid("second text call has no decision"))?;
    let before_errors = session.len();

    // Replay the complete occurrence tape directly, including the pending token
    // exactly once between user appends; this does not use TextSession internals.
    let mut raw_session = bundle.model().new_session();
    let raw_tokens: Vec<u32> = std::iter::once(0)
        .chain(first_ids.iter().copied())
        .chain(std::iter::once(first_token))
        .chain(followup_ids.iter().copied())
        .collect();
    let raw_clock = Instant::now();
    let mut raw_last = None;
    for &token in &raw_tokens {
        raw_last = Some(
            bundle
                .model()
                .step(&mut raw_session, token, ReadMode::Enabled)?,
        );
    }
    let raw_model_ns = raw_clock.elapsed().as_nanos();
    let raw_last = raw_last.ok_or_else(|| invalid("empty raw occurrence replay"))?;
    let expected_hash = digest_u64(&raw_last.probabilities);
    let expected_token = Sampler::new(0)
        .select(&raw_last.probabilities, SamplePolicy::Greedy)
        .map_err(|e| invalid(e.to_string()))?;
    let append_error = session
        .append(&" x".repeat(257))
        .err()
        .map(|e| e.to_string());
    let after_append_error = session.len();
    let generation_error = session
        .generate(256, Selection::Greedy, false)
        .err()
        .map(|e| e.to_string());
    let after_generation_error = session.len();
    let pass = second_decision.probability_sha256_le_u64 == expected_hash
        && second_decision.selected_token as usize == expected_token
        && first.generated_token_ids.len() == 1
        && second.generated_token_ids.len() == 1
        && first.incremental_step_calls == 0
        && second.incremental_step_calls == 0
        && first_appended == first_ids.len()
        && second_appended == followup_ids.len()
        && before_first == 1 + first_ids.len()
        && after_first == before_first + 1
        && after_append == after_first + followup_ids.len()
        && before_errors == after_append + 1
        && raw_session.len() == after_append
        && append_error.is_some()
        && generation_error.is_some()
        && after_append_error == before_errors
        && after_generation_error == before_errors;
    Ok(json!({
        "schema":"uor-r4.loaded-text-session-boundary-check/1", "pass":pass,
        "source_file_sha256":sha256_file(&source)?, "source_prompt":prompt,
        "source_prompt_token_ids":first_ids, "followup_text":followup,
        "followup_token_ids":followup_ids, "first_generated_token_ids":first.generated_token_ids,
        "second_generated_token_ids":second.generated_token_ids,
        "raw_occurrence_tape_token_ids":raw_tokens,
        "raw_second_prediction":expected_token, "raw_second_probability_sha256_le_u64":expected_hash,
        "text_second_probability_sha256_le_u64":second_decision.probability_sha256_le_u64,
        "first_generate_model_calls":first.incremental_step_calls,
        "second_generate_model_calls":second.incremental_step_calls,
        "lengths":{"before_first":before_first,"after_first":after_first,"after_append":after_append,
            "before_errors":before_errors,"after_append_error":after_append_error,
            "after_generation_error":after_generation_error},
        "oversized_append_error":append_error, "oversized_generate_error":generation_error,
        "raw_replay_model_calls":raw_session.len(), "raw_replay_model_ns":raw_model_ns,
        "complete_boundary_check_wall_ns":started.elapsed().as_nanos(),
        "scope":"One actual artifact session continuation and capacity check; no language-quality claim"
    }))
}

fn run(bundle_path: &Path, prior: &Path, output: &Path) -> Result<Value> {
    format::verify_sealed(prior)?;
    let load_clock = Instant::now();
    let bundle = Bundle::load(bundle_path)?;
    let load_nanos = load_clock.elapsed().as_nanos();
    if bundle.model().config().context != CONTEXT {
        return Err(invalid("replay requires retained256 context"));
    }
    let read = replay_mode(&bundle, prior, output, ReadMode::Enabled)?;
    let no_read = replay_mode(&bundle, prior, output, ReadMode::NoRead)?;
    let text_session = check_text_session(&bundle, prior)?;
    let pass = read["pass"] == true && no_read["pass"] == true && text_session["pass"] == true;
    Ok(json!({
        "schema":"uor-r4.standalone-integer-retained-replay/1", "pass":pass,
        "bundle_receipt_sha256":sha256_file(&bundle_path.join("bundle.json"))?,
        "prior_report_manifest_sha256":sha256_file(&prior.join("manifest.json"))?,
        "bundle_load_ns":load_nanos, "read":read, "no_read":no_read, "text_session":text_session,
        "scope":"Exact extraction replay of the existing four full256 windows per mode; no new population, floating model or tolerance",
        "acceptance":"Zero differences in every retained Q48 distribution SHA256 and greedy token; exact normalized probability/attention mass and all causal slots"
    }))
}

fn main() {
    if let Err(error) = main_result() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn main_result() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err(invalid(
            "usage: verify-integer BUNDLE PRIOR_INTEGER_REPORT_ROOT NEW_REPORT_ROOT",
        ));
    }
    let started = Instant::now();
    let bundle = Path::new(&args[0]);
    let prior = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    report_output::claim(output)?;
    let result = run(bundle, prior, output);
    let mut summary = match &result {
        Ok(value) => value.clone(),
        Err(error) => json!({
            "schema":"uor-r4.standalone-integer-retained-replay/1",
            "pass":false, "error":error.to_string()
        }),
    };
    summary["complete_wall_through_summary_ns"] = json!(started.elapsed().as_nanos());
    fs::write(
        output.join("summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    report_output::seal(output)?;
    report_output::verify(output)?;
    println!("{}", serde_json::to_string(&summary)?);
    result?;
    if summary["pass"] != true {
        return Err(invalid(
            "retained integer replay differs; sealed mismatch evidence retained",
        ));
    }
    Ok(())
}
