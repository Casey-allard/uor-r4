use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;
use uor_r4_core::report_output;
use uor_r4_cpu_train::{
    validate_corpus_manifest, CpuConversationSample, CpuFbsForward, CpuFbsHead, CpuMaskedCorpus,
    RankMarginSample,
};
use uor_r4_discrete::normalization_mechanism::orbit_root_coords;
use uor_r4_integer::stack::{read_hamming_uorlut01, read_memory_uorlut01, IntegerStackModel};

const EXPECTED_RESPONSE_ROWS: usize = 616;

struct Config {
    raw: PathBuf,
    anchored: PathBuf,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    out: PathBuf,
    expected_raw_sha256: String,
    expected_anchored_sha256: String,
    expected_tokens_sha256: String,
    expected_mask_sha256: String,
    expected_manifest_sha256: String,
}

struct Arm {
    name: &'static str,
    forward: CpuFbsForward,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RowOutcome {
    query: Vec<i32>,
    token: u32,
    kept_cells: Vec<u32>,
}

#[derive(serde::Serialize)]
struct ArmReading {
    samples: usize,
    target_admitted: u64,
    top1_correct: u64,
    top8_correct: u64,
    target_port_distance_sum: u64,
    target_port_abs_sum: u64,
    query_coordinates: u64,
    query_changed_rows: u64,
    token_changed_rows: u64,
    kept_cell_changed_rows: u64,
    query_sha256: String,
    token_sha256: String,
    kept_cell_sha256: String,
    serving_parity: Option<ServingParity>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
struct ServingParity {
    query_coordinate_mismatches: u64,
    token_mismatches: u64,
    kept_cell_mismatches: u64,
}

impl ServingParity {
    fn exact(&self) -> bool {
        self.query_coordinate_mismatches == 0
            && self.token_mismatches == 0
            && self.kept_cell_mismatches == 0
    }
}

fn main() {
    match run() {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("the report serializes")
        ),
        Err(error) => {
            eprintln!("cpu-table-anchor-isolation: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-table-anchor-isolation RAW_ARTIFACT ANCHORED_ARTIFACT TOKENS MASK MANIFEST OUT \
     --expected-raw-sha256 HEX --expected-anchored-sha256 HEX \
     --expected-tokens-sha256 HEX --expected-mask-sha256 HEX \
     --expected-manifest-sha256 HEX"
        .to_owned()
}

fn parse_args() -> Result<Config, String> {
    let mut values = std::env::args().skip(1);
    let mut path = || values.next().map(PathBuf::from).ok_or_else(usage);
    let mut config = Config {
        raw: path()?,
        anchored: path()?,
        tokens: path()?,
        mask: path()?,
        manifest: path()?,
        out: path()?,
        expected_raw_sha256: String::new(),
        expected_anchored_sha256: String::new(),
        expected_tokens_sha256: String::new(),
        expected_mask_sha256: String::new(),
        expected_manifest_sha256: String::new(),
    };
    while let Some(flag) = values.next() {
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--expected-raw-sha256" => config.expected_raw_sha256 = hex(&value)?,
            "--expected-anchored-sha256" => config.expected_anchored_sha256 = hex(&value)?,
            "--expected-tokens-sha256" => config.expected_tokens_sha256 = hex(&value)?,
            "--expected-mask-sha256" => config.expected_mask_sha256 = hex(&value)?,
            "--expected-manifest-sha256" => config.expected_manifest_sha256 = hex(&value)?,
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    for (name, value) in [
        ("raw", &config.expected_raw_sha256),
        ("anchored", &config.expected_anchored_sha256),
        ("tokens", &config.expected_tokens_sha256),
        ("mask", &config.expected_mask_sha256),
        ("manifest", &config.expected_manifest_sha256),
    ] {
        if value.is_empty() {
            return Err(format!("--expected-{name}-sha256 is required"));
        }
    }
    Ok(config)
}

fn hex(value: &str) -> Result<String, String> {
    let value = value.to_ascii_lowercase();
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(value)
    } else {
        Err(format!("not a SHA-256: {value}"))
    }
}

fn run() -> Result<serde_json::Value, String> {
    let config = parse_args()?;
    let started = Instant::now();
    report_output::claim(&config.out).map_err(|error| {
        format!(
            "claim exclusive report directory {}: {error}",
            config.out.display()
        )
    })?;
    let result = run_claimed(&config, started);
    if let Err(error) = &result {
        let failure = serde_json::json!({
            "schema": "uor-r4.cpu-table-anchor-isolation-failure/1",
            "status": "INSTRUMENT_FAULT",
            "error": error,
            "inputs": inputs_json(&config),
            "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
            "peak_rss_kib": peak_rss_kib(),
        });
        write_new(
            &config.out.join("failed-attempt.json"),
            &serde_json::to_vec_pretty(&failure).map_err(|error| error.to_string())?,
        )?;
    }
    report_output::seal(&config.out)
        .map_err(|error| format!("seal report directory {}: {error}", config.out.display()))?;
    report_output::verify(&config.out)
        .map_err(|error| format!("verify report directory {}: {error}", config.out.display()))?;
    result
}

fn run_claimed(config: &Config, started: Instant) -> Result<serde_json::Value, String> {
    let raw_bytes = std::fs::read(&config.raw).map_err(|error| error.to_string())?;
    let anchored_bytes = std::fs::read(&config.anchored).map_err(|error| error.to_string())?;
    let tokens = std::fs::read(&config.tokens).map_err(|error| error.to_string())?;
    let mask = std::fs::read(&config.mask).map_err(|error| error.to_string())?;
    let manifest = std::fs::read(&config.manifest).map_err(|error| error.to_string())?;
    for (name, bytes, expected) in [
        ("raw", raw_bytes.as_slice(), &config.expected_raw_sha256),
        (
            "anchored",
            anchored_bytes.as_slice(),
            &config.expected_anchored_sha256,
        ),
        ("tokens", tokens.as_slice(), &config.expected_tokens_sha256),
        ("mask", mask.as_slice(), &config.expected_mask_sha256),
        (
            "manifest",
            manifest.as_slice(),
            &config.expected_manifest_sha256,
        ),
    ] {
        let actual = uor_r4_discrete::corpus::sha256(bytes);
        if &actual != expected {
            return Err(format!("{name} SHA-256 differs: got {actual}"));
        }
    }

    let raw_model = IntegerStackModel::parse(&raw_bytes).map_err(|error| error.to_string())?;
    let anchored_model =
        IntegerStackModel::parse(&anchored_bytes).map_err(|error| error.to_string())?;
    let raw_shape = raw_model.shape();
    let anchored_shape = anchored_model.shape();
    if raw_shape.vocab != anchored_shape.vocab
        || raw_shape.width != anchored_shape.width
        || raw_shape.context != anchored_shape.context
    {
        return Err("raw and anchored artifact shapes differ".to_owned());
    }
    for (name, bytes) in [("raw", &raw_bytes), ("anchored", &anchored_bytes)] {
        if read_hamming_uorlut01(bytes)
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Err(format!("{name} artifact has a Hamming read"));
        }
        if read_memory_uorlut01(bytes)
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Err(format!("{name} artifact has turn memory"));
        }
    }
    let identity =
        validate_corpus_manifest(&manifest, &tokens, &mask).map_err(|error| error.to_string())?;
    if identity.vocab != raw_shape.vocab || identity.bos != 0 || identity.split != "dev" {
        return Err("panel vocabulary, BOS or split differs".to_owned());
    }
    let corpus = CpuMaskedCorpus::from_bytes(
        &tokens,
        &mask,
        raw_shape.vocab,
        raw_shape.context,
        identity.bos,
    )
    .map_err(|error| error.to_string())?;
    let samples = response_samples(
        &corpus
            .conversation_examples()
            .map_err(|error| error.to_string())?,
    )?;
    if samples.len() != EXPECTED_RESPONSE_ROWS {
        return Err(format!(
            "panel has {} response rows; the registered panel has {EXPECTED_RESPONSE_ROWS}",
            samples.len()
        ));
    }

    let raw_forward =
        CpuFbsForward::from_artifact(&raw_bytes).map_err(|error| error.to_string())?;
    let raw_table = raw_forward.shared_table().to_vec();
    let anchored_forward =
        CpuFbsForward::from_artifact(&anchored_bytes).map_err(|error| error.to_string())?;
    let anchored_table = anchored_forward.shared_table().to_vec();
    let expected_anchor = anchor_table(&raw_forward);
    if anchored_table != expected_anchor {
        return Err("anchored artifact table is not the registered root projection".to_owned());
    }

    let arms = build_arms(&raw_bytes, &raw_table, &anchored_table)?;
    let raw_rows = evaluate(&arms[0].forward, &samples)?;
    let serving_parity = serving_parity(&raw_model, &samples, &raw_rows)?;
    if !serving_parity.exact() {
        return Err("raw/raw arm differs from the integer serving session".to_owned());
    }

    let mut readings = Vec::new();
    for arm in &arms {
        let rows = evaluate(&arm.forward, &samples)?;
        let repeat = evaluate(&arm.forward, &samples)?;
        if rows
            .iter()
            .map(|row| (&row.query, row.token, &row.kept_cells))
            .ne(repeat
                .iter()
                .map(|row| (&row.query, row.token, &row.kept_cells)))
        {
            return Err(format!("{} arm is nondeterministic", arm.name));
        }
        readings.push((
            arm.name,
            summarize(&arm.forward, &samples, &rows, &raw_rows)?,
            rows,
        ));
    }

    let combined = readings
        .iter()
        .find(|(name, _, _)| *name == "combined")
        .map(|(_, _, rows)| rows)
        .ok_or_else(|| "combined arm is absent".to_owned())?;
    let anchored_rows = evaluate(&anchored_forward, &samples)?;
    if combined
        .iter()
        .map(|row| (&row.query, row.token, &row.kept_cells))
        .ne(anchored_rows
            .iter()
            .map(|row| (&row.query, row.token, &row.kept_cells)))
    {
        return Err("combined hybrid differs from the anchored artifact forward".to_owned());
    }

    let report = serde_json::json!({
        "schema": "uor-r4.cpu-table-anchor-isolation/1",
        "status": "COMPLETE",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "inputs": inputs_json(config),
        "execution": {
            "samples": samples.len(),
            "threads": rayon::current_num_threads(),
            "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
            "peak_rss_kib": peak_rss_kib(),
        },
        "raw_serving": serving_parity,
        "arms": readings.into_iter().map(|(name, reading, _)| (name, reading)).collect::<std::collections::BTreeMap<_, _>>(),
    });
    write_new(
        &config.out.join("report.json"),
        &serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )?;
    Ok(report)
}

fn build_arms(
    raw_bytes: &[u8],
    raw_table: &[i16],
    anchored_table: &[i16],
) -> Result<Vec<Arm>, String> {
    [
        ("raw_raw", raw_table, raw_table),
        ("input_anchored", anchored_table, raw_table),
        ("writer_anchored", raw_table, anchored_table),
        ("combined", anchored_table, anchored_table),
    ]
    .into_iter()
    .map(|(name, input_table, writer_table)| {
        Ok(Arm {
            name,
            forward: CpuFbsForward::from_artifact_with_tables(raw_bytes, input_table, writer_table)
                .map_err(|error| error.to_string())?,
        })
    })
    .collect()
}

fn evaluate(
    forward: &CpuFbsForward,
    samples: &[RankMarginSample],
) -> Result<Vec<RowOutcome>, String> {
    samples
        .par_iter()
        .map_init(
            || forward.clone(),
            |forward, sample| {
                forward.reset();
                let mut step = None;
                for &token in &sample.input {
                    step = Some(
                        forward
                            .step(token, CpuFbsHead::Writer)
                            .map_err(|error| error.to_string())?,
                    );
                }
                let step = step.ok_or_else(|| "sample input is empty".to_owned())?;
                Ok(RowOutcome {
                    query: step.query_q12,
                    token: step.token,
                    kept_cells: step.kept_cells,
                })
            },
        )
        .collect()
}

fn summarize(
    forward: &CpuFbsForward,
    samples: &[RankMarginSample],
    rows: &[RowOutcome],
    raw: &[RowOutcome],
) -> Result<ArmReading, String> {
    let width = forward.width();
    let roots = std::array::from_fn::<_, 16, _>(|root| {
        orbit_root_coords(root as u8)
            .expect("the 16 writer roots are valid")
            .map(|value| value << 12)
    });
    let mut reading = ArmReading {
        samples: samples.len(),
        target_admitted: 0,
        top1_correct: 0,
        top8_correct: 0,
        target_port_distance_sum: 0,
        target_port_abs_sum: 0,
        query_coordinates: 0,
        query_changed_rows: 0,
        token_changed_rows: 0,
        kept_cell_changed_rows: 0,
        query_sha256: String::new(),
        token_sha256: String::new(),
        kept_cell_sha256: String::new(),
        serving_parity: None,
    };
    let mut query_bytes = Vec::new();
    let mut token_bytes = Vec::new();
    let mut cell_bytes = Vec::new();
    for ((sample, row), raw_row) in samples.iter().zip(rows).zip(raw) {
        let diagnostic = forward
            .writer_token_diagnostic(&row.query, sample.target)
            .map_err(|error| error.to_string())?;
        let cell = sample.target & 0xff;
        let roots_for_target = [cell >> 4, cell & 15];
        for (start, root) in [
            (0usize, roots_for_target[0]),
            (width - 8, roots_for_target[1]),
        ] {
            for (coordinate, expected) in roots[root as usize].iter().enumerate() {
                let value = row.query[start + coordinate];
                reading.target_port_distance_sum += u64::from(value.abs_diff(*expected));
                reading.target_port_abs_sum += u64::from(value.unsigned_abs());
            }
        }
        reading.target_admitted += u64::from(diagnostic.admitted);
        reading.top1_correct += u64::from(diagnostic.chosen_token == diagnostic.target_token);
        reading.top8_correct += u64::from(diagnostic.top8);
        reading.query_coordinates += width as u64;
        reading.query_changed_rows += u64::from(row.query != raw_row.query);
        reading.token_changed_rows += u64::from(row.token != raw_row.token);
        reading.kept_cell_changed_rows += u64::from(row.kept_cells != raw_row.kept_cells);
        for value in &row.query {
            query_bytes.extend_from_slice(&value.to_le_bytes());
        }
        token_bytes.extend_from_slice(&row.token.to_le_bytes());
        for value in &row.kept_cells {
            cell_bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    reading.query_sha256 = uor_r4_discrete::corpus::sha256(&query_bytes);
    reading.token_sha256 = uor_r4_discrete::corpus::sha256(&token_bytes);
    reading.kept_cell_sha256 = uor_r4_discrete::corpus::sha256(&cell_bytes);
    Ok(reading)
}

fn serving_parity(
    model: &IntegerStackModel,
    samples: &[RankMarginSample],
    rows: &[RowOutcome],
) -> Result<ServingParity, String> {
    let readings: Result<Vec<_>, String> = samples
        .par_iter()
        .zip(rows)
        .map_init(
            || model.session(),
            |session, (sample, row)| {
                session.reset();
                let mut served = None;
                for &token in &sample.input {
                    served = Some(
                        session
                            .step_native(token)
                            .map_err(|error| error.to_string())?,
                    );
                }
                let served = served.ok_or_else(|| "sample input is empty".to_owned())?;
                Ok(ServingParity {
                    query_coordinate_mismatches: served
                        .query_q12
                        .iter()
                        .zip(&row.query)
                        .filter(|(left, right)| left != right)
                        .count() as u64,
                    token_mismatches: u64::from(served.token != row.token),
                    kept_cell_mismatches: u64::from(served.kept_cells != row.kept_cells),
                })
            },
        )
        .collect();
    let mut parity = ServingParity::default();
    for reading in readings? {
        parity.query_coordinate_mismatches += reading.query_coordinate_mismatches;
        parity.token_mismatches += reading.token_mismatches;
        parity.kept_cell_mismatches += reading.kept_cell_mismatches;
    }
    Ok(parity)
}

fn anchor_table(forward: &CpuFbsForward) -> Vec<i16> {
    let width = forward.width();
    let roots = std::array::from_fn::<_, 16, _>(|root| {
        orbit_root_coords(root as u8)
            .expect("the 16 writer roots are valid")
            .map(|value| value << 12)
    });
    let mut table = forward.shared_table().to_vec();
    for token in 0..forward.vocabulary() {
        let cell = token & 0xff;
        let row = &mut table[token * width..(token + 1) * width];
        for (start, root) in [(0, cell >> 4), (width - 8, cell & 15)] {
            for (coordinate, root_coordinate) in roots[root].iter().enumerate() {
                row[start + coordinate] = *root_coordinate as i16;
            }
        }
    }
    table
}

fn response_samples(examples: &[CpuConversationSample]) -> Result<Vec<RankMarginSample>, String> {
    let mut samples = Vec::new();
    for example in examples {
        let tokens = example
            .prompt
            .iter()
            .chain(&example.reply)
            .copied()
            .collect::<Vec<_>>();
        for target_index in example.prompt.len()..tokens.len() {
            if target_index == 0 {
                continue;
            }
            samples.push(RankMarginSample {
                input: tokens[..target_index].to_vec(),
                target: tokens[target_index],
            });
        }
    }
    if samples.is_empty() {
        return Err("panel has no response-token targets".to_owned());
    }
    Ok(samples)
}

fn inputs_json(config: &Config) -> serde_json::Value {
    serde_json::json!({
        "raw_artifact": config.raw,
        "raw_artifact_sha256": config.expected_raw_sha256,
        "anchored_artifact": config.anchored,
        "anchored_artifact_sha256": config.expected_anchored_sha256,
        "tokens": config.tokens,
        "tokens_sha256": config.expected_tokens_sha256,
        "mask": config.mask,
        "mask_sha256": config.expected_mask_sha256,
        "manifest": config.manifest,
        "manifest_sha256": config.expected_manifest_sha256,
    })
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Err(format!("{} already exists", path.display()));
    }
    std::fs::write(path, bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .unwrap_or_default()
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmHWM:")
                .and_then(|value| value.split_whitespace().next())
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

    fn synthetic_artifact() -> Vec<u8> {
        export_fbs_trunk_uorlut01(&synthetic_fbs_trunk_with(
            193,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 64,
                layers: 2,
                context: 16,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        ))
        .unwrap()
    }

    fn sample(input: Vec<u32>, target: u32) -> RankMarginSample {
        RankMarginSample { input, target }
    }

    #[test]
    fn four_arms_bind_input_and_writer_tables_independently() {
        let artifact = synthetic_artifact();
        let raw = CpuFbsForward::from_artifact(&artifact).unwrap();
        let raw_table = raw.shared_table().to_vec();
        let anchored_table = anchor_table(&raw);
        let arms = build_arms(&artifact, &raw_table, &anchored_table).unwrap();
        assert_eq!(
            arms.iter().map(|arm| arm.name).collect::<Vec<_>>(),
            ["raw_raw", "input_anchored", "writer_anchored", "combined"]
        );

        let samples = [sample(vec![3, 5, 8, 13], 21)];
        let raw_rows = evaluate(&arms[0].forward, &samples).unwrap();
        let input_rows = evaluate(&arms[1].forward, &samples).unwrap();
        let writer_rows = evaluate(&arms[2].forward, &samples).unwrap();
        let combined_rows = evaluate(&arms[3].forward, &samples).unwrap();

        assert_ne!(raw_rows[0].query, input_rows[0].query);
        assert_eq!(raw_rows[0].query, writer_rows[0].query);
        assert_eq!(input_rows[0].query, combined_rows[0].query);
    }

    #[test]
    fn anchor_table_changes_only_the_two_registered_port_blocks() {
        let artifact = synthetic_artifact();
        let raw = CpuFbsForward::from_artifact(&artifact).unwrap();
        let raw_table = raw.shared_table().to_vec();
        let anchored = anchor_table(&raw);
        assert_eq!(raw_table.len(), anchored.len());

        let width = raw.width();
        let mut changed = 0;
        for token in 0..raw.vocabulary() {
            let row = token * width;
            for coordinate in 0..width {
                let differs = raw_table[row + coordinate] != anchored[row + coordinate];
                if !(0..8).contains(&coordinate) && !(width - 8..width).contains(&coordinate) {
                    assert!(!differs);
                }
                changed += usize::from(differs);
            }
        }
        assert!(changed > 0);
    }

    #[test]
    fn response_samples_cover_each_response_after_its_causal_prefix() {
        let rows = response_samples(&[CpuConversationSample {
            document_ordinal: 7,
            prompt: vec![10, 11],
            reply: vec![12, 13, 14],
        }])
        .unwrap();
        assert_eq!(
            rows,
            [
                sample(vec![10, 11], 12),
                sample(vec![10, 11, 12], 13),
                sample(vec![10, 11, 12, 13], 14),
            ]
        );
    }

    #[test]
    fn raw_cpu_forward_matches_serving_at_each_sample_final_decision() {
        let artifact = synthetic_artifact();
        let model = IntegerStackModel::parse(&artifact).unwrap();
        let forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let samples = [sample(vec![3, 5], 8), sample(vec![13, 21], 34)];
        let rows = evaluate(&forward, &samples).unwrap();
        assert!(serving_parity(&model, &samples, &rows).unwrap().exact());
    }

    #[test]
    fn ordered_parallel_rows_match_one_worker() {
        let artifact = synthetic_artifact();
        let model = IntegerStackModel::parse(&artifact).unwrap();
        let forward = CpuFbsForward::from_artifact(&artifact).unwrap();
        let samples = [
            sample(vec![3, 5, 8], 13),
            sample(vec![21, 34], 55),
            sample(vec![89, 144, 233], 377),
        ];
        let read = |workers| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()
                .unwrap()
                .install(|| {
                    let rows = evaluate(&forward, &samples)?;
                    let parity = serving_parity(&model, &samples, &rows)?;
                    Ok::<_, String>((rows, parity))
                })
        };
        assert_eq!(read(1).unwrap(), read(4).unwrap());
    }

    #[test]
    fn response_samples_refuse_an_empty_panel() {
        assert!(response_samples(&[]).is_err());
    }
}
