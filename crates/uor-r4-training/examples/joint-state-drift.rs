//! Descriptive hard-versus-shadow state drift on the existing 64 tune windows.
//! This compares all parameter/interface quantization of the same QAT weights;
//! it is neither an isolated transport-error measurement nor a stability proof.
#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_training::baseline_protocol::{
    base_report, load_evaluator, read_tokens, save_json, verify_identity, Evaluator,
};
use uor_r4_training::joint_campaign::Campaign;
use uor_r4_training::joint_model::{JointModel, ReadMode, CHECKPOINT_SCHEMA};
use uor_r4_training::{sha256_file, Result, TrainingError};

const BLOCKS: usize = 64;
const CONTEXT: usize = 256;
const BATCH: usize = 16;
const QUARTER: usize = 64;

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn identity(path: &Path) -> Result<Value> {
    Ok(json!({"path":path,"bytes":fs::metadata(path)?.len(),"sha256":sha256_file(path)?}))
}

fn identities(directory: &Path, names: &[&str]) -> Result<Vec<Value>> {
    names
        .iter()
        .map(|name| identity(&directory.join(name)))
        .collect()
}

#[derive(Clone, Default)]
struct Moments {
    coordinates: u64,
    finite_pairs: u64,
    hard_nonfinite: u64,
    shadow_nonfinite: u64,
    difference_squares: f64,
    hard_squares: f64,
    shadow_squares: f64,
    maximum_absolute_difference: f64,
}

impl Moments {
    fn observe(&mut self, hard: f32, shadow: f32) {
        self.coordinates += 1;
        if hard.is_finite() {
            self.hard_squares += f64::from(hard).powi(2);
        } else {
            self.hard_nonfinite += 1;
        }
        if shadow.is_finite() {
            self.shadow_squares += f64::from(shadow).powi(2);
        } else {
            self.shadow_nonfinite += 1;
        }
        if hard.is_finite() && shadow.is_finite() {
            let difference = f64::from(hard) - f64::from(shadow);
            self.finite_pairs += 1;
            self.difference_squares += difference * difference;
            self.maximum_absolute_difference =
                self.maximum_absolute_difference.max(difference.abs());
        }
    }

    fn report(&self, width: usize) -> Value {
        let rms = |sum: f64, count: u64| (count != 0).then(|| (sum / count as f64).sqrt());
        let hard_rms = rms(self.hard_squares, self.coordinates - self.hard_nonfinite);
        let shadow_rms = rms(
            self.shadow_squares,
            self.coordinates - self.shadow_nonfinite,
        );
        json!({
            "state_vectors":self.coordinates / width as u64,
            "coordinate_count":self.coordinates,"finite_coordinate_pairs":self.finite_pairs,
            "hard_nonfinite_coordinates":self.hard_nonfinite,
            "shadow_nonfinite_coordinates":self.shadow_nonfinite,
            "coordinate_difference_squared_sum":self.difference_squares,
            "rms_coordinate_difference":rms(self.difference_squares,self.finite_pairs),
            "maximum_absolute_coordinate_difference":(self.finite_pairs != 0).then_some(self.maximum_absolute_difference),
            "hard_coordinate_squared_sum":self.hard_squares,
            "shadow_coordinate_squared_sum":self.shadow_squares,
            "hard_coordinate_rms":hard_rms,"shadow_coordinate_rms":shadow_rms,
            "hard_state_rms_l2_norm":if self.hard_nonfinite == 0 {hard_rms.map(|v|v*(width as f64).sqrt())} else {None},
            "shadow_state_rms_l2_norm":if self.shadow_nonfinite == 0 {shadow_rms.map(|v|v*(width as f64).sqrt())} else {None}
        })
    }
}

fn load_pair(
    checkpoint: &Path,
    packed: &Path,
    evaluator: &Evaluator,
) -> Result<(JointModel, JointModel, Value)> {
    report_output::verify(checkpoint)?;
    report_output::verify(packed)?;
    let campaign = Campaign::load(&checkpoint.join("campaign.json"))?;
    let checkpoint_binding = read_json(&checkpoint.join("checkpoint.json"))?;
    let packed_report = read_json(&packed.join("hard-export-report.json"))?;
    let campaign_value = serde_json::to_value(&campaign)?;
    let checkpoint_sha = sha256_file(&checkpoint.join("checkpoint.json"))?;
    let campaign_sha = sha256_file(&checkpoint.join("campaign.json"))?;
    let hard_manifest_sha = sha256_file(&packed.join("hard-model.json"))?;
    let step = checkpoint_binding["optimizer_step"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| invalid("selected checkpoint optimizer step"))?;
    let visits = step
        .checked_mul(campaign.batch * campaign.context)
        .ok_or_else(|| invalid("selected checkpoint exposure overflow"))?;
    if checkpoint_binding["schema"] != CHECKPOINT_SCHEMA
        || checkpoint_binding["evaluator_sha256"] != evaluator.sha256
        || checkpoint_binding["model_sha256"] != sha256_file(&checkpoint.join("model.safetensors"))?
        || checkpoint_binding["model_config_sha256"]
            != sha256_file(&checkpoint.join("config.json"))?
        || checkpoint_binding["next_data_step"] != json!(step)
        || checkpoint_binding["sampled_target_visits"] != json!(visits)
        || step > campaign.total_steps
        || packed_report["status"] != "PACKED_EXPORT_RELOADED"
        || packed_report["mode"] != "joint-export-hard"
        || packed_report["evaluator_sha256"] != evaluator.sha256
        || packed_report["campaign"] != campaign_value
        || read_json(&packed.join("campaign.json"))? != campaign_value
        || packed_report["artifact"]["parent_checkpoint_sha256"] != checkpoint_sha
        || packed_report["artifact"]["parent_campaign_sha256"] != campaign_sha
        || packed_report["artifact"]["parent_checkpoint_binding"] != checkpoint_binding
        || packed_report["artifact"]["hard_model_manifest_sha256"] != hard_manifest_sha
    {
        return Err(invalid(
            "checkpoint, packed export, campaign or evaluator lineage mismatch",
        ));
    }
    let qat = JointModel::load(checkpoint, &Device::Cpu)?;
    let hard = JointModel::load_hard(packed, &Device::Cpu)?;
    let state = qat
        .quantization()
        .ok_or_else(|| invalid("QAT checkpoint required"))?;
    let transition = campaign
        .quantization_transition
        .as_ref()
        .ok_or_else(|| invalid("QAT campaign transition missing"))?;
    if qat.config != campaign.model
        || hard.config != qat.config
        || qat.config.context != CONTEXT
        || hard.quantization() != Some(state)
        || state.completed_step != step
        || state.start_step != transition.parent_optimizer_step
        || state.ramp_steps != transition.ramp_steps
        || checkpoint_binding["quantization"] != serde_json::to_value(state)?
        || checkpoint_binding["quantization_transition"] != json!(campaign.quantization_transition)
    {
        return Err(invalid(
            "loaded model configuration, quantization spec or selected clock mismatch",
        ));
    }
    let mut expected_binding = checkpoint_binding.clone();
    expected_binding["model_sha256"] = json!(hard_manifest_sha);
    expected_binding["artifact_kind"] =
        json!("packed_quantized_parameters_with_float_numerical_emulator");
    expected_binding
        .as_object_mut()
        .ok_or_else(|| invalid("checkpoint binding object"))?
        .remove("model_config_sha256");
    if packed_report["checkpoint_binding"] != expected_binding {
        return Err(invalid("packed selected-checkpoint binding mismatch"));
    }

    // A matching spec/shape is insufficient: compare every actual decoded value
    // with quantization of this exact checkpoint's learned shadow parameters.
    let mut compared_parameters = 0usize;
    for (name, variable) in qat.variables() {
        let expected = state
            .spec
            .parameter(name, variable.as_tensor(), 1.0, false)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let actual = hard
            .variables()
            .get(name)
            .ok_or_else(|| invalid("packed parameter missing"))?
            .detach()
            .flatten_all()?
            .to_vec1::<f32>()?;
        if expected.len() != actual.len()
            || expected
                .iter()
                .zip(&actual)
                .any(|(a, b)| a.to_bits() != b.to_bits())
        {
            return Err(invalid(format!(
                "packed parameters differ from selected QAT weights: {name}"
            )));
        }
        compared_parameters = compared_parameters
            .checked_add(expected.len())
            .ok_or_else(|| invalid("parameter count overflow"))?;
    }
    let binding = json!({
        "selected_optimizer_step":step,"selected_sampled_target_visits":visits,
        "checkpoint_binding":checkpoint_binding,"packed_checkpoint_binding":expected_binding,
        "quantization":state,
        "quantization_spec_sha256_canonical_json":hex::encode(Sha256::digest(serde_json::to_vec(&state.spec)?)),
        "packed_parameters_bitwise_equal_to_quantized_selected_shadows":true,
        "parameter_coordinates_compared":compared_parameters,
        "checkpoint_files":identities(checkpoint,&["manifest.json","checkpoint.json","campaign.json","config.json","model.safetensors"] )?,
        "packed_files":identities(packed,&["manifest.json","campaign.json","hard-export-report.json","hard-model.json","hard-parameters.json","hard-parameters.bin"] )?,
        "training_source_commit":checkpoint_binding["source_commit"],
        "export_source_commit":packed_report["source_commit"],
        "export_executable_sha256":packed_report["executable_sha256"]
    });
    Ok((hard, qat.without_quantization()?, binding))
}

fn states(model: &JointModel, inputs: &[u32], batch: usize) -> Result<Vec<f32>> {
    let output = model.forward(inputs, batch, CONTEXT, ReadMode::Enabled, false)?;
    if output.states.dims() != [batch, CONTEXT, model.config.width] {
        return Err(invalid("unexpected returned state shape"));
    }
    // Drop probabilities and all other forward outputs before the second model.
    Ok(output.states.detach().flatten_all()?.to_vec1::<f32>()?)
}

fn run(checkpoint: &Path, packed: &Path, evaluator_path: &Path, out: &Path) -> Result<()> {
    let started = Instant::now();
    let evaluator = load_evaluator(evaluator_path)?;
    let mut report = base_report(&evaluator, "joint-state-drift")?;
    report["schema"] = json!("uor-r4.joint-state-drift/1");
    report["scope"] = json!("Descriptive difference between loaded packed quantized computation and the same QAT weights with all quantizers disabled, on original tune64. Includes parameter and interface effects accumulated through recurrence/read/write. Not isolated transport error, a stability proof, an acceptance threshold, or fresh final qualification.");
    report["compiled_source_sha256"] = json!({
        "example":hex::encode(Sha256::digest(include_bytes!("joint-state-drift.rs"))),
        "joint_model":hex::encode(Sha256::digest(include_bytes!("../src/joint_model.rs"))),
        "joint_quantization":hex::encode(Sha256::digest(include_bytes!("../src/joint_quantization.rs")))
    });
    report["device"] = json!("cpu");
    report["cpu_accelerate_compiled"] = json!(cfg!(feature = "cpu-accelerate"));
    report["thread_environment"] = json!({
        "RAYON_NUM_THREADS":std::env::var("RAYON_NUM_THREADS").ok(),
        "VECLIB_MAXIMUM_THREADS":std::env::var("VECLIB_MAXIMUM_THREADS").ok()
    });
    report["gradients_tracked"] = json!(false);
    report["neural_optimizer_steps"] = json!(0);
    save_json(&out.join("run-provenance.json"), &report)?;
    save_json(&out.join("evaluator.json"), &evaluator.document)?;
    let tokens = read_tokens(&evaluator.document["dev_source"])?;
    if tokens.len() < BLOCKS * CONTEXT + 1 {
        return Err(invalid("original tune population is too short"));
    }
    let tokenizer_identities: Vec<_> = evaluator.document["reference_inputs"]
        .as_array()
        .ok_or_else(|| invalid("reference input inventory"))?
        .iter()
        .filter(|item| {
            item["path"]
                .as_str()
                .is_some_and(|p| p.ends_with("/tokenizer.json"))
        })
        .collect();
    if tokenizer_identities.len() != 1 {
        return Err(invalid("exactly one tokenizer identity required"));
    }
    let tokenizer_path = verify_identity(tokenizer_identities[0])?;
    let (hard, shadow, binding) = load_pair(checkpoint, packed, &evaluator)?;
    report["binding"] = binding;
    report["tokenizer"] = identity(&tokenizer_path)?;
    report["dev_source"] = evaluator.document["dev_source"].clone();
    let mut input_digest = Sha256::new();
    for token in &tokens[..BLOCKS * CONTEXT] {
        input_digest.update(token.to_le_bytes());
    }
    report["inputs"] = json!({"blocks":BLOCKS,"batch":BATCH,"context":CONTEXT,
        "input_tokens":BLOCKS*CONTEXT,"first_token_offset":0,"end_token_offset_exclusive":BLOCKS*CONTEXT,
        "model_forward_calls":2*(BLOCKS/BATCH),"model_forward_input_visits":2*BLOCKS*CONTEXT,
        "sha256_le_u16":hex::encode(input_digest.finalize()),"read_mode":"enabled",
        "reset":"Both models reset state/history per complete256-input block. Identical original prefixes; no injected BOS or EOS reset. States are after consuming each observed input; targets are unused."});
    save_json(&out.join("input-bindings.json"), &report)?;

    let width = hard.config.width;
    let mut positions = vec![Moments::default(); CONTEXT];
    let mut quarters = vec![Moments::default(); CONTEXT / QUARTER];
    let mut total = Moments::default();
    for first in (0..BLOCKS).step_by(BATCH) {
        let batch = (BLOCKS - first).min(BATCH);
        let inputs: Vec<u32> = tokens[first * CONTEXT..(first + batch) * CONTEXT]
            .iter()
            .map(|&token| u32::from(token))
            .collect();
        let hard_states = states(&hard, &inputs, batch)?;
        let shadow_states = states(&shadow, &inputs, batch)?;
        for lane in 0..batch {
            for position in 0..CONTEXT {
                for coordinate in 0..width {
                    let index = (lane * CONTEXT + position) * width + coordinate;
                    let (h, s) = (hard_states[index], shadow_states[index]);
                    positions[position].observe(h, s);
                    quarters[position / QUARTER].observe(h, s);
                    total.observe(h, s);
                }
            }
        }
    }
    let finite = total.hard_nonfinite == 0 && total.shadow_nonfinite == 0;
    report["status"] = json!(if finite {
        "DESCRIPTIVE_DIAGNOSTIC_COMPLETE"
    } else {
        "NONFINITE_STATES_OBSERVED"
    });
    report["state_width"] = json!(width);
    report["all_state_coordinates_finite"] = json!(finite);
    report["aggregation"] = json!("RMS coordinate difference=sqrt(sum((hard-shadow)^2) / finite paired coordinates); coordinate RMS=sqrt(sum(state^2) / finite coordinates); RMS L2 state norm=sqrt(mean sum_d state_d^2), emitted only if all corresponding coordinates are finite. Quarters pool coordinates across64positions and64blocks; they do not average position RMS values. Position indices are zero-based, after input processing.");
    report["total"] = total.report(width);
    report["by_position"] = json!(positions.iter().enumerate().map(|(position,moments)|
        json!({"position":position,"prefix_input_tokens":position+1,"metrics":moments.report(width)})).collect::<Vec<_>>());
    report["by_quarter"] = json!(quarters.iter().enumerate().map(|(quarter,moments)|
        json!({"quarter":quarter+1,"position_start":quarter*QUARTER,"position_end_exclusive":(quarter+1)*QUARTER,"metrics":moments.report(width)})).collect::<Vec<_>>());
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    // Save observations before returning a numerical-integrity error; the caller
    // seals failed attempts too. No drift magnitude is an acceptance criterion.
    save_json(&out.join("state-drift.json"), &report)?;
    if !finite {
        return Err(invalid(
            "nonfinite state coordinates; descriptive observations saved",
        ));
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args.len() != 4 {
        return Err(invalid("usage: joint-state-drift SEALED_QAT_CHECKPOINT SEALED_PACKED_EXPORT EVALUATOR_JSON NEW_REPORT_ROOT"));
    }
    let checkpoint = args[0].canonicalize()?;
    let packed = args[1].canonicalize()?;
    let out = &args[3];
    // Requiring an existing output container prevents adding a report beneath
    // either immutable input through a relative path or a symlinked parent.
    let parent = out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()?;
    if parent.starts_with(&checkpoint) || parent.starts_with(&packed) || out.file_name().is_none() {
        return Err(invalid(
            "new report must be outside both sealed input roots",
        ));
    }
    report_output::claim(out)?;
    let result = run(&checkpoint, &packed, &args[2], out);
    if let Err(error) = &result {
        save_json(
            &out.join("failed-attempt.json"),
            &json!({"status":"FAILED_ATTEMPT","error":error.to_string()}),
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result?;
    println!("COMPLETE: {}; sealed and verified", out.display());
    Ok(())
}
