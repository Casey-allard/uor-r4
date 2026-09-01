//! Exact group-address and transport artifact for `R4GroupAddressedRetentionLMV1`.
//!
//! This module owns no language-model arithmetic. It freezes the exact H4 and
//! identity-reindexed C120 operations, the independently versioned token-leaf
//! policy, and the transport-only scrambled-H4 control consumed by the bounded
//! #973 compiler experiment.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::canonical_lexical_ingestion::{
    validate_h4_binary_icosahedral_closure, CanonicalLexicalError, H4BinaryIcosahedralClosure,
};
use crate::corpus_induced_spin_placement::first_primes;

pub const R4_GROUP_GEOMETRY_SCHEMA: u32 = 1;
pub const R4_GROUP_GEOMETRY_DOMAIN: &str = "uor-r4.group-addressed-retention-geometry/1";
pub const R4_GROUP_LEAF_MAP_SCHEMA: u32 = 1;
pub const R4_GROUP_LEAF_MAP_DOMAIN: &str = "uor-r4.group-addressed-retention-prime-leaf-map/1";
pub const R4_GROUP_SCRAMBLE_SCHEMA: u32 = 1;
pub const R4_GROUP_SCRAMBLE_DOMAIN: &str = "uor-r4.group-addressed-retention-transport-scramble/1";
pub const R4_GROUP_ORDER: usize = 120;
pub const R4_GROUP_MAX_TOKEN_ID: u16 = 4095;
pub const R4_GROUP_LEAF_POLICY: &str =
    "BOS token 0 maps to the exact H4 identity; token t>0 maps to zero-based prime p_(t-1) mod 120";
pub const R4_GROUP_SCRAMBLE_POLICY: &str = "identity-fixing deterministic rotation within each exact H4 element-order class; candidate leaves remain true and only transport actions use pi(leaf)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum R4GroupGeometryError {
    Invalid(String),
    Canonical(String),
    Serialization(String),
    ArithmeticOverflow,
}

impl std::fmt::Display for R4GroupGeometryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::Canonical(reason) => write!(formatter, "canonical H4 table: {reason}"),
            Self::Serialization(reason) => write!(formatter, "canonical JSON: {reason}"),
            Self::ArithmeticOverflow => {
                formatter.write_str("group-address geometry arithmetic overflow")
            }
        }
    }
}

impl std::error::Error for R4GroupGeometryError {}

impl From<CanonicalLexicalError> for R4GroupGeometryError {
    fn from(error: CanonicalLexicalError) -> Self {
        Self::Canonical(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4GroupLeafMapV1 {
    pub schema: u32,
    pub domain: String,
    pub policy: String,
    pub max_token_id: u16,
    /// Indexed directly by token ID. These are the true candidate addresses.
    pub leaf_indices: Vec<u16>,
    pub direct_support_indices: Vec<u16>,
    pub direct_support_count: u16,
    /// BLAKE3 of this object serialized canonically with this field empty.
    pub leaf_cid: String,
}

impl R4GroupLeafMapV1 {
    fn seed_bytes(&self) -> Result<Vec<u8>, R4GroupGeometryError> {
        let mut seed = self.clone();
        seed.leaf_cid.clear();
        canonical_json(&seed)
    }

    pub fn reproduce_leaf_cid(&self) -> Result<String, R4GroupGeometryError> {
        Ok(blake3_cid(&self.seed_bytes()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4GroupNonhomomorphismWitnessV1 {
    pub left: u16,
    pub right: u16,
    pub true_product: u16,
    pub permuted_product: u16,
    pub product_of_permuted: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4GroupOrderHistogramEntryV1 {
    pub element_order: u16,
    pub distinct_actions: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4GroupTransportScrambleV1 {
    pub schema: u32,
    pub domain: String,
    pub policy: String,
    /// True H4 table index -> transport-action H4 table index.
    pub permutation: Vec<u16>,
    /// Indexed by token ID. Candidate addresses remain `leaf_map.leaf_indices`.
    pub transport_leaf_indices: Vec<u16>,
    pub moved_count: u16,
    pub element_orders: Vec<u16>,
    pub identity_fixed: bool,
    pub element_orders_preserved: bool,
    pub used_leaf_order_histogram: Vec<R4GroupOrderHistogramEntryV1>,
    pub scrambled_used_action_order_histogram: Vec<R4GroupOrderHistogramEntryV1>,
    pub nonhomomorphism_witness: R4GroupNonhomomorphismWitnessV1,
    pub used_action_generated_subgroup_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4GroupCoverageCensusesV1 {
    pub direct_leaf_support_indices: Vec<u16>,
    pub direct_leaf_support_count: u16,
    pub direct_nonidentity_leaf_support_count: u16,
    pub identity_token_count: u16,
    pub h4_generated_subgroup_indices: Vec<u16>,
    pub h4_generated_subgroup_count: u16,
    pub c120_generated_subgroup_indices: Vec<u16>,
    pub c120_generated_subgroup_count: u16,
    pub scrambled_h4_generated_subgroup_indices: Vec<u16>,
    pub scrambled_h4_generated_subgroup_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct R4GroupGeometryArtifactV1 {
    pub schema: u32,
    pub domain: String,
    pub max_token_id: u16,
    pub group_order: u16,
    pub h4_root_table_kappa: String,
    pub h4_multiplication_table_kappa: String,
    pub identity_index: u16,
    pub inverse_indices: Vec<u16>,
    pub h4_multiplication_indices: Vec<u16>,
    pub c120_inverse_indices: Vec<u16>,
    pub c120_multiplication_indices: Vec<u16>,
    /// Row `g` is the exact permutation `h -> g*h`.
    pub h4_left_regular_permutations: Vec<Vec<u16>>,
    /// Row `g` is the exact identity-reindexed cyclic permutation `h -> g (+) h`.
    pub c120_left_regular_permutations: Vec<Vec<u16>>,
    pub leaf_map: R4GroupLeafMapV1,
    pub scramble: R4GroupTransportScrambleV1,
    pub censuses: R4GroupCoverageCensusesV1,
    /// BLAKE3 of this object serialized canonically with this field empty.
    pub artifact_cid: String,
}

impl R4GroupGeometryArtifactV1 {
    pub fn build(max_token_id: u16) -> Result<Self, R4GroupGeometryError> {
        let artifact = Self::build_unvalidated(max_token_id)?;
        artifact.validate()?;
        Ok(artifact)
    }

    fn build_unvalidated(max_token_id: u16) -> Result<Self, R4GroupGeometryError> {
        if max_token_id != R4_GROUP_MAX_TOKEN_ID {
            return Err(R4GroupGeometryError::Invalid(format!(
                "R4GroupAddressedRetentionLMV1 requires max token ID {R4_GROUP_MAX_TOKEN_ID}, got {max_token_id}"
            )));
        }
        let h4 = validate_h4_binary_icosahedral_closure()?;
        validate_h4(&h4)?;
        let identity = h4.identity_index;

        let c120_multiplication_indices = c120_table(identity)?;
        let c120_inverse_indices = inverse_table(&c120_multiplication_indices, identity)?;
        let h4_left_regular_permutations = left_regular_permutations(&h4.multiplication_indices)?;
        let c120_left_regular_permutations =
            left_regular_permutations(&c120_multiplication_indices)?;

        let leaf_indices = build_leaf_indices(max_token_id, identity)?;
        let direct_support_indices = sorted_unique(&leaf_indices);
        let direct_support_count = u16_len(&direct_support_indices)?;
        let mut leaf_map = R4GroupLeafMapV1 {
            schema: R4_GROUP_LEAF_MAP_SCHEMA,
            domain: R4_GROUP_LEAF_MAP_DOMAIN.to_owned(),
            policy: R4_GROUP_LEAF_POLICY.to_owned(),
            max_token_id,
            leaf_indices,
            direct_support_indices: direct_support_indices.clone(),
            direct_support_count,
            leaf_cid: String::new(),
        };
        leaf_map.leaf_cid = leaf_map.reproduce_leaf_cid()?;

        let element_orders = element_orders(&h4.multiplication_indices, identity)?;
        let permutation = order_preserving_scramble(&element_orders, identity)?;
        let transport_leaf_indices = leaf_map
            .leaf_indices
            .iter()
            .map(|&leaf| permutation.get(usize::from(leaf)).copied())
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                R4GroupGeometryError::Invalid(
                    "leaf index is outside the transport scramble".to_owned(),
                )
            })?;
        let nonhomomorphism_witness =
            find_nonhomomorphism_witness(&h4.multiplication_indices, &permutation)?;

        let h4_generated_subgroup_indices = generated_subgroup(
            &h4.multiplication_indices,
            identity,
            &direct_support_indices,
        )?;
        let c120_generated_subgroup_indices = generated_subgroup(
            &c120_multiplication_indices,
            identity,
            &direct_support_indices,
        )?;
        let scrambled_support_indices = sorted_unique(&transport_leaf_indices);
        let scrambled_h4_generated_subgroup_indices = generated_subgroup(
            &h4.multiplication_indices,
            identity,
            &scrambled_support_indices,
        )?;
        require_full_coverage("H4", &h4_generated_subgroup_indices)?;
        require_full_coverage("identity-reindexed C120", &c120_generated_subgroup_indices)?;
        require_full_coverage(
            "scrambled H4 used actions",
            &scrambled_h4_generated_subgroup_indices,
        )?;

        let used_leaf_order_histogram = order_histogram(&direct_support_indices, &element_orders)?;
        let scrambled_used_action_order_histogram =
            order_histogram(&scrambled_support_indices, &element_orders)?;
        if used_leaf_order_histogram != scrambled_used_action_order_histogram {
            return Err(R4GroupGeometryError::Invalid(
                "transport scramble changed the distinct used-action element-order histogram"
                    .to_owned(),
            ));
        }
        let moved_count = u16_len(
            &permutation
                .iter()
                .enumerate()
                .filter(|(index, mapped)| usize::from(**mapped) != *index)
                .collect::<Vec<_>>(),
        )?;
        let identity_fixed = permutation.get(usize::from(identity)) == Some(&identity);
        let element_orders_preserved = permutation.iter().enumerate().all(|(source, &target)| {
            element_orders.get(source) == element_orders.get(usize::from(target))
        });
        if !identity_fixed || !element_orders_preserved || usize::from(moved_count) < 100 {
            return Err(R4GroupGeometryError::Invalid(format!(
                "transport scramble is not a broad identity-fixing order-preserving permutation (moved={moved_count})"
            )));
        }

        let identity_token_count = u16_len(
            &leaf_map
                .leaf_indices
                .iter()
                .filter(|&&leaf| leaf == identity)
                .collect::<Vec<_>>(),
        )?;
        let direct_nonidentity_leaf_support_count = u16_len(
            &direct_support_indices
                .iter()
                .filter(|&&leaf| leaf != identity)
                .collect::<Vec<_>>(),
        )?;
        let h4_generated_subgroup_count = u16_len(&h4_generated_subgroup_indices)?;
        let c120_generated_subgroup_count = u16_len(&c120_generated_subgroup_indices)?;
        let scrambled_h4_generated_subgroup_count =
            u16_len(&scrambled_h4_generated_subgroup_indices)?;

        let scramble = R4GroupTransportScrambleV1 {
            schema: R4_GROUP_SCRAMBLE_SCHEMA,
            domain: R4_GROUP_SCRAMBLE_DOMAIN.to_owned(),
            policy: R4_GROUP_SCRAMBLE_POLICY.to_owned(),
            permutation,
            transport_leaf_indices,
            moved_count,
            element_orders,
            identity_fixed,
            element_orders_preserved,
            used_leaf_order_histogram,
            scrambled_used_action_order_histogram,
            nonhomomorphism_witness,
            used_action_generated_subgroup_count: scrambled_h4_generated_subgroup_count,
        };
        let censuses = R4GroupCoverageCensusesV1 {
            direct_leaf_support_indices: direct_support_indices,
            direct_leaf_support_count: direct_support_count,
            direct_nonidentity_leaf_support_count,
            identity_token_count,
            h4_generated_subgroup_indices,
            h4_generated_subgroup_count,
            c120_generated_subgroup_indices,
            c120_generated_subgroup_count,
            scrambled_h4_generated_subgroup_indices,
            scrambled_h4_generated_subgroup_count,
        };
        let mut artifact = Self {
            schema: R4_GROUP_GEOMETRY_SCHEMA,
            domain: R4_GROUP_GEOMETRY_DOMAIN.to_owned(),
            max_token_id,
            group_order: u16::try_from(R4_GROUP_ORDER)
                .map_err(|_| R4GroupGeometryError::ArithmeticOverflow)?,
            h4_root_table_kappa: h4.h4_root_table_kappa,
            h4_multiplication_table_kappa: h4.multiplication_table_kappa,
            identity_index: identity,
            inverse_indices: h4.inverse_indices,
            h4_multiplication_indices: h4.multiplication_indices,
            c120_inverse_indices,
            c120_multiplication_indices,
            h4_left_regular_permutations,
            c120_left_regular_permutations,
            leaf_map,
            scramble,
            censuses,
            artifact_cid: String::new(),
        };
        artifact.artifact_cid = artifact.reproduce_artifact_cid()?;
        Ok(artifact)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, R4GroupGeometryError> {
        canonical_json(self)
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, R4GroupGeometryError> {
        let artifact: Self = serde_json::from_slice(bytes)
            .map_err(|error| R4GroupGeometryError::Serialization(error.to_string()))?;
        artifact.validate()?;
        if artifact.canonical_bytes()? != bytes {
            return Err(R4GroupGeometryError::Invalid(
                "geometry artifact bytes are valid JSON but not canonical JSON".to_owned(),
            ));
        }
        Ok(artifact)
    }

    pub fn reproduce_artifact_cid(&self) -> Result<String, R4GroupGeometryError> {
        let mut seed = self.clone();
        seed.artifact_cid.clear();
        Ok(blake3_cid(&canonical_json(&seed)?))
    }

    pub fn validate(&self) -> Result<(), R4GroupGeometryError> {
        if self.schema != R4_GROUP_GEOMETRY_SCHEMA
            || self.domain != R4_GROUP_GEOMETRY_DOMAIN
            || self.max_token_id != R4_GROUP_MAX_TOKEN_ID
            || usize::from(self.group_order) != R4_GROUP_ORDER
        {
            return Err(R4GroupGeometryError::Invalid(
                "geometry artifact schema/domain/bounds do not match the frozen contract"
                    .to_owned(),
            ));
        }
        let expected = Self::build_unvalidated_reference()?;
        if self != &expected {
            return Err(R4GroupGeometryError::Invalid(
                "geometry artifact does not reproduce the canonical frozen artifact".to_owned(),
            ));
        }
        Ok(())
    }

    // Build without recursively invoking `validate`.
    fn build_unvalidated_reference() -> Result<Self, R4GroupGeometryError> {
        Self::build_unvalidated(R4_GROUP_MAX_TOKEN_ID)
    }
}

fn validate_h4(table: &H4BinaryIcosahedralClosure) -> Result<(), R4GroupGeometryError> {
    let expected_products = R4_GROUP_ORDER
        .checked_mul(R4_GROUP_ORDER)
        .ok_or(R4GroupGeometryError::ArithmeticOverflow)?;
    if table.root_count != R4_GROUP_ORDER
        || table.product_count != expected_products
        || table.inverse_indices.len() != R4_GROUP_ORDER
        || table.multiplication_indices.len() != expected_products
        || usize::from(table.identity_index) >= R4_GROUP_ORDER
        || !table.unique_closure_exact
        || !table.identity_exact
        || !table.inverses_exact
        || !table.associativity_exact
        || !table.integer_only_no_rounding
    {
        return Err(R4GroupGeometryError::Invalid(
            "canonical H4 closure is incomplete or not exact".to_owned(),
        ));
    }
    Ok(())
}

fn build_leaf_indices(max_token_id: u16, identity: u16) -> Result<Vec<u16>, R4GroupGeometryError> {
    let count = usize::from(max_token_id)
        .checked_add(1)
        .ok_or(R4GroupGeometryError::ArithmeticOverflow)?;
    let non_bos_count = count
        .checked_sub(1)
        .ok_or(R4GroupGeometryError::ArithmeticOverflow)?;
    let primes = first_primes(non_bos_count).map_err(|error| {
        R4GroupGeometryError::Invalid(format!("canonical first-prime table: {error}"))
    })?;
    let modulus =
        u64::try_from(R4_GROUP_ORDER).map_err(|_| R4GroupGeometryError::ArithmeticOverflow)?;
    let mut leaves = Vec::with_capacity(count);
    leaves.push(identity);
    for token in 1..count {
        let prime_index = token
            .checked_sub(1)
            .ok_or(R4GroupGeometryError::ArithmeticOverflow)?;
        let prime = *primes.get(prime_index).ok_or_else(|| {
            R4GroupGeometryError::Invalid("canonical first-prime table is truncated".to_owned())
        })?;
        leaves.push(
            u16::try_from(prime % modulus).map_err(|_| R4GroupGeometryError::ArithmeticOverflow)?,
        );
    }
    Ok(leaves)
}

fn c120_table(identity: u16) -> Result<Vec<u16>, R4GroupGeometryError> {
    let identity = usize::from(identity);
    if identity >= R4_GROUP_ORDER {
        return Err(R4GroupGeometryError::Invalid(
            "C120 identity offset is outside the group".to_owned(),
        ));
    }
    let mut table = Vec::with_capacity(R4_GROUP_ORDER * R4_GROUP_ORDER);
    for left in 0..R4_GROUP_ORDER {
        let left_delta = (left + R4_GROUP_ORDER - identity) % R4_GROUP_ORDER;
        for right in 0..R4_GROUP_ORDER {
            let right_delta = (right + R4_GROUP_ORDER - identity) % R4_GROUP_ORDER;
            let product = (identity + left_delta + right_delta) % R4_GROUP_ORDER;
            table.push(
                u16::try_from(product).map_err(|_| R4GroupGeometryError::ArithmeticOverflow)?,
            );
        }
    }
    Ok(table)
}

fn inverse_table(table: &[u16], identity: u16) -> Result<Vec<u16>, R4GroupGeometryError> {
    validate_table_shape(table)?;
    let mut inverses = Vec::with_capacity(R4_GROUP_ORDER);
    for element in 0..R4_GROUP_ORDER {
        let element =
            u16::try_from(element).map_err(|_| R4GroupGeometryError::ArithmeticOverflow)?;
        let matches = (0..R4_GROUP_ORDER)
            .map(|candidate| {
                u16::try_from(candidate).map_err(|_| R4GroupGeometryError::ArithmeticOverflow)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|&candidate| {
                table_product(table, element, candidate) == Some(identity)
                    && table_product(table, candidate, element) == Some(identity)
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(R4GroupGeometryError::Invalid(format!(
                "group element {element} has {} two-sided inverses",
                matches.len()
            )));
        }
        inverses.push(matches[0]);
    }
    Ok(inverses)
}

fn left_regular_permutations(table: &[u16]) -> Result<Vec<Vec<u16>>, R4GroupGeometryError> {
    validate_table_shape(table)?;
    let expected = (0..R4_GROUP_ORDER)
        .map(|value| u16::try_from(value).map_err(|_| R4GroupGeometryError::ArithmeticOverflow))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut rows = Vec::with_capacity(R4_GROUP_ORDER);
    for left in 0..R4_GROUP_ORDER {
        let start = left
            .checked_mul(R4_GROUP_ORDER)
            .ok_or(R4GroupGeometryError::ArithmeticOverflow)?;
        let end = start
            .checked_add(R4_GROUP_ORDER)
            .ok_or(R4GroupGeometryError::ArithmeticOverflow)?;
        let row = table
            .get(start..end)
            .ok_or_else(|| R4GroupGeometryError::Invalid("group row is truncated".to_owned()))?
            .to_vec();
        if row.iter().copied().collect::<BTreeSet<_>>() != expected {
            return Err(R4GroupGeometryError::Invalid(format!(
                "left-regular row {left} is not a permutation"
            )));
        }
        rows.push(row);
    }
    Ok(rows)
}

fn element_orders(table: &[u16], identity: u16) -> Result<Vec<u16>, R4GroupGeometryError> {
    validate_table_shape(table)?;
    let mut orders = Vec::with_capacity(R4_GROUP_ORDER);
    for element in 0..R4_GROUP_ORDER {
        let element =
            u16::try_from(element).map_err(|_| R4GroupGeometryError::ArithmeticOverflow)?;
        let mut value = identity;
        let mut found = None;
        for exponent in 1..=R4_GROUP_ORDER {
            value = table_product(table, value, element).ok_or_else(|| {
                R4GroupGeometryError::Invalid("element-order product is out of range".to_owned())
            })?;
            if value == identity {
                found = Some(
                    u16::try_from(exponent)
                        .map_err(|_| R4GroupGeometryError::ArithmeticOverflow)?,
                );
                break;
            }
        }
        orders.push(found.ok_or_else(|| {
            R4GroupGeometryError::Invalid(format!(
                "H4 element {element} has no order at most {R4_GROUP_ORDER}"
            ))
        })?);
    }
    Ok(orders)
}

fn order_preserving_scramble(
    orders: &[u16],
    identity: u16,
) -> Result<Vec<u16>, R4GroupGeometryError> {
    if orders.len() != R4_GROUP_ORDER {
        return Err(R4GroupGeometryError::Invalid(
            "element-order vector has the wrong length".to_owned(),
        ));
    }
    let mut buckets = BTreeMap::<u16, Vec<u16>>::new();
    for (index, &order) in orders.iter().enumerate() {
        buckets
            .entry(order)
            .or_default()
            .push(u16::try_from(index).map_err(|_| R4GroupGeometryError::ArithmeticOverflow)?);
    }
    let mut permutation = vec![0_u16; R4_GROUP_ORDER];
    for bucket in buckets.values() {
        for (position, &source) in bucket.iter().enumerate() {
            let target = if bucket.len() <= 1 {
                source
            } else {
                bucket[(position + 1) % bucket.len()]
            };
            permutation[usize::from(source)] = target;
        }
    }
    if permutation.get(usize::from(identity)) != Some(&identity) {
        return Err(R4GroupGeometryError::Invalid(
            "order-preserving scramble moved the identity".to_owned(),
        ));
    }
    let unique = permutation.iter().copied().collect::<BTreeSet<_>>();
    if unique.len() != R4_GROUP_ORDER {
        return Err(R4GroupGeometryError::Invalid(
            "order-preserving scramble is not bijective".to_owned(),
        ));
    }
    Ok(permutation)
}

fn find_nonhomomorphism_witness(
    table: &[u16],
    permutation: &[u16],
) -> Result<R4GroupNonhomomorphismWitnessV1, R4GroupGeometryError> {
    for left in 0..R4_GROUP_ORDER {
        let left = u16::try_from(left).map_err(|_| R4GroupGeometryError::ArithmeticOverflow)?;
        for right in 0..R4_GROUP_ORDER {
            let right =
                u16::try_from(right).map_err(|_| R4GroupGeometryError::ArithmeticOverflow)?;
            let true_product = table_product(table, left, right).ok_or_else(|| {
                R4GroupGeometryError::Invalid("H4 witness product is out of range".to_owned())
            })?;
            let permuted_product =
                *permutation.get(usize::from(true_product)).ok_or_else(|| {
                    R4GroupGeometryError::Invalid(
                        "H4 witness permutation is out of range".to_owned(),
                    )
                })?;
            let permuted_left = *permutation.get(usize::from(left)).ok_or_else(|| {
                R4GroupGeometryError::Invalid("H4 left scramble is out of range".to_owned())
            })?;
            let permuted_right = *permutation.get(usize::from(right)).ok_or_else(|| {
                R4GroupGeometryError::Invalid("H4 right scramble is out of range".to_owned())
            })?;
            let product_of_permuted = table_product(table, permuted_left, permuted_right)
                .ok_or_else(|| {
                    R4GroupGeometryError::Invalid(
                        "scrambled H4 witness product is out of range".to_owned(),
                    )
                })?;
            if permuted_product != product_of_permuted {
                return Ok(R4GroupNonhomomorphismWitnessV1 {
                    left,
                    right,
                    true_product,
                    permuted_product,
                    product_of_permuted,
                });
            }
        }
    }
    Err(R4GroupGeometryError::Invalid(
        "transport scramble is an H4 homomorphism; no destruction witness exists".to_owned(),
    ))
}

fn generated_subgroup(
    table: &[u16],
    identity: u16,
    generators: &[u16],
) -> Result<Vec<u16>, R4GroupGeometryError> {
    validate_table_shape(table)?;
    if generators
        .iter()
        .any(|&generator| usize::from(generator) >= R4_GROUP_ORDER)
    {
        return Err(R4GroupGeometryError::Invalid(
            "subgroup generator is outside the group".to_owned(),
        ));
    }
    let mut seen = BTreeSet::from([identity]);
    let mut queue = VecDeque::from([identity]);
    while let Some(value) = queue.pop_front() {
        for &generator in generators {
            let product = table_product(table, value, generator).ok_or_else(|| {
                R4GroupGeometryError::Invalid("subgroup product is out of range".to_owned())
            })?;
            if seen.insert(product) {
                queue.push_back(product);
            }
        }
    }
    Ok(seen.into_iter().collect())
}

fn order_histogram(
    support: &[u16],
    orders: &[u16],
) -> Result<Vec<R4GroupOrderHistogramEntryV1>, R4GroupGeometryError> {
    let mut counts = BTreeMap::<u16, usize>::new();
    for &element in support {
        let order = *orders.get(usize::from(element)).ok_or_else(|| {
            R4GroupGeometryError::Invalid("histogram element is outside H4".to_owned())
        })?;
        *counts.entry(order).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(element_order, count)| {
            Ok(R4GroupOrderHistogramEntryV1 {
                element_order,
                distinct_actions: u16::try_from(count)
                    .map_err(|_| R4GroupGeometryError::ArithmeticOverflow)?,
            })
        })
        .collect()
}

fn table_product(table: &[u16], left: u16, right: u16) -> Option<u16> {
    let left = usize::from(left);
    let right = usize::from(right);
    if left >= R4_GROUP_ORDER || right >= R4_GROUP_ORDER {
        return None;
    }
    left.checked_mul(R4_GROUP_ORDER)
        .and_then(|base| base.checked_add(right))
        .and_then(|offset| table.get(offset))
        .copied()
}

fn validate_table_shape(table: &[u16]) -> Result<(), R4GroupGeometryError> {
    let expected = R4_GROUP_ORDER
        .checked_mul(R4_GROUP_ORDER)
        .ok_or(R4GroupGeometryError::ArithmeticOverflow)?;
    if table.len() != expected
        || table
            .iter()
            .any(|&value| usize::from(value) >= R4_GROUP_ORDER)
    {
        return Err(R4GroupGeometryError::Invalid(
            "group multiplication table is not a bounded 120x120 table".to_owned(),
        ));
    }
    Ok(())
}

fn sorted_unique(values: &[u16]) -> Vec<u16> {
    values
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn require_full_coverage(label: &str, subgroup: &[u16]) -> Result<(), R4GroupGeometryError> {
    if subgroup.len() != R4_GROUP_ORDER {
        return Err(R4GroupGeometryError::Invalid(format!(
            "{label} used leaves generate only {}/{} states",
            subgroup.len(),
            R4_GROUP_ORDER
        )));
    }
    Ok(())
}

fn u16_len<T>(values: &[T]) -> Result<u16, R4GroupGeometryError> {
    u16::try_from(values.len()).map_err(|_| R4GroupGeometryError::ArithmeticOverflow)
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, R4GroupGeometryError> {
    serde_json::to_vec(value)
        .map_err(|error| R4GroupGeometryError::Serialization(error.to_string()))
}

fn blake3_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_geometry_artifact_is_full_source_free_and_reproducible() {
        let artifact = R4GroupGeometryArtifactV1::build(R4_GROUP_MAX_TOKEN_ID)
            .expect("build exact group artifact");
        assert_eq!(artifact.group_order, 120);
        assert_eq!(artifact.h4_multiplication_indices.len(), 120 * 120);
        assert_eq!(artifact.c120_multiplication_indices.len(), 120 * 120);
        assert_eq!(artifact.inverse_indices.len(), 120);
        assert_eq!(artifact.c120_inverse_indices.len(), 120);
        assert_eq!(artifact.leaf_map.leaf_indices.len(), 4096);
        assert_eq!(artifact.leaf_map.leaf_indices[0], artifact.identity_index);
        assert_eq!(artifact.leaf_map.leaf_indices[1], 2);
        assert_eq!(artifact.leaf_map.leaf_indices[2], 3);
        assert_eq!(artifact.censuses.h4_generated_subgroup_count, 120);
        assert_eq!(artifact.censuses.c120_generated_subgroup_count, 120);
        assert_eq!(artifact.censuses.scrambled_h4_generated_subgroup_count, 120);
        assert!(artifact.scramble.identity_fixed);
        assert!(artifact.scramble.element_orders_preserved);
        assert!(artifact.scramble.moved_count >= 100);
        assert_ne!(
            artifact.scramble.nonhomomorphism_witness.permuted_product,
            artifact
                .scramble
                .nonhomomorphism_witness
                .product_of_permuted
        );

        let bytes = artifact.canonical_bytes().expect("canonical JSON");
        let reparsed = R4GroupGeometryArtifactV1::from_canonical_bytes(&bytes)
            .expect("verify canonical artifact");
        assert_eq!(artifact, reparsed);
        assert_eq!(
            artifact.artifact_cid,
            artifact.reproduce_artifact_cid().unwrap()
        );
        assert_eq!(
            artifact.leaf_map.leaf_cid,
            artifact.leaf_map.reproduce_leaf_cid().unwrap()
        );
        assert_eq!(bytes, artifact.canonical_bytes().unwrap());
    }

    #[test]
    fn wrong_bound_and_noncanonical_json_fail_closed() {
        assert!(R4GroupGeometryArtifactV1::build(4094).is_err());
        let artifact = R4GroupGeometryArtifactV1::build(R4_GROUP_MAX_TOKEN_ID)
            .expect("build exact group artifact");
        let mut bytes = artifact.canonical_bytes().expect("canonical JSON");
        bytes.push(b'\n');
        assert!(R4GroupGeometryArtifactV1::from_canonical_bytes(&bytes).is_err());
    }
}
