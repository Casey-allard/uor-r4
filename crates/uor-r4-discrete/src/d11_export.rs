//! Discrete SpiralCore/FBS checkpoint exporter and loader for the `UORLUT01`
//! (`uor-r4.lut-discrete/1`) integer table container.
//!
//! Host metadata and container packing only (D24): validates a discrete
//! [`Snapshot`], verifies that every 8D codebook block belongs to the 16-root
//! `Cl(0,6)` anchor orbit, derives each token's writer cell index
//! `root(block 0) * 16 + root(block B - 1)` in `0..256`, and packs the four
//! integer tables (`discrete.table`, `discrete.cells`, `discrete.routes`,
//! `discrete.gates`) into a 64-byte-aligned `UORLUT01` artifact with empty
//! `matrices`. No model forward pass or proposal executes here.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    metadata::BinderPlan, snapshot::Snapshot, Error, ObjectivePolicy, ReadInitialization, ReadSpec,
    Result, Spec, SCHEMA,
};

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

/// Container magic shared with `uor-r4-lut` and `uor-r4-integer`.
pub const MAGIC: &[u8; 8] = b"UORLUT01";
/// Header schema of a discrete SpiralCore/FBS `UORLUT01` artifact.
pub const DISCRETE_LUT_SCHEMA: &str = "uor-r4.lut-discrete/1";
const ALIGN: usize = 64;
const MAX_HEADER: u64 = 64 << 20;

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn is_sha256_hex(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Byte span of a 64-byte-aligned table section relative to the data start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionSpan {
    pub offset: u64,
    pub bytes: u64,
}

/// Element type of a `UORLUT01` table section.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionKind {
    I16,
    I32,
    U32,
}

impl SectionKind {
    fn width(self) -> usize {
        match self {
            Self::I16 => 2,
            Self::I32 | Self::U32 => 4,
        }
    }
}

/// Descriptor of one table section in the `UORLUT01` header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionSpec {
    pub name: String,
    pub kind: SectionKind,
    pub len: usize,
    pub span: SectionSpan,
}

/// Shape and read/writer metadata recorded in the `uor-r4.lut-discrete/1` header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscreteLutShape {
    pub vocabulary: u32,
    pub blocks: u32,
    pub context: u32,
    pub kept_cells: u32,
    pub stages: u32,
    pub read_mode: String,
    pub read_kept: u32,
    pub read_sweeps: u32,
    pub objective_policy: String,
    pub read_initialization: String,
    pub seed: u32,
    pub epoch: u64,
}

/// Provenance recorded in the `source` field of a `uor-r4.lut-discrete/1` header.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscreteLutSource {
    pub snapshot_schema: String,
    pub snapshot_sha256: String,
    pub operator_kappa: String,
    pub composition_kappa: String,
    pub kernel_sha256: String,
    pub source_sha256: String,
    pub geometry_sha256: String,
}

/// Header of a `uor-r4.lut-discrete/1` container.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscreteLutHeader {
    pub schema: String,
    pub shape: DiscreteLutShape,
    pub matrices: Vec<serde_json::Value>,
    pub tables: Vec<SectionSpec>,
    pub source: DiscreteLutSource,
}

/// Return the orbit-root index `0..16` for an 8D coordinate block, or refuse if
/// the block is outside the 16-root `Cl(0,6)` anchor orbit.
pub fn orbit_root_id(block: &[i32]) -> Result<u32> {
    if block.len() != 8 {
        return Err(invalid("orbit root block requires 8 coordinates"));
    }
    geometry::ROOTS
        .iter()
        .position(|root| {
            root.iter()
                .zip(block.iter())
                .all(|(&expected, &actual)| i32::from(expected) == actual)
        })
        .map(|idx| idx as u32)
        .ok_or_else(|| invalid("codebook block is outside the 16-root SpiralCore orbit"))
}

/// Validate a discrete [`Snapshot`] and return each token's writer cell index
/// `root(block 0) * 16 + root(block B - 1)` in `0..256`.
pub fn validate_and_extract_cells(snapshot: &Snapshot) -> Result<Vec<u32>> {
    if snapshot.schema != SCHEMA {
        return Err(invalid(
            "snapshot schema differs from uor-r4.discrete-spiralcore-fbs/2",
        ));
    }
    snapshot.spec.validate()?;
    snapshot.read_spec.validate(snapshot.spec)?;
    snapshot.read_initialization.validate(snapshot.read_spec)?;
    if snapshot.operator_kappa != geometry::OPERATOR_KAPPA
        || snapshot.composition_kappa != geometry::COMPOSITION_KAPPA
    {
        return Err(invalid(
            "snapshot operator_kappa or composition_kappa differs from compiled geometry",
        ));
    }
    if !is_sha256_hex(&snapshot.kernel_sha256)
        || !is_sha256_hex(&snapshot.source_sha256)
        || !is_sha256_hex(&snapshot.geometry_sha256)
    {
        return Err(invalid("snapshot provenance hashes must be 64-digit hex"));
    }
    let vocab = snapshot.spec.vocabulary as usize;
    let blocks = snapshot.spec.blocks as usize;
    let stages = BinderPlan::new(snapshot.spec.context as usize)?.stages();
    let expected_table_len = vocab
        .checked_mul(blocks)
        .and_then(|n| n.checked_mul(8))
        .ok_or_else(|| invalid("table length overflow"))?;
    if snapshot.table.len() != expected_table_len {
        return Err(invalid(
            "snapshot table length differs from vocabulary * blocks * 8",
        ));
    }
    let expected_routes_len = blocks
        .checked_mul(stages)
        .ok_or_else(|| invalid("routes length overflow"))?;
    if snapshot.routes.len() != expected_routes_len || snapshot.routes.iter().any(|&r| r >= 64) {
        return Err(invalid(
            "snapshot routes length or action index (0..64) is invalid",
        ));
    }
    let expected_gates_len = snapshot
        .read_spec
        .gate_count(snapshot.spec.blocks, stages as u32)? as usize;
    if snapshot.gates.len() != expected_gates_len
        || snapshot.gates.iter().any(|&g| !(-1..=1).contains(&g))
    {
        return Err(invalid(
            "snapshot gates length or trit value (-1..=1) is invalid",
        ));
    }
    let mut cells = Vec::with_capacity(vocab);
    for token in 0..vocab {
        let mut first_root = 0u32;
        let mut last_root = 0u32;
        for block in 0..blocks {
            let start = (token * blocks + block) * 8;
            let root = orbit_root_id(&snapshot.table[start..start + 8])?;
            if block == 0 {
                first_root = root;
            }
            if block + 1 == blocks {
                last_root = root;
            }
        }
        cells.push(first_root * 16 + last_root);
    }
    Ok(cells)
}

fn push_section(blob: &mut Vec<u8>, bytes: &[u8]) -> SectionSpan {
    let pad = (ALIGN - blob.len() % ALIGN) % ALIGN;
    blob.resize(blob.len() + pad, 0);
    let offset = blob.len() as u64;
    blob.extend_from_slice(bytes);
    SectionSpan {
        offset,
        bytes: bytes.len() as u64,
    }
}

/// Export a validated discrete [`Snapshot`] into a `UORLUT01` (`uor-r4.lut-discrete/1`)
/// integer container.
pub fn export_d11(snapshot: &Snapshot) -> Result<Vec<u8>> {
    // The integer serving engine has only the scale-matched snap; a control
    // arm is a research model and is never shipped.
    if !snapshot.normalization_mode.is_scale_matched_snap() {
        return Err(invalid(
            "D11 export refuses a non-default normalization_mode: the integer engine has no unit snap",
        ));
    }
    if snapshot.position.is_some() {
        return Err(invalid(
            "D11 export refuses position transport: the integer engine has no position read",
        ));
    }
    let cells = validate_and_extract_cells(snapshot)?;
    let stages = BinderPlan::new(snapshot.spec.context as usize)?.stages() as u32;
    let (read_mode, read_kept, read_sweeps) = match snapshot.read_spec {
        ReadSpec::Route => ("route".to_owned(), 0, 0),
        ReadSpec::Flock { kept } => ("flock".to_owned(), kept, 0),
        ReadSpec::Relation { kept, sweeps } => ("relation".to_owned(), kept, sweeps),
    };
    let objective_policy = match snapshot.objective_policy {
        ObjectivePolicy::NearestAddressMarginV1 => "nearest_address_margin_v1".to_owned(),
        ObjectivePolicy::OccupiedCellRankHingeV1 => "occupied_cell_rank_hinge_v1".to_owned(),
    };
    let read_initialization = match snapshot.read_initialization {
        ReadInitialization::ZeroGatesV1 => "zero_gates_v1".to_owned(),
        ReadInitialization::ActiveLeafV1 => "active_leaf_v1".to_owned(),
        ReadInitialization::RouteBypassV1 | ReadInitialization::RoutePermutedActionsV1 => {
            return Err(invalid(
                "D11 export refuses a Route read-off or permuted control arm",
            ));
        }
    };
    let snapshot_bytes = serde_json::to_vec(snapshot).map_err(|e| invalid(e.to_string()))?;
    let source = DiscreteLutSource {
        snapshot_schema: snapshot.schema.clone(),
        snapshot_sha256: sha256_hex(&snapshot_bytes),
        operator_kappa: snapshot.operator_kappa.clone(),
        composition_kappa: snapshot.composition_kappa.clone(),
        kernel_sha256: snapshot.kernel_sha256.clone(),
        source_sha256: snapshot.source_sha256.clone(),
        geometry_sha256: snapshot.geometry_sha256.clone(),
    };
    let shape = DiscreteLutShape {
        vocabulary: snapshot.spec.vocabulary,
        blocks: snapshot.spec.blocks,
        context: snapshot.spec.context,
        kept_cells: snapshot.spec.kept_cells,
        stages,
        read_mode,
        read_kept,
        read_sweeps,
        objective_policy,
        read_initialization,
        seed: snapshot.seed,
        epoch: snapshot.epoch,
    };

    let mut blob = Vec::new();
    let mut tables = Vec::with_capacity(4);

    let table_bytes: Vec<u8> = snapshot
        .table
        .iter()
        .flat_map(|x| x.to_le_bytes())
        .collect();
    tables.push(SectionSpec {
        name: "discrete.table".to_owned(),
        kind: SectionKind::I32,
        len: snapshot.table.len(),
        span: push_section(&mut blob, &table_bytes),
    });

    let cells_bytes: Vec<u8> = cells.iter().flat_map(|x| x.to_le_bytes()).collect();
    tables.push(SectionSpec {
        name: "discrete.cells".to_owned(),
        kind: SectionKind::U32,
        len: cells.len(),
        span: push_section(&mut blob, &cells_bytes),
    });

    let routes_bytes: Vec<u8> = snapshot
        .routes
        .iter()
        .flat_map(|x| x.to_le_bytes())
        .collect();
    tables.push(SectionSpec {
        name: "discrete.routes".to_owned(),
        kind: SectionKind::U32,
        len: snapshot.routes.len(),
        span: push_section(&mut blob, &routes_bytes),
    });

    let gates_bytes: Vec<u8> = snapshot
        .gates
        .iter()
        .flat_map(|x| x.to_le_bytes())
        .collect();
    tables.push(SectionSpec {
        name: "discrete.gates".to_owned(),
        kind: SectionKind::I32,
        len: snapshot.gates.len(),
        span: push_section(&mut blob, &gates_bytes),
    });

    let header = DiscreteLutHeader {
        schema: DISCRETE_LUT_SCHEMA.to_owned(),
        shape,
        matrices: Vec::new(),
        tables,
        source,
    };
    let json = serde_json::to_vec(&header).map_err(|e| invalid(e.to_string()))?;
    let mut out = Vec::with_capacity(16 + json.len() + ALIGN + blob.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(json.len() as u64).to_le_bytes());
    out.extend_from_slice(&json);
    let pad = (ALIGN - out.len() % ALIGN) % ALIGN;
    out.resize(out.len() + pad, 0);
    out.extend_from_slice(&blob);
    Ok(out)
}

/// Restore and validate a discrete [`Snapshot`] from a `UORLUT01` (`uor-r4.lut-discrete/1`)
/// integer container.
pub fn import_d11(bytes: &[u8]) -> Result<Snapshot> {
    if bytes.len() < 16 || &bytes[..8] != MAGIC {
        return Err(invalid("missing UORLUT01 magic"));
    }
    let mut len_bytes = [0u8; 8];
    len_bytes.copy_from_slice(&bytes[8..16]);
    let header_len = u64::from_le_bytes(len_bytes);
    if header_len > MAX_HEADER || 16 + (header_len as usize) > bytes.len() {
        return Err(invalid("UORLUT01 header length out of bounds"));
    }
    let header: DiscreteLutHeader = serde_json::from_slice(&bytes[16..16 + (header_len as usize)])
        .map_err(|e| invalid(format!("UORLUT01 header JSON: {e}")))?;
    if header.schema != DISCRETE_LUT_SCHEMA || !header.matrices.is_empty() {
        return Err(invalid(
            "discrete UORLUT01 requires schema uor-r4.lut-discrete/1 and empty matrices",
        ));
    }
    let unaligned = 16 + (header_len as usize);
    let data_start = unaligned + (ALIGN - unaligned % ALIGN) % ALIGN;
    if data_start > bytes.len() || bytes[unaligned..data_start].iter().any(|&b| b != 0) {
        return Err(invalid("UORLUT01 header padding is missing or non-zero"));
    }
    let data = &bytes[data_start..];
    let spec = Spec {
        vocabulary: header.shape.vocabulary,
        blocks: header.shape.blocks,
        context: header.shape.context,
        kept_cells: header.shape.kept_cells,
    };
    spec.validate()?;
    let stages = BinderPlan::new(spec.context as usize)?.stages() as u32;
    if header.shape.stages != stages {
        return Err(invalid(
            "UORLUT01 discrete stages differ from context BinderPlan",
        ));
    }
    let read_spec = match (
        header.shape.read_mode.as_str(),
        header.shape.read_kept,
        header.shape.read_sweeps,
    ) {
        ("route", 0, 0) => ReadSpec::Route,
        ("flock", kept, 0) => ReadSpec::Flock { kept },
        ("relation", kept, sweeps) => ReadSpec::Relation { kept, sweeps },
        _ => {
            return Err(invalid(
                "unsupported discrete read_mode/kept/sweeps in UORLUT01",
            ));
        }
    };
    read_spec.validate(spec)?;
    let objective_policy = match header.shape.objective_policy.as_str() {
        "nearest_address_margin_v1" => ObjectivePolicy::NearestAddressMarginV1,
        "occupied_cell_rank_hinge_v1" => ObjectivePolicy::OccupiedCellRankHingeV1,
        _ => return Err(invalid("unsupported objective_policy in UORLUT01")),
    };
    let read_initialization = match header.shape.read_initialization.as_str() {
        "zero_gates_v1" => ReadInitialization::ZeroGatesV1,
        "active_leaf_v1" => ReadInitialization::ActiveLeafV1,
        _ => return Err(invalid("unsupported read_initialization in UORLUT01")),
    };
    read_initialization.validate(read_spec)?;

    let read_section = |name: &str,
                        expected_kind: SectionKind,
                        expected_len: usize|
     -> Result<&[u8]> {
        let entry = header
            .tables
            .iter()
            .find(|t| t.name == name)
            .ok_or_else(|| invalid(format!("missing table {name}")))?;
        if entry.kind != expected_kind || entry.len != expected_len {
            return Err(invalid(format!("table {name} kind or length mismatch")));
        }
        let expected_bytes = expected_len
            .checked_mul(expected_kind.width())
            .ok_or_else(|| invalid("table byte size overflow"))?;
        if (entry.span.offset as usize) % ALIGN != 0 || entry.span.bytes != expected_bytes as u64 {
            return Err(invalid(format!(
                "table {name} alignment or byte span mismatch"
            )));
        }
        let start =
            usize::try_from(entry.span.offset).map_err(|_| invalid("table offset overflow"))?;
        let end = start
            .checked_add(expected_bytes)
            .ok_or_else(|| invalid("table end overflow"))?;
        data.get(start..end)
            .ok_or_else(|| invalid(format!("table {name} span out of bounds")))
    };

    if header.tables.len() != 4 {
        return Err(invalid("discrete UORLUT01 requires exactly four tables"));
    }
    let vocab = spec.vocabulary as usize;
    let blocks = spec.blocks as usize;
    let gate_len = read_spec.gate_count(spec.blocks, stages)? as usize;

    let table: Vec<i32> = read_section("discrete.table", SectionKind::I32, vocab * blocks * 8)?
        .chunks_exact(4)
        .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    let cells: Vec<u32> = read_section("discrete.cells", SectionKind::U32, vocab)?
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    let routes: Vec<u32> = read_section(
        "discrete.routes",
        SectionKind::U32,
        blocks * (stages as usize),
    )?
    .chunks_exact(4)
    .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    .collect();
    let gates: Vec<i32> = read_section("discrete.gates", SectionKind::I32, gate_len)?
        .chunks_exact(4)
        .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();

    let snapshot = Snapshot {
        schema: header.source.snapshot_schema,
        spec,
        read_spec,
        objective_policy,
        read_initialization,
        normalization_mode: crate::normalization_mechanism::NormalizationMode::ScaleMatchedSnapV1,
        position: None,
        seed: header.shape.seed,
        epoch: header.shape.epoch,
        table,
        routes,
        gates,
        operator_kappa: header.source.operator_kappa,
        composition_kappa: header.source.composition_kappa,
        kernel_sha256: header.source.kernel_sha256,
        source_sha256: header.source.source_sha256,
        geometry_sha256: header.source.geometry_sha256,
    };
    let expected_cells = validate_and_extract_cells(&snapshot)?;
    if cells != expected_cells {
        return Err(invalid(
            "discrete.cells table disagrees with endpoint orbit roots of discrete.table",
        ));
    }
    let snapshot_bytes = serde_json::to_vec(&snapshot).map_err(|e| invalid(e.to_string()))?;
    if sha256_hex(&snapshot_bytes) != header.source.snapshot_sha256 {
        return Err(invalid(
            "reconstructed snapshot SHA-256 differs from source.snapshot_sha256",
        ));
    }
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: [i32; 8] = [2, 2, 0, 0, 0, 0, 0, 0];
    const NEG_A: [i32; 8] = [-2, -2, 0, 0, 0, 0, 0, 0];
    const C_ROOT: [i32; 8] = [-2, 2, 0, 0, 0, 0, 0, 0];
    const NEG_B: [i32; 8] = [0, 0, -2, 0, -2, 0, 0, 0];
    const B: [i32; 8] = [0, 0, 2, 0, 2, 0, 0, 0];
    const D_ROOT: [i32; 8] = [2, -2, 0, 0, 0, 0, 0, 0];

    fn sample_snapshot(read_spec: ReadSpec, read_initialization: ReadInitialization) -> Snapshot {
        let tokens = [
            [B, A],
            [A, D_ROOT],
            [NEG_A, NEG_B],
            [C_ROOT, A],
            [D_ROOT, A],
        ];
        let spec = Spec {
            vocabulary: tokens.len() as u32,
            blocks: 2,
            context: 6,
            kept_cells: 2,
        };
        let stages = BinderPlan::new(spec.context as usize).unwrap().stages() as u32;
        let gate_len = read_spec.gate_count(spec.blocks, stages).unwrap() as usize;
        let mut gates = vec![0i32; gate_len];
        if read_spec != ReadSpec::Route {
            gates[2] = 1;
        }
        if matches!(read_spec, ReadSpec::Relation { .. }) {
            gates[6] = 1;
        }
        Snapshot {
            schema: SCHEMA.to_owned(),
            spec,
            read_spec,
            objective_policy: ObjectivePolicy::NearestAddressMarginV1,
            read_initialization,
            normalization_mode:
                crate::normalization_mechanism::NormalizationMode::ScaleMatchedSnapV1,
            position: None,
            seed: 7,
            epoch: 3,
            table: tokens.iter().flatten().flatten().copied().collect(),
            routes: vec![u32::from(geometry::IDENTITY); (spec.blocks * stages) as usize],
            gates,
            operator_kappa: geometry::OPERATOR_KAPPA.to_owned(),
            composition_kappa: geometry::COMPOSITION_KAPPA.to_owned(),
            kernel_sha256: "11".repeat(32),
            source_sha256: "22".repeat(32),
            geometry_sha256: "33".repeat(32),
        }
    }

    #[test]
    fn route_flock_and_relation_snapshots_roundtrip_through_uorlut01() {
        for (read_spec, init) in [
            (ReadSpec::Route, ReadInitialization::ZeroGatesV1),
            (
                ReadSpec::Flock { kept: 2 },
                ReadInitialization::ActiveLeafV1,
            ),
            (
                ReadSpec::Relation { kept: 1, sweeps: 1 },
                ReadInitialization::ActiveLeafV1,
            ),
        ] {
            let snapshot = sample_snapshot(read_spec, init);
            let cells = validate_and_extract_cells(&snapshot).unwrap();
            assert_eq!(cells, vec![224, 15, 19, 32, 240]);
            let bytes = export_d11(&snapshot).unwrap();
            assert_eq!(&bytes[..8], MAGIC);
            let restored = import_d11(&bytes).unwrap();
            assert_eq!(
                serde_json::to_value(&restored).unwrap(),
                serde_json::to_value(&snapshot).unwrap()
            );
        }
    }

    #[test]
    fn export_refuses_a_non_default_normalization_mode() {
        let default = sample_snapshot(
            ReadSpec::Flock { kept: 2 },
            ReadInitialization::ActiveLeafV1,
        );
        let bytes = export_d11(&default).unwrap();
        let mut control = default.clone();
        control.normalization_mode =
            crate::normalization_mechanism::NormalizationMode::UnitSnapControlV1;
        let error = export_d11(&control).unwrap_err().to_string();
        assert!(error.contains("normalization_mode"), "{error}");
        for route_control in [
            ReadInitialization::RouteBypassV1,
            ReadInitialization::RoutePermutedActionsV1,
        ] {
            let err = export_d11(&sample_snapshot(ReadSpec::Route, route_control))
                .unwrap_err()
                .to_string();
            assert!(err.contains("Route read-off or permuted control"), "{err}");
        }
        // The default path is unchanged by the refusal.
        assert_eq!(export_d11(&default).unwrap(), bytes);
    }

    #[test]
    fn export_refuses_position_transport_until_serving_support_exists() {
        use crate::position_mechanism::{PositionConfig, PositionMode, PositionStepTable};

        let default = sample_snapshot(
            ReadSpec::Flock { kept: 2 },
            ReadInitialization::ActiveLeafV1,
        );
        let bytes = export_d11(&default).unwrap();
        let mut positioned = default.clone();
        positioned.position = Some(PositionConfig::new(
            PositionMode::PrefixHolonomyV1,
            PositionStepTable::identity(2).unwrap(),
        ));
        let error = export_d11(&positioned).unwrap_err().to_string();
        assert!(error.contains("position"), "{error}");
        assert_eq!(export_d11(&default).unwrap(), bytes);
    }

    #[test]
    fn exporter_and_importer_refuse_malformed_snapshots_and_corrupted_containers() {
        let base = sample_snapshot(
            ReadSpec::Relation { kept: 1, sweeps: 1 },
            ReadInitialization::ActiveLeafV1,
        );
        // Non-orbit coordinate in table.
        let mut bad_orbit = base.clone();
        bad_orbit.table[0] = 3;
        assert!(export_d11(&bad_orbit).is_err());

        // Route out of range (>= 64).
        let mut bad_route = base.clone();
        bad_route.routes[0] = 64;
        assert!(export_d11(&bad_route).is_err());

        // Gate trit outside {-1, 0, 1}.
        let mut bad_gate = base.clone();
        bad_gate.gates[0] = 2;
        assert!(export_d11(&bad_gate).is_err());

        // Wrong operator_kappa.
        let mut bad_kappa = base.clone();
        bad_kappa.operator_kappa = "wrong".to_owned();
        assert!(export_d11(&bad_kappa).is_err());

        // Corrupt exported UORLUT01 bytes: flip a byte in `discrete.cells` or magic.
        let valid = export_d11(&base).unwrap();
        let mut bad_magic = valid.clone();
        bad_magic[0] ^= 1;
        assert!(import_d11(&bad_magic).is_err());

        let mut flipped_tail = valid.clone();
        let last = flipped_tail.len() - 1;
        flipped_tail[last] ^= 1;
        assert!(import_d11(&flipped_tail).is_err());
    }
}
