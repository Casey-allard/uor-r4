//! One zero-update check that explicit Dot reset preserves the actual parent
//! predictions. Resource supervision is external; no fit or dose is selected.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use candle_core::{Device, Tensor};
use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::baseline_protocol::{
    base_report, load_evaluator, read_tokens, save_json, verify_identity,
};
use uor_r4_training::joint_campaign::Campaign;
use uor_r4_training::joint_evaluation;
use uor_r4_training::joint_model::{JointModel, ReadGeometry, ReadMode};
use uor_r4_training::joint_optimizer::NamedAdamW;
use uor_r4_training::{sha256_file, Result, TrainingError};

const BLOCKS: [usize; 4] = [0, 21, 42, 63];
const CONTEXT: usize = 256;
const EVALUATOR: &str = "d2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62";

fn invalid(message: &str) -> TrainingError {
    TrainingError::Invalid(message.into())
}

fn tensor_sha(tensor: &Tensor) -> Result<String> {
    let mut digest = Sha256::new();
    for value in tensor.flatten_all()?.to_vec1::<f32>()? {
        if !value.is_finite() {
            return Err(invalid("witness tensor contains a nonfinite value"));
        }
        digest.update(value.to_le_bytes());
    }
    Ok(hex::encode(digest.finalize()))
}

fn arrays(model: &JointModel) -> Result<BTreeMap<String, String>> {
    model
        .variables()
        .iter()
        .map(|(name, variable)| Ok((name.clone(), tensor_sha(variable.as_tensor())?)))
        .collect()
}

fn run(campaign_path: &Path, out: &Path) -> Result<()> {
    let cfg = Campaign::load(campaign_path)?;
    if cfg.model.read_geometry != ReadGeometry::Dot
        || cfg.read_initialization.is_some()
        || cfg.context != CONTEXT
        || cfg.model.context != CONTEXT
    {
        return Err(invalid("witness requires a full256 Dot transfer campaign"));
    }
    let evaluator = load_evaluator(&cfg.evaluator_path)?;
    if evaluator.sha256 != EVALUATOR {
        return Err(invalid("witness requires the canonical evaluator"));
    }
    // Reject an unbound executable before constructing or observing a model.
    let mut report = base_report(&evaluator, "dot-transfer-witness")?;
    let declaration = cfg
        .shared_parameter_transfer
        .as_ref()
        .ok_or_else(|| invalid("Dot reset declaration missing"))?;
    let (child, receipt) = cfg.fresh_model_with_provenance(&Device::Cpu)?;
    let receipt = receipt.ok_or_else(|| invalid("Dot reset receipt missing"))?;
    // Fresh construction has verified the exact sealed source and all copied
    // arrays. Load that same parent to observe its actual predictions directly.
    report_output::verify(&declaration.parent_checkpoint)?;
    let parent = JointModel::load(&declaration.parent_checkpoint, &Device::Cpu)?;
    let before = arrays(&child)?;
    if before != arrays(&parent)? || receipt.initial_radial_scalars.is_some() {
        return Err(invalid("Dot reset changed parent parameter inventory"));
    }
    let optimizer = NamedAdamW::new(child.variables(), cfg.optimizer.clone())?;
    if optimizer.step_count() != 0 {
        return Err(invalid("fresh Dot optimizer did not start at zero"));
    }
    let tokens = read_tokens(&evaluator.document["dev_source"])?;
    let mut inputs = Vec::new();
    let mut targets = Vec::new();
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
        windows.push(json!({"block":block,"input_offset":offset,"stored_ids":ids}));
    }
    let original = parent.forward(&inputs, BLOCKS.len(), CONTEXT, ReadMode::Enabled, false)?;
    let reset = child.forward(&inputs, BLOCKS.len(), CONTEXT, ReadMode::Enabled, false)?;
    let original_probabilities = tensor_sha(&original.probabilities)?;
    let reset_probabilities = tensor_sha(&reset.probabilities)?;
    let original_nll = original.loss(&targets)?.to_scalar::<f32>()?;
    let reset_nll = reset.loss(&targets)?.to_scalar::<f32>()?;
    if original_probabilities != reset_probabilities
        || !reset_nll.is_finite()
        || original_nll.to_bits() != reset_nll.to_bits()
    {
        return Err(invalid("Dot reset changed actual full256 predictions"));
    }
    let tokenizer_identity = evaluator.document["reference_inputs"]
        .as_array()
        .and_then(|entries| {
            entries.iter().find(|entry| {
                entry["path"]
                    .as_str()
                    .is_some_and(|path| path.ends_with("/tokenizer.json"))
            })
        })
        .ok_or_else(|| invalid("canonical tokenizer missing"))?;
    let tokenizer_path = verify_identity(tokenizer_identity)?;
    let tokenizer = HfBpeTokenizer::from_dir(
        tokenizer_path
            .parent()
            .ok_or_else(|| invalid("tokenizer parent missing"))?,
    )
    .map_err(|error| TrainingError::Invalid(error.to_string()))?;
    let source_prefix = &inputs[..32];
    let content = if source_prefix.first() == Some(&0) {
        &source_prefix[1..]
    } else {
        source_prefix
    };
    let prompt = tokenizer.decode(content);
    let original_generation = joint_evaluation::generate(
        &parent,
        &tokenizer,
        &prompt,
        ReadMode::Enabled,
        Some(2014),
        8,
    )?;
    let reset_generation = joint_evaluation::generate(
        &child,
        &tokenizer,
        &prompt,
        ReadMode::Enabled,
        Some(2014),
        8,
    )?;
    if original_generation.generated_token_ids != reset_generation.generated_token_ids
        || original_generation
            .decisions
            .iter()
            .map(|row| &row.probabilities_sha256_le_f32)
            .collect::<Vec<_>>()
            != reset_generation
                .decisions
                .iter()
                .map(|row| &row.probabilities_sha256_le_f32)
                .collect::<Vec<_>>()
        || arrays(&child)? != before
        || arrays(&parent)? != before
    {
        return Err(invalid("Dot reset changed generation or parameter arrays"));
    }
    save_json(&out.join("campaign.json"), &cfg)?;
    save_json(&out.join("transfer-receipt.json"), &receipt)?;
    save_json(&out.join("evaluator.json"), &evaluator.document)?;
    save_json(&out.join("inputs.json"), &json!({"windows":windows}))?;
    save_json(
        &out.join("generations.json"),
        &json!({"source_prefix_ids":source_prefix,"decoded_source_ids":content,
        "reencoded_content_ids":tokenizer.encode(&prompt),
        "roundtrip_matches":tokenizer.encode(&prompt).as_slice()==content,
        "parent":original_generation,"dot_reset":reset_generation}),
    )?;
    report["schema"] = json!("uor-r4.dot-transfer-witness/1");
    report["status"] = json!("UNCHANGED_DOT_OUTPUT_WITH_FRESH_OPTIMIZER");
    report["scope"] = json!("One actual same-geometry reset observation on exposed development inputs. No training update, dose selection, general language or performance claim.");
    report["supplied_campaign_sha256"] = json!(sha256_file(campaign_path)?);
    report["saved_campaign_sha256"] = json!(sha256_file(&out.join("campaign.json"))?);
    report["transfer_receipt"] = json!(receipt);
    report["parameter_sha256_le_f32"] = json!(before);
    report["probabilities_sha256_le_f32"] = json!(reset_probabilities);
    report["initial_next_token_nll"] = json!(reset_nll);
    report["parent_and_reset_predictions_identical"] = json!(true);
    report["parent_and_reset_generation_identical"] = json!(true);
    report["parameters_unchanged"] = json!(true);
    report["optimizer_step"] = json!(optimizer.step_count());
    report["optimizer_fingerprint"] = json!(optimizer.continuity_fingerprint()?);
    report["fixed_conditions"] = json!({"device":"cpu","observation_batch":BLOCKS.len(),
        "training_batch":cfg.batch,"training_context":cfg.context,
        "observation_context":CONTEXT,"blocks":BLOCKS,"candidate_updates":0,
        "memory_access":"all causal history within each full256 window"});
    save_json(&out.join("witness.json"), &report)
}

fn main() -> Result<()> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args.len() != 2 {
        return Err(invalid(
            "usage: dot-transfer-witness DOT_CAMPAIGN_JSON NEW_REPORT_ROOT",
        ));
    }
    report_output::claim(&args[1])?;
    let result = run(&args[0], &args[1]);
    if let Err(error) = &result {
        save_json(
            &args[1].join("failed-attempt.json"),
            &json!({"status":"UNVERIFIED","error":error.to_string(),"candidate_updates":0}),
        )?;
    }
    report_output::seal(&args[1])?;
    report_output::verify(&args[1])?;
    result?;
    println!(
        "UNCHANGED_DOT_OUTPUT_WITH_FRESH_OPTIMIZER: {}; sealed and verified",
        args[1].display()
    );
    Ok(())
}
