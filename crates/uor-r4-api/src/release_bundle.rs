//! Versioned release-bundle manifest schema (#655-C0).
//!
//! Schema and structural validation ONLY. This module defines
//! [`ReleaseBundleManifest`] — the versioned record a packaged serving
//! bundle declares: schema version, public model id, instruction-chat
//! capability, ABI/contract version, pinned `uor-matmul` provenance
//! (#655-B), component digests, and tokenizer identity.
//!
//! It does **not** discover, load, or serve a bundle (that is #655-C1, the
//! shared startup loader), does not package bundle contents to a
//! directory or on-disk layout (#655-D), and does not change any default
//! engine selection (#655-E/F). No existing `src/server.rs`, `src/chat.rs`,
//! or CLI code reads or writes this manifest yet — landing the schema
//! first, consumed by a loader in a later focused PR, mirrors the issue's
//! own proposed implementation shape.
//!
//! [`ReleaseBundleManifest::from_compiled_model`] is the one bridge this
//! slice adds: a pure, in-memory constructor from `crate::compile`'s
//! already-computed [`CompiledModel`] output, so a future #655-D
//! packaging step (or this crate's own tests) has a checked way to turn a
//! real compile's digests/provenance into this schema instead of
//! hand-assembling one field by field. It performs no filesystem I/O and
//! discovers nothing.
//!
//! Field shapes mirror existing manifest conventions in this workspace
//! (`uor_r4_graph_compiler::observation::ObservationManifest`): a
//! `schema: u32` version, `#[serde(deny_unknown_fields)]` so an unknown
//! field is a hard parse error rather than silently ignored, and
//! `Option<T>` with `#[serde(default, skip_serializing_if =
//! "Option::is_none")]` for fields a future schema revision may add.

use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::compile::CompiledModel;
use crate::deployed_quality::{
    is_blake3_cid, CompilerIdentity, SelectorIdentity, NORMATIVE_SELECTOR_ID,
};
use crate::engine::AbiVersion;
use uor_r4_core::transformerless::hf_bpe::TokenizerAdapter;

/// Current schema version this crate writes and accepts. A field change
/// that is not additive-with-default bumps this and documents the
/// migration here, mirroring `ObservationManifest::schema`.
pub const RELEASE_BUNDLE_MANIFEST_SCHEMA: u32 = 2;
/// The sole legacy schema retained for explicit research/history reads. It
/// predates the normative selector and deployed-quality-report binding and is
/// therefore never production-valid.
pub const LEGACY_RELEASE_BUNDLE_MANIFEST_SCHEMA: u32 = 1;

/// Declared serving capability of the bundle's compiled model. Mirrors
/// (without reusing — this crate does not depend on the `r4` binary
/// crate) the two capability tiers `src/model.rs`'s `ModelCapability`
/// enforces at the request boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BundleCapability {
    Continuation,
    InstructionChat,
}

/// Pinned `uor-matmul` provenance the bundle's compile-time arithmetic ran
/// against (#655-B). Field values mirror
/// `docs/matrix_operation_census.md`'s pinned rev/codec description.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UorMatmulProvenance {
    /// Git revision of the pinned `uor-matmul`/`uor-matmul-core`
    /// dependency (40-hex-char commit; matches the `rev` pinned in the
    /// workspace `Cargo.toml`s).
    pub rev: String,
    /// Name of the operation/codec profile the bundle's compile-time
    /// arithmetic used, e.g. `"exact-gemm-float"` or
    /// `"certified-exact-fold"` (see `docs/matrix_operation_census.md`).
    pub operation_profile: String,
    /// SPDX license identifier of the pinned dependency source.
    pub license: String,
    /// Content digest of the pinned dependency source tree, when the
    /// producing pipeline computed one. Absent until a future loader/CI
    /// step wires a digest computation; a manifest without it is still
    /// schema-valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_digest: Option<String>,
}

/// blake3 digests of the bundle's runtime-required components. Field-for-
/// field mirror of `uor_r4_api::compile::ComponentDigests` (that type does
/// not derive `Serialize`/`Deserialize`, so this is an owned, persistable
/// value copy, not a re-export) so a future loader (#655-C1) converts
/// between them without a semantic gap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleComponentDigests {
    pub graph: String,
    /// Required in schema 2: independently emitted graph with SKMX/PSIB
    /// removed, used by the deployed-quality sections-absent comparison.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sections_absent_graph: Option<String>,
    /// Required in schema 2: independently emitted graph whose learned-lane
    /// labels are deterministically shuffled for the planted falsifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_shuffled_graph: Option<String>,
    pub signature_artifact: String,
    /// Required in schema 2: exact graded-store bytes used by the same-position
    /// plain-TLA comparator. Production admission re-hashes this component and
    /// reproduces the comparator definition CID carried by the quality report.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tla_comparator_store: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,
    pub score_report: String,
    pub compile_report: String,
    /// Required in schema 2. Optional in the Rust shape only so a schema-1
    /// historical manifest can still deserialize for explicit research use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployed_quality_report: Option<String>,
    /// Required in schema 2: canonical raw evidence that all serving surfaces
    /// replay the same normative selector decision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cross_surface_parity: Option<String>,
    /// Required in schema 2: canonical raw normative witness observations and
    /// their independent replay verdicts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub witness_replay: Option<String>,
}

/// The ABI/format surface the bundle declares. Owned, serializable copy of
/// `uor_r4_api::engine::AbiVersion` (`format_major`/`format_minor`) and
/// `uor_r4_graph_format::inference_contract::ContractVersion`
/// (`contract_*`) — both source types carry a non-serializable shape
/// (`AbiVersion::api_crate_version` is `&'static str`), so this manifest
/// stores the same values as owned fields instead of embedding them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleAbi {
    pub format_major: u8,
    pub format_minor: u8,
    pub contract_major: u16,
    pub contract_minor: u16,
    pub contract_patch: u16,
    pub api_crate_version: String,
}

/// Schema-2 admission identities supplied alongside a completed compile.
/// Grouping these fields keeps the constructor's policy boundary explicit:
/// none is derivable from [`CompiledModel`] alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseAdmissionIdentity {
    pub deployed_quality_report_cid: String,
    pub sections_absent_graph_cid: String,
    pub label_shuffled_graph_cid: String,
    pub tla_comparator_store_cid: String,
    pub cross_surface_parity_cid: String,
    pub witness_replay_cid: String,
    pub selector: SelectorIdentity,
    pub compiler: CompilerIdentity,
}

impl From<AbiVersion> for BundleAbi {
    fn from(abi: AbiVersion) -> Self {
        let (contract_major, contract_minor, contract_patch) = abi.contract.as_tuple();
        Self {
            format_major: abi.format_major,
            format_minor: abi.format_minor,
            contract_major,
            contract_minor,
            contract_patch,
            api_crate_version: abi.api_crate_version.to_string(),
        }
    }
}

/// Versioned manifest binding one packaged serving bundle's declared
/// identity: schema version, public model id, capability, ABI/contract
/// version, pinned `uor-matmul` provenance, component digests, and
/// tokenizer identity.
///
/// Schema and structural validation only (#655-C0) — see the module docs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseBundleManifest {
    pub schema: u32,
    /// The public model identity this bundle serves (e.g. `"r4"`), plus
    /// any versioned/legacy alias the bundle declares. Binding this to an
    /// actual default engine selection is #655-E/F, not this schema.
    pub model_id: String,
    pub capability: BundleCapability,
    pub abi: BundleAbi,
    pub uor_matmul: UorMatmulProvenance,
    pub components: BundleComponentDigests,
    /// Required in schema 2 and fixed to `R4G1Runtime` for production.
    /// Optional only for schema-1 historical deserialization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<SelectorIdentity>,
    /// Required in schema 2. The source revision is an external release
    /// identity; `configuration_cid` is independently reproduced from the
    /// captured graph HEAD plus score/cover configuration bytes at load.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiler: Option<CompilerIdentity>,
    pub tokenizer_adapter: TokenizerAdapter,
    /// Free-text pointer to a fuller provenance record (e.g. a corpus
    /// manifest κ or an issue/PR reference), when the producing pipeline
    /// has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance_note: Option<String>,
}

fn is_git_rev(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|c| c.is_ascii_hexdigit())
}

impl ReleaseBundleManifest {
    /// Build a release-bundle manifest for one completed compile's output
    /// components (the #655-C0 → #655-D bridge). Pure, in-memory
    /// construction: copies `compiled`'s already-computed digests and ABI
    /// provenance field-for-field into this schema. Touches no
    /// filesystem, discovers no bundle directory, and validates nothing
    /// against bytes on disk — packaging bundle contents to a directory
    /// is #655-D; loading one back is #655-C1.
    ///
    /// `model_id`, `capability`, `uor_matmul`, and `provenance_note` are
    /// caller-supplied because [`CompiledModel`] carries none of them: a
    /// public model identity and a serving-capability policy are
    /// application decisions this schema crate does not make on a
    /// caller's behalf, and `uor-matmul` provenance is not yet computed
    /// anywhere in the compile pipeline (see [`UorMatmulProvenance`]'s
    /// `source_digest` field doc) — the caller supplies whatever it
    /// already has pinned (e.g. from `docs/matrix_operation_census.md`)
    /// until a future slice wires an automatic digest.
    ///
    /// The result is a field copy, not a validating constructor: an
    /// empty `model_id` or a malformed `uor_matmul.rev` the caller passed
    /// in still round-trips into the output unchanged. Callers that need
    /// a checked manifest call [`Self::validate`] on the result.
    #[cfg(feature = "full")]
    pub fn from_compiled_model(
        model_id: impl Into<String>,
        capability: BundleCapability,
        compiled: &CompiledModel,
        admission: ReleaseAdmissionIdentity,
        uor_matmul: UorMatmulProvenance,
        provenance_note: Option<String>,
    ) -> Self {
        let provenance = &compiled.provenance;
        let (contract_major, contract_minor, contract_patch) =
            provenance.contract_version.as_tuple();
        Self {
            schema: RELEASE_BUNDLE_MANIFEST_SCHEMA,
            model_id: model_id.into(),
            capability,
            abi: BundleAbi {
                format_major: provenance.format_version.0,
                format_minor: provenance.format_version.1,
                contract_major,
                contract_minor,
                contract_patch,
                api_crate_version: env!("CARGO_PKG_VERSION").to_string(),
            },
            uor_matmul,
            components: BundleComponentDigests {
                graph: provenance.digests.graph.clone(),
                sections_absent_graph: Some(admission.sections_absent_graph_cid),
                label_shuffled_graph: Some(admission.label_shuffled_graph_cid),
                signature_artifact: provenance.digests.signature_artifact.clone(),
                tla_comparator_store: Some(admission.tla_comparator_store_cid),
                tokenizer: provenance.digests.tokenizer.clone(),
                score_report: provenance.digests.score_report.clone(),
                compile_report: provenance.digests.compile_report.clone(),
                deployed_quality_report: Some(admission.deployed_quality_report_cid),
                cross_surface_parity: Some(admission.cross_surface_parity_cid),
                witness_replay: Some(admission.witness_replay_cid),
            },
            selector: Some(admission.selector),
            compiler: Some(admission.compiler),
            tokenizer_adapter: provenance.tokenizer_adapter.clone(),
            provenance_note,
        }
    }

    /// Production validation: schema 2, non-empty identity fields,
    /// well-formed component digests, a plausible pinned `uor-matmul`
    /// revision, the deployed-quality-report CID, and the normative selector.
    /// Schema 1 is deliberately rejected here even though serde can read it.
    ///
    /// Returns `Option<String>` rather than `Result<(), E>` — mirroring
    /// `engine::validate_quality_report` in this same crate — so this
    /// shipped function names no custom error type (R5: a shipped crate's
    /// `Result` may only name the sanctioned graph-substrate/host-source
    /// error surface, which a bundle-manifest schema check is not).
    ///
    /// Structural validity does not certify that the referenced bytes
    /// exist, parse, or match these digests — that is the loader's job
    /// (#655-C1).
    pub fn validate(&self) -> Option<String> {
        if self.schema != RELEASE_BUNDLE_MANIFEST_SCHEMA {
            if self.schema == LEGACY_RELEASE_BUNDLE_MANIFEST_SCHEMA {
                return Some(
                    "release-bundle manifest schema 1 is legacy research evidence: it has no deployed-quality-report/selector binding and cannot authorize production"
                        .to_string(),
                );
            }
            return Some(format!(
                "unsupported release-bundle manifest schema {} (this build reads schema {RELEASE_BUNDLE_MANIFEST_SCHEMA})",
                self.schema
            ));
        }
        self.validate_common(true)
    }

    /// Structural validation for explicit historical/research reads. Schema 1
    /// remains readable here, but success is an availability/shape statement,
    /// never production admission. Schema 2 receives its full binding checks.
    pub fn validate_for_research(&self) -> Option<String> {
        match self.schema {
            LEGACY_RELEASE_BUNDLE_MANIFEST_SCHEMA => self.validate_common(false),
            RELEASE_BUNDLE_MANIFEST_SCHEMA => self.validate_common(true),
            other => Some(format!(
                "unsupported release-bundle manifest schema {other} (research reads schema {LEGACY_RELEASE_BUNDLE_MANIFEST_SCHEMA} or {RELEASE_BUNDLE_MANIFEST_SCHEMA})"
            )),
        }
    }

    fn validate_common(&self, require_deployed_quality: bool) -> Option<String> {
        if self.model_id.trim().is_empty() {
            return Some("model_id is empty".to_string());
        }
        if !is_git_rev(&self.uor_matmul.rev) {
            return Some(format!(
                "uor_matmul.rev {:?} is not a 40-character hex git revision",
                self.uor_matmul.rev
            ));
        }
        if self.uor_matmul.operation_profile.trim().is_empty() {
            return Some("uor_matmul.operation_profile is empty".to_string());
        }
        if self.uor_matmul.license.trim().is_empty() {
            return Some("uor_matmul.license is empty".to_string());
        }
        if let Some(reason) = self.validate_component_digests(require_deployed_quality) {
            return Some(reason);
        }
        if require_deployed_quality {
            let selector = self
                .selector
                .as_ref()
                .ok_or_else(|| "schema 2 requires selector identity R4G1Runtime".to_string());
            let selector = match selector {
                Ok(selector) => selector,
                Err(reason) => return Some(reason),
            };
            if let Some(error) = selector.validate_normative() {
                return Some(error.to_string());
            }
            if selector.id != NORMATIVE_SELECTOR_ID {
                return Some(format!(
                    "selector identity {:?} is not {:?}",
                    selector.id, NORMATIVE_SELECTOR_ID
                ));
            }
            let compiler = match self.compiler.as_ref() {
                Some(compiler) => compiler,
                None => return Some("schema 2 requires compiler identity".to_string()),
            };
            if compiler.revision.len() != 40
                || !compiler
                    .revision
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
            {
                return Some(format!(
                    "compiler.revision {:?} is not a 40-character git revision",
                    compiler.revision
                ));
            }
            if !is_blake3_cid(&compiler.configuration_cid) {
                return Some(format!(
                    "compiler.configuration_cid {:?} is not a blake3:<hex> digest",
                    compiler.configuration_cid
                ));
            }
        }
        None
    }

    fn validate_component_digests(&self, require_deployed_quality: bool) -> Option<String> {
        let required = [
            ("components.graph", self.components.graph.as_str()),
            (
                "components.signature_artifact",
                self.components.signature_artifact.as_str(),
            ),
            (
                "components.score_report",
                self.components.score_report.as_str(),
            ),
            (
                "components.compile_report",
                self.components.compile_report.as_str(),
            ),
        ];
        for (name, digest) in required {
            if !is_blake3_cid(digest) {
                return Some(format!("{name} {digest:?} is not a blake3:<hex> digest"));
            }
        }
        if let Some(digest) = self.components.tokenizer.as_deref() {
            if !is_blake3_cid(digest) {
                return Some(format!(
                    "components.tokenizer {digest:?} is not a blake3:<hex> digest"
                ));
            }
        }
        match self.components.deployed_quality_report.as_deref() {
            Some(digest) if !is_blake3_cid(digest) => {
                return Some(format!(
                    "components.deployed_quality_report {digest:?} is not a blake3:<hex> digest"
                ));
            }
            None if require_deployed_quality => {
                return Some("schema 2 requires components.deployed_quality_report".to_string());
            }
            Some(_) | None => {}
        }
        match self.components.tla_comparator_store.as_deref() {
            Some(digest) if !is_blake3_cid(digest) => {
                return Some(format!(
                    "components.tla_comparator_store {digest:?} is not a blake3:<hex> digest"
                ));
            }
            None if require_deployed_quality => {
                return Some("schema 2 requires components.tla_comparator_store".to_string());
            }
            Some(_) | None => {}
        }
        let schema_two_evidence = [
            (
                "components.sections_absent_graph",
                self.components.sections_absent_graph.as_deref(),
            ),
            (
                "components.label_shuffled_graph",
                self.components.label_shuffled_graph.as_deref(),
            ),
            (
                "components.cross_surface_parity",
                self.components.cross_surface_parity.as_deref(),
            ),
            (
                "components.witness_replay",
                self.components.witness_replay.as_deref(),
            ),
        ];
        for (name, digest) in schema_two_evidence {
            match digest {
                Some(digest) if !is_blake3_cid(digest) => {
                    return Some(format!("{name} {digest:?} is not a blake3:<hex> digest"));
                }
                None if require_deployed_quality => {
                    return Some(format!("schema 2 requires {name}"));
                }
                Some(_) | None => {}
            }
        }
        if self.capability == BundleCapability::InstructionChat
            && self.tokenizer_adapter.family.trim().is_empty()
        {
            return Some(
                "capability is instruction-chat but tokenizer_adapter.family is empty".to_string(),
            );
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_REV: &str = "b13c98449948174f590e337c4dc25dfc394a07d0";
    const VALID_DIGEST: &str =
        "blake3:0000000000000000000000000000000000000000000000000000000000000001";

    fn valid_selector() -> SelectorIdentity {
        SelectorIdentity {
            id: NORMATIVE_SELECTOR_ID.to_string(),
            semantics_version: "1.0.0".to_string(),
            semantics_cid: VALID_DIGEST.to_string(),
        }
    }

    fn valid_manifest() -> ReleaseBundleManifest {
        ReleaseBundleManifest {
            schema: RELEASE_BUNDLE_MANIFEST_SCHEMA,
            model_id: "r4".to_string(),
            capability: BundleCapability::InstructionChat,
            abi: BundleAbi {
                format_major: 1,
                format_minor: 0,
                contract_major: 1,
                contract_minor: 0,
                contract_patch: 0,
                api_crate_version: "0.1.0".to_string(),
            },
            uor_matmul: UorMatmulProvenance {
                rev: VALID_REV.to_string(),
                operation_profile: "exact-gemm-float".to_string(),
                license: "MIT".to_string(),
                source_digest: None,
            },
            components: BundleComponentDigests {
                graph: VALID_DIGEST.to_string(),
                sections_absent_graph: Some(VALID_DIGEST.to_string()),
                label_shuffled_graph: Some(VALID_DIGEST.to_string()),
                signature_artifact: VALID_DIGEST.to_string(),
                tla_comparator_store: Some(VALID_DIGEST.to_string()),
                tokenizer: Some(VALID_DIGEST.to_string()),
                score_report: VALID_DIGEST.to_string(),
                compile_report: VALID_DIGEST.to_string(),
                deployed_quality_report: Some(VALID_DIGEST.to_string()),
                cross_surface_parity: Some(VALID_DIGEST.to_string()),
                witness_replay: Some(VALID_DIGEST.to_string()),
            },
            selector: Some(valid_selector()),
            compiler: Some(CompilerIdentity {
                revision: VALID_REV.to_string(),
                configuration_cid: VALID_DIGEST.to_string(),
            }),
            tokenizer_adapter: TokenizerAdapter {
                family: "hf-byte-bpe".to_string(),
                ..Default::default()
            },
            provenance_note: None,
        }
    }

    /// A minimal but fully-provenanced [`CompiledModel`] fixture, mirroring
    /// `crate::compile`'s own private `compiled_model_with_report` test
    /// helper (that one is not `pub(crate)`, so this module builds its own
    /// rather than reaching into a sibling module's test internals).
    #[cfg(feature = "full")]
    fn compiled_model_fixture() -> CompiledModel {
        use crate::compile::{CompileOptions, CompileProvenance, ComponentDigests};
        use uor_r4_graph_format::{
            FORMAT_VERSION_MAJOR, FORMAT_VERSION_MINOR, INFERENCE_OPERATION_CONTRACT_VERSION,
        };
        CompiledModel {
            graph: b"graph bytes".to_vec(),
            signature_artifact: b"signature bytes".to_vec(),
            tokenizer: Some(b"tokenizer bytes".to_vec()),
            score_report: b"{}".to_vec(),
            compile_report: b"{}".to_vec(),
            provenance: CompileProvenance {
                options: CompileOptions::default(),
                tokenizer_adapter: TokenizerAdapter {
                    family: "hf-byte-bpe".to_string(),
                    ..Default::default()
                },
                format_version: (FORMAT_VERSION_MAJOR, FORMAT_VERSION_MINOR),
                contract_version: INFERENCE_OPERATION_CONTRACT_VERSION,
                digests: ComponentDigests {
                    graph: VALID_DIGEST.to_string(),
                    signature_artifact: VALID_DIGEST.to_string(),
                    tokenizer: Some(VALID_DIGEST.to_string()),
                    score_report: VALID_DIGEST.to_string(),
                    compile_report: VALID_DIGEST.to_string(),
                },
            },
        }
    }

    #[cfg(feature = "full")]
    fn valid_uor_matmul_provenance() -> UorMatmulProvenance {
        UorMatmulProvenance {
            rev: VALID_REV.to_string(),
            operation_profile: "exact-gemm-float".to_string(),
            license: "MIT".to_string(),
            source_digest: None,
        }
    }

    #[cfg(feature = "full")]
    #[test]
    fn from_compiled_model_copies_digests_and_abi_and_passes_validation() {
        let compiled = compiled_model_fixture();
        let manifest = ReleaseBundleManifest::from_compiled_model(
            "r4",
            BundleCapability::InstructionChat,
            &compiled,
            ReleaseAdmissionIdentity {
                deployed_quality_report_cid: VALID_DIGEST.to_string(),
                sections_absent_graph_cid: VALID_DIGEST.to_string(),
                label_shuffled_graph_cid: VALID_DIGEST.to_string(),
                tla_comparator_store_cid: VALID_DIGEST.to_string(),
                cross_surface_parity_cid: VALID_DIGEST.to_string(),
                witness_replay_cid: VALID_DIGEST.to_string(),
                selector: valid_selector(),
                compiler: CompilerIdentity {
                    revision: VALID_REV.to_string(),
                    configuration_cid: VALID_DIGEST.to_string(),
                },
            },
            valid_uor_matmul_provenance(),
            Some("built from a fixture compile".to_string()),
        );
        assert_eq!(manifest.model_id, "r4");
        assert_eq!(manifest.components.graph, compiled.provenance.digests.graph);
        assert_eq!(
            manifest.components.signature_artifact,
            compiled.provenance.digests.signature_artifact
        );
        assert_eq!(
            manifest.components.tokenizer,
            compiled.provenance.digests.tokenizer
        );
        assert_eq!(
            manifest.components.score_report,
            compiled.provenance.digests.score_report
        );
        assert_eq!(
            manifest.components.compile_report,
            compiled.provenance.digests.compile_report
        );
        assert_eq!(
            manifest.components.deployed_quality_report.as_deref(),
            Some(VALID_DIGEST)
        );
        assert_eq!(
            manifest.components.sections_absent_graph.as_deref(),
            Some(VALID_DIGEST)
        );
        assert_eq!(
            manifest.components.label_shuffled_graph.as_deref(),
            Some(VALID_DIGEST)
        );
        assert_eq!(
            manifest.components.tla_comparator_store.as_deref(),
            Some(VALID_DIGEST)
        );
        assert_eq!(
            manifest.components.cross_surface_parity.as_deref(),
            Some(VALID_DIGEST)
        );
        assert_eq!(
            manifest.components.witness_replay.as_deref(),
            Some(VALID_DIGEST)
        );
        assert_eq!(manifest.selector, Some(valid_selector()));
        assert_eq!(
            manifest.tokenizer_adapter,
            compiled.provenance.tokenizer_adapter
        );
        let (major, minor, patch) = compiled.provenance.contract_version.as_tuple();
        assert_eq!(
            manifest.abi.format_major,
            compiled.provenance.format_version.0
        );
        assert_eq!(
            manifest.abi.format_minor,
            compiled.provenance.format_version.1
        );
        assert_eq!(manifest.abi.contract_major, major);
        assert_eq!(manifest.abi.contract_minor, minor);
        assert_eq!(manifest.abi.contract_patch, patch);
        assert_eq!(manifest.abi.api_crate_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(
            manifest.provenance_note.as_deref(),
            Some("built from a fixture compile")
        );
        assert_eq!(manifest.validate(), None, "fixture builds a valid manifest");
    }

    #[cfg(feature = "full")]
    #[test]
    fn from_compiled_model_does_not_validate_a_caller_supplied_bad_field() {
        // The constructor is a field copy, not a validating constructor:
        // a malformed caller-supplied `uor_matmul.rev` still round-trips
        // into the output unchanged, and `.validate()` (called
        // separately) is what catches it.
        let compiled = compiled_model_fixture();
        let mut bad_provenance = valid_uor_matmul_provenance();
        bad_provenance.rev = "not-a-rev".to_string();
        let manifest = ReleaseBundleManifest::from_compiled_model(
            "r4",
            BundleCapability::InstructionChat,
            &compiled,
            ReleaseAdmissionIdentity {
                deployed_quality_report_cid: VALID_DIGEST.to_string(),
                sections_absent_graph_cid: VALID_DIGEST.to_string(),
                label_shuffled_graph_cid: VALID_DIGEST.to_string(),
                tla_comparator_store_cid: VALID_DIGEST.to_string(),
                cross_surface_parity_cid: VALID_DIGEST.to_string(),
                witness_replay_cid: VALID_DIGEST.to_string(),
                selector: valid_selector(),
                compiler: CompilerIdentity {
                    revision: VALID_REV.to_string(),
                    configuration_cid: VALID_DIGEST.to_string(),
                },
            },
            bad_provenance,
            None,
        );
        assert_eq!(manifest.uor_matmul.rev, "not-a-rev");
        let reason = manifest
            .validate()
            .expect("the malformed rev is still caught by validate()");
        assert!(reason.contains("uor_matmul.rev"), "reason was: {reason}");
    }

    #[test]
    fn valid_manifest_passes_validation() {
        assert_eq!(valid_manifest().validate(), None);
    }

    #[test]
    fn round_trips_through_json() {
        let manifest = valid_manifest();
        let bytes = serde_json::to_vec(&manifest).expect("serialize valid manifest");
        let parsed: ReleaseBundleManifest =
            serde_json::from_slice(&bytes).expect("deserialize round trip");
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn unknown_field_is_a_hard_parse_error() {
        let mut value = serde_json::to_value(valid_manifest()).expect("to_value");
        value
            .as_object_mut()
            .expect("object")
            .insert("unexpected_field".to_string(), serde_json::json!(true));
        let result: Result<ReleaseBundleManifest, _> = serde_json::from_value(value);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_unsupported_schema() {
        let mut manifest = valid_manifest();
        manifest.schema = RELEASE_BUNDLE_MANIFEST_SCHEMA + 1;
        let reason = manifest.validate().expect("schema mismatch rejected");
        assert!(reason.contains("schema"), "reason was: {reason}");
    }

    #[test]
    fn rejects_empty_model_id() {
        let mut manifest = valid_manifest();
        manifest.model_id = String::new();
        assert!(manifest.validate().is_some());
    }

    #[test]
    fn rejects_malformed_uor_matmul_rev() {
        let mut manifest = valid_manifest();
        manifest.uor_matmul.rev = "not-a-rev".to_string();
        let reason = manifest.validate().expect("malformed rev rejected");
        assert!(reason.contains("uor_matmul.rev"), "reason was: {reason}");
    }

    #[test]
    fn rejects_malformed_component_digest() {
        let mut manifest = valid_manifest();
        manifest.components.graph = "not-a-digest".to_string();
        let reason = manifest.validate().expect("malformed digest rejected");
        assert!(reason.contains("components.graph"), "reason was: {reason}");
    }

    #[test]
    fn rejects_malformed_optional_tokenizer_digest() {
        let mut manifest = valid_manifest();
        manifest.components.tokenizer = Some("not-a-digest".to_string());
        let reason = manifest
            .validate()
            .expect("malformed tokenizer digest rejected");
        assert!(
            reason.contains("components.tokenizer"),
            "reason was: {reason}"
        );
    }

    #[test]
    fn schema_two_requires_deployed_quality_report_selector_and_compiler() {
        let mut manifest = valid_manifest();
        manifest.components.deployed_quality_report = None;
        let reason = manifest
            .validate()
            .expect("missing deployed-quality report rejected");
        assert!(
            reason.contains("deployed_quality_report"),
            "reason was: {reason}"
        );

        let mut manifest = valid_manifest();
        manifest.components.tla_comparator_store = None;
        let reason = manifest
            .validate()
            .expect("missing TLA comparator store rejected");
        assert!(
            reason.contains("tla_comparator_store"),
            "reason was: {reason}"
        );

        let mut manifest = valid_manifest();
        manifest.components.sections_absent_graph = None;
        let reason = manifest
            .validate()
            .expect("missing sections-absent graph rejected");
        assert!(
            reason.contains("sections_absent_graph"),
            "reason was: {reason}"
        );

        let mut manifest = valid_manifest();
        manifest.components.label_shuffled_graph = None;
        let reason = manifest
            .validate()
            .expect("missing label-shuffled graph rejected");
        assert!(
            reason.contains("label_shuffled_graph"),
            "reason was: {reason}"
        );

        let mut manifest = valid_manifest();
        manifest.components.cross_surface_parity = None;
        let reason = manifest
            .validate()
            .expect("missing cross-surface evidence rejected");
        assert!(
            reason.contains("cross_surface_parity"),
            "reason was: {reason}"
        );

        let mut manifest = valid_manifest();
        manifest.components.witness_replay = None;
        let reason = manifest
            .validate()
            .expect("missing witness replay rejected");
        assert!(reason.contains("witness_replay"), "reason was: {reason}");

        let mut manifest = valid_manifest();
        manifest.selector = None;
        let reason = manifest.validate().expect("missing selector rejected");
        assert!(reason.contains("selector"), "reason was: {reason}");

        let mut manifest = valid_manifest();
        manifest.compiler = None;
        let reason = manifest.validate().expect("missing compiler rejected");
        assert!(reason.contains("compiler"), "reason was: {reason}");
    }

    #[test]
    fn schema_two_rejects_non_normative_selector() {
        let mut manifest = valid_manifest();
        manifest.selector.as_mut().expect("selector").id = "GraphScorer".to_string();
        let reason = manifest.validate().expect("off-serving selector rejected");
        assert!(
            reason.contains(NORMATIVE_SELECTOR_ID),
            "reason was: {reason}"
        );
    }

    #[test]
    fn schema_one_deserializes_for_research_but_never_production_validates() {
        let mut value = serde_json::to_value(valid_manifest()).expect("to value");
        let object = value.as_object_mut().expect("manifest object");
        object.insert(
            "schema".to_string(),
            serde_json::json!(LEGACY_RELEASE_BUNDLE_MANIFEST_SCHEMA),
        );
        object.remove("selector");
        object.remove("compiler");
        object
            .get_mut("components")
            .and_then(serde_json::Value::as_object_mut)
            .expect("components object")
            .retain(|field, _| {
                !matches!(
                    field.as_str(),
                    "deployed_quality_report"
                        | "sections_absent_graph"
                        | "label_shuffled_graph"
                        | "cross_surface_parity"
                        | "witness_replay"
                )
            });
        let parsed: ReleaseBundleManifest =
            serde_json::from_value(value).expect("schema one remains readable");
        assert_eq!(parsed.validate_for_research(), None);
        let reason = parsed
            .validate()
            .expect("schema one never authorizes production");
        assert!(reason.contains("legacy research"), "reason was: {reason}");
    }

    #[test]
    fn instruction_chat_requires_tokenizer_family() {
        let mut manifest = valid_manifest();
        manifest.tokenizer_adapter.family = String::new();
        let reason = manifest
            .validate()
            .expect("missing tokenizer family rejected");
        assert!(
            reason.contains("tokenizer_adapter.family"),
            "reason was: {reason}"
        );
    }

    #[test]
    fn continuation_capability_does_not_require_tokenizer_family() {
        let mut manifest = valid_manifest();
        manifest.capability = BundleCapability::Continuation;
        manifest.tokenizer_adapter.family = String::new();
        assert_eq!(manifest.validate(), None);
    }

    #[test]
    fn abi_conversion_preserves_current_build_version() {
        let bundle_abi = BundleAbi::from(AbiVersion::current());
        assert_eq!(bundle_abi.api_crate_version, env!("CARGO_PKG_VERSION"));
    }
}
