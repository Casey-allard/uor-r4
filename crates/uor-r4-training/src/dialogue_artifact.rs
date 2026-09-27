//! Read-only import of the retained R1d continuous dialogue artifact.
//!
//! The legacy report stores flat F32 parameters, not an integer bundle or an
//! optimizer checkpoint. This adapter uses the current source-compatible Dot
//! cell; it does not claim historical backend bitwise reproduction. The old
//! fit bound token hashes but not mask/tokenizer hashes. Their connection via
//! the prepared corpus manifest is explicitly retrospective.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::joint_evaluation::{self, JointGeneration};
use crate::joint_model::{JointConfig, JointModel, ReadGeometry, ReadMode, Transport};
use crate::{invalid, sha256_file, Result};

const PARAMETER_COUNT: usize = 5_429_826;
const PARAMETER_BYTES: usize = PARAMETER_COUNT * 4;
const HISTORICAL_SOURCE_LABEL: &str = "5109861c93dee0e39d3aa189c325b3a13c4ededc";
const TEMPLATE: &str = r"<|bos|> then turns joined by a single '\n' separator (placed before every turn after the first); a turn is '<marker><content>' with markers 'System: ', 'User: ', 'Assistant: '; every assistant turn ends with <|eos|>; a document-terminal <|eos|> is appended only when the final emitted turn is not assistant. Content is \r\n/\r-normalised and trimmed; interior whitespace preserved.";
const MASK_RULE: &str = "1 = each token of an assistant turn's response content and its terminating <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, and an unmasked document-terminal <|eos|>.";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyModel {
    vocab_size: usize,
    width: usize,
    read_width: usize,
    context: usize,
    transport: Transport,
    seed: u64,
}
impl LegacyModel {
    fn config(&self) -> JointConfig {
        JointConfig {
            vocab_size: self.vocab_size,
            width: self.width,
            read_width: self.read_width,
            context: self.context,
            transport: self.transport,
            seed: self.seed,
            read_geometry: ReadGeometry::Dot,
        }
    }
}

#[derive(Deserialize)]
struct LegacyCampaign {
    schema: String,
    model: LegacyModel,
    context: usize,
    batch: usize,
    total_steps: usize,
    train_tokens: PathBuf,
    train_mask: PathBuf,
    heldout_tokens: PathBuf,
    heldout_mask: PathBuf,
}
#[derive(Deserialize)]
struct FitReport {
    schema: String,
    mode: String,
    status: String,
    source_commit: String,
    executable_sha256: String,
    configuration: LegacyCampaign,
    parameter_count: usize,
    steps_completed: usize,
    sampled_targets: u64,
    train_tokens: FitTokens,
    heldout_tokens: FitTokens,
    final_parameter_artifact: ParameterArtifact,
}
#[derive(Deserialize)]
struct FitTokens {
    path: PathBuf,
    sha256: String,
    tokens: usize,
    response_tokens: usize,
}
#[derive(Deserialize)]
struct ParameterArtifact {
    file: String,
    sha256: String,
    manifest: ParameterManifest,
}
#[derive(Clone, Deserialize)]
struct ParameterManifest {
    file: String,
    coordinate_bytes: usize,
    index: Vec<ParameterIndex>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterIndex {
    name: String,
    shape: Vec<usize>,
    offset: usize,
    elements: usize,
}
#[derive(Deserialize)]
struct PreparedManifest {
    train: PreparedSplit,
    heldout: PreparedSplit,
}
#[derive(Deserialize)]
struct PreparedSplit {
    schema: String,
    mask_schema: String,
    split: String,
    template_rule: String,
    mask_rule: String,
    tokenizer: PreparedTokenizer,
    tokens: usize,
    response_tokens: usize,
    tokens_bytes: u64,
    mask_bytes: u64,
    tokens_sha256: String,
    mask_sha256: String,
}
#[derive(Deserialize)]
struct PreparedTokenizer {
    sha256: String,
    tokenizer_cid: String,
    vocab_size: usize,
    bos_id: u32,
    eos_id: u32,
    unk_id: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ParameterBinding {
    pub name: String,
    pub shape: Vec<usize>,
    pub sha256_le_f32: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct DatasetBinding {
    pub split: String,
    pub tokens_path: PathBuf,
    pub mask_path: PathBuf,
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub tokens: usize,
    pub response_tokens: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct DialogueArtifactProvenance {
    pub schema: &'static str,
    pub fit_root: PathBuf,
    pub fit_report_sha256: String,
    pub fit_seal_sha256: String,
    pub historical_source_commit: String,
    pub historical_executable_sha256: String,
    pub historical_configuration: Value,
    pub current_model: JointConfig,
    pub read_geometry: &'static str,
    pub admission: &'static str,
    pub parameter_sha256: String,
    pub parameter_count: usize,
    pub parameter_bytes: usize,
    pub parameters: Vec<ParameterBinding>,
    pub steps_completed: usize,
    pub sampled_targets: u64,
    pub prepared_manifest_sha256: String,
    pub tokenizer_sha256: String,
    pub protocol: DialogueProtocol,
    pub protocol_identity: String,
    pub datasets: Vec<DatasetBinding>,
    pub compatibility_scope: &'static str,
}

/// The public API exposes no mutable model, training, quantization or export.
pub struct LegacyDialogueArtifact {
    model: JointModel,
    tokenizer: ByteBpeTokenizer,
    generation_tokenizer: HfBpeTokenizer,
    provenance: DialogueArtifactProvenance,
}

/// Explicit ownership handoff for a new offline parameter-only continuation.
/// This contains no historical optimizer, sampler cursor or resumable clock.
pub(crate) struct DialogueTrainingParts {
    pub model: JointModel,
    pub tokenizer: ByteBpeTokenizer,
    pub generation_tokenizer: HfBpeTokenizer,
    pub provenance: DialogueArtifactProvenance,
}

#[derive(Clone, Debug, Serialize)]
pub struct PrefixComparison {
    pub positions: usize,
    pub probability_values: usize,
    pub incremental_step_calls: usize,
    pub max_absolute_probability_delta: f64,
    pub argmax_disagreements: usize,
    pub mode: ReadMode,
    pub scope: &'static str,
}

pub(crate) fn validate_model_config(config: &JointConfig) -> Result<()> {
    if config.vocab_size != 4096
        || config.width != 576
        || config.read_width != 64
        || config.context != 256
        || config.transport != Transport::Quaternion
        || config.read_geometry != ReadGeometry::Dot
    {
        return Err(invalid(
            "offline R1d import requires quaternion/Dot vocabulary4096 width576 read64 context256",
        ));
    }
    Ok(())
}

fn decode_parameters(
    config: &JointConfig,
    manifest: &ParameterManifest,
    bytes: &[u8],
) -> Result<(BTreeMap<String, Vec<f32>>, Vec<ParameterBinding>)> {
    validate_model_config(config)?;
    let shapes = config.shapes();
    if manifest.file != "parameters.f32"
        || manifest.coordinate_bytes != PARAMETER_BYTES
        || bytes.len() != PARAMETER_BYTES
        || manifest.index.len() != 21
        || shapes.len() != 21
    {
        return Err(invalid("dialogue parameter file length or inventory count"));
    }
    let mut parameters = BTreeMap::new();
    let mut bindings = Vec::with_capacity(21);
    let mut next = 0usize;
    for entry in &manifest.index {
        let shape = shapes
            .get(&entry.name)
            .ok_or_else(|| invalid("unknown dialogue parameter"))?;
        if &entry.shape != shape
            || entry.elements != shape.iter().product::<usize>()
            || entry.offset != next
            || parameters.contains_key(&entry.name)
        {
            return Err(invalid(format!(
                "dialogue parameter index/shape: {}",
                entry.name
            )));
        }
        next = next
            .checked_add(entry.elements)
            .ok_or_else(|| invalid("dialogue index overflow"))?;
        let start = entry
            .offset
            .checked_mul(4)
            .ok_or_else(|| invalid("dialogue offset overflow"))?;
        let end = next
            .checked_mul(4)
            .ok_or_else(|| invalid("dialogue extent overflow"))?;
        let raw = bytes
            .get(start..end)
            .ok_or_else(|| invalid("dialogue parameter extent"))?;
        let values: Vec<f32> = raw
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        if values.iter().any(|v| !v.is_finite()) {
            return Err(invalid(format!(
                "nonfinite dialogue parameter: {}",
                entry.name
            )));
        }
        parameters.insert(entry.name.clone(), values);
        bindings.push(ParameterBinding {
            name: entry.name.clone(),
            shape: shape.clone(),
            sha256_le_f32: hex::encode(Sha256::digest(raw)),
        });
    }
    if next != PARAMETER_COUNT || parameters.keys().ne(shapes.keys()) {
        return Err(invalid("incomplete dialogue parameter inventory"));
    }
    Ok((parameters, bindings))
}

fn bind_split(
    name: &str,
    prepared: &PreparedSplit,
    fit: &FitTokens,
    token_path: &Path,
    mask_path: &Path,
    prepared_root: &Path,
    tokenizer_sha256: &str,
    protocol: &DialogueProtocol,
) -> Result<DatasetBinding> {
    let t = &prepared.tokenizer;
    if prepared.schema != "uor-r4-chat-corpus/v1"
        || prepared.mask_schema != "uor-r4-response-mask/u8/v1"
        || prepared.split != name
        || prepared.template_rule != TEMPLATE
        || prepared.mask_rule != MASK_RULE
        || t.sha256 != tokenizer_sha256
        || t.tokenizer_cid != protocol.tokenizer_cid
        || t.vocab_size != 4096
        || (t.bos_id, t.eos_id, t.unk_id) != (0, 1, 2)
        || prepared.tokens != fit.tokens
        || prepared.response_tokens != fit.response_tokens
        || prepared.tokens_sha256 != fit.sha256
        || prepared.response_tokens > prepared.tokens
        || prepared.mask_bytes != prepared.tokens as u64
    {
        return Err(invalid(format!(
            "dialogue {name} corpus/tokenizer metadata mismatch"
        )));
    }
    if fs::canonicalize(token_path)? != fs::canonicalize(&fit.path)?
        || fs::canonicalize(token_path)?
            != fs::canonicalize(prepared_root.join(name).join("tokens.u16"))?
        || fs::canonicalize(mask_path)?
            != fs::canonicalize(prepared_root.join(name).join("response_mask.u8"))?
        || fs::metadata(token_path)?.len() != prepared.tokens_bytes
        || fs::metadata(mask_path)?.len() != prepared.mask_bytes
        || sha256_file(token_path)? != prepared.tokens_sha256
        || sha256_file(mask_path)? != prepared.mask_sha256
    {
        return Err(invalid(format!(
            "dialogue {name} dataset identity mismatch"
        )));
    }
    let reader =
        MmapCorpusReader::open(token_path).map_err(|e| invalid(format!("dialogue corpus: {e}")))?;
    if reader.vocab_size() != 4096 || reader.total_tokens() != prepared.tokens {
        return Err(invalid("dialogue corpus header mismatch"));
    }
    Ok(DatasetBinding {
        split: name.into(),
        tokens_path: token_path.into(),
        mask_path: mask_path.into(),
        tokens_sha256: prepared.tokens_sha256.clone(),
        mask_sha256: prepared.mask_sha256.clone(),
        tokens: prepared.tokens,
        response_tokens: prepared.response_tokens,
    })
}

impl LegacyDialogueArtifact {
    /// Caller claims its distinct output root before invoking this loader.
    pub fn load(
        fit_root: &Path,
        corpus_manifest: &Path,
        tokenizer_path: &Path,
        device: &Device,
    ) -> Result<Self> {
        report_output::verify(fit_root)?;
        if fit_root.join("failed-attempt.json").exists() {
            return Err(invalid("dialogue fit records a failed attempt"));
        }
        let report_bytes = fs::read(fit_root.join("dialogue-fit.json"))?;
        let report: FitReport = serde_json::from_slice(&report_bytes)?;
        let document: Value = serde_json::from_slice(&report_bytes)?;
        let cfg = &report.configuration;
        let config = cfg.model.config();
        validate_model_config(&config)?;
        if report.schema != "uor-r4.dialogue-fit-report/1"
            || report.mode != "dialogue-fit"
            || report.status != "FIT_COMPLETE"
            || report.source_commit != HISTORICAL_SOURCE_LABEL
            || report.executable_sha256.len() != 64
            || !report
                .executable_sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
            || cfg.schema != "uor-r4.dialogue-campaign/1"
            || cfg.context != 256
            || !(1..=64).contains(&cfg.batch)
            || report.steps_completed == 0
            || report.steps_completed > cfg.total_steps
            || report.parameter_count != PARAMETER_COUNT
            || (report.steps_completed as u64)
                .checked_mul(cfg.batch as u64)
                .and_then(|n| n.checked_mul(256))
                != Some(report.sampled_targets)
            || report.final_parameter_artifact.file != "parameters.f32"
            || ["scan", "quantization", "admission"].iter().any(|key| {
                document["configuration"]
                    .get(key)
                    .is_some_and(|v| !v.is_null())
            })
        {
            return Err(invalid(
                "unsupported historical dialogue fit/source/configuration",
            ));
        }
        let parameter_bytes = fs::read(fit_root.join("parameters.f32"))?;
        if hex::encode(Sha256::digest(&parameter_bytes)) != report.final_parameter_artifact.sha256 {
            return Err(invalid("dialogue parameter SHA256 mismatch"));
        }
        let (parameters, bindings) = decode_parameters(
            &config,
            &report.final_parameter_artifact.manifest,
            &parameter_bytes,
        )?;
        let tokenizer_bytes = fs::read(tokenizer_path)?;
        let tokenizer_sha256 = hex::encode(Sha256::digest(&tokenizer_bytes));
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
            .ok_or_else(|| invalid("invalid dialogue tokenizer"))?;
        let generation_tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
            .ok_or_else(|| invalid("invalid joint generation tokenizer"))?;
        let protocol =
            DialogueProtocol::literal_roles_v1(&tokenizer).map_err(|e| invalid(e.to_string()))?;
        if tokenizer.vocab_size() != 4096
            || (protocol.bos_id, protocol.eos_id, protocol.unk_id) != (0, 1, 2)
        {
            return Err(invalid("dialogue tokenizer vocabulary/special IDs"));
        }
        let prepared_bytes = fs::read(corpus_manifest)?;
        let prepared: PreparedManifest = serde_json::from_slice(&prepared_bytes)?;
        let prepared_root = corpus_manifest
            .parent()
            .ok_or_else(|| invalid("prepared manifest has no parent"))?;
        let datasets = vec![
            bind_split(
                "train",
                &prepared.train,
                &report.train_tokens,
                &cfg.train_tokens,
                &cfg.train_mask,
                prepared_root,
                &tokenizer_sha256,
                &protocol,
            )?,
            bind_split(
                "heldout",
                &prepared.heldout,
                &report.heldout_tokens,
                &cfg.heldout_tokens,
                &cfg.heldout_mask,
                prepared_root,
                &tokenizer_sha256,
                &protocol,
            )?,
        ];
        let provenance = DialogueArtifactProvenance {
            schema: "uor-r4.offline-dialogue-artifact/1", fit_root: fit_root.into(),
            fit_report_sha256: hex::encode(Sha256::digest(&report_bytes)),
            fit_seal_sha256: sha256_file(&fit_root.join("manifest.json"))?,
            historical_source_commit: report.source_commit,
            historical_executable_sha256: report.executable_sha256,
            historical_configuration: document["configuration"].clone(), current_model: config.clone(),
            read_geometry: "dot", admission: "full", parameter_sha256: report.final_parameter_artifact.sha256,
            parameter_count: PARAMETER_COUNT, parameter_bytes: PARAMETER_BYTES, parameters: bindings,
            steps_completed: report.steps_completed, sampled_targets: report.sampled_targets,
            prepared_manifest_sha256: hex::encode(Sha256::digest(&prepared_bytes)), tokenizer_sha256,
            protocol_identity: protocol.identity().map_err(|e| invalid(e.to_string()))?, protocol,
            datasets,
            compatibility_scope: "Current-source continuous Dot/Full quaternion replay of the retained R1d cell. Historical source label is not an assertion of clean-tree training or bitwise backend reproduction. Tokenizer/protocol and mask bindings are retrospective through the prepared corpus manifest; only token hashes were recorded by the fit. No optimizer resume, fit, integer export, serving or capability qualification.",
        };
        let model = JointModel::from_offline_dialogue_parameters(config, parameters, device)?;
        Ok(Self {
            model,
            tokenizer,
            generation_tokenizer,
            provenance,
        })
    }

    pub fn tokenizer(&self) -> &ByteBpeTokenizer {
        &self.tokenizer
    }
    pub fn protocol(&self) -> &DialogueProtocol {
        &self.provenance.protocol
    }
    pub fn provenance(&self) -> &DialogueArtifactProvenance {
        &self.provenance
    }

    /// Consume this verified import for a newly declared offline training run.
    /// Load separately for each arm: moving these parts neither clones shared
    /// Vars nor changes the historical provenance into a resume assertion.
    /// The caller owns fresh optimizer/data clocks and the new artifact lineage.
    pub(crate) fn into_training_parts(self) -> DialogueTrainingParts {
        DialogueTrainingParts {
            model: self.model,
            tokenizer: self.tokenizer,
            generation_tokenizer: self.generation_tokenizer,
            provenance: self.provenance,
        }
    }

    /// Exact prefix replay into a fresh continuous session. Greedy selection,
    /// EOS and the existing short-cycle stop are unchanged. Generated history
    /// must be passed as IDs by the caller, never decoded and re-encoded.
    pub fn generate_tokens(
        &self,
        ids: &[u32],
        mode: ReadMode,
        max_new_tokens: usize,
    ) -> Result<JointGeneration> {
        joint_evaluation::generate_tokens(
            &self.model,
            &self.generation_tokenizer,
            ids,
            mode,
            None,
            max_new_tokens,
        )
    }

    /// Compare current-source full forward with current-source incremental
    /// execution, on one supplied prefix (at most256). No historical backend
    /// equality or language acceptance threshold is inferred.
    pub fn compare_prefix(&self, ids: &[u32], mode: ReadMode) -> Result<PrefixComparison> {
        if ids.first() != Some(&0)
            || ids.len() > self.model.config.context
            || ids
                .iter()
                .any(|&id| id as usize >= self.model.config.vocab_size)
        {
            return Err(invalid("dialogue comparison prefix/BOS/context/vocabulary"));
        }
        let full = self
            .model
            .forward(ids, 1, ids.len(), mode, false)?
            .probabilities
            .flatten_all()?
            .to_vec1::<f32>()?;
        let mut session = self.model.new_session(1)?;
        let mut max_delta = 0.0f64;
        let mut disagreements = 0usize;
        for (position, &token) in ids.iter().enumerate() {
            let actual = self
                .model
                .step(&mut session, &[token], mode)?
                .probabilities
                .flatten_all()?
                .to_vec1::<f32>()?;
            let expected = full
                .get(position * 4096..(position + 1) * 4096)
                .ok_or_else(|| invalid("dialogue full-forward row extent"))?;
            if actual.len() != 4096 {
                return Err(invalid("dialogue incremental vocabulary"));
            }
            let a = joint_evaluation::score_probabilities(&actual, 0)?;
            let b = joint_evaluation::score_probabilities(expected, 0)?;
            disagreements += usize::from(a.predicted_token != b.predicted_token);
            for (&a, &b) in actual.iter().zip(expected) {
                max_delta = max_delta.max((f64::from(a) - f64::from(b)).abs());
            }
        }
        Ok(PrefixComparison { positions: ids.len(), probability_values: full.len(),
            incremental_step_calls: session.len(), max_absolute_probability_delta: max_delta,
            argmax_disagreements: disagreements, mode,
            scope: "Current-source full-forward versus incremental probabilities on this prefix only; no historical backend or language qualification" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn config() -> JointConfig {
        JointConfig {
            width: 576,
            seed: 20260926,
            ..JointConfig::default()
        }
    }

    fn flat_fixture() -> (ParameterManifest, Vec<u8>) {
        let mut bytes = vec![0u8; PARAMETER_BYTES];
        let mut index = Vec::new();
        let mut offset = 0usize;
        for (ordinal, (name, shape)) in config().shapes().into_iter().enumerate() {
            let elements = shape.iter().product::<usize>();
            // Distinct boundary values detect an offset interpreted as bytes,
            // name/order reassignment, or an endianness change.
            let value = ordinal as f32 + 0.25;
            bytes[offset * 4..offset * 4 + 4].copy_from_slice(&value.to_le_bytes());
            if elements > 1 {
                let end = (offset + elements - 1) * 4;
                bytes[end..end + 4].copy_from_slice(&(-value).to_le_bytes());
            }
            index.push(ParameterIndex {
                name,
                shape,
                offset,
                elements,
            });
            offset += elements;
        }
        (
            ParameterManifest {
                file: "parameters.f32".into(),
                coordinate_bytes: PARAMETER_BYTES,
                index,
            },
            bytes,
        )
    }

    fn checkpoint_fixture_root(label: &str) -> Result<PathBuf> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| invalid(error.to_string()))?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-r4-dialogue-checkpoint-{}-{label}-{nonce}",
            std::process::id()
        ));
        report_output::claim(&root)?;
        Ok(root)
    }

    #[test]
    fn dialogue_artifact_offline_checkpoint_round_trip_and_independent_arrays() -> Result<()> {
        let (manifest, bytes) = flat_fixture();
        let (parameters, _) = decode_parameters(&config(), &manifest, &bytes)?;
        let model =
            JointModel::from_offline_dialogue_parameters(config(), parameters, &Device::Cpu)?;
        let root = checkpoint_fixture_root("round-trip")?;
        model.save(&root)?;
        report_output::seal(&root)?;

        // A wide offline checkpoint does not become a canonical checkpoint.
        assert!(JointModel::load(&root, &Device::Cpu).is_err());
        let left = JointModel::load_offline_dialogue_checkpoint(&root, &Device::Cpu)?;
        let right = JointModel::load_offline_dialogue_checkpoint(&root, &Device::Cpu)?;
        assert_eq!(left.config, model.config);
        assert_eq!(left.numerical_contract(), model.numerical_contract());
        assert_eq!(left.parameter_count(), PARAMETER_COUNT);
        for (name, expected) in model.variables() {
            let expected = expected.flatten_all()?.to_vec1::<f32>()?;
            for loaded in [&left, &right] {
                let actual = loaded
                    .variables()
                    .get(name)
                    .ok_or_else(|| invalid("missing reloaded dialogue variable"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                assert!(
                    actual
                        .iter()
                        .map(|x| x.to_bits())
                        .eq(expected.iter().map(|x| x.to_bits())),
                    "{name}"
                );
            }
        }
        let prefix = [0, 3];
        let expected = model
            .forward(&prefix, 1, prefix.len(), ReadMode::Enabled, false)?
            .probabilities
            .flatten_all()?
            .to_vec1::<f32>()?;
        let actual = left
            .forward(&prefix, 1, prefix.len(), ReadMode::Enabled, false)?
            .probabilities
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(actual.iter().all(|x| x.is_finite()));
        assert!(actual
            .iter()
            .map(|x| x.to_bits())
            .eq(expected.iter().map(|x| x.to_bits())));

        let name = "copy.gate.bias";
        left.variables()
            .get(name)
            .ok_or_else(|| invalid("missing left copy gate"))?
            .set(&candle_core::Tensor::from_vec(
                vec![91f32],
                (1,),
                &Device::Cpu,
            )?)?;
        let original = model
            .variables()
            .get(name)
            .ok_or_else(|| invalid("missing original copy gate"))?
            .to_vec1::<f32>()?;
        assert_eq!(original, vec![0.25]);
        assert_eq!(
            right
                .variables()
                .get(name)
                .ok_or_else(|| invalid("missing right copy gate"))?
                .to_vec1::<f32>()?,
            original
        );
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn dialogue_artifact_offline_checkpoint_rejects_scope_and_integrity_changes() -> Result<()> {
        let (manifest, bytes) = flat_fixture();
        let (parameters, _) = decode_parameters(&config(), &manifest, &bytes)?;
        let model =
            JointModel::from_offline_dialogue_parameters(config(), parameters, &Device::Cpu)?;
        let source = checkpoint_fixture_root("source")?;
        model.save(&source)?;
        let saved_config: Value = serde_json::from_slice(&fs::read(source.join("config.json"))?)?;
        let saved_weights = fs::read(source.join("model.safetensors"))?;
        fs::remove_dir_all(source)?;

        for case in [
            "unsealed",
            "failed",
            "contract",
            "admission",
            "quantization",
            "geometry",
            "weight_hash",
            "nonfinite",
            "inventory",
        ] {
            // Each malformed input is its own new fixture, never a mutation of
            // the successfully sealed checkpoint or any retained artifact.
            let root = checkpoint_fixture_root(case)?;
            let mut config = saved_config.clone();
            let mut weights = saved_weights.clone();
            let expected_error = match case {
                "unsealed" => None,
                "failed" => {
                    fs::write(root.join("failed-attempt.json"), b"{}")?;
                    Some("records a failed attempt")
                }
                "contract" => {
                    config["numerical_contract"]["read"] = json!("different arithmetic");
                    Some("numerical contract mismatch")
                }
                "admission" => {
                    config["admission"] = json!("recent64");
                    Some("requires continuous Full admission")
                }
                "quantization" => {
                    config["quantization"] = json!({"start_step":0,"ramp_steps":1,
                        "completed_step":0,"spec":{"schema":"test","parameters":{}}});
                    Some("requires continuous Full admission")
                }
                "geometry" => {
                    config["model"]["read_geometry"] = json!("lorentz");
                    Some("offline R1d import requires")
                }
                "weight_hash" => {
                    config["weights_sha256"] = json!("00".repeat(32));
                    Some("weights hash mismatch")
                }
                "nonfinite" => {
                    let end = weights.len();
                    weights[end - 4..].copy_from_slice(&f32::NAN.to_le_bytes());
                    config["weights_sha256"] = json!(hex::encode(Sha256::digest(&weights)));
                    Some("nonfinite loaded joint parameter")
                }
                "inventory" => {
                    let scalar = 0f32.to_le_bytes();
                    let tensor = safetensors::tensor::TensorView::new(
                        safetensors::Dtype::F32,
                        vec![1],
                        &scalar,
                    )?;
                    weights = safetensors::serialize([("copy.gate.bias", tensor)], None)?;
                    config["weights_sha256"] = json!(hex::encode(Sha256::digest(&weights)));
                    Some("parameter names mismatch")
                }
                _ => return Err(invalid("unknown checkpoint test case")),
            };
            fs::write(root.join("config.json"), serde_json::to_vec(&config)?)?;
            fs::write(root.join("model.safetensors"), weights)?;
            if case != "unsealed" {
                report_output::seal(&root)?;
            }
            let error = JointModel::load_offline_dialogue_checkpoint(&root, &Device::Cpu)
                .err()
                .ok_or_else(|| invalid(format!("accepted malformed checkpoint: {case}")))?;
            if let Some(expected) = expected_error {
                assert!(error.to_string().contains(expected), "{case}: {error}");
            }
            fs::remove_dir_all(root)?;
        }
        Ok(())
    }

    #[test]
    fn dialogue_artifact_flat_inventory_preserves_named_f32_boundaries() -> Result<()> {
        let (manifest, bytes) = flat_fixture();
        let (parameters, bindings) = decode_parameters(&config(), &manifest, &bytes)?;
        assert_eq!(parameters.len(), 21);
        assert_eq!(bindings.len(), 21);
        assert_eq!(
            parameters.values().map(Vec::len).sum::<usize>(),
            PARAMETER_COUNT
        );
        for (ordinal, entry) in manifest.index.iter().enumerate() {
            let values = &parameters[&entry.name];
            let expected = ordinal as f32 + 0.25;
            assert_eq!(values[0], expected);
            if values.len() > 1 {
                assert_eq!(values[values.len() - 1], -expected);
            }
        }
        Ok(())
    }

    #[test]
    fn dialogue_artifact_rejects_malformed_inventory_and_nonfinite_values() {
        let (manifest, bytes) = flat_fixture();
        let mut bad = manifest.clone();
        bad.index[1].offset += 1;
        assert!(decode_parameters(&config(), &bad, &bytes).is_err());
        bad = manifest.clone();
        bad.index[1] = bad.index[0].clone();
        assert!(decode_parameters(&config(), &bad, &bytes).is_err());
        bad = manifest.clone();
        bad.index[0].name = "read.lorentz_offset".into();
        assert!(decode_parameters(&config(), &bad, &bytes).is_err());
        bad = manifest.clone();
        bad.index[0].shape = vec![1, 1];
        assert!(decode_parameters(&config(), &bad, &bytes).is_err());
        bad = manifest.clone();
        bad.index[0].elements = usize::MAX;
        assert!(decode_parameters(&config(), &bad, &bytes).is_err());
        assert!(decode_parameters(&config(), &manifest, &bytes[..bytes.len() - 1]).is_err());
        let mut bad_bytes = bytes.clone();
        bad_bytes.push(0);
        assert!(decode_parameters(&config(), &manifest, &bad_bytes).is_err());
        bad_bytes = bytes;
        bad_bytes[..4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(decode_parameters(&config(), &manifest, &bad_bytes).is_err());
    }

    #[test]
    fn dialogue_artifact_configuration_and_exact_prefix_boundaries() -> Result<()> {
        validate_model_config(&config())?;
        // This import never changes the normal checkpoint/runtime width rule.
        assert!(config().validate().is_err());
        for changed in [
            JointConfig {
                width: 256,
                ..config()
            },
            JointConfig {
                context: 64,
                ..config()
            },
            JointConfig {
                transport: Transport::HouseholderPair,
                ..config()
            },
            JointConfig {
                read_geometry: ReadGeometry::Lorentz,
                ..config()
            },
        ] {
            assert!(validate_model_config(&changed).is_err());
        }
        let mut legacy = json!({"vocab_size":4096,"width":576,"read_width":64,
            "context":256,"transport":"quaternion","seed":20260926});
        let parsed: LegacyModel = serde_json::from_value(legacy.clone())?;
        assert_eq!(parsed.config().read_geometry, ReadGeometry::Dot);
        legacy["read_geometry"] = json!("lorentz");
        assert!(serde_json::from_value::<LegacyModel>(legacy).is_err());
        for (ids, horizon, vocabulary) in [
            (vec![], 4, 4096),
            (vec![3], 4, 4096),
            (vec![0, 4096], 4, 4096),
            (vec![0], 0, 4096),
            (vec![0], 4, 4095),
            (vec![0; 253], 4, 4096),
        ] {
            assert!(joint_evaluation::validate_generation_tokens(
                &config(),
                vocabulary,
                &ids,
                horizon
            )
            .is_err());
        }
        // Literal special IDs in content are observed, not stripped or treated
        // as a generated stop; exactly-full declared capacity is allowed.
        joint_evaluation::validate_generation_tokens(&config(), 4096, &[0, 3, 0, 1, 3], 4)?;
        joint_evaluation::validate_generation_tokens(&config(), 4096, &vec![0; 252], 4)?;
        Ok(())
    }

    #[test]
    fn dialogue_artifact_exact_generation_preserves_existing_text_decisions() -> Result<()> {
        let mut vocab = serde_json::Map::new();
        for id in 0..4096u32 {
            let token = match id {
                0 => "<|bos|>".into(),
                1 => "<|eos|>".into(),
                2 => "<|unk|>".into(),
                3 => "a".into(),
                _ => format!("v{id}"),
            };
            vocab.insert(token, json!(id));
        }
        let bytes = serde_json::to_vec(&json!({"model":{"type":"BPE","vocab":vocab,"merges":[]},
            "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
            "added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}))?;
        let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&bytes)
            .ok_or_else(|| invalid("test tokenizer"))?;
        assert_eq!(tokenizer.encode("a"), vec![3]);
        let model = JointModel::new(
            JointConfig {
                width: 128,
                context: 16,
                ..JointConfig::default()
            },
            &Device::Cpu,
        )?;
        for seed in [None, Some(20260927)] {
            let text =
                joint_evaluation::generate(&model, &tokenizer, "a", ReadMode::Enabled, seed, 4)?;
            let exact = joint_evaluation::generate_tokens(
                &model,
                &tokenizer,
                &[0, 3],
                ReadMode::Enabled,
                seed,
                4,
            )?;
            let mut text = serde_json::to_value(text)?;
            let mut exact = serde_json::to_value(exact)?;
            for field in ["elapsed_seconds", "bos_policy"] {
                text.as_object_mut()
                    .ok_or_else(|| invalid("generation object"))?
                    .remove(field);
                exact
                    .as_object_mut()
                    .ok_or_else(|| invalid("generation object"))?
                    .remove(field);
            }
            assert_eq!(text, exact);
        }
        let history = [0, 3, 1, 0, 3];
        let output = joint_evaluation::generate_tokens(
            &model,
            &tokenizer,
            &history,
            ReadMode::NoRead,
            None,
            4,
        )?;
        assert_eq!(output.prompt_token_ids, history);
        assert_eq!(
            output.incremental_step_calls,
            history.len() + output.generated_token_ids.len() - 1
        );
        Ok(())
    }
}
