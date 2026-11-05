use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use serde::Serialize;
use uor_r4_cpu_train::{audit_served_forward, ServedForwardAuditConfig, ServedForwardAuditReport};
use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth};
use uor_r4_integer::stack::{
    export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot, FbsReplyTable, MEMORY_PAD,
};

const SCHEMA: &str = "uor-r4.cpu-scale-serving-bench/1";
const VOCAB: usize = 4_096;
const MLP: usize = 384;
const LAYERS: usize = 6;
const KEPT_CELLS: usize = 8;
const WIDTHS: [usize; 3] = [256, 512, 1_024];
const CONTEXTS: [usize; 3] = [16, 64, 192];
const REPEATS: usize = 3;
const BASE_SEED: u64 = 20_261_105;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Arm {
    Projection,
    Full,
}

impl Arm {
    const ALL: [Self; 2] = [Self::Projection, Self::Full];

    fn name(self) -> &'static str {
        match self {
            Self::Projection => "projection",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
struct ParityReading {
    positions_checked: usize,
    query_coordinates_compared: u64,
    query_coordinate_mismatches: u64,
    token_decisions_compared: u64,
    token_mismatches: u64,
    kept_cells_compared: u64,
    kept_cell_mismatches: u64,
    cpu_decision_sha256: String,
    serving_decision_sha256: String,
}

impl ParityReading {
    fn from_audit(report: &ServedForwardAuditReport) -> Self {
        let parity = &report.parity;
        Self {
            positions_checked: parity.positions_checked,
            query_coordinates_compared: parity.query_coordinates_compared,
            query_coordinate_mismatches: parity.query_coordinate_mismatches,
            token_decisions_compared: parity.token_decisions_compared,
            token_mismatches: parity.token_mismatches,
            kept_cells_compared: parity.kept_cells_compared,
            kept_cell_mismatches: parity.kept_cell_mismatches,
            cpu_decision_sha256: parity.cpu_decision_sha256.clone(),
            serving_decision_sha256: parity.serving_decision_sha256.clone(),
        }
    }

    fn exact(&self) -> bool {
        self.query_coordinate_mismatches == 0
            && self.token_mismatches == 0
            && self.kept_cell_mismatches == 0
            && self.cpu_decision_sha256 == self.serving_decision_sha256
    }
}

#[derive(Debug, Serialize)]
struct CellReading {
    width: usize,
    context: usize,
    arm: &'static str,
    seed: u64,
    hamming_heads: Option<usize>,
    artifact_sha256: String,
    artifact_bytes: u64,
    serialized_table_payload_bytes: u64,
    decoded_resident_table_bytes: u64,
    repeats: usize,
    parity: ParityReading,
    repeat_decision_sha256s: Vec<String>,
    cpu_oracle_tokens_per_second: f64,
    serving_tokens_per_second: f64,
    audit_elapsed_ms: f64,
    peak_rss_kib: u64,
}

#[derive(Debug, Serialize)]
struct NegativeControl {
    name: &'static str,
    refused: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct ScaleServingSummary {
    schema: &'static str,
    substrate: &'static str,
    qualification: &'static str,
    claim_boundary: &'static str,
    widths: Vec<usize>,
    contexts: Vec<usize>,
    arms: Vec<&'static str>,
    vocab: usize,
    mlp: usize,
    layers: usize,
    kept_cells: usize,
    readout_mode: &'static str,
    repeats: usize,
    cpu_model: String,
    logical_cpus: usize,
    cells: Vec<CellReading>,
    negative_controls: Vec<NegativeControl>,
    elapsed_ms: f64,
    peak_rss_kib: u64,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-scale-serving-bench: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let output = parse_args()?;
    let summary = run_grid(&WIDTHS, &CONTEXTS, REPEATS, BASE_SEED)?;
    let mut bytes = serde_json::to_vec_pretty(&summary).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    if let Some(path) = output {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        file.write_all(&bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?;
    } else {
        std::io::stdout()
            .write_all(&bytes)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn parse_args() -> Result<Option<PathBuf>, String> {
    let mut output = None;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--output" => {
                let path = args
                    .next()
                    .ok_or_else(|| format!("--output needs a path\n{}", usage()))?;
                output = Some(PathBuf::from(path));
            }
            "--help" | "-h" => return Err(usage()),
            _ => return Err(format!("unknown argument {argument}\n{}", usage())),
        }
    }
    Ok(output)
}

fn usage() -> String {
    "usage: cpu-scale-serving-bench [--output PATH]".to_owned()
}

fn run_grid(
    widths: &[usize],
    contexts: &[usize],
    repeats: usize,
    base_seed: u64,
) -> Result<ScaleServingSummary, String> {
    if repeats < 2 {
        return Err("scale-serving preflight needs at least two deterministic repeats".to_owned());
    }
    if widths.is_empty() || contexts.is_empty() {
        return Err("scale-serving preflight needs at least one width and context".to_owned());
    }
    let started = Instant::now();
    let mut cells = Vec::with_capacity(widths.len() * contexts.len() * Arm::ALL.len());
    for &width in widths {
        for &context in contexts {
            let seed = cell_seed(base_seed, width, context);
            for arm in Arm::ALL {
                cells.push(run_cell(width, context, arm, seed, repeats)?);
                let rss = peak_rss_kib()?;
                if rss >= 2 * 1_024 * 1_024 {
                    return Err(format!(
                        "peak RSS {rss} KiB exceeds the 2 GiB stop at width {width}, context {context}, arm {}",
                        arm.name()
                    ));
                }
            }
        }
    }
    let negative_controls = run_negative_controls(widths[0].max(64), base_seed ^ 0x5a17)?;
    let peak_rss_kib = peak_rss_kib()?;
    if peak_rss_kib >= 2 * 1_024 * 1_024 {
        return Err(format!(
            "final peak RSS {peak_rss_kib} KiB exceeds the 2 GiB stop"
        ));
    }
    Ok(ScaleServingSummary {
        schema: SCHEMA,
        substrate: "synthetic-integer-fixed-point",
        qualification: "cpu-development-preflight-only; gpu-required-for-model-qualification",
        claim_boundary: "exact export/load/CPU-serving parity and cost at registered widths and contexts; no training, corpus, quality or GPU-memory claim",
        widths: widths.to_vec(),
        contexts: contexts.to_vec(),
        arms: Arm::ALL.into_iter().map(Arm::name).collect(),
        vocab: VOCAB,
        mlp: MLP,
        layers: LAYERS,
        kept_cells: KEPT_CELLS,
        readout_mode: "whole_table_nearest_l1",
        repeats,
        cpu_model: cpu_model(),
        logical_cpus: std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1),
        cells,
        negative_controls,
        elapsed_ms: started.elapsed().as_secs_f64() * 1_000.0,
        peak_rss_kib,
    })
}

fn run_cell(
    width: usize,
    context: usize,
    arm: Arm,
    seed: u64,
    repeats: usize,
) -> Result<CellReading, String> {
    let started = Instant::now();
    let artifact = build_artifact(width, context, arm, seed)?;
    let repeated_artifact = build_artifact(width, context, arm, seed)?;
    if artifact != repeated_artifact {
        return Err(format!(
            "same-seed artifact drift at width {width}, context {context}, arm {}",
            arm.name()
        ));
    }
    let artifact_sha256 = uor_r4_discrete::corpus::sha256(&artifact);
    let tokens = token_sequence(context);
    let reply_start = (arm == Arm::Full).then_some(1usize);
    let mut retained_parity = None;
    let mut repeat_decision_sha256s = Vec::with_capacity(repeats);
    let mut table_bytes = None;
    let mut timed = None::<(f64, f64)>;
    for repeat in 0..repeats {
        let report = audit_served_forward(
            &artifact,
            &ServedForwardAuditConfig {
                tokens: tokens.clone(),
                token_source: None,
                expected_artifact_sha256: Some(artifact_sha256.clone()),
                reply_start,
                measure_timing: repeat + 1 == repeats,
            },
        )
        .map_err(|error| {
            format!(
                "width {width}, context {context}, arm {}, repeat {repeat}: {error}",
                arm.name()
            )
        })?;
        let parity = ParityReading::from_audit(&report);
        if !parity.exact() {
            return Err(format!(
                "CPU/serving parity failed at width {width}, context {context}, arm {}, repeat {repeat}",
                arm.name()
            ));
        }
        if parity.positions_checked != context
            || parity.query_coordinates_compared != (context * width) as u64
            || parity.token_decisions_compared != context as u64
            || parity.kept_cells_compared != (context * KEPT_CELLS) as u64
        {
            return Err(format!(
                "parity coverage differs at width {width}, context {context}, arm {}, repeat {repeat}",
                arm.name()
            ));
        }
        validate_manifest(&report, width, context, arm, repeat)?;
        let current_table_bytes = (
            report.manifest.serialized_table_payload_bytes,
            report.manifest.decoded_resident_table_bytes,
        );
        if let Some(first) = table_bytes {
            if first != current_table_bytes {
                return Err(format!(
                    "table-byte accounting drifted across repeats at width {width}, context {context}, arm {}",
                    arm.name()
                ));
            }
        } else {
            table_bytes = Some(current_table_bytes);
        }
        if let Some(first) = &retained_parity {
            if first != &parity {
                return Err(format!(
                    "decision or parity drift across repeats at width {width}, context {context}, arm {}",
                    arm.name()
                ));
            }
        } else {
            retained_parity = Some(ParityReading {
                positions_checked: parity.positions_checked,
                query_coordinates_compared: parity.query_coordinates_compared,
                query_coordinate_mismatches: parity.query_coordinate_mismatches,
                token_decisions_compared: parity.token_decisions_compared,
                token_mismatches: parity.token_mismatches,
                kept_cells_compared: parity.kept_cells_compared,
                kept_cell_mismatches: parity.kept_cell_mismatches,
                cpu_decision_sha256: parity.cpu_decision_sha256.clone(),
                serving_decision_sha256: parity.serving_decision_sha256.clone(),
            });
        }
        repeat_decision_sha256s.push(parity.cpu_decision_sha256.clone());
        if let Some(timing) = report.timing {
            timed = Some((
                timing.cpu_oracle_tokens_per_second,
                timing.serving_tokens_per_second,
            ));
        }
    }
    let parity = retained_parity.ok_or_else(|| "no repeat was retained".to_owned())?;
    let (serialized_table_payload_bytes, decoded_resident_table_bytes) =
        table_bytes.ok_or_else(|| "no table-byte reading was retained".to_owned())?;
    let (cpu_oracle_tokens_per_second, serving_tokens_per_second) =
        timed.ok_or_else(|| "timed repeat did not produce timing".to_owned())?;
    Ok(CellReading {
        width,
        context,
        arm: arm.name(),
        seed,
        hamming_heads: (arm == Arm::Full).then_some(hamming_heads(width)),
        artifact_sha256,
        artifact_bytes: artifact.len() as u64,
        serialized_table_payload_bytes,
        decoded_resident_table_bytes,
        repeats,
        parity,
        repeat_decision_sha256s,
        cpu_oracle_tokens_per_second,
        serving_tokens_per_second,
        audit_elapsed_ms: started.elapsed().as_secs_f64() * 1_000.0,
        peak_rss_kib: peak_rss_kib()?,
    })
}

fn validate_manifest(
    report: &ServedForwardAuditReport,
    width: usize,
    context: usize,
    arm: Arm,
    repeat: usize,
) -> Result<(), String> {
    let shape = &report.manifest.shape;
    if shape.width != width
        || shape.context != context
        || shape.vocab != VOCAB
        || shape.mlp != MLP
        || shape.layers != LAYERS
        || shape.kept_cells != KEPT_CELLS
        || report.manifest.readout.mode != "whole_table_nearest_l1"
    {
        return Err(format!(
            "artifact shape differs at width {width}, context {context}, arm {}, repeat {repeat}",
            arm.name()
        ));
    }
    match arm {
        Arm::Projection => {
            if report.manifest.hamming.is_some() || report.manifest.memory.is_some() {
                return Err(format!(
                    "projection artifact unexpectedly carries a side table at width {width}, context {context}, repeat {repeat}"
                ));
            }
        }
        Arm::Full => {
            let hamming = report.manifest.hamming.as_ref().ok_or_else(|| {
                format!(
                    "full artifact lost its Hamming tables at width {width}, context {context}, repeat {repeat}"
                )
            })?;
            let mut depths = hamming
                .reads
                .iter()
                .map(|read| read.depth)
                .collect::<Vec<_>>();
            depths.sort_unstable();
            if hamming.heads != hamming_heads(width) || depths != [1, LAYERS] {
                return Err(format!(
                    "full artifact Hamming shape differs at width {width}, context {context}, repeat {repeat}"
                ));
            }
            let memory = report.manifest.memory.as_ref().ok_or_else(|| {
                format!(
                    "full artifact lost its memory tables at width {width}, context {context}, repeat {repeat}"
                )
            })?;
            if memory.opening_rows == 0 || memory.reply_rows == 0 || memory.key_window != context {
                return Err(format!(
                    "full artifact memory shape differs at width {width}, context {context}, repeat {repeat}"
                ));
            }
        }
    }
    Ok(())
}

fn build_artifact(width: usize, context: usize, arm: Arm, seed: u64) -> Result<Vec<u8>, String> {
    let mut snapshot = synthetic_fbs_trunk_with(
        seed,
        FbsSynth {
            vocab: VOCAB,
            d: width,
            mlp: MLP,
            layers: LAYERS,
            context,
            kept: KEPT_CELLS,
            gain_lo: 0.75,
            gain_span: 1.5,
            table: 0.25,
            decay_step: 0.0625,
        },
    );
    snapshot.readout_nearest = true;
    if arm == Arm::Full {
        snapshot.hamming = Some(synthetic_hamming(
            width,
            LAYERS,
            hamming_heads(width),
            seed + 1,
        ));
        let tokens = token_sequence(context);
        let mut reply = (1..context)
            .map(|step| FbsMemoryEntry {
                key: vec![tokens[0], MEMORY_PAD, step as u32, MEMORY_PAD, tokens[step]],
                value: ((step * 97 + 11) % VOCAB) as u32,
            })
            .collect::<Vec<_>>();
        reply.sort_by(|left, right| left.key.cmp(&right.key));
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 1,
            key_window: context,
            opening: vec![FbsMemoryEntry {
                key: vec![tokens[0]],
                value: ((tokens[0] + 1) % VOCAB as u32),
            }],
            opening_weight: vec![1.0; width],
            reply: Some(FbsReplyTable {
                turn: 1,
                entries: reply,
                weight: vec![1.0; width],
            }),
        });
    }
    export_fbs_trunk_uorlut01(&snapshot).map_err(|error| {
        format!(
            "width {width}, context {context}, arm {}: {error}",
            arm.name()
        )
    })
}

fn run_negative_controls(width: usize, seed: u64) -> Result<Vec<NegativeControl>, String> {
    let context = 8usize;
    let artifact = build_artifact(width, context, Arm::Projection, seed)?;
    let tokens = token_sequence(context);
    let mut controls = Vec::new();

    let truncated = audit_served_forward(
        &artifact[..artifact.len().min(32)],
        &ServedForwardAuditConfig::new(tokens.clone()),
    );
    controls.push(control("truncated_artifact", truncated.err()));

    let mut out_of_vocabulary = tokens.clone();
    out_of_vocabulary[0] = VOCAB as u32;
    let invalid_token =
        audit_served_forward(&artifact, &ServedForwardAuditConfig::new(out_of_vocabulary));
    controls.push(control("token_outside_vocabulary", invalid_token.err()));

    let overlong = audit_served_forward(
        &artifact,
        &ServedForwardAuditConfig::new(token_sequence(context + 1)),
    );
    controls.push(control("token_sequence_over_context", overlong.err()));

    let over_width = build_artifact(2_048, context, Arm::Projection, seed ^ 0x2048);
    controls.push(control("width_above_writer_cap", over_width.err()));

    if controls.iter().any(|control| !control.refused) {
        return Err("one or more scale-serving negative controls did not fail closed".to_owned());
    }
    Ok(controls)
}

fn control(name: &'static str, error: Option<impl ToString>) -> NegativeControl {
    match error {
        Some(error) => NegativeControl {
            name,
            refused: true,
            message: error.to_string(),
        },
        None => NegativeControl {
            name,
            refused: false,
            message: "unexpectedly accepted".to_owned(),
        },
    }
}

fn token_sequence(context: usize) -> Vec<u32> {
    (0..context)
        .map(|index| ((index * 73 + 19) % VOCAB) as u32)
        .collect()
}

fn hamming_heads(width: usize) -> usize {
    (width / 64).max(2)
}

fn cell_seed(base: u64, width: usize, context: usize) -> u64 {
    base ^ ((width as u64) << 17) ^ context as u64
}

fn cpu_model() -> String {
    let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") else {
        return "unknown".to_owned();
    };
    cpuinfo
        .lines()
        .find_map(|line| {
            line.strip_prefix("model name")
                .and_then(|rest| rest.split_once(':'))
        })
        .map(|(_, value)| value.trim().to_owned())
        .unwrap_or_else(|| "unknown".to_owned())
}

fn peak_rss_kib() -> Result<u64, String> {
    let status = std::fs::read_to_string("/proc/self/status")
        .map_err(|error| format!("cannot read /proc/self/status: {error}"))?;
    parse_peak_rss_kib(&status)
}

fn parse_peak_rss_kib(status: &str) -> Result<u64, String> {
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| "VmHWM is missing, malformed or zero".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_grid_and_head_widths_are_fixed() {
        assert_eq!(WIDTHS, [256, 512, 1_024]);
        assert_eq!(CONTEXTS, [16, 64, 192]);
        assert_eq!(hamming_heads(256), 4);
        assert_eq!(hamming_heads(512), 8);
        assert_eq!(hamming_heads(1_024), 16);
    }

    #[test]
    fn small_cell_exports_repeats_and_matches_serving() {
        let summary = run_grid(&[64], &[8], 2, 190).unwrap();
        assert_eq!(summary.cells.len(), 2);
        assert!(summary.cells.iter().all(|cell| {
            cell.parity.exact()
                && cell.parity.query_coordinates_compared == 8 * 64
                && cell.parity.kept_cells_compared == 8 * KEPT_CELLS as u64
                && cell.repeat_decision_sha256s.len() == 2
                && cell
                    .repeat_decision_sha256s
                    .windows(2)
                    .all(|pair| pair[0] == pair[1])
        }));
        assert_eq!(summary.negative_controls.len(), 4);
        assert!(summary
            .negative_controls
            .iter()
            .all(|control| control.refused));
    }

    #[test]
    fn peak_rss_requires_a_positive_vmhwm() {
        assert_eq!(
            parse_peak_rss_kib("Name:\ttest\nVmHWM:\t  1234 kB\n").unwrap(),
            1_234
        );
        for invalid in ["Name:\ttest\n", "VmHWM:\t0 kB\n", "VmHWM:\tmany kB\n"] {
            assert!(parse_peak_rss_kib(invalid).is_err());
        }
    }
}
