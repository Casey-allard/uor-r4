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

use std::collections::{BTreeMap, BTreeSet, VecDeque};

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
pub const CL06_FINITE_COMPOSITION_SCHEMA: u32 = 1;
pub const CL06_FINITE_COMPOSITION_DOMAIN: &str = "uor-r4.spiralcore-v63-cl06-finite-composition/1";
pub const CL06_FINITE_COMPOSITION_KAPPA_REFERENCE: &str =
    "blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0";
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
pub const CL06_FINITE_GROUP_ORDER: usize = 64;

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

/// A complete finite composition table over the exact discovery order returned
/// by [`cl06_finite_group`]. An entry at `[left][right]` is the index of the
/// matrix product `states[left] * states[right]`; the operand order is part of
/// the control contract because this group is noncommutative.
///
/// Matrix multiplication is used only while compiling and validating this
/// table. Once compiled, composition and inversion are bounded integer-indexed
/// table reads. The table is an optional order-sensitive control and carries no
/// semantic claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cl06FiniteCompositionTable {
    states: [SignedMatrix8; CL06_FINITE_GROUP_ORDER],
    composition_indexes: [[u8; CL06_FINITE_GROUP_ORDER]; CL06_FINITE_GROUP_ORDER],
    identity_index: u8,
    inverse_indexes: [u8; CL06_FINITE_GROUP_ORDER],
    composition_kappa: String,
}

impl Cl06FiniteCompositionTable {
    pub fn states(&self) -> &[SignedMatrix8; CL06_FINITE_GROUP_ORDER] {
        &self.states
    }

    pub fn composition_indexes(&self) -> &[[u8; CL06_FINITE_GROUP_ORDER]; CL06_FINITE_GROUP_ORDER] {
        &self.composition_indexes
    }

    pub const fn identity_index(&self) -> u8 {
        self.identity_index
    }

    pub fn inverse_indexes(&self) -> &[u8; CL06_FINITE_GROUP_ORDER] {
        &self.inverse_indexes
    }

    pub fn composition_kappa(&self) -> &str {
        &self.composition_kappa
    }

    /// Return the exact index of `states[left] * states[right]`.
    pub fn compose_index(&self, left: u8, right: u8) -> Option<u8> {
        self.composition_indexes
            .get(usize::from(left))
            .and_then(|row| row.get(usize::from(right)))
            .copied()
    }

    pub fn inverse_index(&self, state: u8) -> Option<u8> {
        self.inverse_indexes.get(usize::from(state)).copied()
    }

    pub fn state(&self, index: u8) -> Option<&SignedMatrix8> {
        self.states.get(usize::from(index))
    }

    /// Reproduce the canonical identity of the ordered finite table. The
    /// identity transitively binds the pre-existing exact operator convention,
    /// including both left and right action matrices, without changing that
    /// convention's pinned kappa.
    pub fn reproduce_kappa(&self) -> Result<String, SpiralCoreOperatorError> {
        let operator_kappa = spiralcore_operator_kappa()?;
        let composition_indexes = self
            .composition_indexes
            .iter()
            .map(|row| row.as_slice())
            .collect();
        canonical_kappa(&Cl06FiniteCompositionWire {
            schema: CL06_FINITE_COMPOSITION_SCHEMA,
            domain: CL06_FINITE_COMPOSITION_DOMAIN,
            operator_kappa: &operator_kappa,
            semantic_status: OPERATOR_SEMANTIC_STATUS,
            group_order: CL06_FINITE_GROUP_ORDER,
            states: self.states.as_slice(),
            composition_indexes,
            identity_index: self.identity_index,
            inverse_indexes: self.inverse_indexes.as_slice(),
        })
    }

    /// Recheck every compiled entry and all finite-group laws without relying
    /// on floating-point arithmetic or approximate equality.
    pub fn validate(&self) -> Result<Cl06FiniteCompositionValidation, SpiralCoreOperatorError> {
        let unique_states = self.states.iter().copied().collect::<BTreeSet<_>>();
        if unique_states.len() != CL06_FINITE_GROUP_ORDER {
            return Err(SpiralCoreOperatorError::Invariant(format!(
                "composition table has {} unique states, expected {CL06_FINITE_GROUP_ORDER}",
                unique_states.len()
            )));
        }
        if self.state(self.identity_index) != Some(&SignedMatrix8::identity()) {
            return Err(SpiralCoreOperatorError::Invariant(format!(
                "composition identity index {} does not name I",
                self.identity_index
            )));
        }

        let mut noncommuting_ordered_pairs = 0usize;
        for left in 0..CL06_FINITE_GROUP_ORDER {
            for right in 0..CL06_FINITE_GROUP_ORDER {
                let product_index = usize::from(self.composition_indexes[left][right]);
                let Some(product_state) = self.states.get(product_index) else {
                    return Err(SpiralCoreOperatorError::Invariant(format!(
                        "composition entry ({left},{right}) names out-of-range state {product_index}"
                    )));
                };
                if self.states[left].checked_mul(self.states[right])? != *product_state {
                    return Err(SpiralCoreOperatorError::Invariant(format!(
                        "composition entry ({left},{right}) does not reproduce the exact matrix product"
                    )));
                }
                if self.composition_indexes[left][right] != self.composition_indexes[right][left] {
                    noncommuting_ordered_pairs += 1;
                }
            }
        }
        if noncommuting_ordered_pairs == 0 {
            return Err(SpiralCoreOperatorError::Invariant(
                "finite composition table unexpectedly commutes".to_owned(),
            ));
        }

        let operator_kappa = spiralcore_operator_kappa()?;
        let reproduced_kappa = self.reproduce_kappa()?;
        if self.composition_kappa != reproduced_kappa {
            return Err(SpiralCoreOperatorError::Invariant(format!(
                "finite composition kappa is {}, expected {reproduced_kappa}",
                self.composition_kappa
            )));
        }
        if self.composition_kappa != CL06_FINITE_COMPOSITION_KAPPA_REFERENCE {
            return Err(SpiralCoreOperatorError::Invariant(format!(
                "finite composition kappa is {}, expected {CL06_FINITE_COMPOSITION_KAPPA_REFERENCE}",
                self.composition_kappa
            )));
        }

        let identity = usize::from(self.identity_index);
        for state in 0..CL06_FINITE_GROUP_ORDER {
            if usize::from(self.composition_indexes[identity][state]) != state
                || usize::from(self.composition_indexes[state][identity]) != state
            {
                return Err(SpiralCoreOperatorError::Invariant(format!(
                    "state {state} does not preserve the two-sided identity"
                )));
            }

            let inverse = usize::from(self.inverse_indexes[state]);
            if inverse >= CL06_FINITE_GROUP_ORDER
                || usize::from(self.composition_indexes[state][inverse]) != identity
                || usize::from(self.composition_indexes[inverse][state]) != identity
            {
                return Err(SpiralCoreOperatorError::Invariant(format!(
                    "state {state} has invalid two-sided inverse index {inverse}"
                )));
            }
            let inverse_count = (0..CL06_FINITE_GROUP_ORDER)
                .filter(|candidate| {
                    usize::from(self.composition_indexes[state][*candidate]) == identity
                        && usize::from(self.composition_indexes[*candidate][state]) == identity
                })
                .count();
            if inverse_count != 1 {
                return Err(SpiralCoreOperatorError::Invariant(format!(
                    "state {state} has {inverse_count} two-sided inverses, expected exactly one"
                )));
            }
        }

        for first in 0..CL06_FINITE_GROUP_ORDER {
            for second in 0..CL06_FINITE_GROUP_ORDER {
                for third in 0..CL06_FINITE_GROUP_ORDER {
                    let first_second = usize::from(self.composition_indexes[first][second]);
                    let second_third = usize::from(self.composition_indexes[second][third]);
                    let left_associated = self.composition_indexes[first_second][third];
                    let right_associated = self.composition_indexes[first][second_third];
                    if left_associated != right_associated {
                        return Err(SpiralCoreOperatorError::Invariant(format!(
                            "composition is not associative at ({first},{second},{third})"
                        )));
                    }
                }
            }
        }

        Ok(Cl06FiniteCompositionValidation {
            unique_states: unique_states.len(),
            composition_entries: CL06_FINITE_GROUP_ORDER * CL06_FINITE_GROUP_ORDER,
            associativity_checks: CL06_FINITE_GROUP_ORDER
                * CL06_FINITE_GROUP_ORDER
                * CL06_FINITE_GROUP_ORDER,
            two_sided_inverses: CL06_FINITE_GROUP_ORDER,
            noncommuting_ordered_pairs,
            identity_index: self.identity_index,
            operator_kappa,
            composition_kappa: self.composition_kappa.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cl06FiniteCompositionValidation {
    pub unique_states: usize,
    pub composition_entries: usize,
    pub associativity_checks: usize,
    pub two_sided_inverses: usize,
    pub noncommuting_ordered_pairs: usize,
    pub identity_index: u8,
    pub operator_kappa: String,
    pub composition_kappa: String,
}

#[derive(Serialize)]
struct Cl06FiniteCompositionWire<'a> {
    schema: u32,
    domain: &'static str,
    operator_kappa: &'a str,
    semantic_status: &'static str,
    group_order: usize,
    states: &'a [SignedMatrix8],
    composition_indexes: Vec<&'a [u8]>,
    identity_index: u8,
    inverse_indexes: &'a [u8],
}

/// Compile the exact 64-state SpiralCore control into an integer-indexed
/// composition table. State ordering is deterministic breadth-first discovery
/// from `I` through the existing lexicographically ordered left bivectors.
pub fn cl06_finite_composition_table() -> Result<Cl06FiniteCompositionTable, SpiralCoreOperatorError>
{
    let states: [SignedMatrix8; CL06_FINITE_GROUP_ORDER] =
        cl06_finite_group()?.try_into().map_err(|states: Vec<_>| {
            SpiralCoreOperatorError::Invariant(format!(
                "finite bivector group has {} states, expected {CL06_FINITE_GROUP_ORDER}",
                states.len()
            ))
        })?;
    let mut state_indexes = BTreeMap::new();
    for (index, state) in states.iter().enumerate() {
        if state_indexes.insert(*state, index as u8).is_some() {
            return Err(SpiralCoreOperatorError::Invariant(format!(
                "finite bivector group repeats state {index}"
            )));
        }
    }
    let identity_index = *state_indexes
        .get(&SignedMatrix8::identity())
        .ok_or_else(|| {
            SpiralCoreOperatorError::Invariant(
                "finite bivector group omits the identity".to_owned(),
            )
        })?;

    let mut composition_indexes = [[0u8; CL06_FINITE_GROUP_ORDER]; CL06_FINITE_GROUP_ORDER];
    for (left_index, left) in states.iter().enumerate() {
        for (right_index, right) in states.iter().enumerate() {
            let product = left.checked_mul(*right)?;
            composition_indexes[left_index][right_index] =
                *state_indexes.get(&product).ok_or_else(|| {
                    SpiralCoreOperatorError::Invariant(format!(
                        "finite bivector group is not closed at ({left_index},{right_index})"
                    ))
                })?;
        }
    }

    let identity = usize::from(identity_index);
    let mut inverse_indexes = [0u8; CL06_FINITE_GROUP_ORDER];
    for (state, inverse_index) in inverse_indexes.iter_mut().enumerate() {
        let mut inverse = None;
        for candidate in 0..CL06_FINITE_GROUP_ORDER {
            if usize::from(composition_indexes[state][candidate]) == identity
                && usize::from(composition_indexes[candidate][state]) == identity
            {
                if inverse.replace(candidate as u8).is_some() {
                    return Err(SpiralCoreOperatorError::Invariant(format!(
                        "state {state} has multiple two-sided inverses"
                    )));
                }
            }
        }
        *inverse_index = inverse.ok_or_else(|| {
            SpiralCoreOperatorError::Invariant(format!("state {state} has no two-sided inverse"))
        })?;
    }

    let mut table = Cl06FiniteCompositionTable {
        states,
        composition_indexes,
        identity_index,
        inverse_indexes,
        composition_kappa: String::new(),
    };
    table.composition_kappa = table.reproduce_kappa()?;
    table.validate()?;
    Ok(table)
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

#[cfg(test)]
mod finite_composition_table_tests {
    use super::*;

    #[test]
    fn exact_finite_composition_table_is_complete_associative_and_ordered() {
        let table = cl06_finite_composition_table().expect("exact finite composition table");
        let report = table.validate().expect("complete group-law validation");

        assert_eq!(report.unique_states, CL06_FINITE_GROUP_ORDER);
        assert_eq!(report.composition_entries, 64 * 64);
        assert_eq!(report.associativity_checks, 64 * 64 * 64);
        assert_eq!(report.two_sided_inverses, CL06_FINITE_GROUP_ORDER);
        assert!(report.noncommuting_ordered_pairs > 0);
        assert_eq!(report.identity_index, 0);
        assert_eq!(report.operator_kappa, spiralcore_operator_kappa().unwrap());
        assert_eq!(report.composition_kappa, table.composition_kappa());
        assert_eq!(table.composition_kappa(), table.reproduce_kappa().unwrap());
        assert_eq!(
            table.composition_kappa(),
            CL06_FINITE_COMPOSITION_KAPPA_REFERENCE
        );
        assert_eq!(
            table.state(report.identity_index),
            Some(&SignedMatrix8::identity())
        );
        assert_eq!(
            table.states().as_slice(),
            cl06_finite_group()
                .expect("existing finite group")
                .as_slice()
        );

        for state in 0..CL06_FINITE_GROUP_ORDER as u8 {
            let inverse = table.inverse_index(state).expect("bounded state index");
            assert_eq!(
                table.compose_index(state, inverse),
                Some(table.identity_index())
            );
            assert_eq!(
                table.compose_index(inverse, state),
                Some(table.identity_index())
            );
        }

        let (left, right) = (0..CL06_FINITE_GROUP_ORDER as u8)
            .find_map(|left| {
                (0..CL06_FINITE_GROUP_ORDER as u8).find_map(|right| {
                    (table.compose_index(left, right) != table.compose_index(right, left))
                        .then_some((left, right))
                })
            })
            .expect("the exact finite group is noncommutative");
        let left_then_right = table
            .compose_index(left, right)
            .expect("bounded composition indexes");
        let right_then_left = table
            .compose_index(right, left)
            .expect("bounded composition indexes");
        assert_ne!(left_then_right, right_then_left);
        assert_eq!(
            table.state(left_then_right),
            Some(
                &table
                    .state(left)
                    .expect("left state")
                    .checked_mul(*table.state(right).expect("right state"))
                    .expect("exact signed matrix product")
            )
        );
        assert_eq!(table.compose_index(CL06_FINITE_GROUP_ORDER as u8, 0), None);
        assert_eq!(table.inverse_index(CL06_FINITE_GROUP_ORDER as u8), None);
        assert_eq!(table.state(CL06_FINITE_GROUP_ORDER as u8), None);
        assert_eq!(
            table,
            cl06_finite_composition_table().expect("deterministic recompile")
        );
        assert_eq!(OPERATOR_SEMANTIC_STATUS, "OPTIONAL_CONTROL_PENDING");
    }
}
