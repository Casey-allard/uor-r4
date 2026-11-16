//! `d87-audit --tokenizer <tokenizer.json> --split <dir> --out <dir> [--threads T]
//!  [--bloom-log2 B] [--classify-spans F] <shard>...`
//!
//! Re-derives the split from `<split>/split-summary.json`'s config over the same shards and
//! refuses unless it reproduces that summary byte for byte; then tokenizes every kept record
//! with student-ingest and writes `<out>/audit-summary.json`: the long-span contamination gate
//! (n = 32, 50, 100; the gate reads 50), the exact 13-gram TRAIN/DEV overlap census as a
//! diagnostic, the novel-target counts and the per-split target counts. Beside it,
//! `prompt-templates.tsv` (decoded corpus text) and `contaminated-ids.tsv`, never committed.
//!
//! With `--classify-spans <file>` (claude-414) it also counts each shared 50-token span's
//! TRAIN record multiplicity, writes `<out>/classify-summary.json` (counts only) and decodes the
//! 200 shared spans held by the most DEV-test records to `<file>` (corpus text, never
//! committed).
//!
//! The span dedup (claude-416): `--write-drop <file>` writes the sorted ids of the kept TRAIN
//! records holding a 100-token DEV-test reasoning+answer span, one per line; `--drop-train
//! <file>` skips the listed records in the TRAIN pass (split v2) and refuses an id that is not a
//! kept TRAIN record. Each list's SHA-256 goes into the summary's `train_drop`; the lists hold
//! record ids and are never committed.

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::ExitCode;

use d87_split_audit::{audit_opts, plan, AuditOptions, Splitter, DROP_SPAN, SPANS};

/// The number of shared spans decoded by `--classify-spans`.
const CLASSIFY_TOP: usize = 200;
use m2_student_corpus_ingest::{hex, Coder};
use sha2::{Digest, Sha256};

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let (mut tok, mut split, mut out, mut threads, mut bloom, mut shards) =
        (None, None, None, 12usize, 30u32, vec![]);
    let mut classify: Option<PathBuf> = None;
    let (mut write_drop, mut drop_train): (Option<PathBuf>, Option<PathBuf>) = (None, None);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--classify-spans" => classify = Some(args.next().map(PathBuf::from).ok_or("--classify-spans")?),
            "--write-drop" => write_drop = Some(args.next().map(PathBuf::from).ok_or("--write-drop")?),
            "--drop-train" => drop_train = Some(args.next().map(PathBuf::from).ok_or("--drop-train")?),
            "--tokenizer" => tok = args.next().map(PathBuf::from),
            "--split" => split = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "--threads" => threads = args.next().and_then(|s| s.parse().ok()).ok_or("--threads")?,
            "--bloom-log2" => bloom = args.next().and_then(|s| s.parse().ok()).ok_or("--bloom-log2")?,
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
    let text = std::fs::read_to_string(&summary_path).map_err(|e| format!("{}: {e}", summary_path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let sp = Splitter::from_config_json(&v["config"])?;
    let p = plan(&sp, &shards, threads)?;
    let again = serde_json::to_string_pretty(&p.summary()).map_err(|e| e.to_string())? + "\n";
    if again != text {
        return Err("the shards do not reproduce split-summary.json".into());
    }
    let json = std::fs::read(&tok).map_err(|e| format!("{}: {e}", tok.display()))?;
    let coder = Coder::new(&json)?;
    let mut opts = AuditOptions {
        classify_top: classify.as_ref().map(|_| CLASSIFY_TOP),
        find_drop: write_drop.as_ref().map(|_| DROP_SPAN),
        ..AuditOptions::default()
    };
    let mut applied_sha = None;
    if let Some(file) = &drop_train {
        let list = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
        applied_sha = Some(hex(&Sha256::digest(&list)));
        let text = String::from_utf8(list).map_err(|e| format!("{}: {e}", file.display()))?;
        opts.drop = text.lines().map(str::to_string).collect::<HashSet<_>>();
        if opts.drop.is_empty() {
            return Err(format!("{}: the drop list is empty", file.display()));
        }
        if opts.drop.len() != text.lines().count() || opts.drop.contains("") {
            return Err(format!("{}: a drop id is empty or listed twice", file.display()));
        }
    }
    let (mut a, extras) = audit_opts(&p, &coder, threads, bloom, &opts)?;
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    if let Some(sha) = applied_sha {
        a["train_drop"]["applied_list_sha256"] = sha.into();
    }
    if let Some(file) = &write_drop {
        let list: String = extras.drop_found.iter().map(|id| format!("{id}\n")).collect();
        if let Some(dir) = file.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(file, &list).map_err(|e| format!("{}: {e}", file.display()))?;
        a["train_drop"]["found"]["list_sha256"] = hex(&Sha256::digest(list.as_bytes())).into();
        eprintln!("d87-audit: {} TRAIN records listed for drop in {}", extras.drop_found.len(), file.display());
    }
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    if let (Some(file), Some((mut cv, top))) = (&classify, extras.classify.clone()) {
        let mut tsv = String::from("rank\tdev_test_records\ttrain_records\tclass\ttext\n");
        for (rank, t) in top.iter().enumerate() {
            let text = String::from_utf8_lossy(&coder.tok.decode_bytes(&t.tokens)).replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t");
            let class = if t.copied { "copied" } else { "stock" };
            tsv.push_str(&format!("{}\t{}\t{}\t{class}\t{text}\n", rank + 1, t.dev_records, t.train_records));
        }
        if let Some(dir) = file.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(file, tsv).map_err(|e| format!("{}: {e}", file.display()))?;
        cv["tokenizer_sha256"] = hex(&Sha256::digest(&json)).into();
        cv["split_summary_sha256"] = hex(&Sha256::digest(text.as_bytes())).into();
        if let Some(drop) = a.get("train_drop") {
            cv["train_drop"] = drop.clone();
        }
        let body = serde_json::to_string_pretty(&cv).map_err(|e| e.to_string())? + "\n";
        std::fs::write(out.join("classify-summary.json"), body).map_err(|e| e.to_string())?;
        eprintln!("d87-audit: classify {}; {} spans decoded to {}", cv["bar"], top.len(), file.display());
    }
    // Text and ids stay beside the summary, never inside it: the summary is the committed file.
    let mut tsv = String::from("dev_test_prompt_positions\ttext\n");
    for (c, ids) in &extras.templates {
        let text = String::from_utf8_lossy(&coder.tok.decode_bytes(ids)).replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t");
        tsv.push_str(&format!("{c}\t{text}\n"));
    }
    std::fs::write(out.join("prompt-templates.tsv"), tsv).map_err(|e| e.to_string())?;
    let mut ids = String::new();
    for (n, list) in SPANS.iter().zip(&extras.contaminated) {
        for id in list {
            ids.push_str(&format!("{n}\t{id}\n"));
        }
    }
    std::fs::write(out.join("contaminated-ids.tsv"), ids).map_err(|e| e.to_string())?;
    a["tokenizer_sha256"] = hex(&Sha256::digest(&json)).into();
    a["split_summary_sha256"] = hex(&Sha256::digest(text.as_bytes())).into();
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let body = serde_json::to_string_pretty(&a).map_err(|e| e.to_string())? + "\n";
    std::fs::write(out.join("audit-summary.json"), body).map_err(|e| e.to_string())?;
    eprintln!("d87-audit: gate {}; written to {}", a["gate"], out.display());
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("d87-audit: {e}");
            ExitCode::FAILURE
        }
    }
}
