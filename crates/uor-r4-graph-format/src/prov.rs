//! Borrowed PROV section view (#637 PROV/1) — provenance roots binding
//! the source/geometry/tokenizer/attention/dense identity digests
//! (#597/#600/#601/#602/#704), an SPDX license expression, and a
//! canonically-ordered list of evidence-root κ for deletion support
//! (RFC §3 row 0x08, RFC §5 "PROV" bullet).
//!
//! Format-only freeze (#637 phase 1): this module parses and canonically
//! builds PROV/1 bytes. It does **not** wire PROV into
//! `stage2::validate`'s mandatory-section completeness pass or the
//! compiler's write path — those are #637 phases 2 and 3.
//!
//! Wire layout mirrors FWDA's (`fwda.rs`, issue #399) MAGIC+VERSION+
//! checked-bounds+canonically-sorted-list convention for the variable
//! evidence-root tail, and HEAD's fixed 32-byte digest-slot convention
//! for the identity fields:
//!
//! ```text
//! offset  size   field
//! 0       4B     magic = "PRV1"
//! 4       2B     version u16 (=1)
//! 6       1B     presence bitmap: bit0=source_manifest_kappa(#597)
//!                bit1=geometry(#600) bit2=tokenizer_adapter(#601)
//!                bit3=attention_operator(#602) bit4=dense_operator(#704)
//!                bit5=license bits6-7=reserved(0)
//! 7       1B     reserved (0)
//! 8       4B     evidence_root_count u32
//! 12      4B     license_len u32 (0 when presence bit5 clear)
//! 16      32B    source_manifest_kappa digest (zero-filled when absent)
//! 48      32B    geometry_digest (zero-filled when absent)
//! 80      32B    tokenizer_adapter_digest (zero-filled when absent)
//! 112     32B    attention_operator_digest (zero-filled when absent)
//! 144     32B    dense_operator_digest (zero-filled when absent)
//! 176     ...    license bytes (ASCII SPDX expression, license_len bytes)
//!         ...    evidence roots: evidence_root_count × 32B κ, strictly
//!                ascending, no duplicates
//! ```
//!
//! A presence bit governs whether its digest slot means anything: a
//! clear bit requires an all-zero slot (structural consistency between
//! the flag and the bytes — `parse` rejects a mismatch as tamper, not as
//! a silently-ignored stray value). Digests are opaque 32-byte values
//! here — this section carries identity, not content; the
//! sidecars/manifests that own each component's canonical bytes are
//! unchanged by this freeze.

use crate::error::FormatError;
use crate::sanctioned::NotAProduct;

/// PROV/1 magic bytes.
pub const PROV_MAGIC: [u8; 4] = *b"PRV1";
/// PROV format version this module reads and writes.
pub const PROV_VERSION: u16 = 1;
/// Fixed prefix length before the variable license/evidence-root tail.
pub const PROV_HEADER_LEN: usize = 176;
/// Width of one digest slot / evidence-root entry.
pub const PROV_DIGEST_LEN: usize = 32;

const PRESENCE_SOURCE_MANIFEST_KAPPA: u8 = 1 << 0;
const PRESENCE_GEOMETRY: u8 = 1 << 1;
const PRESENCE_TOKENIZER_ADAPTER: u8 = 1 << 2;
const PRESENCE_ATTENTION_OPERATOR: u8 = 1 << 3;
const PRESENCE_DENSE_OPERATOR: u8 = 1 << 4;
const PRESENCE_LICENSE: u8 = 1 << 5;
const PRESENCE_KNOWN_BITS: u8 = PRESENCE_SOURCE_MANIFEST_KAPPA
    | PRESENCE_GEOMETRY
    | PRESENCE_TOKENIZER_ADAPTER
    | PRESENCE_ATTENTION_OPERATOR
    | PRESENCE_DENSE_OPERATOR
    | PRESENCE_LICENSE;

const SLOT_SOURCE_MANIFEST_KAPPA: usize = 16;
const SLOT_GEOMETRY: usize = 48;
const SLOT_TOKENIZER_ADAPTER: usize = 80;
const SLOT_ATTENTION_OPERATOR: usize = 112;
const SLOT_DENSE_OPERATOR: usize = 144;

/// One digest slot's presence bit and byte offset, in wire order.
const SLOTS: [(u8, usize); 5] = [
    (PRESENCE_SOURCE_MANIFEST_KAPPA, SLOT_SOURCE_MANIFEST_KAPPA),
    (PRESENCE_GEOMETRY, SLOT_GEOMETRY),
    (PRESENCE_TOKENIZER_ADAPTER, SLOT_TOKENIZER_ADAPTER),
    (PRESENCE_ATTENTION_OPERATOR, SLOT_ATTENTION_OPERATOR),
    (PRESENCE_DENSE_OPERATOR, SLOT_DENSE_OPERATOR),
];

/// Borrowed, validated view of one PROV/1 section's bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prov<'a> {
    bytes: &'a [u8],
    presence: u8,
    evidence_root_count: u32,
    evidence_roots_start: usize,
}

impl<'a> Prov<'a> {
    /// Parse and structurally validate one PROV/1 section's bytes.
    ///
    /// Checks (in order): minimum length, magic, version, reserved
    /// byte/bits zero, every presence bit agrees with its digest slot's
    /// zero/non-zero contents, the license length agrees with its
    /// presence bit and is ASCII, the license and evidence-root ranges
    /// resolve under checked arithmetic with no trailing bytes, and the
    /// evidence roots are strictly ascending (which also rejects
    /// duplicates — an equal successor fails the same `>=` check as an
    /// out-of-order one).
    pub fn parse(bytes: &'a [u8]) -> Result<Self, NotAProduct> {
        if bytes.len() < PROV_HEADER_LEN {
            return Err(FormatError::ProvTooShort.into());
        }
        if bytes[0..4] != PROV_MAGIC {
            return Err(FormatError::ProvBadMagic.into());
        }
        if read_u16(&bytes[4..6]) != PROV_VERSION {
            return Err(FormatError::ProvUnsupportedVersion.into());
        }
        let presence = bytes[6];
        if bytes[7] != 0 || presence & !PRESENCE_KNOWN_BITS != 0 {
            return Err(FormatError::ProvNonZeroReserved.into());
        }
        let evidence_root_count = read_u32(&bytes[8..12]);
        let license_len = read_u32(&bytes[12..16]);

        for (bit, slot_start) in SLOTS {
            let slot = &bytes[slot_start..slot_start + PROV_DIGEST_LEN];
            let all_zero = slot.iter().all(|&byte| byte == 0);
            if (presence & bit != 0) == all_zero {
                return Err(FormatError::ProvPresenceMismatch.into());
            }
        }

        if (presence & PRESENCE_LICENSE != 0) == (license_len == 0) {
            return Err(FormatError::ProvInvalidLicenseLength.into());
        }

        let license_start = PROV_HEADER_LEN;
        let license_end = license_start
            .checked_add(license_len as usize)
            .ok_or(FormatError::ProvBounds)?;
        if license_end > bytes.len() {
            return Err(FormatError::ProvBounds.into());
        }
        if !bytes[license_start..license_end].is_ascii() {
            return Err(FormatError::ProvLicenseNotAscii.into());
        }

        let evidence_roots_start = license_end;
        let evidence_bytes_len = (evidence_root_count as usize)
            .checked_mul(PROV_DIGEST_LEN)
            .ok_or(FormatError::ProvBounds)?;
        let evidence_roots_end = evidence_roots_start
            .checked_add(evidence_bytes_len)
            .ok_or(FormatError::ProvBounds)?;
        if evidence_roots_end != bytes.len() {
            return Err(FormatError::ProvBounds.into());
        }

        let mut previous: Option<&[u8]> = None;
        for chunk in bytes[evidence_roots_start..evidence_roots_end].chunks_exact(PROV_DIGEST_LEN) {
            if previous.is_some_and(|last| last >= chunk) {
                return Err(FormatError::ProvEvidenceRootsNotSorted.into());
            }
            previous = Some(chunk);
        }

        Ok(Self {
            bytes,
            presence,
            evidence_root_count,
            evidence_roots_start,
        })
    }

    fn slot(&self, bit: u8, start: usize) -> Option<[u8; PROV_DIGEST_LEN]> {
        if self.presence & bit == 0 {
            return None;
        }
        let mut out = [0u8; PROV_DIGEST_LEN];
        out.copy_from_slice(&self.bytes[start..start + PROV_DIGEST_LEN]);
        Some(out)
    }

    /// #597 source-snapshot manifest root κ digest, when bound.
    pub fn source_manifest_kappa(&self) -> Option<[u8; PROV_DIGEST_LEN]> {
        self.slot(PRESENCE_SOURCE_MANIFEST_KAPPA, SLOT_SOURCE_MANIFEST_KAPPA)
    }

    /// #600 geometry-projection identity digest, when bound.
    pub fn geometry_digest(&self) -> Option<[u8; PROV_DIGEST_LEN]> {
        self.slot(PRESENCE_GEOMETRY, SLOT_GEOMETRY)
    }

    /// #601 tokenizer-adapter identity digest
    /// (`TokenizerAdapter::adapter_digest`), when bound.
    pub fn tokenizer_adapter_digest(&self) -> Option<[u8; PROV_DIGEST_LEN]> {
        self.slot(PRESENCE_TOKENIZER_ADAPTER, SLOT_TOKENIZER_ADAPTER)
    }

    /// #602 source-attention-operator identity digest, when bound.
    pub fn attention_operator_digest(&self) -> Option<[u8; PROV_DIGEST_LEN]> {
        self.slot(PRESENCE_ATTENTION_OPERATOR, SLOT_ATTENTION_OPERATOR)
    }

    /// #704 dense-operator identity digest, when bound.
    pub fn dense_operator_digest(&self) -> Option<[u8; PROV_DIGEST_LEN]> {
        self.slot(PRESENCE_DENSE_OPERATOR, SLOT_DENSE_OPERATOR)
    }

    /// SPDX license expression, when declared. Validated ASCII at parse
    /// time, so this is a plain `&str` with no further checking needed.
    pub fn license(&self) -> Option<&'a str> {
        if self.presence & PRESENCE_LICENSE == 0 {
            return None;
        }
        let bytes = &self.bytes[PROV_HEADER_LEN..self.evidence_roots_start];
        // ASCII was checked in `parse`; ASCII is always valid UTF-8.
        Some(core::str::from_utf8(bytes).unwrap_or(""))
    }

    /// Number of evidence-root entries.
    pub fn evidence_root_count(&self) -> u32 {
        self.evidence_root_count
    }

    /// Evidence-root κ entries, strictly ascending (RFC "deletion
    /// support" roots).
    pub fn evidence_roots(&self) -> EvidenceRoots<'a> {
        EvidenceRoots {
            bytes: &self.bytes[self.evidence_roots_start..],
            remaining: self.evidence_root_count,
        }
    }
}

/// Iterator over one PROV section's evidence-root κ entries.
pub struct EvidenceRoots<'a> {
    bytes: &'a [u8],
    remaining: u32,
}

impl<'a> Iterator for EvidenceRoots<'a> {
    type Item = &'a [u8; PROV_DIGEST_LEN];

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let (chunk, rest) = self.bytes.split_at(PROV_DIGEST_LEN);
        self.bytes = rest;
        self.remaining -= 1;
        chunk.try_into().ok()
    }
}

fn read_u16(bytes: &[u8]) -> u16 {
    u16::from(bytes[0]) | (u16::from(bytes[1]) << 8)
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from(bytes[0])
        | (u32::from(bytes[1]) << 8)
        | (u32::from(bytes[2]) << 16)
        | (u32::from(bytes[3]) << 24)
}

/// Input to [`build`]: the optional identity digests, optional SPDX
/// license expression, and evidence-root κ set for one PROV/1 section.
/// Plain data — no wire-format knowledge required of callers.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProvComponents<'a> {
    pub source_manifest_kappa: Option<[u8; PROV_DIGEST_LEN]>,
    pub geometry_digest: Option<[u8; PROV_DIGEST_LEN]>,
    pub tokenizer_adapter_digest: Option<[u8; PROV_DIGEST_LEN]>,
    pub attention_operator_digest: Option<[u8; PROV_DIGEST_LEN]>,
    pub dense_operator_digest: Option<[u8; PROV_DIGEST_LEN]>,
    pub license: Option<&'a str>,
    /// Evidence-root κ entries. Order does not matter — [`build`] sorts
    /// them into canonical (ascending) order; an exact duplicate is
    /// rejected rather than silently collapsed, since a repeated root is
    /// more likely a caller bug than an intentional weight.
    pub evidence_roots: &'a [[u8; PROV_DIGEST_LEN]],
}

/// Canonically build one PROV/1 section's bytes from `components`.
///
/// Sorts `evidence_roots` into ascending order and rejects an exact
/// duplicate ([`FormatError::ProvEvidenceRootsNotSorted`]); rejects a
/// non-ASCII `license`. The result always round-trips through
/// [`Prov::parse`] to an equivalent view — asserted by this module's
/// tests.
#[cfg(feature = "alloc")]
pub fn build(components: &ProvComponents<'_>) -> Result<alloc::vec::Vec<u8>, NotAProduct> {
    use alloc::vec::Vec;

    if !components.license.is_none_or(|license| license.is_ascii()) {
        return Err(FormatError::ProvLicenseNotAscii.into());
    }
    let mut roots: Vec<[u8; PROV_DIGEST_LEN]> = components.evidence_roots.to_vec();
    roots.sort_unstable();
    if roots.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(FormatError::ProvEvidenceRootsNotSorted.into());
    }

    let mut presence = 0u8;
    let digests: [(u8, Option<[u8; PROV_DIGEST_LEN]>); 5] = [
        (
            PRESENCE_SOURCE_MANIFEST_KAPPA,
            components.source_manifest_kappa,
        ),
        (PRESENCE_GEOMETRY, components.geometry_digest),
        (
            PRESENCE_TOKENIZER_ADAPTER,
            components.tokenizer_adapter_digest,
        ),
        (
            PRESENCE_ATTENTION_OPERATOR,
            components.attention_operator_digest,
        ),
        (PRESENCE_DENSE_OPERATOR, components.dense_operator_digest),
    ];
    for (bit, value) in digests {
        if value.is_some() {
            presence |= bit;
        }
    }
    let license_bytes = components.license.unwrap_or("").as_bytes();
    if components.license.is_some() {
        presence |= PRESENCE_LICENSE;
    }

    let mut out =
        Vec::with_capacity(PROV_HEADER_LEN + license_bytes.len() + roots.len() * PROV_DIGEST_LEN);
    out.extend_from_slice(&PROV_MAGIC);
    out.extend_from_slice(&PROV_VERSION.to_le_bytes());
    out.push(presence);
    out.push(0); // reserved
    out.extend_from_slice(&(roots.len() as u32).to_le_bytes());
    out.extend_from_slice(&(license_bytes.len() as u32).to_le_bytes());
    for (_, value) in digests {
        out.extend_from_slice(&value.unwrap_or([0u8; PROV_DIGEST_LEN]));
    }
    out.extend_from_slice(license_bytes);
    for root in &roots {
        out.extend_from_slice(root);
    }
    Ok(out)
}
