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
pub use crate::fbs_snapshot::{FbsRoutes, FbsTrunkLayer, FbsTrunkSnapshot};

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
/// Header schema of a stack whose projections are SpiralCore/FBS route tables
/// and whose head is the native vector writer: an engine that serves only
/// dense maps refuses it instead of ignoring the route tables.
pub const STACK_FBS_SCHEMA: &str = "uor-r4.lut-stack/4";
/// The native vector writer's limits (`uor-r4-float-train`'s `WriterSpec`).
pub const WRITER_MIN_VOCAB: usize = 256;
pub const WRITER_MAX_VOCAB: usize = 8192;
pub const WRITER_MAX_WIDTH: usize = 1024;
pub const WRITER_MAX_KEPT_CELLS: usize = 16;
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
    /// Projection law: dense 4-bit maps when absent, or `spiralcore_fbs`
    /// (every map a SpiralCore/FBS route table: each output 8-block one signed
    /// axis permutation of one input 8-block, served as a gather and a sign).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection: Option<String>,
    /// Output head: the dense vocabulary map when absent, or `native_vector`
    /// (the native vector writer: `kept_cells` of 256 root-pair cells, then
    /// the L1-nearest token among those cells' rows of `writer.table`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    /// Cells the native vector writer keeps per position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kept_cells: Option<usize>,
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

/// The header schema a shape requires: [`STACK_FBS_SCHEMA`] with SpiralCore/FBS
/// projections, else [`STACK_READ_SCHEMA`] with a softmax-free read or
/// geometric recurrence/Hamilton gate, else [`STACK_POINTER_SCHEMA`] with a
/// pointer head, else [`STACK_SCHEMA`].
pub fn stack_schema_for(shape: &StackShape) -> &'static str {
    if shape.projection_fbs() {
        STACK_FBS_SCHEMA
    } else if shape.read_select.is_some() || shape.rec_geometric() || shape.gate_hamilton() {
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
        } else if self
            .projection
            .as_deref()
            .is_some_and(|p| p != "spiralcore_fbs")
        {
            Some("the projection is not spiralcore_fbs")
        } else if self.head.as_deref().is_some_and(|h| h != "native_vector") {
            Some("the head is not native_vector")
        } else if self.projection_fbs()
            && !(self.rec_geometric()
                && self.gate_hamilton()
                && self.rotation
                && self.head_native_vector())
        {
            Some("SpiralCore/FBS projections require geometric gates, the Hamilton gate, rotations and the native vector head")
        } else if self.projection_fbs() && self.pattern.bytes().any(|c| c != b'r') {
            Some("SpiralCore/FBS projections serve recurrence layers only")
        } else if self.projection_fbs()
            && (self.read_select.is_some() || self.read_weights.is_some() || self.read_binary)
        {
            Some("SpiralCore/FBS trunks carry no read selection or read weights")
        } else if self.head_native_vector() && !self.projection_fbs() {
            Some("the native vector head requires SpiralCore/FBS projections")
        } else if self.kept_cells.is_some() && !self.head_native_vector() {
            Some("kept cells belong to the native vector head")
        } else if self.head_native_vector()
            && self
                .kept_cells
                .is_none_or(|k| k == 0 || k > WRITER_MAX_KEPT_CELLS)
        {
            Some("the native vector head keeps 1 to 16 cells")
        } else if self.head_native_vector()
            && (!(WRITER_MIN_VOCAB..=WRITER_MAX_VOCAB).contains(&self.vocab)
                || self.width > WRITER_MAX_WIDTH)
        {
            Some("the native vector head serves a vocabulary of 256 to 8192 and a width up to 1024")
        } else if self.head_native_vector() && self.pointer.is_some() {
            Some("the native vector head has no pointer head")
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

    /// Whether every projection is a SpiralCore/FBS route table
    /// (`projection == "spiralcore_fbs"`).
    pub fn projection_fbs(&self) -> bool {
        self.projection.as_deref() == Some("spiralcore_fbs")
    }

    /// Whether the output head is the native vector writer
    /// (`head == "native_vector"`).
    pub fn head_native_vector(&self) -> bool {
        self.head.as_deref() == Some("native_vector")
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

/// Encode a finite scalar as a grid code `±(16 + m) 2^(e - 4)` with `1 <= e + 64 <= 127`
/// (`fbs_snapshot::grid_code`, which the ungated FBS converter also uses).
pub fn encode_grid_code(value: f64, positive: bool) -> Result<i16, StackError> {
    crate::fbs_snapshot::grid_code(value, positive).map_err(StackError::GridCodes)
}

/// Decode a grid code `±(16 + m) 2^(e - 4)` to `f64` (`fbs_snapshot::grid_value`).
pub fn decode_grid_code(code: i16) -> f64 {
    crate::fbs_snapshot::grid_value(code)
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

pub(super) fn quantize_fixed_i32(values: &[f32], expected_len: usize, exp: i32, name: &str) -> Result<Vec<i32>, StackError> {
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
        projection: None,
        head: None,
        kept_cells: None,
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

/// Refuse a table of the wrong length or with a non-finite value.
fn check_f32(name: &str, values: &[f32], expected: usize) -> Result<(), StackError> {
    if values.len() != expected {
        return Err(StackError::TableLength {
            name: name.to_owned(),
            len: values.len(),
            expected,
        });
    }
    match values.iter().position(|v| !v.is_finite()) {
        Some(i) => Err(StackError::Numerics(format!("{name}[{i}] is not finite"))),
        None => Ok(()),
    }
}

/// Check one projection's routes against its `in_dim -> out_dim` and pack each
/// as `atom | state << 16`.
fn pack_fbs_routes(
    name: &str,
    routes: &FbsRoutes,
    in_dim: usize,
    out_dim: usize,
) -> Result<Vec<u32>, StackError> {
    let (in_blocks, out_blocks) = (in_dim / 8, out_dim / 8);
    if routes.in_blocks != in_blocks || routes.out_blocks != out_blocks {
        return Err(StackError::Numerics(format!(
            "{name}: routes declare {} -> {} blocks, the trunk needs {in_blocks} -> {out_blocks}",
            routes.in_blocks, routes.out_blocks
        )));
    }
    for (part, len) in [("atoms", routes.atoms.len()), ("states", routes.states.len())] {
        if len != out_blocks {
            return Err(StackError::TableLength {
                name: format!("{name}.{part}"),
                len,
                expected: out_blocks,
            });
        }
    }
    routes
        .atoms
        .iter()
        .zip(&routes.states)
        .enumerate()
        .map(|(o, (&atom, &state))| {
            if usize::from(atom) >= in_blocks {
                return Err(StackError::Numerics(format!(
                    "{name}[{o}]: atom {atom} is not below {in_blocks} input blocks"
                )));
            }
            if state >= 64 {
                return Err(StackError::Numerics(format!(
                    "{name}[{o}]: state {state} is not below 64"
                )));
            }
            Ok(u32::from(atom) | (u32::from(state) << 16))
        })
        .collect()
}

/// Export a SpiralCore/FBS [`FbsTrunkSnapshot`] into a deterministic `UORLUT01`
/// (`uor-r4.lut-stack/4`) byte container: recurrence layers only, every
/// projection as packed FBS routes (no matrices), geometric recurrence gates,
/// the Hamilton gate, and the native vector writer table built from `embed`.
/// Width and MLP width must be multiples of 32; they are refused, not padded.
pub fn export_fbs_trunk_uorlut01(snap: &FbsTrunkSnapshot) -> Result<Vec<u8>, StackError> {
    let shape = StackShape {
        vocab: snap.vocab,
        width: snap.width,
        heads: 1,
        mlp: snap.mlp,
        pattern: "r".repeat(snap.layers.len()),
        read: "dot".to_owned(),
        rotation: true,
        context: snap.context,
        pointer: None,
        read_select: None,
        read_weights: None,
        read_binary: false,
        memory: None,
        rec_gates: Some("geometric".to_owned()),
        gate: Some("hamilton".to_owned()),
        projection: Some("spiralcore_fbs".to_owned()),
        head: Some("native_vector".to_owned()),
        kept_cells: Some(snap.kept_cells),
    };
    // `validate` holds the width and MLP width to multiples of the group (32),
    // so every route table, the gate rows' `width / 4 + width` included, fills
    // whole 8-blocks.
    shape.validate()?;
    if !(snap.rms_eps.is_finite() && snap.rms_eps > 0.0) {
        return Err(StackError::Numerics(format!(
            "rms_eps {} is not finite and positive",
            snap.rms_eps
        )));
    }
    let (d, m) = (snap.width, snap.mlp);
    let lanes = shape.lanes();
    let g = lanes + d;

    let mut b = ExportBuilder::new();
    for (l, layer) in snap.layers.iter().enumerate() {
        let prefix = |part: &str| format!("l{l}.{part}");
        for (part, routes, in_dim, out_dim) in [
            ("rec_in", &layer.rec_in, d, 2 * d),
            ("rec_gate", &layer.rec_gate, d, g),
            ("rec_out", &layer.rec_out, d, d),
            ("mlp_gate", &layer.mlp_gate, d, m),
            ("mlp_up", &layer.mlp_up, d, m),
            ("mlp_down", &layer.mlp_down, m, d),
        ] {
            let name = prefix(&format!("{part}.fbs"));
            let packed = pack_fbs_routes(&name, routes, in_dim, out_dim)?;
            b.push_u32(&name, &packed);
        }
        for (part, gain) in [("rec_norm_gain", &layer.rec_norm), ("mlp_norm_gain", &layer.mlp_norm)] {
            let name = prefix(part);
            check_f32(&name, gain, d)?;
            let codes: Vec<i16> = gain
                .iter()
                .map(|&v| encode_grid_code(f64::from(v), false))
                .collect::<Result<_, _>>()?;
            b.push_i16(&name, &codes);
        }
        check_f32(&prefix("conv_taps"), &layer.conv_taps, super::CONVOLUTION_WIDTH * d)?;
        let taps: Vec<i16> = layer
            .conv_taps
            .iter()
            .map(|&v| encode_grid_code(f64::from(v), false))
            .collect::<Result<_, _>>()?;
        b.push_i16(&prefix("conv_taps"), &taps);
        for (part, values, len) in [
            ("conv_bias", &layer.conv_bias, d),
            ("gate_bias", &layer.gate_bias, g),
            ("decay_offset", &layer.decay, lanes),
        ] {
            let name = prefix(part);
            check_f32(&name, values, len)?;
            b.push_i32(
                &name,
                &quantize_fixed_i32(values, len, super::kernels::RESIDUAL_EXP, &name)?,
            );
        }
    }

    check_f32("final_norm_gain", &snap.final_norm, d)?;
    let final_norm: Vec<i16> = snap
        .final_norm
        .iter()
        .map(|&v| encode_grid_code(f64::from(v), false))
        .collect::<Result<_, _>>()?;
    b.push_i16("final_norm_gain", &final_norm);
    check_f32("writer.table", &snap.embed, shape.vocab * d)?;
    // Q12 in i16 (about [-8, 8)): a value that does not fit would saturate and
    // serve another vector than the float trunk's, so it is refused.
    let writer: Vec<i16> = snap
        .embed
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            i16::try_from((f64::from(v) * 4096.0).round() as i64).map_err(|_| {
                StackError::Numerics(format!(
                    "writer.table[{i}] = {v} does not fit Q12 in i16"
                ))
            })
        })
        .collect::<Result<_, _>>()?;
    b.push_i16("writer.table", &writer);

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
            "exporter": "uor-r4-integer::stack::export_fbs_trunk_uorlut01",
            "projection": "spiralcore_fbs",
            "head": "native_vector",
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

/// A grid-coded scalar as the float references read it: decoded from its
/// exported grid code when `quantized`, else as trained.
fn ref_grid(v: f32, positive: bool, quantized: bool) -> Result<f64, StackError> {
    if quantized {
        Ok(decode_grid_code(encode_grid_code(f64::from(v), positive)?))
    } else {
        Ok(f64::from(v))
    }
}

/// A fixed-point scalar at `2^exp` as the float references read it.
fn ref_fixed(v: f32, exp: i32, quantized: bool) -> f64 {
    if quantized {
        crate::fbs_snapshot::fixed_value(v, exp)
    } else {
        f64::from(v)
    }
}

/// The nearest of the 120 icosian units to `raw` (largest dot product, ties to the first).
pub(super) fn ref_snap_icosian(raw: [f64; 4]) -> [f64; 4] {
    use crate::h4_classifier::H4_ROOT_COEFFICIENTS;
    let phi = 0.5 * (1.0 + 5.0f64.sqrt());
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
}

/// How near the float reference came to a discrete switch on its way to a
/// position (claude-125). Where computations of other precisions disagree at
/// a position, the smallest margin names the near-tie (0 at a tie):
/// - `transition`: the gap between the best and the second-best icosian at a
///   transition snap, relative to the rotation's length, counted only where
///   the transition moves a nonzero state;
/// - `gate`: the same at a Hamilton gate snap (output or feed-forward);
/// - `rate`: the distance of a transition rate to a step of its 16-entry
///   dyadic table.
///
/// A switch the snaps do not make (`FbsQuatMode::Normalize`) has no margin
/// (infinite). `zero_snaps` counts snaps of an all-zero vector, which every
/// root ties: there the float GPU snap and this reference pick different
/// roots, a known edge rather than a near-tie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FbsTieMargins {
    pub transition: f64,
    pub gate: f64,
    pub rate: f64,
    pub zero_snaps: u32,
}

impl FbsTieMargins {
    /// Nothing met yet.
    const NONE: Self = Self {
        transition: f64::INFINITY,
        gate: f64::INFINITY,
        rate: f64::INFINITY,
        zero_snaps: 0,
    };

    /// Fold in the snap of `q` under `mode`, at the transition or at a gate.
    pub(super) fn note_snap(&mut self, q: [f64; 4], mode: FbsQuatMode, transition: bool) {
        let raw = match mode {
            FbsQuatMode::Icosian => q,
            FbsQuatMode::Q12Icosian => q.map(|v| (v * 4096.0).round().clamp(-32768.0, 32767.0)),
            FbsQuatMode::Normalize => return,
        };
        if raw == [0.0; 4] {
            self.zero_snaps += 1;
        }
        let slot = if transition { &mut self.transition } else { &mut self.gate };
        *slot = slot.min(icosian_margin(raw));
    }
}

thread_local! {
    /// The margins of the reference forward [`FbsTrunkSnapshot::tie_margins`]
    /// runs on this thread; `None` (every other forward) records nothing.
    static TIE_MARGINS: std::cell::Cell<Option<FbsTieMargins>> = const { std::cell::Cell::new(None) };
}

/// Fold a margin into the running [`FbsTieMargins`], if one is recording.
fn note_tie(f: impl FnOnce(&mut FbsTieMargins)) {
    TIE_MARGINS.with(|cell| {
        if let Some(mut margins) = cell.get() {
            f(&mut margins);
            cell.set(Some(margins));
        }
    });
}

/// How near the snap of `raw` to the 120 icosians is to a tie: the gap
/// between the best and the second-best dot product, over `raw`'s length
/// (0 for a zero vector). The snap itself is [`ref_snap_icosian`]'s.
pub(super) fn icosian_margin(raw: [f64; 4]) -> f64 {
    use crate::h4_classifier::H4_ROOT_COEFFICIENTS;
    let len = raw.iter().map(|v| v * v).sum::<f64>().sqrt();
    if len == 0.0 {
        return 0.0;
    }
    let phi = 0.5 * (1.0 + 5.0f64.sqrt());
    let (mut best, mut second) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for root in H4_ROOT_COEFFICIENTS.iter() {
        let dot: f64 = (0..4)
            .map(|c| raw[c] * 0.5 * (f64::from(root[c][0]) + f64::from(root[c][1]) * phi))
            .sum();
        if dot > best {
            (best, second) = (dot, best);
        } else if dot > second {
            second = dot;
        }
    }
    (best - second) / len
}

/// The distance of the transition rate `s` (before its clamp) to the nearest
/// step of its 16-entry table, `k / 16` for `k` in 1..=15.
pub(super) fn rate_margin(s: f64) -> f64 {
    (1..16)
        .map(|k| (s - f64::from(k) / 16.0).abs())
        .fold(f64::INFINITY, f64::min)
}

/// The Hamilton product `a b`.
pub(super) fn ref_hamilton(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

fn ref_quat(v: &[f64], block: usize) -> [f64; 4] {
    [v[4 * block], v[4 * block + 1], v[4 * block + 2], v[4 * block + 3]]
}

/// RMS normalisation without a gain.
fn ref_rms_norm(x: &[f64], eps: f64) -> Vec<f64> {
    let mean_sq = x.iter().map(|&v| v * v).sum::<f64>() / (x.len() as f64);
    let inv = 1.0 / (mean_sq + eps).sqrt();
    x.iter().map(|&v| v * inv).collect()
}

/// How a float reference treats a quaternion the integer path snaps to the
/// 120 icosians (d11-exporter-float-trunk, claude-124).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FbsQuatMode {
    /// Snap to the nearest icosian, as the integer path does.
    Icosian,
    /// Round to Q12 steps, then snap: the float sheets' D11 export mode for
    /// the Hamilton gates (`hamilton_sheet.rs`, `try_hamilton_gate_fwd_cuda`).
    Q12Icosian,
    /// Normalize as the float CUDA path does, `q / sqrt(|q|^2 + eps)`.
    Normalize,
}

/// Which quaternions [`FbsTrunkSnapshot::forward_f32_with`] snaps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FbsReferenceSnaps {
    /// The recurrence transition's rotation (normalized with `eps = 1e-6`).
    pub transition: FbsQuatMode,
    /// The recurrence output gate and the feed-forward gate (`eps = 1e-12`).
    pub gates: FbsQuatMode,
}

impl FbsReferenceSnaps {
    /// Everything snapped, as the integer path serves it.
    pub const INTEGER: Self = Self {
        transition: FbsQuatMode::Icosian,
        gates: FbsQuatMode::Icosian,
    };
    /// Nothing snapped, as the plain float GPU trunk computes.
    pub const FLOAT: Self = Self {
        transition: FbsQuatMode::Normalize,
        gates: FbsQuatMode::Normalize,
    };
    /// The float sheets' D11 export mode: Q12-rounded, snapped gates, and a
    /// continuous transition.
    pub const FLOAT_EXPORT: Self = Self {
        transition: FbsQuatMode::Normalize,
        gates: FbsQuatMode::Q12Icosian,
    };
}

/// The epsilon the float CUDA path normalizes the transition with.
const TRANSITION_EPS: f64 = 1e-6;
/// The epsilon the float CUDA Hamilton gate normalizes with.
const GATE_EPS: f64 = 1e-12;

/// A quaternion the integer path snaps, under `mode`.
pub(super) fn ref_quat_mode(q: [f64; 4], mode: FbsQuatMode, eps: f64) -> [f64; 4] {
    match mode {
        FbsQuatMode::Icosian => ref_snap_icosian(q),
        FbsQuatMode::Q12Icosian => {
            ref_snap_icosian(q.map(|v| (v * 4096.0).round().clamp(-32768.0, 32767.0)))
        }
        FbsQuatMode::Normalize => {
            let r = (q.iter().map(|v| v * v).sum::<f64>() + eps).sqrt();
            q.map(|v| v / r)
        }
    }
}

/// The Hamilton feed-forward gate: `hamilton(gate(g), up)` per 4-block, the
/// gate snapped (or normalized) under `mode`.
pub(super) fn ref_hamilton_feed_forward(g: &[f64], up: &[f64], mode: FbsQuatMode) -> Vec<f64> {
    let mut h = vec![0.0f64; g.len()];
    for blk in 0..(g.len() / 4) {
        note_tie(|m| m.note_snap(ref_quat(g, blk), mode, false));
        let prod = ref_hamilton(ref_quat_mode(ref_quat(g, blk), mode, GATE_EPS), ref_quat(up, blk));
        h[4 * blk..4 * blk + 4].copy_from_slice(&prod);
    }
    h
}

/// The scalar tables of one geometric recurrence layer, as the float
/// references read them (`RecGates::Geometric`, Hamilton output gate).
pub(super) struct RefRecurrence {
    /// `4 * d` taps `[tap][channel]`.
    taps: Vec<f64>,
    conv_bias: Vec<f64>,
    /// `lanes` decay offsets.
    decay: Vec<f64>,
}

impl RefRecurrence {
    pub(super) fn new(taps: &[f32], conv_bias: &[f32], decay: &[f32], quantized: bool) -> Result<Self, StackError> {
        let exp = super::kernels::RESIDUAL_EXP;
        Ok(Self {
            taps: taps
                .iter()
                .map(|&v| ref_grid(v, false, quantized))
                .collect::<Result<_, _>>()?,
            conv_bias: conv_bias.iter().map(|&v| ref_fixed(v, exp, quantized)).collect(),
            decay: decay.iter().map(|&v| ref_fixed(v, exp, quantized)).collect(),
        })
    }

    /// One recurrence step: causal convolution of `drive`, the clamped dyadic
    /// rate and icosian transport per lane, then the Hamilton output gate.
    /// `gate_out` is `lanes` decay pre-activations then `d` rotation values
    /// (gate bias already added); `hist` is the `3 * d` convolution history.
    pub(super) fn step(
        &self,
        drive: &[f64],
        out_gate: &[f64],
        gate_out: &[f64],
        hist: &mut [f64],
        state: &mut [f64],
        snaps: FbsReferenceSnaps,
    ) -> Vec<f64> {
        let d = drive.len();
        let lanes = self.decay.len();
        let mut conv = vec![0.0f64; d];
        for i in 0..d {
            conv[i] = self.conv_bias[i]
                + self.taps[i] * drive[i]
                + self.taps[d + i] * hist[i]
                + self.taps[2 * d + i] * hist[d + i]
                + self.taps[3 * d + i] * hist[2 * d + i];
        }
        hist.copy_within(..2 * d, d);
        hist[..d].copy_from_slice(drive);

        let (decays, rotations) = gate_out.split_at(lanes);
        for u in 0..lanes {
            let rate = decays[u] + self.decay[u];
            note_tie(|m| {
                m.rate = m.rate.min(rate_margin(rate));
                // The transition snap matters only where it moves a state.
                if ref_quat(state, u).iter().any(|&v| v != 0.0) {
                    m.note_snap(ref_quat(rotations, u), snaps.transition, true);
                }
            });
            let s = rate.clamp(0.0, 1.0);
            let idx = ((s * 16.0).floor() as usize).min(15);
            let lambda = 2.0f64.powi(-((idx as i32) + 1));
            let keep = (1.0 - lambda * lambda).sqrt();
            let q_rot = ref_quat_mode(ref_quat(rotations, u), snaps.transition, TRANSITION_EPS);
            let trans = [
                lambda * q_rot[0],
                lambda * q_rot[1],
                lambda * q_rot[2],
                lambda * q_rot[3],
            ];
            let moved = ref_hamilton(trans, ref_quat(state, u));
            for c in 0..4 {
                state[4 * u + c] = moved[c] + keep * conv[4 * u + c];
            }
        }
        let mut gated = vec![0.0f64; d];
        for u in 0..lanes {
            note_tie(|m| m.note_snap(ref_quat(out_gate, u), snaps.gates, false));
            let prod = ref_hamilton(
                ref_quat_mode(ref_quat(out_gate, u), snaps.gates, GATE_EPS),
                ref_quat(state, u),
            );
            gated[4 * u..4 * u + 4].copy_from_slice(&prod);
        }
        gated
    }
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
        let round_trip_grid = |v: f32, pos: bool| ref_grid(v, pos, quantized_weights);
        let round_trip_fixed = |v: f32, exp: i32| ref_fixed(v, exp, quantized_weights);
        let rms_norm = |x: &[f64]| ref_rms_norm(x, self.rms_eps);
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
                        let rec = RefRecurrence::new(
                            &r.conv_taps[..super::CONVOLUTION_WIDTH * d],
                            &r.conv_bias[..d],
                            &r.decay[..lanes],
                            quantized_weights,
                        )?;
                        let gated = rec.step(
                            drive,
                            out_gate,
                            &gate_out,
                            &mut rec_histories[l],
                            &mut rec_states[l],
                            FbsReferenceSnaps::INTEGER,
                        );
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
                let h_mlp = ref_hamilton_feed_forward(&g_vec, &u_vec, FbsQuatMode::Icosian);
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

/// One position of [`FbsTrunkSnapshot::forward_f32`]: the native vector
/// writer's choice and what it read.
#[derive(Clone, Debug, PartialEq)]
pub struct FbsWriterOutput {
    /// The best-scoring candidate, ties to the smaller token id.
    pub token: u32,
    /// The kept cells `a * 16 + b`, best first (ties to the smaller cell id).
    pub kept_cells: Vec<u32>,
    /// Every candidate token of the kept cells with its score, ascending token id.
    pub candidates: Vec<(u32, f64)>,
    /// The final query the writer scored (rounded to Q12 when quantized).
    pub query: Vec<f64>,
}

/// The FBS gather of `routes` on `v` (`in_blocks * 8` values): output block
/// `o`, channel `j` is `v[atoms[o]*8 + (code & 7)]`, negated when `code & 8`,
/// with `code` the `j`-th Cl(0,6) signed-axis code of `states[o]`.
pub(crate) fn fbs_reference_gather(routes: &FbsRoutes, v: &[f64]) -> Vec<f64> {
    let mut y = vec![0.0f64; routes.out_blocks * 8];
    for (o, (&atom, &state)) in routes.atoms.iter().zip(&routes.states).enumerate() {
        let base = usize::from(atom) * 8;
        let axes = crate::discrete_d11::cl06_axes(usize::from(state));
        for (j, &code) in axes.iter().enumerate() {
            let x = v[base + usize::from(code & 7)];
            y[o * 8 + j] = if code & 8 != 0 { -x } else { x };
        }
    }
    y
}

/// The native vector writer on the final query `q` over the writer `table`
/// (`vocab * d`): 16 roots on each port, the `kept` best cells, then the
/// nearest candidate token in L1.
fn fbs_native_writer(
    q: &[f64],
    table: &[f64],
    vocab: usize,
    kept: usize,
) -> Result<FbsWriterOutput, StackError> {
    let d = q.len();
    let neg_l1 = |a: &[f64], b: &[f64]| -> f64 { -a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum::<f64>() };
    let roots: Vec<[f64; 8]> = (0..16)
        .map(|r| crate::discrete_d11::orbit_root_coords(r).map(f64::from))
        .collect();
    let port_a: Vec<f64> = roots.iter().map(|r| neg_l1(&q[..8], r)).collect();
    let port_b: Vec<f64> = roots.iter().map(|r| neg_l1(&q[d - 8..], r)).collect();
    let mut cells: Vec<(u32, f64)> = (0..256usize)
        .map(|c| (c as u32, port_a[c / 16] + port_b[c % 16]))
        .collect();
    cells.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let kept_cells: Vec<u32> = cells.iter().take(kept).map(|c| c.0).collect();
    let mut candidates = Vec::new();
    for &cell in &kept_cells {
        for slot in 0..vocab.div_ceil(256) {
            let t = cell as usize + 256 * slot;
            if t < vocab {
                candidates.push((t as u32, neg_l1(q, &table[t * d..(t + 1) * d])));
            }
        }
    }
    candidates.sort_by_key(|c| c.0);
    let mut best: Option<(u32, f64)> = None;
    for &(t, s) in &candidates {
        if best.is_none_or(|(_, b)| s > b) {
            best = Some((t, s));
        }
    }
    let (token, _) = best.ok_or_else(|| {
        StackError::Numerics("the kept cells hold no token below the vocabulary".to_owned())
    })?;
    Ok(FbsWriterOutput {
        token,
        kept_cells,
        candidates,
        query: q.to_vec(),
    })
}

impl FbsTrunkSnapshot {
    /// Offline float reference of the served SpiralCore/FBS trunk forward,
    /// one [`FbsWriterOutput`] per position. A test oracle for the integer
    /// execution, not served code: it runs in `f64` and multiplies freely.
    ///
    /// Each layer gathers its projections through the FBS routes (no
    /// matrices), runs the geometric recurrence of
    /// [`FloatTrunkSnapshot::forward_f32`] (clamped 16-entry dyadic rate,
    /// icosian transport, icosian-snapped Hamilton output gate) and its
    /// icosian-snapped Hamilton feed-forward block; the final query feeds the
    /// native vector writer over the embedding table. When `quantized`, every
    /// table is read as the exporter writes it (grid-coded gains and taps,
    /// Q16 biases and decay offsets, the Q12 writer table, also used as the
    /// input embedding) and the query is rounded to Q12 before scoring.
    pub fn forward_f32(
        &self,
        tokens: &[u32],
        quantized: bool,
    ) -> Result<Vec<FbsWriterOutput>, StackError> {
        self.forward_f32_with(tokens, quantized, FbsReferenceSnaps::INTEGER)
    }

    /// [`Self::forward_f32`] with chosen snaps: [`FbsReferenceSnaps::INTEGER`]
    /// is the integer path's, [`FbsReferenceSnaps::FLOAT`] the plain float GPU
    /// trunk's and [`FbsReferenceSnaps::FLOAT_EXPORT`] the float sheets' D11
    /// export mode, so each float readout has a reference to match.
    pub fn forward_f32_with(
        &self,
        tokens: &[u32],
        quantized: bool,
        snaps: FbsReferenceSnaps,
    ) -> Result<Vec<FbsWriterOutput>, StackError> {
        // Refuse whatever the exporter refuses: the reference only runs
        // snapshots the served path can load.
        export_fbs_trunk_uorlut01(self)?;
        if tokens.len() > self.context {
            return Err(StackError::ContextFull {
                context: self.context,
            });
        }
        if let Some(&token) = tokens.iter().find(|&&t| t as usize >= self.vocab) {
            return Err(StackError::Token {
                token,
                vocab: self.vocab,
            });
        }

        let d = self.width;
        let q12 = |v: f64| (v * 4096.0).round().clamp(-32768.0, 32767.0) / 4096.0;
        let table: Vec<f64> = self
            .embed
            .iter()
            .map(|&v| if quantized { q12(f64::from(v)) } else { f64::from(v) })
            .collect();
        let gain = |g: &[f32]| -> Result<Vec<f64>, StackError> {
            g.iter().map(|&v| ref_grid(v, false, quantized)).collect()
        };
        let norm = |x: &[f64], gain: &[f64]| -> Vec<f64> {
            ref_rms_norm(x, self.rms_eps)
                .into_iter()
                .zip(gain)
                .map(|(v, g)| v * g)
                .collect()
        };
        let add = |x: &mut [f64], y: &[f64]| x.iter_mut().zip(y).for_each(|(xi, yi)| *xi += yi);

        struct Prepared {
            rec_norm: Vec<f64>,
            mlp_norm: Vec<f64>,
            gate_bias: Vec<f64>,
            rec: RefRecurrence,
        }
        let prepared: Vec<Prepared> = self
            .layers
            .iter()
            .map(|layer| -> Result<Prepared, StackError> {
                Ok(Prepared {
                    rec_norm: gain(&layer.rec_norm)?,
                    mlp_norm: gain(&layer.mlp_norm)?,
                    gate_bias: layer
                        .gate_bias
                        .iter()
                        .map(|&v| ref_fixed(v, super::kernels::RESIDUAL_EXP, quantized))
                        .collect(),
                    rec: RefRecurrence::new(&layer.conv_taps, &layer.conv_bias, &layer.decay, quantized)?,
                })
            })
            .collect::<Result<_, _>>()?;
        let final_norm = gain(&self.final_norm)?;

        let mut states = vec![vec![0.0f64; d]; self.layers.len()];
        let mut histories = vec![vec![0.0f64; 3 * d]; self.layers.len()];
        let mut out = Vec::with_capacity(tokens.len());
        for &tok in tokens {
            let t = tok as usize;
            let mut x = table[t * d..(t + 1) * d].to_vec();
            for (l, (layer, p)) in self.layers.iter().zip(&prepared).enumerate() {
                let u = norm(&x, &p.rec_norm);
                let branches = fbs_reference_gather(&layer.rec_in, &u);
                let mut gate_out = fbs_reference_gather(&layer.rec_gate, &u);
                add(&mut gate_out, &p.gate_bias);
                let (drive, out_gate) = branches.split_at(d);
                let gated = p.rec.step(
                    drive,
                    out_gate,
                    &gate_out,
                    &mut histories[l],
                    &mut states[l],
                    snaps,
                );
                add(&mut x, &fbs_reference_gather(&layer.rec_out, &gated));

                let u = norm(&x, &p.mlp_norm);
                let h = ref_hamilton_feed_forward(
                    &fbs_reference_gather(&layer.mlp_gate, &u),
                    &fbs_reference_gather(&layer.mlp_up, &u),
                    snaps.gates,
                );
                add(&mut x, &fbs_reference_gather(&layer.mlp_down, &h));
            }
            let mut q = norm(&x, &final_norm);
            if quantized {
                q.iter_mut().for_each(|v| *v = q12(*v));
            }
            out.push(fbs_native_writer(&q, &table, self.vocab, self.kept_cells)?);
        }
        Ok(out)
    }

    /// The [`FbsTieMargins`] of [`Self::forward_f32_with`] over `tokens`:
    /// every snap and rate met up to the last position, the carried state's
    /// included. A diagnostic, for tracing a disagreement to its near-tie.
    pub fn tie_margins(
        &self,
        tokens: &[u32],
        quantized: bool,
        snaps: FbsReferenceSnaps,
    ) -> Result<FbsTieMargins, StackError> {
        TIE_MARGINS.with(|cell| cell.set(Some(FbsTieMargins::NONE)));
        let run = self.forward_f32_with(tokens, quantized, snaps);
        let margins = TIE_MARGINS.with(|cell| cell.take());
        run?;
        margins.ok_or_else(|| StackError::Numerics("the tie margins were not recorded".to_owned()))
    }
}
