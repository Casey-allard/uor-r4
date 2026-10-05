//! Encode an open-domain knowledge corpus (Step 5 of the 5 October plan,
//! #820) into UORT token stores for `geometric-stack train`, decontaminated
//! against the open chat panel.
//!
//! ```text
//! prepare-knowledge-corpus out=NEW_ROOT tokenizer=TOKENIZER.json \
//!   input=FILE[|FILE...] format=jsonl|parquet source=LABEL licence=SPDX \
//!   panel=PANEL.json[,PANEL.json...] [min_words=20] [heldout_every=100] \
//!   [title=prepend|omit] [max_docs=N]
//! ```
//!
//! Input rows carry `text` and optionally `title` and `id` (the
//! `wikimedia/wikipedia` schema: `id`, `url`, `title`, `text`). `jsonl` reads
//! one JSON object per line; `parquet` (built only with `--features
//! parquet-input`) reads the parquet export directly. Files are read in the
//! order given as one row sequence.
//!
//! Each article becomes `title\n\nbody` (`title=prepend`, the default) with
//! line endings normalised and blank-line runs collapsed
//! (`knowledge_corpus::clean_article`). An article is dropped and counted when
//! it has fewer than `min_words` words, when any of its 8-word windows equals
//! an 8-word window of a panel user turn, or when it contains the whole word
//! sequence of a 4-7 word panel turn (`knowledge_corpus::PanelExclusion`), or
//! when its encoding does not decode back to its exact text. Every dropped
//! panel match is listed in `excluded.jsonl` (row index, id, title, rule,
//! matched words, panel request id).
//!
//! Kept articles are numbered in input order; article `k` goes to
//! `heldout/tokens.u16` when `k % heldout_every == heldout_every - 1`, else to
//! `train/tokens.u16` (`heldout_every=0` disables the split). Each article is
//! followed by `<|eos|>`. The held-out store measures knowledge NLL; it is
//! never trained on. `corpus.json` binds the inputs (sizes and SHA-256),
//! tokenizer, panel files, licence, rules and counts; the root is claimed
//! exclusively before any work and sealed (`manifest.json`) at the end.
//! Encoding runs on all cores in fixed-size batches; output order and bytes do
//! not depend on the thread count.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use rayon::prelude::*;
use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_core::report_output;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::knowledge_corpus::{
    clean_article, parse_panel, word_count, PanelExclusion, MIN_SHORT_REQUEST_WORDS, PANEL_NGRAM,
};
use uor_r4_training::sha256_file;

type Error = Box<dyn std::error::Error>;

const BATCH_ROWS: usize = 4096;
const KEYS: &[&str] = &[
    "out",
    "tokenizer",
    "input",
    "format",
    "source",
    "licence",
    "panel",
    "min_words",
    "heldout_every",
    "title",
    "max_docs",
];

/// One input row.
#[derive(Clone, Debug, Default)]
struct Article {
    id: Option<String>,
    title: Option<String>,
    text: String,
}

/// What became of one row.
enum Outcome {
    Kept(Vec<u16>),
    Short,
    Panel {
        rule: &'static str,
        words: String,
        request: String,
    },
    RoundTrip,
}

struct Config {
    out: PathBuf,
    tokenizer: PathBuf,
    inputs: Vec<PathBuf>,
    format: String,
    source: String,
    licence: String,
    panels: Vec<PathBuf>,
    min_words: usize,
    heldout_every: usize,
    prepend_title: bool,
    max_docs: usize,
}

fn parse_args(raw: &[String]) -> Result<Config, Error> {
    let mut args: BTreeMap<String, String> = BTreeMap::new();
    for argument in raw {
        let (key, value) = argument
            .split_once('=')
            .ok_or_else(|| format!("expected key=value, got {argument}"))?;
        if !KEYS.contains(&key) || args.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("unknown or repeated argument {key}").into());
        }
    }
    let get = |key: &str| -> Result<String, Error> {
        args.get(key)
            .cloned()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("missing {key}=").into())
    };
    let number = |key: &str, default: usize| -> Result<usize, Error> {
        args.get(key).map_or(Ok(default), |v| {
            v.parse()
                .map_err(|_| format!("{key}= must be a number, got {v}").into())
        })
    };
    let format = get("format")?;
    if format != "jsonl" && format != "parquet" {
        return Err(format!("format must be jsonl or parquet, got {format}").into());
    }
    if format == "parquet" && !cfg!(feature = "parquet-input") {
        return Err("format=parquet needs a build with --features parquet-input".into());
    }
    let prepend_title = match args.get("title").map(String::as_str) {
        None | Some("prepend") => true,
        Some("omit") => false,
        Some(other) => return Err(format!("title must be prepend or omit, got {other}").into()),
    };
    let inputs: Vec<PathBuf> = get("input")?.split('|').map(PathBuf::from).collect();
    let panels: Vec<PathBuf> = get("panel")?.split(',').map(PathBuf::from).collect();
    if inputs
        .iter()
        .chain(&panels)
        .any(|p| p.as_os_str().is_empty())
    {
        return Err("empty input or panel path".into());
    }
    Ok(Config {
        out: PathBuf::from(get("out")?),
        tokenizer: PathBuf::from(get("tokenizer")?),
        inputs,
        format,
        source: get("source")?,
        licence: get("licence")?,
        panels,
        min_words: number("min_words", 20)?,
        heldout_every: number("heldout_every", 100)?,
        prepend_title,
        max_docs: number("max_docs", usize::MAX)?,
    })
}

fn article_from_json(line: &str) -> Result<Article, Error> {
    let value: Value = serde_json::from_str(line)?;
    let text = value["text"]
        .as_str()
        .ok_or("a jsonl row has no string text")?
        .to_owned();
    let string = |key: &str| match &value[key] {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    };
    Ok(Article {
        id: string("id"),
        title: string("title"),
        text,
    })
}

#[cfg(feature = "parquet-input")]
fn article_from_parquet(row: &parquet::record::Row) -> Result<Article, Error> {
    use parquet::record::Field;
    let mut article = Article::default();
    let mut has_text = false;
    for (name, field) in row.get_column_iter() {
        let value = match field {
            Field::Str(s) => Some(s.clone()),
            Field::Null => None,
            Field::Long(v) => Some(v.to_string()),
            Field::Int(v) => Some(v.to_string()),
            _ => continue,
        };
        match name.as_str() {
            "text" => {
                article.text = value.unwrap_or_default();
                has_text = true;
            }
            "title" => article.title = value,
            "id" => article.id = value,
            _ => {}
        }
    }
    if !has_text {
        return Err("a parquet row has no text column".into());
    }
    Ok(article)
}

/// Stream every input row, in order, to `sink` in batches of [`BATCH_ROWS`];
/// stops after `max_docs` rows.
fn for_each_batch(
    config: &Config,
    mut sink: impl FnMut(&[Article]) -> Result<(), Error>,
) -> Result<usize, Error> {
    let mut batch: Vec<Article> = Vec::with_capacity(BATCH_ROWS);
    let mut rows = 0usize;
    for input in &config.inputs {
        if rows >= config.max_docs {
            break;
        }
        match config.format.as_str() {
            "jsonl" => {
                let reader = BufReader::new(File::open(input)?);
                for line in reader.lines() {
                    let line = line?;
                    if line.trim().is_empty() {
                        continue;
                    }
                    batch.push(
                        article_from_json(&line)
                            .map_err(|e| format!("{} row {rows}: {e}", input.display()))?,
                    );
                    rows += 1;
                    if batch.len() == BATCH_ROWS {
                        sink(&batch)?;
                        batch.clear();
                    }
                    if rows >= config.max_docs {
                        break;
                    }
                }
            }
            #[cfg(feature = "parquet-input")]
            "parquet" => {
                use parquet::file::reader::{FileReader, SerializedFileReader};
                let reader = SerializedFileReader::new(File::open(input)?)
                    .map_err(|e| format!("{}: {e}", input.display()))?;
                for row in reader
                    .get_row_iter(None)
                    .map_err(|e| format!("{}: {e}", input.display()))?
                {
                    let row = row.map_err(|e| format!("{}: {e}", input.display()))?;
                    batch.push(
                        article_from_parquet(&row)
                            .map_err(|e| format!("{} row {rows}: {e}", input.display()))?,
                    );
                    rows += 1;
                    if batch.len() == BATCH_ROWS {
                        sink(&batch)?;
                        batch.clear();
                    }
                    if rows >= config.max_docs {
                        break;
                    }
                }
            }
            other => return Err(format!("unsupported format {other}").into()),
        }
    }
    if !batch.is_empty() {
        sink(&batch)?;
    }
    Ok(rows)
}

fn process(
    tokenizer: &ByteBpeTokenizer,
    exclusion: &PanelExclusion,
    config: &Config,
    article: &Article,
) -> Outcome {
    let title = config
        .prepend_title
        .then_some(article.title.as_deref())
        .flatten();
    let text = clean_article(title, &article.text);
    if word_count(&text) < config.min_words {
        return Outcome::Short;
    }
    if let Some(hit) = exclusion.first_hit(&text) {
        return Outcome::Panel {
            rule: hit.kind.label(),
            words: hit.words,
            request: hit.request_id,
        };
    }
    let ids = tokenizer.encode(&text);
    if String::from_utf8_lossy(&tokenizer.decode_bytes(&ids)) != text.as_str() {
        return Outcome::RoundTrip;
    }
    match ids
        .iter()
        .map(|&id| u16::try_from(id))
        .collect::<Result<Vec<u16>, _>>()
    {
        Ok(ids) => Outcome::Kept(ids),
        Err(_) => Outcome::RoundTrip,
    }
}

#[derive(Default)]
struct Counts {
    rows: usize,
    kept_train: usize,
    kept_heldout: usize,
    dropped_short: usize,
    dropped_panel_8gram: usize,
    dropped_panel_short_request: usize,
    dropped_round_trip: usize,
    train_doc_tokens: usize,
    heldout_doc_tokens: usize,
}

fn input_identity(path: &Path) -> Result<Value, Error> {
    Ok(json!({
        "path": path.display().to_string(),
        "bytes": fs::metadata(path)?.len(),
        "sha256": sha256_file(path)?,
    }))
}

fn run(raw: &[String]) -> Result<(), Error> {
    let started = Instant::now();
    let config = parse_args(raw)?;
    for path in config.inputs.iter().chain(&config.panels) {
        if !path.is_file() {
            return Err(format!("{}: not a file", path.display()).into());
        }
    }
    let tokenizer_json = fs::read(&config.tokenizer)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    let eos = tokenizer
        .token_id("<|eos|>")
        .ok_or("the tokenizer has no <|eos|>")?;
    let eos = u16::try_from(eos).map_err(|_| "<|eos|> id above u16")?;
    let vocab = u32::try_from(tokenizer.vocab_size()).map_err(|_| "vocabulary too large")?;

    let mut requests = Vec::new();
    let mut panel_files = Vec::new();
    for panel in &config.panels {
        let rows =
            parse_panel(&fs::read(panel)?).map_err(|e| format!("{}: {e}", panel.display()))?;
        let mut identity = input_identity(panel)?;
        identity["requests"] = json!(rows.len());
        panel_files.push(identity);
        requests.extend(rows);
    }
    let exclusion = PanelExclusion::new(&requests);

    // Claim the root exclusively after argument and input validation and
    // before any output.
    report_output::claim(&config.out)?;
    fs::create_dir(config.out.join("train"))?;
    let train_path = config.out.join("train").join("tokens.u16");
    let mut train = CorpusWriter::create(&train_path, vocab)?;
    let heldout_path = config.out.join("heldout").join("tokens.u16");
    let mut heldout = if config.heldout_every > 0 {
        fs::create_dir(config.out.join("heldout"))?;
        Some(CorpusWriter::create(&heldout_path, vocab)?)
    } else {
        None
    };
    let mut excluded = BufWriter::new(File::create_new(config.out.join("excluded.jsonl"))?);

    let mut counts = Counts::default();
    let mut kept_index = 0usize;
    let rows = for_each_batch(&config, |batch| {
        let outcomes: Vec<Outcome> = batch
            .par_iter()
            .map_init(
                || ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json),
                |local, article| match local.as_ref() {
                    Some(local) => process(local, &exclusion, &config, article),
                    None => Outcome::RoundTrip,
                },
            )
            .collect();
        for (article, outcome) in batch.iter().zip(outcomes) {
            let row = counts.rows;
            counts.rows += 1;
            match outcome {
                Outcome::Kept(mut ids) => {
                    let doc_tokens = ids.len();
                    ids.push(eos);
                    let to_heldout = config.heldout_every > 0
                        && kept_index % config.heldout_every == config.heldout_every - 1;
                    kept_index += 1;
                    match (&mut heldout, to_heldout) {
                        (Some(writer), true) => {
                            writer.write_tokens(&ids)?;
                            counts.kept_heldout += 1;
                            counts.heldout_doc_tokens += doc_tokens;
                        }
                        _ => {
                            train.write_tokens(&ids)?;
                            counts.kept_train += 1;
                            counts.train_doc_tokens += doc_tokens;
                        }
                    }
                }
                Outcome::Short => counts.dropped_short += 1,
                Outcome::RoundTrip => counts.dropped_round_trip += 1,
                Outcome::Panel {
                    rule,
                    words,
                    request,
                } => {
                    if rule == "panel_8gram" {
                        counts.dropped_panel_8gram += 1;
                    } else {
                        counts.dropped_panel_short_request += 1;
                    }
                    let record = json!({
                        "row": row,
                        "id": article.id,
                        "title": article.title,
                        "rule": rule,
                        "words": words,
                        "panel_request": request,
                    });
                    serde_json::to_writer(&mut excluded, &record)?;
                    excluded.write_all(b"\n")?;
                }
            }
        }
        Ok(())
    })?;
    excluded.flush()?;
    drop(excluded);
    let train_tokens = train.finish()?;
    let heldout_tokens = match heldout {
        Some(writer) => Some(writer.finish()?),
        None => None,
    };
    if counts.kept_train == 0 {
        return Err("no article was kept for training".into());
    }
    if rows != counts.rows {
        return Err(format!("read {rows} rows but processed {}", counts.rows).into());
    }

    let inputs = config
        .inputs
        .iter()
        .map(|p| input_identity(p))
        .collect::<Result<Vec<_>, _>>()?;
    let corpus = json!({
        "schema": "uor-r4.knowledge-corpus/1",
        "source": config.source,
        "licence": config.licence,
        "format": config.format,
        "inputs": inputs,
        "tokenizer": {
            "path": config.tokenizer.display().to_string(),
            "sha256": sha256_file(&config.tokenizer)?,
            "vocab_size": vocab,
            "eos_id": eos,
        },
        "decontamination": {
            "rule": format!(
                "drop the whole article when any {PANEL_NGRAM}-word window of its normalized words (lowercased, split on characters other than letters, digits and apostrophes) equals a {PANEL_NGRAM}-word window of a panel user turn, or when it contains the whole word sequence of a panel user turn of {MIN_SHORT_REQUEST_WORDS}-{} words; hits are confirmed exactly",
                PANEL_NGRAM - 1
            ),
            "shared_with": "uor_r4_training::milestone_world_v2_probe::EXCLUSION_NGRAM (the M-world probe exclusion)",
            "panel_files": panel_files,
            "panel_requests": exclusion.requests,
            "panel_turns": exclusion.turns,
            "panel_turns_covered_by_8grams": exclusion.ngram_turns,
            "panel_turns_covered_whole": exclusion.short_turns,
            "panel_turns_uncovered_under_4_words": exclusion.uncovered_turns,
            "distinct_panel_8grams": exclusion.ngrams,
            "excluded_list": "excluded.jsonl",
        },
        "document_rule": if config.prepend_title {
            "title, a blank line, then the body with \\r\\n/\\r normalised, line ends trimmed and blank-line runs collapsed; kept only with at least min_words words and an exact decode round trip; each followed by <|eos|>"
        } else {
            "the body with \\r\\n/\\r normalised, line ends trimmed and blank-line runs collapsed; kept only with at least min_words words and an exact decode round trip; each followed by <|eos|>"
        },
        "min_words": config.min_words,
        "max_docs": (config.max_docs != usize::MAX).then_some(config.max_docs),
        "split_rule": format!(
            "kept articles numbered in input order; article k goes to heldout when heldout_every > 0 and k % heldout_every == heldout_every - 1 (heldout_every = {})",
            config.heldout_every
        ),
        "counts": {
            "rows": counts.rows,
            "kept_train": counts.kept_train,
            "kept_heldout": counts.kept_heldout,
            "dropped_short": counts.dropped_short,
            "dropped_panel_8gram": counts.dropped_panel_8gram,
            "dropped_panel_short_request": counts.dropped_panel_short_request,
            "dropped_round_trip": counts.dropped_round_trip,
        },
        "train": {
            "path": "train/tokens.u16",
            "document_tokens": counts.train_doc_tokens,
            "tokens": train_tokens,
            "sha256": sha256_file(&train_path)?,
        },
        "heldout": heldout_tokens.map(|tokens| -> Result<Value, Error> {
            Ok(json!({
                "path": "heldout/tokens.u16",
                "document_tokens": counts.heldout_doc_tokens,
                "tokens": tokens,
                "sha256": sha256_file(&heldout_path)?,
            }))
        }).transpose()?,
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(
        config.out.join("corpus.json"),
        serde_json::to_vec_pretty(&corpus)?,
    )?;
    report_output::seal(&config.out)?;
    report_output::verify(&config.out)?;
    println!(
        "{}: {} rows; kept {} train ({} tokens) + {} heldout ({} tokens); dropped short {}, panel 8-gram {}, panel short request {}, round trip {}",
        config.source,
        counts.rows,
        counts.kept_train,
        train_tokens,
        counts.kept_heldout,
        heldout_tokens.unwrap_or(0),
        counts.dropped_short,
        counts.dropped_panel_8gram,
        counts.dropped_panel_short_request,
        counts.dropped_round_trip
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("prepare-knowledge-corpus: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn arguments_parse_and_reject_malformed() {
        let base = [
            "out=o",
            "tokenizer=t.json",
            "input=a.jsonl|b.jsonl",
            "format=jsonl",
            "source=simplewiki",
            "licence=CC-BY-SA-4.0",
            "panel=p.json,q.json",
        ];
        let config = parse_args(&args(&base)).unwrap();
        assert_eq!(config.inputs.len(), 2);
        assert_eq!(config.panels.len(), 2);
        assert_eq!(config.min_words, 20);
        assert_eq!(config.heldout_every, 100);
        assert!(config.prepend_title);
        let mut bad = base.to_vec();
        bad.push("colour=red");
        assert!(parse_args(&args(&bad)).is_err());
        let mut repeated = base.to_vec();
        repeated.push("format=jsonl");
        assert!(parse_args(&args(&repeated)).is_err());
        assert!(parse_args(&args(&base[..6])).is_err(), "panel= is required");
        let mut format = base.to_vec();
        format[3] = "format=csv";
        assert!(parse_args(&args(&format)).is_err());
        let mut title = base.to_vec();
        title.push("title=upper");
        assert!(parse_args(&args(&title)).is_err());
    }

    #[test]
    fn jsonl_rows_accept_numeric_ids_and_require_text() {
        let article = article_from_json(r#"{"id":12,"title":"Bee","text":"Bees."}"#).unwrap();
        assert_eq!(article.id.as_deref(), Some("12"));
        assert_eq!(article.title.as_deref(), Some("Bee"));
        assert!(article_from_json(r#"{"title":"Bee"}"#).is_err());
    }

    #[cfg(feature = "parquet-input")]
    #[test]
    fn wikimedia_parquet_rows_are_read_in_order() {
        use parquet::data_type::{ByteArray, ByteArrayType};
        use parquet::file::properties::WriterProperties;
        use parquet::file::writer::SerializedFileWriter;
        use parquet::schema::parser::parse_message_type;
        use std::sync::Arc;

        let dir = std::env::temp_dir().join(format!(
            "uor-r4-knowledge-parquet-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("train.parquet");
        let schema = Arc::new(
            parse_message_type(
                "message schema { REQUIRED BYTE_ARRAY id (UTF8); REQUIRED BYTE_ARRAY url (UTF8); \
                 REQUIRED BYTE_ARRAY title (UTF8); REQUIRED BYTE_ARRAY text (UTF8); }",
            )
            .unwrap(),
        );
        let columns: [[&str; 2]; 4] = [
            ["1", "2"],
            ["u1", "u2"],
            ["April", "August"],
            ["April is a month.", "August is a month."],
        ];
        {
            let file = File::create(&path).unwrap();
            let mut writer = SerializedFileWriter::new(
                file,
                schema,
                Arc::new(WriterProperties::builder().build()),
            )
            .unwrap();
            let mut group = writer.next_row_group().unwrap();
            for values in columns {
                let mut column = group.next_column().unwrap().unwrap();
                let data: Vec<ByteArray> = values.iter().map(|v| ByteArray::from(*v)).collect();
                column
                    .typed::<ByteArrayType>()
                    .write_batch(&data, None, None)
                    .unwrap();
                column.close().unwrap();
            }
            group.close().unwrap();
            writer.close().unwrap();
        }
        let config = parse_args(&args(&[
            "out=unused",
            "tokenizer=t.json",
            &format!("input={}", path.display()),
            "format=parquet",
            "source=s",
            "licence=CC-BY-SA-4.0",
            "panel=p.json",
        ]))
        .unwrap();
        let mut seen = Vec::new();
        let rows = for_each_batch(&config, |batch| {
            seen.extend(batch.iter().cloned());
            Ok(())
        })
        .unwrap();
        assert_eq!(rows, 2);
        assert_eq!(seen[0].id.as_deref(), Some("1"));
        assert_eq!(seen[1].title.as_deref(), Some("August"));
        assert_eq!(seen[1].text, "August is a month.");
        fs::remove_dir_all(&dir).unwrap();
    }
}
