//! Opt-in floating-point research reference. This is not the R4G1 serving kernel.
mod adapter;
mod environment;
mod loader;
mod numerics;
pub use environment::floating_point_environment;

pub use adapter::Refusal;
pub use loader::{ExpectedBinding, Manifest, ValidationAudit};
pub use numerics::Diagnostics;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};

pub const CONTRACT_SHA256: &str =
    "e3565728dfa872d9c260dd3d391ae59e0b0d454ebead20cd15cb075b24796115";
pub const OPERATOR_PROFILE: &str = "cpu-scalar-f32-f64-1086/1";
pub const OPERATION: &str = "answer_four_fact_raw_text/v1";
pub(super) const CONTRACT: &str =
    include_str!("../../../../docs/r4_native_reference_1086_contract.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NativeErrorTag {
    ContainerLimit,
    InvalidContainer,
    ArtifactIdentityMismatch,
    UnsupportedManifest,
    UnsupportedProfile,
    SourceBindingMismatch,
    InvalidComponent,
    InvalidTensor,
    InvalidCodecPolicy,
    InvalidFrameTable,
    StateIdentityMismatch,
    UnavailableArtifact,
    UnavailableNativeQualification,
    Busy,
    NumericalFailure,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeError {
    pub tag: NativeErrorTag,
    pub component: Option<String>,
    pub offset: Option<u64>,
}
impl NativeError {
    pub(super) fn new(tag: NativeErrorTag) -> Self {
        Self {
            tag,
            component: None,
            offset: None,
        }
    }
    pub(super) fn at(tag: NativeErrorTag, component: &str, offset: usize) -> Self {
        Self {
            tag,
            component: Some(component.to_owned()),
            offset: Some(offset as u64),
        }
    }
}
impl std::fmt::Display for NativeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.tag)
    }
}
impl std::error::Error for NativeError {}

/// Exact caller-supplied buffer. Transport decoding belongs to the host.
pub struct RawRequest<'a> {
    pub schema: &'a str,
    pub text: &'a [u8],
}
#[derive(Debug, Clone, Serialize)]
pub struct ModelToken {
    pub schema: &'static str,
    pub status: &'static str,
    pub policy_sha256: String,
    pub raw_text_sha256: String,
    pub derived_input_sha256: String,
    pub reader_file_cid: String,
    pub core_file_cid: String,
    pub frame_tree_cid: String,
    pub token_id: u32,
    pub token: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum TextResult {
    Model(ModelToken),
    Refusal(Refusal),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeIdentity {
    pub native_binary_sha256: String,
    pub runtime_receipt_sha256: String,
}

/// A host trust anchor. The host must obtain this digest from its independent
/// accepted release, never from an artifact's assertion of permission.
pub struct ComparisonAdmission {
    release_sha256: String,
    runtime: RuntimeIdentity,
}
impl ComparisonAdmission {
    pub fn from_trusted_release(
        release: &[u8],
        expected_release_sha256: &str,
        runtime: RuntimeIdentity,
    ) -> Result<Self, NativeError> {
        if sha256(release) != expected_release_sha256
            || !valid_hex(expected_release_sha256, 64)
            || !valid_hex(&runtime.native_binary_sha256, 64)
            || !valid_hex(&runtime.runtime_receipt_sha256, 64)
        {
            return Err(NativeError::new(NativeErrorTag::SourceBindingMismatch));
        }
        Ok(Self {
            release_sha256: expected_release_sha256.to_owned(),
            runtime,
        })
    }
}
#[derive(Debug, Serialize)]
pub struct ComparisonOutput {
    pub result: TextResult,
    pub parsed: Option<Value>,
    pub diagnostics: Option<Diagnostics>,
    pub receipt: Value,
}

/// Verified, owned, immutable model state; usable by research callers only
/// after a separately trusted empirical qualification has been attached.
pub struct LoadedResearchReference {
    artifact: Vec<u8>,
    manifest: Manifest,
    weights: numerics::Weights,
    frames: numerics::Frames,
    vocabulary: Vec<String>,
    artifact_sha256: String,
    qualification: Option<(String, RuntimeIdentity)>,
    busy: AtomicBool,
}
impl LoadedResearchReference {
    pub fn load(bytes: Vec<u8>, expected: &ExpectedBinding) -> Result<Self, NativeError> {
        loader::load(bytes, expected)
    }
    pub fn load_audited(
        bytes: Vec<u8>,
        expected: &ExpectedBinding,
    ) -> (Result<Self, NativeError>, ValidationAudit) {
        loader::load_audited(bytes, expected)
    }
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn artifact_sha256(&self) -> &str {
        &self.artifact_sha256
    }
    pub fn owned_artifact_bytes(&self) -> usize {
        self.artifact.len()
    }
    pub fn capability(&self) -> Value {
        json!({"schema":"uor-r4.native-reference-capability/1","operation":OPERATION,
            "artifact_sha256":self.artifact_sha256,"native_state_sha256":self.manifest.native_state_sha256,
            "codec_cid":self.manifest.source_binding["assets"]["vocabulary"]["cid"],
            "policy_sha256":self.manifest.source_binding["policy_sha256"],"operator_profile":OPERATOR_PROFILE,
            "reference_evidence":"CLAUSE_ADAPTER_PRESERVED",
            "native_behavior":if self.qualification.is_some(){"EMPIRICAL_NATIVE_REFERENCE_PRESERVED"}else{"NOT_RUN"},
            "qualification_receipt_sha256":self.qualification.as_ref().map(|x|&x.0),
            "stateless":true,"execution":"cpu-floating-point-research-reference",
            "scope":"controlled-language-known-vocabulary-four-facts-one-query",
            "general_generation":false,"general_context":false,"coding":false,"final_integer_kernel":false})
    }
    /// Validate an externally trusted accepted qualification; no manifest flag
    /// or untrusted result record can promote this engine by itself.
    pub fn qualify(
        &mut self,
        receipt: &[u8],
        trusted_receipt_sha256: &str,
        runtime: RuntimeIdentity,
    ) -> Result<(), NativeError> {
        let v: Value = serde_json::from_slice(receipt)
            .map_err(|_| NativeError::new(NativeErrorTag::UnavailableNativeQualification))?;
        let contract = loader::contract()?;
        let fields = contract["exact_fields"]["qualification_binding"]
            .as_object()
            .ok_or_else(|| NativeError::new(NativeErrorTag::UnsupportedManifest))?;
        if loader::canonical(&v)
            .map_err(|_| NativeError::new(NativeErrorTag::UnavailableNativeQualification))?
            != receipt
            || sha256(receipt) != trusted_receipt_sha256
            || !valid_hex(trusted_receipt_sha256, 64)
            || !loader::exact_keys(&v, fields.keys().map(String::as_str))
            || v["schema"] != "uor-r4.native-reference-qualification/1"
            || v["terminal"] != "NATIVE_REFERENCE_PRESERVED"
            || v["artifact_sha256"] != self.artifact_sha256
            || v["native_state_sha256"] != self.manifest.native_state_sha256
            || v["native_binary_sha256"] != runtime.native_binary_sha256
            || v["runtime_receipt_sha256"] != runtime.runtime_receipt_sha256
            || v["contract_sha256"] != CONTRACT_SHA256
            || v["operator_profile"] != OPERATOR_PROFILE
            || v["request_schema"] != "uor-r4.text-to-clauses/1"
            || v["result_schema"] != "uor-r4.text-binding-result/1"
            || !v["accepted_review_sha256"]
                .as_str()
                .is_some_and(|x| valid_hex(x, 64))
            || !v["comparison_result_sha256"]
                .as_str()
                .is_some_and(|x| valid_hex(x, 64))
            || !valid_hex(&runtime.native_binary_sha256, 64)
            || !valid_hex(&runtime.runtime_receipt_sha256, 64)
        {
            return Err(NativeError::new(
                NativeErrorTag::UnavailableNativeQualification,
            ));
        }
        self.qualification = Some((trusted_receipt_sha256.to_owned(), runtime));
        Ok(())
    }
    pub fn answer(&self, request: RawRequest<'_>) -> Result<TextResult, NativeError> {
        let (_, runtime) = self
            .qualification
            .as_ref()
            .ok_or_else(|| NativeError::new(NativeErrorTag::UnavailableNativeQualification))?;
        Ok(self.execute(request, runtime, true, &mut 0)?.result)
    }
    pub fn compare(
        &self,
        request: RawRequest<'_>,
        admission: &ComparisonAdmission,
        remaining_forwards: u32,
        attempted_forwards: &mut u32,
    ) -> Result<ComparisonOutput, NativeError> {
        *attempted_forwards = 0;
        if admission.release_sha256 != self.manifest.export_provenance.release_sha256 {
            return Err(NativeError::new(NativeErrorTag::SourceBindingMismatch));
        }
        self.execute(
            request,
            &admission.runtime,
            remaining_forwards > 0,
            attempted_forwards,
        )
    }
    fn execute(
        &self,
        request: RawRequest<'_>,
        runtime: &RuntimeIdentity,
        allow_forward: bool,
        attempted_forwards: &mut u32,
    ) -> Result<ComparisonOutput, NativeError> {
        self.busy
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| NativeError::new(NativeErrorTag::Busy))?;
        struct Guard<'a>(&'a AtomicBool);
        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                self.0.store(false, Ordering::Release);
            }
        }
        let _guard = Guard(&self.busy);
        let parsed = if request.schema != "uor-r4.text-to-clauses/1" {
            Err(Refusal::unsupported_schema())
        } else {
            adapter::parse(request.text)
        };
        let (result, diagnostics, parsed, forwards) = match parsed {
            Err(refusal) => (TextResult::Refusal(refusal), None, None, 0u32),
            Ok(p) => {
                if !allow_forward {
                    return Err(NativeError::new(NativeErrorTag::NumericalFailure));
                }
                floating_point_environment()?;
                *attempted_forwards = 1;
                let evaluated =
                    numerics::evaluate(&self.weights, &self.frames, &p.inputs, &p.lengths);
                floating_point_environment()?;
                let d =
                    evaluated.map_err(|_| NativeError::new(NativeErrorTag::NumericalFailure))?;
                let token = self
                    .vocabulary
                    .get(d.token_id as usize)
                    .ok_or_else(|| NativeError::new(NativeErrorTag::NumericalFailure))?
                    .clone();
                let result = TextResult::Model(ModelToken {
                    schema: "uor-r4.text-binding-result/1",
                    status: "MODEL_TOKEN",
                    policy_sha256: self.manifest.source_binding["policy_sha256"]
                        .as_str()
                        .unwrap_or_default()
                        .to_owned(),
                    raw_text_sha256: p.raw_text_sha256.clone(),
                    derived_input_sha256: p.derived_input_sha256.clone(),
                    reader_file_cid: self.manifest.source_binding["assets"]["reader"]["cid"]
                        .as_str()
                        .unwrap_or_default()
                        .to_owned(),
                    core_file_cid: self.manifest.source_binding["assets"]["core"]["cid"]
                        .as_str()
                        .unwrap_or_default()
                        .to_owned(),
                    frame_tree_cid: self.manifest.source_binding["frame_tree_cid"]
                        .as_str()
                        .unwrap_or_default()
                        .to_owned(),
                    token_id: d.token_id,
                    token,
                });
                let parsed = json!({"inputs":[p.inputs],"lengths":[p.lengths],"token_spans":p.token_spans,"clause_spans":p.clause_spans,"raw_text_sha256":p.raw_text_sha256,"derived_input_sha256":p.derived_input_sha256});
                (result, Some(d), Some(parsed), 1)
            }
        };
        let value = serde_json::to_value(&result)
            .map_err(|_| NativeError::new(NativeErrorTag::NumericalFailure))?;
        let receipt = json!({"schema":"uor-r4.native-reference-receipt/1","operation":OPERATION,
            "artifact_sha256":self.artifact_sha256,"native_state_sha256":self.manifest.native_state_sha256,
            "native_binary_sha256":runtime.native_binary_sha256,"runtime_receipt_sha256":runtime.runtime_receipt_sha256,
            "contract_sha256":CONTRACT_SHA256,"operator_profile":OPERATOR_PROFILE,"raw_text_sha256":sha256(request.text),
            "result_sha256":sha256(&loader::canonical(&value)?),"logical_forwards":forwards,"parameter_updates":0});
        Ok(ComparisonOutput {
            result,
            parsed,
            diagnostics,
            receipt,
        })
    }
}
pub fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub(super) fn valid_hex(s: &str, n: usize) -> bool {
    s.len() == n
        && s.bytes()
            .all(|x| x.is_ascii_digit() || (b'a'..=b'f').contains(&x))
}
