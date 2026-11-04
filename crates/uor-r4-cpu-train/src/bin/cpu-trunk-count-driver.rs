//! Exact additive sufficient-statistic training over a public 19-table M1 trunk.
//!
//! D84 CPU development research only. The driver updates count tables on the
//! CPU, exports the complete integer trunk, and checks every held-out decision
//! against integer serving.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use serde_json::json;
use uor_r4_cpu_train::{
    validate_corpus_manifest, CpuDiscreteTrunkReader, CpuMaskedCorpus, RankMarginSample,
};
use uor_r4_discrete::trunk::{
    import_trunk_d11, pack_u32_key, SparseCountTable, TrunkTables, TrunkTrainRow,
};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

const SCHEMA: &str = "uor-r4.cpu-trunk-count-run/1";

#[derive(Clone)]
struct Config {
    artifact: PathBuf,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    out: PathBuf,
    expected_artifact_sha256: String,
    expected_tokens_sha256: String,
    expected_mask_sha256: String,
    expected_manifest_sha256: String,
    train_rows: usize,
    evaluation_stride: usize,
    max_evaluation: usize,
    seed: u64,
    workers: usize,
    select_only: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct DecisionReading {
    rows: u64,
    admitted: u64,
    top1: u64,
    top8: u64,
    query_coordinates: u64,
    query_coordinate_mismatches: u64,
    admitted_cell_mismatches: u64,
    top8_mismatches: u64,
    token_mismatches: u64,
    checksum: u64,
    row_checksums: Vec<u64>,
}

#[derive(Debug, PartialEq, Eq)]
struct DecisionOutcome {
    admitted: bool,
    top1: bool,
    top8: bool,
    query: Vec<i32>,
    top8_tokens: [u32; 8],
    token: u32,
    query_coordinate_mismatches: u64,
    admitted_cell_mismatches: u64,
    top8_mismatches: u64,
    token_mismatches: u64,
}

#[derive(PartialEq, Eq, serde::Serialize)]
struct TableReading {
    total: u64,
    distinct_keys: usize,
}

fn main() {
    let result = parse_args().and_then(|config| {
        let pool = ThreadPoolBuilder::new()
            .num_threads(config.workers)
            .build()
            .map_err(|error| format!("cannot build {}-worker pool: {error}", config.workers))?;
        pool.install(|| run(config))
    });
    match result {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("the report serializes")
        ),
        Err(error) => {
            eprintln!("cpu-trunk-count-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-trunk-count-driver ARTIFACT TOKENS MASK MANIFEST OUT \
     --expected-artifact-sha256 HEX --expected-tokens-sha256 HEX \
     --expected-mask-sha256 HEX --expected-manifest-sha256 HEX \
     [--train-rows N] [--evaluation-stride N] [--max-evaluation N] \
     [--seed N] [--workers N] [--select-only]"
        .to_owned()
}

fn parse_args() -> Result<Config, String> {
    let mut values = std::env::args().skip(1);
    let artifact = PathBuf::from(values.next().ok_or_else(usage)?);
    let tokens = PathBuf::from(values.next().ok_or_else(usage)?);
    let mask = PathBuf::from(values.next().ok_or_else(usage)?);
    let manifest = PathBuf::from(values.next().ok_or_else(usage)?);
    let out = PathBuf::from(values.next().ok_or_else(usage)?);
    let mut config = Config {
        artifact,
        tokens,
        mask,
        manifest,
        out,
        expected_artifact_sha256: String::new(),
        expected_tokens_sha256: String::new(),
        expected_mask_sha256: String::new(),
        expected_manifest_sha256: String::new(),
        train_rows: 128,
        evaluation_stride: 2,
        max_evaluation: 512,
        seed: 1,
        workers: default_workers(),
        select_only: false,
    };
    while let Some(flag) = values.next() {
        if flag == "--select-only" {
            config.select_only = true;
            continue;
        }
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--expected-artifact-sha256" => config.expected_artifact_sha256 = hex(&value)?,
            "--expected-tokens-sha256" => config.expected_tokens_sha256 = hex(&value)?,
            "--expected-mask-sha256" => config.expected_mask_sha256 = hex(&value)?,
            "--expected-manifest-sha256" => config.expected_manifest_sha256 = hex(&value)?,
            "--train-rows" => config.train_rows = number(&value)?,
            "--evaluation-stride" => config.evaluation_stride = number(&value)?,
            "--max-evaluation" => config.max_evaluation = number(&value)?,
            "--seed" => config.seed = number(&value)?,
            "--workers" => config.workers = number(&value)?,
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    for (name, value) in [
        ("artifact", &config.expected_artifact_sha256),
        ("tokens", &config.expected_tokens_sha256),
        ("mask", &config.expected_mask_sha256),
        ("manifest", &config.expected_manifest_sha256),
    ] {
        if value.is_empty() {
            return Err(format!("--expected-{name}-sha256 is required"));
        }
    }
    if config.train_rows == 0
        || config.train_rows > 4096
        || config.evaluation_stride < 2
        || config.max_evaluation == 0
        || config.max_evaluation > 16_384
        || config.workers == 0
        || config.workers > 256
    {
        return Err("training, split, evaluation or worker bounds differ".to_owned());
    }
    Ok(config)
}

fn number<T: std::str::FromStr>(value: &str) -> Result<T, String> {
    value.parse().map_err(|_| format!("not a number: {value}"))
}

fn default_workers() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
}

fn hex(value: &str) -> Result<String, String> {
    let value = value.to_ascii_lowercase();
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(value)
    } else {
        Err(format!("not a SHA-256: {value}"))
    }
}

fn run(config: Config) -> Result<serde_json::Value, String> {
    if !config.select_only && config.out.exists() {
        return Err(format!(
            "output root already exists: {}",
            config.out.display()
        ));
    }
    let started = Instant::now();
    let artifact = read_hashed(&config.artifact, &config.expected_artifact_sha256)?;
    let tokens = read_hashed(&config.tokens, &config.expected_tokens_sha256)?;
    let mask = read_hashed(&config.mask, &config.expected_mask_sha256)?;
    let manifest = read_hashed(&config.manifest, &config.expected_manifest_sha256)?;
    let identity =
        validate_corpus_manifest(&manifest, &tokens, &mask).map_err(|error| error.to_string())?;
    let serving = IntegerDiscreteModel::from_bytes(&artifact).map_err(|error| error.to_string())?;
    let shape = serving.shape();
    let vocabulary = shape.vocabulary as usize;
    let context = shape.context as usize;
    if identity.vocab != vocabulary || identity.bos != 0 {
        return Err("corpus vocabulary or BOS differs from the trunk".to_owned());
    }
    identity
        .validate_context(context)
        .map_err(|error| error.to_string())?;
    drop(serving);
    let corpus = CpuMaskedCorpus::from_bytes(&tokens, &mask, vocabulary, context, 0)
        .map_err(|error| error.to_string())?;
    let split = corpus
        .split(config.evaluation_stride, config.max_evaluation)
        .map_err(|error| error.to_string())?;
    if split.training.len() < config.train_rows {
        return Err(format!(
            "training split has {} rows, fewer than requested {}",
            split.training.len(),
            config.train_rows
        ));
    }
    if config.select_only {
        return Ok(json!({
            "schema": "uor-r4.cpu-trunk-count-selection/1",
            "training_documents": split.training_documents,
            "evaluation_documents": split.evaluation_documents,
            "training_targets": split.training_targets,
            "evaluation_targets": split.evaluation_targets,
            "selected_training_rows": config.train_rows,
            "evaluation_stride": config.evaluation_stride,
            "max_evaluation": config.max_evaluation,
        }));
    }

    let reader =
        CpuDiscreteTrunkReader::from_bytes(&artifact).map_err(|error| error.to_string())?;
    let training = split
        .training
        .get(..config.train_rows)
        .ok_or_else(|| "training split is shorter than requested".to_owned())?
        .par_iter()
        .enumerate()
        .map(|(index, sample)| {
            reader
                .training_row(
                    &sample.input,
                    sample.target,
                    config.seed.wrapping_add(index as u64 + 1),
                )
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let shuffled_targets = shuffled_targets(
        &training
            .iter()
            .map(|row| u32::from(row.target_token))
            .collect::<Vec<_>>(),
        config.seed,
    )?;
    let shuffled = training
        .iter()
        .zip(&shuffled_targets)
        .map(|(row, &target)| {
            Ok(TrunkTrainRow {
                target_token: u16::try_from(target)
                    .map_err(|_| "shuffled target exceeds u16".to_owned())?,
                ..*row
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    std::fs::create_dir_all(&config.out).map_err(|error| error.to_string())?;
    let exact_artifact = reader
        .add_training_rows_consuming(&training)
        .map_err(|error| error.to_string())?;
    let exact_sha256 = uor_r4_discrete::corpus::sha256(&exact_artifact);
    let exact_bytes = exact_artifact.len();
    std::fs::write(config.out.join("exact.uorlut"), &exact_artifact)
        .map_err(|error| error.to_string())?;
    drop(exact_artifact);

    let shuffled_reader =
        CpuDiscreteTrunkReader::from_bytes(&artifact).map_err(|error| error.to_string())?;
    let shuffled_artifact = shuffled_reader
        .add_training_rows_consuming(&shuffled)
        .map_err(|error| error.to_string())?;
    let shuffled_sha256 = uor_r4_discrete::corpus::sha256(&shuffled_artifact);
    let shuffled_bytes = shuffled_artifact.len();
    std::fs::write(config.out.join("shuffled.uorlut"), &shuffled_artifact)
        .map_err(|error| error.to_string())?;
    drop(shuffled_artifact);
    if exact_sha256 == shuffled_sha256 {
        return Err("exact and shuffled count training produced the same artifact".to_owned());
    }

    let source_semantics = semantics(&artifact)?;
    let exact_artifact =
        std::fs::read(config.out.join("exact.uorlut")).map_err(|error| error.to_string())?;
    if semantics(&exact_artifact)? != source_semantics {
        return Err("count training changed a non-count trunk section".to_owned());
    }
    if trunk_tables(&exact_artifact)? != expected_count_tables(&artifact, &training)? {
        return Err("decoded count tables differ from the independent tally".to_owned());
    }
    let source_tables = table_reading(&artifact)?;
    let exact_tables = table_reading(&exact_artifact)?;
    drop(exact_artifact);
    let shuffled_artifact =
        std::fs::read(config.out.join("shuffled.uorlut")).map_err(|error| error.to_string())?;
    if semantics(&shuffled_artifact)? != source_semantics {
        return Err("shuffled count training changed a non-count trunk section".to_owned());
    }
    if trunk_tables(&shuffled_artifact)? != expected_count_tables(&artifact, &shuffled)? {
        return Err("shuffled count tables differ from the independent tally".to_owned());
    }
    let shuffled_tables = table_reading(&shuffled_artifact)?;
    drop(shuffled_artifact);
    for (name, source) in &source_tables {
        let exact = &exact_tables[name];
        if name == "unigram" {
            if exact.total != source.total + training.len() as u64 {
                return Err("exact unigram count increment differs".to_owned());
            }
        } else if exact.total != source.total + training.len() as u64 {
            return Err(format!("exact {name} count increment differs"));
        }
    }
    if source_tables == exact_tables || source_tables == shuffled_tables {
        return Err("a count-training arm left every table unchanged".to_owned());
    }

    let no_update = evaluate(&artifact, &split.evaluation)?;
    let exact_artifact =
        std::fs::read(config.out.join("exact.uorlut")).map_err(|error| error.to_string())?;
    let exact = evaluate(&exact_artifact, &split.evaluation)?;
    drop(exact_artifact);
    let shuffled_artifact =
        std::fs::read(config.out.join("shuffled.uorlut")).map_err(|error| error.to_string())?;
    let shuffled_reading = evaluate(&shuffled_artifact, &split.evaluation)?;
    drop(shuffled_artifact);
    for (name, reading) in [
        ("no_update", &no_update),
        ("exact", &exact),
        ("shuffled", &shuffled_reading),
    ] {
        if reading.query_coordinate_mismatches != 0
            || reading.admitted_cell_mismatches != 0
            || reading.top8_mismatches != 0
            || reading.token_mismatches != 0
        {
            return Err(format!("{name} lost CPU/serving parity"));
        }
    }

    let report = json!({
        "schema": SCHEMA,
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "inputs": {
            "artifact_sha256": config.expected_artifact_sha256,
            "tokens_sha256": config.expected_tokens_sha256,
            "mask_sha256": config.expected_mask_sha256,
            "manifest_sha256": config.expected_manifest_sha256,
        },
        "selection": {
            "training_documents": split.training_documents,
            "evaluation_documents": split.evaluation_documents,
            "training_targets": split.training_targets,
            "evaluation_targets": split.evaluation_targets,
            "selected_training_rows": training.len(),
            "evaluation_stride": config.evaluation_stride,
            "max_evaluation": config.max_evaluation,
            "seed": config.seed,
        },
        "artifacts": {
            "no_update_sha256": config.expected_artifact_sha256,
            "exact_sha256": exact_sha256,
            "shuffled_sha256": shuffled_sha256,
            "exact_bytes": exact_bytes,
            "shuffled_bytes": shuffled_bytes,
        },
        "tables": {
            "source": source_tables,
            "exact": exact_tables,
            "shuffled": shuffled_tables,
        },
        "readings": {
            "no_update": reading_json(&no_update),
            "exact": reading_json(&exact),
            "shuffled": reading_json(&shuffled_reading),
        },
        "changed_decisions": {
            "exact_vs_no_update_rows": changed_rows(&exact, &no_update)?,
            "shuffled_vs_no_update_rows": changed_rows(&shuffled_reading, &no_update)?,
            "exact_vs_shuffled_rows": changed_rows(&exact, &shuffled_reading)?,
        },
        "threads": rayon::current_num_threads(),
        "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
        "peak_rss_kib": peak_rss_kib()?,
        "claim": "CPU development-research writeback test; not GPU qualification or language capability",
    });
    std::fs::write(
        config.out.join("report.json"),
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(report)
}

fn read_hashed(path: &PathBuf, expected: &str) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let actual = uor_r4_discrete::corpus::sha256(&bytes);
    if actual != expected {
        return Err(format!("{} SHA-256 differs: {actual}", path.display()));
    }
    Ok(bytes)
}

fn shuffled_targets(targets: &[u32], seed: u64) -> Result<Vec<u32>, String> {
    if targets.len() < 2 {
        return Err("shuffled count training requires at least two rows".to_owned());
    }
    let mut out = targets.to_vec();
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    for index in (1..out.len()).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let draw = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        out.swap(index, (draw as usize) % (index + 1));
    }
    if out == targets {
        out.rotate_left(1);
    }
    if out == targets {
        return Err("shuffled count training did not change the targets".to_owned());
    }
    Ok(out)
}

fn semantics(
    artifact: &[u8],
) -> Result<
    (
        Vec<u8>,
        uor_r4_discrete::position_mechanism::PositionStepTable,
        uor_r4_discrete::normalization_mechanism::NormFrameTable,
    ),
    String,
> {
    let (snapshot, steps, norms, _) =
        import_trunk_d11(artifact).map_err(|error| error.to_string())?;
    Ok((
        serde_json::to_vec(&snapshot).map_err(|error| error.to_string())?,
        steps,
        norms,
    ))
}

fn table_reading(artifact: &[u8]) -> Result<BTreeMap<String, TableReading>, String> {
    let tables = trunk_tables(artifact)?;
    let mut out = BTreeMap::new();
    out.insert(
        "unigram".to_owned(),
        TableReading {
            total: tables
                .unigram_counts
                .iter()
                .map(|&count| u64::from(count))
                .sum(),
            distinct_keys: tables
                .unigram_counts
                .iter()
                .filter(|&&count| count != 0)
                .count(),
        },
    );
    for (name, table) in sparse_tables(&tables) {
        out.insert(
            name.to_owned(),
            TableReading {
                total: table.total(),
                distinct_keys: table.distinct_keys(),
            },
        );
    }
    Ok(out)
}

fn trunk_tables(artifact: &[u8]) -> Result<TrunkTables, String> {
    import_trunk_d11(artifact)
        .map(|(_, _, _, tables)| tables)
        .map_err(|error| error.to_string())
}

fn expected_count_tables(artifact: &[u8], rows: &[TrunkTrainRow]) -> Result<TrunkTables, String> {
    let mut tables = trunk_tables(artifact)?;
    for row in rows {
        let target = usize::from(row.target_token);
        if target >= tables.unigram_counts.len() {
            return Err("expected count target exceeds vocabulary".to_owned());
        }
        tables.unigram_counts[target] = tables.unigram_counts[target]
            .checked_add(1)
            .ok_or_else(|| "expected unigram count overflows".to_owned())?;
        add_expected(
            &mut tables.read_table,
            u32::from(row.read_code),
            row.target_token,
        );
        add_expected(
            &mut tables.rec_table,
            u32::from(row.rec_code),
            row.target_token,
        );
        add_expected(
            &mut tables.read_rec_table,
            pack_u32_key(row.read_code, row.rec_code),
            row.target_token,
        );
        add_expected(
            &mut tables.rec_shuf_table,
            u32::from(row.rec_shuf_code),
            row.target_token,
        );
        add_expected(
            &mut tables.read_rec_shuf_table,
            pack_u32_key(row.read_code, row.rec_shuf_code),
            row.target_token,
        );
        add_expected(
            &mut tables.rec_unit_table,
            u32::from(row.rec_unit_code),
            row.target_token,
        );
        add_expected(
            &mut tables.read_rec_unit_table,
            pack_u32_key(row.read_code, row.rec_unit_code),
            row.target_token,
        );
        add_expected(
            &mut tables.query_table,
            u32::from(row.query_token),
            row.target_token,
        );
        add_expected(
            &mut tables.query_read_table,
            pack_u32_key(row.query_token, row.read_code),
            row.target_token,
        );
        add_expected(
            &mut tables.query_rec_table,
            pack_u32_key(row.query_token, row.rec_code),
            row.target_token,
        );
        add_expected(
            &mut tables.query_rec_shuf_table,
            pack_u32_key(row.query_token, row.rec_shuf_code),
            row.target_token,
        );
        add_expected(
            &mut tables.query_rec_unit_table,
            pack_u32_key(row.query_token, row.rec_unit_code),
            row.target_token,
        );
    }
    let mut best = 0u16;
    let mut best_count = 0u32;
    for (token, &count) in tables.unigram_counts.iter().enumerate() {
        if count > best_count {
            best = u16::try_from(token).map_err(|_| "expected unigram token exceeds u16")?;
            best_count = count;
        }
    }
    tables.unigram_best_global = best;
    Ok(tables)
}

fn add_expected(table: &mut SparseCountTable, key: u32, token: u16) {
    table.add(key, token);
}

fn sparse_tables(
    tables: &TrunkTables,
) -> [(&'static str, &uor_r4_discrete::trunk::SparseCountTable); 12] {
    [
        ("read", &tables.read_table),
        ("rec", &tables.rec_table),
        ("read_rec", &tables.read_rec_table),
        ("rec_shuf", &tables.rec_shuf_table),
        ("read_rec_shuf", &tables.read_rec_shuf_table),
        ("rec_unit", &tables.rec_unit_table),
        ("read_rec_unit", &tables.read_rec_unit_table),
        ("query", &tables.query_table),
        ("query_read", &tables.query_read_table),
        ("query_rec", &tables.query_rec_table),
        ("query_rec_shuf", &tables.query_rec_shuf_table),
        ("query_rec_unit", &tables.query_rec_unit_table),
    ]
}

fn evaluate(artifact: &[u8], samples: &[RankMarginSample]) -> Result<DecisionReading, String> {
    let cpu = CpuDiscreteTrunkReader::from_bytes(artifact).map_err(|error| error.to_string())?;
    let served = IntegerDiscreteModel::from_bytes(artifact).map_err(|error| error.to_string())?;
    let outcomes = collect_decision_outcomes(samples, |sample| {
        let (decision, admitted, top1, top8) = cpu
            .observe_target(&sample.input, sample.target)
            .map_err(|error| error.to_string())?;
        let serving = served
            .evaluate_prefix(&sample.input)
            .map_err(|error| error.to_string())?;
        let query_coordinate_mismatches = decision
            .query
            .iter()
            .zip(&serving.read_vector)
            .filter(|(left, right)| left != right)
            .count() as u64;
        let admitted_cell_mismatches = u64::from(decision.admitted_cells != serving.admitted_cells);
        let mut served_top8 = [u32::MAX; 8];
        for (slot, &token) in serving.top8.iter().take(8).enumerate() {
            served_top8[slot] = token;
        }
        Ok(DecisionOutcome {
            admitted,
            top1,
            top8,
            token_mismatches: u64::from(decision.token != serving.prediction),
            top8_mismatches: u64::from(decision.top8 != served_top8),
            query_coordinate_mismatches,
            admitted_cell_mismatches,
            query: decision.query.to_vec(),
            top8_tokens: decision.top8,
            token: decision.token,
        })
    })?;
    Ok(summarize_decision_outcomes(outcomes))
}

fn collect_decision_outcomes(
    samples: &[RankMarginSample],
    evaluate: impl Fn(&RankMarginSample) -> Result<DecisionOutcome, String> + Send + Sync,
) -> Result<Vec<DecisionOutcome>, String> {
    samples.par_iter().map(evaluate).collect()
}

fn summarize_decision_outcomes(outcomes: Vec<DecisionOutcome>) -> DecisionReading {
    let mut reading = DecisionReading::default();
    for outcome in outcomes {
        reading.rows += 1;
        reading.admitted += u64::from(outcome.admitted);
        reading.top1 += u64::from(outcome.top1);
        reading.top8 += u64::from(outcome.top8);
        reading.query_coordinates += outcome.query.len() as u64;
        reading.query_coordinate_mismatches += outcome.query_coordinate_mismatches;
        reading.admitted_cell_mismatches += outcome.admitted_cell_mismatches;
        reading.top8_mismatches += outcome.top8_mismatches;
        reading.token_mismatches += outcome.token_mismatches;
        let mut row_checksum = 0u64;
        for &coordinate in &outcome.query {
            reading.checksum = reading
                .checksum
                .rotate_left(7)
                .wrapping_add(coordinate as u32 as u64);
            row_checksum = row_checksum
                .rotate_left(7)
                .wrapping_add(coordinate as u32 as u64);
        }
        for &token in &outcome.top8_tokens {
            reading.checksum = reading
                .checksum
                .rotate_left(7)
                .wrapping_add(u64::from(token));
            row_checksum = row_checksum.rotate_left(7).wrapping_add(u64::from(token));
        }
        reading.checksum = reading
            .checksum
            .rotate_left(7)
            .wrapping_add(u64::from(outcome.token));
        row_checksum = row_checksum
            .rotate_left(7)
            .wrapping_add(u64::from(outcome.token));
        reading.row_checksums.push(row_checksum);
    }
    reading
}

fn changed_rows(left: &DecisionReading, right: &DecisionReading) -> Result<usize, String> {
    if left.row_checksums.len() != right.row_checksums.len() {
        return Err("decision arms have different row counts".to_owned());
    }
    Ok(left
        .row_checksums
        .iter()
        .zip(&right.row_checksums)
        .filter(|(left, right)| left != right)
        .count())
}

fn reading_json(reading: &DecisionReading) -> serde_json::Value {
    json!({
        "rows": reading.rows,
        "admitted": reading.admitted,
        "top1": reading.top1,
        "top8": reading.top8,
        "query_coordinates": reading.query_coordinates,
        "query_coordinate_mismatches": reading.query_coordinate_mismatches,
        "admitted_cell_mismatches": reading.admitted_cell_mismatches,
        "top8_mismatches": reading.top8_mismatches,
        "token_mismatches": reading.token_mismatches,
        "checksum": format!("{:016x}", reading.checksum),
    })
}

fn peak_rss_kib() -> Result<u64, String> {
    let status = std::fs::read_to_string("/proc/self/status")
        .map_err(|error| format!("cannot measure peak RSS: {error}"))?;
    parse_peak_rss_kib(&status)
}

fn parse_peak_rss_kib(status: &str) -> Result<u64, String> {
    let peak = status
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmHWM:")
                .and_then(|value| value.split_whitespace().next())
                .and_then(|value| value.parse().ok())
        })
        .ok_or_else(|| "peak RSS measurement is unavailable".to_owned())?;
    if peak == 0 {
        return Err("peak RSS measurement is zero".to_owned());
    }
    Ok(peak)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_outcome(sample: &RankMarginSample) -> Result<DecisionOutcome, String> {
        let query = (0..=sample.target % 5)
            .map(|coordinate| sample.target as i32 + coordinate as i32)
            .collect::<Vec<_>>();
        let top8_tokens = std::array::from_fn(|slot| sample.target + slot as u32);
        Ok(DecisionOutcome {
            admitted: sample.target.is_multiple_of(2),
            top1: sample.target.is_multiple_of(3),
            top8: sample.target.is_multiple_of(5),
            query,
            top8_tokens,
            token: sample.target,
            query_coordinate_mismatches: 0,
            admitted_cell_mismatches: 0,
            top8_mismatches: 0,
            token_mismatches: 0,
        })
    }

    #[test]
    fn ordered_decision_fold_is_identical_across_worker_counts() {
        let samples = (0u32..64)
            .map(|target| RankMarginSample {
                input: vec![target, target + 1],
                target,
            })
            .collect::<Vec<_>>();
        let expected = summarize_decision_outcomes(
            samples
                .iter()
                .map(test_outcome)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
        );
        for workers in [1usize, 8, 16, 32] {
            let pool = ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()
                .unwrap();
            let reading = pool.install(|| {
                summarize_decision_outcomes(
                    collect_decision_outcomes(&samples, test_outcome).unwrap(),
                )
            });
            assert_eq!(reading, expected, "{workers} workers");
        }
    }

    #[test]
    fn peak_rss_measurement_fails_closed() {
        assert_eq!(parse_peak_rss_kib("VmHWM:\t1234 kB\n").unwrap(), 1234);
        assert!(parse_peak_rss_kib("VmRSS:\t1234 kB\n").is_err());
        assert!(parse_peak_rss_kib("VmHWM:\t0 kB\n").is_err());
        assert!(parse_peak_rss_kib("VmHWM:\tnot-a-number kB\n").is_err());
    }

    #[test]
    fn shuffled_targets_are_deterministic_and_non_vacuous() {
        let targets = [4, 8, 15, 16];
        let first = shuffled_targets(&targets, 9).unwrap();
        let second = shuffled_targets(&targets, 9).unwrap();
        assert_eq!(first, second);
        assert_ne!(first, targets);
        let mut before = targets;
        let mut after = first.clone();
        before.sort_unstable();
        after.sort_unstable();
        assert_eq!(after, before);
        assert!(shuffled_targets(&[7], 9).is_err());
        assert!(shuffled_targets(&[7, 7], 9).is_err());
    }
}
