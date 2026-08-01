//! Transformer-free corpus refresh and extension.
//!
//! This path deliberately starts from a frozen table-native artifact and
//! store. It relabels seed records and extends them with the deployed
//! integer runtime, then rebuilds the store with the existing bounded
//! parallel front-end. No teacher model, floating-point oracle, or matmul is
//! loaded here. The resulting bundle is self-distilled and must not be
//! compared with teacher-parity rows without an explicit provenance change.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use uor_r4_core::transformerless::{compiler, runtime};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    artifacts: PathBuf,
    store: PathBuf,
    seed_meta: PathBuf,
    seed_recs: PathBuf,
    output: PathBuf,
    target: usize,
    threads: usize,
}

#[derive(Debug, Serialize)]
struct Manifest {
    schema: u32,
    mode: &'static str,
    seed_records: usize,
    generated_records: usize,
    records: usize,
    threads: usize,
    seed_meta_kappa: String,
    seed_records_kappa: String,
    artifacts_kappa: String,
    store_kappa: String,
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut artifacts = None;
    let mut store = None;
    let mut seed_meta = None;
    let mut seed_recs = None;
    let mut output = None;
    let mut target = None;
    let mut threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let mut index = 0;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--artifacts" => artifacts = Some(PathBuf::from(value)),
            "--store" => store = Some(PathBuf::from(value)),
            "--seed-meta" => seed_meta = Some(PathBuf::from(value)),
            "--seed-recs" => seed_recs = Some(PathBuf::from(value)),
            "--out" => output = Some(PathBuf::from(value)),
            "--target" => {
                let parsed = value
                    .parse()
                    .map_err(|_| format!("invalid --target value: {value}"))?;
                if parsed == 0 {
                    return Err("--target must be greater than zero".to_owned());
                }
                target = Some(parsed);
            }
            "--threads" => {
                threads = value
                    .parse()
                    .map_err(|_| format!("invalid --threads value: {value}"))?;
                if threads == 0 {
                    return Err("--threads must be greater than zero".to_owned());
                }
            }
            _ => return Err(format!("unknown runtime-corpus option: {flag}")),
        }
        index += 2;
    }

    Ok(Options {
        artifacts: artifacts.ok_or("missing --artifacts")?,
        store: store.ok_or("missing --store")?,
        seed_meta: seed_meta.ok_or("missing --seed-meta")?,
        seed_recs: seed_recs.ok_or("missing --seed-recs")?,
        output: output.ok_or("missing --out")?,
        target: target.ok_or("missing --target")?,
        threads,
    })
}

fn file_kappa(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
}

fn window_for_position(
    corpus: &compiler::Corpus,
    index: usize,
    window: &mut [u32; compiler::WINDOW],
) -> usize {
    let story = corpus.story[index];
    let mut start = index;
    while start > 0 && index - start + 1 < compiler::WINDOW && corpus.story[start - 1] == story {
        start -= 1;
    }
    let mut length = 0;
    for position in start..=index {
        window[length] = corpus.input[position];
        length += 1;
    }
    length
}

fn append_runtime_record(
    records: &mut BufWriter<File>,
    story: u32,
    next: u32,
    position: u32,
    prediction: runtime::Prediction,
) -> Result<(), String> {
    let top_tokens = [prediction.token, 0, 0];
    let top_weights = [100, 0, 0];
    let record = compiler::encode_v3_record(
        story,
        next,
        &top_tokens,
        &top_weights,
        (position, position.saturating_add(1)),
        (u32::MAX, u32::MAX),
    );
    records
        .write_all(&record)
        .map_err(|error| error.to_string())
}

pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    fs::create_dir_all(&options.output).map_err(|error| error.to_string())?;

    let artifact_bytes = fs::read(&options.artifacts).map_err(|error| error.to_string())?;
    let art = compiler::parse_artifacts(&artifact_bytes)
        .ok_or_else(|| format!("invalid artifacts: {}", options.artifacts.display()))?;
    let store_bytes = fs::read(&options.store).map_err(|error| error.to_string())?;
    let store = runtime::parse_store(&store_bytes)
        .ok_or_else(|| format!("invalid store: {}", options.store.display()))?;
    let seed = compiler::load_corpus_from(
        options
            .seed_meta
            .to_str()
            .ok_or("seed metadata path is not UTF-8")?,
        options
            .seed_recs
            .to_str()
            .ok_or("seed records path is not UTF-8")?,
    )
    .ok_or_else(|| "seed corpus is incomplete or malformed".to_owned())?;
    if seed.n == 0 {
        return Err("seed corpus is empty".to_owned());
    }

    let meta_path = options.output.join("corpus.meta");
    let records_path = options.output.join("corpus.records");
    let mut records =
        BufWriter::new(File::create(&records_path).map_err(|error| error.to_string())?);
    let mut runtime = runtime::Runtime::new(&art);
    let mut window = [0u32; compiler::WINDOW];
    let mut previous_story = None;

    let seed_limit = seed.n.min(options.target);
    for index in 0..seed_limit {
        if previous_story != Some(seed.story[index]) {
            runtime.state = Default::default();
            previous_story = Some(seed.story[index]);
        }
        let window_len = window_for_position(&seed, index, &mut window);
        let code = runtime.assign_window(&window[..window_len]);
        let prediction = runtime.predict_witness(&store, &code);
        append_runtime_record(
            &mut records,
            seed.story[index],
            seed.next[index],
            seed.span_start[index],
            prediction,
        )?;
    }

    let mut generated = 0usize;
    let mut records_written = seed_limit;
    let mut story_id = seed.stories as u32;
    let mut generated_window_len = window_for_position(&seed, seed.n - 1, &mut window);
    while records_written < options.target {
        runtime.state = Default::default();
        let mut position = 0u32;
        while position < 128 && records_written < options.target {
            let code = runtime.assign_window(&window[..generated_window_len]);
            let prediction = runtime.predict_witness(&store, &code);
            append_runtime_record(
                &mut records,
                story_id,
                prediction.token,
                position,
                prediction,
            )?;
            if generated_window_len < compiler::WINDOW {
                window[generated_window_len] = prediction.token;
                generated_window_len += 1;
            } else {
                window.copy_within(1.., 0);
                window[compiler::WINDOW - 1] = prediction.token;
            }
            generated += 1;
            records_written += 1;
            position += 1;
        }
        story_id = story_id.saturating_add(1);
        generated_window_len = generated_window_len.min(compiler::WINDOW);
    }
    records.flush().map_err(|error| error.to_string())?;

    let mut meta = [0u8; 25];
    meta[0..8].copy_from_slice(&(records_written as u64).to_le_bytes());
    meta[8..16].copy_from_slice(&(story_id as u64).to_le_bytes());
    meta[16..24].copy_from_slice(&0x52554E54494D45u64.to_le_bytes());
    meta[24] = 1;
    fs::write(&meta_path, meta).map_err(|error| error.to_string())?;

    let final_corpus = compiler::load_corpus_from(
        meta_path
            .to_str()
            .ok_or("output metadata path is not UTF-8")?,
        records_path
            .to_str()
            .ok_or("output records path is not UTF-8")?,
    )
    .ok_or_else(|| "runtime corpus failed its own round-trip validation".to_owned())?;
    let threads = options.threads.min(final_corpus.n.max(1));
    let (rebuilt_store, _) = runtime::build_store_with_threads(&art, &final_corpus, threads)?;
    let rebuilt_store_bytes = runtime::store_bytes(&rebuilt_store);
    fs::write(options.output.join("tless_artifacts.bin"), &artifact_bytes)
        .map_err(|error| error.to_string())?;
    fs::write(options.output.join("tless_store.bin"), &rebuilt_store_bytes)
        .map_err(|error| error.to_string())?;

    let manifest = Manifest {
        schema: 1,
        mode: "runtime-self-distilled-v1",
        seed_records: seed_limit,
        generated_records: generated,
        records: records_written,
        threads,
        seed_meta_kappa: file_kappa(&options.seed_meta)?,
        seed_records_kappa: file_kappa(&options.seed_recs)?,
        artifacts_kappa: format!("blake3:{}", blake3::hash(&artifact_bytes).to_hex()),
        store_kappa: runtime::store_kappa(&rebuilt_store),
    };
    let manifest_json = serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?;
    fs::write(
        options.output.join("runtime_corpus_manifest.json"),
        manifest_json,
    )
    .map_err(|error| error.to_string())?;

    println!(
        "runtime corpus complete: {} records ({} seed + {} generated), {} threads, output {}",
        records_written,
        seed_limit,
        generated,
        threads,
        options.output.display()
    );
    println!("op census: runtime-only path; no teacher or matmul loaded");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_options;

    #[test]
    fn runtime_corpus_requires_all_paths() {
        let args = ["--target", "10"].map(str::to_owned);
        assert!(parse_options(&args).is_err());
    }

    #[test]
    fn runtime_corpus_rejects_zero_target_and_threads() {
        let base = [
            "--artifacts",
            "a",
            "--store",
            "s",
            "--seed-meta",
            "m",
            "--seed-recs",
            "r",
            "--out",
            "o",
        ];
        for extra in [["--target", "0"], ["--threads", "0"]] {
            let mut args = base.map(str::to_owned).to_vec();
            args.extend(extra.map(str::to_owned));
            assert!(parse_options(&args).is_err());
        }
    }
}
