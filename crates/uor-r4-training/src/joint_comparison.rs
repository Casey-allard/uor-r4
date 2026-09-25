//! Same-target comparison of completed D8 evaluations, without loading models.
//!
//! This reads the frozen evaluator-v2 population in corpus order. Seals bind
//! the original evaluations; the derived report checks every target identity
//! and reproduces their means. Numerical component gates do not qualify
//! generation, integer serving, a geometry advantage, or fresh held-out quality.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;

use crate::baseline_protocol::save_json;
use crate::joint_evaluation::PROBABILITY_SUM_TOLERANCE;
use crate::joint_model::{JointConfig, ReadMode, Transport};
use crate::{invalid, sha256_file, Result};

const CONTEXT: usize = 256;
const BLOCKS: usize = 976;
const TUNE_BLOCKS: usize = 64;
const TARGETS: usize = BLOCKS * CONTEXT;
const RECORD_BYTES: usize = 44;
const SUMMARY_TOLERANCE: f64 = 1e-10;
const CACHE_GATE_NLL: f64 = 2.391786178860742;
const READ_GATE_NATS: f64 = 0.02;
const EVALUATOR: &[u8] = include_bytes!("../../../docs/integration/reference-evaluator-v2.json");
const EVALUATOR_SHA: &str = "d2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62";
const REFERENCE_HEADER: &str = "block,position,input_offset,target_offset,input_token,target_token,predicted_token,target_logit,log_normalizer,nll_nats,correct";
const COUNT_HEADER: &str = "block,position,target_offset,input,target,ngram_probability,cache_probability,ngram_nll_nats,cache_nll_nats";
const ARM_NAMES: [&str; 7] = [
    "reference",
    "ngram",
    "cache",
    "quaternion_read",
    "quaternion_no_read",
    "ordinary_read",
    "ordinary_no_read",
];
// A delta is candidate NLL minus comparator NLL. Negative is an improvement.
const PAIRS: [(usize, usize); 10] = [
    (3, 0),
    (3, 1),
    (3, 2),
    (5, 0),
    (5, 1),
    (5, 2),
    (3, 4),
    (5, 6),
    (3, 5),
    (4, 6),
];
const SCOPE: &str = "Artifact-scoped exploratory comparison on previously exposed development. No statistical geometry promotion, fresh final holdout, generation-quality verdict, integer-serving qualification, or energy claim.";

/// `joint-compare BASELINE_ROOT QUAT_READ QUAT_NOREAD ORD_READ ORD_NOREAD NEW_ROOT`.
/// The baseline container contributes its two sealed evaluation children.
pub fn run_cli(args: &[String]) -> Result<()> {
    let started = Instant::now();
    if args.len() != 7
        || args.first().map(String::as_str) != Some("joint-compare")
        || args[1..].iter().any(|arg| arg.is_empty())
    {
        return Err(invalid(
            "usage: joint-compare BASELINE_ROOT QUAT_READ QUAT_NOREAD ORD_READ ORD_NOREAD NEW_ROOT",
        ));
    }
    // Path validation precedes claiming. In particular, never create a derived
    // attempt beneath a sealed input or the preserved baseline container.
    let baseline = fs::canonicalize(&args[1])?;
    let mut roots = vec![
        fs::canonicalize(baseline.join("reference-evaluation"))?,
        fs::canonicalize(baseline.join("count-evaluation"))?,
    ];
    for path in &args[2..6] {
        roots.push(fs::canonicalize(path)?);
    }
    let unique: BTreeSet<_> = roots.iter().collect();
    let out = new_output_path(Path::new(&args[6]))?;
    if unique.len() != 6
        || roots
            .iter()
            .any(|root| !root.is_dir() || out.starts_with(root))
        || out.starts_with(&baseline)
    {
        return Err(invalid(
            "comparison needs six distinct input directories and a separate new output root",
        ));
    }
    report_output::claim(&out)?;
    let mut bindings = Vec::new();
    let result = compare(&roots, &out, started, &mut bindings);

    // Attempt both receipts and sealing even after an input/row validation
    // failure, retaining any completed block summaries as a failed attempt.
    let mut closeout_errors = Vec::new();
    if let Err(error) = save_json(
        &out.join("input-bindings.json"),
        &json!({
            "schema":"uor-r4.joint-comparison-inputs/1", "requested_roots":roots,
            "verified_inputs":bindings,
            "checkpoint_binding_scope":"Bindings copied from sealed evaluation reports. This comparison does not reload or rehash the external model checkpoints."
        }),
    ) {
        closeout_errors.push(error.to_string());
    }
    if result.is_err() || !closeout_errors.is_empty() {
        if let Err(write_error) = save_json(
            &out.join("failed-attempt.json"),
            &json!({
                "status":"FAILED_ATTEMPT", "complete_population":false,
                "operation_error":result.as_ref().err().map(ToString::to_string),
                "receipt_errors":closeout_errors,
                "elapsed_seconds_before_sealing":started.elapsed().as_secs_f64(),
                "partial_blocks_policy":"Any retained block rows are incomplete evidence; no numerical gate passes on a failed attempt."
            }),
        ) {
            closeout_errors.push(write_error.to_string());
        }
    }
    if let Err(error) = report_output::seal(&out) {
        closeout_errors.push(error.to_string());
    }
    if let Err(error) = report_output::verify(&out) {
        closeout_errors.push(error.to_string());
    }
    eprintln!(
        "joint comparison {}: {:.3}s end-to-end, {}",
        if result.is_ok() && closeout_errors.is_empty() {
            "complete"
        } else {
            "failed"
        },
        started.elapsed().as_secs_f64(),
        out.display()
    );
    if !closeout_errors.is_empty() {
        return Err(invalid(format!(
            "comparison closeout failed: {}; operation: {}",
            closeout_errors.join("; "),
            result
                .as_ref()
                .err()
                .map_or_else(|| "completed".to_owned(), ToString::to_string)
        )));
    }
    result
}

fn new_output_path(path: &Path) -> Result<PathBuf> {
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(invalid("new comparison output must not contain '..'"));
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    if absolute.try_exists()? {
        return Err(invalid(
            "comparison output already exists; choose a new attempt",
        ));
    }
    let mut ancestor = absolute.as_path();
    let mut missing = Vec::new();
    while !ancestor.try_exists()? {
        missing.push(
            ancestor
                .file_name()
                .ok_or_else(|| invalid("new output leaf"))?
                .to_owned(),
        );
        ancestor = ancestor
            .parent()
            .ok_or_else(|| invalid("new output parent"))?;
    }
    let mut resolved = fs::canonicalize(ancestor)?;
    for part in missing.into_iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}

fn compare(
    roots: &[PathBuf],
    out: &Path,
    started: Instant,
    bindings: &mut Vec<Value>,
) -> Result<()> {
    let source = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND");
    if source == "UNBOUND" || hex::encode(Sha256::digest(EVALUATOR)) != EVALUATOR_SHA {
        return Err(invalid(
            "joint comparison requires a source-bound build and unchanged frozen evaluator v2",
        ));
    }
    let evaluator: Value = serde_json::from_slice(EVALUATOR)?;
    save_json(&out.join("evaluator.json"), &evaluator)?;
    save_json(
        &out.join("run-provenance.json"),
        &json!({
            "schema":"uor-r4.joint-comparison-provenance/1", "source_commit":source,
            "executable_sha256":sha256_file(&std::env::current_exe()?)?,
            "evaluator_sha256":EVALUATOR_SHA, "scope":SCOPE,
            "model_executions":0, "optimizer_steps":0
        }),
    )?;
    let reference = read_input(
        &roots[0],
        "reference",
        "reference-evaluation.json",
        "reference-targets.csv",
        bindings,
    )?;
    let count = read_input(
        &roots[1],
        "counts",
        "count-evaluation.json",
        "count-targets.csv",
        bindings,
    )?;
    for root in &roots[..2] {
        if read_json(&root.join("evaluator.json"))? != evaluator {
            return Err(invalid(format!(
                "{} evaluator snapshot differs from frozen v2",
                root.display()
            )));
        }
    }
    validate_baselines(&reference, &count, &evaluator)?;
    let expected = [
        (Transport::Quaternion, ReadMode::Enabled),
        (Transport::Quaternion, ReadMode::NoRead),
        (Transport::HouseholderPair, ReadMode::Enabled),
        (Transport::HouseholderPair, ReadMode::NoRead),
    ];
    let mut native = Vec::with_capacity(4);
    for (index, &(transport, mode)) in expected.iter().enumerate() {
        let report = read_input(
            &roots[index + 2],
            ARM_NAMES[index + 3],
            "evaluation-report.json",
            "targets.bin",
            bindings,
        )?;
        validate_joint(&report, transport, mode)?;
        native.push(report);
    }
    validate_pairs(&native)?;

    let mut reference_rows =
        CsvRows::open(&roots[0].join("reference-targets.csv"), REFERENCE_HEADER)?;
    let mut count_rows = CsvRows::open(&roots[1].join("count-targets.csv"), COUNT_HEADER)?;
    let mut joint_rows = roots[2..]
        .iter()
        .map(|root| {
            let path = root.join("targets.bin");
            if fs::metadata(&path)?.len() != (TARGETS * RECORD_BYTES) as u64 {
                return Err(invalid(format!(
                    "{} must contain exactly {TARGETS} complete 44-byte records",
                    path.display()
                )));
            }
            Ok(BufReader::new(File::open(path)?))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut partitions = [Accumulator::default(); 3]; // tune, comparison, full
    let mut horizons = [[Accumulator::default(); 2]; 3]; // positions 0..64, 64..256
    let mut blocks = BufWriter::new(File::create_new(out.join("comparison-blocks.jsonl"))?);
    let scan = (|| -> Result<()> {
        let mut previous_target = None;
        for block in 0..BLOCKS {
            let mut block_stats = Accumulator::default();
            for position in 0..CONTEXT {
                let offset = block * CONTEXT + position;
                let row = reference_row(&mut reference_rows, offset)?;
                if previous_target.is_some_and(|target| target != row.input) {
                    return Err(invalid(format!(
                        "reference input/previous-target chain differs at offset {offset}"
                    )));
                }
                previous_target = Some(row.target);
                let counts = count_row(&mut count_rows, offset, &row)?;
                let mut losses = [row.nll, counts[0], counts[1], 0.0, 0.0, 0.0, 0.0];
                for index in 0..4 {
                    losses[index + 3] = joint_row(
                        &mut joint_rows[index],
                        offset,
                        row.target,
                        expected[index].1,
                        ARM_NAMES[index + 3],
                    )?;
                }
                block_stats.add(losses);
                let partition = usize::from(block >= TUNE_BLOCKS);
                let horizon = usize::from(position >= 64);
                partitions[partition].add(losses);
                partitions[2].add(losses);
                horizons[partition][horizon].add(losses);
                horizons[2][horizon].add(losses);
            }
            serde_json::to_writer(
                &mut blocks,
                &json!({
                    "block":block, "partition":if block < TUNE_BLOCKS { "tune" } else { "comparison" },
                    "input_start":block*CONTEXT, "first_target_offset":block*CONTEXT+1,
                    "target_end_exclusive":(block+1)*CONTEXT+1, "statistics":block_stats.report()
                }),
            )?;
            writeln!(blocks)?;
        }
        reference_rows.require_eof()?;
        count_rows.require_eof()?;
        for (index, reader) in joint_rows.iter_mut().enumerate() {
            let mut extra = [0u8; 1];
            if reader.read(&mut extra)? != 0 {
                return Err(invalid(format!(
                    "{} has records after the complete population",
                    ARM_NAMES[index + 3]
                )));
            }
        }
        Ok(())
    })();
    let flushed = blocks.flush().and_then(|()| blocks.get_ref().sync_all());
    scan?;
    flushed?;
    let agreement = check_reported_means(&partitions, &reference, &count, &native)?;
    if (partitions[1].mean(2) - CACHE_GATE_NLL).abs() > SUMMARY_TOLERANCE {
        return Err(invalid(
            "retained cache comparison mean no longer matches the frozen gate",
        ));
    }
    let exposure_differs = native[0]["checkpoint_binding"]["sampled_target_visits"]
        != native[2]["checkpoint_binding"]["sampled_target_visits"];
    let phase_exposure = json!({
        "quaternion":training_exposure(&native[0])?,
        "ordinary":training_exposure(&native[2])?
    });
    let horizon_scope = if native[0]["campaign"]["context"] == CONTEXT {
        "Prespecified descriptive slice of the same rows: positions0..63 versus64..255. Both slices lie within the current256-token training unroll; the boundary distinguishes early and later positions, not trained versus untrained positions. With a declared64-token warmup, phase exposure is reported separately. No primary gate is changed or decided on a slice."
    } else {
        "Prespecified descriptive slice of the same rows: positions0..63 versus64..255. For a64-window fit without256-token continuation, later positions extend beyond its independently reset training horizon. Consult the bound phase exposures; no primary gate is changed or decided on a slice."
    };
    let position_horizons: serde_json::Map<String, Value> = ["tune", "comparison", "full"]
        .iter()
        .zip(&horizons)
        .map(|(name, slices)| {
            (
                (*name).to_owned(),
                json!({
                    "positions_0_through_63":slices[0].means_report(),
                    "positions_64_through_255":slices[1].means_report()
                }),
            )
        })
        .collect();
    let numeric_gates = [(3, 4), (5, 6)]
        .into_iter()
        .map(|(read, no_read)| {
            let delta = -partitions[1].paired_delta(read, no_read)?;
            Ok(
                json!({"arm":ARM_NAMES[read], "comparison_targets":(BLOCKS-TUNE_BLOCKS)*CONTEXT,
            "enabled_mean_nll_nats":partitions[1].mean(read),
            "frozen_cache_threshold_nats":CACHE_GATE_NLL,
            "likelihood_component_pass":partitions[1].mean(read)<CACHE_GATE_NLL,
            "whole_prefix_no_read_mean_nll_nats":partitions[1].mean(no_read),
            "no_read_minus_enabled_nats":delta, "minimum_read_penalty_nats":READ_GATE_NATS,
            "read_utility_component_pass":delta>=READ_GATE_NATS,
            "generation_quality_gate":"NOT_ASSESSED_BY_COMPARISON",
            "rung1_complete":"NOT_DETERMINED_BY_NUMERICAL_COMPONENTS"}),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    save_json(
        &out.join("comparison-report.json"),
        &json!({
            "schema":"uor-r4.joint-baseline-comparison/1", "status":"COMPLETE", "complete_population":true,
            "scope":SCOPE, "evaluator_sha256":EVALUATOR_SHA, "context":CONTEXT, "stride":CONTEXT,
            "joined_targets":TARGETS, "blocks":BLOCKS, "tune_blocks":TUNE_BLOCKS, "comparison_blocks":BLOCKS-TUNE_BLOCKS,
            "identity_join":"Exact contiguous input offset and shifted target token in all six streams; CSV block/position, input IDs and target offsets also match. Missing, duplicate, reordered, extra and nonfinite rows fail.",
            "input_identity_limit":"Native binary rows omit input token IDs. The common input population is bound by evaluator/checkpoint metadata and exact shifted target identities, not by a native per-row input-ID column.",
            "row_numeric_validation":"Reference F32 logit round-trip and F64 log-normalizer reproduce NLL; native target probabilities reproduce NLL; count probabilities respect their fixed-17-decimal serialization precision. No probability renormalization or rescoring.",
            "paired_delta_convention":"candidate NLL minus comparator NLL; negative is better. Improved/equal/worse use exact parsed F64 ordering, without an epsilon; they are descriptive dependent target counts, not significance tests.",
            "partitions":{"tune":partitions[0].report(),"comparison":partitions[1].report(),"full":partitions[2].report()},
            "position_horizon_means":position_horizons,
            "training_window":native[0]["campaign"]["context"],
            "position_split_matches_training_window":native[0]["campaign"]["context"] == 64,
            "position_horizon_scope":horizon_scope,
            "original_summary_agreement":agreement, "numerical_component_gates":numeric_gates,
            "selected_checkpoint_exposure":{
                "quaternion":native[0]["checkpoint_binding"], "ordinary":native[2]["checkpoint_binding"],
                "phase_exposure":phase_exposure,
                "different_selected_optimization_exposure":exposure_differs,
                "interpretation":if exposure_differs {
                    "Selected weights have different optimization exposure, additionally confounding geometry attribution. The declared planned dose and development/checkpoint settings match; this reader does not independently audit completion of both full fitting jobs or checkpoint selection."
                } else { "Selected target visits match. One seed and this artifact comparison do not establish a geometric advantage." }
            },
            "read_control":"Read and NoRead bind to the same checkpoint within each arm; NoRead rows have exact null mass1 and no selected prior occurrence. This reader does not replay the models or reconstruct full read/probability distributions.",
            "checkpoint_selection":"Caller owns the prospectively frozen selection or final-step rule. This comparison consumes the supplied roots and never reselects using its later F64 row reductions; it does not independently validate a caller's selection receipt. Rung1 used F32 quick_loss tune selection; rung2 declares the final common step.",
            "loaded_generation_evidence":"Original sealed roots retain actual generations and story probes. Their file identities are recorded; correctness, collapse and task-transfer interpretation require inspecting those responses separately.",
            "geometry_promotion":false, "blocks_sha256":sha256_file(&out.join("comparison-blocks.jsonl"))?,
            "elapsed_seconds_before_sealing":started.elapsed().as_secs_f64()
        }),
    )?;
    Ok(())
}

fn read_json(path: &Path) -> Result<Value> {
    if fs::metadata(path)?.len() > 2 * 1024 * 1024 {
        return Err(invalid(format!(
            "comparison metadata exceeds 2MiB: {}",
            path.display()
        )));
    }
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn digest_field(value: &Value, field: &str, length: usize) -> Result<()> {
    if !value[field]
        .as_str()
        .is_some_and(|s| s.len() == length && s.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(invalid(format!("missing or invalid {field} identity")));
    }
    Ok(())
}

fn read_input(
    root: &Path,
    name: &str,
    report_file: &str,
    rows_file: &str,
    bindings: &mut Vec<Value>,
) -> Result<Value> {
    report_output::verify(root)?;
    if root.join("failed-attempt.json").try_exists()? {
        return Err(invalid(format!("{name} input is a failed attempt")));
    }
    let report = read_json(&root.join(report_file))?;
    if report["evaluator_sha256"] != EVALUATOR_SHA {
        return Err(invalid(format!(
            "{name} evaluator SHA differs from frozen v2"
        )));
    }
    digest_field(&report, "source_commit", 40)?;
    digest_field(&report, "executable_sha256", 64)?;
    let rows_sha = sha256_file(&root.join(rows_file))?;
    if rows_file == "targets.bin" && report["targets_sha256"] != rows_sha {
        return Err(invalid(format!("{name} target SHA binding differs")));
    }
    let mut binding = json!({
        "name":name,"root":root,"sealed_file_set_verified":true,
        "manifest_sha256":sha256_file(&root.join(report_output::MANIFEST_FILE))?,
        "report_file":report_file,"report_sha256":sha256_file(&root.join(report_file))?,
        "targets_file":rows_file,"targets_sha256":rows_sha,"targets_bytes":fs::metadata(root.join(rows_file))?.len(),
        "evaluator_sha256":report["evaluator_sha256"],"source_commit":report["source_commit"],
        "executable_sha256":report["executable_sha256"],"campaign":report["campaign"],
        "checkpoint":report["checkpoint"],"checkpoint_binding":report["checkpoint_binding"],
        "evaluation_operation":report["mode"],"artifact":report["artifact"],
        "executed_quantization":report["executed_quantization"],
        "reference_inputs":report["inputs"],"count_model_sha256":report["model_sha256"],
        "count_selection":report["selection"],"requested_device":report["requested_device"]
    });
    if rows_file == "targets.bin" {
        binding["generations_sha256"] = json!(sha256_file(&root.join("generations.json"))?);
        binding["story_probes_sha256"] = json!(sha256_file(&root.join("story-probes.json"))?);
    } else {
        binding["evaluator_snapshot_sha256"] = json!(sha256_file(&root.join("evaluator.json"))?);
    }
    bindings.push(binding);
    Ok(report)
}

fn validate_baselines(reference: &Value, count: &Value, evaluator: &Value) -> Result<()> {
    if reference["schema"] != "uor-r4.reference-baselines-report/1"
        || reference["mode"] != "reference-evaluate"
        || reference["status"] != "REFERENCE_EVALUATION_PASS"
        || reference["complete_population"] != true
        || reference["full"]["complete"] != true
        || reference["full"]["completed_blocks"] != BLOCKS
        || reference["full"]["context_length"] != CONTEXT
        || reference["full"]["stride"] != CONTEXT
        || reference["full"]["input_token_count"] != 250000
        || reference["full"]["gradients_tracked"] != false
        || reference["full"]["optimizer_steps"] != 0
        || reference["inputs"]["development"] != evaluator["dev_source"]
        || reference["inputs"]["reference"] != evaluator["reference_inputs"]
        || count["schema"] != "uor-r4.reference-baselines-report/1"
        || count["mode"] != "ngram-evaluate"
        || count["status"] != "COMPLETE"
        || count["model_fit_evaluator_sha256"] != EVALUATOR_SHA
        || count["neural_optimizer_steps"] != 0
        || count["selection"]["selected_discount"] != 0.9
        || count["selection"]["selected_mixture"] != 0.05
        || count["selection"]["tune_blocks"] != TUNE_BLOCKS
    {
        return Err(invalid(
            "baseline report is incomplete or differs from the retained evaluator/configuration",
        ));
    }
    digest_field(count, "model_sha256", 64)
}

fn gradient_shards(value: &Value) -> Result<u64> {
    match value
        .get("cpu_gradient_shards")
        .map_or(Some(1), Value::as_u64)
    {
        Some(shards @ (1 | 2 | 4)) => Ok(shards),
        _ => Err(invalid("invalid CPU gradient shard binding")),
    }
}

fn validate_joint(report: &Value, transport: Transport, mode: ReadMode) -> Result<()> {
    let evaluation = &report["evaluation"];
    let config: JointConfig = serde_json::from_value(report["campaign"]["model"].clone())?;
    config.validate()?;
    let binding = &report["checkpoint_binding"];
    let step = unsigned(binding, "optimizer_step")?;
    let campaign = &report["campaign"];
    let batch = unsigned(campaign, "batch")?;
    let context = unsigned(campaign, "context")?;
    let visits = step.checked_mul(batch).and_then(|n| n.checked_mul(context));
    if report["schema"] != "uor-r4.joint-recurrent-report/1"
        || !matches!(
            report["mode"].as_str(),
            Some("joint-evaluate" | "joint-evaluate-hard" | "joint-evaluate-shadow")
        )
        || campaign["schema"] != "uor-r4.joint-recurrent-campaign/1"
        || !(1..=64).contains(&batch)
        || !(8..=256).contains(&context)
        || batch % gradient_shards(campaign)? != 0
        || gradient_shards(campaign)? != gradient_shards(binding)?
        || config.context != CONTEXT
        || config.transport != transport
        || evaluation["schema"] != "uor-r4.joint-population-evaluation/1"
        || evaluation["mode"] != json!(mode)
        || evaluation["complete"] != true
        || evaluation["gradients_tracked"] != false
        || evaluation["optimizer_steps"] != 0
        || evaluation["context"] != CONTEXT
        || evaluation["stride"] != CONTEXT
        || evaluation["completed_blocks"] != BLOCKS
        || evaluation["input_token_count"] != 250000
        || evaluation["unscored_tail_targets"] != 143
        || binding["schema"] != "uor-r4.joint-recurrent-checkpoint/1"
        || binding["evaluator_sha256"] != EVALUATOR_SHA
        || binding["next_data_step"] != step
        || binding["data_seed"] != campaign["data_seed"]
        || binding["data_sampler"]
            != "splitmix64-counter-v1;valid-window-union;no-cross-store;step/lane-bound"
        || step == 0
        || step > unsigned(campaign, "total_steps")?
        || visits.is_none()
        || binding["sampled_target_visits"].as_u64() != visits
        || report["targets_format"]["record_bytes"] != RECORD_BYTES
        || report["targets_format"]["endianness"] != "little"
        || report["targets_format"]["fields"]
            != json!([
                "input_offset:u64",
                "target:u32",
                "predicted:u32",
                "target_probability:f64",
                "nll_nats:f64",
                "no_read_mass:f32",
                "copy_gate:f32",
                "top_read_position:i32(-1=None)"
            ])
    {
        return Err(invalid(format!("{transport:?}/{mode:?} evaluation, format, configuration or checkpoint binding differs")));
    }
    digest_field(binding, "model_sha256", 64)?;
    digest_field(binding, "source_commit", 40)?;
    quantization_exposure(report)?;
    training_exposure(report)?;
    Ok(())
}

/// Quantization scales can differ across separately calibrated arms. The exact
/// within-arm spec and clock remain bound to the checkpoint and executed path.
fn quantization_exposure(report: &Value) -> Result<Value> {
    let campaign = &report["campaign"];
    let binding = &report["checkpoint_binding"];
    let transition = &campaign["quantization_transition"];
    let state = &binding["quantization"];
    if binding["quantization_transition"] != *transition {
        return Err(invalid(
            "campaign/checkpoint quantization transition differs",
        ));
    }
    if transition.is_null() {
        if !state.is_null()
            || report["mode"] != "joint-evaluate"
            || !report["executed_quantization"].is_null()
        {
            return Err(invalid(
                "continuous report has undeclared quantization or shadow mode",
            ));
        }
        return Ok(Value::Null);
    }
    digest_field(transition, "parent_checkpoint_sha256", 64)?;
    digest_field(transition, "parent_campaign_sha256", 64)?;
    let start = unsigned(transition, "parent_optimizer_step")?;
    let ramp = unsigned(transition, "ramp_steps")?;
    let step = unsigned(binding, "optimizer_step")?;
    if start == 0
        || ramp == 0
        || step < start
        || campaign["batch"] != 16
        || campaign["context"] != 256
        || !transition["reason"]
            .as_str()
            .is_some_and(|reason| !reason.trim().is_empty())
        || state["start_step"] != start
        || state["ramp_steps"] != ramp
        || state["completed_step"] != step
        || !state["spec"].is_object()
    {
        return Err(invalid(
            "invalid quantized phase schedule or checkpoint clock",
        ));
    }
    if report["mode"] == "joint-evaluate-shadow" {
        if !report["executed_quantization"].is_null()
            || report["evaluation_quantization_strength"] != 0.0
        {
            return Err(invalid("shadow report executes a quantized path"));
        }
    } else if report["executed_quantization"] != *state
        || report["evaluation_quantization_strength"] != 1.0
    {
        return Err(invalid(
            "quantized evaluation differs from full-strength checkpoint specification",
        ));
    }
    if report["mode"] == "joint-evaluate-hard"
        && (report["artifact"]["kind"] != "packed_hard_export"
            || report["artifact"]["hard_model_manifest_sha256"] != binding["model_sha256"])
    {
        return Err(invalid("packed evaluation manifest binding differs"));
    }
    Ok(json!({"parent_optimizer_step":start,"ramp_steps":ramp,
        "completed_quantized_training_updates":step-start,
        "quantized_training_target_visits":(step-start)*4096,
        "quantization_transition":transition,"quantization":state,
        "evaluation_operation":report["mode"],
        "scope":"Quantized numerical emulator or its explicit continuous shadow. Packed parameter codes do not establish an integer serving kernel."}))
}

/// Account for the declared single window transition without treating warmup
/// visits as training at the final horizon. Parent hashes remain arm-specific;
/// the training driver checked them against the actual sealed parent files.
fn training_exposure(report: &Value) -> Result<Value> {
    let campaign = &report["campaign"];
    let binding = &report["checkpoint_binding"];
    let transition = &campaign["training_window_transition"];
    if binding["training_window_transition"] != *transition {
        return Err(invalid("campaign/checkpoint training transition differs"));
    }
    let step = unsigned(binding, "optimizer_step")?;
    let total = unsigned(binding, "sampled_target_visits")?;
    let batch = unsigned(campaign, "batch")?;
    let context = unsigned(campaign, "context")?;
    let per_step = batch
        .checked_mul(context)
        .ok_or_else(|| invalid("phase visit overflow"))?;
    let mut parent_step = 0;
    let mut warmup_visits = 0;
    let mut warmup_full_context_visits = 0;
    let mut warmup = Value::Null;
    if !transition.is_null() {
        digest_field(transition, "parent_checkpoint_sha256", 64)?;
        digest_field(transition, "parent_campaign_sha256", 64)?;
        let old_batch = unsigned(transition, "old_batch")?;
        let old_context = unsigned(transition, "old_context")?;
        parent_step = unsigned(transition, "parent_optimizer_step")?;
        warmup_visits = unsigned(transition, "parent_sampled_target_visits")?;
        if !transition["reason"]
            .as_str()
            .is_some_and(|reason| !reason.trim().is_empty())
            || !(1..=64).contains(&old_batch)
            || !(8..=256).contains(&old_context)
            || transition["new_batch"] != batch
            || transition["new_context"] != context
            || (old_batch == batch && old_context == context)
            || old_batch.checked_mul(old_context) != Some(per_step)
            || parent_step == 0
            || parent_step > step
            || parent_step.checked_mul(per_step) != Some(warmup_visits)
            || warmup_visits > total
            || binding["training_batch"] != batch
            || binding["training_context"] != context
            || binding["sampled_targets_per_step"] != per_step
        {
            return Err(invalid("invalid checkpoint training-phase accounting"));
        }
        if old_context == CONTEXT as u64 {
            warmup_full_context_visits = warmup_visits;
        }
        warmup = json!({"batch":old_batch,"context":old_context,
            "optimizer_steps":parent_step,"target_visits":warmup_visits});
    }
    let current_steps = step
        .checked_sub(parent_step)
        .ok_or_else(|| invalid("phase step underflow"))?;
    let current_visits = total
        .checked_sub(warmup_visits)
        .ok_or_else(|| invalid("phase visit underflow"))?;
    if current_steps.checked_mul(per_step) != Some(current_visits) {
        return Err(invalid("current-phase visits differ from optimizer clock"));
    }
    let full_context_visits = warmup_full_context_visits
        + if context == CONTEXT as u64 {
            current_visits
        } else {
            0
        };
    Ok(json!({
        "selected_global_optimizer_step":step,"selected_total_target_visits":total,
        "warmup":warmup,
        "current_phase":{"batch":batch,"context":context,"starting_global_step":parent_step,
            "optimizer_steps":current_steps,"target_visits":current_visits},
        "full256_context_target_visits":full_context_visits,
        "training_window_transition":transition,
        "quantized_phase":quantization_exposure(report)?,
        "interpretation":"Counts separate exposure by the declared training windows. A256-token unroll makes read ages1..255 eligible for language gradients; exposure alone does not establish effective long-range recall. Weights, optimizer state and global step continue through the declared transition. Parent file hashes are retained from sealed evaluation metadata, not revalidated by loading external checkpoints here."
    }))
}

fn validate_pairs(reports: &[Value]) -> Result<()> {
    for first in [0, 2] {
        for field in [
            "campaign",
            "checkpoint_binding",
            "checkpoint",
            "source_commit",
            "executable_sha256",
            "requested_device",
            "mode",
            "artifact",
            "executed_quantization",
            "evaluation_quantization_strength",
        ] {
            if reports[first][field] != reports[first + 1][field] {
                return Err(invalid(format!("read/NoRead pair has different {field}")));
            }
        }
    }
    let q = &reports[0]["campaign"];
    let ordinary = &reports[2]["campaign"];
    let q_quantization = &q["quantization_transition"];
    let ordinary_quantization = &ordinary["quantization_transition"];
    if q_quantization.is_null() != ordinary_quantization.is_null() {
        return Err(invalid(
            "transport arms differ in quantization phase presence",
        ));
    }
    if !q_quantization.is_null() {
        for field in ["parent_optimizer_step", "ramp_steps"] {
            if q_quantization[field] != ordinary_quantization[field] {
                return Err(invalid(format!(
                    "transport arms differ in quantization {field}"
                )));
            }
        }
        let qs = &reports[0]["checkpoint_binding"]["quantization"]["spec"];
        let os = &reports[2]["checkpoint_binding"]["quantization"]["spec"];
        let qp = qs["parameters"]
            .as_object()
            .ok_or_else(|| invalid("quaternion quantizer parameters"))?;
        let op = os["parameters"]
            .as_object()
            .ok_or_else(|| invalid("ordinary quantizer parameters"))?;
        if qs["schema"] != os["schema"]
            || qp.len() != op.len()
            || qp.iter().any(|(name, format)| {
                op.get(name).is_none_or(|other| {
                    format["bits"] != other["bits"] || format["shape"] != other["shape"]
                })
            })
        {
            return Err(invalid(
                "transport arms use different quantizer algorithms or layouts",
            ));
        }
    }
    if gradient_shards(q)? != gradient_shards(ordinary)? {
        return Err(invalid("transport arms differ in CPU gradient shards"));
    }
    let q_transition = &q["training_window_transition"];
    let ordinary_transition = &ordinary["training_window_transition"];
    if q_transition.is_null() != ordinary_transition.is_null() {
        return Err(invalid(
            "transport arms differ in training transition presence",
        ));
    }
    if !q_transition.is_null() {
        for field in [
            "old_batch",
            "old_context",
            "new_batch",
            "new_context",
            "parent_optimizer_step",
            "parent_sampled_target_visits",
        ] {
            if q_transition[field] != ordinary_transition[field] {
                return Err(invalid(format!(
                    "transport arms differ in training transition {field}"
                )));
            }
        }
    }
    for field in [
        "optimizer",
        "data_seed",
        "batch",
        "context",
        "total_steps",
        "development_every_steps",
        "development_blocks",
        "checkpoint_steps",
    ] {
        if q[field].is_null() || q[field] != ordinary[field] {
            return Err(invalid(format!(
                "transport arms differ in shared campaign {field}"
            )));
        }
    }
    for field in ["vocab_size", "width", "read_width", "context", "seed"] {
        if q["model"][field] != ordinary["model"][field] {
            return Err(invalid(format!(
                "transport arms differ in shared model {field}"
            )));
        }
    }
    for field in [
        "source_commit",
        "executable_sha256",
        "requested_device",
        "mode",
    ] {
        if reports[0][field] != reports[2][field] {
            return Err(invalid(format!(
                "transport evaluation arms differ in {field}"
            )));
        }
    }
    if reports[0]["checkpoint_binding"]["source_commit"]
        != reports[2]["checkpoint_binding"]["source_commit"]
    {
        return Err(invalid(
            "transport checkpoints have different training source commits",
        ));
    }
    Ok(())
}

struct CsvRows {
    reader: BufReader<File>,
    line: Vec<u8>,
    fields: usize,
}
impl CsvRows {
    fn open(path: &Path, header: &str) -> Result<Self> {
        let mut rows = Self {
            reader: BufReader::new(File::open(path)?),
            line: Vec::with_capacity(256),
            fields: header.split(',').count(),
        };
        rows.read_line()?;
        if std::str::from_utf8(&rows.line)
            .map_err(|_| invalid("non-UTF8 CSV header"))?
            .trim_end_matches(['\r', '\n'])
            != header
        {
            return Err(invalid(format!(
                "unexpected comparison CSV header: {}",
                path.display()
            )));
        }
        Ok(rows)
    }
    fn read_line(&mut self) -> Result<()> {
        self.line.clear();
        let length = self
            .reader
            .by_ref()
            .take(1025)
            .read_until(b'\n', &mut self.line)?;
        if length == 0 || length > 1024 {
            return Err(invalid("missing or oversized comparison CSV row"));
        }
        Ok(())
    }
    fn fields(&mut self) -> Result<Vec<&str>> {
        self.read_line()?;
        let row =
            std::str::from_utf8(&self.line).map_err(|_| invalid("non-UTF8 comparison CSV row"))?;
        let fields: Vec<_> = row.trim_end_matches(['\r', '\n']).split(',').collect();
        if fields.len() != self.fields {
            return Err(invalid("comparison CSV column count differs"));
        }
        Ok(fields)
    }
    fn require_eof(&mut self) -> Result<()> {
        let mut extra = [0u8; 1];
        if self.reader.read(&mut extra)? != 0 {
            return Err(invalid("CSV has rows after the complete population"));
        }
        Ok(())
    }
}

fn number<T: FromStr>(text: &str, field: &str, offset: usize) -> Result<T> {
    text.parse()
        .map_err(|_| invalid(format!("invalid {field} at input offset {offset}")))
}
fn finite(text: &str, field: &str, offset: usize) -> Result<f64> {
    let value: f64 = number(text, field, offset)?;
    if !value.is_finite() {
        return Err(invalid(format!(
            "nonfinite {field} at input offset {offset}"
        )));
    }
    Ok(value)
}
fn token(text: &str, field: &str, offset: usize) -> Result<u32> {
    let value = number(text, field, offset)?;
    if value >= 4096 {
        return Err(invalid(format!(
            "out-of-vocabulary {field} at input offset {offset}"
        )));
    }
    Ok(value)
}
fn equal_index(text: &str, expected: usize, field: &str, offset: usize) -> Result<()> {
    if number::<usize>(text, field, offset)? != expected {
        return Err(invalid(format!(
            "missing, duplicate or reordered {field} at input offset {offset}"
        )));
    }
    Ok(())
}
struct ReferenceRow {
    input: u32,
    target: u32,
    nll: f64,
}
fn reference_row(rows: &mut CsvRows, offset: usize) -> Result<ReferenceRow> {
    let f = rows.fields()?;
    for (field, expected, name) in [
        (0, offset / CONTEXT, "reference block"),
        (1, offset % CONTEXT, "reference position"),
        (2, offset, "reference input offset"),
        (3, offset + 1, "reference target offset"),
    ] {
        equal_index(f[field], expected, name, offset)?;
    }
    let input = token(f[4], "reference input", offset)?;
    let target = token(f[5], "reference target", offset)?;
    let predicted = token(f[6], "reference prediction", offset)?;
    // The writer uses F32 Display for the logit; parse it back to F32 before
    // comparing with the F64 loss, rather than treating its decimal as exact.
    let logit: f32 = number(f[7], "reference target logit", offset)?;
    let normalizer = finite(f[8], "reference log normalizer", offset)?;
    let nll = finite(f[9], "reference NLL", offset)?;
    if !logit.is_finite()
        || nll < -1e-12
        || (nll - (normalizer - f64::from(logit))).abs() > 1e-11 * (1.0 + nll.abs())
        || number::<u8>(f[10], "reference correctness", offset)? != u8::from(predicted == target)
    {
        return Err(invalid(format!(
            "reference score arithmetic differs at input offset {offset}"
        )));
    }
    Ok(ReferenceRow { input, target, nll })
}
fn count_row(rows: &mut CsvRows, offset: usize, reference: &ReferenceRow) -> Result<[f64; 2]> {
    let f = rows.fields()?;
    equal_index(f[0], offset / CONTEXT, "count block", offset)?;
    equal_index(f[1], offset % CONTEXT, "count position", offset)?;
    equal_index(f[2], offset + 1, "count target offset", offset)?;
    if token(f[3], "count input", offset)? != reference.input
        || token(f[4], "count target", offset)? != reference.target
    {
        return Err(invalid(format!(
            "count/reference token identity differs at input offset {offset}"
        )));
    }
    let mut losses = [0.0; 2];
    for index in 0..2 {
        let probability = finite(f[5 + index], "count probability", offset)?;
        let nll = finite(f[7 + index], "count NLL", offset)?;
        let reconstructed = (-nll).exp();
        // Probability columns were printed with 17 fractional decimal places.
        // Very small valid values can round to zero; the NLL remains authoritative.
        if !(0.0..=1.0 + 1e-12).contains(&probability)
            || nll < -1e-12
            || (probability - reconstructed).abs() > 1e-17 + 1e-15 * reconstructed
        {
            return Err(invalid(format!(
                "count score arithmetic differs at input offset {offset}"
            )));
        }
        losses[index] = nll;
    }
    Ok(losses)
}
fn bytes<const N: usize>(reader: &mut impl Read) -> Result<[u8; N]> {
    let mut value = [0u8; N];
    reader.read_exact(&mut value)?;
    Ok(value)
}
fn joint_row(
    reader: &mut BufReader<File>,
    offset: usize,
    target: u32,
    mode: ReadMode,
    name: &str,
) -> Result<f64> {
    let mut raw = [0u8; RECORD_BYTES];
    reader.read_exact(&mut raw).map_err(|error| {
        invalid(format!(
            "{name} missing or partial record at offset {offset}: {error}"
        ))
    })?;
    let mut record = Cursor::new(raw);
    let actual_offset = u64::from_le_bytes(bytes(&mut record)?);
    let actual_target = u32::from_le_bytes(bytes(&mut record)?);
    let predicted = u32::from_le_bytes(bytes(&mut record)?);
    let probability = f64::from_le_bytes(bytes(&mut record)?);
    let nll = f64::from_le_bytes(bytes(&mut record)?);
    let no_read = f64::from(f32::from_le_bytes(bytes(&mut record)?));
    let copy = f64::from(f32::from_le_bytes(bytes(&mut record)?));
    let top = i32::from_le_bytes(bytes(&mut record)?);
    let valid_mass =
        |value: f64| value.is_finite() && (0.0..=1.0 + PROBABILITY_SUM_TOLERANCE).contains(&value);
    if actual_offset != offset as u64
        || actual_target != target
        || predicted >= 4096
        || !probability.is_finite()
        || probability <= 0.0
        || probability > 1.0 + PROBABILITY_SUM_TOLERANCE
        || !nll.is_finite()
        || (nll + probability.ln()).abs() > 1e-12 * (1.0 + nll.abs())
        || !valid_mass(no_read)
        || !valid_mass(copy)
        || top < -1
        || (top >= 0 && top as usize >= offset % CONTEXT)
        || (mode == ReadMode::NoRead && (no_read != 1.0 || top != -1))
    {
        return Err(invalid(format!("{name} identity, finite score or causal read metadata differs at input offset {offset}")));
    }
    Ok(nll)
}

#[derive(Clone, Copy, Default)]
struct Sum {
    total: f64,
    correction: f64,
}
impl Sum {
    fn add(&mut self, value: f64) {
        let corrected = value - self.correction;
        let next = self.total + corrected;
        self.correction = (next - self.total) - corrected;
        self.total = next;
    }
}
#[derive(Clone, Copy, Default)]
struct Paired {
    delta: Sum,
    improved: u64,
    equal: u64,
    worse: u64,
}
#[derive(Clone, Copy, Default)]
struct Accumulator {
    targets: u64,
    sums: [Sum; 7],
    pairs: [Paired; 10],
}
impl Accumulator {
    fn add(&mut self, losses: [f64; 7]) {
        self.targets += 1;
        for (sum, value) in self.sums.iter_mut().zip(losses) {
            sum.add(value);
        }
        for (pair, &(candidate, comparator)) in self.pairs.iter_mut().zip(&PAIRS) {
            pair.delta.add(losses[candidate] - losses[comparator]);
            if losses[candidate] < losses[comparator] {
                pair.improved += 1;
            } else if losses[candidate] > losses[comparator] {
                pair.worse += 1;
            } else {
                pair.equal += 1;
            }
        }
    }
    fn mean(&self, arm: usize) -> f64 {
        self.sums[arm].total / self.targets as f64
    }
    fn paired_delta(&self, candidate: usize, comparator: usize) -> Result<f64> {
        let pair = self
            .pairs
            .iter()
            .zip(&PAIRS)
            .find(|(_, indices)| **indices == (candidate, comparator))
            .ok_or_else(|| invalid("undeclared paired comparison"))?;
        Ok(pair.0.delta.total / self.targets as f64)
    }
    fn means_report(&self) -> Value {
        let arms: serde_json::Map<String, Value> = ARM_NAMES
            .iter()
            .enumerate()
            .map(|(index, name)| {
                (
                    (*name).to_owned(),
                    json!({"nll_sum_nats":self.sums[index].total,"mean_nll_nats":self.mean(index)}),
                )
            })
            .collect();
        json!({"scored_targets":self.targets,"arms":arms})
    }
    fn report(&self) -> Value {
        let comparisons: Vec<_> = self.pairs.iter().zip(&PAIRS).map(|(pair, &(candidate, comparator))| {
            json!({"candidate":ARM_NAMES[candidate],"comparator":ARM_NAMES[comparator],
                "improved_targets":pair.improved,"equal_targets":pair.equal,"worse_targets":pair.worse,
                "paired_nll_delta_sum_nats":pair.delta.total,"paired_mean_nll_delta_nats":pair.delta.total/self.targets as f64})
        }).collect();
        let mut report = self.means_report();
        report["comparisons"] = json!(comparisons);
        report
    }
}
fn unsigned(value: &Value, field: &str) -> Result<u64> {
    value[field]
        .as_u64()
        .ok_or_else(|| invalid(format!("missing integer {field}")))
}
fn check_reported_means(
    partitions: &[Accumulator; 3],
    reference: &Value,
    count: &Value,
    native: &[Value],
) -> Result<Value> {
    let mut maximum = 0.0f64;
    for (index, name) in ["tune", "comparison", "full"].into_iter().enumerate() {
        let reference_part = if name == "full" {
            &reference["full"]
        } else {
            &reference["partitions"][name]
        };
        let reported = [
            &reference_part["mean_nll_nats"],
            &count[name]["ngram_nll_nats"],
            &count[name]["cache_nll_nats"],
            &native[0]["evaluation"][name]["mean_nll_nats"],
            &native[1]["evaluation"][name]["mean_nll_nats"],
            &native[2]["evaluation"][name]["mean_nll_nats"],
            &native[3]["evaluation"][name]["mean_nll_nats"],
        ];
        if unsigned(reference_part, "scored_targets")? != partitions[index].targets
            || unsigned(&count[name], "scored_targets")? != partitions[index].targets
            || native.iter().any(|report| {
                report["evaluation"][name]["scored_targets"].as_u64()
                    != Some(partitions[index].targets)
            })
        {
            return Err(invalid(format!(
                "{name} reported target count differs from joined rows"
            )));
        }
        for (arm, value) in reported.into_iter().enumerate() {
            let value = value
                .as_f64()
                .ok_or_else(|| invalid(format!("missing {name}/{} mean", ARM_NAMES[arm])))?;
            let actual = partitions[index].mean(arm);
            let delta = (actual - value).abs();
            if !value.is_finite()
                || !actual.is_finite()
                || !delta.is_finite()
                || delta > SUMMARY_TOLERANCE
            {
                return Err(invalid(format!(
                    "{name}/{} mean differs from joined rows: reported {value}, joined {actual}",
                    ARM_NAMES[arm]
                )));
            }
            maximum = maximum.max(delta);
        }
    }
    Ok(
        json!({"all_21_means_match":true,"maximum_absolute_delta_nats":maximum,
        "absolute_tolerance_nats":SUMMARY_TOLERANCE,"scope":"Serialization/reduction consistency check; not a model-quality or statistical tolerance."}),
    )
}
