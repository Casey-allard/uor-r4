//! The four integer sections of a `UORLUT01` lut-discrete artifact, laid out
//! for the scorer. The bytes are validated first by the clean-room reader,
//! `IntegerDiscreteModel::from_bytes`; this reader then takes the sections at
//! the spans the header records (`discrete_d11.rs:614-812`) and checks that the
//! shape it reads agrees with the validated model's.
use serde_json::Value;
use uor_r4_integer::discrete_d11::{DiscreteReadMode, IntegerDiscreteModel};

const ALIGN: usize = 64;

/// Kernel read-mode codes (`D11_READ_*` in `kernel_core.h`).
pub const READ_ROUTE: i32 = 0;
pub const READ_FLOCK: i32 = 1;
pub const READ_RELATION: i32 = 2;

/// An artifact's sections as the kernel reads them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTables {
    /// vocabulary, blocks, context, kept_cells, stages, read_mode, read_kept,
    /// read_sweeps: the kernel's `shape` argument.
    pub shape: [i32; 8],
    /// `discrete.table`: vocabulary x blocks x 8.
    pub table: Vec<i32>,
    /// `discrete.cells`: vocabulary.
    pub cells: Vec<u32>,
    /// `discrete.routes`: blocks x stages.
    pub routes: Vec<u32>,
    /// `discrete.gates`: blocks x stages, then stages x 5 for a relation read.
    pub gates: Vec<i32>,
    /// The row plans for prefix lengths `1..=context` ([`crate::kernel::row_plans`]).
    pub plans: Vec<i32>,
    /// sha256 of the artifact bytes, as the integer reader reports it.
    pub artifact_sha256: String,
}

fn words(data: &[u8], header: &Value, name: &str, len: usize) -> Result<Vec<[u8; 4]>, String> {
    let entry = header["tables"]
        .as_array()
        .and_then(|tables| tables.iter().find(|t| t["name"] == name))
        .ok_or_else(|| format!("artifact lacks table {name}"))?;
    let offset = entry["span"]["offset"]
        .as_u64()
        .ok_or_else(|| format!("table {name} lacks an offset"))? as usize;
    let bytes = len
        .checked_mul(4)
        .ok_or_else(|| format!("table {name} size overflow"))?;
    if entry["len"].as_u64() != Some(len as u64)
        || entry["span"]["bytes"].as_u64() != Some(bytes as u64)
    {
        return Err(format!("table {name} length disagrees with the shape"));
    }
    let span = offset
        .checked_add(bytes)
        .and_then(|end| data.get(offset..end))
        .ok_or_else(|| format!("table {name} span out of bounds"))?;
    Ok(span
        .chunks_exact(4)
        .map(|b| [b[0], b[1], b[2], b[3]])
        .collect())
}

impl ArtifactTables {
    /// Validates `bytes` with the integer reader, then reads the sections.
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, IntegerDiscreteModel), String> {
        let model = IntegerDiscreteModel::from_bytes(bytes)
            .map_err(|e| format!("the integer reader refused the artifact: {e}"))?;
        let header_len =
            u64::from_le_bytes(bytes[8..16].try_into().map_err(|_| "short header")?) as usize;
        let header: Value = serde_json::from_slice(&bytes[16..16 + header_len])
            .map_err(|e| format!("artifact header: {e}"))?;
        let unaligned = 16 + header_len;
        let data = &bytes[unaligned + (ALIGN - unaligned % ALIGN) % ALIGN..];

        let shape = model.shape();
        let (read_mode, read_kept, read_sweeps) = match model.read_mode() {
            DiscreteReadMode::Route => (READ_ROUTE, 0, 0),
            DiscreteReadMode::Flock { kept } => (READ_FLOCK, kept, 0),
            DiscreteReadMode::Relation { kept, sweeps } => (READ_RELATION, kept, sweeps),
        };
        let as_i32 = |v: u32| i32::try_from(v).map_err(|_| "shape value exceeds i32".to_string());
        let shape_words = [
            as_i32(shape.vocabulary)?,
            as_i32(shape.blocks)?,
            as_i32(shape.context)?,
            as_i32(shape.kept_cells)?,
            as_i32(shape.stages)?,
            read_mode,
            as_i32(read_kept)?,
            as_i32(read_sweeps)?,
        ];
        if header["shape"]["vocabulary"].as_u64() != Some(u64::from(shape.vocabulary))
            || header["shape"]["blocks"].as_u64() != Some(u64::from(shape.blocks))
            || header["shape"]["stages"].as_u64() != Some(u64::from(shape.stages))
        {
            return Err("artifact header shape disagrees with the integer reader".into());
        }
        let vocab = shape.vocabulary as usize;
        let blocks = shape.blocks as usize;
        let stages = shape.stages as usize;
        let gate_len = blocks * stages
            + if read_mode == READ_RELATION {
                5 * stages
            } else {
                0
            };
        let table = words(data, &header, "discrete.table", vocab * blocks * 8)?
            .into_iter()
            .map(i32::from_le_bytes)
            .collect();
        let cells: Vec<u32> = words(data, &header, "discrete.cells", vocab)?
            .into_iter()
            .map(u32::from_le_bytes)
            .collect();
        if cells != model.cells() {
            return Err("discrete.cells disagrees with the integer reader".into());
        }
        let routes = words(data, &header, "discrete.routes", blocks * stages)?
            .into_iter()
            .map(u32::from_le_bytes)
            .collect();
        let gates = words(data, &header, "discrete.gates", gate_len)?
            .into_iter()
            .map(i32::from_le_bytes)
            .collect();
        let tables = Self {
            shape: shape_words,
            table,
            cells,
            routes,
            gates,
            plans: crate::kernel::row_plans(shape.context)?,
            artifact_sha256: model.artifact_sha256().to_owned(),
        };
        Ok((tables, model))
    }
}
