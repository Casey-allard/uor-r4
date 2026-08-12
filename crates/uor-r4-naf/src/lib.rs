//! UOR-NAF v1 interchange slice for r4 (#623; campaign #589).
//!
//! Implements `uor-naf/1-draft.6` capabilities **core, integer, tensor,
//! address, address-binding(integer), address-binding(tensor)** — and nothing
//! else. Explicitly NOT implemented (deferred on #623 with reasons): the Atlas
//! word adapter, state/operator domains (Adapter C needs a canonicalizer
//! completeness proof), `uor-naf-plan` (no registered wire exists), and the
//! `execution`/`optimality` capability sets.
//!
//! # Dual-label discipline (binding, from #623)
//!
//! NAF κ labels are **sha256-only by normative definition** (§8.2: exactly 71
//! lowercase ASCII bytes, `sha256:` + 64 hex; "algorithm agility requires a
//! new outer version"). r4's internal store keys are and remain **blake3**.
//! The two must never be compared or substituted, so this crate's labels are
//! the dedicated type [`NafLabel`] and never plain `String`s; nothing here
//! accepts or emits a blake3 label. §2 of the spec requires exactly this
//! typed separation of commitment kinds.
//!
//! # Provisional identity (recorded)
//!
//! `uor-naf/1-draft.6` is baked into every manifest preimage, so **every κ
//! this crate produces changes when the spec freezes to `uor-naf/1`** (§14
//! mandates full regeneration). Do not pin NAF κ into long-lived artifacts.
//!
//! # Outcome taxonomy
//!
//! [`NafError`] carries the spec's six-state partition (NAF-DOM-006) plus the
//! two refusals the spec keeps distinct from malformedness: resource refusal
//! and policy-out-of-domain (§8.3). An invalid object is never relabeled as a
//! refusal, and vice versa.
//!
//! The GNAF claim-class vocabulary adopted alongside this slice lives in
//! [`claims`].

use sha2::{Digest, Sha256};

pub mod claims;

/// The magic + version + reserved prefix of semantic payloads (§8.1).
pub const UORSEM: [u8; 8] = *b"UORSEM\x01\x00";
/// The magic + version + reserved prefix of artifact payloads (§7.3).
pub const UORNAF: [u8; 8] = *b"UORNAF\x01\x00";
/// The provisional spec identifier — part of every manifest preimage.
pub const SPEC_ID: &str = "uor-naf/1-draft.6";
/// The v1 integer coefficient domain (§4.1, fixed string).
pub const COEFF_DOMAIN: &str = "uor-naf/integer-z/1";

/// The closed v1 storage-profile list (§4.1). Any other value is invalid
/// rather than unresolved.
pub const STORAGE_PROFILES: [&str; 5] = [
    "uor-naf/math-int/1",
    "uor-naf/twos-i8/1",
    "uor-naf/twos-i16/1",
    "uor-naf/twos-i32/1",
    "uor-naf/twos-i64/1",
];

/// §8.3 / NAF-DOM-006: the outcome partition. `Accepted` is the `Ok` arm of
/// this crate's `Result`s; the rest are here. None substitutes for another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NafError {
    /// Malformed or noncanonical bytes / values (with the rejection tag from
    /// the §12.4 corpus where one is defined).
    Invalid(&'static str),
    /// Registry or dependency content is unavailable — NOT malformedness.
    Unresolved,
    /// A resolved, valid domain whose declared operations this slice does not
    /// implement (e.g. atlas-word, state, operator tags).
    Unsupported,
    /// Implemented and valid, but lacking an admitted required warrant.
    Unadmitted,
    /// Observed distinct preimages for one typed commitment — fatal; never
    /// resolved by digest-based selection (§7.4).
    CommitmentFailure,
    /// A declared local resource bound refused the operation. Distinct from
    /// `Invalid` by normative requirement (§7.1/§8.3).
    ResourceRefusal,
    /// Valid object outside a local artifact policy — distinct from both
    /// `Invalid` and `ResourceRefusal` (§8.3).
    PolicyOutOfDomain,
}

pub type Result<T> = std::result::Result<T, NafError>;

// ---------------------------------------------------------------------------
// Arbitrary-precision magnitude (the all-integer profile MUST use
// arbitrary-precision state, §3.3). Little-endian u32 limbs, no leading zero
// limb; only the operations normalization and the grammars need.
// ---------------------------------------------------------------------------

/// A nonnegative arbitrary-precision integer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Nat(Vec<u32>);

impl Nat {
    pub fn zero() -> Self {
        Nat(Vec::new())
    }
    pub fn from_u128(mut v: u128) -> Self {
        let mut limbs = Vec::new();
        while v != 0 {
            limbs.push((v & 0xffff_ffff) as u32);
            v >>= 32;
        }
        Nat(limbs)
    }
    pub fn is_zero(&self) -> bool {
        self.0.is_empty()
    }
    fn low_bits(&self, k: u32) -> u32 {
        let mask = (1u64 << k) - 1;
        (*self.0.first().unwrap_or(&0) as u64 & mask) as u32
    }
    fn trim(&mut self) {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
    }
    /// `self += 1`.
    fn inc(&mut self) {
        for limb in &mut self.0 {
            let (v, carry) = limb.overflowing_add(1);
            *limb = v;
            if !carry {
                return;
            }
        }
        self.0.push(1);
    }
    /// `self -= 1`; caller guarantees nonzero.
    fn dec(&mut self) {
        for limb in &mut self.0 {
            let (v, borrow) = limb.overflowing_sub(1);
            *limb = v;
            if !borrow {
                break;
            }
        }
        self.trim();
    }
    /// `self >>= 1`; caller guarantees evenness where exactness is required.
    fn shr1(&mut self) {
        let mut carry = 0u32;
        for limb in self.0.iter_mut().rev() {
            let new_carry = *limb & 1;
            *limb = (*limb >> 1) | (carry << 31);
            carry = new_carry;
        }
        self.trim();
    }
    /// Minimal LEB128 (`uvar`, §7.1) of this magnitude.
    pub fn to_uvar(&self) -> Vec<u8> {
        if self.is_zero() {
            return vec![0x00];
        }
        let mut bits: Vec<u8> = Vec::new();
        let mut n = self.clone();
        while !n.is_zero() {
            bits.push((n.low_bits(7)) as u8);
            for _ in 0..7 {
                n.shr1();
            }
        }
        let last = bits.len() - 1;
        for (i, b) in bits.iter_mut().enumerate() {
            if i != last {
                *b |= 0x80;
            }
        }
        bits
    }
}

// ---------------------------------------------------------------------------
// uvar (§7.1) — strict minimal LEB128 decode.
// ---------------------------------------------------------------------------

/// Decode one minimal `uvar` from the front of `input`; returns (value,
/// consumed). Rejects truncation and nonminimal spellings (§12.4 tags).
pub fn uvar_decode(input: &[u8]) -> Result<(Nat, usize)> {
    let mut limbs = Nat::zero();
    for (i, &b) in input.iter().enumerate() {
        // Accumulate payload bits: value |= (b & 0x7f) << (7 * i).
        let payload = b & 0x7f;
        if payload != 0 {
            let mut chunk = Nat::from_u128(payload as u128);
            for _ in 0..(7 * i) {
                chunk = {
                    // shl1 via add-to-self.
                    let mut c = chunk.clone();
                    let mut carry = 0u32;
                    for limb in &mut c.0 {
                        let v = (*limb as u64) << 1 | carry as u64;
                        *limb = v as u32;
                        carry = (v >> 32) as u32;
                    }
                    if carry != 0 {
                        c.0.push(carry);
                    }
                    c
                };
            }
            // limbs += chunk
            let mut carry = 0u64;
            for (j, &cl) in chunk.0.iter().enumerate() {
                while limbs.0.len() <= j {
                    limbs.0.push(0);
                }
                let v = limbs.0[j] as u64 + cl as u64 + carry;
                limbs.0[j] = v as u32;
                carry = v >> 32;
            }
            let mut j = chunk.0.len();
            while carry != 0 {
                while limbs.0.len() <= j {
                    limbs.0.push(0);
                }
                let v = limbs.0[j] as u64 + carry;
                limbs.0[j] = v as u32;
                carry = v >> 32;
                j += 1;
            }
        }
        if b & 0x80 == 0 {
            // Final byte: minimality — a multi-byte spelling whose final
            // payload group is zero is forbidden.
            if i > 0 && payload == 0 {
                return Err(NafError::Invalid("nonminimal-uvar"));
            }
            return Ok((limbs, i + 1));
        }
    }
    Err(NafError::Invalid("truncated-uvar"))
}

/// Decode a `uvar` that must fit a machine word (lengths, ranks, extents).
/// A value beyond `usize` is a frame-impossible count on any real input.
fn uvar_decode_usize(input: &[u8]) -> Result<(usize, usize)> {
    let (n, used) = uvar_decode(input)?;
    if n.0.len() > (usize::BITS as usize).div_ceil(32) {
        return Err(NafError::Invalid("frame-impossible-count"));
    }
    let mut v: u128 = 0;
    for (i, &l) in n.0.iter().enumerate() {
        v |= (l as u128) << (32 * i);
    }
    usize::try_from(v)
        .map(|u| (u, used))
        .map_err(|_| NafError::Invalid("frame-impossible-count"))
}

// ---------------------------------------------------------------------------
// Core NAF (§3) — digits, normalization, packing.
// ---------------------------------------------------------------------------

/// A signed arbitrary-precision integer as (negative?, magnitude).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Int {
    pub negative: bool,
    pub magnitude: Nat,
}

impl Int {
    pub fn from_i128(v: i128) -> Self {
        Int {
            negative: v < 0,
            magnitude: Nat::from_u128(v.unsigned_abs()),
        }
    }
    pub fn is_zero(&self) -> bool {
        self.magnitude.is_zero()
    }
}

/// `normalize_integer` (§3.3), total over all integers, arbitrary-precision.
/// Digits are LSB-first in `{-1, 0, +1}`. Negation law: the negative case is
/// the digitwise negation of the positive case, so magnitude arithmetic
/// suffices and no `abs(MIN)` hazard exists.
pub fn normalize_integer(n: &Int) -> Vec<i8> {
    let mut m = n.magnitude.clone();
    let mut digits: Vec<i8> = Vec::new();
    while !m.is_zero() {
        let d: i8 = if m.low_bits(1) == 0 {
            0
        } else if m.low_bits(2) == 1 {
            1
        } else {
            -1
        };
        digits.push(d);
        // m := (m - d) / 2 on the nonnegative magnitude.
        match d {
            1 => m.dec(),
            -1 => m.inc(),
            _ => {}
        }
        m.shr1();
    }
    if n.negative {
        for d in &mut digits {
            *d = -*d;
        }
    }
    digits
}

/// `eval` of a digit sequence (§3.1) — for law tests and decoders.
pub fn eval_digits(digits: &[i8]) -> Int {
    let mut mag = Nat::zero();
    let mut negative = false;
    // Evaluate as sum of signed powers via two magnitudes then subtract —
    // simpler: track (pos - neg) exactly using i128 chunks is not total, so
    // fold from the top: value = value*2 + d, on a signed (neg, mag) pair.
    for &d in digits.iter().rev() {
        // value *= 2
        let mut carry = 0u32;
        for limb in &mut mag.0 {
            let v = (*limb as u64) << 1 | carry as u64;
            *limb = v as u32;
            carry = (v >> 32) as u32;
        }
        if carry != 0 {
            mag.0.push(carry);
        }
        // value += d (signed)
        match (d, negative, mag.is_zero()) {
            (0, _, _) => {}
            (1, false, _) | (1, true, false) => {
                if negative {
                    mag.dec();
                    if mag.is_zero() {
                        negative = false;
                    }
                } else {
                    mag.inc();
                }
            }
            (1, true, true) => {
                negative = false;
                mag.inc();
            }
            (-1, false, true) => {
                negative = true;
                mag.inc();
            }
            (-1, false, false) => {
                mag.dec();
            }
            (-1, true, _) => {
                mag.inc();
            }
            _ => unreachable!("digits are in {{-1,0,1}}"),
        }
    }
    Int {
        negative: negative && !mag.is_zero(),
        magnitude: mag,
    }
}

/// The §3.2 normal-form predicate.
pub fn is_normal(digits: &[i8]) -> bool {
    digits.iter().all(|d| (-1..=1).contains(d))
        && digits.windows(2).all(|w| w[0] * w[1] == 0)
        && digits.last().map(|&d| d != 0).unwrap_or(true)
}

/// Encode `CoreNAFBytes` (§7.2): `uvar(ell) || packed`, two bits per digit.
pub fn core_naf_encode(digits: &[i8]) -> Vec<u8> {
    let mut out = Nat::from_u128(digits.len() as u128).to_uvar();
    let mut byte = 0u8;
    for (i, &d) in digits.iter().enumerate() {
        let code: u8 = match d {
            0 => 0b00,
            1 => 0b01,
            -1 => 0b10,
            _ => unreachable!("digits are in {{-1,0,1}}"),
        };
        byte |= code << ((i % 4) * 2);
        if i % 4 == 3 {
            out.push(byte);
            byte = 0;
        }
    }
    if !digits.len().is_multiple_of(4) {
        out.push(byte);
    }
    out
}

/// Strict prefix decode of `CoreNAFBytes` (§7.2): returns (digits, consumed).
/// Enforces every §7.2 rejection: digit code `11`, adjacency, zero top digit,
/// truncation, nonzero padding, nonminimal `uvar`. Complete consumption is
/// the caller's frame decision (`trailing-bytes` at the outer boundary).
pub fn core_naf_decode(input: &[u8]) -> Result<(Vec<i8>, usize)> {
    let (ell, used) = uvar_decode_usize(input)?;
    let payload_len = ell.div_ceil(4);
    let rest = &input[used..];
    if rest.len() < payload_len {
        return Err(NafError::Invalid("truncated-core-naf"));
    }
    let mut digits = Vec::with_capacity(ell);
    for i in 0..ell {
        let byte = rest[i / 4];
        let code = (byte >> ((i % 4) * 2)) & 0b11;
        let d = match code {
            0b00 => 0i8,
            0b01 => 1,
            0b10 => -1,
            _ => return Err(NafError::Invalid("invalid-digit-code")),
        };
        if let Some(&prev) = digits.last() {
            if prev != 0 && d != 0 {
                return Err(NafError::Invalid("adjacent-nonzero"));
            }
        }
        digits.push(d);
    }
    if ell > 0 {
        if digits[ell - 1] == 0 {
            return Err(NafError::Invalid("zero-top-digit"));
        }
        // Nonzero final padding: bits above position ell in the last byte.
        let last = rest[payload_len - 1];
        let used_fields = ell - (payload_len - 1) * 4;
        if used_fields < 4 && (last >> (used_fields * 2)) != 0 {
            return Err(NafError::Invalid("nonzero-padding"));
        }
    }
    Ok((digits, used + payload_len))
}

// ---------------------------------------------------------------------------
// Grammars: bytes(...), signed integer primitive, semantic + artifact
// payloads for integer and tensor (§7.3, §8.1).
// ---------------------------------------------------------------------------

fn put_bytes(out: &mut Vec<u8>, field: &[u8]) {
    out.extend_from_slice(&Nat::from_u128(field.len() as u128).to_uvar());
    out.extend_from_slice(field);
}

fn get_bytes(input: &[u8]) -> Result<(&[u8], usize)> {
    let (len, used) = uvar_decode_usize(input)?;
    let rest = &input[used..];
    if rest.len() < len {
        return Err(NafError::Invalid("truncated-bytes"));
    }
    Ok((&rest[..len], used + len))
}

/// Encode the §8.1 canonical signed integer primitive.
fn put_signed(out: &mut Vec<u8>, v: &Int) {
    if v.is_zero() {
        out.push(0x00);
    } else {
        out.push(if v.negative { 0x02 } else { 0x01 });
        out.extend_from_slice(&v.magnitude.to_uvar());
    }
}

/// Strict decode of the §8.1 signed integer primitive: (value, consumed).
pub fn signed_decode(input: &[u8]) -> Result<(Int, usize)> {
    match input.first() {
        None => Err(NafError::Invalid("truncated-signed-integer")),
        Some(0x00) => Ok((Int::from_i128(0), 1)),
        Some(&code @ (0x01 | 0x02)) => {
            let (mag, used) = uvar_decode(&input[1..])?;
            if mag.is_zero() {
                return Err(NafError::Invalid("zero-magnitude-with-sign"));
            }
            Ok((
                Int {
                    negative: code == 0x02,
                    magnitude: mag,
                },
                1 + used,
            ))
        }
        Some(_) => Err(NafError::Invalid("invalid-sign-code")),
    }
}

/// A NAF value in this slice's domain: a mathematical integer or a row-major
/// integer tensor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NafValue {
    Integer(Int),
    /// Shape extents and `product(shape)` coefficients, last axis fastest.
    Tensor {
        shape: Vec<usize>,
        values: Vec<Int>,
    },
}

impl NafValue {
    fn domain_tag(&self) -> u8 {
        match self {
            NafValue::Integer(_) => 0x01,
            NafValue::Tensor { .. } => 0x02,
        }
    }
    fn domain_name(&self) -> &'static str {
        match self {
            NafValue::Integer(_) => "integer",
            NafValue::Tensor { .. } => "tensor",
        }
    }
}

/// The storage-profile range check (§7.3: every exactly decoded coefficient
/// MUST satisfy its declared range).
fn profile_admits(profile: &str, v: &Int) -> bool {
    let bound = |bits: u32| {
        // |v| <= 2^(bits-1) (negative) / 2^(bits-1)-1 (nonnegative).
        let limit = 1u128 << (bits - 1);
        let mut mag: u128 = 0;
        if v.magnitude.0.len() > 4 {
            return false;
        }
        for (i, &l) in v.magnitude.0.iter().enumerate() {
            mag |= (l as u128) << (32 * i);
        }
        if v.negative {
            mag <= limit
        } else {
            mag < limit
        }
    };
    match profile {
        "uor-naf/math-int/1" => true,
        "uor-naf/twos-i8/1" => bound(8),
        "uor-naf/twos-i16/1" => bound(16),
        "uor-naf/twos-i32/1" => bound(32),
        "uor-naf/twos-i64/1" => bound(64),
        _ => false,
    }
}

/// Build the exact semantic payload bytes (§8.1). Storage profile is absent
/// by design: same mathematical value ⇒ same semantic payload ⇒ same
/// semantic κ, across every storage profile.
pub fn semantic_payload(value: &NafValue) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&UORSEM);
    out.push(value.domain_tag());
    put_bytes(&mut out, COEFF_DOMAIN.as_bytes());
    match value {
        NafValue::Integer(v) => put_signed(&mut out, v),
        NafValue::Tensor { shape, values } => {
            out.extend_from_slice(&Nat::from_u128(shape.len() as u128).to_uvar());
            for &s in shape {
                out.extend_from_slice(&Nat::from_u128(s as u128).to_uvar());
            }
            out.push(0x01); // row-major is the only v1 order
            for v in values {
                put_signed(&mut out, v);
            }
        }
    }
    out
}

/// Build the exact artifact payload bytes (§7.3) for a storage profile in the
/// closed v1 list, embedding the semantic κ computed from the semantic
/// payload. Range-checks every coefficient against the profile.
pub fn artifact_payload(value: &NafValue, storage_profile: &str) -> Result<Vec<u8>> {
    if !STORAGE_PROFILES.contains(&storage_profile) {
        return Err(NafError::Invalid("unknown-storage-profile"));
    }
    let admit = |v: &Int| {
        if profile_admits(storage_profile, v) {
            Ok(())
        } else {
            Err(NafError::Invalid("storage-range-violation"))
        }
    };
    let semantic_kappa = labels(value).semantic_kappa;
    let mut out = Vec::new();
    out.extend_from_slice(&UORNAF);
    out.push(value.domain_tag());
    put_bytes(&mut out, semantic_kappa.as_bytes());
    put_bytes(&mut out, COEFF_DOMAIN.as_bytes());
    put_bytes(&mut out, storage_profile.as_bytes());
    match value {
        NafValue::Integer(v) => {
            admit(v)?;
            out.extend_from_slice(&core_naf_encode(&normalize_integer(v)));
        }
        NafValue::Tensor { shape, values } => {
            let expect: usize = shape.iter().try_fold(1usize, |acc, &s| {
                acc.checked_mul(s)
                    .ok_or(NafError::Invalid("frame-impossible-count"))
            })?;
            if values.len() != expect {
                return Err(NafError::Invalid("shape-count-mismatch"));
            }
            out.extend_from_slice(&Nat::from_u128(shape.len() as u128).to_uvar());
            for &s in shape {
                out.extend_from_slice(&Nat::from_u128(s as u128).to_uvar());
            }
            out.push(0x01);
            for v in values {
                admit(v)?;
                // NAF adjacency resets at every coefficient boundary (§4).
                out.extend_from_slice(&core_naf_encode(&normalize_integer(v)));
            }
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Labels (§8.2) — sha256-only by normative definition; typed.
// ---------------------------------------------------------------------------

/// A NAF κ label: exactly `sha256:` + 64 lowercase hex. Deliberately NOT a
/// plain string and NOT constructible from one: r4's blake3 labels can never
/// flow into this type (§2 commitment-type separation; #623 dual-label rule).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NafLabel(String);

impl NafLabel {
    fn of(preimage: &[u8]) -> Self {
        let mut h = Sha256::new();
        h.update(preimage);
        NafLabel(format!("sha256:{:x}", h.finalize()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// The §8.2 label chain for one value: manifests are exact one-line JSON in
/// RFC 8785 key order, payload length as a JSON *string*.
#[derive(Debug)]
pub struct LabelChain {
    pub semantic_payload: Vec<u8>,
    pub semantic_manifest: String,
    pub payload_sha256: NafLabel,
    pub semantic_kappa: NafLabel,
}

/// Compute the semantic half of the chain.
pub fn labels(value: &NafValue) -> LabelChain {
    let payload = semantic_payload(value);
    let payload_sha256 = NafLabel::of(&payload);
    let manifest = format!(
        "{{\"domain\":\"{}\",\"kind\":\"semantic\",\"payload_bytes\":\"{}\",\"payload_sha256\":\"{}\",\"spec\":\"{}\"}}",
        value.domain_name(),
        payload.len(),
        payload_sha256.as_str(),
        SPEC_ID
    );
    let semantic_kappa = NafLabel::of(manifest.as_bytes());
    LabelChain {
        semantic_payload: payload,
        semantic_manifest: manifest,
        payload_sha256,
        semantic_kappa,
    }
}

/// The artifact half: manifest + κ over exact artifact payload bytes,
/// embedding the semantic κ.
#[derive(Debug)]
pub struct ArtifactChain {
    pub artifact_payload: Vec<u8>,
    pub artifact_manifest: String,
    pub payload_sha256: NafLabel,
    pub artifact_kappa: NafLabel,
}

pub fn artifact_labels(value: &NafValue, storage_profile: &str) -> Result<ArtifactChain> {
    let semantic = labels(value);
    let payload = artifact_payload(value, storage_profile)?;
    let payload_sha256 = NafLabel::of(&payload);
    let manifest = format!(
        "{{\"domain\":\"{}\",\"kind\":\"artifact\",\"payload_bytes\":\"{}\",\"payload_sha256\":\"{}\",\"semantic_kappa\":\"{}\",\"spec\":\"{}\"}}",
        value.domain_name(),
        payload.len(),
        payload_sha256.as_str(),
        semantic.semantic_kappa.as_str(),
        SPEC_ID
    );
    let artifact_kappa = NafLabel::of(manifest.as_bytes());
    Ok(ArtifactChain {
        artifact_payload: payload,
        artifact_manifest: manifest,
        payload_sha256,
        artifact_kappa,
    })
}

// ---------------------------------------------------------------------------
// Strict artifact decoding (§7.3): full reconstruction + label verification.
// ---------------------------------------------------------------------------

/// Decode + verify a complete v1 artifact payload for this slice's domains.
/// Reconstructs the semantic payload, recomputes the semantic label, and
/// requires byte-equality with the embedded `semantic_kappa` (§7.3). Atlas /
/// state / operator tags return [`NafError::Unsupported`] — valid domains
/// this slice does not implement, never relabeled `Invalid`.
pub fn decode_artifact(bytes: &[u8]) -> Result<(NafValue, String)> {
    if bytes.len() < 9 || bytes[..8] != UORNAF {
        return Err(NafError::Invalid("bad-magic"));
    }
    let tag = bytes[8];
    let mut at = 9usize;
    let (kappa_field, used) = get_bytes(&bytes[at..])?;
    if kappa_field.len() != 71
        || !kappa_field.starts_with(b"sha256:")
        || !kappa_field[7..]
            .iter()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b))
    {
        return Err(NafError::Invalid("bad-semantic-kappa-field"));
    }
    at += used;
    match tag {
        0x01 | 0x02 => {}
        0x03..=0x05 => return Err(NafError::Unsupported),
        _ => return Err(NafError::Invalid("unknown-domain-tag")),
    }
    let (domain, used) = get_bytes(&bytes[at..])?;
    if domain != COEFF_DOMAIN.as_bytes() {
        return Err(NafError::Invalid("bad-coefficient-domain"));
    }
    at += used;
    let (profile, used) = get_bytes(&bytes[at..])?;
    let profile = std::str::from_utf8(profile)
        .map_err(|_| NafError::Invalid("bad-storage-profile"))?
        .to_string();
    if !STORAGE_PROFILES.contains(&profile.as_str()) {
        return Err(NafError::Invalid("unknown-storage-profile"));
    }
    at += used;

    let value = match tag {
        0x01 => {
            let (digits, used) = core_naf_decode(&bytes[at..])?;
            at += used;
            if !is_normal(&digits) {
                return Err(NafError::Invalid("noncanonical-naf"));
            }
            NafValue::Integer(eval_digits(&digits))
        }
        _ => {
            let (rank, used) = uvar_decode_usize(&bytes[at..])?;
            at += used;
            let mut shape = Vec::with_capacity(rank.min(64));
            for _ in 0..rank {
                let (s, used) = uvar_decode_usize(&bytes[at..])?;
                shape.push(s);
                at += used;
            }
            if bytes.get(at) != Some(&0x01) {
                return Err(NafError::Invalid("unknown-order-code"));
            }
            at += 1;
            let count = shape.iter().try_fold(1usize, |acc, &s| {
                acc.checked_mul(s)
                    .ok_or(NafError::Invalid("frame-impossible-count"))
            })?;
            // Frame bound before the loop: each CoreNAFBytes is ≥ 1 byte.
            if count > bytes.len() - at {
                return Err(NafError::Invalid("frame-impossible-count"));
            }
            let mut values = Vec::with_capacity(count);
            for _ in 0..count {
                let (digits, used) = core_naf_decode(&bytes[at..])?;
                at += used;
                if !is_normal(&digits) {
                    return Err(NafError::Invalid("noncanonical-naf"));
                }
                values.push(eval_digits(&digits));
            }
            NafValue::Tensor { shape, values }
        }
    };
    if at != bytes.len() {
        return Err(NafError::Invalid("trailing-bytes"));
    }
    // Range re-check against the declared profile.
    let admits = match &value {
        NafValue::Integer(v) => profile_admits(&profile, v),
        NafValue::Tensor { values, .. } => values.iter().all(|v| profile_admits(&profile, v)),
    };
    if !admits {
        return Err(NafError::Invalid("storage-range-violation"));
    }
    // Reconstruct + verify the embedded semantic κ (§7.3, mandatory).
    let chain = labels(&value);
    if chain.semantic_kappa.as_bytes() != kappa_field {
        return Err(NafError::CommitmentFailure);
    }
    Ok((value, profile))
}
