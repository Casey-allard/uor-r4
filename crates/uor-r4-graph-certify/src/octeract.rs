//! Finite Octeract byte-map conformance primitives (#661/#720).
//!
//! This module owns only the independently executable byte algebra needed by
//! the #661 research screen. It is certifier-side, referenced by no serving
//! path, and does not define or mutate an attention operator. In particular,
//! the five published anchor integers are labels of equivalence classes, not
//! similarity scores.

/// Provenance fields for one supplied research input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceIdentity {
    /// Document title embedded in the supplied PDF metadata.
    pub metadata_title: &'static str,
    /// Title displayed on the first page, normalized to one line.
    pub displayed_title: &'static str,
    /// Local filename under which the input was supplied.
    pub filename: &'static str,
    /// Exact file length in bytes at the #720 source audit.
    pub byte_len: u64,
    /// Lowercase SHA-256 of the exact supplied bytes.
    pub sha256: &'static str,
    /// SPDX-style license audit result. No license was found in either input.
    pub license: &'static str,
    /// Redistribution status derived from the absent license grant.
    pub redistribution: &'static str,
    /// Provenance known to the repository.
    pub provenance: &'static str,
    /// Date on which the local bytes and metadata were audited.
    pub audited_on: &'static str,
}

/// The primary Octeract paper supplied to #661.
pub const OCTERACT_CYPHER_SOURCE: SourceIdentity = SourceIdentity {
    metadata_title: "The Octeract Cypher: Coarse-Graining and Categorical Adjunctions in 8-Bit Phase Space Collapses",
    displayed_title: "The Octeract Cypher: Coarse-Graining and Categorical Adjunctions in 8-Bit Phase Space Collapses",
    filename: "Octeract_Cypher_Paper.pdf",
    byte_len: 54_969,
    sha256: "44bab09a20253437aeef43057ae316fcded5b00fd9f6b180f83843f06d2bbb3c",
    license: "NOASSERTION",
    redistribution: "not-authorized",
    provenance: "supplied-out-of-band-to-repository-maintainer",
    audited_on: "2026-08-14",
};

/// The validation/formalization roadmap supplied to #661.
pub const OCTERACT_VALIDATION_SOURCE: SourceIdentity = SourceIdentity {
    metadata_title: "Validating Octeract Cypher Mathlib",
    displayed_title: "Formal Validation and Mechanized Verification of the Octeract Cypher: A Base-2 Kaprekar Adjunction Framework",
    filename: "Validating Octeract Cypher Mathlib.pdf",
    byte_len: 262_762,
    sha256: "5322c519fa872ca836e2ad23d523ecf655defedd3dd17589ba290dec62a93a5e",
    license: "NOASSERTION",
    redistribution: "not-authorized",
    provenance: "supplied-out-of-band-to-repository-maintainer",
    audited_on: "2026-08-14",
};

/// The five outputs of the 8-bit sort/subtract map, indexed by canonical
/// folded shell `min(k, 8-k)`.
pub const OCTERACT_ANCHORS: [u8; 5] = [0, 127, 189, 217, 225];

/// Hamming weight of one byte. Values outside `0..=8` do not construct this
/// object, so the closed-form operation itself is total (R5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteWeight(u8);

impl ByteWeight {
    /// Construct a byte weight exactly when `value` is in `0..=8`.
    pub const fn new(value: u8) -> Option<Self> {
        if value <= 8 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Return the validated weight.
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Hamming distance together with the caller-declared active width of one
/// byte block. Invalid pairs do not construct this object, leaving folding
/// total over every admitted instantiation (R5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockDistance {
    distance: u8,
    active_bits: u8,
}

impl BlockDistance {
    /// Construct a bounded block distance when `distance <= active_bits <= 8`.
    pub const fn new(distance: u8, active_bits: u8) -> Option<Self> {
        if active_bits <= 8 && distance <= active_bits {
            Some(Self {
                distance,
                active_bits,
            })
        } else {
            None
        }
    }

    /// Return the exact Hamming distance.
    pub const fn distance(self) -> u8 {
        self.distance
    }

    /// Return the caller-declared number of active bits.
    pub const fn active_bits(self) -> u8 {
        self.active_bits
    }
}

/// One of the five folded shells of a complete byte. Values outside `0..=4`
/// do not construct this object, so anchor lookup is total (R5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FullByteShell(u8);

impl FullByteShell {
    /// Construct a full-byte folded shell exactly when `value` is in `0..=4`.
    pub const fn new(value: u8) -> Option<Self> {
        if value <= 4 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Return the folded shell index.
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Canonical folded class together with the orientation needed to recover the
/// unfurled Hamming distance exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrientedClass {
    /// `min(distance, active_bits - distance)`.
    shell: u8,
    /// `true` exactly when `distance > active_bits / 2`.
    high_side: bool,
    /// Number of active bits in the block.
    active_bits: u8,
}

impl OrientedClass {
    /// Construct a canonical oriented class. A high-side spelling of an
    /// even-width equator is non-canonical and therefore does not construct.
    pub const fn new(shell: u8, high_side: bool, active_bits: u8) -> Option<Self> {
        if active_bits > 8 || shell > active_bits / 2 {
            return None;
        }
        if high_side && active_bits.is_multiple_of(2) && shell == active_bits / 2 {
            return None;
        }
        Some(Self {
            shell,
            high_side,
            active_bits,
        })
    }

    /// Return the folded shell.
    pub const fn shell(self) -> u8 {
        self.shell
    }

    /// Return whether this is the high-distance member of the folded pair.
    pub const fn high_side(self) -> bool {
        self.high_side
    }

    /// Return the caller-declared active width.
    pub const fn active_bits(self) -> u8 {
        self.active_bits
    }
}

/// Direct oracle from the defining construction: split a byte into eight
/// binary digits, sort the digits, interpret the ascending and descending
/// arrangements, and subtract. This intentionally does not call
/// [`octeract_closed_form`] or `count_ones`.
pub fn octeract_sort_subtract(value: u8) -> u8 {
    let mut digits = [0u8; 8];
    for (index, digit) in digits.iter_mut().enumerate() {
        *digit = (value >> index) & 1;
    }
    digits.sort_unstable();

    let mut ascending = 0u8;
    let mut descending = 0u8;
    for &digit in &digits {
        ascending = (ascending << 1) | digit;
    }
    for &digit in digits.iter().rev() {
        descending = (descending << 1) | digit;
    }
    descending - ascending
}

/// Closed-form 8-bit map evaluated from a bounded Hamming weight:
/// `257 - (2^(8-k) + 2^k)`.
pub fn octeract_closed_form_from_weight(weight: ByteWeight) -> u8 {
    let weight = weight.get();
    let output = 257u16 - (1u16 << (8 - weight)) - (1u16 << weight);
    output as u8
}

/// Closed-form 8-bit map evaluated from the input byte's Hamming weight.
pub fn octeract_closed_form(value: u8) -> u8 {
    // Every u8 weight is in 0..=8, so this expression is total without a
    // recoverable error branch.
    let weight = value.count_ones();
    let output = 257u16 - (1u16 << (8 - weight)) - (1u16 << weight);
    output as u8
}

/// Fold a bounded block distance through complement symmetry.
pub fn folded_class(block: BlockDistance) -> u8 {
    block.distance.min(block.active_bits - block.distance)
}

/// Fold a bounded block distance while retaining the orientation that makes
/// the representation lossless with respect to ordinary block weight.
pub fn oriented_class(block: BlockDistance) -> OrientedClass {
    OrientedClass {
        shell: folded_class(block),
        high_side: block.distance > block.active_bits / 2,
        active_bits: block.active_bits,
    }
}

/// Recover the exact distance from a canonical oriented class.
pub fn distance_from_oriented(class: OrientedClass) -> u8 {
    if class.high_side {
        class.active_bits - class.shell
    } else {
        class.shell
    }
}

/// Map one full-byte folded shell to the paper's published anchor label.
pub fn anchor_for_shell(shell: FullByteShell) -> u8 {
    OCTERACT_ANCHORS[shell.get() as usize]
}

/// Masked byte Hamming distance used by the current route relation.
pub fn masked_byte_distance(query: u8, key: u8, mask: u8) -> u8 {
    ((query ^ key) & mask).count_ones() as u8
}

/// Safe weight lower bound for masked byte Hamming distance:
/// `abs(wt(q AND m) - wt(k AND m)) <= d_H(q,k;m)`.
pub fn masked_weight_lower_bound(query: u8, key: u8, mask: u8) -> u8 {
    let query_weight = (query & mask).count_ones() as u8;
    let key_weight = (key & mask).count_ones() as u8;
    query_weight.abs_diff(key_weight)
}
