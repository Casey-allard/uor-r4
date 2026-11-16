//! `student-ingest --tokenizer <tokenizer.json> --out <dir> <shard>...`
//!
//! Reads donor JSONL shards (`.jsonl` or `.jsonl.zst`; D41 pilot or D87 production records),
//! writes `<dir>/tokens.u16`, `response_mask.u8`, `segment.u8` and `summary.json`. Refuses the
//! whole run on the first malformed record, naming its id. Never writes next to its inputs.

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::ExitCode;

use m2_student_corpus_ingest::{
    hex, parse_record, read_shard, Coder, Ingest, Segment, ASSISTANT_MARKER, SEGMENTS, SEPARATOR,
    SYSTEM_MARKER, USER_MARKER,
};
use sha2::{Digest, Sha256};

fn usage() -> ExitCode {
    eprintln!("usage: student-ingest --tokenizer <tokenizer.json> --out <dir> <shard>...");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (mut tokenizer, mut out, mut shards) = (None, None, Vec::<PathBuf>::new());
    while let Some(a) = args.next() {
        match a.as_str() {
            "--tokenizer" => tokenizer = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            s if s.starts_with("--") => return usage(),
            s => shards.push(PathBuf::from(s)),
        }
    }
    let (Some(tokenizer), Some(out)) = (tokenizer, out) else {
        return usage();
    };
    if shards.is_empty() {
        return usage();
    }
    match run(&tokenizer, &out, &shards) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("student-ingest: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(tokenizer: &PathBuf, out: &PathBuf, shards: &[PathBuf]) -> Result<(), String> {
    let json = std::fs::read(tokenizer).map_err(|e| format!("{}: {e}", tokenizer.display()))?;
    let coder = Coder::new(&json)?;
    let tokenizer_sha = hex(&Sha256::digest(&json));
    for s in shards {
        if s.parent() == Some(out.as_path()) {
            return Err(format!("{} is inside --out", s.display()));
        }
    }
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    // A summary from an earlier run must not survive a failed one: it would describe other data.
    let summary_path = out.join("summary.json");
    if summary_path.exists() {
        std::fs::remove_file(&summary_path).map_err(|e| format!("summary.json: {e}"))?;
    }
    let create = |name: &str| {
        File::create(out.join(name))
            .map(BufWriter::new)
            .map_err(|e| format!("{name}: {e}"))
    };
    let mut ing = Ingest::new(
        create("tokens.u16")?,
        create("response_mask.u8")?,
        create("segment.u8")?,
    )
    .map_err(|e| e.to_string())?;
    let mut inputs = vec![];
    for s in shards {
        let mut n = 0u64;
        let sha = read_shard(s, &mut |line| {
            let record = parse_record(line)?;
            ing.add(&coder, &record)?;
            n += 1;
            Ok(())
        })
        .map_err(|e| format!("{}: {e}", s.display()))?;
        inputs.push(
            serde_json::json!({"path": s.display().to_string(), "sha256": sha, "records": n}),
        );
        eprintln!("student-ingest: {} records from {}", n, s.display());
    }
    let (_, _, _, stats) = ing.finish().map_err(|e| e.to_string())?;

    let markers: serde_json::Map<String, serde_json::Value> = [
        ("user", USER_MARKER),
        ("assistant", ASSISTANT_MARKER),
        ("system", SYSTEM_MARKER),
        ("separator", SEPARATOR),
    ]
    .iter()
    .map(|(k, t)| ((*k).to_string(), serde_json::json!(coder.tok.encode(t))))
    .collect();
    let per_segment: serde_json::Map<String, serde_json::Value> = SEGMENTS
        .iter()
        .map(|&s: &Segment| {
            let c = &stats.per_segment[&s];
            (
                s.name().to_string(),
                serde_json::json!({
                    "text_bytes": c.text_bytes,
                    "tokens": c.tokens,
                    "dropped_bytes": c.dropped_bytes,
                    "fallback_tokens": c.fallback_tokens,
                    "special_in_text": c.special_in_text,
                    "bytes_per_token": [c.text_bytes, c.tokens],
                    "dropped_per_byte": [c.dropped_bytes, c.text_bytes],
                    "fallback_per_token": [c.fallback_tokens, c.tokens],
                }),
            )
        })
        .collect();
    let top: Vec<serde_json::Value> = stats
        .top_runs(20)
        .into_iter()
        .map(|(k, n)| serde_json::json!({"hex": k, "count": n}))
        .collect();
    let summary = serde_json::json!({
        "unit": "claude-403",
        "tokenizer_sha256": tokenizer_sha,
        "tokenizer_ids": coder.tok.vocab_size(),
        "unemittable_bytes_hex": hex(&coder.unemittable()),
        "marker_ids": markers,
        "inputs": inputs,
        "records": stats.records,
        "total_tokens": stats.total_tokens,
        "per_segment": per_segment,
        "top_odd_runs": top,
        "odd_run_keys": stats.runs.len(),
        "odd_runs_unkeyed": stats.runs_unkeyed,
        "ratios": "each [numerator, denominator] pair is an exact integer ratio",
    });
    let text = serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())?;
    std::fs::write(out.join("summary.json"), text + "\n").map_err(|e| e.to_string())?;
    eprintln!(
        "student-ingest: {} tokens written to {}",
        stats.total_tokens,
        out.display()
    );
    Ok(())
}
