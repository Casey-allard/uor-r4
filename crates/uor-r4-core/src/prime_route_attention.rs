//! Source-free fixed-zeta prime-route attention substrate (ADR-0003 / #958).
//!
//! This module owns only the first algebraic and compiled-address slice:
//! typed identity/null bridging, factor-preserving prime routes, quantized
//! spin/torsion state, canonical manifests, and bounded direct indexes. It
//! deliberately has no source-model, legacy-router, or CLI dependency.

use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU16, NonZeroUsize};

use serde::{Deserialize, Serialize};

use crate::wrap_to_pi;
use crate::zeta_zeros::ZETA_ZEROS;

pub const PRIME_ROUTE_MANIFEST_SCHEMA: u32 = 2;
pub const PRIME_ROUTE_MANIFEST_DOMAIN: &str = "uor-r4.prime-route-spin-manifest/2";
pub const PRIME_REGISTRY_SCHEMA: u32 = 1;
pub const PRIME_REGISTRY_DOMAIN: &str = "uor-r4.prime-registry/1";
pub const ORDERED_PRIME_ROUTE_SCHEMA: u32 = 1;
pub const ORDERED_PRIME_ROUTE_DOMAIN: &str = "uor-r4.ordered-prime-route/1";
pub const ORDERED_SENTENCE_ROUTE_SCHEMA: u32 = 1;
pub const ORDERED_SENTENCE_ROUTE_DOMAIN: &str = "uor-r4.ordered-sentence-route/1";
pub const ZETA_GRID_SCHEMA: u32 = 1;
pub const ZETA_GRID_DOMAIN: &str = "uor-r4.fixed-zeta-grid/1";
pub const ZETA_GRID_REVISION: &str = "uor-r4-core::zeta_zeros:v1";
pub const ZETA_GRID_KAPPA_REFERENCE: &str =
    "blake3:512243ed9e2c1deef0691515caf02ca25e3d5c7990184cd804f6d65c1cc8d94c";
pub const QUANTIZATION_CHART_SCHEMA: u32 = 1;
pub const QUANTIZATION_CHART_DOMAIN: &str = "uor-r4.prime-route-quantization-chart/1";
pub const PHASE_FRACTION_BITS: u8 = 29;
pub const S3_S2_FRACTION_BITS: u8 = 30;
pub const PHASE_INTERVAL: &str = "[-pi,pi)";
pub const SPIN_CHART: &str = "unit-S3--Hopf-->unit-S2+fiber";
pub const RADIAL_RING: &str = "Z[phi]";
/// The source-free compiler is deliberately a tiny canary, not a corpus
/// compiler. These ceilings are unconditional and cannot be raised by a
/// caller through the public compile API.
pub const TINY_CANARY_MAX_SENTENCES: usize = 32;
pub const TINY_CANARY_MAX_ROUTES_PER_SENTENCE: usize = 128;
pub const TINY_CANARY_MAX_TOTAL_ROUTES: usize = 2_048;
pub const TINY_CANARY_MAX_TRANSITIONS: usize = 2_016;
/// One occurrence is one pre-aggregation insertion into I1, I2, or IS.
pub const TINY_CANARY_MAX_OCCURRENCES: usize = 5_800;
/// Maximum UTF-8 byte length of one caller-provided semantic or sentence ID.
pub const TINY_CANARY_MAX_IDENTIFIER_BYTES: usize = 1_024;
/// Maximum combined UTF-8 bytes retained across semantic and sentence IDs.
pub const TINY_CANARY_MAX_TOTAL_IDENTIFIER_BYTES: usize = 256 * 1_024;
pub const CANONICAL_MANIFEST_BODY_MAX_BYTES: usize = 32 * 1024 * 1024;
pub const CANONICAL_MANIFEST_MAX_BYTES: usize = 32 * 1024 * 1024;
pub const MANIFEST_MAX_ADDRESSES: usize = TINY_CANARY_MAX_TOTAL_ROUTES;
pub const MANIFEST_MAX_I1_ROWS: usize = TINY_CANARY_MAX_TRANSITIONS;
pub const MANIFEST_MAX_I2_ROWS: usize = TINY_CANARY_MAX_TRANSITIONS;
pub const MANIFEST_MAX_IS_ROWS: usize = TINY_CANARY_MAX_TRANSITIONS;
pub const MANIFEST_MAX_TOTAL_ROWS: usize = TINY_CANARY_MAX_OCCURRENCES;
pub const MANIFEST_MAX_RETAINED_CANDIDATE_ENTRIES: usize = TINY_CANARY_MAX_OCCURRENCES;
pub const MANIFEST_MAX_CANDIDATES_PER_ROW: u16 = 128;
pub const MANIFEST_MAX_EXPERTS: usize = TINY_CANARY_MAX_TRANSITIONS;
pub const MANIFEST_MAX_NLETS: usize = TINY_CANARY_MAX_SENTENCES;
pub const MANIFEST_MAX_REBUILD_WITNESSES: usize = TINY_CANARY_MAX_SENTENCES;
/// Implemented only in the separately bounded geometric-attention experiment;
/// no product/runtime integration is established by this status.
pub const DIVISOR_FALLBACK_STATUS: &str =
    "IMPLEMENTED_GEOMETRIC_ATTENTION_EXPERIMENT_NOT_PRODUCT_INTEGRATED";
/// Implemented only in the separately bounded geometric-attention experiment;
/// no product/runtime integration is established by this status.
pub const ADJACENT_SPIN_FALLBACK_STATUS: &str =
    "IMPLEMENTED_GEOMETRIC_ATTENTION_EXPERIMENT_NOT_PRODUCT_INTEGRATED";

const PHASE_SCALE: f64 = (1u64 << 29) as f64;
const UNIT_SCALE: f64 = (1u64 << 30) as f64;
const NORMAL_EPSILON: f64 = 1.0e-15;
const BLAKE3_LABEL_PREFIX: &str = "blake3:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TinyCanaryDimension {
    Sentences,
    RoutesPerSentence,
    TotalRoutes,
    Transitions,
    Occurrences,
    IdentifierBytes,
}

impl TinyCanaryDimension {
    const fn label(self) -> &'static str {
        match self {
            Self::Sentences => "sentences",
            Self::RoutesPerSentence => "routes per sentence",
            Self::TotalRoutes => "total routes",
            Self::Transitions => "causal transitions",
            Self::Occurrences => "index occurrences",
            Self::IdentifierBytes => "identifier bytes",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimeRouteError {
    Invalid(String),
    TinyCanaryLimitExceeded {
        dimension: TinyCanaryDimension,
        observed: usize,
        maximum: usize,
    },
    ArithmeticOverflow,
    Addressing(String),
    Serialization(String),
    WorkerSpawnFailed(String),
    WorkerPanicked,
}

impl std::fmt::Display for PrimeRouteError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "invalid prime-route product: {reason}"),
            Self::TinyCanaryLimitExceeded {
                dimension,
                observed,
                maximum,
            } => write!(
                formatter,
                "tiny-canary {} limit exceeded: observed {observed}, maximum {maximum}",
                dimension.label()
            ),
            Self::ArithmeticOverflow => formatter.write_str("prime-route arithmetic overflow"),
            Self::Addressing(reason) => {
                write!(formatter, "prime-route addressing failed: {reason}")
            }
            Self::Serialization(reason) => {
                write!(formatter, "prime-route serialization failed: {reason}")
            }
            Self::WorkerSpawnFailed(reason) => {
                write!(
                    formatter,
                    "prime-route compiler worker spawn failed: {reason}"
                )
            }
            Self::WorkerPanicked => formatter.write_str("prime-route compiler worker panicked"),
        }
    }
}

impl std::error::Error for PrimeRouteError {}

/// The architecture's deliberately typed interpretation of `0^0`.
///
/// This does not overload ordinary arithmetic: the domain tag is part of the
/// canonical manifest and therefore part of its identity.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ZeroPowerBridge {
    ContinuousNull = 0,
    DiscreteEmptyProduct = 1,
}

impl ZeroPowerBridge {
    pub const fn value(self) -> u8 {
        match self {
            Self::ContinuousNull => 0,
            Self::DiscreteEmptyProduct => 1,
        }
    }

    fn from_tag(tag: u8) -> Result<Self, PrimeRouteError> {
        match tag {
            0 => Ok(Self::ContinuousNull),
            1 => Ok(Self::DiscreteEmptyProduct),
            _ => Err(PrimeRouteError::Invalid(format!(
                "unknown zero-power bridge tag {tag}"
            ))),
        }
    }
}

/// One exact prime atom. The first slice uses `u32` so a semiprime fits in
/// `u64`; longer routes retain factors rather than multiplying them.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrimeAtom(u32);

impl PrimeAtom {
    pub fn new(value: u32) -> Result<Self, PrimeRouteError> {
        if !is_prime_u32(value) {
            return Err(PrimeRouteError::Invalid(format!(
                "route atom {value} is not prime"
            )));
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

fn is_prime_u32(value: u32) -> bool {
    if value < 2 {
        return false;
    }
    if value == 2 || value == 3 {
        return true;
    }
    if value.is_multiple_of(2) || value.is_multiple_of(3) {
        return false;
    }
    let mut divisor = 5u32;
    while divisor <= value / divisor {
        if value.is_multiple_of(divisor) || value.is_multiple_of(divisor + 2) {
            return false;
        }
        divisor = match divisor.checked_add(6) {
            Some(next) => next,
            None => break,
        };
    }
    true
}

/// A semiprime expert. Factor order is canonical and does not carry route
/// direction; ordered route state does. Equal factors represent the valid
/// prime-square semiprime produced by an adjacent repeated route atom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemiprimeExpert {
    low: PrimeAtom,
    high: PrimeAtom,
}

impl SemiprimeExpert {
    pub fn new(left: PrimeAtom, right: PrimeAtom) -> Result<Self, PrimeRouteError> {
        let (low, high) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        Ok(Self { low, high })
    }

    pub const fn factors(self) -> [PrimeAtom; 2] {
        [self.low, self.high]
    }

    pub const fn product(self) -> u64 {
        self.low.0 as u64 * self.high.0 as u64
    }

    /// Return the unique prime handoff. Identical experts share a composite
    /// product and disjoint experts share one, so both correctly return none.
    pub fn handoff(self, next: Self) -> Option<PrimeAtom> {
        let gcd = gcd_u64(self.product(), next.product());
        u32::try_from(gcd)
            .ok()
            .and_then(|value| PrimeAtom::new(value).ok())
    }
}

fn gcd_u64(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

/// Ordered prime route plus its lossless commutative factor multiset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedPrimeRoute {
    ordered: Vec<PrimeAtom>,
    factors: Vec<PrimeAtom>,
}

impl OrderedPrimeRoute {
    pub fn new(ordered: Vec<PrimeAtom>) -> Result<Self, PrimeRouteError> {
        if ordered.is_empty() {
            return Err(PrimeRouteError::Invalid(
                "an ordered prime route cannot be empty".to_owned(),
            ));
        }
        let mut factors = ordered.clone();
        factors.sort_unstable();
        Ok(Self { ordered, factors })
    }

    pub fn ordered(&self) -> &[PrimeAtom] {
        &self.ordered
    }

    pub fn factors(&self) -> &[PrimeAtom] {
        &self.factors
    }

    pub fn factor_overlap(&self, other: &Self) -> Vec<PrimeAtom> {
        let mut overlap = Vec::new();
        let mut left = 0usize;
        let mut right = 0usize;
        while left < self.factors.len() && right < other.factors.len() {
            match self.factors[left].cmp(&other.factors[right]) {
                std::cmp::Ordering::Less => left += 1,
                std::cmp::Ordering::Greater => right += 1,
                std::cmp::Ordering::Equal => {
                    overlap.push(self.factors[left]);
                    left += 1;
                    right += 1;
                }
            }
        }
        overlap
    }

    /// Optional diagnostic only. The factor vectors above are normative and
    /// are never replaced with a saturated numeric product.
    pub fn checked_product_u128(&self) -> Option<u128> {
        self.ordered.iter().try_fold(1u128, |product, atom| {
            product.checked_mul(u128::from(atom.0))
        })
    }

    pub fn ordered_kappa(&self) -> Result<String, PrimeRouteError> {
        let wire = OrderedPrimeRouteWire {
            schema: ORDERED_PRIME_ROUTE_SCHEMA,
            domain: ORDERED_PRIME_ROUTE_DOMAIN,
            ordered: self.ordered.iter().map(|atom| atom.0).collect(),
        };
        json_kappa(&canonical_json(&wire)?)
    }
}

#[derive(Serialize)]
struct OrderedPrimeRouteWire<'a> {
    schema: u32,
    domain: &'a str,
    ordered: Vec<u32>,
}

/// Signed phase in radians, quantized at 2^-29 and wrapped into `[-pi, pi)`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhaseQ29(i32);

impl PhaseQ29 {
    pub const ZERO: Self = Self(0);

    pub fn from_radians(radians: f64) -> Result<Self, PrimeRouteError> {
        if !radians.is_finite() {
            return Err(PrimeRouteError::Invalid("phase must be finite".to_owned()));
        }
        let wrapped = wrap_to_pi(radians);
        let scaled = libm::round(wrapped * PHASE_SCALE);
        if scaled < f64::from(i32::MIN) || scaled > f64::from(i32::MAX) {
            return Err(PrimeRouteError::ArithmeticOverflow);
        }
        Ok(Self(scaled as i32))
    }

    pub fn from_raw(raw: i32) -> Result<Self, PrimeRouteError> {
        let radians = f64::from(raw) / PHASE_SCALE;
        if !(-std::f64::consts::PI..std::f64::consts::PI).contains(&radians) {
            return Err(PrimeRouteError::Invalid(
                "raw Q29 phase lies outside [-pi, pi)".to_owned(),
            ));
        }
        Ok(Self(raw))
    }

    pub const fn raw(self) -> i32 {
        self.0
    }

    pub fn to_radians(self) -> f64 {
        f64::from(self.0) / PHASE_SCALE
    }

    pub fn wrapping_add(self, delta: Self) -> Result<Self, PrimeRouteError> {
        Self::from_radians(self.to_radians() + delta.to_radians())
    }
}

/// Exact coefficient pair `a + b*phi`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZPhi {
    pub a: i64,
    pub b: i64,
}

impl ZPhi {
    pub const fn new(a: i64, b: i64) -> Self {
        Self { a, b }
    }

    pub fn times_phi(self) -> Result<Self, PrimeRouteError> {
        Ok(Self {
            a: self.b,
            b: self
                .a
                .checked_add(self.b)
                .ok_or(PrimeRouteError::ArithmeticOverflow)?,
        })
    }

    pub fn times_phi_inverse(self) -> Result<Self, PrimeRouteError> {
        Ok(Self {
            a: self
                .b
                .checked_sub(self.a)
                .ok_or(PrimeRouteError::ArithmeticOverflow)?,
            b: self.a,
        })
    }
}

/// Canonical Q1.30 point on S3. The original sign is retained: `q` and `-q`
/// share a Hopf observation but remain different spin states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitS3Q30([i32; 4]);

impl UnitS3Q30 {
    pub fn from_r4(values: [f64; 4]) -> Result<Self, PrimeRouteError> {
        if values.iter().any(|value| !value.is_finite()) {
            return Err(PrimeRouteError::Invalid(
                "S3 input must contain only finite values".to_owned(),
            ));
        }
        let norm_squared = values.iter().map(|value| value * value).sum::<f64>();
        if norm_squared <= NORMAL_EPSILON {
            return Err(PrimeRouteError::Invalid(
                "the zero R4 vector has no S3 direction".to_owned(),
            ));
        }
        let norm = libm::sqrt(norm_squared);
        let mut quantized = [0i32; 4];
        for (target, value) in quantized.iter_mut().zip(values) {
            let scaled = libm::round((value / norm) * UNIT_SCALE);
            if scaled < f64::from(i32::MIN) || scaled > f64::from(i32::MAX) {
                return Err(PrimeRouteError::ArithmeticOverflow);
            }
            *target = scaled as i32;
        }
        if quantized.iter().all(|value| *value == 0) {
            return Err(PrimeRouteError::Invalid(
                "S3 quantization collapsed to zero".to_owned(),
            ));
        }
        Ok(Self(quantized))
    }

    pub fn from_raw(raw: [i32; 4]) -> Result<Self, PrimeRouteError> {
        if raw.iter().all(|value| *value == 0) {
            return Err(PrimeRouteError::Invalid(
                "the zero Q1.30 vector has no S3 direction".to_owned(),
            ));
        }
        let decoded = raw.map(|value| f64::from(value) / UNIT_SCALE);
        if decoded.iter().any(|value| value.abs() > 1.0 + 1.0e-9) {
            return Err(PrimeRouteError::Invalid(
                "raw Q1.30 S3 coordinate exceeds the unit interval".to_owned(),
            ));
        }
        let norm_squared = decoded.iter().map(|value| value * value).sum::<f64>();
        if (norm_squared - 1.0).abs() > 1.0e-7 {
            return Err(PrimeRouteError::Invalid(
                "raw Q1.30 S3 state is not unit length".to_owned(),
            ));
        }
        Ok(Self(raw))
    }

    pub const fn raw(self) -> [i32; 4] {
        self.0
    }

    pub fn to_r4(self) -> [f64; 4] {
        self.0.map(|value| f64::from(value) / UNIT_SCALE)
    }

    pub fn hopf(self) -> Result<UnitS2Q30, PrimeRouteError> {
        let [a, b, c, d] = self.to_r4();
        UnitS2Q30::from_r3([
            2.0 * (a * c + b * d),
            2.0 * (b * c - a * d),
            a * a + b * b - c * c - d * d,
        ])
    }

    /// Apply the common C2 phase action `(z1,z2) -> exp(i*phase)(z1,z2)`.
    pub fn rotate_common_fiber(self, phase: PhaseQ29) -> Result<Self, PrimeRouteError> {
        let [a, b, c, d] = self.to_r4();
        let radians = phase.to_radians();
        let cosine = libm::cos(radians);
        let sine = libm::sin(radians);
        Self::from_r4([
            a * cosine - b * sine,
            a * sine + b * cosine,
            c * cosine - d * sine,
            c * sine + d * cosine,
        ])
    }
}

/// Canonical Q1.30 S2 Hopf observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnitS2Q30([i32; 3]);

impl UnitS2Q30 {
    pub fn from_r3(values: [f64; 3]) -> Result<Self, PrimeRouteError> {
        if values.iter().any(|value| !value.is_finite()) {
            return Err(PrimeRouteError::Invalid(
                "S2 input must contain only finite values".to_owned(),
            ));
        }
        let norm_squared = values.iter().map(|value| value * value).sum::<f64>();
        if norm_squared <= NORMAL_EPSILON {
            return Err(PrimeRouteError::Invalid(
                "the zero R3 vector has no S2 direction".to_owned(),
            ));
        }
        let norm = libm::sqrt(norm_squared);
        let mut quantized = [0i32; 3];
        for (target, value) in quantized.iter_mut().zip(values) {
            let scaled = libm::round((value / norm) * UNIT_SCALE);
            if scaled < f64::from(i32::MIN) || scaled > f64::from(i32::MAX) {
                return Err(PrimeRouteError::ArithmeticOverflow);
            }
            *target = scaled as i32;
        }
        Ok(Self(quantized))
    }

    pub fn from_raw(raw: [i32; 3]) -> Result<Self, PrimeRouteError> {
        if raw.iter().all(|value| *value == 0) {
            return Err(PrimeRouteError::Invalid(
                "the zero Q1.30 vector has no S2 direction".to_owned(),
            ));
        }
        let decoded = raw.map(|value| f64::from(value) / UNIT_SCALE);
        if decoded.iter().any(|value| value.abs() > 1.0 + 1.0e-9) {
            return Err(PrimeRouteError::Invalid(
                "raw Q1.30 S2 coordinate exceeds the unit interval".to_owned(),
            ));
        }
        let norm_squared = decoded.iter().map(|value| value * value).sum::<f64>();
        if (norm_squared - 1.0).abs() > 1.0e-7 {
            return Err(PrimeRouteError::Invalid(
                "raw Q1.30 S2 state is not unit length".to_owned(),
            ));
        }
        Ok(Self(raw))
    }

    pub const fn raw(self) -> [i32; 3] {
        self.0
    }

    pub fn to_r3(self) -> [f64; 3] {
        self.0.map(|value| f64::from(value) / UNIT_SCALE)
    }
}

/// Full local spin state. Hopf is derived and checked; fiber and torsion stay
/// explicit so the lossy S3 -> S2 projection never becomes route identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpinTorsionState {
    pub s3: UnitS3Q30,
    pub hopf: UnitS2Q30,
    pub fiber: PhaseQ29,
    pub torsion: PhaseQ29,
}

impl SpinTorsionState {
    pub fn new(s3: UnitS3Q30, fiber: PhaseQ29, torsion: PhaseQ29) -> Result<Self, PrimeRouteError> {
        Ok(Self {
            s3,
            hopf: s3.hopf()?,
            fiber,
            torsion,
        })
    }

    fn from_parts(
        s3: UnitS3Q30,
        hopf: UnitS2Q30,
        fiber: PhaseQ29,
        torsion: PhaseQ29,
    ) -> Result<Self, PrimeRouteError> {
        if s3.hopf()? != hopf {
            return Err(PrimeRouteError::Invalid(
                "stored Hopf observation does not derive from stored S3 state".to_owned(),
            ));
        }
        Ok(Self {
            s3,
            hopf,
            fiber,
            torsion,
        })
    }

    pub fn shift_torsion(self, delta: PhaseQ29) -> Result<Self, PrimeRouteError> {
        Ok(Self {
            torsion: self.torsion.wrapping_add(delta)?,
            ..self
        })
    }
}

/// One data-as-location address. All canonical fields are integers or exact
/// strings; no floating point crosses the artifact boundary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeometricAddress {
    pub atom: PrimeAtom,
    pub spin: SpinTorsionState,
    pub radial: ZPhi,
    pub payload_cid: String,
}

impl GeometricAddress {
    pub fn shift_torsion(&self, delta: PhaseQ29) -> Result<Self, PrimeRouteError> {
        Ok(Self {
            spin: self.spin.shift_torsion(delta)?,
            ..self.clone()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticAtom {
    pub semantic_atom_id: String,
    pub payload_cid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrimeBinding {
    pub semantic_atom_id: String,
    pub payload_cid: String,
    pub atom: PrimeAtom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimeRegistry {
    pub bindings: Vec<PrimeBinding>,
    pub registry_kappa: String,
}

impl PrimeRegistry {
    pub fn compile(atoms: &[SemanticAtom]) -> Result<Self, PrimeRouteError> {
        if atoms.is_empty() || atoms.len() > MANIFEST_MAX_ADDRESSES {
            return Err(PrimeRouteError::Invalid(
                "prime registry requires a bounded non-empty semantic atom set".to_owned(),
            ));
        }
        let mut identifier_bytes = 0usize;
        for atom in atoms {
            identifier_bytes = accumulate_identifier_bytes(
                identifier_bytes,
                &atom.semantic_atom_id,
                "semantic atom ID",
            )?;
            validate_blake3_label(&atom.payload_cid, "semantic atom payload CID")?;
        }
        let mut sorted = atoms.to_vec();
        sorted.sort_by(|left, right| {
            (&left.semantic_atom_id, &left.payload_cid)
                .cmp(&(&right.semantic_atom_id, &right.payload_cid))
        });
        if sorted
            .windows(2)
            .any(|pair| pair[0].semantic_atom_id == pair[1].semantic_atom_id)
        {
            return Err(PrimeRouteError::Invalid(
                "semantic atom IDs must be unique".to_owned(),
            ));
        }

        let mut next = 5u32;
        let mut bindings = Vec::with_capacity(sorted.len());
        for semantic in sorted {
            while !is_prime_u32(next) {
                next = next
                    .checked_add(1)
                    .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            }
            let atom = PrimeAtom(next);
            bindings.push(PrimeBinding {
                semantic_atom_id: semantic.semantic_atom_id,
                payload_cid: semantic.payload_cid,
                atom,
            });
            next = next
                .checked_add(1)
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        }
        let registry_kappa = registry_kappa_for(&bindings)?;
        Ok(Self {
            bindings,
            registry_kappa,
        })
    }

    pub fn binding_for_id(&self, semantic_atom_id: &str) -> Option<&PrimeBinding> {
        self.bindings
            .binary_search_by(|binding| binding.semantic_atom_id.as_str().cmp(semantic_atom_id))
            .ok()
            .and_then(|index| self.bindings.get(index))
    }

    /// Revalidate the complete canonical registry, including every binding,
    /// the sequential prime assignment, and the reproduced registry kappa.
    ///
    /// This narrow verifier is for artifacts that bind an already-compiled
    /// registry. It prevents a valid six-binding prefix from hiding malformed
    /// or unbound tail entries.
    pub fn validate_canonical(&self) -> Result<(), PrimeRouteError> {
        self.validate()
    }

    fn validate(&self) -> Result<(), PrimeRouteError> {
        if self.bindings.is_empty() || self.bindings.len() > MANIFEST_MAX_ADDRESSES {
            return Err(PrimeRouteError::Invalid(
                "prime registry is empty or exceeds the manifest binding ceiling".to_owned(),
            ));
        }
        validate_blake3_label(&self.registry_kappa, "prime registry kappa")?;
        let mut previous_id: Option<&str> = None;
        let mut primes = BTreeSet::new();
        let mut expected_prime = 5u32;
        let mut identifier_bytes = 0usize;
        for binding in &self.bindings {
            identifier_bytes = accumulate_identifier_bytes(
                identifier_bytes,
                &binding.semantic_atom_id,
                "prime binding semantic atom ID",
            )?;
            validate_blake3_label(&binding.payload_cid, "prime binding payload CID")?;
            if previous_id.is_some_and(|previous| previous >= binding.semantic_atom_id.as_str()) {
                return Err(PrimeRouteError::Invalid(
                    "prime bindings are not in strict semantic-ID order".to_owned(),
                ));
            }
            if !primes.insert(binding.atom) {
                return Err(PrimeRouteError::Invalid(
                    "prime registry repeats a prime".to_owned(),
                ));
            }
            PrimeAtom::new(binding.atom.0)?;
            while !is_prime_u32(expected_prime) {
                expected_prime = expected_prime
                    .checked_add(1)
                    .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            }
            if binding.atom.0 != expected_prime {
                return Err(PrimeRouteError::Invalid(
                    "prime registry does not use the canonical sequential assignment from 5"
                        .to_owned(),
                ));
            }
            expected_prime = expected_prime
                .checked_add(1)
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            previous_id = Some(&binding.semantic_atom_id);
        }
        if registry_kappa_for(&self.bindings)? != self.registry_kappa {
            return Err(PrimeRouteError::Invalid(
                "prime registry kappa does not reproduce".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct PrimeBindingWire {
    semantic_atom_id: String,
    payload_cid: String,
    prime: u32,
}

#[derive(Serialize)]
struct PrimeRegistryKappaWire<'a> {
    schema: u32,
    domain: &'a str,
    bindings: &'a [PrimeBindingWire],
}

fn registry_kappa_for(bindings: &[PrimeBinding]) -> Result<String, PrimeRouteError> {
    let wires = bindings
        .iter()
        .map(PrimeBindingWire::from)
        .collect::<Vec<_>>();
    json_kappa(&canonical_json(&PrimeRegistryKappaWire {
        schema: PRIME_REGISTRY_SCHEMA,
        domain: PRIME_REGISTRY_DOMAIN,
        bindings: &wires,
    })?)
}

impl From<&PrimeBinding> for PrimeBindingWire {
    fn from(binding: &PrimeBinding) -> Self {
        Self {
            semantic_atom_id: binding.semantic_atom_id.clone(),
            payload_cid: binding.payload_cid.clone(),
            prime: binding.atom.0,
        }
    }
}

#[derive(Serialize)]
struct ZetaGridWire<'a> {
    schema: u32,
    domain: &'a str,
    revision: &'a str,
    ordinate_bits: Vec<u64>,
}

pub fn zeta_grid_kappa() -> Result<String, PrimeRouteError> {
    let wire = ZetaGridWire {
        schema: ZETA_GRID_SCHEMA,
        domain: ZETA_GRID_DOMAIN,
        revision: ZETA_GRID_REVISION,
        ordinate_bits: ZETA_ZEROS
            .iter()
            .map(|ordinate| ordinate.to_bits())
            .collect(),
    };
    json_kappa(&canonical_json(&wire)?)
}

pub fn zeta_phase_delta(
    channel: u16,
    from: PrimeAtom,
    to: PrimeAtom,
) -> Result<PhaseQ29, PrimeRouteError> {
    let gamma = *ZETA_ZEROS.get(usize::from(channel)).ok_or_else(|| {
        PrimeRouteError::Invalid(format!("zeta channel {channel} is out of range"))
    })?;
    let delta = gamma * (libm::log(f64::from(to.0)) - libm::log(f64::from(from.0)));
    PhaseQ29::from_radians(delta)
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, PrimeRouteError> {
    serde_json::to_vec(value).map_err(|error| PrimeRouteError::Serialization(error.to_string()))
}

fn accumulate_identifier_bytes(
    current: usize,
    value: &str,
    field: &str,
) -> Result<usize, PrimeRouteError> {
    if value.len() > TINY_CANARY_MAX_IDENTIFIER_BYTES {
        return Err(PrimeRouteError::Invalid(format!(
            "{field} exceeds the {TINY_CANARY_MAX_IDENTIFIER_BYTES}-byte ceiling"
        )));
    }
    if value.trim().is_empty() {
        return Err(PrimeRouteError::Invalid(format!(
            "{field} must be non-empty"
        )));
    }
    let total = current
        .checked_add(value.len())
        .ok_or(PrimeRouteError::ArithmeticOverflow)?;
    enforce_tiny_canary_limit(
        TinyCanaryDimension::IdentifierBytes,
        total,
        TINY_CANARY_MAX_TOTAL_IDENTIFIER_BYTES,
    )?;
    Ok(total)
}

fn validate_blake3_label(value: &str, field: &str) -> Result<(), PrimeRouteError> {
    let digest = value.strip_prefix(BLAKE3_LABEL_PREFIX).ok_or_else(|| {
        PrimeRouteError::Invalid(format!(
            "{field} must use canonical lowercase blake3:<64 hex> syntax"
        ))
    })?;
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(PrimeRouteError::Invalid(format!(
            "{field} must use canonical lowercase blake3:<64 hex> syntax"
        )));
    }
    Ok(())
}

fn json_kappa(bytes: &[u8]) -> Result<String, PrimeRouteError> {
    let label = uor_addr::json::address_blake3(bytes)
        .map(|outcome| outcome.address.to_string())
        .map_err(|error| PrimeRouteError::Addressing(format!("{error:?}")))?;
    validate_blake3_label(&label, "generated kappa")?;
    Ok(label)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteSentence {
    pub sentence_id: String,
    pub routes: Vec<GeometricAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderedRouteKappa(String);

impl OrderedRouteKappa {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Incremental ordered sentence-route identity.
///
/// Each append hashes one fixed-shape record containing the preceding chain
/// kappa and one address. Maintaining this state therefore costs O(L) small
/// hashes for a route of length L; no prefix ever serializes its full history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedSentenceRouteState {
    route_count: u32,
    chain_kappa: String,
    key: Option<OrderedRouteKappa>,
}

impl OrderedSentenceRouteState {
    pub fn new() -> Result<Self, PrimeRouteError> {
        let seed = OrderedSentenceRouteSeedWire {
            schema: ORDERED_SENTENCE_ROUTE_SCHEMA,
            domain: ORDERED_SENTENCE_ROUTE_DOMAIN,
            bos: true,
        };
        Ok(Self {
            route_count: 0,
            chain_kappa: json_kappa(&canonical_json(&seed)?)?,
            key: None,
        })
    }

    pub fn append(&mut self, address: &GeometricAddress) -> Result<(), PrimeRouteError> {
        validate_blake3_label(
            &address.payload_cid,
            "incremental sentence-route payload CID",
        )?;
        if address.spin.s3.hopf()? != address.spin.hopf {
            return Err(PrimeRouteError::Invalid(
                "incremental sentence-route address carries an inconsistent Hopf observation"
                    .to_owned(),
            ));
        }
        let route_count = self
            .route_count
            .checked_add(1)
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        let wire = OrderedSentenceRouteStepWire {
            schema: ORDERED_SENTENCE_ROUTE_SCHEMA,
            domain: ORDERED_SENTENCE_ROUTE_DOMAIN,
            previous_chain_kappa: &self.chain_kappa,
            route_count,
            route: AddressWire::from(address),
        };
        let next = OrderedRouteKappa(json_kappa(&canonical_json(&wire)?)?);
        self.route_count = route_count;
        self.chain_kappa.clone_from(&next.0);
        self.key = Some(next);
        Ok(())
    }

    pub const fn route_count(&self) -> u32 {
        self.route_count
    }

    /// Empty BOS state has no lookup key because IS is causal and requires at
    /// least one observed route.
    pub fn key(&self) -> Option<&OrderedRouteKappa> {
        self.key.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteCandidate {
    pub next: GeometricAddress,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateRow {
    candidates: Vec<RouteCandidate>,
}

impl CandidateRow {
    pub fn candidates(&self) -> &[RouteCandidate] {
        &self.candidates
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteIndexes {
    last_one: BTreeMap<GeometricAddress, CandidateRow>,
    last_two: BTreeMap<(GeometricAddress, GeometricAddress), CandidateRow>,
    sentence: BTreeMap<OrderedRouteKappa, CandidateRow>,
}

impl RouteIndexes {
    pub fn last_one(&self, last: &GeometricAddress) -> Option<&CandidateRow> {
        self.last_one.get(last)
    }

    pub fn last_two(
        &self,
        previous: &GeometricAddress,
        last: &GeometricAddress,
    ) -> Option<&CandidateRow> {
        self.last_two.get(&(previous.clone(), last.clone()))
    }

    pub fn sentence_precomputed(&self, key: &OrderedRouteKappa) -> Option<&CandidateRow> {
        self.sentence.get(key)
    }

    /// Diagnostic convenience wrapper. Serving code should maintain an
    /// [`OrderedSentenceRouteState`] and call [`Self::sentence_precomputed`].
    pub fn sentence(&self, history: &[GeometricAddress]) -> Option<&CandidateRow> {
        ordered_sentence_key(history)
            .ok()
            .and_then(|key| self.sentence_precomputed(&key))
    }

    pub fn row_counts(&self) -> (usize, usize, usize) {
        (
            self.last_one.len(),
            self.last_two.len(),
            self.sentence.len(),
        )
    }

    /// Primary bounded lookup for a maintained causal state. It reads at most
    /// one row from I1, I2, and IS and never clones or rehashes full history.
    pub fn lookup_precomputed(
        &self,
        previous: Option<&GeometricAddress>,
        last: &GeometricAddress,
        sentence_key: &OrderedRouteKappa,
        maximum_candidates: NonZeroU16,
    ) -> Result<DirectLookupTrace, PrimeRouteError> {
        self.lookup_keys(
            last,
            previous.map(|previous| (previous, last)),
            Some(sentence_key),
            maximum_candidates,
        )
    }

    /// Diagnostic intervention wrapper. It accepts full history so tests and
    /// audits can perturb causal controls; serving code should use
    /// [`Self::lookup_precomputed`] with maintained state.
    pub fn lookup(
        &self,
        history: &[GeometricAddress],
        maximum_candidates: NonZeroU16,
        intervention: &RouteIntervention,
    ) -> Result<DirectLookupTrace, PrimeRouteError> {
        if history.is_empty() {
            return Err(PrimeRouteError::Invalid(
                "direct route lookup requires non-empty causal history".to_owned(),
            ));
        }

        let mut effective = history.to_vec();
        if let RouteIntervention::ShiftLastTorsion(delta) = intervention {
            let last = effective.last_mut().ok_or_else(|| {
                PrimeRouteError::Invalid("route history unexpectedly empty".to_owned())
            })?;
            *last = last.shift_torsion(*delta)?;
        }

        let effective_last = effective.last().ok_or_else(|| {
            PrimeRouteError::Invalid("route history unexpectedly empty".to_owned())
        })?;
        let last_one_key = match intervention {
            RouteIntervention::LastOne(replacement) => replacement,
            _ => effective_last,
        };
        let last_two_key = match intervention {
            RouteIntervention::LastTwo(previous, last) => Some((previous, last)),
            _ if effective.len() >= 2 => Some((
                &effective[effective.len() - 2],
                &effective[effective.len() - 1],
            )),
            _ => None,
        };
        let sentence_history = match intervention {
            RouteIntervention::Sentence(replacement) => replacement.as_slice(),
            _ => effective.as_slice(),
        };
        let sentence_key = ordered_sentence_key(sentence_history)?;

        self.lookup_keys(
            last_one_key,
            last_two_key,
            Some(&sentence_key),
            maximum_candidates,
        )
    }

    fn lookup_keys(
        &self,
        last_one_key: &GeometricAddress,
        last_two_key: Option<(&GeometricAddress, &GeometricAddress)>,
        sentence_key: Option<&OrderedRouteKappa>,
        maximum_candidates: NonZeroU16,
    ) -> Result<DirectLookupTrace, PrimeRouteError> {
        let mut merged = BTreeMap::<GeometricAddress, [u32; 3]>::new();
        let mut rows_read = [false; 3];
        let mut candidate_entries_read = 0usize;

        if let Some(row) = self.last_one.get(last_one_key) {
            rows_read[0] = true;
            candidate_entries_read = candidate_entries_read
                .checked_add(row.candidates.len())
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            for candidate in &row.candidates {
                merged.entry(candidate.next.clone()).or_default()[0] = candidate.count;
            }
        }
        if let Some((previous, last)) = last_two_key {
            if let Some(row) = self.last_two.get(&(previous.clone(), last.clone())) {
                rows_read[1] = true;
                candidate_entries_read = candidate_entries_read
                    .checked_add(row.candidates.len())
                    .ok_or(PrimeRouteError::ArithmeticOverflow)?;
                for candidate in &row.candidates {
                    merged.entry(candidate.next.clone()).or_default()[1] = candidate.count;
                }
            }
        }
        if let Some(key) = sentence_key {
            if let Some(row) = self.sentence.get(key) {
                rows_read[2] = true;
                candidate_entries_read = candidate_entries_read
                    .checked_add(row.candidates.len())
                    .ok_or(PrimeRouteError::ArithmeticOverflow)?;
                for candidate in &row.candidates {
                    merged.entry(candidate.next.clone()).or_default()[2] = candidate.count;
                }
            }
        }

        let mut candidates = merged
            .into_iter()
            .map(|(next, counts)| DirectCandidate {
                next,
                last_one_count: counts[0],
                last_two_count: counts[1],
                sentence_count: counts[2],
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            let left_sources = u8::from(left.last_one_count > 0)
                + u8::from(left.last_two_count > 0)
                + u8::from(left.sentence_count > 0);
            let right_sources = u8::from(right.last_one_count > 0)
                + u8::from(right.last_two_count > 0)
                + u8::from(right.sentence_count > 0);
            let left_total = u64::from(left.last_one_count)
                + u64::from(left.last_two_count)
                + u64::from(left.sentence_count);
            let right_total = u64::from(right.last_one_count)
                + u64::from(right.last_two_count)
                + u64::from(right.sentence_count);
            (
                std::cmp::Reverse(left_sources),
                std::cmp::Reverse(left_total),
                std::cmp::Reverse(left.sentence_count),
                std::cmp::Reverse(left.last_two_count),
                std::cmp::Reverse(left.last_one_count),
                &left.next,
            )
                .cmp(&(
                    std::cmp::Reverse(right_sources),
                    std::cmp::Reverse(right_total),
                    std::cmp::Reverse(right.sentence_count),
                    std::cmp::Reverse(right.last_two_count),
                    std::cmp::Reverse(right.last_one_count),
                    &right.next,
                ))
        });
        candidates.truncate(usize::from(maximum_candidates.get()));

        Ok(DirectLookupTrace {
            rows_read,
            candidate_entries_read,
            candidates,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteIntervention {
    None,
    LastOne(GeometricAddress),
    LastTwo(GeometricAddress, GeometricAddress),
    Sentence(Vec<GeometricAddress>),
    ShiftLastTorsion(PhaseQ29),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectCandidate {
    pub next: GeometricAddress,
    pub last_one_count: u32,
    pub last_two_count: u32,
    pub sentence_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectLookupTrace {
    /// I1, I2, and IS respectively; each index can contribute at most one row.
    pub rows_read: [bool; 3],
    pub candidate_entries_read: usize,
    pub candidates: Vec<DirectCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZetaGridBinding {
    pub revision: String,
    pub channels: u16,
    pub grid_kappa: String,
}

impl ZetaGridBinding {
    pub fn fixed() -> Result<Self, PrimeRouteError> {
        let grid_kappa = zeta_grid_kappa()?;
        if grid_kappa != ZETA_GRID_KAPPA_REFERENCE {
            return Err(PrimeRouteError::Invalid(
                "compiled zeta basis does not match its pinned kappa".to_owned(),
            ));
        }
        Ok(Self {
            revision: ZETA_GRID_REVISION.to_owned(),
            channels: u16::try_from(ZETA_ZEROS.len())
                .map_err(|_| PrimeRouteError::ArithmeticOverflow)?,
            grid_kappa,
        })
    }

    fn validate(&self) -> Result<(), PrimeRouteError> {
        validate_blake3_label(&self.grid_kappa, "zeta-grid kappa")?;
        let fixed = Self::fixed()?;
        if self != &fixed {
            return Err(PrimeRouteError::Invalid(
                "manifest zeta grid is not the immutable fixed grid".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Immutable numeric and coordinate-chart contract for manifest geometry.
///
/// These fields make the artifact's integer interpretation explicit. A
/// decoder must reject any profile other than this exact contract rather than
/// interpreting the same integer bytes under a different scale or chart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantizationChartProfile {
    pub schema: u32,
    pub domain: String,
    pub phase_fraction_bits: u8,
    pub phase_interval: String,
    pub s3_fraction_bits: u8,
    pub s2_fraction_bits: u8,
    pub spin_chart: String,
    pub radial_ring: String,
    pub zeta_grid_revision: String,
    pub zeta_channels: u16,
    pub zeta_grid_kappa: String,
}

impl QuantizationChartProfile {
    pub fn fixed() -> Result<Self, PrimeRouteError> {
        let zeta_grid = ZetaGridBinding::fixed()?;
        Ok(Self::for_zeta_grid(&zeta_grid))
    }

    fn for_zeta_grid(zeta_grid: &ZetaGridBinding) -> Self {
        Self {
            schema: QUANTIZATION_CHART_SCHEMA,
            domain: QUANTIZATION_CHART_DOMAIN.to_owned(),
            phase_fraction_bits: PHASE_FRACTION_BITS,
            phase_interval: PHASE_INTERVAL.to_owned(),
            s3_fraction_bits: S3_S2_FRACTION_BITS,
            s2_fraction_bits: S3_S2_FRACTION_BITS,
            spin_chart: SPIN_CHART.to_owned(),
            radial_ring: RADIAL_RING.to_owned(),
            zeta_grid_revision: zeta_grid.revision.clone(),
            zeta_channels: zeta_grid.channels,
            zeta_grid_kappa: zeta_grid.grid_kappa.clone(),
        }
    }

    fn validate(&self, zeta_grid: &ZetaGridBinding) -> Result<(), PrimeRouteError> {
        validate_blake3_label(&self.zeta_grid_kappa, "quantization-chart zeta-grid kappa")?;
        let fixed = Self::for_zeta_grid(zeta_grid);
        if self != &fixed {
            return Err(PrimeRouteError::Invalid(
                "manifest quantization/chart profile is not the immutable fixed profile".to_owned(),
            ));
        }
        Ok(())
    }
}

/// One canonical, unordered semiprime expert observed on an adjacent route
/// edge. Prime-square self-loops are retained rather than rejected. Direction
/// remains in the ordered n-let and rebuild witness; this table aggregates
/// equal factor pairs only.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemiprimeExpertRecord {
    pub factors: [PrimeAtom; 2],
    pub product: u64,
    pub occurrence_count: u32,
}

/// One sentence's explicit ordered prime n-let. `factor_multiset` is the same
/// population in canonical commutative order, so repetitions are retained in
/// both views while `ordered_primes` retains direction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderedNletRecord {
    pub sentence_id: String,
    pub ordered_primes: Vec<PrimeAtom>,
    pub factor_multiset: Vec<PrimeAtom>,
    pub ordered_kappa: String,
}

/// Bounded source witness for deterministic index reconstruction. Indices
/// address the manifest's strictly sorted `addresses` registry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SentenceRouteWitness {
    pub sentence_id: String,
    pub address_indices: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestProvenance {
    pub tokenizer_cid: String,
    pub corpus_cid: String,
    pub compiler_cid: String,
    pub cost_profile_cid: String,
}

impl ManifestProvenance {
    fn validate(&self) -> Result<(), PrimeRouteError> {
        for (field, value) in [
            ("tokenizer CID", self.tokenizer_cid.as_str()),
            ("corpus CID", self.corpus_cid.as_str()),
            ("compiler CID", self.compiler_cid.as_str()),
            ("cost-profile CID", self.cost_profile_cid.as_str()),
        ] {
            validate_blake3_label(value, field)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSpinManifest {
    pub schema: u32,
    pub manifest_kappa: String,
    pub zeta_grid: ZetaGridBinding,
    pub quantization_chart: QuantizationChartProfile,
    pub bridge: ZeroPowerBridge,
    pub maximum_candidates: NonZeroU16,
    pub prime_registry: PrimeRegistry,
    pub addresses: Vec<GeometricAddress>,
    pub experts: Vec<SemiprimeExpertRecord>,
    pub nlets: Vec<OrderedNletRecord>,
    pub rebuild_witnesses: Vec<SentenceRouteWitness>,
    pub indexes: RouteIndexes,
    pub provenance: ManifestProvenance,
}

impl CompiledSpinManifest {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, PrimeRouteError> {
        // Typed callers receive the same reconstructive witness binding as the
        // byte decoder. This function never participates in that validation,
        // so the check cannot recurse through canonical encoding.
        validate_decoded_manifest(self)?;
        let body = ManifestBodyWire::from_manifest(self);
        let body_bytes = canonical_json(&body)?;
        validate_canonical_manifest_body_size(body_bytes.len())?;
        let expected = json_kappa(&body_bytes)?;
        if expected != self.manifest_kappa {
            return Err(PrimeRouteError::Invalid(
                "manifest kappa does not reproduce".to_owned(),
            ));
        }
        let prefix = canonical_manifest_envelope_prefix(&self.manifest_kappa)?;
        canonical_manifest_envelope_bytes(prefix, &body_bytes)
    }

    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, PrimeRouteError> {
        if bytes.len() > CANONICAL_MANIFEST_MAX_BYTES {
            return Err(PrimeRouteError::Invalid(format!(
                "canonical manifest exceeds the {CANONICAL_MANIFEST_MAX_BYTES}-byte ceiling"
            )));
        }
        let envelope: ManifestEnvelopeWire = serde_json::from_slice(bytes)
            .map_err(|error| PrimeRouteError::Serialization(error.to_string()))?;
        if envelope.schema != PRIME_ROUTE_MANIFEST_SCHEMA
            || envelope.domain != PRIME_ROUTE_MANIFEST_DOMAIN
            || envelope.body.schema != PRIME_ROUTE_MANIFEST_SCHEMA
        {
            return Err(PrimeRouteError::Invalid(
                "manifest schema or domain is unsupported".to_owned(),
            ));
        }
        validate_blake3_label(&envelope.manifest_kappa, "encoded manifest kappa")?;
        envelope.body.validate_shape()?;
        let body_bytes = canonical_json(&envelope.body)?;
        validate_canonical_manifest_body_size(body_bytes.len())?;
        let expected_kappa = json_kappa(&body_bytes)?;
        if expected_kappa != envelope.manifest_kappa {
            return Err(PrimeRouteError::Invalid(
                "encoded manifest kappa does not reproduce".to_owned(),
            ));
        }
        let manifest = envelope.body.into_manifest(envelope.manifest_kappa)?;
        let canonical = manifest.canonical_bytes()?;
        if canonical != bytes {
            return Err(PrimeRouteError::Invalid(
                "manifest bytes are not canonical".to_owned(),
            ));
        }
        Ok(manifest)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimeRouteCompileMetadata {
    pub requested_workers: usize,
    pub used_workers: usize,
    pub sentences: usize,
    pub route_steps: usize,
    pub causal_transitions: usize,
    pub index_occurrences: usize,
    /// Maximum number of worker scopes simultaneously active, observed with
    /// an atomic counter. Operational evidence only.
    pub peak_active_workers: usize,
    pub worker_reports: Vec<PrimeRouteWorkerReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimeRouteWorkerReport {
    pub partition_id: usize,
    pub sentence_count: usize,
    pub assigned_transitions: usize,
    pub completed_transitions: usize,
    pub elapsed: std::time::Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimeRouteCompilation {
    pub manifest: CompiledSpinManifest,
    /// Operational evidence only. This is deliberately outside canonical
    /// manifest bytes and cannot alter `manifest_kappa`.
    pub metadata: PrimeRouteCompileMetadata,
}

type CandidateCounts = BTreeMap<GeometricAddress, u32>;

#[derive(Default)]
struct PartialIndexes {
    last_one: BTreeMap<GeometricAddress, CandidateCounts>,
    last_two: BTreeMap<(GeometricAddress, GeometricAddress), CandidateCounts>,
    sentence: BTreeMap<OrderedRouteKappa, CandidateCounts>,
}

#[derive(Debug)]
struct WorkPartition {
    partition_id: usize,
    sentence_indices: Vec<usize>,
    assigned_transitions: usize,
}

struct PartitionBuild {
    partial: PartialIndexes,
    report: PrimeRouteWorkerReport,
}

fn build_rebuild_witnesses(
    sentences: &[RouteSentence],
    addresses: &[GeometricAddress],
) -> Result<Vec<SentenceRouteWitness>, PrimeRouteError> {
    let mut witnesses = Vec::with_capacity(sentences.len());
    for sentence in sentences {
        let address_indices = sentence
            .routes
            .iter()
            .map(|address| {
                let index = addresses.binary_search(address).map_err(|_| {
                    PrimeRouteError::Invalid(
                        "sentence route is absent from the sorted address registry".to_owned(),
                    )
                })?;
                u16::try_from(index).map_err(|_| PrimeRouteError::ArithmeticOverflow)
            })
            .collect::<Result<Vec<_>, PrimeRouteError>>()?;
        witnesses.push(SentenceRouteWitness {
            sentence_id: sentence.sentence_id.clone(),
            address_indices,
        });
    }
    Ok(witnesses)
}

fn sentences_from_rebuild_witnesses(
    addresses: &[GeometricAddress],
    witnesses: &[SentenceRouteWitness],
) -> Result<Vec<RouteSentence>, PrimeRouteError> {
    witnesses
        .iter()
        .map(|witness| {
            let routes = witness
                .address_indices
                .iter()
                .map(|index| {
                    addresses.get(usize::from(*index)).cloned().ok_or_else(|| {
                        PrimeRouteError::Invalid(
                            "rebuild witness address index is out of range".to_owned(),
                        )
                    })
                })
                .collect::<Result<Vec<_>, PrimeRouteError>>()?;
            Ok(RouteSentence {
                sentence_id: witness.sentence_id.clone(),
                routes,
            })
        })
        .collect()
}

fn records_from_rebuild_witnesses(
    addresses: &[GeometricAddress],
    witnesses: &[SentenceRouteWitness],
) -> Result<(Vec<SemiprimeExpertRecord>, Vec<OrderedNletRecord>), PrimeRouteError> {
    let sentences = sentences_from_rebuild_witnesses(addresses, witnesses)?;
    records_from_sentences(&sentences)
}

fn records_from_sentences(
    sentences: &[RouteSentence],
) -> Result<(Vec<SemiprimeExpertRecord>, Vec<OrderedNletRecord>), PrimeRouteError> {
    let mut expert_counts = BTreeMap::<SemiprimeExpert, u32>::new();
    let mut nlets = Vec::with_capacity(sentences.len());
    for sentence in sentences {
        let ordered_primes = sentence
            .routes
            .iter()
            .map(|address| address.atom)
            .collect::<Vec<_>>();
        for pair in ordered_primes.windows(2) {
            let expert = SemiprimeExpert::new(pair[0], pair[1])?;
            let count = expert_counts.entry(expert).or_default();
            *count = count
                .checked_add(1)
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        }
        let route = OrderedPrimeRoute::new(ordered_primes.clone())?;
        nlets.push(OrderedNletRecord {
            sentence_id: sentence.sentence_id.clone(),
            ordered_primes,
            factor_multiset: route.factors().to_vec(),
            ordered_kappa: route.ordered_kappa()?,
        });
    }
    let experts = expert_counts
        .into_iter()
        .map(|(expert, occurrence_count)| SemiprimeExpertRecord {
            factors: expert.factors(),
            product: expert.product(),
            occurrence_count,
        })
        .collect();
    Ok((experts, nlets))
}

fn indexes_from_rebuild_witnesses(
    addresses: &[GeometricAddress],
    witnesses: &[SentenceRouteWitness],
    maximum_candidates: NonZeroU16,
) -> Result<RouteIndexes, PrimeRouteError> {
    let sentences = sentences_from_rebuild_witnesses(addresses, witnesses)?;
    let sentence_indices = (0..sentences.len()).collect::<Vec<_>>();
    let (partial, completed_transitions) = build_partial(&sentences, &sentence_indices)?;
    let expected_transitions = sentences.iter().try_fold(0usize, |total, sentence| {
        total
            .checked_add(sentence.routes.len().saturating_sub(1))
            .ok_or(PrimeRouteError::ArithmeticOverflow)
    })?;
    if completed_transitions != expected_transitions {
        return Err(PrimeRouteError::Invalid(
            "rebuild witness transition accounting is inconsistent".to_owned(),
        ));
    }
    Ok(finalize_indexes(partial, maximum_candidates))
}

pub fn compile_spin_manifest(
    sentences: &[RouteSentence],
    prime_registry: PrimeRegistry,
    bridge: ZeroPowerBridge,
    provenance: ManifestProvenance,
    maximum_candidates: NonZeroU16,
    workers: NonZeroUsize,
) -> Result<PrimeRouteCompilation, PrimeRouteError> {
    prime_registry.validate()?;
    provenance.validate()?;
    if maximum_candidates.get() > MANIFEST_MAX_CANDIDATES_PER_ROW {
        return Err(PrimeRouteError::Invalid(format!(
            "candidate row bound exceeds the hard cap of {MANIFEST_MAX_CANDIDATES_PER_ROW}"
        )));
    }
    if sentences.is_empty() {
        return Err(PrimeRouteError::Invalid(
            "route compilation requires at least one sentence".to_owned(),
        ));
    }
    enforce_tiny_canary_limit(
        TinyCanaryDimension::Sentences,
        sentences.len(),
        TINY_CANARY_MAX_SENTENCES,
    )?;

    let binding_by_prime = prime_registry
        .bindings
        .iter()
        .map(|binding| (binding.atom, binding.payload_cid.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut identifier_bytes =
        prime_registry
            .bindings
            .iter()
            .try_fold(0usize, |total, binding| {
                accumulate_identifier_bytes(
                    total,
                    &binding.semantic_atom_id,
                    "prime binding semantic atom ID",
                )
            })?;
    let mut sentence_ids = BTreeSet::new();
    let mut route_steps = 0usize;
    let mut causal_transitions = 0usize;
    let mut index_occurrences = 0usize;
    for sentence in sentences {
        identifier_bytes =
            accumulate_identifier_bytes(identifier_bytes, &sentence.sentence_id, "sentence ID")?;
        if !sentence_ids.insert(sentence.sentence_id.as_str()) {
            return Err(PrimeRouteError::Invalid(
                "sentence IDs must be unique".to_owned(),
            ));
        }
        if sentence.routes.is_empty() {
            return Err(PrimeRouteError::Invalid(
                "sentences require at least one route".to_owned(),
            ));
        }
        enforce_tiny_canary_limit(
            TinyCanaryDimension::RoutesPerSentence,
            sentence.routes.len(),
            TINY_CANARY_MAX_ROUTES_PER_SENTENCE,
        )?;
        let sentence_transitions = sentence.routes.len() - 1;
        let sentence_occurrences = sentence_transitions
            .checked_mul(2)
            .and_then(|base| base.checked_add(sentence.routes.len().saturating_sub(2)))
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        route_steps = route_steps
            .checked_add(sentence.routes.len())
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        causal_transitions = causal_transitions
            .checked_add(sentence_transitions)
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        index_occurrences = index_occurrences
            .checked_add(sentence_occurrences)
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
    }
    enforce_tiny_canary_limit(
        TinyCanaryDimension::TotalRoutes,
        route_steps,
        TINY_CANARY_MAX_TOTAL_ROUTES,
    )?;
    enforce_tiny_canary_limit(
        TinyCanaryDimension::Transitions,
        causal_transitions,
        TINY_CANARY_MAX_TRANSITIONS,
    )?;
    enforce_tiny_canary_limit(
        TinyCanaryDimension::Occurrences,
        index_occurrences,
        TINY_CANARY_MAX_OCCURRENCES,
    )?;
    if causal_transitions == 0 {
        return Err(PrimeRouteError::Invalid(
            "tiny-canary compilation requires at least one causal transition".to_owned(),
        ));
    }

    let mut address_set = BTreeSet::new();
    for sentence in sentences {
        for address in &sentence.routes {
            let expected_payload = binding_by_prime.get(&address.atom).ok_or_else(|| {
                PrimeRouteError::Invalid(format!(
                    "route prime {} is absent from the registry",
                    address.atom.0
                ))
            })?;
            if *expected_payload != address.payload_cid {
                return Err(PrimeRouteError::Invalid(format!(
                    "route prime {} payload does not match its registry binding",
                    address.atom.0
                )));
            }
            if address.spin.s3.hopf()? != address.spin.hopf {
                return Err(PrimeRouteError::Invalid(
                    "route address carries an inconsistent Hopf observation".to_owned(),
                ));
            }
            address_set.insert(address.clone());
        }
    }
    let mut sorted = sentences.to_vec();
    sorted.sort_by(|left, right| left.sentence_id.cmp(&right.sentence_id));
    let addresses = address_set.into_iter().collect::<Vec<_>>();
    let rebuild_witnesses = build_rebuild_witnesses(&sorted, &addresses)?;
    let (experts, nlets) = records_from_sentences(&sorted)?;

    #[cfg(target_arch = "wasm32")]
    let used_workers = 1usize;
    #[cfg(not(target_arch = "wasm32"))]
    let used_workers = workers
        .get()
        .min(
            sorted
                .iter()
                .filter(|sentence| sentence.routes.len() > 1)
                .count(),
        )
        .max(1);

    let partitions = plan_work_partitions(&sorted, used_workers)?;
    let (builds, peak_active_workers) = if used_workers == 1 {
        (vec![build_partition(&sorted, &partitions[0])?], 1)
    } else {
        #[cfg(target_arch = "wasm32")]
        {
            unreachable!("the wasm counterpart always selects one serial partition")
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            build_partitions_parallel(&sorted, &partitions)?
        }
    };
    let mut worker_reports = Vec::with_capacity(builds.len());
    let mut partials = Vec::with_capacity(builds.len());
    for build in builds {
        worker_reports.push(build.report);
        partials.push(build.partial);
    }
    worker_reports.sort_by_key(|report| report.partition_id);
    let assigned_transitions = worker_reports.iter().try_fold(0usize, |total, report| {
        total
            .checked_add(report.assigned_transitions)
            .ok_or(PrimeRouteError::ArithmeticOverflow)
    })?;
    let completed_transitions = worker_reports.iter().try_fold(0usize, |total, report| {
        total
            .checked_add(report.completed_transitions)
            .ok_or(PrimeRouteError::ArithmeticOverflow)
    })?;
    let reported_sentences = worker_reports.iter().try_fold(0usize, |total, report| {
        total
            .checked_add(report.sentence_count)
            .ok_or(PrimeRouteError::ArithmeticOverflow)
    })?;
    if worker_reports.len() != used_workers
        || worker_reports.iter().enumerate().any(|(index, report)| {
            report.partition_id != index
                || report.sentence_count == 0
                || report.assigned_transitions == 0
                || report.completed_transitions != report.assigned_transitions
        })
        || reported_sentences != sorted.len()
        || assigned_transitions != causal_transitions
        || completed_transitions != causal_transitions
    {
        return Err(PrimeRouteError::Invalid(
            "worker partition accounting is inconsistent".to_owned(),
        ));
    }
    let merged = merge_partials(partials)?;
    let indexes = finalize_indexes(merged, maximum_candidates);
    let zeta_grid = ZetaGridBinding::fixed()?;
    let quantization_chart = QuantizationChartProfile::for_zeta_grid(&zeta_grid);
    let mut manifest = CompiledSpinManifest {
        schema: PRIME_ROUTE_MANIFEST_SCHEMA,
        manifest_kappa: String::new(),
        zeta_grid,
        quantization_chart,
        bridge,
        maximum_candidates,
        prime_registry,
        addresses,
        experts,
        nlets,
        rebuild_witnesses,
        indexes,
        provenance,
    };
    let body = ManifestBodyWire::from_manifest(&manifest);
    let body_bytes = canonical_json(&body)?;
    validate_canonical_manifest_body_size(body_bytes.len())?;
    manifest.manifest_kappa = json_kappa(&body_bytes)?;
    let envelope_prefix = canonical_manifest_envelope_prefix(&manifest.manifest_kappa)?;
    validate_canonical_manifest_sizes(body_bytes.len(), envelope_prefix.len())?;
    // The compiler owns typed, bounded inputs, so one full typed validation is
    // sufficient before return. Strict byte canonicality remains enforced by
    // `canonical_bytes` and `decode_canonical`; self-encoding, decoding, and
    // re-encoding here only repeated the same validation and hashing work.
    validate_manifest(&manifest)?;

    Ok(PrimeRouteCompilation {
        manifest,
        metadata: PrimeRouteCompileMetadata {
            requested_workers: workers.get(),
            used_workers,
            sentences: sorted.len(),
            route_steps,
            causal_transitions,
            index_occurrences,
            peak_active_workers,
            worker_reports,
        },
    })
}

fn enforce_tiny_canary_limit(
    dimension: TinyCanaryDimension,
    observed: usize,
    maximum: usize,
) -> Result<(), PrimeRouteError> {
    if observed > maximum {
        return Err(PrimeRouteError::TinyCanaryLimitExceeded {
            dimension,
            observed,
            maximum,
        });
    }
    Ok(())
}

fn plan_work_partitions(
    sentences: &[RouteSentence],
    workers: usize,
) -> Result<Vec<WorkPartition>, PrimeRouteError> {
    if workers == 0 || workers > sentences.len() {
        return Err(PrimeRouteError::Invalid(
            "worker partition count is outside the sentence population".to_owned(),
        ));
    }
    let mut ranked = sentences
        .iter()
        .enumerate()
        .map(|(index, sentence)| (index, sentence.routes.len() - 1))
        .collect::<Vec<_>>();
    ranked.sort_by(|(left_index, left_weight), (right_index, right_weight)| {
        (
            std::cmp::Reverse(*left_weight),
            &sentences[*left_index].sentence_id,
            *left_index,
        )
            .cmp(&(
                std::cmp::Reverse(*right_weight),
                &sentences[*right_index].sentence_id,
                *right_index,
            ))
    });

    let mut partitions = (0..workers)
        .map(|partition_id| WorkPartition {
            partition_id,
            sentence_indices: Vec::new(),
            assigned_transitions: 0,
        })
        .collect::<Vec<_>>();
    for (rank, (sentence_index, transitions)) in ranked.into_iter().enumerate() {
        let target = if rank < workers {
            rank
        } else {
            partitions
                .iter()
                .enumerate()
                .min_by_key(|(_, partition)| {
                    (
                        partition.assigned_transitions,
                        partition.sentence_indices.len(),
                        partition.partition_id,
                    )
                })
                .map(|(index, _)| index)
                .ok_or_else(|| {
                    PrimeRouteError::Invalid("worker partition plan is empty".to_owned())
                })?
        };
        let partition = &mut partitions[target];
        partition.sentence_indices.push(sentence_index);
        partition.assigned_transitions = partition
            .assigned_transitions
            .checked_add(transitions)
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
    }
    for partition in &mut partitions {
        partition.sentence_indices.sort_unstable();
        if partition.sentence_indices.is_empty() || partition.assigned_transitions == 0 {
            return Err(PrimeRouteError::Invalid(
                "worker partition plan produced an empty or zero-transition partition".to_owned(),
            ));
        }
    }
    Ok(partitions)
}

fn build_partition(
    sentences: &[RouteSentence],
    partition: &WorkPartition,
) -> Result<PartitionBuild, PrimeRouteError> {
    let started = std::time::Instant::now();
    let (partial, completed_transitions) = build_partial(sentences, &partition.sentence_indices)?;
    if partition.assigned_transitions == 0
        || completed_transitions != partition.assigned_transitions
    {
        return Err(PrimeRouteError::Invalid(format!(
            "partition {} completed {completed_transitions} of {} assigned transitions",
            partition.partition_id, partition.assigned_transitions
        )));
    }
    Ok(PartitionBuild {
        partial,
        report: PrimeRouteWorkerReport {
            partition_id: partition.partition_id,
            sentence_count: partition.sentence_indices.len(),
            assigned_transitions: partition.assigned_transitions,
            completed_transitions,
            elapsed: started.elapsed(),
        },
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn build_partitions_parallel(
    sentences: &[RouteSentence],
    partitions: &[WorkPartition],
) -> Result<(Vec<PartitionBuild>, usize), PrimeRouteError> {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let active = AtomicUsize::new(0);
    let peak_active = AtomicUsize::new(0);
    let start_gate = WorkerStartGate::new();
    let builds = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(partitions.len());
        for partition in partitions {
            let active = &active;
            let peak_active = &peak_active;
            let start_gate = &start_gate;
            let spawn = std::thread::Builder::new()
                .name(format!("prime-route-{}", partition.partition_id))
                .spawn_scoped(scope, move || {
                    if !start_gate.wait() {
                        return Err(PrimeRouteError::WorkerSpawnFailed(
                            "worker start was aborted after a sibling spawn failed".to_owned(),
                        ));
                    }
                    let now_active = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak_active.fetch_max(now_active, Ordering::SeqCst);
                    let result = build_partition(sentences, partition);
                    active.fetch_sub(1, Ordering::SeqCst);
                    result
                });
            match spawn {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    start_gate.abort();
                    return Err(PrimeRouteError::WorkerSpawnFailed(error.to_string()));
                }
            }
        }
        start_gate.start();
        handles
            .into_iter()
            .map(|handle| handle.join().map_err(|_| PrimeRouteError::WorkerPanicked)?)
            .collect::<Result<Vec<_>, PrimeRouteError>>()
    })?;
    Ok((builds, peak_active.load(Ordering::SeqCst)))
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkerStartState {
    Pending,
    Started,
    Aborted,
}

#[cfg(not(target_arch = "wasm32"))]
struct WorkerStartGate {
    state: std::sync::Mutex<WorkerStartState>,
    changed: std::sync::Condvar,
}

#[cfg(not(target_arch = "wasm32"))]
impl WorkerStartGate {
    fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(WorkerStartState::Pending),
            changed: std::sync::Condvar::new(),
        }
    }

    fn wait(&self) -> bool {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        while *state == WorkerStartState::Pending {
            state = match self.changed.wait(state) {
                Ok(state) => state,
                Err(poisoned) => poisoned.into_inner(),
            };
        }
        *state == WorkerStartState::Started
    }

    fn start(&self) {
        self.release(WorkerStartState::Started);
    }

    fn abort(&self) {
        self.release(WorkerStartState::Aborted);
    }

    fn release(&self, next: WorkerStartState) {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        *state = next;
        self.changed.notify_all();
    }
}

fn build_partial(
    sentences: &[RouteSentence],
    sentence_indices: &[usize],
) -> Result<(PartialIndexes, usize), PrimeRouteError> {
    let mut partial = PartialIndexes::default();
    let mut completed_transitions = 0usize;
    for &sentence_index in sentence_indices {
        let sentence = sentences.get(sentence_index).ok_or_else(|| {
            PrimeRouteError::Invalid("partition sentence index is out of range".to_owned())
        })?;
        let mut sentence_state = OrderedSentenceRouteState::new()?;
        sentence_state.append(&sentence.routes[0])?;
        for next_index in 1..sentence.routes.len() {
            let next = sentence.routes[next_index].clone();
            increment(
                partial
                    .last_one
                    .entry(sentence.routes[next_index - 1].clone())
                    .or_default(),
                next.clone(),
            )?;
            if next_index >= 2 {
                increment(
                    partial
                        .last_two
                        .entry((
                            sentence.routes[next_index - 2].clone(),
                            sentence.routes[next_index - 1].clone(),
                        ))
                        .or_default(),
                    next.clone(),
                )?;
            }
            increment(
                partial
                    .sentence
                    .entry(
                        sentence_state
                            .key()
                            .ok_or_else(|| {
                                PrimeRouteError::Invalid(
                                    "sentence route state unexpectedly has no key".to_owned(),
                                )
                            })?
                            .clone(),
                    )
                    .or_default(),
                next,
            )?;
            completed_transitions = completed_transitions
                .checked_add(1)
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            if next_index + 1 < sentence.routes.len() {
                sentence_state.append(&sentence.routes[next_index])?;
            }
        }
    }
    Ok((partial, completed_transitions))
}

fn increment(counts: &mut CandidateCounts, next: GeometricAddress) -> Result<(), PrimeRouteError> {
    let count = counts.entry(next).or_default();
    *count = count
        .checked_add(1)
        .ok_or(PrimeRouteError::ArithmeticOverflow)?;
    Ok(())
}

fn merge_partials(partials: Vec<PartialIndexes>) -> Result<PartialIndexes, PrimeRouteError> {
    let mut merged = PartialIndexes::default();
    for partial in partials {
        merge_map(&mut merged.last_one, partial.last_one)?;
        merge_map(&mut merged.last_two, partial.last_two)?;
        merge_map(&mut merged.sentence, partial.sentence)?;
    }
    Ok(merged)
}

fn merge_map<K: Ord>(
    target: &mut BTreeMap<K, CandidateCounts>,
    source: BTreeMap<K, CandidateCounts>,
) -> Result<(), PrimeRouteError> {
    for (key, candidates) in source {
        let row = target.entry(key).or_default();
        for (candidate, count) in candidates {
            let total = row.entry(candidate).or_default();
            *total = total
                .checked_add(count)
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        }
    }
    Ok(())
}

fn finalize_indexes(partial: PartialIndexes, maximum_candidates: NonZeroU16) -> RouteIndexes {
    RouteIndexes {
        last_one: finalize_map(partial.last_one, maximum_candidates),
        last_two: finalize_map(partial.last_two, maximum_candidates),
        sentence: finalize_map(partial.sentence, maximum_candidates),
    }
}

fn finalize_map<K: Ord>(
    source: BTreeMap<K, CandidateCounts>,
    maximum_candidates: NonZeroU16,
) -> BTreeMap<K, CandidateRow> {
    source
        .into_iter()
        .map(|(key, candidates)| {
            let mut candidates = candidates
                .into_iter()
                .map(|(next, count)| RouteCandidate { next, count })
                .collect::<Vec<_>>();
            candidates.sort_by(|left, right| {
                (std::cmp::Reverse(left.count), &left.next)
                    .cmp(&(std::cmp::Reverse(right.count), &right.next))
            });
            candidates.truncate(usize::from(maximum_candidates.get()));
            (key, CandidateRow { candidates })
        })
        .collect()
}

pub fn ordered_sentence_key(
    history: &[GeometricAddress],
) -> Result<OrderedRouteKappa, PrimeRouteError> {
    if history.is_empty() {
        return Err(PrimeRouteError::Invalid(
            "ordered sentence route cannot be empty".to_owned(),
        ));
    }
    let mut state = OrderedSentenceRouteState::new()?;
    for address in history {
        state.append(address)?;
    }
    state.key().cloned().ok_or_else(|| {
        PrimeRouteError::Invalid("ordered sentence route unexpectedly has no key".to_owned())
    })
}

#[derive(Serialize)]
struct OrderedSentenceRouteSeedWire<'a> {
    schema: u32,
    domain: &'a str,
    bos: bool,
}

#[derive(Serialize)]
struct OrderedSentenceRouteStepWire<'a> {
    schema: u32,
    domain: &'a str,
    previous_chain_kappa: &'a str,
    route_count: u32,
    route: AddressWire,
}

#[derive(Serialize, Deserialize)]
struct ManifestEnvelopeWire {
    schema: u32,
    domain: String,
    manifest_kappa: String,
    body: ManifestBodyWire,
}

#[derive(Serialize)]
struct ManifestEnvelopePrefixWire<'a> {
    schema: u32,
    domain: &'a str,
    manifest_kappa: &'a str,
}

const MANIFEST_ENVELOPE_BODY_FIELD: &[u8] = b",\"body\":";

fn canonical_manifest_envelope_prefix(manifest_kappa: &str) -> Result<Vec<u8>, PrimeRouteError> {
    canonical_json(&ManifestEnvelopePrefixWire {
        schema: PRIME_ROUTE_MANIFEST_SCHEMA,
        domain: PRIME_ROUTE_MANIFEST_DOMAIN,
        manifest_kappa,
    })
}

fn validate_canonical_manifest_body_size(body_len: usize) -> Result<(), PrimeRouteError> {
    if body_len > CANONICAL_MANIFEST_BODY_MAX_BYTES {
        return Err(PrimeRouteError::Invalid(format!(
            "canonical manifest body exceeds the {CANONICAL_MANIFEST_BODY_MAX_BYTES}-byte ceiling"
        )));
    }
    Ok(())
}

fn validate_canonical_manifest_sizes(
    body_len: usize,
    envelope_prefix_len: usize,
) -> Result<usize, PrimeRouteError> {
    validate_canonical_manifest_body_size(body_len)?;
    let envelope_len = envelope_prefix_len
        .checked_add(MANIFEST_ENVELOPE_BODY_FIELD.len())
        .and_then(|length| length.checked_add(body_len))
        .ok_or(PrimeRouteError::ArithmeticOverflow)?;
    if envelope_len > CANONICAL_MANIFEST_MAX_BYTES {
        return Err(PrimeRouteError::Invalid(format!(
            "canonical manifest exceeds the {CANONICAL_MANIFEST_MAX_BYTES}-byte ceiling"
        )));
    }
    Ok(envelope_len)
}

fn canonical_manifest_envelope_bytes(
    mut prefix: Vec<u8>,
    body_bytes: &[u8],
) -> Result<Vec<u8>, PrimeRouteError> {
    let expected_len = validate_canonical_manifest_sizes(body_bytes.len(), prefix.len())?;
    let additional = expected_len
        .checked_sub(prefix.len())
        .ok_or(PrimeRouteError::ArithmeticOverflow)?;
    prefix
        .try_reserve(additional)
        .map_err(|error| PrimeRouteError::Serialization(error.to_string()))?;
    if prefix.pop() != Some(b'}') {
        return Err(PrimeRouteError::Serialization(
            "canonical manifest envelope prefix is malformed".to_owned(),
        ));
    }
    prefix.extend_from_slice(MANIFEST_ENVELOPE_BODY_FIELD);
    prefix.extend_from_slice(body_bytes);
    prefix.push(b'}');
    if prefix.len() != expected_len {
        return Err(PrimeRouteError::Serialization(
            "canonical manifest envelope length accounting failed".to_owned(),
        ));
    }
    Ok(prefix)
}

#[derive(Serialize, Deserialize)]
struct ManifestBodyWire {
    schema: u32,
    domain: String,
    zeta_grid: ZetaGridBindingWire,
    quantization_chart: QuantizationChartWire,
    bridge: u8,
    maximum_candidates: u16,
    prime_registry_kappa: String,
    prime_bindings: Vec<PrimeBindingWire>,
    addresses: Vec<AddressWire>,
    experts: Vec<SemiprimeExpertRecordWire>,
    nlets: Vec<OrderedNletRecordWire>,
    rebuild_witnesses: Vec<SentenceRouteWitnessWire>,
    last_one: Vec<LastOneRowWire>,
    last_two: Vec<LastTwoRowWire>,
    sentence: Vec<SentenceRowWire>,
    provenance: ProvenanceWire,
}

impl ManifestBodyWire {
    fn validate_shape(&self) -> Result<(), PrimeRouteError> {
        if self.schema != PRIME_ROUTE_MANIFEST_SCHEMA || self.domain != PRIME_ROUTE_MANIFEST_DOMAIN
        {
            return Err(PrimeRouteError::Invalid(
                "manifest body schema or domain is unsupported".to_owned(),
            ));
        }
        if self.maximum_candidates == 0 || self.maximum_candidates > MANIFEST_MAX_CANDIDATES_PER_ROW
        {
            return Err(PrimeRouteError::Invalid(format!(
                "manifest candidate bound must be in 1..={MANIFEST_MAX_CANDIDATES_PER_ROW}"
            )));
        }
        if self.prime_bindings.is_empty()
            || self.prime_bindings.len() > MANIFEST_MAX_ADDRESSES
            || self.addresses.is_empty()
            || self.addresses.len() > MANIFEST_MAX_ADDRESSES
        {
            return Err(PrimeRouteError::Invalid(
                "manifest binding/address population is empty or exceeds its ceiling".to_owned(),
            ));
        }
        let mut identifier_bytes = 0usize;
        for binding in &self.prime_bindings {
            identifier_bytes = accumulate_identifier_bytes(
                identifier_bytes,
                &binding.semantic_atom_id,
                "stored prime-binding semantic atom ID",
            )?;
        }
        if self.experts.is_empty() || self.experts.len() > MANIFEST_MAX_EXPERTS {
            return Err(PrimeRouteError::Invalid(
                "manifest semiprime-expert population is empty or exceeds its ceiling".to_owned(),
            ));
        }
        if self.nlets.is_empty()
            || self.nlets.len() > MANIFEST_MAX_NLETS
            || self.rebuild_witnesses.is_empty()
            || self.rebuild_witnesses.len() > MANIFEST_MAX_REBUILD_WITNESSES
            || self.nlets.len() != self.rebuild_witnesses.len()
        {
            return Err(PrimeRouteError::Invalid(
                "manifest n-let/rebuild-witness population is empty, inconsistent, or exceeds its ceiling"
                    .to_owned(),
            ));
        }

        let mut previous_sentence_id: Option<&str> = None;
        let mut witness_route_steps = 0usize;
        let mut witness_transitions = 0usize;
        let mut witness_occurrences = 0usize;
        for witness in &self.rebuild_witnesses {
            identifier_bytes = accumulate_identifier_bytes(
                identifier_bytes,
                &witness.sentence_id,
                "stored rebuild-witness sentence ID",
            )?;
            if previous_sentence_id.is_some_and(|previous| previous >= witness.sentence_id.as_str())
            {
                return Err(PrimeRouteError::Invalid(
                    "manifest rebuild witnesses are not in strict sentence-ID order".to_owned(),
                ));
            }
            if witness.address_indices.is_empty()
                || witness.address_indices.len() > TINY_CANARY_MAX_ROUTES_PER_SENTENCE
            {
                return Err(PrimeRouteError::Invalid(
                    "manifest rebuild witness route population is empty or exceeds its ceiling"
                        .to_owned(),
                ));
            }
            if witness
                .address_indices
                .iter()
                .any(|index| usize::from(*index) >= self.addresses.len())
            {
                return Err(PrimeRouteError::Invalid(
                    "rebuild witness address index is out of range".to_owned(),
                ));
            }
            witness_route_steps = witness_route_steps
                .checked_add(witness.address_indices.len())
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            let transitions = witness.address_indices.len() - 1;
            witness_transitions = witness_transitions
                .checked_add(transitions)
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            let occurrences = transitions
                .checked_mul(2)
                .and_then(|base| base.checked_add(witness.address_indices.len().saturating_sub(2)))
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            witness_occurrences = witness_occurrences
                .checked_add(occurrences)
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            previous_sentence_id = Some(&witness.sentence_id);
        }
        enforce_tiny_canary_limit(
            TinyCanaryDimension::TotalRoutes,
            witness_route_steps,
            TINY_CANARY_MAX_TOTAL_ROUTES,
        )?;
        enforce_tiny_canary_limit(
            TinyCanaryDimension::Transitions,
            witness_transitions,
            TINY_CANARY_MAX_TRANSITIONS,
        )?;
        enforce_tiny_canary_limit(
            TinyCanaryDimension::Occurrences,
            witness_occurrences,
            TINY_CANARY_MAX_OCCURRENCES,
        )?;
        if witness_transitions == 0 {
            return Err(PrimeRouteError::Invalid(
                "manifest rebuild witnesses require at least one causal transition".to_owned(),
            ));
        }

        let mut nlet_route_steps = 0usize;
        for (nlet, witness) in self.nlets.iter().zip(&self.rebuild_witnesses) {
            accumulate_identifier_bytes(0, &nlet.sentence_id, "stored n-let sentence ID")?;
            if nlet.sentence_id != witness.sentence_id
                || nlet.ordered_primes.is_empty()
                || nlet.ordered_primes.len() > TINY_CANARY_MAX_ROUTES_PER_SENTENCE
                || nlet.ordered_primes.len() != nlet.factor_multiset.len()
                || nlet.ordered_primes.len() != witness.address_indices.len()
            {
                return Err(PrimeRouteError::Invalid(
                    "manifest ordered n-let shape does not match its rebuild witness".to_owned(),
                ));
            }
            nlet_route_steps = nlet_route_steps
                .checked_add(nlet.ordered_primes.len())
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            validate_blake3_label(&nlet.ordered_kappa, "stored ordered n-let kappa")?;
        }
        enforce_tiny_canary_limit(
            TinyCanaryDimension::TotalRoutes,
            nlet_route_steps,
            TINY_CANARY_MAX_TOTAL_ROUTES,
        )?;

        let expert_occurrences = self.experts.iter().try_fold(0usize, |total, expert| {
            if expert.occurrence_count == 0 {
                return Err(PrimeRouteError::Invalid(
                    "manifest semiprime experts require positive occurrence counts".to_owned(),
                ));
            }
            total
                .checked_add(
                    usize::try_from(expert.occurrence_count)
                        .map_err(|_| PrimeRouteError::ArithmeticOverflow)?,
                )
                .ok_or(PrimeRouteError::ArithmeticOverflow)
        })?;
        enforce_tiny_canary_limit(
            TinyCanaryDimension::Transitions,
            expert_occurrences,
            TINY_CANARY_MAX_TRANSITIONS,
        )?;
        if expert_occurrences != witness_transitions {
            return Err(PrimeRouteError::Invalid(
                "manifest semiprime-expert occurrences do not cover every witnessed transition"
                    .to_owned(),
            ));
        }
        if self.last_one.is_empty()
            || self.last_one.len() > MANIFEST_MAX_I1_ROWS
            || self.last_two.len() > MANIFEST_MAX_I2_ROWS
            || self.sentence.is_empty()
            || self.sentence.len() > MANIFEST_MAX_IS_ROWS
        {
            return Err(PrimeRouteError::Invalid(
                "manifest causal row population is empty or exceeds its per-index ceiling"
                    .to_owned(),
            ));
        }
        let total_rows = self
            .last_one
            .len()
            .checked_add(self.last_two.len())
            .and_then(|total| total.checked_add(self.sentence.len()))
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        if total_rows > MANIFEST_MAX_TOTAL_ROWS {
            return Err(PrimeRouteError::Invalid(format!(
                "manifest row population exceeds the {MANIFEST_MAX_TOTAL_ROWS}-row ceiling"
            )));
        }

        let declared_cap = usize::from(self.maximum_candidates);
        let mut retained_candidates = 0usize;
        let mut retained_evidence = 0u64;
        for candidates in self
            .last_one
            .iter()
            .map(|row| row.candidates.as_slice())
            .chain(self.last_two.iter().map(|row| row.candidates.as_slice()))
            .chain(self.sentence.iter().map(|row| row.candidates.as_slice()))
        {
            if candidates.is_empty()
                || candidates.len() > declared_cap
                || candidates.len() > usize::from(MANIFEST_MAX_CANDIDATES_PER_ROW)
                || candidates.iter().any(|candidate| candidate.count == 0)
            {
                return Err(PrimeRouteError::Invalid(
                    "manifest candidate row is empty, non-positive, or exceeds its cap".to_owned(),
                ));
            }
            retained_candidates = retained_candidates
                .checked_add(candidates.len())
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            let row_evidence = candidates.iter().try_fold(0u64, |total, candidate| {
                total
                    .checked_add(u64::from(candidate.count))
                    .ok_or(PrimeRouteError::ArithmeticOverflow)
            })?;
            if row_evidence > TINY_CANARY_MAX_TRANSITIONS as u64 {
                return Err(PrimeRouteError::Invalid(format!(
                    "candidate-row evidence exceeds the {TINY_CANARY_MAX_TRANSITIONS}-transition ceiling"
                )));
            }
            retained_evidence = retained_evidence
                .checked_add(row_evidence)
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        }
        if retained_candidates > MANIFEST_MAX_RETAINED_CANDIDATE_ENTRIES {
            return Err(PrimeRouteError::Invalid(format!(
                "manifest retained candidates exceed the {MANIFEST_MAX_RETAINED_CANDIDATE_ENTRIES}-entry ceiling"
            )));
        }
        if retained_evidence > TINY_CANARY_MAX_OCCURRENCES as u64 {
            return Err(PrimeRouteError::Invalid(format!(
                "manifest retained evidence exceeds the {TINY_CANARY_MAX_OCCURRENCES}-occurrence ceiling"
            )));
        }

        validate_blake3_label(&self.zeta_grid.grid_kappa, "stored zeta-grid kappa")?;
        validate_blake3_label(
            &self.quantization_chart.zeta_grid_kappa,
            "stored quantization-chart zeta-grid kappa",
        )?;
        validate_blake3_label(&self.prime_registry_kappa, "stored prime-registry kappa")?;
        for binding in &self.prime_bindings {
            validate_blake3_label(&binding.payload_cid, "stored prime-binding payload CID")?;
        }
        for address in &self.addresses {
            validate_blake3_label(&address.payload_cid, "stored address payload CID")?;
        }
        for row in &self.last_one {
            validate_blake3_label(&row.key.payload_cid, "stored I1-key payload CID")?;
            validate_candidate_wire_payloads(&row.candidates)?;
        }
        for row in &self.last_two {
            validate_blake3_label(&row.previous.payload_cid, "stored I2-previous payload CID")?;
            validate_blake3_label(&row.last.payload_cid, "stored I2-last payload CID")?;
            validate_candidate_wire_payloads(&row.candidates)?;
        }
        for row in &self.sentence {
            validate_blake3_label(&row.key, "stored IS key")?;
            validate_candidate_wire_payloads(&row.candidates)?;
        }
        for (field, value) in [
            ("tokenizer CID", self.provenance.tokenizer_cid.as_str()),
            ("corpus CID", self.provenance.corpus_cid.as_str()),
            ("compiler CID", self.provenance.compiler_cid.as_str()),
            (
                "cost-profile CID",
                self.provenance.cost_profile_cid.as_str(),
            ),
        ] {
            validate_blake3_label(value, field)?;
        }
        Ok(())
    }

    fn from_manifest(manifest: &CompiledSpinManifest) -> Self {
        Self {
            schema: manifest.schema,
            domain: PRIME_ROUTE_MANIFEST_DOMAIN.to_owned(),
            zeta_grid: ZetaGridBindingWire {
                revision: manifest.zeta_grid.revision.clone(),
                channels: manifest.zeta_grid.channels,
                grid_kappa: manifest.zeta_grid.grid_kappa.clone(),
            },
            quantization_chart: QuantizationChartWire::from(&manifest.quantization_chart),
            bridge: manifest.bridge as u8,
            maximum_candidates: manifest.maximum_candidates.get(),
            prime_registry_kappa: manifest.prime_registry.registry_kappa.clone(),
            prime_bindings: manifest
                .prime_registry
                .bindings
                .iter()
                .map(PrimeBindingWire::from)
                .collect(),
            addresses: manifest.addresses.iter().map(AddressWire::from).collect(),
            experts: manifest
                .experts
                .iter()
                .map(SemiprimeExpertRecordWire::from)
                .collect(),
            nlets: manifest
                .nlets
                .iter()
                .map(OrderedNletRecordWire::from)
                .collect(),
            rebuild_witnesses: manifest
                .rebuild_witnesses
                .iter()
                .map(SentenceRouteWitnessWire::from)
                .collect(),
            last_one: manifest
                .indexes
                .last_one
                .iter()
                .map(|(key, row)| LastOneRowWire {
                    key: AddressWire::from(key),
                    candidates: row.candidates.iter().map(CandidateWire::from).collect(),
                })
                .collect(),
            last_two: manifest
                .indexes
                .last_two
                .iter()
                .map(|((previous, last), row)| LastTwoRowWire {
                    previous: AddressWire::from(previous),
                    last: AddressWire::from(last),
                    candidates: row.candidates.iter().map(CandidateWire::from).collect(),
                })
                .collect(),
            sentence: manifest
                .indexes
                .sentence
                .iter()
                .map(|(key, row)| SentenceRowWire {
                    key: key.0.clone(),
                    candidates: row.candidates.iter().map(CandidateWire::from).collect(),
                })
                .collect(),
            provenance: ProvenanceWire {
                tokenizer_cid: manifest.provenance.tokenizer_cid.clone(),
                corpus_cid: manifest.provenance.corpus_cid.clone(),
                compiler_cid: manifest.provenance.compiler_cid.clone(),
                cost_profile_cid: manifest.provenance.cost_profile_cid.clone(),
            },
        }
    }

    fn into_manifest(
        self,
        manifest_kappa: String,
    ) -> Result<CompiledSpinManifest, PrimeRouteError> {
        validate_blake3_label(&manifest_kappa, "stored manifest kappa")?;
        self.validate_shape()?;
        if self.domain != PRIME_ROUTE_MANIFEST_DOMAIN {
            return Err(PrimeRouteError::Invalid(
                "manifest body domain is unsupported".to_owned(),
            ));
        }
        let zeta_grid = ZetaGridBinding {
            revision: self.zeta_grid.revision,
            channels: self.zeta_grid.channels,
            grid_kappa: self.zeta_grid.grid_kappa,
        };
        let quantization_chart = self.quantization_chart.into_profile();
        let bridge = ZeroPowerBridge::from_tag(self.bridge)?;
        let maximum_candidates = NonZeroU16::new(self.maximum_candidates).ok_or_else(|| {
            PrimeRouteError::Invalid("manifest candidate bound is zero".to_owned())
        })?;

        let bindings = self
            .prime_bindings
            .into_iter()
            .map(|binding| {
                Ok(PrimeBinding {
                    semantic_atom_id: binding.semantic_atom_id,
                    payload_cid: binding.payload_cid,
                    atom: PrimeAtom::new(binding.prime)?,
                })
            })
            .collect::<Result<Vec<_>, PrimeRouteError>>()?;
        let prime_registry = PrimeRegistry {
            bindings,
            registry_kappa: self.prime_registry_kappa,
        };
        let addresses = self
            .addresses
            .into_iter()
            .map(AddressWire::into_address)
            .collect::<Result<Vec<_>, PrimeRouteError>>()?;
        let experts = self
            .experts
            .into_iter()
            .map(|record| {
                let low = PrimeAtom::new(record.low_prime)?;
                let high = PrimeAtom::new(record.high_prime)?;
                let expert = SemiprimeExpert::new(low, high)?;
                if expert.factors() != [low, high] {
                    return Err(PrimeRouteError::Invalid(
                        "semiprime expert factors are not in strict canonical order".to_owned(),
                    ));
                }
                Ok(SemiprimeExpertRecord {
                    factors: [low, high],
                    product: record.product,
                    occurrence_count: record.occurrence_count,
                })
            })
            .collect::<Result<Vec<_>, PrimeRouteError>>()?;
        let nlets = self
            .nlets
            .into_iter()
            .map(|record| {
                Ok(OrderedNletRecord {
                    sentence_id: record.sentence_id,
                    ordered_primes: record
                        .ordered_primes
                        .into_iter()
                        .map(PrimeAtom::new)
                        .collect::<Result<Vec<_>, PrimeRouteError>>()?,
                    factor_multiset: record
                        .factor_multiset
                        .into_iter()
                        .map(PrimeAtom::new)
                        .collect::<Result<Vec<_>, PrimeRouteError>>()?,
                    ordered_kappa: record.ordered_kappa,
                })
            })
            .collect::<Result<Vec<_>, PrimeRouteError>>()?;
        let rebuild_witnesses = self
            .rebuild_witnesses
            .into_iter()
            .map(|witness| SentenceRouteWitness {
                sentence_id: witness.sentence_id,
                address_indices: witness.address_indices,
            })
            .collect();

        let mut last_one = BTreeMap::new();
        for row in self.last_one {
            let key = row.key.into_address()?;
            let candidates = candidate_row_from_wires(row.candidates, maximum_candidates)?;
            if last_one.insert(key, candidates).is_some() {
                return Err(PrimeRouteError::Invalid(
                    "manifest repeats an I1 key".to_owned(),
                ));
            }
        }
        let mut last_two = BTreeMap::new();
        for row in self.last_two {
            let key = (row.previous.into_address()?, row.last.into_address()?);
            let candidates = candidate_row_from_wires(row.candidates, maximum_candidates)?;
            if last_two.insert(key, candidates).is_some() {
                return Err(PrimeRouteError::Invalid(
                    "manifest repeats an I2 key".to_owned(),
                ));
            }
        }
        let mut sentence = BTreeMap::new();
        for row in self.sentence {
            validate_blake3_label(&row.key, "stored IS key")?;
            let key = OrderedRouteKappa(row.key);
            let candidates = candidate_row_from_wires(row.candidates, maximum_candidates)?;
            if sentence.insert(key, candidates).is_some() {
                return Err(PrimeRouteError::Invalid(
                    "manifest repeats an IS key".to_owned(),
                ));
            }
        }

        let manifest = CompiledSpinManifest {
            schema: self.schema,
            manifest_kappa,
            zeta_grid,
            quantization_chart,
            bridge,
            maximum_candidates,
            prime_registry,
            addresses,
            experts,
            nlets,
            rebuild_witnesses,
            indexes: RouteIndexes {
                last_one,
                last_two,
                sentence,
            },
            provenance: ManifestProvenance {
                tokenizer_cid: self.provenance.tokenizer_cid,
                corpus_cid: self.provenance.corpus_cid,
                compiler_cid: self.provenance.compiler_cid,
                cost_profile_cid: self.provenance.cost_profile_cid,
            },
        };
        validate_decoded_manifest(&manifest)?;
        Ok(manifest)
    }
}

#[derive(Serialize, Deserialize)]
struct ZetaGridBindingWire {
    revision: String,
    channels: u16,
    grid_kappa: String,
}

#[derive(Serialize, Deserialize)]
struct QuantizationChartWire {
    schema: u32,
    domain: String,
    phase_fraction_bits: u8,
    phase_interval: String,
    s3_fraction_bits: u8,
    s2_fraction_bits: u8,
    spin_chart: String,
    radial_ring: String,
    zeta_grid_revision: String,
    zeta_channels: u16,
    zeta_grid_kappa: String,
}

impl From<&QuantizationChartProfile> for QuantizationChartWire {
    fn from(profile: &QuantizationChartProfile) -> Self {
        Self {
            schema: profile.schema,
            domain: profile.domain.clone(),
            phase_fraction_bits: profile.phase_fraction_bits,
            phase_interval: profile.phase_interval.clone(),
            s3_fraction_bits: profile.s3_fraction_bits,
            s2_fraction_bits: profile.s2_fraction_bits,
            spin_chart: profile.spin_chart.clone(),
            radial_ring: profile.radial_ring.clone(),
            zeta_grid_revision: profile.zeta_grid_revision.clone(),
            zeta_channels: profile.zeta_channels,
            zeta_grid_kappa: profile.zeta_grid_kappa.clone(),
        }
    }
}

impl QuantizationChartWire {
    fn into_profile(self) -> QuantizationChartProfile {
        QuantizationChartProfile {
            schema: self.schema,
            domain: self.domain,
            phase_fraction_bits: self.phase_fraction_bits,
            phase_interval: self.phase_interval,
            s3_fraction_bits: self.s3_fraction_bits,
            s2_fraction_bits: self.s2_fraction_bits,
            spin_chart: self.spin_chart,
            radial_ring: self.radial_ring,
            zeta_grid_revision: self.zeta_grid_revision,
            zeta_channels: self.zeta_channels,
            zeta_grid_kappa: self.zeta_grid_kappa,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SemiprimeExpertRecordWire {
    low_prime: u32,
    high_prime: u32,
    product: u64,
    occurrence_count: u32,
}

impl From<&SemiprimeExpertRecord> for SemiprimeExpertRecordWire {
    fn from(record: &SemiprimeExpertRecord) -> Self {
        Self {
            low_prime: record.factors[0].0,
            high_prime: record.factors[1].0,
            product: record.product,
            occurrence_count: record.occurrence_count,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct OrderedNletRecordWire {
    sentence_id: String,
    ordered_primes: Vec<u32>,
    factor_multiset: Vec<u32>,
    ordered_kappa: String,
}

impl From<&OrderedNletRecord> for OrderedNletRecordWire {
    fn from(record: &OrderedNletRecord) -> Self {
        Self {
            sentence_id: record.sentence_id.clone(),
            ordered_primes: record.ordered_primes.iter().map(|prime| prime.0).collect(),
            factor_multiset: record.factor_multiset.iter().map(|prime| prime.0).collect(),
            ordered_kappa: record.ordered_kappa.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SentenceRouteWitnessWire {
    sentence_id: String,
    address_indices: Vec<u16>,
}

impl From<&SentenceRouteWitness> for SentenceRouteWitnessWire {
    fn from(witness: &SentenceRouteWitness) -> Self {
        Self {
            sentence_id: witness.sentence_id.clone(),
            address_indices: witness.address_indices.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ProvenanceWire {
    tokenizer_cid: String,
    corpus_cid: String,
    compiler_cid: String,
    cost_profile_cid: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct AddressWire {
    prime: u32,
    s3_q30: [i32; 4],
    hopf_q30: [i32; 3],
    fiber_q29: i32,
    torsion_q29: i32,
    radial_a: i64,
    radial_b: i64,
    payload_cid: String,
}

impl From<&GeometricAddress> for AddressWire {
    fn from(address: &GeometricAddress) -> Self {
        Self {
            prime: address.atom.0,
            s3_q30: address.spin.s3.raw(),
            hopf_q30: address.spin.hopf.raw(),
            fiber_q29: address.spin.fiber.raw(),
            torsion_q29: address.spin.torsion.raw(),
            radial_a: address.radial.a,
            radial_b: address.radial.b,
            payload_cid: address.payload_cid.clone(),
        }
    }
}

impl AddressWire {
    fn into_address(self) -> Result<GeometricAddress, PrimeRouteError> {
        validate_blake3_label(&self.payload_cid, "geometric-address payload CID")?;
        let s3 = UnitS3Q30::from_raw(self.s3_q30)?;
        let hopf = UnitS2Q30::from_raw(self.hopf_q30)?;
        Ok(GeometricAddress {
            atom: PrimeAtom::new(self.prime)?,
            spin: SpinTorsionState::from_parts(
                s3,
                hopf,
                PhaseQ29::from_raw(self.fiber_q29)?,
                PhaseQ29::from_raw(self.torsion_q29)?,
            )?,
            radial: ZPhi::new(self.radial_a, self.radial_b),
            payload_cid: self.payload_cid,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct CandidateWire {
    next: AddressWire,
    count: u32,
}

impl From<&RouteCandidate> for CandidateWire {
    fn from(candidate: &RouteCandidate) -> Self {
        Self {
            next: AddressWire::from(&candidate.next),
            count: candidate.count,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct LastOneRowWire {
    key: AddressWire,
    candidates: Vec<CandidateWire>,
}

#[derive(Serialize, Deserialize)]
struct LastTwoRowWire {
    previous: AddressWire,
    last: AddressWire,
    candidates: Vec<CandidateWire>,
}

#[derive(Serialize, Deserialize)]
struct SentenceRowWire {
    key: String,
    candidates: Vec<CandidateWire>,
}

fn validate_candidate_wire_payloads(wires: &[CandidateWire]) -> Result<(), PrimeRouteError> {
    for wire in wires {
        validate_blake3_label(&wire.next.payload_cid, "stored candidate payload CID")?;
    }
    Ok(())
}

fn candidate_row_from_wires(
    wires: Vec<CandidateWire>,
    maximum_candidates: NonZeroU16,
) -> Result<CandidateRow, PrimeRouteError> {
    if wires.is_empty()
        || wires.len() > usize::from(maximum_candidates.get())
        || wires.len() > usize::from(MANIFEST_MAX_CANDIDATES_PER_ROW)
    {
        return Err(PrimeRouteError::Invalid(
            "candidate row is empty or exceeds its declared bound".to_owned(),
        ));
    }
    let candidates = wires
        .into_iter()
        .map(|wire| {
            if wire.count == 0 {
                return Err(PrimeRouteError::Invalid(
                    "candidate count must be positive".to_owned(),
                ));
            }
            Ok(RouteCandidate {
                next: wire.next.into_address()?,
                count: wire.count,
            })
        })
        .collect::<Result<Vec<_>, PrimeRouteError>>()?;
    let row = CandidateRow { candidates };
    validate_candidate_row(&row)?;
    Ok(row)
}

fn validate_candidate_row(row: &CandidateRow) -> Result<(), PrimeRouteError> {
    if row.candidates.is_empty() || row.candidates.iter().any(|candidate| candidate.count == 0) {
        return Err(PrimeRouteError::Invalid(
            "candidate rows require positive evidence".to_owned(),
        ));
    }
    let row_evidence = row.candidates.iter().try_fold(0u64, |total, candidate| {
        total
            .checked_add(u64::from(candidate.count))
            .ok_or(PrimeRouteError::ArithmeticOverflow)
    })?;
    if row_evidence > TINY_CANARY_MAX_TRANSITIONS as u64 {
        return Err(PrimeRouteError::Invalid(format!(
            "candidate-row evidence exceeds the {TINY_CANARY_MAX_TRANSITIONS}-transition ceiling"
        )));
    }
    let mut unique_next_addresses = BTreeSet::new();
    for candidate in &row.candidates {
        if !unique_next_addresses.insert(&candidate.next) {
            return Err(PrimeRouteError::Invalid(
                "candidate row repeats a next address".to_owned(),
            ));
        }
    }
    for pair in row.candidates.windows(2) {
        let left = (std::cmp::Reverse(pair[0].count), &pair[0].next);
        let right = (std::cmp::Reverse(pair[1].count), &pair[1].next);
        if left >= right {
            return Err(PrimeRouteError::Invalid(
                "candidate row is not in strict canonical rank order".to_owned(),
            ));
        }
    }
    Ok(())
}

fn validate_manifest_record_shapes(manifest: &CompiledSpinManifest) -> Result<(), PrimeRouteError> {
    if manifest.experts.is_empty() || manifest.experts.len() > MANIFEST_MAX_EXPERTS {
        return Err(PrimeRouteError::Invalid(
            "manifest semiprime-expert population is empty or exceeds its ceiling".to_owned(),
        ));
    }
    if manifest.nlets.is_empty()
        || manifest.nlets.len() > MANIFEST_MAX_NLETS
        || manifest.rebuild_witnesses.is_empty()
        || manifest.rebuild_witnesses.len() > MANIFEST_MAX_REBUILD_WITNESSES
        || manifest.nlets.len() != manifest.rebuild_witnesses.len()
    {
        return Err(PrimeRouteError::Invalid(
            "manifest n-let/rebuild-witness population is empty, inconsistent, or exceeds its ceiling"
                .to_owned(),
        ));
    }

    let mut identifier_bytes =
        manifest
            .prime_registry
            .bindings
            .iter()
            .try_fold(0usize, |total, binding| {
                accumulate_identifier_bytes(
                    total,
                    &binding.semantic_atom_id,
                    "prime binding semantic atom ID",
                )
            })?;
    let mut previous_sentence_id: Option<&str> = None;
    let mut route_steps = 0usize;
    let mut transitions = 0usize;
    let mut occurrences = 0usize;
    let mut used_addresses = BTreeSet::new();
    for witness in &manifest.rebuild_witnesses {
        identifier_bytes = accumulate_identifier_bytes(
            identifier_bytes,
            &witness.sentence_id,
            "rebuild-witness sentence ID",
        )?;
        if previous_sentence_id.is_some_and(|previous| previous >= witness.sentence_id.as_str()) {
            return Err(PrimeRouteError::Invalid(
                "manifest rebuild witnesses are not in strict sentence-ID order".to_owned(),
            ));
        }
        if witness.address_indices.is_empty()
            || witness.address_indices.len() > TINY_CANARY_MAX_ROUTES_PER_SENTENCE
        {
            return Err(PrimeRouteError::Invalid(
                "manifest rebuild witness route population is empty or exceeds its ceiling"
                    .to_owned(),
            ));
        }
        for index in &witness.address_indices {
            if usize::from(*index) >= manifest.addresses.len() {
                return Err(PrimeRouteError::Invalid(
                    "rebuild witness address index is out of range".to_owned(),
                ));
            }
            used_addresses.insert(*index);
        }
        route_steps = route_steps
            .checked_add(witness.address_indices.len())
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        let sentence_transitions = witness.address_indices.len() - 1;
        transitions = transitions
            .checked_add(sentence_transitions)
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        occurrences = occurrences
            .checked_add(
                sentence_transitions
                    .checked_mul(2)
                    .and_then(|base| {
                        base.checked_add(witness.address_indices.len().saturating_sub(2))
                    })
                    .ok_or(PrimeRouteError::ArithmeticOverflow)?,
            )
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        previous_sentence_id = Some(&witness.sentence_id);
    }
    enforce_tiny_canary_limit(
        TinyCanaryDimension::TotalRoutes,
        route_steps,
        TINY_CANARY_MAX_TOTAL_ROUTES,
    )?;
    enforce_tiny_canary_limit(
        TinyCanaryDimension::Transitions,
        transitions,
        TINY_CANARY_MAX_TRANSITIONS,
    )?;
    enforce_tiny_canary_limit(
        TinyCanaryDimension::Occurrences,
        occurrences,
        TINY_CANARY_MAX_OCCURRENCES,
    )?;
    if transitions == 0 {
        return Err(PrimeRouteError::Invalid(
            "manifest rebuild witnesses require at least one causal transition".to_owned(),
        ));
    }
    if used_addresses.len() != manifest.addresses.len() {
        return Err(PrimeRouteError::Invalid(
            "manifest address registry contains an address absent from the rebuild witness"
                .to_owned(),
        ));
    }

    let mut nlet_route_steps = 0usize;
    for (nlet, witness) in manifest.nlets.iter().zip(&manifest.rebuild_witnesses) {
        accumulate_identifier_bytes(0, &nlet.sentence_id, "ordered n-let sentence ID")?;
        validate_blake3_label(&nlet.ordered_kappa, "ordered n-let kappa")?;
        if nlet.sentence_id != witness.sentence_id
            || nlet.ordered_primes.is_empty()
            || nlet.ordered_primes.len() > TINY_CANARY_MAX_ROUTES_PER_SENTENCE
            || nlet.ordered_primes.len() != nlet.factor_multiset.len()
            || nlet.ordered_primes.len() != witness.address_indices.len()
            || nlet
                .factor_multiset
                .windows(2)
                .any(|pair| pair[0] > pair[1])
        {
            return Err(PrimeRouteError::Invalid(
                "manifest ordered n-let shape is noncanonical or does not match its rebuild witness"
                    .to_owned(),
            ));
        }
        nlet_route_steps = nlet_route_steps
            .checked_add(nlet.ordered_primes.len())
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
    }
    if nlet_route_steps != route_steps {
        return Err(PrimeRouteError::Invalid(
            "manifest ordered n-lets do not cover the rebuild-witness route population".to_owned(),
        ));
    }

    let mut expert_occurrences = 0usize;
    let mut previous_factors: Option<[PrimeAtom; 2]> = None;
    for record in &manifest.experts {
        let expert = SemiprimeExpert::new(record.factors[0], record.factors[1])?;
        if expert.factors() != record.factors
            || expert.product() != record.product
            || record.occurrence_count == 0
            || previous_factors.is_some_and(|previous| previous >= record.factors)
        {
            return Err(PrimeRouteError::Invalid(
                "manifest semiprime-expert record is noncanonical".to_owned(),
            ));
        }
        expert_occurrences = expert_occurrences
            .checked_add(
                usize::try_from(record.occurrence_count)
                    .map_err(|_| PrimeRouteError::ArithmeticOverflow)?,
            )
            .ok_or(PrimeRouteError::ArithmeticOverflow)?;
        previous_factors = Some(record.factors);
    }
    if expert_occurrences != transitions {
        return Err(PrimeRouteError::Invalid(
            "manifest semiprime-expert occurrences do not cover every witnessed transition"
                .to_owned(),
        ));
    }
    Ok(())
}

fn validate_manifest_body(manifest: &CompiledSpinManifest) -> Result<(), PrimeRouteError> {
    if manifest.schema != PRIME_ROUTE_MANIFEST_SCHEMA {
        return Err(PrimeRouteError::Invalid(
            "manifest schema is unsupported".to_owned(),
        ));
    }
    manifest.zeta_grid.validate()?;
    manifest.quantization_chart.validate(&manifest.zeta_grid)?;
    manifest.prime_registry.validate()?;
    manifest.provenance.validate()?;
    validate_manifest_record_shapes(manifest)?;
    if manifest.maximum_candidates.get() > MANIFEST_MAX_CANDIDATES_PER_ROW {
        return Err(PrimeRouteError::Invalid(format!(
            "manifest candidate bound exceeds the hard cap of {MANIFEST_MAX_CANDIDATES_PER_ROW}"
        )));
    }
    if manifest.addresses.is_empty()
        || manifest.addresses.len() > MANIFEST_MAX_ADDRESSES
        || manifest.addresses.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(PrimeRouteError::Invalid(
            "manifest addresses must be bounded, non-empty, and strictly sorted".to_owned(),
        ));
    }
    if manifest.indexes.last_one.is_empty()
        || manifest.indexes.last_one.len() > MANIFEST_MAX_I1_ROWS
        || manifest.indexes.last_two.len() > MANIFEST_MAX_I2_ROWS
        || manifest.indexes.sentence.is_empty()
        || manifest.indexes.sentence.len() > MANIFEST_MAX_IS_ROWS
    {
        return Err(PrimeRouteError::Invalid(
            "manifest causal row population is empty or exceeds its per-index ceiling".to_owned(),
        ));
    }
    let total_rows = manifest
        .indexes
        .last_one
        .len()
        .checked_add(manifest.indexes.last_two.len())
        .and_then(|total| total.checked_add(manifest.indexes.sentence.len()))
        .ok_or(PrimeRouteError::ArithmeticOverflow)?;
    if total_rows > MANIFEST_MAX_TOTAL_ROWS {
        return Err(PrimeRouteError::Invalid(format!(
            "manifest row population exceeds the {MANIFEST_MAX_TOTAL_ROWS}-row ceiling"
        )));
    }
    let (retained_candidates, retained_evidence) = manifest
        .indexes
        .last_one
        .values()
        .chain(manifest.indexes.last_two.values())
        .chain(manifest.indexes.sentence.values())
        .try_fold((0usize, 0u64), |(entries, evidence), row| {
            let entries = entries
                .checked_add(row.candidates.len())
                .ok_or(PrimeRouteError::ArithmeticOverflow)?;
            let evidence = row
                .candidates
                .iter()
                .try_fold(evidence, |total, candidate| {
                    total
                        .checked_add(u64::from(candidate.count))
                        .ok_or(PrimeRouteError::ArithmeticOverflow)
                })?;
            Ok::<_, PrimeRouteError>((entries, evidence))
        })?;
    if retained_candidates > MANIFEST_MAX_RETAINED_CANDIDATE_ENTRIES {
        return Err(PrimeRouteError::Invalid(format!(
            "manifest retained candidates exceed the {MANIFEST_MAX_RETAINED_CANDIDATE_ENTRIES}-entry ceiling"
        )));
    }
    if retained_evidence > TINY_CANARY_MAX_OCCURRENCES as u64 {
        return Err(PrimeRouteError::Invalid(format!(
            "manifest retained evidence exceeds the {TINY_CANARY_MAX_OCCURRENCES}-occurrence ceiling"
        )));
    }

    let registry = manifest
        .prime_registry
        .bindings
        .iter()
        .map(|binding| (binding.atom, binding.payload_cid.as_str()))
        .collect::<BTreeMap<_, _>>();
    let address_set = manifest.addresses.iter().collect::<BTreeSet<_>>();
    for address in &manifest.addresses {
        validate_blake3_label(&address.payload_cid, "manifest address payload CID")?;
        if registry.get(&address.atom).copied() != Some(address.payload_cid.as_str())
            || address.spin.s3.hopf()? != address.spin.hopf
        {
            return Err(PrimeRouteError::Invalid(
                "manifest address is not bound to its registry or Hopf state".to_owned(),
            ));
        }
    }

    for (key, row) in &manifest.indexes.last_one {
        if !address_set.contains(key) {
            return Err(PrimeRouteError::Invalid(
                "I1 key is absent from the address registry".to_owned(),
            ));
        }
        validate_manifest_row(row, &address_set, manifest.maximum_candidates)?;
    }
    for ((previous, last), row) in &manifest.indexes.last_two {
        if !address_set.contains(previous) || !address_set.contains(last) {
            return Err(PrimeRouteError::Invalid(
                "I2 key is absent from the address registry".to_owned(),
            ));
        }
        validate_manifest_row(row, &address_set, manifest.maximum_candidates)?;
    }
    for (key, row) in &manifest.indexes.sentence {
        validate_blake3_label(&key.0, "IS key")?;
        validate_manifest_row(row, &address_set, manifest.maximum_candidates)?;
    }
    Ok(())
}

fn validate_manifest_row(
    row: &CandidateRow,
    addresses: &BTreeSet<&GeometricAddress>,
    maximum_candidates: NonZeroU16,
) -> Result<(), PrimeRouteError> {
    if row.candidates.len() > usize::from(maximum_candidates.get())
        || row.candidates.len() > usize::from(MANIFEST_MAX_CANDIDATES_PER_ROW)
    {
        return Err(PrimeRouteError::Invalid(
            "candidate row exceeds the manifest bound".to_owned(),
        ));
    }
    validate_candidate_row(row)?;
    if row
        .candidates
        .iter()
        .any(|candidate| !addresses.contains(&candidate.next))
    {
        return Err(PrimeRouteError::Invalid(
            "candidate address is absent from the manifest registry".to_owned(),
        ));
    }
    Ok(())
}

fn validate_manifest(manifest: &CompiledSpinManifest) -> Result<(), PrimeRouteError> {
    validate_manifest_body(manifest)?;
    validate_blake3_label(&manifest.manifest_kappa, "manifest kappa")?;
    Ok(())
}

fn validate_decoded_manifest(manifest: &CompiledSpinManifest) -> Result<(), PrimeRouteError> {
    validate_manifest(manifest)?;
    let (expected_experts, expected_nlets) =
        records_from_rebuild_witnesses(&manifest.addresses, &manifest.rebuild_witnesses)?;
    if manifest.experts != expected_experts {
        return Err(PrimeRouteError::Invalid(
            "manifest semiprime experts do not reproduce from the rebuild witness".to_owned(),
        ));
    }
    if manifest.nlets != expected_nlets {
        return Err(PrimeRouteError::Invalid(
            "manifest ordered n-lets do not reproduce from the rebuild witness".to_owned(),
        ));
    }
    let expected_indexes = indexes_from_rebuild_witnesses(
        &manifest.addresses,
        &manifest.rebuild_witnesses,
        manifest.maximum_candidates,
    )?;
    if manifest.indexes != expected_indexes {
        return Err(PrimeRouteError::Invalid(
            "manifest I1/I2/IS indexes do not reproduce from the rebuild witness".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    use super::WorkerStartGate;

    #[test]
    fn worker_start_gate_abort_releases_a_waiter_without_starting_work() {
        let gate = Arc::new(WorkerStartGate::new());
        let waiter_gate = Arc::clone(&gate);
        let (ready_sender, ready_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();
        let waiter = std::thread::spawn(move || {
            ready_sender.send(()).unwrap();
            result_sender.send(waiter_gate.wait()).unwrap();
        });

        ready_receiver.recv_timeout(Duration::from_secs(1)).unwrap();
        gate.abort();
        assert!(!result_receiver
            .recv_timeout(Duration::from_secs(1))
            .unwrap());
        waiter.join().unwrap();
    }
}
