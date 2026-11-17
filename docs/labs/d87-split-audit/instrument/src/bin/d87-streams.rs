//! `d87-streams --tokenizer <tokenizer.json> --split <dir> --out <dir> [--threads T]
//!  [--bloom-log2 B] <shard>...`
//!
//! The sealed per-split token streams (claude-420). It re-derives the split from
//! `<split>/split-summary.json`'s config over the same shards and refuses unless the result
//! reproduces that summary byte for byte. Then, for TRAIN, DEV-tune and DEV-test, it writes:
//! - `<split>.tokens.u16`: a UORT stream;
//! - `<split>.mask.u8`: the response mask;
//! - `<split>.segment.u8`: the segment tags;
//! - `<split>.json`: counts and the sha256 of each file.
//!
//! For the two DEV splits it also writes `<split>.novel.u8`, each position's novel-target class.
//! `id-bytes.u8` is the per-id byte table. The streams and masks are corpus data and are never
//! committed; the `.json` summaries hold only counts and digests.

use std::path::PathBuf;
use std::process::ExitCode;

use d87_split_audit::{plan, streams::write_streams, Splitter};
use m2_student_corpus_ingest::{hex, Coder};
use sha2::{Digest, Sha256};

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let (mut tok, mut split, mut out, mut threads, mut bloom, mut shards) =
        (None, None, None, 12usize, 30u32, vec![]);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--tokenizer" => tok = args.next().map(PathBuf::from),
            "--split" => split = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "--threads" => {
                threads = args
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or("--threads")?
            }
            "--bloom-log2" => {
                bloom = args
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or("--bloom-log2")?
            }
            s if s.starts_with("--") => return Err(format!("unknown flag {s}")),
            s => shards.push(PathBuf::from(s)),
        }
    }
    let (Some(tok), Some(split), Some(out)) = (tok, split, out) else {
        return Err("--tokenizer, --split and --out are required".into());
    };
    if !(6..=34).contains(&bloom) {
        return Err("--bloom-log2 must be 6..=34".into());
    }
    let summary_path = split.join("split-summary.json");
    let text = std::fs::read_to_string(&summary_path)
        .map_err(|e| format!("{}: {e}", summary_path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let sp = Splitter::from_config_json(&v["config"])?;
    let p = plan(&sp, &shards, threads)?;
    let again = serde_json::to_string_pretty(&p.summary()).map_err(|e| e.to_string())? + "\n";
    if again != text {
        return Err("the shards do not reproduce split-summary.json".into());
    }
    let json = std::fs::read(&tok).map_err(|e| format!("{}: {e}", tok.display()))?;
    let coder = Coder::new(&json)?;
    let tokenizer_sha = hex(&Sha256::digest(&json));
    let summaries = write_streams(&p, &coder, threads, bloom, &out, &tokenizer_sha)?;
    for s in &summaries {
        eprintln!(
            "d87-streams: {}: {} records, {} tokens, {} target bytes, {} dropped bytes",
            s["split"], s["records"], s["tokens"], s["target_bytes"], s["dropped_bytes"]
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("d87-streams: {e}");
            ExitCode::FAILURE
        }
    }
}
