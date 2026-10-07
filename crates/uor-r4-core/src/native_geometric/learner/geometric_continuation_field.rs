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
        if bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(ContinuationFieldError::Shape(
                "continuation artifact byte limit",
            ));
        }
        let artifact = serde_json::from_slice(bytes)
            .map_err(|e| ContinuationFieldError::Artifact(e.to_string()))?;
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
        if artifact.packed_unary.len() != m.lanes * BYTES_PER_LANE {
            return Err(ContinuationFieldError::Shape(
                "continuation packed unary dimensions",
            ));
        }
        if m.payload_sha256 != hash(&artifact.packed_unary) {
            return Err(ContinuationFieldError::Binding(
                "continuation coefficient digest",
            ));
        }
        for (byte_index, &byte) in artifact.packed_unary.iter().enumerate() {
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
        self.metadata().schema == SHARED_SCHEMA
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
        if lane >= self.lanes() || relative >= 120 {
            return Err(ContinuationFieldError::Shape("continuation lane/relative"));
        }
        let index = self.lane_byte_offsets[lane] + usize::from(relative >> 1);
        Ok(decode(self.packed_unary()[index], relative & 1 != 0))
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
}
