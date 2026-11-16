//! pair-picks (claude-384): the trainer's own pair-row picks for every prompt of a scored panel,
//! the oracle of the served replay check.
//!
//! Usage: `pair-picks CHECKPOINT_DIR SAMPLES_JSON OUT_JSONL`
//!
//! `CHECKPOINT_DIR` is a `models/step-N` folder holding `pair_rows.safetensors` and
//! `pair_rows.dict` (claude-370). `SAMPLES_JSON` is a scoring pass's
//! `samples-*.json` (an array of objects with `position` and `prompt`, the prompt's token ids).
//! For each prompt, the float crate's own plan (`pair_rows::last_position_rows`, the turn open at
//! the last position) names the rows the trunk reads there. One line per prompt is written to
//! `OUT_JSONL`: `{"index", "position", "rows", "addresses"}`, rows as u32 row numbers and
//! addresses as the u32 prime products. Weighed rows are refused (they have no served form). The
//! output is never written over, and the file's SHA-256 is printed.
//!
//! It trains nothing and evaluates nothing; it opens the GPU only to load the checkpoint.

use std::path::Path;

use candle_core::Device;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uor_r4_float_train::fbs_export::check_pair_rows_servable;
use uor_r4_float_train::pair_rows::{last_position_rows, PairRows};
use uor_r4_float_train::{FloatTrainError, Result};
use uor_r4_integer::fbs_snapshot::write_new_file;

#[derive(Deserialize)]
struct Sample {
    position: usize,
    prompt: Vec<u32>,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [dir, samples, out] = args.as_slice() else {
        eprintln!("usage: pair-picks CHECKPOINT_DIR SAMPLES_JSON OUT_JSONL");
        std::process::exit(2);
    };
    if Path::new(out).exists() {
        return Err(FloatTrainError::Invalid(format!(
            "{out}: exists; it is not written over"
        )));
    }
    let samples: Vec<Sample> = serde_json::from_slice(&std::fs::read(samples)?)?;
    let device = Device::new_cuda(0)?;
    let rows = PairRows::load(Path::new(dir), &device)?
        .ok_or_else(|| FloatTrainError::Invalid(format!("{dir}: no pair rows")))?;
    check_pair_rows_servable(&rows)?;
    let (dict, marks) = (rows.dict(), rows.marks());
    let mut text = String::new();
    let mut picked = 0usize;
    for (index, sample) in samples.iter().enumerate() {
        let picks = last_position_rows(dict, marks, &sample.prompt)?;
        picked += usize::from(!picks.is_empty());
        let addresses: Vec<u32> = picks
            .iter()
            .map(|&r| dict.addresses()[r as usize])
            .collect();
        let line = serde_json::json!({
            "index": index,
            "position": sample.position,
            "rows": picks,
            "addresses": addresses,
        });
        text.push_str(&serde_json::to_string(&line)?);
        text.push('\n');
    }
    write_new_file(Path::new(out), text.as_bytes())?;
    println!("{}  {out}", hex::encode(Sha256::digest(text.as_bytes())));
    eprintln!("{} prompts, {picked} with picks", samples.len());
    Ok(())
}
