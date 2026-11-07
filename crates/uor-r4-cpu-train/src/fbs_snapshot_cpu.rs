//! CPU-only conversion of a saved SpiralCore/FBS checkpoint to the integer snapshot.
//!
//! This is deliberately a data converter, not a CPU model forward. It reads the saved F32
//! tensors, reconstructs the deterministic SpiralCore/FBS route plans, verifies their pinned
//! fingerprints, decodes the learned route selections, carries the saved Hamming read, and
//! applies the run's served contract. No CUDA device is opened.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;
use std::sync::{Arc, OnceLock};

use safetensors::{Dtype, SafeTensors};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uor_r4_core::spiralcore_operator::{
    cl06_bivectors, cl06_finite_composition_table, SignedMatrix8,
};
use uor_r4_integer::fbs_snapshot::{
    run_readout_nearest, write_new_file, FbsHammingRead, FbsHammingSnapshot, FbsTrunkLayer,
    FbsTrunkSnapshot,
};
use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01_with, FbsExportSnaps, FbsRoutes, IntegerStackModel,
};

const SCHEMA: &str = "spiralcore-fbs-projection/1";
const FBS_SOURCE_SHA256: &str = "917cb942811971d63cbe499f3a1641d3360c4dde7ebdc6b87e298c3cf1bbd747";
const DIMENSION: usize = 8;
const ORBIT: usize = 16;
const MAX_LEVELS: usize = 8;
const MAX_ATOMS: usize = 6560;
const ABSENT: u32 = u32::MAX;
const CONVOLUTION_WIDTH: usize = 4;
const RMS_EPSILON: f64 = 1e-5;
const HAMMING_DEPTHS: [usize; 2] = [2, 6];
const HAMMING_READOUT_DEPTH: usize = HAMMING_DEPTHS[1];
const FBS_PROJECTIONS: [&str; 6] = [
    "rec.in", "rec.gate", "rec.out", "mlp.gate", "mlp.up", "mlp.down",
];
const NATIVE_VECTOR_SCHEMA: &str = "uor-r4.shared-native-vector-writer/1";
const NATIVE_VECTOR_IDENTITY_PARTITION: &str = "token-id-modulo-256; quotient is collision slot";
const NATIVE_VECTOR_QUERY_PORTS: &str = "first-and-last-whole-8D-block; alias at width8";
const NATIVE_VECTOR_SCORE: &str = "negative-mean-L1-displacement; raw";
const NATIVE_VECTOR_ADDRESS_CREDIT: &str = "target-cell-versus-admitted-cell-margin; surrogate";

fn invalid(message: impl Into<String>) -> String {
    message.into()
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[derive(Clone, Debug, Serialize)]
struct Binder {
    children: Option<[u32; 3]>,
    atom: Option<u32>,
    level: u32,
    support_length: u64,
    first_atom: u32,
    atom_count: u32,
    parent: Option<u32>,
}

#[derive(Debug, Serialize)]
struct Geometry {
    operator_kappa: String,
    composition_kappa: String,
    roots: [[i8; DIMENSION]; ORBIT],
    axes: Vec<u32>,
    compose: Vec<u32>,
    root_of_state: Vec<u32>,
    bfs_actions: Vec<u32>,
    bfs_edges: Vec<Vec<u8>>,
    branch_targets: Vec<u32>,
    identity: u32,
}

fn geometry() -> Result<Arc<Geometry>, String> {
    static VALUE: OnceLock<Result<Arc<Geometry>, String>> = OnceLock::new();
    VALUE
        .get_or_init(|| compile_geometry().map(Arc::new))
        .clone()
}

fn compile_geometry() -> Result<Geometry, String> {
    let table = cl06_finite_composition_table().map_err(|error| error.to_string())?;
    let anchor = [2, 2, 0, 0, 0, 0, 0, 0];
    let mut points = BTreeSet::new();
    for state in table.states() {
        points.insert(
            state
                .apply_root_key(anchor)
                .map_err(|error| error.to_string())?,
        );
    }
    let mut ordered = vec![anchor];
    ordered.extend(points.into_iter().filter(|root| *root != anchor));
    let roots: [[i8; DIMENSION]; ORBIT] = ordered
        .try_into()
        .map_err(|_| invalid("SpiralCore anchor orbit is not sixteen roots"))?;
    let indices: BTreeMap<_, _> = roots
        .iter()
        .copied()
        .enumerate()
        .map(|(index, root)| (root, index as u32))
        .collect();
    let state_index = |matrix: &SignedMatrix8| -> Result<u32, String> {
        table
            .states()
            .iter()
            .position(|state| state == matrix)
            .map(|index| index as u32)
            .ok_or_else(|| invalid("BFS action is outside the bound SpiralCore group"))
    };
    let identity = state_index(&SignedMatrix8::identity())?;
    let mut axes = Vec::with_capacity(64 * DIMENSION);
    for matrix in table.states() {
        for row in matrix.entries() {
            let mut selected = None;
            for (column, &value) in row.iter().enumerate() {
                if value == 0 {
                    continue;
                }
                if selected.is_some() || !matches!(value, -1 | 1) {
                    return Err(invalid("non-permutation SpiralCore action"));
                }
                selected = Some(column as u32 | if value < 0 { 8 } else { 0 });
            }
            axes.push(selected.ok_or_else(|| invalid("empty SpiralCore axis"))?);
        }
    }
    let mut compose = Vec::with_capacity(4096);
    for left in table.states() {
        for right in table.states() {
            compose.push(state_index(
                &left
                    .checked_mul(*right)
                    .map_err(|error| error.to_string())?,
            )?);
        }
    }
    let root_of_state = table
        .states()
        .iter()
        .map(|state| {
            let root = state
                .apply_root_key(anchor)
                .map_err(|error| error.to_string())?;
            indices
                .get(&root)
                .copied()
                .ok_or_else(|| invalid("missing orbit root"))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let bivectors = cl06_bivectors().map_err(|error| error.to_string())?;
    let edge_actions = bivectors
        .iter()
        .map(|bivector| state_index(&bivector.matrix))
        .collect::<Result<Vec<_>, String>>()?;
    let mut neighbors = Vec::new();
    for root in &roots {
        let mut adjacent = Vec::new();
        for bivector in &bivectors {
            let image = bivector
                .matrix
                .apply_root_key(*root)
                .map_err(|error| error.to_string())?;
            adjacent.push(
                *indices
                    .get(&image)
                    .ok_or_else(|| invalid("edge leaves orbit"))?,
            );
        }
        neighbors.push(adjacent);
    }
    let mut bfs_actions = Vec::with_capacity(ORBIT * ORBIT);
    let mut bfs_edges = Vec::with_capacity(ORBIT * ORBIT);
    for start in 0..ORBIT {
        let mut paths: Vec<Option<(u32, Vec<u8>)>> = vec![None; ORBIT];
        paths[start] = Some((identity, Vec::new()));
        let mut queue = VecDeque::from([start]);
        while let Some(from) = queue.pop_front() {
            let (state, history) = paths[from]
                .clone()
                .ok_or_else(|| invalid("missing BFS prefix"))?;
            for (edge, &to) in neighbors[from].iter().enumerate() {
                if paths[to as usize].is_some() {
                    continue;
                }
                let mut next = history.clone();
                next.push(edge as u8);
                let action = compose[edge_actions[edge] as usize * 64 + state as usize];
                paths[to as usize] = Some((action, next));
                queue.push_back(to as usize);
            }
        }
        for path in paths {
            let (action, history) = path.ok_or_else(|| invalid("disconnected SpiralCore orbit"))?;
            bfs_actions.push(action);
            bfs_edges.push(history);
        }
    }
    let generator = bivectors
        .first()
        .ok_or_else(|| invalid("missing bivector"))?
        .matrix;
    let inverse = generator
        .checked_power(3)
        .map_err(|error| error.to_string())?;
    let mut branch_targets = Vec::with_capacity(ORBIT * 3);
    for root in &roots {
        let mut ports = BTreeSet::new();
        for action in [SignedMatrix8::identity(), generator, inverse] {
            let image = action
                .apply_root_key(*root)
                .map_err(|error| error.to_string())?;
            let port = *indices
                .get(&image)
                .ok_or_else(|| invalid("port leaves orbit"))?;
            ports.insert(port);
            branch_targets.push(port);
        }
        if ports.len() != 3 {
            return Err(invalid("ternary geometry ports coincide"));
        }
    }
    let report = table.validate().map_err(|error| error.to_string())?;
    Ok(Geometry {
        operator_kappa: report.operator_kappa,
        composition_kappa: table.reproduce_kappa().map_err(|error| error.to_string())?,
        roots,
        axes,
        compose,
        root_of_state,
        bfs_actions,
        bfs_edges,
        branch_targets,
        identity,
    })
}

#[derive(Debug, Serialize)]
struct RoutePlan {
    schema: &'static str,
    fbs_source_sha256: &'static str,
    atom_mode: [u32; 3],
    compiler_sha256: String,
    kernel_sha256: String,
    input_blocks: usize,
    output_blocks: usize,
    stages: usize,
    binders: Vec<Binder>,
    frontier: Vec<u32>,
    #[serde(serialize_with = "serialize_geometry")]
    geometry: Arc<Geometry>,
    #[serde(skip)]
    fingerprint: String,
}

fn serialize_geometry<S>(value: &Arc<Geometry>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    value.as_ref().serialize(serializer)
}

impl RoutePlan {
    fn new(input_width: usize, output_width: usize) -> Result<Self, String> {
        if input_width == 0
            || output_width == 0
            || !input_width.is_multiple_of(8)
            || !output_width.is_multiple_of(8)
        {
            return Err(invalid(
                "SpiralCore/FBS projection requires nonzero whole 8D carriers",
            ));
        }
        let input_blocks = input_width / 8;
        if input_blocks > MAX_ATOMS {
            return Err(invalid(
                "FBS projection exceeds the bound eight-rung frontier",
            ));
        }
        let mut binders: Vec<Binder> = Vec::new();
        let mut frontier: Vec<u32> = Vec::new();
        for atom in 0..input_blocks {
            let id = binders.len() as u32;
            binders.push(Binder {
                children: None,
                atom: Some(atom as u32),
                level: 0,
                support_length: 8,
                first_atom: atom as u32,
                atom_count: 1,
                parent: None,
            });
            frontier.push(id);
            loop {
                if frontier.len() < 3 {
                    break;
                }
                let count = frontier.len();
                let children = [
                    frontier[count - 3],
                    frontier[count - 2],
                    frontier[count - 1],
                ];
                let level = binders[children[0] as usize].level;
                if children
                    .iter()
                    .any(|&child| binders[child as usize].level != level)
                {
                    break;
                }
                let parent = binders.len() as u32;
                let span = children
                    .iter()
                    .map(|&child| binders[child as usize].support_length)
                    .sum::<u64>()
                    + 1;
                let atom_count = children
                    .iter()
                    .map(|&child| binders[child as usize].atom_count)
                    .sum();
                let first_atom = binders[children[0] as usize].first_atom;
                for child in children {
                    binders[child as usize].parent = Some(parent);
                }
                binders.push(Binder {
                    children: Some(children),
                    atom: None,
                    level: level + 1,
                    support_length: span,
                    first_atom,
                    atom_count,
                    parent: None,
                });
                frontier.truncate(count - 3);
                frontier.push(parent);
            }
        }
        let stages = 1 + binders
            .iter()
            .map(|binder| binder.level as usize)
            .max()
            .unwrap_or(0);
        if frontier.len() > ORBIT || stages > MAX_LEVELS {
            return Err(invalid(
                "FBS frontier cannot be represented by the bound SpiralCore orbit",
            ));
        }
        let mut plan = Self {
            schema: SCHEMA,
            fbs_source_sha256: FBS_SOURCE_SHA256,
            atom_mode: [2, 1, 8],
            compiler_sha256: sha256_hex(include_bytes!(
                "../../uor-r4-float-train/src/spiralcore_fbs.rs"
            )),
            kernel_sha256: sha256_hex(include_bytes!(
                "../../uor-r4-float-train/src/spiralcore_fbs.cu"
            )),
            input_blocks,
            output_blocks: output_width / 8,
            stages,
            binders,
            frontier,
            geometry: geometry()?,
            fingerprint: String::new(),
        };
        plan.fingerprint =
            sha256_hex(&serde_json::to_vec(&plan).map_err(|error| invalid(error.to_string()))?);
        Ok(plan)
    }

    fn query_shape(&self) -> [usize; 3] {
        [self.output_blocks, self.stages, DIMENSION]
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeVectorHead {
    schema: String,
    spec: WriterSpec,
    source_sha256: String,
    geometry: String,
    identity_partition: String,
    query_ports: String,
    score: String,
    address_credit: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WriterSpec {
    kept_cells: usize,
    vocabulary: usize,
    width: usize,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectionBinding {
    schema: String,
    fbs_source_sha256: String,
    plans: BTreeMap<String, String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct ModelConfig {
    arch: String,
    vocab_size: usize,
    width: usize,
    heads: usize,
    mlp_hidden: usize,
    context: usize,
    pattern: String,
    read: String,
    rotation: bool,
    seed: u64,
    #[serde(default)]
    rotation_group: Option<String>,
    memory: Option<Value>,
    select: Option<Value>,
    pointer: Option<Value>,
    read_router: Option<Value>,
    objective: Value,
    gate: String,
    rec_gates: String,
    quaternion_weights: String,
    head: String,
    native_vector_head: NativeVectorHead,
    spiralcore_fbs_projection: ProjectionBinding,
}

fn require_keys(value: &Value, expected: &[&str], what: &str) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid(format!("{what} root is not an object")))?;
    let actual: BTreeSet<_> = object.keys().map(String::as_str).collect();
    let expected: BTreeSet<_> = expected.iter().copied().collect();
    if actual != expected {
        let missing: Vec<_> = expected.difference(&actual).copied().collect();
        let extra: Vec<_> = actual.difference(&expected).copied().collect();
        return Err(invalid(format!(
            "{what} inventory differs; missing={missing:?}, extra={extra:?}"
        )));
    }
    Ok(())
}

fn parse_model_config(bytes: &[u8]) -> Result<ModelConfig, String> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|error| invalid(format!("model config: {error}")))?;
    require_keys(
        &value,
        &[
            "arch",
            "context",
            "gate",
            "head",
            "heads",
            "mlp_hidden",
            "native_vector_head",
            "objective",
            "pattern",
            "quaternion_weights",
            "read",
            "rec_gates",
            "rotation",
            "seed",
            "spiralcore_fbs_projection",
            "vocab_size",
            "width",
        ],
        "model config",
    )?;
    let config: ModelConfig =
        serde_json::from_value(value).map_err(|error| invalid(format!("model config: {error}")))?;
    if config.arch != "geometric"
        || config.read != "dot"
        || !config.rotation
        || config.rotation_group.is_some()
        || config.gate != "hamilton"
        || config.rec_gates != "geometric"
        || config.quaternion_weights != "spiralcore_fbs"
        || config.head != "spiralcore_product"
        || config.memory.is_some()
        || config.select.is_some()
        || config.pointer.is_some()
        || config.read_router.is_some()
    {
        return Err(invalid(
            "model config is not the served geometric FBS recipe",
        ));
    }
    let rivals = config
        .objective
        .get("margin_hinge")
        .and_then(|value| value.get("rivals"))
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid("model config objective is not margin_hinge with rivals"))?;
    if !(1..=64).contains(&rivals) {
        return Err(invalid("model config margin rivals is outside 1..=64"));
    }
    if config.vocab_size == 0
        || config.width == 0
        || !config.width.is_multiple_of(32)
        || config.mlp_hidden == 0
        || !config.mlp_hidden.is_multiple_of(32)
        || config.heads == 0
        || !config.heads.is_multiple_of(2)
        || !config.pattern.bytes().all(|byte| byte == b'r')
        || !(2..=4096).contains(&config.context)
    {
        return Err(invalid(
            "model config dimensions are not a served FBS trunk",
        ));
    }
    validate_native_vector_binding(&config)?;
    Ok(config)
}

fn validate_native_vector_binding(config: &ModelConfig) -> Result<(), String> {
    let head = &config.native_vector_head;
    let spec = &head.spec;
    let native_writer_sha256 = sha256_hex(include_bytes!(
        "../../uor-r4-float-train/src/native_vector_writer.rs"
    ));
    let native_writer_geometry = RoutePlan::new(8, 8)?.fingerprint;
    if head.schema != NATIVE_VECTOR_SCHEMA
        || head.source_sha256 != native_writer_sha256
        || head.geometry != native_writer_geometry
        || head.identity_partition != NATIVE_VECTOR_IDENTITY_PARTITION
        || head.query_ports != NATIVE_VECTOR_QUERY_PORTS
        || head.score != NATIVE_VECTOR_SCORE
        || head.address_credit != NATIVE_VECTOR_ADDRESS_CREDIT
        || spec.vocabulary != config.vocab_size
        || spec.width != config.width
        || !(256..=8192).contains(&spec.vocabulary)
        || spec.width == 0
        || !spec.width.is_multiple_of(8)
        || spec.width > 1024
        || !(1..=16).contains(&spec.kept_cells)
    {
        return Err(invalid("model config native vector head binding differs"));
    }
    Ok(())
}

fn projection_widths(part: &str, width: usize, mlp: usize) -> (usize, usize) {
    match part {
        "rec.in" => (width, 2 * width),
        "rec.gate" => (width, width / 4 + width),
        "rec.out" => (width, width),
        "mlp.gate" | "mlp.up" => (width, mlp),
        _ => (mlp, width),
    }
}

fn build_plans(config: &ModelConfig) -> Result<BTreeMap<(usize, usize), RoutePlan>, String> {
    let mut plans = BTreeMap::new();
    for part in FBS_PROJECTIONS {
        let (input_width, output_width) = projection_widths(part, config.width, config.mlp_hidden);
        plans
            .entry((input_width, output_width))
            .or_insert(RoutePlan::new(input_width, output_width)?);
    }
    Ok(plans)
}

fn validate_projection_binding(
    config: &ModelConfig,
    plans: &BTreeMap<(usize, usize), RoutePlan>,
) -> Result<(), String> {
    let binding = &config.spiralcore_fbs_projection;
    if binding.schema != SCHEMA || binding.fbs_source_sha256 != FBS_SOURCE_SHA256 {
        return Err(invalid("SpiralCore/FBS projection metadata differs"));
    }
    let expected: BTreeMap<String, String> = plans
        .iter()
        .map(|(&(input, output), plan)| (format!("{input}/{output}"), plan.fingerprint.clone()))
        .collect();
    if binding.plans != expected {
        return Err(invalid(format!(
            "SpiralCore/FBS plan fingerprints differ: config={:?}, CPU={expected:?}",
            binding.plans
        )));
    }
    Ok(())
}

fn f32_tensor(tensors: &SafeTensors<'_>, name: &str, shape: &[usize]) -> Result<Vec<f32>, String> {
    let tensor = tensors
        .tensor(name)
        .map_err(|error| invalid(format!("{name}: {error}")))?;
    if tensor.dtype() != Dtype::F32 || tensor.shape() != shape {
        return Err(invalid(format!(
            "{name} is {:?} {:?}, not F32 {shape:?}",
            tensor.dtype(),
            tensor.shape()
        )));
    }
    let values: Vec<f32> = tensor
        .data()
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().expect("four bytes")))
        .collect();
    if let Some(index) = values.iter().position(|value| !value.is_finite()) {
        return Err(invalid(format!("{name}[{index}] is not finite")));
    }
    Ok(values)
}

fn u32_tensor(tensors: &SafeTensors<'_>, name: &str, shape: &[usize]) -> Result<Vec<u32>, String> {
    let tensor = tensors
        .tensor(name)
        .map_err(|error| invalid(format!("{name}: {error}")))?;
    if tensor.dtype() != Dtype::U32 || tensor.shape() != shape {
        return Err(invalid(format!(
            "{name} is {:?} {:?}, not U32 {shape:?}",
            tensor.dtype(),
            tensor.shape()
        )));
    }
    Ok(tensor
        .data()
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four bytes")))
        .collect())
}

fn expected_trunk_shapes(
    config: &ModelConfig,
    plans: &BTreeMap<(usize, usize), RoutePlan>,
) -> BTreeMap<String, Vec<usize>> {
    let width = config.width;
    let lanes = width / 4;
    let mut shapes = BTreeMap::from([
        (
            "embedding.weight".to_owned(),
            vec![config.vocab_size, width],
        ),
        ("final_norm.weight".to_owned(), vec![width]),
    ]);
    for layer in 0..config.pattern.len() {
        for (name, shape) in [
            ("rec_norm.weight", vec![width]),
            ("mlp_norm.weight", vec![width]),
            ("rec.conv.weight", vec![CONVOLUTION_WIDTH, width]),
            ("rec.conv.bias", vec![width]),
            ("rec.gate.bias", vec![lanes + width]),
            ("rec.decay", vec![lanes]),
        ] {
            shapes.insert(format!("layers.{layer:02}.{name}"), shape);
        }
        for part in FBS_PROJECTIONS {
            let (input_width, output_width) = projection_widths(part, width, config.mlp_hidden);
            let plan = plans
                .get(&(input_width, output_width))
                .expect("all projection plans were built");
            shapes.insert(
                format!("layers.{layer:02}.{part}.spiralcore_fbs"),
                plan.query_shape().to_vec(),
            );
        }
    }
    shapes
}

fn validate_trunk_inventory(
    tensors: &SafeTensors<'_>,
    expected: &BTreeMap<String, Vec<usize>>,
) -> Result<(), String> {
    let actual: BTreeSet<&str> = tensors.names().into_iter().collect();
    let expected_names: BTreeSet<&str> = expected.keys().map(String::as_str).collect();
    if actual != expected_names {
        let missing: Vec<_> = expected_names.difference(&actual).copied().collect();
        let extra: Vec<_> = actual.difference(&expected_names).copied().collect();
        return Err(invalid(format!(
            "trunk tensor inventory differs; missing={missing:?}, extra={extra:?}"
        )));
    }
    Ok(())
}

fn routes_for_queries(plan: &RoutePlan, queries: &[f32]) -> Result<FbsRoutes, String> {
    let expected = plan.output_blocks * plan.stages * DIMENSION;
    if queries.len() != expected {
        return Err(invalid(format!(
            "route query has {} values, expected {expected}",
            queries.len()
        )));
    }
    let geometry = &plan.geometry;
    let mut atoms = Vec::with_capacity(plan.output_blocks);
    let mut states = Vec::with_capacity(plan.output_blocks);
    for output in 0..plan.output_blocks {
        let query =
            &queries[output * plan.stages * DIMENSION..(output + 1) * plan.stages * DIMENSION];
        let mut state = geometry.identity;
        let mut node = ABSENT;
        let mut atom = ABSENT;
        for stage in 0..plan.stages {
            let root = geometry.root_of_state[state as usize];
            let count = if stage == 0 { plan.frontier.len() } else { 3 };
            let mut selected = 0usize;
            let mut best = f32::INFINITY;
            for candidate in 0..count {
                let goal = if stage == 0 {
                    candidate as u32
                } else {
                    geometry.branch_targets[root as usize * 3 + candidate]
                };
                let mut distance = 0.0f32;
                for axis in 0..DIMENSION {
                    let delta = query[stage * DIMENSION + axis]
                        - f32::from(geometry.roots[goal as usize][axis]);
                    distance += delta * delta;
                }
                if !distance.is_finite() {
                    return Err(invalid(format!(
                        "output {output} stage {stage} has non-finite route distance"
                    )));
                }
                if distance < best {
                    best = distance;
                    selected = candidate;
                }
            }
            let goal = if stage == 0 {
                selected as u32
            } else {
                geometry.branch_targets[root as usize * 3 + selected]
            };
            let action = geometry.bfs_actions[root as usize * ORBIT + goal as usize];
            state = geometry.compose[action as usize * 64 + state as usize];
            node = if stage == 0 {
                plan.frontier[selected]
            } else {
                plan.binders[node as usize]
                    .children
                    .ok_or_else(|| invalid("route descended below a leaf"))?[selected]
            };
            atom = plan.binders[node as usize].atom.unwrap_or(ABSENT);
            if atom != ABSENT {
                break;
            }
        }
        if atom == ABSENT || atom as usize >= plan.input_blocks {
            return Err(invalid(format!(
                "output {output} did not terminate at an input block"
            )));
        }
        atoms.push(u16::try_from(atom).map_err(|_| invalid("route atom does not fit u16"))?);
        states.push(u8::try_from(state).map_err(|_| invalid("route state does not fit u8"))?);
    }
    Ok(FbsRoutes {
        in_blocks: plan.input_blocks,
        out_blocks: plan.output_blocks,
        atoms,
        states,
    })
}

fn parse_hamming(
    bytes: &[u8],
    width: usize,
    expected_heads: usize,
    layers: usize,
) -> Result<FbsHammingSnapshot, String> {
    if layers != HAMMING_READOUT_DEPTH {
        return Err(invalid(format!(
            "Hamming readout runs at depth {HAMMING_READOUT_DEPTH}, but the trunk has {layers} layers"
        )));
    }
    let tensors = SafeTensors::deserialize(bytes)
        .map_err(|error| invalid(format!("Hamming read: {error}")))?;
    let heads = *u32_tensor(&tensors, "heads", &[1])?
        .first()
        .ok_or_else(|| invalid("Hamming head count is empty"))? as usize;
    if heads == 0 || !heads.is_multiple_of(2) {
        return Err(invalid("Hamming head count is not a positive even number"));
    }
    if heads != expected_heads {
        return Err(invalid(format!(
            "Hamming head count {heads} differs from checkpoint head count {expected_heads}"
        )));
    }
    let copy_key = if tensors.tensor("copy_key").is_ok() {
        Some(
            *u32_tensor(&tensors, "copy_key", &[1])?
                .first()
                .ok_or_else(|| invalid("Hamming copy key is empty"))? as usize,
        )
    } else {
        None
    };
    let copy_keys = if tensors.tensor("copy_keys").is_ok() {
        Some(u32_tensor(&tensors, "copy_keys", &[heads / 2])?)
    } else {
        None
    };
    if copy_key.is_some() && copy_keys.is_some() {
        return Err(invalid(
            "Hamming read records both a copy key and per-head copy keys",
        ));
    }
    let mut expected = BTreeSet::from(["heads".to_owned()]);
    if copy_key.is_some() {
        expected.insert("copy_key".to_owned());
    }
    if copy_keys.is_some() {
        expected.insert("copy_keys".to_owned());
    }
    let quaternions = [width / 4, 4];
    let mut reads = Vec::with_capacity(HAMMING_DEPTHS.len());
    for depth in HAMMING_DEPTHS {
        for name in [
            "query_rotation",
            "key_rotation",
            "value_rotation",
            "margin",
            "weight",
        ] {
            expected.insert(format!("d{depth}.{name}"));
        }
        let margin = f32_tensor(&tensors, &format!("d{depth}.margin"), &[heads])?;
        if let Some(index) = margin.iter().position(|margin| *margin <= 0.0) {
            return Err(invalid(format!(
                "Hamming d{depth} margin[{index}] is not positive"
            )));
        }
        reads.push(FbsHammingRead {
            query_rotation: f32_tensor(
                &tensors,
                &format!("d{depth}.query_rotation"),
                &quaternions,
            )?,
            key_rotation: f32_tensor(&tensors, &format!("d{depth}.key_rotation"), &quaternions)?,
            value_rotation: f32_tensor(
                &tensors,
                &format!("d{depth}.value_rotation"),
                &quaternions,
            )?,
            margin,
            weight: f32_tensor(&tensors, &format!("d{depth}.weight"), &[2, width])?,
        });
    }
    let actual: BTreeSet<String> = tensors.names().into_iter().map(str::to_owned).collect();
    if actual != expected {
        let missing: Vec<_> = expected.difference(&actual).cloned().collect();
        let extra: Vec<_> = actual.difference(&expected).cloned().collect();
        return Err(invalid(format!(
            "Hamming tensor inventory differs; missing={missing:?}, extra={extra:?}"
        )));
    }
    let copy_key = copy_key.unwrap_or(1);
    if !matches!(copy_key, 1 | 2) {
        return Err(invalid("Hamming copy key is not 1 or 2"));
    }
    let copy_keys: Option<Vec<usize>> = copy_keys
        .map(|keys| {
            keys.into_iter()
                .map(|key| {
                    let key = key as usize;
                    if matches!(key, 1 | 2) {
                        Ok(key)
                    } else {
                        Err(invalid("Hamming per-head copy key is not 1 or 2"))
                    }
                })
                .collect::<Result<_, _>>()
        })
        .transpose()?;
    Ok(FbsHammingSnapshot {
        heads,
        depths: HAMMING_DEPTHS.to_vec(),
        reads,
        copy_key,
        copy_keys,
    })
}

fn validate_run_config(
    text: &str,
    config: &ModelConfig,
    snaps: FbsExportSnaps,
) -> Result<(bool, bool), String> {
    let value: Value =
        serde_json::from_str(text).map_err(|error| invalid(format!("run config: {error}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| invalid("run config root is not an object"))?;
    let field_bool = |name: &str| -> Result<bool, String> {
        match object.get(name) {
            None => Ok(false),
            Some(Value::Bool(value)) => Ok(*value),
            Some(_) => Err(invalid(format!("run config {name} is not a bool"))),
        }
    };
    if object.get("width").and_then(Value::as_u64) != Some(config.width as u64) {
        return Err(invalid("run config width differs from the checkpoint"));
    }
    if object.get("seed").and_then(Value::as_u64) != Some(config.seed) {
        return Err(invalid("run config seed differs from the checkpoint"));
    }
    if object.get("hamming_read").and_then(Value::as_u64) != Some(config.heads as u64) {
        return Err(invalid(
            "run config Hamming read count differs from the checkpoint",
        ));
    }
    if field_bool("fold_scores")? || field_bool("reflect_floor")? {
        return Err(invalid(
            "run config uses a rate transform the integer exporter does not serve",
        ));
    }
    if field_bool("train_as_served")? && (snaps.gates || snaps.transition) {
        return Err(invalid(
            "train-as-served checkpoints require both --raw-gates and --raw-transition",
        ));
    }
    let nearest = run_readout_nearest(text)?;
    Ok((field_bool("retain_rates")?, nearest))
}

/// Convert a saved FBS checkpoint without opening CUDA.
pub fn snapshot_from_saved_checkpoint(
    checkpoint: &Path,
    run_config: &Path,
    snaps: FbsExportSnaps,
) -> Result<FbsTrunkSnapshot, String> {
    let entries: BTreeSet<String> = std::fs::read_dir(checkpoint)
        .map_err(|error| invalid(format!("{}: {error}", checkpoint.display())))?
        .map(|entry| {
            entry
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .map_err(|error| error.to_string())
        })
        .collect::<Result<_, _>>()?;
    for entry in &entries {
        if entry.ends_with(".safetensors")
            && ![
                "model.safetensors",
                "optimizer.safetensors",
                "hamming_read.safetensors",
            ]
            .contains(&entry.as_str())
        {
            return Err(invalid(format!(
                "checkpoint holds unsupported served tensor file {entry}"
            )));
        }
    }
    let model_bytes = std::fs::read(checkpoint.join("model.safetensors"))
        .map_err(|error| invalid(format!("model.safetensors: {error}")))?;
    let config_bytes = std::fs::read(checkpoint.join("config.json"))
        .map_err(|error| invalid(format!("config.json: {error}")))?;
    let hamming_bytes = std::fs::read(checkpoint.join("hamming_read.safetensors"))
        .map_err(|error| invalid(format!("hamming_read.safetensors: {error}")))?;
    let config = parse_model_config(&config_bytes)?;
    let plans = build_plans(&config)?;
    validate_projection_binding(&config, &plans)?;
    let run_text = std::fs::read_to_string(run_config)
        .map_err(|error| invalid(format!("{}: {error}", run_config.display())))?;
    let (retain_rates, readout_nearest) = validate_run_config(&run_text, &config, snaps)?;

    let tensors = SafeTensors::deserialize(&model_bytes)
        .map_err(|error| invalid(format!("model.safetensors: {error}")))?;
    let shapes = expected_trunk_shapes(&config, &plans);
    validate_trunk_inventory(&tensors, &shapes)?;
    let width = config.width;
    let lanes = width / 4;
    let mut layers = Vec::with_capacity(config.pattern.len());
    for layer in 0..config.pattern.len() {
        let prefix = format!("layers.{layer:02}");
        let mut routes = Vec::with_capacity(FBS_PROJECTIONS.len());
        for part in FBS_PROJECTIONS {
            let (input_width, output_width) = projection_widths(part, width, config.mlp_hidden);
            let plan = &plans[&(input_width, output_width)];
            let values = f32_tensor(
                &tensors,
                &format!("{prefix}.{part}.spiralcore_fbs"),
                &plan.query_shape(),
            )?;
            routes.push(routes_for_queries(plan, &values)?);
        }
        let [rec_in, rec_gate, rec_out, mlp_gate, mlp_up, mlp_down]: [FbsRoutes; 6] = routes
            .try_into()
            .map_err(|_| invalid("six routes per layer"))?;
        layers.push(FbsTrunkLayer {
            rec_norm: f32_tensor(&tensors, &format!("{prefix}.rec_norm.weight"), &[width])?,
            mlp_norm: f32_tensor(&tensors, &format!("{prefix}.mlp_norm.weight"), &[width])?,
            rec_in,
            rec_gate,
            rec_out,
            mlp_gate,
            mlp_up,
            mlp_down,
            conv_taps: f32_tensor(
                &tensors,
                &format!("{prefix}.rec.conv.weight"),
                &[CONVOLUTION_WIDTH, width],
            )?,
            conv_bias: f32_tensor(&tensors, &format!("{prefix}.rec.conv.bias"), &[width])?,
            gate_bias: f32_tensor(
                &tensors,
                &format!("{prefix}.rec.gate.bias"),
                &[lanes + width],
            )?,
            decay: f32_tensor(&tensors, &format!("{prefix}.rec.decay"), &[lanes])?,
        });
    }
    Ok(FbsTrunkSnapshot {
        vocab: config.vocab_size,
        width,
        mlp: config.mlp_hidden,
        context: config.context,
        rms_eps: RMS_EPSILON,
        kept_cells: config.native_vector_head.spec.kept_cells,
        embed: f32_tensor(&tensors, "embedding.weight", &[config.vocab_size, width])?,
        final_norm: f32_tensor(&tensors, "final_norm.weight", &[width])?,
        layers,
        hamming: Some(parse_hamming(
            &hamming_bytes,
            width,
            config.heads,
            config.pattern.len(),
        )?),
        memory: None,
        retain_rates,
        readout_nearest,
    })
}

/// Build and validate the snapshot, then export the integer artifact.
pub fn export_saved_checkpoint(
    checkpoint: &Path,
    run_config: &Path,
    snapshot_out: &Path,
    artifact_out: &Path,
    snaps: FbsExportSnaps,
) -> Result<(String, String), String> {
    if snapshot_out.exists() || artifact_out.exists() {
        return Err(invalid("snapshot or artifact output already exists"));
    }
    let snapshot = snapshot_from_saved_checkpoint(checkpoint, run_config, snaps)?;
    let snapshot_bytes =
        serde_json::to_vec(&snapshot).map_err(|error| invalid(error.to_string()))?;
    let snapshot_sha256 = sha256_hex(&snapshot_bytes);
    let artifact = export_fbs_trunk_uorlut01_with(&snapshot, snaps)
        .map_err(|error| invalid(error.to_string()))?;
    let artifact_sha256 = sha256_hex(&artifact);
    IntegerStackModel::parse(&artifact).map_err(|error| invalid(error.to_string()))?;
    write_new_file(snapshot_out, &snapshot_bytes)
        .map_err(|error| invalid(format!("{}: {error}", snapshot_out.display())))?;
    write_new_file(artifact_out, &artifact)
        .map_err(|error| invalid(format!("{}: {error}", artifact_out.display())))?;
    Ok((snapshot_sha256, artifact_sha256))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use safetensors::{serialize, tensor::TensorView};
    use serde_json::json;

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "uor-r4-cpu-fbs-export-{name}-{}",
                std::process::id()
            ));
            if path.exists() {
                std::fs::remove_dir_all(&path).unwrap();
            }
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn tensor_file(tensors: Vec<(&str, Dtype, Vec<usize>, Vec<u8>)>) -> Vec<u8> {
        let tensors: Vec<_> = tensors
            .into_iter()
            .map(|(name, dtype, shape, data)| (name.to_owned(), dtype, shape, data))
            .collect();
        let views: Vec<_> = tensors
            .iter()
            .map(|(name, dtype, shape, data)| {
                (
                    name.clone(),
                    TensorView::new(*dtype, shape.clone(), data).unwrap(),
                )
            })
            .collect();
        serialize(views, None).unwrap()
    }

    fn f32_bytes(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
        values
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>()
    }

    fn model_config_value() -> Value {
        let mut plans = BTreeMap::new();
        for (input, output) in [(32, 64), (32, 40), (32, 32)] {
            plans.insert(
                format!("{input}/{output}"),
                RoutePlan::new(input, output).unwrap().fingerprint,
            );
        }
        let native_writer_sha256 = sha256_hex(include_bytes!(
            "../../uor-r4-float-train/src/native_vector_writer.rs"
        ));
        let native_writer_geometry = RoutePlan::new(8, 8).unwrap().fingerprint;
        json!({
            "arch": "geometric",
            "context": 32,
            "gate": "hamilton",
            "head": "spiralcore_product",
            "heads": 2,
            "mlp_hidden": 32,
            "native_vector_head": {
                "schema": NATIVE_VECTOR_SCHEMA,
                "spec": {
                    "kept_cells": 1,
                    "vocabulary": 256,
                    "width": 32
                },
                "source_sha256": native_writer_sha256,
                "geometry": native_writer_geometry,
                "identity_partition": NATIVE_VECTOR_IDENTITY_PARTITION,
                "query_ports": NATIVE_VECTOR_QUERY_PORTS,
                "score": NATIVE_VECTOR_SCORE,
                "address_credit": NATIVE_VECTOR_ADDRESS_CREDIT
            },
            "objective": {"margin_hinge": {"rivals": 2}},
            "pattern": "rr",
            "quaternion_weights": "spiralcore_fbs",
            "read": "dot",
            "rec_gates": "geometric",
            "rotation": true,
            "seed": 1,
            "spiralcore_fbs_projection": {
                "schema": SCHEMA,
                "fbs_source_sha256": FBS_SOURCE_SHA256,
                "plans": plans
            },
            "vocab_size": 256,
            "width": 32
        })
    }

    fn hamming_tensor_map(
        width: usize,
        heads: usize,
        extra: bool,
    ) -> Vec<(&'static str, Dtype, Vec<usize>, Vec<u8>)> {
        let quaternions = [width / 4, 4];
        let mut tensors = vec![
            (
                "heads",
                Dtype::U32,
                vec![1],
                (heads as u32).to_le_bytes().to_vec(),
            ),
            (
                "d2.query_rotation",
                Dtype::F32,
                quaternions.to_vec(),
                f32_bytes(vec![0.0; width]),
            ),
            (
                "d2.key_rotation",
                Dtype::F32,
                quaternions.to_vec(),
                f32_bytes(vec![0.0; width]),
            ),
            (
                "d2.value_rotation",
                Dtype::F32,
                quaternions.to_vec(),
                f32_bytes(vec![0.0; width]),
            ),
            (
                "d2.margin",
                Dtype::F32,
                vec![heads],
                f32_bytes(vec![1.0; heads]),
            ),
            (
                "d2.weight",
                Dtype::F32,
                vec![2, width],
                f32_bytes(vec![1.0; 2 * width]),
            ),
            (
                "d6.query_rotation",
                Dtype::F32,
                quaternions.to_vec(),
                f32_bytes(vec![0.0; width]),
            ),
            (
                "d6.key_rotation",
                Dtype::F32,
                quaternions.to_vec(),
                f32_bytes(vec![0.0; width]),
            ),
            (
                "d6.value_rotation",
                Dtype::F32,
                quaternions.to_vec(),
                f32_bytes(vec![0.0; width]),
            ),
            (
                "d6.margin",
                Dtype::F32,
                vec![heads],
                f32_bytes(vec![1.0; heads]),
            ),
            (
                "d6.weight",
                Dtype::F32,
                vec![2, width],
                f32_bytes(vec![1.0; 2 * width]),
            ),
        ];
        if extra {
            tensors.push(("foreign", Dtype::F32, vec![1], f32_bytes([0.0])));
        }
        tensors
    }

    #[test]
    fn b64l_projection_fingerprints_match_the_trained_binding() {
        let plans = [
            (
                256,
                256,
                "82a27aea1f49cf6e6b947192480584a0be0f90d1dab8a7c8763df6776dc4bf10",
            ),
            (
                256,
                320,
                "3fc1be258ebf4f2041299207656ca2a36a0caa97301f0de68541d25a469f2180",
            ),
            (
                256,
                384,
                "ebc6d0e4e8d5e6252f4b12a87c9dea3898ae7bab4b68bdcfd3aa5517a58226e7",
            ),
            (
                256,
                512,
                "fb54bd1ee8fbfa40ce6307b9696d460b3ff27fa42cd77b52bd457c5aad324b1e",
            ),
            (
                384,
                256,
                "f8c4a34b363c130fd166cd97cc158be70352360596d5adcb31c2efc686d89b02",
            ),
        ];
        for (input, output, fingerprint) in plans {
            assert_eq!(
                RoutePlan::new(input, output).unwrap().fingerprint,
                fingerprint
            );
        }
        assert_eq!(
            RoutePlan::new(8, 8).unwrap().fingerprint,
            "32a15af2d48680e331d5c1823dbb128dd23e01c4a9cf2f98d8bd0ef0c1a2937c"
        );
    }

    #[test]
    fn route_query_shape_is_derived_from_the_projection() {
        let plan = RoutePlan::new(256, 384).unwrap();
        assert_eq!(plan.query_shape(), [48, 4, 8]);
        assert_eq!(
            routes_for_queries(&plan, &vec![0.0; 48 * 4 * 8])
                .unwrap()
                .out_blocks,
            48
        );
    }

    #[test]
    fn model_config_rejects_malformed_missing_and_extra_root_fields() {
        let malformed = parse_model_config(b"{").unwrap_err();
        assert!(malformed.contains("model config"));

        let mut missing = model_config_value();
        missing.as_object_mut().unwrap().remove("context");
        let error = parse_model_config(&serde_json::to_vec(&missing).unwrap()).unwrap_err();
        assert!(error.contains("missing=[\"context\"]"), "{error}");

        let mut extra = model_config_value();
        extra["foreign"] = json!(true);
        let error = parse_model_config(&serde_json::to_vec(&extra).unwrap()).unwrap_err();
        assert!(error.contains("extra=[\"foreign\"]"), "{error}");

        let mut malformed_seed = model_config_value();
        malformed_seed["seed"] = json!("1");
        let error = parse_model_config(&serde_json::to_vec(&malformed_seed).unwrap()).unwrap_err();
        assert!(error.contains("model config"), "{error}");
    }

    #[test]
    fn model_config_rejects_a_foreign_served_contract() {
        let mut foreign = model_config_value();
        foreign["head"] = json!("native_vector_nearest");
        let error = parse_model_config(&serde_json::to_vec(&foreign).unwrap()).unwrap_err();
        assert!(
            error.contains("not the served geometric FBS recipe"),
            "{error}"
        );
    }

    #[test]
    fn model_config_rejects_a_foreign_native_vector_binding() {
        for (field, value) in [
            ("geometry", json!("0".repeat(64))),
            ("source_sha256", json!("0".repeat(64))),
            ("query_ports", json!("foreign-port-contract")),
        ] {
            let mut foreign = model_config_value();
            foreign["native_vector_head"][field] = value;
            let error = parse_model_config(&serde_json::to_vec(&foreign).unwrap()).unwrap_err();
            assert!(
                error.contains("native vector head binding differs"),
                "{field}: {error}"
            );
        }
    }

    #[test]
    fn projection_binding_rejects_a_wrong_fingerprint_set() {
        let config =
            parse_model_config(&serde_json::to_vec(&model_config_value()).unwrap()).unwrap();
        let plans = build_plans(&config).unwrap();
        validate_projection_binding(&config, &plans).unwrap();

        let mut wrong = config;
        wrong
            .spiralcore_fbs_projection
            .plans
            .insert("32/32".to_owned(), "0".repeat(64));
        let error = validate_projection_binding(&wrong, &plans).unwrap_err();
        assert!(error.contains("plan fingerprints differ"), "{error}");
    }

    #[test]
    fn trunk_inventory_rejects_missing_and_extra_tensors() {
        let bytes = tensor_file(vec![("present", Dtype::F32, vec![1], f32_bytes([0.0]))]);
        let tensors = SafeTensors::deserialize(&bytes).unwrap();

        let missing = BTreeMap::from([
            ("missing".to_owned(), vec![1]),
            ("present".to_owned(), vec![1]),
        ]);
        let error = validate_trunk_inventory(&tensors, &missing).unwrap_err();
        assert!(error.contains("missing=[\"missing\"]"), "{error}");

        let extra = BTreeMap::from([("different".to_owned(), vec![1])]);
        let error = validate_trunk_inventory(&tensors, &extra).unwrap_err();
        assert!(error.contains("extra=[\"present\"]"), "{error}");
    }

    #[test]
    fn hamming_inventory_rejects_missing_and_extra_tensors() {
        let complete = tensor_file(hamming_tensor_map(32, 2, false));
        parse_hamming(&complete, 32, 2, 6).unwrap();

        let mut missing = hamming_tensor_map(32, 2, false);
        missing.retain(|(name, ..)| *name != "d6.weight");
        let missing = tensor_file(missing);
        let error = parse_hamming(&missing, 32, 2, 6).unwrap_err();
        assert!(error.contains("d6.weight"), "{error}");

        let extra = tensor_file(hamming_tensor_map(32, 2, true));
        let error = parse_hamming(&extra, 32, 2, 6).unwrap_err();
        assert!(error.contains("extra=[\"foreign\"]"), "{error}");
    }

    #[test]
    fn hamming_read_rejects_a_head_count_that_differs_from_the_checkpoint() {
        let bytes = tensor_file(hamming_tensor_map(32, 4, false));
        let error = parse_hamming(&bytes, 32, 2, 6).unwrap_err();
        assert!(
            error.contains("Hamming head count 4 differs from checkpoint head count 2"),
            "{error}"
        );
    }

    #[test]
    fn hamming_read_rejects_a_trunk_that_is_not_the_six_layer_readout_shape() {
        let bytes = tensor_file(hamming_tensor_map(32, 2, false));
        let error = parse_hamming(&bytes, 32, 2, 7).unwrap_err();
        assert!(
            error.contains("Hamming readout runs at depth 6, but the trunk has 7 layers"),
            "{error}"
        );
    }

    #[test]
    fn run_config_rejects_non_boolean_contract_flags() {
        let config =
            parse_model_config(&serde_json::to_vec(&model_config_value()).unwrap()).unwrap();
        for (field, value) in [
            ("train_as_served", json!("true")),
            ("retain_rates", json!(1)),
            ("fold_scores", json!("false")),
            ("reflect_floor", json!(null)),
        ] {
            let mut run = json!({"width": 32, "seed": 1, "hamming_read": 2});
            run[field] = value;
            let error = validate_run_config(
                &serde_json::to_string(&run).unwrap(),
                &config,
                FbsExportSnaps::SNAPPED,
            )
            .unwrap_err();
            assert!(error.contains(&format!("{field} is not a bool")), "{error}");
        }
        let error = validate_run_config(
            r#"{"width":32,"seed":2,"hamming_read":2}"#,
            &config,
            FbsExportSnaps::SNAPPED,
        )
        .unwrap_err();
        assert!(error.contains("run config seed differs"), "{error}");
    }

    #[test]
    fn f32_tensor_rejects_non_finite_values() {
        let bytes = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![2],
            f32_bytes([0.0, f32::NAN]),
        )]);
        let tensors = SafeTensors::deserialize(&bytes).unwrap();
        let error = f32_tensor(&tensors, "embedding.weight", &[2]).unwrap_err();
        assert!(
            error.contains("embedding.weight[1] is not finite"),
            "{error}"
        );
    }

    #[test]
    fn export_refuses_an_existing_output_before_reading_the_checkpoint() {
        let root = TestDir::new("existing-output");
        let snapshot = root.0.join("snapshot.json");
        let artifact = root.0.join("artifact.uorlut");
        std::fs::write(&snapshot, b"occupied").unwrap();

        let error = export_saved_checkpoint(
            &root.0.join("absent-checkpoint"),
            &root.0.join("absent-run-config.json"),
            &snapshot,
            &artifact,
            FbsExportSnaps::SNAPPED,
        )
        .unwrap_err();
        assert!(error.contains("output already exists"), "{error}");
        assert_eq!(std::fs::read(&snapshot).unwrap(), b"occupied");
        assert!(!artifact.exists());
    }
}
