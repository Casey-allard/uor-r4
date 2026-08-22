//! Fixed-width domain newtypes (PDF §20/§21 style).
//!
//! Every value crossing the serialization boundary uses one of these
//! fixed-width types — never `usize`, never platform-dependent layout
//! (RFC §1 rule 4). All are `#[repr(transparent)]` over their raw integer
//! or byte-array representation.

/// Region (graph node) identifier.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u32);

/// Offset relative to the start of its containing section (RFC §1 rule 3).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionOffset(pub u32);

/// Token identifier into the compiled vocabulary.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenId(pub u32);

/// Quantized fixed-point log-domain score.
///
/// Semantic format: one global `i32` Q16.16 declaration in HEAD
/// (RFC §9.3). This type is the raw carrier only; storage-time dyadic
/// descriptors (`{width, shift, zero_point}`) and shift+add decoding are
/// carried by the EMIT tables, not here.
#[repr(transparent)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ScoreQ(pub i32);

impl ScoreQ {
    /// Scale factor of the Q16.16 format (2^16).
    pub const SCALE: f32 = 65536.0;
    /// Additive identity.
    pub const ZERO: ScoreQ = ScoreQ(0);
    /// Minimum representable score.
    pub const MIN: ScoreQ = ScoreQ(i32::MIN);
    /// Maximum representable score.
    pub const MAX: ScoreQ = ScoreQ(i32::MAX);

    /// Wrap a raw Q16.16 bit pattern.
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    /// The raw Q16.16 bit pattern.
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Construct from a log-domain float (compiler-side convenience; std only
    /// — the deployed runtime never converts floats).
    /// NaN maps to ZERO; out-of-range values clamp.
    #[cfg(feature = "std")]
    pub fn from_logprob(lp: f32) -> Self {
        if lp.is_nan() {
            return Self::ZERO;
        }
        let scaled = (lp * Self::SCALE).round();
        ScoreQ(scaled.clamp(i32::MIN as f32, i32::MAX as f32) as i32)
    }

    /// Convert back to a log-domain float (compiler-side convenience; std only).
    #[cfg(feature = "std")]
    pub fn to_logprob(self) -> f32 {
        self.0 as f32 / Self::SCALE
    }

    /// Saturating addition (multiplication-free arithmetic).
    pub fn saturating_add(self, rhs: Self) -> Self {
        ScoreQ(self.0.saturating_add(rhs.0))
    }

    /// Saturating subtraction (multiplication-free arithmetic).
    pub fn saturating_sub(self, rhs: Self) -> Self {
        ScoreQ(self.0.saturating_sub(rhs.0))
    }
}

impl core::ops::Add for ScoreQ {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }
}

impl core::ops::AddAssign for ScoreQ {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.saturating_add(rhs);
    }
}

impl core::ops::Sub for ScoreQ {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
    }
}

impl core::ops::SubAssign for ScoreQ {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.saturating_sub(rhs);
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for ScoreQ {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ScoreQ({:.4})", self.to_logprob())
    }
}

/// Multiresolution depth of a region.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Depth(pub u8);

/// Calibrated acceptance radius of a region (masked-Hamming bound).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Radius(pub u16);

/// Content address (blake3 digest) of an artifact or artifact part.
///
/// CIDs preserve identity and provenance of bytes; they are not semantic
/// hashes and are never used as routing codes (GLOSSARY, "κ / content
/// CID").
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtifactCid(pub [u8; 32]);

/// Section identifier (RFC §3 inventory).
///
/// Known IDs `0x01..=0x0B` carry the RFC §3 "Mandatory" column. The RFC
/// does not specify how a reader classifies *unknown* IDs as mandatory or
/// optional; this crate resolves the ambiguity with a PNG-style
/// critical/ancillary bit (see [`SectionId::OPTIONAL_BIT`] and
/// [`SectionId::mandatory`]).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionId(pub u32);

impl SectionId {
    /// HEAD — identities and limits (mandatory).
    pub const HEAD: SectionId = SectionId(0x01);
    /// CODE — token codes, rolling-state programs, code layout (mandatory).
    pub const CODE: SectionId = SectionId(0x02);
    /// NODE — packed region records (mandatory).
    pub const NODE: SectionId = SectionId(0x03);
    /// EDGE — refinement/overlap/forward edges + reverse index (mandatory).
    pub const EDGE: SectionId = SectionId(0x04);
    /// ROUT — decision programs, prototypes, masks, shortlists (mandatory).
    pub const ROUT: SectionId = SectionId(0x05);
    /// EMIT — root priors, emission/transition residuals (mandatory).
    pub const EMIT: SectionId = SectionId(0x06);
    /// EXCT — exact-context residual evidence (optional).
    pub const EXCT: SectionId = SectionId(0x07);
    /// PROV — provenance roots (mandatory).
    pub const PROV: SectionId = SectionId(0x08);
    /// CERT — certification metadata (optional).
    pub const CERT: SectionId = SectionId(0x09);
    /// PTCH — patch-epoch header (optional, Phase 9).
    pub const PTCH: SectionId = SectionId(0x0A);
    /// SECT — per-section hash table (optional, reserved; RFC §9.2).
    pub const SECT: SectionId = SectionId(0x0B);
    /// RTNX — route translation index (optional, Phase 9).
    pub const RTNX: SectionId = SectionId(0x0C);
    /// NGRAM — packed bigram/trigram context rows (optional).
    pub const NGRAM: SectionId = SectionId(Self::OPTIONAL_BIT | 0x0E);
    /// FWDA — packed forward-anchor rows for infill serving (optional,
    /// issue #399). Absent section means the channel is off; every
    /// pre-FWDA artifact remains valid.
    pub const FWDA: SectionId = SectionId(Self::OPTIONAL_BIT | 0x0F);
    /// FMM — compiler-precomputed far-field translation table (optional).
    ///
    /// The optional bit is part of the wire ID so older readers can skip the
    /// section while preserving the R4G1 unknown-optional-section rule.
    pub const FMM: SectionId = SectionId(Self::OPTIONAL_BIT | 0x0D);
    /// PSTATE — packed persistent-prompt-state segment lane (optional,
    /// issue #836, lowering the #835 segment lane). Absent section, or a
    /// reader that does not consume it, behaves exactly as before
    /// (absent-section identity); every pre-PSTATE artifact remains valid.
    pub const PSTATE: SectionId = SectionId(Self::OPTIONAL_BIT | 0x10);
    /// SKMX — packed skip-conditioned residual joint table (optional, issue
    /// #897, lowering the D1-selected/#897-phase-0-confirmed 1-token
    /// skip-mix scorer). Absent section, or a reader that does not consume
    /// it, behaves exactly as before (absent-section identity).
    pub const SKMX: SectionId = SectionId(Self::OPTIONAL_BIT | 0x11);
    /// PSIB — packed unconditioned Ψ-bag fallback table for the #897
    /// skip-mix scorer (optional). Absent section, or a reader that does
    /// not consume it, behaves exactly as before (absent-section identity).
    pub const PSIB: SectionId = SectionId(Self::OPTIONAL_BIT | 0x12);
    /// PSCH — bounded-planning schema: slot shape, operator effect
    /// vocabulary, frozen capacities, and ordinal band thresholds
    /// (optional, issue #843). Absent section, or a reader that does not
    /// consume it, behaves exactly as before (absent-section identity).
    pub const PSCH: SectionId = SectionId(Self::OPTIONAL_BIT | 0x13);
    /// PTRN — packed transition rule table plus its operator index
    /// (optional, issue #843). Absent-section identity as above.
    pub const PTRN: SectionId = SectionId(Self::OPTIONAL_BIT | 0x14);
    /// PGOL — packed goal and forbidden-region predicates for one planning
    /// query (optional, issue #843). Absent-section identity as above.
    pub const PGOL: SectionId = SectionId(Self::OPTIONAL_BIT | 0x15);
    /// PWIT — versioned, self-contained plan witness (optional, issue
    /// #843). Absent-section identity as above.
    pub const PWIT: SectionId = SectionId(Self::OPTIONAL_BIT | 0x16);

    /// Ancillary bit classifying *unknown* section IDs.
    ///
    /// Version-policy resolution (RFC §1 rule 2 / §8, which require
    /// rejecting unknown mandatory sections while skipping unknown
    /// optional ones, without defining how to tell them apart): an
    /// unknown ID with bit 31 set is optional and skipped; an unknown ID
    /// with bit 31 clear is mandatory and rejected. Writers adding a new
    /// optional section in a minor version bump set this bit so older
    /// readers skip it.
    pub const OPTIONAL_BIT: u32 = 0x8000_0000;

    /// The raw wire value.
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// True when the ID is in the RFC §3 inventory or a known optional
    /// extension implemented by this reader.
    pub const fn is_known(self) -> bool {
        matches!(self.0, 0x01..=0x0C)
            || self.0 == Self::FMM.0
            || self.0 == Self::NGRAM.0
            || self.0 == Self::FWDA.0
            || self.0 == Self::PSTATE.0
            || self.0 == Self::SKMX.0
            || self.0 == Self::PSIB.0
            || self.0 == Self::PSCH.0
            || self.0 == Self::PTRN.0
            || self.0 == Self::PGOL.0
            || self.0 == Self::PWIT.0
    }

    /// Mandatory-ness per the RFC §3 column for known IDs.
    ///
    /// Unknown IDs default to mandatory unless [`SectionId::OPTIONAL_BIT`]
    /// is set (see the bit's documentation).
    pub const fn mandatory(self) -> bool {
        match self.0 {
            0x01..=0x06 | 0x08 => true,
            0x07 | 0x09..=0x0C => false,
            value if value == Self::NGRAM.0 => false,
            value if value == Self::FWDA.0 => false,
            value if value == Self::FMM.0 => false,
            value if value == Self::PSTATE.0 => false,
            value if value == Self::SKMX.0 => false,
            value if value == Self::PSIB.0 => false,
            value if value == Self::PSCH.0 => false,
            value if value == Self::PTRN.0 => false,
            value if value == Self::PGOL.0 => false,
            value if value == Self::PWIT.0 => false,
            _ => self.0 & Self::OPTIONAL_BIT == 0,
        }
    }
}
