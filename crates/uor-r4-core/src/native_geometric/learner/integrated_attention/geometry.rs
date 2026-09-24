//! Artifact-bound finite geometry for the integrated sparse reader.
//!
//! `new_2i` is an **offline/export** constructor: the historical table builder may
//! use floating point while constructing its canonical table. A loaded artifact
//! deserializes these integer bytes and calls `validate`; neither validation nor
//! the selected read/score path calls that builder. The containing model artifact
//! must bind this table digest together with its learned root codebook.
//!
//! All element IDs are signed-root IDs: `q` and `-q` stay distinct. Composition
//! means left factor followed by right factor. A query-local action is composed
//! on the **right** of its query frame, so the relative code is `(q a)^-1 k`.
//! `Cyclic120` is a matched ordinary finite-action comparator, not a claim that
//! cyclic codes are the best possible ordinary representation.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

use super::super::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};

const FORMAT_VERSION: u16 = 1;
const MAX_LANES: usize = 32;
const MAX_EDGES: usize = 64;
const _: [(); 120] = [(); GROUP_ORDER];
const _: [(); 128] = [(); ROW_STRIDE];

/// Errors are kept typed at the artifact and library boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeometryError {
    UnsupportedVersion(u16),
    InvalidLength {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    InvalidElement(u8),
    InvalidLaneCount(usize),
    InvalidLane(u8),
    InvalidEdgeIndex(usize),
    InvalidEdge {
        left: u8,
        right: u8,
    },
    DuplicateEdge {
        left: u8,
        right: u8,
    },
    InvalidCoefficient(i8),
    InvalidTable(&'static str),
    DigestMismatch,
    CounterOverflow,
    ScoreOverflow,
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(v) => write!(f, "unsupported geometry format version {v}"),
            Self::InvalidLength {
                field,
                expected,
                actual,
            } => {
                write!(f, "{field} length {actual}, expected {expected}")
            }
            Self::InvalidElement(id) => write!(f, "group element {id} outside 0..120"),
            Self::InvalidLaneCount(n) => write!(f, "lane count {n} outside 1..={MAX_LANES}"),
            Self::InvalidLane(lane) => write!(f, "lane index {lane} outside configured lanes"),
            Self::InvalidEdgeIndex(edge) => write!(f, "edge index {edge} outside configured edges"),
            Self::InvalidEdge { left, right } => write!(f, "invalid lane edge ({left},{right})"),
            Self::DuplicateEdge { left, right } => {
                write!(f, "duplicate lane edge ({left},{right})")
            }
            Self::InvalidCoefficient(w) => {
                write!(f, "coefficient {w} outside signed four-bit range")
            }
            Self::InvalidTable(why) => write!(f, "invalid finite algebra table: {why}"),
            Self::DigestMismatch => write!(f, "finite algebra table digest mismatch"),
            Self::CounterOverflow => write!(f, "geometry read counter overflow"),
            Self::ScoreOverflow => write!(f, "geometry energy accumulator overflow"),
        }
    }
}

impl std::error::Error for GeometryError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlgebraKind {
    BinaryIcosahedral,
    Cyclic120,
}

impl AlgebraKind {
    const fn tag(self) -> u8 {
        match self {
            Self::BinaryIcosahedral => 1,
            Self::Cyclic120 => 2,
        }
    }
}

/// Padded 120-element multiplication table with its inverse table and binding digest.
///
/// Call [`Self::validate`] once after deserialization and before admitting a model
/// artifact. The digest detects accidental table/codebook mismatch; the enclosing
/// artifact must cryptographically bind this digest to the learned codes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiniteAlgebra {
    format_version: u16,
    kind: AlgebraKind,
    identity: u8,
    product: Vec<u8>,
    inverse: Vec<u8>,
    digest: [u8; 32],
}

/// Selected group-table reads, separate from learned coefficient accesses.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgebraReadCounts {
    pub product_reads: u64,
    pub inverse_reads: u64,
}

impl FiniteAlgebra {
    /// Construct the project's canonical `2I` table for offline artifact export.
    /// This may initialize the historical floating-point table builder.
    pub fn new_2i() -> Result<Self, GeometryError> {
        let source = group_table();
        let mut table = Self {
            format_version: FORMAT_VERSION,
            kind: AlgebraKind::BinaryIcosahedral,
            identity: source.identity,
            product: source.product.to_vec(),
            inverse: source.inverse.to_vec(),
            digest: [0; 32],
        };
        table.digest = table.compute_digest();
        table.validate()?;
        Ok(table)
    }

    /// Construct the ordinary cyclic order-120 comparator for offline export.
    pub fn new_c120() -> Result<Self, GeometryError> {
        let mut product = vec![0u8; GROUP_ORDER << 7];
        let mut inverse = vec![0u8; GROUP_ORDER];
        for a in 0..GROUP_ORDER {
            for b in 0..GROUP_ORDER {
                let sum = a + b;
                product[(a << 7) | b] = if sum >= GROUP_ORDER {
                    (sum - GROUP_ORDER) as u8
                } else {
                    sum as u8
                };
            }
            inverse[a] = if a == 0 { 0 } else { (GROUP_ORDER - a) as u8 };
        }
        let mut table = Self {
            format_version: FORMAT_VERSION,
            kind: AlgebraKind::Cyclic120,
            identity: 0,
            product,
            inverse,
            digest: [0; 32],
        };
        table.digest = table.compute_digest();
        table.validate()?;
        Ok(table)
    }

    pub const fn kind(&self) -> AlgebraKind {
        self.kind
    }

    pub const fn identity(&self) -> u8 {
        self.identity
    }

    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub fn serialized_table_bytes(&self) -> usize {
        self.product.len() + self.inverse.len()
    }

    fn compute_digest(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"uor-r4.integrated-attention.finite-algebra.v1");
        h.update(&self.format_version.to_le_bytes());
        h.update(&[self.kind.tag(), self.identity]);
        h.update(&self.inverse);
        h.update(&self.product);
        *h.finalize().as_bytes()
    }

    /// Validate loaded integer bytes without invoking the offline table builder.
    /// Checks shape, signed IDs, a two-sided identity/inverse, row permutations,
    /// associativity, kind-specific invariants and the serialized digest.
    pub fn validate(&self) -> Result<(), GeometryError> {
        if self.format_version != FORMAT_VERSION {
            return Err(GeometryError::UnsupportedVersion(self.format_version));
        }
        let expected_product = GROUP_ORDER << 7;
        if self.product.len() != expected_product {
            return Err(GeometryError::InvalidLength {
                field: "product",
                expected: expected_product,
                actual: self.product.len(),
            });
        }
        if self.inverse.len() != GROUP_ORDER {
            return Err(GeometryError::InvalidLength {
                field: "inverse",
                expected: GROUP_ORDER,
                actual: self.inverse.len(),
            });
        }
        if usize::from(self.identity) >= GROUP_ORDER {
            return Err(GeometryError::InvalidElement(self.identity));
        }
        if self.compute_digest() != self.digest {
            return Err(GeometryError::DigestMismatch);
        }
        for a in 0..GROUP_ORDER {
            let mut seen = [false; GROUP_ORDER];
            for b in 0..GROUP_ORDER {
                let value = usize::from(self.product[(a << 7) | b]);
                if value >= GROUP_ORDER {
                    return Err(GeometryError::InvalidTable("product ID out of range"));
                }
                if seen[value] {
                    return Err(GeometryError::InvalidTable("row is not a permutation"));
                }
                seen[value] = true;
            }
            for b in GROUP_ORDER..ROW_STRIDE {
                if self.product[(a << 7) | b] != 0 {
                    return Err(GeometryError::InvalidTable("nonzero padded product cell"));
                }
            }
            let inv = usize::from(self.inverse[a]);
            if inv >= GROUP_ORDER {
                return Err(GeometryError::InvalidTable("inverse ID out of range"));
            }
            let e = usize::from(self.identity);
            if usize::from(self.product[(a << 7) | e]) != a
                || usize::from(self.product[(e << 7) | a]) != a
                || usize::from(self.product[(a << 7) | inv]) != e
                || usize::from(self.product[(inv << 7) | a]) != e
            {
                return Err(GeometryError::InvalidTable("identity or inverse law"));
            }
        }
        for a in 0..GROUP_ORDER {
            for b in 0..GROUP_ORDER {
                let ab = usize::from(self.product[(a << 7) | b]);
                for c in 0..GROUP_ORDER {
                    let bc = usize::from(self.product[(b << 7) | c]);
                    if self.product[(ab << 7) | c] != self.product[(a << 7) | bc] {
                        return Err(GeometryError::InvalidTable("associativity"));
                    }
                }
            }
        }
        match self.kind {
            AlgebraKind::Cyclic120 => self.validate_cyclic(),
            AlgebraKind::BinaryIcosahedral => self.validate_icosahedral_orders(),
        }
    }

    fn validate_cyclic(&self) -> Result<(), GeometryError> {
        if self.identity != 0 {
            return Err(GeometryError::InvalidTable("cyclic identity is not zero"));
        }
        for a in 0..GROUP_ORDER {
            for b in 0..GROUP_ORDER {
                let sum = a + b;
                let expected = if sum >= GROUP_ORDER {
                    sum - GROUP_ORDER
                } else {
                    sum
                };
                if usize::from(self.product[(a << 7) | b]) != expected {
                    return Err(GeometryError::InvalidTable("noncanonical cyclic product"));
                }
            }
        }
        Ok(())
    }

    fn validate_icosahedral_orders(&self) -> Result<(), GeometryError> {
        // The binary icosahedral group has this element-order census. It is a
        // structural sanity check, not a replacement for the enclosing artifact
        // digest that binds exact root numbering and learned codes.
        let mut census = [0usize; 121];
        for a in 0..GROUP_ORDER {
            let mut x = usize::from(self.identity);
            let mut order = 0usize;
            loop {
                x = usize::from(self.product[(x << 7) | a]);
                order += 1;
                if x == usize::from(self.identity) {
                    break;
                }
                if order >= GROUP_ORDER {
                    return Err(GeometryError::InvalidTable(
                        "element order exceeds group order",
                    ));
                }
            }
            census[order] += 1;
        }
        let expected = [(1, 1), (2, 1), (3, 20), (4, 30), (5, 24), (6, 20), (10, 24)];
        for (order, count) in expected {
            if census[order] != count {
                return Err(GeometryError::InvalidTable("wrong 2I element-order census"));
            }
            census[order] = 0;
        }
        if census.iter().any(|&count| count != 0) {
            return Err(GeometryError::InvalidTable("unexpected 2I element order"));
        }
        Ok(())
    }

    /// One selected composition table read. The loaded artifact must have been
    /// validated once; bounds are still checked on every public call.
    pub fn compose(&self, left: u8, right: u8) -> Result<u8, GeometryError> {
        check_id(left)?;
        check_id(right)?;
        let result = self
            .product
            .get((usize::from(left) << 7) | usize::from(right))
            .copied()
            .ok_or(GeometryError::InvalidTable("truncated product table"))?;
        check_id(result)?;
        Ok(result)
    }

    /// One selected inverse table read.
    pub fn inverse(&self, element: u8) -> Result<u8, GeometryError> {
        check_id(element)?;
        let result = self
            .inverse
            .get(usize::from(element))
            .copied()
            .ok_or(GeometryError::InvalidTable("truncated inverse table"))?;
        check_id(result)?;
        Ok(result)
    }

    pub fn compose_counted(
        &self,
        left: u8,
        right: u8,
        reads: &mut AlgebraReadCounts,
    ) -> Result<u8, GeometryError> {
        let result = self.compose(left, right)?;
        reads.product_reads = reads
            .product_reads
            .checked_add(1)
            .ok_or(GeometryError::CounterOverflow)?;
        Ok(result)
    }

    pub fn inverse_counted(
        &self,
        element: u8,
        reads: &mut AlgebraReadCounts,
    ) -> Result<u8, GeometryError> {
        let result = self.inverse(element)?;
        reads.inverse_reads = reads
            .inverse_reads
            .checked_add(1)
            .ok_or(GeometryError::CounterOverflow)?;
        Ok(result)
    }

    /// Query-local convention: `(query * local_action)^-1 * key`.
    /// Under common left multiplication of query and key, this is unchanged
    /// when `local_action` is itself fixed in the query's local frame.
    pub fn relative(&self, query: u8, local_action: u8, key: u8) -> Result<u8, GeometryError> {
        let acted_query = self.compose(query, local_action)?;
        self.compose(self.inverse(acted_query)?, key)
    }

    pub fn relative_counted(
        &self,
        query: u8,
        local_action: u8,
        key: u8,
        reads: &mut AlgebraReadCounts,
    ) -> Result<u8, GeometryError> {
        let acted_query = self.compose_counted(query, local_action, reads)?;
        let inverse = self.inverse_counted(acted_query, reads)?;
        self.compose_counted(inverse, key, reads)
    }

    /// Fill caller-owned output with lane-wise relative codes; no serving allocation.
    pub fn relative_lanes(
        &self,
        queries: &[u8],
        local_actions: &[u8],
        keys: &[u8],
        out: &mut [u8],
    ) -> Result<(), GeometryError> {
        let n = queries.len();
        if n == 0 || n > MAX_LANES {
            return Err(GeometryError::InvalidLaneCount(n));
        }
        for (name, actual) in [
            ("local_actions", local_actions.len()),
            ("keys", keys.len()),
            ("relative_output", out.len()),
        ] {
            if actual != n {
                return Err(GeometryError::InvalidLength {
                    field: name,
                    expected: n,
                    actual,
                });
            }
        }
        for lane in 0..n {
            check_id(queries[lane])?;
            check_id(local_actions[lane])?;
            check_id(keys[lane])?;
        }
        for lane in 0..n {
            out[lane] = self.relative(queries[lane], local_actions[lane], keys[lane])?;
        }
        Ok(())
    }

    pub fn relative_lanes_counted(
        &self,
        queries: &[u8],
        local_actions: &[u8],
        keys: &[u8],
        out: &mut [u8],
        reads: &mut AlgebraReadCounts,
    ) -> Result<(), GeometryError> {
        let n = queries.len();
        if n == 0 || n > MAX_LANES {
            return Err(GeometryError::InvalidLaneCount(n));
        }
        for (name, actual) in [
            ("local_actions", local_actions.len()),
            ("keys", keys.len()),
            ("relative_output", out.len()),
        ] {
            if actual != n {
                return Err(GeometryError::InvalidLength {
                    field: name,
                    expected: n,
                    actual,
                });
            }
        }
        for lane in 0..n {
            check_id(queries[lane])?;
            check_id(local_actions[lane])?;
            check_id(keys[lane])?;
        }
        for lane in 0..n {
            out[lane] =
                self.relative_counted(queries[lane], local_actions[lane], keys[lane], reads)?;
        }
        Ok(())
    }
}

fn check_id(id: u8) -> Result<(), GeometryError> {
    if usize::from(id) < GROUP_ORDER {
        Ok(())
    } else {
        Err(GeometryError::InvalidElement(id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanePair {
    pub left: u8,
    pub right: u8,
}

/// Number of selected table entries/bytes actually loaded while scoring.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnergyReadCounts {
    pub coefficients: u64,
    pub packed_bytes: u64,
}

impl EnergyReadCounts {
    fn record(&mut self) -> Result<(), GeometryError> {
        let coefficients = self
            .coefficients
            .checked_add(1)
            .ok_or(GeometryError::CounterOverflow)?;
        let packed_bytes = self
            .packed_bytes
            .checked_add(1)
            .ok_or(GeometryError::CounterOverflow)?;
        self.coefficients = coefficients;
        self.packed_bytes = packed_bytes;
        Ok(())
    }
}

/// Sparse one- and two-lane energy tables. Each signed coefficient occupies a
/// four-bit nibble, range -8..=7. Padding to 128 IDs makes served indexing
/// shifts/bit-ops, including pair-table row selection. Metadata bias and legal
/// version/scope masks belong to the caller; they must be bounded and counted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnergyTables {
    format_version: u16,
    lanes: u8,
    edges: Vec<LanePair>,
    unary_packed: Vec<u8>,
    pair_packed: Vec<u8>,
}

impl EnergyTables {
    /// Offline builder with an explicit sparse lane-pair graph.
    pub fn zeroed(lanes: u8, edges: Vec<LanePair>) -> Result<Self, GeometryError> {
        if lanes == 0 || usize::from(lanes) > MAX_LANES {
            return Err(GeometryError::InvalidLaneCount(usize::from(lanes)));
        }
        if edges.len() > MAX_EDGES {
            return Err(GeometryError::InvalidLength {
                field: "edges",
                expected: MAX_EDGES,
                actual: edges.len(),
            });
        }
        let tables = Self {
            format_version: FORMAT_VERSION,
            lanes,
            unary_packed: vec![0; usize::from(lanes) << 6],
            pair_packed: vec![0; edges.len() << 13],
            edges,
        };
        tables.validate()?;
        Ok(tables)
    }

    pub const fn lanes(&self) -> u8 {
        self.lanes
    }

    pub fn edges(&self) -> &[LanePair] {
        &self.edges
    }

    /// Meaningful learned coefficients, excluding padded element IDs.
    pub fn coefficient_slots(&self) -> usize {
        let lanes = usize::from(self.lanes);
        let edges = self.edges.len();
        // 120 = 128 - 8; 14,400 = 8,192 + 4,096 + 2,048 + 64.
        ((lanes << 7) - (lanes << 3)) + (edges << 13) + (edges << 12) + (edges << 11) + (edges << 6)
    }

    /// Actual packed parameter bytes, including bounded 128-stride padding.
    pub fn packed_bytes(&self) -> usize {
        self.unary_packed.len() + self.pair_packed.len()
    }

    pub fn validate(&self) -> Result<(), GeometryError> {
        self.validate_shape()?;
        for lane in 0..usize::from(self.lanes) {
            for id in GROUP_ORDER..ROW_STRIDE {
                if self.raw_unary_nibble(lane, id)? != 0 {
                    return Err(GeometryError::InvalidTable("nonzero unary padding"));
                }
            }
        }
        for edge in 0..self.edges.len() {
            for left in 0..ROW_STRIDE {
                for right in 0..ROW_STRIDE {
                    if (left >= GROUP_ORDER || right >= GROUP_ORDER)
                        && self.raw_pair_nibble(edge, left, right)? != 0
                    {
                        return Err(GeometryError::InvalidTable("nonzero pair padding"));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_shape(&self) -> Result<(), GeometryError> {
        if self.format_version != FORMAT_VERSION {
            return Err(GeometryError::UnsupportedVersion(self.format_version));
        }
        let lanes = usize::from(self.lanes);
        if lanes == 0 || lanes > MAX_LANES {
            return Err(GeometryError::InvalidLaneCount(lanes));
        }
        if self.edges.len() > MAX_EDGES {
            return Err(GeometryError::InvalidLength {
                field: "edges",
                expected: MAX_EDGES,
                actual: self.edges.len(),
            });
        }
        if self.unary_packed.len() != lanes << 6 {
            return Err(GeometryError::InvalidLength {
                field: "unary_packed",
                expected: lanes << 6,
                actual: self.unary_packed.len(),
            });
        }
        if self.pair_packed.len() != self.edges.len() << 13 {
            return Err(GeometryError::InvalidLength {
                field: "pair_packed",
                expected: self.edges.len() << 13,
                actual: self.pair_packed.len(),
            });
        }
        for (i, edge) in self.edges.iter().enumerate() {
            if edge.left >= edge.right || usize::from(edge.right) >= lanes {
                return Err(GeometryError::InvalidEdge {
                    left: edge.left,
                    right: edge.right,
                });
            }
            if self.edges[..i].iter().any(|old| old == edge) {
                return Err(GeometryError::DuplicateEdge {
                    left: edge.left,
                    right: edge.right,
                });
            }
        }
        Ok(())
    }

    fn checked_lane(&self, lane: u8, id: u8) -> Result<(), GeometryError> {
        if lane >= self.lanes {
            return Err(GeometryError::InvalidLane(lane));
        }
        check_id(id)
    }

    fn checked_edge(&self, edge: usize, left: u8, right: u8) -> Result<(), GeometryError> {
        if edge >= self.edges.len() {
            return Err(GeometryError::InvalidEdgeIndex(edge));
        }
        check_id(left)?;
        check_id(right)
    }

    fn raw_unary_nibble(&self, lane: usize, id: usize) -> Result<u8, GeometryError> {
        let index = (lane << 6) | (id >> 1);
        let byte = *self
            .unary_packed
            .get(index)
            .ok_or(GeometryError::InvalidTable("unary truncated"))?;
        Ok((byte >> ((id & 1) << 2)) & 0x0f)
    }

    fn raw_pair_nibble(&self, edge: usize, left: usize, right: usize) -> Result<u8, GeometryError> {
        let index = (edge << 13) | (left << 6) | (right >> 1);
        let byte = *self
            .pair_packed
            .get(index)
            .ok_or(GeometryError::InvalidTable("pair truncated"))?;
        Ok((byte >> ((right & 1) << 2)) & 0x0f)
    }

    /// Read one trained unary coefficient. This is also usable by the offline
    /// trainer when updating selected factors.
    pub fn get_unary(&self, lane: u8, relative: u8) -> Result<i8, GeometryError> {
        self.checked_lane(lane, relative)?;
        Ok(decode_nibble(self.raw_unary_nibble(
            usize::from(lane),
            usize::from(relative),
        )?))
    }

    /// Read one trained lane-pair coefficient by edge index.
    pub fn get_pair(&self, edge: usize, left: u8, right: u8) -> Result<i8, GeometryError> {
        self.checked_edge(edge, left, right)?;
        Ok(decode_nibble(self.raw_pair_nibble(
            edge,
            usize::from(left),
            usize::from(right),
        )?))
    }

    /// Set one signed four-bit unary coefficient during offline fitting/export.
    pub fn set_unary(&mut self, lane: u8, relative: u8, weight: i8) -> Result<(), GeometryError> {
        self.checked_lane(lane, relative)?;
        check_weight(weight)?;
        let index = (usize::from(lane) << 6) | (usize::from(relative) >> 1);
        set_nibble(
            &mut self.unary_packed,
            index,
            usize::from(relative & 1),
            weight,
        )
    }

    /// Set one signed four-bit pair coefficient during offline fitting/export.
    pub fn set_pair(
        &mut self,
        edge: usize,
        left: u8,
        right: u8,
        weight: i8,
    ) -> Result<(), GeometryError> {
        self.checked_edge(edge, left, right)?;
        check_weight(weight)?;
        let index = (edge << 13) | (usize::from(left) << 6) | (usize::from(right) >> 1);
        set_nibble(&mut self.pair_packed, index, usize::from(right & 1), weight)
    }

    /// Sum only the selected one- and two-lane factors. Relative IDs are
    /// produced with [`FiniteAlgebra::relative_lanes`] or a matched ordinary
    /// encoder. The caller supplies an explicit metadata bias and eligibility
    /// mask outside this primitive.
    pub fn score(
        &self,
        relatives: &[u8],
        reads: &mut EnergyReadCounts,
    ) -> Result<i32, GeometryError> {
        if self.lanes == 0 || usize::from(self.lanes) > MAX_LANES {
            return Err(GeometryError::InvalidLaneCount(usize::from(self.lanes)));
        }
        if relatives.len() != usize::from(self.lanes) {
            return Err(GeometryError::InvalidLength {
                field: "relative lanes",
                expected: usize::from(self.lanes),
                actual: relatives.len(),
            });
        }
        for &relative in relatives {
            check_id(relative)?;
        }
        let mut sum = 0i32;
        for (lane, &relative) in relatives.iter().enumerate() {
            let weight = i32::from(self.get_unary(lane as u8, relative)?);
            reads.record()?;
            sum = sum
                .checked_add(weight)
                .ok_or(GeometryError::ScoreOverflow)?;
        }
        for (index, edge) in self.edges.iter().enumerate() {
            let left =
                *relatives
                    .get(usize::from(edge.left))
                    .ok_or(GeometryError::InvalidEdge {
                        left: edge.left,
                        right: edge.right,
                    })?;
            let right =
                *relatives
                    .get(usize::from(edge.right))
                    .ok_or(GeometryError::InvalidEdge {
                        left: edge.left,
                        right: edge.right,
                    })?;
            let weight = i32::from(self.get_pair(index, left, right)?);
            reads.record()?;
            sum = sum
                .checked_add(weight)
                .ok_or(GeometryError::ScoreOverflow)?;
        }
        Ok(sum)
    }
}

fn check_weight(weight: i8) -> Result<(), GeometryError> {
    if (-8..=7).contains(&weight) {
        Ok(())
    } else {
        Err(GeometryError::InvalidCoefficient(weight))
    }
}

fn decode_nibble(nibble: u8) -> i8 {
    if nibble < 8 {
        nibble as i8
    } else {
        (nibble as i8) - 16
    }
}

fn set_nibble(bytes: &mut [u8], index: usize, odd: usize, weight: i8) -> Result<(), GeometryError> {
    let byte = bytes
        .get_mut(index)
        .ok_or(GeometryError::InvalidTable("truncated packed energy"))?;
    let shift = odd << 2;
    let mask = 0x0fu8 << shift;
    *byte = (*byte & !mask) | (((weight as u8) & 0x0f) << shift);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_finite_algebras_roundtrip_and_obey_relative_left_invariance() {
        for algebra in [
            FiniteAlgebra::new_2i().unwrap(),
            FiniteAlgebra::new_c120().unwrap(),
        ] {
            let json = serde_json::to_vec(&algebra).unwrap();
            let loaded: FiniteAlgebra = serde_json::from_slice(&json).unwrap();
            loaded.validate().unwrap();
            assert_eq!(algebra, loaded);
            assert_eq!(
                loaded.serialized_table_bytes(),
                GROUP_ORDER * ROW_STRIDE + GROUP_ORDER
            );
            let mut reads = AlgebraReadCounts::default();
            assert_eq!(
                loaded.relative_counted(2, 3, 5, &mut reads).unwrap(),
                loaded.relative(2, 3, 5).unwrap()
            );
            assert_eq!(reads.product_reads, 2);
            assert_eq!(reads.inverse_reads, 1);
            let mut lane_reads = AlgebraReadCounts::default();
            let mut relatives = [0u8; 2];
            loaded
                .relative_lanes_counted(
                    &[2, 34],
                    &[3, 47],
                    &[5, 91],
                    &mut relatives,
                    &mut lane_reads,
                )
                .unwrap();
            assert_eq!(relatives[0], loaded.relative(2, 3, 5).unwrap());
            assert_eq!(relatives[1], loaded.relative(34, 47, 91).unwrap());
            assert_eq!(lane_reads.product_reads, 4);
            assert_eq!(lane_reads.inverse_reads, 2);
            for h in [loaded.identity(), 1, 17, 119] {
                for (q, a, k) in [(2, 3, 5), (34, 47, 91), (119, 0, 1)] {
                    let hq = loaded.compose(h, q).unwrap();
                    let hk = loaded.compose(h, k).unwrap();
                    assert_eq!(
                        loaded.relative(q, a, k).unwrap(),
                        loaded.relative(hq, a, hk).unwrap()
                    );
                }
            }
        }
    }

    #[test]
    fn corrupted_loaded_table_is_rejected_without_offline_builder() {
        let mut loaded = FiniteAlgebra::new_c120().unwrap();
        loaded.product[(1 << 7) | 2] = 120;
        assert_eq!(loaded.validate(), Err(GeometryError::DigestMismatch));
        loaded.digest = loaded.compute_digest();
        assert!(matches!(
            loaded.validate(),
            Err(GeometryError::InvalidTable(_))
        ));
        assert_eq!(
            loaded.compose(120, 0),
            Err(GeometryError::InvalidElement(120))
        );
    }

    #[test]
    fn selected_packed_energy_uses_only_one_and_two_lane_factors() {
        let edges = vec![
            LanePair { left: 0, right: 1 },
            LanePair { left: 1, right: 2 },
        ];
        let mut energy = EnergyTables::zeroed(3, edges).unwrap();
        energy.set_unary(0, 5, -8).unwrap();
        energy.set_unary(1, 119, 7).unwrap();
        energy.set_pair(0, 5, 119, 3).unwrap();
        energy.set_pair(1, 119, 0, -2).unwrap();
        energy.validate().unwrap();
        assert_eq!(energy.get_unary(0, 5).unwrap(), -8);
        assert_eq!(energy.get_pair(0, 5, 119).unwrap(), 3);
        assert_eq!(energy.coefficient_slots(), 3 * 120 + 2 * 120 * 120);
        assert_eq!(energy.packed_bytes(), 3 * 64 + 2 * 8192);
        let mut reads = EnergyReadCounts::default();
        assert_eq!(energy.score(&[5, 119, 0], &mut reads).unwrap(), 0);
        assert_eq!(reads.coefficients, 5);
        assert_eq!(reads.packed_bytes, 5);
        let loaded: EnergyTables =
            serde_json::from_slice(&serde_json::to_vec(&energy).unwrap()).unwrap();
        loaded.validate().unwrap();
        assert_eq!(loaded, energy);
    }

    #[test]
    fn malformed_coefficients_edges_and_lanes_fail() {
        assert!(matches!(
            EnergyTables::zeroed(2, vec![LanePair { left: 0, right: 0 }]),
            Err(GeometryError::InvalidEdge { .. })
        ));
        let mut energy = EnergyTables::zeroed(2, vec![LanePair { left: 0, right: 1 }]).unwrap();
        assert_eq!(
            energy.set_unary(0, 0, 8),
            Err(GeometryError::InvalidCoefficient(8))
        );
        assert_eq!(
            energy.set_pair(0, 120, 0, 1),
            Err(GeometryError::InvalidElement(120))
        );
        assert!(matches!(
            energy.score(&[0], &mut EnergyReadCounts::default()),
            Err(GeometryError::InvalidLength { .. })
        ));
        assert_eq!(
            energy.score(&[0, 120], &mut EnergyReadCounts::default()),
            Err(GeometryError::InvalidElement(120))
        );
        assert!(matches!(
            FiniteAlgebra::new_c120()
                .unwrap()
                .relative_lanes(&[0, 1], &[0], &[0, 1], &mut [0; 2]),
            Err(GeometryError::InvalidLength { .. })
        ));
    }
}
