//! Prepare a human-written real-text corpus (`uor-r4-chat-corpus/v1`) from
//! OASST2 conversation trees or Wikipedia parquet shards, with a held-out
//! split by source id and the frozen panels' user turns excluded.
//!
//! ```text
//! prepare-real-text <oasst|wiki> out=NEW_DIR tokenizer=TOKENIZER.json \
//!   input=FILE[,FILE...] panel=PANEL.json[,PANEL.json...] \
//!   source=STR revision=STR licence=STR attribution=STR heldout_per_mille=K \
//!   [protocol=2] [vocab_size=N] [max_tokens=8192] [max_docs=N]
//! prepare-real-text panel-turns split=DIR tokenizer=TOKENIZER.json name=NAME out=NEW_FILE
//! ```
//!
//! - `panel-turns` reads a frozen panel's user turns back from its split
//!   (`tokens.u16`, `response_mask.u8`) with [`real_text::split_user_turns`],
//!   and writes them in the `{id, user_turns}` form `panel=` reads, with ids
//!   `NAME:<document index>`. It checks the same tokenizer pin first.
//!
//! - `oasst` reads the `trees.jsonl` export, one tree per line, and keeps each
//!   tree's best-ranked thread ([`real_text::walk_tree`]).
//! - `wiki` reads `wikimedia/wikipedia` parquet rows (`id`, `title`, `text`);
//!   a column chunk compressed with a codec other than UNCOMPRESSED, SNAPPY or
//!   ZSTD is refused by name before the output root is claimed.
//!
//! The tokenizer's SHA-256 must equal [`real_text::PINNED_TOKENIZER_SHA256`],
//! and `vocab_size`, the tokenizer's id count when omitted, must equal it;
//! either mismatch is refused before the output root is claimed. With
//! `max_docs`, the run stops once that many documents are kept, and the
//! manifest's source counts cover only the input read up to then
//! (`totals.stopped_at_max_docs`). Every document is
//! built by [`real_text::DocumentBuilder`] (panel exclusion, special text,
//! round trip, length) and written to `out/heldout/` when
//! [`real_text::heldout`] holds for its source id, else to `out/train/`. Each
//! split has `tokens.u16`, `response_mask.u8` and `manifest.json` in the
//! layout `prepare-chat-parquet` writes; every excluded document is listed in
//! `out/excluded.jsonl`. Encoding runs on all cores in fixed-size batches;
//! output order and bytes do not depend on the thread count.
//!
//! Built only with `--features parquet-input`.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use parquet::basic::Compression;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::{Field, Row};
use rayon::prelude::*;
use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::real_text::{
    self, Built, DocDrop, DocumentBuilder, HitKind, NodeDrop, OasstTree, PanelExclusion,
    PanelRequest, Thread, TreeOutcome,
};

type Result<T> = std::result::Result<T, String>;

const BATCH: usize = 4096;
const EXCLUDED_TEXT_CHARS: usize = 200;
const KEYS: [&str; 13] = [
    "out",
    "tokenizer",
    "input",
    "panel",
    "source",
    "revision",
    "licence",
    "attribution",
    "heldout_per_mille",
    "protocol",
    "vocab_size",
    "max_tokens",
    "max_docs",
];
const REQUIRED: [&str; 9] = [
    "out",
    "tokenizer",
    "input",
    "panel",
    "source",
    "revision",
    "licence",
    "attribution",
    "heldout_per_mille",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Oasst,
    Wiki,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Self::Oasst => "oasst",
            Self::Wiki => "wiki",
        }
    }
}

struct Config {
    mode: Mode,
    out: PathBuf,
    tokenizer: PathBuf,
    inputs: Vec<PathBuf>,
    panels: Vec<PathBuf>,
    source: String,
    revision: String,
    licence: String,
    attribution: String,
    heldout_per_mille: u64,
    protocol: u8,
    vocab_size: Option<u32>,
    max_tokens: usize,
    max_docs: Option<usize>,
}

/// `key=value` arguments, each key one of `keys`, given once and non-empty,
/// with every key of `required` present.
fn key_values<'a>(
    args: &'a [String],
    keys: &[&'static str],
    required: &[&'static str],
) -> Result<BTreeMap<&'static str, &'a str>> {
    let mut values = BTreeMap::new();
    for arg in args {
        let (key, value) = arg
            .split_once('=')
            .ok_or_else(|| format!("{arg:?}: expected key=value"))?;
        let key = keys
            .iter()
            .copied()
            .find(|k| *k == key)
            .ok_or_else(|| format!("unknown argument {key}="))?;
        if value.is_empty() {
            return Err(format!("{key}= is empty"));
        }
        if values.insert(key, value).is_some() {
            return Err(format!("{key}= given twice"));
        }
    }
    for key in required {
        if !values.contains_key(key) {
            return Err(format!("missing {key}="));
        }
    }
    Ok(values)
}

fn parse_args(args: &[String]) -> Result<Config> {
    let (mode, rest) = args
        .split_first()
        .ok_or("missing mode: oasst, wiki or panel-turns")?;
    let mode = match mode.as_str() {
        "oasst" => Mode::Oasst,
        "wiki" => Mode::Wiki,
        other => {
            return Err(format!(
                "unknown mode {other:?}: expected oasst, wiki or panel-turns"
            ))
        }
    };
    let values = key_values(rest, &KEYS, &REQUIRED)?;
    let number = |key: &str, default: u64| -> Result<u64> {
        values.get(key).map_or(Ok(default), |v| {
            v.parse().map_err(|_| format!("{key}= must be a number"))
        })
    };
    let paths = |key: &str| -> Result<Vec<PathBuf>> {
        let list: Vec<PathBuf> = values[key].split(',').map(PathBuf::from).collect();
        if list.iter().any(|p| p.as_os_str().is_empty()) {
            return Err(format!("{key}= holds an empty path"));
        }
        Ok(list)
    };
    let heldout_per_mille = number("heldout_per_mille", 0)?;
    if heldout_per_mille > 1000 {
        return Err("heldout_per_mille= must be 0..=1000".into());
    }
    let protocol = match number("protocol", 2)? {
        1 => 1,
        2 => 2,
        _ => return Err("protocol= must be 1 or 2".into()),
    };
    let vocab_size = match values.get("vocab_size") {
        None => None,
        Some(_) => {
            Some(u32::try_from(number("vocab_size", 0)?).map_err(|_| "vocab_size= too large")?)
        }
    };
    let max_tokens =
        usize::try_from(number("max_tokens", 8192)?).map_err(|_| "max_tokens= too large")?;
    if max_tokens == 0 {
        return Err("max_tokens= must be positive".into());
    }
    let max_docs = match values.get("max_docs") {
        None => None,
        Some(_) => Some(
            usize::try_from(number("max_docs", 0)?)
                .ok()
                .filter(|n| *n > 0)
                .ok_or("max_docs= must be positive")?,
        ),
    };
    Ok(Config {
        mode,
        out: PathBuf::from(values["out"]),
        tokenizer: PathBuf::from(values["tokenizer"]),
        inputs: paths("input")?,
        panels: paths("panel")?,
        source: values["source"].to_owned(),
        revision: values["revision"].to_owned(),
        licence: values["licence"].to_owned(),
        attribution: values["attribution"].to_owned(),
        heldout_per_mille,
        protocol,
        vocab_size,
        max_tokens,
        max_docs,
    })
}

/// A codec this build can read; anything else is refused by name.
fn codec_supported(codec: Compression) -> std::result::Result<(), String> {
    match codec {
        Compression::UNCOMPRESSED | Compression::SNAPPY | Compression::ZSTD(_) => Ok(()),
        other => Err(other.to_string()),
    }
}

fn open_shard(path: &Path) -> Result<SerializedFileReader<File>> {
    SerializedFileReader::new(File::open(path).map_err(|e| format!("{}: {e}", path.display()))?)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn check_codecs(path: &Path, reader: &SerializedFileReader<File>) -> Result<()> {
    for group in reader.metadata().row_groups() {
        for column in group.columns() {
            codec_supported(column.compression()).map_err(|codec| {
                format!(
                    "{}: column {} uses the unsupported codec {codec}",
                    path.display(),
                    column.column_path().string()
                )
            })?;
        }
    }
    Ok(())
}

/// One document to build: a thread or an article.
enum Item {
    Thread(Thread),
    Article {
        id: String,
        title: Option<String>,
        text: String,
    },
}

impl Item {
    fn source_id(&self) -> &str {
        match self {
            Self::Thread(thread) => &thread.tree_id,
            Self::Article { id, .. } => id,
        }
    }

    /// The key and text that name the document in `excluded.jsonl`.
    fn label(&self) -> (&'static str, String) {
        let (key, text) = match self {
            Self::Thread(thread) => ("first_user_turn", thread.first_user_turn()),
            Self::Article { title, .. } => ("title", title.as_deref().unwrap_or("")),
        };
        (key, text.chars().take(EXCLUDED_TEXT_CHARS).collect())
    }
}

fn article_from_row(path: &Path, row: &Row) -> Result<Item> {
    let mut id = None;
    let mut title = None;
    let mut text = None;
    for (name, field) in row.get_column_iter() {
        let slot = match name.as_str() {
            "id" => &mut id,
            "title" => &mut title,
            "text" => &mut text,
            _ => continue,
        };
        let value = match field {
            Field::Str(s) => Some(s.clone()),
            Field::Null => None,
            Field::Long(v) => Some(v.to_string()),
            Field::Int(v) => Some(v.to_string()),
            _ => {
                return Err(format!(
                    "{}: column {name} holds a value that is not a string, an integer or null",
                    path.display()
                ))
            }
        };
        *slot = Some(value);
    }
    let id = id
        .flatten()
        .ok_or_else(|| format!("{}: a row has no id", path.display()))?;
    let text = text.ok_or_else(|| format!("{}: a row has no text column", path.display()))?;
    Ok(Item::Article {
        id,
        title: title.flatten(),
        text: text.unwrap_or_default(),
    })
}

/// One output split: its writers and running counts.
struct Split {
    name: &'static str,
    tokens_path: PathBuf,
    mask_path: PathBuf,
    writer: CorpusWriter,
    mask: BufWriter<File>,
    documents: usize,
    tokens: usize,
    response_tokens: usize,
    content_bytes: usize,
}

impl Split {
    fn create(out: &Path, name: &'static str, vocab_size: u32) -> Result<Self> {
        let dir = out.join(name);
        fs::create_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let tokens_path = dir.join("tokens.u16");
        let mask_path = dir.join("response_mask.u8");
        let writer = CorpusWriter::create(&tokens_path, vocab_size).map_err(|e| e.to_string())?;
        let mask = BufWriter::new(File::create(&mask_path).map_err(|e| e.to_string())?);
        Ok(Self {
            name,
            tokens_path,
            mask_path,
            writer,
            mask,
            documents: 0,
            tokens: 0,
            response_tokens: 0,
            content_bytes: 0,
        })
    }

    fn write(&mut self, document: &real_text::Document) -> Result<()> {
        self.writer
            .write_tokens(&document.tokens)
            .map_err(|e| e.to_string())?;
        self.mask
            .write_all(&document.mask)
            .map_err(|e| e.to_string())?;
        self.documents += 1;
        self.tokens += document.tokens.len();
        self.response_tokens += document.mask.iter().filter(|&&m| m == 1).count();
        self.content_bytes += document.content_bytes;
        Ok(())
    }

    /// Close both files and check they hold the same number of positions.
    fn finish(self) -> Result<Finished> {
        let written = self.writer.finish().map_err(|e| e.to_string())?;
        let mut mask = self.mask;
        mask.flush().map_err(|e| e.to_string())?;
        drop(mask);
        let mask_bytes = fs::metadata(&self.mask_path)
            .map_err(|e| e.to_string())?
            .len();
        // The trainer reads tokens and mask as parallel arrays.
        if written as usize != self.tokens || mask_bytes as usize != self.tokens {
            return Err(format!(
                "{}: wrote {written} tokens and {mask_bytes} mask bytes, expected {}",
                self.name, self.tokens
            ));
        }
        Ok(Finished {
            name: self.name,
            tokens_bytes: fs::metadata(&self.tokens_path)
                .map_err(|e| e.to_string())?
                .len(),
            tokens_sha256: uor_r4_training::sha256_file(&self.tokens_path)
                .map_err(|e| e.to_string())?,
            mask_sha256: uor_r4_training::sha256_file(&self.mask_path)
                .map_err(|e| e.to_string())?,
            tokens_path: self.tokens_path,
            mask_bytes,
            documents: self.documents,
            tokens: self.tokens,
            response_tokens: self.response_tokens,
            content_bytes: self.content_bytes,
        })
    }
}

struct Finished {
    name: &'static str,
    tokens_path: PathBuf,
    tokens_bytes: u64,
    tokens_sha256: String,
    mask_bytes: u64,
    mask_sha256: String,
    documents: usize,
    tokens: usize,
    response_tokens: usize,
    content_bytes: usize,
}

/// Counts across both splits.
#[derive(Default)]
struct Totals {
    trees_read: usize,
    rows_read: usize,
    not_ready: usize,
    root_dropped: BTreeMap<&'static str, usize>,
    child_nodes_dropped: BTreeMap<&'static str, usize>,
    tree_no_assistant_reply: usize,
    documents_considered: usize,
    excluded: BTreeMap<&'static str, usize>,
    dropped: BTreeMap<&'static str, usize>,
    not_processed_after_max_docs: usize,
}

struct Writers<'a> {
    train: Split,
    heldout: Split,
    excluded: BufWriter<File>,
    totals: Totals,
    heldout_per_mille: u64,
    max_docs: Option<usize>,
    builder: &'a DocumentBuilder<'a>,
}

impl Writers<'_> {
    fn kept(&self) -> usize {
        self.train.documents + self.heldout.documents
    }

    fn done(&self) -> bool {
        self.max_docs.is_some_and(|max| self.kept() >= max)
    }

    /// Build a batch on all cores, then write it in input order. Returns
    /// false once `max_docs` documents are kept.
    fn flush(&mut self, batch: &mut Vec<Item>) -> Result<bool> {
        let builder = self.builder;
        let built: Vec<std::result::Result<Built, real_text::RealTextError>> = batch
            .par_iter()
            .map(|item| match item {
                Item::Thread(thread) => builder.thread(thread),
                Item::Article { title, text, .. } => builder.article(title.as_deref(), text),
            })
            .collect();
        let total = batch.len();
        for (index, (item, built)) in batch.iter().zip(built).enumerate() {
            if self.done() {
                self.totals.not_processed_after_max_docs += total - index;
                break;
            }
            self.totals.documents_considered += 1;
            match built.map_err(|e| format!("{}: {e}", item.source_id()))? {
                Built::Kept(document) => {
                    if real_text::heldout(item.source_id(), self.heldout_per_mille) {
                        self.heldout.write(&document)?;
                    } else {
                        self.train.write(&document)?;
                    }
                }
                Built::Excluded(hit) => {
                    *self.totals.excluded.entry(hit.kind.label()).or_default() += 1;
                    let (key, text) = item.label();
                    let mut line = serde_json::Map::new();
                    line.insert("source_id".into(), json!(item.source_id()));
                    line.insert(key.into(), json!(text));
                    line.insert("rule".into(), json!(hit.kind.label()));
                    line.insert("matched_words".into(), json!(hit.words));
                    line.insert("panel_request_id".into(), json!(hit.request_id));
                    serde_json::to_writer(&mut self.excluded, &Value::Object(line))
                        .map_err(|e| e.to_string())?;
                    self.excluded.write_all(b"\n").map_err(|e| e.to_string())?;
                }
                Built::Dropped(reason) => {
                    *self.totals.dropped.entry(reason.label()).or_default() += 1;
                }
            }
        }
        batch.clear();
        Ok(!self.done())
    }
}

fn every_label<const N: usize>(
    labels: [&'static str; N],
    counts: &BTreeMap<&'static str, usize>,
) -> Value {
    let mut map = serde_json::Map::new();
    for label in labels {
        map.insert(label.into(), json!(counts.get(label).copied().unwrap_or(0)));
    }
    Value::Object(map)
}

const TURN_KEYS: [&str; 4] = ["split", "tokenizer", "name", "out"];

/// `panel-turns`: a frozen panel's user turns, read back from its split by
/// [`real_text::split_user_turns`] and written as the `{id, user_turns}` list
/// `panel=` reads. The id is `NAME:<document index>`; a document with no user
/// turn is skipped and counted. Returns the summary line `run` prints.
fn panel_turns(args: &[String], pin: &str) -> Result<Value> {
    let values = key_values(args, &TURN_KEYS, &TURN_KEYS)?;
    let split = Path::new(values["split"]);
    let out = Path::new(values["out"]);
    let name = values["name"];
    if out.exists() {
        return Err(format!("{}: already exists", out.display()));
    }
    let tokenizer_path = Path::new(values["tokenizer"]);
    let tokenizer_json =
        fs::read(tokenizer_path).map_err(|e| format!("{}: {e}", tokenizer_path.display()))?;
    real_text::check_tokenizer_sha(&tokenizer_json, pin).map_err(|e| e.to_string())?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    // Only its special ids are read; both protocols share them.
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).map_err(|e| e.to_string())?;
    let tokens_path = split.join("tokens.u16");
    let tokens: Vec<u32> = MmapCorpusReader::open(&tokens_path)
        .map_err(|e| format!("{}: {e}", tokens_path.display()))?
        .as_slice()
        .iter()
        .map(|&t| u32::from(t))
        .collect();
    let mask_path = split.join("response_mask.u8");
    let mask = fs::read(&mask_path).map_err(|e| format!("{}: {e}", mask_path.display()))?;
    let documents = real_text::split_user_turns(&tokens, &mask, &tokenizer, &protocol)
        .map_err(|e| format!("{}: {e}", split.display()))?;
    let rows: Vec<Value> = documents
        .iter()
        .enumerate()
        .filter(|(_, turns)| !turns.is_empty())
        .map(|(index, turns)| json!({"id": format!("{name}:{index}"), "user_turns": turns}))
        .collect();
    if rows.is_empty() {
        return Err(format!(
            "{}: no document holds a user turn",
            split.display()
        ));
    }
    // Hashed before `out` is created, so a failure here writes nothing.
    let tokens_sha256 = uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?;
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(out)
        .map_err(|e| format!("{}: {e}", out.display()))?;
    file.write_all(&serde_json::to_vec_pretty(&rows).map_err(|e| e.to_string())?)
        .map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(json!({
        "split": split.display().to_string(),
        "tokens_sha256": tokens_sha256,
        "documents": documents.len(),
        "with_user_turns": rows.len(),
        "without_user_turns": documents.len() - rows.len(),
        "user_turns": documents.iter().map(Vec::len).sum::<usize>(),
    }))
}

fn run(args: &[String]) -> Result<()> {
    run_with_pin(args, real_text::PINNED_TOKENIZER_SHA256)
}

fn run_with_pin(args: &[String], pin: &str) -> Result<()> {
    if args.first().map(String::as_str) == Some("panel-turns") {
        println!("{}", panel_turns(&args[1..], pin)?);
        return Ok(());
    }
    let config = parse_args(args)?;
    for path in config.inputs.iter().chain(&config.panels) {
        if !path.is_file() {
            return Err(format!("{}: not a file", path.display()));
        }
    }
    // The pin is checked before the output root is claimed or anything written.
    let tokenizer_json =
        fs::read(&config.tokenizer).map_err(|e| format!("{}: {e}", config.tokenizer.display()))?;
    real_text::check_tokenizer_sha(&tokenizer_json, pin).map_err(|e| e.to_string())?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or("unreadable tokenizer.json")?;
    // The bytes the pin was checked on, not a second read of the file.
    let tokenizer_sha256 = uor_r4_training::sha256_bytes(&tokenizer_json);
    let tokenizer_ids =
        u32::try_from(tokenizer.vocab_size()).map_err(|_| "the tokenizer has too many ids")?;
    let vocab_size = config.vocab_size.unwrap_or(tokenizer_ids);
    if vocab_size != tokenizer_ids {
        return Err(format!(
            "vocab_size={vocab_size} differs from the tokenizer's {tokenizer_ids} ids"
        ));
    }
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, config.protocol)
        .map_err(|e| e.to_string())?;

    let mut requests: Vec<PanelRequest> = Vec::new();
    let mut panel_entries = Vec::new();
    let mut panel_names = std::collections::BTreeSet::new();
    for path in &config.panels {
        let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let panel =
            real_text::parse_panel(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        if panel.is_empty() {
            return Err(format!("{}: the panel has no requests", path.display()));
        }
        let name = path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into(),
        );
        // A hit names its panel by file name, so two panels may not share one.
        if !panel_names.insert(name.clone()) {
            return Err(format!("two panels share the file name {name}"));
        }
        panel_entries.push(json!({
            "path": path.display().to_string(),
            "sha256": uor_r4_training::sha256_file(path).map_err(|e| e.to_string())?,
            "requests": panel.len(),
            "user_turns": panel.iter().map(|r| r.user_turns.len()).sum::<usize>(),
        }));
        // Request ids are named by their panel file, so ids never collide
        // across panels and a hit names the panel it came from.
        requests.extend(panel.into_iter().map(|r| PanelRequest {
            id: format!("{name}:{}", r.id),
            user_turns: r.user_turns,
        }));
    }
    let exclusion = PanelExclusion::new(&requests);
    let max_tokens = Some(config.max_tokens);
    let builder = DocumentBuilder::new(&tokenizer, &protocol, &exclusion, max_tokens)
        .map_err(|e| e.to_string())?;

    let mut input_entries = Vec::new();
    for path in &config.inputs {
        input_entries.push(json!({
            "path": path.display().to_string(),
            "bytes": fs::metadata(path).map_err(|e| e.to_string())?.len(),
            "sha256": uor_r4_training::sha256_file(path).map_err(|e| e.to_string())?,
        }));
    }

    // A shard this build cannot decode is refused before anything is written.
    if matches!(config.mode, Mode::Wiki) {
        for path in &config.inputs {
            check_codecs(path, &open_shard(path)?)?;
        }
    }

    // Claim the root exclusively before writing anything into it.
    report_output::claim(&config.out).map_err(|e| e.to_string())?;
    let excluded_path = config.out.join("excluded.jsonl");
    let mut writers = Writers {
        train: Split::create(&config.out, "train", vocab_size)?,
        heldout: Split::create(&config.out, "heldout", vocab_size)?,
        excluded: BufWriter::new(File::create(&excluded_path).map_err(|e| e.to_string())?),
        totals: Totals::default(),
        heldout_per_mille: config.heldout_per_mille,
        max_docs: config.max_docs,
        builder: &builder,
    };

    let mut batch: Vec<Item> = Vec::with_capacity(BATCH);
    let stopped_early = match config.mode {
        Mode::Oasst => read_oasst(&config.inputs, &mut writers, &mut batch)?,
        Mode::Wiki => read_wiki(&config.inputs, &mut writers, &mut batch)?,
    };
    writers.flush(&mut batch)?;
    let stopped_at_max_docs = stopped_early || writers.totals.not_processed_after_max_docs > 0;
    writers
        .excluded
        .flush()
        .map_err(|e| format!("{}: {e}", excluded_path.display()))?;

    let Writers {
        train,
        heldout,
        totals,
        ..
    } = writers;
    let finished = [train.finish()?, heldout.finish()?];
    let excluded_total: usize = totals.excluded.values().sum();
    let dropped_total: usize = totals.dropped.values().sum();
    let kept_total: usize = finished.iter().map(|f| f.documents).sum();
    if kept_total + excluded_total + dropped_total != totals.documents_considered {
        return Err(format!(
            "accounting defect: kept {kept_total} + excluded {excluded_total} + dropped \
             {dropped_total} != considered {}",
            totals.documents_considered
        ));
    }

    let node_labels = NodeDrop::ALL.map(NodeDrop::label);
    let mut totals_json = json!({
        "documents_considered": totals.documents_considered,
        "documents_kept": {"train": finished[0].documents, "heldout": finished[1].documents},
        "excluded": every_label([HitKind::Ngram, HitKind::ShortRequest].map(HitKind::label),
            &totals.excluded),
        "dropped": every_label(DocDrop::ALL.map(DocDrop::label), &totals.dropped),
        "not_processed_after_max_docs": totals.not_processed_after_max_docs,
        // True when max_docs was reached and reading stopped there: the counts
        // here then describe the input read up to that point. When the cap
        // falls exactly on a batch's end, that may still be the whole input.
        "stopped_at_max_docs": stopped_at_max_docs,
    });
    let extra = match config.mode {
        Mode::Oasst => json!({
            "trees_read": totals.trees_read,
            "not_ready": totals.not_ready,
            "root_dropped": every_label(node_labels, &totals.root_dropped),
            "child_nodes_dropped": every_label(node_labels, &totals.child_nodes_dropped),
            "no_assistant_reply": totals.tree_no_assistant_reply,
            "threads": totals.documents_considered + totals.not_processed_after_max_docs,
        }),
        Mode::Wiki => json!({ "rows_read": totals.rows_read }),
    };
    if let (Value::Object(map), Value::Object(more)) = (&mut totals_json, extra) {
        map.extend(more);
    }
    let uncovered: Vec<Value> = exclusion
        .uncovered
        .iter()
        .map(|u| json!({"request_id": u.request_id, "text": u.text}))
        .collect();
    let content_convention = match config.mode {
        Mode::Oasst => real_text::OASST_CONTENT_CONVENTION,
        Mode::Wiki => real_text::WIKI_CONTENT_CONVENTION,
    };
    let mut manifests = Vec::new();
    for split in &finished {
        let manifest = json!({
            "schema": "uor-r4-chat-corpus/v1",
            "mask_schema": "uor-r4-response-mask/u8/v1",
            "split": split.name,
            "generator": "prepare-real-text",
            "mode": config.mode.label(),
            "source": config.source,
            "revision": config.revision,
            "licence": config.licence,
            "attribution": config.attribution,
            "inputs": input_entries,
            "tokenizer": {
                "path": config.tokenizer.display().to_string(),
                "sha256": tokenizer_sha256,
                "vocab_size": vocab_size,
                "bos_id": protocol.bos_id,
            },
            "dialogue_protocol": protocol.schema,
            "bos_id": protocol.bos_id,
            "eos_id": protocol.eos_id,
            "panels": panel_entries,
            "exclusion": {
                "requests": exclusion.requests,
                "user_turns": exclusion.turns,
                "ngram_turns": exclusion.ngram_turns,
                "short_turns": exclusion.short_turns,
                "distinct_ngrams": exclusion.ngrams,
                "uncovered_short_turns": uncovered.len(),
                "uncovered": uncovered,
            },
            "template_rule": content_convention,
            "mask_rule": content_convention,
            "content_convention": content_convention,
            "node_filter_rule": real_text::NODE_FILTER_RULE,
            "walk_rule": real_text::WALK_RULE,
            "split_rule": real_text::SPLIT_RULE,
            "heldout_per_mille": config.heldout_per_mille,
            "max_tokens": config.max_tokens,
            "max_docs": config.max_docs,
            "rows_used": split.documents,
            "documents": split.documents,
            "tokens": split.tokens,
            "response_tokens": split.response_tokens,
            "content_bytes": split.content_bytes,
            "bytes_per_token": if split.tokens == 0 {
                Value::Null
            } else {
                json!(split.content_bytes as f64 / split.tokens as f64)
            },
            "tokens_bytes": split.tokens_bytes,
            "mask_bytes": split.mask_bytes,
            "tokens_sha256": split.tokens_sha256,
            "mask_sha256": split.mask_sha256,
            "totals": totals_json,
        });
        let path = split
            .tokens_path
            .parent()
            .ok_or("split without a directory")?
            .join("manifest.json");
        let bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
        manifests.push((path, bytes));
    }
    // Both manifests are written before either is put in place, so a failed
    // write never leaves one split looking complete without the other.
    for (path, bytes) in &manifests {
        let partial = path.with_extension("json.partial");
        fs::write(&partial, bytes).map_err(|e| format!("{}: {e}", partial.display()))?;
    }
    for (path, _) in &manifests {
        let partial = path.with_extension("json.partial");
        fs::rename(&partial, path).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    println!(
        "{}: considered {}, kept train {} ({} tokens) heldout {} ({} tokens), excluded {}, \
         dropped {}",
        config.mode.label(),
        totals.documents_considered,
        finished[0].documents,
        finished[0].tokens,
        finished[1].documents,
        finished[1].tokens,
        excluded_total,
        dropped_total
    );
    Ok(())
}

/// Returns true when `max_docs` was reached and the read stopped there.
fn read_oasst(
    inputs: &[PathBuf],
    writers: &mut Writers<'_>,
    batch: &mut Vec<Item>,
) -> Result<bool> {
    for path in inputs {
        let reader = BufReader::new(File::open(path).map_err(|e| e.to_string())?);
        for (index, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("{}: {e}", path.display()))?;
            let at = || format!("{}:{}", path.display(), index + 1);
            let tree: OasstTree =
                serde_json::from_str(&line).map_err(|e| format!("{}: {e}", at()))?;
            writers.totals.trees_read += 1;
            let outcome = real_text::walk_tree(&tree, &mut writers.totals.child_nodes_dropped)
                .map_err(|e| format!("{}: {e}", at()))?;
            match outcome {
                TreeOutcome::Thread(thread) => batch.push(Item::Thread(thread)),
                TreeOutcome::NotReady => writers.totals.not_ready += 1,
                TreeOutcome::RootDropped(reason) => {
                    *writers
                        .totals
                        .root_dropped
                        .entry(reason.label())
                        .or_default() += 1;
                }
                TreeOutcome::NoAssistantReply => writers.totals.tree_no_assistant_reply += 1,
            }
            if batch.len() == BATCH && !writers.flush(batch)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Returns true when `max_docs` was reached and the read stopped there. The
/// codecs were checked before the output root was claimed.
fn read_wiki(inputs: &[PathBuf], writers: &mut Writers<'_>, batch: &mut Vec<Item>) -> Result<bool> {
    for path in inputs {
        let reader = open_shard(path)?;
        for row in reader
            .get_row_iter(None)
            .map_err(|e| format!("{}: {e}", path.display()))?
        {
            let row = row.map_err(|e| format!("{}: {e}", path.display()))?;
            writers.totals.rows_read += 1;
            batch.push(article_from_row(path, &row)?);
            if batch.len() == BATCH && !writers.flush(batch)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
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
    use parquet::basic::{GzipLevel, ZstdLevel};
    use parquet::data_type::{ByteArray, ByteArrayType};
    use parquet::file::metadata::ParquetMetaDataWriter;
    use parquet::file::properties::WriterProperties;
    use parquet::file::writer::SerializedFileWriter;
    use parquet::schema::parser::parse_message_type;
    use std::sync::Arc;

    const BOS: u16 = 0;
    const EOS: u16 = 1;

    /// The GPT-2 byte alphabet: printable bytes map to themselves, the rest
    /// to code points from 256 up (copied from the `real_text` tests).
    fn byte_alphabet() -> Vec<char> {
        let printable = |b: u32| {
            (0x21..=0x7E).contains(&b) || (0xA1..=0xAC).contains(&b) || (0xAE..=0xFF).contains(&b)
        };
        let mut extra = 0;
        (0..256u32)
            .map(|b| {
                if printable(b) {
                    char::from_u32(b).unwrap()
                } else {
                    extra += 1;
                    char::from_u32(255 + extra).unwrap()
                }
            })
            .collect()
    }

    /// Specials at ids 0-2 and byte `b` at id `b + 3`, no merges.
    fn tokenizer_json() -> String {
        let mut vocab = serde_json::Map::new();
        for (id, s) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*s).to_owned(), json!(id));
        }
        for (byte, ch) in byte_alphabet().iter().enumerate() {
            vocab.insert(ch.to_string(), json!(byte + 3));
        }
        json!({
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
            "added_tokens": [
                {"id": 0, "content": "<|bos|>"},
                {"id": 1, "content": "<|eos|>"},
                {"id": 2, "content": "<|unk|>"}
            ],
            "model": {"type": "BPE", "vocab": vocab, "merges": []}
        })
        .to_string()
    }

    fn bytes_of(text: &str) -> Vec<u16> {
        text.bytes().map(|b| u16::from(b) + 3).collect()
    }

    /// A fresh directory per test and process, removed first.
    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("prepare-real-text-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("tokenizer.json"), tokenizer_json()).unwrap();
        dir
    }

    fn pin(dir: &Path) -> String {
        uor_r4_training::sha256_file(&dir.join("tokenizer.json")).unwrap()
    }

    fn write_panel(dir: &Path, panel: &Value) -> PathBuf {
        let path = dir.join("panel.json");
        fs::write(&path, serde_json::to_vec(panel).unwrap()).unwrap();
        path
    }

    fn args(dir: &Path, mode: &str, input: &Path, panel: &Path, k: u64) -> Vec<String> {
        vec![
            mode.to_owned(),
            format!("out={}", dir.join("out").display()),
            format!("tokenizer={}", dir.join("tokenizer.json").display()),
            format!("input={}", input.display()),
            format!("panel={}", panel.display()),
            "source=test-source".to_owned(),
            "revision=rev-1".to_owned(),
            "licence=Test-Licence".to_owned(),
            "attribution=Test authors".to_owned(),
            format!("heldout_per_mille={k}"),
        ]
    }

    fn read_split(out: &Path, name: &str) -> (Vec<u16>, Vec<u8>, Value) {
        let dir = out.join(name);
        let tokens = MmapCorpusReader::open(dir.join("tokens.u16"))
            .unwrap()
            .as_slice()
            .to_vec();
        let mask = fs::read(dir.join("response_mask.u8")).unwrap();
        let manifest =
            serde_json::from_slice(&fs::read(dir.join("manifest.json")).unwrap()).unwrap();
        (tokens, mask, manifest)
    }

    fn write_wiki_parquet(path: &Path, rows: &[(&str, &str, &str)]) {
        write_wiki_parquet_typed(path, rows, "(STRING)");
    }

    /// `title_type` is the title column's annotation: `(STRING)`, or empty
    /// for plain bytes.
    fn write_wiki_parquet_typed(path: &Path, rows: &[(&str, &str, &str)], title_type: &str) {
        let schema = Arc::new(
            parse_message_type(&format!(
                "message m {{ required binary id (STRING); required binary url (STRING); \
                 required binary title {title_type}; required binary text (STRING); }}"
            ))
            .unwrap(),
        );
        let props = Arc::new(WriterProperties::builder().build());
        let mut writer =
            SerializedFileWriter::new(File::create(path).unwrap(), schema, props).unwrap();
        let mut group = writer.next_row_group().unwrap();
        let columns: [Vec<String>; 4] = [
            rows.iter().map(|r| r.0.to_owned()).collect(),
            rows.iter()
                .map(|r| format!("https://example.org/{}", r.0))
                .collect(),
            rows.iter().map(|r| r.1.to_owned()).collect(),
            rows.iter().map(|r| r.2.to_owned()).collect(),
        ];
        for values in &columns {
            let mut column = group.next_column().unwrap().unwrap();
            let values: Vec<ByteArray> =
                values.iter().map(|v| ByteArray::from(v.as_str())).collect();
            column
                .typed::<ByteArrayType>()
                .write_batch(&values, None, None)
                .unwrap();
            column.close().unwrap();
        }
        group.close().unwrap();
        writer.close().unwrap();
    }

    #[test]
    fn wiki_documents_open_with_bos_then_spaced_content_then_eos() {
        let dir = scratch("wiki");
        // Residues: a 250 (held out at 500), b 946 and c 930 (train).
        assert_eq!(
            ["a", "b", "c"].map(real_text::split_residue),
            [250, 946, 930]
        );
        let input = dir.join("in.parquet");
        let rows = [
            ("a", "Alpha", "Alpha is first."),
            ("b", "Beta", "Beta is\r\nsecond."),
            ("c", "Gamma", "Gamma is third."),
        ];
        write_wiki_parquet(&input, &rows);
        let panel = write_panel(
            &dir,
            &json!([{"id": "p", "user_turns": ["unrelated words"]}]),
        );
        run_with_pin(&args(&dir, "wiki", &input, &panel, 500), &pin(&dir)).unwrap();
        let out = dir.join("out");
        let expect = |rows: &[(&str, &str, &str)]| {
            let mut tokens = Vec::new();
            let mut mask = Vec::new();
            for (_, title, body) in rows {
                let content = real_text::clean_article(Some(title), body);
                let content = real_text::normalize_content(&content);
                let ids = bytes_of(&format!(" {content}"));
                // Protocol 2: one space opens the content (byte 0x20 at id 35).
                assert_eq!(ids[0], 0x20 + 3);
                tokens.push(BOS);
                mask.push(0);
                mask.extend(std::iter::repeat_n(1, ids.len() + 1));
                tokens.extend(ids);
                tokens.push(EOS);
            }
            (tokens, mask)
        };
        let (train, train_mask, train_manifest) = read_split(&out, "train");
        let (heldout, heldout_mask, heldout_manifest) = read_split(&out, "heldout");
        assert_eq!((train, train_mask), expect(&rows[1..]));
        assert_eq!((heldout, heldout_mask), expect(&rows[..1]));
        assert_eq!(train_manifest["mode"], "wiki");
        assert_eq!(train_manifest["totals"]["rows_read"], 3);
        assert_eq!(train_manifest["documents"], 2);
        assert_eq!(heldout_manifest["documents"], 1);
        assert_eq!(
            train_manifest["content_convention"],
            real_text::WIKI_CONTENT_CONVENTION
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_wrong_pin_is_refused_before_anything_is_written() {
        let dir = scratch("pin");
        let input = dir.join("trees.jsonl");
        fs::write(&input, "").unwrap();
        let panel = write_panel(&dir, &json!([{"id": "p", "user_turns": ["any words"]}]));
        let mut args = args(&dir, "oasst", &input, &panel, 0);
        let nested = dir.join("nested");
        args[1] = format!("out={}", nested.join("out").display());
        let wrong = "0".repeat(64);
        let error = run_with_pin(&args, &wrong).unwrap_err();
        assert!(error.contains(&wrong), "{error}");
        assert!(
            !nested.exists(),
            "the pin must be checked before the out root is claimed"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    fn node(id: &str, role: &str, text: &str, rank: Option<i64>, replies: Vec<Value>) -> Value {
        json!({
            "message_id": id, "text": text, "role": role, "lang": "en",
            "review_result": true, "deleted": false, "rank": rank, "synthetic": false,
            "replies": replies,
        })
    }

    fn tree(id: &str, prompt: Value) -> String {
        json!({"message_tree_id": id, "tree_state": "ready_for_export", "prompt": prompt})
            .to_string()
    }

    #[test]
    fn oasst_end_to_end_splits_excludes_counts_and_records_every_field() {
        let dir = scratch("oasst");
        // At 700: t-keep-a 797 goes to train, t-keep-b 608 is held out.
        assert_eq!(real_text::split_residue("t-keep-a"), 797);
        assert_eq!(real_text::split_residue("t-keep-b"), 608);
        let reply = |id: &str, text: &str| vec![node(id, "assistant", text, Some(0), vec![])];
        let mut spanish = node("es-root", "prompter", "hola", None, reply("es-a", "hola"));
        spanish["lang"] = json!("es");
        let lines = [
            tree(
                "t-keep-a",
                node("a-u", "prompter", "Hello there", None, reply("a-a", "Hi")),
            ),
            tree(
                "t-keep-b",
                node("b-u", "prompter", "Good day", None, reply("b-a", "Hey")),
            ),
            tree(
                "t-excl",
                node(
                    "x-u",
                    "prompter",
                    "Please, one two three four five six seven eight now",
                    None,
                    reply("x-a", "Sure"),
                ),
            ),
            tree("t-es", spanish),
        ];
        let input = dir.join("trees.jsonl");
        fs::write(&input, lines.join("\n") + "\n").unwrap();
        let panel = write_panel(
            &dir,
            &json!([{"id": "p1", "user_turns": [
                "one two three four five six seven eight nine", "hi there"
            ]}]),
        );
        run_with_pin(&args(&dir, "oasst", &input, &panel, 700), &pin(&dir)).unwrap();
        let out = dir.join("out");
        let (train, train_mask, m) = read_split(&out, "train");
        let (heldout, heldout_mask, h) = read_split(&out, "heldout");

        // Every field the CPU trainer's manifest check reads, and their sums.
        for (tokens, mask, manifest, split) in [
            (&train, &train_mask, &m, "train"),
            (&heldout, &heldout_mask, &h, "heldout"),
        ] {
            assert_eq!(manifest["schema"], "uor-r4-chat-corpus/v1");
            assert_eq!(manifest["mask_schema"], "uor-r4-response-mask/u8/v1");
            assert_eq!(manifest["split"], split);
            assert_eq!(manifest["documents"], 1);
            assert_eq!(manifest["tokens"], tokens.len());
            assert_eq!(manifest["mask_bytes"], mask.len());
            assert_eq!(
                manifest["response_tokens"],
                mask.iter().filter(|&&v| v == 1).count()
            );
            let tokens_path = out.join(split).join("tokens.u16");
            assert_eq!(
                manifest["tokens_bytes"],
                fs::metadata(&tokens_path).unwrap().len()
            );
            assert_eq!(
                manifest["tokens_sha256"],
                uor_r4_training::sha256_file(&tokens_path).unwrap()
            );
            assert_eq!(
                manifest["mask_sha256"],
                uor_r4_training::sha256_file(&out.join(split).join("response_mask.u8")).unwrap()
            );
            assert_eq!(manifest["tokenizer"]["sha256"], pin(&dir));
            // The test tokenizer's own id count: 3 specials and 256 bytes.
            assert_eq!(manifest["tokenizer"]["vocab_size"], 259);
            assert_eq!(manifest["tokenizer"]["bos_id"], 0);
            assert_eq!(manifest["max_tokens"], 8192);
            assert_eq!(manifest["totals"]["stopped_at_max_docs"], false);
            assert_eq!(tokens[0], BOS);
            // Content bytes over all tokens; one token per byte here, plus
            // markers, so at most 1.
            let content = manifest["content_bytes"].as_f64().unwrap();
            let per_token = manifest["bytes_per_token"].as_f64().unwrap();
            assert!(content > 0.0);
            assert!((per_token - content / tokens.len() as f64).abs() < 1e-12);
            assert!(per_token <= 1.0, "{per_token}");
        }
        // Each split holds its own thread, and no excluded one.
        let text_of = |tokens: &[u16]| -> String {
            let bytes: Vec<u8> = tokens
                .iter()
                .filter(|&&t| t > 2)
                .map(|&t| (t - 3) as u8)
                .collect();
            String::from_utf8(bytes).unwrap()
        };
        assert!(
            text_of(&train).contains("Hello there"),
            "{}",
            text_of(&train)
        );
        assert!(
            text_of(&heldout).contains("Good day"),
            "{}",
            text_of(&heldout)
        );
        assert!(!text_of(&train).contains("Good day") && !text_of(&heldout).contains("Hello"));
        for tokens in [&train, &heldout] {
            assert!(
                !text_of(tokens).contains("one two three"),
                "an excluded thread was written"
            );
        }

        for key in ["source", "revision", "licence", "attribution"] {
            assert!(m[key].as_str().is_some_and(|v| !v.is_empty()), "{key}");
        }
        assert_eq!(m["mode"], "oasst");
        assert_eq!(
            m["inputs"][0]["sha256"],
            uor_r4_training::sha256_file(&input).unwrap()
        );
        assert_eq!(m["panels"][0]["requests"], 1);
        assert_eq!(m["split_rule"], real_text::SPLIT_RULE);
        assert_eq!(m["heldout_per_mille"], 700);
        let totals = &m["totals"];
        assert_eq!(totals["trees_read"], 4);
        assert_eq!(totals["not_ready"], 0);
        assert_eq!(totals["root_dropped"]["lang_not_en"], 1);
        assert_eq!(totals["root_dropped"]["deleted"], 0);
        assert_eq!(totals["threads"], 3);
        assert_eq!(totals["documents_considered"], 3);
        assert_eq!(totals["documents_kept"]["train"], 1);
        assert_eq!(totals["documents_kept"]["heldout"], 1);
        assert_eq!(totals["excluded"]["panel_8gram"], 1);
        assert_eq!(totals["excluded"]["panel_short_request"], 0);
        for reason in DocDrop::ALL {
            assert_eq!(totals["dropped"][reason.label()], 0, "{}", reason.label());
        }
        assert_eq!(m["exclusion"]["uncovered_short_turns"], 1);
        assert_eq!(m["exclusion"]["uncovered"][0]["text"], "hi there");
        assert_eq!(
            m["exclusion"]["uncovered"][0]["request_id"],
            "panel.json:p1"
        );

        let excluded = fs::read_to_string(out.join("excluded.jsonl")).unwrap();
        let lines: Vec<Value> = excluded
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["source_id"], "t-excl");
        assert_eq!(lines[0]["rule"], "panel_8gram");
        assert_eq!(lines[0]["panel_request_id"], "panel.json:p1");
        assert_eq!(
            lines[0]["first_user_turn"],
            "Please, one two three four five six seven eight now"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_or_unknown_arguments_are_refused() {
        let dir = scratch("args");
        let input = dir.join("trees.jsonl");
        fs::write(&input, "").unwrap();
        let panel = write_panel(&dir, &json!([{"id": "p", "user_turns": ["any words"]}]));
        let full = args(&dir, "oasst", &input, &panel, 0);
        assert!(parse_args(&full).is_ok());
        for key in REQUIRED {
            let without: Vec<String> = full
                .iter()
                .filter(|a| !a.starts_with(&format!("{key}=")))
                .cloned()
                .collect();
            let error = parse_args(&without).err().unwrap_or_default();
            assert_eq!(error, format!("missing {key}="), "{key}");
        }
        let refusal = |with: &[String]| parse_args(with).err().unwrap_or_default();
        let mut unknown = full.clone();
        unknown.push("colour=blue".into());
        assert_eq!(refusal(&unknown), "unknown argument colour=");
        let mut twice = full.clone();
        twice.push("source=again".into());
        assert_eq!(refusal(&twice), "source= given twice");
        let mut empty_value = full.clone();
        empty_value[5] = "source=".into();
        assert_eq!(refusal(&empty_value), "source= is empty");
        let mut bad_mode = full.clone();
        bad_mode[0] = "books".into();
        assert!(refusal(&bad_mode).starts_with("unknown mode"));
        assert_eq!(full[9], "heldout_per_mille=0");
        let mut over = full.clone();
        over[9] = "heldout_per_mille=1001".into();
        assert_eq!(refusal(&over), "heldout_per_mille= must be 0..=1000");
        for (extra, expected) in [
            ("protocol=3", "protocol= must be 1 or 2"),
            ("vocab_size=many", "vocab_size= must be a number"),
            ("max_tokens=0", "max_tokens= must be positive"),
            ("max_docs=0", "max_docs= must be positive"),
        ] {
            let mut with = full.clone();
            with.push(extra.into());
            assert_eq!(refusal(&with), expected, "{extra}");
        }
        // Two panels with one file name would make hits ambiguous.
        let other = dir.join("other");
        fs::create_dir_all(&other).unwrap();
        let second = write_panel(&other, &json!([{"id": "q", "user_turns": ["more words"]}]));
        let mut same_name = full.clone();
        same_name[4] = format!("panel={},{}", panel.display(), second.display());
        let error = run_with_pin(&same_name, &pin(&dir)).unwrap_err();
        assert!(
            error.contains("two panels share the file name panel.json"),
            "{error}"
        );
        assert!(!dir.join("out").exists());
        let mut empty_panel = full;
        let empty = dir.join("empty.json");
        fs::write(&empty, "[]").unwrap();
        empty_panel[4] = format!("panel={}", empty.display());
        let error = run_with_pin(&empty_panel, &pin(&dir)).unwrap_err();
        assert!(error.contains("empty.json"), "{error}");
        assert!(!dir.join("out").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unsupported_codecs_are_refused_by_name() {
        assert!(codec_supported(Compression::UNCOMPRESSED).is_ok());
        assert!(codec_supported(Compression::SNAPPY).is_ok());
        assert!(codec_supported(Compression::ZSTD(ZstdLevel::default())).is_ok());
        let lz4 = codec_supported(Compression::LZ4).unwrap_err();
        assert!(lz4.contains("LZ4"), "{lz4}");
        let gzip = codec_supported(Compression::GZIP(GzipLevel::default())).unwrap_err();
        assert!(gzip.contains("GZIP"), "{gzip}");
    }

    /// Rewrite a parquet file's footer so every column chunk names `codec`,
    /// leaving the pages as written. This build can write only the codecs it
    /// reads, so a shard with an unsupported one is made this way.
    fn mark_codec(path: &Path, codec: Compression) {
        let bytes = fs::read(path).unwrap();
        let tail = bytes.len() - 8;
        let footer = u32::from_le_bytes(bytes[tail..tail + 4].try_into().unwrap()) as usize;
        let metadata = open_shard(path).unwrap().metadata().clone();
        // Without a loaded page index the writer re-encodes only the footer,
        // and every offset in it still points into the pages kept above.
        assert!(metadata.page_index().is_none());
        let mut builder = metadata.into_builder();
        let groups = builder
            .take_row_groups()
            .into_iter()
            .map(|group| {
                let columns = group
                    .columns()
                    .iter()
                    .map(|c| {
                        c.clone()
                            .into_builder()
                            .set_compression(codec)
                            .build()
                            .unwrap()
                    })
                    .collect();
                group
                    .into_builder()
                    .set_column_metadata(columns)
                    .build()
                    .unwrap()
            })
            .collect();
        let metadata = builder.set_row_groups(groups).build();
        let mut out = bytes[..tail - footer].to_vec();
        ParquetMetaDataWriter::new(&mut out, &metadata)
            .finish()
            .unwrap();
        fs::write(path, out).unwrap();
    }

    #[test]
    fn a_shard_with_an_unsupported_codec_is_refused_before_anything_is_written() {
        let dir = scratch("codec");
        let input = dir.join("shard.parquet");
        write_wiki_parquet(&input, &[("a", "Title", "Body text here.")]);
        mark_codec(&input, Compression::GZIP(GzipLevel::default()));
        let panel = write_panel(&dir, &json!([{"id": "p", "user_turns": ["any words"]}]));
        let args = args(&dir, "wiki", &input, &panel, 0);
        let error = run_with_pin(&args, &pin(&dir)).unwrap_err();
        assert!(
            error.contains("column id uses the unsupported codec GZIP"),
            "{error}"
        );
        assert!(
            !dir.join("out").exists(),
            "the codecs must be checked before the out root is claimed"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_vocab_size_unlike_the_tokenizers_is_refused_before_anything_is_written() {
        let dir = scratch("vocab");
        let input = dir.join("trees.jsonl");
        fs::write(&input, "").unwrap();
        let panel = write_panel(&dir, &json!([{"id": "p", "user_turns": ["any words"]}]));
        let mut args = args(&dir, "oasst", &input, &panel, 0);
        args.push("vocab_size=4096".into());
        let error = run_with_pin(&args, &pin(&dir)).unwrap_err();
        assert!(
            error.contains("vocab_size=4096 differs from the tokenizer's 259 ids"),
            "{error}"
        );
        assert!(!dir.join("out").exists());
        // The tokenizer's own count is accepted, and recorded.
        let last = args.len() - 1;
        args[last] = "vocab_size=259".into();
        run_with_pin(&args, &pin(&dir)).unwrap();
        let (tokens, _, manifest) = read_split(&dir.join("out"), "train");
        assert!(tokens.is_empty());
        assert_eq!(manifest["tokenizer"]["vocab_size"], 259);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn max_docs_stops_the_run_and_the_manifest_says_so() {
        let dir = scratch("max-docs");
        let input = dir.join("shard.parquet");
        write_wiki_parquet(
            &input,
            &[
                ("a", "A", "First body."),
                ("b", "B", "Second body."),
                ("c", "C", "Third body."),
            ],
        );
        let panel = write_panel(&dir, &json!([{"id": "p", "user_turns": ["any words"]}]));
        let mut args = args(&dir, "wiki", &input, &panel, 0);
        args.push("max_docs=1".into());
        run_with_pin(&args, &pin(&dir)).unwrap();
        let (_, _, train) = read_split(&dir.join("out"), "train");
        let (_, _, heldout) = read_split(&dir.join("out"), "heldout");
        assert_eq!(train["documents"], 1);
        assert_eq!(heldout["documents"], 0);
        let totals = &train["totals"];
        assert_eq!(totals["documents_considered"], 1);
        assert_eq!(totals["not_processed_after_max_docs"], 2);
        assert_eq!(totals["rows_read"], 3);
        assert_eq!(totals["stopped_at_max_docs"], true);
        assert_eq!(heldout["totals"], train["totals"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_column_holding_an_unexpected_type_is_refused() {
        let dir = scratch("column-type");
        let input = dir.join("shard.parquet");
        // A title stored as plain bytes, not a string.
        write_wiki_parquet_typed(&input, &[("a", "A", "Body.")], "");
        let panel = write_panel(&dir, &json!([{"id": "p", "user_turns": ["any words"]}]));
        let error = run_with_pin(&args(&dir, "wiki", &input, &panel, 0), &pin(&dir)).unwrap_err();
        assert!(
            error.contains("column title holds a value that is not a string"),
            "{error}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn panel_turns_reads_back_the_user_turns_a_build_wrote() {
        let dir = scratch("turns");
        let answer = |id: &str, text: &str, replies| node(id, "assistant", text, Some(0), replies);
        let second = node(
            "b-u2",
            "prompter",
            "And a second question",
            None,
            vec![answer("b-a2", "Second answer", vec![])],
        );
        let lines = [
            tree(
                "t-one",
                node(
                    "a-u",
                    "prompter",
                    "Hello there",
                    None,
                    vec![answer("a-a", "Hi", vec![])],
                ),
            ),
            tree(
                "t-two",
                node(
                    "b-u",
                    "prompter",
                    "First question",
                    None,
                    vec![answer("b-a", "First answer", vec![second])],
                ),
            ),
        ];
        let input = dir.join("trees.jsonl");
        fs::write(&input, lines.join("\n") + "\n").unwrap();
        let panel = write_panel(
            &dir,
            &json!([{"id": "p", "user_turns": ["no words match"]}]),
        );
        run_with_pin(&args(&dir, "oasst", &input, &panel, 0), &pin(&dir)).unwrap();
        let turn_args = |out: &Path| {
            vec![
                "panel-turns".to_owned(),
                format!("split={}", dir.join("out").join("train").display()),
                format!("tokenizer={}", dir.join("tokenizer.json").display()),
                "name=t".to_owned(),
                format!("out={}", out.display()),
            ]
        };
        let turns = dir.join("turns.json");
        run_with_pin(&turn_args(&turns), &pin(&dir)).unwrap();
        let bytes = fs::read(&turns).unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&bytes).unwrap(),
            json!([
                {"id": "t:0", "user_turns": ["Hello there"]},
                {"id": "t:1", "user_turns": ["First question", "And a second question"]},
            ])
        );
        // The written list is one `panel=` reads.
        assert_eq!(real_text::parse_panel(&bytes).unwrap().len(), 2);
        let error = run_with_pin(&turn_args(&turns), &pin(&dir)).unwrap_err();
        assert!(error.ends_with("already exists"), "{error}");
        let fresh = dir.join("fresh.json");
        let error = run_with_pin(&turn_args(&fresh), &"0".repeat(64)).unwrap_err();
        assert!(error.contains("is not the pinned"), "{error}");
        assert!(!fresh.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn panel_turns_skips_and_counts_a_document_with_no_user_turn() {
        let dir = scratch("turns-skip");
        // An article first (no prompt run), then two conversations, laid out
        // as protocol 2 writes them: the prompt runs have mask 0.
        let documents: [&[(&str, u8)]; 3] = [
            &[("Just an article.", 1)],
            &[("User: Hello there\nAssistant:", 0), (" Hi", 1)],
            &[("User: First\nUser: Second\nAssistant:", 0), (" Both", 1)],
        ];
        let (mut tokens, mut mask) = (Vec::new(), Vec::new());
        for document in documents {
            tokens.push(BOS);
            mask.push(0);
            for &(text, m) in document {
                let ids = bytes_of(text);
                mask.resize(mask.len() + ids.len(), m);
                tokens.extend(ids);
            }
            tokens.push(EOS);
            mask.push(1);
        }
        let split = dir.join("split");
        fs::create_dir(&split).unwrap();
        let mut writer = CorpusWriter::create(split.join("tokens.u16"), 3 + 256).unwrap();
        writer.write_tokens(&tokens).unwrap();
        writer.finish().unwrap();
        fs::write(split.join("response_mask.u8"), &mask).unwrap();
        let turns = dir.join("turns.json");
        let turn_args = [
            "panel-turns".to_owned(),
            format!("split={}", split.display()),
            format!("tokenizer={}", dir.join("tokenizer.json").display()),
            "name=t".to_owned(),
            format!("out={}", turns.display()),
        ];
        let summary = panel_turns(&turn_args[1..], &pin(&dir)).unwrap();
        // The ids keep each document's index in the split, skipped ones included.
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(&turns).unwrap()).unwrap(),
            json!([
                {"id": "t:1", "user_turns": ["Hello there"]},
                {"id": "t:2", "user_turns": ["First", "Second"]},
            ])
        );
        let sha = uor_r4_training::sha256_file(&split.join("tokens.u16")).unwrap();
        assert_eq!(
            summary,
            json!({
                "split": split.display().to_string(),
                "tokens_sha256": sha,
                "documents": 3,
                "with_user_turns": 2,
                "without_user_turns": 1,
                "user_turns": 3,
            })
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
