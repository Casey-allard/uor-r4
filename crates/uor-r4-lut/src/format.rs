//! Packed artifact: `MAGIC | u64 header length | JSON header | 64-byte-aligned sections`.
//!
//! The header holds integers only. Each 4-bit weight matrix is stored row-major
//! as offset-binary nibbles (`q + 8`, two per byte, low nibble first) and one
//! scale byte per group of [`crate::GROUP`] weights: low nibble `m`, high nibble
//! `de`, meaning the group scale `(16 + m) 2^(exp_base + de - 4)`.

use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{format_error, invalid, Result};

pub const MAGIC: &[u8; 8] = b"UORLUT01";
pub const SCHEMA: &str = "uor-r4.lut-llama/1";
const ALIGN: usize = 64;
const MAX_HEADER: u64 = 64 << 20;

/// Model dimensions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shape {
    pub vocab: usize,
    pub width: usize,
    pub layers: usize,
    pub heads: usize,
    pub kv_heads: usize,
    pub head_dim: usize,
    pub ffn: usize,
    pub max_positions: usize,
}

impl Shape {
    pub fn validate(&self) -> Result<()> {
        let dims = [
            self.vocab,
            self.width,
            self.layers,
            self.heads,
            self.kv_heads,
            self.head_dim,
            self.ffn,
            self.max_positions,
        ];
        if dims.contains(&0)
            || self.heads * self.head_dim != self.width
            || !self.heads.is_multiple_of(self.kv_heads)
            || !self.head_dim.is_multiple_of(2)
            || !self.width.is_multiple_of(crate::GROUP)
            || !self.ffn.is_multiple_of(crate::GROUP)
            || !(self.kv_heads * self.head_dim).is_multiple_of(crate::GROUP)
            || self.vocab > 1 << 20
            || self.width > 1 << 14
            || self.ffn > 1 << 16
            || self.layers > 256
            || self.max_positions > 1 << 16
        {
            return Err(format_error("inconsistent or unsupported model shape"));
        }
        Ok(())
    }
}

/// `mantissa * 2^exp`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fixed {
    pub mantissa: i64,
    pub exp: i32,
}

/// Integer conventions shared by the exporter and the engine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Numerics {
    /// RMSNorm epsilon.
    pub rms_eps: Fixed,
    /// `1 / sqrt(head_dim)` in Q30.
    pub score_scale_q30: i64,
    /// Table `exp`: entry `i` is `round(2^31 exp(-i 2^exp_step_log2))`.
    pub exp_step_log2: i32,
    /// Table `silu`: entry `i` is `round(2^16 silu(x))` for
    /// `x = (i - half) 2^silu_step_log2`, `half = 2^(silu_range_log2 - silu_step_log2)`.
    pub silu_step_log2: i32,
    pub silu_range_log2: i32,
    /// Tables `rope_cos`/`rope_sin`: `round(2^rope_q cos/sin)`, `[position][pair]`.
    pub rope_q: u32,
}

/// A learned Lorentz cache memory over the final normalized state (lab M4;
/// trained by `uor-r4-training`'s `cache_memory`). Its maps are the matrices
/// `cache.query` and `cache.key` (`dim x width`) and `cache.gate`
/// (`1 x width`); the table `arcosh` holds `arcosh(1 + u)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheSpec {
    /// Only `lorentz` is served.
    pub geometry: String,
    pub dim: usize,
    /// A query at position `t` reads entries at positions `<= t - gap`.
    pub gap: usize,
    /// Score scale `beta = (16 + beta_m) 2^(beta_e - 4)`; the score is `-beta d^2`.
    pub beta_m: u8,
    pub beta_e: i32,
    /// Gate bias at exponent -16.
    pub gate_bias: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub offset: u64,
    pub bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixSpec {
    pub name: String,
    pub rows: usize,
    pub cols: usize,
    pub exp_base: i32,
    pub nibbles: Span,
    pub scales: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableKind {
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSpec {
    pub name: String,
    pub kind: TableKind,
    pub len: usize,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub schema: String,
    pub shape: Shape,
    pub group: usize,
    pub numerics: Numerics,
    pub matrices: Vec<MatrixSpec>,
    pub tables: Vec<TableSpec>,
    /// Provenance recorded by the exporter; never read by serving arithmetic.
    pub source: serde_json::Value,
    /// Optional learned cache memory; absent from plain exports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache: Option<CacheSpec>,
}

/// Values of one sealed table.
pub enum TableValues<'a> {
    I16(&'a [i16]),
    I32(&'a [i32]),
    U32(&'a [u32]),
}

/// Offline builder used by the exporter.
pub struct ArtifactBuilder {
    header: Header,
    blob: Vec<u8>,
}

impl ArtifactBuilder {
    pub fn new(shape: Shape, numerics: Numerics, source: serde_json::Value) -> Result<Self> {
        shape.validate()?;
        Ok(Self {
            header: Header {
                schema: SCHEMA.to_owned(),
                shape,
                group: crate::GROUP,
                numerics,
                matrices: Vec::new(),
                tables: Vec::new(),
                source,
                cache: None,
            },
            blob: Vec::new(),
        })
    }

    fn push(&mut self, bytes: &[u8]) -> Span {
        let pad = (ALIGN - self.blob.len() % ALIGN) % ALIGN;
        self.blob.resize(self.blob.len() + pad, 0);
        let offset = self.blob.len() as u64;
        self.blob.extend_from_slice(bytes);
        Span {
            offset,
            bytes: bytes.len() as u64,
        }
    }

    pub fn add_matrix(
        &mut self,
        name: &str,
        rows: usize,
        cols: usize,
        exp_base: i32,
        nibbles: &[u8],
        scales: &[u8],
    ) -> Result<()> {
        if rows == 0
            || cols == 0
            || !cols.is_multiple_of(crate::GROUP)
            || nibbles.len() != rows * cols / 2
            || scales.len() != rows * cols / crate::GROUP
            || self.header.matrices.iter().any(|m| m.name == name)
        {
            return Err(invalid(format!("matrix {name}: inconsistent packing")));
        }
        let nibbles = self.push(nibbles);
        let scales = self.push(scales);
        self.header.matrices.push(MatrixSpec {
            name: name.to_owned(),
            rows,
            cols,
            exp_base,
            nibbles,
            scales,
        });
        Ok(())
    }

    pub fn add_table(&mut self, name: &str, values: TableValues<'_>) -> Result<()> {
        if self.header.tables.iter().any(|t| t.name == name) {
            return Err(invalid(format!("duplicate table {name}")));
        }
        let (kind, len, bytes) = match values {
            TableValues::I16(v) => (
                TableKind::I16,
                v.len(),
                v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>(),
            ),
            TableValues::I32(v) => (
                TableKind::I32,
                v.len(),
                v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            ),
            TableValues::U32(v) => (
                TableKind::U32,
                v.len(),
                v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            ),
        };
        let span = self.push(&bytes);
        self.header.tables.push(TableSpec {
            name: name.to_owned(),
            kind,
            len,
            span,
        });
        Ok(())
    }

    /// Declare the learned cache (its matrices and table are added separately).
    pub fn set_cache(&mut self, cache: CacheSpec) {
        self.header.cache = Some(cache);
    }

    /// The complete artifact bytes.
    pub fn finish(self) -> Result<Vec<u8>> {
        let json = serde_json::to_vec(&self.header)?;
        let mut out = Vec::with_capacity(16 + json.len() + ALIGN + self.blob.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(json.len() as u64).to_le_bytes());
        out.extend_from_slice(&json);
        let pad = (ALIGN - out.len() % ALIGN) % ALIGN;
        out.resize(out.len() + pad, 0);
        out.extend_from_slice(&self.blob);
        Ok(out)
    }
}

/// A parsed, validated artifact held in memory.
pub struct Artifact {
    pub header: Header,
    pub sha256: String,
    bytes: Vec<u8>,
    data_start: usize,
}

impl Artifact {
    pub fn load(path: &Path) -> Result<Self> {
        Self::parse(std::fs::read(path)?)
    }

    pub fn parse(bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() < 16 || &bytes[..8] != MAGIC {
            return Err(format_error("missing UORLUT01 magic"));
        }
        let mut length = [0u8; 8];
        length.copy_from_slice(&bytes[8..16]);
        let header_len = u64::from_le_bytes(length);
        if header_len > MAX_HEADER || 16 + header_len as usize > bytes.len() {
            return Err(format_error("header length out of bounds"));
        }
        let header: Header = serde_json::from_slice(&bytes[16..16 + header_len as usize])?;
        if header.schema != SCHEMA || header.group != crate::GROUP {
            return Err(format_error("unsupported schema or group size"));
        }
        header.shape.validate()?;
        let unaligned = 16 + header_len as usize;
        let data_start = unaligned + (ALIGN - unaligned % ALIGN) % ALIGN;
        if data_start > bytes.len() {
            return Err(format_error("data section missing"));
        }
        let data_len = (bytes.len() - data_start) as u64;
        let inside = |span: &Span| {
            span.offset
                .checked_add(span.bytes)
                .is_some_and(|end| end <= data_len)
        };
        for m in &header.matrices {
            if !inside(&m.nibbles)
                || !inside(&m.scales)
                || !m.cols.is_multiple_of(crate::GROUP)
                || m.nibbles.bytes != (m.rows * m.cols / 2) as u64
                || m.scales.bytes != (m.rows * m.cols / crate::GROUP) as u64
            {
                return Err(format_error(format!("matrix {} is malformed", m.name)));
            }
        }
        for t in &header.tables {
            if !inside(&t.span) || t.span.bytes != (t.len * t.kind.width()) as u64 {
                return Err(format_error(format!("table {} is malformed", t.name)));
            }
        }
        let sha256 = hex::encode(Sha256::digest(&bytes));
        Ok(Self {
            header,
            sha256,
            bytes,
            data_start,
        })
    }

    pub fn section(&self, span: Span) -> &[u8] {
        let start = self.data_start + span.offset as usize;
        &self.bytes[start..start + span.bytes as usize]
    }

    pub fn matrix(&self, name: &str) -> Result<&MatrixSpec> {
        self.header
            .matrices
            .iter()
            .find(|m| m.name == name)
            .ok_or_else(|| format_error(format!("missing matrix {name}")))
    }

    fn table(&self, name: &str, kind: TableKind) -> Result<&[u8]> {
        let spec = self
            .header
            .tables
            .iter()
            .find(|t| t.name == name)
            .ok_or_else(|| format_error(format!("missing table {name}")))?;
        if spec.kind != kind {
            return Err(format_error(format!(
                "table {name} has the wrong element type"
            )));
        }
        Ok(self.section(spec.span))
    }

    pub fn table_i16(&self, name: &str) -> Result<Vec<i16>> {
        Ok(self
            .table(name, TableKind::I16)?
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect())
    }

    pub fn table_i32(&self, name: &str) -> Result<Vec<i32>> {
        Ok(self
            .table(name, TableKind::I32)?
            .chunks_exact(4)
            .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect())
    }

    pub fn table_u32(&self, name: &str) -> Result<Vec<u32>> {
        Ok(self
            .table(name, TableKind::U32)?
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect())
    }
}
