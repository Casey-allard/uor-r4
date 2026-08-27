//! Exact, bounded reproduction of the SpiralCore v63 octonion/Cl(0,6)
//! operator convention.
//!
//! The source convention uses the oriented Fano cycles `(124)`, `(235)`,
//! `(346)`, `(457)`, `(561)`, `(672)`, and `(713)`. Left and right
//! multiplication by the first six imaginary basis units give two independent
//! signed 8x8 representations. The fifteen left bivectors `B_ij = L_i L_j`
//! are paired, in lexicographic order, with the fifteen square-free semiprimes
//! made from a canonical six-prime registry prefix.
//!
//! This module is deliberately narrower than the reference page. It does not
//! identify E8 with R4, H4, S3, or the Hopf S2 observation; it does not treat
//! telecom Bell multi-frequency signaling as a quantum Bell construction; it
//! does not use IPv6 spelling as semantic route identity; and it does not
//! define transport between different six-prime charts. The optional operator
//! chart is metadata until a causal control establishes semantic value.

use std::collections::{BTreeSet, VecDeque};

use serde::Serialize;

use crate::prime_route_attention::{PrimeAtom, PrimeRegistry, SemiprimeExpert};

pub const SPIRALCORE_V63_REFERENCE_SHA256: &str =
    "3f8e6a98186999cca6c55ea42cd8b496935837c2987379f39e8e659b56360215";
pub const SPIRALCORE_OPERATOR_SCHEMA: u32 = 1;
pub const SPIRALCORE_OPERATOR_DOMAIN: &str = "uor-r4.spiralcore-v63-cl06-operator/1";
pub const SPIRALCORE_OPERATOR_KAPPA_REFERENCE: &str =
    "blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b";
pub const SIX_PRIME_CHART_SCHEMA: u32 = 1;
pub const SIX_PRIME_CHART_DOMAIN: &str = "uor-r4.spiralcore-v63-six-prime-chart/1";
pub const CHART_TRANSPORT_STATUS: &str = "NOT_ESTABLISHED";
pub const OPERATOR_SEMANTIC_STATUS: &str = "OPTIONAL_CONTROL_PENDING";
pub const CANONICAL_SIX_PRIME_VALUES: [u32; 6] = [5, 7, 11, 13, 17, 19];
pub const OCTONION_FANO_CYCLES: [[u8; 3]; 7] = [
    [1, 2, 4],
    [2, 3, 5],
    [3, 4, 6],
    [4, 5, 7],
    [5, 6, 1],
    [6, 7, 2],
    [7, 1, 3],
];

const BASIS_DIMENSION: usize = 8;
const CL06_GENERATOR_COUNT: usize = 6;
const CL06_BIVECTOR_COUNT: usize = 15;
const FINITE_GROUP_LIMIT: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpiralCoreOperatorError {
    BasisIndex(u8),
    ArithmeticOverflow,
    InvalidRegistry(String),
    Invariant(String),
    Serialization(String),
    Addressing(String),
}

impl std::fmt::Display for SpiralCoreOperatorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BasisIndex(index) => {
                write!(formatter, "octonion basis index {index} is outside 0..=7")
            }
            Self::ArithmeticOverflow => formatter.write_str("signed operator arithmetic overflow"),
            Self::InvalidRegistry(reason) => write!(formatter, "invalid prime registry: {reason}"),
            Self::Invariant(reason) => {
                write!(
                    formatter,
                    "SpiralCore v63 operator invariant failed: {reason}"
                )
            }
            Self::Serialization(reason) => {
                write!(formatter, "operator canonicalization failed: {reason}")
            }
            Self::Addressing(reason) => write!(formatter, "operator addressing failed: {reason}"),
        }
    }
}

impl std::error::Error for SpiralCoreOperatorError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedBasisUnit {
    pub sign: i8,
    pub index: u8,
}

/// Exact multiplication of two octonion basis units under the v63 Fano
/// orientation.
pub fn octonion_basis_product(
    left: u8,
    right: u8,
) -> Result<SignedBasisUnit, SpiralCoreOperatorError> {
    if left >= BASIS_DIMENSION as u8 {
        return Err(SpiralCoreOperatorError::BasisIndex(left));
    }
    if right >= BASIS_DIMENSION as u8 {
        return Err(SpiralCoreOperatorError::BasisIndex(right));
    }
    if left == 0 {
        return Ok(SignedBasisUnit {
            sign: 1,
            index: right,
        });
    }
    if right == 0 {
        return Ok(SignedBasisUnit {
            sign: 1,
            index: left,
        });
    }
    if left == right {
        return Ok(SignedBasisUnit { sign: -1, index: 0 });
    }

    for [a, b, c] in OCTONION_FANO_CYCLES {
        for [x, y, z] in [[a, b, c], [b, c, a], [c, a, b]] {
            if left == x && right == y {
                return Ok(SignedBasisUnit { sign: 1, index: z });
            }
        }
        for [x, y, z] in [[b, a, c], [c, b, a], [a, c, b]] {
            if left == x && right == y {
                return Ok(SignedBasisUnit { sign: -1, index: z });
            }
        }
    }

    Err(SpiralCoreOperatorError::Invariant(format!(
        "Fano table has no product for e{left}e{right}"
    )))
}

fn signed_basis_product(
    left: SignedBasisUnit,
    right: SignedBasisUnit,
) -> Result<SignedBasisUnit, SpiralCoreOperatorError> {
    let product = octonion_basis_product(left.index, right.index)?;
    let sign = left
        .sign
        .checked_mul(right.sign)
        .and_then(|value| value.checked_mul(product.sign))
        .ok_or(SpiralCoreOperatorError::ArithmeticOverflow)?;
    Ok(SignedBasisUnit {
        sign,
        index: product.index,
    })
}

/// Exact basis associator `(e_i e_j)e_k - e_i(e_j e_k)`.
pub fn octonion_basis_associator(
    first: u8,
    second: u8,
    third: u8,
) -> Result<[i8; BASIS_DIMENSION], SpiralCoreOperatorError> {
    let positive = SignedBasisUnit {
        sign: 1,
        index: first,
    };
    let middle = SignedBasisUnit {
        sign: 1,
        index: second,
    };
    let negative = SignedBasisUnit {
        sign: 1,
        index: third,
    };
    let left = signed_basis_product(signed_basis_product(positive, middle)?, negative)?;
    let right = signed_basis_product(positive, signed_basis_product(middle, negative)?)?;
    let mut associator = [0i8; BASIS_DIMENSION];
    associator[usize::from(left.index)] = associator[usize::from(left.index)]
        .checked_add(left.sign)
        .ok_or(SpiralCoreOperatorError::ArithmeticOverflow)?;
    associator[usize::from(right.index)] = associator[usize::from(right.index)]
        .checked_sub(right.sign)
        .ok_or(SpiralCoreOperatorError::ArithmeticOverflow)?;
    Ok(associator)
}

/// An exact signed 8x8 matrix. Constructors in this module produce signed
/// permutation matrices; callers cannot inject an unchecked matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct SignedMatrix8([[i8; BASIS_DIMENSION]; BASIS_DIMENSION]);

impl SignedMatrix8 {
    pub const fn identity() -> Self {
        let mut entries = [[0i8; BASIS_DIMENSION]; BASIS_DIMENSION];
        let mut index = 0usize;
        while index < BASIS_DIMENSION {
            entries[index][index] = 1;
            index += 1;
        }
        Self(entries)
    }

    pub const fn entries(&self) -> &[[i8; BASIS_DIMENSION]; BASIS_DIMENSION] {
        &self.0
    }

    pub fn checked_mul(self, right: Self) -> Result<Self, SpiralCoreOperatorError> {
        let mut product = [[0i8; BASIS_DIMENSION]; BASIS_DIMENSION];
        for (row, product_row) in product.iter_mut().enumerate() {
            for (column, product_entry) in product_row.iter_mut().enumerate() {
                let mut sum = 0i16;
                for inner in 0..BASIS_DIMENSION {
                    sum = sum
                        .checked_add(
                            i16::from(self.0[row][inner]) * i16::from(right.0[inner][column]),
                        )
                        .ok_or(SpiralCoreOperatorError::ArithmeticOverflow)?;
                }
                *product_entry =
                    i8::try_from(sum).map_err(|_| SpiralCoreOperatorError::ArithmeticOverflow)?;
            }
        }
        Ok(Self(product))
    }

    pub fn checked_power(self, mut exponent: u64) -> Result<Self, SpiralCoreOperatorError> {
        let mut result = Self::identity();
        let mut base = self;
        while exponent > 0 {
            if exponent % 2 == 1 {
                result = result.checked_mul(base)?;
            }
            exponent /= 2;
            if exponent > 0 {
                base = base.checked_mul(base)?;
            }
        }
        Ok(result)
    }

    pub fn negated(self) -> Result<Self, SpiralCoreOperatorError> {
        let mut entries = self.0;
        for row in &mut entries {
            for entry in row {
                *entry = entry
                    .checked_neg()
                    .ok_or(SpiralCoreOperatorError::ArithmeticOverflow)?;
            }
        }
        Ok(Self(entries))
    }
}

fn basis_action_matrix(
    basis_index: u8,
    left_action: bool,
) -> Result<SignedMatrix8, SpiralCoreOperatorError> {
    if basis_index >= BASIS_DIMENSION as u8 {
        return Err(SpiralCoreOperatorError::BasisIndex(basis_index));
    }
    let mut entries = [[0i8; BASIS_DIMENSION]; BASIS_DIMENSION];
    for column in 0..BASIS_DIMENSION as u8 {
        let product = if left_action {
            octonion_basis_product(basis_index, column)?
        } else {
            octonion_basis_product(column, basis_index)?
        };
        entries[usize::from(product.index)][usize::from(column)] = product.sign;
    }
    Ok(SignedMatrix8(entries))
}

pub fn octonion_left_basis_matrix(
    basis_index: u8,
) -> Result<SignedMatrix8, SpiralCoreOperatorError> {
    basis_action_matrix(basis_index, true)
}

pub fn octonion_right_basis_matrix(
    basis_index: u8,
) -> Result<SignedMatrix8, SpiralCoreOperatorError> {
    basis_action_matrix(basis_index, false)
}

pub fn cl06_left_generators(
) -> Result<[SignedMatrix8; CL06_GENERATOR_COUNT], SpiralCoreOperatorError> {
    let mut generators = [SignedMatrix8::identity(); CL06_GENERATOR_COUNT];
    for (index, generator) in generators.iter_mut().enumerate() {
        *generator = octonion_left_basis_matrix(index as u8 + 1)?;
    }
    Ok(generators)
}

pub fn cl06_right_generators(
) -> Result<[SignedMatrix8; CL06_GENERATOR_COUNT], SpiralCoreOperatorError> {
    let mut generators = [SignedMatrix8::identity(); CL06_GENERATOR_COUNT];
    for (index, generator) in generators.iter_mut().enumerate() {
        *generator = octonion_right_basis_matrix(index as u8 + 1)?;
    }
    Ok(generators)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Cl06Bivector {
    pub index: u8,
    /// Zero-based generator slots, matching the v63 JavaScript table.
    pub pair: [u8; 2],
    pub matrix: SignedMatrix8,
}

pub fn cl06_bivectors() -> Result<[Cl06Bivector; CL06_BIVECTOR_COUNT], SpiralCoreOperatorError> {
    let generators = cl06_left_generators()?;
    let mut bivectors = Vec::with_capacity(CL06_BIVECTOR_COUNT);
    for left in 0..CL06_GENERATOR_COUNT {
        for right in left + 1..CL06_GENERATOR_COUNT {
            bivectors.push(Cl06Bivector {
                index: bivectors.len() as u8,
                pair: [left as u8, right as u8],
                matrix: generators[left].checked_mul(generators[right])?,
            });
        }
    }
    bivectors.try_into().map_err(|entries: Vec<_>| {
        SpiralCoreOperatorError::Invariant(format!(
            "constructed {} bivectors instead of {CL06_BIVECTOR_COUNT}",
            entries.len()
        ))
    })
}

fn product_in_order(matrices: &[SignedMatrix8]) -> Result<SignedMatrix8, SpiralCoreOperatorError> {
    matrices
        .iter()
        .try_fold(SignedMatrix8::identity(), |product, matrix| {
            product.checked_mul(*matrix)
        })
}

pub fn cl06_left_volume() -> Result<SignedMatrix8, SpiralCoreOperatorError> {
    product_in_order(&cl06_left_generators()?)
}

pub fn cl06_right_volume() -> Result<SignedMatrix8, SpiralCoreOperatorError> {
    product_in_order(&cl06_right_generators()?)
}

/// Generate the finite group produced by right-multiplying the identity and
/// every discovered matrix by each of the fifteen left bivectors. The fixed
/// ceiling is only a fail-closed guard; the reproduced convention has 64
/// elements.
pub fn cl06_finite_group() -> Result<Vec<SignedMatrix8>, SpiralCoreOperatorError> {
    let bivectors = cl06_bivectors()?;
    let identity = SignedMatrix8::identity();
    let mut seen = BTreeSet::from([identity]);
    let mut queue = VecDeque::from([identity]);
    let mut discovery_order = vec![identity];
    while let Some(current) = queue.pop_front() {
        for bivector in &bivectors {
            let next = current.checked_mul(bivector.matrix)?;
            if seen.insert(next) {
                if seen.len() > FINITE_GROUP_LIMIT {
                    return Err(SpiralCoreOperatorError::Invariant(format!(
                        "finite group exceeded the {FINITE_GROUP_LIMIT}-matrix ceiling"
                    )));
                }
                queue.push_back(next);
                discovery_order.push(next);
            }
        }
    }
    Ok(discovery_order)
}

type WideMatrix8 = [[i16; BASIS_DIMENSION]; BASIS_DIMENSION];

fn anticommutator(
    left: SignedMatrix8,
    right: SignedMatrix8,
) -> Result<WideMatrix8, SpiralCoreOperatorError> {
    let forward = left.checked_mul(right)?;
    let reverse = right.checked_mul(left)?;
    let mut result = [[0i16; BASIS_DIMENSION]; BASIS_DIMENSION];
    for (row_index, row) in result.iter_mut().enumerate() {
        for (column_index, entry) in row.iter_mut().enumerate() {
            *entry = i16::from(forward.0[row_index][column_index])
                + i16::from(reverse.0[row_index][column_index]);
        }
    }
    Ok(result)
}

fn commutator(
    left: SignedMatrix8,
    right: SignedMatrix8,
) -> Result<WideMatrix8, SpiralCoreOperatorError> {
    let forward = left.checked_mul(right)?;
    let reverse = right.checked_mul(left)?;
    let mut result = [[0i16; BASIS_DIMENSION]; BASIS_DIMENSION];
    for (row_index, row) in result.iter_mut().enumerate() {
        for (column_index, entry) in row.iter_mut().enumerate() {
            *entry = i16::from(forward.0[row_index][column_index])
                - i16::from(reverse.0[row_index][column_index]);
        }
    }
    Ok(result)
}

fn bivector_coordinates(
    matrix: &WideMatrix8,
    bivectors: &[Cl06Bivector; CL06_BIVECTOR_COUNT],
) -> Result<[i16; CL06_BIVECTOR_COUNT], SpiralCoreOperatorError> {
    let mut coordinates = [0i16; CL06_BIVECTOR_COUNT];
    for (basis_index, bivector) in bivectors.iter().enumerate() {
        let mut numerator = 0i32;
        let mut denominator = 0i32;
        for (matrix_row, basis_row) in matrix.iter().zip(&bivector.matrix.0) {
            for (matrix_entry, basis_entry) in matrix_row.iter().zip(basis_row) {
                let basis_entry = i32::from(*basis_entry);
                numerator += i32::from(*matrix_entry) * basis_entry;
                denominator += basis_entry * basis_entry;
            }
        }
        if denominator == 0 || numerator % denominator != 0 {
            return Err(SpiralCoreOperatorError::Invariant(format!(
                "commutator has a non-integral coordinate on bivector {basis_index}"
            )));
        }
        coordinates[basis_index] = i16::try_from(numerator / denominator)
            .map_err(|_| SpiralCoreOperatorError::ArithmeticOverflow)?;
    }

    let mut reconstructed = [[0i32; BASIS_DIMENSION]; BASIS_DIMENSION];
    for (basis_index, bivector) in bivectors.iter().enumerate() {
        for (row_index, row) in reconstructed.iter_mut().enumerate() {
            for (column_index, entry) in row.iter_mut().enumerate() {
                *entry += i32::from(coordinates[basis_index])
                    * i32::from(bivector.matrix.0[row_index][column_index]);
            }
        }
    }
    for (reconstructed_row, matrix_row) in reconstructed.iter().zip(matrix) {
        for (reconstructed_entry, matrix_entry) in reconstructed_row.iter().zip(matrix_row) {
            if *reconstructed_entry != i32::from(*matrix_entry) {
                return Err(SpiralCoreOperatorError::Invariant(
                    "bivector commutator left the exact 15-dimensional span".to_owned(),
                ));
            }
        }
    }
    Ok(coordinates)
}

/// Exact Killing form from the adjoint action of the fifteen bivectors.
pub fn cl06_killing_form(
) -> Result<[[i32; CL06_BIVECTOR_COUNT]; CL06_BIVECTOR_COUNT], SpiralCoreOperatorError> {
    let bivectors = cl06_bivectors()?;
    let mut structure = [[[0i16; CL06_BIVECTOR_COUNT]; CL06_BIVECTOR_COUNT]; CL06_BIVECTOR_COUNT];
    for left in 0..CL06_BIVECTOR_COUNT {
        for right in 0..CL06_BIVECTOR_COUNT {
            structure[left][right] = bivector_coordinates(
                &commutator(bivectors[left].matrix, bivectors[right].matrix)?,
                &bivectors,
            )?;
        }
    }

    let mut killing = [[0i32; CL06_BIVECTOR_COUNT]; CL06_BIVECTOR_COUNT];
    for left in 0..CL06_BIVECTOR_COUNT {
        for right in 0..CL06_BIVECTOR_COUNT {
            let mut trace = 0i32;
            for (inner, left_column) in structure[left].iter().enumerate() {
                for (row, left_value) in left_column.iter().enumerate() {
                    trace += i32::from(*left_value) * i32::from(structure[right][row][inner]);
                }
            }
            killing[left][right] = trace;
        }
    }
    Ok(killing)
}

#[derive(Serialize)]
struct OperatorConventionWire {
    schema: u32,
    domain: &'static str,
    reference_sha256: &'static str,
    fano_cycles: [[u8; 3]; 7],
    left_generators: [SignedMatrix8; CL06_GENERATOR_COUNT],
    right_generators: [SignedMatrix8; CL06_GENERATOR_COUNT],
    bivectors: [Cl06Bivector; CL06_BIVECTOR_COUNT],
    finite_group: Vec<SignedMatrix8>,
}

fn canonical_kappa<T: Serialize>(value: &T) -> Result<String, SpiralCoreOperatorError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| SpiralCoreOperatorError::Serialization(error.to_string()))?;
    let address = uor_addr::json::address_blake3(&bytes)
        .map_err(|error| SpiralCoreOperatorError::Addressing(format!("{error:?}")))?
        .address
        .to_string();
    validate_blake3_label(&address)?;
    Ok(address)
}

fn validate_blake3_label(value: &str) -> Result<(), SpiralCoreOperatorError> {
    let Some(digest) = value.strip_prefix("blake3:") else {
        return Err(SpiralCoreOperatorError::InvalidRegistry(
            "kappa lacks the blake3 prefix".to_owned(),
        ));
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(SpiralCoreOperatorError::InvalidRegistry(
            "kappa is not canonical lowercase blake3:<64 hex>".to_owned(),
        ));
    }
    Ok(())
}

/// Kappa of the exact v63 Fano, left/right generator, bivector, and finite
/// group convention. It excludes E8 roots and every networking presentation.
pub fn spiralcore_operator_kappa() -> Result<String, SpiralCoreOperatorError> {
    canonical_kappa(&OperatorConventionWire {
        schema: SPIRALCORE_OPERATOR_SCHEMA,
        domain: SPIRALCORE_OPERATOR_DOMAIN,
        reference_sha256: SPIRALCORE_V63_REFERENCE_SHA256,
        fano_cycles: OCTONION_FANO_CYCLES,
        left_generators: cl06_left_generators()?,
        right_generators: cl06_right_generators()?,
        bivectors: cl06_bivectors()?,
        finite_group: cl06_finite_group()?,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimeCarrier {
    pub slot: u8,
    pub semantic_atom_id: String,
    pub payload_cid: String,
    pub atom: PrimeAtom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimeBivectorSlot {
    pub index: u8,
    pub carrier_slots: [u8; 2],
    pub expert: SemiprimeExpert,
    pub bivector: Cl06Bivector,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SixPrimeOperatorChart {
    source_registry_kappa: String,
    operator_kappa: String,
    carriers: [PrimeCarrier; CL06_GENERATOR_COUNT],
    slots: [PrimeBivectorSlot; CL06_BIVECTOR_COUNT],
    chart_kappa: String,
}

#[derive(Serialize)]
struct PrimeCarrierWire<'a> {
    slot: u8,
    semantic_atom_id: &'a str,
    payload_cid: &'a str,
    prime: u32,
}

#[derive(Serialize)]
struct PrimeBivectorSlotWire {
    index: u8,
    carrier_slots: [u8; 2],
    prime_factors: [u32; 2],
    semiprime: u64,
    bivector_index: u8,
    bivector_pair: [u8; 2],
}

#[derive(Serialize)]
struct SixPrimeChartWire<'a> {
    schema: u32,
    domain: &'static str,
    source_registry_kappa: &'a str,
    operator_kappa: &'a str,
    chart_transport_status: &'static str,
    semantic_status: &'static str,
    carriers: Vec<PrimeCarrierWire<'a>>,
    slots: Vec<PrimeBivectorSlotWire>,
}

impl SixPrimeOperatorChart {
    /// Select the first six bindings in canonical registry order. The current
    /// registry contract assigns the sequential primes 5, 7, 11, 13, 17, 19;
    /// this constructor refuses any other prefix instead of inventing a chart
    /// rollover rule.
    pub fn from_registry(registry: &PrimeRegistry) -> Result<Self, SpiralCoreOperatorError> {
        registry
            .validate_canonical()
            .map_err(|error| SpiralCoreOperatorError::InvalidRegistry(error.to_string()))?;
        if registry.bindings.len() < CL06_GENERATOR_COUNT {
            return Err(SpiralCoreOperatorError::InvalidRegistry(format!(
                "six-prime chart requires at least {CL06_GENERATOR_COUNT} bindings"
            )));
        }
        let selected = &registry.bindings[..CL06_GENERATOR_COUNT];
        for (index, binding) in selected.iter().enumerate() {
            if binding.atom.value() != CANONICAL_SIX_PRIME_VALUES[index] {
                return Err(SpiralCoreOperatorError::InvalidRegistry(format!(
                    "carrier {index} is prime {}, expected canonical prime {}",
                    binding.atom.value(),
                    CANONICAL_SIX_PRIME_VALUES[index]
                )));
            }
            if binding.semantic_atom_id.trim().is_empty() {
                return Err(SpiralCoreOperatorError::InvalidRegistry(format!(
                    "carrier {index} has an empty semantic atom ID"
                )));
            }
            validate_blake3_label(&binding.payload_cid)?;
            if index > 0 && selected[index - 1].semantic_atom_id >= binding.semantic_atom_id {
                return Err(SpiralCoreOperatorError::InvalidRegistry(
                    "carrier semantic atom IDs are not in strict canonical order".to_owned(),
                ));
            }
        }

        let carriers: [PrimeCarrier; CL06_GENERATOR_COUNT] =
            std::array::from_fn(|index| PrimeCarrier {
                slot: index as u8,
                semantic_atom_id: selected[index].semantic_atom_id.clone(),
                payload_cid: selected[index].payload_cid.clone(),
                atom: selected[index].atom,
            });
        let bivectors = cl06_bivectors()?;
        let mut slots = Vec::with_capacity(CL06_BIVECTOR_COUNT);
        for left in 0..CL06_GENERATOR_COUNT {
            for right in left + 1..CL06_GENERATOR_COUNT {
                let index = slots.len();
                let expert = SemiprimeExpert::new(carriers[left].atom, carriers[right].atom)
                    .map_err(|error| SpiralCoreOperatorError::InvalidRegistry(error.to_string()))?;
                slots.push(PrimeBivectorSlot {
                    index: index as u8,
                    carrier_slots: [left as u8, right as u8],
                    expert,
                    bivector: bivectors[index],
                });
            }
        }
        let slots: [PrimeBivectorSlot; CL06_BIVECTOR_COUNT] =
            slots.try_into().map_err(|entries: Vec<_>| {
                SpiralCoreOperatorError::Invariant(format!(
                    "constructed {} chart slots instead of {CL06_BIVECTOR_COUNT}",
                    entries.len()
                ))
            })?;
        let operator_kappa = spiralcore_operator_kappa()?;
        let mut chart = Self {
            source_registry_kappa: registry.registry_kappa.clone(),
            operator_kappa,
            carriers,
            slots,
            chart_kappa: String::new(),
        };
        chart.chart_kappa = chart.reproduce_kappa()?;
        Ok(chart)
    }

    pub fn source_registry_kappa(&self) -> &str {
        &self.source_registry_kappa
    }

    pub fn operator_kappa(&self) -> &str {
        &self.operator_kappa
    }

    pub fn carriers(&self) -> &[PrimeCarrier; CL06_GENERATOR_COUNT] {
        &self.carriers
    }

    pub fn slots(&self) -> &[PrimeBivectorSlot; CL06_BIVECTOR_COUNT] {
        &self.slots
    }

    pub fn chart_kappa(&self) -> &str {
        &self.chart_kappa
    }

    pub fn reproduce_kappa(&self) -> Result<String, SpiralCoreOperatorError> {
        let carriers = self
            .carriers
            .iter()
            .map(|carrier| PrimeCarrierWire {
                slot: carrier.slot,
                semantic_atom_id: &carrier.semantic_atom_id,
                payload_cid: &carrier.payload_cid,
                prime: carrier.atom.value(),
            })
            .collect();
        let slots = self
            .slots
            .iter()
            .map(|slot| PrimeBivectorSlotWire {
                index: slot.index,
                carrier_slots: slot.carrier_slots,
                prime_factors: slot.expert.factors().map(PrimeAtom::value),
                semiprime: slot.expert.product(),
                bivector_index: slot.bivector.index,
                bivector_pair: slot.bivector.pair,
            })
            .collect();
        canonical_kappa(&SixPrimeChartWire {
            schema: SIX_PRIME_CHART_SCHEMA,
            domain: SIX_PRIME_CHART_DOMAIN,
            source_registry_kappa: &self.source_registry_kappa,
            operator_kappa: &self.operator_kappa,
            chart_transport_status: CHART_TRANSPORT_STATUS,
            semantic_status: OPERATOR_SEMANTIC_STATUS,
            carriers,
            slots,
        })
    }

    pub fn validate(&self) -> Result<(), SpiralCoreOperatorError> {
        validate_blake3_label(&self.source_registry_kappa)?;
        validate_blake3_label(&self.operator_kappa)?;
        validate_blake3_label(&self.chart_kappa)?;
        if self.operator_kappa != spiralcore_operator_kappa()? {
            return Err(SpiralCoreOperatorError::Invariant(
                "chart operator kappa does not reproduce".to_owned(),
            ));
        }
        for (index, carrier) in self.carriers.iter().enumerate() {
            if carrier.slot as usize != index
                || carrier.atom.value() != CANONICAL_SIX_PRIME_VALUES[index]
            {
                return Err(SpiralCoreOperatorError::Invariant(format!(
                    "carrier {index} changed canonical slot or prime"
                )));
            }
        }
        let expected_bivectors = cl06_bivectors()?;
        for (index, slot) in self.slots.iter().enumerate() {
            let [left, right] = slot.carrier_slots;
            if slot.index as usize != index
                || usize::from(left) >= CL06_GENERATOR_COUNT
                || usize::from(right) >= CL06_GENERATOR_COUNT
                || left >= right
                || slot.bivector != expected_bivectors[index]
            {
                return Err(SpiralCoreOperatorError::Invariant(format!(
                    "bivector slot {index} changed canonical ordering"
                )));
            }
            let expected_expert = SemiprimeExpert::new(
                self.carriers[usize::from(left)].atom,
                self.carriers[usize::from(right)].atom,
            )
            .map_err(|error| SpiralCoreOperatorError::Invariant(error.to_string()))?;
            if slot.expert != expected_expert {
                return Err(SpiralCoreOperatorError::Invariant(format!(
                    "bivector slot {index} changed semiprime factors"
                )));
            }
        }
        if self.chart_kappa != self.reproduce_kappa()? {
            return Err(SpiralCoreOperatorError::Invariant(
                "six-prime chart kappa does not reproduce".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpiralCoreOperatorValidation {
    pub left_anticommutators: usize,
    pub right_anticommutators: usize,
    pub bivectors: usize,
    pub commutators: usize,
    pub finite_group_size: usize,
    pub killing_diagonal: i32,
    pub operator_kappa: String,
}

/// Run the complete bounded source-free fixture set reproduced from v63.
pub fn validate_spiralcore_v63_operator(
) -> Result<SpiralCoreOperatorValidation, SpiralCoreOperatorError> {
    let mut expected_associator = [0i8; BASIS_DIMENSION];
    expected_associator[6] = -2;
    if octonion_basis_associator(1, 2, 3)? != expected_associator {
        return Err(SpiralCoreOperatorError::Invariant(
            "[e1,e2,e3] is not -2e6".to_owned(),
        ));
    }

    let left = cl06_left_generators()?;
    let right = cl06_right_generators()?;
    for generators in [&left, &right] {
        for first in 0..CL06_GENERATOR_COUNT {
            for second in 0..CL06_GENERATOR_COUNT {
                let observed = anticommutator(generators[first], generators[second])?;
                for (row, observed_row) in observed.iter().enumerate() {
                    for (column, entry) in observed_row.iter().enumerate() {
                        let expected = if first == second && row == column {
                            -2
                        } else {
                            0
                        };
                        if *entry != expected {
                            return Err(SpiralCoreOperatorError::Invariant(format!(
                                "Cl(0,6) anticommutator failed for generators {first},{second}"
                            )));
                        }
                    }
                }
            }
        }
    }

    if cl06_left_volume()? != octonion_left_basis_matrix(7)? {
        return Err(SpiralCoreOperatorError::Invariant(
            "L1...L6 is not L7".to_owned(),
        ));
    }
    if cl06_right_volume()? != octonion_right_basis_matrix(7)?.negated()? {
        return Err(SpiralCoreOperatorError::Invariant(
            "R1...R6 is not -R7".to_owned(),
        ));
    }

    let identity = SignedMatrix8::identity();
    let negative_identity = identity.negated()?;
    let bivectors = cl06_bivectors()?;
    for bivector in &bivectors {
        if bivector.matrix.checked_power(2)? != negative_identity {
            return Err(SpiralCoreOperatorError::Invariant(format!(
                "bivector {} does not square to -I",
                bivector.index
            )));
        }
        if bivector.matrix.checked_power(4)? != identity {
            return Err(SpiralCoreOperatorError::Invariant(format!(
                "bivector {} does not have fourth power I",
                bivector.index
            )));
        }
    }

    let killing = cl06_killing_form()?;
    for (row, killing_row) in killing.iter().enumerate() {
        for (column, value) in killing_row.iter().enumerate() {
            let expected = if row == column { -32 } else { 0 };
            if *value != expected {
                return Err(SpiralCoreOperatorError::Invariant(format!(
                    "Killing form entry ({row},{column}) is {value}, expected {expected}"
                )));
            }
        }
    }

    let group_size = cl06_finite_group()?.len();
    if group_size != 64 {
        return Err(SpiralCoreOperatorError::Invariant(format!(
            "finite bivector group size is {group_size}, expected 64"
        )));
    }

    let operator_kappa = spiralcore_operator_kappa()?;
    if operator_kappa != SPIRALCORE_OPERATOR_KAPPA_REFERENCE {
        return Err(SpiralCoreOperatorError::Invariant(format!(
            "operator kappa is {operator_kappa}, expected {SPIRALCORE_OPERATOR_KAPPA_REFERENCE}"
        )));
    }

    Ok(SpiralCoreOperatorValidation {
        left_anticommutators: CL06_GENERATOR_COUNT * CL06_GENERATOR_COUNT,
        right_anticommutators: CL06_GENERATOR_COUNT * CL06_GENERATOR_COUNT,
        bivectors: CL06_BIVECTOR_COUNT,
        commutators: CL06_BIVECTOR_COUNT * CL06_BIVECTOR_COUNT,
        finite_group_size: group_size,
        killing_diagonal: -32,
        operator_kappa,
    })
}
