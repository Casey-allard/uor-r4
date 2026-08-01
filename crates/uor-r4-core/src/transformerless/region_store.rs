//! I/O-side region objects and resolver-backed store access (#263).
//!
//! TLS1 remains the compact monolithic store used by the existing runtime.
//! This module gives each `(depth, prefix)` subtree an independently
//! addressable canonical object and provides the adapter needed to serve a
//! prediction from a manifest plus a resolver. No prediction-kernel code is
//! changed here.

use std::collections::BTreeMap;

use super::compiler::STAGES;
use super::runtime::{Prediction, Store};

/// Region-object wire schema version.
pub const REGION_OBJECT_SCHEMA: u64 = 1;
/// Manifest wire schema version.
pub const REGION_MANIFEST_SCHEMA: u32 = 1;

/// One canonical region object: a graded prefix and its token distribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionObject {
    /// Store grade / prefix depth.
    pub depth: u8,
    /// The class-prefix key at `depth`.
    pub key: Vec<u8>,
    /// Deterministically ordered next-token evidence.
    pub distribution: BTreeMap<u32, u32>,
}

/// One manifest entry identifying a resolver-backed region object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionReference {
    /// Store grade / prefix depth.
    pub depth: u8,
    /// The class-prefix key at `depth`.
    pub key: Vec<u8>,
    /// UOR κ-label of the canonical region object.
    pub kappa: String,
    /// Canonical object byte length.
    pub bytes: u64,
}

/// A manifest sufficient to address every region without loading payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionManifest {
    /// Manifest schema version.
    pub schema: u32,
    /// UOR κ-label of the canonical manifest skeleton.
    pub manifest_kappa: String,
    /// Region references in `(depth, key)` order.
    pub regions: Vec<RegionReference>,
}

/// Export result containing the manifest and objects to place in a CAS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionExport {
    pub manifest: RegionManifest,
    pub objects: Vec<RegionObject>,
}

/// Errors from canonical region serialization or address derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionObjectError {
    InvalidSchema,
    InvalidKeyDepth,
    DuplicateToken,
    Malformed,
    AddressingFailed,
}

impl core::fmt::Display for RegionObjectError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::InvalidSchema => "unsupported region-object schema",
            Self::InvalidKeyDepth => "region key length does not match depth",
            Self::DuplicateToken => "region object contains a duplicate token",
            Self::Malformed => "malformed region object or manifest",
            Self::AddressingFailed => "uor-addr rejected the region skeleton",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for RegionObjectError {}

/// Resolver errors are separate from a cache miss (`Ok(None)`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionResolveError {
    InvalidObject(RegionObjectError),
    Backend(String),
}

impl core::fmt::Display for RegionResolveError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidObject(error) => write!(formatter, "invalid region object: {error}"),
            Self::Backend(error) => write!(formatter, "region resolver backend failed: {error}"),
        }
    }
}

impl std::error::Error for RegionResolveError {}

/// Fetch a region object by its manifest κ-label.
pub trait RegionResolver {
    fn resolve(&self, kappa: &str) -> Result<Option<RegionObject>, RegionResolveError>;
}

/// In-memory resolver useful for tests, local caches, and composition.
#[derive(Debug, Clone, Default)]
pub struct MemoryRegionResolver {
    objects: BTreeMap<String, RegionObject>,
}

impl MemoryRegionResolver {
    pub fn from_export(export: &RegionExport) -> Result<Self, RegionObjectError> {
        let mut resolver = Self::default();
        for object in &export.objects {
            resolver.insert(object.clone())?;
        }
        Ok(resolver)
    }

    pub fn insert(&mut self, object: RegionObject) -> Result<String, RegionObjectError> {
        let kappa = region_kappa(&object)?;
        self.objects.insert(kappa.clone(), object);
        Ok(kappa)
    }
}

impl RegionResolver for MemoryRegionResolver {
    fn resolve(&self, kappa: &str) -> Result<Option<RegionObject>, RegionResolveError> {
        Ok(self.objects.get(kappa).cloned())
    }
}

/// Convert every TLS1 entry into one canonical region object.
pub fn export_store(store: &Store) -> Result<RegionExport, RegionObjectError> {
    let mut objects = Vec::new();
    let mut regions = Vec::new();
    for (depth, level) in store.iter().enumerate() {
        let depth = u8::try_from(depth).map_err(|_| RegionObjectError::InvalidKeyDepth)?;
        for (key, distribution) in level {
            if key.len() != usize::from(depth) {
                return Err(RegionObjectError::InvalidKeyDepth);
            }
            let object = RegionObject {
                depth,
                key: key.clone(),
                distribution: distribution.clone(),
            };
            let bytes = canonical_region_bytes(&object)?;
            let kappa = region_kappa(&object)?;
            regions.push(RegionReference {
                depth,
                key: key.clone(),
                kappa,
                bytes: bytes.len() as u64,
            });
            objects.push(object);
        }
    }
    let manifest_kappa = manifest_kappa_for(&regions)?;
    Ok(RegionExport {
        manifest: RegionManifest {
            schema: REGION_MANIFEST_SCHEMA,
            manifest_kappa,
            regions,
        },
        objects,
    })
}

/// Canonical CBOR bytes for one region object.
pub fn canonical_region_bytes(object: &RegionObject) -> Result<Vec<u8>, RegionObjectError> {
    validate_object(object)?;
    let mut out = Vec::new();
    cbor_array(&mut out, 4);
    cbor_uint(&mut out, REGION_OBJECT_SCHEMA);
    cbor_uint(&mut out, u64::from(object.depth));
    cbor_bytes(&mut out, &object.key);
    cbor_array(&mut out, object.distribution.len() as u64);
    for (&token, &count) in &object.distribution {
        cbor_array(&mut out, 2);
        cbor_uint(&mut out, u64::from(token));
        cbor_uint(&mut out, u64::from(count));
    }
    Ok(out)
}

#[derive(serde::Deserialize)]
struct RegionWire(u64, u8, Vec<u8>, Vec<(u32, u32)>);

/// Decode and validate one canonical region object.
pub fn decode_region_bytes(bytes: &[u8]) -> Result<RegionObject, RegionObjectError> {
    let mut cursor = std::io::Cursor::new(bytes);
    let RegionWire(schema, depth, key, entries): RegionWire =
        ciborium::de::from_reader(&mut cursor).map_err(|_| RegionObjectError::Malformed)?;
    if cursor.position() != bytes.len() as u64 || schema != REGION_OBJECT_SCHEMA {
        return Err(RegionObjectError::Malformed);
    }
    let mut distribution = BTreeMap::new();
    for (token, count) in entries {
        if distribution.insert(token, count).is_some() {
            return Err(RegionObjectError::DuplicateToken);
        }
    }
    let object = RegionObject {
        depth,
        key,
        distribution,
    };
    if canonical_region_bytes(&object)?.as_slice() != bytes {
        return Err(RegionObjectError::Malformed);
    }
    Ok(object)
}

/// Address one canonical region object through the pinned UOR CBOR axis.
pub fn region_kappa(object: &RegionObject) -> Result<String, RegionObjectError> {
    let bytes = canonical_region_bytes(object)?;
    uor_addr::cbor::address_blake3(&bytes)
        .map(|outcome| outcome.address.to_string())
        .map_err(|_| RegionObjectError::AddressingFailed)
}

/// Resolve the deepest available prefix and apply deterministic token argmax.
pub fn predict_witness_with_resolver<R: RegionResolver>(
    resolver: &R,
    manifest: &RegionManifest,
    code: &[u8; STAGES],
) -> Result<Prediction, RegionResolveError> {
    if manifest.schema != REGION_MANIFEST_SCHEMA {
        return Err(RegionResolveError::InvalidObject(
            RegionObjectError::InvalidSchema,
        ));
    }
    for depth in (0..=STAGES).rev() {
        let depth_u8 = depth as u8;
        let key = &code[..depth];
        let Some(reference) = manifest
            .regions
            .iter()
            .find(|region| region.depth == depth_u8 && region.key == key)
        else {
            continue;
        };
        let Some(object) = resolver.resolve(&reference.kappa)? else {
            continue;
        };
        if object.depth != depth_u8 || object.key != key {
            return Err(RegionResolveError::InvalidObject(
                RegionObjectError::InvalidKeyDepth,
            ));
        }
        let object_bytes =
            canonical_region_bytes(&object).map_err(RegionResolveError::InvalidObject)?;
        let object_kappa = region_kappa(&object).map_err(RegionResolveError::InvalidObject)?;
        if object_bytes.len() as u64 != reference.bytes || object_kappa != reference.kappa {
            return Err(RegionResolveError::InvalidObject(
                RegionObjectError::Malformed,
            ));
        }
        return Ok(argmax(depth_u8, &object.distribution));
    }
    Ok(Prediction::default())
}

fn argmax(depth: u8, distribution: &BTreeMap<u32, u32>) -> Prediction {
    let mut best = Prediction::default();
    let mut best_count = None;
    for (&token, &count) in distribution {
        if best_count.is_none_or(|best_count| count > best_count) {
            best = Prediction {
                token,
                depth,
                count,
            };
            best_count = Some(count);
        }
    }
    best
}

fn validate_object(object: &RegionObject) -> Result<(), RegionObjectError> {
    if object.key.len() != usize::from(object.depth) {
        return Err(RegionObjectError::InvalidKeyDepth);
    }
    Ok(())
}

/// Canonical CBOR bytes for a region manifest.
pub fn canonical_manifest_bytes(manifest: &RegionManifest) -> Result<Vec<u8>, RegionObjectError> {
    if manifest.schema != REGION_MANIFEST_SCHEMA {
        return Err(RegionObjectError::InvalidSchema);
    }
    validate_regions(&manifest.regions)?;
    let mut bytes = Vec::new();
    cbor_array(&mut bytes, 4);
    cbor_uint(&mut bytes, u64::from(REGION_MANIFEST_SCHEMA));
    cbor_text(&mut bytes, "r4-region-manifest");
    cbor_text(&mut bytes, &manifest.manifest_kappa);
    cbor_array(&mut bytes, manifest.regions.len() as u64);
    for region in &manifest.regions {
        cbor_array(&mut bytes, 4);
        cbor_uint(&mut bytes, u64::from(region.depth));
        cbor_bytes(&mut bytes, &region.key);
        cbor_text(&mut bytes, &region.kappa);
        cbor_uint(&mut bytes, region.bytes);
    }
    Ok(bytes)
}

/// Derive the manifest κ-label from its canonical skeleton.
pub fn manifest_kappa_for(regions: &[RegionReference]) -> Result<String, RegionObjectError> {
    validate_regions(regions)?;
    let mut bytes = Vec::new();
    cbor_array(&mut bytes, 3);
    cbor_uint(&mut bytes, u64::from(REGION_MANIFEST_SCHEMA));
    cbor_text(&mut bytes, "r4-region-manifest");
    cbor_array(&mut bytes, regions.len() as u64);
    for region in regions {
        cbor_array(&mut bytes, 4);
        cbor_uint(&mut bytes, u64::from(region.depth));
        cbor_bytes(&mut bytes, &region.key);
        cbor_text(&mut bytes, &region.kappa);
        cbor_uint(&mut bytes, region.bytes);
    }
    uor_addr::cbor::address_blake3(&bytes)
        .map(|outcome| outcome.address.to_string())
        .map_err(|_| RegionObjectError::AddressingFailed)
}

fn validate_regions(regions: &[RegionReference]) -> Result<(), RegionObjectError> {
    let mut previous: Option<(&u8, &[u8])> = None;
    for region in regions {
        if region.key.len() != usize::from(region.depth) {
            return Err(RegionObjectError::InvalidKeyDepth);
        }
        let current = (&region.depth, region.key.as_slice());
        if previous.is_some_and(|previous| current <= previous) {
            return Err(RegionObjectError::Malformed);
        }
        previous = Some(current);
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct ManifestWire(u32, String, String, Vec<(u8, Vec<u8>, String, u64)>);

/// Decode and validate a canonical region manifest.
pub fn decode_manifest_bytes(bytes: &[u8]) -> Result<RegionManifest, RegionObjectError> {
    let mut cursor = std::io::Cursor::new(bytes);
    let ManifestWire(schema, domain, kappa, entries): ManifestWire =
        ciborium::de::from_reader(&mut cursor).map_err(|_| RegionObjectError::Malformed)?;
    if cursor.position() != bytes.len() as u64
        || schema != REGION_MANIFEST_SCHEMA
        || domain != "r4-region-manifest"
    {
        return Err(RegionObjectError::Malformed);
    }
    let regions = entries
        .into_iter()
        .map(|(depth, key, kappa, bytes)| RegionReference {
            depth,
            key,
            kappa,
            bytes,
        })
        .collect::<Vec<_>>();
    let expected = manifest_kappa_for(&regions)?;
    if expected != kappa {
        return Err(RegionObjectError::Malformed);
    }
    let manifest = RegionManifest {
        schema,
        manifest_kappa: kappa,
        regions,
    };
    if canonical_manifest_bytes(&manifest)?.as_slice() != bytes {
        return Err(RegionObjectError::Malformed);
    }
    Ok(manifest)
}

fn cbor_array(out: &mut Vec<u8>, length: u64) {
    cbor_head(out, 4, length);
}

fn cbor_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    cbor_head(out, 2, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

fn cbor_text(out: &mut Vec<u8>, text: &str) {
    cbor_head(out, 3, text.len() as u64);
    out.extend_from_slice(text.as_bytes());
}

fn cbor_uint(out: &mut Vec<u8>, value: u64) {
    cbor_head(out, 0, value);
}

fn cbor_head(out: &mut Vec<u8>, major: u8, value: u64) {
    if value <= 23 {
        out.push((major << 5) | value as u8);
    } else if value <= u8::MAX as u64 {
        out.extend_from_slice(&[(major << 5) | 24, value as u8]);
    } else if value <= u16::MAX as u64 {
        out.push((major << 5) | 25);
        out.extend_from_slice(&(value as u16).to_be_bytes());
    } else if value <= u32::MAX as u64 {
        out.push((major << 5) | 26);
        out.extend_from_slice(&(value as u32).to_be_bytes());
    } else {
        out.push((major << 5) | 27);
        out.extend_from_slice(&value.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transformerless::runtime::predict_witness_plain;

    fn fixture_store() -> Store {
        let mut store: Store = (0..=STAGES).map(|_| BTreeMap::new()).collect();
        store[0].insert(vec![], BTreeMap::from([(7, 2), (9, 1)]));
        store[1].insert(vec![1], BTreeMap::from([(11, 4), (12, 3)]));
        store[2].insert(vec![1, 2], BTreeMap::from([(21, 5)]));
        store
    }

    #[test]
    fn manifest_resolver_matches_monolithic_predictions() {
        let store = fixture_store();
        let export = export_store(&store).expect("fixture exports");
        let resolver = MemoryRegionResolver::from_export(&export).expect("resolver loads");
        let code = [1, 2, 3, 4];
        let expected = predict_witness_plain(&store, &code);
        let actual = predict_witness_with_resolver(&resolver, &export.manifest, &code)
            .expect("resolver prediction");
        assert_eq!(actual, expected);
        assert!(export.manifest.manifest_kappa.starts_with("blake3:"));
    }

    #[test]
    fn region_kappas_are_deterministic_and_payload_bound() {
        let first = RegionObject {
            depth: 1,
            key: vec![4],
            distribution: BTreeMap::from([(3, 1)]),
        };
        let second = RegionObject {
            distribution: BTreeMap::from([(3, 2)]),
            ..first.clone()
        };
        assert_eq!(region_kappa(&first), region_kappa(&first));
        assert_ne!(region_kappa(&first), region_kappa(&second));
        assert_eq!(
            canonical_region_bytes(&first).unwrap(),
            canonical_region_bytes(&first).unwrap()
        );
    }
}
