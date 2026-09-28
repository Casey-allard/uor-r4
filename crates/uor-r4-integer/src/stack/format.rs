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

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::StackError;

/// Container magic.
pub const MAGIC: &[u8; 8] = b"UORLUT01";
/// Header schema of a geometric-stack artifact.
pub const STACK_SCHEMA: &str = "uor-r4.lut-stack/1";
/// Weights per scale group along a matrix row.
pub const GROUP: usize = 32;
/// Alignment of the data start and of every section.
const ALIGN: usize = 64;
/// The largest header accepted (64 MiB).
const MAX_HEADER: u64 = 64 << 20;

/// Dimensions of a geometric stack.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct StackShape {
    pub vocab: usize,
    pub width: usize,
    pub heads: usize,
    /// MLP width, padded by the exporter to a multiple of the group size.
    pub mlp: usize,
    /// One letter per layer: `r` quaternion transport recurrence, `a` read.
    pub pattern: String,
    /// Read score: `dot` or `lorentz`.
    pub read: String,
    /// Learned rotations in the recurrence; `false` is identity transport.
    pub rotation: bool,
    /// Positions a session serves: the length of each read's age table.
    pub context: usize,
}

impl StackShape {
    /// The shape rules of the container: the exporter's invariants and the
    /// serving limits.
    pub fn validate(&self) -> Result<(), StackError> {
        let dims = [self.vocab, self.width, self.heads, self.mlp, self.context];
        let reason = if dims.contains(&0) {
            Some("a dimension is zero")
        } else if self.pattern.is_empty() || self.pattern.len() > 256 {
            Some("the layer pattern is empty or longer than 256 layers")
        } else if self.pattern.bytes().any(|c| c != b'r' && c != b'a') {
            Some("the layer pattern holds a letter other than r and a")
        } else if self.read != "dot" && self.read != "lorentz" {
            Some("the read score is neither dot nor lorentz")
        } else if !self.width.is_multiple_of(4)
            || !self.width.is_multiple_of(GROUP)
            || !self.width.is_multiple_of(self.heads)
        {
            Some("the width is not a multiple of 4, of the group and of the heads")
        } else if !self.mlp.is_multiple_of(GROUP) {
            Some("the MLP width is not a multiple of the group")
        } else if self.width / self.heads > 256 {
            Some("the head width exceeds 256")
        } else if self.vocab > 1 << 20
            || self.width > 1 << 14
            || self.mlp > 1 << 16
            || self.context > 1 << 16
        {
            Some("a dimension exceeds its serving limit")
        } else {
            None
        };
        match reason {
            Some(reason) => Err(StackError::Shape(reason)),
            None => Ok(()),
        }
    }

    pub fn layers(&self) -> usize {
        self.pattern.len()
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
}

/// `mantissa * 2^exp`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub struct Fixed {
    pub mantissa: i64,
    pub exp: i32,
}

/// Integer conventions of a stack artifact.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
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
    data: &'a [u8],
    matrices: Vec<MatrixEntry>,
    tables: Vec<TableEntry>,
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
        if header.schema != STACK_SCHEMA {
            return Err(StackError::Schema(header.schema));
        }
        if header.group != GROUP {
            return Err(StackError::Group(header.group));
        }
        header.shape.validate()?;
        let data_start = header_end
            .checked_add((ALIGN - header_end % ALIGN) % ALIGN)
            .filter(|start| *start <= bytes.len())
            .ok_or(StackError::MissingData)?;
        let data = &bytes[data_start..];
        for (i, m) in header.matrices.iter().enumerate() {
            if header.matrices[..i]
                .iter()
                .any(|other| other.name == m.name)
            {
                return Err(StackError::DuplicateSection(m.name.clone()));
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
                return Err(StackError::MatrixSection(m.name.clone()));
            }
        }
        for (i, t) in header.tables.iter().enumerate() {
            if header.tables[..i].iter().any(|other| other.name == t.name) {
                return Err(StackError::DuplicateSection(t.name.clone()));
            }
            let sized = t
                .len
                .checked_mul(t.kind.width())
                .is_some_and(|b| b as u64 == t.span.bytes);
            if !sized || range(t.span, data.len()).is_none() {
                return Err(StackError::TableSection(t.name.clone()));
            }
        }
        Ok(Self {
            shape: header.shape,
            numerics: header.numerics,
            sha256: hex::encode(Sha256::digest(bytes)),
            data,
            matrices: header.matrices,
            tables: header.tables,
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
            .iter()
            .find(|m| m.name == name)
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
            .iter()
            .find(|t| t.name == name)
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
