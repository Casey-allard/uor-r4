//! Prepare a literal-role dialogue split (`uor-r4-chat-corpus/v1`) directly
//! from Hugging Face parquet exports whose rows carry a `messages` column
//! (`list<struct<content: string, role: string>>`), e.g. SmolTalk and
//! UltraChat 200k.
//!
//! ```text
//! prepare-chat-parquet out=NEW_DIR tokenizer=TOKENIZER.json \
//!   source=LABEL:LICENCE:SKIP:FILE[|FILE...] [source=...] \
//!   [protocol=2] [max_tokens=8192] [vocab_size=4096]
//! ```
//!
//! Each `source` reads its files in the order given as one row sequence and
//! skips its first `SKIP` rows. chat-v0 was fetched from row 0 of the same
//! splits, so a `SKIP` past chat-v0's last fetched row keeps the new rows
//! disjoint from chat-v0's train and held-out documents (and the chat panel
//! drawn from that held-out split). Documents are encoded with the dialogue
//! protocol and appended in row order; rows are dropped and counted when
//! empty, without an assistant response, longer than `max_tokens`, or holding
//! literal special-token text (the trainer refuses a store with any).
//! Encoding runs on all cores in fixed-size batches; output order and bytes
//! do not depend on the thread count.
//!
//! Built only with `--features parquet-input`.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::{Field, Row};
use rayon::prelude::*;
use serde_json::json;
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

const BATCH_ROWS: usize = 4096;

struct Source {
    label: String,
    licence: String,
    skip: usize,
    files: Vec<PathBuf>,
}

fn parse_source(spec: &str) -> Result<Source> {
    let mut parts = spec.splitn(4, ':');
    let (Some(label), Some(licence), Some(skip), Some(files)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(format!(
            "source={spec}: expected LABEL:LICENCE:SKIP:FILE[|FILE...]"
        ));
    };
    let skip = skip
        .parse()
        .map_err(|_| format!("source={spec}: SKIP must be a number"))?;
    let files: Vec<PathBuf> = files.split('|').map(PathBuf::from).collect();
    if label.is_empty() || licence.is_empty() || files.iter().any(|f| f.as_os_str().is_empty()) {
        return Err(format!("source={spec}: empty label, licence or file"));
    }
    Ok(Source {
        label: label.to_owned(),
        licence: licence.to_owned(),
        skip,
        files,
    })
}

/// The `(role, content)` pairs of a row's `messages` column. A missing or
/// non-list column is an error: the source is not the expected schema.
fn messages(row: &Row) -> Result<Vec<(String, String)>> {
    let (_, field) = row
        .get_column_iter()
        .find(|(name, _)| name.as_str() == "messages")
        .ok_or("a row has no messages column")?;
    let Field::ListInternal(list) = field else {
        return Err("the messages column is not a list".into());
    };
    let mut out = Vec::with_capacity(list.elements().len());
    for element in list.elements() {
        let Field::Group(message) = element else {
            return Err("a message is not a struct".into());
        };
        let mut role = String::new();
        let mut content = String::new();
        for (name, value) in message.get_column_iter() {
            let text = match value {
                Field::Str(s) => s.clone(),
                Field::Null => String::new(),
                _ => return Err(format!("message field {name} is not a string")),
            };
            match name.as_str() {
                "role" => role = text,
                "content" => content = text,
                _ => {}
            }
        }
        out.push((role, content));
    }
    Ok(out)
}

#[derive(Default)]
struct Counts {
    rows_total: usize,
    rows_skipped_prefix: usize,
    rows_used: usize,
    rows_dropped_empty: usize,
    rows_dropped_no_response: usize,
    rows_dropped_oversized: usize,
    rows_dropped_special: usize,
    messages_skipped_empty: usize,
    messages_skipped_unknown_role: usize,
    tokens: usize,
    response_tokens: usize,
}

enum Outcome {
    Kept(Vec<u16>, Vec<u8>),
    Empty,
    NoResponse,
    Oversized,
    Special,
    /// A token id above u16 or a token/mask length mismatch: an encoder
    /// defect, which stops the run instead of being counted as a drop.
    Defect,
}

fn encode_row(
    encoder: &DialogueEncoder<'_>,
    row: &[(String, String)],
    max_tokens: usize,
) -> (Outcome, usize, usize) {
    let messages: Vec<Message<'_>> = row
        .iter()
        .map(|(role, content)| Message {
            role: role.as_str(),
            content: content.as_str(),
        })
        .collect();
    let encoded = encoder.encode_document(&messages);
    let skipped = (encoded.skipped_empty, encoded.skipped_unknown_role);
    let outcome = if encoded.emitted_turns == 0 {
        Outcome::Empty
    } else if encoded.special_token_occurrences > 0 {
        Outcome::Special
    } else if !encoded.response_mask.contains(&1) {
        Outcome::NoResponse
    } else if encoded.tokens.len() > max_tokens {
        Outcome::Oversized
    } else {
        match encoded
            .tokens
            .iter()
            .map(|&t| u16::try_from(t))
            .collect::<std::result::Result<Vec<u16>, _>>()
        {
            Ok(ids) if ids.len() == encoded.response_mask.len() => {
                Outcome::Kept(ids, encoded.response_mask)
            }
            _ => Outcome::Defect,
        }
    };
    (outcome, skipped.0, skipped.1)
}

fn run(args: &[String]) -> Result<()> {
    let value = |key: &str| args.iter().find_map(|a| a.strip_prefix(&format!("{key}=")));
    let number = |key: &str, default: u64| -> Result<u64> {
        value(key).map_or(Ok(default), |v| {
            v.parse().map_err(|_| format!("{key}= must be a number"))
        })
    };
    let out = PathBuf::from(value("out").ok_or("missing out=")?);
    let tokenizer_path = PathBuf::from(value("tokenizer").ok_or("missing tokenizer=")?);
    let sources: Vec<Source> = args
        .iter()
        .filter_map(|a| a.strip_prefix("source="))
        .map(parse_source)
        .collect::<Result<_>>()?;
    if sources.is_empty() {
        return Err("at least one source= is required".into());
    }
    let max_tokens = number("max_tokens", 8192)? as usize;
    let vocab_size =
        u32::try_from(number("vocab_size", 4096)?).map_err(|_| "vocab_size too large")?;
    let version = u8::try_from(number("protocol", 2)?).map_err(|_| "protocol must be 1 or 2")?;
    for source in &sources {
        for file in &source.files {
            if !file.is_file() {
                return Err(format!("{}: not a file", file.display()));
            }
        }
    }

    let tokenizer_json = fs::read(&tokenizer_path).map_err(|e| e.to_string())?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    let protocol =
        DialogueProtocol::literal_roles_version(&tokenizer, version).map_err(|e| e.to_string())?;
    let encoder: DialogueEncoder<'_> = protocol.bind(&tokenizer).map_err(|e| e.to_string())?;

    // Claim the root exclusively before writing anything into it.
    report_output::claim(&out).map_err(|e| e.to_string())?;
    let tokens_path = out.join("tokens.u16");
    let mask_path = out.join("response_mask.u8");
    let mut writer = CorpusWriter::create(&tokens_path, vocab_size).map_err(|e| e.to_string())?;
    let mut mask = BufWriter::new(File::create(&mask_path).map_err(|e| e.to_string())?);

    let mut all = Counts::default();
    let mut file_entries = Vec::new();
    for source in &sources {
        let mut counts = Counts::default();
        let mut input_files = Vec::new();
        let mut batch: Vec<Vec<(String, String)>> = Vec::with_capacity(BATCH_ROWS);
        let flush = |batch: &mut Vec<Vec<(String, String)>>,
                     counts: &mut Counts,
                     writer: &mut CorpusWriter,
                     mask: &mut BufWriter<File>|
         -> Result<()> {
            let encoded: Vec<(Outcome, usize, usize)> = batch
                .par_iter()
                .map(|row| encode_row(&encoder, row, max_tokens))
                .collect();
            for (outcome, empty, unknown) in encoded {
                counts.messages_skipped_empty += empty;
                counts.messages_skipped_unknown_role += unknown;
                match outcome {
                    Outcome::Kept(ids, row_mask) => {
                        writer.write_tokens(&ids).map_err(|e| e.to_string())?;
                        mask.write_all(&row_mask).map_err(|e| e.to_string())?;
                        counts.rows_used += 1;
                        counts.tokens += ids.len();
                        counts.response_tokens += row_mask.iter().filter(|&&m| m == 1).count();
                    }
                    Outcome::Empty => counts.rows_dropped_empty += 1,
                    Outcome::NoResponse => counts.rows_dropped_no_response += 1,
                    Outcome::Oversized => counts.rows_dropped_oversized += 1,
                    Outcome::Special => counts.rows_dropped_special += 1,
                    Outcome::Defect => {
                        return Err("the encoder produced a token id above u16 or a token/mask \
                                    length mismatch"
                            .into())
                    }
                }
            }
            batch.clear();
            Ok(())
        };
        for file in &source.files {
            let reader = SerializedFileReader::new(File::open(file).map_err(|e| e.to_string())?)
                .map_err(|e| format!("{}: {e}", file.display()))?;
            let file_rows = usize::try_from(reader.metadata().file_metadata().num_rows())
                .map_err(|_| "negative row count")?;
            input_files.push(json!({
                "path": file.display().to_string(),
                "bytes": fs::metadata(file).map_err(|e| e.to_string())?.len(),
                "sha256": uor_r4_training::sha256_file(file).map_err(|e| e.to_string())?,
                "rows": file_rows,
            }));
            for row in reader
                .get_row_iter(None)
                .map_err(|e| format!("{}: {e}", file.display()))?
            {
                let row = row.map_err(|e| format!("{}: {e}", file.display()))?;
                counts.rows_total += 1;
                if counts.rows_total <= source.skip {
                    counts.rows_skipped_prefix += 1;
                    continue;
                }
                batch.push(messages(&row)?);
                if batch.len() == BATCH_ROWS {
                    flush(&mut batch, &mut counts, &mut writer, &mut mask)?;
                }
            }
        }
        flush(&mut batch, &mut counts, &mut writer, &mut mask)?;
        if counts.rows_total < source.skip {
            return Err(format!(
                "{}: {} rows, fewer than SKIP={}",
                source.label, counts.rows_total, source.skip
            ));
        }
        println!(
            "{}: {} rows read, {} skipped (prefix), {} used, {} tokens ({} response); dropped \
             empty {}, no response {}, oversized {}, special {}",
            source.label,
            counts.rows_total,
            counts.rows_skipped_prefix,
            counts.rows_used,
            counts.tokens,
            counts.response_tokens,
            counts.rows_dropped_empty,
            counts.rows_dropped_no_response,
            counts.rows_dropped_oversized,
            counts.rows_dropped_special
        );
        file_entries.push(json!({
            "label": source.label,
            "licence": source.licence,
            "skip_rows": source.skip,
            "input_files": input_files,
            "tokens": counts.tokens,
            "response_tokens": counts.response_tokens,
            "special_token_occurrences": 0,
            "rows_total": counts.rows_total,
            "rows_skipped_prefix": counts.rows_skipped_prefix,
            "rows_used": counts.rows_used,
            "rows_dropped_empty": counts.rows_dropped_empty,
            "rows_dropped_no_response": counts.rows_dropped_no_response,
            "rows_dropped_oversized": counts.rows_dropped_oversized,
            "rows_dropped_special_text": counts.rows_dropped_special,
            "messages_skipped_empty": counts.messages_skipped_empty,
            "messages_skipped_unknown_role": counts.messages_skipped_unknown_role,
        }));
        all.rows_total += counts.rows_total;
        all.rows_used += counts.rows_used;
        all.rows_dropped_empty += counts.rows_dropped_empty;
        all.rows_dropped_no_response += counts.rows_dropped_no_response;
        all.rows_dropped_oversized += counts.rows_dropped_oversized;
        all.rows_dropped_special += counts.rows_dropped_special;
        all.messages_skipped_empty += counts.messages_skipped_empty;
        all.messages_skipped_unknown_role += counts.messages_skipped_unknown_role;
        all.tokens += counts.tokens;
        all.response_tokens += counts.response_tokens;
    }
    let written = writer.finish().map_err(|e| e.to_string())?;
    mask.flush().map_err(|e| e.to_string())?;
    drop(mask);
    let mask_bytes = fs::metadata(&mask_path).map_err(|e| e.to_string())?.len();
    // The trainer reads tokens and mask as parallel arrays; any disagreement
    // would misalign every later document.
    if written as usize != all.tokens || mask_bytes as usize != all.tokens || all.tokens == 0 {
        return Err(format!(
            "wrote {written} tokens and {mask_bytes} mask bytes, expected {} of each (non-zero)",
            all.tokens
        ));
    }

    let manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "split": "train",
        "template_rule": "<|bos|> then turns joined by a single '\\n' separator (placed before every turn after the first); a turn is '<marker><content>' with markers 'System: ', 'User: ', 'Assistant: '; every assistant turn ends with <|eos|>; a document-terminal <|eos|> is appended only when the final emitted turn is not assistant. Content is \\r\\n/\\r-normalised and trimmed; interior whitespace preserved.",
        "mask_rule": "1 = each token of an assistant turn's response content and its terminating <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, and an unmasked document-terminal <|eos|>.",
        "drops": {
            "rows_dropped_no_messages": 0,
            "rows_dropped_empty": all.rows_dropped_empty,
            "rows_dropped_no_response": all.rows_dropped_no_response,
            "rows_dropped_oversized": all.rows_dropped_oversized,
            "rows_dropped_special_text": all.rows_dropped_special,
            "messages_skipped_empty": all.messages_skipped_empty,
            "messages_skipped_unknown_role": all.messages_skipped_unknown_role,
            "special_token_occurrences": 0
        },
        "files": file_entries,
        "generator": "prepare-chat-parquet",
        "disjointness_rule": "each source skips its first skip_rows rows (row order = parquet files in the order given), past the rows chat-v0 fetched from row 0 of the same split",
        "max_tokens": max_tokens,
        "tokenizer": {"path": tokenizer_path.display().to_string(), "sha256": uor_r4_training::sha256_file(&tokenizer_path).map_err(|e| e.to_string())?},
        "dialogue_protocol": protocol.schema,
        "bos_id": protocol.bos_id,
        "eos_id": protocol.eos_id,
        "rows_total": all.rows_total,
        "rows_used": all.rows_used,
        "tokens": all.tokens,
        "response_tokens": all.response_tokens,
        "response_fraction": all.response_tokens as f64 / all.tokens as f64,
        "tokens_bytes": fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len(),
        "mask_bytes": mask_bytes,
        "tokens_sha256": uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        "mask_sha256": uor_r4_training::sha256_file(&mask_path).map_err(|e| e.to_string())?,
    });
    fs::write(
        out.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "{} rows used, {} tokens, {} response tokens ({:.1}%)",
        all.rows_used,
        all.tokens,
        all.response_tokens,
        100.0 * all.response_tokens as f64 / all.tokens as f64
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_spec_parses_and_rejects_malformed() {
        let source = parse_source("magpie:Apache-2.0:13900:a.parquet|b.parquet").unwrap();
        assert_eq!(source.label, "magpie");
        assert_eq!(source.licence, "Apache-2.0");
        assert_eq!(source.skip, 13900);
        assert_eq!(source.files.len(), 2);
        assert!(parse_source("magpie:Apache-2.0:x:a.parquet").is_err());
        assert!(parse_source("magpie:Apache-2.0:5").is_err());
        assert!(parse_source("magpie::5:a.parquet").is_err());
        assert!(parse_source("magpie:MIT:5:a.parquet|").is_err());
    }
}
