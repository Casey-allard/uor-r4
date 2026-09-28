//! Verified continuous FullPrefix child import for an explicit development export.
//!
//! The immediate parent is the evolved checkpoint, not the nested historical
//! R1d parameter inventory. Seals establish file identity, not capability. Corpus
//! bytes were checked by the fit; this import joins its saved corpus bindings to
//! the supplied prepared manifest without rerunning the corpus or its sampler.

use crate::{
    dialogue_artifact::ParameterBinding,
    dialogue_episodes::SAMPLER_ID,
    invalid,
    joint_model::{JointConfig, JointModel},
    sha256_file, Result,
};
use candle_core::Device;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uor_r4_core::report_output;
use uor_r4_tokenizer::{dialogue::DialogueProtocol, ByteBpeTokenizer};

const SCHEMA: &str = "uor-r4.dialogue-child-artifact/1";
const SCHEDULE: &str = "uor-r4.dialogue-response-schedule-chain/1";

#[derive(Clone, Debug, Serialize)]
pub struct DialogueChildProvenance {
    pub schema: &'static str,
    pub report_root: PathBuf,
    pub report_sha256: String,
    pub report_seal_sha256: String,
    pub campaign_sha256: String,
    pub checkpoint_root: PathBuf,
    pub checkpoint_sha256: String,
    pub checkpoint_seal_sha256: String,
    pub configuration_sha256: String,
    /// SHA256 of the immediate child's actual model.safetensors file.
    pub parameter_sha256: String,
    pub parameter_fingerprint: String,
    pub parameters: Vec<ParameterBinding>,
    pub parameter_count: usize,
    pub current_model: JointConfig,
    pub admission: &'static str,
    pub read_geometry: &'static str,
    pub policy: &'static str,
    pub steps_completed: usize,
    pub supervised_target_visits: u64,
    pub train_index_sha256: String,
    pub executed_schedule_sha256: String,
    pub sampler: &'static str,
    pub schedule_chain: &'static str,
    pub learning_contract: Value,
    pub training_source_commit: String,
    pub training_executable_sha256: String,
    pub prepared_manifest_sha256: String,
    pub tokenizer_sha256: String,
    pub protocol: DialogueProtocol,
    pub protocol_identity: String,
    /// Historical R1d receipt retained verbatim, including its evidence limits.
    pub ancestor: Value,
    pub scope: &'static str,
    // Only this loader can construct a verified receipt. Public diagnostic
    // fields may be edited by callers, but export detects any such mutation.
    #[serde(skip)]
    verified_digest: String,
}
impl DialogueChildProvenance {
    fn digest(&self) -> Result<String> {
        Ok(hex::encode(Sha256::digest(serde_json::to_vec(self)?)))
    }
    pub(crate) fn validate_verified(&self) -> Result<()> {
        if self.digest()? != self.verified_digest {
            return Err(invalid("dialogue child verified provenance was changed"));
        }
        Ok(())
    }
    pub(crate) fn validate_parameters(&self, model: &JointModel) -> Result<()> {
        self.validate_verified()?;
        let (fingerprint, bindings) = parameter_bindings(model)?;
        if model.config != self.current_model
            || fingerprint != self.parameter_fingerprint
            || serde_json::to_value(&bindings)? != serde_json::to_value(&self.parameters)?
        {
            return Err(invalid(
                "dialogue child parameters differ from verified checkpoint",
            ));
        }
        Ok(())
    }
}

pub struct DialogueChildArtifact {
    model: JointModel,
    tokenizer: ByteBpeTokenizer,
    provenance: DialogueChildProvenance,
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn hash(value: &Value, key: &str, length: usize) -> Result<String> {
    let text = value[key]
        .as_str()
        .ok_or_else(|| invalid(format!("missing child {key}")))?;
    if text.len() != length || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid(format!("invalid child {key} identity")));
    }
    Ok(text.into())
}
fn count(value: &Value, key: &str) -> Result<u64> {
    value[key]
        .as_u64()
        .ok_or_else(|| invalid(format!("missing child {key} counter")))
}
fn learning_contract(campaign: &Value) -> Result<Value> {
    // Parsing the existing typed campaign prevents extra undeclared fields.
    let typed: crate::dialogue_learning::Campaign = serde_json::from_value(campaign.clone())?;
    let mut contract = serde_json::to_value(typed)?;
    let object = contract
        .as_object_mut()
        .ok_or_else(|| invalid("child campaign object"))?;
    for key in ["resume_from", "max_process_seconds", "stop_file"] {
        object.remove(key);
    }
    Ok(contract)
}

fn validate_records(
    result: &Value,
    campaign: &Value,
    checkpoint: &Value,
    config: &Value,
    execution: &Value,
) -> Result<JointConfig> {
    let model: JointConfig = serde_json::from_value(config["model"].clone())?;
    crate::dialogue_artifact::validate_model_config(&model)?;
    let steps = count(result, "completed_step")?;
    let starting = count(result, "starting_step")?;
    let updates = count(result, "new_optimizer_updates")?;
    let targets = count(result, "cumulative_supervised_target_visits")?;
    let batch = count(campaign, "batch")?;
    if result["schema"] != "uor-r4.dialogue-prefix-fit/1"
        || result["status"] != "TARGET_COMPLETE"
        || campaign["schema"] != "uor-r4.dialogue-prefix-campaign/1"
        || campaign["policy"] != "full_prefix"
        || result["campaign"] != *campaign
        || checkpoint["schema"] != "uor-r4.dialogue-prefix-checkpoint/1"
        || checkpoint["learning_contract"] != learning_contract(campaign)?
        || steps == 0
        || count(campaign, "total_steps")? != steps
        || starting.checked_add(updates) != Some(steps)
        || !(1..=64).contains(&batch)
        || updates.checked_mul(batch).and_then(|v| v.checked_mul(256))
            != Some(count(result, "new_tensor_positions")?)
        || count(checkpoint, "next_data_step")? != steps
        || count(checkpoint, "optimizer_step")? != steps
        || count(&checkpoint["optimizer"], "step")? != steps
        || count(&result["restored_optimizer"], "step")? != steps
        || result["restored_optimizer"] != checkpoint["optimizer_fingerprint"]
        || targets == 0
        || count(checkpoint, "supervised_target_visits")? != targets
        || count(result, "new_supervised_target_visits")? > targets
        || (starting == 0 && count(result, "new_supervised_target_visits")? != targets)
        || result["parent"] != checkpoint["parent"]
        || checkpoint["parent"]["current_model"] != config["model"]
        || checkpoint["parent"]["schema"] != "uor-r4.offline-dialogue-artifact/1"
        || checkpoint["parent"]["admission"] != "full"
        || checkpoint["parent"]["read_geometry"] != "dot"
        || checkpoint["parent"]["parameter_sha256"] != campaign["parent_parameter_sha256"]
        || checkpoint["parent"]["prepared_manifest_sha256"] != campaign["prepared_manifest_sha256"]
        || checkpoint["parent"]["tokenizer_sha256"] != campaign["tokenizer_sha256"]
        || !config["quantization"].is_null()
        || config.get("admission").is_some_and(|v| v != "full")
        || result["sampler"] != SAMPLER_ID
        || checkpoint["sampler"] != SAMPLER_ID
        || result["schedule_chain"] != SCHEDULE
        || checkpoint["schedule_chain"] != SCHEDULE
        || result["source_commit"] != checkpoint["source_commit"]
        || result["source_commit"] != execution["source_commit"]
        || result["executable_sha256"] != execution["executable_sha256"]
        || result["final_parameter_fingerprint"] != checkpoint["parameter_fingerprint"]
        || result["train_index_sha256"] != checkpoint["train_index_sha256"]
        || result["executed_schedule_sha256"] != checkpoint["executed_schedule_sha256"]
    {
        return Err(invalid(
            "dialogue child report/checkpoint/clock/learning lineage mismatch",
        ));
    }
    for (value, key, length) in [
        (result, "source_commit", 40),
        (result, "executable_sha256", 64),
        (result, "final_parameter_fingerprint", 64),
        (result, "train_index_sha256", 64),
        (result, "executed_schedule_sha256", 64),
        (config, "weights_sha256", 64),
        (campaign, "prepared_manifest_sha256", 64),
        (campaign, "tokenizer_sha256", 64),
        (&checkpoint["parent"], "fit_seal_sha256", 64),
        (&checkpoint["parent"], "fit_report_sha256", 64),
        (&checkpoint["parent"], "parameter_sha256", 64),
    ] {
        hash(value, key, length)?;
    }
    Ok(model)
}

/// The fit already checked corpus bytes. Join the immutable recorded dataset
/// identities to the actual prepared manifest and bound tokenizer; no corpus
/// scan, episode resampling or historical optimizer/model load is performed.
fn validate_corpus(
    parent: &Value,
    prepared: &Value,
    tokenizer_sha: &str,
    protocol: &DialogueProtocol,
) -> Result<()> {
    let datasets = parent["datasets"]
        .as_array()
        .ok_or_else(|| invalid("child corpus bindings"))?;
    if datasets.len() != 2 {
        return Err(invalid("child corpus split inventory"));
    }
    for name in ["train", "heldout"] {
        let entries: Vec<_> = datasets.iter().filter(|d| d["split"] == name).collect();
        let p = &prepared[name];
        if entries.len() != 1 {
            return Err(invalid("child corpus duplicate/missing split"));
        }
        let d = entries[0];
        if p["schema"] != "uor-r4-chat-corpus/v1"
            || p["mask_schema"] != "uor-r4-response-mask/u8/v1"
            || p["split"] != name
            || p["tokenizer"]["sha256"] != tokenizer_sha
            || p["tokenizer"]["tokenizer_cid"] != protocol.tokenizer_cid
            || p["tokenizer"]["vocab_size"] != 4096
            || p["tokenizer"]["bos_id"] != 0
            || p["tokenizer"]["eos_id"] != 1
            || p["tokenizer"]["unk_id"] != 2
            || p["tokens"] != d["tokens"]
            || p["response_tokens"] != d["response_tokens"]
            || p["tokens_sha256"] != d["tokens_sha256"]
            || p["mask_sha256"] != d["mask_sha256"]
            || count(p, "response_tokens")? > count(p, "tokens")?
        {
            return Err(invalid(
                "dialogue child prepared corpus/tokenizer binding mismatch",
            ));
        }
        hash(p, "tokens_sha256", 64)?;
        hash(p, "mask_sha256", 64)?;
    }
    Ok(())
}

pub(crate) fn parameter_bindings(model: &JointModel) -> Result<(String, Vec<ParameterBinding>)> {
    let mut fingerprint = Sha256::new();
    let mut parameters = Vec::new();
    for (name, variable) in model.variables() {
        fingerprint.update((name.len() as u64).to_le_bytes());
        fingerprint.update(name.as_bytes());
        let mut hash = Sha256::new();
        for value in variable.flatten_all()?.to_vec1::<f32>()? {
            if !value.is_finite() {
                return Err(invalid("nonfinite child parameter"));
            }
            fingerprint.update(value.to_le_bytes());
            hash.update(value.to_le_bytes());
        }
        parameters.push(ParameterBinding {
            name: name.clone(),
            shape: variable.dims().to_vec(),
            sha256_le_f32: hex::encode(hash.finalize()),
        });
    }
    Ok((hex::encode(fingerprint.finalize()), parameters))
}

impl DialogueChildArtifact {
    /// Callers claim their own output root before loading. Only a sealed,
    /// completed FullPrefix continuous child is admitted; this is not resume.
    pub fn load(
        report_root: &Path,
        prepared_manifest: &Path,
        tokenizer_path: &Path,
        device: &Device,
    ) -> Result<Self> {
        report_output::verify(report_root)?;
        if report_root.join("failed-attempt.json").exists() {
            return Err(invalid("failed child fit"));
        }
        let checkpoint_root = report_root.join("checkpoint-final");
        report_output::verify(&checkpoint_root)?;
        let result = read_json(&report_root.join("result.json"))?;
        let campaign = read_json(&report_root.join("campaign.json"))?;
        let execution = read_json(&report_root.join("execution-binding.json"))?;
        let checkpoint = read_json(&checkpoint_root.join("checkpoint.json"))?;
        let config = read_json(&checkpoint_root.join("config.json"))?;
        let model_config = validate_records(&result, &campaign, &checkpoint, &config, &execution)?;
        let optimizer_path = checkpoint_root.join(crate::joint_optimizer::MOMENT_FILE);
        let optimizer_metadata = read_json(&checkpoint_root.join("optimizer.json"))?;
        if checkpoint["optimizer"]["moments_sha256"] != sha256_file(&optimizer_path)?
            || checkpoint["optimizer"]["moments_bytes"] != fs::metadata(&optimizer_path)?.len()
            || optimizer_metadata["moments_sha256"] != checkpoint["optimizer"]["moments_sha256"]
            || optimizer_metadata["moments_bytes"] != checkpoint["optimizer"]["moments_bytes"]
            || optimizer_metadata["step"] != checkpoint["optimizer_step"]
            || optimizer_metadata["config"] != campaign["optimizer"]
        {
            return Err(invalid(
                "dialogue child saved optimizer identity/clock mismatch",
            ));
        }

        // The historical absolute checkpoint locator is informational after a
        // report is relocated, but it must identify this report's final child.
        if Path::new(
            result["checkpoint"]
                .as_str()
                .ok_or_else(|| invalid("child checkpoint locator"))?,
        )
        .file_name()
            != Some(std::ffi::OsStr::new("checkpoint-final"))
        {
            return Err(invalid("child result does not select final checkpoint"));
        }
        let prepared_sha = sha256_file(prepared_manifest)?;
        let tokenizer_bytes = fs::read(tokenizer_path)?;
        let tokenizer_sha = hex::encode(Sha256::digest(&tokenizer_bytes));
        if campaign["prepared_manifest_sha256"] != prepared_sha
            || campaign["tokenizer_sha256"] != tokenizer_sha
        {
            return Err(invalid("child supplied corpus/tokenizer identity mismatch"));
        }
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
            .ok_or_else(|| invalid("invalid child tokenizer"))?;
        let ancestor = checkpoint["parent"].clone();
        let protocol =
            DialogueProtocol::from_json(&serde_json::to_vec(&ancestor["protocol"])?, &tokenizer)
                .map_err(|e| invalid(e.to_string()))?;
        let protocol_identity = protocol.identity().map_err(|e| invalid(e.to_string()))?;
        if tokenizer.vocab_size() != 4096
            || (protocol.bos_id, protocol.eos_id, protocol.unk_id) != (0, 1, 2)
            || ancestor["protocol_identity"] != protocol_identity
        {
            return Err(invalid("child protocol identity mismatch"));
        }
        validate_corpus(
            &ancestor,
            &read_json(prepared_manifest)?,
            &tokenizer_sha,
            &protocol,
        )?;
        let model = JointModel::load_offline_dialogue_checkpoint(&checkpoint_root, device)?;
        let (fingerprint, parameters) = parameter_bindings(&model)?;
        if result["final_parameter_fingerprint"] != fingerprint || model.config != model_config {
            return Err(invalid(
                "child loaded arrays do not match recorded fingerprint",
            ));
        }
        let mut provenance = DialogueChildProvenance {
            schema:SCHEMA, report_root:report_root.into(), report_sha256:sha256_file(&report_root.join("result.json"))?,
            report_seal_sha256:sha256_file(&report_root.join("manifest.json"))?, campaign_sha256:sha256_file(&report_root.join("campaign.json"))?,
            checkpoint_root:checkpoint_root.clone(), checkpoint_sha256:sha256_file(&checkpoint_root.join("checkpoint.json"))?,
            checkpoint_seal_sha256:sha256_file(&checkpoint_root.join("manifest.json"))?, configuration_sha256:sha256_file(&checkpoint_root.join("config.json"))?,
            parameter_sha256:hash(&config,"weights_sha256",64)?, parameter_fingerprint:fingerprint,
            parameters, parameter_count:model.parameter_count(), current_model:model_config, admission:"full",read_geometry:"dot",policy:"full_prefix",
            steps_completed:usize::try_from(count(&result,"completed_step")?).map_err(|_| invalid("child clock overflow"))?,
            supervised_target_visits:count(&result,"cumulative_supervised_target_visits")?,
            train_index_sha256:hash(&result,"train_index_sha256",64)?, executed_schedule_sha256:hash(&result,"executed_schedule_sha256",64)?,
            sampler:SAMPLER_ID, schedule_chain:SCHEDULE, learning_contract:checkpoint["learning_contract"].clone(),
            training_source_commit:hash(&result,"source_commit",40)?, training_executable_sha256:hash(&result,"executable_sha256",64)?,
            prepared_manifest_sha256:prepared_sha, tokenizer_sha256:tokenizer_sha,protocol,protocol_identity,ancestor,
            scope:"Verified immediate continuous FullPrefix checkpoint and saved training lineage. Local child clocks exclude historical ancestor updates. Corpus identities join the sealed fit to the prepared manifest; no corpus rescan or training replay. Historical retrospective bindings remain in ancestor. No capability or conversion-quality claim.",
            verified_digest:String::new(),
        };
        provenance.verified_digest = provenance.digest()?;
        Ok(Self {
            model,
            tokenizer,
            provenance,
        })
    }
    pub fn model(&self) -> &JointModel {
        &self.model
    }
    pub fn tokenizer(&self) -> &ByteBpeTokenizer {
        &self.tokenizer
    }
    pub fn protocol(&self) -> &DialogueProtocol {
        &self.provenance.protocol
    }
    pub fn provenance(&self) -> &DialogueChildProvenance {
        &self.provenance
    }
    pub(crate) fn into_export_parts(self) -> Result<(JointModel, DialogueChildProvenance)> {
        self.provenance.validate_parameters(&self.model)?;
        Ok((self.model, self.provenance))
    }
    pub(crate) fn into_rounding_parts(
        self,
    ) -> Result<(JointModel, ByteBpeTokenizer, DialogueChildProvenance)> {
        self.provenance.validate_parameters(&self.model)?;
        Ok((self.model, self.tokenizer, self.provenance))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn save(path: &Path, value: &Value) -> Result<()> {
        fs::write(path, serde_json::to_vec_pretty(value)?)?;
        Ok(())
    }
    pub(crate) fn fixture() -> Result<(PathBuf, PathBuf, PathBuf, PathBuf)> {
        fixture_values(false)
    }
    pub(crate) fn fixture_with_fractions() -> Result<(PathBuf, PathBuf, PathBuf, PathBuf)> {
        fixture_values(true)
    }
    fn fixture_values(fractions: bool) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf)> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("dialogue-child-{}-{nonce}", std::process::id()));
        fs::create_dir(&root)?;
        let report = root.join("fit");
        report_output::claim(&report)?;
        let cp = report.join("checkpoint-final");
        report_output::claim(&cp)?;
        let config = JointConfig {
            width: 576,
            ..Default::default()
        };
        let parameters = config
            .shapes()
            .into_iter()
            .map(|(name, shape)| {
                let mut values = vec![0f32; shape.iter().product()];
                values[0] = 0.375;
                if fractions && values.len() > 1 {
                    values[1] = 0.12345;
                }
                (name, values)
            })
            .collect();
        let model =
            JointModel::from_offline_dialogue_parameters(config.clone(), parameters, &Device::Cpu)?;
        model.save(&cp)?;
        let (fingerprint, _) = parameter_bindings(&model)?;
        let mut vocab = serde_json::Map::new();
        for id in 0..4096u32 {
            let token = match id {
                0 => "<|bos|>".into(),
                1 => "<|eos|>".into(),
                2 => "<|unk|>".into(),
                _ => format!("v{id}"),
            };
            vocab.insert(token, json!(id));
        }
        let tokenizer_path = root.join("tokenizer.json");
        save(
            &tokenizer_path,
            &json!({"model":{"type":"BPE","vocab":vocab,"merges":[]},
            "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
            "added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}),
        )?;
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
            .ok_or_else(|| invalid("child fixture tokenizer"))?;
        let protocol =
            DialogueProtocol::literal_roles_v1(&tokenizer).map_err(|e| invalid(e.to_string()))?;
        let tokenizer_sha = sha256_file(&tokenizer_path)?;
        let h = "a".repeat(64);
        let source = "b".repeat(40);
        let mut prepared = json!({});
        let mut datasets = vec![];
        for name in ["train", "heldout"] {
            prepared[name] = json!({"schema":"uor-r4-chat-corpus/v1","mask_schema":"uor-r4-response-mask/u8/v1",
                "split":name,"tokens":10,"response_tokens":5,"tokens_sha256":h,"mask_sha256":h,
                "tokenizer":{"sha256":tokenizer_sha,"tokenizer_cid":protocol.tokenizer_cid,
                    "vocab_size":4096,"bos_id":0,"eos_id":1,"unk_id":2}});
            datasets.push(json!({"split":name,"tokens":10,"response_tokens":5,"tokens_sha256":h,"mask_sha256":h}));
        }
        let prepared_path = root.join("prepared.json");
        save(&prepared_path, &prepared)?;
        let prepared_sha = sha256_file(&prepared_path)?;
        let ancestor = json!({"schema":"uor-r4.offline-dialogue-artifact/1","current_model":config,
            "admission":"full","read_geometry":"dot","fit_seal_sha256":h,"fit_report_sha256":h,"parameter_sha256":h,
            "prepared_manifest_sha256":prepared_sha,"tokenizer_sha256":tokenizer_sha,"protocol":protocol,
            "protocol_identity":protocol.identity().map_err(|e|invalid(e.to_string()))?,"datasets":datasets,"steps_completed":2237});
        let campaign = json!({"schema":"uor-r4.dialogue-prefix-campaign/1","parent_fit_root":"historical",
            "prepared_manifest":prepared_path,"tokenizer_path":tokenizer_path,"parent_parameter_sha256":h,
            "prepared_manifest_sha256":prepared_sha,"tokenizer_sha256":tokenizer_sha,"policy":"full_prefix",
            "data_seed":7,"total_steps":2,"batch":1,"cpu_gradient_shards":1,
            "expected_train_response_runs":10,"expected_eligible_train_responses":2,
            "optimizer":{"learning_rate":0.0001,"weight_decay":0.01,"parameter_abs_limit":1000000.0,"allowed_missing_gradients":[]},
            "development_seed":8,"development_episodes":1,"development_batch":1,"development_every_steps":0,"checkpoint_every_steps":0,
            "max_process_seconds":60.0,"stop_file":null,"resume_from":null,"requests_path":"requests.json","requests_sha256":h,"max_new_tokens":1});
        // This fixture tests byte identity and recorded clocks, not Adam resume.
        let moment_bytes = b"opaque optimizer fixture";
        fs::write(cp.join(crate::joint_optimizer::MOMENT_FILE), moment_bytes)?;
        let moments_sha = hex::encode(Sha256::digest(moment_bytes));
        let optimizer = json!({"step":2,"variables":21,"moments_bytes":moment_bytes.len(),"moments_sha256":moments_sha});
        save(
            &cp.join("optimizer.json"),
            &json!({"step":2,"moments_bytes":moment_bytes.len(),"moments_sha256":moments_sha,"config":campaign["optimizer"]}),
        )?;
        let continuity = json!({"step":2});
        let checkpoint = json!({"schema":"uor-r4.dialogue-prefix-checkpoint/1","source_commit":source,
            "learning_contract":learning_contract(&campaign)?,"parent":ancestor,"train_index_sha256":h,
            "sampler":SAMPLER_ID,"schedule_chain":SCHEDULE,"next_data_step":2,"optimizer_step":2,
            "supervised_target_visits":8,"executed_schedule_sha256":h,"parameter_fingerprint":fingerprint,
            "optimizer":optimizer,"optimizer_fingerprint":continuity});
        save(&cp.join("checkpoint.json"), &checkpoint)?;
        report_output::seal(&cp)?;
        let result = json!({"schema":"uor-r4.dialogue-prefix-fit/1","status":"TARGET_COMPLETE","campaign":campaign,
            "checkpoint":cp,"completed_step":2,"starting_step":0,"new_optimizer_updates":2,
            "new_tensor_positions":512,"new_supervised_target_visits":8,"cumulative_supervised_target_visits":8,
            "restored_optimizer":continuity,"parent":ancestor,"sampler":SAMPLER_ID,"schedule_chain":SCHEDULE,
            "source_commit":source,"executable_sha256":h,"final_parameter_fingerprint":fingerprint,
            "train_index_sha256":h,"executed_schedule_sha256":h});
        save(&report.join("result.json"), &result)?;
        save(&report.join("campaign.json"), &campaign)?;
        save(
            &report.join("execution-binding.json"),
            &json!({"source_commit":source,"executable_sha256":h}),
        )?;
        report_output::seal(&report)?;
        Ok((root, report, prepared_path, tokenizer_path))
    }

    #[test]
    fn dialogue_child_export_binds_evolved_arrays_and_local_clock() -> Result<()> {
        let (root, report, prepared, tokenizer) = fixture()?;
        let artifact = DialogueChildArtifact::load(&report, &prepared, &tokenizer, &Device::Cpu)?;
        let before = parameter_bindings(artifact.model())?;
        assert_eq!(artifact.provenance().steps_completed, 2);
        assert_eq!(artifact.provenance().ancestor["steps_completed"], 2237);
        assert_ne!(
            artifact.provenance().parameter_sha256,
            artifact.provenance().ancestor["parameter_sha256"]
        );
        let (mut calibrated, parent) =
            JointModel::calibrated_dialogue_child_integer_parent(artifact)?;
        assert_eq!(
            serde_json::to_value(parameter_bindings(&calibrated)?)?,
            serde_json::to_value(before)?
        );
        let q = calibrated
            .quantization()
            .ok_or_else(|| invalid("fixture calibration"))?;
        assert_eq!((q.start_step, q.completed_step, q.ramp_steps), (2, 2, 1));
        assert!(calibrated.set_completed_step(3).is_err());
        let output = root.join("packed");
        report_output::claim(&output)?;
        let source = "c".repeat(40);
        let hash = "d".repeat(64);
        let sources = BTreeMap::from([("converter.rs".into(), hash.clone())]);
        let hard = calibrated
            .save_dialogue_child_integer_hard(&output, &parent, &source, &hash, &sources)?;
        assert_eq!(
            hard["conversion_provenance"]["schema"],
            "uor-r4.native-dialogue576-child-conversion/1"
        );
        assert_eq!(
            hard["conversion_provenance"]["parent"]["parameter_sha256"],
            parent.parameter_sha256
        );
        assert_eq!(hard["conversion_provenance"]["new_model_updates"], 0);
        let untouched = root.join("must-not-exist");
        let mut changed = parent.clone();
        changed.steps_completed = 2237;
        assert!(calibrated
            .save_dialogue_child_integer_hard(&untouched, &changed, &source, &hash, &sources)
            .is_err());
        let variable = calibrated
            .variables()
            .get("copy.gate.bias")
            .ok_or_else(|| invalid("fixture variable"))?;
        variable.set(&candle_core::Tensor::zeros(
            variable.shape(),
            candle_core::DType::F32,
            &Device::Cpu,
        )?)?;
        assert!(calibrated
            .save_dialogue_child_integer_hard(&untouched, &parent, &source, &hash, &sources)
            .is_err());
        assert!(!untouched.exists());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn dialogue_child_import_rejects_lineage_clock_tokenizer_and_seal_changes() -> Result<()> {
        let (root, report, prepared, tokenizer) = fixture()?;
        let r = read_json(&report.join("result.json"))?;
        let c = read_json(&report.join("campaign.json"))?;
        let cp = read_json(&report.join("checkpoint-final/checkpoint.json"))?;
        let cfg = read_json(&report.join("checkpoint-final/config.json"))?;
        let e = read_json(&report.join("execution-binding.json"))?;
        for pointer in [
            "/optimizer_step",
            "/parameter_fingerprint",
            "/source_commit",
            "/executed_schedule_sha256",
        ] {
            let mut changed = cp.clone();
            *changed
                .pointer_mut(pointer)
                .ok_or_else(|| invalid("test pointer"))? = json!(999);
            assert!(
                validate_records(&r, &c, &changed, &cfg, &e).is_err(),
                "{pointer}"
            );
        }
        let mut role = c.clone();
        role["policy"] = json!("role_only");
        assert!(validate_records(&r, &role, &cp, &cfg, &e).is_err());
        let mut quant = cfg.clone();
        quant["quantization"] = json!({});
        assert!(validate_records(&r, &c, &cp, &quant, &e).is_err());
        let bad_tokenizer = root.join("changed-tokenizer.json");
        fs::write(&bad_tokenizer, b"{}")?;
        assert!(
            DialogueChildArtifact::load(&report, &prepared, &bad_tokenizer, &Device::Cpu).is_err()
        );
        fs::write(report.join("result.json"), b"{}")?;
        assert!(DialogueChildArtifact::load(&report, &prepared, &tokenizer, &Device::Cpu).is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
