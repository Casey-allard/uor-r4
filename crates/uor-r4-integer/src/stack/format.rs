//! Clean-room reader of the `UORLUT01` stack container written by
//! `uor-r4-training`'s `stack_export` (schema `uor-r4.lut-stack/1`).
//!
//! Layout: the 8-byte magic `UORLUT01`, a little-endian `u64` header length,
//! the JSON header, zero padding to a multiple of 64 bytes, then the data
//! sections at 64-byte-aligned offsets relative to the start of the data.
//! A 4-bit matrix is stored row-major as offset-binary nibbles `q + 8`, two
//! per byte with the even column in the low nibble, and one scale byte per
//! row and group of [`GROUP`] columns: low nibble `m`, high nibble `de`,
//! meaning the group scale `(16 + m) 2^(exp_base + de - 4)`. Tables are
//! little-endian `i16`, `i32` or `u32` arrays.
//!
//! Loading is outside the served numerical path: it may allocate, format
//! messages and use ordinary index arithmetic. Every offset, length and
//! product of declared sizes is checked, so malformed input returns a
//! [`StackError`] instead of panicking.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::StackError;

/// Container magic.
pub const MAGIC: &[u8; 8] = b"UORLUT01";
/// Header schema of a geometric-stack artifact.
pub const STACK_SCHEMA: &str = "uor-r4.lut-stack/1";
/// Header schema of a stack with a pointer-copy head (`uor-r4-lut`'s
/// `STACK_POINTER_SCHEMA`): an engine built before the pointer port refuses it
/// instead of ignoring the head and serving the plain distribution.
pub const STACK_POINTER_SCHEMA: &str = "uor-r4.lut-stack/2";
/// Header schema of a stack whose reads are softmax-free flock rank reads
/// (`read_select` in the shape), with or without a pointer head: an engine
/// that serves only softmax reads refuses it instead of serving softmax.
pub const STACK_READ_SCHEMA: &str = "uor-r4.lut-stack/3";
/// Weights per scale group along a matrix row.
pub const GROUP: usize = 32;
/// Alignment of the data start and of every section.
const ALIGN: usize = 64;
/// The largest header accepted (64 MiB).
const MAX_HEADER: u64 = 64 << 20;

/// Serving limits of a stack's dimensions.
pub const MAX_VOCAB: usize = 1 << 20;
pub const MAX_WIDTH: usize = 1 << 14;
pub const MAX_MLP: usize = 1 << 16;
pub const MAX_CONTEXT: usize = 1 << 16;
pub const MAX_LAYERS: usize = 256;
/// The largest read cache a session may allocate (4 GiB): the keys and
/// values of every position of the context over all read layers, and their
/// Lorentz lifts. The dimension limits alone would allow 8 GiB per read layer.
pub const MAX_READ_CACHE_BYTES: u64 = 1 << 32;

/// Dimensions of a geometric stack.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StackShape {
    pub vocab: usize,
    pub width: usize,
    pub heads: usize,
    /// MLP width, padded by the exporter to a multiple of the group size.
    pub mlp: usize,
    /// One letter per layer: `r` quaternion transport recurrence, `a` read.
    pub pattern: String,
    /// Read score: `dot`, `lorentz` or `l2` (the flat Euclidean control of
    /// `lorentz`, with the same learned per-head scale and offset).
    pub read: String,
    /// Learned rotations in the recurrence; `false` is identity transport.
    pub rotation: bool,
    /// Positions a session serves: the length of each read's age table.
    pub context: usize,
    /// The pointer-copy head after the final norm; absent on a model without
    /// one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer: Option<StackPointer>,
    /// The flock selection of a softmax-free read; absent on a softmax read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_select: Option<StackReadSelect>,
    /// The read weights over the selected positions: `rank` (the fixed rank
    /// table) or `learned` (a per-layer, per-head Q31 table section
    /// `read_rank.<layer>`); absent on a softmax read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_weights: Option<String>,
    /// Hamming-rank selection over sign-bit codes instead of the score.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub read_binary: bool,
    /// A product-key memory replacing the read at the listed layers; absent
    /// on a model without one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<StackMemory>,
    /// Recurrence gate law: default table law when absent, or `geometric`
    /// (clamped linear opening into the 16 dyadic rates plus icosian-snapped
    /// Hamilton output gate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rec_gates: Option<String>,
    /// Feed-forward gate law: default table law when absent, or `hamilton`
    /// (4-channel Hamilton product with the gate snapped to the 120 unit
    /// icosians of `2I`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<String>,
}

/// The product-key memory of a layer (`memory` in the shape): the layer
/// indices whose **MLP** the memory replaces, and the product-key geometry
/// the served engine must reproduce — `heads` query blocks of `key_dim` per
/// row, `sub_keys` sub-keys per side of each block, `top_k` kept per side,
/// and the score the sub-keys are ranked by.
///
/// The served arithmetic of a memory layer is a *lookup*: two per-side
/// rankings of `sub_keys` sub-keys, a `top_k × top_k` candidate join, a
/// softmax over the kept candidates and a weighted gather of `width`-wide
/// value rows. It is recorded here so that a container can **carry** it; the
/// D11 kernel that serves it is not implemented yet, so no exporter writes
/// this section and a shape carrying one is refused by
/// [`StackShape::validate`] rather than half-served.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StackMemory {
    /// The layer indices the memory replaces, ascending and distinct.
    pub layers: Vec<usize>,
    pub heads: usize,
    /// The width of one head's query block; each side scores `key_dim / 2`.
    pub key_dim: usize,
    pub sub_keys: usize,
    pub top_k: usize,
    /// `dot` or `lorentz`.
    pub score: String,
}

/// The largest selection of a softmax-free read: the recent window, the `k`
/// selected positions, the sink (always position 0) and the NoRead slot.
pub const MAX_READ_SELECT: usize = 256;

/// The flock selection of a softmax-free read (`read_select` in the shape):
/// the `window` most recent positions, the sink at position 0 and the `k`
/// best-scoring other positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StackReadSelect {
    pub window: usize,
    pub k: usize,
}

/// The header schema a shape requires: [`STACK_READ_SCHEMA`] with a
/// softmax-free read or geometric recurrence/Hamilton gate, else
/// [`STACK_POINTER_SCHEMA`] with a pointer head, else [`STACK_SCHEMA`].
pub fn stack_schema_for(shape: &StackShape) -> &'static str {
    if shape.read_select.is_some() || shape.rec_geometric() || shape.gate_hamilton() {
        STACK_READ_SCHEMA
    } else if shape.pointer.is_some() {
        STACK_POINTER_SCHEMA
    } else {
        STACK_SCHEMA
    }
}

/// The largest pointer query and key width (the read's head-width limit).
/// It equals the D10 comparator's `uor_r4_lut::format::MAX_POINTER_DIM`
/// (asserted by `uor-r4-training`'s `stack_d11_oracle` test).
pub const MAX_POINTER_DIM: usize = 256;

/// The pointer-copy head (`pointer` in the shape): a query and key of width
/// `dim` and a gate logit read from the final normalized state, whose served
/// distribution is the mixture of the generated and copy distributions (see
/// the module documentation of [`super::session`]).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StackPointer {
    pub dim: usize,
    /// `dot` or `lorentz`.
    pub score: String,
    /// `1 / sqrt(dim)` in Q30 (the Dot score; recorded for both scores).
    pub score_scale_q30: i64,
}

impl StackPointer {
    pub fn lorentz(&self) -> bool {
        self.score == "lorentz"
    }
}

impl StackShape {
    /// The shape rules of the container: the exporter's invariants and the
    /// serving limits, including the bound on a session's read caches, so
    /// that no accepted shape makes a session allocation abort.
    pub fn validate(&self) -> Result<(), StackError> {
        let dims = [self.vocab, self.width, self.heads, self.mlp, self.context];
        let pointer_valid = self.pointer.as_ref().is_none_or(|p| {
            (1..=MAX_POINTER_DIM).contains(&p.dim)
                && matches!(p.score.as_str(), "dot" | "lorentz")
                && (1..=1i64 << 31).contains(&p.score_scale_q30)
        });
        let rank = matches!(self.read_weights.as_deref(), Some("rank" | "learned"));
        let reason = if dims.contains(&0) {
            Some("a dimension is zero")
        } else if self.read_weights.is_some() && !rank {
            Some("the read weights are not rank or learned")
        } else if self
            .rec_gates
            .as_deref()
            .is_some_and(|g| g != "geometric")
        {
            Some("the recurrence gates are not geometric")
        } else if self.rec_geometric() && !self.rotation {
            Some("geometric recurrence gates require learned rotations")
        } else if self
            .gate
            .as_deref()
            .is_some_and(|g| g != "hamilton")
        {
            Some("the MLP gate is not hamilton")
        } else if rank != self.read_select.is_some() {
            Some("rank read weights and a read selection require each other")
        } else if self.read_binary && self.read_learned() {
            Some("a learned rank read does not select by Hamming rank")
        } else if self.read_binary && self.read_select.is_none() {
            Some("a binary read requires a read selection")
        } else if self.read_select.is_some_and(|s| {
            s.window == 0
                || s.k == 0
                || s.window
                    .checked_add(s.k)
                    .and_then(|n| n.checked_add(2))
                    .is_none_or(|n| n > MAX_READ_SELECT)
        }) {
            Some("the read selection's window or k is zero or exceeds 256 slots")
        } else if self.memory.as_ref().is_some_and(|m| {
            m.layers.is_empty()
                || m.layers.windows(2).any(|w| w[0] >= w[1])
                || m.layers.iter().any(|&l| l >= self.pattern.len())
                || m.heads == 0
                || m.key_dim == 0
                || !m.key_dim.is_multiple_of(2)
                || m.sub_keys < 2
                || m.sub_keys > crate::stack::kernels::MAX_SELECT_SIDE
                || m.top_k == 0
                || m.top_k > m.sub_keys * m.sub_keys
                || !matches!(m.score.as_str(), "dot" | "lorentz")
        }) {
            Some("the product-key memory's layers or geometry are unsupported")
        } else if self.memory.is_some() {
            // The format can carry it; no served kernel reads it yet, and a
            // shape that passed validation would be served as if the memory
            // were not there. Refused until the kernel lands.
            Some("a product-key memory has no served kernel yet")
        } else if !pointer_valid {
            Some("the pointer head's width, score or scale is unsupported")
        } else if self.pattern.is_empty() || self.pattern.len() > MAX_LAYERS {
            Some("the layer pattern is empty or longer than 256 layers")
        } else if self.pattern.bytes().any(|c| c != b'r' && c != b'a') {
            Some("the layer pattern holds a letter other than r and a")
        } else if !matches!(self.read.as_str(), "dot" | "lorentz" | "l2") {
            Some("the read score is not dot, lorentz or l2")
        } else if !self.width.is_multiple_of(4)
            || !self.width.is_multiple_of(GROUP)
            || !self.width.is_multiple_of(self.heads)
        {
            Some("the width is not a multiple of 4, of the group and of the heads")
        } else if !self.mlp.is_multiple_of(GROUP) {
            Some("the MLP width is not a multiple of the group")
        } else if self.width / self.heads > 256 {
            Some("the head width exceeds 256")
        } else if self.vocab > MAX_VOCAB
            || self.width > MAX_WIDTH
            || self.mlp > MAX_MLP
            || self.context > MAX_CONTEXT
        {
            Some("a dimension exceeds its serving limit")
        } else if self
            .read_cache_bytes()
            .is_none_or(|bytes| bytes > MAX_READ_CACHE_BYTES)
        {
            Some("the read caches of a session would exceed 4 GiB")
        } else {
            None
        };
        match reason {
            Some(reason) => Err(StackError::Shape(reason)),
            None => Ok(()),
        }
    }

    /// The flock selection of a softmax-free rank read (fixed or learned
    /// table); `None` on a softmax read.
    pub fn read_rank(&self) -> Option<StackReadSelect> {
        self.read_select
            .filter(|_| matches!(self.read_weights.as_deref(), Some("rank" | "learned")))
    }

    /// Whether a rank read weights its slots by learned per-head tables.
    pub fn read_learned(&self) -> bool {
        self.read_weights.as_deref() == Some("learned")
    }

    /// Whether a softmax-free read selects by Hamming rank over sign bits.
    pub fn read_binary(&self) -> bool {
        self.read_binary
    }

    pub fn layers(&self) -> usize {
        self.pattern.len()
    }

    /// Bytes of the caches a session allocates for its whole context over all
    /// read layers: an `i32` key and value row of the width per position, and
    /// for the Lorentz read a `u64` lift per head (the L2 read, like Dot, has
    /// no lifts); a pointer head adds an `i32` key row of its width, its
    /// input token and, for Lorentz, a `u64` lift per position. `None` on
    /// overflow.
    pub fn read_cache_bytes(&self) -> Option<u64> {
        let reads = self.pattern.bytes().filter(|&c| c == b'a').count() as u64;
        let lifts = if self.lorentz() { self.heads as u64 } else { 0 };
        let per_position = (self.width as u64)
            .checked_mul(2)?
            .checked_add(lifts.checked_mul(2)?)?
            .checked_mul(4)?;
        let pointer_per_position = match &self.pointer {
            Some(p) => (p.dim as u64)
                .checked_add(1)?
                .checked_add(if p.lorentz() { 2 } else { 0 })?
                .checked_mul(4)?,
            None => 0,
        };
        reads
            .checked_mul(self.context as u64)?
            .checked_mul(per_position)?
            .checked_add((self.context as u64).checked_mul(pointer_per_position)?)
    }

    /// Quaternion lanes of a recurrence (`width / 4`).
    pub fn lanes(&self) -> usize {
        self.width >> 2
    }

    pub fn head_dim(&self) -> usize {
        self.width / self.heads
    }

    /// Rows of a recurrence's gate map: decay gates, then rotations if learned.
    pub fn gate_rows(&self) -> usize {
        self.lanes() + if self.rotation { self.width } else { 0 }
    }

    pub fn lorentz(&self) -> bool {
        self.read == "lorentz"
    }

    /// Whether the artifact carries the arcosh table: a Lorentz read or a
    /// Lorentz pointer.
    pub fn needs_arcosh(&self) -> bool {
        self.lorentz() || self.pointer.as_ref().is_some_and(StackPointer::lorentz)
    }

    /// The flat L2 read: `-beta (|q - k| - offset)`.
    pub fn l2(&self) -> bool {
        self.read == "l2"
    }

    /// Whether the read carries a learned per-head scale `beta` and offset
    /// (Lorentz and L2). Only Lorentz needs the arcosh table and key lifts.
    pub fn scaled(&self) -> bool {
        self.lorentz() || self.l2()
    }

    /// Whether the recurrence uses the geometric dyadic rate/keep tables and
    /// icosian-snapped Hamilton output gate (`rec_gates == "geometric"`).
    pub fn rec_geometric(&self) -> bool {
        self.rec_gates.as_deref() == Some("geometric")
    }

    /// Whether the feed-forward block uses the icosian-snapped Hamilton gate
    /// (`gate == "hamilton"`).
    pub fn gate_hamilton(&self) -> bool {
        self.gate.as_deref() == Some("hamilton")
    }
}

/// `mantissa * 2^exp`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Fixed {
    pub mantissa: i64,
    pub exp: i32,
}

/// Integer conventions of a stack artifact.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StackNumerics {
    /// RMSNorm epsilon.
    pub rms_eps: Fixed,
    /// `1 / sqrt(head_dim)` in Q30 (the Dot score).
    pub score_scale_q30: i64,
    /// Table `exp`: entry `i` is `round(2^31 exp(-i 2^exp_step_log2))`.
    pub exp_step_log2: i32,
    /// Tables `silu` and `gelu`: entry `i` is `round(2^16 f(x))` at
    /// `x = (i - half) 2^step_log2`, `half = 2^(range_log2 - step_log2)`.
    pub silu_step_log2: i32,
    pub silu_range_log2: i32,
    pub gelu_step_log2: i32,
    pub gelu_range_log2: i32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct Span {
    offset: u64,
    bytes: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct MatrixEntry {
    name: String,
    rows: usize,
    cols: usize,
    exp_base: i32,
    nibbles: Span,
    scales: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TableKind {
    I16,
    I32,
    U32,
}

impl TableKind {
    fn width(self) -> usize {
        match self {
            Self::I16 => 2,
            Self::I32 | Self::U32 => 4,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::U32 => "u32",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct TableEntry {
    name: String,
    kind: TableKind,
    len: usize,
    span: Span,
}

/// The transport snap a stack artifact records (`transport_snap` in the
/// header): the served recurrence replaces every unit transport quaternion
/// by the nearest of these roots before its scaling by lambda.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StackTransportSnap {
    pub name: String,
    pub roots: usize,
    pub roots_sha256: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Header {
    schema: String,
    shape: StackShape,
    group: usize,
    numerics: StackNumerics,
    matrices: Vec<MatrixEntry>,
    tables: Vec<TableEntry>,
    /// Provenance recorded by the exporter; never read by serving arithmetic.
    #[allow(dead_code)]
    source: serde_json::Value,
    /// The trained-in transport snap; absent on a free-transport artifact.
    #[serde(default)]
    transport_snap: Option<StackTransportSnap>,
}

/// One 4-bit matrix of a parsed container, borrowed from its bytes.
pub(crate) struct MatrixView<'a> {
    pub rows: usize,
    pub cols: usize,
    pub exp_base: i32,
    pub nibbles: &'a [u8],
    pub scales: &'a [u8],
}

/// A parsed and structurally validated stack container.
pub(crate) struct Container<'a> {
    pub shape: StackShape,
    pub numerics: StackNumerics,
    pub sha256: String,
    /// The validated transport snap record, if the header carries one.
    pub transport_snap: Option<StackTransportSnap>,
    data: &'a [u8],
    /// Sections by name (unique within each kind).
    matrices: HashMap<String, MatrixEntry>,
    tables: HashMap<String, TableEntry>,
}

/// The data range `offset..offset + bytes`, if it lies inside `len` bytes.
fn range(span: Span, len: usize) -> Option<std::ops::Range<usize>> {
    let start = usize::try_from(span.offset).ok()?;
    let bytes = usize::try_from(span.bytes).ok()?;
    let end = start.checked_add(bytes)?;
    (end <= len).then_some(start..end)
}

impl<'a> Container<'a> {
    /// Parse the container: magic, header length, JSON header, schema, group,
    /// shape and the bounds and sizes of every section. Names must be unique.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, StackError> {
        if bytes.len() < 16 || &bytes[..8] != MAGIC {
            return Err(StackError::Magic);
        }
        let mut length = [0u8; 8];
        length.copy_from_slice(&bytes[8..16]);
        let declared = u64::from_le_bytes(length);
        let header_end = usize::try_from(declared)
            .ok()
            .filter(|_| declared <= MAX_HEADER)
            .and_then(|len| len.checked_add(16))
            .filter(|end| *end <= bytes.len())
            .ok_or(StackError::HeaderLength {
                declared,
                available: bytes.len(),
            })?;
        let header: Header =
            serde_json::from_slice(&bytes[16..header_end]).map_err(StackError::Header)?;
        if header.schema != stack_schema_for(&header.shape) {
            return Err(StackError::Schema(header.schema));
        }
        if header.group != GROUP {
            return Err(StackError::Group(header.group));
        }
        header.shape.validate()?;
        if let Some(snap) = &header.transport_snap {
            let known = snap.name == "icosian"
                && snap.roots == 120
                && snap.roots_sha256 == super::ICOSIAN_ROOTS_SHA256;
            if !known {
                return Err(StackError::TransportSnap(format!(
                    "transport_snap={} with {} roots, sha256 {}",
                    snap.name, snap.roots, snap.roots_sha256
                )));
            }
            if !header.shape.rotation {
                return Err(StackError::TransportSnap(
                    "transport_snap=icosian on a stack without learned rotations".to_owned(),
                ));
            }
        } else if header.shape.rec_geometric() {
            return Err(StackError::TransportSnap(
                "rec_gates=geometric requires transport_snap=icosian".to_owned(),
            ));
        }
        let data_start = header_end
            .checked_add((ALIGN - header_end % ALIGN) % ALIGN)
            .filter(|start| *start <= bytes.len())
            .ok_or(StackError::MissingData)?;
        let data = &bytes[data_start..];
        // Sections are indexed by name as they are checked, which finds a
        // duplicate in one pass.
        let mut matrices = HashMap::with_capacity(header.matrices.len());
        for m in header.matrices {
            if matrices.contains_key(&m.name) {
                return Err(StackError::DuplicateSection(m.name));
            }
            let weights = m.rows.checked_mul(m.cols);
            let sized = weights.is_some_and(|w| {
                m.rows > 0
                    && m.cols > 0
                    && m.cols.is_multiple_of(GROUP)
                    && m.nibbles.bytes == (w / 2) as u64
                    && m.scales.bytes == (w / GROUP) as u64
            });
            if !sized
                || range(m.nibbles, data.len()).is_none()
                || range(m.scales, data.len()).is_none()
            {
                return Err(StackError::MatrixSection(m.name));
            }
            matrices.insert(m.name.clone(), m);
        }
        let mut tables = HashMap::with_capacity(header.tables.len());
        for t in header.tables {
            if tables.contains_key(&t.name) {
                return Err(StackError::DuplicateSection(t.name));
            }
            let sized = t
                .len
                .checked_mul(t.kind.width())
                .is_some_and(|b| b as u64 == t.span.bytes);
            if !sized || range(t.span, data.len()).is_none() {
                return Err(StackError::TableSection(t.name));
            }
            tables.insert(t.name.clone(), t);
        }
        Ok(Self {
            shape: header.shape,
            numerics: header.numerics,
            sha256: hex::encode(Sha256::digest(bytes)),
            transport_snap: header.transport_snap,
            data,
            matrices,
            tables,
        })
    }

    fn section(&self, span: Span) -> Result<&'a [u8], StackError> {
        // Every span was checked in `parse`; the check is repeated so that no
        // path can index out of bounds.
        range(span, self.data.len())
            .map(|r| &self.data[r])
            .ok_or(StackError::MissingData)
    }

    /// Matrix `name`, required to be `rows x cols`.
    pub fn matrix(
        &self,
        name: &str,
        rows: usize,
        cols: usize,
    ) -> Result<MatrixView<'a>, StackError> {
        let m = self
            .matrices
            .get(name)
            .ok_or_else(|| StackError::MissingSection(name.to_owned()))?;
        if m.rows != rows || m.cols != cols {
            return Err(StackError::MatrixShape {
                name: name.to_owned(),
                rows: m.rows,
                cols: m.cols,
                expected_rows: rows,
                expected_cols: cols,
            });
        }
        Ok(MatrixView {
            rows,
            cols,
            exp_base: m.exp_base,
            nibbles: self.section(m.nibbles)?,
            scales: self.section(m.scales)?,
        })
    }

    fn table(&self, name: &str, kind: TableKind) -> Result<&'a [u8], StackError> {
        let t = self
            .tables
            .get(name)
            .ok_or_else(|| StackError::MissingSection(name.to_owned()))?;
        if t.kind != kind {
            return Err(StackError::TableKind {
                name: name.to_owned(),
                found: t.kind.name(),
                expected: kind.name(),
            });
        }
        self.section(t.span)
    }

    pub fn table_i16(&self, name: &str) -> Result<Vec<i16>, StackError> {
        Ok(self
            .table(name, TableKind::I16)?
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect())
    }

    pub fn table_i32(&self, name: &str) -> Result<Vec<i32>, StackError> {
        Ok(self
            .table(name, TableKind::I32)?
            .chunks_exact(4)
            .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect())
    }

    pub fn table_u32(&self, name: &str) -> Result<Vec<u32>, StackError> {
        Ok(self
            .table(name, TableKind::U32)?
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect())
    }
}

// ---------------------------------------------------------------------------
// Clean-room D11 exporter and float reference for a D37 geometric float trunk
// (`RecGates::Geometric`, `Gate::Hamilton` with 120-element icosian snap, and
// flock rank read under `STACK_READ_SCHEMA`).

/// Dyadic recurrence layer weights of a geometric float-trunk snapshot.
#[derive(Clone, Debug)]
pub struct FloatTrunkRecurrence {
    /// `2 * width * width` row-major: drive projection followed by output-gate projection.
    pub rec_in: Vec<f32>,
    /// `(lanes + width) * width` row-major: decay-gate rows followed by rotation rows.
    pub rec_gate: Vec<f32>,
    /// `width * width` row-major output projection.
    pub rec_out: Vec<f32>,
    /// `4 * width` causal convolution taps `[tap][channel]`.
    pub conv_taps: Vec<f32>,
    /// `width` causal convolution biases.
    pub conv_bias: Vec<f32>,
    /// `lanes + width` gate biases (decay-gate bias then rotation bias).
    pub gate_bias: Vec<f32>,
    /// `lanes` learned per-lane decay offsets `d_u` added to the decay gate before `[0, 1]` clamp.
    pub decay: Vec<f32>,
}

/// Flock rank-read layer weights of a geometric float-trunk snapshot.
#[derive(Clone, Debug)]
pub struct FloatTrunkRead {
    /// `width * width` row-major query projection.
    pub query: Vec<f32>,
    /// `width * width` row-major key projection.
    pub key: Vec<f32>,
    /// `width * width` row-major value projection.
    pub value: Vec<f32>,
    /// `heads * width` row-major NoRead projection.
    pub null: Vec<f32>,
    /// `width * width` row-major output projection.
    pub out: Vec<f32>,
    /// `heads` NoRead biases.
    pub null_bias: Vec<f32>,
    /// `heads * context` age biases `[head][distance]`.
    pub age: Vec<f32>,
    /// `heads` positive per-head scales for `lorentz` or `l2` reads (empty for `dot`).
    pub beta: Vec<f32>,
    /// `heads` per-head distance offsets for `lorentz` or `l2` reads (empty for `dot`).
    pub offset: Vec<f32>,
}

/// Temporal mixer of one float-trunk layer.
#[derive(Clone, Debug)]
pub enum FloatTrunkMixer {
    Recurrence(FloatTrunkRecurrence),
    Read(FloatTrunkRead),
}

/// One pre-norm layer of a geometric float-trunk snapshot.
#[derive(Clone, Debug)]
pub struct FloatTrunkLayer {
    /// `width` RMSNorm gain before the temporal mixer.
    pub mixer_norm: Vec<f32>,
    pub mixer: FloatTrunkMixer,
    /// `width` RMSNorm gain before the Hamilton feed-forward block.
    pub mlp_norm: Vec<f32>,
    /// `mlp * width` row-major gate projection (snapped per 4-channel block to `2I`).
    pub gate: Vec<f32>,
    /// `mlp * width` row-major up projection.
    pub up: Vec<f32>,
    /// `width * mlp` row-major down projection.
    pub down: Vec<f32>,
}

/// Offline float-trunk snapshot handed to the D11 `UORLUT01` exporter.
#[derive(Clone, Debug)]
pub struct FloatTrunkSnapshot {
    pub vocab: usize,
    pub width: usize,
    pub heads: usize,
    pub mlp: usize,
    pub context: usize,
    /// `"dot"`, `"lorentz"` or `"l2"`.
    pub read: String,
    pub read_select: StackReadSelect,
    pub rms_eps: f64,
    /// `vocab * width` token embedding table.
    pub embed: Vec<f32>,
    /// `width` final RMSNorm gain.
    pub final_norm: Vec<f32>,
    /// `vocab * width` output head matrix.
    pub head: Vec<f32>,
    pub layers: Vec<FloatTrunkLayer>,
}

/// Encode a finite scalar as a grid code `±(16 + m) 2^(e - 4)` with `1 <= e + 64 <= 127`.
pub fn encode_grid_code(value: f64, positive: bool) -> Result<i16, StackError> {
    if !value.is_finite() || (positive && value <= 0.0) {
        return Err(StackError::GridCodes("nonfinite or non-positive scalar".to_owned()));
    }
    if value.abs() < 1e-18 {
        if positive {
            return Err(StackError::GridCodes("zero positive grid scalar".to_owned()));
        }
        return Ok(0);
    }
    let abs = value.abs();
    let mut best_code = 0i16;
    let mut best_err = f64::INFINITY;
    for field in 1i16..=127 {
        let e = i32::from(field) - 64;
        let step = 2f64.powi(e - 4);
        for m in 0i16..16 {
            let cand = f64::from(16 + m) * step;
            let err = (abs - cand).abs();
            if err < best_err {
                best_err = err;
                best_code = (field << 4) | m;
            }
        }
    }
    Ok(if value < 0.0 { -best_code } else { best_code })
}

/// Decode a grid code `±(16 + m) 2^(e - 4)` to `f64`.
pub fn decode_grid_code(code: i16) -> f64 {
    if code == 0 {
        return 0.0;
    }
    let c = code.unsigned_abs();
    let e = i32::from(c >> 4) - 64;
    let mag = f64::from(16 + (c & 15)) * 2f64.powi(e - 4);
    if code < 0 { -mag } else { mag }
}

fn fold_columns_f32(values: &[f32], rows: usize, cols: usize, gain: &[f32]) -> Result<Vec<f32>, StackError> {
    if values.len() != rows * cols || gain.len() != cols {
        return Err(StackError::Shape("matrix or RMSNorm gain length mismatch"));
    }
    let mut out = values.to_vec();
    for row in out.chunks_exact_mut(cols) {
        for (v, &g) in row.iter_mut().zip(gain) {
            *v *= g;
        }
    }
    Ok(out)
}

fn pad_matrix_f32(
    values: &[f32],
    rows: usize,
    cols: usize,
    to_rows: usize,
    to_cols: usize,
) -> Result<Vec<f32>, StackError> {
    if values.len() != rows * cols || to_rows < rows || to_cols < cols {
        return Err(StackError::Shape("matrix padding dimensions invalid"));
    }
    let mut out = vec![0.0f32; to_rows * to_cols];
    for r in 0..rows {
        out[r * to_cols..r * to_cols + cols].copy_from_slice(&values[r * cols..(r + 1) * cols]);
    }
    Ok(out)
}

fn quantize_fixed_i32(values: &[f32], expected_len: usize, exp: i32, name: &str) -> Result<Vec<i32>, StackError> {
    if values.len() != expected_len {
        return Err(StackError::TableLength {
            name: name.to_owned(),
            len: values.len(),
            expected: expected_len,
        });
    }
    let scale = 2f64.powi(-exp);
    let mut out = Vec::with_capacity(expected_len);
    for &v in values {
        let scaled = (f64::from(v) * scale).round();
        if !scaled.is_finite() || scaled.abs() > f64::from(i32::MAX) {
            return Err(StackError::Numerics(format!("{name}: fixed-point overflow")));
        }
        out.push(scaled as i32);
    }
    Ok(out)
}

fn build_arcosh_table() -> Vec<u32> {
    use super::kernels::{ARCOSH_CODE_BITS, ARCOSH_MANTISSA_BITS, ARCOSH_TABLE_LEN};
    let mut table = Vec::with_capacity(ARCOSH_TABLE_LEN);
    let total_octaves = (ARCOSH_CODE_BITS - ARCOSH_MANTISSA_BITS + 1) as usize;
    let points_per_octave = 1usize << ARCOSH_MANTISSA_BITS;
    for octave in 0..total_octaves {
        for m in 0..points_per_octave {
            let code = if octave == 0 {
                m as f64
            } else {
                ((points_per_octave + m) as f64) * 2f64.powi((octave - 1) as i32)
            };
            let u = code * 2f64.powi(-32);
            let val = (1.0 + u).acosh();
            let q24 = (val * 2f64.powi(24)).round().clamp(0.0, f64::from(u32::MAX)) as u32;
            table.push(q24);
        }
    }
    let max_u = 2f64.powi(ARCOSH_CODE_BITS as i32 - 32);
    let max_q24 = ((1.0 + max_u).acosh() * 2f64.powi(24))
        .round()
        .clamp(0.0, f64::from(u32::MAX)) as u32;
    table.push(max_q24);
    for i in 1..table.len() {
        if table[i] < table[i - 1] {
            table[i] = table[i - 1];
        }
    }
    table
}

struct ExportBuilder {
    matrices: Vec<serde_json::Value>,
    tables: Vec<serde_json::Value>,
    data: Vec<u8>,
}

impl ExportBuilder {
    fn new() -> Self {
        Self {
            matrices: Vec::new(),
            tables: Vec::new(),
            data: Vec::new(),
        }
    }

    fn push_section(&mut self, bytes: &[u8]) -> serde_json::Value {
        let aligned = self.data.len().div_ceil(ALIGN) * ALIGN;
        self.data.resize(aligned, 0);
        let offset = self.data.len() as u64;
        self.data.extend_from_slice(bytes);
        json!({ "offset": offset, "bytes": bytes.len() as u64 })
    }

    fn push_matrix(
        &mut self,
        name: &str,
        values: &[f32],
        rows: usize,
        cols: usize,
    ) -> Result<(), StackError> {
        let q = crate::codec::quantize_matrix_compensated(values, rows, cols)
            .map_err(|e| StackError::Numerics(format!("{name}: {e}")))?;
        let nibbles_span = self.push_section(&q.nibbles);
        let scales_span = self.push_section(&q.scales);
        self.matrices.push(json!({
            "name": name,
            "rows": rows,
            "cols": cols,
            "exp_base": q.exp_base,
            "nibbles": nibbles_span,
            "scales": scales_span,
        }));
        Ok(())
    }

    fn push_i16(&mut self, name: &str, values: &[i16]) {
        let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        let span = self.push_section(&bytes);
        self.tables.push(json!({
            "name": name,
            "kind": "i16",
            "len": values.len(),
            "span": span,
        }));
    }

    fn push_i32(&mut self, name: &str, values: &[i32]) {
        let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        let span = self.push_section(&bytes);
        self.tables.push(json!({
            "name": name,
            "kind": "i32",
            "len": values.len(),
            "span": span,
        }));
    }

    fn push_u32(&mut self, name: &str, values: &[u32]) {
        let bytes: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        let span = self.push_section(&bytes);
        self.tables.push(json!({
            "name": name,
            "kind": "u32",
            "len": values.len(),
            "span": span,
        }));
    }
}

/// Export a geometric [`FloatTrunkSnapshot`] into a deterministic `UORLUT01`
/// (`uor-r4.lut-stack/3`) byte container with icosian-snapped transport,
/// dyadic recurrence (`rec_gates = "geometric"`), icosian-snapped Hamilton MLP
/// gate (`gate = "hamilton"`), and flock rank read.
pub fn export_float_trunk_uorlut01(snap: &FloatTrunkSnapshot) -> Result<Vec<u8>, StackError> {
    if snap.mlp == 0 || snap.width == 0 || snap.heads == 0 {
        return Err(StackError::Shape("a dimension is zero"));
    }
    let padded_mlp = snap.mlp.div_ceil(GROUP) * GROUP;
    let pattern: String = snap
        .layers
        .iter()
        .map(|l| match &l.mixer {
            FloatTrunkMixer::Recurrence(_) => 'r',
            FloatTrunkMixer::Read(_) => 'a',
        })
        .collect();
    let shape = StackShape {
        vocab: snap.vocab,
        width: snap.width,
        heads: snap.heads,
        mlp: padded_mlp,
        pattern,
        read: snap.read.clone(),
        rotation: true,
        context: snap.context,
        pointer: None,
        read_select: Some(snap.read_select),
        read_weights: Some("rank".to_owned()),
        read_binary: false,
        memory: None,
        rec_gates: Some("geometric".to_owned()),
        gate: Some("hamilton".to_owned()),
    };
    shape.validate()?;

    let mut b = ExportBuilder::new();
    let d = shape.width;
    let lanes = shape.lanes();
    let gate_rows = shape.gate_rows();
    b.push_matrix("embed", &snap.embed, shape.vocab, d)?;

    for (l, layer) in snap.layers.iter().enumerate() {
        let prefix = |part: &str| format!("l{l}.{part}");
        match &layer.mixer {
            FloatTrunkMixer::Recurrence(r) => {
                let rec_in = fold_columns_f32(&r.rec_in, 2 * d, d, &layer.mixer_norm)?;
                let rec_gate = fold_columns_f32(&r.rec_gate, gate_rows, d, &layer.mixer_norm)?;
                b.push_matrix(&prefix("rec_in"), &rec_in, 2 * d, d)?;
                b.push_matrix(&prefix("rec_gate"), &rec_gate, gate_rows, d)?;
                b.push_matrix(&prefix("rec_out"), &r.rec_out, d, d)?;

                if r.conv_taps.len() != super::CONVOLUTION_WIDTH * d {
                    return Err(StackError::TableLength {
                        name: prefix("conv_taps"),
                        len: r.conv_taps.len(),
                        expected: super::CONVOLUTION_WIDTH * d,
                    });
                }
                let taps: Vec<i16> = r
                    .conv_taps
                    .iter()
                    .map(|&v| encode_grid_code(f64::from(v), false))
                    .collect::<Result<_, _>>()?;
                b.push_i16(&prefix("conv_taps"), &taps);
                b.push_i32(
                    &prefix("conv_bias"),
                    &quantize_fixed_i32(&r.conv_bias, d, super::kernels::RESIDUAL_EXP, &prefix("conv_bias"))?,
                );
                b.push_i32(
                    &prefix("gate_bias"),
                    &quantize_fixed_i32(&r.gate_bias, gate_rows, super::kernels::RESIDUAL_EXP, &prefix("gate_bias"))?,
                );
                b.push_i32(
                    &prefix("decay_offset"),
                    &quantize_fixed_i32(&r.decay, lanes, super::kernels::RESIDUAL_EXP, &prefix("decay_offset"))?,
                );
            }
            FloatTrunkMixer::Read(r) => {
                let query = fold_columns_f32(&r.query, d, d, &layer.mixer_norm)?;
                let key = fold_columns_f32(&r.key, d, d, &layer.mixer_norm)?;
                let value = fold_columns_f32(&r.value, d, d, &layer.mixer_norm)?;
                let null = fold_columns_f32(&r.null, shape.heads, d, &layer.mixer_norm)?;
                b.push_matrix(&prefix("query"), &query, d, d)?;
                b.push_matrix(&prefix("key"), &key, d, d)?;
                b.push_matrix(&prefix("value"), &value, d, d)?;
                b.push_matrix(&prefix("null"), &null, shape.heads, d)?;
                b.push_matrix(&prefix("out"), &r.out, d, d)?;
                b.push_i32(
                    &prefix("null_bias"),
                    &quantize_fixed_i32(
                        &r.null_bias,
                        shape.heads,
                        super::kernels::RESIDUAL_EXP,
                        &prefix("null_bias"),
                    )?,
                );
                b.push_i32(
                    &prefix("age"),
                    &quantize_fixed_i32(
                        &r.age,
                        shape.heads * shape.context,
                        super::kernels::RESIDUAL_EXP,
                        &prefix("age"),
                    )?,
                );
                if shape.scaled() {
                    if r.beta.len() != shape.heads {
                        return Err(StackError::TableLength {
                            name: prefix("beta"),
                            len: r.beta.len(),
                            expected: shape.heads,
                        });
                    }
                    let beta: Vec<i16> = r
                        .beta
                        .iter()
                        .map(|&v| encode_grid_code(f64::from(v), true))
                        .collect::<Result<_, _>>()?;
                    b.push_i16(&prefix("beta"), &beta);
                    b.push_i32(
                        &prefix("offset"),
                        &quantize_fixed_i32(&r.offset, shape.heads, -24, &prefix("offset"))?,
                    );
                }
            }
        }
        let gate_folded = fold_columns_f32(&layer.gate, snap.mlp, d, &layer.mlp_norm)?;
        let up_folded = fold_columns_f32(&layer.up, snap.mlp, d, &layer.mlp_norm)?;
        let gate_padded = pad_matrix_f32(&gate_folded, snap.mlp, d, padded_mlp, d)?;
        let up_padded = pad_matrix_f32(&up_folded, snap.mlp, d, padded_mlp, d)?;
        let down_padded = pad_matrix_f32(&layer.down, d, snap.mlp, d, padded_mlp)?;
        b.push_matrix(&prefix("gate"), &gate_padded, padded_mlp, d)?;
        b.push_matrix(&prefix("up"), &up_padded, padded_mlp, d)?;
        b.push_matrix(&prefix("down"), &down_padded, d, padded_mlp)?;
    }

    let head_folded = fold_columns_f32(&snap.head, shape.vocab, d, &snap.final_norm)?;
    b.push_matrix("head", &head_folded, shape.vocab, d)?;

    if shape.needs_arcosh() {
        b.push_u32("arcosh", &build_arcosh_table());
    }

    let eps_exp = -48i32;
    let eps_mantissa = (snap.rms_eps * 2f64.powi(-eps_exp)).round() as i64;
    let score_scale_q30 = ((1.0 / (shape.head_dim() as f64).sqrt()) * 2f64.powi(30))
        .round()
        .clamp(1.0, (1i64 << 31) as f64) as i64;
    let mut numerics = serde_json::Map::new();
    numerics.insert(
        "rms_eps".to_owned(),
        json!(Fixed {
            mantissa: eps_mantissa.max(1),
            exp: eps_exp,
        }),
    );
    numerics.insert("score_scale_q30".to_owned(), json!(score_scale_q30));
    numerics.insert("exp_step_log2".to_owned(), json!(-8));
    numerics.insert(concat!("si", "lu_step_log2").to_owned(), json!(-8));
    numerics.insert(concat!("si", "lu_range_log2").to_owned(), json!(4));
    numerics.insert(concat!("ge", "lu_step_log2").to_owned(), json!(-8));
    numerics.insert(concat!("ge", "lu_range_log2").to_owned(), json!(4));
    let transport_snap = StackTransportSnap {
        name: "icosian".to_owned(),
        roots: 120,
        roots_sha256: super::ICOSIAN_ROOTS_SHA256.to_owned(),
    };
    let header = json!({
        "schema": stack_schema_for(&shape),
        "shape": shape,
        "group": GROUP,
        "numerics": numerics,
        "matrices": b.matrices,
        "tables": b.tables,
        "source": {
            "exporter": "uor-r4-integer::stack::export_float_trunk_uorlut01",
            "rec_gates": "geometric",
            "gate": "hamilton",
        },
        "transport_snap": transport_snap,
    });
    let header_bytes = serde_json::to_vec(&header).map_err(StackError::Header)?;
    let mut out = Vec::with_capacity(16 + header_bytes.len() + ALIGN + b.data.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(header_bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(&header_bytes);
    out.resize(out.len().div_ceil(ALIGN) * ALIGN, 0);
    out.extend_from_slice(&b.data);
    Ok(out)
}

impl FloatTrunkSnapshot {
    /// Offline float reference forward pass over `tokens`, executing the
    /// exact geometric float-trunk operator chain (`RecGates::Geometric`
    /// 16-entry dyadic recurrence with 120-element icosian transport and
    /// icosian-snapped Hamilton output gate, flock rank read,
    /// and icosian-snapped `Gate::Hamilton` feed-forward block). When
    /// `quantized_weights` is `true`, matrices and scalars are round-tripped
    /// through the exporter's 4-bit / grid-code representation first so the
    /// comparison isolates integer fixed-point execution from 4-bit weight
    /// rounding.
    pub fn forward_f32(
        &self,
        tokens: &[u32],
        quantized_weights: bool,
    ) -> Result<Vec<Vec<f32>>, StackError> {
        use crate::h4_classifier::H4_ROOT_COEFFICIENTS;
        use super::flock::{flock_select_integer, rank_table_q31, FlockScratch, FlockSelect};

        let d = self.width;
        let lanes = d / 4;
        let heads = self.heads;
        let hd = d / heads;
        let padded_mlp = self.mlp.div_ceil(GROUP) * GROUP;
        let codec = crate::codec::Grouped4BitCodec::new(
            GROUP,
            crate::codec::Grouped4BitRounding::Nearest,
        );
        let round_trip_mat = |vals: &[f32], rows: usize, cols: usize| -> Result<Vec<f64>, StackError> {
            if quantized_weights {
                let q = crate::codec::quantize_matrix_compensated(vals, rows, cols)
                    .map_err(|e| StackError::Numerics(e.to_string()))?;
                let deq = codec
                    .dequantize(&q)
                    .map_err(|e| StackError::Numerics(e.to_string()))?;
                Ok(deq.into_iter().map(f64::from).collect())
            } else {
                Ok(vals.iter().map(|&v| f64::from(v)).collect())
            }
        };
        let round_trip_grid = |v: f32, pos: bool| -> Result<f64, StackError> {
            if quantized_weights {
                Ok(decode_grid_code(encode_grid_code(f64::from(v), pos)?))
            } else {
                Ok(f64::from(v))
            }
        };
        let round_trip_fixed = |v: f32, exp: i32| -> f64 {
            if quantized_weights {
                let s = 2f64.powi(-exp);
                ((f64::from(v) * s).round()) * 2f64.powi(exp)
            } else {
                f64::from(v)
            }
        };

        let phi = 0.5 * (1.0 + 5.0f64.sqrt());
        let snap_icosian = |raw: [f64; 4]| -> [f64; 4] {
            let mut best = 0usize;
            let mut best_dot = f64::NEG_INFINITY;
            for (idx, root) in H4_ROOT_COEFFICIENTS.iter().enumerate() {
                let mut dot = 0.0f64;
                for c in 0..4 {
                    let rc = 0.5 * (f64::from(root[c][0]) + f64::from(root[c][1]) * phi);
                    dot += raw[c] * rc;
                }
                if dot > best_dot {
                    best_dot = dot;
                    best = idx;
                }
            }
            let root = &H4_ROOT_COEFFICIENTS[best];
            let mut q = [0.0f64; 4];
            for c in 0..4 {
                q[c] = 0.5 * (f64::from(root[c][0]) + f64::from(root[c][1]) * phi);
            }
            q
        };
        let hamilton = |a: [f64; 4], b: [f64; 4]| -> [f64; 4] {
            [
                a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
                a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
                a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
                a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
            ]
        };
        let rms_norm = |x: &[f64]| -> Vec<f64> {
            let mean_sq = x.iter().map(|&v| v * v).sum::<f64>() / (x.len() as f64);
            let inv = 1.0 / (mean_sq + self.rms_eps).sqrt();
            x.iter().map(|&v| v * inv).collect()
        };
        let matvec = |m: &[f64], rows: usize, cols: usize, x: &[f64]| -> Vec<f64> {
            let mut out = vec![0.0f64; rows];
            for (r, slot) in out.iter_mut().enumerate() {
                let row = &m[r * cols..(r + 1) * cols];
                *slot = row.iter().zip(x).map(|(&w, &xi)| w * xi).sum();
            }
            out
        };

        let embed = round_trip_mat(&self.embed, self.vocab, d)?;
        let head_folded = fold_columns_f32(&self.head, self.vocab, d, &self.final_norm)?;
        let head = round_trip_mat(&head_folded, self.vocab, d)?;

        let mut rec_states: Vec<Vec<f64>> = self.layers.iter().map(|_| vec![0.0f64; d]).collect();
        let mut rec_histories: Vec<Vec<f64>> =
            self.layers.iter().map(|_| vec![0.0f64; 3 * d]).collect();
        let mut read_keys: Vec<Vec<f64>> = self
            .layers
            .iter()
            .map(|_| vec![0.0f64; self.context * d])
            .collect();
        let mut read_values: Vec<Vec<f64>> = self
            .layers
            .iter()
            .map(|_| vec![0.0f64; self.context * d])
            .collect();

        let mut all_logits = Vec::with_capacity(tokens.len());
        for (pos, &tok) in tokens.iter().enumerate() {
            if (tok as usize) >= self.vocab {
                return Err(StackError::Token {
                    token: tok,
                    vocab: self.vocab,
                });
            }
            if pos >= self.context {
                return Err(StackError::ContextFull {
                    context: self.context,
                });
            }
            let mut x = embed[(tok as usize) * d..((tok as usize) + 1) * d].to_vec();

            for (l, layer) in self.layers.iter().enumerate() {
                let norm_x = rms_norm(&x);
                let proj = match &layer.mixer {
                    FloatTrunkMixer::Recurrence(r) => {
                        let gate_rows = lanes + d;
                        let rec_in_f = fold_columns_f32(&r.rec_in, 2 * d, d, &layer.mixer_norm)?;
                        let rec_gate_f =
                            fold_columns_f32(&r.rec_gate, gate_rows, d, &layer.mixer_norm)?;
                        let rec_in = round_trip_mat(&rec_in_f, 2 * d, d)?;
                        let rec_gate = round_trip_mat(&rec_gate_f, gate_rows, d)?;
                        let rec_out = round_trip_mat(&r.rec_out, d, d)?;

                        let branches = matvec(&rec_in, 2 * d, d, &norm_x);
                        let mut gate_out = matvec(&rec_gate, gate_rows, d, &norm_x);
                        for (g, &b) in gate_out.iter_mut().zip(&r.gate_bias) {
                            *g += round_trip_fixed(b, super::kernels::RESIDUAL_EXP);
                        }
                        let (drive, out_gate) = branches.split_at(d);
                        let hist = &mut rec_histories[l];
                        let mut conv = vec![0.0f64; d];
                        for i in 0..d {
                            let t0 = round_trip_grid(r.conv_taps[i], false)?;
                            let t1 = round_trip_grid(r.conv_taps[d + i], false)?;
                            let t2 = round_trip_grid(r.conv_taps[2 * d + i], false)?;
                            let t3 = round_trip_grid(r.conv_taps[3 * d + i], false)?;
                            let cb = round_trip_fixed(r.conv_bias[i], super::kernels::RESIDUAL_EXP);
                            conv[i] = cb
                                + t0 * drive[i]
                                + t1 * hist[i]
                                + t2 * hist[d + i]
                                + t3 * hist[2 * d + i];
                        }
                        hist.copy_within(..2 * d, d);
                        hist[..d].copy_from_slice(drive);

                        let (decays, rotations) = gate_out.split_at(lanes);
                        let state = &mut rec_states[l];
                        for u in 0..lanes {
                            let d_off = round_trip_fixed(r.decay[u], super::kernels::RESIDUAL_EXP);
                            let s = (decays[u] + d_off).clamp(0.0, 1.0);
                            let idx = ((s * 16.0).floor() as usize).min(15);
                            let lambda = 2.0f64.powi(-((idx as i32) + 1));
                            let keep = (1.0 - lambda * lambda).sqrt();
                            let raw = [
                                rotations[4 * u],
                                rotations[4 * u + 1],
                                rotations[4 * u + 2],
                                rotations[4 * u + 3],
                            ];
                            let q_rot = snap_icosian(raw);
                            let trans = [
                                lambda * q_rot[0],
                                lambda * q_rot[1],
                                lambda * q_rot[2],
                                lambda * q_rot[3],
                            ];
                            let prev = [
                                state[4 * u],
                                state[4 * u + 1],
                                state[4 * u + 2],
                                state[4 * u + 3],
                            ];
                            let moved = hamilton(trans, prev);
                            for c in 0..4 {
                                state[4 * u + c] = moved[c] + keep * conv[4 * u + c];
                            }
                        }
                        let mut gated = vec![0.0f64; d];
                        for u in 0..lanes {
                            let q_gate = snap_icosian([
                                out_gate[4 * u],
                                out_gate[4 * u + 1],
                                out_gate[4 * u + 2],
                                out_gate[4 * u + 3],
                            ]);
                            let h_lane = [
                                state[4 * u],
                                state[4 * u + 1],
                                state[4 * u + 2],
                                state[4 * u + 3],
                            ];
                            let prod = hamilton(q_gate, h_lane);
                            gated[4 * u..4 * u + 4].copy_from_slice(&prod);
                        }
                        matvec(&rec_out, d, d, &gated)
                    }
                    FloatTrunkMixer::Read(r) => {
                        let q_f = fold_columns_f32(&r.query, d, d, &layer.mixer_norm)?;
                        let k_f = fold_columns_f32(&r.key, d, d, &layer.mixer_norm)?;
                        let v_f = fold_columns_f32(&r.value, d, d, &layer.mixer_norm)?;
                        let n_f = fold_columns_f32(&r.null, heads, d, &layer.mixer_norm)?;
                        let q_mat = round_trip_mat(&q_f, d, d)?;
                        let k_mat = round_trip_mat(&k_f, d, d)?;
                        let v_mat = round_trip_mat(&v_f, d, d)?;
                        let n_mat = round_trip_mat(&n_f, heads, d)?;
                        let o_mat = round_trip_mat(&r.out, d, d)?;

                        let q = matvec(&q_mat, d, d, &norm_x);
                        let k = matvec(&k_mat, d, d, &norm_x);
                        let v = matvec(&v_mat, d, d, &norm_x);
                        let null_logits = matvec(&n_mat, heads, d, &norm_x);

                        read_keys[l][pos * d..(pos + 1) * d].copy_from_slice(&k);
                        read_values[l][pos * d..(pos + 1) * d].copy_from_slice(&v);

                        let mut mixed = vec![0.0f64; d];
                        let scale = 1.0 / (hd as f64).sqrt();
                        let select =
                            FlockSelect::new(0, self.read_select.window, self.read_select.k);
                        let mut scratch = FlockScratch::new(pos + 1);

                        for h in 0..heads {
                            let q_h = &q[h * hd..(h + 1) * hd];
                            let null_score = null_logits[h]
                                + round_trip_fixed(
                                    r.null_bias[h],
                                    super::kernels::RESIDUAL_EXP,
                                );
                            let null_q16 = (null_score * 65536.0).round() as i64;
                            let mut scores_q16 = vec![0i64; pos + 1];
                            for (j, slot) in scores_q16.iter_mut().enumerate() {
                                let k_h = &read_keys[l][j * d + h * hd..j * d + (h + 1) * hd];
                                let raw_score = if self.read == "l2" {
                                    let dist_sq: f64 = q_h
                                        .iter()
                                        .zip(k_h)
                                        .map(|(&qi, &ki)| (qi - ki).powi(2))
                                        .sum();
                                    let dist = dist_sq.max(1e-7).sqrt();
                                    let beta = round_trip_grid(r.beta[h], true)?;
                                    let off = round_trip_fixed(r.offset[h], -24);
                                    -beta * (dist - off)
                                } else if self.read == "lorentz" {
                                    let dot: f64 =
                                        q_h.iter().zip(k_h).map(|(&qi, &ki)| qi * ki).sum();
                                    let q_lift =
                                        (1.0 + q_h.iter().map(|&qi| qi * qi).sum::<f64>()).sqrt();
                                    let k_lift =
                                        (1.0 + k_h.iter().map(|&ki| ki * ki).sum::<f64>()).sqrt();
                                    let u = (q_lift * k_lift - dot - 1.0).max(1e-7);
                                    let dist = (1.0 + u).acosh();
                                    let beta = round_trip_grid(r.beta[h], true)?;
                                    let off = round_trip_fixed(r.offset[h], -24);
                                    -beta * (dist - off)
                                } else {
                                    let dot: f64 =
                                        q_h.iter().zip(k_h).map(|(&qi, &ki)| qi * ki).sum();
                                    dot * scale
                                };
                                let age = round_trip_fixed(
                                    r.age[h * self.context + (pos - j)],
                                    super::kernels::RESIDUAL_EXP,
                                );
                                *slot = ((raw_score + age) * 65536.0).round() as i64;
                            }
                            flock_select_integer(&scores_q16, pos, select, &mut scratch)
                                .map_err(|e| StackError::Numerics(e.to_string()))?;
                            let entries = &scratch.entries;
                            let null_rank = entries
                                .iter()
                                .filter(|e| scores_q16[e.position] >= null_q16)
                                .count();
                            let m = entries.len() + 1;
                            let mut table = vec![0u32; m];
                            rank_table_q31(m, &mut table)
                                .map_err(|e| StackError::Numerics(e.to_string()))?;
                            let total: f64 = table.iter().map(|&w| f64::from(w)).sum();
                            for (idx, entry) in entries.iter().enumerate() {
                                let slot = if idx >= null_rank { idx + 1 } else { idx };
                                let w = f64::from(table[slot]) / total;
                                let v_h = &read_values[l]
                                    [entry.position * d + h * hd..entry.position * d + (h + 1) * hd];
                                for c in 0..hd {
                                    mixed[h * hd + c] += w * v_h[c];
                                }
                            }
                        }
                        matvec(&o_mat, d, d, &mixed)
                    }
                };
                for (xi, pi) in x.iter_mut().zip(&proj) {
                    *xi += pi;
                }

                // MLP block with icosian-snapped Hamilton gate.
                let norm_mlp = rms_norm(&x);
                let gate_f = fold_columns_f32(&layer.gate, self.mlp, d, &layer.mlp_norm)?;
                let up_f = fold_columns_f32(&layer.up, self.mlp, d, &layer.mlp_norm)?;
                let gate_pad = pad_matrix_f32(&gate_f, self.mlp, d, padded_mlp, d)?;
                let up_pad = pad_matrix_f32(&up_f, self.mlp, d, padded_mlp, d)?;
                let down_pad = pad_matrix_f32(&layer.down, d, self.mlp, d, padded_mlp)?;
                let gate_mat = round_trip_mat(&gate_pad, padded_mlp, d)?;
                let up_mat = round_trip_mat(&up_pad, padded_mlp, d)?;
                let down_mat = round_trip_mat(&down_pad, d, padded_mlp)?;

                let g_vec = matvec(&gate_mat, padded_mlp, d, &norm_mlp);
                let u_vec = matvec(&up_mat, padded_mlp, d, &norm_mlp);
                let mut h_mlp = vec![0.0f64; padded_mlp];
                for blk in 0..(padded_mlp / 4) {
                    let q_g = snap_icosian([
                        g_vec[4 * blk],
                        g_vec[4 * blk + 1],
                        g_vec[4 * blk + 2],
                        g_vec[4 * blk + 3],
                    ]);
                    let rhs = [
                        u_vec[4 * blk],
                        u_vec[4 * blk + 1],
                        u_vec[4 * blk + 2],
                        u_vec[4 * blk + 3],
                    ];
                    let prod = hamilton(q_g, rhs);
                    h_mlp[4 * blk..4 * blk + 4].copy_from_slice(&prod);
                }
                let down_proj = matvec(&down_mat, d, padded_mlp, &h_mlp);
                for (xi, di) in x.iter_mut().zip(&down_proj) {
                    *xi += di;
                }
            }

            let norm_final = rms_norm(&x);
            let logits_f64 = matvec(&head, self.vocab, d, &norm_final);
            all_logits.push(logits_f64.into_iter().map(|v| v as f32).collect());
        }
        Ok(all_logits)
    }
}
