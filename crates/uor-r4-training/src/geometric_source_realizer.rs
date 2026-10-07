//! Parent-free learned Copy/Period/Stop realization of an admitted source view.
//!
//! This module is a child of `geometric_occurrence_consumer` so its prepared
//! context and coefficient sources are reused without widening their APIs.
//! Existing geometric occurrence scores become Copy logits, existing NoRead
//! becomes Stop, and an independent q4 NoRead-shaped operator supplies Period.
//! There is no parent distribution, authored output schedule or target input to
//! native reading. The supported output alphabet is source-view tokens plus
//! the tokenizer-bound period and protocol EOS; it is not general prose.
//!
//! Native execution reuses the occurrence reader (including its currently
//! unused per-head reductions), shares one prepared integer context with Period,
//! and calls the joint action reducer. The trace and allocation boundaries
//! are explicit, not an allocation-free or compiled-instruction claim.
//! Training retains exact native masses in the loss forward and substitutes
//! the declared softmax of summed-head scores only for its first-order adjoint.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use candle_core::{Device, IndexOp, Tensor, Var};
use serde::{Deserialize, Serialize};
use uor_r4_integer::{
    geometric_cue_carrier::{CueAngularConfig, CueAngularQ4, CueScoreMode, NativeCueCarrier},
    geometric_no_read::{NativeGeometricNoRead, NoReadConfig},
    geometric_occurrence_read::SelectedRecordFrame,
    geometric_potential::AddressLane,
    geometric_potential_q4::pack_coefficients,
    geometric_prefix_transport::{
        NativePrefixTransport, PrefixAngularConfig, PrefixAngularQ4, PrefixScoreMode,
    },
    geometric_source_end_transport::{
        NativeSourceEndTransport, SourceEndAngularConfig, SourceEndAngularQ4, SourceEndScoreMode,
    },
    h4_tables::H4Code,
};

use super::{
    ConsumerIdentity, ConsumerWeights, NativeConsumerArtifact, PreparedConsumerStep,
    SOURCE_VIEW_POLICY, SOURCE_VIEW_SCHEMA,
};
use crate::{
    geometric_context::NativeContextTrace,
    geometric_context_credit::{
        frozen_cue_root_forward, frozen_cue_state_forward, frozen_no_read_forward,
        frozen_potential_forward, frozen_potential_selected_forward, ContextPotentialPair,
    },
    geometric_no_read::{NoReadBatch, NoReadWeights},
    geometric_potential_q4::PotentialQ4Pair,
    geometric_source_actions::{SourceActionBinding, POLICY as ACTION_POLICY},
    geometric_source_emission_view::SourceEmissionView,
    geometric_transport_state_credit::frozen_prefix_state_forward,
    invalid, sha256_bytes, Result,
};

pub const SCHEMA: &str = "uor-r4.geometric-source-realizer/1";
pub const SOURCE_SCHEMA: &str = "uor-r4.geometric-source-realizer-source/1";
pub const POLICY: &str = "parent-free-source-view-Copy-Period-Stop;current-packed-H4-context+potential+Stop+independent-Period;sum-head-logits-before-one-native-normalization;token-alias-mass-sum;no-authored-phase-or-length;128-sequence/1";
pub const SURROGATE: &str = "native-joint-Q31-token-probability-forward;softmax-summed-head-Q24-score-adjoint;quarter-grid-coefficient-STE+frozen-coefficient-context-credit;subtraction-first-zero-forward;target-loss-only;prepared-source-immutable-until-drop/1";
const SOURCE_FILES: [&str; 9] = [
    "tokenizer.json",
    "consumer/context-config.json",
    "consumer/context.safetensors",
    "consumer/potential/metadata.json",
    "consumer/potential/potential-q4-parameters.safetensors",
    "consumer/no-read/metadata.json",
    "consumer/no-read/no-read-parameters.safetensors",
    "period/metadata.json",
    "period/no-read-parameters.safetensors",
];
const NATIVE_FILES: [&str; 7] = [
    "tokenizer.json",
    "period-q4.bin",
    "consumer/metadata.json",
    "consumer/context-q4.bin",
    "consumer/potential-q4.bin",
    "consumer/no-read-q4.bin",
    "consumer/exp-q31.bin",
];

pub struct SourceRealizerWeights {
    consumer: ConsumerWeights,
    period: NoReadWeights,
    binding: SourceActionBinding,
    tokenizer_bytes: Vec<u8>,
    period_seed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterIdentity {
    shape: Vec<usize>,
    f32_sha256: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceMetadata {
    schema: String,
    policy: String,
    surrogate: String,
    source_view_policy: String,
    action_policy: String,
    tokenizer_sha256: String,
    period: NoReadConfig,
    period_seed: u64,
    files: BTreeMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeMetadata {
    schema: String,
    policy: String,
    surrogate: String,
    source_view_policy: String,
    action_policy: String,
    identity: ConsumerIdentity,
    period: NoReadConfig,
    source_parameters: BTreeMap<String, ParameterIdentity>,
    files: BTreeMap<String, String>,
}

impl SourceRealizerWeights {
    /// Consume the admitted consumer source and initialize only the additional
    /// Period coefficients. The explicit seed has no answer/data dependence.
    pub fn new(
        consumer: ConsumerWeights,
        tokenizer_bytes: &[u8],
        period_seed: u64,
    ) -> Result<Self> {
        let c = consumer.config();
        let period = NoReadWeights::new(c.vocab_size, c.heads, c.lanes_per_head)?;
        let mut state = if period_seed == 0 {
            0x9e3779b97f4a7c15
        } else {
            period_seed
        };
        super::set_seeded(period.parameters(), &mut state)?;
        let result = Self {
            consumer,
            period,
            binding: SourceActionBinding::new(tokenizer_bytes)?,
            tokenizer_bytes: tokenizer_bytes.to_vec(),
            period_seed,
        };
        result.validate()?;
        Ok(result)
    }

    fn validate(&self) -> Result<()> {
        let c = self.consumer.config();
        let expected = NoReadConfig {
            vocabulary: c.vocab_size,
            heads: c.heads,
            latent_lanes_per_head: c.lanes_per_head,
        };
        if *self.period.config() != expected
            || *self.consumer.no_read.config() != expected
            || self.binding.vocab_size() != c.vocab_size
            || self.consumer.potential.parent().tokenizer_sha256 != self.binding.tokenizer_sha256()
            || sha256_bytes(&self.tokenizer_bytes) != self.binding.tokenizer_sha256()
        {
            return Err(invalid(
                "realizer tokenizer/source/Period dimensions differ",
            ));
        }
        self.period.packed_coefficients()?;
        self.consumer.context.packed_coefficients()?;
        self.consumer.potential.packed_coefficients()?;
        self.consumer.no_read.packed_coefficients()?;
        Ok(())
    }

    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }

    pub fn parameters(&self) -> BTreeMap<String, Var> {
        self.consumer
            .parameters()
            .into_iter()
            .map(|(name, var)| (format!("consumer.{name}"), var))
            .chain(
                self.period
                    .parameters()
                    .iter()
                    .map(|(name, var)| (format!("period.{name}"), var.clone())),
            )
            .collect()
    }

    /// Existing observation coefficients only; optimizer selection must precede clipping.
    pub fn observation_root_parameters(&self) -> BTreeMap<String, Var> {
        self.parameters()
            .into_iter()
            .filter(|(name, _)| observation_root_parameter(name))
            .collect()
    }

    /// Root-only would-export identity. This is not an independent artifact reload.
    pub fn compile_observation_rebound(
        &self,
        frozen_parent: &NativeSourceRealizer,
    ) -> Result<NativeSourceRealizer> {
        frozen_parent.artifact_binding()?;
        verify_observation_inventory(
            &parameter_identities(self)?,
            &frozen_parent.metadata.source_parameters,
        )?;
        let mut native = self.compile(frozen_parent.metadata.identity.clone())?;
        let (_, metadata) = native.export_payloads()?;
        native.compiled_metadata_sha256 = Some(sha256_bytes(&metadata));
        Ok(native)
    }

    /// Only the existing Stop and Period shadows; no new terminal features.
    pub fn terminal_parameters(&self) -> BTreeMap<String, Var> {
        self.parameters()
            .into_iter()
            .filter(|(name, _)| terminal_parameter(name))
            .collect()
    }

    /// Export a new complete parent, then honestly rebind unchanged cue/prefix
    /// payloads to its actual loaded native identity. The caller owns the report
    /// root/seal; this artifact directory is created exclusively beneath it.
    pub fn save_terminal_rebound(
        &self,
        path: &Path,
        frozen_parent: &NativeSourceRealizer,
        frozen_native_root: &Path,
        frozen_cue: &NativeCueCarrier<'_>,
        frozen_prefix: &NativePrefixTransport<'_>,
    ) -> Result<TerminalRebindReceipt> {
        self.validate()?;
        let old_binding = frozen_parent.artifact_binding()?;
        if crate::sha256_file(&frozen_native_root.join("metadata.json"))?
            != old_binding.metadata_sha256
            || frozen_cue.metadata().parent_artifact != old_binding
            || frozen_prefix.metadata().parent_artifact != old_binding
            || frozen_prefix.metadata().frozen_cue != *frozen_cue.metadata()
        {
            return Err(invalid("terminal frozen parent/cue/prefix binding differs"));
        }
        let now = parameter_identities(self)?;
        let frozen_nonterminal = frozen_parent
            .metadata
            .source_parameters
            .iter()
            .filter(|(name, _)| !terminal_parameter(name))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<BTreeMap<_, _>>();
        let current_nonterminal = now
            .iter()
            .filter(|(name, _)| !terminal_parameter(name))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<BTreeMap<_, _>>();
        if current_nonterminal != frozen_nonterminal {
            return Err(invalid(
                "terminal export changed frozen nonterminal source bits",
            ));
        }
        fs::create_dir(path)?;
        self.save_source(&path.join("source"))?;
        self.compile(frozen_parent.metadata.identity.clone())?
            .save(&path.join("native"))?;
        let source =
            SourceRealizerWeights::load_source(&path.join("source"), &self.tokenizer_bytes)?;
        let parent = NativeSourceRealizer::load(
            &path.join("native"),
            &source,
            &frozen_parent.metadata.identity,
        )?;
        let binding = parent.artifact_binding()?;
        // These numeric payloads cannot change. Consumer and outer metadata do
        // change legitimately because the terminal source/payload is bound there.
        let mut frozen_files = BTreeMap::new();
        for name in [
            "tokenizer.json",
            "consumer/context-q4.bin",
            "consumer/potential-q4.bin",
            "consumer/exp-q31.bin",
        ] {
            let old = crate::sha256_file(&frozen_native_root.join(name))?;
            let new = crate::sha256_file(&path.join("native").join(name))?;
            if old != new {
                return Err(invalid(format!(
                    "terminal export changed frozen payload {name}"
                )));
            }
            frozen_files.insert(name.to_owned(), new);
        }
        if parent.consumer.context.config() != frozen_parent.consumer.context.config()
            || parent.consumer.potential.config() != frozen_parent.consumer.potential.config()
            || parent.binding != frozen_parent.binding
        {
            return Err(invalid(
                "terminal export changed frozen configuration/tokenizer/algebra",
            ));
        }
        // Execute the exported native inventory through the independent integer
        // loader before publishing sidecars; never manufacture its binding.
        let integer = uor_r4_integer::geometric_source_realizer::NativeSourceRealizer::load_native(
            &path.join("native"),
            &binding,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let cue = integer
            .compile_cue_carrier(
                CueAngularQ4::new(
                    frozen_cue.metadata().potential,
                    frozen_cue.packed_coefficients(),
                )
                .map_err(|e| invalid(e.to_string()))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
        let prefix = integer
            .compile_prefix_transport(
                &cue,
                PrefixAngularQ4::new(
                    frozen_prefix.metadata().potential,
                    frozen_prefix.packed_coefficients(),
                )
                .map_err(|e| invalid(e.to_string()))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
        if cue.metadata().context != frozen_cue.metadata().context
            || cue.metadata().context_packed_sha256 != frozen_cue.metadata().context_packed_sha256
            || cue.metadata().algebra_sha256 != frozen_cue.metadata().algebra_sha256
            || prefix.metadata().algebra_sha256 != frozen_prefix.metadata().algebra_sha256
        {
            return Err(invalid("terminal rebind changed frozen geometry/context"));
        }
        for (name, metadata, packed) in [
            (
                "cue",
                serde_json::to_value(cue.metadata())?,
                frozen_cue.packed_coefficients(),
            ),
            (
                "prefix",
                serde_json::to_value(prefix.metadata())?,
                frozen_prefix.packed_coefficients(),
            ),
        ] {
            let root = path.join(name);
            fs::create_dir(&root)?;
            fs::write(
                root.join("native-metadata.json"),
                serde_json::to_vec_pretty(&metadata)?,
            )?;
            fs::write(root.join(format!("{name}-q4.bin")), packed)?;
        }
        let receipt = TerminalRebindReceipt {
            schema: "uor-r4.geometric-terminal-rebind/1".into(),
            old_parent: old_binding,
            new_parent: binding,
            frozen_numeric_payloads_sha256: frozen_files,
            frozen_nonterminal_source_receipts: serde_json::to_value(current_nonterminal)?,
            terminal_source_receipts: serde_json::to_value(
                now.into_iter()
                    .filter(|(name, _)| terminal_parameter(name))
                    .collect::<BTreeMap<_, _>>(),
            )?,
            cue_packed_sha256: sha256_bytes(cue.packed_coefficients()),
            prefix_packed_sha256: sha256_bytes(prefix.packed_coefficients()),
            old_cue_metadata: serde_json::to_value(frozen_cue.metadata())?,
            new_cue_metadata: serde_json::to_value(cue.metadata())?,
            old_prefix_metadata: serde_json::to_value(frozen_prefix.metadata())?,
            new_prefix_metadata: serde_json::to_value(prefix.metadata())?,
            native_metadata_sha256: crate::sha256_file(&path.join("native/metadata.json"))?,
            cue_native_metadata_sha256: crate::sha256_file(&path.join("cue/native-metadata.json"))?,
            prefix_native_metadata_sha256: crate::sha256_file(
                &path.join("prefix/native-metadata.json"),
            )?,
        };
        fs::write(
            path.join("terminal-rebind.json"),
            serde_json::to_vec_pretty(&receipt)?,
        )?;
        Ok(receipt)
    }
    /// Compile a current terminal-only parent without writing its full payload.
    /// Its execution identity is derived from the exact would-export inventory;
    /// it is not evidence of an independent file reload.
    pub fn compile_terminal_rebound(
        &self,
        frozen_parent: &NativeSourceRealizer,
    ) -> Result<NativeSourceRealizer> {
        frozen_parent.artifact_binding()?;
        let current = parameter_identities(self)?;
        for (name, receipt) in &frozen_parent.metadata.source_parameters {
            if !terminal_parameter(name) && current.get(name) != Some(receipt) {
                return Err(invalid(
                    "terminal compile changed frozen nonterminal source bits",
                ));
            }
        }
        if current.len() != frozen_parent.metadata.source_parameters.len() {
            return Err(invalid("terminal compile source inventory differs"));
        }
        let mut native = self.compile(frozen_parent.metadata.identity.clone())?;
        let (_, metadata) = native.export_payloads()?;
        native.compiled_metadata_sha256 = Some(sha256_bytes(&metadata));
        Ok(native)
    }

    pub fn project_quarter_range(&self) -> Result<()> {
        self.consumer.project_quarter_range()?;
        self.period.project_shadow_range()
    }

    pub fn compile(&self, identity: ConsumerIdentity) -> Result<NativeSourceRealizer> {
        self.validate()?;
        if identity.tokenizer_sha256 != self.binding.tokenizer_sha256() {
            return Err(invalid("realizer tokenizer differs from source identity"));
        }
        let consumer = self.consumer.compile_source_view(identity.clone())?;
        let period = self.period.native()?;
        let metadata = NativeMetadata {
            schema: SCHEMA.into(),
            policy: POLICY.into(),
            surrogate: SURROGATE.into(),
            source_view_policy: SOURCE_VIEW_POLICY.into(),
            action_policy: ACTION_POLICY.into(),
            identity,
            period: *self.period.config(),
            source_parameters: parameter_identities(self)?,
            files: BTreeMap::new(),
        };
        Ok(NativeSourceRealizer {
            loaded_metadata_sha256: None,
            compiled_metadata_sha256: None,
            consumer,
            period,
            binding: self.binding.clone(),
            tokenizer_bytes: self.tokenizer_bytes.clone(),
            metadata,
        })
    }

    /// Prepare once per optimizer update. All source Vars must remain unchanged
    /// until this object and its loss graphs are dropped; this includes updates
    /// through previously cloned Vars. Recompile after each completed update.
    pub fn prepare<'a>(
        &'a self,
        native: &'a NativeSourceRealizer,
    ) -> Result<PreparedSourceRealizer<'a>> {
        native.validate_source(self)?;
        Ok(PreparedSourceRealizer {
            source: self,
            native,
            consumer: self.consumer.prepare(&native.consumer)?,
            potential_coefficient_credit: false,
        })
    }

    pub fn save_source(&self, path: &Path) -> Result<()> {
        self.validate()?;
        fs::create_dir(path)?;
        self.consumer.save_source(&path.join("consumer"))?;
        self.period.save(&path.join("period"))?;
        fs::write(path.join("tokenizer.json"), &self.tokenizer_bytes)?;
        let metadata = SourceMetadata {
            schema: SOURCE_SCHEMA.into(),
            policy: POLICY.into(),
            surrogate: SURROGATE.into(),
            source_view_policy: SOURCE_VIEW_POLICY.into(),
            action_policy: ACTION_POLICY.into(),
            tokenizer_sha256: self.binding.tokenizer_sha256().into(),
            period: *self.period.config(),
            period_seed: self.period_seed,
            files: snapshot(path, &SOURCE_FILES)?,
        };
        fs::write(
            path.join("metadata.json"),
            serde_json::to_vec_pretty(&metadata)?,
        )?;
        Ok(())
    }

    pub fn load_source(path: &Path, tokenizer_bytes: &[u8]) -> Result<Self> {
        let metadata: SourceMetadata =
            serde_json::from_slice(&fs::read(path.join("metadata.json"))?)?;
        if metadata.schema != SOURCE_SCHEMA
            || metadata.policy != POLICY
            || metadata.surrogate != SURROGATE
            || metadata.source_view_policy != SOURCE_VIEW_POLICY
            || metadata.action_policy != ACTION_POLICY
            || metadata.tokenizer_sha256 != sha256_bytes(tokenizer_bytes)
            || fs::read(path.join("tokenizer.json"))? != tokenizer_bytes
            || snapshot(path, &SOURCE_FILES)? != metadata.files
        {
            return Err(invalid(
                "realizer source policy/tokenizer/file binding differs",
            ));
        }
        let result = Self {
            consumer: ConsumerWeights::load_source(&path.join("consumer"))?,
            period: NoReadWeights::load(&path.join("period"))?,
            binding: SourceActionBinding::new(tokenizer_bytes)?,
            tokenizer_bytes: tokenizer_bytes.to_vec(),
            period_seed: metadata.period_seed,
        };
        result.validate()?;
        if *result.period.config() != metadata.period {
            return Err(invalid("realizer saved Period config differs"));
        }
        Ok(result)
    }
}

fn parameter_identities(
    weights: &SourceRealizerWeights,
) -> Result<BTreeMap<String, ParameterIdentity>> {
    weights
        .parameters()
        .into_iter()
        .map(|(name, value)| {
            let values = value.flatten_all()?.to_vec1::<f32>()?;
            if values.iter().any(|x| !x.is_finite()) {
                return Err(invalid("realizer nonfinite source parameter"));
            }
            let bytes = values
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>();
            Ok((
                name,
                ParameterIdentity {
                    shape: value.dims().to_vec(),
                    f32_sha256: sha256_bytes(&bytes),
                },
            ))
        })
        .collect()
}

/// Exact admitted source Var names; no prefix-based permission widening.
pub fn observation_root_parameter(name: &str) -> bool {
    matches!(
        name,
        "consumer.context.token_root"
            | "consumer.context.self_root"
            | "consumer.context.neighbor_root"
    )
}
fn verify_observation_inventory(
    current: &BTreeMap<String, ParameterIdentity>,
    frozen: &BTreeMap<String, ParameterIdentity>,
) -> Result<()> {
    if current.keys().ne(frozen.keys()) {
        return Err(invalid("observation compile source inventory differs"));
    }
    for (name, receipt) in frozen {
        let now = current
            .get(name)
            .ok_or_else(|| invalid("observation compile source family absent"))?;
        if now.shape != receipt.shape || (!observation_root_parameter(name) && now != receipt) {
            return Err(invalid(
                "observation compile changed frozen family or source shape",
            ));
        }
    }
    Ok(())
}

pub fn terminal_parameter(name: &str) -> bool {
    matches!(
        name,
        "consumer.no_read.coefficients" | "period.coefficients"
    )
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalRebindReceipt {
    pub schema: String,
    pub old_parent: uor_r4_integer::geometric_source_realizer::NativeArtifactBinding,
    pub new_parent: uor_r4_integer::geometric_source_realizer::NativeArtifactBinding,
    pub frozen_numeric_payloads_sha256: BTreeMap<String, String>,
    pub frozen_nonterminal_source_receipts: serde_json::Value,
    pub terminal_source_receipts: serde_json::Value,
    pub cue_packed_sha256: String,
    pub prefix_packed_sha256: String,
    pub old_cue_metadata: serde_json::Value,
    pub new_cue_metadata: serde_json::Value,
    pub old_prefix_metadata: serde_json::Value,
    pub new_prefix_metadata: serde_json::Value,
    pub native_metadata_sha256: String,
    pub cue_native_metadata_sha256: String,
    pub prefix_native_metadata_sha256: String,
}

/// Compare identical source/query/prefix reads. Terminal logits and global
/// normalized masses may change; raw Copy scores and encoders must not.
pub fn verify_terminal_copy_frozen(
    old: &uor_r4_integer::geometric_source_realizer::PrefixBankRealizerTrace,
    new: &uor_r4_integer::geometric_source_realizer::PrefixBankRealizerTrace,
) -> Result<()> {
    let a = &old.cue_bank.bank;
    let b = &new.cue_bank.bank;
    if a.context != b.context
        || a.candidates != b.candidates
        || a.heads.len() != b.heads.len()
        || a.heads
            .iter()
            .zip(&b.heads)
            .any(|(x, y)| x.scores_q24 != y.scores_q24)
        || old.cue_bank.carrier.query != new.cue_bank.carrier.query
        || old.cue_bank.carrier.cues != new.cue_bank.carrier.cues
        || old.cue_bank.carrier.candidate_cue_indices != new.cue_bank.carrier.candidate_cue_indices
        || old.cue_bank.carrier.angular_indices != new.cue_bank.carrier.angular_indices
        || old.cue_bank.carrier.relative_roots != new.cue_bank.carrier.relative_roots
        || old.cue_bank.carrier.copy_q24 != new.cue_bank.carrier.copy_q24
        || old.prefix.response != new.prefix.response
        || old.prefix.sources != new.prefix.sources
        || old.prefix.candidate_source_indices != new.prefix.candidate_source_indices
        || old.prefix.candidate_offsets != new.prefix.candidate_offsets
        || old.prefix.angular_indices != new.prefix.angular_indices
        || old.prefix.relative_roots != new.prefix.relative_roots
        || old.prefix.copy_q24 != new.prefix.copy_q24
    {
        return Err(invalid(
            "terminal update changed fixed-input Copy/context/cue/prefix",
        ));
    }
    let mut old_cue = serde_json::to_value(old.cue_bank.carrier.metadata.clone())?;
    let mut new_cue = serde_json::to_value(new.cue_bank.carrier.metadata.clone())?;
    old_cue
        .as_object_mut()
        .ok_or_else(|| invalid("cue metadata object absent"))?
        .remove("parent_artifact");
    new_cue
        .as_object_mut()
        .ok_or_else(|| invalid("cue metadata object absent"))?
        .remove("parent_artifact");
    let mut old_prefix = serde_json::to_value(old.prefix.metadata.clone())?;
    let mut new_prefix = serde_json::to_value(new.prefix.metadata.clone())?;
    for metadata in [&mut old_prefix, &mut new_prefix] {
        metadata
            .as_object_mut()
            .ok_or_else(|| invalid("prefix metadata object absent"))?
            .remove("parent_artifact");
        metadata["frozen_cue"]
            .as_object_mut()
            .ok_or_else(|| invalid("prefix frozen cue metadata absent"))?
            .remove("parent_artifact");
    }
    if old_cue != new_cue || old_prefix != new_prefix {
        return Err(invalid(
            "terminal update changed frozen sidecar numerical metadata",
        ));
    }
    Ok(())
}

/// Verify the exact recursive file set, excluding only this root's own metadata.
/// Never traverse symbolic links or trust file paths supplied by metadata.
fn snapshot(root: &Path, expected: &[&str]) -> Result<BTreeMap<String, String>> {
    fn visit(
        root: &Path,
        here: &Path,
        files: &mut BTreeMap<String, String>,
        dirs: &mut BTreeSet<String>,
    ) -> Result<()> {
        for entry in fs::read_dir(here)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let path = entry.path();
            let rel = path
                .strip_prefix(root)
                .map_err(|_| invalid("realizer file escaped root"))?
                .to_str()
                .ok_or_else(|| invalid("realizer path is not UTF8"))?
                .replace('\\', "/");
            if ty.is_dir() {
                dirs.insert(rel);
                visit(root, &path, files, dirs)?;
            } else if ty.is_file() {
                if rel != "metadata.json" {
                    files.insert(rel, sha256_bytes(&fs::read(path)?));
                }
            } else {
                return Err(invalid("realizer source contains a non-file/non-directory"));
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    let mut dirs = BTreeSet::new();
    visit(root, root, &mut files, &mut dirs)?;
    let expected_files = expected
        .iter()
        .map(|x| x.to_string())
        .collect::<BTreeSet<_>>();
    let mut expected_dirs = BTreeSet::new();
    for name in expected {
        let mut parent = Path::new(name).parent();
        while let Some(p) = parent.filter(|p| !p.as_os_str().is_empty()) {
            expected_dirs.insert(p.to_string_lossy().replace('\\', "/"));
            parent = p.parent();
        }
    }
    if files.keys().cloned().collect::<BTreeSet<_>>() != expected_files || dirs != expected_dirs {
        return Err(invalid("realizer exact file/directory inventory differs"));
    }
    Ok(files)
}

pub use uor_r4_integer::geometric_source_realizer::{
    BankRealizerTrace, ObservedCode, RealizerTrace, SerializableContextReplay, SourceBankSegment,
};
fn context_replay_matches(replay: &SerializableContextReplay, trace: &NativeContextTrace) -> bool {
    trace.batch == 1
        && trace.time == replay.tokens.len()
        && trace.heads == replay.heads
        && trace.lanes_per_head == replay.lanes_per_head
        && trace.states == replay.states
        && trace.actions == replay.actions
        && trace.emitted_roots == replay.raw_roots
        && trace.categories == replay.categories
        && trace.coefficient_reads == replay.coefficient_reads
        && trace
            .codes
            .iter()
            .copied()
            .map(ObservedCode::from)
            .eq(replay.codes.iter().cloned())
}

pub struct NativeSourceRealizer {
    loaded_metadata_sha256: Option<String>,
    compiled_metadata_sha256: Option<String>,
    consumer: NativeConsumerArtifact,
    period: NativeGeometricNoRead,
    binding: SourceActionBinding,
    tokenizer_bytes: Vec<u8>,
    metadata: NativeMetadata,
}

impl NativeSourceRealizer {
    /// Current packed Stop and Period payloads, in that order. This exposes
    /// numerical bytes only and does not assert independently loaded provenance.
    pub fn terminal_packed_payloads(&self) -> (&[u8], &[u8]) {
        (
            self.consumer.no_read.packed_coefficients(),
            self.period.packed_coefficients(),
        )
    }

    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    pub fn artifact_binding(
        &self,
    ) -> Result<uor_r4_integer::geometric_source_realizer::NativeArtifactBinding> {
        let metadata_sha256 = self.loaded_metadata_sha256.clone().ok_or_else(|| {
            invalid("feedback learning requires an independently saved/reloaded native parent")
        })?;
        Ok(self.binding_with_digest(metadata_sha256))
    }

    /// A loaded identity or an explicitly derived terminal would-export identity.
    /// Only artifact_binding() establishes independently loaded provenance.
    pub fn execution_binding(
        &self,
    ) -> Result<uor_r4_integer::geometric_source_realizer::NativeArtifactBinding> {
        let digest = self
            .loaded_metadata_sha256
            .as_ref()
            .or(self.compiled_metadata_sha256.as_ref())
            .ok_or_else(|| {
                invalid("native execution requires loaded or terminal-derived inventory")
            })?;
        Ok(self.binding_with_digest(digest.clone()))
    }

    fn binding_with_digest(
        &self,
        metadata_sha256: String,
    ) -> uor_r4_integer::geometric_source_realizer::NativeArtifactBinding {
        let i = &self.metadata.identity;
        uor_r4_integer::geometric_source_realizer::NativeArtifactBinding {
            metadata_sha256,
            identity: uor_r4_integer::geometric_source_realizer::ArtifactIdentity {
                tokenizer_sha256: i.tokenizer_sha256.clone(),
                parent_checkpoint_manifest_sha256: i.parent_checkpoint_manifest_sha256.clone(),
                parent_model_sha256: i.parent_model_sha256.clone(),
                parent_config_sha256: i.parent_config_sha256.clone(),
            },
        }
    }
    /// Target-free bank execution delegates to the same shared integer kernel.
    pub fn read_bank(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
    ) -> Result<BankRealizerTrace> {
        Ok(
            uor_r4_integer::geometric_source_realizer::RealizerExecution {
                context: &self.consumer.context,
                potential_tables: &self.consumer.potential_tables,
                no_read: &self.consumer.no_read,
                geometry: &self.consumer.geometry,
                exp: &self.consumer.exp,
                period: &self.period,
                binding: &self.binding,
            }
            .read_bank(segments, query, prefix)?,
        )
    }
    pub fn compile_cue_carrier(&self, angular: CueAngularQ4) -> Result<NativeCueCarrier<'_>> {
        NativeCueCarrier::compile(
            self.execution_binding()?,
            &self.consumer.context,
            &self.consumer.geometry,
            angular,
        )
        .map_err(|e| invalid(e.to_string()))
    }
    /// Target-free sidecar execution through the shared integer kernel. The
    /// retained parent heads and source states remain the native factual base.
    pub fn read_bank_with_cue_carrier(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        carrier: &uor_r4_integer::geometric_cue_carrier::NativeCueCarrier<'_>,
    ) -> Result<uor_r4_integer::geometric_source_realizer::CueBankRealizerTrace> {
        let parent = self.execution_binding()?;
        uor_r4_integer::geometric_source_realizer::RealizerExecution {
            context: &self.consumer.context,
            potential_tables: &self.consumer.potential_tables,
            no_read: &self.consumer.no_read,
            geometry: &self.consumer.geometry,
            exp: &self.consumer.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read_bank_with_cue_carrier(segments, query, prefix, &parent, carrier)
        .map_err(|e| invalid(e.to_string()))
    }
    pub fn compile_prefix_transport(
        &self,
        cue: &NativeCueCarrier<'_>,
        angular: PrefixAngularQ4,
    ) -> Result<NativePrefixTransport<'_>> {
        NativePrefixTransport::compile(
            self.execution_binding()?,
            &self.consumer.context,
            &self.consumer.geometry,
            cue.metadata().clone(),
            angular,
        )
        .map_err(|e| invalid(e.to_string()))
    }
    pub fn read_bank_with_prefix_transport(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        cue: &NativeCueCarrier<'_>,
        transport: &NativePrefixTransport<'_>,
    ) -> Result<uor_r4_integer::geometric_source_realizer::PrefixBankRealizerTrace> {
        let parent = self.execution_binding()?;
        uor_r4_integer::geometric_source_realizer::RealizerExecution {
            context: &self.consumer.context,
            potential_tables: &self.consumer.potential_tables,
            no_read: &self.consumer.no_read,
            geometry: &self.consumer.geometry,
            exp: &self.consumer.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read_bank_with_prefix_transport(segments, query, prefix, &parent, cue, transport)
        .map_err(|e| invalid(e.to_string()))
    }
    pub fn compile_source_end_transport(
        &self,
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
        angular: SourceEndAngularQ4,
    ) -> Result<NativeSourceEndTransport<'_>> {
        NativeSourceEndTransport::compile(
            self.execution_binding()?,
            &self.consumer.context,
            &self.consumer.geometry,
            cue.metadata().clone(),
            prefix.metadata().clone(),
            angular,
        )
        .map_err(|e| invalid(e.to_string()))
    }
    pub fn read_bank_with_source_end_transport(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        actual_prefix: &[u32],
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
        end: &NativeSourceEndTransport<'_>,
    ) -> Result<uor_r4_integer::geometric_source_realizer::SourceEndBankRealizerTrace> {
        let parent = self.execution_binding()?;
        uor_r4_integer::geometric_source_realizer::RealizerExecution {
            context: &self.consumer.context,
            potential_tables: &self.consumer.potential_tables,
            no_read: &self.consumer.no_read,
            geometry: &self.consumer.geometry,
            exp: &self.consumer.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read_bank_with_source_end_transport(
            segments,
            query,
            actual_prefix,
            &parent,
            cue,
            prefix,
            end,
        )
        .map_err(|e| invalid(e.to_string()))
    }

    pub fn read_dependent(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        feedback: &uor_r4_integer::geometric_read_feedback::NativeReadFeedback,
        mode: uor_r4_integer::geometric_read_feedback::FeedbackInputMode,
    ) -> Result<uor_r4_integer::geometric_source_realizer::DependentReadTrace> {
        Ok(
            uor_r4_integer::geometric_source_realizer::RealizerExecution {
                context: &self.consumer.context,
                potential_tables: &self.consumer.potential_tables,
                no_read: &self.consumer.no_read,
                geometry: &self.consumer.geometry,
                exp: &self.consumer.exp,
                period: &self.period,
                binding: &self.binding,
            }
            .read_dependent(
                frame,
                view,
                query,
                prefix,
                &self.execution_binding()?,
                feedback,
                mode,
            )?,
        )
    }
    /// Offline target-free interventions reuse the same shared integer scorer.
    pub fn read_dependent_action_traces(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        feedback: &uor_r4_integer::geometric_read_feedback::NativeReadFeedback,
        mode: uor_r4_integer::geometric_read_feedback::FeedbackInputMode,
    ) -> Result<uor_r4_integer::geometric_source_realizer::NativeActionCounterfactualTrace> {
        Ok(
            uor_r4_integer::geometric_source_realizer::RealizerExecution {
                context: &self.consumer.context,
                potential_tables: &self.consumer.potential_tables,
                no_read: &self.consumer.no_read,
                geometry: &self.consumer.geometry,
                exp: &self.consumer.exp,
                period: &self.period,
                binding: &self.binding,
            }
            .read_dependent_action_traces(
                frame,
                view,
                query,
                prefix,
                &self.execution_binding()?,
                feedback,
                mode,
            )?,
        )
    }
    fn validate_source(&self, source: &SourceRealizerWeights) -> Result<()> {
        source.validate()?;
        self.consumer.validate_source(&source.consumer)?;
        if self.consumer.metadata.schema != SOURCE_VIEW_SCHEMA
            || self.consumer.metadata.source_view_policy.as_deref() != Some(SOURCE_VIEW_POLICY)
            || self.period.config() != *source.period.config()
            || self.period.packed_coefficients() != source.period.packed_coefficients()?
            || self.metadata.source_parameters != parameter_identities(source)?
            || self.tokenizer_bytes != source.tokenizer_bytes
        {
            return Err(invalid(
                "realizer current source differs from compiled source",
            ));
        }
        Ok(())
    }

    pub fn read(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
    ) -> Result<RealizerTrace> {
        Ok(
            uor_r4_integer::geometric_source_realizer::RealizerExecution {
                context: &self.consumer.context,
                potential_tables: &self.consumer.potential_tables,
                no_read: &self.consumer.no_read,
                geometry: &self.consumer.geometry,
                exp: &self.consumer.exp,
                period: &self.period,
                binding: &self.binding,
            }
            .read(frame, view, query, prefix)?,
        )
    }
    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({"consumer":self.consumer.stats(),"period":self.period.stats(),
            "extra_context_replays_per_read":0,"original_context_replays_per_read":1,"final_joint_reductions_per_read":1,
            "inherited_per_head_reductions_discarded":self.consumer.metadata.context.heads,
            "scope":"native integer components inside allocating wrapper; no parent-model inference, no complete-path opcode/allocation qualification"})
    }

    /// Exact serializer shared by disk export and terminal execution identities.
    fn export_payloads(&self) -> Result<(BTreeMap<String, Vec<u8>>, Vec<u8>)> {
        let mut payloads = BTreeMap::new();
        let exp = self
            .consumer
            .exp
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>();
        let consumer_payloads = [
            self.consumer.context.packed_coefficients(),
            self.consumer.potential.packed_coefficients(),
            self.consumer.no_read.packed_coefficients(),
            exp.as_slice(),
        ];
        let mut consumer_metadata = self.consumer.metadata.clone();
        for (name, bytes) in super::NATIVE_FILES.into_iter().zip(consumer_payloads) {
            consumer_metadata
                .files
                .insert(name.into(), sha256_bytes(bytes));
            payloads.insert(format!("consumer/{name}"), bytes.to_vec());
        }
        payloads.insert(
            "consumer/metadata.json".into(),
            serde_json::to_vec_pretty(&consumer_metadata)?,
        );
        payloads.insert("tokenizer.json".into(), self.tokenizer_bytes.clone());
        payloads.insert(
            "period-q4.bin".into(),
            self.period.packed_coefficients().to_vec(),
        );
        let mut metadata = self.metadata.clone();
        metadata.files = payloads
            .iter()
            .map(|(name, bytes)| (name.clone(), sha256_bytes(bytes)))
            .collect();
        Ok((payloads, serde_json::to_vec_pretty(&metadata)?))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let (payloads, metadata) = self.export_payloads()?;
        fs::create_dir(path)?;
        fs::create_dir(path.join("consumer"))?;
        for (name, bytes) in payloads {
            fs::write(path.join(name), bytes)?;
        }
        fs::write(path.join("metadata.json"), metadata)?;
        Ok(())
    }

    pub fn load(
        path: &Path,
        source: &SourceRealizerWeights,
        identity: &ConsumerIdentity,
    ) -> Result<Self> {
        identity.validate()?;
        source.validate()?;
        let metadata_bytes = fs::read(path.join("metadata.json"))?;
        let metadata: NativeMetadata = serde_json::from_slice(&metadata_bytes)?;
        let tokenizer_bytes = fs::read(path.join("tokenizer.json"))?;
        if metadata.schema != SCHEMA
            || metadata.policy != POLICY
            || metadata.surrogate != SURROGATE
            || metadata.source_view_policy != SOURCE_VIEW_POLICY
            || metadata.action_policy != ACTION_POLICY
            || metadata.identity != *identity
            || metadata.period != *source.period.config()
            || tokenizer_bytes != source.tokenizer_bytes
            || sha256_bytes(&tokenizer_bytes) != identity.tokenizer_sha256
            || metadata.source_parameters != parameter_identities(source)?
            || snapshot(path, &NATIVE_FILES)? != metadata.files
        {
            return Err(invalid(
                "realizer native/source/tokenizer policy or file identity differs",
            ));
        }
        let packed = fs::read(path.join("period-q4.bin"))?;
        if packed != source.period.packed_coefficients()? {
            return Err(invalid("realizer exported Period differs from source"));
        }
        // Execute admitted saved bytes, not a freshly compiled replacement.
        let result = Self {
            loaded_metadata_sha256: Some(sha256_bytes(&metadata_bytes)),
            compiled_metadata_sha256: None,
            consumer: NativeConsumerArtifact::load(
                &path.join("consumer"),
                &source.consumer,
                identity,
            )?,
            period: NativeGeometricNoRead::new(metadata.period, &packed)
                .map_err(|e| invalid(e.to_string()))?,
            binding: SourceActionBinding::new(&tokenizer_bytes)?,
            tokenizer_bytes,
            metadata,
        };
        result.validate_source(source)?;
        Ok(result)
    }
}

pub use crate::geometric_read_feedback::FeedbackBridgeWeights;
pub struct DependentRealizerLoss {
    pub loss: Tensor,
    pub trace: uor_r4_integer::geometric_source_realizer::DependentReadTrace,
    pub target_probability: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct NativeRouteLaneUtility {
    pub lane: usize,
    pub factual_action: u8,
    pub target_mass_q31: Vec<u64>,
    pub total_weight_q31: Vec<u64>,
    pub target_probabilities: Vec<f64>,
    pub policy_logits_nat: Vec<f32>,
    pub policy_probabilities: Vec<f32>,
    pub mixture_probability: f64,
    pub policy_adjoint: Vec<f64>,
    pub zero_support_actions: usize,
    pub best_target_probability: f64,
    pub useful_actions: usize,
}
pub struct NativeRouteRealizerLoss {
    pub loss: Tensor,
    pub trace: uor_r4_integer::geometric_source_realizer::DependentReadTrace,
    pub target_probability: f64,
    pub utilities: Vec<NativeRouteLaneUtility>,
    pub counterfactual_actions: usize,
    pub counterfactual_seconds: f64,
}

pub struct RealizerLoss {
    pub loss: Tensor,
    pub trace: RealizerTrace,
    pub target_probability: f64,
}
pub struct BankRealizerLoss {
    pub loss: Tensor,
    pub trace: BankRealizerTrace,
    pub target_probability: f64,
}
/// Offline-only shadows for the capacity-matched, integer cue angular readout.
/// Parent context, ordinary source heads and terminal heads are immutable.
pub struct CueAngularWeights {
    coefficients: Var,
    metadata: CueAngularSourceMetadata,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CueAngularSourceMetadata {
    schema: String,
    parent: uor_r4_integer::geometric_source_realizer::NativeArtifactBinding,
    parent_native_files_sha256: BTreeMap<String, String>,
    config: CueAngularConfig,
    source_sha256: String,
    packed_sha256: String,
    native_metadata: serde_json::Value,
}
fn cue_parent_files(
    native: &NativeSourceRealizer,
    root: &Path,
) -> Result<BTreeMap<String, String>> {
    if crate::sha256_file(&root.join("metadata.json"))?
        != native.artifact_binding()?.metadata_sha256
    {
        return Err(invalid("cue bound parent metadata differs"));
    }
    [
        "metadata.json",
        "consumer/context-q4.bin",
        "consumer/metadata.json",
        "tokenizer.json",
    ]
    .into_iter()
    .map(|name| Ok((name.into(), crate::sha256_file(&root.join(name))?)))
    .collect()
}
impl CueAngularWeights {
    pub fn zero(
        parent: &NativeSourceRealizer,
        native_root: &Path,
        mode: CueScoreMode,
    ) -> Result<Self> {
        let c = parent.consumer.context.config();
        let config = CueAngularConfig {
            heads: c.heads,
            lanes_per_head: c.lanes_per_head,
            mode,
        };
        let count = config
            .coefficient_count()
            .map_err(|e| invalid(e.to_string()))?;
        let result = Self {
            coefficients: Var::from_vec(vec![0f32; count], count, &Device::Cpu)?,
            metadata: CueAngularSourceMetadata {
                schema: "uor-r4.geometric-cue-angular-source/1".into(),
                parent: parent.artifact_binding()?,
                parent_native_files_sha256: cue_parent_files(parent, native_root)?,
                config,
                source_sha256: String::new(),
                packed_sha256: String::new(),
                native_metadata: serde_json::Value::Null,
            },
        };
        let mut result = result;
        result.metadata.native_metadata =
            serde_json::to_value(parent.compile_cue_carrier(result.native()?)?.metadata())?;
        Ok(result)
    }
    pub fn config(&self) -> CueAngularConfig {
        self.metadata.config
    }
    pub fn parent_binding(
        &self,
    ) -> &uor_r4_integer::geometric_source_realizer::NativeArtifactBinding {
        &self.metadata.parent
    }
    pub fn parameters(&self) -> BTreeMap<String, Var> {
        BTreeMap::from([("cue.coefficients".into(), self.coefficients.clone())])
    }
    pub fn project_shadow_range(&self) -> Result<()> {
        let v = self.coefficients.to_vec1::<f32>()?;
        if v.iter().any(|x| !x.is_finite()) {
            return Err(invalid("nonfinite cue angular shadow"));
        }
        self.coefficients.set(&Tensor::from_vec(
            v.into_iter()
                .map(|x| x.clamp(-1.75, 1.75))
                .collect::<Vec<_>>(),
            self.coefficients.shape(),
            &Device::Cpu,
        )?)?;
        Ok(())
    }
    pub fn packed_coefficients(&self) -> Result<Vec<u8>> {
        let v = self.coefficients.to_vec1::<f32>()?;
        if v.iter().any(|x| !x.is_finite() || *x < -1.75 || *x > 1.75) {
            return Err(invalid("cue angular shadow outside legal quarter range"));
        }
        pack_coefficients(
            &v.into_iter()
                .map(|x| (x * 4.).round() as i8)
                .collect::<Vec<_>>(),
        )
        .map_err(|e| invalid(e.to_string()))
    }
    pub fn native(&self) -> Result<CueAngularQ4> {
        CueAngularQ4::new(self.config(), &self.packed_coefficients()?)
            .map_err(|e| invalid(e.to_string()))
    }
    fn source_bytes(&self) -> Result<Vec<u8>> {
        Ok(self
            .coefficients
            .to_vec1::<f32>()?
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect())
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        fs::create_dir(path)?;
        let source = self.source_bytes()?;
        let packed = self.packed_coefficients()?;
        let mut m = self.metadata.clone();
        m.source_sha256 = sha256_bytes(&source);
        m.packed_sha256 = sha256_bytes(&packed);
        m.native_metadata["potential_packed_sha256"] = serde_json::json!(m.packed_sha256);
        fs::write(
            path.join("native-metadata.json"),
            serde_json::to_vec_pretty(&m.native_metadata)?,
        )?;
        fs::write(path.join("cue-source-f32.bin"), source)?;
        fs::write(path.join("cue-q4.bin"), packed)?;
        fs::write(path.join("metadata.json"), serde_json::to_vec_pretty(&m)?)?;
        Ok(())
    }
    pub fn load(path: &Path, parent: &NativeSourceRealizer, native_root: &Path) -> Result<Self> {
        let m: CueAngularSourceMetadata =
            serde_json::from_slice(&fs::read(path.join("metadata.json"))?)?;
        let raw = fs::read(path.join("cue-source-f32.bin"))?;
        let packed = fs::read(path.join("cue-q4.bin"))?;
        let count = m
            .config
            .coefficient_count()
            .map_err(|e| invalid(e.to_string()))?;
        if m.schema != "uor-r4.geometric-cue-angular-source/1"
            || m.parent != parent.artifact_binding()?
            || m.parent_native_files_sha256 != cue_parent_files(parent, native_root)?
            || raw.len() != count * 4
            || m.source_sha256 != sha256_bytes(&raw)
            || m.packed_sha256 != sha256_bytes(&packed)
        {
            return Err(invalid("cue source/native parent bundle binding differs"));
        }
        let values = raw
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        let result = Self {
            coefficients: Var::from_vec(values, count, &Device::Cpu)?,
            metadata: m,
        };
        if result.packed_coefficients()? != packed {
            return Err(invalid("cue source/native packed replay differs"));
        }
        let carrier = parent.compile_cue_carrier(result.native()?)?;
        if serde_json::to_value(carrier.metadata())? != result.metadata.native_metadata
            || serde_json::from_slice::<serde_json::Value>(&fs::read(
                path.join("native-metadata.json"),
            )?)? != result.metadata.native_metadata
            || carrier.metadata().parent_artifact != *result.parent_binding()
        {
            return Err(invalid("cue compiled parent differs"));
        }
        Ok(result)
    }
}
pub struct PrefixAngularWeights {
    coefficients: Var,
    metadata: PrefixAngularSourceMetadata,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrefixAngularSourceMetadata {
    schema: String,
    parent: uor_r4_integer::geometric_source_realizer::NativeArtifactBinding,
    parent_native_files_sha256: BTreeMap<String, String>,
    cue_native_files_sha256: BTreeMap<String, String>,
    cue_native_metadata: serde_json::Value,
    config: PrefixAngularConfig,
    source_sha256: String,
    packed_sha256: String,
    native_metadata: serde_json::Value,
}
fn prefix_parent_files(
    native: &NativeSourceRealizer,
    root: &Path,
) -> Result<BTreeMap<String, String>> {
    if crate::sha256_file(&root.join("metadata.json"))?
        != native.artifact_binding()?.metadata_sha256
    {
        return Err(invalid("cue bound parent metadata differs"));
    }
    [
        "metadata.json",
        "consumer/context-q4.bin",
        "consumer/metadata.json",
        "tokenizer.json",
    ]
    .into_iter()
    .map(|name| Ok((name.into(), crate::sha256_file(&root.join(name))?)))
    .collect()
}
fn prefix_cue_files(root: &Path, cue: &NativeCueCarrier<'_>) -> Result<BTreeMap<String, String>> {
    let metadata = fs::read(root.join("native-metadata.json"))?;
    if serde_json::from_slice::<serde_json::Value>(&metadata)?
        != serde_json::to_value(cue.metadata())?
        || fs::read(root.join("cue-q4.bin"))? != cue.packed_coefficients()
    {
        return Err(invalid("prefix frozen cue native bundle differs"));
    }
    ["native-metadata.json", "cue-q4.bin"]
        .into_iter()
        .map(|name| Ok((name.into(), crate::sha256_file(&root.join(name))?)))
        .collect()
}
impl PrefixAngularWeights {
    pub fn zero(
        parent: &NativeSourceRealizer,
        native_root: &Path,
        cue_root: &Path,
        cue: &NativeCueCarrier<'_>,
        mode: PrefixScoreMode,
    ) -> Result<Self> {
        let c = parent.consumer.context.config();
        let config = PrefixAngularConfig {
            heads: c.heads,
            lanes_per_head: c.lanes_per_head,
            mode,
        };
        let count = config
            .coefficient_count()
            .map_err(|e| invalid(e.to_string()))?;
        let result = Self {
            coefficients: Var::from_vec(vec![0f32; count], count, &Device::Cpu)?,
            metadata: PrefixAngularSourceMetadata {
                schema: "uor-r4.geometric-prefix-angular-source/1".into(),
                parent: parent.artifact_binding()?,
                parent_native_files_sha256: prefix_parent_files(parent, native_root)?,
                cue_native_files_sha256: prefix_cue_files(cue_root, cue)?,
                cue_native_metadata: serde_json::to_value(cue.metadata())?,
                config,
                source_sha256: String::new(),
                packed_sha256: String::new(),
                native_metadata: serde_json::Value::Null,
            },
        };
        let mut result = result;
        result.metadata.native_metadata = serde_json::to_value(
            parent
                .compile_prefix_transport(cue, result.native()?)?
                .metadata(),
        )?;
        Ok(result)
    }
    pub fn config(&self) -> PrefixAngularConfig {
        self.metadata.config
    }
    pub fn parent_binding(
        &self,
    ) -> &uor_r4_integer::geometric_source_realizer::NativeArtifactBinding {
        &self.metadata.parent
    }
    pub fn parameters(&self) -> BTreeMap<String, Var> {
        BTreeMap::from([("prefix.coefficients".into(), self.coefficients.clone())])
    }
    pub fn project_shadow_range(&self) -> Result<()> {
        let v = self.coefficients.to_vec1::<f32>()?;
        if v.iter().any(|x| !x.is_finite()) {
            return Err(invalid("nonfinite prefix angular shadow"));
        }
        self.coefficients.set(&Tensor::from_vec(
            v.into_iter()
                .map(|x| x.clamp(-1.75, 1.75))
                .collect::<Vec<_>>(),
            self.coefficients.shape(),
            &Device::Cpu,
        )?)?;
        Ok(())
    }
    pub fn packed_coefficients(&self) -> Result<Vec<u8>> {
        let v = self.coefficients.to_vec1::<f32>()?;
        if v.iter().any(|x| !x.is_finite() || *x < -1.75 || *x > 1.75) {
            return Err(invalid("prefix angular shadow outside legal quarter range"));
        }
        pack_coefficients(
            &v.into_iter()
                .map(|x| (x * 4.).round() as i8)
                .collect::<Vec<_>>(),
        )
        .map_err(|e| invalid(e.to_string()))
    }
    pub fn native(&self) -> Result<PrefixAngularQ4> {
        PrefixAngularQ4::new(self.config(), &self.packed_coefficients()?)
            .map_err(|e| invalid(e.to_string()))
    }
    fn source_bytes(&self) -> Result<Vec<u8>> {
        Ok(self
            .coefficients
            .to_vec1::<f32>()?
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect())
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        fs::create_dir(path)?;
        let source = self.source_bytes()?;
        let packed = self.packed_coefficients()?;
        let mut m = self.metadata.clone();
        m.source_sha256 = sha256_bytes(&source);
        m.packed_sha256 = sha256_bytes(&packed);
        m.native_metadata["potential_packed_sha256"] = serde_json::json!(m.packed_sha256);
        fs::write(
            path.join("native-metadata.json"),
            serde_json::to_vec_pretty(&m.native_metadata)?,
        )?;
        fs::write(path.join("prefix-source-f32.bin"), source)?;
        fs::write(path.join("prefix-q4.bin"), packed)?;
        fs::write(path.join("metadata.json"), serde_json::to_vec_pretty(&m)?)?;
        Ok(())
    }
    pub fn load(
        path: &Path,
        parent: &NativeSourceRealizer,
        native_root: &Path,
        cue_root: &Path,
        cue: &NativeCueCarrier<'_>,
    ) -> Result<Self> {
        let m: PrefixAngularSourceMetadata =
            serde_json::from_slice(&fs::read(path.join("metadata.json"))?)?;
        let raw = fs::read(path.join("prefix-source-f32.bin"))?;
        let packed = fs::read(path.join("prefix-q4.bin"))?;
        let count = m
            .config
            .coefficient_count()
            .map_err(|e| invalid(e.to_string()))?;
        if m.schema != "uor-r4.geometric-prefix-angular-source/1"
            || m.parent != parent.artifact_binding()?
            || m.parent_native_files_sha256 != prefix_parent_files(parent, native_root)?
            || m.cue_native_files_sha256 != prefix_cue_files(cue_root, cue)?
            || m.cue_native_metadata != serde_json::to_value(cue.metadata())?
            || raw.len() != count * 4
            || m.source_sha256 != sha256_bytes(&raw)
            || m.packed_sha256 != sha256_bytes(&packed)
        {
            return Err(invalid(
                "prefix source/native parent bundle binding differs",
            ));
        }
        let values = raw
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        let result = Self {
            coefficients: Var::from_vec(values, count, &Device::Cpu)?,
            metadata: m,
        };
        if result.packed_coefficients()? != packed {
            return Err(invalid("prefix source/native packed replay differs"));
        }
        let carrier = parent.compile_prefix_transport(cue, result.native()?)?;
        if serde_json::to_value(carrier.metadata())? != result.metadata.native_metadata
            || serde_json::from_slice::<serde_json::Value>(&fs::read(
                path.join("native-metadata.json"),
            )?)? != result.metadata.native_metadata
            || carrier.metadata().parent_artifact != *result.parent_binding()
        {
            return Err(invalid("prefix compiled parent differs"));
        }
        Ok(result)
    }
}
pub struct SourceEndAngularWeights {
    period_coefficients: Var,
    stop_coefficients: Var,
    metadata: SourceEndAngularSourceMetadata,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceEndAngularSourceMetadata {
    schema: String,
    parent: uor_r4_integer::geometric_source_realizer::NativeArtifactBinding,
    parent_native_files_sha256: BTreeMap<String, String>,
    cue_native_files_sha256: BTreeMap<String, String>,
    prefix_native_files_sha256: BTreeMap<String, String>,
    cue_native_metadata: serde_json::Value,
    prefix_native_metadata: serde_json::Value,
    config: SourceEndAngularConfig,
    period_source_sha256: String,
    stop_source_sha256: String,
    period_packed_sha256: String,
    stop_packed_sha256: String,
    native_metadata: serde_json::Value,
}
fn source_end_prefix_files(
    root: &Path,
    prefix: &NativePrefixTransport<'_>,
) -> Result<BTreeMap<String, String>> {
    if serde_json::from_slice::<serde_json::Value>(&fs::read(root.join("native-metadata.json"))?)?
        != serde_json::to_value(prefix.metadata())?
        || fs::read(root.join("prefix-q4.bin"))? != prefix.packed_coefficients()
    {
        return Err(invalid("source-end frozen prefix bundle differs"));
    }
    ["native-metadata.json", "prefix-q4.bin"]
        .into_iter()
        .map(|name| Ok((name.into(), crate::sha256_file(&root.join(name))?)))
        .collect()
}
fn source_end_source_bytes(var: &Var) -> Result<Vec<u8>> {
    Ok(var
        .to_vec1::<f32>()?
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect())
}
fn source_end_packed(var: &Var) -> Result<Vec<u8>> {
    let values = var.to_vec1::<f32>()?;
    if values.iter().any(|x| !x.is_finite() || x.abs() > 1.75) {
        return Err(invalid("source-end shadow outside legal quarter range"));
    }
    pack_coefficients(
        &values
            .into_iter()
            .map(|x| (x * 4.).round() as i8)
            .collect::<Vec<_>>(),
    )
    .map_err(|e| invalid(e.to_string()))
}
impl SourceEndAngularWeights {
    pub fn zero(
        parent: &NativeSourceRealizer,
        native_root: &Path,
        cue_root: &Path,
        prefix_root: &Path,
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
        mode: SourceEndScoreMode,
    ) -> Result<Self> {
        let c = parent.consumer.context.config();
        let config = SourceEndAngularConfig {
            heads: c.heads,
            lanes_per_head: c.lanes_per_head,
            mode,
        };
        let count = config
            .coefficient_count()
            .map_err(|e| invalid(e.to_string()))?;
        let mut result = Self {
            period_coefficients: Var::from_vec(vec![0f32; count], count, &Device::Cpu)?,
            stop_coefficients: Var::from_vec(vec![0f32; count], count, &Device::Cpu)?,
            metadata: SourceEndAngularSourceMetadata {
                schema: "uor-r4.geometric-source-end-angular-source/1".into(),
                parent: parent.artifact_binding()?,
                parent_native_files_sha256: prefix_parent_files(parent, native_root)?,
                cue_native_files_sha256: prefix_cue_files(cue_root, cue)?,
                prefix_native_files_sha256: source_end_prefix_files(prefix_root, prefix)?,
                cue_native_metadata: serde_json::to_value(cue.metadata())?,
                prefix_native_metadata: serde_json::to_value(prefix.metadata())?,
                config,
                period_source_sha256: String::new(),
                stop_source_sha256: String::new(),
                period_packed_sha256: String::new(),
                stop_packed_sha256: String::new(),
                native_metadata: serde_json::Value::Null,
            },
        };
        result.metadata.native_metadata = serde_json::to_value(
            parent
                .compile_source_end_transport(cue, prefix, result.native()?)?
                .metadata(),
        )?;
        Ok(result)
    }
    pub fn config(&self) -> SourceEndAngularConfig {
        self.metadata.config
    }
    pub fn parent_binding(
        &self,
    ) -> &uor_r4_integer::geometric_source_realizer::NativeArtifactBinding {
        &self.metadata.parent
    }
    pub fn parameters(&self) -> BTreeMap<String, Var> {
        BTreeMap::from([
            (
                "source_end.period_coefficients".into(),
                self.period_coefficients.clone(),
            ),
            (
                "source_end.stop_coefficients".into(),
                self.stop_coefficients.clone(),
            ),
        ])
    }
    pub fn project_shadow_range(&self) -> Result<()> {
        for var in [&self.period_coefficients, &self.stop_coefficients] {
            let values = var.to_vec1::<f32>()?;
            if values.iter().any(|x| !x.is_finite()) {
                return Err(invalid("nonfinite source-end shadow"));
            }
            var.set(&Tensor::from_vec(
                values
                    .into_iter()
                    .map(|x| x.clamp(-1.75, 1.75))
                    .collect::<Vec<_>>(),
                var.shape(),
                &Device::Cpu,
            )?)?;
        }
        Ok(())
    }
    pub fn period_packed_coefficients(&self) -> Result<Vec<u8>> {
        source_end_packed(&self.period_coefficients)
    }
    pub fn stop_packed_coefficients(&self) -> Result<Vec<u8>> {
        source_end_packed(&self.stop_coefficients)
    }
    pub fn native(&self) -> Result<SourceEndAngularQ4> {
        SourceEndAngularQ4::new(
            self.config(),
            &self.period_packed_coefficients()?,
            &self.stop_packed_coefficients()?,
        )
        .map_err(|e| invalid(e.to_string()))
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        let period_source = source_end_source_bytes(&self.period_coefficients)?;
        let stop_source = source_end_source_bytes(&self.stop_coefficients)?;
        let period_packed = self.period_packed_coefficients()?;
        let stop_packed = self.stop_packed_coefficients()?;
        let mut m = self.metadata.clone();
        m.period_source_sha256 = sha256_bytes(&period_source);
        m.stop_source_sha256 = sha256_bytes(&stop_source);
        m.period_packed_sha256 = sha256_bytes(&period_packed);
        m.stop_packed_sha256 = sha256_bytes(&stop_packed);
        m.native_metadata["period_packed_sha256"] = serde_json::json!(m.period_packed_sha256);
        m.native_metadata["stop_packed_sha256"] = serde_json::json!(m.stop_packed_sha256);
        fs::create_dir(path)?;
        fs::write(path.join("source-end-period-f32.bin"), period_source)?;
        fs::write(path.join("source-end-stop-f32.bin"), stop_source)?;
        fs::write(path.join("source-end-period-q4.bin"), period_packed)?;
        fs::write(path.join("source-end-stop-q4.bin"), stop_packed)?;
        fs::write(
            path.join("native-metadata.json"),
            serde_json::to_vec_pretty(&m.native_metadata)?,
        )?;
        fs::write(path.join("metadata.json"), serde_json::to_vec_pretty(&m)?)?;
        Ok(())
    }
    pub fn load(
        path: &Path,
        parent: &NativeSourceRealizer,
        native_root: &Path,
        cue_root: &Path,
        prefix_root: &Path,
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
    ) -> Result<Self> {
        let m: SourceEndAngularSourceMetadata =
            serde_json::from_slice(&fs::read(path.join("metadata.json"))?)?;
        let period_raw = fs::read(path.join("source-end-period-f32.bin"))?;
        let stop_raw = fs::read(path.join("source-end-stop-f32.bin"))?;
        let period_packed = fs::read(path.join("source-end-period-q4.bin"))?;
        let stop_packed = fs::read(path.join("source-end-stop-q4.bin"))?;
        let count = m
            .config
            .coefficient_count()
            .map_err(|e| invalid(e.to_string()))?;
        if m.schema != "uor-r4.geometric-source-end-angular-source/1"
            || m.parent != parent.artifact_binding()?
            || m.parent_native_files_sha256 != prefix_parent_files(parent, native_root)?
            || m.cue_native_files_sha256 != prefix_cue_files(cue_root, cue)?
            || m.prefix_native_files_sha256 != source_end_prefix_files(prefix_root, prefix)?
            || m.cue_native_metadata != serde_json::to_value(cue.metadata())?
            || m.prefix_native_metadata != serde_json::to_value(prefix.metadata())?
            || period_raw.len() != count * 4
            || stop_raw.len() != count * 4
            || m.period_source_sha256 != sha256_bytes(&period_raw)
            || m.stop_source_sha256 != sha256_bytes(&stop_raw)
            || m.period_packed_sha256 != sha256_bytes(&period_packed)
            || m.stop_packed_sha256 != sha256_bytes(&stop_packed)
        {
            return Err(invalid("source-end source/native/frozen bundles differ"));
        }
        let decode = |raw: &[u8]| {
            raw.chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect::<Vec<_>>()
        };
        let result = Self {
            period_coefficients: Var::from_vec(decode(&period_raw), count, &Device::Cpu)?,
            stop_coefficients: Var::from_vec(decode(&stop_raw), count, &Device::Cpu)?,
            metadata: m,
        };
        if result.period_packed_coefficients()? != period_packed
            || result.stop_packed_coefficients()? != stop_packed
        {
            return Err(invalid("source-end shadow/native replay differs"));
        }
        let carrier = parent.compile_source_end_transport(cue, prefix, result.native()?)?;
        if serde_json::to_value(carrier.metadata())? != result.metadata.native_metadata
            || serde_json::from_slice::<serde_json::Value>(&fs::read(
                path.join("native-metadata.json"),
            )?)? != result.metadata.native_metadata
        {
            return Err(invalid("source-end native metadata differs"));
        }
        Ok(result)
    }
}
pub struct SourceEndBankRealizerLoss {
    pub loss: Tensor,
    pub trace: uor_r4_integer::geometric_source_realizer::SourceEndBankRealizerTrace,
    pub target_probability: f64,
}

pub struct ObservationBankRealizerLoss {
    pub loss: Tensor,
    pub trace: uor_r4_integer::geometric_source_realizer::SourceEndBankRealizerTrace,
    pub target_probability: f64,
}

pub struct CueBankRealizerLoss {
    pub loss: Tensor,
    pub trace: uor_r4_integer::geometric_source_realizer::CueBankRealizerTrace,
    pub target_probability: f64,
}

pub struct PrefixBankRealizerLoss {
    pub loss: Tensor,
    pub trace: uor_r4_integer::geometric_source_realizer::PrefixBankRealizerTrace,
    pub target_probability: f64,
}

pub struct PreparedSourceRealizer<'a> {
    source: &'a SourceRealizerWeights,
    native: &'a NativeSourceRealizer,
    consumer: PreparedConsumerStep<'a>,
    potential_coefficient_credit: bool,
}

impl PreparedSourceRealizer<'_> {
    pub fn loss(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        target: u32,
    ) -> Result<RealizerLoss> {
        if target as usize >= self.source.binding.vocab_size() {
            return Err(invalid("realizer target is out of vocabulary"));
        }
        let trace = self.native.read(frame, view, query, prefix)?;
        let mass = trace
            .actions
            .token_masses
            .iter()
            .find(|m| m.token_id == target)
            .map_or(0, |m| m.weight_q31);
        if mass == 0 {
            return Err(invalid(
                "realizer target has zero native action mass; no parent or probability floor",
            ));
        }
        let target_probability = mass as f64 / trace.actions.total_weight_q31 as f64;
        let ids = &trace.period_context.tokens;
        let time = ids.len();
        let n = view.emitted_token_ids().len();
        let c = self.source.consumer.config();
        let width = c.heads * c.lanes_per_head;
        let context = self.consumer.context.forward(ids, 1, time, false)?;
        if !context_replay_matches(&trace.period_context, &context.trace) {
            return Err(invalid("realizer native/training context replay differs"));
        }
        let absent = AddressLane::new(1, 0, false).map_err(|e| invalid(e.to_string()))?;
        let content = vec![absent; time * width];
        let copy_coeff = self.source.consumer.potential.forward_codes(
            1,
            time,
            &content,
            &context.trace.codes,
        )?;
        let copy_context =
            frozen_potential_forward(&self.source.consumer.potential, &content, &context)?;
        if copy_coeff.scores_q24 != copy_context.scores_q24 {
            return Err(invalid("realizer Copy credit hard scores differ"));
        }
        let copy = (&copy_coeff.scores + (&copy_context.scores - copy_context.scores.detach())?)?;
        let latent = context
            .trace
            .states
            .iter()
            .flatten()
            .map(|&x| H4Code::try_from(x).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        let held = vec![H4Code::IDENTITY; latent.len()];
        let valid = vec![false; time];
        let batch = NoReadBatch {
            ids,
            batch: 1,
            time,
            latent: &latent,
            observed: &context.trace.codes,
            held: &held,
            span_valid: &valid,
        };
        let period_coeff = self.source.period.forward(batch)?;
        let stop_coeff = self.source.consumer.no_read.forward(batch)?;
        let period_context =
            frozen_no_read_forward(&self.source.period, ids, &context, &held, &valid)?;
        let stop_context =
            frozen_no_read_forward(&self.source.consumer.no_read, ids, &context, &held, &valid)?;
        let mut heads = Vec::with_capacity(c.heads);
        for h in 0..c.heads {
            let hard = &trace.source.view_kernel_trace.heads[h];
            let at = h * time + time - 1;
            if hard
                .scores_q24
                .iter()
                .enumerate()
                .any(|(k, &s)| s != copy_coeff.scores_q24[at * time + k])
                || hard.no_read_q24 != stop_context.scores_q24[at]
                || trace.period_q24[h] != period_context.scores_q24[at]
            {
                return Err(invalid(
                    "realizer native/training Copy/Period/Stop Q24 differs",
                ));
            }
            let period = scalar_credit(
                trace.period_q24[h],
                &period_coeff.i((0, h, time - 1))?,
                &period_context.scores.i((0, h, time - 1))?,
            )?;
            let stop = scalar_credit(
                hard.no_read_q24,
                &stop_coeff.i((0, h, time - 1))?,
                &stop_context.scores.i((0, h, time - 1))?,
            )?;
            heads.push(Tensor::cat(
                &[
                    copy.i((0, h, time - 1))?.narrow(0, 0, n)?,
                    period.reshape(1)?,
                    stop.reshape(1)?,
                ],
                0,
            )?);
        }
        let summed = Tensor::stack(&heads, 0)?.sum(0)?;
        let loss = marginal_action_loss(&trace.actions, &summed, target, target_probability)?;
        Ok(RealizerLoss {
            loss,
            trace,
            target_probability,
        })
    }

    /// Ordinary bank-wide alias CE. All source records are admitted by the
    /// target-free integer reader. Context cues influence recurrence but never
    /// become Copy candidates. Native probability is the exact hard forward;
    /// existing coefficient STE and source/query context credit supply adjoints.
    pub fn loss_bank(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        target: u32,
    ) -> Result<BankRealizerLoss> {
        self.loss_bank_with_context_credit(segments, query, prefix, target, true)
    }

    /// Ordinary token marginal on a nonempty factual source bank and query.
    /// No-source/empty-query behavior is outside this scoped learning adapter.
    /// Only observation-root Vars are admitted by the caller's optimizer. Frozen
    /// prefix/end choices are stopped-gradient branches, not differentiable routes.
    pub fn loss_bank_observation(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        actual_prefix: &[u32],
        target: u32,
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
        end: &NativeSourceEndTransport<'_>,
    ) -> Result<ObservationBankRealizerLoss> {
        if target as usize >= self.source.binding.vocab_size() {
            return Err(invalid("observation target out of vocabulary"));
        }
        // Every source/route/action is obtained before labels enter marginal loss.
        let trace = self.native.read_bank_with_source_end_transport(
            segments,
            query,
            actual_prefix,
            cue,
            prefix,
            end,
        )?;
        let actions = &trace.actions;
        let mass = actions
            .token_masses
            .iter()
            .find(|v| v.token_id == target)
            .map_or(0, |v| v.weight_q31);
        if mass == 0 || actions.total_weight_q31 == 0 || mass > actions.total_weight_q31 {
            return Err(invalid(
                "observation native target has zero/invalid support; no floor",
            ));
        }
        let probability = mass as f64 / actions.total_weight_q31 as f64;
        let bank = &trace.prefix_bank.cue_bank.bank;
        let carrier = &trace.prefix_bank.cue_bank.carrier;
        let ids = &bank.context.tokens;
        let time = ids.len();
        let c = self.source.consumer.config();
        let lanes = c.heads * c.lanes_per_head;
        let count = bank.candidates.len();
        if time == 0
            || count == 0
            || bank.heads.len() != c.heads
            || actions.actions.len() != count + 2
            || carrier.query.token_ids != query
        {
            return Err(invalid("observation complete bank shape/query differs"));
        }
        let context = self.consumer.context.forward(ids, 1, time, false)?;
        if !context_replay_matches(&bank.context, &context.trace) {
            return Err(invalid("observation full replay differs"));
        }
        let positions = bank
            .candidates
            .iter()
            .enumerate()
            .map(|(j, v)| {
                if v.bank_index != j || v.context_position >= time - 1 {
                    return Err(invalid(
                        "observation candidate ordinal/causal position differs",
                    ));
                }
                Ok(v.context_position as u32)
            })
            .collect::<Result<Vec<_>>>()?;
        let index = Tensor::from_vec(positions.clone(), count, &Device::Cpu)?;
        let absent = AddressLane::new(H4Code::IDENTITY.index(), 0, false)
            .map_err(|e| invalid(e.to_string()))?;
        let copy = frozen_potential_forward(
            &self.source.consumer.potential,
            &vec![absent; time * lanes],
            &context,
        )?;
        let query_output = self
            .consumer
            .context
            .forward(query, 1, query.len(), false)?;
        let mut cue_outputs = Vec::with_capacity(carrier.cues.len());
        for local in &carrier.cues {
            let previous = local
                .source_segment_index
                .checked_sub(1)
                .ok_or_else(|| invalid("observation cue predecessor absent"))?;
            if previous != local.context_segment_index {
                return Err(invalid("observation cue predecessor ordinal differs"));
            }
            let Some(SourceBankSegment::Context { token_ids, .. }) = segments.get(previous) else {
                return Err(invalid("observation cue input is not Context"));
            };
            if *token_ids != local.state.token_ids.as_slice() {
                return Err(invalid("observation cue input token IDs differ"));
            }
            if !matches!(
                segments.get(local.source_segment_index),
                Some(SourceBankSegment::Source { .. })
            ) {
                return Err(invalid("observation cue Source ordinal differs"));
            }
            cue_outputs.push(self.consumer.context.forward(
                token_ids,
                1,
                token_ids.len(),
                false,
            )?);
        }
        // The adapter checks final native states, roots, categories and codes;
        // independent identity-reset encodes are connected to current root Vars.
        let cue_credit = frozen_cue_root_forward(
            carrier.metadata.potential,
            cue.packed_coefficients(),
            carrier,
            &query_output,
            &cue_outputs,
        )?;
        if cue_credit.scores_q24 != carrier.copy_q24 {
            return Err(invalid("observation cue hard scores differ"));
        }
        let mut heads = Vec::with_capacity(c.heads);
        for h in 0..c.heads {
            if bank.heads[h].scores_q24.len() != count
                || trace
                    .prefix_bank
                    .prefix
                    .copy_q24
                    .get(h)
                    .is_none_or(|v| v.len() != count)
            {
                return Err(invalid("observation head/candidate score shape differs"));
            }
            let at = h * time + time - 1;
            for (j, &position) in positions.iter().enumerate() {
                let expected = copy.scores_q24[at * time + position as usize]
                    .checked_add(carrier.copy_q24[h][j])
                    .and_then(|v| v.checked_add(trace.prefix_bank.prefix.copy_q24[h][j]))
                    .ok_or_else(|| invalid("observation hard Copy sum overflow"))?;
                if expected != bank.heads[h].scores_q24[j] {
                    return Err(invalid("observation combined Copy hard parity differs"));
                }
            }
            heads.push(
                (copy.scores.i((0, h, time - 1))?.index_select(&index, 0)?
                    + cue_credit.scores.i(h)?)?,
            );
        }
        for j in 0..count {
            let summed = bank.heads.iter().try_fold(0i64, |sum, head| {
                sum.checked_add(head.scores_q24[j])
                    .ok_or_else(|| invalid("observation final Copy sum overflow"))
            })?;
            let original = bank
                .actions
                .actions
                .get(j)
                .ok_or_else(|| invalid("observation nested Copy action absent"))?;
            let final_action = &actions.actions[j];
            if final_action.score_q24 != summed
                || final_action.action != original.action
                || final_action.token_id != original.token_id
                || final_action.action_offset != original.action_offset
            {
                return Err(invalid("SourceEnd changed factual Copy score/identity"));
            }
        }
        let credit = Tensor::cat(
            &[
                Tensor::stack(&heads, 0)?.sum(0)?,
                Tensor::zeros(2, candle_core::DType::F32, &Device::Cpu)?,
            ],
            0,
        )?;
        // Zero terminal adjoints never zero numerical Period/Stop: the existing
        // marginal primitive anchors EVERY action to authoritative final scores.
        let loss = marginal_action_loss(actions, &credit, target, probability)?;
        Ok(ObservationBankRealizerLoss {
            loss,
            trace,
            target_probability: probability,
        })
    }

    /// Ordinary bank alias CE with the native geometric context frozen. Native
    /// codes and latent states are constants; only readout coefficient STEs
    /// carry credit. No differentiable context replay or context adjoint is built.
    /// Only cue coefficients carry credit; the complete parent bank remains
    /// factual and frozen. Targets enter after native bank+cue execution.
    pub fn loss_bank_cue(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        target: u32,
        weights: &CueAngularWeights,
        carrier: &NativeCueCarrier<'_>,
    ) -> Result<CueBankRealizerLoss> {
        if weights.parent_binding() != &self.native.artifact_binding()?
            || weights.packed_coefficients()?.as_slice() != carrier.packed_coefficients()
        {
            return Err(invalid("cue current packed/source parent differs"));
        }
        let trace = self
            .native
            .read_bank_with_cue_carrier(segments, query, prefix, carrier)?;
        if trace.carrier.metadata.potential != weights.config() {
            return Err(invalid("cue source/native angular mode differs"));
        }
        let actions = &trace.bank.actions;
        let mass = actions
            .token_masses
            .iter()
            .find(|v| v.token_id == target)
            .map_or(0, |v| v.weight_q31);
        if mass == 0 || actions.total_weight_q31 == 0 || mass > actions.total_weight_q31 {
            return Err(invalid(
                "cue native target has zero/invalid support; no floor",
            ));
        }
        let count = trace.bank.candidates.len();
        let lanes = weights.config().heads * weights.config().lanes_per_head;
        if count == 0
            || trace.carrier.angular_indices.len() != lanes
            || trace
                .carrier
                .angular_indices
                .iter()
                .any(|v| v.len() != count)
        {
            return Err(invalid("cue native coefficient address shape differs"));
        }
        let mut indices = Vec::with_capacity(lanes * count);
        let mut masks = Vec::with_capacity(lanes * count);
        for (lane, row) in trace.carrier.angular_indices.iter().enumerate() {
            for index in row {
                if index.is_some_and(|i| i >= 120) {
                    return Err(invalid("cue native angular address exceeds120"));
                }
                indices.push((lane * 120 + usize::from(index.unwrap_or(0))) as u32);
                masks.push(if index.is_some() { 1f32 } else { 0f32 });
            }
        }
        let selected = weights
            .coefficients
            .index_select(&Tensor::from_vec(indices, lanes * count, &Device::Cpu)?, 0)?;
        let mask = Tensor::from_vec(masks, lanes * count, &Device::Cpu)?;
        let copies = (selected * mask)?.reshape((lanes, count))?.sum(0)?;
        let credit = Tensor::cat(
            &[
                copies,
                Tensor::zeros(2, candle_core::DType::F32, &Device::Cpu)?,
            ],
            0,
        )?;
        let probability = mass as f64 / actions.total_weight_q31 as f64;
        let loss = marginal_action_loss(actions, &credit, target, probability)?;
        Ok(CueBankRealizerLoss {
            loss,
            trace,
            target_probability: probability,
        })
    }

    /// Only the prefix table receives credit. Parent heads, cue table and both
    /// native prefix encoders are constants; labels enter after native scoring.
    pub fn loss_bank_prefix(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        target: u32,
        weights: &PrefixAngularWeights,
        cue: &NativeCueCarrier<'_>,
        carrier: &NativePrefixTransport<'_>,
    ) -> Result<PrefixBankRealizerLoss> {
        if weights.parent_binding() != &self.native.artifact_binding()?
            || weights.packed_coefficients()?.as_slice() != carrier.packed_coefficients()
            || weights.metadata.cue_native_metadata != serde_json::to_value(cue.metadata())?
        {
            return Err(invalid("prefix current packed/source parent differs"));
        }
        let trace = self
            .native
            .read_bank_with_prefix_transport(segments, query, prefix, cue, carrier)?;
        if trace.prefix.metadata.potential != weights.config() {
            return Err(invalid("prefix source/native angular mode differs"));
        }
        let actions = &trace.cue_bank.bank.actions;
        let mass = actions
            .token_masses
            .iter()
            .find(|v| v.token_id == target)
            .map_or(0, |v| v.weight_q31);
        if mass == 0 || actions.total_weight_q31 == 0 || mass > actions.total_weight_q31 {
            return Err(invalid(
                "prefix native target has zero/invalid support; no floor",
            ));
        }
        let count = trace.cue_bank.bank.candidates.len();
        let lanes = weights.config().heads * weights.config().lanes_per_head;
        if count == 0
            || trace.prefix.angular_indices.len() != lanes
            || trace
                .prefix
                .angular_indices
                .iter()
                .any(|v| v.len() != count)
        {
            return Err(invalid("prefix native coefficient address shape differs"));
        }
        let mut indices = Vec::with_capacity(lanes * count);
        let mut masks = Vec::with_capacity(lanes * count);
        for (lane, row) in trace.prefix.angular_indices.iter().enumerate() {
            for index in row {
                if *index >= 120 {
                    return Err(invalid("prefix native angular address exceeds120"));
                }
                indices.push((lane * 120 + usize::from(*index)) as u32);
                masks.push(1f32);
            }
        }
        let selected = weights
            .coefficients
            .index_select(&Tensor::from_vec(indices, lanes * count, &Device::Cpu)?, 0)?;
        let mask = Tensor::from_vec(masks, lanes * count, &Device::Cpu)?;
        let copies = (selected * mask)?.reshape((lanes, count))?.sum(0)?;
        let credit = Tensor::cat(
            &[
                copies,
                Tensor::zeros(2, candle_core::DType::F32, &Device::Cpu)?,
            ],
            0,
        )?;
        let probability = mass as f64 / actions.total_weight_q31 as f64;
        let loss = marginal_action_loss(actions, &credit, target, probability)?;
        Ok(PrefixBankRealizerLoss {
            loss,
            trace,
            target_probability: probability,
        })
    }

    /// Learn only two endpoint tables. The factual provisional Source route is
    /// native, frozen and target-free; no source-selection adjoint is claimed.
    pub fn loss_bank_source_end(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        actual_prefix: &[u32],
        target: u32,
        weights: &SourceEndAngularWeights,
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
        end: &NativeSourceEndTransport<'_>,
    ) -> Result<SourceEndBankRealizerLoss> {
        if weights.parent_binding() != &self.native.artifact_binding()?
            || weights.period_packed_coefficients()?.as_slice() != end.period_packed_coefficients()
            || weights.stop_packed_coefficients()?.as_slice() != end.stop_packed_coefficients()
            || weights.metadata.cue_native_metadata != serde_json::to_value(cue.metadata())?
            || weights.metadata.prefix_native_metadata != serde_json::to_value(prefix.metadata())?
        {
            return Err(invalid(
                "source-end current source/native/frozen bindings differ",
            ));
        }
        let trace = self.native.read_bank_with_source_end_transport(
            segments,
            query,
            actual_prefix,
            cue,
            prefix,
            end,
        )?;
        if trace.source_end.metadata.potential != weights.config() {
            return Err(invalid("source-end angular mode/config differs"));
        }
        let actions = &trace.actions;
        let mass = actions
            .token_masses
            .iter()
            .find(|v| v.token_id == target)
            .map_or(0, |v| v.weight_q31);
        if mass == 0 || actions.total_weight_q31 == 0 || mass > actions.total_weight_q31 {
            return Err(invalid(
                "source-end target has zero/invalid native support; no floor",
            ));
        }
        let lanes = weights.config().heads * weights.config().lanes_per_head;
        if trace.source_end.angular_indices.len() != lanes {
            return Err(invalid("source-end native angular shape differs"));
        }
        let mut indices = Vec::with_capacity(lanes);
        for (lane, &index) in trace.source_end.angular_indices.iter().enumerate() {
            if index >= 120 {
                return Err(invalid("source-end angular address exceeds120"));
            }
            indices.push((lane * 120 + usize::from(index)) as u32);
        }
        let period_values = weights.period_coefficients.to_vec1::<f32>()?;
        let stop_values = weights.stop_coefficients.to_vec1::<f32>()?;
        let mut expected_period = vec![0i64; weights.config().heads];
        let mut expected_stop = expected_period.clone();
        if trace.source_end.selected_source_index.is_some() {
            for (lane, &index) in indices.iter().enumerate() {
                let head = lane / weights.config().lanes_per_head;
                expected_period[head] +=
                    ((period_values[index as usize] * 4.).round() as i64) << 22;
                expected_stop[head] += ((stop_values[index as usize] * 4.).round() as i64) << 22;
            }
        }
        if trace.source_end.period_q24 != expected_period
            || trace.source_end.stop_q24 != expected_stop
        {
            return Err(invalid("source-end native/gather hard residuals differ"));
        }
        let ids = Tensor::from_vec(indices, lanes, &Device::Cpu)?;
        let mask = Tensor::from_vec(
            vec![
                if trace.source_end.selected_source_index.is_some() {
                    1f32
                } else {
                    0f32
                };
                lanes
            ],
            lanes,
            &Device::Cpu,
        )?;
        let period = (weights.period_coefficients.index_select(&ids, 0)? * &mask)?
            .sum_all()?
            .unsqueeze(0)?;
        let stop = (weights.stop_coefficients.index_select(&ids, 0)? * &mask)?
            .sum_all()?
            .unsqueeze(0)?;
        let credit = Tensor::cat(
            &[
                Tensor::zeros(
                    trace.prefix_bank.cue_bank.bank.candidates.len(),
                    candle_core::DType::F32,
                    &Device::Cpu,
                )?,
                period,
                stop,
            ],
            0,
        )?;
        let probability = mass as f64 / actions.total_weight_q31 as f64;
        let loss = marginal_action_loss(actions, &credit, target, probability)?;
        Ok(SourceEndBankRealizerLoss {
            loss,
            trace,
            target_probability: probability,
        })
    }

    /// Actual parent+cue+prefix Copy scores remain factual. Only the existing
    /// Stop and Period shadows receive credit; labels enter after native read.
    pub fn loss_bank_terminal(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        target: u32,
        cue: &NativeCueCarrier<'_>,
        transport: &NativePrefixTransport<'_>,
    ) -> Result<PrefixBankRealizerLoss> {
        if target as usize >= self.source.binding.vocab_size() {
            return Err(invalid("terminal target out of vocabulary"));
        }
        let trace = self
            .native
            .read_bank_with_prefix_transport(segments, query, prefix, cue, transport)?;
        let bank = &trace.cue_bank.bank;
        let mass = bank
            .actions
            .token_masses
            .iter()
            .find(|m| m.token_id == target)
            .map_or(0, |m| m.weight_q31);
        let den = bank.actions.total_weight_q31;
        if mass == 0 || den == 0 || mass > den {
            return Err(invalid(
                "terminal native target zero/invalid support; no floor",
            ));
        }
        let c = self.source.consumer.config();
        let width = c.heads * c.lanes_per_head;
        let time = bank.context.tokens.len();
        if time == 0
            || bank.heads.len() != c.heads
            || bank.period_q24.len() != c.heads
            || bank.context.states.len() != time
            || bank.context.codes.len() != time * width
        {
            return Err(invalid("terminal native final-row shape differs"));
        }
        let latent = bank.context.states[time - 1]
            .iter()
            .copied()
            .map(|x| H4Code::try_from(x).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        let codes = bank.context.codes[(time - 1) * width..time * width]
            .iter()
            .map(|code| {
                AddressLane::new(code.root, code.radius_bin, code.present)
                    .map_err(|e| invalid(e.to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
        let held = vec![H4Code::IDENTITY; width];
        let valid = [false];
        let batch = NoReadBatch {
            ids: &bank.context.tokens[time - 1..],
            batch: 1,
            time: 1,
            latent: &latent,
            observed: &codes,
            held: &held,
            span_valid: &valid,
        };
        let period = self.source.period.forward(batch)?;
        let stop = self.source.consumer.no_read.forward(batch)?;
        let token = bank.context.tokens[time - 1] as usize;
        let period_hard = self
            .native
            .period
            .score(token, &latent, &codes, None)
            .map_err(|e| invalid(e.to_string()))?;
        let stop_hard = self
            .native
            .consumer
            .no_read
            .score(token, &latent, &codes, None)
            .map_err(|e| invalid(e.to_string()))?;
        let mut heads = Vec::with_capacity(c.heads);
        for h in 0..c.heads {
            if period_hard[h] != bank.period_q24[h] || stop_hard[h] != bank.heads[h].no_read_q24 {
                return Err(invalid("terminal final-row native Q24 differs"));
            }
            heads.push(Tensor::cat(
                &[
                    Tensor::zeros(bank.candidates.len(), candle_core::DType::F32, &Device::Cpu)?,
                    period.i((0, h, 0))?.reshape(1)?,
                    stop.i((0, h, 0))?.reshape(1)?,
                ],
                0,
            )?);
        }
        // Zero Copy adjoint, not zero Copy logits. marginal_action_loss anchors
        // every action to its actual final native score and alias probability.
        let credit = Tensor::stack(&heads, 0)?.sum(0)?;
        let probability = mass as f64 / den as f64;
        let loss = marginal_action_loss(&bank.actions, &credit, target, probability)?;
        Ok(PrefixBankRealizerLoss {
            loss,
            trace,
            target_probability: probability,
        })
    }

    pub fn loss_bank_readout(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        target: u32,
    ) -> Result<BankRealizerLoss> {
        self.loss_bank_with_context_credit(segments, query, prefix, target, false)
    }

    fn loss_bank_with_context_credit(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        target: u32,
        learn_context: bool,
    ) -> Result<BankRealizerLoss> {
        if target as usize >= self.source.binding.vocab_size() {
            return Err(invalid("bank realizer target is out of vocabulary"));
        }
        let trace = self.native.read_bank(segments, query, prefix)?;
        let mass = trace
            .actions
            .token_masses
            .iter()
            .find(|m| m.token_id == target)
            .map_or(0, |m| m.weight_q31);
        if mass == 0 || trace.actions.total_weight_q31 == 0 || mass > trace.actions.total_weight_q31
        {
            return Err(invalid(
                "bank target has zero/invalid native mass; no parent or probability floor",
            ));
        }
        let target_probability = mass as f64 / trace.actions.total_weight_q31 as f64;
        let ids = &trace.context.tokens;
        let time = ids.len();
        let c = self.source.consumer.config();
        let width = c.heads * c.lanes_per_head;
        if time == 0
            || trace.candidates.is_empty()
            || trace.heads.len() != c.heads
            || trace.period_q24.len() != c.heads
        {
            return Err(invalid("bank native replay/head/candidate shape differs"));
        }
        let positions = trace
            .candidates
            .iter()
            .enumerate()
            .map(|(j, candidate)| {
                if candidate.bank_index != j || candidate.context_position >= time - 1 {
                    return Err(invalid("bank candidate index/causal position differs"));
                }
                Ok(candidate.context_position as u32)
            })
            .collect::<Result<Vec<_>>>()?;
        let index = Tensor::from_vec(positions.clone(), positions.len(), &Device::Cpu)?;
        let context = if learn_context {
            let context = self.consumer.context.forward(ids, 1, time, false)?;
            if !context_replay_matches(&trace.context, &context.trace) {
                return Err(invalid("bank native/training context replay differs"));
            }
            Some(context)
        } else {
            None
        };
        let codes = trace
            .context
            .codes
            .iter()
            .map(|code| {
                AddressLane::new(code.root, code.radius_bin, code.present)
                    .map_err(|e| invalid(e.to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
        let latent = trace
            .context
            .states
            .iter()
            .flatten()
            .map(|&x| H4Code::try_from(x).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        if trace.context.heads != c.heads
            || trace.context.lanes_per_head != c.lanes_per_head
            || codes.len() != time * width
            || latent.len() != time * width
        {
            return Err(invalid("bank frozen native context shape differs"));
        }
        let absent = AddressLane::new(1, 0, false).map_err(|e| invalid(e.to_string()))?;
        let content = vec![absent; time * width];
        let copy_coeff = self
            .source
            .consumer
            .potential
            .forward_codes(1, time, &content, &codes)?;
        let copy = if let Some(context) = &context {
            let credit =
                frozen_potential_forward(&self.source.consumer.potential, &content, context)?;
            if copy_coeff.scores_q24 != credit.scores_q24 {
                return Err(invalid("bank Copy credit hard scores differ"));
            }
            (&copy_coeff.scores + (&credit.scores - credit.scores.detach())?)?
        } else {
            copy_coeff.scores.clone()
        };
        let held = vec![H4Code::IDENTITY; latent.len()];
        let valid = vec![false; time];
        let batch = NoReadBatch {
            ids,
            batch: 1,
            time,
            latent: &latent,
            observed: &codes,
            held: &held,
            span_valid: &valid,
        };
        let period_coeff = self.source.period.forward(batch)?;
        let stop_coeff = self.source.consumer.no_read.forward(batch)?;
        let period_context = context
            .as_ref()
            .map(|context| frozen_no_read_forward(&self.source.period, ids, context, &held, &valid))
            .transpose()?;
        let stop_context = context
            .as_ref()
            .map(|context| {
                frozen_no_read_forward(&self.source.consumer.no_read, ids, context, &held, &valid)
            })
            .transpose()?;
        // Independently check the terminal native scores from the SAME final
        // reported state, without generating any differentiable state credit.
        let final_latent = &latent[(time - 1) * width..time * width];
        let final_codes = &codes[(time - 1) * width..time * width];
        let period_hard = self
            .native
            .period
            .score(ids[time - 1] as usize, final_latent, final_codes, None)
            .map_err(|e| invalid(e.to_string()))?;
        let stop_hard = self
            .native
            .consumer
            .no_read
            .score(ids[time - 1] as usize, final_latent, final_codes, None)
            .map_err(|e| invalid(e.to_string()))?;
        let mut heads = Vec::with_capacity(c.heads);
        for h in 0..c.heads {
            let hard = &trace.heads[h];
            let at = h * time + time - 1;
            if hard.scores_q24.len() != positions.len()
                || hard
                    .scores_q24
                    .iter()
                    .zip(&positions)
                    .any(|(&score, &position)| {
                        score != copy_coeff.scores_q24[at * time + position as usize]
                    })
                || hard.no_read_q24 != stop_hard[h]
                || trace.period_q24[h] != period_hard[h]
            {
                return Err(invalid(
                    "bank native/training actual-position Copy/Period/Stop Q24 differs",
                ));
            }
            if period_context
                .as_ref()
                .is_some_and(|x| x.scores_q24[at] != period_hard[h])
                || stop_context
                    .as_ref()
                    .is_some_and(|x| x.scores_q24[at] != stop_hard[h])
            {
                return Err(invalid("bank terminal context credit hard scores differ"));
            }
            let period_surrogate = period_coeff.i((0, h, time - 1))?;
            let stop_surrogate = stop_coeff.i((0, h, time - 1))?;
            let period_credit = match &period_context {
                Some(x) => x.scores.i((0, h, time - 1))?,
                None => Tensor::zeros_like(&period_surrogate)?,
            };
            let stop_credit = match &stop_context {
                Some(x) => x.scores.i((0, h, time - 1))?,
                None => Tensor::zeros_like(&stop_surrogate)?,
            };
            let period = scalar_credit(trace.period_q24[h], &period_surrogate, &period_credit)?;
            let stop = scalar_credit(hard.no_read_q24, &stop_surrogate, &stop_credit)?;
            heads.push(Tensor::cat(
                &[
                    copy.i((0, h, time - 1))?.index_select(&index, 0)?,
                    period.reshape(1)?,
                    stop.reshape(1)?,
                ],
                0,
            )?);
        }
        let summed = Tensor::stack(&heads, 0)?.sum(0)?;
        let loss = marginal_action_loss(&trace.actions, &summed, target, target_probability)?;
        Ok(BankRealizerLoss {
            loss,
            trace,
            target_probability,
        })
    }

    /// Bridge-only dependent loss. All producer/stage1/source/readout factors are
    /// detached fixed copies; target is used only after both native reads finish.
    /// Native conditional latent-action mixture; only bridge logits carry credit.
    /// No target is supplied to the shared native intervention scorer.
    pub fn loss_native_route_with_native(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        target: u32,
        bridge: &FeedbackBridgeWeights,
        feedback: &uor_r4_integer::geometric_read_feedback::NativeReadFeedback,
        mode: uor_r4_integer::geometric_read_feedback::FeedbackInputMode,
    ) -> Result<NativeRouteRealizerLoss> {
        if feedback.metadata().parent_artifact != *bridge.parent_binding()
            || feedback.metadata().context != bridge.context_config()
            || feedback.value_packed() != bridge.value_packed()
            || feedback.bridge_packed() != bridge.packed_coefficients()?
            || bridge.parent_binding() != &self.native.artifact_binding()?
            || target as usize >= self.source.binding.vocab_size()
        {
            return Err(invalid(
                "native route compiled bridge/parent/target binding differs",
            ));
        }
        let started = std::time::Instant::now();
        let interventions = self
            .native
            .read_dependent_action_traces(frame, view, query, prefix, feedback, mode)?;
        let counterfactual_seconds = started.elapsed().as_secs_f64();
        let trace = interventions.factual;
        let probability = |actions: &uor_r4_integer::geometric_source_actions::ActionTrace| -> Result<(u64, f64)> {
            if actions.total_weight_q31 == 0 {
                return Err(invalid("native route action total mass is zero"));
            }
            let mass = actions.token_masses.iter().find(|m| m.token_id == target)
                .map_or(0, |m| m.weight_q31);
            if mass > actions.total_weight_q31 {
                return Err(invalid("native route target mass exceeds total"));
            }
            Ok((mass, mass as f64 / actions.total_weight_q31 as f64))
        };
        let (_, target_probability) = probability(&trace.stage2.actions)?;
        if target_probability == 0.0 || !target_probability.is_finite() {
            return Err(invalid(
                "native route factual target has zero/nonfinite support; no floor",
            ));
        }
        let logits = bridge.policy_logits(&trace.feedback)?;
        if interventions.lanes.len() != logits.len() {
            return Err(invalid("native route lane count differs"));
        }
        let mut mixtures = Vec::with_capacity(logits.len());
        let mut utilities = Vec::with_capacity(logits.len());
        for (lane, (records, policy)) in interventions.lanes.iter().zip(&logits).enumerate() {
            if records.lane != lane || records.actions.len() != 120 {
                return Err(invalid(
                    "native route exhaustive lane/action ordering differs",
                ));
            }
            let factual_action = trace.feedback.actions[lane];
            let mut masses = Vec::with_capacity(120);
            let mut totals = Vec::with_capacity(120);
            let mut probabilities = Vec::with_capacity(120);
            for (action, record) in records.actions.iter().enumerate() {
                let mut expected = trace.feedback.actions.clone();
                expected[lane] = action as u8;
                if record.action != action as u8 || record.action_vector != expected {
                    return Err(invalid("native route intervention changed another lane"));
                }
                if record.action == factual_action
                    && (record.final_actions != trace.stage2.actions
                        || record.controller_snapshot != trace.stage2_controller_snapshot)
                {
                    return Err(invalid(
                        "native route factual intervention differs from actual read",
                    ));
                }
                let (mass, u) = probability(&record.final_actions)?;
                masses.push(mass);
                totals.push(record.final_actions.total_weight_q31);
                probabilities.push(u);
            }
            let mixture =
                crate::geometric_read_feedback::native_probability_mixture(policy, &probabilities)?;
            let p = candle_nn::ops::softmax(policy, 0)?.to_vec1::<f32>()?;
            let support = p
                .iter()
                .zip(&probabilities)
                .map(|(p, u)| f64::from(*p) * u)
                .sum::<f64>();
            if support <= 0.0 || !support.is_finite() {
                return Err(invalid(
                    "native route detached mixture support is zero/nonfinite",
                ));
            }
            utilities.push(NativeRouteLaneUtility {
                lane,
                factual_action,
                target_mass_q31: masses,
                total_weight_q31: totals,
                zero_support_actions: probabilities.iter().filter(|u| **u == 0.0).count(),
                best_target_probability: probabilities.iter().copied().fold(0.0, f64::max),
                useful_actions: probabilities
                    .iter()
                    .filter(|u| **u > target_probability)
                    .count(),
                policy_adjoint: p
                    .iter()
                    .zip(&probabilities)
                    .map(|(p, u)| f64::from(*p) - f64::from(*p) * u / support)
                    .collect(),
                target_probabilities: probabilities,
                policy_logits_nat: policy.to_vec1::<f32>()?,
                policy_probabilities: p,
                mixture_probability: support,
            });
            mixtures.push(mixture);
        }
        let mean = Tensor::stack(&mixtures, 0)?.mean_all()?;
        let loss =
            crate::geometric_read_feedback::anchor_native_route_loss(target_probability, &mean)?;
        Ok(NativeRouteRealizerLoss {
            loss,
            trace,
            target_probability,
            utilities,
            counterfactual_actions: logits.len() * 120,
            counterfactual_seconds,
        })
    }

    pub fn loss_dependent(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        target: u32,
        bridge: &FeedbackBridgeWeights,
        mode: uor_r4_integer::geometric_read_feedback::FeedbackInputMode,
    ) -> Result<DependentRealizerLoss> {
        let feedback = bridge.compile()?;
        self.loss_dependent_with_native(frame, view, query, prefix, target, bridge, &feedback, mode)
    }

    /// Reuse a compiled feedback artifact across prefixes of one unchanged
    /// update. Admission compares all current packed/frozen component bytes;
    /// the caller must rebuild after changing bridge shadows.
    pub fn loss_dependent_with_native(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        target: u32,
        bridge: &FeedbackBridgeWeights,
        feedback: &uor_r4_integer::geometric_read_feedback::NativeReadFeedback,
        mode: uor_r4_integer::geometric_read_feedback::FeedbackInputMode,
    ) -> Result<DependentRealizerLoss> {
        if feedback.metadata().parent_artifact != *bridge.parent_binding()
            || feedback.metadata().context != bridge.context_config()
            || feedback.value_packed() != bridge.value_packed()
            || feedback.bridge_packed() != bridge.packed_coefficients()?
        {
            return Err(invalid(
                "dependent compiled bridge differs from current source/frozen producer",
            ));
        }
        if target as usize >= self.source.binding.vocab_size() {
            return Err(invalid("dependent target is out of vocabulary"));
        }
        if bridge.parent_binding() != &self.native.artifact_binding()? {
            return Err(invalid("dependent bridge parent differs"));
        }
        let trace = self
            .native
            .read_dependent(frame, view, query, prefix, feedback, mode)?;
        let mass = trace
            .stage2
            .actions
            .token_masses
            .iter()
            .find(|m| m.token_id == target)
            .map_or(0, |m| m.weight_q31);
        if mass == 0 {
            return Err(invalid(
                "dependent target has zero native mass; no probability floor",
            ));
        }
        let target_probability = mass as f64 / trace.stage2.actions.total_weight_q31 as f64;
        let n = view.emitted_token_ids().len();
        let time = n + 1;
        let refined = bridge.refined_latent(&trace.feedback)?;
        let snapshot = crate::geometric_context_credit::frozen_snapshot_observation(
            &self.source.consumer.context,
            &self.native.consumer.context,
            &trace.stage1.period_context,
            n,
            &trace.stage2_controller_snapshot,
            &refined,
        )?;
        let copy = crate::geometric_context_credit::frozen_snapshot_potential_forward(
            &self.source.consumer.potential,
            &snapshot,
        )?;
        let mut ids = trace.stage1.period_context.tokens[..n].to_vec();
        ids.push(trace.stage2_controller_snapshot.last_token_id);
        let stop = crate::geometric_context_credit::frozen_snapshot_no_read_forward(
            &self.source.consumer.no_read,
            &ids,
            &snapshot,
        )?;
        let period = crate::geometric_context_credit::frozen_snapshot_no_read_forward(
            &self.source.period,
            &ids,
            &snapshot,
        )?;
        let mut heads = Vec::new();
        for h in 0..bridge.context_config().heads {
            let at = h * time + time - 1;
            let hard = &trace.stage2.actions.head_scores[h];
            if hard
                .copy_q24
                .iter()
                .enumerate()
                .any(|(k, &q)| q != copy.scores_q24[at * time + k])
                || hard.stop_q24 != stop.scores_q24[at]
                || hard.period_q24 != period.scores_q24[at]
            {
                return Err(invalid(
                    "dependent final native/credit Copy Period Stop scores differ",
                ));
            }
            heads.push(Tensor::cat(
                &[
                    copy.scores.i((0, h, time - 1))?.narrow(0, 0, n)?,
                    period.scores.i((0, h, time - 1))?.reshape(1)?,
                    stop.scores.i((0, h, time - 1))?.reshape(1)?,
                ],
                0,
            )?);
        }
        let summed = Tensor::stack(&heads, 0)?.sum(0)?;
        let loss =
            marginal_action_loss(&trace.stage2.actions, &summed, target, target_probability)?;
        Ok(DependentRealizerLoss {
            loss,
            trace,
            target_probability,
        })
    }
}

fn marginal_action_loss(
    actions: &uor_r4_integer::geometric_source_actions::ActionTrace,
    summed: &Tensor,
    target: u32,
    target_probability: f64,
) -> Result<Tensor> {
    let n = actions
        .actions
        .len()
        .checked_sub(2)
        .ok_or_else(|| invalid("joint action shape differs"))?;
    if summed.to_vec1::<f32>()?.iter().any(|x| !x.is_finite()) {
        return Err(invalid("realizer summed-score surrogate is nonfinite"));
    }
    let hard = Tensor::from_vec(
        actions
            .actions
            .iter()
            .map(|a| (a.score_q24 as f64 / 16_777_216.) as f32)
            .collect::<Vec<_>>(),
        n + 2,
        &Device::Cpu,
    )?;
    let joint = (&hard + (summed - summed.detach())?)?;
    let soft = candle_nn::ops::softmax(&joint, 0)?;
    let mask = Tensor::from_vec(
        actions
            .actions
            .iter()
            .map(|a| if a.token_id == target { 1f32 } else { 0f32 })
            .collect::<Vec<_>>(),
        n + 2,
        &Device::Cpu,
    )?;
    let surrogate_probability = (soft * mask)?.sum_all()?;
    // Round the *aggregated integer token mass* once at this floating loss
    // boundary, rather than separately rounding each alias probability.
    let hard_probability = Tensor::new(target_probability as f32, &Device::Cpu)?;
    let probability =
        (&hard_probability + (&surrogate_probability - surrogate_probability.detach())?)?;
    if !probability.to_scalar::<f32>()?.is_finite() || probability.to_scalar::<f32>()? <= 0. {
        return Err(invalid(
            "realizer native probability cannot be represented by loss",
        ));
    }
    Ok(probability.log()?.neg()?)
}

fn scalar_credit(hard_q24: i64, coefficient: &Tensor, context: &Tensor) -> Result<Tensor> {
    if !coefficient.to_scalar::<f32>()?.is_finite() || !context.to_scalar::<f32>()?.is_finite() {
        return Err(invalid("realizer action score surrogate is nonfinite"));
    }
    let hard = Tensor::new((hard_q24 as f64 / 16_777_216.) as f32, &Device::Cpu)?;
    Ok((&hard + ((coefficient - coefficient.detach())? + (context - context.detach())?)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_source_emission_view::SourceEmissionCompiler;
    use std::sync::atomic::{AtomicU64, Ordering};
    use uor_r4_integer::geometric_cue_carrier::{CueJointConfig, CueJointQ4};
    use uor_r4_integer::geometric_occurrence_read::{FrameMetadata, FrameStatus, SourceIdentity};
    const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5,"Ġ":6,"Ġa":7},"merges":["Ġ a"]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        path: std::path::PathBuf,
        weights: SourceRealizerWeights,
        identity: ConsumerIdentity,
    }
    impl Fixture {
        fn new() -> Result<Self> {
            Self::new_with_lanes(1)
        }
        fn new_with_lanes(lanes: usize) -> Result<Self> {
            let path = std::env::temp_dir().join(format!(
                "uor-source-realizer-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path)?;
            let consumer =
                ConsumerWeights::initialize(&path.join("base"), TOK.as_bytes(), 8, 1, lanes, 41)?;
            let weights = SourceRealizerWeights::new(consumer, TOK.as_bytes(), 43)?;
            // Uniform hard scores give exact, independently known alias loss
            // and coefficient adjoint signs, without a model training dose.
            for var in weights.parameters().values() {
                var.set(&Tensor::zeros_like(var.as_tensor())?)?;
            }
            let identity = ConsumerIdentity {
                tokenizer_sha256: sha256_bytes(TOK.as_bytes()),
                parent_checkpoint_manifest_sha256: "a".repeat(64),
                parent_model_sha256: "b".repeat(64),
                parent_config_sha256: "c".repeat(64),
            };
            Ok(Self {
                path,
                weights,
                identity,
            })
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
    fn frame(ids: &[u32]) -> SelectedRecordFrame<'_> {
        SelectedRecordFrame {
            identity: SourceIdentity {
                record: 7,
                commit: 9,
            },
            metadata: FrameMetadata {
                scope: b"fixture",
                entity: &[],
                relation: 3,
                view: 0,
                status: FrameStatus::Found,
            },
            token_ids: ids,
        }
    }

    fn dependent_fixture() -> Result<(Fixture, NativeSourceRealizer, FeedbackBridgeWeights)> {
        let fixture = Fixture::new()?;
        let w = &fixture.weights;
        let set = |var: &Var, values: Vec<f32>| -> Result<()> {
            var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
            Ok(())
        };
        let root = &w.consumer.context.parameters()["self_root"];
        let mut values = vec![0.; root.elem_count()];
        values[0] = -1.75;
        values[4] = 1.75;
        set(root, values)?;
        let category = &w.consumer.context.parameters()["token_category"];
        let mut values = vec![0.; category.elem_count()];
        for row in values.chunks_exact_mut(33) {
            row[1] = 1.75;
        }
        set(category, values)?;
        for (weights, sign) in [(&w.consumer.no_read, 1.), (&w.period, -1.)] {
            let var = &weights.parameters()["coefficients"];
            let mut v = vec![0.; var.elem_count()];
            v[1 + weights.config().vocabulary] = sign;
            set(var, v)?;
        }
        let potential = &w.consumer.potential.parameters()["context_unary"];
        let mut values = vec![0.; potential.elem_count()];
        values[0] = 1.;
        set(potential, values)?;
        let native = w.compile(fixture.identity.clone())?;
        let path = fixture.path.join("dependent-native");
        native.save(&path)?;
        let native = NativeSourceRealizer::load(&path, w, &fixture.identity)?;
        let context = uor_r4_integer::geometric_context_q4::ContextQ4Config {
            vocab_size: 8,
            heads: 1,
            lanes_per_head: 1,
        };
        let config =
            uor_r4_integer::geometric_read_feedback::NativeReadFeedback::value_config(context);
        let mut values = Vec::new();
        for (name, shape) in config
            .coefficient_shapes()
            .map_err(|e| invalid(e.to_string()))?
        {
            let mut q = vec![0; shape.iter().product::<usize>()];
            if name == "own_root" {
                for row in q.chunks_exact_mut(120 * 4) {
                    row[0] = -7;
                    row[4] = 7;
                }
            }
            if name == "token_category" {
                for row in q.chunks_exact_mut(32) {
                    row[1] = 7;
                }
            }
            values.extend(q);
        }
        let packed = uor_r4_integer::geometric_value_q4::pack_coefficients(&values)
            .map_err(|e| invalid(e.to_string()))?;
        let count =
            uor_r4_integer::geometric_read_feedback::NativeReadFeedback::bridge_coefficient_count(
                context,
            )?;
        let feedback = uor_r4_integer::geometric_read_feedback::NativeReadFeedback::compile(
            native.artifact_binding()?,
            context,
            &packed,
            &vec![0; count.div_ceil(2)],
        )?;
        let bridge = FeedbackBridgeWeights::from_native(&feedback)?;
        Ok((fixture, native, bridge))
    }
    #[test]
    fn native_route_exact_alias_utilities_are_target_free_and_only_bridge_has_credit() -> Result<()>
    {
        let (fixture, native, bridge) = dependent_fixture()?;
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let prepared = fixture.weights.prepare(&native)?;
        let update = bridge.compile()?;
        for mode in [
            uor_r4_integer::geometric_read_feedback::FeedbackInputMode::JointCopy,
            uor_r4_integer::geometric_read_feedback::FeedbackInputMode::RoleSurface,
        ] {
            let mut previous = None;
            for target in [4, 3, 1] {
                let ordinary = prepared.loss(frame(&ids), &view, &[5], &[], target)?;
                let out = prepared.loss_native_route_with_native(
                    frame(&ids),
                    &view,
                    &[5],
                    &[],
                    target,
                    &bridge,
                    &update,
                    mode,
                )?;
                assert_eq!(out.trace.stage1, ordinary.trace);
                assert_eq!(out.trace.stage2, ordinary.trace);
                assert_eq!(
                    out.loss.to_scalar::<f32>()?.to_bits(),
                    ordinary.loss.to_scalar::<f32>()?.to_bits()
                );
                if let Some(prior) = &previous {
                    assert_eq!(prior, &out.trace);
                }
                for utility in &out.utilities {
                    assert_eq!(utility.target_probabilities.len(), 120);
                    assert_eq!(
                        utility.target_probabilities[utility.factual_action as usize],
                        out.target_probability
                    );
                    let target_mass = ordinary
                        .trace
                        .actions
                        .token_masses
                        .iter()
                        .find(|m| m.token_id == target)
                        .map_or(0, |m| m.weight_q31);
                    assert_eq!(
                        utility.target_mass_q31[utility.factual_action as usize],
                        target_mass
                    );
                }
                let grads = out.loss.backward()?;
                let var = &bridge.parameters()["bridge.coefficients"];
                let g = grads
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("route policy credit disconnected"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                assert!(g.iter().all(|g| g.is_finite()));
                for (name, var) in fixture.weights.parameters() {
                    assert!(grads.get(var.as_tensor()).is_none(), "frozen{name}");
                }
                previous = Some(out.trace);
            }
        }
        Ok(())
    }
    #[test]
    fn dependent_identity_marginal_alias_loss_matches_parent_and_relabel_is_trace_free(
    ) -> Result<()> {
        let (fixture, native, bridge) = dependent_fixture()?;
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        assert_eq!(view.emitted_token_ids(), &[7, 4, 4]);
        let prepared = fixture.weights.prepare(&native)?;
        let mut previous = None;
        for mode in [
            uor_r4_integer::geometric_read_feedback::FeedbackInputMode::JointCopy,
            uor_r4_integer::geometric_read_feedback::FeedbackInputMode::RoleSurface,
        ] {
            for target in [4, 3, 1] {
                let ordinary = prepared.loss(frame(&ids), &view, &[5], &[], target)?;
                let dependent = prepared.loss_dependent(
                    frame(&ids),
                    &view,
                    &[5],
                    &[],
                    target,
                    &bridge,
                    mode,
                )?;
                assert_eq!(dependent.trace.stage1, ordinary.trace);
                assert_eq!(dependent.trace.stage2, ordinary.trace);
                assert_eq!(dependent.target_probability, ordinary.target_probability);
                assert_eq!(
                    dependent.loss.to_scalar::<f32>()?.to_bits(),
                    ordinary.loss.to_scalar::<f32>()?.to_bits()
                );
                if let Some(t) = &previous {
                    assert_eq!(&dependent.trace, t);
                }
                previous = Some(dependent.trace);
            }
            previous = None;
        }
        assert!(prepared
            .loss_dependent(
                frame(&ids),
                &view,
                &[5],
                &[],
                2,
                &bridge,
                uor_r4_integer::geometric_read_feedback::FeedbackInputMode::JointCopy
            )
            .is_err());
        Ok(())
    }
    #[test]
    fn dependent_final_copy_period_stop_credit_reaches_only_bridge() -> Result<()> {
        let (fixture, native, bridge) = dependent_fixture()?;
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let prepared = fixture.weights.prepare(&native)?;
        for target in [4, 3, 1] {
            let out = prepared.loss_dependent(
                frame(&ids),
                &view,
                &[5],
                &[],
                target,
                &bridge,
                uor_r4_integer::geometric_read_feedback::FeedbackInputMode::JointCopy,
            )?;
            let grads = out.loss.backward()?;
            let var = &bridge.parameters()["bridge.coefficients"];
            let g = grads
                .get(var.as_tensor())
                .ok_or_else(|| invalid("bridge final CE credit absent"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            assert!(g.iter().all(|x| x.is_finite()));
            assert!(g.iter().any(|x| x.abs() > 1e-8), "target{target}");
            for (name, var) in fixture.weights.parameters() {
                assert!(grads.get(var.as_tensor()).is_none(), "frozen{name}");
            }
        }
        Ok(())
    }
    #[test]
    fn dependent_shadow_quantum_crossing_changes_actual_native_refined_snapshot() -> Result<()> {
        let (fixture, native, bridge) = dependent_fixture()?;
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let var = &bridge.parameters()["bridge.coefficients"];
        let mut values = vec![0f32; var.elem_count()];
        values[0] = 0.124;
        var.set(&Tensor::from_vec(
            values.clone(),
            var.shape(),
            &Device::Cpu,
        )?)?;
        let cached = bridge.compile()?;
        let before = native.read_dependent(
            frame(&ids),
            &view,
            &[5],
            &[],
            &cached,
            uor_r4_integer::geometric_read_feedback::FeedbackInputMode::JointCopy,
        )?;
        values[0] = 0.125;
        var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
        bridge.project_shadow_range()?;
        assert_eq!(var.flatten_all()?.to_vec1::<f32>()?[0], 0.125);
        assert!(fixture
            .weights
            .prepare(&native)?
            .loss_dependent_with_native(
                frame(&ids),
                &view,
                &[5],
                &[],
                4,
                &bridge,
                &cached,
                uor_r4_integer::geometric_read_feedback::FeedbackInputMode::JointCopy
            )
            .is_err());
        let after = native.read_dependent(
            frame(&ids),
            &view,
            &[5],
            &[],
            &bridge.compile()?,
            uor_r4_integer::geometric_read_feedback::FeedbackInputMode::JointCopy,
        )?;
        assert_eq!(before.stage1, after.stage1);
        assert_eq!(before.feedback.actions, vec![1]);
        assert_eq!(after.feedback.actions, vec![0]);
        assert_ne!(
            before.stage2_controller_snapshot.states,
            after.stage2_controller_snapshot.states
        );
        assert_ne!(
            before.stage2.actions.head_scores,
            after.stage2.actions.head_scores
        );
        Ok(())
    }

    #[test]
    fn terminal_final_row_alias_loss_matches_full_row_gradients() -> Result<()> {
        let (fixture, native, _) = dependent_fixture()?;
        let cue = native.compile_cue_carrier(
            CueAngularQ4::new(
                CueAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: CueScoreMode::DirectedRelative,
                },
                &[0x11; 60],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let transport = native.compile_prefix_transport(
            &cue,
            PrefixAngularQ4::new(
                PrefixAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: PrefixScoreMode::DirectedRelative,
                },
                &[0x11; 60],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let segments = [SourceBankSegment::Source {
            frame: frame(&ids),
            view: &view,
            event: 7,
        }];
        let prepared = fixture.weights.prepare(&native)?;
        for target in [4, 3, 1] {
            let out =
                prepared.loss_bank_terminal(&segments, &[5], &[4], target, &cue, &transport)?;
            let expected =
                native.read_bank_with_prefix_transport(&segments, &[5], &[4], &cue, &transport)?;
            assert_eq!(out.trace, expected);
            assert!(
                (f64::from(out.loss.to_scalar::<f32>()?) + out.target_probability.ln()).abs()
                    < 1e-6
            );
            let bank = &out.trace.cue_bank.bank;
            let time = bank.context.tokens.len();
            let roots = bank
                .context
                .states
                .iter()
                .flatten()
                .copied()
                .map(|x| H4Code::try_from(x).map_err(|e| invalid(e.to_string())))
                .collect::<Result<Vec<_>>>()?;
            let codes = bank
                .context
                .codes
                .iter()
                .map(|c| {
                    AddressLane::new(c.root, c.radius_bin, c.present)
                        .map_err(|e| invalid(e.to_string()))
                })
                .collect::<Result<Vec<_>>>()?;
            let held = vec![H4Code::IDENTITY; roots.len()];
            let valid = vec![false; time];
            let full = NoReadBatch {
                ids: &bank.context.tokens,
                batch: 1,
                time,
                latent: &roots,
                observed: &codes,
                held: &held,
                span_valid: &valid,
            };
            let period = fixture
                .weights
                .period
                .forward(full)?
                .i((0, 0, time - 1))?
                .reshape(1)?;
            let stop = fixture
                .weights
                .consumer
                .no_read
                .forward(full)?
                .i((0, 0, time - 1))?
                .reshape(1)?;
            let credit = Tensor::cat(
                &[
                    Tensor::zeros(bank.candidates.len(), candle_core::DType::F32, &Device::Cpu)?,
                    period,
                    stop,
                ],
                0,
            )?;
            let full_loss =
                marginal_action_loss(&bank.actions, &credit, target, out.target_probability)?;
            let a = out.loss.backward()?;
            let b = full_loss.backward()?;
            for (name, var) in fixture.weights.parameters() {
                if terminal_parameter(&name) {
                    let x = a
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid("terminal row credit absent"))?
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    let y = b
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid("terminal full credit absent"))?
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    assert!(x.iter().all(|v| v.is_finite()));
                    assert!(x.iter().any(|v| v.abs() > 1e-8));
                    assert!(x.iter().zip(y).all(|(x, y)| (*x - y).abs() < 1e-6));
                } else {
                    assert!(a.get(var.as_tensor()).is_none());
                }
            }
        }
        Ok(())
    }
    #[test]
    fn source_end_zero_alias_loss_gradients_and_bound_bundle_replay() -> Result<()> {
        let (fixture, native, _) = dependent_fixture()?;
        let native_root = fixture.path.join("dependent-native");
        let cue_root = fixture.path.join("source-end-cue");
        let prefix_root = fixture.path.join("source-end-prefix");
        let cue_weights =
            CueAngularWeights::zero(&native, &native_root, CueScoreMode::DirectedRelative)?;
        cue_weights.save(&cue_root)?;
        let cue = native.compile_cue_carrier(cue_weights.native()?)?;
        let prefix_weights = PrefixAngularWeights::zero(
            &native,
            &native_root,
            &cue_root,
            &cue,
            PrefixScoreMode::DirectedRelative,
        )?;
        prefix_weights.save(&prefix_root)?;
        let prefix = native.compile_prefix_transport(&cue, prefix_weights.native()?)?;
        let weights = SourceEndAngularWeights::zero(
            &native,
            &native_root,
            &cue_root,
            &prefix_root,
            &cue,
            &prefix,
            SourceEndScoreMode::DirectedRelative,
        )?;
        let end = native.compile_source_end_transport(&cue, &prefix, weights.native()?)?;
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let segments = [SourceBankSegment::Source {
            frame: frame(&ids),
            view: &view,
            event: 7,
        }];
        let original =
            native.read_bank_with_prefix_transport(&segments, &[5], &[4], &cue, &prefix)?;
        let prepared = fixture.weights.prepare(&native)?;
        for target in [4, 3, 1] {
            let out = prepared.loss_bank_source_end(
                &segments,
                &[5],
                &[4],
                target,
                &weights,
                &cue,
                &prefix,
                &end,
            )?;
            assert_eq!(out.trace.prefix_bank, original);
            assert_eq!(out.trace.actions, original.cue_bank.bank.actions);
            assert!(
                (f64::from(out.loss.to_scalar::<f32>()?) + out.target_probability.ln()).abs()
                    < 1e-5
            );
            let grad = out.loss.backward()?;
            for var in weights.parameters().values() {
                let values = grad
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("source-end gradient absent"))?
                    .to_vec1::<f32>()?;
                assert!(values.iter().all(|x| x.is_finite()));
                assert!(values.iter().any(|x| x.abs() > 1e-8));
            }
            let frozen_source_parameters = fixture.weights.parameters();
            let frozen_cue_parameters = cue_weights.parameters();
            let frozen_prefix_parameters = prefix_weights.parameters();
            for var in frozen_source_parameters
                .values()
                .chain(frozen_cue_parameters.values())
                .chain(frozen_prefix_parameters.values())
            {
                assert!(grad.get(var.as_tensor()).is_none());
            }
        }
        let empty = [SourceBankSegment::Context {
            token_ids: &[4],
            role: 1,
            event: 19,
        }];
        // Context-only banks are outside the retained reader's admitted
        // Source contract; the loss preserves that error without a fake target.
        assert!(prepared
            .loss_bank_source_end(&empty, &[5], &[], 3, &weights, &cue, &prefix, &end)
            .is_err());
        let root = fixture.path.join("source-end-bundle");
        weights.save(&root)?;
        let loaded = SourceEndAngularWeights::load(
            &root,
            &native,
            &native_root,
            &cue_root,
            &prefix_root,
            &cue,
            &prefix,
        )?;
        assert_eq!(
            loaded.period_packed_coefficients()?,
            weights.period_packed_coefficients()?
        );
        assert_eq!(
            loaded.stop_packed_coefficients()?,
            weights.stop_packed_coefficients()?
        );
        let mut values = weights.period_coefficients.to_vec1::<f32>()?;
        values[0] = 0.125;
        weights.period_coefficients.set(&Tensor::from_vec(
            values,
            weights.period_coefficients.shape(),
            &Device::Cpu,
        )?)?;
        let mut expected = vec![0i8; 120];
        expected[0] = 1;
        assert_eq!(
            weights.period_packed_coefficients()?,
            pack_coefficients(&expected).map_err(|e| invalid(e.to_string()))?
        );
        assert!(prepared
            .loss_bank_source_end(&segments, &[5], &[4], 4, &weights, &cue, &prefix, &end)
            .is_err());
        fs::write(prefix_root.join("prefix-q4.bin"), vec![0x11; 60])?;
        assert!(SourceEndAngularWeights::load(
            &root,
            &native,
            &native_root,
            &cue_root,
            &prefix_root,
            &cue,
            &prefix
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn terminal_compiled_inventory_matches_zero_and_changed_disk_exports() -> Result<()> {
        let (fixture, frozen, _) = dependent_fixture()?;
        for stage in 0..2 {
            if stage == 1 {
                let parameters = fixture.weights.terminal_parameters();
                let var = &parameters["period.coefficients"];
                let mut values = var.flatten_all()?.to_vec1::<f32>()?;
                values[0] += 0.25;
                var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
            }
            let current = fixture.weights.compile_terminal_rebound(&frozen)?;
            assert!(current.artifact_binding().is_err());
            fixture.weights.prepare(&current)?;
            let path = fixture.path.join(format!("compiled-inventory-{stage}"));
            current.save(&path)?;
            let loaded =
                NativeSourceRealizer::load(&path, &fixture.weights, &frozen.metadata.identity)?;
            assert_eq!(current.execution_binding()?, loaded.artifact_binding()?);
            assert_eq!(
                current.execution_binding()?.metadata_sha256,
                crate::sha256_file(&path.join("metadata.json"))?
            );
            if stage == 0 {
                assert_eq!(current.execution_binding()?, frozen.artifact_binding()?);
            } else {
                assert_ne!(current.execution_binding()?, frozen.artifact_binding()?);
            }
            // Compare nested export bytes to the original consumer serializer,
            // not merely to the shared new outer serializer.
            let consumer_path = fixture.path.join(format!("consumer-export-{stage}"));
            current.consumer.save(&consumer_path)?;
            for name in super::super::NATIVE_FILES
                .into_iter()
                .chain(["metadata.json"])
            {
                assert_eq!(
                    fs::read(consumer_path.join(name))?,
                    fs::read(path.join("consumer").join(name))?
                );
            }
            let cue = current.compile_cue_carrier(
                CueAngularQ4::new(
                    CueAngularConfig {
                        heads: 1,
                        lanes_per_head: 1,
                        mode: CueScoreMode::DirectedRelative,
                    },
                    &[0x11; 60],
                )
                .map_err(|e| invalid(e.to_string()))?,
            )?;
            let prefix = current.compile_prefix_transport(
                &cue,
                PrefixAngularQ4::new(
                    PrefixAngularConfig {
                        heads: 1,
                        lanes_per_head: 1,
                        mode: PrefixScoreMode::DirectedRelative,
                    },
                    &[0x11; 60],
                )
                .map_err(|e| invalid(e.to_string()))?,
            )?;
            let ids = [4, 4, 4];
            let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
            let segments = [SourceBankSegment::Source {
                frame: frame(&ids),
                view: &view,
                event: 7,
            }];
            let trace =
                current.read_bank_with_prefix_transport(&segments, &[5], &[4], &cue, &prefix)?;
            let prepared = fixture.weights.prepare(&current)?;
            let loss = prepared.loss_bank_terminal(&segments, &[5], &[4], 4, &cue, &prefix)?;
            assert_eq!(trace, loss.trace);
        }
        Ok(())
    }

    #[test]
    fn terminal_rebind_preserves_copy_and_rejects_stale_parent() -> Result<()> {
        let (fixture, native, _) = dependent_fixture()?;
        let old_root = fixture.path.join("dependent-native");
        let cue = native.compile_cue_carrier(
            CueAngularQ4::new(
                CueAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: CueScoreMode::DirectedRelative,
                },
                &[0x11; 60],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let prefix = native.compile_prefix_transport(
            &cue,
            PrefixAngularQ4::new(
                PrefixAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: PrefixScoreMode::DirectedRelative,
                },
                &[0x11; 60],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let segments = [SourceBankSegment::Source {
            frame: frame(&ids),
            view: &view,
            event: 7,
        }];
        let before =
            native.read_bank_with_prefix_transport(&segments, &[5], &[4], &cue, &prefix)?;
        let terminal_params = fixture.weights.terminal_parameters();
        let var = &terminal_params["period.coefficients"];
        let mut values = var.flatten_all()?.to_vec1::<f32>()?;
        values[0] += 0.25;
        var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
        let path = fixture.path.join("terminal-rebound");
        let receipt = fixture
            .weights
            .save_terminal_rebound(&path, &native, &old_root, &cue, &prefix)?;
        assert_ne!(receipt.old_parent, receipt.new_parent);
        assert_eq!(
            receipt.cue_packed_sha256,
            sha256_bytes(cue.packed_coefficients())
        );
        assert_eq!(
            receipt.prefix_packed_sha256,
            sha256_bytes(prefix.packed_coefficients())
        );
        let source = SourceRealizerWeights::load_source(&path.join("source"), TOK.as_bytes())?;
        let parent = NativeSourceRealizer::load(&path.join("native"), &source, &fixture.identity)?;
        assert!(parent
            .read_bank_with_prefix_transport(&segments, &[5], &[4], &cue, &prefix)
            .is_err());
        let newcue = parent.compile_cue_carrier(
            CueAngularQ4::new(
                cue.metadata().potential,
                &fs::read(path.join("cue/cue-q4.bin"))?,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let newprefix = parent.compile_prefix_transport(
            &newcue,
            PrefixAngularQ4::new(
                prefix.metadata().potential,
                &fs::read(path.join("prefix/prefix-q4.bin"))?,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        assert_eq!(
            serde_json::to_value(newcue.metadata())?,
            serde_json::from_slice::<serde_json::Value>(&fs::read(
                path.join("cue/native-metadata.json")
            )?)?
        );
        assert_eq!(
            serde_json::to_value(newprefix.metadata())?,
            serde_json::from_slice::<serde_json::Value>(&fs::read(
                path.join("prefix/native-metadata.json")
            )?)?
        );
        let after =
            parent.read_bank_with_prefix_transport(&segments, &[5], &[4], &newcue, &newprefix)?;
        verify_terminal_copy_frozen(&before, &after)?;
        assert_ne!(
            before.cue_bank.bank.period_q24,
            after.cue_bank.bank.period_q24
        );
        let context = &fixture.weights.consumer.context.parameters()["token_category"];
        let mut changed = context.flatten_all()?.to_vec1::<f32>()?;
        changed[0] += 0.25;
        context.set(&Tensor::from_vec(changed, context.shape(), &Device::Cpu)?)?;
        assert!(fixture
            .weights
            .save_terminal_rebound(
                &fixture.path.join("bad-nonterminal"),
                &native,
                &old_root,
                &cue,
                &prefix
            )
            .is_err());
        Ok(())
    }
    #[test]
    fn prefix_zero_alias_loss_credit_and_native_bundle_binding() -> Result<()> {
        let (fixture, native, _) = dependent_fixture()?;
        let parent = fixture.path.join("dependent-native");
        let cue_weights =
            CueAngularWeights::zero(&native, &parent, CueScoreMode::DirectedRelative)?;
        let cue_path = fixture.path.join("prefix-frozen-cue");
        cue_weights.save(&cue_path)?;
        let cue = native.compile_cue_carrier(cue_weights.native()?)?;
        let weights = PrefixAngularWeights::zero(
            &native,
            &parent,
            &cue_path,
            &cue,
            PrefixScoreMode::DirectedRelative,
        )?;
        let transport = native.compile_prefix_transport(&cue, weights.native()?)?;
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let segments = [SourceBankSegment::Source {
            frame: frame(&ids),
            view: &view,
            event: 7,
        }];
        let prepared = fixture.weights.prepare(&native)?;
        for target in [4, 3, 1] {
            let out = prepared.loss_bank_prefix(
                &segments,
                &[5],
                &[],
                target,
                &weights,
                &cue,
                &transport,
            )?;
            assert_eq!(
                out.trace.cue_bank,
                native.read_bank_with_cue_carrier(&segments, &[5], &[], &cue)?
            );
            assert!(
                (f64::from(out.loss.to_scalar::<f32>()?) + out.target_probability.ln()).abs()
                    < 1e-6
            );
            let grad = out.loss.backward()?;
            for var in fixture
                .weights
                .parameters()
                .values()
                .chain(cue_weights.parameters().values())
            {
                assert!(grad.get(var.as_tensor()).is_none());
            }
            let g = grad
                .get(weights.coefficients.as_tensor())
                .ok_or_else(|| invalid("prefix credit missing"))?
                .to_vec1::<f32>()?;
            assert!(g.iter().all(|x| x.is_finite()));
            assert!(g.iter().any(|x| x.abs() > 1e-8));
        }
        let path = fixture.path.join("prefix-bundle");
        weights.save(&path)?;
        let restored = PrefixAngularWeights::load(&path, &native, &parent, &cue_path, &cue)?;
        assert_eq!(
            restored.packed_coefficients()?,
            weights.packed_coefficients()?
        );
        let mut values = weights.coefficients.to_vec1::<f32>()?;
        values[0] = 0.25;
        weights.coefficients.set(&Tensor::from_vec(
            values,
            weights.coefficients.shape(),
            &Device::Cpu,
        )?)?;
        assert!(prepared
            .loss_bank_prefix(&segments, &[5], &[], 4, &weights, &cue, &transport)
            .is_err());
        fs::write(cue_path.join("cue-q4.bin"), [1u8])?;
        assert!(PrefixAngularWeights::load(&path, &native, &parent, &cue_path, &cue).is_err());
        Ok(())
    }
    #[test]
    fn cue_loss_zero_replay_alias_credit_and_frozen_parent_graph() -> Result<()> {
        let (fixture, native, _) = dependent_fixture()?;
        let cue = CueAngularWeights::zero(
            &native,
            &fixture.path.join("dependent-native"),
            CueScoreMode::DirectedRelative,
        )?;
        let carrier = native.compile_cue_carrier(cue.native()?)?;
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let segments = [
            SourceBankSegment::Context {
                token_ids: &[5],
                role: 1,
                event: 42,
            },
            SourceBankSegment::Source {
                frame: frame(&ids),
                view: &view,
                event: 7,
            },
        ];
        let prepared = fixture.weights.prepare(&native)?;
        for target in [4, 3, 1] {
            let out = prepared.loss_bank_cue(&segments, &[5], &[], target, &cue, &carrier)?;
            assert_eq!(out.trace.bank, native.read_bank(&segments, &[5], &[])?);
            assert_eq!(out.trace.carrier.angular_indices.len(), 1);
            assert!(out.trace.carrier.angular_indices[0]
                .iter()
                .all(Option::is_some));
            assert!(
                (f64::from(out.loss.to_scalar::<f32>()?) + out.target_probability.ln()).abs()
                    < 1e-6
            );
            let grads = out.loss.backward()?;
            for var in fixture.weights.parameters().values() {
                assert!(grads.get(var.as_tensor()).is_none());
            }
            let g = grads
                .get(cue.coefficients.as_tensor())
                .ok_or_else(|| invalid("cue credit missing"))?
                .to_vec1::<f32>()?;
            assert!(g.iter().all(|v| v.is_finite()));
            assert!(g.iter().any(|v| v.abs() > 1e-8));
        }
        Ok(())
    }
    #[test]
    fn cue_quantum_bundle_and_stale_source_mode_are_checked() -> Result<()> {
        let (fixture, native, _) = dependent_fixture()?;
        let parent = fixture.path.join("dependent-native");
        let cue = CueAngularWeights::zero(&native, &parent, CueScoreMode::DirectedRelative)?;
        let path = fixture.path.join("cue-bundle");
        cue.save(&path)?;
        let restored = CueAngularWeights::load(&path, &native, &parent)?;
        assert_eq!(cue.packed_coefficients()?, restored.packed_coefficients()?);
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let segments = [
            SourceBankSegment::Context {
                token_ids: &[5],
                role: 1,
                event: 42,
            },
            SourceBankSegment::Source {
                frame: frame(&ids),
                view: &view,
                event: 7,
            },
        ];
        let old = native.compile_cue_carrier(cue.native()?)?;
        let before = native.read_bank_with_cue_carrier(&segments, &[5], &[], &old)?;
        let index = usize::from(
            before.carrier.angular_indices[0][0]
                .ok_or_else(|| invalid("fixture must have active cue"))?,
        );
        let mut values = vec![0f32; cue.coefficients.elem_count()];
        values[index] = 0.125;
        cue.coefficients.set(&Tensor::from_vec(
            values,
            cue.coefficients.shape(),
            &Device::Cpu,
        )?)?;
        cue.project_shadow_range()?;
        assert!(fixture
            .weights
            .prepare(&native)?
            .loss_bank_cue(&segments, &[5], &[], 1, &cue, &old)
            .is_err());
        let current = native.compile_cue_carrier(cue.native()?)?;
        let after = native.read_bank_with_cue_carrier(&segments, &[5], &[], &current)?;
        assert_eq!(before.bank.context, after.bank.context);
        assert_ne!(before.bank.actions, after.bank.actions);
        fs::write(
            path.join("cue-q4.bin"),
            vec![0x11u8; cue.packed_coefficients()?.len()],
        )?;
        assert!(CueAngularWeights::load(&path, &native, &parent).is_err());
        Ok(())
    }
    #[test]
    fn cue_structural_absence_keeps_zero_scores_and_zero_credit() -> Result<()> {
        let (fixture, native, _) = dependent_fixture()?;
        let cue = CueAngularWeights::zero(
            &native,
            &fixture.path.join("dependent-native"),
            CueScoreMode::CueUnary,
        )?;
        let carrier = native.compile_cue_carrier(cue.native()?)?;
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let segments = [SourceBankSegment::Source {
            frame: frame(&ids),
            view: &view,
            event: 7,
        }];
        let out = fixture.weights.prepare(&native)?.loss_bank_cue(
            &segments,
            &[5],
            &[],
            1,
            &cue,
            &carrier,
        )?;
        assert_eq!(out.trace.bank, native.read_bank(&segments, &[5], &[])?);
        assert!(out.trace.carrier.angular_indices[0]
            .iter()
            .all(Option::is_none));
        let g = out.loss.backward()?;
        let credit = g
            .get(cue.coefficients.as_tensor())
            .ok_or_else(|| invalid("masked cue graph missing"))?
            .to_vec1::<f32>()?;
        assert!(credit.iter().all(|x| *x == 0.));
        Ok(())
    }

    #[test]
    fn bank_loss_single_source_and_interleaved_global_aliases_preserve_native() -> Result<()> {
        let fixture = Fixture::new()?;
        let w = &fixture.weights;
        let native = w.compile(fixture.identity.clone())?;
        let compiler = SourceEmissionCompiler::new(TOK.as_bytes())?;
        let original = [4, 4, 4];
        let view = compiler.compile(&original)?;
        let single = [SourceBankSegment::Source {
            frame: frame(&original),
            view: &view,
            event: 42,
        }];
        let prepared = w.prepare(&native)?;
        for target in [4, 3, 1] {
            let old = prepared.loss(frame(&original), &view, &[5], &[3], target)?;
            let bank = prepared.loss_bank(&single, &[5], &[3], target)?;
            assert_eq!(bank.trace.actions, old.trace.actions);
            assert_eq!(bank.trace.context, old.trace.period_context);
            assert_eq!(bank.target_probability, old.target_probability);
            assert_eq!(bank.loss.to_scalar::<f32>()?, old.loss.to_scalar::<f32>()?);
            let old_g = old.loss.backward()?;
            let bank_g = bank.loss.backward()?;
            for name in ["period.coefficients", "consumer.no_read.coefficients"] {
                let vars = w.parameters();
                let var = vars
                    .get(name)
                    .ok_or_else(|| invalid("fixture variable absent"))?;
                let a = old_g
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("old gradient absent"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let b = bank_g
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid("bank gradient absent"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                assert_eq!(a, b);
            }
        }
        let second = [4, 4];
        let view2 = compiler.compile(&second)?;
        let mut second_frame = frame(&second);
        second_frame.identity = SourceIdentity {
            record: 8,
            commit: 11,
        };
        second_frame.metadata.relation = 4;
        let bank = [
            SourceBankSegment::Context {
                token_ids: &[5, 3],
                role: 1,
                event: 42,
            },
            SourceBankSegment::Source {
                frame: frame(&original),
                view: &view,
                event: 42,
            },
            SourceBankSegment::Context {
                token_ids: &[3],
                role: 1,
                event: 7,
            },
            SourceBankSegment::Source {
                frame: second_frame,
                view: &view2,
                event: 7,
            },
        ];
        for (target, p) in [(4, 3. / 7.), (7, 2. / 7.), (3, 1. / 7.), (1, 1. / 7.)] {
            let output = prepared.loss_bank(&bank, &[5], &[3], target)?;
            assert_eq!(
                output
                    .trace
                    .candidates
                    .iter()
                    .map(|c| c.context_position)
                    .collect::<Vec<_>>(),
                [2, 3, 4, 6, 7]
            );
            assert_eq!(
                output
                    .trace
                    .candidates
                    .iter()
                    .map(|c| c.event)
                    .collect::<Vec<_>>(),
                [42, 42, 42, 7, 7]
            );
            assert_eq!(output.trace.context.tokens, [5, 3, 7, 4, 4, 3, 7, 4, 5, 3]);
            assert_eq!(output.trace.actions.actions.len(), 7);
            assert_eq!(output.target_probability, p);
            assert!((f64::from(output.loss.to_scalar::<f32>()?) + p.ln()).abs() < 1e-6);
        }
        assert!(prepared.loss_bank(&bank, &[5], &[], 5).is_err());
        assert!(prepared.loss_bank_readout(&bank, &[5], &[], 5).is_err());
        Ok(())
    }
    #[test]
    fn bank_loss_actual_positions_and_copy_terminal_context_credit_are_connected() -> Result<()> {
        let (fixture, _, _) = dependent_fixture()?;
        let w = &fixture.weights;
        // Break the zero-transition alternating pattern on cue/query token5,
        // making source positions carry distinct signed latent/readout roots.
        let transition = w
            .consumer
            .context
            .parameters()
            .get("token_transition")
            .ok_or_else(|| invalid("token transition absent"))?;
        let mut values = vec![0f32; transition.elem_count()];
        values[5 * 120 + 1] = 1.75;
        transition.set(&Tensor::from_vec(values, transition.shape(), &Device::Cpu)?)?;
        let native = w.compile(fixture.identity.clone())?;
        let ids = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let segments = [
            SourceBankSegment::Context {
                token_ids: &[5, 3],
                role: 1,
                event: 42,
            },
            SourceBankSegment::Source {
                frame: frame(&ids),
                view: &view,
                event: 7,
            },
        ];
        let prepared = w.prepare(&native)?;
        let mut any_context = false;
        for target in [4, 3, 1] {
            let output = prepared.loss_bank(&segments, &[5], &[3], target)?;
            assert_eq!(
                output
                    .trace
                    .candidates
                    .iter()
                    .map(|c| c.context_position)
                    .collect::<Vec<_>>(),
                [2, 3, 4]
            );
            let readout = prepared.loss_bank_readout(&segments, &[5], &[3], target)?;
            assert_eq!(readout.trace, output.trace);
            assert_eq!(readout.target_probability, output.target_probability);
            assert_eq!(
                readout.loss.to_scalar::<f32>()?,
                output.loss.to_scalar::<f32>()?
            );
            let grads = output.loss.backward()?;
            let readout_grads = readout.loss.backward()?;
            let vars = w.parameters();
            for (name, var) in &vars {
                if name.starts_with("consumer.context.") {
                    assert!(
                        readout_grads.get(var.as_tensor()).is_none(),
                        "context graph retained: {name}"
                    );
                } else {
                    let a = grads.get(var.as_tensor());
                    let b = readout_grads.get(var.as_tensor());
                    assert_eq!(
                        a.is_some(),
                        b.is_some(),
                        "readout connectivity differs: {name}"
                    );
                    if let (Some(a), Some(b)) = (a, b) {
                        let a = a.flatten_all()?.to_vec1::<f32>()?;
                        let b = b.flatten_all()?.to_vec1::<f32>()?;
                        assert_eq!(a.len(), b.len());
                        for (&x, &y) in a.iter().zip(&b) {
                            assert!(x.is_finite() && y.is_finite());
                            assert!(
                                (x - y).abs() <= 2e-6 + 2e-5 * x.abs().max(y.abs()),
                                "readout adjoint differs: {name} target{target}: {x} vs {y}"
                            );
                        }
                    }
                }
            }
            for name in [
                "consumer.potential.context_unary",
                "consumer.no_read.coefficients",
                "period.coefficients",
            ] {
                let var = vars
                    .get(name)
                    .ok_or_else(|| invalid(format!("missing {name}")))?;
                let g = grads
                    .get(var.as_tensor())
                    .ok_or_else(|| invalid(format!("disconnected {name}")))?
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                assert!(g.iter().all(|x| x.is_finite()));
                assert!(g.iter().any(|x| x.abs() > 1e-9), "{name} target{target}");
            }
            for (name, var) in &vars {
                if name.starts_with("consumer.context.") {
                    let g = grads
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid(format!("disconnected {name}")))?
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    assert!(g.iter().all(|x| x.is_finite()));
                    any_context |= g.iter().any(|x| x.abs() > 1e-9);
                }
            }
        }
        assert!(any_context);
        Ok(())
    }

    #[test]
    fn source_realizer_copy_period_stop_loss_aliases_and_repeated_backward() -> Result<()> {
        let fixture = Fixture::new()?;
        let w = &fixture.weights;
        let native = w.compile(fixture.identity.clone())?;
        let original = [4, 4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&original)?;
        assert_eq!(view.emitted_token_ids(), &[7, 4, 4]);
        let expected = native.read(frame(&original), &view, &[5], &[])?;
        let prepared = w.prepare(&native)?;
        for (target, p) in [(4, 0.4f64), (3, 0.2), (1, 0.2)] {
            let output = prepared.loss(frame(&original), &view, &[5], &[], target)?;
            assert_eq!(output.trace, expected);
            assert_eq!(output.target_probability, p);
            assert!((f64::from(output.loss.to_scalar::<f32>()?) + p.ln()).abs() < 1e-6);
            let grad = output.loss.backward()?;
            for (name, selected) in [
                ("period.coefficients", target == 3),
                ("consumer.no_read.coefficients", target == 1),
            ] {
                let variable = w
                    .parameters()
                    .remove(name)
                    .ok_or_else(|| invalid("fixture parameter absent"))?;
                let bias = grad
                    .get(variable.as_tensor())
                    .ok_or_else(|| invalid("realizer action gradient disconnected"))?
                    .flatten_all()?
                    .to_vec1::<f32>()?[0];
                let expected = if selected { -0.8 } else { 0.2 };
                assert!(
                    (bias - expected).abs() < 1e-5,
                    "{name} target{target}: {bias}"
                );
            }
        }
        assert!(prepared
            .loss(frame(&original), &view, &[5], &[], 5)
            .is_err());
        // A changed output prefix is consumed by context and never added as a
        // copy candidate. No forced end/period phase is supplied to this API.
        let later = native.read(frame(&original), &view, &[5], &[3, 7])?;
        assert_eq!(later.period_context.tokens, [7, 4, 4, 5, 3, 7]);
        assert_eq!(later.actions.actions.len(), expected.actions.actions.len());
        drop(prepared);
        // Isolate Period-to-context credit: Copy and Stop coefficients stay
        // zero, and only one retained-root coordinate enters Period. Its
        // coefficient is fixed before taking ordinary Period-token CE.
        let period = w
            .period
            .parameters()
            .get("coefficients")
            .ok_or_else(|| invalid("fixture Period absent"))?;
        let mut values = period.flatten_all()?.to_vec1::<f32>()?;
        values[1 + w.consumer.config().vocab_size + 1] = 0.25;
        period.set(&Tensor::from_vec(values, period.shape(), &Device::Cpu)?)?;
        let current = w.compile(fixture.identity.clone())?;
        let output = w
            .prepare(&current)?
            .loss(frame(&original), &view, &[5], &[], 3)?;
        let grad = output.loss.backward()?;
        let transition = w
            .parameters()
            .remove("consumer.context.token_transition")
            .ok_or_else(|| invalid("fixture transition absent"))?;
        let credit = grad
            .get(transition.as_tensor())
            .ok_or_else(|| invalid("Period-to-context credit disconnected"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(credit[7 * 120..8 * 120].iter().any(|x| x.abs() > 1e-9));
        Ok(())
    }

    #[test]
    fn source_realizer_source_and_native_reload_reject_tamper_and_stale_shadows() -> Result<()> {
        let fixture = Fixture::new()?;
        let source_path = fixture.path.join("source");
        let native_path = fixture.path.join("native");
        fixture.weights.save_source(&source_path)?;
        let w = SourceRealizerWeights::load_source(&source_path, TOK.as_bytes())?;
        let native = w.compile(fixture.identity.clone())?;
        native.save(&native_path)?;
        let loaded = NativeSourceRealizer::load(&native_path, &w, &fixture.identity)?;
        let original = [4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&original)?;
        assert_eq!(
            loaded.read(frame(&original), &view, &[5], &[])?,
            native.read(frame(&original), &view, &[5], &[])?
        );
        let file = native_path.join("period-q4.bin");
        let bytes = fs::read(&file)?;
        let mut changed = bytes.clone();
        changed[0] ^= 1;
        fs::write(&file, &changed)?;
        // Even a resealed file digest cannot authorize coefficients absent
        // from the independently loaded source.
        let meta_path = native_path.join("metadata.json");
        let meta_bytes = fs::read(&meta_path)?;
        let mut meta: NativeMetadata = serde_json::from_slice(&meta_bytes)?;
        meta.files
            .insert("period-q4.bin".into(), sha256_bytes(&changed));
        fs::write(&meta_path, serde_json::to_vec(&meta)?)?;
        assert!(NativeSourceRealizer::load(&native_path, &w, &fixture.identity).is_err());
        fs::write(&file, bytes)?;
        fs::write(&meta_path, &meta_bytes)?;
        fs::write(native_path.join("unexpected.bin"), b"extra")?;
        assert!(NativeSourceRealizer::load(&native_path, &w, &fixture.identity).is_err());
        fs::remove_file(native_path.join("unexpected.bin"))?;
        let period = w
            .period
            .parameters()
            .get("coefficients")
            .ok_or_else(|| invalid("fixture Period absent"))?;
        let mut values = period.flatten_all()?.to_vec1::<f32>()?;
        values[0] = 0.01;
        period.set(&Tensor::from_vec(values, period.shape(), &Device::Cpu)?)?;
        assert!(w.prepare(&native).is_err()); // Same q4 bin still has changed source bits.
        let fresh = w.compile(fixture.identity.clone())?;
        w.prepare(&fresh)?;
        let source_meta = source_path.join("metadata.json");
        let mut metadata: SourceMetadata = serde_json::from_slice(&fs::read(&source_meta)?)?;
        metadata.policy = "unknown".into();
        fs::write(source_meta, serde_json::to_vec(&metadata)?)?;
        assert!(SourceRealizerWeights::load_source(&source_path, TOK.as_bytes()).is_err());
        Ok(())
    }
    #[test]
    fn observation_inventory_admits_only_root_values_with_fixed_shapes() -> Result<()> {
        let identity = ParameterIdentity {
            shape: vec![1],
            f32_sha256: "a".repeat(64),
        };
        let mut frozen = BTreeMap::new();
        for name in [
            "consumer.context.token_root",
            "consumer.context.self_root",
            "consumer.context.neighbor_root",
            "consumer.context.token_category",
            "consumer.context.token_transition",
            "period.coefficients",
        ] {
            frozen.insert(name.to_string(), identity.clone());
        }
        let mut current = frozen.clone();
        for name in [
            "consumer.context.token_root",
            "consumer.context.self_root",
            "consumer.context.neighbor_root",
        ] {
            current
                .get_mut(name)
                .ok_or_else(|| invalid("fixture key absent"))?
                .f32_sha256 = "b".repeat(64);
        }
        verify_observation_inventory(&current, &frozen)?;
        for name in [
            "consumer.context.token_category",
            "consumer.context.token_transition",
            "period.coefficients",
        ] {
            let mut changed = current.clone();
            changed
                .get_mut(name)
                .ok_or_else(|| invalid("fixture key absent"))?
                .f32_sha256 = "c".repeat(64);
            assert!(verify_observation_inventory(&changed, &frozen).is_err());
        }
        current
            .get_mut("consumer.context.token_root")
            .ok_or_else(|| invalid("fixture key absent"))?
            .shape = vec![2];
        assert!(verify_observation_inventory(&current, &frozen).is_err());
        assert!(!observation_root_parameter(
            "consumer.context.token_root_extra"
        ));
        Ok(())
    }
    #[test]
    fn observation_rebound_retains_loaded_guard_and_matches_actual_saved_inventory() -> Result<()> {
        let (fixture, frozen, _) = dependent_fixture()?;
        let zero = fixture.weights.compile_observation_rebound(&frozen)?;
        assert_eq!(zero.execution_binding()?, frozen.artifact_binding()?);
        let roots = fixture.weights.observation_root_parameters();
        let var = roots
            .get("consumer.context.token_root")
            .ok_or_else(|| invalid("fixture root absent"))?;
        let mut values = var.flatten_all()?.to_vec1::<f32>()?;
        values[0] += 0.25;
        var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
        assert!(fixture.weights.compile_terminal_rebound(&frozen).is_err());
        let current = fixture.weights.compile_observation_rebound(&frozen)?;
        assert!(current.artifact_binding().is_err());
        fixture.weights.prepare(&current)?;
        let path = fixture.path.join("observation-inventory");
        current.save(&path)?;
        let loaded = NativeSourceRealizer::load(&path, &fixture.weights, &fixture.identity)?;
        assert_eq!(current.execution_binding()?, loaded.artifact_binding()?);
        assert_ne!(current.execution_binding()?, frozen.artifact_binding()?);
        let forbidden = fixture.weights.parameters();
        let var = forbidden
            .get("period.coefficients")
            .ok_or_else(|| invalid("fixture terminal absent"))?;
        let mut values = var.flatten_all()?.to_vec1::<f32>()?;
        values[0] += 0.25;
        var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
        assert!(fixture
            .weights
            .compile_observation_rebound(&frozen)
            .is_err());
        Ok(())
    }

    #[test]
    fn observation_loss_anchors_final_source_end_period_mass_and_zeroes_terminal_credit(
    ) -> Result<()> {
        let (fixture, native, _) = dependent_fixture()?;
        let c = native.consumer.context.config();
        let n = c.heads * c.lanes_per_head * 120;
        let zero = pack_coefficients(&vec![0; n]).map_err(|e| invalid(e.to_string()))?;
        let cue = native.compile_cue_carrier(
            CueAngularQ4::new(
                CueAngularConfig {
                    heads: c.heads,
                    lanes_per_head: c.lanes_per_head,
                    mode: CueScoreMode::DirectedRelative,
                },
                &zero,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let prefix = native.compile_prefix_transport(
            &cue,
            PrefixAngularQ4::new(
                PrefixAngularConfig {
                    heads: c.heads,
                    lanes_per_head: c.lanes_per_head,
                    mode: PrefixScoreMode::DirectedRelative,
                },
                &zero,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let period = pack_coefficients(&vec![4; n]).map_err(|e| invalid(e.to_string()))?;
        let end = native.compile_source_end_transport(
            &cue,
            &prefix,
            SourceEndAngularQ4::new(
                SourceEndAngularConfig {
                    heads: c.heads,
                    lanes_per_head: c.lanes_per_head,
                    mode: SourceEndScoreMode::DirectedRelative,
                },
                &period,
                &zero,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let ids = [4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let segments = [SourceBankSegment::Source {
            frame: frame(&ids),
            view: &view,
            event: 7,
        }];
        let baseline =
            native.read_bank_with_prefix_transport(&segments, &[5], &[4], &cue, &prefix)?;
        let prepared = fixture.weights.prepare(&native)?;
        let out = prepared.loss_bank_observation(&segments, &[5], &[4], 3, &cue, &prefix, &end)?;
        assert!(out.trace.source_end.selected_source_index.is_some());
        assert_ne!(out.trace.actions, baseline.cue_bank.bank.actions);
        let expected = -(out.target_probability as f32).ln();
        assert!((out.loss.to_scalar::<f32>()? - expected).abs() < 1e-6);
        let mass = out
            .trace
            .actions
            .token_masses
            .iter()
            .find(|v| v.token_id == 3)
            .ok_or_else(|| invalid("fixture final Period unsupported"))?
            .weight_q31;
        assert_eq!(
            out.target_probability,
            mass as f64 / out.trace.actions.total_weight_q31 as f64
        );
        let gradients = out.loss.backward()?;
        for (_, var) in fixture.weights.terminal_parameters() {
            assert!(gradients.get(var.as_tensor()).is_none());
        }
        assert!(fixture
            .weights
            .observation_root_parameters()
            .values()
            .any(|v| gradients.get(v.as_tensor()).is_some()));
        Ok(())
    }
    #[test]
    fn observation_root_change_preserves_actual_latent_and_category_replay() -> Result<()> {
        let (fixture, frozen, _) = dependent_fixture()?;
        let ids = [4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let before = frozen.read(frame(&ids), &view, &[5], &[4])?;
        let var = fixture
            .weights
            .observation_root_parameters()
            .get("consumer.context.token_root")
            .cloned()
            .ok_or_else(|| invalid("fixture root absent"))?;
        let lanes = frozen.consumer.context.config().heads
            * frozen.consumer.context.config().lanes_per_head;
        let mut values = var.flatten_all()?.to_vec1::<f32>()?;
        // Both ±identity winners receive a negative bias; root3 wins for
        // either latent polarity without changing recurrence or categories.
        values[5 * lanes * 120] = -1.75;
        values[5 * lanes * 120 + 1] = -1.75;
        values[5 * lanes * 120 + 3] = 1.75;
        var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
        let current = fixture.weights.compile_observation_rebound(&frozen)?;
        let after = current.read(frame(&ids), &view, &[5], &[4])?;
        assert_eq!(before.period_context.states, after.period_context.states);
        assert_eq!(before.period_context.actions, after.period_context.actions);
        assert_eq!(
            before.period_context.categories,
            after.period_context.categories
        );
        assert_ne!(
            before.period_context.raw_roots,
            after.period_context.raw_roots
        );
        let c = frozen.consumer.context.config();
        let zero = pack_coefficients(&vec![0; c.heads * c.lanes_per_head * 120])
            .map_err(|e| invalid(e.to_string()))?;
        let stale = frozen.compile_cue_carrier(
            CueAngularQ4::new(
                CueAngularConfig {
                    heads: c.heads,
                    lanes_per_head: c.lanes_per_head,
                    mode: CueScoreMode::DirectedRelative,
                },
                &zero,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let segments = [SourceBankSegment::Source {
            frame: frame(&ids),
            view: &view,
            event: 7,
        }];
        assert!(current
            .read_bank_with_cue_carrier(&segments, &[5], &[4], &stale)
            .is_err());
        Ok(())
    }

    // Replaces only the contextual state-credit term. The independent potential
    // coefficient graph, cue/prefix graphs, Generate graph and native pool stay
    // exactly as constructed by the actual bank path.
    #[cfg(feature = "cuda")]
    fn dense_bank_context_copy_reference(
        source: &SourceRealizerWeights,
        output: &crate::geometric_bank_generate::BankGenerateOutput,
    ) -> Result<Tensor> {
        let copy = output
            .copy
            .as_ref()
            .ok_or_else(|| invalid("Copy bank absent"))?;
        let bank = &copy.trace.cue_bank.bank;
        let c = source.consumer.config();
        let time = bank.context.tokens.len();
        let positions = bank
            .candidates
            .iter()
            .map(|v| v.context_position)
            .collect::<Vec<_>>();
        let pairs = positions
            .iter()
            .map(|&key| ContextPotentialPair {
                batch: 0,
                query: time - 1,
                key,
            })
            .collect::<Vec<_>>();
        let absent = AddressLane::new(H4Code::IDENTITY.index(), 0, false)
            .map_err(|e| invalid(e.to_string()))?;
        let content = vec![absent; time * c.heads * c.lanes_per_head];
        let device = copy.context.root_logits.device();
        device.synchronize()?;
        let start = std::time::Instant::now();
        let selected = frozen_potential_selected_forward(
            &source.consumer.potential,
            &content,
            &copy.context,
            &pairs,
        )?;
        device.synchronize()?;
        let selected_seconds = start.elapsed().as_secs_f64();
        let start = std::time::Instant::now();
        let dense = frozen_potential_forward(&source.consumer.potential, &content, &copy.context)?;
        device.synchronize()?;
        let dense_seconds = start.elapsed().as_secs_f64();
        let indices = Tensor::from_vec(
            positions
                .iter()
                .map(|&p| u32::try_from(p).map_err(|_| invalid("position exceeds u32")))
                .collect::<Result<Vec<_>>>()?,
            positions.len(),
            device,
        )?;
        let mut heads = Vec::new();
        for h in 0..c.heads {
            for (j, &position) in positions.iter().enumerate() {
                assert_eq!(
                    selected.scores_q24[j * c.heads + h],
                    dense.scores_q24[(h * time + time - 1) * time + position]
                );
            }
            heads.push(
                dense
                    .scores
                    .i((0, h, time - 1))?
                    .index_select(&indices, 0)?,
            );
        }
        let dense_sum = Tensor::stack(&heads, 0)?.sum(0)?;
        let selected_sum = selected.scores.sum(1)?;
        assert_eq!(dense_sum.to_vec1::<f32>()?, selected_sum.to_vec1::<f32>()?);
        let replacement =
            ((&dense_sum - dense_sum.detach())? - (&selected_sum - selected_sum.detach())?)?;
        let result = (&copy.copy_raw + replacement)?;
        assert_eq!(result.to_vec1::<f32>()?, copy.copy_raw.to_vec1::<f32>()?);
        println!(
            "selected_context_profile {}",
            serde_json::json!({
                "time":time,"heads":c.heads,"lanes_per_head":c.lanes_per_head,
                "source_occurrences":positions.len(),"dense_pairs_per_head":time*(time+1)/2,
                "selected_pairs_per_head":pairs.len(),"dense_forward_prepare_sync_seconds":dense_seconds,
                "selected_forward_prepare_sync_seconds":selected_seconds,
                "scope":"same factual bank/context snapshot; native hard scoring and host staging included; no optimizer update; not whole-fit timing"
            })
        );
        Ok(result)
    }

    #[cfg(feature = "cuda")]
    fn assert_same_device_adjoint(a: &Tensor, b: &Tensor) -> Result<()> {
        assert_eq!(a.dims(), b.dims());
        assert!(a.device().same_device(b.device()));
        for (a, b) in a
            .flatten_all()?
            .to_vec1::<f32>()?
            .into_iter()
            .zip(b.flatten_all()?.to_vec1::<f32>()?)
        {
            assert!(a.is_finite() && b.is_finite());
            assert!(
                (a - b).abs() <= 2e-4 + 2e-5 * a.abs(),
                "selected/dense adjoint differs: {a} vs {b}"
            );
        }
        Ok(())
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn composed_cuda_reloaded_source_full_bank_native_and_adjoint_parity() -> Result<()> {
        // An explicitly requested CUDA instrument fails when no GPU is present.
        let device = Device::new_cuda(0)?;
        let fixture = Fixture::new_with_lanes(2)?;
        for (name, var) in fixture.weights.parameters() {
            let mut values = vec![0.; var.elem_count()];
            if context_state_parameter(&name) {
                for (i, x) in values.iter_mut().enumerate() {
                    *x = (i % 3) as f32 * 0.25 - 0.25;
                }
                if name == "consumer.context.token_category" {
                    for row in values.chunks_exact_mut(33) {
                        row.fill(0.);
                        row[17] = 1.5;
                    }
                }
            }
            if matches!(
                name.as_str(),
                "consumer.potential.context_unary"
                    | "consumer.potential.context_radius"
                    | "consumer.potential.context_presence"
            ) {
                for (i, x) in values.iter_mut().enumerate() {
                    *x = (i % 5) as f32 * 0.25 - 0.5;
                }
            }
            var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
        }
        let source_root = fixture.path.join("cuda-bank-source");
        let native_root = fixture.path.join("cuda-bank-native");
        fixture.weights.save_source(&source_root)?;
        fixture
            .weights
            .compile(fixture.identity.clone())?
            .save(&native_root)?;
        for learn_potential in [false, true] {
            let staged = if learn_potential {
                SourceRealizerWeights::load_context_potential_on_device(
                    &source_root,
                    TOK.as_bytes(),
                    &device,
                )?
            } else {
                SourceRealizerWeights::load_source_on_device(&source_root, TOK.as_bytes(), &device)?
            };
            assert_eq!(
                parameter_identities(&fixture.weights)?,
                parameter_identities(&staged)?
            );
            assert!(staged
                .context_state_parameters()
                .values()
                .all(|v| v.device().same_device(&device)));
            assert!(staged
                .potential_parameters()
                .values()
                .all(|v| if learn_potential {
                    v.device().same_device(&device)
                } else {
                    v.device().is_cpu()
                }));
            let native =
                NativeSourceRealizer::load(&native_root, &fixture.weights, &fixture.identity)?;
            let gpu_native = NativeSourceRealizer::load(&native_root, &staged, &fixture.identity)?;
            assert_eq!(native.artifact_binding()?, gpu_native.artifact_binding()?);
            let cue = native.compile_cue_carrier(
                CueAngularQ4::new(
                    CueAngularConfig {
                        heads: 1,
                        lanes_per_head: 2,
                        mode: CueScoreMode::DirectedRelative,
                    },
                    &vec![0x12; 120],
                )
                .map_err(|e| invalid(e.to_string()))?,
            )?;
            let prefix = native.compile_prefix_transport(
                &cue,
                PrefixAngularQ4::new(
                    PrefixAngularConfig {
                        heads: 1,
                        lanes_per_head: 2,
                        mode: PrefixScoreMode::DirectedRelative,
                    },
                    &vec![0x12; 120],
                )
                .map_err(|e| invalid(e.to_string()))?,
            )?;
            // Sidecars retain references to their exact execution parent, so the
            // independently reloaded CUDA parent needs its own equivalent sidecars.
            let gpu_cue = gpu_native.compile_cue_carrier(cue.angular_source())?;
            let gpu_prefix = gpu_native.compile_prefix_transport(
                &gpu_cue,
                PrefixAngularQ4::new(
                    PrefixAngularConfig {
                        heads: 1,
                        lanes_per_head: 2,
                        mode: PrefixScoreMode::DirectedRelative,
                    },
                    &vec![0x12; 120],
                )
                .map_err(|e| invalid(e.to_string()))?,
            )?;
            assert_eq!(cue.metadata(), gpu_cue.metadata());
            assert_eq!(prefix.metadata(), gpu_prefix.metadata());
            let cpu_prepared = if learn_potential {
                fixture
                    .weights
                    .prepare_context_potential_on_device(&native, &Device::Cpu)?
            } else {
                fixture.weights.prepare_on_device(&native, &Device::Cpu)?
            };
            let gpu_prepared = if learn_potential {
                staged.prepare_context_potential_on_device(&gpu_native, &device)?
            } else {
                staged.prepare_on_device(&gpu_native, &device)?
            };
            let cpu_generate = crate::geometric_generate_learning::GenerateLearningWeights::seeded(
                fixture.weights.binding().clone(),
                2,
                1001,
                &Device::Cpu,
            )?;
            let gpu_generate = crate::geometric_generate_learning::GenerateLearningWeights::seeded(
                staged.binding().clone(),
                2,
                1001,
                &device,
            )?;
            let cpu_snapshot = cpu_generate.prepare_native()?;
            let gpu_snapshot = gpu_generate.prepare_native()?;
            assert_eq!(
                cpu_snapshot
                    .native
                    .to_bytes()
                    .map_err(|e| invalid(e.to_string()))?,
                gpu_snapshot
                    .native
                    .to_bytes()
                    .map_err(|e| invalid(e.to_string()))?
            );
            let exp = fs::read(native_root.join("consumer/exp-q31.bin"))?;
            let mut cpu_pool = crate::geometric_bank_generate::PreparedBankGenerate::new(
                &cpu_prepared,
                &cpu_generate,
                &cpu_snapshot,
                &exp,
            )?;
            let mut gpu_pool = crate::geometric_bank_generate::PreparedBankGenerate::new(
                &gpu_prepared,
                &gpu_generate,
                &gpu_snapshot,
                &exp,
            )?;
            let ids = [4, 4];
            let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
            let segments = [
                SourceBankSegment::Context {
                    token_ids: &[5],
                    role: 1,
                    event: 7,
                },
                SourceBankSegment::Source {
                    frame: frame(&ids),
                    view: &view,
                    event: 7,
                },
            ];
            let compare = |cpu: &Tensor, gpu: &Tensor| -> Result<bool> {
                assert_eq!(cpu.dims(), gpu.dims());
                assert!(cpu.device().is_cpu());
                assert!(gpu.device().same_device(&device));
                let left = cpu.flatten_all()?.to_vec1::<f32>()?;
                let right = gpu.flatten_all()?.to_vec1::<f32>()?;
                let mut nonzero = false;
                for (&a, &b) in left.iter().zip(&right) {
                    if !a.is_finite() || !b.is_finite() || (a - b).abs() > 2e-4 + 2e-5 * a.abs() {
                        return Err(invalid(format!(
                            "composed CUDA adjoint differs: CPU {a}, GPU {b}"
                        )));
                    }
                    nonzero |= a.abs() > 1e-8;
                }
                Ok(nonzero)
            };
            for own in [&[][..], &[4][..]] {
                let cpu = cpu_pool.forward_bank(&segments, &[5], own, &cue, &prefix)?;
                let gpu = gpu_pool.forward_bank(&segments, &[5], own, &gpu_cue, &gpu_prefix)?;
                assert_eq!(cpu.actions, gpu.actions);
                assert_eq!(cpu.copy_scores_q24, gpu.copy_scores_q24);
                assert_eq!(cpu.generate.scores_q24, gpu.generate.scores_q24);
                assert_eq!(cpu.final_state_codes, gpu.final_state_codes);
                assert_eq!(cpu.copy_token_ids, gpu.copy_token_ids);
                assert_eq!(cpu.context()?.trace.states, gpu.context()?.trace.states);
                assert!(gpu.context()?.state_choices.device().same_device(&device));
                let dense_copy = dense_bank_context_copy_reference(&staged, &gpu)?;
                for target in [5, 4] {
                    for credit in [
                        crate::geometric_generate_learning::VocabularyScoreAdjoint::Clipped,
                        crate::geometric_generate_learning::VocabularyScoreAdjoint::RawIdentity,
                    ] {
                        // absent-Copy Generate and source-covered alias, same native pool.
                        let cpu_loss = cpu.loss_with_credit(target, credit)?;
                        let gpu_loss = gpu.loss_with_credit(target, credit)?;
                        let dense_loss =
                        crate::geometric_generate_learning::vocabulary_marginal_loss_with_credit(
                            &gpu.actions,
                            &gpu.generate.raw_scores,
                            Some(&dense_copy),
                            target,
                            credit,
                        )?;
                        assert_eq!(gpu_loss.to_scalar::<f32>()?, dense_loss.to_scalar::<f32>()?);
                        // Offline floating reductions may differ by rounding; native
                        // scores, actions and state codes remain exact checks above.
                        let cl = cpu_loss.to_scalar::<f32>()?;
                        let gl = gpu_loss.to_scalar::<f32>()?;
                        assert!(cl.is_finite() && gl.is_finite());
                        assert!((cl - gl).abs() <= 1e-6 + 1e-6 * cl.abs());
                        let cpu_grads = cpu_loss.backward()?;
                        let gpu_grads = gpu_loss.backward()?;
                        let dense_grads = dense_loss.backward()?;
                        let mut context_nonzero = false;
                        for (name, cv) in fixture.weights.context_state_parameters() {
                            let gv = &staged.context_state_parameters()[&name];
                            let cg = cpu_grads.get(cv.as_tensor()).ok_or_else(|| {
                                invalid(format!("CPU context adjoint absent: {name}"))
                            })?;
                            let gg = gpu_grads.get(gv.as_tensor()).ok_or_else(|| {
                                invalid(format!("CUDA context adjoint absent: {name}"))
                            })?;
                            context_nonzero |= compare(cg, gg)?;
                            let dg = dense_grads.get(gv.as_tensor()).ok_or_else(|| {
                                invalid(format!("dense context adjoint absent: {name}"))
                            })?;
                            assert_same_device_adjoint(gg, dg)?;
                        }
                        assert!(
                            context_nonzero,
                            "full-bank context path must have nonzero credit"
                        );
                        for source in [&fixture.weights, &staged] {
                            let grads = if source.consumer.context.device().is_cpu() {
                                &cpu_grads
                            } else {
                                &gpu_grads
                            };
                            for (name, var) in source.parameters() {
                                if !context_state_parameter(&name)
                                    && !(learn_potential && potential_parameter(&name))
                                {
                                    assert!(
                                        grads.get(var.as_tensor()).is_none(),
                                        "frozen source family gained credit: {name}"
                                    );
                                }
                            }
                        }
                        if learn_potential {
                            let mut potential_nonzero = false;
                            for (name, cv) in fixture.weights.potential_parameters() {
                                let gv = &staged.potential_parameters()[&name];
                                match (
                                    cpu_grads.get(cv.as_tensor()),
                                    gpu_grads.get(gv.as_tensor()),
                                    dense_grads.get(gv.as_tensor()),
                                ) {
                                    (Some(cg), Some(gg), Some(dg)) => {
                                        potential_nonzero |= compare(cg, gg)?;
                                        assert_same_device_adjoint(gg, dg)?;
                                    }
                                    (None, None, None) => {}
                                    _ => {
                                        return Err(invalid(format!(
                                            "potential credit support differs: {name}"
                                        )))
                                    }
                                }
                            }
                            assert!(
                                potential_nonzero,
                                "learned potential path must retain nonzero coefficient credit"
                            );
                        }
                        let mut generate_nonzero = false;
                        for (name, cv) in cpu_generate.parameters() {
                            if cv.elem_count() == 0 {
                                continue;
                            }
                            let gv = &gpu_generate.parameters()[&name];
                            let cg = cpu_grads.get(cv.as_tensor()).ok_or_else(|| {
                                invalid(format!("CPU Generate adjoint absent: {name}"))
                            })?;
                            let gg = gpu_grads.get(gv.as_tensor()).ok_or_else(|| {
                                invalid(format!("CUDA Generate adjoint absent: {name}"))
                            })?;
                            generate_nonzero |= compare(cg, gg)?;
                        }
                        assert!(
                            generate_nonzero,
                            "full-bank Generate path must have nonzero credit"
                        );
                    }
                }
            }
        }
        Ok(())
    }

    #[cfg(feature = "cuda")]
    #[test]
    #[ignore = "requires CUDA and UOR_SELECTED_CREDIT_ROOT authenticated retained fit"]
    fn selected_context_retained_bank_cuda_zero_update() -> Result<()> {
        use crate::geometric_generate_learning::{
            vocabulary_marginal_loss_with_credit, VocabularyScoreAdjoint,
        };
        use uor_r4_core::native_geometric::learner::geometric_generate::NativeGeometricGenerate;
        use uor_r4_integer::geometric_vocabulary_actions::NativeVocabularyActions;
        let root = std::path::PathBuf::from(
            std::env::var("UOR_SELECTED_CREDIT_ROOT")
                .map_err(|_| invalid("UOR_SELECTED_CREDIT_ROOT is required"))?,
        );
        uor_r4_core::report_output::verify(&root).map_err(|e| invalid(e.to_string()))?;
        let read =
            |p: &Path| -> Result<serde_json::Value> { Ok(serde_json::from_slice(&fs::read(p)?)?) };
        let cp = root.join("checkpoint-0000");
        let receipt = read(&cp.join("receipt.json"))?;
        println!(
            "retained_selected_identity {}",
            serde_json::json!({
                "root":root,"report_sha256":sha256_bytes(&fs::read(root.join("report.json"))?),
                "manifest_sha256":sha256_bytes(&fs::read(root.join("manifest.json"))?),
                "checkpoint_receipt_sha256":sha256_bytes(&fs::read(cp.join("receipt.json"))?),
                "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND"),
                "device":"cuda","optimizer_updates":0
            })
        );
        let meta = read(&cp.join("native/metadata.json"))?;
        let identity: ConsumerIdentity = serde_json::from_value(meta["identity"].clone())?;
        let tok = fs::read(cp.join("native/tokenizer.json"))?;
        let device = Device::new_cuda(0)?;
        let source =
            SourceRealizerWeights::load_source_on_device(&cp.join("source"), &tok, &device)?;
        let native = NativeSourceRealizer::load(&cp.join("native"), &source, &identity)?;
        let prepared = source.prepare_on_device(&native, &device)?;
        assert_eq!(
            serde_json::to_value(native.artifact_binding()?)?,
            receipt["parent"]
        );
        let bytes = fs::read(cp.join("generate.bin"))?;
        assert_eq!(
            sha256_bytes(&bytes),
            "41c7beae25e98fdc658ff667959a7c02a6b916b04298a425f7f7f48285d21511"
        );
        let generate = NativeGeometricGenerate::from_bytes(&bytes, source.binding())
            .map_err(|e| invalid(e.to_string()))?;
        let mut pool = NativeVocabularyActions::new(
            source.binding().clone(),
            &fs::read(cp.join("native/consumer/exp-q31.bin"))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let c = source.consumer.config();
        let initial_parameter_identities = parameter_identities(&source)?;
        let absent = AddressLane::new(H4Code::IDENTITY.index(), 0, false)
            .map_err(|e| invalid(e.to_string()))?;
        let mut positions_checked = 0;
        // These saved steps retain the complete causal bank, every occurrence,
        // and its native source provenance. No source is selected using a label.
        for row_index in [0, 8] {
            let row_bytes =
                fs::read(root.join(format!("development-0000-row-{row_index:04}.json")))?;
            let row: serde_json::Value = serde_json::from_slice(&row_bytes)?;
            let row_sha256 = sha256_bytes(&row_bytes);
            for step_index in [0, 3] {
                let step = &row["canonical"][step_index]["native"];
                let provenance = &step["source_provenance"];
                let ids: Vec<u32> = serde_json::from_value(provenance["causal_tokens"].clone())?;
                if ids.is_empty() {
                    return Err(invalid("retained context is empty"));
                }
                let context = prepared.context_output(&ids, false)?;
                let candidates = provenance["candidates"]
                    .as_array()
                    .ok_or_else(|| invalid("retained physical candidates absent"))?;
                let pairs = candidates
                    .iter()
                    .enumerate()
                    .map(|(j, v)| {
                        if v["bank_index"].as_u64() != Some(j as u64) {
                            return Err(invalid("retained occurrence order differs"));
                        }
                        let key = v["context_position"]
                            .as_u64()
                            .and_then(|v| usize::try_from(v).ok())
                            .ok_or_else(|| invalid("retained context position absent"))?;
                        Ok(ContextPotentialPair {
                            batch: 0,
                            query: ids.len() - 1,
                            key,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let content = vec![absent; ids.len() * c.heads * c.lanes_per_head];
                device.synchronize()?;
                let start = std::time::Instant::now();
                let selected = frozen_potential_selected_forward(
                    &source.consumer.potential,
                    &content,
                    &context,
                    &pairs,
                )?;
                device.synchronize()?;
                let selected_seconds = start.elapsed().as_secs_f64();
                let start = std::time::Instant::now();
                let dense =
                    frozen_potential_forward(&source.consumer.potential, &content, &context)?;
                device.synchronize()?;
                let dense_seconds = start.elapsed().as_secs_f64();
                let index = Tensor::from_vec(
                    pairs
                        .iter()
                        .map(|p| {
                            u32::try_from(p.key)
                                .map_err(|_| invalid("retained context index exceeds u32"))
                        })
                        .collect::<Result<Vec<_>>>()?,
                    pairs.len(),
                    &device,
                )?;
                let mut heads = Vec::new();
                let contextual: Vec<i64> = serde_json::from_value(
                    provenance["copy_components_q24"]["contextual"].clone(),
                )?;
                for h in 0..c.heads {
                    for (j, p) in pairs.iter().enumerate() {
                        assert_eq!(
                            selected.scores_q24[j * c.heads + h],
                            dense.scores_q24[(h * ids.len() + ids.len() - 1) * ids.len() + p.key]
                        );
                    }
                    heads.push(
                        dense
                            .scores
                            .i((0, h, ids.len() - 1))?
                            .index_select(&index, 0)?,
                    );
                }
                assert_eq!(contextual.len(), pairs.len());
                for (j, &score) in contextual.iter().enumerate() {
                    let sum = (0..c.heads).try_fold(0i64, |sum, h| {
                        sum.checked_add(selected.scores_q24[j * c.heads + h])
                            .ok_or_else(|| invalid("selected native sum overflow"))
                    })?;
                    assert_eq!(sum, score);
                }
                let selected_sum = selected.scores.sum(1)?;
                let dense_sum = Tensor::stack(&heads, 0)?.sum(0)?;
                let scores: Vec<i64> = serde_json::from_value(step["copy_raw_scores_q24"].clone())?;
                let copy_ids: Vec<u32> = serde_json::from_value(step["copy_token_ids"].clone())?;
                let states: Vec<u8> = serde_json::from_value(step["retained_state_codes"].clone())?;
                let states = states
                    .into_iter()
                    .map(H4Code::try_from)
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| invalid(e.to_string()))?;
                let mut generated = vec![0i64; generate.vocab_size()];
                generate
                    .score_into(&states, &mut generated, &mut Default::default())
                    .map_err(|e| invalid(e.to_string()))?;
                assert_eq!(
                    sha256_bytes(&serde_json::to_vec(&generated)?),
                    step["generate_raw_scores_sha256"]
                        .as_str()
                        .ok_or_else(|| invalid("Generate digest absent"))?
                );
                let actions = pool
                    .reduce_trace(&generated, &copy_ids, &scores)
                    .map_err(|e| invalid(e.to_string()))?;
                assert_eq!(
                    serde_json::to_value(&actions.summary)?,
                    step["pool"]["summary"]
                );
                let anchor = Tensor::from_vec(
                    scores
                        .iter()
                        .map(|&v| (v as f64 / (1u64 << 24) as f64) as f32)
                        .collect::<Vec<_>>(),
                    scores.len(),
                    &device,
                )?;
                let selected_copy = (&anchor + (&selected_sum - selected_sum.detach())?)?;
                let dense_copy = (&anchor + (&dense_sum - dense_sum.detach())?)?;
                let gen = Tensor::from_vec(
                    generated
                        .iter()
                        .map(|&v| (v as f64 / (1u64 << 24) as f64) as f32)
                        .collect::<Vec<_>>(),
                    generated.len(),
                    &device,
                )?;
                // The label enters only after the complete native action pool.
                let targets: Vec<u32> =
                    serde_json::from_value(row["canonical_target_ids_labels_only"].clone())?;
                for credit in [
                    VocabularyScoreAdjoint::Clipped,
                    VocabularyScoreAdjoint::RawIdentity,
                ] {
                    let selected_loss = vocabulary_marginal_loss_with_credit(
                        &actions,
                        &gen,
                        Some(&selected_copy),
                        targets[step_index],
                        credit,
                    )?;
                    let dense_loss = vocabulary_marginal_loss_with_credit(
                        &actions,
                        &gen,
                        Some(&dense_copy),
                        targets[step_index],
                        credit,
                    )?;
                    assert_eq!(
                        selected_loss.to_scalar::<f32>()?,
                        dense_loss.to_scalar::<f32>()?
                    );
                    device.synchronize()?;
                    let start = std::time::Instant::now();
                    let sg = selected_loss.backward()?;
                    device.synchronize()?;
                    let selected_backward_seconds = start.elapsed().as_secs_f64();
                    let start = std::time::Instant::now();
                    let dg = dense_loss.backward()?;
                    device.synchronize()?;
                    let dense_backward_seconds = start.elapsed().as_secs_f64();
                    let mut nonzero = false;
                    for (name, var) in source.context_state_parameters() {
                        let a = sg.get(var.as_tensor()).ok_or_else(|| {
                            invalid(format!("selected actual context credit absent: {name}"))
                        })?;
                        let b = dg.get(var.as_tensor()).ok_or_else(|| {
                            invalid(format!("dense actual context credit absent: {name}"))
                        })?;
                        assert_same_device_adjoint(a, b)?;
                        nonzero |= a
                            .flatten_all()?
                            .to_vec1::<f32>()?
                            .iter()
                            .any(|v| v.abs() > 1e-8);
                    }
                    assert!(nonzero, "retained actual context credit must be nonzero");
                    println!(
                        "retained_selected_credit {}",
                        serde_json::json!({
                            "row_index":row_index,"row_id":row["id"],"row_sha256":row_sha256,
                            "teacher_step":step_index,"credit":format!("{credit:?}"),
                            "time":ids.len(),"heads":c.heads,"lanes_per_head":c.lanes_per_head,
                            "selected_pairs_per_head":pairs.len(),"dense_pairs_per_head":ids.len()*(ids.len()+1)/2,
                            "selected_forward_prepare_sync_seconds":selected_seconds,"dense_forward_prepare_sync_seconds":dense_seconds,
                            "selected_backward_sync_seconds":selected_backward_seconds,"dense_backward_sync_seconds":dense_backward_seconds,
                            "optimizer_updates":0,"scope":"contextual state-credit only; complete native alias loss; retained teacher-prefix conditions; no whole-fit or language claim"
                        })
                    );
                    positions_checked += 1;
                }
            }
        }
        assert_eq!(positions_checked, 8);
        assert_eq!(parameter_identities(&source)?, initial_parameter_identities);
        Ok(())
    }

    #[test]
    fn composed_device_source_cpu_staging_preserves_exact_bits_and_preparation() -> Result<()> {
        let fixture = Fixture::new_with_lanes(2)?;
        let saved = fixture.path.join("device-source");
        fixture.weights.save_source(&saved)?;
        let staged = SourceRealizerWeights::load_source_on_device(
            &saved,
            &fixture.weights.tokenizer_bytes,
            &Device::Cpu,
        )?;
        assert_eq!(
            parameter_identities(&fixture.weights)?,
            parameter_identities(&staged)?
        );
        let native = fixture.weights.compile(fixture.identity.clone())?;
        let _prepared = staged.prepare_on_device(&native, &Device::Cpu)?;
        assert!(staged
            .context_state_parameters()
            .values()
            .all(|v| v.device().same_device(&Device::Cpu)));
        Ok(())
    }

    #[test]
    fn composed_state_native_alias_parity_and_context_only_credit() -> Result<()> {
        let fixture = Fixture::new_with_lanes(2)?;
        for (name, var) in fixture.weights.parameters() {
            let mut values = vec![0.; var.elem_count()];
            if context_state_parameter(&name) {
                for (i, x) in values.iter_mut().enumerate() {
                    *x = (i % 3) as f32 * 0.25 - 0.25;
                }
                if name == "consumer.context.token_category" {
                    for row in values.chunks_exact_mut(33) {
                        row.fill(0.);
                        row[17] = 1.5;
                    }
                }
            } else if terminal_parameter(&name) {
                for (i, x) in values.iter_mut().enumerate() {
                    *x = (i % 3) as f32 * 0.25 - 0.25;
                }
            }
            var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
        }
        let root = fixture.path.join("composed-parent");
        fixture
            .weights
            .compile(fixture.identity.clone())?
            .save(&root)?;
        let native = NativeSourceRealizer::load(&root, &fixture.weights, &fixture.identity)?;
        let cue = native.compile_cue_carrier(
            CueAngularQ4::new(
                CueAngularConfig {
                    heads: 1,
                    lanes_per_head: 2,
                    mode: CueScoreMode::DirectedRelative,
                },
                &vec![0x12; 120],
            )
            .map_err(|e| invalid(e.to_string()))?
            .with_joint(
                CueJointQ4::new(
                    CueJointConfig {
                        head: 0,
                        left_lane: 0,
                        right_lane: 1,
                    },
                    vec![0x12; 8],
                )
                .map_err(|e| invalid(e.to_string()))?,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let prefix = native.compile_prefix_transport(
            &cue,
            PrefixAngularQ4::new(
                PrefixAngularConfig {
                    heads: 1,
                    lanes_per_head: 2,
                    mode: PrefixScoreMode::DirectedRelative,
                },
                &vec![0x12; 120],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let end = native.compile_source_end_transport(
            &cue,
            &prefix,
            SourceEndAngularQ4::new(
                SourceEndAngularConfig {
                    heads: 1,
                    lanes_per_head: 2,
                    mode: SourceEndScoreMode::DirectedRelative,
                },
                &vec![0x12; 120],
                &vec![0x21; 120],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let ids = [4, 4];
        let view = SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&ids)?;
        let segments = [
            SourceBankSegment::Context {
                token_ids: &[5],
                role: 1,
                event: 7,
            },
            SourceBankSegment::Source {
                frame: frame(&ids),
                view: &view,
                event: 7,
            },
        ];
        let prepared = fixture.weights.prepare(&native)?;
        for own in [&[][..], &[4][..]] {
            let factual = native.read_bank_with_source_end_transport(
                &segments,
                &[5],
                own,
                &cue,
                &prefix,
                &end,
            )?;
            let copy_only =
                prepared.forward_bank_composed_copy(&segments, &[5], own, &cue, &prefix)?;
            assert_eq!(copy_only.trace, factual.prefix_bank);
            assert!(context_replay_matches(
                &factual.prefix_bank.cue_bank.bank.context,
                &copy_only.context.trace
            ));
            let count = factual.prefix_bank.cue_bank.bank.candidates.len();
            assert_eq!(copy_only.copy_raw.dims(), [count]);
            let raw = copy_only.copy_raw.to_vec1::<f32>()?;
            for (j, &score) in raw.iter().enumerate() {
                let expected = factual
                    .prefix_bank
                    .cue_bank
                    .bank
                    .heads
                    .iter()
                    .map(|h| h.scores_q24[j])
                    .sum::<i64>();
                assert_eq!(score, (expected as f64 / 16_777_216.) as f32);
            }
            assert_eq!(copy_only.state_credit_components.len(), 3);
            let generate = crate::geometric_generate_learning::GenerateLearningWeights::seeded(
                fixture.weights.binding().clone(),
                2,
                1001,
                &Device::Cpu,
            )?;
            let snapshot = generate.prepare_native()?;
            let exp = fs::read(root.join("consumer/exp-q31.bin"))?;
            let mut unified = crate::geometric_bank_generate::PreparedBankGenerate::new(
                &prepared, &generate, &snapshot, &exp,
            )?;
            let joint = unified.forward_bank(&segments, &[5], own, &cue, &prefix)?;
            // The emission compiler preserves the leading-space token identity:
            // the original [a,a] becomes [space-a,a], not two raw a aliases.
            assert_eq!(joint.copy_token_ids, view.emitted_token_ids());
            assert_eq!(joint.copy_token_ids, vec![7, 4]);
            assert_eq!(
                joint.context()?.trace.states,
                copy_only.context.trace.states
            );
            assert_eq!(
                joint
                    .final_state_codes
                    .iter()
                    .map(|c| c.index())
                    .collect::<Vec<_>>(),
                *copy_only
                    .context
                    .trace
                    .states
                    .last()
                    .ok_or_else(|| invalid("missing final state"))?
            );
            let alias_mass = joint
                .actions
                .actions
                .iter()
                .filter(|a| a.token_id == 4)
                .map(|a| a.weight_q31)
                .sum::<u64>();
            assert_eq!(
                joint
                    .actions
                    .token_masses
                    .iter()
                    .find(|m| m.token_id == 4)
                    .ok_or_else(|| invalid("missing unified target mass"))?
                    .weight_q31,
                alias_mass
            );
            // Ordinary token 5 is absent from the source. It is still admitted
            // by Generate before its label enters the common loss.
            assert!(joint.loss(5)?.to_scalar::<f32>()?.is_finite());
            let coadapted = fixture
                .weights
                .prepare_context_potential_on_device(&native, &Device::Cpu)?;
            let mut coadapted_pool = crate::geometric_bank_generate::PreparedBankGenerate::new(
                &coadapted, &generate, &snapshot, &exp,
            )?;
            let coadapted_out = coadapted_pool.forward_bank(&segments, &[5], own, &cue, &prefix)?;
            assert_eq!(coadapted_out.actions, joint.actions);
            for target in [5, 4] {
                // absent-Copy Generate and source-covered alias
                let default_loss = joint.loss(target)?;
                let learned_loss = coadapted_out.loss(target)?;
                assert_eq!(
                    default_loss.to_scalar::<f32>()?.to_bits(),
                    learned_loss.to_scalar::<f32>()?.to_bits()
                );
                let frozen_grads = default_loss.backward()?;
                let learned_grads = learned_loss.backward()?;
                for (name, var) in fixture.weights.parameters() {
                    if !context_state_parameter(&name) {
                        assert!(
                            frozen_grads.get(var.as_tensor()).is_none(),
                            "frozen source coefficient graph {name}"
                        );
                    }
                }
                let mut nonzero = false;
                for (name, var) in fixture.weights.potential_parameters() {
                    assert!(frozen_grads.get(var.as_tensor()).is_none());
                    let gradient = learned_grads
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid("potential credit disconnected"))?;
                    let values = gradient.flatten_all()?.to_vec1::<f32>()?;
                    assert!(values.iter().all(|v| v.is_finite()));
                    if matches!(
                        name.as_str(),
                        "consumer.potential.content_unary"
                            | "consumer.potential.content_radius"
                            | "consumer.potential.pair"
                    ) {
                        assert!(values.iter().all(|v| *v == 0.));
                    }
                    nonzero |= values.iter().any(|v| v.abs() > 1e-8);
                }
                assert!(
                    nonzero,
                    "actual full-vocabulary loss must reach potential coefficients"
                );
                for (name, var) in fixture.weights.context_state_parameters() {
                    let left = frozen_grads
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid("frozen context gradient missing"))?
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    let right = learned_grads
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid("coadapted context gradient missing"))?
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    assert_eq!(
                        left, right,
                        "coefficient credit altered existing context credit {name}"
                    );
                }
            }
            let no_source = unified.forward_no_source(&[5, 4])?;
            assert!(no_source.copy.is_none());
            assert!(no_source.copy_token_ids.is_empty());
            assert!(no_source.loss(5)?.to_scalar::<f32>()?.is_finite());
            assert!(unified.forward_no_source(&[]).is_err());
            let loaded = uor_r4_core::native_geometric::learner::geometric_generate::NativeGeometricGenerate::from_bytes(
                &snapshot.native.to_bytes().map_err(|e| invalid(e.to_string()))?, fixture.weights.binding(),
            ).map_err(|e| invalid(e.to_string()))?;
            let mut reloaded_scores = vec![0; fixture.weights.binding().vocab_size()];
            loaded.score_into(&joint.final_state_codes, &mut reloaded_scores,
                &mut uor_r4_core::native_geometric::learner::geometric_generate::GenerateReadCounts::default())
                .map_err(|e| invalid(e.to_string()))?;
            assert_eq!(reloaded_scores, joint.generate.scores_q24);
            assert!(prepared
                .forward_bank_composed_copy(&segments, &[], own, &cue, &prefix)
                .is_err());
        }
        Ok(())
    }

    #[test]
    fn composed_state_export_rebind_and_frozen_inventory_rejection() -> Result<()> {
        let (fixture, native, _) = dependent_fixture()?;
        let root = fixture.path.join("dependent-native");
        let cue = native.compile_cue_carrier(
            CueAngularQ4::new(
                CueAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: CueScoreMode::DirectedRelative,
                },
                &[0x12; 60],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let prefix = native.compile_prefix_transport(
            &cue,
            PrefixAngularQ4::new(
                PrefixAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: PrefixScoreMode::DirectedRelative,
                },
                &[0x12; 60],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        let end = native.compile_source_end_transport(
            &cue,
            &prefix,
            SourceEndAngularQ4::new(
                SourceEndAngularConfig {
                    heads: 1,
                    lanes_per_head: 1,
                    mode: SourceEndScoreMode::DirectedRelative,
                },
                &[0x12; 60],
                &[0x21; 60],
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        assert_eq!(
            fixture
                .weights
                .compile_context_state_rebound(&native)?
                .execution_binding()?,
            native.artifact_binding()?
        );
        let params = fixture.weights.context_state_parameters();
        assert_eq!(params.len(), 6); // single lane has no neighbor family.
        let var = &params["consumer.context.token_transition"];
        let mut values = var.flatten_all()?.to_vec1::<f32>()?;
        values[0] += 0.25;
        var.set(&Tensor::from_vec(values, var.shape(), &Device::Cpu)?)?;
        let current = fixture.weights.compile_context_state_rebound(&native)?;
        assert_ne!(current.execution_binding()?, native.artifact_binding()?);
        let dest = fixture.path.join("composed-rebound");
        let receipt = fixture
            .weights
            .save_context_state_rebound(&dest, &native, &root, &cue, &prefix, &end)?;
        assert_eq!(
            receipt["new_parent"],
            serde_json::to_value(current.execution_binding()?)?
        );
        let source = SourceRealizerWeights::load_source(&dest.join("source"), TOK.as_bytes())?;
        let loaded = NativeSourceRealizer::load(&dest.join("native"), &source, &fixture.identity)?;
        assert_eq!(current.execution_binding()?, loaded.artifact_binding()?);
        assert!(loaded
            .read_bank_with_source_end_transport(
                &[SourceBankSegment::Source {
                    frame: frame(&[4]),
                    view: &SourceEmissionCompiler::new(TOK.as_bytes())?.compile(&[4])?,
                    event: 7
                }],
                &[5],
                &[],
                &cue,
                &prefix,
                &end
            )
            .is_err());
        for file in [
            "cue/cue-q4.bin",
            "prefix/prefix-q4.bin",
            "source-end/source-end-period-q4.bin",
            "source-end/source-end-stop-q4.bin",
        ] {
            assert!(dest.join(file).is_file());
        }
        let potential =
            &fixture.weights.potential_parameters()["consumer.potential.context_presence"];
        let mut values = potential.flatten_all()?.to_vec1::<f32>()?;
        values[0] += 0.25;
        potential.set(&Tensor::from_vec(values, potential.shape(), &Device::Cpu)?)?;
        assert!(fixture
            .weights
            .compile_context_state_rebound(&native)
            .is_err());
        let learned = fixture.weights.compile_context_potential_rebound(&native)?;
        assert_ne!(learned.execution_binding()?, current.execution_binding()?);
        let potential_dest = fixture.path.join("context-potential-rebound");
        let potential_receipt = fixture.weights.save_context_potential_rebound(
            &potential_dest,
            &native,
            &root,
            &cue,
            &prefix,
            &end,
        )?;
        assert_eq!(potential_receipt["learned_potential"], true);
        let restored_source =
            SourceRealizerWeights::load_source(&potential_dest.join("source"), TOK.as_bytes())?;
        let restored = NativeSourceRealizer::load(
            &potential_dest.join("native"),
            &restored_source,
            &fixture.identity,
        )?;
        assert_eq!(restored.artifact_binding()?, learned.execution_binding()?);
        assert_ne!(
            fs::read(potential_dest.join("native/consumer/potential-q4.bin"))?,
            fs::read(root.join("consumer/potential-q4.bin"))?
        );
        assert_eq!(
            fs::read(potential_dest.join("cue/cue-q4.bin"))?,
            fs::read(dest.join("cue/cue-q4.bin"))?
        );
        assert_eq!(
            fs::read(potential_dest.join("prefix/prefix-q4.bin"))?,
            fs::read(dest.join("prefix/prefix-q4.bin"))?
        );
        let terminal = &fixture.weights.terminal_parameters()["period.coefficients"];
        let mut values = terminal.flatten_all()?.to_vec1::<f32>()?;
        values[0] += 0.25;
        terminal.set(&Tensor::from_vec(values, terminal.shape(), &Device::Cpu)?)?;
        assert!(fixture
            .weights
            .compile_context_state_rebound(&native)
            .is_err());
        assert!(fixture
            .weights
            .compile_context_potential_rebound(&native)
            .is_err());
        Ok(())
    }
}

// Active full-bank geometric Generate integration; exact retained source bodies.
impl SourceRealizerWeights {
    pub fn context_state_parameters(&self) -> BTreeMap<String, Var> {
        self.parameters()
            .into_iter()
            .filter(|(name, _)| context_state_parameter(name))
            .collect()
    }

    pub fn compile_context_state_rebound(
        &self,
        frozen_parent: &NativeSourceRealizer,
    ) -> Result<NativeSourceRealizer> {
        frozen_parent.artifact_binding()?;
        verify_context_state_inventory(
            &parameter_identities(self)?,
            &frozen_parent.metadata.source_parameters,
        )?;
        let mut native = self.compile(frozen_parent.metadata.identity.clone())?;
        if native.consumer.context.config() != frozen_parent.consumer.context.config() {
            return Err(invalid("composed context configuration changed"));
        }
        let (_, metadata) = native.export_payloads()?;
        native.compiled_metadata_sha256 = Some(sha256_bytes(&metadata));
        Ok(native)
    }

    pub fn potential_parameters(&self) -> BTreeMap<String, Var> {
        self.consumer
            .potential
            .parameters()
            .into_iter()
            .map(|(name, var)| (format!("consumer.potential.{name}"), var.clone()))
            .collect()
    }

    pub fn compile_context_potential_rebound(
        &self,
        frozen_parent: &NativeSourceRealizer,
    ) -> Result<NativeSourceRealizer> {
        frozen_parent.artifact_binding()?;
        verify_context_potential_inventory(
            &parameter_identities(self)?,
            &frozen_parent.metadata.source_parameters,
        )?;
        let mut native = self.compile(frozen_parent.metadata.identity.clone())?;
        if native.consumer.context.config() != frozen_parent.consumer.context.config()
            || native.consumer.potential.config() != frozen_parent.consumer.potential.config()
        {
            return Err(invalid("composed context/potential configuration changed"));
        }
        let (_, metadata) = native.export_payloads()?;
        native.compiled_metadata_sha256 = Some(sha256_bytes(&metadata));
        Ok(native)
    }

    pub fn project_potential_range(&self) -> Result<()> {
        self.consumer.potential.project_shadow_range()
    }

    pub fn save_context_state_rebound(
        &self,
        path: &Path,
        frozen_parent: &NativeSourceRealizer,
        frozen_native_root: &Path,
        frozen_cue: &NativeCueCarrier<'_>,
        frozen_prefix: &NativePrefixTransport<'_>,
        frozen_end: &NativeSourceEndTransport<'_>,
    ) -> Result<serde_json::Value> {
        self.save_context_learning_rebound(
            path,
            frozen_parent,
            frozen_native_root,
            frozen_cue,
            frozen_prefix,
            frozen_end,
            false,
        )
    }

    pub fn save_context_potential_rebound(
        &self,
        path: &Path,
        frozen_parent: &NativeSourceRealizer,
        frozen_native_root: &Path,
        frozen_cue: &NativeCueCarrier<'_>,
        frozen_prefix: &NativePrefixTransport<'_>,
        frozen_end: &NativeSourceEndTransport<'_>,
    ) -> Result<serde_json::Value> {
        self.save_context_learning_rebound(
            path,
            frozen_parent,
            frozen_native_root,
            frozen_cue,
            frozen_prefix,
            frozen_end,
            true,
        )
    }

    fn save_context_learning_rebound(
        &self,
        path: &Path,
        frozen_parent: &NativeSourceRealizer,
        frozen_native_root: &Path,
        frozen_cue: &NativeCueCarrier<'_>,
        frozen_prefix: &NativePrefixTransport<'_>,
        frozen_end: &NativeSourceEndTransport<'_>,
        allow_potential: bool,
    ) -> Result<serde_json::Value> {
        let old_binding = frozen_parent.artifact_binding()?;
        if crate::sha256_file(&frozen_native_root.join("metadata.json"))?
            != old_binding.metadata_sha256
            || frozen_cue.metadata().parent_artifact != old_binding
            || frozen_prefix.metadata().parent_artifact != old_binding
            || frozen_prefix.metadata().frozen_cue != *frozen_cue.metadata()
            || frozen_end.metadata().parent_artifact != old_binding
            || frozen_end.metadata().frozen_cue != *frozen_cue.metadata()
            || frozen_end.metadata().frozen_prefix != *frozen_prefix.metadata()
        {
            return Err(invalid("composed frozen parent/sidecar binding differs"));
        }
        let current = if allow_potential {
            self.compile_context_potential_rebound(frozen_parent)?
        } else {
            self.compile_context_state_rebound(frozen_parent)?
        };
        let (old_files, _) = frozen_parent.export_payloads()?;
        let (new_files, _) = current.export_payloads()?;
        let mut frozen_files = BTreeMap::new();
        for (name, bytes) in &old_files {
            if fs::read(frozen_native_root.join(name))? != *bytes {
                return Err(invalid(
                    "composed frozen native file differs from loaded parent",
                ));
            }
            if matches!(
                name.as_str(),
                "consumer/context-q4.bin" | "consumer/metadata.json"
            ) || (allow_potential && name == "consumer/potential-q4.bin")
            {
                continue;
            }
            if new_files.get(name) != Some(bytes) {
                return Err(invalid(format!(
                    "composed export changed frozen payload {name}"
                )));
            }
            frozen_files.insert(name.clone(), sha256_bytes(bytes));
        }
        fs::create_dir(path)?;
        self.save_source(&path.join("source"))?;
        current.save(&path.join("native"))?;
        let source = Self::load_source(&path.join("source"), &self.tokenizer_bytes)?;
        let loaded = NativeSourceRealizer::load(
            &path.join("native"),
            &source,
            &frozen_parent.metadata.identity,
        )?;
        let new_binding = loaded.artifact_binding()?;
        if current.execution_binding()? != new_binding {
            return Err(invalid(
                "composed exported identity differs from current execution",
            ));
        }
        let integer = uor_r4_integer::geometric_source_realizer::NativeSourceRealizer::load_native(
            &path.join("native"),
            &new_binding,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let cue = integer
            .compile_cue_carrier(frozen_cue.angular_source())
            .map_err(|e| invalid(e.to_string()))?;
        let prefix = integer
            .compile_prefix_transport(
                &cue,
                PrefixAngularQ4::new(
                    frozen_prefix.metadata().potential,
                    frozen_prefix.packed_coefficients(),
                )
                .map_err(|e| invalid(e.to_string()))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
        let end = integer
            .compile_source_end_transport(
                &cue,
                &prefix,
                SourceEndAngularQ4::new(
                    frozen_end.metadata().potential,
                    frozen_end.period_packed_coefficients(),
                    frozen_end.stop_packed_coefficients(),
                )
                .map_err(|e| invalid(e.to_string()))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
        if cue.packed_coefficients() != frozen_cue.packed_coefficients()
            || prefix.packed_coefficients() != frozen_prefix.packed_coefficients()
            || end.period_packed_coefficients() != frozen_end.period_packed_coefficients()
            || end.stop_packed_coefficients() != frozen_end.stop_packed_coefficients()
            || cue.joint().map(|j| j.packed_coefficients())
                != frozen_cue.joint().map(|j| j.packed_coefficients())
            || cue.metadata().algebra_sha256 != frozen_cue.metadata().algebra_sha256
        {
            return Err(invalid("composed rebind changed numeric sidecar/algebra"));
        }
        for (name, metadata, files) in [
            (
                "cue",
                serde_json::to_value(cue.metadata())?,
                vec![("cue-q4.bin", cue.packed_coefficients())],
            ),
            (
                "prefix",
                serde_json::to_value(prefix.metadata())?,
                vec![("prefix-q4.bin", prefix.packed_coefficients())],
            ),
            (
                "source-end",
                serde_json::to_value(end.metadata())?,
                vec![
                    ("source-end-period-q4.bin", end.period_packed_coefficients()),
                    ("source-end-stop-q4.bin", end.stop_packed_coefficients()),
                ],
            ),
        ] {
            let root = path.join(name);
            fs::create_dir(&root)?;
            fs::write(
                root.join("native-metadata.json"),
                serde_json::to_vec_pretty(&metadata)?,
            )?;
            for (file, bytes) in files {
                fs::write(root.join(file), bytes)?;
            }
        }
        if let Some(joint) = cue.joint() {
            fs::write(
                path.join("cue/cue-joint-q4.bin"),
                joint.packed_coefficients(),
            )?;
        }
        Ok(
            serde_json::json!({"schema":if allow_potential {"uor-r4.composed-context-potential-rebind/1"} else {"uor-r4.composed-context-state-rebind/1"},"learned_potential":allow_potential,"old_parent":old_binding,"new_parent":new_binding,"frozen_numeric_payloads_sha256":frozen_files,
            "cue_packed_sha256":sha256_bytes(cue.packed_coefficients()),"cue_joint_packed_sha256":cue.joint().map(|j|sha256_bytes(j.packed_coefficients())),
            "prefix_packed_sha256":sha256_bytes(prefix.packed_coefficients()),"end_period_packed_sha256":sha256_bytes(end.period_packed_coefficients()),"end_stop_packed_sha256":sha256_bytes(end.stop_packed_coefficients()),
            "new_potential_packed_sha256":sha256_bytes(&source.consumer.potential.packed_coefficients()?),"new_context_packed_sha256":cue.metadata().context_packed_sha256,"context_state_source_receipts":parameter_identities(&source)?.into_iter().filter(|(name,_)|context_state_parameter(name)).collect::<BTreeMap<_,_>>(),
            "independent_integer_reload":true,"source_file_copy_or_metadata_transplant":false}),
        )
    }

    pub fn prepare_on_device<'a>(
        &'a self,
        native: &'a NativeSourceRealizer,
        device: &Device,
    ) -> Result<PreparedSourceRealizer<'a>> {
        if self
            .context_state_parameters()
            .values()
            .any(|v| !v.device().same_device(device))
        {
            return Err(invalid("realizer prepared parameter device differs"));
        }
        self.prepare(native)
    }

    pub fn load_source_on_device(
        path: &Path,
        tokenizer_bytes: &[u8],
        device: &Device,
    ) -> Result<Self> {
        let source = Self::load_source(path, tokenizer_bytes)?;
        let result = Self {
            consumer: ConsumerWeights {
                context: source.consumer.context.to_device(device)?,
                potential: source.consumer.potential,
                no_read: source.consumer.no_read,
            },
            period: source.period,
            binding: source.binding,
            tokenizer_bytes: source.tokenizer_bytes,
            period_seed: source.period_seed,
        };
        result.validate()?;
        Ok(result)
    }

    pub fn load_context_potential_on_device(
        path: &Path,
        tokenizer_bytes: &[u8],
        device: &Device,
    ) -> Result<Self> {
        let mut source = Self::load_source_on_device(path, tokenizer_bytes, device)?;
        source.consumer.potential = source.consumer.potential.to_device(device)?;
        source.validate()?;
        Ok(source)
    }

    pub fn prepare_context_potential_on_device<'a>(
        &'a self,
        native: &'a NativeSourceRealizer,
        device: &Device,
    ) -> Result<PreparedSourceRealizer<'a>> {
        if !self.consumer.potential.device()?.same_device(device) {
            return Err(invalid("composed potential parameter device differs"));
        }
        let mut prepared = self.prepare_on_device(native, device)?;
        prepared.potential_coefficient_credit = true;
        Ok(prepared)
    }
}

pub struct ComposedCopyBankOutput {
    pub trace: uor_r4_integer::geometric_source_realizer::PrefixBankRealizerTrace,
    pub copy_raw: Tensor,
    pub context: crate::geometric_context::ContextQ4Output,
    pub state_credit_components: BTreeMap<&'static str, Tensor>,
}

struct ComposedCopyAssembly {
    context: crate::geometric_context::ContextQ4Output,
    sources: Vec<crate::geometric_context::ContextQ4Output>,
    response: Option<crate::geometric_context::ContextQ4Output>,
    contextual_copy: Tensor,
    cue_copy: Tensor,
    prefix_copy: Tensor,
    copy_raw: Tensor,
}

pub fn context_state_parameter(name: &str) -> bool {
    matches!(
        name,
        "consumer.context.token_transition"
            | "consumer.context.self_transition"
            | "consumer.context.neighbor_transition"
            | "consumer.context.token_root"
            | "consumer.context.self_root"
            | "consumer.context.neighbor_root"
            | "consumer.context.token_category"
            | "consumer.context.self_category"
            | "consumer.context.neighbor_category"
    )
}

fn verify_context_state_inventory(
    current: &BTreeMap<String, ParameterIdentity>,
    frozen: &BTreeMap<String, ParameterIdentity>,
) -> Result<()> {
    if current.keys().ne(frozen.keys()) {
        return Err(invalid("composed context source inventory differs"));
    }
    for (name, receipt) in frozen {
        let now = current
            .get(name)
            .ok_or_else(|| invalid("composed source family absent"))?;
        if now.shape != receipt.shape || (!context_state_parameter(name) && now != receipt) {
            return Err(invalid("composed changed frozen family or shape"));
        }
    }
    Ok(())
}

fn potential_parameter(name: &str) -> bool {
    matches!(
        name,
        "consumer.potential.content_unary"
            | "consumer.potential.context_unary"
            | "consumer.potential.pair"
            | "consumer.potential.content_radius"
            | "consumer.potential.context_radius"
            | "consumer.potential.content_presence"
            | "consumer.potential.context_presence"
    )
}

fn verify_context_potential_inventory(
    current: &BTreeMap<String, ParameterIdentity>,
    frozen: &BTreeMap<String, ParameterIdentity>,
) -> Result<()> {
    if current.keys().ne(frozen.keys()) {
        return Err(invalid(
            "composed context/potential source inventory differs",
        ));
    }
    for (name, receipt) in frozen {
        let now = current
            .get(name)
            .ok_or_else(|| invalid("composed source family absent"))?;
        if now.shape != receipt.shape
            || (!context_state_parameter(name) && !potential_parameter(name) && now != receipt)
        {
            return Err(invalid(
                "composed context/potential changed frozen family or shape",
            ));
        }
    }
    Ok(())
}

impl PreparedSourceRealizer<'_> {
    /// Current in-memory execution receipt, including rebound parameter metadata.
    /// This is not a claim that the snapshot was independently saved and reloaded.
    pub fn execution_binding(
        &self,
    ) -> Result<uor_r4_integer::geometric_source_realizer::NativeArtifactBinding> {
        self.native.execution_binding()
    }

    pub fn binding(&self) -> &SourceActionBinding {
        &self.source.binding
    }

    pub fn context_output(
        &self,
        ids: &[u32],
        reset_each_token: bool,
    ) -> Result<crate::geometric_context::ContextQ4Output> {
        if ids.is_empty() {
            return Err(invalid("context diagnostic input is empty"));
        }
        self.consumer
            .context
            .forward(ids, 1, ids.len(), reset_each_token)
    }

    fn assemble_bank_composed_copy(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        actual_prefix: &[u32],
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
        trace: &uor_r4_integer::geometric_source_realizer::PrefixBankRealizerTrace,
        prefix_temporal_utility: bool,
    ) -> Result<ComposedCopyAssembly> {
        let bank = &trace.cue_bank.bank;
        let carrier = &trace.cue_bank.carrier;
        let transport = &trace.prefix;
        let ids = &bank.context.tokens;
        let time = ids.len();
        let c = self.source.consumer.config();
        let width = c.heads * c.lanes_per_head;
        let count = bank.candidates.len();
        if time == 0
            || query.is_empty()
            || count == 0
            || bank.heads.len() != c.heads
            || carrier.query.token_ids != query
            || transport.response.token_ids != actual_prefix
        {
            return Err(invalid("composed native bank/query/prefix shape differs"));
        }
        let context = self.consumer.context.forward(ids, 1, time, false)?;
        if !context_replay_matches(&bank.context, &context.trace) {
            return Err(invalid("composed full context replay differs"));
        }
        let positions = bank
            .candidates
            .iter()
            .enumerate()
            .map(|(j, v)| {
                if v.bank_index != j || v.context_position >= time - 1 {
                    return Err(invalid("composed candidate replay position differs"));
                }
                u32::try_from(v.context_position)
                    .map_err(|_| invalid("composed context position exceeds u32"))
            })
            .collect::<Result<Vec<_>>>()?;
        let absent = AddressLane::new(H4Code::IDENTITY.index(), 0, false)
            .map_err(|e| invalid(e.to_string()))?;
        // Every factual Source occurrence is retained in physical bank order.
        // The loss consumes no other context-pair output; the full chronological
        // context graph above remains connected to these query/source reads.
        let selected_pairs = positions
            .iter()
            .map(|&position| ContextPotentialPair {
                batch: 0,
                query: time - 1,
                key: position as usize,
            })
            .collect::<Vec<_>>();
        let copy = frozen_potential_selected_forward(
            &self.source.consumer.potential,
            &vec![absent; time * width],
            &context,
            &selected_pairs,
        )?;
        let coefficient_credit = if self.potential_coefficient_credit {
            let mut pairs = Vec::with_capacity(c.heads * count);
            for h in 0..c.heads {
                let q = ((time - 1) * c.heads + h) * c.lanes_per_head;
                for &position in &positions {
                    let k = (position as usize * c.heads + h) * c.lanes_per_head;
                    pairs.push(PotentialQ4Pair {
                        head: h,
                        query_content: &copy.content_codes[q..q + c.lanes_per_head],
                        candidate_content: &copy.content_codes[k..k + c.lanes_per_head],
                        query_context: &copy.context_codes[q..q + c.lanes_per_head],
                        candidate_context: &copy.context_codes[k..k + c.lanes_per_head],
                    });
                }
            }
            let mut chunks = Vec::new();
            for (chunk_index, selected) in pairs.chunks(4096).enumerate() {
                let out = self
                    .source
                    .consumer
                    .potential
                    .forward_selected_codes(selected)?;
                for (j, &score) in out.scores_q24.iter().enumerate() {
                    let flat = chunk_index * 4096 + j;
                    let h = flat / count;
                    let candidate = flat % count;
                    if score != copy.scores_q24[candidate * c.heads + h] {
                        return Err(invalid("composed coefficient/context hard scores differ"));
                    }
                }
                // Subtraction first preserves the exact native forward anchor.
                chunks.push((&out.scores - out.scores.detach())?);
            }
            Some(Tensor::cat(&chunks, 0)?.reshape((c.heads, count))?)
        } else {
            None
        };
        let query_output = self
            .consumer
            .context
            .forward(query, 1, query.len(), false)?;
        let mut cues = Vec::with_capacity(carrier.cues.len());
        for local in &carrier.cues {
            let previous = local
                .source_segment_index
                .checked_sub(1)
                .ok_or_else(|| invalid("composed cue predecessor absent"))?;
            let Some(SourceBankSegment::Context { token_ids, .. }) = segments.get(previous) else {
                return Err(invalid("composed cue input not Context"));
            };
            if previous != local.context_segment_index
                || *token_ids != local.state.token_ids.as_slice()
                || token_ids.is_empty()
                || !matches!(
                    segments.get(local.source_segment_index),
                    Some(SourceBankSegment::Source { .. })
                )
            {
                return Err(invalid("composed cue tokens/reset mapping differs"));
            }
            cues.push(
                self.consumer
                    .context
                    .forward(token_ids, 1, token_ids.len(), false)?,
            );
        }
        let cue_credit = frozen_cue_state_forward(
            carrier.metadata.potential,
            cue.packed_coefficients(),
            cue.joint(),
            carrier,
            &query_output,
            &cues,
        )?;
        if cue_credit.scores_q24 != carrier.copy_q24 {
            return Err(invalid("composed cue hard scores differ"));
        }
        let mut sources = Vec::with_capacity(transport.sources.len());
        for source in &transport.sources {
            let Some(SourceBankSegment::Source { view, .. }) =
                segments.get(source.source_segment_index)
            else {
                return Err(invalid("composed local source input not Source"));
            };
            if source.token_ids.as_slice() != view.emitted_token_ids()
                || source.token_ids.is_empty()
            {
                return Err(invalid("composed local emitted source tokens differ"));
            }
            sources.push(self.consumer.context.forward(
                &source.token_ids,
                1,
                source.token_ids.len(),
                false,
            )?);
        }
        let response = if actual_prefix.is_empty() {
            None
        } else {
            Some(
                self.consumer
                    .context
                    .forward(actual_prefix, 1, actual_prefix.len(), false)?,
            )
        };
        let prefix_credit = if prefix_temporal_utility {
            crate::geometric_transport_state_credit::frozen_prefix_state_forward_temporal_utility(
                prefix,
                transport,
                &sources,
                response.as_ref(),
            )?
        } else {
            frozen_prefix_state_forward(prefix, transport, &sources, response.as_ref())?
        };
        if prefix_credit.scores_q24 != transport.copy_q24 {
            return Err(invalid("composed prefix hard scores differ"));
        }
        let mut contextual_heads = Vec::with_capacity(c.heads);
        let mut copy_heads = Vec::with_capacity(c.heads);
        for h in 0..c.heads {
            if bank.heads[h].scores_q24.len() != count {
                return Err(invalid("composed Copy head shape differs"));
            }
            for j in 0..count {
                let score = copy.scores_q24[j * c.heads + h]
                    .checked_add(cue_credit.scores_q24[h][j])
                    .and_then(|x| x.checked_add(prefix_credit.scores_q24[h][j]))
                    .ok_or_else(|| invalid("composed Copy score overflow"))?;
                if score != bank.heads[h].scores_q24[j] {
                    return Err(invalid("composed complete native Copy scores differ"));
                }
            }
            let contextual = copy.scores.i((.., h))?;
            let contextual = if let Some(credit) = &coefficient_credit {
                (&contextual + credit.i(h)?)?
            } else {
                contextual
            };
            let raw = (&contextual + cue_credit.scores.i(h)?)?;
            copy_heads.push((&raw + prefix_credit.scores.i(h)?)?);
            contextual_heads.push(contextual);
        }
        Ok(ComposedCopyAssembly {
            context,
            sources,
            response,
            contextual_copy: Tensor::stack(&contextual_heads, 0)?,
            cue_copy: cue_credit.scores,
            prefix_copy: prefix_credit.scores,
            copy_raw: Tensor::stack(&copy_heads, 0)?.sum(0)?,
        })
    }

    pub fn forward_bank_composed_copy(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        actual_prefix: &[u32],
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
    ) -> Result<ComposedCopyBankOutput> {
        self.forward_bank_composed_copy_with_credit(
            segments,
            query,
            actual_prefix,
            cue,
            prefix,
            false,
        )
    }

    pub fn forward_bank_composed_copy_with_prefix_utility(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        actual_prefix: &[u32],
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
    ) -> Result<ComposedCopyBankOutput> {
        self.forward_bank_composed_copy_with_credit(
            segments,
            query,
            actual_prefix,
            cue,
            prefix,
            true,
        )
    }

    fn forward_bank_composed_copy_with_credit(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        actual_prefix: &[u32],
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
        prefix_temporal_utility: bool,
    ) -> Result<ComposedCopyBankOutput> {
        let trace = self.native.read_bank_with_prefix_transport(
            segments,
            query,
            actual_prefix,
            cue,
            prefix,
        )?;
        let assembled = self.assemble_bank_composed_copy(
            segments,
            query,
            actual_prefix,
            cue,
            prefix,
            &trace,
            prefix_temporal_utility,
        )?;
        // Bind the factual forward to the native integer sum once. Summing
        // separately converted components can round differently, while the
        // vocabulary loss requires exact raw-score parity.
        let hard_copy: Vec<f32> = (0..trace.cue_bank.bank.candidates.len())
            .map(|j| {
                let sum = trace
                    .cue_bank
                    .bank
                    .heads
                    .iter()
                    .try_fold(0i64, |sum, head| {
                        sum.checked_add(head.scores_q24[j])
                            .ok_or_else(|| invalid("composed summed Copy score overflow"))
                    })?;
                Ok((sum as f64 / (1u64 << 24) as f64) as f32)
            })
            .collect::<Result<_>>()?;
        let hard_copy = Tensor::from_vec(
            hard_copy,
            assembled.copy_raw.shape(),
            assembled.copy_raw.device(),
        )?;
        let copy_raw = (&hard_copy + (&assembled.copy_raw - assembled.copy_raw.detach())?)?;
        let state_credit_components = [
            ("contextual_copy", assembled.contextual_copy.sum(0)?),
            ("cue_copy", assembled.cue_copy.sum(0)?),
            ("prefix_copy", assembled.prefix_copy.sum(0)?),
        ]
        .into_iter()
        .collect();
        Ok(ComposedCopyBankOutput {
            trace,
            copy_raw,
            context: assembled.context,
            state_credit_components,
        })
    }
}
