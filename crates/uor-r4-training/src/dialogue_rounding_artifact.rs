//! Fixed-grid preparation and portable export for a verified dialogue child.
//!
//! The nearest artifact supplies the grid, not the starting shadows. Independent
//! continuous arrays retain their interior fractions after range projection.
//! Alpha learning has a separate clock; neither zero-update converter is reused
//! to describe the learned result.

use crate::{
    dialogue_artifact::ParameterBinding,
    dialogue_child_artifact::{parameter_bindings, DialogueChildArtifact, DialogueChildProvenance},
    invalid,
    joint_model::JointModel,
    joint_quantization::{ProjectionStatistics, HARD_PARAMETERS_FILE},
    joint_rounding::{LearnedRounding, RoundingConfig},
    sha256_file, Result,
};
use candle_core::{DType, Device, Tensor};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use uor_r4_core::report_output;
use uor_r4_integer::config::ServingProfile;
use uor_r4_tokenizer::ByteBpeTokenizer;

pub const PREPARATION_SCHEMA: &str = "uor-r4.dialogue-rounding-preparation/1";
pub const LEARNED_SCHEMA: &str = "uor-r4.native-dialogue576-child-rounding/1";

#[derive(Clone, Debug, Serialize)]
pub struct DialogueRoundingPreparation {
    pub schema: &'static str,
    pub child_parameter_sha256: String,
    pub child_parameter_fingerprint: String,
    pub child_steps_completed: usize,
    pub nearest_root: PathBuf,
    pub nearest_manifest_sha256: String,
    pub nearest_seal_sha256: String,
    pub nearest_payload_sha256: String,
    pub frozen_spec_sha256: String,
    pub nearest_conversion: Value,
    pub projection: ProjectionStatistics,
    pub projected_parameter_fingerprint: String,
    pub projected_parameters: Vec<ParameterBinding>,
    pub initial_hard_arrays_equal_nearest: bool,
    pub scope: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use candle_core::Var;
    use std::collections::HashMap;

    fn fixture() -> Result<(PathBuf, PathBuf, PathBuf, PathBuf, PathBuf)> {
        let (root, child, prepared, tokenizer) =
            crate::dialogue_child_artifact::tests::fixture_with_fractions()?;
        let artifact = DialogueChildArtifact::load(&child, &prepared, &tokenizer, &Device::Cpu)?;
        let (model, provenance) = JointModel::calibrated_dialogue_child_integer_parent(artifact)?;
        let nearest = root.join("nearest");
        report_output::claim(&nearest)?;
        model.save_dialogue_child_integer_hard(
            &nearest,
            &provenance,
            &"b".repeat(40),
            &"c".repeat(64),
            &BTreeMap::from([("fixture.rs".into(), "d".repeat(64))]),
        )?;
        report_output::seal(&nearest)?;
        Ok((root, child, prepared, tokenizer, nearest))
    }
    fn recipe() -> RoundingConfig {
        RoundingConfig {
            steps: 1,
            warmup_steps: 0,
            beta_start: 2.0,
            beta_end: 2.0,
            regularization: 0.01,
        }
    }
    fn save(path: &Path, value: &Value) -> Result<()> {
        fs::write(path, serde_json::to_vec_pretty(value)?)?;
        Ok(())
    }
    fn checkpoint(
        parent: &DialogueRoundingParent,
        learner: &LearnedRounding,
        root: &Path,
    ) -> Result<()> {
        report_output::claim(root)?;
        let h = "a".repeat(64);
        let p = parent.child();
        let campaign_value = json!({"schema":"uor-r4.dialogue-rounding-campaign/1",
            "child_report_root":p.report_root,"prepared_manifest":"prepared.json","tokenizer_path":"tokenizer.json",
            "nearest_packed_root":parent.provenance().nearest_root,
            "child_parameter_sha256":p.parameter_sha256,"prepared_manifest_sha256":p.prepared_manifest_sha256,
            "tokenizer_sha256":p.tokenizer_sha256,"nearest_hard_manifest_sha256":parent.provenance().nearest_manifest_sha256,
            "train_index_sha256":p.train_index_sha256,"rounding":learner.config(),
            "optimizer":{"learning_rate":0.01,"weight_decay":0.0,"parameter_abs_limit":30.0,"allowed_missing_gradients":[]},
            "data_seed":p.learning_contract["data_seed"],"data_start_step":p.steps_completed,
            "batch":1,"cpu_gradient_shards":1,"checkpoint_steps":[],
            "max_process_seconds":60.0,"stop_file":null,"resume_from":null});
        let campaign: crate::dialogue_rounding::Campaign =
            serde_json::from_value(campaign_value.clone())?;
        campaign.validate()?;
        save(&root.join("campaign.json"), &campaign_value)?;
        let variables = learner
            .variables()
            .iter()
            .map(|(name, var)| (name.clone(), var.detach()))
            .collect::<HashMap<_, _>>();
        candle_core::safetensors::save(&variables, root.join("rounding-variables.safetensors"))?;
        // This fixture proves alpha/export byte joins; optimizer continuation is
        // tested by the campaign. Export intentionally does not decode moments.
        let moments = b"opaque moment fixture, not a resumable optimizer";
        fs::write(root.join(crate::joint_optimizer::MOMENT_FILE), moments)?;
        let moment_sha = sha256_file(&root.join(crate::joint_optimizer::MOMENT_FILE))?;
        let optimizer = json!({"step":1,"moments_bytes":moments.len(),"moments_sha256":moment_sha});
        save(
            &root.join("optimizer.json"),
            &json!({"step":1,"moments_bytes":moments.len(),
            "moments_sha256":moment_sha,"config":campaign_value["optimizer"]}),
        )?;
        save(
            &root.join("checkpoint.json"),
            &json!({"schema":"uor-r4.dialogue-rounding-checkpoint/1",
            "preparation":parent.provenance(),"campaign_sha256":sha256_file(&root.join("campaign.json"))?,
            "learning_contract":campaign.learning_contract()?,"completed_updates":1,"next_data_step":p.steps_completed+1,
            "supervised_target_visits":3,"padded_position_visits":256,"executed_schedule_sha256":h,
            "variables_sha256":sha256_file(&root.join("rounding-variables.safetensors"))?,
            "optimizer":optimizer,"optimizer_fingerprint":{"step":1},
            "source_commit":"b".repeat(40),"executable_sha256":h}),
        )?;
        report_output::seal(root)?;
        Ok(())
    }

    #[test]
    fn dialogue_rounding_artifact_keeps_fractions_and_independent_parent() -> Result<()> {
        let (root, child, prepared, tokenizer, nearest) = fixture()?;
        let untouched = DialogueChildArtifact::load(&child, &prepared, &tokenizer, &Device::Cpu)?;
        let before = parameter_bindings(untouched.model())?.0;
        let parent =
            DialogueRoundingParent::load(&child, &prepared, &tokenizer, &nearest, &Device::Cpu)?;
        let learner = parent.new_learner(recipe())?;
        let raw = parent.model().variables()["embedding.weight"]
            .flatten_all()?
            .to_vec1::<f32>()?;
        let hard = learner.parameters(true)?["embedding.weight"]
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(raw[1], 0.12345);
        assert_ne!(raw[1], hard[1]);
        assert_eq!(
            parent.provenance().nearest_payload_sha256,
            sha256_file(&nearest.join(HARD_PARAMETERS_FILE))?
        );
        assert_eq!(
            parent
                .model()
                .quantization()
                .ok_or_else(|| invalid("test quantization"))?
                .start_step,
            2
        );
        let v: &Var = &parent.model().variables()["embedding.weight"];
        v.set(&Tensor::zeros(v.shape(), DType::F32, &Device::Cpu)?)?;
        assert!(parent.new_learner(recipe()).is_err());
        assert_eq!(parameter_bindings(untouched.model())?.0, before);
        assert_eq!(
            parameter_bindings(&JointModel::load_offline_dialogue_checkpoint(
                &child.join("checkpoint-final"),
                &Device::Cpu
            )?)?
            .0,
            before
        );
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn dialogue_rounding_artifact_exports_saved_alpha_and_rejects_relabeling() -> Result<()> {
        let (root, child, prepared, tokenizer, nearest) = fixture()?;
        let parent =
            DialogueRoundingParent::load(&child, &prepared, &tokenizer, &nearest, &Device::Cpu)?;
        let learner = parent.new_learner(recipe())?;
        let alpha = &learner.variables()["embedding.weight"];
        alpha.set(&Tensor::full(-30f32, alpha.shape(), &Device::Cpu)?)?;
        let cp = root.join("alpha-checkpoint");
        checkpoint(&parent, &learner, &cp)?;
        let out = root.join("learned");
        report_output::claim(&out)?;
        let source = "b".repeat(40);
        let hash = "c".repeat(64);
        let sources = BTreeMap::from([("fixture.rs".into(), hash.clone())]);
        let manifest = parent.save_learned_hard(&out, &learner, &cp, &source, &hash, &sources)?;
        assert_eq!(
            manifest["conversion_provenance"]["rounding_stage"]["completed_updates"],
            1
        );
        assert_eq!(manifest["quantization"]["completed_step"], 2);
        assert_eq!(
            manifest["conversion_provenance"]["hard_payload_sha256"],
            sha256_file(&out.join(HARD_PARAMETERS_FILE))?
        );
        report_output::seal(&out)?;
        let reloaded = parent.load_scoring_hard(&out)?;
        assert_eq!(
            manifest["conversion_provenance"]["learned_parameter_fingerprint"],
            parameter_bindings(&reloaded)?.0
        );
        let never = root.join("never-written");
        assert!(reloaded
            .save_dialogue_child_integer_hard(&never, parent.child(), &source, &hash, &sources)
            .is_err());
        alpha.set(&Tensor::full(30f32, alpha.shape(), &Device::Cpu)?)?;
        assert!(parent
            .save_learned_hard(&never, &learner, &cp, &source, &hash, &sources)
            .is_err());
        assert!(!never.exists());
        fs::write(cp.join("checkpoint.json"), b"{}")?;
        assert!(parent
            .save_learned_hard(&never, &learner, &cp, &source, &hash, &sources)
            .is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }
}

pub struct DialogueRoundingParent {
    model: JointModel,
    tokenizer: ByteBpeTokenizer,
    child: DialogueChildProvenance,
    provenance: DialogueRoundingPreparation,
    preparation_sha256: String,
}

pub(crate) fn value_sha256(value: &impl Serialize) -> Result<String> {
    // Canonical Value object ordering also permits portable validation by the
    // integer crate without depending on this offline Rust struct layout.
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(
        &serde_json::to_value(value)?,
    )?)))
}
fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn valid_hash(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn portable_child(mut value: Value) -> Value {
    if let Some(object) = value.as_object_mut() {
        object.remove("report_root");
        object.remove("checkpoint_root");
    }
    value
}

fn require_initial_hard_equality(model: &JointModel, nearest: &JointModel) -> Result<()> {
    let spec = &model
        .quantization()
        .ok_or_else(|| invalid("missing fixed grids"))?
        .spec;
    if model.config != nearest.config || model.variables().keys().ne(nearest.variables().keys()) {
        return Err(invalid("rounding nearest model inventory differs"));
    }
    for (name, variable) in model.variables() {
        let hard = spec.parameter(name, variable.as_tensor(), 1.0, false)?;
        let a = hard.flatten_all()?.to_vec1::<f32>()?;
        let b = nearest.variables()[name].flatten_all()?.to_vec1::<f32>()?;
        // Each frozen dyadic grid is injective over legal codes. Numeric zero
        // equality deliberately treats +/-0 as the same serialized zero code.
        if hard.dims() != nearest.variables()[name].dims()
            || a.len() != b.len()
            || a.iter().zip(&b).any(|(&x, &y)| !x.is_finite() || x != y)
        {
            return Err(invalid(format!(
                "initial hard codes differ from nearest: {name}"
            )));
        }
    }
    Ok(())
}

impl DialogueRoundingParent {
    /// Caller claims its output root first. No model forward or calibration is
    /// performed; all original model/corpus/checkpoint files remain immutable.
    pub fn load(
        child_root: &Path,
        prepared_manifest: &Path,
        tokenizer_path: &Path,
        nearest_root: &Path,
        device: &Device,
    ) -> Result<Self> {
        let artifact =
            DialogueChildArtifact::load(child_root, prepared_manifest, tokenizer_path, device)?;
        let (mut model, tokenizer, child) = artifact.into_rounding_parts()?;
        report_output::verify(nearest_root)?;
        if nearest_root.join("failed-attempt.json").exists() {
            return Err(invalid("failed nearest dialogue artifact"));
        }
        let manifest = read_json(&nearest_root.join("hard-model.json"))?;
        let conversion = &manifest["conversion_provenance"];
        if conversion["schema"] != "uor-r4.native-dialogue576-child-conversion/1"
            || portable_child(conversion["parent"].clone())
                != portable_child(serde_json::to_value(&child)?)
            || conversion["tokenizer_sha256"] != child.tokenizer_sha256
            || conversion["protocol_identity"] != child.protocol_identity
            || [
                "new_forward_calls",
                "new_generation_calls",
                "new_optimizer_updates",
                "new_model_updates",
            ]
            .iter()
            .any(|key| conversion[*key] != 0)
        {
            return Err(invalid(
                "nearest artifact is not the unchanged verified dialogue child",
            ));
        }
        let nearest =
            JointModel::load_hard_for_profile(nearest_root, device, ServingProfile::Dialogue576)?;
        let state = nearest
            .quantization()
            .ok_or_else(|| invalid("nearest frozen state missing"))?;
        model.configure_frozen_dialogue_rounding(state, child.steps_completed)?;
        require_initial_hard_equality(&model, &nearest)?;
        let projection = state.spec.project_parameters(model.variables())?;
        require_initial_hard_equality(&model, &nearest)?;
        let (fingerprint, parameters) = parameter_bindings(&model)?;
        let provenance = DialogueRoundingPreparation {
            schema: PREPARATION_SCHEMA,
            child_parameter_sha256: child.parameter_sha256.clone(),
            child_parameter_fingerprint: child.parameter_fingerprint.clone(),
            child_steps_completed: child.steps_completed,
            nearest_root: nearest_root.into(),
            nearest_manifest_sha256: sha256_file(&nearest_root.join("hard-model.json"))?,
            nearest_seal_sha256: sha256_file(&nearest_root.join("manifest.json"))?,
            nearest_payload_sha256: sha256_file(&nearest_root.join(HARD_PARAMETERS_FILE))?,
            frozen_spec_sha256: value_sha256(&state.spec)?,
            nearest_conversion: conversion.clone(),
            projection,
            projected_parameter_fingerprint: fingerprint,
            projected_parameters: parameters,
            initial_hard_arrays_equal_nearest: true,
            scope: "Independent continuous-child fractions projected only to the existing nearest grid range; all initial legal hard codes equal the retained nearest artifact. No new calibration, model forward, optimizer update or language claim.",
        };
        let preparation_sha256 = value_sha256(&provenance)?;
        Ok(Self {
            model,
            tokenizer,
            child,
            provenance,
            preparation_sha256,
        })
    }
    pub fn model(&self) -> &JointModel {
        &self.model
    }
    pub fn tokenizer(&self) -> &ByteBpeTokenizer {
        &self.tokenizer
    }
    pub fn child(&self) -> &DialogueChildProvenance {
        &self.child
    }
    pub fn provenance(&self) -> &DialogueRoundingPreparation {
        &self.provenance
    }
    pub fn preparation_sha256(&self) -> &str {
        &self.preparation_sha256
    }

    /// Load either this exact nearest control or its typed learned-code child
    /// for the existing offline scorer. Serving/evaluation never needs the
    /// original alpha checkpoint or its optimizer moments.
    pub fn load_scoring_hard(&self, root: &Path) -> Result<JointModel> {
        self.validate_unchanged()?;
        report_output::verify(root)?;
        if root.join("failed-attempt.json").exists() {
            return Err(invalid("failed dialogue code artifact"));
        }
        let manifest = read_json(&root.join("hard-model.json"))?;
        uor_r4_integer::bundle::validate_dialogue_conversion_schema(
            &manifest,
            &json!(self.child.tokenizer_sha256),
            &self.tokenizer,
        )?;
        let p = &manifest["conversion_provenance"];
        let learned = p["schema"] == LEARNED_SCHEMA;
        if learned {
            if portable_child(p["parent"].clone())
                != portable_child(serde_json::to_value(&self.child)?)
                || p["preparation"] != serde_json::to_value(&self.provenance)?
                || p["preparation_sha256"] != self.preparation_sha256
            {
                return Err(invalid("learned dialogue scoring parent differs"));
            }
        } else if p["schema"] != "uor-r4.native-dialogue576-child-conversion/1"
            || sha256_file(&root.join("hard-model.json"))?
                != self.provenance.nearest_manifest_sha256
        {
            return Err(invalid(
                "dialogue scoring artifact is not the exact nearest control",
            ));
        }
        let device = self
            .model
            .variables()
            .values()
            .next()
            .ok_or_else(|| invalid("parent variables missing"))?
            .device();
        let model = JointModel::load_hard_for_profile(root, device, ServingProfile::Dialogue576)?;
        if learned {
            let (fingerprint, parameters) = parameter_bindings(&model)?;
            if p["learned_parameter_fingerprint"] != fingerprint
                || p["learned_parameters"] != serde_json::to_value(parameters)?
            {
                return Err(invalid("learned decoded parameter bindings differ"));
            }
        }
        Ok(model)
    }

    /// Bind initialization to the actual alpha hard-choice operator as well as
    /// the parameter quantizer. No forward or optimization is performed.
    pub fn new_learner(&self, config: RoundingConfig) -> Result<LearnedRounding> {
        self.validate_unchanged()?;
        let spec = &self
            .model
            .quantization()
            .ok_or_else(|| invalid("missing grids"))?
            .spec;
        let learner = LearnedRounding::new(spec, self.model.variables(), config)?;
        let actual = learner.parameters(true)?;
        for (name, variable) in self.model.variables() {
            let expected = spec.parameter(name, variable.as_tensor(), 1.0, false)?;
            let a = actual
                .get(name)
                .ok_or_else(|| invalid("missing initial alpha hard parameter"))?;
            if a.dims() != expected.dims()
                || a.flatten_all()?.to_vec1::<f32>()? != expected.flatten_all()?.to_vec1::<f32>()?
            {
                return Err(invalid(format!(
                    "initial alpha hard choice differs from nearest: {name}"
                )));
            }
        }
        Ok(learner)
    }

    fn validate_unchanged(&self) -> Result<()> {
        self.child.validate_verified()?;
        let state = self
            .model
            .quantization()
            .ok_or_else(|| invalid("rounding grids missing"))?;
        if value_sha256(&self.provenance)? != self.preparation_sha256
            || value_sha256(&state.spec)? != self.provenance.frozen_spec_sha256
            || parameter_bindings(&self.model)?.0 != self.provenance.projected_parameter_fingerprint
            || state.start_step != self.child.steps_completed
            || state.completed_step != self.child.steps_completed
        {
            return Err(invalid(
                "rounding parent, grids or preparation were changed",
            ));
        }
        Ok(())
    }

    fn materialize(&self, learner: &LearnedRounding) -> Result<JointModel> {
        self.validate_unchanged()?;
        // Canonicalize negative zero before hashing the values that the codec
        // actually decodes. This is not a change of legal integer code.
        let parameters = learner
            .parameters(true)?
            .into_iter()
            .map(|(name, tensor)| {
                let mut values = tensor.flatten_all()?.to_vec1::<f32>()?;
                for value in &mut values {
                    if *value == 0.0 {
                        *value = 0.0;
                    }
                }
                Ok((
                    name,
                    Tensor::from_vec(values, tensor.dims(), tensor.device())?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        self.model.materialize_rounding_codes(parameters)
    }

    /// Export only the terminal, sealed alpha checkpoint. The saved alpha file
    /// is reloaded over this verified parent's exact fractions and grids; the
    /// caller's learner must match it but cannot supply arbitrary lower choices.
    pub fn save_learned_hard(
        &self,
        directory: &Path,
        learner: &LearnedRounding,
        checkpoint_root: &Path,
        source_commit: &str,
        executable_sha256: &str,
        source_sha256: &BTreeMap<String, String>,
    ) -> Result<Value> {
        self.validate_unchanged()?;
        if !valid_hash(source_commit, 40)
            || !valid_hash(executable_sha256, 64)
            || source_sha256.is_empty()
            || source_sha256.values().any(|v| !valid_hash(v, 64))
        {
            return Err(invalid("learned dialogue export source binding"));
        }
        let (stage, restored) = self.load_stage(checkpoint_root, learner)?;
        let hard = self.materialize(&restored)?;
        let (fingerprint, parameters) = parameter_bindings(&hard)?;
        let provenance = json!({
            "schema":LEARNED_SCHEMA,"parent":self.child,"preparation":self.provenance,
            "preparation_sha256":self.preparation_sha256,"rounding_stage":stage,
            "learned_parameter_fingerprint":fingerprint,"learned_parameters":parameters,
            "tokenizer_sha256":self.child.tokenizer_sha256,"protocol_identity":self.child.protocol_identity,
            "conversion_source_commit":source_commit,"conversion_executable_sha256":executable_sha256,
            "source_sha256":source_sha256,"serving_profile":"dialogue576","admission":"full",
            "new_model_optimizer_updates":0,
            "scope":"Development learned legal-code artifact. Parent model clock remains the immediate child's local clock; alpha learning/exposure is recorded separately in rounding_stage. Export reloads sealed alpha tensors over the original fixed-grid fractions. No learned alpha or floating shadows are served. No language, geometry or efficiency acceptance is implied."
        });
        hard.save_hard_profile(directory, ServingProfile::Dialogue576, Some(&provenance))
    }

    fn load_stage(&self, root: &Path, live: &LearnedRounding) -> Result<(Value, LearnedRounding)> {
        report_output::verify(root)?;
        if root.join("failed-attempt.json").exists() {
            return Err(invalid("failed dialogue rounding checkpoint"));
        }
        let campaign_path = root.join("campaign.json");
        let campaign_value = read_json(&campaign_path)?;
        let campaign: crate::dialogue_rounding::Campaign =
            serde_json::from_value(campaign_value.clone())?;
        campaign.validate()?;
        let checkpoint = read_json(&root.join("checkpoint.json"))?;
        let optimizer = read_json(&root.join(crate::joint_optimizer::METADATA_FILE))?;
        let alpha_path = root.join("rounding-variables.safetensors");
        let moments_path = root.join(crate::joint_optimizer::MOMENT_FILE);
        let completed = checkpoint["completed_updates"]
            .as_u64()
            .ok_or_else(|| invalid("alpha completed clock"))?;
        let start = campaign_value["data_start_step"]
            .as_u64()
            .ok_or_else(|| invalid("alpha data start"))?;
        let batch = campaign_value["batch"]
            .as_u64()
            .ok_or_else(|| invalid("alpha batch"))?;
        let eos_visits = completed
            .checked_mul(batch)
            .ok_or_else(|| invalid("alpha EOS exposure overflow"))?;
        let positions = eos_visits
            .checked_mul(256)
            .ok_or_else(|| invalid("alpha position exposure overflow"))?;
        let targets = checkpoint["supervised_target_visits"]
            .as_u64()
            .ok_or_else(|| invalid("alpha target visits"))?;
        if checkpoint["schema"] != "uor-r4.dialogue-rounding-checkpoint/1"
            || campaign_value["schema"] != "uor-r4.dialogue-rounding-campaign/1"
            || checkpoint["preparation"] != serde_json::to_value(&self.provenance)?
            || checkpoint["campaign_sha256"] != sha256_file(&campaign_path)?
            || checkpoint["learning_contract"] != campaign.learning_contract()?
            || campaign_value["rounding"] != serde_json::to_value(live.config())?
            || completed == 0
            || campaign_value["rounding"]["steps"] != completed
            || start != self.child.steps_completed as u64
            || checkpoint["next_data_step"].as_u64() != start.checked_add(completed)
            || campaign_value["data_seed"] != self.child.learning_contract["data_seed"]
            || campaign_value["child_parameter_sha256"] != self.child.parameter_sha256
            || campaign_value["prepared_manifest_sha256"] != self.child.prepared_manifest_sha256
            || campaign_value["tokenizer_sha256"] != self.child.tokenizer_sha256
            || campaign_value["nearest_hard_manifest_sha256"]
                != self.provenance.nearest_manifest_sha256
            || campaign_value["train_index_sha256"] != self.child.train_index_sha256
            || !(1..=64).contains(&batch)
            || checkpoint["padded_position_visits"] != positions
            || targets < eos_visits
            || targets > positions
            || checkpoint["variables_sha256"] != sha256_file(&alpha_path)?
            || checkpoint["optimizer"]["moments_sha256"] != sha256_file(&moments_path)?
            || checkpoint["optimizer"]["moments_bytes"] != fs::metadata(&moments_path)?.len()
            || checkpoint["optimizer"]["step"] != completed
            || checkpoint["optimizer_fingerprint"]["step"] != completed
            || optimizer["step"] != completed
            || optimizer["config"] != campaign_value["optimizer"]
            || optimizer["moments_sha256"] != checkpoint["optimizer"]["moments_sha256"]
            || optimizer["moments_bytes"] != checkpoint["optimizer"]["moments_bytes"]
        {
            return Err(invalid(
                "dialogue alpha checkpoint parent/recipe/clock/hash mismatch",
            ));
        }
        for (key, length) in [
            ("source_commit", 40),
            ("executable_sha256", 64),
            ("executed_schedule_sha256", 64),
        ] {
            if checkpoint[key]
                .as_str()
                .is_none_or(|value| !valid_hash(value, length))
            {
                return Err(invalid("alpha checkpoint source/schedule identity"));
            }
        }
        let values = candle_core::safetensors::load(
            &alpha_path,
            self.model
                .variables()
                .values()
                .next()
                .ok_or_else(|| invalid("missing parent variables"))?
                .device(),
        )?;
        let restored = self.new_learner(live.config().clone())?;
        if values.len() != restored.variables().len()
            || live.variables().keys().ne(restored.variables().keys())
        {
            return Err(invalid("alpha checkpoint variable inventory"));
        }
        let limit = campaign_value["optimizer"]["parameter_abs_limit"]
            .as_f64()
            .ok_or_else(|| invalid("alpha parameter limit"))?;
        for (name, variable) in restored.variables() {
            let value = values
                .get(name)
                .ok_or_else(|| invalid("missing saved alpha"))?;
            if value.dtype() != DType::F32
                || value.dims() != variable.dims()
                || live.variables()[name].dims() != variable.dims()
            {
                return Err(invalid("saved alpha shape/dtype"));
            }
            let a = value.flatten_all()?.to_vec1::<f32>()?;
            let b = live.variables()[name].flatten_all()?.to_vec1::<f32>()?;
            if a.len() != b.len()
                || a.iter().zip(&b).any(|(&x, &y)| {
                    !x.is_finite() || f64::from(x.abs()) > limit || x.to_bits() != y.to_bits()
                })
            {
                return Err(invalid("live alpha differs from sealed checkpoint"));
            }
        }
        for (name, variable) in restored.variables() {
            variable.set(&values[name])?;
        }
        let stage = json!({
            "schema":"uor-r4.dialogue-rounding-stage/1","checkpoint_root":root,
            "checkpoint_sha256":sha256_file(&root.join("checkpoint.json"))?,
            "checkpoint_seal_sha256":sha256_file(&root.join("manifest.json"))?,
            "campaign_sha256":sha256_file(&campaign_path)?,
            "variables_sha256":sha256_file(&alpha_path)?,
            "optimizer_metadata_sha256":sha256_file(&root.join(crate::joint_optimizer::METADATA_FILE))?,
            "optimizer_moments_sha256":sha256_file(&moments_path)?,
            "preparation_sha256":self.preparation_sha256,
            "recipe":checkpoint["learning_contract"],"completed_updates":completed,
            "data_start_step":start,"next_data_step":checkpoint["next_data_step"],
            "supervised_target_visits":targets,"eos_target_visits":eos_visits,"padded_position_visits":positions,
            "executed_schedule_sha256":checkpoint["executed_schedule_sha256"],
            "source_commit":checkpoint["source_commit"],"executable_sha256":checkpoint["executable_sha256"],
            "optimizer_step":completed,
            "scope":"Verified sealed terminal alpha arrays, recipe and optimizer byte identities; optimizer moments are not executed or restored by export. Reconstructed code choices use the verified original projected parent and frozen grid."
        });
        Ok((stage, restored))
    }
}
