//! Exact Hologram v4 application composition and strict Holo/1 validation.

use crate::error::PrismError;
use crate::holo::canonical::{content_id, encode_value};
use hologram::archive::{HoloLoader, HoloWriter, SectionKind};
use hologram::space::{
    address_bytes, AppManifest, Capabilities, CapabilitySet, Layer, LayerKind, Realization,
    WASM_CONTRACT_CORE_V1,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Hologram's required application-directory extension.
pub const DIRECTORY_EXTENSION: &str =
    "https://hologram.foundation/extension/application-directory/v1";
/// Prism's producer-provenance extension.
pub const PRISM_EXTENSION: &str = "https://uor.foundation/extension/prismpm-model/v1";

/// All acyclic, pre-archive inputs required to compose one Holo/1 application.
#[derive(Debug, Clone)]
pub struct ApplicationArchiveInput {
    /// Human product name, used only by generated metadata.
    pub application_name: String,
    /// Core-Wasm v1 guest bytes.
    pub guest_wasm: Vec<u8>,
    /// Canonical HOLOVIEW v1 payload.
    pub view_bundle: Vec<u8>,
    /// Canonical non-Holo Prism model document.
    pub model_document: Vec<u8>,
    /// Canonical generated source-manifest bytes, also the Metadata section.
    pub source_manifest: Vec<u8>,
    /// Closed provenance fields excluding kappas computed by this function.
    pub provenance: ArchiveProvenance,
}

/// Closed pre-archive provenance values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveProvenance {
    /// LexLean source identity.
    pub source_id: String,
    /// LexLean semantic identity.
    pub semantic_id: String,
    /// LexLean compiler-semantics identity.
    pub compiler_semantics_id: String,
    /// LexLean snapshot identity.
    pub snapshot_id: String,
    /// Generated stdlib semantic identity.
    pub stdlib_semantics_id: String,
    /// SHA-256 of the packaged generated stdlib crate.
    pub prism_stdlib_crate_sha256: String,
    /// Exact LexLean source revision.
    pub lexlean_commit: String,
    /// SHA-256 of the LexLean crate.
    pub lexlean_package_sha256: String,
    /// Exact lean4-prod source revision.
    pub lean4_prod_commit: String,
    /// Pinned Hologram Live revision.
    pub hologram_live_commit: String,
    /// Pinned uor-hologram revision.
    pub uor_hologram_commit: String,
    /// Core-Wasm target profile identity.
    pub target_profile_id: String,
    /// SHA-256 of generated Lean evidence.
    pub lean_manifest_sha256: String,
    /// SHA-256 of LCNF evidence.
    pub lcnf_manifest_sha256: String,
    /// SHA-256 of the generated core source/package closure.
    pub generated_core_sha256: String,
    /// Generated Cargo package name.
    pub cargo_name: String,
    /// Generated Cargo package version.
    pub cargo_version: String,
    /// SHA-256 of the generated `.crate` bytes.
    pub cargo_crate_sha256: String,
    /// Typed View-model identity.
    pub view_model_id: String,
    /// SHA-256 of the generated browser projection.
    pub browser_projection_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "tag", rename_all = "lowercase", deny_unknown_fields)]
enum BrowserProjection {
    None,
    Present { sha256: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "tag", rename_all = "lowercase", deny_unknown_fields)]
enum ViewBinding {
    None,
    Present {
        view_model_id: String,
        view_content_kappa: String,
        browser_projection: BrowserProjection,
    },
}

/// The exact, closed `prismpm/model-provenance/1` wire value.  It deliberately
/// contains only pre-archive evidence: footer, archive, and attestation
/// identities are computed after these bytes have been embedded.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ModelProvenanceV1 {
    schema: String,
    model_content_kappa: String,
    model_id: String,
    source_id: String,
    semantic_id: String,
    compiler_semantics_id: String,
    snapshot_id: String,
    stdlib_semantics_id: String,
    prism_stdlib_crate_sha256: String,
    lexlean_commit: String,
    lexlean_package_sha256: String,
    lean4_prod_commit: String,
    hologram_live_commit: String,
    uor_hologram_commit: String,
    target_profile_id: String,
    core_wasm_contract: String,
    lean_manifest_sha256: String,
    lcnf_manifest_sha256: String,
    generated_core_sha256: String,
    cargo_name: String,
    cargo_version: String,
    cargo_crate_sha256: String,
    guest_content_kappa: String,
    view_binding: ViewBinding,
    application_kappa: String,
}

/// Distinct identities returned with a composed archive.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HoloIdentities {
    /// Kappa of the guest layer.
    pub guest_content_kappa: String,
    /// Kappa of the View layer.
    pub view_content_kappa: String,
    /// Kappa of the Prism model blob.
    pub model_content_kappa: String,
    /// Kappa of the canonical application manifest.
    pub application_kappa: String,
    /// Hexadecimal archive footer fingerprint.
    pub archive_fingerprint: String,
    /// Kappa of the complete archive object.
    pub archive_kappa: String,
}

/// Fully composed archive and its extension/evidence payloads.
#[derive(Debug, Clone)]
pub struct GeneratedHolo {
    /// Binary Hologram v4 archive.
    pub bytes: Vec<u8>,
    /// Canonical AppManifest bytes.
    pub application_manifest: Vec<u8>,
    /// Canonical empty CapabilitySet bytes.
    pub capability_request: Vec<u8>,
    /// Canonical application-directory JSON bytes.
    pub directory: Vec<u8>,
    /// Canonical Prism provenance-extension bytes.
    pub prism_extension: Vec<u8>,
    /// All non-interchangeable identities.
    pub identities: HoloIdentities,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Directory {
    schema_version: u16,
    primary_layer: Option<u32>,
    requires_kappa: String,
    layers: Vec<DirectoryLayer>,
    children: Vec<DirectoryChild>,
    blobs: Vec<DirectoryBlob>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DirectoryLayer {
    position: u32,
    kind: String,
    content_kappa: String,
    entry: String,
    contract: Option<String>,
    architecture: Option<String>,
    surface: Option<String>,
    engine: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DirectoryChild {
    position: u32,
    application_kappa: String,
    capabilities_kappa: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DirectoryBlob {
    kappa: String,
    byte_length: u64,
}

fn empty_capabilities() -> Capabilities {
    Capabilities {
        storage_roots: Vec::new(),
        storage_quota_bytes: 0,
        network_fetch_endpoints: Vec::new(),
        network_announce_endpoints: Vec::new(),
        publish_channels: Vec::new(),
        subscribe_channels: Vec::new(),
        memory_max_bytes: 0,
        cpu_time_per_event_ms: 0,
        priority_weight: 0,
    }
}

fn extension(key: &str, value: &[u8]) -> Result<Vec<u8>, PrismError> {
    let length = u16::try_from(key.len())
        .map_err(|_| PrismError::new("PP3013", "extension key is too long"))?;
    let mut bytes = Vec::with_capacity(2 + key.len() + value.len());
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(key.as_bytes());
    bytes.extend_from_slice(value);
    Ok(bytes)
}

fn digest_is_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validate_provenance(value: &ArchiveProvenance) -> Result<(), PrismError> {
    let digests = [
        &value.source_id,
        &value.semantic_id,
        &value.compiler_semantics_id,
        &value.snapshot_id,
        &value.stdlib_semantics_id,
        &value.prism_stdlib_crate_sha256,
        &value.lexlean_package_sha256,
        &value.target_profile_id,
        &value.lean_manifest_sha256,
        &value.lcnf_manifest_sha256,
        &value.generated_core_sha256,
        &value.cargo_crate_sha256,
        &value.view_model_id,
        &value.browser_projection_sha256,
    ];
    if digests.into_iter().any(|digest| !digest_is_valid(digest))
        || value.lexlean_commit.len() != 40
        || value.lean4_prod_commit.len() != 40
        || value.hologram_live_commit.len() != 40
        || value.uor_hologram_commit.len() != 40
        || value.cargo_name.is_empty()
        || value.cargo_version.is_empty()
    {
        return Err(PrismError::new(
            "PP3015",
            "model provenance contains an invalid identity",
        ));
    }
    Ok(())
}

/// Compose one deterministic, self-contained Wasm + portable-View application.
pub fn compose_application(input: &ApplicationArchiveInput) -> Result<GeneratedHolo, PrismError> {
    validate_provenance(&input.provenance)?;
    if input.application_name.is_empty()
        || !input.guest_wasm.starts_with(b"\0asm")
        || !input.view_bundle.starts_with(b"HOLOVIEW\0\x01")
        || input.model_document.first() != Some(&b'{')
    {
        return Err(PrismError::new(
            "PP3010",
            "application layers or model document are malformed",
        ));
    }

    let capability_request = CapabilitySet::new(empty_capabilities()).canonicalize();
    let capability_kappa = address_bytes(&capability_request);
    let guest_kappa = address_bytes(&input.guest_wasm);
    let view_kappa = address_bytes(&input.view_bundle);
    let model_kappa = address_bytes(&input.model_document);
    let manifest = AppManifest {
        primary: Some(0),
        requires: capability_kappa,
        layers: vec![
            Layer::wasm_with_contract(guest_kappa, "holo_run", WASM_CONTRACT_CORE_V1),
            Layer {
                kind: LayerKind::View,
                content: view_kappa,
                entry: "index.html".to_owned(),
                aux: "portable".to_owned(),
            },
        ],
        children: Vec::new(),
    };
    manifest.validate().map_err(|error| {
        PrismError::new("PP3011", format!("invalid application manifest: {error:?}"))
    })?;
    let application_manifest = manifest.canonicalize();
    let application_kappa = address_bytes(&application_manifest);

    let mut blob_rows = [
        (capability_kappa, capability_request.as_slice()),
        (guest_kappa, input.guest_wasm.as_slice()),
        (model_kappa, input.model_document.as_slice()),
        (view_kappa, input.view_bundle.as_slice()),
    ];
    blob_rows.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let directory_value = Directory {
        schema_version: 1,
        primary_layer: Some(0),
        requires_kappa: capability_kappa.to_string(),
        layers: vec![
            DirectoryLayer {
                position: 0,
                kind: "wasm".to_owned(),
                content_kappa: guest_kappa.to_string(),
                entry: "holo_run".to_owned(),
                contract: Some(WASM_CONTRACT_CORE_V1.to_owned()),
                architecture: None,
                surface: None,
                engine: None,
            },
            DirectoryLayer {
                position: 1,
                kind: "view".to_owned(),
                content_kappa: view_kappa.to_string(),
                entry: "index.html".to_owned(),
                contract: None,
                architecture: None,
                surface: Some("portable".to_owned()),
                engine: None,
            },
        ],
        children: Vec::new(),
        blobs: blob_rows
            .iter()
            .map(|(kappa, bytes)| DirectoryBlob {
                kappa: kappa.to_string(),
                byte_length: bytes.len() as u64,
            })
            .collect(),
    };
    let directory = serde_json::to_vec(&directory_value)
        .map_err(|error| PrismError::new("PP9001", error.to_string()))?;

    let model_id = content_id(&input.model_document);
    let provenance = &input.provenance;
    let prism_extension_value = serde_json::to_value(ModelProvenanceV1 {
        schema: "prismpm/model-provenance/1".to_owned(),
        model_content_kappa: model_kappa.to_string(),
        model_id,
        source_id: provenance.source_id.clone(),
        semantic_id: provenance.semantic_id.clone(),
        compiler_semantics_id: provenance.compiler_semantics_id.clone(),
        snapshot_id: provenance.snapshot_id.clone(),
        stdlib_semantics_id: provenance.stdlib_semantics_id.clone(),
        prism_stdlib_crate_sha256: provenance.prism_stdlib_crate_sha256.clone(),
        lexlean_commit: provenance.lexlean_commit.clone(),
        lexlean_package_sha256: provenance.lexlean_package_sha256.clone(),
        lean4_prod_commit: provenance.lean4_prod_commit.clone(),
        hologram_live_commit: provenance.hologram_live_commit.clone(),
        uor_hologram_commit: provenance.uor_hologram_commit.clone(),
        target_profile_id: provenance.target_profile_id.clone(),
        core_wasm_contract: WASM_CONTRACT_CORE_V1.to_owned(),
        lean_manifest_sha256: provenance.lean_manifest_sha256.clone(),
        lcnf_manifest_sha256: provenance.lcnf_manifest_sha256.clone(),
        generated_core_sha256: provenance.generated_core_sha256.clone(),
        cargo_name: provenance.cargo_name.clone(),
        cargo_version: provenance.cargo_version.clone(),
        cargo_crate_sha256: provenance.cargo_crate_sha256.clone(),
        guest_content_kappa: guest_kappa.to_string(),
        view_binding: ViewBinding::Present {
            view_model_id: provenance.view_model_id.clone(),
            view_content_kappa: view_kappa.to_string(),
            browser_projection: BrowserProjection::Present {
                sha256: provenance.browser_projection_sha256.clone(),
            },
        },
        application_kappa: application_kappa.to_string(),
    })
    .map_err(|error| PrismError::new("PP9001", error.to_string()))?;
    let prism_extension = encode_value(&prism_extension_value)?;

    let mut sections = vec![
        (SectionKind::AppManifest, application_manifest.clone()),
        (SectionKind::Metadata, input.source_manifest.clone()),
        (
            SectionKind::Extension,
            extension(DIRECTORY_EXTENSION, &directory)?,
        ),
        (
            SectionKind::Extension,
            extension(PRISM_EXTENSION, &prism_extension)?,
        ),
    ];
    for (kappa, bytes) in blob_rows {
        let mut blob = Vec::with_capacity(71 + bytes.len());
        blob.extend_from_slice(kappa.as_bytes());
        blob.extend_from_slice(bytes);
        sections.push((SectionKind::ContentBlob, blob));
    }
    let bytes = HoloWriter::assemble(sections);
    let fingerprint = bytes
        .get(bytes.len().saturating_sub(32)..)
        .ok_or_else(|| PrismError::new("PP3001", "archive has no footer"))?;
    let identities = HoloIdentities {
        guest_content_kappa: guest_kappa.to_string(),
        view_content_kappa: view_kappa.to_string(),
        model_content_kappa: model_kappa.to_string(),
        application_kappa: application_kappa.to_string(),
        archive_fingerprint: fingerprint
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        archive_kappa: address_bytes(&bytes).to_string(),
    };
    validate_application(&bytes)?;
    Ok(GeneratedHolo {
        bytes,
        application_manifest,
        capability_request,
        directory,
        prism_extension,
        identities,
    })
}

/// Strictly validate the closed Holo/1 Calculator/portable-app archive profile.
pub fn validate_application(bytes: &[u8]) -> Result<(), PrismError> {
    if bytes.starts_with(b"{") || !bytes.starts_with(b"HOLO\x04\0") {
        return Err(PrismError::new(
            "PP3001",
            "a .holo file must be a binary Hologram v4 archive",
        ));
    }
    let loader = HoloLoader::from_bytes(bytes)
        .map_err(|error| PrismError::new("PP3001", format!("Hologram archive: {error}")))?;
    let plan = loader
        .into_plan()
        .map_err(|error| PrismError::new("PP3001", format!("Hologram plan: {error}")))?;
    let expected = [
        SectionKind::AppManifest,
        SectionKind::Metadata,
        SectionKind::Extension,
        SectionKind::Extension,
        SectionKind::ContentBlob,
        SectionKind::ContentBlob,
        SectionKind::ContentBlob,
        SectionKind::ContentBlob,
    ];
    if plan.sections().len() != expected.len()
        || plan
            .sections()
            .iter()
            .zip(expected)
            .any(|(actual, expected)| actual.kind != expected)
    {
        return Err(PrismError::new(
            "PP3003",
            "archive section set or order is not the closed Holo/1 profile",
        ));
    }
    let manifest_bytes = plan
        .app_manifest()
        .ok_or_else(|| PrismError::new("PP3005", "archive has no AppManifest"))?;
    let manifest = AppManifest::decode(manifest_bytes)
        .map_err(|error| PrismError::new("PP3005", format!("AppManifest: {error:?}")))?;
    if manifest.canonicalize() != manifest_bytes
        || manifest.validate().is_err()
        || manifest.primary != Some(0)
        || manifest.layers.len() != 2
        || manifest.layers[0].kind != LayerKind::WasmCodemodule
        || manifest.layers[0].entry != "holo_run"
        || manifest.layers[0].aux != WASM_CONTRACT_CORE_V1
        || manifest.layers[1].kind != LayerKind::View
        || manifest.layers[1].entry != "index.html"
        || manifest.layers[1].aux != "portable"
        || !manifest.children.is_empty()
    {
        return Err(PrismError::new(
            "PP3005",
            "application manifest disagrees with Holo/1",
        ));
    }
    let blobs = plan
        .content_blobs()
        .map_err(|error| PrismError::new("PP3003", error.to_string()))?;
    let mut seen = BTreeMap::new();
    for (label, content) in &blobs {
        let expected = address_bytes(content);
        if expected.as_bytes() != *label || seen.insert(label.to_vec(), *content).is_some() {
            return Err(PrismError::new(
                "PP3008",
                "content blob label is duplicate or does not match its bytes",
            ));
        }
    }
    let refs = <AppManifest as Realization>::references(manifest_bytes)
        .map_err(|error| PrismError::new("PP3005", format!("manifest references: {error:?}")))?;
    if refs
        .iter()
        .any(|reference| !seen.contains_key(reference.as_bytes()))
    {
        return Err(PrismError::new(
            "PP3009",
            "fat archive omits a manifest dependency",
        ));
    }
    let extensions = plan
        .extensions()
        .map_err(|error| PrismError::new("PP3013", error.to_string()))?;
    if extensions.len() != 2
        || extensions[0].0 != DIRECTORY_EXTENSION
        || extensions[1].0 != PRISM_EXTENSION
    {
        return Err(PrismError::new(
            "PP3013",
            "archive extension set is not canonical",
        ));
    }
    let declared: Directory = serde_json::from_slice(extensions[0].1)
        .map_err(|error| PrismError::new("PP3012", format!("directory: {error}")))?;
    let mut expected_blobs = blobs
        .iter()
        .map(|(label, content)| DirectoryBlob {
            kappa: std::str::from_utf8(label).unwrap_or_default().to_owned(),
            byte_length: content.len() as u64,
        })
        .collect::<Vec<_>>();
    expected_blobs.sort_by(|left, right| left.kappa.as_bytes().cmp(right.kappa.as_bytes()));
    if declared.schema_version != 1
        || declared.primary_layer != Some(0)
        || declared.requires_kappa != manifest.requires.to_string()
        || declared.layers.len() != 2
        || declared.children != Vec::<DirectoryChild>::new()
        || declared.blobs != expected_blobs
    {
        return Err(PrismError::new(
            "PP3012",
            "application directory disagrees with manifest or blobs",
        ));
    }
    let provenance: ModelProvenanceV1 = serde_json::from_slice(extensions[1].1)
        .map_err(|error| PrismError::new("PP3015", format!("Prism extension: {error}")))?;
    let (view_model_id, view_content_kappa, browser_sha256) = match &provenance.view_binding {
        ViewBinding::Present {
            view_model_id,
            view_content_kappa,
            browser_projection: BrowserProjection::Present { sha256 },
        } => (view_model_id, view_content_kappa, sha256),
        _ => {
            return Err(PrismError::new(
                "PP3015",
                "portable View applications require a bound browser projection",
            ));
        }
    };
    let model_bytes = seen.get(provenance.model_content_kappa.as_bytes());
    let guest_bytes = seen.get(provenance.guest_content_kappa.as_bytes());
    let view_bytes = seen.get(view_content_kappa.as_bytes());
    let digest_fields = [
        &provenance.model_id,
        &provenance.source_id,
        &provenance.semantic_id,
        &provenance.compiler_semantics_id,
        &provenance.snapshot_id,
        &provenance.stdlib_semantics_id,
        &provenance.prism_stdlib_crate_sha256,
        &provenance.lexlean_package_sha256,
        &provenance.target_profile_id,
        &provenance.lean_manifest_sha256,
        &provenance.lcnf_manifest_sha256,
        &provenance.generated_core_sha256,
        &provenance.cargo_crate_sha256,
        view_model_id,
        browser_sha256,
    ];
    if provenance.schema != "prismpm/model-provenance/1"
        || provenance.core_wasm_contract != WASM_CONTRACT_CORE_V1
        || provenance.application_kappa != address_bytes(manifest_bytes).to_string()
        || provenance.guest_content_kappa != manifest.layers[0].content.to_string()
        || *view_content_kappa != manifest.layers[1].content.to_string()
        || model_bytes.is_none()
        || guest_bytes.is_none_or(|bytes| !bytes.starts_with(b"\0asm"))
        || view_bytes.is_none_or(|bytes| !bytes.starts_with(b"HOLOVIEW\0\x01"))
        || model_bytes.is_none_or(|bytes| content_id(bytes) != provenance.model_id)
        || digest_fields
            .into_iter()
            .any(|digest| !digest_is_valid(digest))
        || provenance.lexlean_commit.len() != 40
        || provenance.lean4_prod_commit.len() != 40
        || provenance.hologram_live_commit.len() != 40
        || provenance.uor_hologram_commit.len() != 40
        || provenance.cargo_name.is_empty()
        || provenance.cargo_version.is_empty()
    {
        return Err(PrismError::new(
            "PP3015",
            "Prism provenance is missing or disagrees with archive content",
        ));
    }
    let _ = Sha256::digest(extensions[1].1);
    Ok(())
}
