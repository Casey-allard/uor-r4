//! The snap parity report of a snapped model directory: the D11 engine's
//! root selections and logits against the float snapped forward, on evenly
//! spaced windows of given tokens.
//!
//! `stack-snap-parity model=DIR tokens=FILE out=DIR [windows=N]`

use std::fs;
use std::path::{Path, PathBuf};

use uor_r4_core::report_output;
use uor_r4_training::stack_snap_parity::snap_parity_report;
use uor_r4_training::{Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

fn main() -> Result<()> {
    let mut model = None;
    let mut tokens = None;
    let mut out = None;
    let mut windows = 8usize;
    for argument in std::env::args().skip(1) {
        let (key, value) = argument
            .split_once('=')
            .ok_or_else(|| invalid(format!("arguments are key=value, got {argument}")))?;
        match key {
            "model" => model = Some(PathBuf::from(value)),
            "tokens" => tokens = Some(PathBuf::from(value)),
            "out" => out = Some(PathBuf::from(value)),
            "windows" => {
                windows = value
                    .parse()
                    .map_err(|_| invalid(format!("windows must be a count, got {value}")))?
            }
            _ => return Err(invalid(format!("unknown argument {key}="))),
        }
    }
    let model = model.ok_or_else(|| invalid("model= is required"))?;
    let tokens = tokens.ok_or_else(|| invalid("tokens= is required"))?;
    let out = out.ok_or_else(|| invalid("out= is required"))?;
    report_output::claim(&out)?;
    let tokens = read_tokens(&tokens, vocabulary(&model)?)?;
    let report = snap_parity_report(&model, &tokens, windows)?;
    fs::write(
        out.join("snap-parity.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

fn vocabulary(model: &Path) -> Result<usize> {
    let config: serde_json::Value = serde_json::from_slice(&fs::read(model.join("config.json"))?)?;
    config["vocab_size"]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| invalid("the model config has no vocabulary size"))
}

fn read_tokens(path: &Path, vocabulary: usize) -> Result<Vec<u32>> {
    let bytes = fs::read(path)?;
    if bytes.starts_with(b"UORT") {
        let reader = uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader::open(path)
            .map_err(|e| invalid(format!("{}: {e}", path.display())))?;
        if reader.vocab_size() as usize > vocabulary {
            return Err(invalid(format!(
                "{} declares a larger vocabulary",
                path.display()
            )));
        }
        let tokens: Vec<u32> = reader.as_slice().iter().map(|&id| u32::from(id)).collect();
        if tokens.iter().any(|&id| id as usize >= vocabulary) {
            return Err(invalid(format!(
                "{} has ids outside the vocabulary",
                path.display()
            )));
        }
        return Ok(tokens);
    }
    if bytes.len() % 2 != 0 {
        return Err(invalid(format!(
            "{} is not a u16 token file",
            path.display()
        )));
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect();
    if tokens.iter().any(|&id| id as usize >= vocabulary) {
        return Err(invalid(format!(
            "{} has ids outside the vocabulary",
            path.display()
        )));
    }
    Ok(tokens)
}
