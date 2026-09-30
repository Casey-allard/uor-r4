//! Packed artifact: `MAGIC | u64 header length | JSON header | 64-byte-aligned sections`.
//!
//! The header holds integers only. Each 4-bit weight matrix is stored row-major
//! as offset-binary nibbles (`q + 8`, two per byte, low nibble first) and one
//! scale byte per group of [`crate::GROUP`] weights: low nibble `m`, high nibble
//! `de`, meaning the group scale `(16 + m) 2^(exp_base + de - 4)`.
//!
//! Two headers share this container: [`Header`] (schema [`SCHEMA`], a converted
//! Llama checkpoint) and [`StackHeader`] (schema [`STACK_SCHEMA`], the lab's
//! geometric stack, served by [`crate::stack`]).

use std::path::Path;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{format_error, invalid, Result};

pub const MAGIC: &[u8; 8] = b"UORLUT01";
pub const SCHEMA: &str = "uor-r4.lut-llama/1";
pub const STACK_SCHEMA: &str = "uor-r4.lut-stack/1";
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

/// A header the container can carry: its own validation and its sections.
pub trait Sections: Clone + Serialize + DeserializeOwned {
    /// Schema, group size and shape checks, before any section is read.
    fn validate(&self) -> Result<()>;
    /// The checks of [`Sections::validate`] for offline reference
    /// construction: identical except that a header record only a serving
    /// engine interprets (the stack's `transport_snap`) is accepted. Serving
    /// code must use [`Sections::validate`].
    fn validate_for_reference(&self) -> Result<()> {
        self.validate()
    }
    fn matrices(&self) -> &[MatrixSpec];
    fn tables(&self) -> &[TableSpec];
    fn matrices_mut(&mut self) -> &mut Vec<MatrixSpec>;
    fn tables_mut(&mut self) -> &mut Vec<TableSpec>;
}

impl Sections for Header {
    fn validate(&self) -> Result<()> {
        if self.schema != SCHEMA || self.group != crate::GROUP {
            return Err(format_error("unsupported schema or group size"));
        }
        self.shape.validate()
    }

    fn matrices(&self) -> &[MatrixSpec] {
        &self.matrices
    }

    fn tables(&self) -> &[TableSpec] {
        &self.tables
    }

    fn matrices_mut(&mut self) -> &mut Vec<MatrixSpec> {
        &mut self.matrices
    }

    fn tables_mut(&mut self) -> &mut Vec<TableSpec> {
        &mut self.tables
    }
}

/// Dimensions of a geometric stack (`uor-r4-training`'s `geometric_stack`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackShape {
    pub vocab: usize,
    pub width: usize,
    pub heads: usize,
    /// MLP width, padded to a multiple of the group size with zero units.
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
    pub fn validate(&self) -> Result<()> {
        let dims = [self.vocab, self.width, self.heads, self.mlp, self.context];
        if dims.contains(&0)
            || self.pattern.is_empty()
            || self.pattern.bytes().any(|c| c != b'r' && c != b'a')
            || (self.read != "dot" && self.read != "lorentz")
            || !self.width.is_multiple_of(4)
            || !self.width.is_multiple_of(crate::GROUP)
            || !self.width.is_multiple_of(self.heads)
            || !self.mlp.is_multiple_of(crate::GROUP)
            || self.width / self.heads > 256
            || self.vocab > 1 << 20
            || self.width > 1 << 14
            || self.mlp > 1 << 16
            || self.pattern.len() > 256
            || self.context > 1 << 16
        {
            return Err(format_error("inconsistent or unsupported stack shape"));
        }
        Ok(())
    }

    pub fn layers(&self) -> usize {
        self.pattern.len()
    }

    /// Quaternion lanes of a recurrence (`width / 4`).
    pub fn lanes(&self) -> usize {
        self.width / 4
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

/// Integer conventions of a stack artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackNumerics {
    /// RMSNorm epsilon.
    pub rms_eps: Fixed,
    /// `1 / sqrt(head_dim)` in Q30 (the Dot score).
    pub score_scale_q30: i64,
    /// Table `exp`: entry `i` is `round(2^31 exp(-i 2^exp_step_log2))`.
    pub exp_step_log2: i32,
    /// Tables `silu` and `gelu` (tanh approximation): entry `i` is
    /// `round(2^16 f(x))` for `x = (i - half) 2^step_log2`,
    /// `half = 2^(range_log2 - step_log2)`.
    pub silu_step_log2: i32,
    pub silu_range_log2: i32,
    pub gelu_step_log2: i32,
    pub gelu_range_log2: i32,
}

/// Header of a stack artifact. Per layer `l`, a recurrence has the matrices
/// `l{l}.rec_in` (`2 width x width`: the drive, then the output gate),
/// `l{l}.rec_gate` (`gate_rows x width`) and `l{l}.rec_out`, and the tables
/// `l{l}.conv_taps` (grid codes, `[4][width]`, tap `s` on the input `s`
/// positions back), `l{l}.conv_bias` and `l{l}.gate_bias` (exponent -16) and
/// `l{l}.decay_rate` (grid codes of `8 softplus(-decay)` per lane). A read has
/// `l{l}.query`, `l{l}.key`, `l{l}.value`, `l{l}.null` (`heads x width`) and
/// `l{l}.out`, and the tables `l{l}.null_bias` and `l{l}.age` (`[heads]
/// [context]`, exponent -16), plus for Lorentz `l{l}.beta` (grid codes) and
/// `l{l}.offset` (exponent -24). Every layer has `l{l}.gate`, `l{l}.up` and
/// `l{l}.down`. Norm gains are folded into the maps that read the normalized
/// state; `head` carries the final norm's gain. Tables `exp`, `silu`, `gelu`,
/// and for Lorentz `arcosh`, are shared.
/// The transport snap a stack artifact records (`transport_snap` in the
/// header): the trained-in replacement of every unit transport quaternion by
/// the nearest of these roots before its scaling by lambda. Only the
/// multiplier-free engine (`uor_r4_integer::stack`) serves it; this crate's
/// D10 comparator computes the free transport, so its read path refuses an
/// artifact that carries the record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackTransportSnap {
    pub name: String,
    pub roots: usize,
    pub roots_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StackHeader {
    pub schema: String,
    pub shape: StackShape,
    pub group: usize,
    pub numerics: StackNumerics,
    pub matrices: Vec<MatrixSpec>,
    pub tables: Vec<TableSpec>,
    /// Provenance recorded by the exporter; never read by serving arithmetic.
    pub source: serde_json::Value,
    /// The trained-in transport snap; absent (and absent from the JSON) on a
    /// free-transport artifact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport_snap: Option<StackTransportSnap>,
}

impl Sections for StackHeader {
    fn validate(&self) -> Result<()> {
        if self.schema != STACK_SCHEMA || self.group != crate::GROUP {
            return Err(format_error("unsupported schema or group size"));
        }
        if let Some(snap) = &self.transport_snap {
            return Err(format_error(format!(
                "the artifact records transport_snap={} ({} roots), which only the \
                 multiplier-free stack engine serves; this engine computes the free transport",
                snap.name, snap.roots
            )));
        }
        self.shape.validate()
    }

    fn validate_for_reference(&self) -> Result<()> {
        if self.schema != STACK_SCHEMA || self.group != crate::GROUP {
            return Err(format_error("unsupported schema or group size"));
        }
        self.shape.validate()
    }

    fn matrices(&self) -> &[MatrixSpec] {
        &self.matrices
    }

    fn tables(&self) -> &[TableSpec] {
        &self.tables
    }

    fn matrices_mut(&mut self) -> &mut Vec<MatrixSpec> {
        &mut self.matrices
    }

    fn tables_mut(&mut self) -> &mut Vec<TableSpec> {
        &mut self.tables
    }
}

/// Values of one sealed table.
pub enum TableValues<'a> {
    I16(&'a [i16]),
    I32(&'a [i32]),
    U32(&'a [u32]),
}

/// Offline builder used by the exporters.
pub struct Builder<H> {
    header: H,
    blob: Vec<u8>,
}

/// Builder of a converted Llama checkpoint's artifact.
pub type ArtifactBuilder = Builder<Header>;
/// Builder of a geometric stack's artifact.
pub type StackArtifactBuilder = Builder<StackHeader>;

impl Builder<Header> {
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

    /// Declare the learned cache (its matrices and table are added separately).
    pub fn set_cache(&mut self, cache: CacheSpec) {
        self.header.cache = Some(cache);
    }
}

impl Builder<StackHeader> {
    pub fn new(
        shape: StackShape,
        numerics: StackNumerics,
        source: serde_json::Value,
    ) -> Result<Self> {
        shape.validate()?;
        Ok(Self {
            header: StackHeader {
                schema: STACK_SCHEMA.to_owned(),
                shape,
                group: crate::GROUP,
                numerics,
                matrices: Vec::new(),
                tables: Vec::new(),
                source,
                transport_snap: None,
            },
            blob: Vec::new(),
        })
    }

    /// Record the trained-in transport snap the artifact is served with
    /// (`uor_r4_integer::stack`; this crate's engine refuses such artifacts).
    pub fn set_transport_snap(&mut self, name: String, roots: usize, roots_sha256: String) {
        self.header.transport_snap = Some(StackTransportSnap {
            name,
            roots,
            roots_sha256,
        });
    }
}

impl<H: Sections> Builder<H> {
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
            || self.header.matrices().iter().any(|m| m.name == name)
        {
            return Err(invalid(format!("matrix {name}: inconsistent packing")));
        }
        let nibbles = self.push(nibbles);
        let scales = self.push(scales);
        self.header.matrices_mut().push(MatrixSpec {
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
        if self.header.tables().iter().any(|t| t.name == name) {
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
        self.header.tables_mut().push(TableSpec {
            name: name.to_owned(),
            kind,
            len,
            span,
        });
        Ok(())
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
pub struct Container<H> {
    pub header: H,
    pub sha256: String,
    bytes: Vec<u8>,
    data_start: usize,
}

/// A converted Llama checkpoint's artifact.
pub type Artifact = Container<Header>;
/// A geometric stack's artifact.
pub type StackArtifact = Container<StackHeader>;

/// The schema string of an artifact's header, without validating the rest.
pub fn schema_of(bytes: &[u8]) -> Result<String> {
    #[derive(Deserialize)]
    struct Schema {
        schema: String,
    }
    Ok(serde_json::from_slice::<Schema>(header_bytes(bytes)?)?.schema)
}

fn header_bytes(bytes: &[u8]) -> Result<&[u8]> {
    if bytes.len() < 16 || &bytes[..8] != MAGIC {
        return Err(format_error("missing UORLUT01 magic"));
    }
    let mut length = [0u8; 8];
    length.copy_from_slice(&bytes[8..16]);
    let header_len = u64::from_le_bytes(length);
    if header_len > MAX_HEADER || 16 + header_len as usize > bytes.len() {
        return Err(format_error("header length out of bounds"));
    }
    Ok(&bytes[16..16 + header_len as usize])
}

impl<H: Sections> Container<H> {
    pub fn load(path: &Path) -> Result<Self> {
        Self::parse(std::fs::read(path)?)
    }

    pub fn parse(bytes: Vec<u8>) -> Result<Self> {
        Self::parse_impl(bytes, true)
    }

    fn parse_impl(bytes: Vec<u8>, serving: bool) -> Result<Self> {
        let header_len = header_bytes(&bytes)?.len();
        let header: H = serde_json::from_slice(&bytes[16..16 + header_len])?;
        if serving {
            header.validate()?;
        } else {
            header.validate_for_reference()?;
        }
        let unaligned = 16 + header_len;
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
        for m in header.matrices() {
            if !inside(&m.nibbles)
                || !inside(&m.scales)
                || !m.cols.is_multiple_of(crate::GROUP)
                || m.nibbles.bytes != (m.rows * m.cols / 2) as u64
                || m.scales.bytes != (m.rows * m.cols / crate::GROUP) as u64
            {
                return Err(format_error(format!("matrix {} is malformed", m.name)));
            }
        }
        for t in header.tables() {
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
            .matrices()
            .iter()
            .find(|m| m.name == name)
            .ok_or_else(|| format_error(format!("missing matrix {name}")))
    }

    fn table(&self, name: &str, kind: TableKind) -> Result<&[u8]> {
        let spec = self
            .header
            .tables()
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

impl Container<StackHeader> {
    /// Parse a stack artifact for OFFLINE reference construction only: this
    /// behaves identically to [`Container::parse`] except that a
    /// `transport_snap` record is accepted rather than refused, so the
    /// exporter's dequantized float reference can be built from a snapped
    /// artifact. Serving code must use [`Container::parse`] (or
    /// `uor_r4_lut::stack::StackModel::from_artifact`), which refuses a
    /// snapped artifact.
    pub fn parse_for_reference(bytes: Vec<u8>) -> Result<Self> {
        Self::parse_impl(bytes, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack_shape() -> StackShape {
        StackShape {
            vocab: 64,
            width: 64,
            heads: 2,
            mlp: 64,
            pattern: "ra".to_owned(),
            read: "lorentz".to_owned(),
            rotation: true,
            context: 8,
        }
    }

    #[test]
    fn schemas_keep_the_two_headers_apart() {
        let numerics = StackNumerics {
            rms_eps: Fixed {
                mantissa: 1,
                exp: -48,
            },
            score_scale_q30: 1 << 28,
            exp_step_log2: -8,
            silu_step_log2: -8,
            silu_range_log2: 4,
            gelu_step_log2: -8,
            gelu_range_log2: 4,
        };
        let mut builder =
            StackArtifactBuilder::new(stack_shape(), numerics, serde_json::json!({})).unwrap();
        builder
            .add_table("exp", TableValues::U32(&[1 << 31, 1 << 30]))
            .unwrap();
        assert!(builder.add_table("exp", TableValues::U32(&[1])).is_err());
        let bytes = builder.finish().unwrap();
        assert_eq!(schema_of(&bytes).unwrap(), STACK_SCHEMA);
        let artifact = StackArtifact::parse(bytes.clone()).unwrap();
        assert_eq!(artifact.table_u32("exp").unwrap(), vec![1 << 31, 1 << 30]);
        assert!(artifact.table_i32("exp").is_err());
        assert!(
            Artifact::parse(bytes).is_err(),
            "a stack artifact is not a Llama artifact"
        );
    }

    #[test]
    fn a_snap_free_stack_header_serializes_without_the_record_and_a_snapped_one_is_refused_on_read()
    {
        let numerics = StackNumerics {
            rms_eps: Fixed {
                mantissa: 1,
                exp: -48,
            },
            score_scale_q30: 1 << 28,
            exp_step_log2: -8,
            silu_step_log2: -8,
            silu_range_log2: 4,
            gelu_step_log2: -8,
            gelu_range_log2: 4,
        };
        let plain =
            StackArtifactBuilder::new(stack_shape(), numerics.clone(), serde_json::json!({}))
                .unwrap()
                .finish()
                .unwrap();
        // No `transport_snap` key: a snap-free artifact is byte-identical to
        // one written before the field existed.
        let len = u64::from_le_bytes(plain[8..16].try_into().unwrap()) as usize;
        let header: serde_json::Value = serde_json::from_slice(&plain[16..16 + len]).unwrap();
        assert!(header.get("transport_snap").is_none());
        assert!(StackArtifact::parse(plain).is_ok());

        let mut builder =
            StackArtifactBuilder::new(stack_shape(), numerics, serde_json::json!({})).unwrap();
        builder.set_transport_snap("icosian".to_owned(), 120, "ab".repeat(32));
        // The writer path still finishes; only this crate's read path refuses.
        let snapped = builder.finish().unwrap();
        let len = u64::from_le_bytes(snapped[8..16].try_into().unwrap()) as usize;
        let header: serde_json::Value = serde_json::from_slice(&snapped[16..16 + len]).unwrap();
        assert_eq!(header["transport_snap"]["name"], "icosian");
        let refusal = match StackArtifact::parse(snapped.clone()) {
            Err(error) => error,
            Ok(_) => panic!("the D10 read path refuses the snap"),
        };
        assert!(
            refusal.to_string().contains("transport_snap=icosian"),
            "{refusal}"
        );
        // The offline reference parse accepts the same bytes and keeps the
        // record; the serving parse above is unchanged.
        let reference = StackArtifact::parse_for_reference(snapped).expect("reference parse");
        let record = reference.header.transport_snap.expect("the record is kept");
        assert_eq!(record.name, "icosian");
        assert_eq!(record.roots, 120);
    }

    #[test]
    fn stack_shapes_are_validated() {
        assert!(stack_shape().validate().is_ok());
        for broken in [
            StackShape {
                pattern: "rx".to_owned(),
                ..stack_shape()
            },
            StackShape {
                read: "cosine".to_owned(),
                ..stack_shape()
            },
            StackShape {
                mlp: 40,
                ..stack_shape()
            },
            StackShape {
                heads: 3,
                ..stack_shape()
            },
            StackShape {
                pattern: String::new(),
                ..stack_shape()
            },
        ] {
            assert!(broken.validate().is_err(), "{broken:?}");
        }
    }
}
