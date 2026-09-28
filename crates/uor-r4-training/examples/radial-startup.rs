//! One no-update startup observation on four fixed canonical development windows.
//! Uses the same campaign initialization/transfer path as actual fitting. No
//! optimizer, calibration search, checkpoint selection or language verdict.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::baseline_protocol::{
    base_report, load_evaluator, read_tokens, save_json, verify_identity,
};
use uor_r4_training::joint_campaign::{Campaign, ReadInitialization};
use uor_r4_training::joint_evaluation;
use uor_r4_training::joint_model::{
    JointConfig, JointModel, ReadGeometry, ReadMode, Transport, LORENTZ_LOG_BETA, LORENTZ_OFFSET,
};
use uor_r4_training::joint_optimizer::AdamConfig;
use uor_r4_training::{sha256_file, Result, TrainingError};

const CONTEXT: usize = 256;
const BLOCKS: [usize; 4] = [0, 21, 42, 63];
const SEED: u64 = 240924;
const EVALUATOR_SHA256: &str = "d2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62";
const GRADIENT_NAMES: [&str; 6] = [
    "read.query.weight",
    "read.key.weight",
    "read.value.weight",
    "read.no_read.weight",
    LORENTZ_LOG_BETA,
    LORENTZ_OFFSET,
];

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

fn campaign(evaluator: &Path, geometry: ReadGeometry) -> Campaign {
    Campaign {
        schema: "uor-r4.joint-recurrent-campaign/1".into(),
        evaluator_path: evaluator.to_path_buf(),
        model: JointConfig {
            vocab_size: 4096,
            width: 256,
            read_width: 64,
            context: CONTEXT,
            transport: Transport::Quaternion,
            seed: SEED,
            read_geometry: geometry,
        },
        optimizer: AdamConfig::default(),
        data_seed: SEED,
        batch: BLOCKS.len(),
        context: CONTEXT,
        cpu_gradient_shards: 1,
        // A valid campaign configuration supplies initialization only. No fit
        // function or optimizer is constructed, and this step is never run.
        total_steps: 1,
        development_every_steps: 0,
        development_blocks: 0,
        checkpoint_steps: Vec::new(),
        max_process_seconds: 1,
        stop_file: None,
        training_window_transition: None,
        quantization_transition: None,
        projection_transition: None,
        read_initialization: Some(ReadInitialization::UnitScale),
        shared_parameter_transfer: None,
        end_weight: None,
        geometric_read: None,
        trial_scope: "No-update startup witness only; total_steps is unused; four fixed development windows, no training or model-quality decision".into(),
    }
}

fn parameter_hashes(model: &JointModel) -> Result<BTreeMap<String, String>> {
    model
        .variables()
        .iter()
        .map(|(name, variable)| {
            let mut digest = Sha256::new();
            for value in variable.flatten_all()?.to_vec1::<f32>()? {
                digest.update(value.to_le_bytes());
            }
            Ok((name.clone(), hex::encode(digest.finalize())))
        })
        .collect()
}

fn scalar(model: &JointModel, name: &str) -> Result<f32> {
    model
        .variables()
        .get(name)
        .ok_or_else(|| invalid(format!("missing scalar {name}")))?
        .flatten_all()?
        .to_vec1::<f32>()?
        .first()
        .copied()
        .ok_or_else(|| invalid(format!("empty scalar {name}")))
}

struct Observation {
    report: Value,
    first_states: Vec<Vec<f32>>,
    first_read: Vec<(f64, f64)>,
    integrity: bool,
}

fn observe(model: &JointModel, inputs: &[u32], targets: &[u32]) -> Result<Observation> {
    let started = Instant::now();
    let before = parameter_hashes(model)?;
    let output = model.forward(inputs, BLOCKS.len(), CONTEXT, ReadMode::Enabled, true)?;
    let loss = output.loss(targets)?;
    let nll = f64::from(loss.to_scalar::<f32>()?);
    let gradients = loss.backward()?;
    let mut gradient_rows = BTreeMap::new();
    let mut integrity = nll.is_finite();
    let mut all_required_gradients_nonzero = true;
    for name in GRADIENT_NAMES {
        let variable = model
            .variables()
            .get(name)
            .ok_or_else(|| invalid(format!("missing parameter {name}")))?;
        let Some(gradient) = gradients.get(variable.as_tensor()) else {
            gradient_rows.insert(name, json!({"status":"MISSING"}));
            integrity = false;
            all_required_gradients_nonzero = false;
            continue;
        };
        let values = gradient.flatten_all()?.to_vec1::<f32>()?;
        let finite = values.iter().all(|x| x.is_finite());
        let nonzero = values
            .iter()
            .filter(|x| x.is_finite() && **x != 0.0)
            .count();
        let norm = finite.then(|| {
            values
                .iter()
                .map(|&x| f64::from(x).powi(2))
                .sum::<f64>()
                .sqrt()
        });
        let max_abs = finite.then(|| values.iter().map(|x| x.abs()).fold(0f32, f32::max));
        gradient_rows.insert(
            name,
            json!({"coordinates":values.len(),"finite":finite,
            "nonzero_coordinates":nonzero,"l2_norm":norm,"maximum_absolute":max_abs}),
        );
        integrity &= finite;
        all_required_gradients_nonzero &= nonzero > 0;
    }
    drop(gradients);
    let no_reads = output.no_read_mass.to_vec2::<f32>()?;
    let reads = output.read_masses.to_vec3::<f32>()?;
    let gates = output.copy_gate.to_vec2::<f32>()?;
    let first_states = output
        .states
        .narrow(1, 0, 1)?
        .squeeze(1)?
        .to_vec2::<f32>()?;
    let mut positions = Vec::with_capacity(BLOCKS.len() * CONTEXT);
    let mut first_read = Vec::with_capacity(BLOCKS.len());
    let mut mass_sum = 0.0;
    let mut null_sum = 0.0;
    let mut entropy_sum = 0.0;
    let mut entropy_count = 0usize;
    let mut zero_read_positions = 0usize;
    let mut max_normalization_error = 0f64;
    for lane in 0..BLOCKS.len() {
        for position in 0..CONTEXT {
            let row = &reads[lane][position][..position];
            let read_mass: f64 = row.iter().map(|&x| f64::from(x)).sum();
            let no_read = f64::from(no_reads[lane][position]);
            let finite = read_mass.is_finite()
                && no_read.is_finite()
                && row.iter().all(|&x| x.is_finite() && x >= 0.0)
                && no_read >= 0.0
                && gates[lane][position].is_finite();
            integrity &= finite;
            let entropy = (finite && read_mass > 0.0).then(|| {
                row.iter()
                    .filter(|&&x| x > 0.0)
                    .map(|&x| {
                        let probability = f64::from(x) / read_mass;
                        -probability * probability.ln()
                    })
                    .sum::<f64>()
            });
            if position > 0 {
                mass_sum += read_mass;
                null_sum += no_read;
                zero_read_positions += usize::from(read_mass == 0.0);
                if let Some(entropy) = entropy {
                    entropy_sum += entropy;
                    entropy_count += 1;
                }
                if finite {
                    max_normalization_error =
                        max_normalization_error.max((read_mass + no_read - 1.0).abs());
                }
            }
            if position == 1 {
                first_read.push((read_mass, no_read));
            }
            // Keep the small complete occurrence row at inspectable prefix
            // lengths; all positions retain aggregate observations.
            let occurrences = [1, 63, 127, 255].contains(&position).then(|| row.to_vec());
            positions.push(json!({"window":lane,"block":BLOCKS[lane],"position":position,
                "input_token":inputs[lane*CONTEXT+position],"target_token":targets[lane*CONTEXT+position],
                "causal_keys":position,"read_mass":read_mass,"no_read_mass":no_read,
                "conditional_read_entropy_nats":entropy,"copy_gate":gates[lane][position],
                "read_occurrence_masses":occurrences}));
        }
    }
    let after = parameter_hashes(model)?;
    let unchanged = before == after;
    integrity &= unchanged;
    let count = (BLOCKS.len() * (CONTEXT - 1)) as f64;
    Ok(Observation {
        report: json!({"geometry":model.config.read_geometry,"model":model.config,
            "constructor_numerical_contract":model.numerical_contract(),
            "actual_initialization":{"campaign_override":"unit_scale",
                "log_beta":scalar(model,LORENTZ_LOG_BETA)?,
                "beta":f64::from(scalar(model,LORENTZ_LOG_BETA)?).exp(),
                "offset":scalar(model,LORENTZ_OFFSET)?,
                "note":"The pinned numerical contract documents constructor defaults. Campaign::fresh_model applies the explicit override before this observation."},
            "parameter_sha256_le_f32_before":before,"parameter_sha256_le_f32_after":after,
            "parameters_unchanged":unchanged,"observed_initial_next_token_nll":nll,
            "gradient_loss":"population mean next-token NLL over four full256 windows, read enabled",
            "gradients":gradient_rows,
            "all_required_gradient_arrays_have_nonzero_coordinates":all_required_gradients_nonzero,
            "summary_excludes_empty_history_position0":{
                "positions":count as usize,"mean_read_mass":mass_sum/count,"mean_no_read_mass":null_sum/count,
                "mean_conditional_read_entropy_nats":(entropy_count>0).then(||entropy_sum/entropy_count as f64),
                "defined_entropy_positions":entropy_count,"zero_read_positions":zero_read_positions,
                "all_causal_read_positions_nonzero":zero_read_positions==0,
                "maximum_mass_normalization_error":max_normalization_error},
            "positions":positions,"seconds":started.elapsed().as_secs_f64()}),
        first_states,
        first_read,
        integrity,
    })
}

fn run(
    evaluator_path: &Path,
    out: &Path,
    transfer_campaign_path: Option<&Path>,
    phase: &mut &'static str,
) -> Result<String> {
    let started = Instant::now();
    let evaluator = load_evaluator(evaluator_path)?;
    if evaluator.sha256 != EVALUATOR_SHA256 {
        return Err(invalid(
            "startup witness requires the pinned canonical evaluator v2",
        ));
    }
    let mut report = base_report(&evaluator, "radial-startup")?;
    report["schema"] = json!("uor-r4.radial-startup/1");
    report["scope"] = json!("One no-update initializer observation on previously exposed natural development prefixes. No training, calibration search, decoding change, model-quality or curvature claim. Later reader states diverge; aggregate NoRead ordering is not a theorem.");
    report["optimizer_constructed"] = json!(false);
    report["fixed_conditions"] = json!({"device":"cpu","parameter_seed":SEED,
        "initializer":"unit_scale","state_width":256,"read_vector_width":64,
        "observation_context":CONTEXT,"batch":BLOCKS.len(),"development_blocks":BLOCKS,
        "memory_access":"all causal occurrences within each full256 window; no selected-event policy",
        "transport":"quaternion","optimizer_updates":0});
    let lorentz_campaign = match transfer_campaign_path {
        Some(path) => {
            let cfg = Campaign::load(path)?;
            if cfg.shared_parameter_transfer.is_none()
                || cfg.read_initialization != Some(ReadInitialization::UnitScale)
                || cfg.model != campaign(evaluator_path, ReadGeometry::Lorentz).model
                || cfg.context != CONTEXT
                || cfg.training_window_transition.is_some()
                || cfg.quantization_transition.is_some()
                || cfg.projection_transition.is_some()
                || sha256_file(&cfg.evaluator_path)? != evaluator.sha256
            {
                return Err(invalid("transfer startup requires the fixed Lorentz model, unit_scale, shared transfer, canonical evaluator and no window/quantization/projection transition"));
            }
            report["supplied_transfer_campaign"] = json!({"path":path,"sha256":sha256_file(path)?});
            cfg
        }
        None => campaign(evaluator_path, ReadGeometry::Lorentz),
    };
    let mut affine_campaign = lorentz_campaign.clone();
    affine_campaign.model.read_geometry = ReadGeometry::LorentzAffine;
    report["initialization_source"] = json!(if transfer_campaign_path.is_some() {
        "learned_shared_parameter_transfer"
    } else {
        "fresh_unit_scale"
    });
    report["batch_scope"] = json!({"observation_batch":BLOCKS.len(),
        "declared_training_batch":lorentz_campaign.batch,
        "declared_training_gradient_shards":lorentz_campaign.cpu_gradient_shards,
        "note":"One unsharded four-window observation per arm. Campaign training batch, shards, dose and sampler are provenance only; no fit or optimizer is constructed."});
    let tokens = read_tokens(&evaluator.document["dev_source"])?;
    let tokenizer_identity = evaluator.document["reference_inputs"]
        .as_array()
        .and_then(|items| {
            items.iter().find(|item| {
                item["path"]
                    .as_str()
                    .is_some_and(|p| p.ends_with("/tokenizer.json"))
            })
        })
        .ok_or_else(|| invalid("canonical tokenizer identity missing"))?;
    let tokenizer_path = verify_identity(tokenizer_identity)?;
    let tokenizer = HfBpeTokenizer::from_dir(
        tokenizer_path
            .parent()
            .ok_or_else(|| invalid("tokenizer parent"))?,
    )
    .map_err(|error| invalid(format!("canonical tokenizer: {error}")))?;
    let mut inputs = Vec::with_capacity(BLOCKS.len() * CONTEXT);
    let mut targets = Vec::with_capacity(BLOCKS.len() * CONTEXT);
    let mut windows = Vec::new();
    for block in BLOCKS {
        let offset = block * CONTEXT;
        let ids: Vec<u32> = tokens
            .get(offset..offset + CONTEXT + 1)
            .ok_or_else(|| invalid("fixed development window unavailable"))?
            .iter()
            .map(|&id| u32::from(id))
            .collect();
        inputs.extend_from_slice(&ids[..CONTEXT]);
        targets.extend_from_slice(&ids[1..]);
        windows.push(json!({"block":block,"input_offset":offset,"stored_ids":ids,
            "decoded_input":tokenizer.decode(&ids[..CONTEXT])}));
    }
    save_json(&out.join("evaluator.json"), &evaluator.document)?;
    save_json(
        &out.join("inputs.json"),
        &json!({"verified_dev_identity":evaluator.document["dev_source"],
        "verified_tokenizer_identity":tokenizer_identity,"windows":windows,
        "selection":"Blocks0,21,42,63 fixed prospectively in the existing64-block tune region; fresh state per window; no injected BOS or EOS reset; targets shifted once.",
        "training_stores":"Not read or used; this witness has no optimizer updates."}),
    )?;
    let mut campaign_identities = Vec::new();
    for (name, cfg) in [
        ("lorentz", &lorentz_campaign),
        ("lorentz_affine", &affine_campaign),
    ] {
        let path = out.join(format!("{name}-campaign.json"));
        save_json(&path, cfg)?;
        if transfer_campaign_path.is_none() {
            Campaign::load(&path)?;
        }
        campaign_identities.push(json!({"path":path,"sha256":sha256_file(&path)?,
            "initializer":cfg.read_initialization,"scope":"initializer-only configuration; no fit"}));
    }
    *phase = "model_initialization";
    let (lorentz, lorentz_receipt) = lorentz_campaign.fresh_model_with_provenance(&Device::Cpu)?;
    let (affine, affine_receipt) = affine_campaign.fresh_model_with_provenance(&Device::Cpu)?;
    if lorentz_receipt.is_some() != transfer_campaign_path.is_some()
        || affine_receipt.is_some() != transfer_campaign_path.is_some()
    {
        return Err(invalid("startup transfer receipt presence differs"));
    }
    if transfer_campaign_path.is_some() {
        let path = out.join("transfer-receipts.json");
        save_json(
            &path,
            &json!({"lorentz":lorentz_receipt,"lorentz_affine":affine_receipt}),
        )?;
        report["transfer_receipts"] = json!({"path":path,"sha256":sha256_file(&path)?});
    }
    let parameters_identical = parameter_hashes(&lorentz)? == parameter_hashes(&affine)?;
    if !parameters_identical
        || scalar(&lorentz, LORENTZ_LOG_BETA)? != 0.0
        || scalar(&affine, LORENTZ_LOG_BETA)? != 0.0
    {
        return Err(invalid("shared unit-scale initialization differs"));
    }
    *phase = "forward_backward_observation";
    let left = observe(&lorentz, &inputs, &targets)?;
    save_json(&out.join("lorentz.json"), &left.report)?;
    let right = observe(&affine, &inputs, &targets)?;
    save_json(&out.join("lorentz_affine.json"), &right.report)?;
    *phase = "startup_generation";
    let source_prefix = &inputs[..32];
    let source_leading_bos_removed = source_prefix.first().copied() == Some(0);
    let natural_slice = if source_leading_bos_removed {
        &source_prefix[1..]
    } else {
        source_prefix
    };
    let prompt = tokenizer.decode(natural_slice);
    let reencoded_content_ids = tokenizer.encode(&prompt);
    let generated = [
        joint_evaluation::generate(
            &lorentz,
            &tokenizer,
            &prompt,
            ReadMode::Enabled,
            Some(2014),
            8,
        )?,
        joint_evaluation::generate(
            &affine,
            &tokenizer,
            &prompt,
            ReadMode::Enabled,
            Some(2014),
            8,
        )?,
    ];
    save_json(
        &out.join("generations.json"),
        &json!({
            "scope":if transfer_campaign_path.is_some() {
                "No-update learned-transfer startup outputs only; no adaptation or quality gate."
            } else {
                "Untrained startup outputs only; no quality gate."
            },
            "optimizer_updates":0,"selection_policy_changed":false,"decoder_sweep":false,
            "source_block":0,"source_prefix_token_ids":source_prefix,
            "source_leading_bos_removed":source_leading_bos_removed,
            "decoded_source_token_ids":natural_slice,
            "reencoded_content_token_ids":reencoded_content_ids,
            "roundtrip_matches_decoded_source_ids":reencoded_content_ids.as_slice()==natural_slice,
            "prompt_construction":"Take the first32 stored dev IDs, remove their leading BOS0 if present, and decode the remainder. The unchanged generation API re-encodes this text and prepends one BOS. Actual re-encoded content and each generation's complete prompt IDs are recorded; general token-boundary equivalence is not assumed.",
            "arms":["lorentz","lorentz_affine"],"generations":generated,
        }),
    )?;
    let generation_parameters_unchanged = serde_json::to_value(parameter_hashes(&lorentz)?)?
        == left.report["parameter_sha256_le_f32_after"]
        && serde_json::to_value(parameter_hashes(&affine)?)?
            == right.report["parameter_sha256_le_f32_after"];
    let first_states_identical = left
        .first_states
        .iter()
        .flatten()
        .zip(right.first_states.iter().flatten())
        .all(|(left, right)| left.to_bits() == right.to_bits());
    let log_odds = |read: f64, null: f64| {
        (read > 0.0 && null > 0.0 && read.is_finite() && null.is_finite())
            .then(|| (read / null).ln())
    };
    let first_reads: Vec<Value> = left
        .first_read
        .iter()
        .zip(&right.first_read)
        .enumerate()
        .map(|(lane, (&(lr, ln), &(ar, an)))| {
            let l = log_odds(lr, ln);
            let a = log_odds(ar, an);
            json!({"window":lane,"block":BLOCKS[lane],"position":1,"causal_key_position":0,
            "lorentz_read":lr,"lorentz_no_read":ln,"affine_read":ar,"affine_no_read":an,
            "lorentz_read_log_odds":l,"affine_read_log_odds":a,
            "lorentz_minus_affine_read_log_odds":l.zip(a).map(|(x,y)|x-y)})
        })
        .collect();
    let status = if parameters_identical
        && first_states_identical
        && generation_parameters_unchanged
        && left.integrity
        && right.integrity
    {
        "DESCRIPTIVE_STARTUP_COMPLETE"
    } else {
        "UNVERIFIED"
    };
    report["status"] = json!(status);
    report["campaigns"] = json!(campaign_identities);
    report["input_records_sha256"] = json!(sha256_file(&out.join("inputs.json"))?);
    report["shared_all_parameter_arrays_identical_before_observation"] =
        json!(parameters_identical);
    report["parameters_unchanged_after_generation"] = json!(generation_parameters_unchanged);
    report["generation_records_sha256"] = json!(sha256_file(&out.join("generations.json"))?);
    report["first_common_read"] = json!({"post_token0_states_bit_identical":first_states_identical,
        "post_token0_states_lorentz":left.first_states,"post_token0_states_affine":right.first_states,
        "observations":first_reads,
        "interpretation":"At position0 neither arm has a causal key. Identical parameters and post-token0 states imply common query/key/null/age at position1. Its single-key read log-odds difference observes the score-law difference, up to F32 normalization. Later trajectories differ; no later aggregate NoRead ordering is asserted. No private core formula is duplicated."});
    report["decision_boundary"]=json!("Completion requires finite observations, connected finite required gradients and unchanged shared parameters. Measured zero gradients or causal read masses remain descriptive activation findings, not unverified data and not automatic retry triggers. Their explicit booleans and counts inform whether the useful-gradient premise holds. No preferred read mass, entropy, loss threshold or language acceptance gate is imposed.");
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    save_json(&out.join("startup.json"), &report)?;
    Ok(status.into())
}

fn main() -> Result<()> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if !(2..=3).contains(&args.len()) {
        return Err(invalid(
            "usage: radial-startup EVALUATOR_JSON NEW_REPORT_ROOT [LORENTZ_TRANSFER_CAMPAIGN_JSON]",
        ));
    }
    let out = &args[1];
    report_output::claim(out)?;
    let mut phase = "input_validation";
    let result = run(&args[0], out, args.get(2).map(PathBuf::as_path), &mut phase);
    if let Err(error) = &result {
        save_json(
            &out.join("failed-attempt.json"),
            &json!({"status":if phase=="input_validation" {"NOT_RUN"}else{"UNVERIFIED"},
            "phase":phase,"error":error.to_string(),"optimizer_updates":0,
            "scope":"Failed startup observation is not model-quality evidence"}),
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    let status = result?;
    println!("{status}: {}; sealed and verified", out.display());
    Ok(())
}
