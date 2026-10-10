//! Optional local-continuation energy over the existing Generate prototypes.
//! The caller supplies its independently replayed query/actual-prefix carrier.
//! This field neither selects a Source occurrence nor changes vocabulary identity.
#![forbid(unsafe_code)]

use super::geometric_generate::{GenerateMetadata, NativeGeometricGenerate};
use super::integrated_attention::geometry::{AlgebraReadCounts, GeometryError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_integer::{geometric_source_realizer::NativeArtifactBinding, h4_tables::H4Code};

const SCHEMA: &str = "uor-r4.native-continuation-field/1";
const POLICY: &str = "bound-Generate-prototypes;directed-inverse-continuation-times-prototype;shared-lane-full120-unary;strict-q4[-7,7];score-code-shift20-Q24;all-steps/1";
const SHARED_SCHEMA: &str = "uor-r4.native-continuation-field/2";
const SHARED_POLICY: &str = "bound-Generate-prototypes;directed-inverse-continuation-times-prototype;shared-lane-full120-unary;strict-q4[-7,7];score-code-shift22-Q24;each-physical-Copy-and-Generate-alias;one-common-clip;all-steps/2";
const SHARED_SCORE_SHIFT: u32 = 22;
const MAX_LANES: usize = 8;
const BYTES_PER_LANE: usize = 60;
const SCORE_SHIFT: u32 = 20;
const MAX_ARTIFACT_BYTES: usize = 16 * 1024;
const CROSS_SCHEMA: &str = "uor-r4.native-continuation-field/3";
const CROSS_POLICY: &str = "bound-Generate-prototypes;ordered-inverse-factual-times-prototype,inverse-local-times-prototype;shared-lane-full120-cross-state;strict-q4[-7,7];128-stride-zero-padding;score-code-shift22-Q24;each-physical-Copy-and-Generate-alias;one-common-clip;all-steps/3";
const MAX_CROSS_ARTIFACT_BYTES: usize = 512 * 1024;

#[derive(Debug)]
pub enum ContinuationFieldError {
    Shape(&'static str),
    Binding(&'static str),
    Coefficient { index: usize, value: i8 },
    Geometry(GeometryError),
    Artifact(String),
}
impl std::fmt::Display for ContinuationFieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ContinuationFieldError {}
impl From<GeometryError> for ContinuationFieldError {
    fn from(value: GeometryError) -> Self {
        Self::Geometry(value)
    }
}
type Result<T> = std::result::Result<T, ContinuationFieldError>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContinuationFieldMetadata {
    pub schema: String,
    pub policy: String,
    pub source_binding: NativeArtifactBinding,
    pub generate_sha256: String,
    pub generate_metadata: GenerateMetadata,
    pub prototype_sha256: String,
    pub lanes: usize,
    pub vocab_size: usize,
    pub score_shift: u32,
    pub payload_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    metadata: ContinuationFieldMetadata,
    packed_unary: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    packed_cross_state: Option<Vec<u8>>,
}
#[derive(Clone, Debug)]
pub struct NativeContinuationField {
    artifact: Artifact,
    prototype_offsets: Vec<usize>,
    lane_byte_offsets: [usize; MAX_LANES],
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuationReadCounts {
    pub algebra: AlgebraReadCounts,
    pub prototype_reads: u64,
    pub coefficient_reads: u64,
    pub scores: u64,
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn digest_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn decode(byte: u8, high: bool) -> i8 {
    let nibble = if high { byte >> 4 } else { byte & 15 };
    if nibble < 8 {
        nibble as i8
    } else {
        nibble as i8 - 16
    }
}
fn bump(value: &mut u64) -> Result<()> {
    *value = value.checked_add(1).ok_or(GeometryError::CounterOverflow)?;
    Ok(())
}
impl NativeContinuationField {
    pub fn compile(
        binding: &NativeArtifactBinding,
        generate: &NativeGeometricGenerate,
        packed_unary: &[u8],
    ) -> Result<Self> {
        Self::compile_policy(binding, generate, packed_unary, false)
    }
    /// Explicit v2 shared-action field; v1 Generate-only bytes stay unchanged.
    pub fn compile_shared_action(
        binding: &NativeArtifactBinding,
        generate: &NativeGeometricGenerate,
        packed_unary: &[u8],
    ) -> Result<Self> {
        Self::compile_policy(binding, generate, packed_unary, true)
    }
    /// Shared token energy conditioned jointly on factual and local reply states.
    /// The packed pair payload uses 128-stride padding, with every unused ID zero.
    pub fn compile_cross_state(
        binding: &NativeArtifactBinding,
        generate: &NativeGeometricGenerate,
        packed_pairs: &[u8],
    ) -> Result<Self> {
        if generate.lanes() == 0
            || generate.lanes() > MAX_LANES
            || packed_pairs.len() != generate.lanes() << 13
        {
            return Err(ContinuationFieldError::Shape(
                "cross-state packed dimensions",
            ));
        }
        let mut artifact = Self::compile_shared_action(
            binding,
            generate,
            &vec![0; generate.lanes() * BYTES_PER_LANE],
        )?
        .artifact;
        artifact.metadata.schema = CROSS_SCHEMA.into();
        artifact.metadata.policy = CROSS_POLICY.into();
        artifact.metadata.payload_sha256 = hash(packed_pairs);
        artifact.packed_unary.clear();
        artifact.packed_cross_state = Some(packed_pairs.to_vec());
        Self::admit(artifact, binding, generate)
    }
    fn compile_policy(
        binding: &NativeArtifactBinding,
        generate: &NativeGeometricGenerate,
        packed_unary: &[u8],
        shared: bool,
    ) -> Result<Self> {
        let metadata = ContinuationFieldMetadata {
            schema: if shared { SHARED_SCHEMA } else { SCHEMA }.into(),
            policy: if shared { SHARED_POLICY } else { POLICY }.into(),
            source_binding: binding.clone(),
            generate_sha256: hash(
                &generate
                    .to_bytes()
                    .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?,
            ),
            generate_metadata: generate.metadata().clone(),
            prototype_sha256: hash(generate.prototypes()),
            lanes: generate.lanes(),
            vocab_size: generate.vocab_size(),
            score_shift: if shared {
                SHARED_SCORE_SHIFT
            } else {
                SCORE_SHIFT
            },
            payload_sha256: hash(packed_unary),
        };
        Self::admit(
            Artifact {
                metadata,
                packed_unary: packed_unary.to_vec(),
                packed_cross_state: None,
            },
            binding,
            generate,
        )
    }
    pub fn zeroed(
        binding: &NativeArtifactBinding,
        generate: &NativeGeometricGenerate,
    ) -> Result<Self> {
        Self::compile(
            binding,
            generate,
            &vec![0; generate.lanes() * BYTES_PER_LANE],
        )
    }
    pub fn from_bytes(
        bytes: &[u8],
        binding: &NativeArtifactBinding,
        generate: &NativeGeometricGenerate,
    ) -> Result<Self> {
        if bytes.len() > MAX_CROSS_ARTIFACT_BYTES {
            return Err(ContinuationFieldError::Shape(
                "continuation artifact byte limit",
            ));
        }
        let artifact: Artifact = serde_json::from_slice(bytes)
            .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?;
        if artifact.metadata.schema != CROSS_SCHEMA && bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(ContinuationFieldError::Shape(
                "legacy continuation artifact byte limit",
            ));
        }
        Self::admit(artifact, binding, generate)
    }
    fn admit(
        artifact: Artifact,
        binding: &NativeArtifactBinding,
        generate: &NativeGeometricGenerate,
    ) -> Result<Self> {
        let m = &artifact.metadata;
        for digest in [
            &binding.metadata_sha256,
            &binding.identity.tokenizer_sha256,
            &binding.identity.parent_checkpoint_manifest_sha256,
            &binding.identity.parent_model_sha256,
            &binding.identity.parent_config_sha256,
        ] {
            if !digest_valid(digest) {
                return Err(ContinuationFieldError::Binding(
                    "invalid native binding digest",
                ));
            }
        }
        if !((m.schema == SCHEMA && m.policy == POLICY && m.score_shift == SCORE_SHIFT)
            || (m.schema == SHARED_SCHEMA
                && m.policy == SHARED_POLICY
                && m.score_shift == SHARED_SCORE_SHIFT)
            || (m.schema == CROSS_SCHEMA
                && m.policy == CROSS_POLICY
                && m.score_shift == SHARED_SCORE_SHIFT))
        {
            return Err(ContinuationFieldError::Binding(
                "continuation schema/policy/units",
            ));
        }
        if m.source_binding != *binding
            || binding.identity.tokenizer_sha256 != generate.metadata().tokenizer_sha256
        {
            return Err(ContinuationFieldError::Binding(
                "continuation native parent/tokenizer",
            ));
        }
        if m.lanes == 0
            || m.lanes > MAX_LANES
            || m.lanes != generate.lanes()
            || m.vocab_size != generate.vocab_size()
            || m.generate_metadata != *generate.metadata()
            || generate.metadata().score_shift != SCORE_SHIFT
        {
            return Err(ContinuationFieldError::Binding(
                "continuation Generate dimensions/metadata",
            ));
        }
        let generate_bytes = generate
            .to_bytes()
            .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?;
        if m.generate_sha256 != hash(&generate_bytes)
            || m.prototype_sha256 != hash(generate.prototypes())
        {
            return Err(ContinuationFieldError::Binding(
                "continuation Generate/prototype digest",
            ));
        }
        let payload = if m.schema == CROSS_SCHEMA {
            let pairs =
                artifact
                    .packed_cross_state
                    .as_deref()
                    .ok_or(ContinuationFieldError::Shape(
                        "cross-state pair payload absent",
                    ))?;
            if !artifact.packed_unary.is_empty() || pairs.len() != m.lanes << 13 {
                return Err(ContinuationFieldError::Shape(
                    "cross-state packed dimensions",
                ));
            }
            for lane in 0..m.lanes {
                for factual in 0..128usize {
                    for local in 0..128usize {
                        if factual >= 120 || local >= 120 {
                            let index = (lane << 13) | (factual << 6) | (local >> 1);
                            if decode(pairs[index], local & 1 != 0) != 0 {
                                return Err(ContinuationFieldError::Shape(
                                    "cross-state nonzero padding",
                                ));
                            }
                        }
                    }
                }
            }
            pairs
        } else {
            if artifact.packed_cross_state.is_some()
                || artifact.packed_unary.len() != m.lanes * BYTES_PER_LANE
            {
                return Err(ContinuationFieldError::Shape(
                    "continuation packed unary dimensions or foreign pair payload",
                ));
            }
            artifact.packed_unary.as_slice()
        };
        if m.payload_sha256 != hash(payload) {
            return Err(ContinuationFieldError::Binding(
                "continuation coefficient digest",
            ));
        }
        for (byte_index, &byte) in payload.iter().enumerate() {
            for high in [false, true] {
                if decode(byte, high) == -8 {
                    return Err(ContinuationFieldError::Coefficient {
                        index: byte_index * 2 + usize::from(high),
                        value: -8,
                    });
                }
            }
        }
        let prototype_offsets = (0..m.vocab_size).map(|token| token * m.lanes).collect();
        let mut lane_byte_offsets = [0; MAX_LANES];
        for (lane, offset) in lane_byte_offsets.iter_mut().enumerate().take(m.lanes) {
            *offset = lane * BYTES_PER_LANE;
        }
        Ok(Self {
            artifact,
            prototype_offsets,
            lane_byte_offsets,
        })
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self.artifact)
            .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))
    }
    pub fn metadata(&self) -> &ContinuationFieldMetadata {
        &self.artifact.metadata
    }
    pub fn applies_to_copy(&self) -> bool {
        self.metadata().schema == SHARED_SCHEMA || self.is_cross_state()
    }
    pub fn is_cross_state(&self) -> bool {
        self.metadata().schema == CROSS_SCHEMA
    }
    pub fn packed_cross_state(&self) -> Option<&[u8]> {
        self.artifact.packed_cross_state.as_deref()
    }
    pub fn coefficient_cross_state(
        &self,
        lane: usize,
        factual_relative: u8,
        local_relative: u8,
    ) -> Result<i8> {
        if !self.is_cross_state()
            || lane >= self.lanes()
            || factual_relative >= 120
            || local_relative >= 120
        {
            return Err(ContinuationFieldError::Shape(
                "cross-state lane/relative/mode",
            ));
        }
        let pairs = self
            .packed_cross_state()
            .ok_or(ContinuationFieldError::Shape(
                "cross-state pair payload absent",
            ))?;
        let index =
            (lane << 13) | (usize::from(factual_relative) << 6) | usize::from(local_relative >> 1);
        Ok(decode(pairs[index], local_relative & 1 != 0))
    }
    pub fn score_shift(&self) -> u32 {
        self.metadata().score_shift
    }
    pub fn lanes(&self) -> usize {
        self.metadata().lanes
    }
    pub fn vocab_size(&self) -> usize {
        self.metadata().vocab_size
    }
    pub fn packed_unary(&self) -> &[u8] {
        &self.artifact.packed_unary
    }
    pub fn generate_sha256(&self) -> &str {
        &self.metadata().generate_sha256
    }
    pub fn coefficient_unary(&self, lane: usize, relative: u8) -> Result<i8> {
        if self.is_cross_state() || lane >= self.lanes() || relative >= 120 {
            return Err(ContinuationFieldError::Shape("continuation lane/relative"));
        }
        let index = self.lane_byte_offsets[lane] + usize::from(relative >> 1);
        Ok(decode(self.packed_unary()[index], relative & 1 != 0))
    }
    /// Score the ordered factual/local interaction without allocation or floating point.
    /// Legacy fields ignore the additional factual input and retain their exact scorer.
    pub fn score_delta_with_factual_into(
        &self,
        factual: &[H4Code],
        local: &[H4Code],
        generate: &NativeGeometricGenerate,
        out: &mut [i64],
        counts: &mut ContinuationReadCounts,
    ) -> Result<()> {
        if !self.is_cross_state() {
            return self.score_delta_into(local, generate, out, counts);
        }
        if factual.len() != self.lanes()
            || local.len() != self.lanes()
            || out.len() != self.vocab_size()
        {
            return Err(ContinuationFieldError::Shape(
                "cross-state state/output dimensions",
            ));
        }
        if generate.metadata() != &self.metadata().generate_metadata {
            return Err(ContinuationFieldError::Binding(
                "cross-state serving Generate differs",
            ));
        }
        let mut factual_inverse = [0u8; MAX_LANES];
        let mut local_inverse = [0u8; MAX_LANES];
        for lane in 0..self.lanes() {
            factual_inverse[lane] = generate
                .algebra()
                .inverse_counted(factual[lane].index(), &mut counts.algebra)?;
            local_inverse[lane] = generate
                .algebra()
                .inverse_counted(local[lane].index(), &mut counts.algebra)?;
        }
        for (token, output) in out.iter_mut().enumerate() {
            let mut sum = 0i32;
            let base = self.prototype_offsets[token];
            for lane in 0..self.lanes() {
                let prototype = generate.prototypes()[base + lane];
                bump(&mut counts.prototype_reads)?;
                let factual_relative = generate.algebra().compose_counted(
                    factual_inverse[lane],
                    prototype,
                    &mut counts.algebra,
                )?;
                let local_relative = generate.algebra().compose_counted(
                    local_inverse[lane],
                    prototype,
                    &mut counts.algebra,
                )?;
                sum += i32::from(self.coefficient_cross_state(
                    lane,
                    factual_relative,
                    local_relative,
                )?);
                bump(&mut counts.coefficient_reads)?;
            }
            *output = i64::from(sum) << self.score_shift();
            bump(&mut counts.scores)?;
        }
        Ok(())
    }
    /// Overwrites every token delta. Admission hashes are not repeated in the hot
    /// path: the typed Generate has private immutable payload and bound metadata.
    pub fn score_delta_into(
        &self,
        state: &[H4Code],
        generate: &NativeGeometricGenerate,
        out: &mut [i64],
        counts: &mut ContinuationReadCounts,
    ) -> Result<()> {
        if self.is_cross_state() {
            return Err(ContinuationFieldError::Shape(
                "cross-state scoring requires factual and local states",
            ));
        }
        if state.len() != self.lanes() || out.len() != self.vocab_size() {
            return Err(ContinuationFieldError::Shape(
                "continuation state/output dimensions",
            ));
        }
        if generate.metadata() != &self.metadata().generate_metadata {
            return Err(ContinuationFieldError::Binding(
                "continuation serving Generate differs",
            ));
        }
        let mut inverse = [0u8; MAX_LANES];
        for (lane, code) in state.iter().enumerate() {
            inverse[lane] = generate
                .algebra()
                .inverse_counted(code.index(), &mut counts.algebra)?;
        }
        for (token, output) in out.iter_mut().enumerate() {
            let mut sum = 0i32;
            let base = self.prototype_offsets[token];
            for (lane, &inv) in inverse.iter().enumerate().take(self.lanes()) {
                let prototype = generate.prototypes()[base + lane];
                bump(&mut counts.prototype_reads)?;
                let relative =
                    generate
                        .algebra()
                        .compose_counted(inv, prototype, &mut counts.algebra)?;
                sum += i32::from(self.coefficient_unary(lane, relative)?);
                bump(&mut counts.coefficient_reads)?;
            }
            *output = i64::from(sum) << self.score_shift();
            bump(&mut counts.scores)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::integrated_attention::geometry::EnergyTables;
    use super::*;
    use uor_r4_integer::{
        geometric_source_actions::SourceActionBinding, geometric_source_realizer::ArtifactIdentity,
    };
    const TOKENIZER: &[u8] = br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    fn fixture() -> Result<(NativeArtifactBinding, NativeGeometricGenerate)> {
        let source = SourceActionBinding::new(TOKENIZER)
            .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?;
        let generate = NativeGeometricGenerate::compile(
            &source,
            2,
            &[1, 1, 0, 1, 3, 5, 5, 3, 31, 57, 31, 57],
            &[0; 3],
            EnergyTables::zeroed(2, vec![])?,
        )
        .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?;
        let binding = NativeArtifactBinding {
            metadata_sha256: "a".repeat(64),
            identity: ArtifactIdentity {
                tokenizer_sha256: generate.metadata().tokenizer_sha256.clone(),
                parent_checkpoint_manifest_sha256: "b".repeat(64),
                parent_model_sha256: "c".repeat(64),
                parent_config_sha256: "d".repeat(64),
            },
        };
        Ok((binding, generate))
    }
    fn codes(a: u8, b: u8) -> Result<Vec<H4Code>> {
        [a, b]
            .into_iter()
            .map(|v| {
                H4Code::try_from(v).map_err(|e| ContinuationFieldError::Artifact(e.to_string()))
            })
            .collect()
    }
    #[test]
    fn continuation_zero_and_round_trip_are_exact() -> Result<()> {
        let (binding, generate) = fixture()?;
        let field = NativeContinuationField::zeroed(&binding, &generate)?;
        let bytes = field.to_bytes()?;
        let restored = NativeContinuationField::from_bytes(&bytes, &binding, &generate)?;
        assert_eq!(bytes, restored.to_bytes()?);
        let mut out = vec![1; generate.vocab_size()];
        let mut counts = ContinuationReadCounts::default();
        restored.score_delta_into(&codes(31, 57)?, &generate, &mut out, &mut counts)?;
        assert_eq!(out, vec![0; generate.vocab_size()]);
        assert_eq!(counts.algebra.inverse_reads, 2);
        assert_eq!(counts.algebra.product_reads, 12);
        assert_eq!(counts.coefficient_reads, 12);
        assert_eq!(counts.prototype_reads, 12);
        assert_eq!(counts.scores, 6);
        Ok(())
    }
    #[test]
    fn continuation_uses_directed_signed_full_h4_relation() -> Result<()> {
        let (binding, generate) = fixture()?;
        let mut packed = vec![0; 120];
        // One lane covers every signed root with nonconstant coefficients.
        // Independent algebra expectations exercise directed relative indices.
        for relative in 0..120u8 {
            let c = if relative & 1 == 0 {
                (relative % 7) as i8 + 1
            } else {
                -((relative % 7) as i8 + 1)
            };
            let nibble = (c as u8) & 15;
            let index = usize::from(relative >> 1);
            if relative & 1 == 0 {
                packed[index] |= nibble;
            } else {
                packed[index] |= nibble << 4;
            }
        }
        let field = NativeContinuationField::compile(&binding, &generate, &packed)?;
        for state in 0..120u8 {
            let mut out = vec![0; 6];
            field.score_delta_into(
                &codes(state, 1)?,
                &generate,
                &mut out,
                &mut ContinuationReadCounts::default(),
            )?;
            for (token, &score) in out.iter().enumerate() {
                let relative = generate.algebra().compose(
                    generate.algebra().inverse(state)?,
                    generate.prototypes()[token * 2],
                )?;
                let expected = if relative & 1 == 0 {
                    (relative % 7) as i8 + 1
                } else {
                    -((relative % 7) as i8 + 1)
                };
                assert_eq!(score, i64::from(expected) << 20);
            }
        }
        Ok(())
    }
    #[test]
    fn continuation_rejects_parent_payload_and_generate_changes() -> Result<()> {
        let (binding, generate) = fixture()?;
        let field = NativeContinuationField::zeroed(&binding, &generate)?;
        let mut wrong = binding.clone();
        wrong.metadata_sha256 = "e".repeat(64);
        assert!(
            NativeContinuationField::from_bytes(&field.to_bytes()?, &wrong, &generate).is_err()
        );
        let mut artifact: serde_json::Value = serde_json::from_slice(&field.to_bytes()?)
            .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?;
        artifact["packed_unary"][0] = serde_json::json!(1);
        assert!(NativeContinuationField::from_bytes(
            &serde_json::to_vec(&artifact)
                .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?,
            &binding,
            &generate
        )
        .is_err());
        assert!(NativeContinuationField::compile(&binding, &generate, &[0x88; 120]).is_err());
        assert!(NativeContinuationField::compile(&binding, &generate, &[0; 119]).is_err());
        let source = SourceActionBinding::new(TOKENIZER)
            .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?;
        let different = NativeGeometricGenerate::compile(
            &source,
            2,
            &[1; 12],
            &[0; 3],
            EnergyTables::zeroed(2, vec![])?,
        )
        .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?;
        assert!(
            NativeContinuationField::from_bytes(&field.to_bytes()?, &binding, &different).is_err()
        );
        assert!(field
            .score_delta_into(
                &codes(1, 1)?,
                &different,
                &mut [0; 6],
                &mut ContinuationReadCounts::default()
            )
            .is_err());
        Ok(())
    }
    #[test]
    fn continuation_shared_action_policy_scale_roundtrip_and_legacy_bytes() -> Result<()> {
        let (binding, generate) = fixture()?;
        let legacy = NativeContinuationField::compile(&binding, &generate, &[0x77; 120])?;
        let legacy_bytes = legacy.to_bytes()?;
        let shared =
            NativeContinuationField::compile_shared_action(&binding, &generate, &[0x77; 120])?;
        assert!(!legacy.applies_to_copy());
        assert_eq!(legacy.score_shift(), 20);
        assert!(shared.applies_to_copy());
        assert_eq!(shared.score_shift(), 22);
        let restored =
            NativeContinuationField::from_bytes(&shared.to_bytes()?, &binding, &generate)?;
        assert_eq!(restored.to_bytes()?, shared.to_bytes()?);
        let mut old = [0; 6];
        let mut new = [0; 6];
        legacy.score_delta_into(
            &codes(119, 0)?,
            &generate,
            &mut old,
            &mut ContinuationReadCounts::default(),
        )?;
        restored.score_delta_into(
            &codes(119, 0)?,
            &generate,
            &mut new,
            &mut ContinuationReadCounts::default(),
        )?;
        assert_eq!(new, [14 << 22; 6]);
        assert_eq!(old, [14 << 20; 6]);
        assert_eq!(
            NativeContinuationField::from_bytes(&legacy_bytes, &binding, &generate)?.to_bytes()?,
            legacy_bytes
        );
        let mut corrupt: serde_json::Value = serde_json::from_slice(&shared.to_bytes()?)
            .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?;
        corrupt["metadata"]["score_shift"] = serde_json::json!(20);
        assert!(NativeContinuationField::from_bytes(
            &serde_json::to_vec(&corrupt)
                .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?,
            &binding,
            &generate
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn continuation_score_bound_and_negative_codes() -> Result<()> {
        let (binding, generate) = fixture()?;
        for (byte, expected) in [(0x77, 14i64), (0x99, -14i64)] {
            let field = NativeContinuationField::compile(&binding, &generate, &[byte; 120])?;
            let mut out = [0; 6];
            field.score_delta_into(
                &codes(119, 0)?,
                &generate,
                &mut out,
                &mut ContinuationReadCounts::default(),
            )?;
            assert_eq!(out, [expected << 20; 6]);
        }
        Ok(())
    }

    fn put_cross(packed: &mut [u8], lane: usize, factual: usize, local: usize, value: i8) {
        let index = (lane << 13) | (factual << 6) | (local >> 1);
        let shift = (local & 1) << 2;
        packed[index] = (packed[index] & !(15 << shift)) | (((value as u8) & 15) << shift);
    }

    #[test]
    fn continuation_cross_state_zero_roundtrip_and_legacy_contract() -> Result<()> {
        let (binding, generate) = fixture()?;
        let zero =
            NativeContinuationField::compile_cross_state(&binding, &generate, &vec![0; 2 << 13])?;
        assert!(zero.is_cross_state());
        assert!(zero.applies_to_copy());
        assert_eq!(zero.score_shift(), 22);
        assert!(zero.packed_unary().is_empty());
        assert_eq!(zero.packed_cross_state().map(<[u8]>::len), Some(2 << 13));
        let bytes = zero.to_bytes()?;
        assert!(bytes.len() > MAX_ARTIFACT_BYTES);
        assert_eq!(
            NativeContinuationField::from_bytes(&bytes, &binding, &generate)?.to_bytes()?,
            bytes
        );
        let mut output = [99; 6];
        let mut counts = ContinuationReadCounts::default();
        zero.score_delta_with_factual_into(
            &codes(31, 57)?,
            &codes(119, 0)?,
            &generate,
            &mut output,
            &mut counts,
        )?;
        assert_eq!(output, [0; 6]);
        assert_eq!(counts.algebra.inverse_reads, 4);
        assert_eq!(counts.algebra.product_reads, 24);
        assert_eq!(counts.prototype_reads, 12);
        assert_eq!(counts.coefficient_reads, 12);
        assert_eq!(counts.scores, 6);
        assert!(zero
            .score_delta_into(&codes(119, 0)?, &generate, &mut output, &mut counts,)
            .is_err());
        assert!(zero.coefficient_unary(0, 1).is_err());
        assert!(zero
            .score_delta_with_factual_into(
                &[],
                &codes(119, 0)?,
                &generate,
                &mut output,
                &mut counts,
            )
            .is_err());
        for shared in [false, true] {
            let field =
                NativeContinuationField::compile_policy(&binding, &generate, &[0x21; 120], shared)?;
            assert!(!field.is_cross_state());
            let old_bytes = field.to_bytes()?;
            assert!(!String::from_utf8_lossy(&old_bytes).contains("packed_cross_state"));
            let mut expected = [0; 6];
            field.score_delta_into(
                &codes(119, 0)?,
                &generate,
                &mut expected,
                &mut ContinuationReadCounts::default(),
            )?;
            field.score_delta_with_factual_into(
                &[],
                &codes(119, 0)?,
                &generate,
                &mut output,
                &mut ContinuationReadCounts::default(),
            )?;
            assert_eq!(expected, output);
            assert_eq!(
                NativeContinuationField::from_bytes(&old_bytes, &binding, &generate,)?
                    .to_bytes()?,
                old_bytes
            );
            let mut oversized = old_bytes;
            oversized.resize(MAX_ARTIFACT_BYTES + 1, b' ');
            assert!(NativeContinuationField::from_bytes(&oversized, &binding, &generate,).is_err());
        }
        Ok(())
    }

    #[test]
    fn continuation_cross_state_directed_full120_pair_indices_and_bounds() -> Result<()> {
        let (binding, generate) = fixture()?;
        let mut packed = vec![0; 2 << 13];
        for factual in 0..120 {
            for local in 0..120 {
                let value = ((factual + 2 * local + factual * local) % 15) as i8 - 7;
                put_cross(&mut packed, 0, factual, local, value);
            }
        }
        let field = NativeContinuationField::compile_cross_state(&binding, &generate, &packed)?;
        for factual in 0..120u8 {
            for local in [0u8, 1, 31, 57, 119] {
                let mut output = [0; 6];
                field.score_delta_with_factual_into(
                    &codes(factual, 1)?,
                    &codes(local, 1)?,
                    &generate,
                    &mut output,
                    &mut ContinuationReadCounts::default(),
                )?;
                for (token, score) in output.iter().enumerate() {
                    let prototype = generate.prototypes()[token * 2];
                    let left = usize::from(
                        generate
                            .algebra()
                            .compose(generate.algebra().inverse(factual)?, prototype)?,
                    );
                    let right = usize::from(
                        generate
                            .algebra()
                            .compose(generate.algebra().inverse(local)?, prototype)?,
                    );
                    let expected = ((left + 2 * right + left * right) % 15) as i64 - 7;
                    assert_eq!(*score, expected << 22);
                }
            }
        }
        assert!(field.coefficient_cross_state(2, 0, 0).is_err());
        assert!(field.coefficient_cross_state(0, 120, 0).is_err());
        assert!(field.coefficient_cross_state(0, 0, 120).is_err());
        for value in [-7i8, 7] {
            let mut extreme = vec![0; 2 << 13];
            for lane in 0..2 {
                for factual in 0..120 {
                    for local in 0..120 {
                        put_cross(&mut extreme, lane, factual, local, value);
                    }
                }
            }
            let field =
                NativeContinuationField::compile_cross_state(&binding, &generate, &extreme)?;
            let mut output = [0; 6];
            field.score_delta_with_factual_into(
                &codes(119, 0)?,
                &codes(31, 57)?,
                &generate,
                &mut output,
                &mut ContinuationReadCounts::default(),
            )?;
            assert_eq!(output, [(2 * i64::from(value)) << 22; 6]);
        }
        Ok(())
    }

    #[test]
    fn continuation_cross_state_nonseparable_two_carrier_interaction() -> Result<()> {
        let (binding, generate) = fixture()?;
        let mut packed = vec![0; 2 << 13];
        for (factual, local, value) in [(0, 0, 1), (0, 1, 2), (1, 0, -3), (1, 1, 4)] {
            put_cross(&mut packed, 0, factual, local, value);
        }
        let field = NativeContinuationField::compile_cross_state(&binding, &generate, &packed)?;
        let prototype = generate.prototypes()[0];
        let mut scores = [[0i64; 2]; 2];
        for (left, row) in scores.iter_mut().enumerate() {
            for (right, score) in row.iter_mut().enumerate() {
                // inv(state)*prototype=relative, hence state=prototype*inv(relative).
                let factual = generate
                    .algebra()
                    .compose(prototype, generate.algebra().inverse(left as u8)?)?;
                let local = generate
                    .algebra()
                    .compose(prototype, generate.algebra().inverse(right as u8)?)?;
                let mut output = [0; 6];
                field.score_delta_with_factual_into(
                    &codes(factual, 1)?,
                    &codes(local, 1)?,
                    &generate,
                    &mut output,
                    &mut ContinuationReadCounts::default(),
                )?;
                *score = output[0];
            }
        }
        assert_eq!(scores, [[1 << 22, 2 << 22], [-3 << 22, 4 << 22]]);
        // An additive factual-only plus local-only scorer has zero mixed difference.
        assert_ne!(scores[0][0] + scores[1][1] - scores[0][1] - scores[1][0], 0);
        Ok(())
    }

    #[test]
    fn continuation_cross_state_rejects_mixed_payload_padding_and_binding() -> Result<()> {
        let (binding, generate) = fixture()?;
        let valid = vec![0; 2 << 13];
        let field = NativeContinuationField::compile_cross_state(&binding, &generate, &valid)?;
        assert!(NativeContinuationField::compile_cross_state(
            &binding,
            &generate,
            &valid[..valid.len() - 1],
        )
        .is_err());
        for (factual, local, value) in [(120, 0, 1), (0, 120, 1), (0, 0, -8)] {
            let mut bad = valid.clone();
            put_cross(&mut bad, 0, factual, local, value);
            assert!(
                NativeContinuationField::compile_cross_state(&binding, &generate, &bad).is_err()
            );
        }
        let mut wrong = binding.clone();
        wrong.metadata_sha256 = "e".repeat(64);
        assert!(
            NativeContinuationField::from_bytes(&field.to_bytes()?, &wrong, &generate,).is_err()
        );
        let mut artifact = field.artifact.clone();
        artifact.packed_unary.push(0);
        assert!(NativeContinuationField::admit(artifact, &binding, &generate).is_err());
        let mut artifact = field.artifact.clone();
        artifact.packed_cross_state = None;
        assert!(NativeContinuationField::admit(artifact, &binding, &generate).is_err());
        let mut artifact = field.artifact.clone();
        artifact.metadata.policy = SHARED_POLICY.into();
        assert!(NativeContinuationField::admit(artifact, &binding, &generate).is_err());
        let mut artifact = field.artifact.clone();
        artifact.metadata.payload_sha256 = "f".repeat(64);
        assert!(NativeContinuationField::admit(artifact, &binding, &generate).is_err());
        let mut artifact = NativeContinuationField::zeroed(&binding, &generate)?.artifact;
        artifact.packed_cross_state = Some(valid);
        assert!(NativeContinuationField::admit(artifact, &binding, &generate).is_err());
        assert!(NativeContinuationField::from_bytes(
            &vec![b' '; MAX_CROSS_ARTIFACT_BYTES + 1],
            &binding,
            &generate,
        )
        .is_err());
        Ok(())
    }
}
