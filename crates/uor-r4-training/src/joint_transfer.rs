//! Explicit parameter-only initialization of radial readers and their matched
//! Dot reset control from a learned Dot checkpoint. This is a new campaign
//! lineage, never an optimizer resume.

use std::collections::BTreeMap;
use std::path::PathBuf;

use candle_core::{DType, Var};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::baseline_protocol::{load_evaluator, verify_identity};
use crate::joint_admission::AdmissionPolicy;
use crate::joint_campaign::{load_bound_checkpoint, Campaign, ReadInitialization};
use crate::joint_model::{
    lorentz_initial_offset, JointConfig, JointModel, ReadGeometry, LORENTZ_LOG_BETA, LORENTZ_OFFSET,
};
use crate::joint_optimizer::AdamConfig;
use crate::{invalid, sha256_file, Result};

const RECEIPT_SCHEMA: &str = "uor-r4.shared-parameter-transfer/1";

/// No optimizer moments or clocks are transferred, including shared variables.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferOptimizer {
    ResetAllMomentsAndClocks,
}

/// Comparison arms declare the same new seed and begin at local step zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "policy", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransferSampler {
    FreshCounter { data_seed: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SharedParameterTransfer {
    pub parent_checkpoint: PathBuf,
    pub parent_checkpoint_sha256: String,
    pub parent_campaign_sha256: String,
    pub parent_optimizer_step: usize,
    pub parent_sampled_target_visits: usize,
    pub optimizer: TransferOptimizer,
    pub sampler: TransferSampler,
}

/// Hash of the exact F32 little-endian array, in its existing contiguous order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferredParameter {
    pub name: String,
    pub shape: Vec<usize>,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitialRadialScalars {
    pub log_beta: f32,
    pub offset: f32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferTokenizerIdentity {
    pub path: PathBuf,
    pub bytes: u64,
    pub sha256: String,
}

/// Immutable evidence of initialization. These hashes describe the arrays at
/// transfer time; subsequent training must not compare its evolved arrays to
/// them or reopen the original parent merely to resume the new checkpoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferReceipt {
    pub schema: String,
    pub specification: SharedParameterTransfer,
    pub parent_model: JointConfig,
    pub parent_optimizer: AdamConfig,
    pub parent_training_batch: usize,
    pub parent_training_context: usize,
    pub parent_data_seed: u64,
    pub parent_model_sha256: String,
    pub parent_model_config_sha256: String,
    pub parent_source_commit: String,
    pub evaluator_sha256: String,
    pub tokenizer: TransferTokenizerIdentity,
    pub target_read_geometry: ReadGeometry,
    pub copied_parameters: Vec<TransferredParameter>,
    /// SHA-256 of serde_json's compact serialization of copied_parameters.
    pub shared_parameters_sha256: String,
    pub copied_scalar_count: usize,
    /// Radial receipts retain their existing object representation. The Dot
    /// reset control has no radial parameters and omits this field entirely.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_radial_scalars: Option<InitialRadialScalars>,
    pub optimizer_start_step: usize,
    pub next_data_step: usize,
}

impl SharedParameterTransfer {
    /// Validate immutable declaration fields without reopening its parent.
    /// Later declared quantization/projection transitions may retain this
    /// historical initializer; first application rejects such transitions.
    pub fn validate_declaration(&self, cfg: &Campaign) -> Result<()> {
        cfg.model.validate()?;
        let TransferSampler::FreshCounter { data_seed } = self.sampler;
        let initializer_matches = match cfg.model.read_geometry {
            ReadGeometry::Dot => cfg.read_initialization.is_none(),
            ReadGeometry::Lorentz | ReadGeometry::LorentzAffine => {
                cfg.read_initialization == Some(ReadInitialization::UnitScale)
            }
        };
        if !self.parent_checkpoint.is_absolute()
            || cfg.shared_parameter_transfer.as_ref() != Some(self)
            || !hex_digest(&self.parent_checkpoint_sha256, 64)
            || !hex_digest(&self.parent_campaign_sha256, 64)
            || self.parent_optimizer_step == 0
            || self.parent_sampled_target_visits == 0
            || !initializer_matches
            || cfg.data_seed != data_seed
        {
            return Err(invalid("invalid shared-parameter transfer declaration"));
        }
        Ok(())
    }

    /// Apply once to an unprepared continuous model before any optimizer
    /// is constructed. All logical checks precede Var::set. A device write
    /// failure is returned and the caller must discard the target/attempt.
    pub fn apply(
        &self,
        cfg: &Campaign,
        evaluator_sha: &str,
        target: &mut JointModel,
    ) -> Result<TransferReceipt> {
        self.validate_declaration(cfg)?;
        if cfg.shared_parameter_transfer.as_ref() != Some(self)
            || !hex_digest(evaluator_sha, 64)
            || cfg.training_window_transition.is_some()
            || cfg.quantization_transition.is_some()
            || cfg.projection_transition.is_some()
            || target.config != cfg.model
            || target.quantization().is_some()
            || target.admission_policy() != AdmissionPolicy::Full
        {
            return Err(invalid(
                "transfer requires a fresh continuous full-access campaign",
            ));
        }
        // Existing model boundary rejects prepared/hard/precision views. A
        // continuous model has no clock to mutate, so this only validates it.
        target.set_completed_step(0)?;
        let scalars = initial_scalars(target)?;
        validate_initial_scalars(&cfg.model, &scalars)?;
        let evaluator = load_evaluator(&cfg.evaluator_path)?;
        if evaluator.sha256 != evaluator_sha {
            return Err(invalid(
                "transfer evaluator file differs from requested identity",
            ));
        }
        let references = evaluator.document["reference_inputs"]
            .as_array()
            .ok_or_else(|| invalid("transfer evaluator reference inventory missing"))?;
        let mut tokenizers = references.iter().filter(|identity| {
            identity["path"].as_str().is_some_and(|p| {
                std::path::Path::new(p)
                    .file_name()
                    .is_some_and(|n| n == "tokenizer.json")
            })
        });
        let tokenizer = tokenizers
            .next()
            .ok_or_else(|| invalid("transfer evaluator tokenizer missing"))?;
        if tokenizers.next().is_some() {
            return Err(invalid(
                "transfer evaluator tokenizer identity is ambiguous",
            ));
        }
        let tokenizer_path = verify_identity(tokenizer)?;
        let tokenizer = TransferTokenizerIdentity {
            path: tokenizer_path,
            bytes: tokenizer["bytes"]
                .as_u64()
                .ok_or_else(|| invalid("transfer tokenizer size missing"))?,
            sha256: bound_string(tokenizer, "sha256")?,
        };
        if sha256_file(&self.parent_checkpoint.join("checkpoint.json"))?
            != self.parent_checkpoint_sha256
            || sha256_file(&self.parent_checkpoint.join("campaign.json"))?
                != self.parent_campaign_sha256
        {
            return Err(invalid("transfer parent checkpoint/campaign hash differs"));
        }
        let parent =
            load_bound_checkpoint(&self.parent_checkpoint, target.device(), evaluator_sha)?;
        let mut expected_parent = cfg.model.clone();
        expected_parent.read_geometry = ReadGeometry::Dot;
        if parent.model.config != expected_parent
            || parent.model.quantization().is_some()
            || parent.model.admission_policy() != AdmissionPolicy::Full
            || parent.campaign.shared_parameter_transfer.is_some()
            || parent.campaign.optimizer != cfg.optimizer
            || parent.campaign.batch != cfg.batch
            || parent.campaign.context != cfg.context
            || parent.campaign.data_seed == cfg.data_seed
            || parent.binding["optimizer_step"] != serde_json::json!(self.parent_optimizer_step)
            || parent.binding["sampled_target_visits"]
                != serde_json::json!(self.parent_sampled_target_visits)
        {
            return Err(invalid(
                "transfer parent differs from declared continuous Dot lineage",
            ));
        }
        let expected_shapes = expected_parent.shapes();
        validate_inventory(&parent.model, &expected_shapes)?;
        validate_inventory(target, &cfg.model.shapes())?;
        let parameters = parameter_manifest(&parent.model, &expected_shapes)?;
        let scalar_count = count_scalars(&parameters)?;
        let receipt = TransferReceipt {
            schema: RECEIPT_SCHEMA.into(),
            specification: self.clone(),
            parent_model: expected_parent,
            parent_optimizer: parent.campaign.optimizer.clone(),
            parent_training_batch: parent.campaign.batch,
            parent_training_context: parent.campaign.context,
            parent_data_seed: parent.campaign.data_seed,
            parent_model_sha256: bound_string(&parent.binding, "model_sha256")?,
            parent_model_config_sha256: sha256_file(&self.parent_checkpoint.join("config.json"))?,
            parent_source_commit: bound_string(&parent.binding, "source_commit")?,
            evaluator_sha256: evaluator_sha.into(),
            tokenizer,
            target_read_geometry: cfg.model.read_geometry,
            shared_parameters_sha256: manifest_digest(&parameters)?,
            copied_parameters: parameters,
            copied_scalar_count: scalar_count,
            initial_radial_scalars: scalars,
            optimizer_start_step: 0,
            next_data_step: 0,
        };
        self.validate_receipt(cfg, &receipt)?;
        for name in expected_shapes.keys() {
            let source = parent
                .model
                .variables()
                .get(name)
                .ok_or_else(|| invalid("validated transfer source disappeared"))?;
            let destination = target
                .variables()
                .get(name)
                .ok_or_else(|| invalid("validated transfer destination disappeared"))?;
            destination.set(&source.as_detached_tensor())?;
        }
        if parameter_manifest(target, &expected_shapes)? != receipt.copied_parameters
            || initial_scalars(target)? != receipt.initial_radial_scalars
        {
            return Err(invalid(
                "transferred arrays or preserved radial scalars differ",
            ));
        }
        Ok(receipt)
    }

    /// Check a self-contained historical receipt, never current learned arrays.
    pub fn validate_receipt(&self, cfg: &Campaign, receipt: &TransferReceipt) -> Result<()> {
        self.validate_declaration(cfg)?;
        let mut parent_config = cfg.model.clone();
        parent_config.read_geometry = ReadGeometry::Dot;
        let shapes = parent_config.shapes();
        let observed: BTreeMap<_, _> = receipt
            .copied_parameters
            .iter()
            .map(|parameter| (parameter.name.clone(), parameter.shape.clone()))
            .collect();
        let expected_visits = self
            .parent_optimizer_step
            .checked_mul(receipt.parent_training_batch)
            .and_then(|n| n.checked_mul(receipt.parent_training_context));
        validate_historical_windows(cfg, receipt)?;
        if receipt.schema != RECEIPT_SCHEMA
            || receipt.specification != *self
            || receipt.parent_model != parent_config
            || receipt.parent_optimizer != cfg.optimizer
            || receipt.parent_data_seed == cfg.data_seed
            || expected_visits != Some(self.parent_sampled_target_visits)
            || receipt.target_read_geometry != cfg.model.read_geometry
            || !hex_digest(&receipt.parent_model_sha256, 64)
            || !hex_digest(&receipt.parent_model_config_sha256, 64)
            || !hex_digest(&receipt.parent_source_commit, 40)
            || !hex_digest(&receipt.evaluator_sha256, 64)
            || !receipt.tokenizer.path.is_absolute()
            || receipt.tokenizer.bytes == 0
            || !hex_digest(&receipt.tokenizer.sha256, 64)
            || observed != shapes
            || receipt.copied_parameters.len() != shapes.len()
            || receipt
                .copied_parameters
                .windows(2)
                .any(|p| p[0].name >= p[1].name)
            || receipt
                .copied_parameters
                .iter()
                .any(|p| !hex_digest(&p.sha256, 64))
            || receipt.shared_parameters_sha256 != manifest_digest(&receipt.copied_parameters)?
            || receipt.copied_scalar_count != count_scalars(&receipt.copied_parameters)?
            || receipt.optimizer_start_step != 0
            || receipt.next_data_step != 0
        {
            return Err(invalid(
                "transfer receipt differs from immutable campaign lineage",
            ));
        }
        validate_initial_scalars(&cfg.model, &receipt.initial_radial_scalars)
    }
}

fn validate_historical_windows(cfg: &Campaign, receipt: &TransferReceipt) -> Result<()> {
    let historical_targets = receipt
        .parent_training_batch
        .checked_mul(receipt.parent_training_context);
    if !(1..=64).contains(&receipt.parent_training_batch)
        || !(8..=cfg.model.context).contains(&receipt.parent_training_context)
        || !(1..=64).contains(&cfg.batch)
        || !(8..=cfg.model.context).contains(&cfg.context)
        || historical_targets != cfg.batch.checked_mul(cfg.context)
    {
        return Err(invalid(
            "transfer historical/current window exposure differs",
        ));
    }
    if receipt.parent_training_batch != cfg.batch || receipt.parent_training_context != cfg.context
    {
        let transition = cfg
            .training_window_transition
            .as_ref()
            .ok_or_else(|| invalid("transfer windows changed without a declared transition"))?;
        // The latest transition may follow other valid transitions. Its old
        // dimensions need not equal the original transfer parent's, but every
        // hop must preserve targets/update. Campaign resume binds this latest
        // declaration to the actual immediate parent checkpoint and campaign.
        if !(1..=64).contains(&transition.old_batch)
            || !(8..=cfg.model.context).contains(&transition.old_context)
            || transition.old_batch.checked_mul(transition.old_context) != historical_targets
            || transition.new_batch != cfg.batch
            || transition.new_context != cfg.context
            || (transition.old_batch == cfg.batch && transition.old_context == cfg.context)
            || transition.parent_optimizer_step == 0
            || historical_targets.and_then(|n| n.checked_mul(transition.parent_optimizer_step))
                != Some(transition.parent_sampled_target_visits)
            || !hex_digest(&transition.parent_checkpoint_sha256, 64)
            || !hex_digest(&transition.parent_campaign_sha256, 64)
            || transition.reason.trim().is_empty()
        {
            return Err(invalid("transfer window-transition declaration differs"));
        }
    }
    Ok(())
}

/// The campaign spec and receipt must either both be absent (legacy) or both be
/// bound. This helper performs no I/O and does not reapply initialization.
pub fn validate_binding(cfg: &Campaign, binding: &Value) -> Result<Option<TransferReceipt>> {
    if binding["shared_parameter_transfer"] != serde_json::to_value(&cfg.shared_parameter_transfer)?
    {
        return Err(invalid(
            "checkpoint/campaign shared-parameter transfer differs",
        ));
    }
    match &cfg.shared_parameter_transfer {
        None if binding["transfer_receipt"].is_null() => Ok(None),
        None => Err(invalid("transfer receipt has no campaign declaration")),
        Some(specification) => {
            let receipt: TransferReceipt =
                serde_json::from_value(binding["transfer_receipt"].clone())?;
            specification.validate_receipt(cfg, &receipt)?;
            if binding["evaluator_sha256"] != receipt.evaluator_sha256 {
                return Err(invalid("transfer/checkpoint evaluator differs"));
            }
            Ok(Some(receipt))
        }
    }
}

fn hex_digest(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn bound_string(binding: &Value, key: &str) -> Result<String> {
    binding[key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| invalid(format!("transfer parent missing {key}")))
}

fn initial_scalars(model: &JointModel) -> Result<Option<InitialRadialScalars>> {
    if model.config.read_geometry == ReadGeometry::Dot {
        if [LORENTZ_LOG_BETA, LORENTZ_OFFSET]
            .iter()
            .any(|name| model.variables().contains_key(*name))
        {
            return Err(invalid("Dot transfer target contains radial parameters"));
        }
        return Ok(None);
    }
    let scalar = |name: &str| -> Result<f32> {
        let variable = model
            .variables()
            .get(name)
            .ok_or_else(|| invalid(format!("transfer target missing {name}")))?;
        if variable.dims() != [1] || variable.dtype() != DType::F32 {
            return Err(invalid("transfer radial scalar shape/dtype differs"));
        }
        variable
            .to_vec1::<f32>()?
            .first()
            .copied()
            .ok_or_else(|| invalid("empty transfer radial scalar"))
    };
    Ok(Some(InitialRadialScalars {
        log_beta: scalar(LORENTZ_LOG_BETA)?,
        offset: scalar(LORENTZ_OFFSET)?,
    }))
}

fn validate_initial_scalars(
    config: &JointConfig,
    scalars: &Option<InitialRadialScalars>,
) -> Result<()> {
    match (config.read_geometry, scalars) {
        (ReadGeometry::Dot, None) => Ok(()),
        (ReadGeometry::Lorentz | ReadGeometry::LorentzAffine, Some(scalars))
            if scalars.log_beta.to_bits() == 0.0f32.to_bits()
                && scalars.offset.to_bits()
                    == (lorentz_initial_offset(config) as f32).to_bits() =>
        {
            Ok(())
        }
        _ => Err(invalid(
            "transfer radial scalar metadata differs from target geometry/initialization",
        )),
    }
}

fn validate_inventory(model: &JointModel, expected: &BTreeMap<String, Vec<usize>>) -> Result<()> {
    let actual: BTreeMap<_, _> = model
        .variables()
        .iter()
        .map(|(name, variable)| (name.clone(), variable.dims().to_vec()))
        .collect();
    if &actual != expected
        || model.variables().values().any(|v| {
            v.dtype() != DType::F32 || !v.is_contiguous() || !v.device().same_device(model.device())
        })
    {
        return Err(invalid(
            "transfer parameter inventory/shape/dtype/device differs",
        ));
    }
    Ok(())
}

fn parameter_digest(variable: &Var) -> Result<String> {
    let mut digest = Sha256::new();
    for value in variable.flatten_all()?.to_vec1::<f32>()? {
        if !value.is_finite() {
            return Err(invalid("nonfinite transfer parameter"));
        }
        digest.update(value.to_le_bytes());
    }
    Ok(hex::encode(digest.finalize()))
}

fn parameter_manifest(
    model: &JointModel,
    shapes: &BTreeMap<String, Vec<usize>>,
) -> Result<Vec<TransferredParameter>> {
    shapes
        .iter()
        .map(|(name, shape)| {
            let variable = model
                .variables()
                .get(name)
                .ok_or_else(|| invalid("transfer manifest parameter missing"))?;
            Ok(TransferredParameter {
                name: name.clone(),
                shape: shape.clone(),
                sha256: parameter_digest(variable)?,
            })
        })
        .collect()
}

fn manifest_digest(parameters: &[TransferredParameter]) -> Result<String> {
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(parameters)?)))
}

fn count_scalars(parameters: &[TransferredParameter]) -> Result<usize> {
    parameters.iter().try_fold(0usize, |total, parameter| {
        let count = parameter
            .shape
            .iter()
            .try_fold(1usize, |n, &d| n.checked_mul(d))
            .ok_or_else(|| invalid("transfer parameter shape overflow"))?;
        total
            .checked_add(count)
            .ok_or_else(|| invalid("transfer parameter count overflow"))
    })
}

#[cfg(test)]
pub(crate) fn transfer_test_fixture() -> Result<(PathBuf, Campaign)> {
    let fixture = tests::transfer_fixture(AdmissionPolicy::Full)?;
    Ok((fixture.root, fixture.campaign))
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::{Device, Tensor};
    use serde_json::json;
    use std::fs;
    use uor_r4_core::report_output;

    pub(super) struct Fixture {
        pub(super) root: PathBuf,
        pub(super) campaign: Campaign,
        specification: SharedParameterTransfer,
        evaluator_sha: String,
        parameters: Vec<TransferredParameter>,
    }

    pub(super) fn transfer_fixture(admission: AdmissionPolicy) -> Result<Fixture> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| invalid("test clock"))?
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("uor-transfer-{}-{nonce}", std::process::id()));
        report_output::claim(&root)?;
        let tokenizer_path = root.join("tokenizer.json");
        let tokenizer_bytes = br#"{"scope":"synthetic identity fixture; no tokenization"}"#;
        fs::write(&tokenizer_path, tokenizer_bytes)?;
        let evaluator_path = root.join("evaluator.json");
        fs::write(
            &evaluator_path,
            serde_json::to_vec(&json!({
                "schema":"uor-r4.reference-evaluator/2", "context":256, "stride":256,
                "vocabulary":4096, "full_blocks":976, "tune_blocks":64,
                "comparison_blocks":912, "scored_targets":249856,
                "ngram":{"discount_grid":[0.5,0.75,0.9]},
                "cache":{"mixture_grid":[0,0.05,0.1,0.2,0.4,0.6],"capacity":256},
                "reference_inputs":[{"path":tokenizer_path,"bytes":tokenizer_bytes.len(),
                    "sha256":sha256_file(&tokenizer_path)?}]
            }))?,
        )?;
        let evaluator_sha = sha256_file(&evaluator_path)?;
        let parent_config = JointConfig {
            width: 128,
            context: 16,
            seed: 7,
            ..JointConfig::default()
        };
        let mut model = JointModel::new(parent_config.clone(), &Device::Cpu)?;
        model.set_admission_policy(admission)?;
        // Change every array so a missed name cannot pass as a coincidentally
        // identical constructor value. This is a serialized fixture, not a fit.
        for (index, variable) in model.variables().values().enumerate() {
            variable.set(&variable.affine(1.0, (index + 1) as f64 / 1000.0)?)?;
        }
        let parameters = parameter_manifest(&model, &parent_config.shapes())?;
        let parent = root.join("parent");
        report_output::claim(&parent)?;
        model.save(&parent)?;
        let parent_campaign: Campaign = serde_json::from_value(json!({
            "schema":"uor-r4.joint-recurrent-campaign/1", "evaluator_path":evaluator_path,
            "model":parent_config,"optimizer":AdamConfig::default(),"data_seed":42,
            "batch":2,"context":8,"cpu_gradient_shards":1,"total_steps":3,
            "development_every_steps":0,"development_blocks":0,"checkpoint_steps":[],
            "max_process_seconds":60,"trial_scope":"synthetic transfer boundary fixture"
        }))?;
        fs::write(
            parent.join("campaign.json"),
            serde_json::to_vec(&parent_campaign)?,
        )?;
        // Only the existing bound-model load contract is under test; no model
        // optimizer execution, performance or trained-artifact claim is made.
        fs::write(
            parent.join("checkpoint.json"),
            serde_json::to_vec(&json!({
                "schema":"uor-r4.joint-recurrent-checkpoint/1", "status":"SYNTHETIC_FIXTURE",
            "optimizer_step":2,"sampled_target_visits":32,"next_data_step":2,
            "data_seed":42,"training_batch":2,"training_context":8,
            "cpu_gradient_shards":1,"sampled_targets_per_step":16,
                "training_window_transition":null,"quantization_transition":null,
                "quantization":null,"evaluator_sha256":evaluator_sha,
                "data_sampler":"splitmix64-counter-v1;valid-window-union;no-cross-store;step/lane-bound",
                "model_sha256":sha256_file(&parent.join("model.safetensors"))?,
                "model_config_sha256":sha256_file(&parent.join("config.json"))?,
                "source_commit":"a".repeat(40)
            }))?,
        )?;
        report_output::seal(&parent)?;
        report_output::verify(&parent)?;
        let specification = SharedParameterTransfer {
            parent_checkpoint: parent.clone(),
            parent_checkpoint_sha256: sha256_file(&parent.join("checkpoint.json"))?,
            parent_campaign_sha256: sha256_file(&parent.join("campaign.json"))?,
            parent_optimizer_step: 2,
            parent_sampled_target_visits: 32,
            optimizer: TransferOptimizer::ResetAllMomentsAndClocks,
            sampler: TransferSampler::FreshCounter { data_seed: 43 },
        };
        let mut campaign = parent_campaign;
        campaign.model.read_geometry = ReadGeometry::Lorentz;
        campaign.read_initialization = Some(ReadInitialization::UnitScale);
        campaign.data_seed = 43;
        campaign.shared_parameter_transfer = Some(specification.clone());
        Ok(Fixture {
            root,
            campaign,
            specification,
            evaluator_sha,
            parameters,
        })
    }

    fn raw_target(cfg: &Campaign) -> Result<JointModel> {
        let target = JointModel::new(cfg.model.clone(), &Device::Cpu)?;
        if cfg.model.read_geometry != ReadGeometry::Dot {
            target
                .variables()
                .get(LORENTZ_LOG_BETA)
                .ok_or_else(|| invalid("test log(beta) missing"))?
                .set(&Tensor::new(&[0.0f32], &Device::Cpu)?)?;
        }
        Ok(target)
    }

    fn select_geometry(cfg: &mut Campaign, geometry: ReadGeometry) {
        cfg.model.read_geometry = geometry;
        cfg.read_initialization = match geometry {
            ReadGeometry::Dot => None,
            ReadGeometry::Lorentz | ReadGeometry::LorentzAffine => {
                Some(ReadInitialization::UnitScale)
            }
        };
    }

    #[test]
    fn radial_transfer_copies_all_shared_arrays_and_preserves_only_new_scalars() -> Result<()> {
        let mut fixture = transfer_fixture(AdmissionPolicy::Full)?;
        let original_file_sha = sha256_file(
            &fixture
                .specification
                .parent_checkpoint
                .join("model.safetensors"),
        )?;
        let mut previous_manifest = None;
        for geometry in [
            ReadGeometry::Dot,
            ReadGeometry::Lorentz,
            ReadGeometry::LorentzAffine,
        ] {
            select_geometry(&mut fixture.campaign, geometry);
            let mut target = raw_target(&fixture.campaign)?;
            let initial = initial_scalars(&target)?;
            let receipt = fixture.specification.apply(
                &fixture.campaign,
                &fixture.evaluator_sha,
                &mut target,
            )?;
            assert_eq!(receipt.copied_parameters, fixture.parameters);
            assert_eq!(receipt.initial_radial_scalars, initial);
            assert_eq!(initial_scalars(&target)?, initial);
            assert_eq!(
                receipt.copied_parameters.len() + if initial.is_some() { 2 } else { 0 },
                target.variables().len()
            );
            let serialized = serde_json::to_vec(&receipt)?;
            let mut legacy_shape = serde_json::to_value(&receipt)?;
            if geometry == ReadGeometry::Dot {
                assert!(initial.is_none());
                assert!(legacy_shape.get("initial_radial_scalars").is_none());
            } else {
                // The prior non-optional field was this same JSON object,
                // without an Option wrapper or any schema/version change.
                let old_object = json!({"log_beta":0.0,
                    "offset":lorentz_initial_offset(&fixture.campaign.model) as f32});
                assert_eq!(legacy_shape["initial_radial_scalars"], old_object);
                legacy_shape["initial_radial_scalars"] = old_object;
            }
            let decoded: TransferReceipt = serde_json::from_value(legacy_shape)?;
            assert_eq!(serde_json::to_vec(&decoded)?, serialized);
            fixture
                .specification
                .validate_receipt(&fixture.campaign, &decoded)?;
            let mut wrong_scalars = receipt.clone();
            wrong_scalars.initial_radial_scalars = if geometry == ReadGeometry::Dot {
                Some(InitialRadialScalars {
                    log_beta: 0.0,
                    offset: lorentz_initial_offset(&fixture.campaign.model) as f32,
                })
            } else {
                None
            };
            assert!(fixture
                .specification
                .validate_receipt(&fixture.campaign, &wrong_scalars)
                .is_err());
            if let Some(previous) = previous_manifest {
                assert_eq!(receipt.shared_parameters_sha256, previous);
            }
            previous_manifest = Some(receipt.shared_parameters_sha256);
            assert_eq!(receipt.optimizer_start_step, 0);
            assert_eq!(receipt.next_data_step, 0);
        }
        assert_eq!(
            sha256_file(
                &fixture
                    .specification
                    .parent_checkpoint
                    .join("model.safetensors")
            )?,
            original_file_sha
        );
        report_output::verify(&fixture.specification.parent_checkpoint)?;
        fs::remove_dir_all(fixture.root)?;
        Ok(())
    }

    #[test]
    fn radial_transfer_rejects_wrong_lineage_before_any_array_copy() -> Result<()> {
        let fixture = transfer_fixture(AdmissionPolicy::Full)?;
        let mut target = raw_target(&fixture.campaign)?;
        let before = parameter_manifest(&target, &fixture.campaign.model.shapes())?;
        for mismatch in 0..5 {
            let mut cfg = fixture.campaign.clone();
            let mut spec = fixture.specification.clone();
            match mismatch {
                0 => spec.parent_checkpoint_sha256 = "0".repeat(64),
                1 => spec.parent_sampled_target_visits += 8,
                2 => cfg.model.transport = crate::joint_model::Transport::HouseholderPair,
                3 => cfg.optimizer.learning_rate *= 2.0,
                4 => {
                    cfg.data_seed = 42;
                    spec.sampler = TransferSampler::FreshCounter { data_seed: 42 };
                }
                _ => unreachable!(),
            }
            cfg.shared_parameter_transfer = Some(spec.clone());
            assert!(spec
                .apply(&cfg, &fixture.evaluator_sha, &mut target)
                .is_err());
            assert_eq!(
                parameter_manifest(&target, &fixture.campaign.model.shapes())?,
                before
            );
        }
        assert!(fixture
            .specification
            .apply(&fixture.campaign, &"f".repeat(64), &mut target)
            .is_err());
        let mut dot = fixture.campaign.clone();
        select_geometry(&mut dot, ReadGeometry::Dot);
        let mut dot_target = raw_target(&dot)?;
        let dot_before = parameter_manifest(&dot_target, &dot.model.shapes())?;
        dot.read_initialization = Some(ReadInitialization::UnitScale);
        assert!(fixture
            .specification
            .apply(&dot, &fixture.evaluator_sha, &mut dot_target)
            .is_err());
        assert_eq!(
            parameter_manifest(&dot_target, &dot.model.shapes())?,
            dot_before
        );
        dot.read_initialization = None;
        // Public config mutation cannot relabel a 23-array radial model as
        // the 21-array Dot reset control and retain hidden radial parameters.
        target.config.read_geometry = ReadGeometry::Dot;
        assert!(fixture
            .specification
            .apply(&dot, &fixture.evaluator_sha, &mut target)
            .is_err());
        target.config.read_geometry = fixture.campaign.model.read_geometry;
        let mut missing_radial_initializer = fixture.campaign.clone();
        missing_radial_initializer.read_initialization = None;
        assert!(fixture
            .specification
            .apply(
                &missing_radial_initializer,
                &fixture.evaluator_sha,
                &mut target
            )
            .is_err());
        fs::write(fixture.root.join("tokenizer.json"), b"changed identity")?;
        assert!(fixture
            .specification
            .apply(&fixture.campaign, &fixture.evaluator_sha, &mut target)
            .is_err());
        assert_eq!(
            parameter_manifest(&target, &fixture.campaign.model.shapes())?,
            before
        );
        fs::remove_dir_all(fixture.root)?;
        let intervention = transfer_fixture(AdmissionPolicy::Recent64)?;
        let mut target = raw_target(&intervention.campaign)?;
        assert!(intervention
            .specification
            .apply(
                &intervention.campaign,
                &intervention.evaluator_sha,
                &mut target
            )
            .is_err());
        fs::remove_dir_all(intervention.root)?;
        Ok(())
    }

    #[test]
    fn radial_transfer_receipt_remains_historical_without_parent_or_initial_live_arrays(
    ) -> Result<()> {
        for geometry in [
            ReadGeometry::Dot,
            ReadGeometry::Lorentz,
            ReadGeometry::LorentzAffine,
        ] {
            let mut fixture = transfer_fixture(AdmissionPolicy::Full)?;
            select_geometry(&mut fixture.campaign, geometry);
            let mut target = raw_target(&fixture.campaign)?;
            let receipt = fixture.specification.apply(
                &fixture.campaign,
                &fixture.evaluator_sha,
                &mut target,
            )?;
            let binding = json!({"shared_parameter_transfer":fixture.specification,
            "transfer_receipt":receipt,"evaluator_sha256":fixture.evaluator_sha});
            // Simulate evolved values; validation must bind history, not reset or
            // require a learned tensor to equal its original transferred array.
            let evolved_names: &[&str] = if geometry == ReadGeometry::Dot {
                &["output.bias"]
            } else {
                &[LORENTZ_LOG_BETA, "output.bias"]
            };
            for &name in evolved_names {
                let variable = target
                    .variables()
                    .get(name)
                    .ok_or_else(|| invalid("test variable"))?;
                variable.set(&variable.affine(1.0, 0.25)?)?;
            }
            fs::remove_dir_all(&fixture.root)?;
            assert_eq!(
                validate_binding(&fixture.campaign, &binding)?,
                Some(receipt.clone())
            );
            let mut changed_windows = fixture.campaign.clone();
            changed_windows.batch = 1;
            changed_windows.context = 16;
            assert!(validate_binding(&changed_windows, &binding).is_err());
            changed_windows.training_window_transition =
                Some(crate::joint_campaign::TrainingWindowTransition {
                    parent_checkpoint_sha256: "e".repeat(64),
                    parent_campaign_sha256: "f".repeat(64),
                    reason: "Declared synthetic continuation at equal targets per update".into(),
                    old_batch: 2,
                    old_context: 8,
                    new_batch: 1,
                    new_context: 16,
                    parent_optimizer_step: 1,
                    parent_sampled_target_visits: 16,
                });
            assert_eq!(
                validate_binding(&changed_windows, &binding)?,
                Some(receipt.clone())
            );
            changed_windows.batch = 2;
            assert!(validate_binding(&changed_windows, &binding).is_err());
            let mut corrupt = receipt.clone();
            corrupt.copied_parameters.pop();
            corrupt.shared_parameters_sha256 = manifest_digest(&corrupt.copied_parameters)?;
            assert!(fixture
                .specification
                .validate_receipt(&fixture.campaign, &corrupt)
                .is_err());
            let mut wrong = binding.clone();
            wrong["transfer_receipt"] = Value::Null;
            assert!(validate_binding(&fixture.campaign, &wrong).is_err());
            wrong = binding;
            wrong["evaluator_sha256"] = json!("d".repeat(64));
            assert!(validate_binding(&fixture.campaign, &wrong).is_err());
            let mut legacy = fixture.campaign;
            legacy.shared_parameter_transfer = None;
            assert!(validate_binding(&legacy, &json!({}))?.is_none());
            let mut invalid_policy = serde_json::to_value(fixture.specification)?;
            invalid_policy["optimizer"] = json!("restore_parent");
            assert!(serde_json::from_value::<SharedParameterTransfer>(invalid_policy).is_err());
        }
        Ok(())
    }
}
