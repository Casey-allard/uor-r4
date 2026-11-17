//! Read-only CPU audit of Claude-334's saved carry-hold checkpoint fields.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(target_os = "linux")]
use std::io::Read;
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
#[cfg(target_os = "linux")]
use std::os::unix::fs::OpenOptionsExt;
use uor_r4_cpu_train::ask_offset_audit::{
    AskHoldSummary, AskOffsetRelationReading, TrainedAskOffsets,
};

const SCHEMA: &str = "uor-r4.cpu-carry-hold-audit/2";
const BATCH_SCHEMA: &str = "uor-r4.cpu-carry-hold-batch/1";
const MANIFEST_SCHEMA: &str = "uor-r4.cpu-carry-hold-manifest/1";
const RELATION_TOKENS: std::ops::Range<u32> = 16..32;
const QUERY_SHIFT: u32 = 16;
const DEFAULT_WORKERS: usize = 8;
const MAX_WORKERS: usize = 32;

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CarryHoldManifest {
    schema: String,
    arms: Vec<CarryHoldArm>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CarryHoldArm {
    id: String,
    label: String,
    seed: u32,
    checkpoint: String,
    model_sha256: String,
    hamming_sha256: String,
    expected_hold_rows: usize,
}

#[derive(Clone, Debug, Serialize)]
struct CheckpointReading {
    model_sha256: String,
    hamming_sha256: String,
    vocab: usize,
    width: usize,
    heads: usize,
    ask_heads: usize,
    depths: Vec<usize>,
    hold_rows: usize,
    hold: AskHoldSummary,
    ask: AskSummary,
    relations: Vec<AskOffsetRelationReading>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct AskSummary {
    relations: usize,
    cells: usize,
    zero_exact_codes: usize,
    trained_exact_codes: usize,
    zero_total_bits: u64,
    trained_total_bits: u64,
    improvement_bits: i64,
    improved_cells: usize,
    regressed_cells: usize,
    zero_target_nearest: usize,
    trained_target_nearest: usize,
}

#[derive(Debug)]
enum Invocation {
    Single(BTreeMap<String, String>),
    Batch {
        manifest: PathBuf,
        out: PathBuf,
        workers: usize,
    },
}

#[derive(Debug)]
struct ValidatedManifest {
    root: PathBuf,
    checkpoints: Vec<PathBuf>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-carry-hold-audit: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match parse_args(std::env::args().skip(1))? {
        Invocation::Single(args) => run_with(&args),
        Invocation::Batch {
            manifest,
            out,
            workers,
        } => run_batch(&manifest, &out, workers),
    }
}

fn run_with(args: &BTreeMap<String, String>) -> Result<()> {
    let checkpoint = Path::new(&args["checkpoint"]);
    let expected_rows = parse_hold_rows(&args["expected-hold-rows"])?;
    let out = PathBuf::from(&args["out"]);
    require_new_output(&out)?;
    let reading = audit_checkpoint(
        checkpoint,
        None,
        &args["expected-model-sha256"],
        &args["expected-hamming-sha256"],
        expected_rows,
    )?;
    let executable_sha256 = executable_sha256()?;
    let report = serde_json::json!({
        "schema": SCHEMA,
        "checkpoint": reading,
        "settings": {
            "relation_tokens": RELATION_TOKENS,
            "query_shift": QUERY_SHIFT,
            "expected_hold_rows": expected_rows,
        },
        "hardware": {
            "cpu": cpu_model(),
            "executable_sha256": executable_sha256,
        },
        "claim": "read-only held-checkpoint structure, grouped gain and ask-selection audit; not B2, carry, capability or serving evidence"
    });
    let mut bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    write_new(&out, &bytes)?;
    println!(
        "{}",
        serde_json::json!({
            "schema": SCHEMA,
            "out": out,
            "sha256": uor_r4_discrete::corpus::sha256(&bytes),
            "hold": report["checkpoint"]["hold"],
            "ask": report["checkpoint"]["ask"],
        })
    );
    Ok(())
}

fn run_batch(manifest_path: &Path, out: &Path, workers: usize) -> Result<()> {
    require_new_output(out)?;
    if workers == 0 || workers > MAX_WORKERS {
        return Err(format!("--workers must be within 1..={MAX_WORKERS}"));
    }
    let bytes = std::fs::read(manifest_path)
        .map_err(|error| format!("{}: {error}", manifest_path.display()))?;
    let manifest_sha256 = uor_r4_discrete::corpus::sha256(&bytes);
    let manifest: CarryHoldManifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("{}: {error}", manifest_path.display()))?;
    let validated = validate_manifest(manifest_path, &manifest)?;
    let executable_sha256 = executable_sha256()?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build()
        .map_err(|error| format!("worker pool: {error}"))?;
    let root = &validated.root;
    let arms = pool.install(|| {
        manifest
            .arms
            .par_iter()
            .zip(validated.checkpoints.par_iter())
            .map(|(arm, checkpoint)| {
                Ok(serde_json::json!({
                    "id": arm.id,
                    "label": arm.label,
                    "seed": arm.seed,
                    "checkpoint": arm.checkpoint,
                    "reading": audit_checkpoint(
                        checkpoint,
                        Some(root),
                        &arm.model_sha256,
                        &arm.hamming_sha256,
                        arm.expected_hold_rows,
                    )?,
                }))
            })
            .collect::<Result<Vec<_>>>()
    })?;
    let report = serde_json::json!({
        "schema": BATCH_SCHEMA,
        "manifest_sha256": manifest_sha256,
        "settings": {
            "relation_tokens": RELATION_TOKENS,
            "query_shift": QUERY_SHIFT,
            "workers": workers,
        },
        "hardware": {
            "cpu": cpu_model(),
            "executable_sha256": executable_sha256,
        },
        "arms": arms,
        "claim": "read-only multi-checkpoint grouped gain and ask-selection audit; not B2, carry, capability or serving evidence"
    });
    let mut encoded = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    encoded.push(b'\n');
    write_new(out, &encoded)?;
    println!(
        "{}",
        serde_json::json!({
            "schema": BATCH_SCHEMA,
            "out": out,
            "sha256": uor_r4_discrete::corpus::sha256(&encoded),
            "manifest_sha256": manifest_sha256,
            "arms": manifest.arms.len(),
            "workers": workers,
        })
    );
    Ok(())
}

fn audit_checkpoint(
    checkpoint: &Path,
    root: Option<&Path>,
    expected_model_sha256: &str,
    expected_hamming_sha256: &str,
    expected_rows: usize,
) -> Result<CheckpointReading> {
    let model_path = checkpoint.join("model.safetensors");
    let hamming_path = checkpoint.join("hamming_read.safetensors");
    let model = read_checkpoint_file(&model_path, root)?;
    let hamming = read_checkpoint_file(&hamming_path, root)?;
    let model_sha = uor_r4_discrete::corpus::sha256(&model);
    let hamming_sha = uor_r4_discrete::corpus::sha256(&hamming);
    validate_sha256(expected_model_sha256, "model")?;
    validate_sha256(expected_hamming_sha256, "hamming")?;
    if model_sha != expected_model_sha256 {
        return Err(format!(
            "model.safetensors SHA-256 differs: got {model_sha}"
        ));
    }
    if hamming_sha != expected_hamming_sha256 {
        return Err(format!(
            "hamming_read.safetensors SHA-256 differs: got {hamming_sha}"
        ));
    }

    let offsets = TrainedAskOffsets::from_bytes(&model, &hamming)?;
    let depths = offsets.depths().collect::<Vec<_>>();
    if depths != [2, 6] {
        return Err(format!(
            "held checkpoint read depths are {depths:?}, not the Claude-334 [2, 6]"
        ));
    }
    let hold = offsets
        .hold_summary()
        .ok_or_else(|| "checkpoint carries no saved d2.hold".to_owned())?;
    if hold.question_gain.is_none()
        || hold.header_gain.is_none()
        || hold.value_gain.is_none()
        || hold.rest_gain_mean.is_none()
    {
        return Err(format!(
            "vocabulary {} does not cover the two-class question, header and value token groups",
            offsets.vocab()
        ));
    }
    let type_weight_depths = hold
        .type_weight_by_depth
        .iter()
        .map(|weight| weight.depth)
        .collect::<Vec<_>>();
    if type_weight_depths != depths {
        return Err(format!(
            "checkpoint carries type weights for depths {type_weight_depths:?}, expected {depths:?}"
        ));
    }
    if hold.rows != expected_rows {
        return Err(format!(
            "checkpoint holds for {} rows, expected {expected_rows}",
            hold.rows
        ));
    }
    let relations = offsets.audit_relations(RELATION_TOKENS, QUERY_SHIFT)?;
    let ask = summarize_ask(&relations);
    Ok(CheckpointReading {
        model_sha256: model_sha,
        hamming_sha256: hamming_sha,
        vocab: offsets.vocab(),
        width: offsets.width(),
        heads: offsets.heads(),
        ask_heads: offsets.ask_head_count(),
        depths,
        hold_rows: hold.rows,
        hold,
        ask,
        relations,
    })
}

fn read_checkpoint_file(path: &Path, root: Option<&Path>) -> Result<Vec<u8>> {
    let Some(root) = root else {
        return std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()));
    };
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if !canonical.starts_with(root) {
        return Err(format!(
            "{} resolves outside the manifest directory",
            path.display()
        ));
    }
    #[cfg(not(target_os = "linux"))]
    {
        return Err(format!(
            "{}: manifest checkpoint containment needs Linux /proc/self/fd; use single-checkpoint mode",
            canonical.display()
        ));
    }
    #[cfg(target_os = "linux")]
    {
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW)
            .open(&canonical)
            .map_err(|error| format!("{}: {error}", canonical.display()))?;
        if !file
            .metadata()
            .map_err(|error| format!("{}: {error}", canonical.display()))?
            .is_file()
        {
            return Err(format!("{} is not a regular file", canonical.display()));
        }
        let opened = PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()))
            .canonicalize()
            .map_err(|error| format!("opened {}: {error}", canonical.display()))?;
        if !opened.starts_with(root) {
            return Err(format!(
                "opened {} resolves outside the manifest directory",
                canonical.display()
            ));
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|error| format!("{}: {error}", canonical.display()))?;
        Ok(bytes)
    }
}

fn summarize_ask(relations: &[AskOffsetRelationReading]) -> AskSummary {
    let cells = relations
        .iter()
        .flat_map(|relation| relation.cells.iter())
        .collect::<Vec<_>>();
    AskSummary {
        relations: relations.len(),
        cells: cells.len(),
        zero_exact_codes: cells.iter().filter(|cell| cell.zero_bits == 0).count(),
        trained_exact_codes: cells.iter().filter(|cell| cell.trained_bits == 0).count(),
        zero_total_bits: cells
            .iter()
            .map(|cell| u64::from(cell.zero_bits))
            .sum::<u64>(),
        trained_total_bits: cells
            .iter()
            .map(|cell| u64::from(cell.trained_bits))
            .sum::<u64>(),
        improvement_bits: cells
            .iter()
            .map(|cell| i64::from(cell.improvement_bits))
            .sum::<i64>(),
        improved_cells: cells
            .iter()
            .filter(|cell| cell.improvement_bits > 0)
            .count(),
        regressed_cells: cells
            .iter()
            .filter(|cell| cell.improvement_bits < 0)
            .count(),
        zero_target_nearest: cells.iter().filter(|cell| cell.zero_target_nearest).count(),
        trained_target_nearest: cells
            .iter()
            .filter(|cell| cell.trained_target_nearest)
            .count(),
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Invocation> {
    const SINGLE_FLAGS: [&str; 5] = [
        "checkpoint",
        "expected-model-sha256",
        "expected-hamming-sha256",
        "expected-hold-rows",
        "out",
    ];
    const BATCH_FLAGS: [&str; 3] = ["manifest", "out", "workers"];
    let mut all = BTreeMap::new();
    let mut args = args;
    while let Some(argument) = args.next() {
        let name = argument
            .strip_prefix("--")
            .ok_or_else(|| format!("unknown argument {argument}"))?;
        if !SINGLE_FLAGS.contains(&name) && !BATCH_FLAGS.contains(&name) {
            return Err(format!("unknown argument {argument}"));
        }
        let value = args
            .next()
            .ok_or_else(|| format!("{argument} needs a value"))?;
        if all.insert(name.to_owned(), value).is_some() {
            return Err(format!("{argument} given twice"));
        }
    }
    if all.contains_key("manifest") {
        if all.keys().any(|name| !BATCH_FLAGS.contains(&name.as_str())) {
            return Err("manifest mode accepts only --manifest, --out and --workers".to_owned());
        }
        if !all.contains_key("out") {
            return Err("--out is required in manifest mode".to_owned());
        }
        let workers = match all.get("workers") {
            Some(value) => value
                .parse::<usize>()
                .map_err(|error| format!("--workers: {error}"))?,
            None => DEFAULT_WORKERS,
        };
        if workers == 0 || workers > MAX_WORKERS {
            return Err(format!("--workers must be within 1..={MAX_WORKERS}"));
        }
        return Ok(Invocation::Batch {
            manifest: PathBuf::from(&all["manifest"]),
            out: PathBuf::from(&all["out"]),
            workers,
        });
    }
    if all
        .keys()
        .any(|name| !SINGLE_FLAGS.contains(&name.as_str()))
    {
        return Err("single-checkpoint mode accepts only --checkpoint, the two SHA-256 pins, --expected-hold-rows and --out".to_owned());
    }
    for name in SINGLE_FLAGS {
        if !all.contains_key(name) {
            return Err(format!("--{name} is required"));
        }
    }
    Ok(Invocation::Single(all))
}

fn parse_hold_rows(value: &str) -> Result<usize> {
    let rows = value
        .parse::<usize>()
        .map_err(|error| format!("--expected-hold-rows: {error}"))?;
    if rows == 0 {
        return Err("--expected-hold-rows must be positive".to_owned());
    }
    Ok(rows)
}

fn validate_manifest(path: &Path, manifest: &CarryHoldManifest) -> Result<ValidatedManifest> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "manifest schema is {:?}, not {MANIFEST_SCHEMA}",
            manifest.schema
        ));
    }
    if manifest.arms.is_empty() {
        return Err("manifest carries no arms".to_owned());
    }
    let root = manifest_root(path)?;
    let mut ids = BTreeSet::new();
    let mut labels_seeds = BTreeSet::new();
    let mut paths_seen = BTreeSet::new();
    let mut hashes_seen = BTreeSet::new();
    let mut checkpoints = Vec::with_capacity(manifest.arms.len());
    for arm in &manifest.arms {
        if arm.id.is_empty() || arm.label.is_empty() {
            return Err("arm id and label must not be empty".to_owned());
        }
        if arm.seed == 0 {
            return Err(format!("arm {} has seed 0", arm.id));
        }
        if arm.expected_hold_rows == 0 {
            return Err(format!("arm {} has zero expected hold rows", arm.id));
        }
        validate_sha256(&arm.model_sha256, "model")?;
        validate_sha256(&arm.hamming_sha256, "hamming")?;
        if !ids.insert(arm.id.as_str()) {
            return Err(format!("duplicate arm id {}", arm.id));
        }
        if !labels_seeds.insert((arm.label.as_str(), arm.seed)) {
            return Err(format!("duplicate label/seed {}/{}", arm.label, arm.seed));
        }
        let resolved = resolve_checkpoint(&root, &arm.checkpoint)?;
        if !paths_seen.insert(resolved.clone()) {
            return Err(format!("duplicate checkpoint path {}", arm.checkpoint));
        }
        if !hashes_seen.insert((arm.model_sha256.as_str(), arm.hamming_sha256.as_str())) {
            return Err(format!(
                "duplicate model/Hamming checkpoint hashes on arm {}",
                arm.id
            ));
        }
        checkpoints.push(resolved);
    }
    Ok(ValidatedManifest { root, checkpoints })
}

fn manifest_root(path: &Path) -> Result<PathBuf> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn resolve_checkpoint(root: &Path, checkpoint: &str) -> Result<PathBuf> {
    let relative = Path::new(checkpoint);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err(format!(
            "checkpoint path {checkpoint:?} must be a contained relative path"
        ));
    }
    let resolved = root
        .join(relative)
        .canonicalize()
        .map_err(|error| format!("{}: {error}", root.join(relative).display()))?;
    if !resolved.starts_with(root) {
        return Err(format!(
            "checkpoint path {checkpoint:?} resolves outside the manifest directory"
        ));
    }
    if !resolved
        .metadata()
        .map_err(|error| format!("{}: {error}", resolved.display()))?
        .is_dir()
    {
        return Err(format!("{} is not a directory", resolved.display()));
    }
    Ok(resolved)
}

fn require_new_output(out: &Path) -> Result<()> {
    if out.exists() {
        return Err(format!(
            "{} exists; reports are never replaced",
            out.display()
        ));
    }
    Ok(())
}

fn validate_sha256(value: &str, name: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} SHA-256 is malformed"));
    }
    Ok(())
}

fn cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("model name\t: ")
                    .map(std::string::ToString::to_string)
            })
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

fn executable_sha256() -> Result<String> {
    let path = std::env::current_exe().map_err(|error| error.to_string())?;
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    Ok(uor_r4_discrete::corpus::sha256(&bytes))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(format!("{}: {error}", path.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use safetensors::{serialize, tensor::TensorView, Dtype};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "uor-r4-carry-hold-audit-{name}-{}",
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

    fn f32_bytes(values: &[f32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn u32_bytes(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn tensor_file(tensors: Vec<(&str, Dtype, Vec<usize>, Vec<u8>)>) -> Vec<u8> {
        let tensors = tensors
            .into_iter()
            .map(|(name, dtype, shape, data)| (name.to_owned(), dtype, shape, data))
            .collect::<Vec<_>>();
        let views = tensors
            .iter()
            .map(|(name, dtype, shape, data)| {
                (
                    name.clone(),
                    TensorView::new(*dtype, shape.clone(), data).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        serialize(views, None).unwrap()
    }

    const TEST_VOCAB: usize = 1_100;

    fn write_checkpoint(dir: &Path, salt: f32) -> (String, String) {
        write_checkpoint_with_d6_weight(dir, salt, true)
    }

    fn write_checkpoint_with_d6_weight(
        dir: &Path,
        salt: f32,
        include_d6_weight: bool,
    ) -> (String, String) {
        let (vocab, width, heads) = (TEST_VOCAB, 8usize, 2usize);
        let mut table = vec![1.0f32; vocab * width];
        table[18 * width..19 * width].fill(-1.0);
        table[1_000 * width] = salt;
        let model = tensor_file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(&table),
        )]);
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        let mut ask = vec![0.0f32; vocab * width];
        ask[33 * width..34 * width].fill(-2.0);
        let mut gain = vec![0.0f32; vocab];
        gain[1] = 0.5;
        gain[2] = -0.25;
        gain[32] = 1.0;
        gain[47] = -1.0;
        gain[48] = 0.5;
        gain[49] = 0.5;
        gain[50] = -0.25;
        gain[51] = -0.25;
        gain[528] = 2.0;
        gain[1_039] = 4.0;
        let mut d2_weight = vec![0.0f32; 2 * width];
        d2_weight[width + 1] = 1.5;
        let mut d6_weight = vec![0.0f32; 2 * width];
        d6_weight[width + 2] = -0.5;
        let mut tensors = vec![
            ("heads", Dtype::U32, vec![1], u32_bytes(&[heads as u32])),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width / 4, 4],
                f32_bytes(&rotation),
            ),
            ("d2.ask", Dtype::F32, vec![vocab, width], f32_bytes(&ask)),
            (
                "d2.weight",
                Dtype::F32,
                vec![2, width],
                f32_bytes(&d2_weight),
            ),
            ("d2.hold", Dtype::F32, vec![vocab], f32_bytes(&gain)),
            ("hold_rows", Dtype::U32, vec![1], u32_bytes(&[8])),
            (
                "d6.key_rotation",
                Dtype::F32,
                vec![width / 4, 4],
                f32_bytes(&rotation),
            ),
            ("d6.ask", Dtype::F32, vec![vocab, width], f32_bytes(&ask)),
        ];
        if include_d6_weight {
            tensors.push((
                "d6.weight",
                Dtype::F32,
                vec![2, width],
                f32_bytes(&d6_weight),
            ));
        }
        let hamming = tensor_file(tensors);
        std::fs::write(dir.join("model.safetensors"), &model).unwrap();
        std::fs::write(dir.join("hamming_read.safetensors"), &hamming).unwrap();
        (
            uor_r4_discrete::corpus::sha256(&model),
            uor_r4_discrete::corpus::sha256(&hamming),
        )
    }

    fn args(
        dir: &TestDir,
        checkpoint: &Path,
        model: &str,
        hamming: &str,
    ) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("checkpoint".to_owned(), checkpoint.display().to_string()),
            ("expected-model-sha256".to_owned(), model.to_owned()),
            ("expected-hamming-sha256".to_owned(), hamming.to_owned()),
            ("expected-hold-rows".to_owned(), "8".to_owned()),
            (
                "out".to_owned(),
                dir.0.join("report.json").display().to_string(),
            ),
        ])
    }

    fn retain_receipt(source: &Path, destination: &Path) -> Result<()> {
        let bytes =
            std::fs::read(source).map_err(|error| format!("{}: {error}", source.display()))?;
        write_new(destination, &bytes)
    }

    #[test]
    fn retained_receipt_collision_preserves_the_original_bytes() {
        let dir = TestDir::new("retention");
        let source = dir.0.join("report.json");
        let destination = dir.0.join("retained.json");
        let original = b"first receipt with trailing bytes\n";
        std::fs::write(&source, original).unwrap();
        retain_receipt(&source, &destination).unwrap();
        std::fs::write(&source, b"new\n").unwrap();
        assert!(retain_receipt(&source, &destination).is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), original);
    }

    #[test]
    fn synthetic_report_is_complete_and_deterministic() {
        let dir = TestDir::new("report");
        let checkpoint = dir.0.join("checkpoint");
        std::fs::create_dir(&checkpoint).unwrap();
        let (model, hamming) = write_checkpoint(&checkpoint, 1.25);
        let args = args(&dir, &checkpoint, &model, &hamming);
        run_with(&args).unwrap();
        let out = PathBuf::from(&args["out"]);
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
        assert_eq!(report["schema"], SCHEMA);
        assert_eq!(report["checkpoint"]["hold_rows"], 8);
        assert_eq!(report["checkpoint"]["hold"]["gain_nonzero"], 10);
        assert_eq!(report["checkpoint"]["hold"]["write_weight_nonzero"], 1);
        assert_eq!(
            report["checkpoint"]["hold"]["question_gain"],
            serde_json::json!({"mean": 0.0, "min": -1.0, "max": 1.0})
        );
        assert_eq!(
            report["checkpoint"]["hold"]["header_gain"],
            serde_json::json!({"mean": 0.125, "min": -0.25, "max": 0.5})
        );
        assert_eq!(
            report["checkpoint"]["hold"]["value_gain"],
            serde_json::json!({"mean": 6.0 / 512.0, "min": 0.0, "max": 4.0})
        );
        assert!(
            (report["checkpoint"]["hold"]["rest_gain_mean"]
                .as_f64()
                .unwrap()
                - 0.25 / 568.0)
                .abs()
                < 1e-15
        );
        assert_eq!(
            report["checkpoint"]["hold"]["type_weight_by_depth"],
            serde_json::json!([
                {"depth": 2, "mean_abs": 0.1875},
                {"depth": 6, "mean_abs": 0.0625}
            ])
        );
        assert_eq!(report["checkpoint"]["ask"]["trained_target_nearest"], 28);
        assert_eq!(report["checkpoint"]["ask"]["regressed_cells"], 2);

        let mut repeat_args = args.clone();
        repeat_args.insert(
            "out".to_owned(),
            dir.0.join("repeat.json").display().to_string(),
        );
        run_with(&repeat_args).unwrap();
        assert_eq!(
            std::fs::read(&out).unwrap(),
            std::fs::read(&repeat_args["out"]).unwrap()
        );
        assert!(run_with(&args)
            .unwrap_err()
            .contains("reports are never replaced"));
        if let Some(path) = std::env::var_os("UOR_KEEP_CARRY_HOLD_AUDIT_EVIDENCE") {
            retain_receipt(&out, Path::new(&path)).unwrap();
        }
    }

    #[test]
    fn hashes_and_expected_rows_are_refused() {
        let dir = TestDir::new("refuse");
        let checkpoint = dir.0.join("checkpoint");
        std::fs::create_dir(&checkpoint).unwrap();
        let (model, hamming) = write_checkpoint(&checkpoint, 1.0);

        let mut bad_hash = args(&dir, &checkpoint, &model, &hamming);
        bad_hash.insert("expected-model-sha256".to_owned(), "0".repeat(64));
        assert!(run_with(&bad_hash)
            .unwrap_err()
            .contains("model.safetensors SHA-256 differs"));

        let mut bad_hamming = args(&dir, &checkpoint, &model, &hamming);
        bad_hamming.insert("expected-hamming-sha256".to_owned(), "0".repeat(64));
        assert!(run_with(&bad_hamming)
            .unwrap_err()
            .contains("hamming_read.safetensors SHA-256 differs"));

        let mut bad_rows = args(&dir, &checkpoint, &model, &hamming);
        bad_rows.insert("expected-hold-rows".to_owned(), "7".to_owned());
        assert!(run_with(&bad_rows)
            .unwrap_err()
            .contains("checkpoint holds for 8 rows, expected 7"));

        let missing_weight = dir.0.join("missing-weight");
        std::fs::create_dir(&missing_weight).unwrap();
        let (model, hamming) = write_checkpoint_with_d6_weight(&missing_weight, 1.0, false);
        let mut missing_weight_args = args(&dir, &missing_weight, &model, &hamming);
        missing_weight_args.insert(
            "out".to_owned(),
            dir.0.join("missing-weight.json").display().to_string(),
        );
        assert!(run_with(&missing_weight_args)
            .unwrap_err()
            .contains("type weights for depths [2], expected [2, 6]"));
    }

    #[test]
    fn parser_accepts_single_and_batch_shapes() {
        let base = [
            "--checkpoint",
            "c",
            "--expected-model-sha256",
            "m",
            "--expected-hamming-sha256",
            "h",
            "--expected-hold-rows",
            "8",
            "--out",
            "o",
        ];
        assert!(matches!(
            parse_args(base.into_iter().map(str::to_owned)).unwrap(),
            Invocation::Single(_)
        ));
        let batch = parse_args(
            ["--manifest", "m", "--out", "o"]
                .into_iter()
                .map(str::to_owned),
        )
        .unwrap();
        assert!(matches!(
            batch,
            Invocation::Batch {
                workers: DEFAULT_WORKERS,
                ..
            }
        ));
        assert!(parse_args(["--checkpoint", "c"].into_iter().map(str::to_owned)).is_err());
        assert!(parse_args(
            ["--manifest", "m", "--out", "o", "--workers", "0"]
                .into_iter()
                .map(str::to_owned)
        )
        .is_err());
        assert!(parse_args(
            ["--manifest", "m", "--out", "o", "--workers", "33"]
                .into_iter()
                .map(str::to_owned)
        )
        .is_err());
        assert!(parse_args(
            base.into_iter()
                .chain(["--workers", "8"])
                .map(str::to_owned)
        )
        .is_err());
    }

    #[test]
    fn basename_manifest_uses_the_current_directory() {
        let current = std::env::current_dir().unwrap().canonicalize().unwrap();
        assert_eq!(manifest_root(Path::new("MANIFEST.json")).unwrap(), current);
        assert_eq!(
            manifest_root(Path::new("./MANIFEST.json")).unwrap(),
            current
        );
    }

    fn write_batch_fixture(dir: &TestDir) -> PathBuf {
        let first = dir.0.join("first");
        let second = dir.0.join("second");
        std::fs::create_dir(&first).unwrap();
        std::fs::create_dir(&second).unwrap();
        let (model1, hamming1) = write_checkpoint(&first, 1.0);
        let (model2, hamming2) = write_checkpoint(&second, 1.25);
        let manifest = serde_json::json!({
            "schema": MANIFEST_SCHEMA,
            "arms": [
                {
                    "id": "aqh-s2",
                    "label": "B2-aqH",
                    "seed": 2,
                    "checkpoint": "second",
                    "model_sha256": model2,
                    "hamming_sha256": hamming2,
                    "expected_hold_rows": 8
                },
                {
                    "id": "aqh-s1",
                    "label": "B2-aqH",
                    "seed": 1,
                    "checkpoint": "first",
                    "model_sha256": model1,
                    "hamming_sha256": hamming1,
                    "expected_hold_rows": 8
                }
            ]
        });
        let manifest_path = dir.0.join("manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        manifest_path
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn batch_refuses_a_valid_manifest_without_linux_descriptor_checks() {
        let dir = TestDir::new("batch-refusal");
        let manifest_path = write_batch_fixture(&dir);
        let out = dir.0.join("batch.json");
        let error = run_batch(&manifest_path, &out, 2).unwrap_err();
        assert!(error.contains("manifest checkpoint containment needs Linux /proc/self/fd"));
        assert!(error.contains("use single-checkpoint mode"));
        assert!(!out.exists());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn batch_preserves_order_and_repeats_byte_for_byte() {
        let dir = TestDir::new("batch");
        let manifest_path = write_batch_fixture(&dir);
        let out = dir.0.join("batch.json");
        run_batch(&manifest_path, &out, 2).unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
        assert_eq!(report["arms"][0]["id"], "aqh-s2");
        assert_eq!(report["arms"][1]["id"], "aqh-s1");
        assert_eq!(
            report["arms"][0]["reading"]["hold"]["question_gain"]["max"],
            1.0
        );

        let repeat = dir.0.join("batch-repeat.json");
        run_batch(&manifest_path, &repeat, 2).unwrap();
        assert_eq!(
            std::fs::read(&out).unwrap(),
            std::fs::read(&repeat).unwrap()
        );
        if let Some(path) = std::env::var_os("UOR_KEEP_CARRY_HOLD_BATCH_EVIDENCE") {
            retain_receipt(&out, Path::new(&path)).unwrap();
        }
        assert!(run_batch(&manifest_path, &out, 2)
            .unwrap_err()
            .contains("reports are never replaced"));
    }

    #[test]
    fn manifest_rejects_duplicate_and_unsafe_identities() {
        let dir = TestDir::new("manifest-refuse");
        let checkpoint = dir.0.join("checkpoint");
        std::fs::create_dir(&checkpoint).unwrap();
        let (model, hamming) = write_checkpoint(&checkpoint, 1.0);
        let arm = serde_json::json!({
            "id": "aqh-s1",
            "label": "B2-aqH",
            "seed": 1,
            "checkpoint": "checkpoint",
            "model_sha256": model,
            "hamming_sha256": hamming,
            "expected_hold_rows": 8
        });
        let manifest_path = dir.0.join("manifest.json");
        let bytes = |arms: serde_json::Value| {
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": MANIFEST_SCHEMA,
                "arms": arms
            }))
            .unwrap()
        };
        std::fs::write(&manifest_path, bytes(serde_json::json!([arm.clone(), arm]))).unwrap();
        assert!(run_batch(&manifest_path, &dir.0.join("one.json"), 1)
            .unwrap_err()
            .contains("duplicate arm id"));

        let mut duplicate_label = arm.clone();
        duplicate_label["id"] = serde_json::json!("aqh-s1-again");
        std::fs::write(
            &manifest_path,
            bytes(serde_json::json!([arm.clone(), duplicate_label])),
        )
        .unwrap();
        assert!(run_batch(&manifest_path, &dir.0.join("label.json"), 1)
            .unwrap_err()
            .contains("duplicate label/seed"));

        let mut duplicate_path = arm.clone();
        duplicate_path["id"] = serde_json::json!("ah-s2");
        duplicate_path["label"] = serde_json::json!("B2-aH");
        duplicate_path["seed"] = serde_json::json!(2);
        duplicate_path["checkpoint"] = serde_json::json!("./checkpoint");
        std::fs::write(
            &manifest_path,
            bytes(serde_json::json!([arm.clone(), duplicate_path])),
        )
        .unwrap();
        assert!(run_batch(&manifest_path, &dir.0.join("path.json"), 1)
            .unwrap_err()
            .contains("duplicate checkpoint path"));

        let copied_checkpoint = dir.0.join("checkpoint-copy");
        std::fs::create_dir(&copied_checkpoint).unwrap();
        for name in ["model.safetensors", "hamming_read.safetensors"] {
            std::fs::copy(checkpoint.join(name), copied_checkpoint.join(name)).unwrap();
        }
        let mut duplicate_hashes = arm.clone();
        duplicate_hashes["id"] = serde_json::json!("ah-s2");
        duplicate_hashes["label"] = serde_json::json!("B2-aH");
        duplicate_hashes["seed"] = serde_json::json!(2);
        duplicate_hashes["checkpoint"] = serde_json::json!("checkpoint-copy");
        std::fs::write(
            &manifest_path,
            bytes(serde_json::json!([arm.clone(), duplicate_hashes])),
        )
        .unwrap();
        assert!(run_batch(&manifest_path, &dir.0.join("hashes.json"), 1)
            .unwrap_err()
            .contains("duplicate model/Hamming checkpoint hashes"));

        let traversal = serde_json::json!({
            "id": "bad",
            "label": "B2-aH",
            "seed": 2,
            "checkpoint": "../outside",
            "model_sha256": "0".repeat(64),
            "hamming_sha256": "1".repeat(64),
            "expected_hold_rows": 8
        });
        std::fs::write(&manifest_path, bytes(serde_json::json!([traversal]))).unwrap();
        assert!(run_batch(&manifest_path, &dir.0.join("two.json"), 1)
            .unwrap_err()
            .contains("contained relative path"));

        let wrong_schema =
            serde_json::to_vec_pretty(&serde_json::json!({"schema": "wrong", "arms": []})).unwrap();
        std::fs::write(&manifest_path, wrong_schema).unwrap();
        assert!(run_batch(&manifest_path, &dir.0.join("three.json"), 1)
            .unwrap_err()
            .contains("manifest schema"));
    }

    #[cfg(unix)]
    #[test]
    fn batch_rejects_child_symlinks_outside_the_manifest_root() {
        use std::os::unix::fs::symlink;

        let dir = TestDir::new("symlink-escape");
        let manifest_root = dir.0.join("manifest-root");
        let outside = dir.0.join("outside-checkpoint");
        let arm_dir = manifest_root.join("arm");
        std::fs::create_dir(&manifest_root).unwrap();
        std::fs::create_dir(&outside).unwrap();
        std::fs::create_dir(&arm_dir).unwrap();
        let (model, hamming) = write_checkpoint(&outside, 1.0);
        for name in ["model.safetensors", "hamming_read.safetensors"] {
            symlink(outside.join(name), arm_dir.join(name)).unwrap();
        }
        let manifest_path = manifest_root.join("manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": MANIFEST_SCHEMA,
                "arms": [{
                    "id": "aqh-s1",
                    "label": "B2-aqH",
                    "seed": 1,
                    "checkpoint": "arm",
                    "model_sha256": model,
                    "hamming_sha256": hamming,
                    "expected_hold_rows": 8
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(run_batch(&manifest_path, &dir.0.join("escaped.json"), 1)
            .unwrap_err()
            .contains("resolves outside the manifest directory"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn batch_rejects_non_regular_checkpoint_files() {
        let dir = TestDir::new("non-regular");
        let manifest_root = dir.0.join("manifest-root");
        let arm_dir = manifest_root.join("arm");
        std::fs::create_dir(&manifest_root).unwrap();
        std::fs::create_dir(&arm_dir).unwrap();
        std::fs::create_dir(arm_dir.join("model.safetensors")).unwrap();
        let manifest_path = manifest_root.join("manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": MANIFEST_SCHEMA,
                "arms": [{
                    "id": "aqh-s1",
                    "label": "B2-aqH",
                    "seed": 1,
                    "checkpoint": "arm",
                    "model_sha256": "0".repeat(64),
                    "hamming_sha256": "1".repeat(64),
                    "expected_hold_rows": 8
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(
            run_batch(&manifest_path, &dir.0.join("non-regular.json"), 1)
                .unwrap_err()
                .contains("is not a regular file")
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn batch_refuses_fifo_without_blocking() {
        use std::sync::mpsc;
        use std::time::Duration;

        let dir = TestDir::new("fifo");
        let manifest_root = dir.0.join("manifest-root");
        let arm_dir = manifest_root.join("arm");
        std::fs::create_dir(&manifest_root).unwrap();
        std::fs::create_dir(&arm_dir).unwrap();
        let fifo = arm_dir.join("model.safetensors");
        let status = std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap();
        assert!(status.success(), "mkfifo failed: {status}");
        let manifest_path = manifest_root.join("manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": MANIFEST_SCHEMA,
                "arms": [{
                    "id": "aqh-s1",
                    "label": "B2-aqH",
                    "seed": 1,
                    "checkpoint": "arm",
                    "model_sha256": "0".repeat(64),
                    "hamming_sha256": "1".repeat(64),
                    "expected_hold_rows": 8
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        let out = dir.0.join("fifo.json");
        let worker_manifest = manifest_path.clone();
        let worker_out = out.clone();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(run_batch(&worker_manifest, &worker_out, 1));
        });
        let result = receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("FIFO open blocked instead of failing closed");
        assert!(result.unwrap_err().contains("is not a regular file"));
        assert!(!out.exists());
    }
}
