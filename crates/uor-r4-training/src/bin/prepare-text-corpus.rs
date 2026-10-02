//! Encode a public prose or dialogue text file into a UORT token store for
//! `geometric-stack train` (the scale ladder's language-model phase, #820).
//!
//! ```text
//! prepare-text-corpus out=NEW_DIR tokenizer=TOKENIZER.json input=FILE \
//!   format=tinystories|tinydialogues [max_docs=N]
//! ```
//!
//! Documents are split by the format's rule, each is encoded with the given
//! byte-level BPE (each worker thread holds its own tokenizer), checked to
//! decode back to its exact text, and written in input order, each followed
//! by the tokenizer's `<|eos|>`. A document that does not round-trip is
//! dropped and counted. `OUT/tokens.u16` is the store; `OUT/manifest.json`
//! binds the input, tokenizer and rule with SHA-256 digests.
//!
//! Formats:
//! - `tinystories`: stories separated by `<|endoftext|>`, each trimmed.
//! - `tinydialogues`: one conversation per line; the literal two-character
//!   sequence `\n` becomes a newline, speaker labels `**Name**:` become
//!   `Name:`, and each line is trimmed with blank lines collapsed.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use rayon::prelude::*;
use serde_json::json;
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::sha256_file;

type Error = Box<dyn std::error::Error>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("prepare-text-corpus: {error}");
            ExitCode::FAILURE
        }
    }
}

/// The documents of `text` under `format`.
fn documents(text: &str, format: &str) -> Result<Vec<String>, Error> {
    match format {
        "tinystories" => Ok(text
            .split("<|endoftext|>")
            .map(str::trim)
            .filter(|d| !d.is_empty())
            .map(str::to_owned)
            .collect()),
        "tinydialogues" => Ok(text
            .lines()
            .map(dialogue)
            .filter(|d| !d.is_empty())
            .collect()),
        other => Err(format!("format must be tinystories or tinydialogues, got {other}").into()),
    }
}

/// One TinyDialogues line as plain dialogue text.
fn dialogue(line: &str) -> String {
    let unescaped = line.replace("\\n", "\n");
    let mut lines = Vec::new();
    for raw in unescaped.lines() {
        let mut row = raw.trim().to_owned();
        if row.is_empty() {
            continue;
        }
        if let Some(rest) = row.strip_prefix("**") {
            if let Some(end) = rest.find("**:") {
                row = format!("{}:{}", &rest[..end], &rest[end + 3..]);
            }
        }
        lines.push(row);
    }
    lines.join("\n")
}

fn run() -> Result<(), Error> {
    let started = Instant::now();
    let mut args: BTreeMap<String, String> = BTreeMap::new();
    for argument in std::env::args().skip(1) {
        let (key, value) = argument
            .split_once('=')
            .ok_or_else(|| format!("expected key=value, got {argument}"))?;
        if !["out", "tokenizer", "input", "format", "max_docs"].contains(&key)
            || args.insert(key.to_owned(), value.to_owned()).is_some()
        {
            return Err(format!("unknown or repeated argument {key}").into());
        }
    }
    let get = |key: &str| {
        args.get(key)
            .cloned()
            .ok_or_else(|| format!("missing {key}="))
    };
    let (out, tokenizer_path, input, format) = (
        PathBuf::from(get("out")?),
        PathBuf::from(get("tokenizer")?),
        PathBuf::from(get("input")?),
        get("format")?,
    );
    let max_docs: usize = match args.get("max_docs") {
        Some(v) => v.parse().map_err(|_| format!("invalid max_docs={v}"))?,
        None => usize::MAX,
    };
    let tokenizer_json = fs::read(&tokenizer_path)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    let eos = tokenizer
        .token_id("<|eos|>")
        .ok_or("the tokenizer has no <|eos|>")?;
    let vocab = u32::try_from(tokenizer.vocab_size()).map_err(|_| "vocabulary too large")?;
    fs::create_dir(&out).map_err(|e| format!("{} must be new: {e}", out.display()))?;
    let text = fs::read_to_string(&input)?;
    let mut docs = documents(&text, &format)?;
    docs.truncate(max_docs);
    drop(text);
    let encoded: Vec<Option<Vec<u16>>> = docs
        .par_iter()
        .map_init(
            || ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json),
            |local, doc| {
                let local = local.as_ref()?;
                let ids = local.encode(doc);
                if String::from_utf8_lossy(&local.decode_bytes(&ids)) != doc.as_str() {
                    return None;
                }
                ids.iter().map(|&id| u16::try_from(id).ok()).collect()
            },
        )
        .collect();
    let tokens_path = out.join("tokens.u16");
    let mut writer = CorpusWriter::create(&tokens_path, vocab)?;
    let (mut kept, mut dropped, mut doc_tokens) = (0usize, 0usize, 0usize);
    for ids in encoded {
        match ids {
            Some(mut ids) => {
                doc_tokens += ids.len();
                ids.push(eos as u16);
                writer.write_tokens(&ids)?;
                kept += 1;
            }
            None => dropped += 1,
        }
    }
    let total = writer.finish()?;
    let manifest = json!({
        "schema": "uor-r4.text-corpus/1",
        "format": format,
        "input": input.display().to_string(),
        "input_sha256": sha256_file(&input)?,
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "vocab_size": vocab,
        "eos_id": eos,
        "documents_kept": kept,
        "documents_dropped_round_trip": dropped,
        "max_docs": if max_docs == usize::MAX { None } else { Some(max_docs) },
        "document_tokens": doc_tokens,
        "tokens": total,
        "tokens_sha256": sha256_file(&tokens_path)?,
        "rule": "documents split by the format's rule, each encoded and kept only if it decodes to its exact text, written in input order, each followed by <|eos|>",
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!("kept {kept} documents, dropped {dropped}; {total} tokens");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_split_and_clean_documents() {
        let stories = "One day.\n<|endoftext|>\n Two days. \n<|endoftext|>\n";
        assert_eq!(
            documents(stories, "tinystories").unwrap(),
            vec!["One day.", "Two days."]
        );
        let line = r#"**Dad**: "Hi!" \n\n **Child**: "Paint!""#;
        assert_eq!(dialogue(line), "Dad: \"Hi!\"\nChild: \"Paint!\"");
        assert!(documents("x", "other").is_err());
    }
}
