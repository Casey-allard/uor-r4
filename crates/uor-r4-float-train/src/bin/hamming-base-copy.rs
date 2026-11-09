//! claude-339: a copy of a checkpoint's `hamming_read.safetensors` with the ask heads and the
//! question carry left out, so the laptop's `cpu-fbs-export` (claude-322's export path) can
//! snapshot the trunk and the base Hamming reads of an ask checkpoint on the CPU.
//!
//! Why: `fbs-snapshot` builds the trunk tables on the pod (claude-339's I-xg, 9 Nov 2026), and
//! one quantized atom of claude-322's seed-1 trunk came out differently there than on the laptop,
//! so the GPU path does not reproduce claude-322's artifact. `cpu-fbs-export` does, but it refuses
//! a Hamming inventory with the ask tensors. This tool writes the base inventory only; the ask
//! entries are then taken from the pod's snapshot, whose `hamming_ask` is built on the CPU.
//!
//! Every kept tensor is copied with its dtype, shape and bytes unchanged, and the file's metadata
//! is kept. Only `carry_rows`, `d{depth}.ask` and `d{depth}.carry` are left out; any other name
//! outside the base inventory (`heads`, `copy_key` or `copy_keys`, and five reads per depth)
//! refuses, so nothing is dropped silently.
//!
//! usage: hamming-base-copy IN.safetensors OUT.safetensors   (OUT must not exist)

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use safetensors::tensor::{Metadata, SafeTensors};

const BASE_READS: [&str; 5] = [
    "query_rotation",
    "key_rotation",
    "value_rotation",
    "margin",
    "weight",
];

/// `Some(true)` for a tensor to keep, `Some(false)` for an ask or carry tensor to leave out, and
/// `None` for a name this tool does not know.
fn classify(name: &str) -> Option<bool> {
    // The base inventory `cpu-fbs-export` checks (fbs_snapshot_cpu.rs): the head count, an
    // optional copy key or per-head copy keys, and the five reads per depth.
    if matches!(name, "heads" | "copy_key" | "copy_keys") {
        return Some(true);
    }
    if name == "carry_rows" {
        return Some(false);
    }
    let (depth, part) = name.split_once('.')?;
    let digits = depth.strip_prefix('d')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if BASE_READS.contains(&part) {
        Some(true)
    } else if part == "ask" || part == "carry" {
        Some(false)
    } else {
        None
    }
}

fn run(input: &Path, output: &Path) -> Result<(), String> {
    if output.exists() {
        return Err(format!("{}: already exists", output.display()));
    }
    let bytes = fs::read(input).map_err(|e| format!("{}: {e}", input.display()))?;
    let tensors =
        SafeTensors::deserialize(&bytes).map_err(|e| format!("{}: {e}", input.display()))?;
    let (_, metadata): (usize, Metadata) =
        SafeTensors::read_metadata(&bytes).map_err(|e| format!("{}: {e}", input.display()))?;
    let mut kept = Vec::new();
    let mut dropped = BTreeSet::new();
    for (name, view) in tensors.tensors() {
        match classify(&name) {
            Some(true) => kept.push((name, view)),
            Some(false) => {
                dropped.insert(name);
            }
            None => {
                return Err(format!(
                    "{}: unknown tensor {name}; nothing written",
                    input.display()
                ))
            }
        }
    }
    if kept.is_empty() {
        return Err(format!(
            "{}: no base Hamming tensor; nothing written",
            input.display()
        ));
    }
    kept.sort_by(|a, b| a.0.cmp(&b.0));
    let names: Vec<String> = kept.iter().map(|(name, _)| name.clone()).collect();
    let out = safetensors::serialize(
        kept.iter().map(|(n, v)| (n.as_str(), v)),
        metadata.metadata().clone(),
    )
    .map_err(|e| e.to_string())?;
    fs::write(output, &out).map_err(|e| format!("{}: {e}", output.display()))?;
    println!(
        "{}",
        serde_json::json!({ "kept": names, "dropped": dropped, "out": output.display().to_string() })
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [input, output] => run(Path::new(input), Path::new(output)),
        _ => Err("usage: hamming-base-copy IN.safetensors OUT.safetensors".to_owned()),
    };
    if let Err(error) = result {
        eprintln!("hamming-base-copy: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::classify;

    #[test]
    fn keeps_the_base_reads_and_leaves_out_the_asks() {
        assert_eq!(classify("d2.query_rotation"), Some(true));
        assert_eq!(classify("d6.weight"), Some(true));
        assert_eq!(classify("d6.ask"), Some(false));
        assert_eq!(classify("d6.carry"), Some(false));
        assert_eq!(classify("carry_rows"), Some(false));
        assert_eq!(classify("d6.other"), None);
        assert_eq!(classify("x6.weight"), None);
        assert_eq!(classify("d.weight"), None);
        assert_eq!(classify("weight"), None);
        assert_eq!(classify("heads"), Some(true));
        assert_eq!(classify("copy_keys"), Some(true));
    }
}
