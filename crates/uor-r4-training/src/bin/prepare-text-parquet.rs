//! Stream one string column of Hugging Face parquet exports into a UORT token
//! store for `geometric-stack train` (a pretrain corpus, #820).
//!
//! ```text
//! prepare-text-parquet out=NEW_DIR tokenizer=TOKENIZER.json \
//!   input=FILE[|FILE...] [column=text] [max_docs=N] [max_tokens=N] [min_chars=0]
//! ```
//!
//! Each row's `column` is encoded with the byte-level BPE and written as the
//! ids followed by `<|eos|>`, exactly as `prepare-text-corpus` writes a
//! document. Rows that are null, empty or shorter than `min_chars` characters
//! are skipped and counted. Rows are read in fixed-size batches, encoded on
//! all cores and appended in row order, so the output bytes do not depend on
//! the thread count and the corpus is never held in memory. The run stops
//! after `max_docs` documents, or after the document that reaches `max_tokens`
//! tokens (documents are never cut). `OUT/manifest.json` uses the
//! `prepare-text-corpus` schema and binds each input file by SHA-256.
//!
//! Built only with `--features parquet-input`.

use std::fs::{self, File};
use std::path::PathBuf;

use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use rayon::prelude::*;
use serde_json::json;
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_core::report_output;
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

const BATCH_ROWS: usize = 4096;

struct Summary {
    documents: usize,
    tokens: u64,
}

fn number(args: &[String], key: &str) -> Result<Option<u64>> {
    args.iter()
        .find_map(|a| a.strip_prefix(&format!("{key}=")))
        .map(|v| v.parse().map_err(|_| format!("{key}= must be a number")))
        .transpose()
}

fn convert(args: &[String]) -> Result<Summary> {
    const KEYS: [&str; 7] = [
        "out",
        "tokenizer",
        "input",
        "column",
        "max_docs",
        "max_tokens",
        "min_chars",
    ];
    for argument in args {
        let key = argument.split_once('=').map(|(k, _)| k);
        if !key.is_some_and(|k| KEYS.contains(&k)) {
            return Err(format!("unknown argument {argument}"));
        }
    }
    let value = |key: &str| args.iter().find_map(|a| a.strip_prefix(&format!("{key}=")));
    let out = PathBuf::from(value("out").ok_or("missing out=")?);
    let tokenizer_path = PathBuf::from(value("tokenizer").ok_or("missing tokenizer=")?);
    let inputs: Vec<PathBuf> = value("input")
        .ok_or("missing input=")?
        .split('|')
        .map(PathBuf::from)
        .collect();
    let column = value("column").unwrap_or("text");
    let max_docs = number(args, "max_docs")?.map_or(usize::MAX, |n| n as usize);
    let max_tokens = number(args, "max_tokens")?.unwrap_or(u64::MAX);
    let min_chars = number(args, "min_chars")?.unwrap_or(0) as usize;
    for file in &inputs {
        if !file.is_file() {
            return Err(format!("{}: not a file", file.display()));
        }
    }
    let tokenizer_json = fs::read(&tokenizer_path).map_err(|e| e.to_string())?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    let eos = tokenizer
        .token_id("<|eos|>")
        .ok_or("the tokenizer has no <|eos|>")?;
    let eos = u16::try_from(eos).map_err(|_| "the <|eos|> id does not fit u16")?;
    let vocab = u32::try_from(tokenizer.vocab_size()).map_err(|_| "vocabulary too large")?;

    // Claim the root exclusively before writing anything into it.
    report_output::claim(&out).map_err(|e| e.to_string())?;
    let tokens_path = out.join("tokens.u16");
    let mut writer = CorpusWriter::create(&tokens_path, vocab).map_err(|e| e.to_string())?;

    let (mut documents, mut tokens) = (0usize, 0u64);
    let (mut rows_total, mut rows_skipped) = (0usize, 0usize);
    let mut sources = Vec::new();
    let mut batch: Vec<String> = Vec::with_capacity(BATCH_ROWS);
    let mut done = false;
    let flush = |batch: &mut Vec<String>,
                 writer: &mut CorpusWriter,
                 documents: &mut usize,
                 tokens: &mut u64|
     -> Result<bool> {
        let encoded: Vec<Vec<u32>> = batch.par_iter().map(|doc| tokenizer.encode(doc)).collect();
        batch.clear();
        for ids in encoded {
            let mut row = ids
                .iter()
                .map(|&id| u16::try_from(id))
                .collect::<std::result::Result<Vec<u16>, _>>()
                .map_err(|_| "a token id does not fit u16")?;
            row.push(eos);
            writer.write_tokens(&row).map_err(|e| e.to_string())?;
            *documents += 1;
            *tokens += row.len() as u64;
            if *documents >= max_docs || *tokens >= max_tokens {
                return Ok(true);
            }
        }
        Ok(false)
    };
    'files: for file in &inputs {
        let reader = SerializedFileReader::new(File::open(file).map_err(|e| e.to_string())?)
            .map_err(|e| format!("{}: {e}", file.display()))?;
        sources.push(json!({
            "file": file.file_name().map(|n| n.to_string_lossy().into_owned()),
            "path": file.display().to_string(),
            "bytes": fs::metadata(file).map_err(|e| e.to_string())?.len(),
            "sha256": uor_r4_training::sha256_file(file).map_err(|e| e.to_string())?,
        }));
        for row in reader
            .get_row_iter(None)
            .map_err(|e| format!("{}: {e}", file.display()))?
        {
            let row = row.map_err(|e| format!("{}: {e}", file.display()))?;
            rows_total += 1;
            let (_, field) = row
                .get_column_iter()
                .find(|(name, _)| name.as_str() == column)
                .ok_or_else(|| format!("a row has no {column} column"))?;
            match field {
                Field::Str(text) if !text.is_empty() && text.chars().count() >= min_chars => {
                    batch.push(text.clone());
                }
                Field::Str(_) | Field::Null => rows_skipped += 1,
                _ => return Err(format!("the {column} column is not a string")),
            }
            if batch.len() == BATCH_ROWS {
                done = flush(&mut batch, &mut writer, &mut documents, &mut tokens)?;
                if done {
                    break 'files;
                }
            }
        }
    }
    if !done {
        flush(&mut batch, &mut writer, &mut documents, &mut tokens)?;
    }
    let total = writer.finish().map_err(|e| e.to_string())?;
    if total != tokens {
        return Err(format!("wrote {total} tokens, expected {tokens}"));
    }
    let manifest = json!({
        "schema": "uor-r4.text-corpus/1",
        "format": "parquet-text",
        "column": column,
        "sources": sources,
        "tokenizer": {
            "path": tokenizer_path.display().to_string(),
            "sha256": uor_r4_training::sha256_file(&tokenizer_path).map_err(|e| e.to_string())?,
        },
        "vocab_size": vocab,
        "eos_id": eos,
        "documents_kept": documents,
        "rows_total": rows_total,
        "rows_skipped_empty_or_short": rows_skipped,
        "max_docs": if max_docs == usize::MAX { None } else { Some(max_docs) },
        "max_tokens": if max_tokens == u64::MAX { None } else { Some(max_tokens) },
        "min_chars": min_chars,
        "tokens": total,
        "tokens_sha256": uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        "rule": "each row's column encoded and written in row order, followed by <|eos|>; null, empty and shorter-than-min_chars rows skipped; stops after the document that reaches max_docs or max_tokens",
    });
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(Summary {
        documents,
        tokens: total,
    })
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match convert(&args) {
        Ok(summary) => println!("{} documents, {} tokens", summary.documents, summary.tokens),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parquet::data_type::{ByteArray, ByteArrayType};
    use parquet::file::properties::WriterProperties;
    use parquet::file::writer::SerializedFileWriter;
    use parquet::schema::parser::parse_message_type;
    use std::sync::Arc;
    use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;

    const TOKENIZER: &str = r#"{
        "pre_tokenizer": {"type":"ByteLevel", "add_prefix_space":false},
        "added_tokens":[{"id":8,"content":"<|eos|>"}],
        "model":{"type":"BPE",
            "vocab":{"a":0,"b":1,"c":2,"ab":3,"bc":4,"Ã":5,"©":6,"Ġ":7},
            "merges":["b c","a b"]}
    }"#;

    fn write_parquet(path: &std::path::Path, rows: &[&str]) {
        let schema =
            Arc::new(parse_message_type("message m { required binary text (STRING); }").unwrap());
        let props = Arc::new(WriterProperties::builder().build());
        let mut writer =
            SerializedFileWriter::new(File::create(path).unwrap(), schema, props).unwrap();
        let mut group = writer.next_row_group().unwrap();
        let mut column = group.next_column().unwrap().unwrap();
        let values: Vec<ByteArray> = rows.iter().map(|r| ByteArray::from(*r)).collect();
        column
            .typed::<ByteArrayType>()
            .write_batch(&values, None, None)
            .unwrap();
        column.close().unwrap();
        group.close().unwrap();
        writer.close().unwrap();
    }

    fn run(dir: &std::path::Path, name: &str, extra: &[&str]) -> (Summary, Vec<u16>) {
        let out = dir.join(name);
        let mut args = vec![
            format!("out={}", out.display()),
            format!("tokenizer={}", dir.join("tokenizer.json").display()),
            format!("input={}", dir.join("in.parquet").display()),
        ];
        args.extend(extra.iter().map(|s| (*s).to_owned()));
        let summary = convert(&args).unwrap();
        let tokens = MmapCorpusReader::open(out.join("tokens.u16"))
            .unwrap()
            .as_slice()
            .to_vec();
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest["tokens"], summary.tokens);
        assert_eq!(manifest["sources"][0]["file"], "in.parquet");
        (summary, tokens)
    }

    #[test]
    fn rows_become_tokens_then_eos_and_limits_stop_early() {
        let dir = std::env::temp_dir().join(format!("prepare-text-parquet-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("tokenizer.json"), TOKENIZER).unwrap();
        write_parquet(&dir.join("in.parquet"), &["abc", "bca", "ab"]);
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(TOKENIZER.as_bytes()).unwrap();
        let expected = |rows: &[&str]| -> Vec<u16> {
            rows.iter()
                .flat_map(|r| {
                    let mut ids: Vec<u16> = tokenizer.encode(r).iter().map(|&i| i as u16).collect();
                    ids.push(8);
                    ids
                })
                .collect()
        };

        let (summary, tokens) = run(&dir, "all", &[]);
        assert_eq!(summary.documents, 3);
        assert_eq!(tokens, expected(&["abc", "bca", "ab"]));

        let (summary, tokens) = run(&dir, "two", &["max_docs=2"]);
        assert_eq!(summary.documents, 2);
        assert_eq!(tokens, expected(&["abc", "bca"]));

        let (summary, tokens) = run(&dir, "short", &["min_chars=3"]);
        assert_eq!(summary.documents, 2);
        assert_eq!(tokens, expected(&["abc", "bca"]));

        let (_, tokens) = run(&dir, "again", &[]);
        assert_eq!(tokens, expected(&["abc", "bca", "ab"]));
        fs::remove_dir_all(&dir).unwrap();
    }
}
