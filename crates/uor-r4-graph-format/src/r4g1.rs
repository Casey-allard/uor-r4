//! The canonical `uor-addr` realization for an R4G1 artifact.
//!
//! R4G1's wire CIDs remain byte-level integrity checks: they protect the
//! exact container that a runtime mapped.  This module provides the
//! representation-level address used in manifests, reports, and serving
//! attestations.  It deliberately omits offsets, alignment padding, and the
//! two wire CIDs, so equivalent containers have the same address.
//!
//! The realization is a small Merkle skeleton.  A section address commits to
//! its semantic table identity (`id` and `flags`) and payload.  The artifact
//! address then commits to the sorted list of section identities and section
//! addresses.  Both levels are fed through the pinned CBOR realization of
//! `uor-addr` on its BLAKE3 axis.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::{FormatError, GraphView, SectionId};

/// Version of the R4G1 realization skeleton.
pub const REALIZATION_VERSION: u64 = 1;
/// Stable realization discriminator included in every skeleton.
pub const REALIZATION_NAME: &str = "r4g1";

/// Failure while validating or addressing an R4G1 artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealizationError {
    /// The container failed R4G1 structural or semantic validation.
    InvalidArtifact(FormatError),
    /// The generated skeleton was not accepted by the CBOR realization.
    AddressingFailed,
}

impl core::fmt::Display for RealizationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidArtifact(error) => write!(formatter, "invalid R4G1 artifact: {error}"),
            Self::AddressingFailed => write!(formatter, "uor-addr rejected the R4G1 skeleton"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RealizationError {}

/// A section's representation-level address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionAddress {
    /// Wire section identifier.
    pub id: SectionId,
    /// Section-table flags, which are part of the section's identity.
    pub flags: u32,
    /// BLAKE3-axis UOR κ-label for this section.
    pub kappa: String,
}

/// The complete address and replayable skeleton inputs for one artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R4G1Address {
    /// BLAKE3-axis UOR κ-label for the canonical artifact skeleton.
    pub artifact_kappa: String,
    /// Per-section addresses in canonical section-ID order.
    pub sections: Vec<SectionAddress>,
    /// Canonical CBOR skeleton addressed by `artifact_kappa`.
    pub skeleton: Vec<u8>,
}

/// Address a validated R4G1 container using its canonical section skeleton.
pub fn address(bytes: &[u8]) -> Result<R4G1Address, RealizationError> {
    let view = GraphView::parse(bytes).map_err(RealizationError::InvalidArtifact)?;
    view.verify_cids()
        .map_err(RealizationError::InvalidArtifact)?;

    let mut sections = Vec::with_capacity(view.sections().len());
    for section in view.sections() {
        let section_skeleton = section_skeleton(section.id, section.flags, section.payload);
        let kappa = address_cbor(&section_skeleton)?;
        sections.push(SectionAddress {
            id: section.id,
            flags: section.flags,
            kappa,
        });
    }

    let skeleton = artifact_skeleton(&sections);
    let artifact_kappa = address_cbor(&skeleton)?;
    Ok(R4G1Address {
        artifact_kappa,
        sections,
        skeleton,
    })
}

/// Return only the representation-level artifact κ-label.
pub fn artifact_kappa(bytes: &[u8]) -> Result<String, RealizationError> {
    Ok(address(bytes)?.artifact_kappa)
}

/// Return the representation-level κ-label for one section.
pub fn section_kappa(bytes: &[u8], id: SectionId) -> Result<Option<String>, RealizationError> {
    Ok(address(bytes)?
        .sections
        .into_iter()
        .find(|section| section.id == id)
        .map(|section| section.kappa))
}

fn address_cbor(skeleton: &[u8]) -> Result<String, RealizationError> {
    uor_addr::cbor::address_blake3(skeleton)
        .map(|outcome| outcome.address.to_string())
        .map_err(|_| RealizationError::AddressingFailed)
}

fn section_skeleton(id: SectionId, flags: u32, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 48);
    cbor_array(&mut out, 5);
    cbor_uint(&mut out, REALIZATION_VERSION);
    cbor_text(&mut out, REALIZATION_NAME);
    cbor_uint(&mut out, u64::from(id.raw()));
    cbor_uint(&mut out, u64::from(flags));
    cbor_bytes(&mut out, payload);
    out
}

fn artifact_skeleton(sections: &[SectionAddress]) -> Vec<u8> {
    let mut out = Vec::with_capacity(32 + sections.len() * 96);
    cbor_array(&mut out, 3);
    cbor_uint(&mut out, REALIZATION_VERSION);
    cbor_text(&mut out, REALIZATION_NAME);
    cbor_array(&mut out, sections.len() as u64);
    for section in sections {
        cbor_array(&mut out, 3);
        cbor_uint(&mut out, u64::from(section.id.raw()));
        cbor_uint(&mut out, u64::from(section.flags));
        cbor_text(&mut out, &section.kappa);
    }
    out
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
    debug_assert!(major < 8);
    if value <= 23 {
        out.push((major << 5) | value as u8);
    } else if value <= u8::MAX as u64 {
        out.push((major << 5) | 24);
        out.push(value as u8);
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

    #[test]
    fn cbor_heads_are_minimally_encoded() {
        let mut out = Vec::new();
        cbor_uint(&mut out, 23);
        cbor_uint(&mut out, 24);
        cbor_uint(&mut out, 256);
        assert_eq!(out, [0x17, 0x18, 0x18, 0x19, 0x01, 0x00]);
    }
}
